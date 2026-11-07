//! The basic generator, `generation.allium`'s `GenerationResult`: a puzzle of a tier
//! from a seed, or from a [`RandomStream`] the caller holds, with what replays it. The
//! solution is never shown. Exporting these answers nothing of where puzzles are made,
//! which `generation.allium`'s first open question leaves open.
//!
//! The result crosses through the `bigint` serialiser: its seed, start index and draw
//! count are `bigint`s, so `JSON.stringify` throws on it. A refusal, on either path,
//! throws a `GenerateError` that carries `tier` and `draws_taken`.

use pawdoku::generation::{GENERATION_VERSION, Tier, generate_from_stream};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::RandomStream;
use crate::error::refused;
use crate::types;

/// Generates a puzzle of `tier` from `stream`, which is advanced in place by the draws
/// taken: its `index` moves on by `draws_taken`, and a refusal leaves it past the draws
/// it gave, because they stay taken. The result's `seed` is `null`, and its
/// `start_index` is where the stream stood.
///
/// # Errors
///
/// Throws a `GenerateError`, with `tier` and `draws_taken`, when generation is refused,
/// a replay stream running out among the reasons; and an `Error` when `tier` is not a
/// tier.
///
/// @throws {GenerationRefused} When generation is refused; the stream keeps the draws.
///
/// ```js
/// const stream = RandomStream.seeded(2024n);
/// const { givens, draws_taken } = generate("Three", stream);
/// stream.index === draws_taken; // true
/// ```
#[expect(
    clippy::doc_markdown,
    reason = "a JSDoc @throws names a TypeScript type, unquoted"
)]
#[wasm_bindgen(unchecked_return_type = "GenerationResult")]
pub fn generate(
    #[wasm_bindgen(unchecked_param_type = "Tier")] tier: JsValue,
    stream: &mut RandomStream,
) -> Result<JsValue, JsValue> {
    let tier: Tier = types::read_held(tier)?;
    let result =
        generate_from_stream(tier, stream.as_dyn()).map_err(|refusal| refused(&refusal))?;
    types::generation(&result)
}

/// Generates a puzzle of `tier` from `seed`, a `bigint` from `0n` to `2n ** 64n - 1n`;
/// one outside that range is reduced modulo `2n ** 64n` silently, and the result
/// carries the seed as reduced. The same tier and seed give the same puzzle under the
/// same two versions.
///
/// # Errors
///
/// Throws a `GenerateError`, with `tier` and `draws_taken`, when generation is refused,
/// which no seed is known to be; and an `Error` when `tier` is not a tier.
///
/// @throws {GenerationRefused} When generation is refused.
///
/// ```js
/// const result = generateFromSeed("Three", 0n);
/// localStorage.setItem("seed", String(result.seed));
/// ```
#[expect(
    clippy::doc_markdown,
    reason = "a JSDoc @throws names a TypeScript type, unquoted"
)]
#[wasm_bindgen(js_name = generateFromSeed, unchecked_return_type = "GenerationResult")]
pub fn generate_from_seed(
    #[wasm_bindgen(unchecked_param_type = "Tier")] tier: JsValue,
    seed: u64,
) -> Result<JsValue, JsValue> {
    let tier: Tier = types::read(tier)?;
    let result =
        pawdoku::generation::generate_from_seed(tier, seed).map_err(|refusal| refused(&refusal))?;
    types::generation(&result)
}

/// The name of the generator, `"generation-1"`, the engine's `GENERATION_VERSION`.
/// Keep it beside a seed: a seed replays only under the generator and the random
/// stream it was generated with. A function, not a constant, because wasm-bindgen
/// exports no constant.
///
/// ```js
/// generationVersion(); // "generation-1"
/// ```
#[wasm_bindgen(js_name = generationVersion)]
#[must_use]
pub fn generation_version() -> String {
    GENERATION_VERSION.into()
}
