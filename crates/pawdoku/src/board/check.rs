//! A check: one question about one cell, asked and answered.

use crate::sudoku::Position;

/// One check as it was asked and answered: whether the digit the player had placed in
/// one cell is the solution's there.
///
/// A check is kept for as long as its board is, in the order asked. It is not a move:
/// it changes nothing on the board and undo never takes it back. Its answer is a yes
/// or a no, and the digit it holds is the player's own. Nothing of the solution is in
/// it but that answer.
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
///
/// // The solution's digit at row 1, column 3 is 4, so a 2 there is wrong.
/// board.place(here, 2)?;
/// let check = board.check(here)?;
/// assert!(!check.is_right());
/// assert_eq!(check.digit(), 2);
///
/// // The check is kept, and it is not a move.
/// assert_eq!(board.checks().collect::<Vec<_>>(), [check]);
/// assert_eq!(board.moves().count(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Check {
    index: usize,
    after_move: usize,
    target: Position,
    digit: u8,
    is_right: bool,
}

impl Check {
    /// `CheckCell`: the check that is `index` on its board, asked about `digit` at
    /// `target` while `after_move` moves stood.
    pub(super) const fn new(
        index: usize,
        after_move: usize,
        target: Position,
        digit: u8,
        is_right: bool,
    ) -> Self {
        Self {
            index,
            after_move,
            target,
            digit,
            is_right,
        }
    }

    /// Which check this is on its board, counted from 1 in the order asked.
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
    /// assert_eq!(board.check(Position::new(1, 3))?.index(), 1);
    /// assert_eq!(board.check(Position::new(1, 3))?.index(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }

    /// How many moves stood when the check was asked. A count and not a move: after an
    /// undo and a new move the same count may name a different move.
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
    /// board.write_mark(Position::new(1, 4), 6)?;
    /// board.place(Position::new(1, 3), 4)?;
    ///
    /// assert_eq!(board.check(Position::new(1, 3))?.after_move(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn after_move(self) -> usize {
        self.after_move
    }

    /// The cell the question was about.
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
    /// assert_eq!(board.check(Position::new(1, 3))?.target(), Position::new(1, 3));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn target(self) -> Position {
        self.target
    }

    /// The digit the question was about: the player's digit as it stood in the cell
    /// when the check was asked. The cell may since have changed.
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
    /// let check = board.check(here)?;
    ///
    /// // The player changes the cell; the check still says what it was asked about.
    /// board.place(here, 4)?;
    /// assert_eq!(check.digit(), 2);
    /// assert_eq!(board.checks().next(), Some(check));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn digit(self) -> u8 {
        self.digit
    }

    /// The answer: whether that digit is the solution's at that cell.
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
    ///
    /// board.place(here, 2)?;
    /// assert!(!board.check(here)?.is_right());
    /// board.place(here, 4)?;
    /// assert!(board.check(here)?.is_right());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_right(self) -> bool {
        self.is_right
    }
}
