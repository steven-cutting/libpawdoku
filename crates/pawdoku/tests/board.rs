//! A puzzle played through `Board` alone, from outside the crate: a whole game, what
//! the `Playing` surface exposes, and a property that drives arbitrary scripts of the
//! seven operations and holds the board to `docs/specs/board.allium` after every step.
//! Each rule is proved clause by clause in `tests/board_rules.rs`.
//!
//! Two helpers gather what the guarantees compare. The picture is every cell's digit
//! and its note, shown or waiting: it is what undo and redo restore, and what a move's
//! reading is. The full view is everything `Playing` exposes: it is what a refused
//! operation leaves alone.

#[cfg(test)]
mod tests {
    use pawdoku::board::{Board, BoardCell, Check, Move, MoveKind, PlayError};
    use pawdoku::solver::solve;
    use pawdoku::sudoku::{Given, Position, Status};
    use proptest::prelude::*;

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

    /// A cell the fixture leaves to the player. Its solution digit is 4.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`]. Its solution digit is 6.
    const BESIDE: Position = Position::new(1, 4);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// Every position of the grid, a row at a time from the top.
    fn positions() -> impl Iterator<Item = Position> {
        (1..=9).flat_map(|row| (1..=9).map(move |column| Position::new(row, column)))
    }

    /// Where a position on the grid comes in grid order, counted from 0.
    fn place_of(position: Position) -> usize {
        usize::from(position.row() - 1) * 9 + usize::from(position.column() - 1)
    }

    /// Peers by the specification's words, with nothing of the implementation: another
    /// position that shares a row, a column or a box.
    fn are_peers(a: Position, b: Position) -> bool {
        let same_box =
            (a.row() - 1) / 3 == (b.row() - 1) / 3 && (a.column() - 1) / 3 == (b.column() - 1) / 3;
        a != b && (a.row() == b.row() || a.column() == b.column() || same_box)
    }

    /// The filled cells of a picture of a grid, as givens.
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

    /// The fixture's one solution, as the solver finds it: each position with its digit.
    fn solution() -> Vec<(Position, u8)> {
        let proof = solve(givens(FIXTURE)).unwrap();
        let digits = proof.solution().iter().flatten().copied();
        positions().zip(digits).collect()
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

        /// The move this makes when it is accepted: its kind, its target and its digit.
        /// Undo, redo and a check make none.
        fn made(self) -> Option<(MoveKind, Position, Option<u8>)> {
            match self {
                Self::Place(at, digit) => Some((MoveKind::Place, at, Some(digit))),
                Self::Erase(at) => Some((MoveKind::Erase, at, None)),
                Self::WriteMark(at, digit) => Some((MoveKind::WriteMark, at, Some(digit))),
                Self::StrikeMark(at, digit) => Some((MoveKind::StrikeMark, at, Some(digit))),
                Self::Undo | Self::Redo | Self::Check(_) => None,
            }
        }
    }

    /// The marks of a note as bits, bit `d` for the mark `d`, read from the digits the
    /// note hands out.
    fn marks(digits: impl IntoIterator<Item = u8>) -> u16 {
        digits
            .into_iter()
            .fold(0, |marks, digit| marks | 1 << digit)
    }

    /// Every cell's digit and the marks of its note, in grid order.
    type Picture = Vec<(Option<u8>, u16)>;

    /// The picture: every cell's digit and its note, shown or waiting. It is what undo
    /// and redo restore. A note beneath a digit is not shown, so it is read from the
    /// latest standing move, which `TheMovesReplayToTheBoard` pins to the board; with
    /// no move standing, no digit of the player's stands and no note waits.
    fn picture(board: &Board) -> Picture {
        let latest = board.moves().filter(|made| !made.is_undone()).last();
        let cells = board.cells().map(|cell| {
            let waiting = latest.as_ref().map(|made| made.note_after(cell.position()));
            let note = cell.note().or(waiting).unwrap_or_default();
            (cell.digit(), marks(note.digits()))
        });
        cells.collect()
    }

    /// The board as it stood once `made` and no later move had been made.
    fn reading(made: &Move) -> Picture {
        let cells = positions().map(|position| {
            let note = made.note_after(position);
            (made.digit_after(position), marks(note.digits()))
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

    /// The full view: everything `Playing` exposes. A refused operation leaves it
    /// alone, and a board reopened from a record shows the same one.
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

    /// Holds that every operation on the cell at `at`, and undo and redo, is refused
    /// because the puzzle is solved, and that none of them changes the board.
    fn everything_is_refused_on(board: &mut Board, at: Position) {
        let solved = view(board);
        let asked = [
            Op::Place(at, 4),
            Op::Place(at, 1),
            Op::Erase(at),
            Op::WriteMark(at, 4),
            Op::StrikeMark(at, 4),
            Op::Check(at),
            Op::Undo,
            Op::Redo,
        ];
        for op in asked {
            assert_eq!(op.ask(board), Err(PlayError::Solved), "{op:?}");
            assert_eq!(
                view(board),
                solved,
                "{op:?} was refused and changed the board"
            );
        }
    }

    #[test]
    fn a_whole_game_is_played_through_the_board_and_then_everything_is_refused() {
        let mut board = open();
        let empty: Vec<(Position, u8)> = solution()
            .into_iter()
            .filter(|&(position, _)| !board.cell(position).unwrap().is_given())
            .collect();
        assert_eq!(empty.len(), 51);
        for &(position, digit) in &empty {
            assert_eq!(board.status(), Status::Unsolved);
            assert_eq!(board.place(position, digit), Ok(()));
        }

        // Solved, by the rules' definition: full and consistent.
        let facts = (board.status(), board.is_full(), board.is_consistent());
        assert_eq!(facts, (Status::Solved, true, true));
        assert_eq!(board.moves().filter(|made| !made.is_undone()).count(), 51);
        assert_eq!((board.can_undo(), board.can_redo()), (false, false));

        // Solved is final: every move, undo, redo and check is refused.
        everything_is_refused_on(&mut board, empty[50].0);
        everything_is_refused_on(&mut board, GIVEN);

        // Every past board can still be read, and the last reading is the board.
        let readings: Vec<Picture> = board.moves().map(|made| reading(&made)).collect();
        let filled = |picture: &Picture| picture.iter().filter(|cell| cell.0.is_some()).count();
        let counts: Vec<usize> = readings.iter().map(filled).collect();
        assert_eq!(counts, (31..=81).collect::<Vec<_>>());
        assert_eq!(readings[50], picture(&board));
    }

    /// A board with four moves made, the last of them undone, and one check asked: a
    /// mark of 4 and of 7 beside [`FREE`], then a 2 and a 4 placed in it.
    fn in_play() -> (Board, Check) {
        let mut board = open();
        let script = [
            Op::WriteMark(BESIDE, 4),
            Op::WriteMark(BESIDE, 7),
            Op::Place(FREE, 2),
            Op::Place(FREE, 4),
        ];
        for op in script {
            op.ask(&mut board).unwrap();
        }
        let asked = board.check(FREE).unwrap();
        board.undo().unwrap();
        (board, asked)
    }

    #[test]
    fn everything_playing_exposes_can_be_read_through_the_board() {
        let (board, asked) = in_play();

        // The puzzle's three facts, and the two offers.
        let facts = (board.status(), board.is_full(), board.is_consistent());
        assert_eq!(facts, (Status::Unsolved, false, true));
        assert_eq!((board.can_undo(), board.can_redo()), (true, true));

        // A cell: where it is, what it holds, how it stands, and its note while shown.
        let cell = board.cell(FREE).unwrap();
        assert_eq!((cell.row(), cell.column(), cell.position()), (1, 3, FREE));
        assert_eq!((cell.digit(), cell.is_given()), (Some(2), false));
        assert_eq!((cell.is_conflicting(), cell.note()), (false, None));
        let beside = board.cell(BESIDE).unwrap().note().unwrap();
        assert_eq!(beside.digits().collect::<Vec<_>>(), [4, 7]);
        assert_eq!(
            (beside.contains(7), beside.is_empty(), beside.len()),
            (true, false, 2)
        );

        // A move: its index, kind, target, digit, whether it is undone, its readings.
        let last = board.moves().last().unwrap();
        assert_eq!((last.index(), last.kind()), (4, MoveKind::Place));
        assert_eq!(
            (last.target(), last.digit(), last.is_undone()),
            (FREE, Some(4), true)
        );
        assert_eq!(last.digit_after(FREE), Some(4));
        assert_eq!(last.note_after(BESIDE).digits().collect::<Vec<_>>(), [7]);

        // A check: its index, how many moves stood, its target, its digit, its answer.
        let kept = board.checks().next().unwrap();
        assert_eq!(kept, asked);
        assert_eq!(
            (kept.index(), kept.after_move(), kept.target()),
            (1, 4, FREE)
        );
        assert_eq!((kept.digit(), kept.is_right()), (4, true));
    }

    // The property ---------------------------------------------------------------

    /// What `Playing` offers: the `when` clause of `op`, read from what the board lets
    /// a caller read, which is `can_undo`, `can_redo` and each cell's facts.
    fn is_offered(board: &Board, op: Op) -> bool {
        let cell = |position| board.cell(position);
        let marked = |cell: BoardCell| cell.note().is_some_and(|note| !note.is_empty());
        let offered = match op {
            Op::Place(at, _) => cell(at).is_some_and(|cell| !cell.is_given()),
            Op::Erase(at) | Op::Check(at) => cell(at).is_some_and(BoardCell::holds_players_digit),
            Op::WriteMark(at, _) => cell(at).is_some_and(BoardCell::accepts_marks),
            Op::StrikeMark(at, _) => {
                cell(at).is_some_and(|cell| cell.accepts_marks() && marked(cell))
            }
            Op::Undo => board.can_undo(),
            Op::Redo => board.can_redo(),
        };
        board.status() == Status::Unsolved && offered
    }

    /// Whether the clauses a rule asks of its cell hold, for a cell that is not given.
    /// Written from the `requires` clauses of `board.allium` in the plain facts a cell
    /// shows: its digit and its note.
    fn the_cells_clauses_hold(cell: BoardCell, op: Op) -> bool {
        let is_digit = |digit: u8| (1..=9).contains(&digit);
        let is_marked = |digit| cell.note().is_some_and(|note| note.contains(digit));
        let is_empty = cell.digit().is_none();
        match op {
            Op::Place(_, digit) => is_digit(digit) && cell.digit() != Some(digit),
            Op::Erase(_) | Op::Check(_) => !is_empty,
            Op::WriteMark(_, digit) => is_empty && is_digit(digit) && !is_marked(digit),
            Op::StrikeMark(_, digit) => is_empty && is_marked(digit),
            Op::Undo | Op::Redo => true,
        }
    }

    /// The oracle for acceptance, which is the rule: `op` is accepted exactly when every
    /// `requires` clause of its rule holds and its position names a cell.
    fn is_accepted_by_its_rule(board: &Board, op: Op) -> bool {
        let standing = board.moves().filter(|made| !made.is_undone()).count();
        let players_cell = |at| board.cell(at).filter(|cell| !cell.is_given());
        let requires = match op {
            Op::Undo => standing > 0,
            Op::Redo => board.moves().count() > standing,
            Op::Place(at, _)
            | Op::Erase(at)
            | Op::WriteMark(at, _)
            | Op::StrikeMark(at, _)
            | Op::Check(at) => {
                players_cell(at).is_some_and(|cell| the_cells_clauses_hold(cell, op))
            }
        };
        board.status() == Status::Unsolved && requires
    }

    /// What a move leaves, by the `ensures` clauses of its rule: the picture after `op`
    /// is made on the picture `before`. A placement strikes its digit from every peer.
    fn expected(before: &Picture, op: Op) -> Picture {
        let mut after = before.clone();
        match op {
            Op::Place(target, digit) => {
                after[place_of(target)].0 = Some(digit);
                let peers = positions().filter(|&at| are_peers(at, target));
                peers.for_each(|peer| after[place_of(peer)].1 &= !marks([digit]));
            }
            Op::Erase(target) => after[place_of(target)].0 = None,
            Op::WriteMark(target, digit) => after[place_of(target)].1 |= marks([digit]),
            Op::StrikeMark(target, digit) => after[place_of(target)].1 &= !marks([digit]),
            Op::Undo | Op::Redo | Op::Check(_) => {}
        }
        after
    }

    /// What the property keeps as a script runs: the picture of the board as it was
    /// opened, the picture taken after each move on the record, and how many of those
    /// moves stand. A new move discards the pictures of the undone ones; undo and redo
    /// discard none.
    struct Kept {
        opening: Picture,
        after: Vec<Picture>,
        standing: usize,
    }

    impl Kept {
        /// The picture taken when as many moves stood as stand now.
        fn stood(&self) -> &Picture {
            let latest = self.standing.checked_sub(1);
            latest.map_or(&self.opening, |latest| &self.after[latest])
        }
    }

    /// A move: it joins the record standing, at one past the count that stood; every
    /// move that was undone is gone; the checks are unchanged; and the picture is what
    /// its rule ensures.
    fn after_a_move(before: &View, after: &View, kept: &mut Kept, op: Op) {
        let standing = kept.standing;
        assert_eq!(
            after.moves.len(),
            standing + 1,
            "ANewMoveDiscardsWhatWasUndone"
        );
        assert_eq!(after.moves[..standing], before.moves[..standing]);
        let made = &after.moves[standing];
        assert_eq!((made.index, made.is_undone), (standing + 1, false));
        assert_eq!(Some((made.kind, made.target, made.digit)), op.made());
        assert_eq!(after.checks, before.checks);
        assert_eq!(after.picture, expected(&before.picture, op), "{op:?}");
        kept.after.truncate(standing);
        kept.after.push(after.picture.clone());
        kept.standing += 1;
    }

    /// An undo: the picture equals the one taken before the move taken back. That move
    /// is still on the record with its index, now undone; no other move changed; the
    /// checks are unchanged; and redo is offered.
    fn after_an_undo(before: &View, after: &View, kept: &mut Kept) {
        kept.standing -= 1;
        assert_eq!(&after.picture, kept.stood(), "UndoAndRedoAreExact");
        let mut moves = before.moves.clone();
        assert!(!moves[kept.standing].is_undone);
        moves[kept.standing].is_undone = true;
        assert_eq!(after.moves, moves);
        assert_eq!(after.checks, before.checks, "EveryCheckIsKept");
        assert!(after.can_redo);
    }

    /// A redo: the picture equals the one taken after that move. The move stands
    /// again; no other move changed; the checks are unchanged.
    fn after_a_redo(before: &View, after: &View, kept: &mut Kept) {
        let mut moves = before.moves.clone();
        assert!(moves[kept.standing].is_undone);
        moves[kept.standing].is_undone = false;
        kept.standing += 1;
        assert_eq!(&after.picture, kept.stood(), "UndoAndRedoAreExact");
        assert_eq!(after.moves, moves);
        assert_eq!(after.checks, before.checks, "EveryCheckIsKept");
    }

    /// A check of the cell at `at`, whose solution digit is `right`: the picture and
    /// the moves are unchanged, and the checks grew by one, which says what was asked.
    fn after_a_check(before: &View, after: &View, at: Position, right: u8) {
        assert_eq!(after.picture, before.picture);
        assert_eq!(after.moves, before.moves);
        assert_eq!(
            (after.can_undo, after.can_redo),
            (before.can_undo, before.can_redo)
        );
        let (asked, earlier) = after.checks.split_last().unwrap();
        assert_eq!(earlier, before.checks, "EveryCheckIsKept");
        let digit = before.picture[place_of(at)].0.unwrap();
        let standing = before.moves.iter().filter(|made| !made.is_undone).count();
        assert_eq!(
            (asked.index(), asked.after_move()),
            (earlier.len() + 1, standing)
        );
        assert_eq!((asked.target(), asked.digit()), (at, digit));
        assert_eq!(asked.is_right(), digit == right, "ACheckNeverTellsTheDigit");
    }

    /// The invariants of the cells, as far as the surface shows them, and each cell's
    /// derived facts.
    fn the_cells_keep_their_invariants(view: &View) {
        let at: Vec<Position> = view.cells.iter().map(|cell| cell.position()).collect();
        let grid: Vec<Position> = positions().collect();
        assert_eq!(at, grid, "TheBoardIsWhole, OneBoardCellToAPosition");
        let given = view.cells.iter().filter(|cell| cell.is_given());
        let given: Vec<Given> = given
            .map(|cell| Given::new(cell.position(), cell.digit().unwrap()))
            .collect();
        assert_eq!(given, givens(FIXTURE), "GivensNeverMove");

        for (cell, &(_, note)) in view.cells.iter().zip(&view.picture) {
            let is_empty = cell.digit().is_none();
            assert_eq!(note & !marks(1..=9), 0, "MarksAreDigits");
            assert!(!cell.is_given() || note == 0, "NoMarkInAGiven");
            assert_eq!(cell.holds_players_digit(), !is_empty && !cell.is_given());
            assert_eq!(cell.accepts_marks(), is_empty && !cell.is_given());
            assert_eq!(cell.shows_note(), is_empty);
            assert_eq!(
                cell.note().is_some(),
                is_empty,
                "a note is shown only while empty"
            );
        }
    }

    /// The invariants of the record as a whole, as far as the surface shows them.
    fn the_record_keeps_its_invariants(view: &View, kept: &Kept) {
        let count = view.moves.len();
        let indices: Vec<usize> = view.moves.iter().map(|made| made.index).collect();
        let from_one: Vec<usize> = (1..=count).collect();
        assert_eq!(
            indices, from_one,
            "MoveIndicesRunFromOne, MoveIndicesAreDistinct"
        );

        let standing = view.moves.iter().take_while(|made| !made.is_undone).count();
        let undone = &view.moves[standing..];
        assert!(
            undone.iter().all(|made| made.is_undone),
            "UndoneMovesAreTheLatest"
        );
        assert_eq!(standing, kept.standing);
        let next = undone.first().map(|made| made.index);
        assert!(
            next.is_none_or(|next| next == standing + 1),
            "WhatIsReTakenComesNext"
        );

        let unsolved = view.puzzle.0 == Status::Unsolved;
        assert!(
            unsolved || undone.is_empty(),
            "ASolvedBoardHasNothingUndone"
        );
        assert_eq!(
            view.puzzle.0 == Status::Solved,
            view.puzzle.1 && view.puzzle.2
        );
        assert_eq!(
            view.can_undo,
            unsolved && standing > 0,
            "UndoStopsWithThePuzzle"
        );
        assert_eq!(
            view.can_redo,
            unsolved && !undone.is_empty(),
            "UndoStopsWithThePuzzle"
        );
        assert_eq!(&view.picture, kept.stood(), "TheMovesReplayToTheBoard");

        let checks: Vec<usize> = view.checks.iter().map(|check| check.index()).collect();
        let from_one: Vec<usize> = (1..=view.checks.len()).collect();
        assert_eq!(
            checks, from_one,
            "ChecksRunFromOne, CheckIndicesAreDistinct"
        );
        let on_given = |check: &Check| view.cells[place_of(check.target())].is_given();
        assert!(!view.checks.iter().any(on_given), "ACheckIsOnAPlayersCell");
    }

    /// The invariants of each move, as far as the surface shows them, and reading back:
    /// every move on the record, standing or undone, reads the picture taken after it.
    fn each_move_keeps_its_invariants(view: &View, kept: &Kept) {
        let mut before = &kept.opening;
        for made in &view.moves {
            let target = place_of(made.target);
            assert!(!view.cells[target].is_given(), "MovesNeverTouchAGiven");

            let is_erasure = made.kind == MoveKind::Erase;
            assert_eq!(
                made.digit.is_none(),
                is_erasure,
                "MovesCarryWhatTheirKindNeeds"
            );
            let displaced = before[target].0.is_some();
            assert!(!is_erasure || displaced, "MovesCarryWhatTheirKindNeeds");

            let notes = positions().zip(before.iter().zip(&made.after));
            let mut struck = notes.filter(|&(at, (was, is))| was.1 != is.1 && at != made.target);
            let by_upkeep = made.kind == MoveKind::Place;
            let only_peers = struck.all(|(at, _)| by_upkeep && are_peers(at, made.target));
            assert!(only_peers, "UpkeepStrikesPeersOfAPlacement");
            before = &made.after;
        }
        let readings: Vec<&Picture> = view.moves.iter().map(|made| &made.after).collect();
        let taken: Vec<&Picture> = kept.after.iter().collect();
        assert_eq!(readings, taken, "EveryPastBoardIsReadable");
    }

    /// Asks `op` of the board and holds the board to its rules: the operation is
    /// accepted exactly when its rule accepts it and never where it is not offered, a
    /// refusal changes nothing, an accepted operation does what its rule says, and
    /// every invariant holds afterwards. `right` is the solution, for the checks.
    fn step(board: &mut Board, kept: &mut Kept, op: Op, right: &[(Position, u8)]) {
        let before = view(board);
        let offered = is_offered(board, op);
        let accepted = is_accepted_by_its_rule(board, op);
        let outcome = op.ask(board);
        let after = view(board);
        assert_eq!(
            outcome.is_ok(),
            accepted,
            "{op:?} against its rule: {outcome:?}"
        );
        assert!(
            offered || !accepted,
            "{op:?} was accepted where it is not offered"
        );
        match (outcome, op) {
            (Err(_), _) => assert_eq!(after, before, "{op:?} was refused and changed the board"),
            (Ok(()), Op::Undo) => after_an_undo(&before, &after, kept),
            (Ok(()), Op::Redo) => after_a_redo(&before, &after, kept),
            (Ok(()), Op::Check(at)) => after_a_check(&before, &after, at, right[place_of(at)].1),
            (Ok(()), _) => after_a_move(&before, &after, kept, op),
        }
        the_cells_keep_their_invariants(&after);
        the_record_keeps_its_invariants(&after, kept);
        each_move_keeps_its_invariants(&after, kept);
    }

    /// One step of a script: an operation, or the placement of the solution's digit,
    /// which is what lets a script reach the end of the puzzle.
    #[derive(Debug, Clone, Copy)]
    enum Ask {
        Op(Op),
        PlaceRight(Position),
    }

    impl Ask {
        /// The operation to ask. `right` is the solution; a position off the grid has
        /// no digit there, and the placement is asked with a digit no cell holds.
        fn op(self, right: &[(Position, u8)]) -> Op {
            match self {
                Self::Op(op) => op,
                Self::PlaceRight(at) => {
                    let digit = right.iter().find(|(position, _)| *position == at);
                    Op::Place(at, digit.map_or(0, |&(_, digit)| digit))
                }
            }
        }
    }

    /// A position for a script. Most are in the first two rows, so that moves meet in
    /// one cell and among peers; some are in the last row, whose empty cells are the
    /// last a prefilled board has left; a few are anywhere, off the grid included.
    fn position() -> impl Strategy<Value = Position> {
        let cell = |(row, column)| Position::new(row, column);
        prop_oneof![
            5 => (1..=2_u8, 1..=9_u8).prop_map(cell),
            3 => (9..=9_u8, 1..=9_u8).prop_map(cell),
            1 => (1..=9_u8, 1..=9_u8).prop_map(cell),
            1 => (0..=10_u8, 0..=10_u8).prop_map(cell),
        ]
    }

    /// A digit for a script: mostly one of four, so that marks and placements meet;
    /// sometimes any digit; now and then one out of range.
    fn digit() -> impl Strategy<Value = u8> {
        prop_oneof![6 => 1..=4_u8, 2 => 1..=9_u8, 1 => 0..=11_u8]
    }

    /// One step of a script: any of the seven operations, refused ones included.
    fn ask() -> impl Strategy<Value = Ask> {
        let place = (position(), digit()).prop_map(|(at, digit)| Op::Place(at, digit));
        let write = (position(), digit()).prop_map(|(at, digit)| Op::WriteMark(at, digit));
        let strike = (position(), digit()).prop_map(|(at, digit)| Op::StrikeMark(at, digit));
        prop_oneof![
            3 => place.prop_map(Ask::Op),
            3 => position().prop_map(Ask::PlaceRight),
            2 => position().prop_map(|at| Ask::Op(Op::Erase(at))),
            4 => write.prop_map(Ask::Op),
            2 => strike.prop_map(Ask::Op),
            3 => Just(Ask::Op(Op::Undo)),
            2 => Just(Ask::Op(Op::Redo)),
            2 => position().prop_map(|at| Ask::Op(Op::Check(at))),
        ]
    }

    /// How many of the fixture's fifty-one empty cells a script finds already filled
    /// with the solution's digits, in grid order: none, or nearly all, so that some
    /// scripts solve the puzzle and go on, and some begin on a solved one.
    fn filled() -> impl Strategy<Value = usize> {
        prop_oneof![2 => Just(0_usize), 1 => 45..=51_usize]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// The state machine: arbitrary sequences of the seven operations, refused
        /// ones included, with every step held to the rules and the invariants.
        #[test]
        fn any_script_keeps_the_board_to_its_rules(
            filled in filled(),
            script in prop::collection::vec(ask(), 0..48),
        ) {
            let right = solution();
            let mut board = open();
            let mut kept = Kept {
                opening: picture(&board),
                after: Vec::new(),
                standing: 0,
            };
            let empty = right.iter().filter(|(at, _)| !board.cell(*at).unwrap().is_given());
            let head: Vec<Op> = empty.take(filled).map(|&(at, digit)| Op::Place(at, digit)).collect();
            let tail = script.into_iter().map(|ask| ask.op(&right));
            for op in head.into_iter().chain(tail) {
                step(&mut board, &mut kept, op, &right);
            }
        }
    }
}
