//! The values JavaScript catches: one conversion from each of the engine's refusals to
//! a JavaScript `Error` whose `name` is the refusal's Rust type and whose `message` is
//! always the refusal's `Display` text (S12, question 2). A game branches on `e.name`
//! with no need to read the message; the message is the engine's stable text.
//!
//! A value of the wrong shape, which serde refuses before the engine sees it, throws
//! serde-wasm-bindgen's own `Error`, named `Error`, whose text is serde's and not stable.

use core::fmt::Display;

use js_sys::{Error, Object, Reflect};
use pawdoku::board::{FromRecordError, PlayError, ReopenError};
use pawdoku::generation::{GenerateError, GenerationRefused};
use pawdoku::random::RandomError;
use pawdoku::solver::SolveError;
use wasm_bindgen::JsValue;

use crate::types;

/// A refusal of the engine's, with the name JavaScript sees it under.
pub(crate) trait Refusal: Display {
    /// The `name` of the `Error` thrown for it: the Rust type's name, or for
    /// [`FromRecordError`] the name of the stage that refused.
    fn name(&self) -> &'static str;
}

impl Refusal for PlayError {
    fn name(&self) -> &'static str {
        "PlayError"
    }
}

impl Refusal for ReopenError {
    fn name(&self) -> &'static str {
        "ReopenError"
    }
}

impl Refusal for SolveError {
    fn name(&self) -> &'static str {
        "SolveError"
    }
}

impl Refusal for GenerateError {
    fn name(&self) -> &'static str {
        "GenerateError"
    }
}

impl Refusal for RandomError {
    fn name(&self) -> &'static str {
        "RandomError"
    }
}

/// The stage that refused, so that a game branches the same way whichever way it
/// drives the board; the index that names no move, and any variant added later, keep
/// the type's own name.
impl Refusal for FromRecordError {
    fn name(&self) -> &'static str {
        match self {
            Self::Reopen(_) => "ReopenError",
            Self::Play(_) => "PlayError",
            _ => "FromRecordError",
        }
    }
}

/// The `Error` JavaScript catches for `refusal`: named for it, its message the
/// refusal's `Display` text.
pub(crate) fn thrown<R: Refusal>(refusal: &R) -> JsValue {
    let error = Error::new(&refusal.to_string());
    error.set_name(refusal.name());
    error.into()
}

/// The `Error` JavaScript catches for a refused generation, on either path: named
/// `GenerateError`, its message the core's text, with `tier` (a string) and
/// `draws_taken` (a `bigint`) as own properties. They are defined, not set, so they
/// are not enumerable and `JSON.stringify` of the error does not meet the `bigint`.
/// The accessors are read with no `match`, so no variant the core adds later can reach
/// JavaScript without both.
pub(crate) fn refused(refusal: &GenerationRefused) -> JsValue {
    let error = Error::new(&refusal.to_string());
    error.set_name("GenerateError");
    let tier = types::board(&refusal.tier()).unwrap_or(JsValue::NULL);
    define(&error, "tier", &tier);
    define(&error, "draws_taken", &JsValue::from(refusal.draws_taken()));
    error.into()
}

/// Defines `key` on `target` as a read-only property that is not enumerable.
fn define(target: &Object, key: &str, value: &JsValue) {
    let descriptor = Object::new();
    // Setting a property on a plain object just made cannot fail.
    if Reflect::set(&descriptor, &JsValue::from_str("value"), value).unwrap_or(false) {
        Object::define_property(target, &JsValue::from_str(key), &descriptor);
    }
}
