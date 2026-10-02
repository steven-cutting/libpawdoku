//! The Pawdoku engine: classic sudoku, specified in `docs/specs/` and built here.
//!
//! This crate is `no_std` with `alloc`: it has no clock, no threads, no filesystem and
//! no source of randomness of its own. What it needs from the outside world it takes
//! through traits, so that it runs the same in a browser, in Python and on the command
//! line. Randomness is the one effect, and [`random`] is its boundary: a seeded stream
//! of draws the caller supplies. See `docs/explanation/architecture.md`.
//!
//! Behaviour arrives one module to a specification module. [`sudoku`] is the first: the
//! rules, from `docs/specs/sudoku.allium`, beneath every module to come. Today it
//! exports the grid's two figures and the two value types; its puzzle stays inside the
//! crate until the solver can make the proof a puzzle is set from.
//!
//! **Not final.** This is a draft: the public items, their names and their signatures
//! may change as implementation continues.

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod random;
pub mod sudoku;
