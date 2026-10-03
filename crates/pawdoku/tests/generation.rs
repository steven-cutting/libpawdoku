//! The basic generator from outside the crate: `generate` through the public API, with
//! its draws scripted through the fake, and the two tests that pin what seeds give.
//!
//! The clauses of `generation.allium` that a result cannot show, each order, each end
//! of a visit and each grid attempt, are proved by the unit tests under
//! `src/generation/`. Here are the guarantees of the `Generating` surface.

#[cfg(test)]
mod tests {
    use pawdoku::board::Board;
    use pawdoku::generation::{GENERATION_VERSION, GenerateError, Tier, generate};
    use pawdoku::random::{RANDOM_VERSION, RandomError, RandomStream, ReplayStream, SeededStream};
    use pawdoku::solver::{Verdict, search};
    use pawdoku::sudoku::{Position, Status, WellPosed};
    use proptest::prelude::*;

    const TIERS: [Tier; 5] = [Tier::One, Tier::Two, Tier::Three, Tier::Four, Tier::Five];

    /// The most draws a generation can take: a hundred grid attempts of 22 draws, one for
    /// the bound and 81 for a drawn order. A script this long never runs out.
    const MOST_DRAWS: usize = 100 * 22 + 1 + 81;

    /// The draws a first grid attempt takes when it finds its grid, and the bound's.
    const GRID_AND_BOUND: u64 = 22 + 1;

    fn replaying(script: Vec<f64>) -> ReplayStream {
        ReplayStream::new(script).unwrap()
    }

    /// A script that is not all one draw, made without a generator: the fractional parts
    /// of the multiples of the golden ratio's conjugate.
    fn a_varied_script(len: usize) -> Vec<f64> {
        let step = |at: &mut f64, ()| {
            *at = (*at + 0.618_033_988_749_895).fract();
            Some(*at)
        };
        core::iter::repeat_n((), len).scan(0.0, step).collect()
    }

    /// The draws of a seeding that ends short after nine givens. Zeros put 1 to 8 along
    /// row 1; the tenth of the 73 empty positions is (2,9), and the last of its seven
    /// digits is 9; the first empty position is then (1,9), which has no digit left.
    fn a_short_seeding() -> Vec<f64> {
        let mut script = vec![0.0; 16];
        script.extend([9.5 / 73.0, 6.5 / 7.0, 0.0]);
        script
    }

    /// Every position, a row at a time.
    fn positions() -> impl Iterator<Item = Position> {
        (1..=9).flat_map(|row| (1..=9).map(move |column| Position::new(row, column)))
    }

    /// The solution's digit at a position.
    fn digit_at(proof: &WellPosed, position: Position) -> u8 {
        proof.solution()[usize::from(position.row() - 1)][usize::from(position.column() - 1)]
    }

    /// The givens as 81 cells, a row after a row, with a dot for an empty cell.
    fn picture(proof: &WellPosed) -> String {
        let is_given = |position: Position| {
            let mut givens = proof.givens().iter();
            givens.any(|given| given.position() == position)
        };
        let cell = |position: Position| {
            if is_given(position) {
                char::from(b'0' + digit_at(proof, position))
            } else {
                '.'
            }
        };
        positions().map(cell).collect()
    }

    /// How many givens a row holds, and how many a column.
    fn in_line(proof: &WellPosed, line: u8) -> (usize, usize) {
        let givens = proof.givens();
        let in_row = givens.iter().filter(|given| given.position().row() == line);
        let in_column = givens
            .iter()
            .filter(|given| given.position().column() == line);
        (in_row.count(), in_column.count())
    }

    /// What the `Generating` surface guarantees of a finished generation, as far as its
    /// result shows: `OneSolutionAndItIsTheGrid` and `TheRestrictionsHold`.
    fn assert_the_guarantees(tier: Tier, proof: &WellPosed) {
        // `GivensSitInTheSolutionGrid`.
        for given in proof.givens() {
            assert_eq!(digit_at(proof, given.position()), given.digit());
        }
        // `GivensLeftAreWellPosed`, and the one solution is the grid.
        let found = search(proof.givens().iter().copied());
        assert_eq!(found.verdict(), Verdict::OneSolution);
        assert_eq!(found.solutions(), [*proof.solution()]);
        // `GivensStayAtOrAboveTheBound` with `TheBoundIsInTheTiersRange`; and the first
        // visit always empties its position.
        assert!(proof.givens().len() >= tier.fewest_givens());
        assert!(proof.givens().len() <= 80);
        // `RowsAndColumnsKeepTheFloor`.
        for line in 1..=9 {
            let (in_row, in_column) = in_line(proof, line);
            assert!(in_row >= tier.floor() && in_column >= tier.floor());
        }
    }

    /// The bound a generation drew, worked out again from its script: the draw after the
    /// grid's, which is the last but the 81 of a drawn order, chooses among the counts of
    /// the tier's range, fewest first.
    fn the_bound_drawn(tier: Tier, script: &[f64], draws_taken: u64) -> usize {
        let for_visits = if tier.number() <= 2 { 81 } else { 0 };
        let at = usize::try_from(draws_taken).unwrap() - for_visits - 1;
        let width = u8::try_from(tier.most_givens() - tier.fewest_givens() + 1).unwrap();
        let product = script[at] * f64::from(width);
        let index = (0..width).rev().find(|&index| f64::from(index) <= product);
        tier.fewest_givens() + usize::from(index.unwrap())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(40))]

        #[test]
        fn every_tier_gives_a_puzzle_that_keeps_the_guarantees(
            tier in prop::sample::select(TIERS.to_vec()),
            script in prop::collection::vec(0.0..1.0_f64, MOST_DRAWS),
        ) {
            let mut stream = replaying(script.clone());
            let proof = generate(tier, &mut stream).unwrap();
            assert_the_guarantees(tier, &proof);
            prop_assert!(proof.givens().len() >= the_bound_drawn(tier, &script, stream.index()));
        }

        #[test]
        fn the_same_draws_give_the_same_givens(
            tier in prop::sample::select(TIERS.to_vec()),
            script in prop::collection::vec(0.0..1.0_f64, MOST_DRAWS),
        ) {
            let (mut first, mut second) = (replaying(script.clone()), replaying(script));
            prop_assert_eq!(generate(tier, &mut first), generate(tier, &mut second));
            prop_assert_eq!(first.index(), second.index());
        }
    }

    /// `DrawsInOrder`: 22 for a grid attempt that seeds all eleven, one for the bound and,
    /// under the drawn order alone, one for each visit. Zeros seed a grid at once.
    #[test]
    fn the_draws_taken_follow_from_the_grid_attempts_and_the_tier() {
        for tier in TIERS {
            let mut stream = replaying(vec![0.0; MOST_DRAWS]);
            let proof = generate(tier, &mut stream).unwrap();
            let for_visits = if tier.number() <= 2 { 81 } else { 0 };
            assert_eq!(stream.index(), GRID_AND_BOUND + for_visits);
            assert_the_guarantees(tier, &proof);
        }
        // A failed attempt's draws stay taken: nineteen for the short seeding.
        let mut script = a_short_seeding();
        script.extend([0.0; 23]);
        let mut stream = replaying(script);
        assert!(generate(Tier::Five, &mut stream).is_ok());
        assert_eq!(stream.index(), 19 + GRID_AND_BOUND);
    }

    /// `SameDrawsSameGivens`, for a stream that has already been drawn from: replay is
    /// stated over the draws, so what gives the second puzzle again is a stream brought to
    /// the index the second call began at.
    #[test]
    fn two_calls_on_one_stream_give_two_puzzles_and_the_index_replays_the_second() {
        let script = a_varied_script(2 * MOST_DRAWS);
        let mut stream = replaying(script.clone());
        let first = generate(Tier::Four, &mut stream).unwrap();
        let began_at = stream.index();
        let second = generate(Tier::Four, &mut stream).unwrap();
        assert_ne!(first, second);

        let mut brought_on = replaying(script.clone());
        for _ in 0..began_at {
            brought_on.next_draw().unwrap();
        }
        assert_eq!(brought_on.index(), began_at);
        assert_eq!(generate(Tier::Four, &mut brought_on).unwrap(), second);
        assert_eq!(brought_on.index(), stream.index());

        // A fresh stream of the same draws gives the first again, and not the second.
        assert_eq!(generate(Tier::Four, &mut replaying(script)).unwrap(), first);
    }

    /// `AStreamThatRunsOut`: the boundary's own refusal, at whichever draw it happens.
    #[test]
    fn a_stream_that_runs_out_says_so_at_whichever_draw() {
        let ran_out = |tier: Tier, draws: usize| {
            let mut stream = replaying(vec![0.0; draws]);
            let refusal = generate(tier, &mut stream).unwrap_err();
            // The draws given before it stay taken, and the one not given is not.
            assert_eq!(stream.index(), u64::try_from(draws).unwrap());
            refusal
        };
        let exhausted = |index: u64| GenerateError::Stream(RandomError::Exhausted { index });
        // In the grid.
        assert_eq!(ran_out(Tier::Five, 0), exhausted(0));
        assert_eq!(ran_out(Tier::Five, 10), exhausted(10));
        // At the bound.
        assert_eq!(ran_out(Tier::Five, 22), exhausted(22));
        // In a drawn order: at its first visit, and at its last.
        assert_eq!(ran_out(Tier::One, 23), exhausted(23));
        assert_eq!(ran_out(Tier::Two, 103), exhausted(103));
        // A fixed order draws nothing, so 23 draws are enough for it.
        assert!(generate(Tier::Three, &mut replaying(vec![0.0; 23])).is_ok());
    }

    /// `FollowGridAttempt`'s refusal, `EndsAreEarned` and `GridAttemptsStayWithinTheLimit`:
    /// a hundred failed grid attempts, and nothing to show.
    #[test]
    fn spent_attempts_are_refused_with_the_count() {
        let mut script: Vec<f64> = (0..100).flat_map(|_| a_short_seeding()).collect();
        script.extend([0.0; 200]);
        let mut stream = replaying(script);
        let refusal = generate(Tier::One, &mut stream).unwrap_err();
        assert_eq!(refusal, GenerateError::GridAttemptsSpent { attempts: 100 });
        // A refused generation takes the draws of its attempts and no more: no bound.
        assert_eq!(stream.index(), 100 * 19);
    }

    #[test]
    fn each_refusal_reads_as_its_text() {
        let mut script: Vec<f64> = (0..100).flat_map(|_| a_short_seeding()).collect();
        let spent = generate(Tier::Five, &mut replaying(script.clone())).unwrap_err();
        assert_eq!(
            spent.to_string(),
            "no solution grid was found in 100 grid attempts"
        );
        script.truncate(100 * 19 - 1);
        let ran_out = generate(Tier::Five, &mut replaying(script)).unwrap_err();
        assert_eq!(
            ran_out.to_string(),
            "the replay stream is exhausted at draw 1899"
        );
    }

    #[test]
    fn a_generated_puzzle_opens_as_a_board_and_is_played_to_its_end() {
        let mut stream = replaying(a_varied_script(MOST_DRAWS));
        let proof = generate(Tier::Three, &mut stream).unwrap();
        let mut board = Board::open(proof.givens().iter().copied()).unwrap();
        assert_eq!(board.status(), Status::Unsolved);
        for position in positions() {
            let is_given = proof
                .givens()
                .iter()
                .any(|given| given.position() == position);
            if !is_given {
                board.place(position, digit_at(&proof, position)).unwrap();
            }
        }
        assert_eq!(board.status(), Status::Solved);
    }

    /// A stream of a caller's own that breaks the boundary's word: its draws go round a
    /// list that is mostly outside `[0, 1)`.
    #[derive(Debug)]
    struct Cycling {
        draws: Vec<f64>,
        index: u64,
    }

    impl RandomStream for Cycling {
        fn next_draw(&mut self) -> Result<f64, RandomError> {
            let at = usize::try_from(self.index).unwrap() % self.draws.len();
            self.index += 1;
            Ok(self.draws[at])
        }

        fn index(&self) -> u64 {
            self.index
        }
    }

    /// `ReplayStream` refuses a draw outside `[0, 1)`, so this is the one test whose draws
    /// do not come through the fake: it needs a stream the fake will not be.
    #[test]
    fn a_stream_that_breaks_its_word_cannot_make_generate_panic() {
        let lists = [
            vec![1.0],
            vec![f64::NAN],
            vec![-3.0],
            vec![f64::INFINITY, 0.3],
            vec![1.0, 2.5, -1.0, f64::NAN, f64::INFINITY, 0.3, 0.9],
        ];
        for (tier, draws) in TIERS.into_iter().zip(lists) {
            let mut stream = Cycling { draws, index: 0 };
            match generate(tier, &mut stream) {
                Ok(proof) => assert_the_guarantees(tier, &proof),
                Err(refusal) => {
                    assert_eq!(refusal, GenerateError::GridAttemptsSpent { attempts: 100 });
                }
            }
        }
    }

    /// The puzzle a fresh `SeededStream` of a seed gives in a tier.
    fn from_seed(tier: Tier, seed: u64) -> (WellPosed, u64) {
        let mut stream = SeededStream::new(seed);
        let proof = generate(tier, &mut stream).unwrap();
        (proof, stream.index())
    }

    /// What seed zero gives in each tier, pinned beside the two versions.
    ///
    /// This test and the others below are the only ones that drive `SeededStream`: every
    /// other test supplies its draws through the fake. They are the exception because what
    /// they pin is what a seed gives.
    ///
    /// These givens follow from three things: this module's draws, the solver's tie-break
    /// (the order it takes tied cells and tied digits in), and what its propagation strikes
    /// before it guesses. The last two decide which solutions of eleven seeded givens the
    /// search reaches, and so the grid. Whoever changes any of the three so that this
    /// literal changes, changes `GENERATION_VERSION` in the same commit. One seed is a
    /// tripwire and not a proof: `a_hundred_seeds_fold_to_the_pinned_number` covers more.
    #[test]
    fn seed_zero_gives_the_pinned_puzzles() {
        assert_eq!(GENERATION_VERSION, "generation-1");
        assert_eq!(RANDOM_VERSION, "seeded-stream-1");
        // Each is 81 cells, a row after a row, with a dot for an empty cell.
        let pinned = [
            "2...38.51317546.29..8.1276..41325..66.218934.5.34671.213685429798.2716.472...3518",
            "2...38.5.31..46.29..8.1.76..41325..66..189.4.5.34671.213685429.9...71..4.2...3518",
            "...7.8.5.3.7.4.........2.638.1.2.9...7.1.9.4.5.3.6.1.2.3.8.4.9.9.5.7.6.4.2.6.3.1.",
            ".......5131.............76.84............9..5.9346........5..9.98527..3.724693518",
            ".................9.....2763..1........2..9.45..3.6.18..3..54....85.7.6...2..93518",
        ];
        assert_eq!(TIERS.map(|tier| picture(&from_seed(tier, 0).0)), pinned);
    }

    /// The givens of seeds 0 to 99 in tier 5, folded into one number.
    ///
    /// The rule, written out: begin from `0xcbf2_9ce4_8422_2325`; for each seed in order,
    /// and for each of its puzzle's 81 cells a row after a row, exclusive-or the number
    /// with the cell's given digit, or with zero for an empty cell, and multiply it by
    /// `0x0100_0000_01b3`, wrapping. It is the FNV-1a fold over 8,100 cells.
    ///
    /// A change in the solver can move other seeds' grids and leave seed zero's alone, so
    /// this pins a hundred. As above, the givens follow from this module's draws, from the
    /// solver's tie-break and from what its propagation strikes, and whoever changes any
    /// of them so that this number changes, changes `GENERATION_VERSION` in the same commit.
    #[test]
    fn a_hundred_seeds_fold_to_the_pinned_number() {
        assert_eq!(GENERATION_VERSION, "generation-1");
        assert_eq!(RANDOM_VERSION, "seeded-stream-1");
        let fold = |number: u64, cell: char| {
            let digit = u64::from(cell.to_digit(10).unwrap_or(0));
            (number ^ digit).wrapping_mul(0x0100_0000_01b3)
        };
        let folded = (0..100).fold(0xcbf2_9ce4_8422_2325, |number, seed| {
            picture(&from_seed(Tier::Five, seed).0)
                .chars()
                .fold(number, fold)
        });
        assert_eq!(folded, 0xe8fb_1cd6_cf4c_a761);
    }

    /// Seeds 0 to 99 in one tier: each keeps the guarantees, and the figures the ticket
    /// that built the module quotes are printed. The figures follow from the solver and are
    /// not asserted.
    fn a_hundred_seeds_keep_the_guarantees(tier: Tier) {
        let puzzles: Vec<(WellPosed, u64)> = (0..100).map(|seed| from_seed(tier, seed)).collect();
        for (proof, _) in &puzzles {
            assert_the_guarantees(tier, proof);
        }
        let counts = puzzles.iter().map(|(proof, _)| proof.givens().len());
        let within = counts.clone().filter(|&count| count <= tier.most_givens());
        let draws = puzzles.iter().map(|&(_, draws_taken)| draws_taken);
        println!(
            "tier {}: givens {} to {}, within the range {} of 100, draws taken {} to {}",
            tier.number(),
            counts.clone().min().unwrap(),
            counts.max().unwrap(),
            within.count(),
            draws.clone().min().unwrap(),
            draws.max().unwrap(),
        );
    }

    #[test]
    fn a_hundred_seeds_keep_the_guarantees_in_tier_1() {
        a_hundred_seeds_keep_the_guarantees(Tier::One);
    }

    #[test]
    fn a_hundred_seeds_keep_the_guarantees_in_tier_2() {
        a_hundred_seeds_keep_the_guarantees(Tier::Two);
    }

    #[test]
    fn a_hundred_seeds_keep_the_guarantees_in_tier_3() {
        a_hundred_seeds_keep_the_guarantees(Tier::Three);
    }

    #[test]
    fn a_hundred_seeds_keep_the_guarantees_in_tier_4() {
        a_hundred_seeds_keep_the_guarantees(Tier::Four);
    }

    #[test]
    fn a_hundred_seeds_keep_the_guarantees_in_tier_5() {
        a_hundred_seeds_keep_the_guarantees(Tier::Five);
    }
}
