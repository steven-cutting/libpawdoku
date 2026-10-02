//! Why the board refused.

use crate::sudoku::{MoveError, Position};

/// Why an operation on a board was refused: one variant for each `requires` clause of
/// the board's rules, and one for a position that names no cell.
///
/// A refused operation changes nothing: no move joins the record, no undone move is
/// discarded and no check is kept. When several clauses fail at once the first of them
/// is reported, in the order each operation documents: a position off the grid, then a
/// solved puzzle, then the rule's own clauses as `board.allium` writes them.
///
/// The `Display` text of each variant is stable API: bindings build their exceptions
/// from it.
///
/// ```
/// use pawdoku::board::{Board, PlayError};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
///
/// let given = Position::new(1, 1);
/// let refusal = board.place(given, 6).unwrap_err();
/// assert_eq!(refusal, PlayError::GivenCell { position: given });
/// assert_eq!(refusal.to_string(), "the cell at row 1, column 1 holds a given");
///
/// assert_eq!(board.undo(), Err(PlayError::NothingToUndo));
/// assert_eq!(board.moves().count(), 0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PlayError {
    /// The position is off the grid, so there is no cell to act on.
    #[error("row {}, column {} names no cell of the grid", .position.row(), .position.column())]
    NoSuchCell {
        /// The position that was named.
        position: Position,
    },
    /// The puzzle is solved. Nothing is placed, erased, marked, checked, taken back or
    /// re-taken on a solved puzzle.
    #[error("the puzzle is solved, and solved is final")]
    Solved,
    /// The cell to place in holds a given, which is never the player's to change.
    #[error("the cell at row {}, column {} holds a given", .position.row(), .position.column())]
    GivenCell {
        /// The given cell's position.
        position: Position,
    },
    /// The digit to place or to write as a mark is not from 1 to 9.
    #[error("the digit {digit} is outside 1 to 9")]
    DigitOutOfRange {
        /// The digit that was offered.
        digit: u8,
    },
    /// The cell already holds the digit to place. A move that changes nothing is not a
    /// move: this is the board's own guard, which the rules do not have.
    #[error(
        "the cell at row {}, column {} already holds {digit}",
        .position.row(),
        .position.column()
    )]
    DigitAlreadyStands {
        /// The cell's position.
        position: Position,
        /// The digit that stands there.
        digit: u8,
    },
    /// The cell to erase or to check holds no digit of the player's own: it is empty,
    /// or it holds a given.
    #[error(
        "the cell at row {}, column {} holds no digit of the player's",
        .position.row(),
        .position.column()
    )]
    NoPlayersDigit {
        /// The cell's position.
        position: Position,
    },
    /// The cell takes no marks: it holds a digit, the setter's or the player's.
    #[error(
        "the cell at row {}, column {} holds a digit and takes no marks",
        .position.row(),
        .position.column()
    )]
    MarksNotAccepted {
        /// The cell's position.
        position: Position,
    },
    /// The mark to write is already in the cell's note.
    #[error(
        "the mark {digit} is already written at row {}, column {}",
        .position.row(),
        .position.column()
    )]
    MarkAlreadyWritten {
        /// The cell's position.
        position: Position,
        /// The mark.
        digit: u8,
    },
    /// The mark to strike is not in the cell's note.
    #[error(
        "the mark {digit} is not written at row {}, column {}",
        .position.row(),
        .position.column()
    )]
    MarkNotThere {
        /// The cell's position.
        position: Position,
        /// The mark.
        digit: u8,
    },
    /// No move stands, so there is nothing to take back.
    #[error("no move stands, so there is nothing to undo")]
    NothingToUndo,
    /// No move is undone, so there is nothing to re-take.
    #[error("no move is undone, so there is nothing to redo")]
    NothingToRedo,
}

/// The rules' refusal as the board's. The board restates the rules' guards and checks
/// them first, so no operation a caller can make gets a refusal this way; the
/// conversion is what carries one if a defect ever lets a move through to the rules,
/// so that it is a refusal and never a panic.
impl From<MoveError> for PlayError {
    fn from(refusal: MoveError) -> Self {
        match refusal {
            MoveError::NoSuchCell { position } => Self::NoSuchCell { position },
            MoveError::AlreadySolved => Self::Solved,
            MoveError::GivenCell { position } => Self::GivenCell { position },
            MoveError::DigitOutOfRange { digit } => Self::DigitOutOfRange { digit },
            MoveError::EmptyCell { position } => Self::NoPlayersDigit { position },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PlayError;
    use crate::sudoku::{MoveError, Position};
    use alloc::string::ToString;

    #[test]
    fn refusal_texts_are_stable() {
        let position = Position::new(2, 7);
        let texts = [
            (
                PlayError::NoSuchCell {
                    position: Position::new(0, 10),
                },
                "row 0, column 10 names no cell of the grid",
            ),
            (
                PlayError::Solved,
                "the puzzle is solved, and solved is final",
            ),
            (
                PlayError::GivenCell { position },
                "the cell at row 2, column 7 holds a given",
            ),
            (
                PlayError::DigitOutOfRange { digit: 10 },
                "the digit 10 is outside 1 to 9",
            ),
            (
                PlayError::DigitAlreadyStands { position, digit: 4 },
                "the cell at row 2, column 7 already holds 4",
            ),
            (
                PlayError::NoPlayersDigit { position },
                "the cell at row 2, column 7 holds no digit of the player's",
            ),
            (
                PlayError::MarksNotAccepted { position },
                "the cell at row 2, column 7 holds a digit and takes no marks",
            ),
            (
                PlayError::MarkAlreadyWritten { position, digit: 4 },
                "the mark 4 is already written at row 2, column 7",
            ),
            (
                PlayError::MarkNotThere { position, digit: 4 },
                "the mark 4 is not written at row 2, column 7",
            ),
            (
                PlayError::NothingToUndo,
                "no move stands, so there is nothing to undo",
            ),
            (
                PlayError::NothingToRedo,
                "no move is undone, so there is nothing to redo",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }

    /// No operation reaches the conversion, since the board checks the rules' guards
    /// before it puts a move to them. It is what carries a refusal if a defect ever
    /// does, so it is tested by itself.
    #[test]
    fn the_rules_refusals_convert_to_the_boards() {
        let position = Position::new(2, 7);
        let conversions = [
            (
                MoveError::NoSuchCell { position },
                PlayError::NoSuchCell { position },
            ),
            (MoveError::AlreadySolved, PlayError::Solved),
            (
                MoveError::GivenCell { position },
                PlayError::GivenCell { position },
            ),
            (
                MoveError::DigitOutOfRange { digit: 0 },
                PlayError::DigitOutOfRange { digit: 0 },
            ),
            (
                MoveError::EmptyCell { position },
                PlayError::NoPlayersDigit { position },
            ),
        ];
        for (rules, boards) in conversions {
            assert_eq!(PlayError::from(rules), boards);
        }
    }
}
