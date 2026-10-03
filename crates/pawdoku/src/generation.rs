//! Generation: how a setter that is a program finds givens. Today the module holds the
//! basic generator, a published method taken as it stands.
//!
//! This module is the basic way of `docs/specs/generation.allium` in Rust. It stands on
//! [`sudoku`](crate::sudoku) and [`solver`](crate::solver), and takes its draws through
//! [`random`](crate::random). The method is that of "Sudoku Puzzles Generating: from
//! Easy to Evil", the paper the specification cites, which calls it digging holes.
//!
//! There is one entry, [`generate`]. It is given a [`Tier`] and a stream of draws, and
//! gives the proof a puzzle is set from, or a [`GenerateError`].
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
//! and [`RANDOM_VERSION`](crate::random::RANDOM_VERSION).
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

use crate::random::{RandomError, RandomStream};
use crate::solver::{SolveError, solve};
use crate::sudoku::WellPosed;
use choice::index_among;
use core::ops::RangeInclusive;
use grid::{GRID_ATTEMPT_LIMIT, draw_grid};
use order::{Order, visits};
use removal::{Limits, remove};

/// The name of this module's way of making a puzzle.
///
/// It equals the specification's default `config.generation_version`. A consumer that
/// records it beside [`RANDOM_VERSION`](crate::random::RANDOM_VERSION), a tier and a
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
/// A tier is a range the count of givens is bounded within, a floor for every row and
/// column, and an order of removal. The figures are the source paper's, which calls
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
    /// let met_its_range = proof.givens().len() <= tier.most_givens();
    /// assert!(met_its_range || proof.givens().len() > 31);
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
    /// The library's [`SeededStream`](crate::random::SeededStream) never does this; the
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
/// drawn from gives another puzzle than a fresh one from the same seed. So what a
/// caller records to have a puzzle again is the tier, [`GENERATION_VERSION`],
/// [`RANDOM_VERSION`](crate::random::RANDOM_VERSION), the seed, and the index the
/// stream stood at before the call ([`RandomStream::index`]); or the caller simply
/// begins a fresh stream for each puzzle, and the index is zero.
///
/// The draws are taken in this order and no others are taken. For each grid attempt,
/// two for each seeded given, 22 in all, or `2k + 1` for an attempt that stops short
/// after `k` givens. Then one for the bound. Then, for [`Tier::One`] and [`Tier::Two`]
/// alone, one for each of the 81 visits. A first attempt that finds its grid takes 23
/// draws in tiers 3 to 5 and 104 in tiers 1 and 2, whatever removal does.
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
/// // The stream has moved on, so a second call gives another puzzle.
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

#[cfg(test)]
mod tests {
    use super::order::Order;
    use super::{GENERATION_VERSION, GenerateError, Tier, draw_bound};
    use crate::random::{RandomError, RandomStream, ReplayStream};
    use crate::solver::SolveError;
    use crate::sudoku::WellPosedError;
    use alloc::string::ToString;
    use alloc::vec;

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
}
