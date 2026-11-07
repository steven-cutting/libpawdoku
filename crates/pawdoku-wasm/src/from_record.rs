//! Play from a record, `board.allium`'s `PlayingFromARecord`: the core's four functions
//! of `pawdoku::board` (T41), each handed a record or givens and giving back plain
//! values, with nothing held between calls. Each call on a record reopens it, which
//! costs a search and a replay of every move; a caller that plays a whole game holds a
//! [`Board`](crate::Board) instead. A value read from a held board equals the one these
//! give.
//!
//! Every record is read version first, as [`Board::reopen`](crate::Board::reopen)
//! reads it, and a version's refusal throws the `ReopenError` that `Board.reopen`
//! throws, with the same name and text.

use pawdoku::board::RECORD_VERSION;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::board::{givens, record};
use crate::error::thrown;
use crate::types;

/// Opens a board on `givens` and hands back its record and its view; nothing is held.
///
/// # Errors
///
/// Throws a `SolveError` when the givens have no solution or several, and an `Error`
/// when `givens` is not an array of givens.
///
/// ```js
/// let { record, view } = open(givens);
/// ```
#[wasm_bindgen(unchecked_return_type = "Played")]
pub fn open(
    #[wasm_bindgen(unchecked_param_type = "Given[]")] givens: JsValue,
) -> Result<JsValue, JsValue> {
    let givens = self::givens(givens)?;
    let played = pawdoku::board::open(givens).map_err(|refusal| thrown(&refusal))?;
    types::board(&played)
}

/// Puts one operation to `record`: reopens it, makes the operation and hands back the
/// record after it and the view, with the check it added for a `CheckCell`. The record
/// handed in is not changed.
///
/// # Errors
///
/// Throws a `ReopenError` when the record does not reopen, a version of another
/// record among them; a `PlayError` when the board refuses the operation; and an
/// `Error` when a value is not the shape asked for.
///
/// ```js
/// ({ record, view } = play(record, { kind: "Place", position: { row: 1, column: 3 }, digit: 4 }));
/// ```
#[wasm_bindgen(unchecked_return_type = "Played")]
pub fn play(
    #[wasm_bindgen(unchecked_param_type = "BoardRecord")] record: JsValue,
    #[wasm_bindgen(unchecked_param_type = "Operation")] operation: JsValue,
) -> Result<JsValue, JsValue> {
    let record = self::record(record)?;
    let operation = types::read_operation(operation)?;
    let played = pawdoku::board::play(&record, operation).map_err(|refusal| thrown(&refusal))?;
    types::board(&played)
}

/// The view of the board `record` reopens to.
///
/// # Errors
///
/// Throws a `ReopenError` when the record does not reopen, and an `Error` when
/// `record` is not the shape of a record.
///
/// ```js
/// const view = read(record);
/// ```
#[wasm_bindgen(unchecked_return_type = "BoardView")]
pub fn read(
    #[wasm_bindgen(unchecked_param_type = "BoardRecord")] record: JsValue,
) -> Result<JsValue, JsValue> {
    let record = self::record(record)?;
    let view = pawdoku::board::read(&record).map_err(|refusal| thrown(&refusal))?;
    types::board(&view)
}

/// The 81 cells as move `moveIndex` of `record` left them, counted from 1.
///
/// # Errors
///
/// Throws a `ReopenError` when the record does not reopen, a `FromRecordError` when no
/// move has that index, and an `Error` when a value is not the shape asked for.
///
/// ```js
/// const { cells } = readingAfter(record, 1);
/// ```
#[wasm_bindgen(js_name = readingAfter, unchecked_return_type = "ReadingAfter")]
pub fn reading_after(
    #[wasm_bindgen(unchecked_param_type = "BoardRecord")] record: JsValue,
    #[wasm_bindgen(js_name = moveIndex, unchecked_param_type = "number")] move_index: JsValue,
) -> Result<JsValue, JsValue> {
    let record = self::record(record)?;
    let move_index: usize = types::read(move_index)?;
    let reading =
        pawdoku::board::reading_after(&record, move_index).map_err(|refusal| thrown(&refusal))?;
    types::board(&reading)
}

/// The format version every record this package writes names, `"record-1"`, the
/// engine's `RECORD_VERSION`. A record of another version is refused. A function, not
/// a constant, because wasm-bindgen exports no constant.
///
/// ```js
/// recordVersion(); // "record-1"
/// ```
#[wasm_bindgen(js_name = recordVersion)]
#[must_use]
pub fn record_version() -> String {
    RECORD_VERSION.into()
}
