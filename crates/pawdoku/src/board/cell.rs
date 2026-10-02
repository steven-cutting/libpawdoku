//! A cell as the board shows it.

use super::note::Note;
use crate::sudoku::{Cell, Position};

/// One cell of a board as it stands: the puzzle's cell at its position, and the note
/// the player has written there. A value, read from the board and never written back.
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
/// let board = Board::open(givens)?;
///
/// // Row 1 begins 5, 3 and an empty cell.
/// let given = board.cell(Position::new(1, 1)).expect("row 1, column 1 is a cell");
/// assert_eq!(given.digit(), Some(5));
/// assert!(given.is_given());
///
/// let empty = board.cell(Position::new(1, 3)).expect("row 1, column 3 is a cell");
/// assert_eq!(empty.digit(), None);
/// assert!(empty.accepts_marks());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct BoardCell {
    cell: Cell,
    note: Note,
}

impl BoardCell {
    /// The board's cell that stands on `cell` of the puzzle and keeps `note`.
    pub(super) const fn new(cell: Cell, note: Note) -> Self {
        Self { cell, note }
    }

    /// The note the cell keeps, whether it is shown or waits beneath a digit.
    pub(super) const fn kept_note(self) -> Note {
        self.note
    }

    /// The digit the player has placed here, or nothing for an empty cell and for a
    /// given.
    pub(super) fn players_digit(self) -> Option<u8> {
        self.cell.digit().filter(|_| !self.cell.is_given())
    }

    /// Whether `other` is a peer of this cell, as `sudoku.allium` has it: another cell
    /// that shares its row, its column or its box.
    pub(super) const fn is_peer_of(self, other: Self) -> bool {
        let (a, b) = (self.cell, other.cell);
        let shares_a_box = a.band() == b.band() && a.stack() == b.stack();
        let is_another = a.row() != b.row() || a.column() != b.column();
        (a.row() == b.row() || a.column() == b.column() || shares_a_box) && is_another
    }

    /// Where the cell sits. Every operation on a cell takes its position; undo and redo
    /// take none.
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
    /// let board = Board::open(givens)?;
    ///
    /// // The cells come a row at a time from the top.
    /// let tenth = board.cells().nth(9).expect("a board has eighty-one cells");
    /// assert_eq!(tenth.position(), Position::new(2, 1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn position(self) -> Position {
        Position::new(self.cell.row(), self.cell.column())
    }

    /// The cell's row, counted from the top.
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
    /// let board = Board::open(givens)?;
    /// let cell = board.cell(Position::new(4, 7)).expect("row 4, column 7 is a cell");
    /// assert_eq!(cell.row(), 4);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn row(self) -> u8 {
        self.cell.row()
    }

    /// The cell's column, counted from the left.
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
    /// let board = Board::open(givens)?;
    /// let cell = board.cell(Position::new(4, 7)).expect("row 4, column 7 is a cell");
    /// assert_eq!(cell.column(), 7);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn column(self) -> u8 {
        self.cell.column()
    }

    /// The digit the cell holds, the setter's or the player's, or nothing while it is
    /// empty.
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
    /// let digit_at = |board: &Board, position| board.cell(position).and_then(|cell| cell.digit());
    ///
    /// assert_eq!(digit_at(&board, Position::new(1, 1)), Some(5));
    /// assert_eq!(digit_at(&board, Position::new(1, 3)), None);
    /// board.place(Position::new(1, 3), 4)?;
    /// assert_eq!(digit_at(&board, Position::new(1, 3)), Some(4));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn digit(self) -> Option<u8> {
        self.cell.digit()
    }

    /// Whether the setter wrote the cell's digit. No move, mark or check is ever about
    /// a given.
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
    /// let is_given = |position| board.cell(position).is_some_and(|cell| cell.is_given());
    /// assert!(is_given(Position::new(1, 1)));
    /// // A digit the player placed is not a given.
    /// assert!(!is_given(Position::new(1, 3)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_given(self) -> bool {
        self.cell.is_given()
    }

    /// Whether the cell holds a digit that one of its peers holds too.
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
    /// // A second 5 in row 1, beside the given 5 at its head.
    /// board.place(Position::new(1, 3), 5)?;
    ///
    /// let conflicts = |position| board.cell(position).is_some_and(|cell| cell.is_conflicting());
    /// assert!(conflicts(Position::new(1, 1)));
    /// assert!(conflicts(Position::new(1, 3)));
    /// assert!(!conflicts(Position::new(1, 2)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_conflicting(self) -> bool {
        self.cell.is_conflicting()
    }

    /// The cell's note while it is shown, which is while the cell is empty. A note
    /// beneath a digit is kept and not shown: it comes back when the digit is erased.
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
    /// let note = |board: &Board| board.cell(here).and_then(|cell| cell.note());
    ///
    /// board.write_mark(here, 4)?;
    /// assert!(note(&board).is_some_and(|note| note.contains(4)));
    ///
    /// // A digit hides the note, and erasing the digit shows it again.
    /// board.place(here, 2)?;
    /// assert_eq!(note(&board), None);
    /// board.erase(here)?;
    /// assert!(note(&board).is_some_and(|note| note.contains(4)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn note(self) -> Option<Note> {
        self.shows_note().then_some(self.note)
    }

    /// Whether the cell holds a digit of the player's own: one that is not a given.
    /// Erasing and the check ask for one.
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
    /// let holds = |board: &Board, position| {
    ///     board.cell(position).is_some_and(|cell| cell.holds_players_digit())
    /// };
    ///
    /// assert!(!holds(&board, Position::new(1, 1)));
    /// assert!(!holds(&board, Position::new(1, 3)));
    /// board.place(Position::new(1, 3), 4)?;
    /// assert!(holds(&board, Position::new(1, 3)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn holds_players_digit(self) -> bool {
        self.players_digit().is_some()
    }

    /// Whether a mark may be written or struck here: the cell is empty and is not a
    /// given.
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
    /// let accepts = |board: &Board, position| {
    ///     board.cell(position).is_some_and(|cell| cell.accepts_marks())
    /// };
    ///
    /// assert!(!accepts(&board, Position::new(1, 1)));
    /// assert!(accepts(&board, Position::new(1, 3)));
    /// board.place(Position::new(1, 3), 4)?;
    /// assert!(!accepts(&board, Position::new(1, 3)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn accepts_marks(self) -> bool {
        self.cell.digit().is_none() && !self.cell.is_given()
    }

    /// Whether the cell's note is shown: the cell is empty. A note waits beneath a
    /// digit.
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
    /// let shows = |board: &Board, position| board.cell(position).is_some_and(|cell| cell.shows_note());
    ///
    /// assert!(shows(&board, Position::new(1, 3)));
    /// board.place(Position::new(1, 3), 4)?;
    /// assert!(!shows(&board, Position::new(1, 3)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn shows_note(self) -> bool {
        self.cell.digit().is_none()
    }
}
