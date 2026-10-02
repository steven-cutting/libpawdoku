//! A move as the board hands it out.

use super::journal::Made;
use super::note::Note;
use super::reading::Reading;
use crate::sudoku::Position;

/// The four things a player does to a board. A check is not among them: it changes
/// nothing and is never taken back.
///
/// ```
/// use pawdoku::board::{Board, MoveKind};
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
/// board.write_mark(here, 4)?;
/// board.strike_mark(here, 4)?;
/// board.place(here, 4)?;
/// board.erase(here)?;
///
/// let kinds: Vec<MoveKind> = board.moves().map(|made| made.kind()).collect();
/// assert_eq!(
///     kinds,
///     [MoveKind::WriteMark, MoveKind::StrikeMark, MoveKind::Place, MoveKind::Erase]
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MoveKind {
    /// A digit placed in a cell.
    Place,
    /// A cell's digit erased.
    Erase,
    /// A mark written in a cell's note.
    WriteMark,
    /// A mark struck from a cell's note.
    StrikeMark,
}

/// One move on a board's record: what the player did, whether it still stands, and
/// the board as it stood once the move had been made.
///
/// A move is a value. It stays on the record, with its index, from when it is made
/// until a new move discards it; undo marks it undone and redo makes it stand again.
/// Its reading, [`digit_after`](Self::digit_after) and [`note_after`](Self::note_after),
/// is the same whether it stands or is undone.
///
/// ```
/// use pawdoku::board::{Board, MoveKind};
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
/// let made = board.moves().next().expect("one move is on the record");
/// assert_eq!((made.index(), made.kind()), (1, MoveKind::Place));
/// assert_eq!((made.target(), made.digit()), (here, Some(4)));
/// assert!(made.is_undone());
/// // The cell is empty again, and the move still says what it left there.
/// assert_eq!(made.digit_after(here), Some(4));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Move {
    index: usize,
    kind: MoveKind,
    target: Position,
    digit: Option<u8>,
    is_undone: bool,
    after: Reading,
}

impl Move {
    /// The move that is `index` on its board, read from what the board keeps of it,
    /// with the board as it stood after it.
    pub(super) const fn new(index: usize, made: &Made, is_undone: bool, after: Reading) -> Self {
        Self {
            index,
            kind: made.kind(),
            target: made.target(),
            digit: made.digit(),
            is_undone,
            after,
        }
    }

    /// Which move this is on its board, counted from 1 in the order made. An undone
    /// move keeps its index until a new move discards it.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// board.place(Position::new(1, 3), 4)?;
    /// board.place(Position::new(1, 4), 6)?;
    ///
    /// let indices: Vec<usize> = board.moves().map(|made| made.index()).collect();
    /// assert_eq!(indices, [1, 2]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Which of the four moves this is.
    ///
    /// ```
    /// use pawdoku::board::{Board, MoveKind};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// board.write_mark(Position::new(1, 3), 4)?;
    ///
    /// let made = board.moves().next().expect("one move is on the record");
    /// assert_eq!(made.kind(), MoveKind::WriteMark);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn kind(&self) -> MoveKind {
        self.kind
    }

    /// The cell the move was made on. It is never a given.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
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
    /// let made = board.moves().next().expect("one move is on the record");
    /// assert_eq!(made.target(), Position::new(1, 3));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn target(&self) -> Position {
        self.target
    }

    /// The digit placed, written or struck. An erasure has none.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// board.place(Position::new(1, 3), 4)?;
    /// board.erase(Position::new(1, 3))?;
    ///
    /// let digits: Vec<Option<u8>> = board.moves().map(|made| made.digit()).collect();
    /// assert_eq!(digits, [Some(4), None]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn digit(&self) -> Option<u8> {
        self.digit
    }

    /// Whether the move has been taken back and not re-taken. The undone moves are
    /// always the latest on the record.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// board.place(Position::new(1, 3), 4)?;
    /// board.place(Position::new(1, 4), 6)?;
    /// board.undo()?;
    ///
    /// let undone: Vec<bool> = board.moves().map(|made| made.is_undone()).collect();
    /// assert_eq!(undone, [false, true]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_undone(&self) -> bool {
        self.is_undone
    }

    /// Reading back: the digit the cell at `position` held once this move and no later
    /// one had been made. It is nothing where the cell was empty, and nothing where the
    /// grid has no cell.
    ///
    /// The reading is worked out from the moves, from the first to this one, and taking
    /// it changes nothing on the board.
    ///
    /// ```
    /// use pawdoku::board::Board;
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
    /// board.place(here, 2)?;
    /// board.place(here, 4)?;
    ///
    /// let after: Vec<Option<u8>> = board.moves().map(|made| made.digit_after(here)).collect();
    /// assert_eq!(after, [Some(2), Some(4)]);
    /// // A given is in every reading.
    /// assert!(board.moves().all(|made| made.digit_after(Position::new(1, 1)) == Some(5)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn digit_after(&self, position: Position) -> Option<u8> {
        self.after.digit_at(position)
    }

    /// Reading back: the note the cell at `position` kept once this move and no later
    /// one had been made, whether it was shown then or waited beneath a digit. It is
    /// empty where the grid has no cell.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let peer = Position::new(1, 4);
    /// board.write_mark(peer, 4)?;
    /// // Placing a 4 in the same row strikes the mark from the peer's note.
    /// board.place(Position::new(1, 3), 4)?;
    ///
    /// let held: Vec<bool> = board.moves().map(|made| made.note_after(peer).contains(4)).collect();
    /// assert_eq!(held, [true, false]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn note_after(&self, position: Position) -> Note {
        self.after.note_at(position)
    }
}
