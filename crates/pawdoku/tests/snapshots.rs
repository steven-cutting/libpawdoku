//! Pictures of what the public API hands out, for review and change detection. A
//! snapshot proves no clause: the tests that assert are in the other files, and every
//! test here is named `snapshot_...`. The helpers read the public API and nothing else.
//!
//! The guess counts these pictures show follow from the solver's tie-break, as
//! `the_guess_counts_follow_from_the_tie_break` in `tests/solver.rs` says, and so does
//! which two solutions are shown, and in which order, where there are many.

#[cfg(test)]
mod tests {
    use core::fmt::Write;
    use pawdoku::solver::{SearchResult, search, solve};
    use pawdoku::sudoku::{Given, Grid, Position};

    /// The rules' fixture: thirty givens, one solution, solved by singles alone.
    const FIXTURE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// Norvig's hard puzzle, which needs a guess; `tests/solver.rs` cites it.
    const NEEDS_A_GUESS: &str =
        "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";

    /// The fixture without its given at row 3, column 8: exactly two solutions.
    const TWO_SOLUTIONS: &str =
        "53..7....6..195....98......8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// The filled cells of a picture read a row after a row, as givens.
    fn givens(picture: &str) -> Vec<Given> {
        let cells = picture.bytes().zip(0_u8..);
        cells
            .filter(|(cell, _)| cell.is_ascii_digit())
            .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
            .collect()
    }

    /// A grid as nine rows of digits, with a dot for an empty cell.
    fn draw(grid: &Grid<Option<u8>>) -> String {
        let cell = |digit: &Option<u8>| digit.map_or('.', |digit| char::from(b'0' + digit));
        let rows = grid.iter().map(|row| row.iter().map(cell).collect());
        rows.collect::<Vec<String>>().join("\n")
    }

    /// The grid that holds each given at its position. The givens are on the grid.
    fn laid_out(givens: &[Given]) -> Grid<Option<u8>> {
        let mut grid = [[None; 9]; 9];
        for given in givens {
            let (row, column) = (given.position().row(), given.position().column());
            grid[usize::from(row - 1)][usize::from(column - 1)] = Some(given.digit());
        }
        grid
    }

    /// A search result as its verdict, its guess count and each solution as a grid.
    fn render(result: &SearchResult) -> String {
        let mut picture = format!(
            "verdict: {:?}\nguesses: {}\n",
            result.verdict(),
            result.guesses()
        );
        for (number, solution) in (1..).zip(result.solutions()) {
            let filled = solution.map(|row| row.map(Some));
            let _ = write!(picture, "\nsolution {number}\n{}\n", draw(&filled));
        }
        picture
    }

    /// Givens on the grid and what `search` makes of them.
    fn render_search(picture: &str) -> String {
        let givens = givens(picture);
        let drawn = draw(&laid_out(&givens));
        format!("givens\n{drawn}\n\n{}", render(&search(givens)))
    }

    #[test]
    fn snapshot_the_search_of_the_fixture() {
        insta::assert_snapshot!(render_search(FIXTURE));
    }

    #[test]
    fn snapshot_the_search_of_a_puzzle_that_needs_a_guess() {
        insta::assert_snapshot!(render_search(NEEDS_A_GUESS));
    }

    #[test]
    fn snapshot_the_search_of_givens_with_two_solutions() {
        insta::assert_snapshot!(render_search(TWO_SOLUTIONS));
    }

    #[test]
    fn snapshot_the_search_of_malformed_givens() {
        // A position off the grid cannot be drawn, so these givens are listed.
        let malformed = [
            Given::new(Position::new(1, 1), 5),
            Given::new(Position::new(0, 10), 3),
            Given::new(Position::new(2, 2), 12),
        ];
        let mut picture = String::from("givens\n");
        for given in malformed {
            let (row, column) = (given.position().row(), given.position().column());
            writeln!(picture, "row {row}, column {column}: {}", given.digit()).unwrap();
        }
        write!(picture, "\n{}", render(&search(malformed))).unwrap();
        insta::assert_snapshot!(picture);
    }

    // This file holds a solution on purpose: the proof exposes it.
    #[test]
    fn snapshot_the_proof_of_the_fixture() {
        let proof = solve(givens(FIXTURE)).unwrap();
        let held: Vec<Given> = proof.givens().iter().copied().collect();
        let solution = proof.solution().map(|row| row.map(Some));
        insta::assert_snapshot!(format!(
            "givens\n{}\n\nsolution\n{}\n",
            draw(&laid_out(&held)),
            draw(&solution)
        ));
    }

    #[test]
    fn snapshot_the_three_refusals() {
        let every_cell = solve(givens(FIXTURE))
            .unwrap()
            .solution()
            .map(|row| row.map(Some));
        let nothing_left = givens(&draw(&every_cell).replace('\n', ""));
        let refusals = [
            ("no solution", solve([Given::new(Position::new(0, 10), 3)])),
            ("several solutions", solve(givens(TWO_SOLUTIONS))),
            ("nothing left to play", solve(nothing_left)),
        ];
        let mut picture = String::from("SolveError (Display)\n");
        for (case, refused) in refusals {
            writeln!(picture, "{case}: {}", refused.unwrap_err()).unwrap();
        }
        insta::assert_snapshot!(picture);
    }
}
