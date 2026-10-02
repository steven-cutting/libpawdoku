//! Pictures of what the public API hands out, for review and change detection. A
//! snapshot proves no clause: the tests that assert are in the other files, and every
//! test here is named `snapshot_...`. The helpers read the public API and nothing else.
//!
//! A board is drawn through `Board` alone, so no picture of one holds anything read
//! from the solution but a check's yes or no.
//!
//! The guess counts these pictures show follow from the solver's tie-break, as
//! `the_guess_counts_follow_from_the_tie_break` in `tests/solver.rs` says, and so does
//! which two solutions are shown, and in which order, where there are many.

#[cfg(test)]
mod tests {
    use core::fmt::Write;
    use pawdoku::board::{Board, BoardCell, MoveKind, PlayError};
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

    // The board ------------------------------------------------------------------

    /// One step of a script played on a board: something asked of it, which it accepts.
    type Step = fn(&mut Board) -> Result<(), PlayError>;

    /// A cell the fixture leaves empty, whose solution digit is 4, and the cell beside
    /// it in the row.
    const FREE: Position = Position::new(1, 3);
    const BESIDE: Position = Position::new(1, 4);

    /// Two more cells the fixture leaves empty: one below the given 3 of row 1, whose
    /// solution digit is 7, and one below the given 5 of row 1, in its box.
    const BELOW: Position = Position::new(2, 2);
    const CORNER: Position = Position::new(3, 1);

    /// The scripted game, played on the fixture. It uses every kind of move, upkeep
    /// striking a peer's mark, an undo, a redo, a move that discards an undone one, and
    /// a check answered each way. It leaves a note waiting beneath a digit, a conflict
    /// standing and one move undone. The record's snapshots write this same board.
    const THE_SCRIPTED_GAME: [Step; 18] = [
        |board| board.write_mark(BESIDE, 4),
        |board| board.write_mark(BESIDE, 6),
        |board| board.write_mark(BESIDE, 9),
        |board| board.strike_mark(BESIDE, 9),
        |board| board.write_mark(FREE, 2),
        // Upkeep strikes the 4 beside it; the 2 waits beneath the digit.
        |board| board.place(FREE, 4),
        |board| board.check(FREE).map(drop),
        // A second 3 in column 2: wrong, and in conflict with the given above it.
        |board| board.place(BELOW, 3),
        |board| board.check(BELOW).map(drop),
        |board| board.erase(BELOW),
        // A second 5 in column 1 and in the box: a conflict that stays.
        |board| board.place(CORNER, 5),
        |board| board.write_mark(BELOW, 7),
        Board::undo,
        Board::redo,
        Board::undo,
        // A new move discards the undone mark.
        |board| board.write_mark(BELOW, 1),
        |board| board.place(BELOW, 7),
        Board::undo,
    ];

    /// A board opened on the fixture with `script` played on it.
    fn played(script: &[Step]) -> Board {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        for step in script {
            step(&mut board).unwrap();
        }
        board
    }

    /// The fixture played to its end: every cell the setter left, filled in grid order
    /// with the digit the solver's proof holds for it.
    fn played_to_the_end() -> Board {
        let solution = *solve(givens(FIXTURE)).unwrap().solution();
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        let empty: Vec<Position> = board
            .cells()
            .filter(|cell| !cell.is_given())
            .map(BoardCell::position)
            .collect();
        for at in empty {
            let digit = solution[usize::from(at.row() - 1)][usize::from(at.column() - 1)];
            board.place(at, digit).unwrap();
        }
        board
    }

    /// The board's cells as nine rows, each cell drawn by `mark`.
    fn draw_cells(board: &Board, mark: impl Fn(BoardCell) -> char) -> String {
        let cells: Vec<char> = board.cells().map(mark).collect();
        let rows = cells.chunks(9).map(|row| row.iter().collect());
        rows.collect::<Vec<String>>().join("\n")
    }

    fn place(at: Position) -> String {
        format!("row {}, column {}", at.row(), at.column())
    }

    /// Each cell's note that holds a mark, shown or waiting. A note beneath a digit is
    /// not shown, so it is read from the latest standing move.
    fn render_notes(board: &Board) -> String {
        let latest = board.moves().filter(|made| !made.is_undone()).last();
        let mut picture = String::new();
        for cell in board.cells() {
            let waiting = latest.as_ref().map(|made| made.note_after(cell.position()));
            let state = if cell.shows_note() { "" } else { " (waiting)" };
            let note = cell.note().or(waiting).unwrap_or_default();
            let marks: Vec<String> = note.digits().map(|digit| digit.to_string()).collect();
            if !note.is_empty() {
                let at = place(cell.position());
                writeln!(picture, "{at}: {}{state}", marks.join(" ")).unwrap();
            }
        }
        picture
    }

    /// Every move with its index, kind, target, digit and whether it is undone.
    fn render_moves(board: &Board) -> String {
        let mut picture = String::new();
        for made in board.moves() {
            let kind = match made.kind() {
                MoveKind::Place => "place",
                MoveKind::Erase => "erase",
                MoveKind::WriteMark => "write mark",
                MoveKind::StrikeMark => "strike mark",
                _ => "a kind this picture does not know",
            };
            let digit = made
                .digit()
                .map_or(String::new(), |digit| format!(" {digit}"));
            let undone = if made.is_undone() { " (undone)" } else { "" };
            let at = place(made.target());
            writeln!(picture, "{}: {kind}{digit} at {at}{undone}", made.index()).unwrap();
        }
        picture
    }

    /// Every check with its index, how many moves stood, what was asked and the answer.
    fn render_checks(board: &Board) -> String {
        let mut picture = String::new();
        for check in board.checks() {
            let answer = if check.is_right() { "right" } else { "wrong" };
            let (index, stood, digit) = (check.index(), check.after_move(), check.digit());
            let at = place(check.target());
            writeln!(
                picture,
                "{index}: after move {stood}, {digit} at {at}: {answer}"
            )
            .unwrap();
        }
        picture
    }

    /// A board as text, read through `Board` alone: the puzzle's facts and the two
    /// offers; the grid; the givens; the conflicts; each note, shown or waiting; every
    /// move; and every check.
    fn render_board(board: &Board) -> String {
        let digit = |digit: Option<u8>| digit.map_or('.', |digit| char::from(b'0' + digit));
        let none_or = |list: String| {
            let list = list.trim_end();
            String::from(if list.is_empty() { "none" } else { list })
        };
        let facts = format!(
            "status: {:?}\nfull: {}\nconsistent: {}\ncan undo: {}\ncan redo: {}",
            board.status(),
            board.is_full(),
            board.is_consistent(),
            board.can_undo(),
            board.can_redo(),
        );
        let conflict = |cell: BoardCell| if cell.is_conflicting() { 'x' } else { '.' };
        let sections = [
            ("digits", draw_cells(board, |cell| digit(cell.digit()))),
            (
                "givens",
                draw_cells(board, |cell| {
                    digit(cell.digit().filter(|_| cell.is_given()))
                }),
            ),
            ("conflicts", draw_cells(board, conflict)),
            ("notes", none_or(render_notes(board))),
            ("moves", none_or(render_moves(board))),
            ("checks", none_or(render_checks(board))),
        ];
        let mut picture = facts;
        for (title, body) in sections {
            write!(picture, "\n\n{title}\n{body}").unwrap();
        }
        picture + "\n"
    }

    #[test]
    fn snapshot_the_board_as_opened() {
        insta::assert_snapshot!(render_board(&played(&[])));
    }

    #[test]
    fn snapshot_the_board_after_the_scripted_game() {
        insta::assert_snapshot!(render_board(&played(&THE_SCRIPTED_GAME)));
    }

    #[test]
    fn snapshot_the_board_played_to_the_end() {
        insta::assert_snapshot!(render_board(&played_to_the_end()));
    }

    #[test]
    fn snapshot_the_boards_refusals() {
        let (free, given) = (FREE, Position::new(1, 1));
        let mut board = played(&[]);
        let mut refusals: Vec<(&str, Result<(), PlayError>)> = vec![
            (
                "a position off the grid",
                board.place(Position::new(0, 10), 4),
            ),
            ("placing on a given", board.place(given, 4)),
            ("a digit out of range", board.place(free, 10)),
            ("erasing an empty cell", board.erase(free)),
            (
                "striking a mark that is not written",
                board.strike_mark(free, 4),
            ),
            ("undo with no move standing", board.undo()),
            ("redo with no move undone", board.redo()),
        ];
        board.write_mark(free, 4).unwrap();
        refusals.push(("writing a mark already written", board.write_mark(free, 4)));
        board.place(free, 4).unwrap();
        refusals.push(("placing the digit that stands", board.place(free, 4)));
        refusals.push(("writing a mark beneath a digit", board.write_mark(free, 2)));
        refusals.push(("undo on a solved puzzle", played_to_the_end().undo()));

        let mut picture = String::from("PlayError (Display)\n");
        for (case, refused) in refusals {
            writeln!(picture, "{case}: {}", refused.unwrap_err()).unwrap();
        }
        insta::assert_snapshot!(picture);
    }
}
