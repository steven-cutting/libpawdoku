//! The solver from outside the crate: `search` gives any givens a true verdict, the
//! solutions and the count of guesses, and `solve` gives the proof or a refusal. Each
//! test is named after the clause of `docs/specs/solver.allium` it proves.

extern crate alloc;

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use pawdoku::solver::{SearchResult, SolveError, Verdict, search, solve};
    use pawdoku::sudoku::{Given, Grid, Position, WellPosedError};
    use proptest::prelude::*;

    /// The rules' fixture: the example of the article `sudoku.allium` cites. Thirty givens,
    /// one solution, and naked and hidden singles alone solve it.
    const FIXTURE: [&str; 9] = [
        "53..7....",
        "6..195...",
        ".98....6.",
        "8...6...3",
        "4..8.3..1",
        "7...2...6",
        ".6....28.",
        "...419..5",
        "....8..79",
    ];

    /// The one solution of [`FIXTURE`].
    const FIXTURE_SOLUTION: [&str; 9] = [
        "534678912",
        "672195348",
        "198342567",
        "859761423",
        "426853791",
        "713924856",
        "961537284",
        "287419635",
        "345286179",
    ];

    /// A puzzle singles cannot solve, from Peter Norvig's "Solving Every Sudoku Puzzle"
    /// (<https://norvig.com/sudoku.html>, read 2026-10-01), the essay `solver.allium` cites.
    /// It is the essay's `grid2`, which it takes from a list of hard puzzles
    /// (`magictour.free.fr/top95`): seventeen givens and one solution.
    const NEEDS_A_GUESS: [&str; 9] = [
        "4.....8.5",
        ".3.......",
        "...7.....",
        ".2.....6.",
        "....8.4..",
        "....1....",
        "...6.3.7.",
        "5..2.....",
        "1.4......",
    ];

    /// The solution the essay prints for [`NEEDS_A_GUESS`].
    const NEEDS_A_GUESS_SOLUTION: [&str; 9] = [
        "417369825",
        "632158947",
        "958724316",
        "825437169",
        "791586432",
        "346912758",
        "289643571",
        "573291684",
        "164875293",
    ];

    /// [`FIXTURE`] without its given at row 3, column 8, a 6: twenty-nine givens and
    /// exactly two solutions, [`FIXTURE_SOLUTION`] and [`OTHER_SOLUTION`]. Made for these
    /// tests by trying each given of the fixture in turn; the oracle below counts the two.
    const TWO_SOLUTIONS: [&str; 9] = [
        "53..7....",
        "6..195...",
        ".98......",
        "8...6...3",
        "4..8.3..1",
        "7...2...6",
        ".6....28.",
        "...419..5",
        "....8..79",
    ];

    /// The solution of [`TWO_SOLUTIONS`] that is not the fixture's. It parts from it in
    /// twelve cells.
    const OTHER_SOLUTION: [&str; 9] = [
        "534678912",
        "672195438",
        "198342657",
        "819764523",
        "426853791",
        "753921846",
        "961537284",
        "287419365",
        "345286179",
    ];

    /// [`FIXTURE_SOLUTION`] with four cells emptied: rows 3 and 6 of columns 5 and 6, which
    /// hold 4 and 2 one way round and can as well hold them the other. Exactly two
    /// solutions, and each empty cell has two candidates.
    const TWO_SOLUTIONS_DENSE: [&str; 9] = [
        "534678912",
        "672195348",
        "1983..567",
        "859761423",
        "426853791",
        "7139..856",
        "961537284",
        "287419635",
        "345286179",
    ];

    /// Thirty-four cells of [`FIXTURE_SOLUTION`], chosen so that no empty cell has one
    /// candidate to begin with and singles still solve it: `PlaceHiddenSingle` has to make
    /// the first placement. Made for these tests by thinning the solution.
    const HIDDEN_SINGLES: [&str; 9] = [
        ".34.7..1.",
        "....95...",
        "198.4...7",
        "...7614.3",
        "4......9.",
        "713.248..",
        "96...7.84",
        "...41.6..",
        "...28....",
    ];

    /// Givens with no conflict among them that leave row 1, column 1 no candidate: its row
    /// holds 1, 2 and 3, its column 4, 5 and 6, and its box 7, 8 and 9.
    const NO_CANDIDATE: [&str; 9] = [
        "...123...",
        ".78......",
        ".9.......",
        "4........",
        "5........",
        "6........",
        ".........",
        ".........",
        ".........",
    ];

    /// Givens with no conflict among them that leave row 1 no place for a 1, while every
    /// cell still has a candidate.
    const NO_PLACE: [&str; 9] = [
        "........2",
        "1........",
        "...1.....",
        "......1..",
        ".........",
        ".........",
        ".......1.",
        ".........",
        ".........",
    ];

    /// Givens with no conflict among them that make row 1, column 1 the only place for a 1
    /// in its row and the only place for a 2 in its column.
    const OVERDEMANDED: [&str; 9] = [
        ".........",
        "....1.2..",
        "...2...1.",
        ".1.......",
        "..2......",
        ".........",
        "..1......",
        ".2.......",
        ".........",
    ];

    /// The filled cells of a picture, as givens, in grid order: a digit for a filled cell
    /// and a dot for an empty one.
    fn givens(rows: [&str; 9]) -> Vec<Given> {
        let cell = |row: u8, column: u8, cell: u8| {
            cell.is_ascii_digit()
                .then(|| Given::new(Position::new(row, column), cell - b'0'))
        };
        let row = |(row, line): (u8, &str)| {
            let cells = (1..).zip(line.bytes());
            cells
                .filter_map(move |(column, held)| cell(row, column, held))
                .collect::<Vec<_>>()
        };
        (1..).zip(rows).flat_map(row).collect()
    }

    /// A picture of a full grid, as the grid a result or a proof holds.
    fn grid(rows: [&str; 9]) -> Grid<u8> {
        rows.map(|row| {
            let mut digits = [0; 9];
            for (digit, cell) in digits.iter_mut().zip(row.bytes()) {
                *digit = cell - b'0';
            }
            digits
        })
    }

    /// What `RefuseMalformedGivens` leaves: `none`, nothing guessed and nothing found.
    fn assert_refused(result: &SearchResult) {
        assert_eq!(result.verdict(), Verdict::NoSolution);
        assert_eq!(result.guesses(), 0);
        assert!(result.solutions().is_empty());
    }

    #[test]
    fn a_given_off_the_grid_gets_none_with_no_guess() {
        for position in [
            Position::new(0, 1),
            Position::new(10, 1),
            Position::new(1, 0),
            Position::new(1, 10),
            Position::new(u8::MAX, u8::MAX),
        ] {
            assert_refused(&search([Given::new(position, 5)]));
            let mut spoiled = givens(FIXTURE);
            spoiled.push(Given::new(position, 5));
            assert_refused(&search(spoiled));
        }
    }

    #[test]
    fn a_digit_out_of_range_gets_none_with_no_guess() {
        for digit in [0, 10, u8::MAX] {
            assert_refused(&search([Given::new(Position::new(5, 5), digit)]));
        }
    }

    #[test]
    fn two_digits_to_one_position_get_none_with_no_guess() {
        let here = Position::new(1, 3);
        assert_refused(&search([Given::new(here, 4), Given::new(here, 2)]));
        // The fixture's solution holds 4 there, and a second digit still spoils it.
        let mut spoiled = givens(FIXTURE);
        spoiled.extend([Given::new(here, 4), Given::new(here, 2)]);
        assert_refused(&search(spoiled));
    }

    #[test]
    fn singles_alone_solve_the_fixture_with_no_guess() {
        let result = search(givens(FIXTURE));
        assert_eq!(result.verdict(), Verdict::OneSolution);
        assert_eq!(result.guesses(), 0);
        assert_eq!(result.solutions(), [grid(FIXTURE_SOLUTION)]);
    }

    #[test]
    fn two_givens_that_agree_are_one_given() {
        let mut twice = givens(FIXTURE);
        twice.extend(givens(FIXTURE));
        twice.push(Given::new(Position::new(1, 1), 5));
        assert_eq!(search(twice), search(givens(FIXTURE)));
    }

    #[test]
    fn anything_may_be_handed_over() {
        assert_eq!(search([]).verdict(), Verdict::ManySolutions);
        let conflicting = [1, 9].map(|column| Given::new(Position::new(1, column), 5));
        assert_eq!(search(conflicting).verdict(), Verdict::NoSolution);
        let every_cell = givens(FIXTURE_SOLUTION);
        assert_eq!(every_cell.len(), 81);
        let result = search(every_cell);
        assert_eq!(result.verdict(), Verdict::OneSolution);
        assert_eq!(result.guesses(), 0);
        assert_eq!(result.solutions(), [grid(FIXTURE_SOLUTION)]);
    }

    /// The digits a cell could hold, going by the givens among its peers and nothing else.
    fn open_digits(givens: &[Given], position: Position) -> usize {
        let is_peer = |other: Position| {
            let same_box = (other.row() - 1) / 3 == (position.row() - 1) / 3
                && (other.column() - 1) / 3 == (position.column() - 1) / 3;
            other.row() == position.row() || other.column() == position.column() || same_box
        };
        let seen = |digit: u8| {
            let mut peers = givens.iter().filter(|given| is_peer(given.position()));
            peers.any(|given| given.digit() == digit)
        };
        (1..=9).filter(|&digit| !seen(digit)).count()
    }

    #[test]
    fn hidden_singles_solve_a_puzzle_with_no_naked_single() {
        let puzzle = givens(HIDDEN_SINGLES);
        let given_at = |position| puzzle.iter().any(|given| given.position() == position);
        let empty = (1..=9)
            .flat_map(|row| (1..=9).map(move |column| Position::new(row, column)))
            .filter(|&position| !given_at(position));
        // No empty cell has one candidate, so `PlaceNakedSingle` has nowhere to begin: the
        // first digit placed is one `PlaceHiddenSingle` places.
        assert_eq!(empty.clone().count(), 47);
        assert!(
            empty
                .clone()
                .all(|position| open_digits(&puzzle, position) > 1)
        );

        let result = search(puzzle);
        assert_eq!(result.verdict(), Verdict::OneSolution);
        assert_eq!(result.guesses(), 0);
        assert_eq!(result.solutions(), [grid(FIXTURE_SOLUTION)]);
    }

    #[test]
    fn givens_in_conflict_get_none_with_no_guess() {
        // One digit twice in a row, in a column, and in a box where neither is shared.
        for second in [
            Position::new(1, 9),
            Position::new(9, 1),
            Position::new(2, 2),
        ] {
            let conflicting = [Position::new(1, 1), second].map(|at| Given::new(at, 5));
            assert_refused(&search(conflicting));
        }
        // The fixture with a second 5 in its first row.
        let mut spoiled = givens(FIXTURE);
        spoiled.push(Given::new(Position::new(1, 9), 5));
        assert_refused(&search(spoiled));
    }

    #[test]
    fn a_cell_with_no_candidate_ends_the_search_with_no_guess() {
        assert_refused(&search(givens(NO_CANDIDATE)));
    }

    #[test]
    fn a_unit_with_no_place_for_a_digit_ends_the_search_with_no_guess() {
        assert_refused(&search(givens(NO_PLACE)));
    }

    #[test]
    fn a_cell_that_is_the_only_place_for_two_digits_ends_the_search_with_no_guess() {
        assert_refused(&search(givens(OVERDEMANDED)));
    }

    #[test]
    fn a_published_puzzle_that_needs_a_guess_is_solved() {
        let result = search(givens(NEEDS_A_GUESS));
        assert_eq!(result.verdict(), Verdict::OneSolution);
        assert_eq!(result.solutions(), [grid(NEEDS_A_GUESS_SOLUTION)]);
        assert!(result.guesses() >= 1);
    }

    /// **These numbers follow from the tie-break and not from the specification.**
    /// `solver.allium` leaves the order of tied cells and tied branches to the
    /// implementation and asks only that it be the same each time. The rule the module
    /// documents is the first cell in grid order and the lowest digit first. Another rule
    /// would be the same solver with other counts here, and a consumer that has recorded a
    /// count would see it change. The counts were first worked out apart from this crate,
    /// by a model written from the specification's text, on 2026-10-01.
    #[test]
    fn the_guess_counts_follow_from_the_tie_break() {
        assert_eq!(search(givens(FIXTURE)).guesses(), 0);
        assert_eq!(search(givens(NEEDS_A_GUESS)).guesses(), 50);
        assert_eq!(search(givens(TWO_SOLUTIONS)).guesses(), 1);
        assert_eq!(search([]).guesses(), 47);
    }

    #[test]
    fn a_split_counts_one_guess_whatever_its_children() {
        // Four empty cells, two candidates each: one split, and its two children are both
        // solved by propagation alone.
        let result = search(givens(TWO_SOLUTIONS_DENSE));
        assert_eq!(result.verdict(), Verdict::ManySolutions);
        assert_eq!(result.guesses(), 1);
    }

    #[test]
    fn the_second_solution_concludes_many_with_both_exposed() {
        let result = search(givens(TWO_SOLUTIONS));
        assert_eq!(result.verdict(), Verdict::ManySolutions);
        let [first, second] = result.solutions() else {
            panic!("many exposes two solutions");
        };
        assert_ne!(first, second);
        // The lower digit is guessed first, and the fixture's own solution holds the lower
        // digit in the cell split on, so it is found first.
        assert_eq!(*first, grid(FIXTURE_SOLUTION));
        assert_eq!(*second, grid(OTHER_SOLUTION));
    }

    #[test]
    fn one_solution_concludes_one() {
        let result = search(givens(FIXTURE));
        assert_eq!(result.verdict(), Verdict::OneSolution);
        assert_eq!(result.solutions().len(), 1);
    }

    #[test]
    fn no_solution_concludes_none() {
        // Every given fits its row, column and box, and nothing completes them: the given
        // at row 1, column 3 is a 2 where the fixture's one solution holds a 4.
        let mut spoiled = givens(FIXTURE);
        spoiled.push(Given::new(Position::new(1, 3), 2));
        let result = search(spoiled);
        assert_eq!(result.verdict(), Verdict::NoSolution);
        assert!(result.solutions().is_empty());
    }

    #[test]
    fn the_empty_grid_concludes_many_and_stops_at_two() {
        let result = search([]);
        assert_eq!(result.verdict(), Verdict::ManySolutions);
        assert_eq!(result.solutions().len(), 2);
    }

    #[test]
    fn the_result_exposes_the_verdict_the_guesses_and_every_digit() {
        let result = search(givens(TWO_SOLUTIONS));
        assert_eq!(result.verdict(), Verdict::ManySolutions);
        assert_eq!(result.guesses(), 1);
        let pictures = [FIXTURE_SOLUTION, OTHER_SOLUTION];
        for (solution, picture) in result.solutions().iter().zip(pictures) {
            for given in givens(picture) {
                let (row, column) = (given.position().row(), given.position().column());
                let digit = solution[usize::from(row - 1)][usize::from(column - 1)];
                assert_eq!(digit, given.digit(), "row {row}, column {column}");
            }
        }
        // The surface's `search.status` has no value of its own: `search` hands a result
        // over only once the search has concluded, and being handed it says so.
    }

    #[test]
    fn solve_gives_the_proof_for_the_fixture() {
        let proof = solve(givens(FIXTURE)).unwrap();
        assert!(proof.givens().iter().copied().eq(givens(FIXTURE)));
        assert_eq!(proof.givens().len(), 30);
        assert_eq!(*proof.solution(), grid(FIXTURE_SOLUTION));
    }

    #[test]
    fn solve_refuses_givens_with_no_solution() {
        let malformed = [Given::new(Position::new(0, 1), 5)];
        assert_eq!(solve(malformed), Err(SolveError::NoSolution));
        assert_eq!(solve(givens(NO_PLACE)), Err(SolveError::NoSolution));
        let mut spoiled = givens(FIXTURE);
        spoiled.push(Given::new(Position::new(1, 3), 2));
        assert_eq!(solve(spoiled), Err(SolveError::NoSolution));
    }

    #[test]
    fn solve_refuses_givens_with_several_solutions() {
        assert_eq!(solve([]), Err(SolveError::ManySolutions));
        assert_eq!(solve(givens(TWO_SOLUTIONS)), Err(SolveError::ManySolutions));
    }

    #[test]
    fn solve_refuses_givens_that_leave_nothing_to_play() {
        let every_cell = givens(FIXTURE_SOLUTION);
        assert_eq!(search(every_cell.clone()).verdict(), Verdict::OneSolution);
        assert_eq!(
            solve(every_cell.clone()),
            Err(SolveError::NotPosed(WellPosedError::NothingLeftToPlay {
                givens: 81
            }))
        );
        // Eighty givens leave one cell, and that is something to play.
        let proof = solve(every_cell.into_iter().skip(1)).unwrap();
        assert_eq!(proof.givens().len(), 80);
        assert_eq!(*proof.solution(), grid(FIXTURE_SOLUTION));
    }

    #[test]
    fn each_refusal_reads_as_its_text() {
        let texts = [
            (solve(givens(NO_PLACE)), "the givens have no solution"),
            (solve([]), "the givens have more than one solution"),
            (
                solve(givens(FIXTURE_SOLUTION)),
                "81 givens leave nothing to play: a puzzle has fewer givens than its 81 cells",
            ),
        ];
        for (refused, text) in texts {
            assert_eq!(refused.unwrap_err().to_string(), text);
        }
    }

    // ---------------------------------------------------------------- the oracle ---
    //
    // `VerdictIsTrue` ties the verdict to `sudoku.allium`'s `solution_count`, so it needs a
    // count the solver did not produce. This is one, written from that black box's words
    // and nothing of the solver: digits tried cell by cell in grid order, no candidates and
    // no propagation, stopping at two. It keeps its place in a list of the empty cells
    // where another would recurse.

    /// A grid as the oracle keeps it: a digit, or 0 for an empty cell.
    type Plain = [[u8; 9]; 9];

    /// Whether `digit` at `row`, `column` (from 0) repeats in the cell's row, column or box.
    fn repeats(grid: &Plain, row: usize, column: usize, digit: u8) -> bool {
        let held = |r: usize, c: usize| (r, c) != (row, column) && grid[r][c] == digit;
        let (top, left) = (row / 3 * 3, column / 3 * 3);
        (0..9).any(|i| held(row, i) || held(i, column) || held(top + i / 3, left + i % 3))
    }

    /// The givens written into a grid, or nothing when one is off the grid, out of range,
    /// or one of two digits to a position: `solution_count` is 0 for those.
    fn written(givens: &[Given]) -> Option<Plain> {
        let mut grid = [[0; 9]; 9];
        for given in givens {
            let (row, column) = (given.position().row(), given.position().column());
            let on_grid = (1..=9).contains(&row) && (1..=9).contains(&column);
            if !on_grid || !(1..=9).contains(&given.digit()) {
                return None;
            }
            let cell = &mut grid[usize::from(row - 1)][usize::from(column - 1)];
            if *cell != 0 && *cell != given.digit() {
                return None;
            }
            *cell = given.digit();
        }
        Some(grid)
    }

    /// How many solutions the givens have, counted no further than two.
    fn count_solutions(givens: &[Given]) -> usize {
        let Some(mut grid) = written(givens) else {
            return 0;
        };
        let cells = (0..9).flat_map(|row| (0..9).map(move |column| (row, column)));
        let (given, empty): (Vec<_>, Vec<_>) =
            cells.partition(|&(row, column)| grid[row][column] != 0);
        if given
            .iter()
            .any(|&(row, column)| repeats(&grid, row, column, grid[row][column]))
        {
            return 0;
        }
        let (mut found, mut at) = (0, 0);
        loop {
            // Every empty cell holds a digit: a solution. Count it and step back.
            let filled = at == empty.len();
            found += usize::from(filled);
            if filled && (found == 2 || at == 0) {
                return found;
            }
            at -= usize::from(filled);
            let (row, column) = empty[at];
            let next =
                (grid[row][column] + 1..=9).find(|&digit| !repeats(&grid, row, column, digit));
            grid[row][column] = next.unwrap_or(0);
            match (next, at) {
                (Some(_), _) => at += 1,
                (None, 0) => return found,
                (None, _) => at -= 1,
            }
        }
    }

    #[test]
    fn the_oracle_counts_the_fixtures() {
        assert_eq!(count_solutions(&givens(FIXTURE)), 1);
        assert_eq!(count_solutions(&givens(FIXTURE_SOLUTION)), 1);
        assert_eq!(count_solutions(&givens(TWO_SOLUTIONS)), 2);
        assert_eq!(count_solutions(&givens(TWO_SOLUTIONS_DENSE)), 2);
        assert_eq!(count_solutions(&[]), 2);
        assert_eq!(count_solutions(&givens(NO_CANDIDATE)), 0);
        let here = Position::new(1, 1);
        assert_eq!(count_solutions(&[Given::new(here, 0)]), 0);
        assert_eq!(count_solutions(&[Given::new(Position::new(0, 1), 5)]), 0);
        assert_eq!(
            count_solutions(&[Given::new(here, 5), Given::new(here, 6)]),
            0
        );
        let conflicting = [here, Position::new(1, 9)].map(|at| Given::new(at, 5));
        assert_eq!(count_solutions(&conflicting), 0);
    }

    // ------------------------------------------------------------ the properties ---

    /// Cells of the fixture's solution, each kept with a chance drawn from `density`: givens
    /// that have one solution or many, and never none.
    fn thinned(density: core::ops::Range<f64>) -> impl Strategy<Value = Vec<Given>> {
        let kept =
            density.prop_flat_map(|chance| prop::collection::vec(prop::bool::weighted(chance), 81));
        kept.prop_map(|kept| {
            let cells = givens(FIXTURE_SOLUTION).into_iter().zip(kept);
            cells
                .filter_map(|(given, kept)| kept.then_some(given))
                .collect()
        })
    }

    /// A dense thinning with one given's digit changed: mostly givens with no solution, by
    /// a conflict or by a digit that fits its peers and is still wrong.
    fn spoiled() -> impl Strategy<Value = Vec<Given>> {
        (thinned(0.7..1.0), any::<prop::sample::Index>(), 1_u8..=8).prop_map(
            |(mut givens, which, shift)| {
                if !givens.is_empty() {
                    let which = which.index(givens.len());
                    let given = &mut givens[which];
                    *given = Given::new(given.position(), (given.digit() + shift - 1) % 9 + 1);
                }
                givens
            },
        )
    }

    /// Givens of any kind: positions off the grid on either side, digits out of range on
    /// either side, and repeats.
    fn arbitrary() -> impl Strategy<Value = Vec<Given>> {
        let given = (0_u8..=10, 0_u8..=10, 0_u8..=10)
            .prop_map(|(row, column, digit)| Given::new(Position::new(row, column), digit));
        prop::collection::vec(given, 0..12)
    }

    /// A few well-formed givens, which may conflict.
    fn scattered() -> impl Strategy<Value = Vec<Given>> {
        let given = (1_u8..=9, 1_u8..=9, 1_u8..=9)
            .prop_map(|(row, column, digit)| Given::new(Position::new(row, column), digit));
        prop::collection::vec(given, 0..8)
    }

    /// Arbitrary givens with one more that is certainly malformed: off the grid, or a digit
    /// out of range. The oracle counts these without a search, whatever the rest are.
    fn malformed() -> impl Strategy<Value = Vec<Given>> {
        let spoiler = prop_oneof![
            (1_u8..=9, 1_u8..=9)
                .prop_map(|(row, column)| Given::new(Position::new(row, column), 0)),
            (1_u8..=9, 10_u8..).prop_map(|(row, digit)| Given::new(Position::new(row, row), digit)),
            (0_u8..=10, 1_u8..=9).prop_map(|(row, digit)| Given::new(Position::new(row, 0), digit)),
            (10_u8.., 1_u8..=9).prop_map(|(row, digit)| Given::new(Position::new(row, 5), digit)),
        ];
        (arbitrary(), spoiler, any::<prop::sample::Index>()).prop_map(
            |(mut givens, spoiler, at)| {
                givens.insert(at.index(givens.len() + 1), spoiler);
                givens
            },
        )
    }

    /// Every kind of givens above.
    fn any_givens() -> impl Strategy<Value = Vec<Given>> {
        prop_oneof![
            thinned(0.0..1.0),
            spoiled(),
            arbitrary(),
            scattered(),
            malformed()
        ]
    }

    /// Whether two positions are peers, by the specification's words.
    fn are_peers(a: Position, b: Position) -> bool {
        let same_box =
            (a.row() - 1) / 3 == (b.row() - 1) / 3 && (a.column() - 1) / 3 == (b.column() - 1) / 3;
        a != b && (a.row() == b.row() || a.column() == b.column() || same_box)
    }

    /// A solution's cells as givens: every position with the digit it holds.
    fn cells(solution: &Grid<u8>) -> Vec<Given> {
        let row = |(row, digits): (u8, &[u8; 9])| {
            let cells = (1..).zip(*digits);
            cells.map(move |(column, digit)| Given::new(Position::new(row, column), digit))
        };
        (1..).zip(solution).flat_map(row).collect()
    }

    /// `SolvedBranchesAreSolutions` for one solution: full, no two peers with one digit, and
    /// every given where the setter put it.
    fn is_a_solution_of(solution: &Grid<u8>, givens: &[Given]) -> bool {
        let cells = cells(solution);
        let full = cells.len() == 81 && cells.iter().all(|cell| (1..=9).contains(&cell.digit()));
        let conflict = |a: &Given| {
            let mut others = cells.iter();
            others.any(|b| are_peers(a.position(), b.position()) && a.digit() == b.digit())
        };
        full && !cells.iter().any(conflict) && givens.iter().all(|given| cells.contains(given))
    }

    proptest! {
        #[test]
        fn every_search_concludes(givens in any_givens()) {
            // Coming back at all is the conclusion: the test would hang otherwise.
            let result = search(givens);
            prop_assert!(result.solutions().len() <= 2);
        }

        /// The oracle has no propagation, so it is only handed givens it can count quickly:
        /// dense cells of one solution, a dense set with one digit changed, and givens that
        /// are certainly malformed. Sparse givens that are well-formed and have no solution
        /// can keep it busy without end, so no strategy here can produce them.
        #[test]
        fn the_verdict_agrees_with_the_oracle(
            givens in prop_oneof![thinned(0.6..1.0), spoiled(), malformed()]
        ) {
            let verdict = match count_solutions(&givens) {
                0 => Verdict::NoSolution,
                1 => Verdict::OneSolution,
                _ => Verdict::ManySolutions,
            };
            prop_assert_eq!(search(givens).verdict(), verdict);
        }

        #[test]
        fn the_same_givens_get_the_same_result_in_any_order(
            (givens, shuffled) in any_givens().prop_flat_map(|givens| {
                (Just(givens.clone()), Just(givens).prop_shuffle())
            })
        ) {
            let result = search(givens.clone());
            prop_assert_eq!(&search(givens.clone()), &result);
            prop_assert_eq!(&search(shuffled), &result);
            prop_assert_eq!(&search(givens.into_iter().rev()), &result);
        }

        #[test]
        fn a_result_keeps_its_invariants(givens in any_givens()) {
            let result = search(givens.clone());
            let solutions = result.solutions();
            for solution in solutions {
                prop_assert!(is_a_solution_of(solution, &givens), "SolvedBranchesAreSolutions");
            }
            let distinct = !matches!(solutions, [first, second] if first == second);
            prop_assert!(distinct, "SolutionsAreDistinct");
            prop_assert!(solutions.len() <= 2, "NoMoreSolutionsThanSought");
            let count = match result.verdict() {
                Verdict::NoSolution => 0,
                Verdict::OneSolution => 1,
                _ => 2,
            };
            prop_assert_eq!(solutions.len(), count, "VerdictMatchesSolutions");
        }

        #[test]
        fn solve_gives_a_proof_exactly_when_the_verdict_is_one(givens in any_givens()) {
            let result = search(givens.clone());
            let distinct: BTreeSet<Given> = givens.iter().copied().collect();
            match solve(givens) {
                Ok(proof) => {
                    prop_assert_eq!(result.verdict(), Verdict::OneSolution);
                    prop_assert_eq!(result.solutions(), [*proof.solution()]);
                    prop_assert_eq!(proof.givens(), &distinct);
                    prop_assert!(distinct.len() < 81);
                }
                Err(SolveError::NoSolution) => {
                    prop_assert_eq!(result.verdict(), Verdict::NoSolution);
                }
                Err(SolveError::ManySolutions) => {
                    prop_assert_eq!(result.verdict(), Verdict::ManySolutions);
                }
                Err(refusal) => {
                    let nothing_left = WellPosedError::NothingLeftToPlay { givens: 81 };
                    prop_assert_eq!(refusal, SolveError::NotPosed(nothing_left));
                    prop_assert_eq!(result.verdict(), Verdict::OneSolution);
                    prop_assert_eq!(distinct.len(), 81);
                }
            }
        }
    }
}
