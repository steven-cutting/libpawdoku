//! Play from a record, from outside the crate: `board.allium`'s `PlayingFromARecord`
//! held to a board that is kept. Every scripted game of `tests/board.rs` and
//! `tests/snapshots.rs` is played both ways, step by step, and a property plays
//! arbitrary scripts both ways, refused operations included. The refusals, what a view
//! never shows, and that nothing is kept between calls are here too.
//!
//! Under the `serde` feature the wire shape is read field by field. The library brings
//! no format, so a small serializer here captures what a value hands to serde as a
//! tree of plain values, and a deserializer reads that tree back for the two values
//! that deserialise, `Record` and `Operation`.
//!
//! The scripts are restated here, each naming the file it comes from: an integration
//! test file cannot reach another's helpers.

#[cfg(test)]
mod tests {
    use pawdoku::board::{
        Board, BoardView, Check, FromRecordError, Operation, PlayError, ReadingAfter, Record, open,
        play, read, reading_after,
    };
    use pawdoku::solver::solve;
    use pawdoku::sudoku::{Given, Position};
    use proptest::prelude::*;

    /// The rules' fixture: thirty givens, one solution, rows top to bottom.
    const FIXTURE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// A cell the fixture leaves to the player. Its solution digit is 4.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`].
    const BESIDE: Position = Position::new(1, 4);

    /// Two more cells left to the player: below the given 3 of row 1, whose solution
    /// digit is 7, and below the given 5 of row 1, in its box.
    const BELOW: Position = Position::new(2, 2);
    const CORNER: Position = Position::new(3, 1);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// The fixture's givens.
    fn givens() -> Vec<Given> {
        let cells = FIXTURE.bytes().zip(0_u8..);
        cells
            .filter(|(cell, _)| cell.is_ascii_digit())
            .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
            .collect()
    }

    /// The cells the fixture leaves empty, in grid order, each with its solution digit.
    fn the_rest() -> Vec<(Position, u8)> {
        let proof = solve(givens()).unwrap();
        let digits = proof.solution().iter().flatten().copied();
        let positions = (1..=9).flat_map(|row| (1..=9).map(move |column| (row, column)));
        let cells = positions.zip(digits);
        let given = |at: Position| givens().iter().any(|given| given.position() == at);
        cells
            .map(|((row, column), digit)| (Position::new(row, column), digit))
            .filter(|&(at, _)| !given(at))
            .collect()
    }

    /// One of `Playing`'s seven operations, as a script holds it: the same thing asked
    /// of a held board and handed to `play`. A test cannot match `Operation`, which is
    /// `#[non_exhaustive]`, without a wildcard arm, so a script uses this.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Op {
        Place(Position, u8),
        Erase(Position),
        WriteMark(Position, u8),
        StrikeMark(Position, u8),
        CheckCell(Position),
        Undo,
        Redo,
    }

    impl Op {
        /// The operation handed to `play`.
        const fn operation(self) -> Operation {
            match self {
                Self::Place(position, digit) => Operation::Place { position, digit },
                Self::Erase(position) => Operation::Erase { position },
                Self::WriteMark(position, digit) => Operation::WriteMark { position, digit },
                Self::StrikeMark(position, digit) => Operation::StrikeMark { position, digit },
                Self::CheckCell(position) => Operation::CheckCell { position },
                Self::Undo => Operation::Undo,
                Self::Redo => Operation::Redo,
            }
        }

        /// Asks it of a held board through the board's own method, with the check it
        /// added when it is a check.
        fn ask(self, board: &mut Board) -> Result<Option<Check>, PlayError> {
            match self {
                Self::Place(position, digit) => board.place(position, digit).map(|()| None),
                Self::Erase(position) => board.erase(position).map(|()| None),
                Self::WriteMark(position, digit) => {
                    board.write_mark(position, digit).map(|()| None)
                }
                Self::StrikeMark(position, digit) => {
                    board.strike_mark(position, digit).map(|()| None)
                }
                Self::CheckCell(position) => board.check(position).map(Some),
                Self::Undo => board.undo().map(|()| None),
                Self::Redo => board.redo().map(|()| None),
            }
        }
    }

    /// `THE_SCRIPTED_GAME` of `tests/snapshots.rs`: every kind of move, upkeep striking
    /// a peer's mark, an undo, a redo, a move that discards an undone one, and a check
    /// answered each way.
    const THE_SCRIPTED_GAME: [Op; 18] = [
        Op::WriteMark(BESIDE, 4),
        Op::WriteMark(BESIDE, 6),
        Op::WriteMark(BESIDE, 9),
        Op::StrikeMark(BESIDE, 9),
        Op::WriteMark(FREE, 2),
        Op::Place(FREE, 4),
        Op::CheckCell(FREE),
        Op::Place(BELOW, 3),
        Op::CheckCell(BELOW),
        Op::Erase(BELOW),
        Op::Place(CORNER, 5),
        Op::WriteMark(BELOW, 7),
        Op::Undo,
        Op::Redo,
        Op::Undo,
        Op::WriteMark(BELOW, 1),
        Op::Place(BELOW, 7),
        Op::Undo,
    ];

    /// `every_kind_of_move` of `tests/board.rs`: a note waiting beneath a digit and a
    /// note upkeep has struck from.
    const EVERY_KIND_OF_MOVE: [Op; 8] = [
        Op::WriteMark(BESIDE, 4),
        Op::WriteMark(BESIDE, 7),
        Op::WriteMark(FREE, 2),
        Op::Place(FREE, 4),
        Op::StrikeMark(BESIDE, 7),
        Op::Place(FREE, 2),
        Op::Erase(FREE),
        Op::Place(FREE, 4),
    ];

    /// `in_play` of `tests/board.rs`: four moves, a check, and the last move undone.
    const IN_PLAY: [Op; 6] = [
        Op::WriteMark(BESIDE, 4),
        Op::WriteMark(BESIDE, 7),
        Op::Place(FREE, 2),
        Op::Place(FREE, 4),
        Op::CheckCell(FREE),
        Op::Undo,
    ];

    /// The whole game of `tests/board.rs`, and `played_to_the_end` of
    /// `tests/snapshots.rs`: every empty cell given its solution digit, in grid order.
    fn the_whole_game() -> Vec<Op> {
        let rest = the_rest().into_iter();
        rest.map(|(at, digit)| Op::Place(at, digit)).collect()
    }

    /// Every scripted game.
    fn the_scripted_games() -> [Vec<Op>; 4] {
        [
            THE_SCRIPTED_GAME.to_vec(),
            EVERY_KIND_OF_MOVE.to_vec(),
            IN_PLAY.to_vec(),
            the_whole_game(),
        ]
    }

    /// Asks `op` both ways, of the held `board` and of `record` through `play`, and
    /// holds them to the same answer: the same record, view and check when it is
    /// accepted, and the same refusal, wrapped, when it is not. Hands back the record
    /// the next call takes, and whether the operation was accepted.
    fn both_ways(board: &mut Board, record: &Record, op: Op) -> (Record, bool) {
        let kept = record.clone();
        let held = op.ask(board);
        let played = play(record, op.operation());
        assert_eq!(record, &kept, "{op:?} changed the record handed over");
        match (held, played) {
            (Ok(check), Ok(played)) => {
                assert_eq!(played.record(), &board.write(), "{op:?}");
                assert_eq!(played.view(), &BoardView::of(board), "{op:?}");
                assert_eq!(played.check(), check.as_ref(), "{op:?}");
                (played.record().clone(), true)
            }
            (held, played) => {
                let held = held.map(drop).map_err(FromRecordError::Play);
                assert_eq!(played.map(drop), held, "{op:?}");
                (kept, false)
            }
        }
    }

    /// Every move's reading, both ways, and the index past the last refused both ways.
    fn the_readings_agree(board: &Board, record: &Record) {
        let last = board.moves().count();
        for move_index in 0..=last + 1 {
            assert_eq!(
                reading_after(record, move_index),
                ReadingAfter::of(board, move_index),
                "after move {move_index}"
            );
        }
    }

    // The two ways ---------------------------------------------------------------

    /// `RecordsKeepStepWithAHeldBoard` and `TheSameBoardFromEitherWay`, for every
    /// scripted game: equal records and views at every step, an equal reading after
    /// the latest move at every step, and after every move at the end.
    #[test]
    fn every_scripted_game_is_the_same_game_either_way() {
        for script in the_scripted_games() {
            let mut board = Board::open(givens()).unwrap();
            let opened = open(givens()).unwrap();
            assert_eq!(opened.record(), &board.write());
            assert_eq!(opened.view(), &BoardView::of(&board));
            let mut record = opened.record().clone();
            for op in script {
                let (after, accepted) = both_ways(&mut board, &record, op);
                assert!(accepted, "{op:?} is part of a scripted game");
                record = after;
                assert_eq!(read(&record), Ok(BoardView::of(&board)));
                let latest = board.moves().count();
                assert_eq!(
                    reading_after(&record, latest),
                    ReadingAfter::of(&board, latest)
                );
            }
            the_readings_agree(&board, &record);
        }
    }

    // The refusals ---------------------------------------------------------------

    /// A board opened on the fixture with the whole game played on it: solved.
    fn solved() -> Board {
        let mut board = Board::open(givens()).unwrap();
        for op in the_whole_game() {
            op.ask(&mut board).unwrap();
        }
        board
    }

    /// `ARefusalHandsBackNothingElse` for the guard: every operation a fresh board and
    /// a solved one refuse is refused by `play` with the board's own `PlayError`, and
    /// the record handed over is unchanged.
    #[test]
    fn an_operation_the_board_refuses_is_refused_with_its_play_error() {
        let refused = [
            Op::Place(GIVEN, 6),
            Op::Place(Position::new(0, 10), 4),
            Op::Place(FREE, 10),
            Op::Erase(FREE),
            Op::WriteMark(GIVEN, 4),
            Op::StrikeMark(FREE, 4),
            Op::CheckCell(FREE),
            Op::Undo,
            Op::Redo,
        ];
        for mut board in [Board::open(givens()).unwrap(), solved()] {
            let record = board.write();
            for op in refused {
                let (after, accepted) = both_ways(&mut board, &record, op);
                assert!(!accepted, "{op:?} is refused");
                assert_eq!(after, record);
            }
        }
        // A solved board refuses even what a fresh one offers.
        let record = solved().write();
        let refusal = play(&record, Op::Place(FREE, 4).operation()).unwrap_err();
        assert_eq!(refusal, FromRecordError::Play(PlayError::Solved));
    }

    /// A move index is counted from 1 and names a move the record holds, undone ones
    /// included: 0, one past the last and the largest index are refused, with the
    /// record unchanged.
    #[test]
    fn an_index_that_names_no_move_is_refused() {
        let mut board = Board::open(givens()).unwrap();
        for op in IN_PLAY {
            op.ask(&mut board).unwrap();
        }
        let record = board.write();
        let kept = record.clone();
        assert!(reading_after(&record, 4).is_ok(), "the undone fourth move");
        for move_index in [0, 5, usize::MAX] {
            let refusal = FromRecordError::NoSuchMove { move_index };
            assert_eq!(reading_after(&record, move_index), Err(refusal.clone()));
            assert_eq!(ReadingAfter::of(&board, move_index), Err(refusal));
        }
        assert_eq!(record, kept);
    }

    // What is shown ---------------------------------------------------------------

    /// `TheSolutionIsNeverShown` and `ACheckNeverTellsTheDigit`: a view's digits are
    /// the board's, so an empty cell holds none, and a wrong digit checked shows only
    /// the player's digit and a no.
    #[test]
    fn a_view_shows_no_digit_the_player_has_not_placed() {
        let opened = open(givens()).unwrap();
        let view = opened.view();
        for cell in view.cells() {
            assert_eq!(cell.digit().is_some(), cell.is_given(), "{cell:?}");
        }

        let placed = play(opened.record(), Op::Place(FREE, 9).operation()).unwrap();
        let checked = play(placed.record(), Op::CheckCell(FREE).operation()).unwrap();
        let check = checked.check().unwrap();
        assert_eq!((check.digit(), check.is_right()), (9, false));
        assert_eq!(checked.view().checks(), [*check]);
        // The cell shows the player's 9 and nothing of the solution's 4.
        let free = checked.view().cells()[2];
        assert_eq!((free.position(), free.digit()), (FREE, Some(9)));
        let reading = reading_after(checked.record(), 1).unwrap();
        assert_eq!(reading.cells()[2].digit(), Some(9));
    }

    /// `NothingIsKeptBetweenCalls`: each call asked again, with other calls between,
    /// hands back an equal value.
    #[test]
    fn nothing_is_kept_between_calls() {
        let opened = open(givens()).unwrap();
        let place = Op::Place(FREE, 4).operation();
        let first = play(opened.record(), place).unwrap();
        let first_read = read(first.record()).unwrap();
        let first_reading = reading_after(first.record(), 1).unwrap();

        // Other calls between, on this record and on others.
        let marked = play(first.record(), Op::WriteMark(BESIDE, 6).operation()).unwrap();
        let _ = play(marked.record(), Op::Undo.operation()).unwrap();
        let _ = play(opened.record(), Op::Redo.operation()).unwrap_err();
        let _ = open([]).unwrap_err();

        assert_eq!(open(givens()).unwrap(), opened);
        assert_eq!(play(opened.record(), place).unwrap(), first);
        assert_eq!(read(first.record()).unwrap(), first_read);
        assert_eq!(reading_after(first.record(), 1).unwrap(), first_reading);
    }

    // The property -----------------------------------------------------------------

    /// A position for a script, as `tests/board.rs` draws one: mostly in the first two
    /// rows, some in the last, a few anywhere, off the grid included.
    fn position() -> impl Strategy<Value = Position> {
        let cell = |(row, column)| Position::new(row, column);
        prop_oneof![
            5 => (1..=2_u8, 1..=9_u8).prop_map(cell),
            3 => (9..=9_u8, 1..=9_u8).prop_map(cell),
            1 => (1..=9_u8, 1..=9_u8).prop_map(cell),
            1 => (0..=10_u8, 0..=10_u8).prop_map(cell),
        ]
    }

    /// A digit for a script: mostly one of four, sometimes any, now and then out of
    /// range.
    fn digit() -> impl Strategy<Value = u8> {
        prop_oneof![6 => 1..=4_u8, 2 => 1..=9_u8, 1 => 0..=11_u8]
    }

    /// Any of the seven operations, refused ones included.
    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            4 => (position(), digit()).prop_map(|(at, digit)| Op::Place(at, digit)),
            2 => position().prop_map(Op::Erase),
            4 => (position(), digit()).prop_map(|(at, digit)| Op::WriteMark(at, digit)),
            2 => (position(), digit()).prop_map(|(at, digit)| Op::StrikeMark(at, digit)),
            2 => position().prop_map(Op::CheckCell),
            3 => Just(Op::Undo),
            2 => Just(Op::Redo),
        ]
    }

    /// How many of the fixture's empty cells a script finds filled with the solution's
    /// digits: none, or nearly all, so that some scripts reach a solved board.
    fn filled() -> impl Strategy<Value = usize> {
        prop_oneof![2 => Just(0_usize), 1 => 48..=51_usize]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// `TheSameBoardFromEitherWay`: any script, from the fixture with some of its
        /// cells filled, played on a held board and from records, gives equal records
        /// and views after every operation, the same refusal for every refused one,
        /// and equal readings after every move at the end.
        #[test]
        fn any_script_is_the_same_either_way(
            filled in filled(),
            script in prop::collection::vec(op(), 0..40),
        ) {
            let head = the_whole_game().into_iter().take(filled);
            let mut board = Board::open(givens()).unwrap();
            let mut record = open(givens()).unwrap().record().clone();
            for op in head.chain(script) {
                record = both_ways(&mut board, &record, op).0;
            }
            prop_assert_eq!(&record, &board.write());
            the_readings_agree(&board, &record);
        }
    }

    // The wire shape -------------------------------------------------------------

    #[cfg(feature = "serde")]
    use super::wire::{Value, capture, restore};

    /// The record of `THE_SCRIPTED_GAME` and the board it leaves.
    #[cfg(feature = "serde")]
    fn the_scripted_board() -> Board {
        let mut board = Board::open(givens()).unwrap();
        for op in THE_SCRIPTED_GAME {
            op.ask(&mut board).unwrap();
        }
        board
    }

    /// Each of the seven operations, as a caller hands it over.
    #[cfg(feature = "serde")]
    const EVERY_OPERATION: [Op; 7] = [
        Op::Place(FREE, 4),
        Op::Erase(FREE),
        Op::WriteMark(FREE, 2),
        Op::StrikeMark(FREE, 2),
        Op::CheckCell(FREE),
        Op::Undo,
        Op::Redo,
    ];

    /// `Operation` is tagged by `kind`, carries `position` and `digit` where it takes
    /// them and nothing else, and reads back as it was written. A kind no operation has
    /// is refused.
    #[cfg(feature = "serde")]
    #[test]
    fn an_operation_round_trips_tagged_by_its_kind() {
        let shapes: [(&str, &[&str]); 7] = [
            ("Place", &["kind", "position", "digit"]),
            ("Erase", &["kind", "position"]),
            ("WriteMark", &["kind", "position", "digit"]),
            ("StrikeMark", &["kind", "position", "digit"]),
            ("CheckCell", &["kind", "position"]),
            ("Undo", &["kind"]),
            ("Redo", &["kind"]),
        ];
        for (op, (kind, keys)) in EVERY_OPERATION.into_iter().zip(shapes) {
            let operation = op.operation();
            let wire = capture(&operation);
            assert_eq!(wire.keys(), keys);
            assert_eq!(wire.get("kind"), &Value::Text(kind.to_owned()));
            if keys.contains(&"position") {
                assert_eq!(wire.get("position").keys(), ["row", "column"]);
            }
            assert_eq!(restore::<Operation>(wire), Ok(operation));
        }
        let unknown = Value::Object(vec![("kind".to_owned(), Value::Text("Hint".to_owned()))]);
        assert!(restore::<Operation>(unknown).is_err());
    }

    /// A record reads back as it was written, and its shape is the one the record's
    /// snapshot shows: casing did not move it.
    #[cfg(feature = "serde")]
    #[test]
    fn a_record_round_trips() {
        let record = the_scripted_board().write();
        let wire = capture(&record);
        assert_eq!(
            wire.keys(),
            ["version", "givens", "moves", "undone", "checks"]
        );
        assert_eq!(wire.get("moves").items()[0].keys(), ["WriteMark"]);
        assert_eq!(restore::<Record>(wire), Ok(record));
    }

    /// The view of the scripted board, captured, and the board.
    #[cfg(feature = "serde")]
    fn the_scripted_view() -> (Value, Board) {
        let board = the_scripted_board();
        (capture(&BoardView::of(&board)), board)
    }

    /// A view serialises to exactly what `Playing` exposes, in order, under the
    /// specification's names, with enum values in `PascalCase`; a cell is flat.
    #[cfg(feature = "serde")]
    #[test]
    fn a_view_and_its_cells_serialise_to_their_fields() {
        let (wire, board) = the_scripted_view();
        let view = [
            "status",
            "is_full",
            "is_consistent",
            "can_undo",
            "can_redo",
            "cells",
            "moves",
            "checks",
        ];
        assert_eq!(wire.keys(), view);
        assert_eq!(wire.get("status"), &Value::Text("Unsolved".to_owned()));
        assert_eq!(wire.get("can_redo"), &Value::Bool(true));

        // A cell is flat; its note is there only while it is shown, and is a list.
        let cells = wire.get("cells").items();
        assert_eq!(cells.len(), 81);
        let cell = |at: Position| &cells[usize::from((at.row() - 1) * 9 + at.column() - 1)];
        let shown = [
            "row",
            "column",
            "digit",
            "is_given",
            "is_conflicting",
            "note",
        ];
        assert_eq!(cell(BELOW).keys(), shown);
        assert_eq!(cell(BELOW).get("digit"), &Value::Null);
        assert_eq!(
            cell(BELOW).get("note"),
            &Value::List(vec![Value::Number(1)])
        );
        assert_eq!(cell(FREE).keys(), shown[..5]);
        assert_eq!(cell(FREE).get("digit"), &Value::Number(4));
        assert_eq!(cell(CORNER).get("is_conflicting"), &Value::Bool(true));
        for (wire, cell) in cells.iter().zip(board.cells()) {
            let digit = cell
                .digit()
                .map_or(Value::Null, |d| Value::Number(d.into()));
            assert_eq!(wire.get("digit"), &digit, "TheSolutionIsNeverShown");
        }
    }

    /// A move is its index, kind, target, digit and whether it is undone, never its
    /// reading; an erasure's digit is null.
    #[cfg(feature = "serde")]
    #[test]
    fn a_move_serialises_without_its_reading() {
        let (wire, board) = the_scripted_view();
        let moves = wire.get("moves").items();
        assert_eq!(moves.len(), board.moves().count());
        for made in moves {
            let fields = ["index", "kind", "target", "digit", "is_undone"];
            assert_eq!(made.keys(), fields);
        }
        let kinds: Vec<&Value> = moves.iter().map(|made| made.get("kind")).collect();
        for kind in ["Place", "Erase", "WriteMark", "StrikeMark"] {
            assert!(kinds.contains(&&Value::Text(kind.to_owned())));
        }
        let erased = moves
            .iter()
            .find(|made| made.get("kind") == &Value::Text("Erase".to_owned()));
        assert_eq!(erased.unwrap().get("digit"), &Value::Null);
    }

    /// A check is its index, the moves that stood, its target, the player's digit and
    /// the answer: nothing of the solution's digit.
    #[cfg(feature = "serde")]
    #[test]
    fn a_check_serialises_with_the_players_digit_alone() {
        let (wire, _) = the_scripted_view();
        let checks = wire.get("checks").items();
        assert_eq!(checks.len(), 2);
        let fields = ["index", "after_move", "target", "digit", "is_right"];
        for check in checks {
            assert_eq!(check.keys(), fields);
        }
        assert_eq!(checks[1].get("digit"), &Value::Number(3));
        assert_eq!(checks[1].get("is_right"), &Value::Bool(false));
    }

    /// A reading is its move's index and 81 flat cells whose note is always a list,
    /// a note waiting beneath a digit included.
    #[cfg(feature = "serde")]
    #[test]
    fn a_reading_serialises_to_its_fields() {
        let board = the_scripted_board();
        // The sixth move placed 4 at FREE over the mark 2, which waits beneath it.
        let wire = capture(&ReadingAfter::of(&board, 6).unwrap());
        assert_eq!(wire.keys(), ["move_index", "cells"]);
        assert_eq!(wire.get("move_index"), &Value::Number(6));
        let cells = wire.get("cells").items();
        assert_eq!(cells.len(), 81);
        for cell in cells {
            assert_eq!(cell.keys(), ["row", "column", "digit", "note"]);
            assert!(matches!(cell.get("note"), Value::List(_)));
        }
        let free = &cells[2];
        assert_eq!(
            (free.get("row"), free.get("column")),
            (&Value::Number(1), &Value::Number(3))
        );
        assert_eq!(free.get("digit"), &Value::Number(4));
        assert_eq!(free.get("note"), &Value::List(vec![Value::Number(2)]));
        assert_eq!(cells[3].get("digit"), &Value::Null);
    }

    /// What a call hands back is `{ record, view }`, and `check` after a check only.
    #[cfg(feature = "serde")]
    #[test]
    fn what_a_call_hands_back_serialises_with_a_check_only_after_one() {
        let opened = open(givens()).unwrap();
        assert_eq!(capture(&opened).keys(), ["record", "view"]);
        let placed = play(opened.record(), Op::Place(FREE, 4).operation()).unwrap();
        assert_eq!(capture(&placed).keys(), ["record", "view"]);
        let checked = play(placed.record(), Op::CheckCell(FREE).operation()).unwrap();
        let wire = capture(&checked);
        assert_eq!(wire.keys(), ["record", "view", "check"]);
        assert_eq!(wire.get("check"), &capture(checked.check().unwrap()));
        assert_eq!(wire.get("record"), &capture(checked.record()));
        assert_eq!(wire.get("view"), &capture(checked.view()));
    }

    /// A search result is `{ verdict, guesses, solutions }`, each verdict in `PascalCase`,
    /// each solution nine rows of nine digits.
    #[cfg(feature = "serde")]
    #[test]
    fn a_search_result_serialises_to_its_fields() {
        use pawdoku::solver::search;

        let none = search([Given::new(Position::new(0, 10), 3)]);
        for (result, verdict, count) in [
            (search(givens()), "OneSolution", 1),
            (search([]), "ManySolutions", 2),
            (none, "NoSolution", 0),
        ] {
            let wire = capture(&result);
            assert_eq!(wire.keys(), ["verdict", "guesses", "solutions"]);
            assert_eq!(wire.get("verdict"), &Value::Text(verdict.to_owned()));
            let solutions = wire.get("solutions").items();
            assert_eq!(solutions.len(), count);
            for solution in solutions {
                assert_eq!(solution.items().len(), 9);
                assert!(solution.items().iter().all(|row| row.items().len() == 9));
            }
        }
    }

    /// A generation result is its seven fields in order, `seed` null from a stream, and
    /// no solution.
    #[cfg(feature = "serde")]
    #[test]
    fn a_generation_result_serialises_to_its_fields() {
        use pawdoku::generation::{Tier, generate_from_seed, generate_from_stream};
        use pawdoku::random::SeededStream;

        let fields = [
            "tier",
            "seed",
            "start_index",
            "draws_taken",
            "generation_version",
            "random_version",
            "givens",
        ];
        let seeded = capture(&generate_from_seed(Tier::Three, 0).unwrap());
        assert_eq!(seeded.keys(), fields);
        assert_eq!(seeded.get("tier"), &Value::Text("Three".to_owned()));
        assert_eq!(seeded.get("seed"), &Value::Number(0));
        assert_eq!(
            seeded.get("givens").items()[0].keys(),
            ["position", "digit"]
        );

        let streamed = generate_from_stream(Tier::Three, &mut SeededStream::new(0)).unwrap();
        let streamed = capture(&streamed);
        assert_eq!(streamed.keys(), fields);
        assert_eq!(streamed.get("seed"), &Value::Null);
        assert_eq!(streamed.clone().with("seed", &Value::Number(0)), seeded);
    }

    /// `ARefusalHandsBackNothingElse` for a record that does not reopen: refused before
    /// the guard and before the index, with the `ReopenError` inside, and the record
    /// handed over unchanged. A record no board wrote can only arrive by serde.
    #[cfg(feature = "serde")]
    #[test]
    fn a_record_that_does_not_reopen_is_refused_with_its_reopen_error() {
        let written = capture(&Board::open(givens()).unwrap().write());
        let records = [
            written
                .clone()
                .with("version", &Value::Text("record-0".to_owned())),
            written.with("undone", &Value::Number(1)),
        ];
        for wire in records {
            let record: Record = restore(wire).unwrap();
            let kept = record.clone();
            let refusal = Board::reopen(&record).unwrap_err();
            let wrapped = FromRecordError::Reopen(refusal.clone());
            // Undo is refused by the guard too, and index 1 names no move.
            assert_eq!(play(&record, Operation::Undo), Err(wrapped.clone()));
            assert_eq!(reading_after(&record, 1), Err(wrapped.clone()));
            assert_eq!(read(&record), Err(refusal.clone()));
            assert_eq!(wrapped.to_string(), refusal.to_string());
            assert_eq!(record, kept);
        }
    }
}

/// What a value hands to serde, captured as plain values: no format, and nothing
/// a value does not hand over.
#[cfg(test)]
#[cfg(feature = "serde")]
mod wire {
    use serde::de::value::{Error, MapAccessDeserializer, SeqDeserializer};
    use serde::de::{self, IntoDeserializer};
    use serde::ser::{self, Error as _, Impossible};

    /// A captured value. A struct is its fields in the order handed over.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) enum Value {
        Null,
        Bool(bool),
        Number(u64),
        Text(String),
        List(Vec<Self>),
        Object(Vec<(String, Self)>),
    }

    impl Value {
        /// The names of an object's fields, in order; none for anything else.
        pub(super) fn keys(&self) -> Vec<&str> {
            match self {
                Self::Object(fields) => fields.iter().map(|(key, _)| key.as_str()).collect(),
                _ => Vec::new(),
            }
        }

        /// The field `name` of an object.
        pub(super) fn get(&self, name: &str) -> &Self {
            let Self::Object(fields) = self else {
                panic!("{self:?} is not an object");
            };
            let found = fields.iter().find(|(key, _)| key == name);
            &found.unwrap_or_else(|| panic!("no field {name}")).1
        }

        /// The items of a list.
        pub(super) fn items(&self) -> &[Self] {
            let Self::List(items) = self else {
                panic!("{self:?} is not a list");
            };
            items
        }

        /// The value's object with the field `name` set to `to`.
        pub(super) fn with(mut self, name: &str, to: &Self) -> Self {
            let Self::Object(fields) = &mut self else {
                return self;
            };
            for (_, value) in fields.iter_mut().filter(|(key, _)| key == name) {
                value.clone_from(to);
            }
            self
        }
    }

    /// Captures `value` as it serialises.
    pub(super) fn capture<T: serde::Serialize>(value: &T) -> Value {
        value.serialize(Capture).unwrap()
    }

    /// Reads a `T` back from a captured value.
    pub(super) fn restore<T: de::DeserializeOwned>(value: Value) -> Result<T, Error> {
        T::deserialize(value)
    }

    /// The serializer that captures. What no wire value uses is refused.
    struct Capture;

    fn unused<T>(what: &str) -> Result<T, Error> {
        Err(Error::custom(format!("no wire value serialises {what}")))
    }

    impl ser::Serializer for Capture {
        type Ok = Value;
        type Error = Error;
        type SerializeSeq = Items;
        type SerializeTuple = Items;
        type SerializeTupleStruct = Impossible<Value, Error>;
        type SerializeTupleVariant = Impossible<Value, Error>;
        type SerializeMap = Impossible<Value, Error>;
        type SerializeStruct = Fields;
        type SerializeStructVariant = Variant;

        fn serialize_bool(self, value: bool) -> Result<Value, Error> {
            Ok(Value::Bool(value))
        }
        fn serialize_i8(self, _: i8) -> Result<Value, Error> {
            unused("a signed number")
        }
        fn serialize_i16(self, _: i16) -> Result<Value, Error> {
            unused("a signed number")
        }
        fn serialize_i32(self, _: i32) -> Result<Value, Error> {
            unused("a signed number")
        }
        fn serialize_i64(self, _: i64) -> Result<Value, Error> {
            unused("a signed number")
        }
        fn serialize_u8(self, value: u8) -> Result<Value, Error> {
            Ok(Value::Number(value.into()))
        }
        fn serialize_u16(self, value: u16) -> Result<Value, Error> {
            Ok(Value::Number(value.into()))
        }
        fn serialize_u32(self, value: u32) -> Result<Value, Error> {
            Ok(Value::Number(value.into()))
        }
        fn serialize_u64(self, value: u64) -> Result<Value, Error> {
            Ok(Value::Number(value))
        }
        fn serialize_f32(self, _: f32) -> Result<Value, Error> {
            unused("a float")
        }
        fn serialize_f64(self, _: f64) -> Result<Value, Error> {
            unused("a float")
        }
        fn serialize_char(self, _: char) -> Result<Value, Error> {
            unused("a char")
        }
        fn serialize_str(self, value: &str) -> Result<Value, Error> {
            Ok(Value::Text(value.to_owned()))
        }
        fn serialize_bytes(self, _: &[u8]) -> Result<Value, Error> {
            unused("bytes")
        }
        fn serialize_none(self) -> Result<Value, Error> {
            Ok(Value::Null)
        }
        fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<Value, Error> {
            value.serialize(self)
        }
        fn serialize_unit(self) -> Result<Value, Error> {
            Ok(Value::Null)
        }
        fn serialize_unit_struct(self, _: &'static str) -> Result<Value, Error> {
            Ok(Value::Null)
        }
        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
        ) -> Result<Value, Error> {
            Ok(Value::Text(variant.to_owned()))
        }
        fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
            self,
            _: &'static str,
            value: &T,
        ) -> Result<Value, Error> {
            value.serialize(self)
        }
        fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Value, Error> {
            unused("a newtype variant")
        }
        fn serialize_seq(self, _: Option<usize>) -> Result<Items, Error> {
            Ok(Items(Vec::new()))
        }
        fn serialize_tuple(self, _: usize) -> Result<Items, Error> {
            Ok(Items(Vec::new()))
        }
        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Error> {
            unused("a tuple struct")
        }
        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Error> {
            unused("a tuple variant")
        }
        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Error> {
            unused("a map")
        }
        fn serialize_struct(self, _: &'static str, _: usize) -> Result<Fields, Error> {
            Ok(Fields(Vec::new()))
        }
        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
            _: usize,
        ) -> Result<Variant, Error> {
            Ok(Variant(variant, Fields(Vec::new())))
        }
    }

    /// A list or a tuple, item by item.
    struct Items(Vec<Value>);

    impl ser::SerializeSeq for Items {
        type Ok = Value;
        type Error = Error;
        fn serialize_element<T: ?Sized + serde::Serialize>(
            &mut self,
            value: &T,
        ) -> Result<(), Error> {
            self.0.push(value.serialize(Capture)?);
            Ok(())
        }
        fn end(self) -> Result<Value, Error> {
            Ok(Value::List(self.0))
        }
    }

    impl ser::SerializeTuple for Items {
        type Ok = Value;
        type Error = Error;
        fn serialize_element<T: ?Sized + serde::Serialize>(
            &mut self,
            value: &T,
        ) -> Result<(), Error> {
            ser::SerializeSeq::serialize_element(self, value)
        }
        fn end(self) -> Result<Value, Error> {
            ser::SerializeSeq::end(self)
        }
    }

    /// A struct, field by field. A skipped field is left out.
    struct Fields(Vec<(String, Value)>);

    impl ser::SerializeStruct for Fields {
        type Ok = Value;
        type Error = Error;
        fn serialize_field<T: ?Sized + serde::Serialize>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), Error> {
            self.0.push((key.to_owned(), value.serialize(Capture)?));
            Ok(())
        }
        fn end(self) -> Result<Value, Error> {
            Ok(Value::Object(self.0))
        }
    }

    /// A struct variant, as serde's externally tagged default writes it: an object
    /// of one field, named for the variant.
    struct Variant(&'static str, Fields);

    impl ser::SerializeStructVariant for Variant {
        type Ok = Value;
        type Error = Error;
        fn serialize_field<T: ?Sized + serde::Serialize>(
            &mut self,
            key: &'static str,
            value: &T,
        ) -> Result<(), Error> {
            ser::SerializeStruct::serialize_field(&mut self.1, key, value)
        }
        fn end(self) -> Result<Value, Error> {
            let fields = ser::SerializeStruct::end(self.1)?;
            Ok(Value::Object(vec![(self.0.to_owned(), fields)]))
        }
    }

    /// An object's fields, handed back to serde one at a time.
    struct Entries(
        <Vec<(String, Value)> as IntoIterator>::IntoIter,
        Option<Value>,
    );

    impl<'de> de::MapAccess<'de> for Entries {
        type Error = Error;
        fn next_key_seed<K: de::DeserializeSeed<'de>>(
            &mut self,
            seed: K,
        ) -> Result<Option<K::Value>, Error> {
            let Some((key, value)) = self.0.next() else {
                return Ok(None);
            };
            self.1 = Some(value);
            seed.deserialize(key.into_deserializer()).map(Some)
        }
        fn next_value_seed<V: de::DeserializeSeed<'de>>(
            &mut self,
            seed: V,
        ) -> Result<V::Value, Error> {
            let value = self
                .1
                .take()
                .ok_or_else(|| Error::custom("a value first"))?;
            seed.deserialize(value)
        }
    }

    impl IntoDeserializer<'_, Error> for Value {
        type Deserializer = Self;
        fn into_deserializer(self) -> Self {
            self
        }
    }

    impl<'de> de::Deserializer<'de> for Value {
        type Error = Error;

        fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self {
                Self::Null => visitor.visit_unit(),
                Self::Bool(value) => visitor.visit_bool(value),
                Self::Number(value) => visitor.visit_u64(value),
                Self::Text(value) => visitor.visit_string(value),
                Self::List(items) => visitor.visit_seq(SeqDeserializer::new(items.into_iter())),
                Self::Object(fields) => visitor.visit_map(Entries(fields.into_iter(), None)),
            }
        }

        fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self {
                Self::Null => visitor.visit_none(),
                value => visitor.visit_some(value),
            }
        }

        /// A unit variant is its name; a struct variant is an object of one field,
        /// named for it.
        fn deserialize_enum<V: de::Visitor<'de>>(
            self,
            _: &'static str,
            _: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Error> {
            match self {
                Self::Text(variant) => visitor.visit_enum(variant.into_deserializer()),
                Self::Object(fields) => {
                    let entries = Entries(fields.into_iter(), None);
                    visitor.visit_enum(MapAccessDeserializer::new(entries))
                }
                other => Err(Error::custom(format!("{other:?} is not an enum"))),
            }
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct identifier ignored_any
        }
    }
}
