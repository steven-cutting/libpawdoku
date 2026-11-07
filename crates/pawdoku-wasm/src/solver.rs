//! The solver's search, `solver.allium`'s `SearchResult`: a setter's tool, which shows
//! the solutions it finds. A `Board` puts its givens to the solver itself, so a game
//! that only plays never needs this.

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::board::givens;
use crate::types;

/// Searches `givens` for solutions, stopping at the second, and hands back the verdict,
/// the solutions found and the guesses made. Givens that are malformed or conflict are
/// a verdict of no solution, not a refusal.
///
/// # Errors
///
/// Throws an `Error` when `givens` is not an array of givens.
///
/// ```js
/// const { verdict, guesses, solutions } = search(givens);
/// verdict; // "OneSolution"
/// ```
#[wasm_bindgen(unchecked_return_type = "SearchResult")]
pub fn search(
    #[wasm_bindgen(unchecked_param_type = "Given[]")] givens: JsValue,
) -> Result<JsValue, JsValue> {
    let givens = self::givens(givens)?;
    types::board(&pawdoku::solver::search(givens))
}
