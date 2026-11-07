//! The `Board` class: one board held for as long as its puzzle is in play, the
//! playable abstraction of decision 0014 as JavaScript holds it.
//!
//! Every value crosses through [`crate::types`]: positions, digits and indices are read
//! through serde, so a digit of `260` is refused rather than wrapped to a `4`, and the
//! view, the reading, the record and a check come back as the core's wire shape. A
//! method reads a digit or an index with [`types::read_held`], which refuses an object
//! unread, and a position with [`types::read_held_position`], from its own fields as
//! `play` reads an operation's, so no JavaScript of the caller's throws through Rust while
//! the board is borrowed: a getter that throws comes back as its own exception and
//! leaves the board usable.

use pawdoku::board::{Board as CoreBoard, BoardView, ReadingAfter, Record, RecordVersion};
use pawdoku::sudoku::{Given, Position};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::error::thrown;
use crate::types;

/// A record read from JavaScript: its version first, through the core's
/// [`RecordVersion`], so that a record of another version, or of a later shape, is
/// refused with the version's text and not with serde's; then the rest of it.
pub(crate) enum Unread {
    /// The value is not the shape of a record: serde's `Error`.
    Shape(JsValue),
    /// The record names another version, or none.
    Version(pawdoku::board::ReopenError),
}

/// Reads `value` as a record, version first. This crate writes no reading of a
/// version of its own: [`RecordVersion::readable`] is the core's.
pub(crate) fn record(value: JsValue) -> Result<Record, Unread> {
    let version: RecordVersion = types::read(value.clone()).map_err(Unread::Shape)?;
    version.readable().map_err(Unread::Version)?;
    types::read(value).map_err(Unread::Shape)
}

/// Reads `value` as the givens of a puzzle.
pub(crate) fn givens(value: JsValue) -> Result<Vec<Given>, JsValue> {
    types::read(value)
}

/// A sudoku board in play: the puzzle, the player's digits, notes, moves and checks.
///
/// Open one with [`Board::open`] or reopen one with [`Board::reopen`], hold it for as
/// long as the puzzle is in play, and call `free()` once when the puzzle is left. A
/// refused operation throws and changes nothing. The solution is never shown; a check
/// says only whether a digit is right.
///
/// ```js
/// const board = Board.open(givens);
/// board.place({ row: 1, column: 3 }, 4);
/// const view = board.view();
/// localStorage.setItem("game", JSON.stringify(board.write()));
/// board.free();
/// ```
#[wasm_bindgen]
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Board {
    board: CoreBoard,
}

const _: () = {
    const fn assert_bounds<T: Send + Sync + 'static + Clone + core::fmt::Debug>() {}
    assert_bounds::<Board>();
};

#[wasm_bindgen]
impl Board {
    /// Opens a board on `givens`, which the solver must find exactly one solution for.
    ///
    /// # Errors
    ///
    /// Throws a `SolveError` when the givens have no solution or several, and an
    /// `Error` when `givens` is not an array of givens.
    ///
    /// ```js
    /// const board = Board.open([{ position: { row: 1, column: 1 }, digit: 5 }, ...rest]);
    /// ```
    pub fn open(
        #[wasm_bindgen(unchecked_param_type = "Given[]")] givens: JsValue,
    ) -> Result<Self, JsValue> {
        let givens = self::givens(givens)?;
        let board = CoreBoard::open(givens).map_err(|refusal| thrown(&refusal))?;
        Ok(Self { board })
    }

    /// Reopens the board `record` was written from, as it was left. The record's
    /// version is read first.
    ///
    /// # Errors
    ///
    /// Throws a `ReopenError` naming both versions when the record names another
    /// version or none, a `ReopenError` when the record does not read back, and an
    /// `Error` when `record` is not the shape of a record.
    ///
    /// ```js
    /// const board = Board.reopen(JSON.parse(localStorage.getItem("game")));
    /// ```
    pub fn reopen(
        #[wasm_bindgen(unchecked_param_type = "BoardRecord")] record: JsValue,
    ) -> Result<Self, JsValue> {
        let record = self::record(record).map_err(|unread| match unread {
            Unread::Shape(error) => error,
            Unread::Version(refusal) => thrown(&refusal),
        })?;
        let board = CoreBoard::reopen(&record).map_err(|refusal| thrown(&refusal))?;
        Ok(Self { board })
    }

    /// The board as it stands: its status, its four flags, its 81 cells, its moves
    /// and its checks, without the readings after its moves.
    ///
    /// # Errors
    ///
    /// None in practice: the view always serialises.
    ///
    /// ```js
    /// const { status, cells } = board.view();
    /// ```
    #[wasm_bindgen(unchecked_return_type = "BoardView")]
    pub fn view(&self) -> Result<JsValue, JsValue> {
        types::board(&BoardView::of(&self.board))
    }

    /// The 81 cells as move `index` left them, counted from 1; an undone move still
    /// has its reading.
    ///
    /// # Errors
    ///
    /// Throws a `FromRecordError` when no move has that index, and an `Error` when
    /// `index` is not a whole number in range.
    ///
    /// ```js
    /// const { cells } = board.readingAfter(1);
    /// ```
    #[wasm_bindgen(
        js_name = readingAfter,
        unchecked_return_type = "ReadingAfter"
    )]
    pub fn reading_after(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: JsValue,
    ) -> Result<JsValue, JsValue> {
        let index: usize = types::read_held(index)?;
        let reading = ReadingAfter::of(&self.board, index).map_err(|refusal| thrown(&refusal))?;
        types::board(&reading)
    }

    /// The record of the board, to keep between visits: `JSON.stringify` it.
    ///
    /// # Errors
    ///
    /// None in practice: the record always serialises.
    ///
    /// ```js
    /// const kept = JSON.stringify(board.write());
    /// ```
    #[wasm_bindgen(unchecked_return_type = "BoardRecord")]
    pub fn write(&self) -> Result<JsValue, JsValue> {
        types::board(&self.board.write())
    }

    /// Places `digit` in the cell at `position`.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when the move is refused, and an `Error` when a
    /// value is not the shape asked for; either way the board is unchanged.
    ///
    /// ```js
    /// board.place({ row: 1, column: 3 }, 4);
    /// ```
    pub fn place(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Position")] position: JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] digit: JsValue,
    ) -> Result<(), JsValue> {
        let (position, digit) = cell_and_digit(position, digit)?;
        self.board
            .place(position, digit)
            .map_err(|refusal| thrown(&refusal))
    }

    /// Erases the player's digit from the cell at `position`.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when the move is refused, and an `Error` when
    /// `position` is not a position; either way the board is unchanged.
    ///
    /// ```js
    /// board.erase({ row: 1, column: 3 });
    /// ```
    pub fn erase(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Position")] position: JsValue,
    ) -> Result<(), JsValue> {
        let position: Position = types::read_held_position(position)?;
        self.board
            .erase(position)
            .map_err(|refusal| thrown(&refusal))
    }

    /// Writes the mark `digit` in the note of the cell at `position`.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when the move is refused, and an `Error` when a
    /// value is not the shape asked for; either way the board is unchanged.
    ///
    /// ```js
    /// board.writeMark({ row: 1, column: 4 }, 6);
    /// ```
    #[wasm_bindgen(js_name = writeMark)]
    pub fn write_mark(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Position")] position: JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] digit: JsValue,
    ) -> Result<(), JsValue> {
        let (position, digit) = cell_and_digit(position, digit)?;
        self.board
            .write_mark(position, digit)
            .map_err(|refusal| thrown(&refusal))
    }

    /// Strikes the mark `digit` from the note of the cell at `position`.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when the move is refused, and an `Error` when a
    /// value is not the shape asked for; either way the board is unchanged.
    ///
    /// ```js
    /// board.strikeMark({ row: 1, column: 4 }, 6);
    /// ```
    #[wasm_bindgen(js_name = strikeMark)]
    pub fn strike_mark(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Position")] position: JsValue,
        #[wasm_bindgen(unchecked_param_type = "number")] digit: JsValue,
    ) -> Result<(), JsValue> {
        let (position, digit) = cell_and_digit(position, digit)?;
        self.board
            .strike_mark(position, digit)
            .map_err(|refusal| thrown(&refusal))
    }

    /// Checks the player's digit in the cell at `position` against the solution, and
    /// returns the check: whether the digit is right, never the right digit.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when the check is refused, and an `Error` when
    /// `position` is not a position; either way the board is unchanged.
    ///
    /// ```js
    /// const { is_right } = board.check({ row: 1, column: 3 });
    /// ```
    #[wasm_bindgen(unchecked_return_type = "Check")]
    pub fn check(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Position")] position: JsValue,
    ) -> Result<JsValue, JsValue> {
        let position: Position = types::read_held_position(position)?;
        let check = self
            .board
            .check(position)
            .map_err(|refusal| thrown(&refusal))?;
        types::board(&check)
    }

    /// Undoes the last move that stands.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when there is none to undo, or the puzzle is
    /// solved; the board is unchanged.
    ///
    /// ```js
    /// if (board.view().can_undo) board.undo();
    /// ```
    pub fn undo(&mut self) -> Result<(), JsValue> {
        self.board.undo().map_err(|refusal| thrown(&refusal))
    }

    /// Makes the last undone move stand again.
    ///
    /// # Errors
    ///
    /// Throws the board's `PlayError` when there is none to redo, or the puzzle is
    /// solved; the board is unchanged.
    ///
    /// ```js
    /// if (board.view().can_redo) board.redo();
    /// ```
    pub fn redo(&mut self) -> Result<(), JsValue> {
        self.board.redo().map_err(|refusal| thrown(&refusal))
    }
}

/// A position and a digit read from JavaScript, the digit through serde so that it is
/// refused rather than wrapped when it is not a `u8`.
fn cell_and_digit(position: JsValue, digit: JsValue) -> Result<(Position, u8), JsValue> {
    Ok((
        types::read_held_position(position)?,
        types::read_held(digit)?,
    ))
}
