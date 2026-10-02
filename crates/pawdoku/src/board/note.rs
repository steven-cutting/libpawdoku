//! Notes: one cell's marks, and the board's eighty-one of them.

use crate::sudoku::{Grid, LINE, Position, SIDE, in_range};
use core::fmt;

/// One cell's note: the marks the player has written in it, each a digit from 1 to 9
/// the cell might hold.
///
/// A note is a value the board hands out. It is read and compared, and changed only by
/// the board's moves.
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
/// board.write_mark(here, 4)?;
/// board.write_mark(here, 2)?;
///
/// let note = board.cell(here).and_then(|cell| cell.note()).expect("an empty cell shows its note");
/// assert_eq!(note.digits().collect::<Vec<u8>>(), [2, 4]);
/// assert_eq!(format!("{note:?}"), "{2, 4}");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct Note {
    /// A bit to a digit: bit `d` is set while the mark `d` is written.
    marks: u16,
}

impl Note {
    /// The note with the one mark `digit`, or the empty note for what is not a digit
    /// from 1 to 9.
    const fn only(digit: u8) -> Self {
        let marks = if in_range(digit) { 1 << digit } else { 0 };
        Self { marks }
    }

    /// Every mark of either note.
    const fn joined(self, other: Self) -> Self {
        Self {
            marks: self.marks | other.marks,
        }
    }

    /// The marks of this note that `other` does not hold.
    const fn less(self, other: Self) -> Self {
        Self {
            marks: self.marks & !other.marks,
        }
    }

    /// Whether the two notes share a mark.
    const fn overlaps(self, other: Self) -> bool {
        self.marks & other.marks != 0
    }

    /// The note with the mark `digit` written, or the same note for what is not a digit.
    pub(super) const fn with(self, digit: u8) -> Self {
        self.joined(Self::only(digit))
    }

    /// The note with the mark `digit` struck.
    pub(super) const fn without(self, digit: u8) -> Self {
        self.less(Self::only(digit))
    }

    /// Whether the mark `digit` is written. No note holds a mark that is not a digit
    /// from 1 to 9.
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
    /// board.write_mark(here, 4)?;
    ///
    /// let note = board.cell(here).and_then(|cell| cell.note()).expect("an empty cell shows its note");
    /// assert!(note.contains(4));
    /// assert!(!note.contains(2));
    /// assert!(!note.contains(0));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn contains(self, digit: u8) -> bool {
        self.overlaps(Self::only(digit))
    }

    /// The marks written, from the lowest digit to the highest.
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
    /// for digit in [4, 1, 2] {
    ///     board.write_mark(here, digit)?;
    /// }
    ///
    /// let note = board.cell(here).and_then(|cell| cell.note()).expect("an empty cell shows its note");
    /// assert_eq!(note.digits().collect::<Vec<u8>>(), [1, 2, 4]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn digits(self) -> impl Iterator<Item = u8> {
        (1..=SIDE).filter(move |&digit| self.contains(digit))
    }

    /// How many marks are written.
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
    /// board.write_mark(here, 4)?;
    /// board.write_mark(here, 2)?;
    ///
    /// let note = board.cell(here).and_then(|cell| cell.note()).expect("an empty cell shows its note");
    /// assert_eq!(note.len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn len(self) -> usize {
        self.marks.count_ones() as usize
    }

    /// Whether no mark is written. Every note starts so.
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
    /// assert!(note(&board).is_some_and(|note| note.is_empty()));
    /// board.write_mark(here, 4)?;
    /// assert!(note(&board).is_some_and(|note| !note.is_empty()));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.marks == 0
    }
}

/// Shows the note as the set of its marks, lowest first.
impl fmt::Debug for Note {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.digits()).finish()
    }
}

/// What a grid holds at `position`, or nothing where the grid has no such cell.
pub(super) fn held<T: Copy>(grid: &Grid<T>, position: Position) -> Option<T> {
    let row = usize::from(position.row()).checked_sub(1)?;
    let column = usize::from(position.column()).checked_sub(1)?;
    grid.get(row)?.get(column).copied()
}

/// The place a grid keeps for `position`, or nothing where the grid has no such cell.
pub(super) fn slot<T>(grid: &mut Grid<T>, position: Position) -> Option<&mut T> {
    let row = usize::from(position.row()).checked_sub(1)?;
    let column = usize::from(position.column()).checked_sub(1)?;
    grid.get_mut(row)?.get_mut(column)
}

/// The note of every cell of a board, whether it is shown or waits beneath a digit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Notes {
    notes: Grid<Note>,
}

impl Notes {
    /// `LayOutBoard`: every note starts empty.
    pub(super) fn empty() -> Self {
        Self {
            notes: [[Note::default(); LINE]; LINE],
        }
    }

    /// The note at `position`, which is empty where the grid has no cell.
    pub(super) fn at(&self, position: Position) -> Note {
        held(&self.notes, position).unwrap_or_default()
    }

    /// Writes the mark `digit` in the note at `position`.
    pub(super) fn write(&mut self, position: Position, digit: u8) {
        if let Some(note) = slot(&mut self.notes, position) {
            *note = note.with(digit);
        }
    }

    /// Strikes the mark `digit` from the note at `position`.
    pub(super) fn strike(&mut self, position: Position, digit: u8) {
        if let Some(note) = slot(&mut self.notes, position) {
            *note = note.without(digit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Note, Notes};
    use crate::sudoku::Position;
    use alloc::format;
    use alloc::vec::Vec;

    #[test]
    fn a_note_holds_only_digits_from_one_to_nine() {
        let every_byte = (0..=u8::MAX).fold(Note::default(), Note::with);
        assert_eq!(
            every_byte.digits().collect::<Vec<u8>>(),
            [1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert_eq!(every_byte.len(), 9);
        for digit in [0, 10, 16, 255] {
            assert!(!every_byte.contains(digit));
            assert_eq!(Note::default().with(digit), Note::default());
            assert_eq!(every_byte.without(digit), every_byte);
        }
    }

    #[test]
    fn marks_are_written_and_struck() {
        let note = Note::default().with(7).with(2).with(7);
        assert_eq!(note.digits().collect::<Vec<u8>>(), [2, 7]);
        assert_eq!(format!("{note:?}"), "{2, 7}");
        assert_eq!(
            note.without(2).without(5).digits().collect::<Vec<u8>>(),
            [7]
        );
        assert!(note.without(2).without(7).is_empty());
    }

    #[test]
    fn a_note_off_the_grid_is_empty_and_takes_no_mark() {
        let mut notes = Notes::empty();
        let here = Position::new(9, 9);
        for off_grid in [Position::new(0, 1), Position::new(1, 10)] {
            notes.write(off_grid, 4);
            notes.strike(off_grid, 4);
            assert!(notes.at(off_grid).is_empty());
        }
        notes.write(here, 4);
        assert!(notes.at(here).contains(4));
        notes.strike(here, 4);
        assert_eq!(notes, Notes::empty());
    }
}
