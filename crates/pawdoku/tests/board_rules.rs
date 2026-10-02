//! The board's rules from outside the crate, clause by clause: opening, the four moves
//! with upkeep, undo and redo, reading back and the check, each through `Board` alone.
//! Each test is named after the clause of `docs/specs/board.allium` it proves. The
//! whole game and the property over arbitrary scripts are in `tests/board.rs`.

#[cfg(test)]
mod tests {
    use pawdoku::board::{Board, BoardCell, Check, Move, MoveKind, Note, PlayError};
    use pawdoku::solver::{SolveError, solve};
    use pawdoku::sudoku::{Given, Position, Status, WellPosedError};

    /// The rules' fixture: thirty givens, one solution, rows top to bottom.
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

    /// [`FIXTURE`] without its given at row 3, column 8: exactly two solutions, as
    /// `tests/solver.rs` shows.
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

    /// A cell the fixture leaves to the player. Its solution digit is 4, and the 5 of
    /// [`GIVEN`] is in its row and its box.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`]. Its solution digit is 6.
    const BESIDE: Position = Position::new(1, 4);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// Four positions that name no cell, one past each edge of the grid.
    const OFF_GRID: [Position; 4] = [
        Position::new(0, 1),
        Position::new(1, 0),
        Position::new(10, 1),
        Position::new(1, 10),
    ];

    /// Every position of the grid, a row at a time from the top.
    fn positions() -> impl Iterator<Item = Position> {
        (1..=9).flat_map(|row| (1..=9).map(move |column| Position::new(row, column)))
    }

    /// The filled cells of a picture, as givens.
    fn givens(rows: [&str; 9]) -> Vec<Given> {
        let cells = positions().zip(rows.iter().flat_map(|row| row.bytes()));
        cells
            .filter(|(_, cell)| cell.is_ascii_digit())
            .map(|(position, cell)| Given::new(position, cell - b'0'))
            .collect()
    }

    /// A board opened on the fixture.
    fn open() -> Board {
        Board::open(givens(FIXTURE)).unwrap()
    }

    /// One thing a caller may ask of a board.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Op {
        Place(Position, u8),
        Erase(Position),
        WriteMark(Position, u8),
        StrikeMark(Position, u8),
        Undo,
        Redo,
        Check(Position),
    }

    impl Op {
        /// Asks it of `board`, and says whether it was accepted or why it was refused.
        fn ask(self, board: &mut Board) -> Result<(), PlayError> {
            match self {
                Self::Place(position, digit) => board.place(position, digit),
                Self::Erase(position) => board.erase(position),
                Self::WriteMark(position, digit) => board.write_mark(position, digit),
                Self::StrikeMark(position, digit) => board.strike_mark(position, digit),
                Self::Undo => board.undo(),
                Self::Redo => board.redo(),
                Self::Check(position) => board.check(position).map(drop),
            }
        }
    }

    /// Every cell's digit and the marks of its note, in grid order.
    type Picture = Vec<(Option<u8>, Vec<u8>)>;

    /// The picture: every cell's digit and its note, shown or waiting. It is what undo
    /// and redo restore. A note beneath a digit is not shown, so it is read from the
    /// latest standing move, which `TheMovesReplayToTheBoard` pins to the board; with
    /// no move standing, no digit of the player's stands and no note waits.
    fn picture(board: &Board) -> Picture {
        let latest = board.moves().filter(|made| !made.is_undone()).last();
        let cells = board.cells().map(|cell| {
            let waiting = latest.as_ref().map(|made| made.note_after(cell.position()));
            let note = cell.note().or(waiting).unwrap_or_default();
            (cell.digit(), note.digits().collect())
        });
        cells.collect()
    }

    /// The board as it stood once `made` and no later move had been made.
    fn reading(made: &Move) -> Picture {
        let cells = positions().map(|position| {
            let marks = made.note_after(position).digits().collect();
            (made.digit_after(position), marks)
        });
        cells.collect()
    }

    /// One move as `Playing` exposes it, its readings included.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MoveView {
        index: usize,
        kind: MoveKind,
        target: Position,
        digit: Option<u8>,
        is_undone: bool,
        after: Picture,
    }

    /// The full view: everything `Playing` exposes. A refused operation leaves it alone.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct View {
        picture: Picture,
        /// The puzzle's three facts: its status, whether it is full, whether consistent.
        puzzle: (Status, bool, bool),
        can_undo: bool,
        can_redo: bool,
        cells: Vec<BoardCell>,
        moves: Vec<MoveView>,
        checks: Vec<Check>,
    }

    fn moves(board: &Board) -> Vec<MoveView> {
        let moves = board.moves().map(|made| MoveView {
            index: made.index(),
            kind: made.kind(),
            target: made.target(),
            digit: made.digit(),
            is_undone: made.is_undone(),
            after: reading(&made),
        });
        moves.collect()
    }

    fn view(board: &Board) -> View {
        View {
            picture: picture(board),
            puzzle: (board.status(), board.is_full(), board.is_consistent()),
            can_undo: board.can_undo(),
            can_redo: board.can_redo(),
            cells: board.cells().collect(),
            moves: moves(board),
            checks: board.checks().collect(),
        }
    }

    /// Asks `op` of `board` and holds that it is refused with `refusal` and that the
    /// full view is as it was.
    fn assert_refused(board: &mut Board, op: Op, refusal: &PlayError) {
        let before = view(board);
        assert_eq!(op.ask(board).as_ref(), Err(refusal), "{op:?}");
        assert_eq!(
            view(board),
            before,
            "{op:?} was refused and changed the board"
        );
    }

    fn digit_at(board: &Board, position: Position) -> Option<u8> {
        board.cell(position).unwrap().digit()
    }

    #[test]
    fn opening_gives_eighty_one_cells_no_note_no_move_and_no_check() {
        let board = open();
        let cells: Vec<Position> = board.cells().map(BoardCell::position).collect();
        assert_eq!(cells, positions().collect::<Vec<_>>());
        assert!(board.cells().all(|cell| {
            let none_shown = cell.note().is_none_or(Note::is_empty);
            none_shown && cell.is_given() != cell.shows_note()
        }));
        assert_eq!(board.moves().count(), 0);
        assert_eq!(board.checks().count(), 0);
        assert!(!board.can_undo());
        assert!(!board.can_redo());
        assert_eq!(board.status(), Status::Unsolved);
        assert!(!board.is_full());
        assert!(board.is_consistent());
    }

    #[test]
    fn a_board_shows_its_givens_and_leaves_every_other_cell_empty() {
        let board = open();
        let pictured = FIXTURE.iter().flat_map(|row| row.bytes());
        for (position, pictured) in positions().zip(pictured) {
            let cell = board.cell(position).unwrap();
            let digit = pictured.is_ascii_digit().then(|| pictured - b'0');
            assert_eq!(
                (cell.row(), cell.column()),
                (position.row(), position.column())
            );
            assert_eq!(cell.digit(), digit);
            assert_eq!(cell.is_given(), digit.is_some());
            assert!(!cell.is_conflicting());
        }
        assert_eq!(board.cells().filter(|cell| cell.is_given()).count(), 30);
        assert!(board.cell(Position::new(0, 1)).is_none());
        assert!(board.cell(Position::new(1, 10)).is_none());
    }

    #[test]
    fn opening_is_refused_for_givens_with_no_solution() {
        let fives = [1, 2].map(|column| Given::new(Position::new(1, column), 5));
        assert_eq!(Board::open(fives).unwrap_err(), SolveError::NoSolution);
    }

    #[test]
    fn opening_is_refused_for_givens_with_several_solutions() {
        let refusal = Board::open(givens(TWO_SOLUTIONS)).unwrap_err();
        assert_eq!(refusal, SolveError::ManySolutions);
    }

    #[test]
    fn opening_is_refused_for_givens_that_leave_nothing_to_play() {
        let solution = *solve(givens(FIXTURE)).unwrap().solution();
        let digits = solution.iter().flatten();
        let every_cell = positions()
            .zip(digits)
            .map(|(at, &digit)| Given::new(at, digit));
        let refusal = Board::open(every_cell).unwrap_err();
        let nothing_left = WellPosedError::NothingLeftToPlay { givens: 81 };
        assert_eq!(refusal, SolveError::NotPosed(nothing_left));
    }

    #[test]
    fn a_placement_puts_the_digit_in_the_cell_and_joins_the_record() {
        let mut board = open();
        let before = picture(&board);
        assert_eq!(board.place(FREE, 4), Ok(()));
        assert_eq!(digit_at(&board, FREE), Some(4));
        assert!(board.cell(FREE).unwrap().holds_players_digit());

        let changed = picture(&board).into_iter().zip(before);
        assert_eq!(changed.filter(|(after, before)| after != before).count(), 1);

        let made = moves(&board);
        assert_eq!(made.len(), 1);
        assert_eq!((made[0].index, made[0].kind), (1, MoveKind::Place));
        assert_eq!((made[0].target, made[0].digit), (FREE, Some(4)));
        assert!(!made[0].is_undone);
    }

    #[test]
    fn placing_is_refused_on_a_given_cell() {
        let mut board = open();
        let refusal = PlayError::GivenCell { position: GIVEN };
        assert_refused(&mut board, Op::Place(GIVEN, 6), &refusal);
        // The given's own digit is refused for the same reason.
        assert_refused(&mut board, Op::Place(GIVEN, 5), &refusal);
    }

    #[test]
    fn placing_is_refused_for_a_digit_out_of_range() {
        let mut board = open();
        for digit in [0, 10, 255] {
            let refusal = PlayError::DigitOutOfRange { digit };
            assert_refused(&mut board, Op::Place(FREE, digit), &refusal);
        }
    }

    #[test]
    fn a_placement_over_the_players_other_digit_is_a_move_that_remembers_what_it_displaced() {
        let mut board = open();
        board.place(FREE, 2).unwrap();
        assert_eq!(board.place(FREE, 4), Ok(()));
        assert_eq!(digit_at(&board, FREE), Some(4));

        let made = moves(&board);
        assert_eq!(made.len(), 2);
        assert_eq!((made[1].index, made[1].kind), (2, MoveKind::Place));
        // What it displaced is in the reading of the move before it.
        assert_eq!(board.moves().next().unwrap().digit_after(FREE), Some(2));
    }

    #[test]
    fn a_conflicting_placement_is_a_move() {
        let mut board = open();
        // A second 5 in row 1, beside the given 5 at its head.
        assert_eq!(board.place(FREE, 5), Ok(()));
        assert!(board.cell(FREE).unwrap().is_conflicting());
        assert!(board.cell(GIVEN).unwrap().is_conflicting());
        assert!(!board.cell(BESIDE).unwrap().is_conflicting());
        assert!(!board.is_consistent());
        assert_eq!(board.moves().count(), 1);
        assert_eq!(board.status(), Status::Unsolved);
    }

    /// The marks shown at `position`, or nothing while a digit hides the note.
    fn shown(board: &Board, position: Position) -> Option<Vec<u8>> {
        let note = board.cell(position).unwrap().note();
        note.map(|note| note.digits().collect())
    }

    #[test]
    fn an_erasure_empties_the_cell_and_joins_the_record() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        assert_eq!(board.erase(FREE), Ok(()));
        assert_eq!(digit_at(&board, FREE), None);
        assert_eq!(picture(&board), picture(&open()));

        let made = moves(&board);
        assert_eq!(made.len(), 2);
        assert_eq!((made[1].index, made[1].kind), (2, MoveKind::Erase));
        assert_eq!((made[1].target, made[1].digit), (FREE, None));
        assert!(!made[1].is_undone);
    }

    #[test]
    fn erasing_is_refused_on_a_cell_with_no_digit_of_the_players() {
        let mut board = open();
        for position in [FREE, GIVEN] {
            let refusal = PlayError::NoPlayersDigit { position };
            assert_refused(&mut board, Op::Erase(position), &refusal);
        }
    }

    #[test]
    fn a_written_mark_joins_the_note_and_the_record() {
        let mut board = open();
        assert_eq!(board.write_mark(FREE, 4), Ok(()));
        assert_eq!(board.write_mark(FREE, 2), Ok(()));
        assert_eq!(shown(&board, FREE), Some(vec![2, 4]));
        assert_eq!(digit_at(&board, FREE), None);

        let made = moves(&board);
        assert_eq!(made.len(), 2);
        assert_eq!((made[0].index, made[0].kind), (1, MoveKind::WriteMark));
        assert_eq!((made[0].target, made[0].digit), (FREE, Some(4)));
        assert_eq!((made[1].index, made[1].digit), (2, Some(2)));
    }

    #[test]
    fn a_struck_mark_leaves_the_note_and_joins_the_record() {
        let mut board = open();
        board.write_mark(FREE, 4).unwrap();
        board.write_mark(FREE, 2).unwrap();
        assert_eq!(board.strike_mark(FREE, 4), Ok(()));
        assert_eq!(shown(&board, FREE), Some(vec![2]));

        let made = moves(&board);
        assert_eq!(made.len(), 3);
        assert_eq!((made[2].index, made[2].kind), (3, MoveKind::StrikeMark));
        assert_eq!((made[2].target, made[2].digit), (FREE, Some(4)));
    }

    #[test]
    fn writing_a_mark_is_refused_on_a_cell_that_does_not_accept_marks() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        for position in [GIVEN, FREE] {
            let refusal = PlayError::MarksNotAccepted { position };
            assert_refused(&mut board, Op::WriteMark(position, 2), &refusal);
        }
    }

    #[test]
    fn writing_a_mark_is_refused_for_a_digit_out_of_range() {
        let mut board = open();
        for digit in [0, 10, 255] {
            let refusal = PlayError::DigitOutOfRange { digit };
            assert_refused(&mut board, Op::WriteMark(FREE, digit), &refusal);
        }
    }

    #[test]
    fn writing_a_mark_already_written_is_refused() {
        let mut board = open();
        board.write_mark(FREE, 4).unwrap();
        let refusal = PlayError::MarkAlreadyWritten {
            position: FREE,
            digit: 4,
        };
        assert_refused(&mut board, Op::WriteMark(FREE, 4), &refusal);
    }

    #[test]
    fn striking_a_mark_is_refused_on_a_cell_that_does_not_accept_marks() {
        let mut board = open();
        board.write_mark(FREE, 2).unwrap();
        board.place(FREE, 4).unwrap();
        // The 2 waits beneath the digit, and is not there to strike while it waits.
        for position in [GIVEN, FREE] {
            let refusal = PlayError::MarksNotAccepted { position };
            assert_refused(&mut board, Op::StrikeMark(position, 2), &refusal);
        }
    }

    #[test]
    fn striking_a_mark_that_is_not_there_is_refused() {
        let mut board = open();
        board.write_mark(FREE, 4).unwrap();
        for digit in [2, 0, 10, 255] {
            let refusal = PlayError::MarkNotThere {
                position: FREE,
                digit,
            };
            assert_refused(&mut board, Op::StrikeMark(FREE, digit), &refusal);
        }
    }

    #[test]
    fn a_mark_a_placed_peer_rules_out_may_still_be_written() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        // A 4 stands beside it in the row, and the given 5 heads the row.
        assert_eq!(board.write_mark(BESIDE, 4), Ok(()));
        assert_eq!(board.write_mark(BESIDE, 5), Ok(()));
        assert_eq!(shown(&board, BESIDE), Some(vec![4, 5]));
    }

    #[test]
    fn the_note_beneath_a_digit_shows_again_when_it_is_erased() {
        let mut board = open();
        board.write_mark(FREE, 2).unwrap();
        board.write_mark(FREE, 4).unwrap();
        board.place(FREE, 4).unwrap();
        assert_eq!(shown(&board, FREE), None);
        assert_eq!(picture(&board)[2], (Some(4), vec![2, 4]));

        board.erase(FREE).unwrap();
        assert_eq!(shown(&board, FREE), Some(vec![2, 4]));
    }

    /// Peers by the specification's words, with nothing of the implementation: another
    /// position that shares a row, a column or a box.
    fn are_peers(a: Position, b: Position) -> bool {
        let same_box =
            (a.row() - 1) / 3 == (b.row() - 1) / 3 && (a.column() - 1) / 3 == (b.column() - 1) / 3;
        a != b && (a.row() == b.row() || a.column() == b.column() || same_box)
    }

    #[test]
    fn a_placed_digit_is_struck_from_every_peer_that_holds_it_and_from_no_other_cell() {
        // Empty cells: one in the row of FREE, one in its column, one in its box, and
        // one that shares none of the three.
        let (in_row, in_column, in_box) = (BESIDE, Position::new(4, 3), Position::new(2, 2));
        let apart = Position::new(4, 4);
        assert!(
            [in_row, in_column, in_box]
                .iter()
                .all(|&peer| are_peers(FREE, peer))
        );
        assert!(!are_peers(FREE, apart));

        let mut board = open();
        for position in [in_row, in_column, in_box, apart] {
            board.write_mark(position, 4).unwrap();
        }
        board.write_mark(in_row, 2).unwrap();
        let before = picture(&board);

        board.place(FREE, 4).unwrap();
        assert_eq!(shown(&board, in_row), Some(vec![2]));
        assert_eq!(shown(&board, in_column), Some(vec![]));
        assert_eq!(shown(&board, in_box), Some(vec![]));
        assert_eq!(shown(&board, apart), Some(vec![4]));

        // Nothing else moved: the target and its three marked peers, and no other cell.
        let after = picture(&board);
        let changed = positions().zip(before.iter().zip(&after));
        let changed: Vec<Position> = changed
            .filter(|(_, (before, after))| before != after)
            .map(|(position, _)| position)
            .collect();
        assert_eq!(changed, [FREE, in_row, in_box, in_column]);
    }

    #[test]
    fn upkeep_reaches_a_note_hidden_beneath_a_digit() {
        let mut board = open();
        board.write_mark(BESIDE, 4).unwrap();
        board.write_mark(BESIDE, 2).unwrap();
        board.place(BESIDE, 6).unwrap();
        assert_eq!(picture(&board)[3], (Some(6), vec![2, 4]));

        board.place(FREE, 4).unwrap();
        assert_eq!(picture(&board)[3], (Some(6), vec![2]));
        board.erase(BESIDE).unwrap();
        assert_eq!(shown(&board, BESIDE), Some(vec![2]));
    }

    #[test]
    fn the_targets_own_note_is_kept_beneath_its_digit() {
        let mut board = open();
        board.write_mark(FREE, 4).unwrap();
        board.write_mark(FREE, 2).unwrap();
        board.place(FREE, 4).unwrap();
        // The placed digit's own mark is kept too: a cell is not its own peer.
        assert_eq!(picture(&board)[2], (Some(4), vec![2, 4]));
    }

    #[test]
    fn erasing_gives_nothing_back_to_the_peers() {
        let mut board = open();
        board.write_mark(BESIDE, 4).unwrap();
        board.place(FREE, 4).unwrap();
        board.erase(FREE).unwrap();
        assert_eq!(shown(&board, BESIDE), Some(vec![]));
        assert_eq!(shown(&board, FREE), Some(vec![]));
    }

    fn undone(board: &Board) -> Vec<bool> {
        board.moves().map(|made| made.is_undone()).collect()
    }

    #[test]
    fn placing_the_standing_digit_again_is_refused_and_discards_nothing() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        board.place(BESIDE, 6).unwrap();
        board.undo().unwrap();

        let refusal = PlayError::DigitAlreadyStands {
            position: FREE,
            digit: 4,
        };
        assert_refused(&mut board, Op::Place(FREE, 4), &refusal);
        // The undone move was not discarded: it is re-taken.
        assert_eq!(board.redo(), Ok(()));
        assert_eq!(digit_at(&board, BESIDE), Some(6));
    }

    #[test]
    fn undoing_a_placement_leaves_the_board_as_it_stood() {
        let (in_box, apart) = (Position::new(2, 2), Position::new(4, 4));
        let mut board = open();
        for position in [BESIDE, in_box, apart, FREE] {
            board.write_mark(position, 4).unwrap();
        }
        board.write_mark(BESIDE, 2).unwrap();
        let before = picture(&board);

        board.place(FREE, 4).unwrap();
        let asked = board.check(FREE).unwrap();
        assert_ne!(picture(&board), before);
        assert_eq!(board.undo(), Ok(()));

        // Exact: every digit and every note, the struck marks back where they were.
        assert_eq!(picture(&board), before);
        assert_eq!(shown(&board, BESIDE), Some(vec![2, 4]));
        assert_eq!(shown(&board, in_box), Some(vec![4]));
        assert_eq!(shown(&board, apart), Some(vec![4]));

        // What the undo leaves on the record: the move, undone, with its index; the
        // offer to redo it; and the check asked meanwhile.
        let made = moves(&board);
        assert_eq!(made.len(), 6);
        assert_eq!((made[5].index, made[5].kind), (6, MoveKind::Place));
        assert_eq!(undone(&board), [false, false, false, false, false, true]);
        assert!(board.can_redo());
        assert_eq!(board.checks().collect::<Vec<_>>(), [asked]);
    }

    #[test]
    fn undoing_a_placement_over_a_digit_puts_the_former_digit_back() {
        let mut board = open();
        board.place(FREE, 2).unwrap();
        board.place(FREE, 4).unwrap();
        board.undo().unwrap();
        assert_eq!(digit_at(&board, FREE), Some(2));
        board.undo().unwrap();
        assert_eq!(digit_at(&board, FREE), None);
        assert_eq!(picture(&board), picture(&open()));
    }

    #[test]
    fn undoing_an_erasure_puts_the_digit_back() {
        let mut board = open();
        board.write_mark(FREE, 2).unwrap();
        board.place(FREE, 4).unwrap();
        let before = picture(&board);
        board.erase(FREE).unwrap();
        assert_eq!(board.undo(), Ok(()));
        assert_eq!(digit_at(&board, FREE), Some(4));
        assert_eq!(picture(&board), before);
    }

    #[test]
    fn undoing_a_written_mark_strikes_it_and_undoing_a_struck_mark_writes_it() {
        let mut board = open();
        board.write_mark(FREE, 2).unwrap();
        board.write_mark(FREE, 4).unwrap();
        board.strike_mark(FREE, 2).unwrap();
        assert_eq!(shown(&board, FREE), Some(vec![4]));

        board.undo().unwrap();
        assert_eq!(shown(&board, FREE), Some(vec![2, 4]));
        board.undo().unwrap();
        assert_eq!(shown(&board, FREE), Some(vec![2]));
        // Recorded and re-taken exactly as a placement is.
        assert_eq!(undone(&board), [false, true, true]);
        board.redo().unwrap();
        board.redo().unwrap();
        assert_eq!(shown(&board, FREE), Some(vec![4]));
        assert_eq!(undone(&board), [false, false, false]);
    }

    /// A script that uses every kind of move, upkeep and a placement over a digit.
    const EVERY_KIND: [Op; 7] = [
        Op::WriteMark(BESIDE, 4),
        Op::WriteMark(BESIDE, 7),
        Op::Place(FREE, 2),
        Op::Place(FREE, 4),
        Op::StrikeMark(BESIDE, 7),
        Op::Erase(FREE),
        Op::WriteMark(FREE, 9),
    ];

    /// Plays `script` on a new board, with the picture before the first move and after
    /// each one.
    fn played(script: &[Op]) -> (Board, Vec<Picture>) {
        let mut board = open();
        let mut pictures = vec![picture(&board)];
        for op in script {
            op.ask(&mut board).unwrap();
            pictures.push(picture(&board));
        }
        (board, pictures)
    }

    #[test]
    fn redo_retakes_each_kind_of_move_exactly() {
        let (mut board, pictures) = played(&EVERY_KIND);
        for before in pictures.iter().rev().skip(1) {
            assert_eq!(board.undo(), Ok(()));
            assert_eq!(&picture(&board), before);
        }
        assert_eq!(undone(&board), [true; 7]);
        for after in pictures.iter().skip(1) {
            assert_eq!(board.redo(), Ok(()));
            assert_eq!(&picture(&board), after);
        }
        assert_eq!(undone(&board), [false; 7]);
    }

    #[test]
    fn undo_takes_back_the_latest_standing_move_and_redo_retakes_the_earliest_undone() {
        let (mut board, _) = played(&EVERY_KIND[..4]);
        board.undo().unwrap();
        assert_eq!(undone(&board), [false, false, false, true]);
        board.undo().unwrap();
        board.undo().unwrap();
        assert_eq!(undone(&board), [false, true, true, true]);
        // What is re-taken comes right after the standing moves: move 2 of the 4.
        board.redo().unwrap();
        assert_eq!(undone(&board), [false, false, true, true]);
        let indices: Vec<usize> = board.moves().map(|made| made.index()).collect();
        assert_eq!(indices, [1, 2, 3, 4]);
    }

    #[test]
    fn undo_is_refused_with_nothing_standing_and_redo_with_nothing_undone() {
        let mut board = open();
        assert_refused(&mut board, Op::Undo, &PlayError::NothingToUndo);
        assert_refused(&mut board, Op::Redo, &PlayError::NothingToRedo);

        board.place(FREE, 4).unwrap();
        assert_refused(&mut board, Op::Redo, &PlayError::NothingToRedo);
        board.undo().unwrap();
        assert_refused(&mut board, Op::Undo, &PlayError::NothingToUndo);
    }

    #[test]
    fn a_new_move_discards_every_undone_move() {
        let (mut board, _) = played(&EVERY_KIND[..4]);
        board.undo().unwrap();
        board.undo().unwrap();
        assert_eq!(board.write_mark(FREE, 7), Ok(()));

        let made = moves(&board);
        assert_eq!(made.len(), 3);
        assert_eq!((made[2].index, made[2].kind), (3, MoveKind::WriteMark));
        assert_eq!((made[2].target, made[2].digit), (FREE, Some(7)));
        assert_eq!(undone(&board), [false, false, false]);
        // They are gone from the record and cannot be re-taken.
        assert!(!board.can_redo());
        assert_refused(&mut board, Op::Redo, &PlayError::NothingToRedo);
        assert_eq!(digit_at(&board, FREE), None);
    }

    #[test]
    fn can_undo_and_can_redo_follow_the_moves() {
        let mut board = open();
        assert_eq!((board.can_undo(), board.can_redo()), (false, false));
        board.place(FREE, 4).unwrap();
        assert_eq!((board.can_undo(), board.can_redo()), (true, false));
        board.place(BESIDE, 6).unwrap();
        board.undo().unwrap();
        assert_eq!((board.can_undo(), board.can_redo()), (true, true));
        board.undo().unwrap();
        assert_eq!((board.can_undo(), board.can_redo()), (false, true));
        board.redo().unwrap();
        board.redo().unwrap();
        assert_eq!((board.can_undo(), board.can_redo()), (true, false));
    }

    #[test]
    fn a_check_answers_yes_for_the_solutions_digit_and_no_for_another() {
        let solution = *solve(givens(FIXTURE)).unwrap().solution();
        let mut board = open();
        for (position, &right) in positions().zip(solution.iter().flatten()) {
            if board.cell(position).unwrap().is_given() {
                continue;
            }
            let wrong = right % 9 + 1;
            board.place(position, wrong).unwrap();
            assert!(!board.check(position).unwrap().is_right());
            board.place(position, right).unwrap();
            assert!(board.check(position).unwrap().is_right());
            board.erase(position).unwrap();
        }
    }

    #[test]
    fn a_check_is_refused_on_an_empty_cell_and_on_a_given() {
        let mut board = open();
        for position in [FREE, GIVEN] {
            let refusal = PlayError::NoPlayersDigit { position };
            assert_refused(&mut board, Op::Check(position), &refusal);
        }
        // A mark is not a digit.
        board.write_mark(FREE, 4).unwrap();
        let refusal = PlayError::NoPlayersDigit { position: FREE };
        assert_refused(&mut board, Op::Check(FREE), &refusal);
    }

    #[test]
    fn a_check_records_the_digit_as_it_stood_and_how_many_moves_stood() {
        let mut board = open();
        board.write_mark(BESIDE, 6).unwrap();
        board.place(FREE, 2).unwrap();
        let first = board.check(FREE).unwrap();
        assert_eq!((first.index(), first.after_move()), (1, 2));
        assert_eq!(
            (first.target(), first.digit(), first.is_right()),
            (FREE, 2, false)
        );

        // The cell changes, and the first check still says what it was asked about.
        board.place(FREE, 4).unwrap();
        let second = board.check(FREE).unwrap();
        assert_eq!((second.index(), second.after_move()), (2, 3));
        assert_eq!(
            (second.target(), second.digit(), second.is_right()),
            (FREE, 4, true)
        );
        assert_eq!(board.checks().collect::<Vec<_>>(), [first, second]);
    }

    #[test]
    fn a_check_is_not_a_move_and_undo_does_not_remove_it() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        let before = (picture(&board), moves(&board));
        let asked = board.check(FREE).unwrap();
        assert_eq!((picture(&board), moves(&board)), before);
        assert!(!board.can_redo());

        board.undo().unwrap();
        assert_eq!(board.checks().collect::<Vec<_>>(), [asked]);
        board.redo().unwrap();
        board.erase(FREE).unwrap();
        assert_eq!(board.checks().collect::<Vec<_>>(), [asked]);
    }

    #[test]
    fn after_move_is_a_count_and_not_a_reference() {
        let mut board = open();
        board.place(FREE, 4).unwrap();
        board.place(BESIDE, 6).unwrap();
        let asked = board.check(BESIDE).unwrap();
        assert_eq!(asked.after_move(), 2);

        // The second move is taken back and another takes its index.
        board.undo().unwrap();
        board.write_mark(BESIDE, 1).unwrap();
        assert_eq!(moves(&board)[1].kind, MoveKind::WriteMark);
        assert_eq!(board.checks().collect::<Vec<_>>(), [asked]);
        assert_eq!(board.checks().next().unwrap().after_move(), 2);
    }

    #[test]
    fn a_check_never_tells_the_digit() {
        let mut board = open();
        board.place(FREE, 2).unwrap();
        let asked = board.check(FREE).unwrap();
        // The digit a check holds is the player's own, and its answer is a no.
        assert_eq!((asked.digit(), asked.is_right()), (2, false));
        let shown = format!("{asked:?}");
        assert!(!shown.contains('4'), "{shown}");
    }

    #[test]
    fn a_board_does_not_print_its_solution() {
        let solution = *solve(givens(FIXTURE)).unwrap().solution();
        let mut board = open();
        board.place(FREE, 2).unwrap();
        board.check(FREE).unwrap();
        let shown = format!("{board:?}");
        for row in solution {
            let row: String = row.iter().map(|&digit| char::from(b'0' + digit)).collect();
            assert!(!shown.contains(&row), "{row} is in {shown}");
        }
    }

    /// The fixture's one solution, as the solver finds it.
    fn solution() -> Vec<(Position, u8)> {
        let proof = solve(givens(FIXTURE)).unwrap();
        let digits = proof.solution().iter().flatten().copied();
        positions().zip(digits).collect()
    }

    /// The fixture played to its end: every cell the setter left, filled with the
    /// solution's digit, in grid order.
    fn solved() -> Board {
        let mut board = open();
        for (position, digit) in solution() {
            if !board.cell(position).unwrap().is_given() {
                board.place(position, digit).unwrap();
            }
        }
        board
    }

    /// Each of the five operations on a cell, asked of the cell at `position`.
    fn on_a_cell(position: Position) -> [Op; 5] {
        [
            Op::Place(position, 4),
            Op::Erase(position),
            Op::WriteMark(position, 4),
            Op::StrikeMark(position, 4),
            Op::Check(position),
        ]
    }

    #[test]
    fn a_position_off_the_grid_is_refused_by_every_operation() {
        let mut board = open();
        for position in OFF_GRID {
            let refusal = PlayError::NoSuchCell { position };
            for op in on_a_cell(position) {
                assert_refused(&mut board, op, &refusal);
            }
        }
    }

    #[test]
    fn refusals_come_in_a_fixed_order() {
        // Off the grid comes before solved, and solved before every clause of a cell.
        let mut board = solved();
        let off_grid = OFF_GRID[0];
        for op in on_a_cell(off_grid) {
            let refusal = PlayError::NoSuchCell { position: off_grid };
            assert_refused(&mut board, op, &refusal);
        }
        for op in on_a_cell(GIVEN).into_iter().chain(on_a_cell(FREE)) {
            assert_refused(&mut board, op, &PlayError::Solved);
        }
        assert_refused(&mut board, Op::Place(FREE, 0), &PlayError::Solved);

        // Then the rule's own clauses, in the order `board.allium` writes them.
        let mut board = open();
        board.place(FREE, 4).unwrap();
        let given = PlayError::GivenCell { position: GIVEN };
        assert_refused(&mut board, Op::Place(GIVEN, 0), &given);
        let no_marks = PlayError::MarksNotAccepted { position: FREE };
        assert_refused(&mut board, Op::WriteMark(FREE, 0), &no_marks);
        assert_refused(&mut board, Op::StrikeMark(FREE, 0), &no_marks);
    }

    #[test]
    fn every_move_reads_back_the_board_as_it_stood_after_it() {
        let (mut board, pictures) = played(&EVERY_KIND);
        board.undo().unwrap();
        board.undo().unwrap();
        board.undo().unwrap();
        assert_eq!(
            undone(&board),
            [false, false, false, false, true, true, true]
        );

        // Standing or undone, each move reads the picture taken when it was made.
        let readings: Vec<Picture> = board.moves().map(|made| reading(&made)).collect();
        assert_eq!(readings, pictures[1..]);
        // The readings are not all one picture: the script changed the board each time.
        assert!(pictures.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn reading_back_takes_nothing_back() {
        let (mut board, _) = played(&EVERY_KIND);
        board.undo().unwrap();
        board.check(FREE).unwrap_err();
        let before = view(&board);
        for made in board.moves() {
            for position in positions() {
                let _ = (made.digit_after(position), made.note_after(position));
            }
        }
        assert_eq!(view(&board), before);
    }

    #[test]
    fn the_reading_at_the_latest_standing_move_is_the_board() {
        let (mut board, _) = played(&EVERY_KIND);
        for _ in 0..3 {
            let standing = board.moves().filter(|made| !made.is_undone());
            let latest = standing.last().unwrap();
            let digits = |cell: BoardCell| latest.digit_after(cell.position()) == cell.digit();
            assert!(board.cells().all(digits));
            let shown = board
                .cells()
                .filter_map(|cell| Some((cell.position(), cell.note()?)));
            let notes: Vec<(Position, Note)> = shown.collect();
            assert!(!notes.is_empty());
            assert!(
                notes
                    .iter()
                    .all(|&(at, note)| latest.note_after(at) == note)
            );
            board.undo().unwrap();
        }
    }

    #[test]
    fn a_reading_off_the_grid_is_empty() {
        let (board, _) = played(&EVERY_KIND);
        for made in board.moves() {
            for position in OFF_GRID {
                assert_eq!(made.digit_after(position), None);
                assert!(made.note_after(position).is_empty());
            }
        }
    }

    /// A cell's three derived facts: whether it holds a digit of the player's, whether
    /// it accepts marks and whether it shows its note.
    fn facts(board: &Board, position: Position) -> (bool, bool, bool) {
        let cell = board.cell(position).unwrap();
        let facts = (cell.holds_players_digit(), cell.accepts_marks());
        (facts.0, facts.1, cell.shows_note())
    }

    #[test]
    fn a_cells_facts_follow_its_digit_and_whether_it_is_given() {
        let mut board = open();
        assert_eq!(facts(&board, GIVEN), (false, false, false));
        assert_eq!(facts(&board, FREE), (false, true, true));
        board.place(FREE, 4).unwrap();
        assert_eq!(facts(&board, FREE), (true, false, false));
        board.erase(FREE).unwrap();
        assert_eq!(facts(&board, FREE), (false, true, true));
    }

    #[test]
    fn a_note_is_shown_only_while_its_cell_is_empty() {
        let (board, _) = played(&EVERY_KIND[..4]);
        assert_eq!(digit_at(&board, FREE), Some(4));
        for cell in board.cells() {
            assert_eq!(cell.note().is_some(), cell.digit().is_none());
            assert_eq!(cell.shows_note(), cell.digit().is_none());
        }
        assert_eq!(shown(&board, FREE), None);
        assert_eq!(shown(&board, GIVEN), None);
        assert_eq!(shown(&board, BESIDE), Some(vec![7]));
    }

    #[test]
    fn an_offered_operation_may_still_be_refused() {
        let mut board = open();
        board.write_mark(FREE, 4).unwrap();
        board.place(BESIDE, 6).unwrap();
        assert_eq!(board.status(), Status::Unsolved);

        // `Place` is offered on any cell that is not given.
        assert!(!board.cell(BESIDE).unwrap().is_given());
        let out_of_range = PlayError::DigitOutOfRange { digit: 10 };
        assert_refused(&mut board, Op::Place(BESIDE, 10), &out_of_range);
        let stands = PlayError::DigitAlreadyStands {
            position: BESIDE,
            digit: 6,
        };
        assert_refused(&mut board, Op::Place(BESIDE, 6), &stands);

        // `WriteMark` is offered on any cell that accepts marks.
        assert!(board.cell(FREE).unwrap().accepts_marks());
        assert_refused(&mut board, Op::WriteMark(FREE, 10), &out_of_range);
        let written = PlayError::MarkAlreadyWritten {
            position: FREE,
            digit: 4,
        };
        assert_refused(&mut board, Op::WriteMark(FREE, 4), &written);

        // `StrikeMark` is offered on such a cell once its note holds a mark.
        assert!(
            board
                .cell(FREE)
                .unwrap()
                .note()
                .is_some_and(|note| !note.is_empty())
        );
        let not_there = PlayError::MarkNotThere {
            position: FREE,
            digit: 2,
        };
        assert_refused(&mut board, Op::StrikeMark(FREE, 2), &not_there);
    }

    #[test]
    fn two_boards_never_share_a_puzzle() {
        // A board owns its puzzle, so the invariant holds by ownership: no second board
        // can reach it. A clone has a puzzle of its own, and so has a board opened on
        // the same givens.
        let (first, _) = played(&EVERY_KIND[..4]);
        let before = view(&first);
        let mut second = first.clone();
        second.undo().unwrap();
        second.place(BESIDE, 6).unwrap();
        second.check(BESIDE).unwrap();
        assert_eq!(view(&first), before);
        assert_ne!(view(&second), before);

        let mut third = open();
        third.place(FREE, 9).unwrap();
        assert_eq!(view(&open()), view(&open()));
        assert_ne!(view(&third), view(&open()));
    }
}
