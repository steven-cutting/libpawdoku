//! Generation: how a setter that is a program finds givens. Today the module holds the
//! basic generator, a published method taken as it stands.
//!
//! This module is the basic way of `docs/specs/generation.allium` in Rust. It stands on
//! [`sudoku`](crate::sudoku) and [`solver`](crate::solver), and takes its draws through
//! [`random`](crate::random). The method is that of "Sudoku Puzzles Generating: from
//! Easy to Evil", the paper the specification cites, which calls it digging holes.
//!
//! The entry is [`generate`]. It is given a [`Tier`] and a stream of draws, and gives
//! the proof a puzzle is set from, or a [`GenerateError`]. Two more are built on it for
//! a caller across a binding: [`generate_from_seed`] and [`generate_from_stream`] give a
//! [`GenerationResult`], what was made and what replays it, without the solution, or a
//! [`GenerationRefused`].
//!
//! # How a puzzle is made
//!
//! 1. **The solution grid.** Eleven givens that do not conflict are drawn, each a
//!    position among the empty ones and then a digit among those its row, column and
//!    box still allow. They go to the solver, and a solution it finds is the grid. When
//!    it finds two, the grid is the one with the lower digit at the first position,
//!    row by row, where they differ. A seeding that cannot go on, or has no solution,
//!    is a failed grid attempt, and another begins; a hundred failed attempts are a
//!    refusal.
//! 2. **The bound.** One draw chooses a count of givens within the tier's range.
//! 3. **Removal.** Each of the 81 positions is visited once, in the tier's order. A
//!    removal is refused when it would leave fewer givens than the bound, or leave the
//!    position's row or column with fewer than the tier's floor. Otherwise the givens
//!    without the position go to the solver: on a verdict of one the given is gone for
//!    good, and on many it stays for good.
//! 4. **The result.** The givens left, with the solver's proof that they are
//!    well-posed. Their one solution is the solution grid.
//!
//! What was reached is what comes back. The floor or the verdict often stops removal
//! with more givens than the tier's range holds, and nothing is drawn again to mend
//! it: [`Tier::most_givens`] says whether a result met its range.
//!
//! # A tier is a construction setting
//!
//! A tier says how a puzzle is made and nothing of how it solves. Nothing here says
//! one tier's puzzles are harder than another's, and no rating is made.
//!
//! # What decides the puzzle
//!
//! The tier and the draws, and nothing else: no clock and no hidden source of chance.
//! The same tier and the same draws give the same puzzle, for [`GENERATION_VERSION`]
//! and [`RANDOM_VERSION`].
//!
//! **Not final.** This is a draft: it may change as implementation continues.
//!
//! ```
//! use pawdoku::generation::{Tier, generate};
//! use pawdoku::random::SeededStream;
//!
//! let mut stream = SeededStream::new(7);
//! let proof = generate(Tier::Five, &mut stream)?;
//!
//! assert!(proof.givens().len() >= Tier::Five.fewest_givens());
//! // Every given is the solution's digit at its position.
//! for given in proof.givens() {
//!     let (row, column) = (given.position().row(), given.position().column());
//!     assert_eq!(proof.solution()[usize::from(row - 1)][usize::from(column - 1)], given.digit());
//! }
//! # Ok::<(), pawdoku::generation::GenerateError>(())
//! ```

mod choice;
mod grid;
mod order;
mod removal;

use crate::random::{RANDOM_VERSION, RandomError, RandomStream, SeededStream};
use crate::solver::{SolveError, solve};
use crate::sudoku::{Given, WellPosed};
use alloc::collections::BTreeSet;
use choice::index_among;
use core::ops::RangeInclusive;
use grid::{GRID_ATTEMPT_LIMIT, draw_grid};
use order::{Order, visits};
use removal::{Limits, remove};

/// The name of this module's way of making a puzzle.
///
/// It equals the specification's default `config.generation_version`. A consumer that
/// records it beside [`RANDOM_VERSION`], a tier and a
/// seed can have the same puzzle again.
///
/// The name changes whenever the same draws would give other givens or a generation
/// would take a different number of draws. That can happen with nothing in this module
/// touched. Eleven givens nearly always have many solutions, and which two the solver
/// reaches follows from the order it takes tied cells and tied digits in, which
/// [`solver`](crate::solver) documents under "The tie-break", and from what its
/// propagation strikes before it guesses. A change to either is a new name here. The
/// tests that pin what seeds give are in `tests/generation.rs`, and whoever changes
/// what they pin changes this constant in the same commit.
///
/// ```
/// assert_eq!(pawdoku::generation::GENERATION_VERSION, "generation-1");
/// ```
pub const GENERATION_VERSION: &str = "generation-1";

/// One of the five construction settings of the basic generator, numbered 1 to 5.
///
/// A tier is a range the bound is drawn within, a floor for every row and column, and
/// an order of removal. The bound is the count of givens removal will not go below; a
/// puzzle may be left with more givens than the range holds. The figures are the source paper's, which calls
/// its settings levels and names them; here a tier claims nothing about how hard its
/// puzzles are.
///
/// | Tier | Range of the bound | Floor | Order of removal |
/// | --- | --- | --- | --- |
/// | [`One`](Self::One) | 51 to 60 | 5 | drawn |
/// | [`Two`](Self::Two) | 36 to 49 | 4 | drawn |
/// | [`Three`](Self::Three) | 32 to 35 | 3 | every other cell along the S path, then the rest |
/// | [`Four`](Self::Four) | 28 to 31 | 2 | the S path: rows alternately from the left and the right |
/// | [`Five`](Self::Five) | 22 to 27 | 0 | row by row |
///
/// ```
/// use pawdoku::generation::Tier;
///
/// let tier = Tier::Two;
/// assert_eq!(tier.number(), 2);
/// assert_eq!((tier.fewest_givens(), tier.most_givens()), (36, 49));
/// assert_eq!(tier.floor(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Tier {
    /// Tier 1: a bound from 51 to 60, a floor of 5, positions visited in a drawn order.
    One,
    /// Tier 2: a bound from 36 to 49, a floor of 4, positions visited in a drawn order.
    Two,
    /// Tier 3: a bound from 32 to 35, a floor of 3, every other cell and then the rest.
    Three,
    /// Tier 4: a bound from 28 to 31, a floor of 2, along the S path.
    Four,
    /// Tier 5: a bound from 22 to 27, no floor, row by row.
    Five,
}

impl Tier {
    /// The tier's number, from 1 to 5, as the specification numbers it.
    ///
    /// ```
    /// use pawdoku::generation::Tier;
    ///
    /// assert_eq!(Tier::One.number(), 1);
    /// assert_eq!(Tier::Five.number(), 5);
    /// ```
    #[must_use]
    pub const fn number(self) -> u8 {
        self.setting().number
    }

    /// The low end of the range the bound is drawn within. A puzzle of this tier never
    /// has fewer givens.
    ///
    /// ```
    /// use pawdoku::generation::Tier;
    ///
    /// assert_eq!(Tier::Five.fewest_givens(), 22);
    /// ```
    #[must_use]
    pub const fn fewest_givens(self) -> usize {
        self.setting().fewest_givens
    }

    /// The high end of the range the bound is drawn within. A puzzle of this tier may
    /// have more givens, when the floor or the solver's verdict stopped removal early;
    /// one that does has not met its range.
    ///
    /// ```
    /// use pawdoku::generation::{Tier, generate};
    /// use pawdoku::random::SeededStream;
    ///
    /// assert_eq!(Tier::Five.most_givens(), 27);
    ///
    /// let tier = Tier::Four;
    /// let proof = generate(tier, &mut SeededStream::new(1))?;
    /// let count = proof.givens().len();
    /// // Never below the range, and within it exactly when at or below its high end.
    /// assert!(count >= tier.fewest_givens());
    /// let met_its_range = count <= tier.most_givens();
    /// assert_eq!(met_its_range, (28..=31).contains(&count));
    /// # Ok::<(), pawdoku::generation::GenerateError>(())
    /// ```
    #[must_use]
    pub const fn most_givens(self) -> usize {
        self.setting().most_givens
    }

    /// The fewest givens any row or column of a puzzle of this tier is left with.
    ///
    /// ```
    /// use pawdoku::generation::Tier;
    ///
    /// assert_eq!(Tier::One.floor(), 5);
    /// assert_eq!(Tier::Five.floor(), 0);
    /// ```
    #[must_use]
    pub const fn floor(self) -> usize {
        self.setting().floor
    }

    /// The tier's figures and its order, as `generation.allium`'s config and its
    /// `order_of` give them. They are the source paper's, but for tier 1's upper end,
    /// and are not a caller's to change.
    const fn setting(self) -> Setting {
        match self {
            Self::One => Setting::new(1, 51..=60, 5, Order::Drawn),
            Self::Two => Setting::new(2, 36..=49, 4, Order::Drawn),
            Self::Three => Setting::new(3, 32..=35, 3, Order::EveryOther),
            Self::Four => Setting::new(4, 28..=31, 2, Order::SPath),
            Self::Five => Setting::new(5, 22..=27, 0, Order::RowByRow),
        }
    }

    /// The order of removal the source paper assigns the tier: `order_of`.
    const fn order(self) -> Order {
        self.setting().order
    }
}

/// What a tier is: its number, the range its bound is drawn within, its floor and its
/// order of removal.
struct Setting {
    number: u8,
    fewest_givens: usize,
    most_givens: usize,
    floor: usize,
    order: Order,
}

impl Setting {
    const fn new(number: u8, range: RangeInclusive<usize>, floor: usize, order: Order) -> Self {
        Self {
            number,
            fewest_givens: *range.start(),
            most_givens: *range.end(),
            floor,
            order,
        }
    }
}

/// Why [`generate`] gave no puzzle.
///
/// The `Display` text of each variant is stable API: bindings build their exceptions
/// from it.
///
/// ```
/// use pawdoku::generation::{GenerateError, Tier, generate};
/// use pawdoku::random::{RandomError, ReplayStream};
///
/// // A script of five draws ends before the solution grid is drawn.
/// let mut stream = ReplayStream::new(vec![0.0; 5])?;
/// let refusal = generate(Tier::Three, &mut stream).unwrap_err();
/// assert_eq!(refusal, GenerateError::Stream(RandomError::Exhausted { index: 5 }));
/// assert_eq!(refusal.to_string(), "the replay stream is exhausted at draw 5");
/// # Ok::<(), RandomError>(())
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GenerateError {
    /// The stream had no draw to give, at whichever draw that happened.
    ///
    /// The library's [`SeededStream`] never does this; the
    /// [`ReplayStream`](crate::random::ReplayStream) fake does when its script ends.
    /// This is the boundary's own refusal, carried as it is. It is not a failed grid
    /// attempt, and the draws given before it stay taken.
    #[error(transparent)]
    Stream(#[from] RandomError),
    /// Every grid attempt failed, so there is no solution grid and nothing to show.
    #[error("no solution grid was found in {attempts} grid attempts")]
    GridAttemptsSpent {
        /// How many grid attempts failed: the specification's `grid_attempt_limit`.
        attempts: u32,
    },
    /// The solver refused the givens removal left.
    ///
    /// No tier and no stream reaches this. Removal begins from a full grid and keeps a
    /// removal only on a verdict of one, so the givens left have one solution; and the
    /// first visit always empties its position, so something is left to play. The
    /// variant is how a defect would arrive: as a refusal and never a panic.
    #[error(transparent)]
    NotProved(#[from] SolveError),
}

/// The bound: one draw chooses among the counts of the tier's range, fewest first.
fn draw_bound(tier: Tier, stream: &mut dyn RandomStream) -> Result<usize, RandomError> {
    let width = tier.most_givens() - tier.fewest_givens() + 1;
    Ok(tier.fewest_givens() + index_among(stream.next_draw()?, width))
}

/// Makes a puzzle of `tier` from the draws `stream` gives: `Generate(tier)`, and every
/// rule it sets off, run to the generation's end.
///
/// The proof holds the givens and their one solution, which is the solution grid the
/// puzzle was dug from. A [`Puzzle`](crate::sudoku::Puzzle) is set from it, and a board
/// is opened on its givens.
///
/// # The draws, and having a puzzle again
///
/// `generate` draws from wherever the stream stands. A stream a caller has already
/// drawn from gives other draws than a fresh one from the same seed, and so, but for
/// a stream that repeats itself, another puzzle. So what a
/// caller records to have a puzzle again is the tier, [`GENERATION_VERSION`],
/// [`RANDOM_VERSION`], the seed, and the index the
/// stream stood at before the call ([`RandomStream::index`]); or the caller simply
/// begins a fresh stream for each puzzle, and the index is zero.
///
/// The draws are taken in this order and no others are taken. For each grid attempt,
/// two for each seeded given, 22 in all, or `2k + 1` for an attempt that stops short
/// after `k` givens. Then one for the bound. Then, for [`Tier::One`] and [`Tier::Two`]
/// alone, one for each of the 81 visits. A first attempt that finds its grid takes 23
/// draws in tiers 3 to 5 and 104 in tiers 1 and 2, whatever removal does. How many a
/// call took is the stream's index after it less its index before, as
/// `after.wrapping_sub(before)`: a [`SeededStream`]'s
/// index wraps past `u64::MAX`, so a plain subtraction can overflow.
///
/// # Errors
///
/// - [`GenerateError::Stream`] when the stream has no draw to give.
/// - [`GenerateError::GridAttemptsSpent`] when a hundred grid attempts have failed.
/// - [`GenerateError::NotProved`] for no tier and no stream; it is there so that a
///   defect is a refusal.
///
/// It does not panic, whatever the stream gives.
///
/// ```
/// use pawdoku::generation::{Tier, generate};
/// use pawdoku::random::{RandomStream, SeededStream};
/// use pawdoku::sudoku::{Puzzle, Status};
///
/// let mut stream = SeededStream::new(2024);
/// let first = generate(Tier::Four, &mut stream)?;
/// assert!(first.givens().len() >= Tier::Four.fewest_givens());
/// assert_eq!(Puzzle::set(first.clone()).status(), Status::Unsolved);
///
/// // The stream has moved on, so a second call takes other draws: here, another puzzle.
/// let began_at = stream.index();
/// assert_eq!(began_at, 23);
/// let second = generate(Tier::Four, &mut stream)?;
/// assert_ne!(first, second);
///
/// // The same seed, from index zero, gives the first puzzle again.
/// assert_eq!(generate(Tier::Four, &mut SeededStream::new(2024))?, first);
/// # Ok::<(), pawdoku::generation::GenerateError>(())
/// ```
pub fn generate(tier: Tier, stream: &mut dyn RandomStream) -> Result<WellPosed, GenerateError> {
    let spent = GenerateError::GridAttemptsSpent {
        attempts: GRID_ATTEMPT_LIMIT,
    };
    let solution = draw_grid(stream)?.ok_or(spent)?;
    let limits = Limits {
        bound: draw_bound(tier, stream)?,
        floor: tier.floor(),
    };
    let givens = remove(&solution, visits(tier.order(), stream), limits)?;
    Ok(solve(givens)?)
}

/// `generation.allium`'s `GenerationResult`, finished: what a generation made and what
/// replays it, and never the solution.
///
/// It holds the tier asked for, the seed when there is one, the stream's index at the
/// start, how many draws the generation took, [`GENERATION_VERSION`] and
/// [`RANDOM_VERSION`], and the givens. The same tier, a stream giving the same draws
/// from `start_index`, and the same two versions give the same givens, so a seed and a
/// start index stand for the puzzle. The solution grid is not in it, though
/// [`generate`]'s proof holds one: a result is made to be handed to a player's side.
///
/// Under the `serde` feature it serialises with its fields in that order. It does not
/// deserialise: a result is made here, never read back.
///
/// ```
/// use pawdoku::generation::{GENERATION_VERSION, Tier, generate, generate_from_seed};
/// use pawdoku::random::SeededStream;
///
/// let result = generate_from_seed(Tier::Three, 2024)?;
/// assert_eq!((result.seed(), result.start_index()), (Some(2024), 0));
/// assert_eq!(result.generation_version(), GENERATION_VERSION);
///
/// // The seed replays it: generate gives the same givens from the same draws.
/// let proof = generate(Tier::Three, &mut SeededStream::new(2024))?;
/// assert_eq!(result.givens(), proof.givens());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct GenerationResult {
    // Derived: the field names are the wire names, in the order S12 settled.
    tier: Tier,
    seed: Option<u64>,
    start_index: u64,
    draws_taken: u64,
    generation_version: &'static str,
    random_version: &'static str,
    givens: BTreeSet<Given>,
}

impl GenerationResult {
    /// The tier the puzzle was made in, as it was asked for.
    #[must_use]
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// The seed the stream began from, for [`generate_from_seed`]; nothing for
    /// [`generate_from_stream`], since a stream does not know its seed.
    #[must_use]
    pub const fn seed(&self) -> Option<u64> {
        self.seed
    }

    /// The stream's index before the first draw: zero from a seed.
    #[must_use]
    pub const fn start_index(&self) -> u64 {
        self.start_index
    }

    /// How many draws the generation took: the stream's index after it,
    /// `wrapping_sub` the index before.
    #[must_use]
    pub const fn draws_taken(&self) -> u64 {
        self.draws_taken
    }

    /// [`GENERATION_VERSION`], under which the givens follow from the draws.
    #[must_use]
    pub const fn generation_version(&self) -> &'static str {
        self.generation_version
    }

    /// [`RANDOM_VERSION`], under which a seed gives its draws.
    #[must_use]
    pub const fn random_version(&self) -> &'static str {
        self.random_version
    }

    /// The givens the generation left, which have exactly one solution.
    #[must_use]
    pub const fn givens(&self) -> &BTreeSet<Given> {
        &self.givens
    }
}

/// Why [`generate_from_seed`] or [`generate_from_stream`] made no puzzle: the
/// [`GenerateError`] [`generate`] refused with, with the tier asked for and how many
/// draws were taken before the refusal, which `GenerationResult` exposes whether or not
/// it finished.
///
/// Its `Display` text is the [`GenerateError`]'s, unchanged. The tier and the count
/// are read through [`tier`](Self::tier) and [`draws_taken`](Self::draws_taken), so no
/// binding has to match the enum to read them.
///
/// ```
/// use pawdoku::generation::{Tier, generate_from_stream};
/// use pawdoku::random::ReplayStream;
///
/// // Five draws run out while the solution grid is still being drawn.
/// let mut stream = ReplayStream::new(vec![0.0; 5])?;
/// let refused = generate_from_stream(Tier::Two, &mut stream).unwrap_err();
/// assert_eq!((refused.tier(), refused.draws_taken()), (Tier::Two, 5));
/// assert_eq!(refused.to_string(), "the replay stream is exhausted at draw 5");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GenerationRefused {
    /// [`generate`] refused. The text is the refusal's own.
    #[error("{error}")]
    Refused {
        /// The tier asked for.
        tier: Tier,
        /// How many draws were taken before the refusal, counted as
        /// [`GenerationResult::draws_taken`] counts them.
        draws_taken: u64,
        /// What [`generate`] refused with.
        error: GenerateError,
    },
}

impl GenerationRefused {
    /// The tier asked for.
    #[must_use]
    pub const fn tier(&self) -> Tier {
        let Self::Refused { tier, .. } = self;
        *tier
    }

    /// How many draws were taken before the refusal.
    #[must_use]
    pub const fn draws_taken(&self) -> u64 {
        let Self::Refused { draws_taken, .. } = self;
        *draws_taken
    }
}

/// [`generate`] from `seed`, on a [`SeededStream`] begun at index zero: the stateless
/// way, which a binding offers to a caller that holds no stream.
///
/// # Errors
///
/// A [`GenerationRefused`] carrying what [`generate`] refused with. No seed is known
/// to be refused.
///
/// ```
/// use pawdoku::generation::{Tier, generate_from_seed};
///
/// let result = generate_from_seed(Tier::Five, 7)?;
/// assert!(result.givens().len() >= Tier::Five.fewest_givens());
/// assert_eq!(generate_from_seed(Tier::Five, 7)?, result);
/// # Ok::<(), pawdoku::generation::GenerationRefused>(())
/// ```
pub fn generate_from_seed(tier: Tier, seed: u64) -> Result<GenerationResult, GenerationRefused> {
    generate_recorded(tier, Some(seed), &mut SeededStream::new(seed))
}

/// [`generate`] from `stream`, wherever it stands: the way a caller that holds a
/// stream takes, reading the stream's index before and after. The seed is not known,
/// so the result names none.
///
/// # Errors
///
/// A [`GenerationRefused`] carrying what [`generate`] refused with, and the draws taken
/// before it.
///
/// ```
/// use pawdoku::generation::{Tier, generate_from_stream};
/// use pawdoku::random::{RandomStream, SeededStream};
///
/// let mut stream = SeededStream::new(2024);
/// let first = generate_from_stream(Tier::Four, &mut stream)?;
/// let second = generate_from_stream(Tier::Four, &mut stream)?;
/// assert_eq!(second.start_index(), first.draws_taken());
/// assert_eq!(second.seed(), None);
/// assert_eq!(stream.index(), first.draws_taken() + second.draws_taken());
/// # Ok::<(), pawdoku::generation::GenerationRefused>(())
/// ```
pub fn generate_from_stream(
    tier: Tier,
    stream: &mut dyn RandomStream,
) -> Result<GenerationResult, GenerationRefused> {
    generate_recorded(tier, None, stream)
}

/// The one way both entries take: [`generate`] on `stream`, with the index read before
/// and after, on a refusal as on a result.
fn generate_recorded(
    tier: Tier,
    seed: Option<u64>,
    stream: &mut dyn RandomStream,
) -> Result<GenerationResult, GenerationRefused> {
    let start_index = stream.index();
    let generated = generate(tier, stream);
    let draws_taken = stream.index().wrapping_sub(start_index);
    match generated {
        Ok(proof) => Ok(GenerationResult {
            tier,
            seed,
            start_index,
            draws_taken,
            generation_version: GENERATION_VERSION,
            random_version: RANDOM_VERSION,
            givens: proof.givens().clone(),
        }),
        Err(error) => Err(GenerationRefused::Refused {
            tier,
            draws_taken,
            error,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::order::Order;
    use super::{
        GENERATION_VERSION, GenerateError, GenerationRefused, Tier, draw_bound, generate,
        generate_from_seed, generate_from_stream,
    };
    use crate::random::{RANDOM_VERSION, RandomError, RandomStream, ReplayStream, SeededStream};
    use crate::solver::SolveError;
    use crate::sudoku::WellPosedError;
    use alloc::format;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    const TIERS: [Tier; 5] = [Tier::One, Tier::Two, Tier::Three, Tier::Four, Tier::Five];

    /// The default of `config.generation_version` in `docs/specs/generation.allium`.
    #[test]
    fn the_version_is_the_specifications() {
        assert_eq!(GENERATION_VERSION, "generation-1");
    }

    /// The fifteen figures of `generation.allium`'s config, which are the source
    /// paper's but for tier 1's upper end.
    #[test]
    fn each_tier_says_its_figures() {
        let figures = |tier: Tier| {
            let range = (tier.fewest_givens(), tier.most_givens());
            (tier.number(), range, tier.floor())
        };
        assert_eq!(figures(Tier::One), (1, (51, 60), 5));
        assert_eq!(figures(Tier::Two), (2, (36, 49), 4));
        assert_eq!(figures(Tier::Three), (3, (32, 35), 3));
        assert_eq!(figures(Tier::Four), (4, (28, 31), 2));
        assert_eq!(figures(Tier::Five), (5, (22, 27), 0));
    }

    #[test]
    fn each_tier_has_the_order_the_paper_assigns_it() {
        let orders = [
            Order::Drawn,
            Order::Drawn,
            Order::EveryOther,
            Order::SPath,
            Order::RowByRow,
        ];
        assert_eq!(TIERS.map(Tier::order), orders);
    }

    #[test]
    fn the_bound_is_drawn_within_the_tiers_range_fewest_first() {
        let largest_draw = 1.0 - f64::EPSILON / 2.0;
        for tier in TIERS {
            let middle = usize::midpoint(tier.fewest_givens(), tier.most_givens());
            let mut stream = ReplayStream::new(vec![0.0, largest_draw, 0.5]).unwrap();
            assert_eq!(draw_bound(tier, &mut stream), Ok(tier.fewest_givens()));
            assert_eq!(draw_bound(tier, &mut stream), Ok(tier.most_givens()));
            // A draw of one half among an even count of counts picks the upper middle.
            assert_eq!(draw_bound(tier, &mut stream), Ok(middle + 1));
            assert_eq!(stream.index(), 3);
            assert!(draw_bound(tier, &mut stream).is_err());
        }
    }

    #[test]
    fn refusal_texts_are_stable() {
        let texts = [
            (
                GenerateError::Stream(RandomError::Exhausted { index: 22 }),
                "the replay stream is exhausted at draw 22",
            ),
            (
                GenerateError::GridAttemptsSpent { attempts: 100 },
                "no solution grid was found in 100 grid attempts",
            ),
            (
                GenerateError::NotProved(SolveError::ManySolutions),
                "the givens have more than one solution",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }

    /// No tier and no stream reaches the solver's refusals through `generate`: the
    /// givens removal leaves have one solution and are fewer than 81. The conversion is
    /// what carries them if a defect ever does, so it is tested by itself.
    #[test]
    fn the_refusal_converts_from_the_solvers() {
        for error in [
            SolveError::NoSolution,
            SolveError::ManySolutions,
            SolveError::NotPosed(WellPosedError::NothingLeftToPlay { givens: 81 }),
        ] {
            let refusal = GenerateError::from(error.clone());
            assert_eq!(refusal, GenerateError::NotProved(error.clone()));
            assert_eq!(refusal.to_string(), error.to_string());
        }
    }

    #[test]
    fn the_refusal_converts_from_the_streams() {
        let error = RandomError::Exhausted { index: 7 };
        let refusal = GenerateError::from(error.clone());
        assert_eq!(refusal, GenerateError::Stream(error.clone()));
        assert_eq!(refusal.to_string(), error.to_string());
    }

    /// A draw that picks index `index` among `n`: the middle of its share of `[0, 1)`.
    fn pick(index: u8, n: u8) -> f64 {
        (f64::from(index) + 0.5) / f64::from(n)
    }

    /// The draws of a grid attempt that ends short after nine givens, as
    /// `generation/grid.rs` writes them: nineteen draws.
    fn a_short_seeding() -> Vec<f64> {
        let mut script = vec![0.0; 16];
        script.extend([pick(9, 73), pick(6, 7), 0.0]);
        script
    }

    /// The draws of a full grid attempt with no solution, as `generation/grid.rs` writes
    /// them: twenty-two draws.
    fn a_seeding_with_no_solution() -> Vec<f64> {
        let mut script = vec![0.0; 16];
        script.extend([pick(9, 73), pick(6, 7)]);
        script.extend([pick(1, 72), 0.0, pick(1, 71), 0.0]);
        script
    }

    fn replaying(draws: Vec<f64>) -> ReplayStream {
        ReplayStream::new(draws).unwrap()
    }

    #[test]
    fn from_a_seed_is_generate_with_a_seeded_stream() {
        for (seed, tier) in [(0, Tier::Three), (7, Tier::Five), (u64::MAX, Tier::One)] {
            let mut stream = SeededStream::new(seed);
            let proof = generate(tier, &mut stream).unwrap();
            let result = generate_from_seed(tier, seed).unwrap();
            assert_eq!(result.givens(), proof.givens());
            assert_eq!(result.tier(), tier);
            assert_eq!(result.seed(), Some(seed));
            assert_eq!(result.start_index(), 0);
            assert_eq!(result.draws_taken(), stream.index());
            assert_eq!(result.generation_version(), GENERATION_VERSION);
            assert_eq!(result.random_version(), RANDOM_VERSION);
        }
    }

    #[test]
    fn a_grid_found_at_once_takes_23_draws_or_104_where_the_order_is_drawn() {
        for (tier, draws) in TIERS.into_iter().zip([104, 104, 23, 23, 23]) {
            let result = generate_from_stream(tier, &mut replaying(vec![0.0; draws])).unwrap();
            assert_eq!(result.draws_taken(), u64::try_from(draws).unwrap());
        }
    }

    #[test]
    fn from_a_stream_records_where_it_began_and_only_its_own_draws() {
        let mut stream = replaying(vec![0.0; 7 + 23]);
        for _ in 0..7 {
            stream.next_draw().unwrap();
        }
        let result = generate_from_stream(Tier::Three, &mut stream).unwrap();
        assert_eq!(result.seed(), None);
        assert_eq!((result.start_index(), result.draws_taken()), (7, 23));
        assert_eq!(
            result.givens(),
            generate(Tier::Three, &mut replaying(vec![0.0; 23]))
                .unwrap()
                .givens()
        );

        // Five zeros run out in the grid, and every draw taken is counted.
        let refused = generate_from_stream(Tier::Four, &mut replaying(vec![0.0; 5]));
        let refused = refused.unwrap_err();
        assert_eq!(refused.draws_taken(), 5);
        assert_eq!(
            refused.to_string(),
            "the replay stream is exhausted at draw 5"
        );

        // A stream drawn three times first counts only its own two.
        let mut stream = replaying(vec![0.0; 5]);
        for _ in 0..3 {
            stream.next_draw().unwrap();
        }
        let refused = generate_from_stream(Tier::Four, &mut stream).unwrap_err();
        assert_eq!((refused.draws_taken(), stream.index()), (2, 5));
    }

    #[test]
    fn a_hundred_failed_attempts_are_refused_with_every_draw_counted() {
        let spent = GenerateError::GridAttemptsSpent { attempts: 100 };
        for (seeding, draws) in [
            (a_short_seeding(), 1900),
            (a_seeding_with_no_solution(), 2200),
        ] {
            let hundred: Vec<f64> = (0..100).flat_map(|_| seeding.clone()).collect();
            let refused = generate_from_stream(Tier::Two, &mut replaying(hundred)).unwrap_err();
            assert_eq!(refused.draws_taken(), draws);
            assert_eq!(refused.to_string(), spent.to_string());
            let GenerationRefused::Refused { error, .. } = refused;
            assert_eq!(error, spent);
        }
    }

    #[test]
    fn each_tier_comes_back_as_asked() {
        for tier in TIERS {
            assert_eq!(generate_from_seed(tier, 0).unwrap().tier(), tier);
            let refused = generate_from_stream(tier, &mut replaying(vec![])).unwrap_err();
            assert_eq!((refused.tier(), refused.draws_taken()), (tier, 0));
        }
    }

    /// The result holds the givens and what replays them, and not the solution: no row
    /// of the solution grid is in what it prints.
    #[test]
    fn the_result_holds_no_solution() {
        let mut stream = SeededStream::new(0);
        let proof = generate(Tier::Five, &mut stream).unwrap();
        let printed = format!("{:?}", generate_from_seed(Tier::Five, 0).unwrap());
        let printed: alloc::string::String = printed.split_whitespace().collect();
        for row in proof.solution() {
            assert!(!printed.contains(&format!("{row:?}").replace(' ', "")));
        }
        assert!(!printed.contains("solution"));
    }
}
