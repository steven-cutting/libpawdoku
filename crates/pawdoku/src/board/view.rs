//! What a board shows, as one value: the view of a board, and the reading after one
//! of its moves.
//!
//! Both are made by associated functions from a `&Board`, not by methods of
//! [`Board`], whose methods are what `Playing` offers and are kept at that. Each holds
//! what `board.allium`'s `Playing` exposes and nothing more, so nothing in either is
//! read from the solution but a check's yes or no.

use super::{Board, BoardCell, Check, FromRecordError, Move, Note};
use crate::sudoku::{Position, Status};
use alloc::vec::Vec;

/// A board as `Playing` exposes it, less the readings after its moves: its status,
/// whether it is full, whether it is consistent, whether undo and redo have anything
/// to do, its 81 cells in row order, its moves and its checks.
///
/// It is what [`play`](super::play) and [`read`](super::read) hand back, and what a
/// binding hands out for a held board, so that both ways of driving a board show the
/// same value. Each move in it can still read back the board after it, through
/// [`Move::digit_after`] and [`Move::note_after`]; under the `serde` feature a move
/// serialises without that reading, which [`ReadingAfter`] gives one move at a time.
///
/// ```
/// use pawdoku::board::{Board, BoardView};
/// use pawdoku::sudoku::{Given, Position, Status};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
/// board.place(Position::new(1, 3), 4)?;
///
/// let view = BoardView::of(&board);
/// assert_eq!(view.status(), Status::Unsolved);
/// assert!(view.can_undo() && !view.can_redo());
/// assert_eq!(view.cells().len(), 81);
/// assert_eq!(view.cells()[2].digit(), Some(4));
/// assert_eq!(view.moves().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the four flags are the four Playing exposes, each its own question"
)]
#[non_exhaustive]
pub struct BoardView {
    // The field names are the wire names: each is the specification's.
    status: Status,
    is_full: bool,
    is_consistent: bool,
    can_undo: bool,
    can_redo: bool,
    cells: Vec<BoardCell>,
    moves: Vec<Move>,
    checks: Vec<Check>,
}

impl BoardView {
    /// The view of `board` as it stands now. Reading it changes nothing.
    #[must_use]
    pub fn of(board: &Board) -> Self {
        Self {
            status: board.status(),
            is_full: board.is_full(),
            is_consistent: board.is_consistent(),
            can_undo: board.can_undo(),
            can_redo: board.can_redo(),
            cells: board.cells().collect(),
            moves: board.moves().collect(),
            checks: board.checks().collect(),
        }
    }

    /// [`Board::status`] when the view was made.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// [`Board::is_full`] when the view was made.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.is_full
    }

    /// [`Board::is_consistent`] when the view was made.
    #[must_use]
    pub const fn is_consistent(&self) -> bool {
        self.is_consistent
    }

    /// [`Board::can_undo`] when the view was made.
    #[must_use]
    pub const fn can_undo(&self) -> bool {
        self.can_undo
    }

    /// [`Board::can_redo`] when the view was made.
    #[must_use]
    pub const fn can_redo(&self) -> bool {
        self.can_redo
    }

    /// [`Board::cells`] when the view was made: 81 cells, a row at a time from the top.
    #[must_use]
    pub fn cells(&self) -> &[BoardCell] {
        &self.cells
    }

    /// [`Board::moves`] when the view was made, in the order made.
    #[must_use]
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// [`Board::checks`] when the view was made, in the order asked.
    #[must_use]
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }
}

/// One cell as it stood once a move and no later one had been made: `board.allium`'s
/// `Reading`, which is [`Move::digit_after`] and [`Move::note_after`] for that cell.
///
/// The note is the cell's note whether it was shown then or waited beneath a digit.
/// Under the `serde` feature a reading serialises flat, as its row, its column, its
/// digit (`null` for none) and its note's digits, always a list.
///
/// ```
/// use pawdoku::board::{Board, ReadingAfter};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
/// let here = Position::new(1, 3);
/// board.write_mark(here, 2)?;
/// board.place(here, 4)?;
///
/// // After the placement, the cell holds 4 and its mark waits beneath it.
/// let reading = ReadingAfter::of(&board, 2)?;
/// let cell = reading.cells()[2];
/// assert_eq!((cell.position(), cell.digit()), (here, Some(4)));
/// assert!(cell.note().contains(2));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CellReading {
    position: Position,
    digit: Option<u8>,
    note: Note,
}

impl CellReading {
    /// Where the cell is.
    #[must_use]
    pub const fn position(self) -> Position {
        self.position
    }

    /// The cell's row, from 1 to 9.
    #[must_use]
    pub const fn row(self) -> u8 {
        self.position.row()
    }

    /// The cell's column, from 1 to 9.
    #[must_use]
    pub const fn column(self) -> u8 {
        self.position.column()
    }

    /// The digit the cell held, the setter's or the player's, or nothing where it was
    /// empty.
    #[must_use]
    pub const fn digit(self) -> Option<u8> {
        self.digit
    }

    /// The note the cell kept, shown or waiting beneath a digit.
    #[must_use]
    pub const fn note(self) -> Note {
        self.note
    }
}

/// A cell serialises flat, under its own `row` and `column` and never through
/// `#[serde(flatten)]`, so that every binding reads it as a plain object; its note is
/// always a list of digits.
#[cfg(feature = "serde")]
impl serde::Serialize for CellReading {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let mut cell = serializer.serialize_struct("CellReading", 4)?;
        cell.serialize_field("row", &self.row())?;
        cell.serialize_field("column", &self.column())?;
        cell.serialize_field("digit", &self.digit)?;
        cell.serialize_field("note", &self.note)?;
        cell.end()
    }
}

/// The board as it stood once one move and no later one had been made: every cell's
/// [`CellReading`], in row order, and the index of the move.
///
/// It is made one move at a time, on asking, and never carried inside a move or a
/// [`BoardView`], because all of a long game's readings are many times the size of its
/// view. An undone move still has a reading: the board it left when it was made.
///
/// ```
/// use pawdoku::board::{Board, FromRecordError, ReadingAfter};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
/// let here = Position::new(1, 3);
/// board.place(here, 4)?;
/// board.undo()?;
///
/// // The undone placement still reads back the board it left.
/// let reading = ReadingAfter::of(&board, 1)?;
/// assert_eq!(reading.move_index(), 1);
/// assert_eq!(reading.cells()[2].digit(), Some(4));
///
/// // Moves count from 1, and there is no second move.
/// let refusal = FromRecordError::NoSuchMove { move_index: 2 };
/// assert_eq!(ReadingAfter::of(&board, 2), Err(refusal));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct ReadingAfter {
    move_index: usize,
    cells: Vec<CellReading>,
}

impl ReadingAfter {
    /// The reading after the move of index `move_index` on `board`, counted from 1 as
    /// [`Move::index`] counts, undone moves included.
    ///
    /// # Errors
    ///
    /// [`FromRecordError::NoSuchMove`] when no move of `board` has that index: 0, or
    /// anything past the last move.
    pub fn of(board: &Board, move_index: usize) -> Result<Self, FromRecordError> {
        let made = board.moves().find(|made| made.index() == move_index);
        let made = made.ok_or(FromRecordError::NoSuchMove { move_index })?;
        let read = |cell: BoardCell| CellReading {
            position: cell.position(),
            digit: made.digit_after(cell.position()),
            note: made.note_after(cell.position()),
        };
        Ok(Self {
            move_index,
            cells: board.cells().map(read).collect(),
        })
    }

    /// The index of the move read after, from 1.
    #[must_use]
    pub const fn move_index(&self) -> usize {
        self.move_index
    }

    /// Every cell as the move left it, a row at a time from the top.
    #[must_use]
    pub fn cells(&self) -> &[CellReading] {
        &self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::{BoardView, ReadingAfter};
    use crate::board::{Board, FromRecordError};
    use crate::sudoku::{Given, Position, Status};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    /// The rules' fixture: thirty givens, one solution, rows top to bottom.
    const FIXTURE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// A cell the fixture leaves to the player. Its solution digit is 4.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`].
    const BESIDE: Position = Position::new(1, 4);

    /// The filled cells of a picture read a row after a row, as givens.
    fn givens(picture: &str) -> BTreeSet<Given> {
        let cells = picture.bytes().zip(0_u8..);
        cells
            .filter(|(cell, _)| cell.is_ascii_digit())
            .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
            .collect()
    }

    /// Every kind of move, a check each way, a note beneath a digit and an undone move:
    /// marks 2 and 7 at [`FREE`], a mark at [`BESIDE`] written and struck, 9 placed at
    /// [`FREE`] over its marks and checked, erased, 4 placed and checked, and that
    /// placement undone.
    fn played() -> Board {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        board.write_mark(FREE, 2).unwrap();
        board.write_mark(FREE, 7).unwrap();
        board.write_mark(BESIDE, 7).unwrap();
        board.strike_mark(BESIDE, 7).unwrap();
        board.place(FREE, 9).unwrap();
        assert!(!board.check(FREE).unwrap().is_right());
        board.erase(FREE).unwrap();
        board.place(FREE, 4).unwrap();
        assert!(board.check(FREE).unwrap().is_right());
        board.undo().unwrap();
        board
    }

    #[test]
    fn a_view_holds_what_playing_exposes() {
        for board in [Board::open(givens(FIXTURE)).unwrap(), played()] {
            let view = BoardView::of(&board);
            assert_eq!(view.status(), board.status());
            assert_eq!(view.is_full(), board.is_full());
            assert_eq!(view.is_consistent(), board.is_consistent());
            assert_eq!(view.can_undo(), board.can_undo());
            assert_eq!(view.can_redo(), board.can_redo());
            assert_eq!(view.cells(), board.cells().collect::<Vec<_>>());
            assert_eq!(view.moves(), board.moves().collect::<Vec<_>>());
            assert_eq!(view.checks(), board.checks().collect::<Vec<_>>());
        }
        // The played board is not a fresh one: every part of the view has something in it.
        let view = BoardView::of(&played());
        assert_eq!((view.status(), view.cells().len()), (Status::Unsolved, 81));
        assert_eq!((view.moves().len(), view.checks().len()), (7, 2));
        assert!(view.can_undo() && view.can_redo());
    }

    #[test]
    fn the_reading_after_a_move_is_digit_after_and_note_after() {
        let board = played();
        for made in board.moves() {
            let reading = ReadingAfter::of(&board, made.index()).unwrap();
            assert_eq!(reading.move_index(), made.index());
            let cells = reading.cells();
            assert_eq!(cells.len(), 81);
            let in_row_order = cells.iter().map(|cell| (cell.row(), cell.column()));
            let expected = (1..=9).flat_map(|row| (1..=9).map(move |column| (row, column)));
            assert!(in_row_order.eq(expected));
            for cell in cells {
                assert_eq!(cell.position(), Position::new(cell.row(), cell.column()));
                assert_eq!(cell.digit(), made.digit_after(cell.position()));
                assert_eq!(cell.note(), made.note_after(cell.position()));
            }
        }
        // After the fifth move, the 9 at FREE stands on the marks 2 and 7, and the
        // reading gives the note waiting beneath it.
        let reading = ReadingAfter::of(&board, 5).unwrap();
        let free = reading
            .cells()
            .iter()
            .find(|cell| cell.position() == FREE)
            .unwrap();
        assert_eq!(free.digit(), Some(9));
        assert_eq!(free.note().digits().collect::<Vec<_>>(), [2, 7]);
    }

    #[test]
    fn an_undone_move_still_has_a_reading() {
        let board = played();
        let last = board.moves().last().unwrap();
        assert!(last.is_undone());
        let reading = ReadingAfter::of(&board, last.index()).unwrap();
        let free = reading
            .cells()
            .iter()
            .find(|cell| cell.position() == FREE)
            .unwrap();
        assert_eq!(free.digit(), Some(4));
        assert_eq!(board.cell(FREE).unwrap().digit(), None);
    }

    #[test]
    fn an_index_that_names_no_move_is_refused() {
        let board = played();
        let past = board.moves().count() + 1;
        for move_index in [0, past, usize::MAX] {
            assert_eq!(
                ReadingAfter::of(&board, move_index),
                Err(FromRecordError::NoSuchMove { move_index })
            );
        }
        let fresh = Board::open(givens(FIXTURE)).unwrap();
        assert_eq!(
            ReadingAfter::of(&fresh, 1),
            Err(FromRecordError::NoSuchMove { move_index: 1 })
        );
    }
}
