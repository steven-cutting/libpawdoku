---
title: "Decision 0008: A pure no_std core"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_no_std_core]
requires: []
---

# Decision 0008: A pure no_std core

## Context

The engine is built to be consumed from a browser through wasm-bindgen, from Python
through pyo3 and from a command line, and each asks something of a type before it can
cross the boundary: pyo3 wants types it can hold from any thread, wasm-bindgen wants no
part of `std` the browser cannot provide, and both build their exceptions from an error's
text. Building for a target with no standard library at all is what makes those
properties mechanical rather than conventional. `AGENTS.md` states them as invariants;
this record says why they were chosen and what they cost.

## Decision

A list a reader can check against `Cargo.toml` and `crates/pawdoku/src/lib.rs`:

- `#![no_std]` with `extern crate alloc` in the core, and `#![forbid(unsafe_code)]` in
  every crate through the workspace lint table.
- `cargo check --target wasm32v1-none` under the feature powerset as the mechanical proof:
  a target with no std at all, beside `wasm32-unknown-unknown`, which wasm-bindgen
  targets. `just wasm-check` runs both and is gate 9 of `just check`.
- `rand` and `getrandom` banned by cargo-deny; the seed comes from the caller through the
  randomness boundary (decision 0003).
- Every public type is `Send + Sync + 'static`, `Clone` and `Debug`, asserted in
  `crates/pawdoku/tests/api_bounds.rs`, so a type that loses a bound fails a test here
  rather than a build in a binding.
- Public enums and structs are `#[non_exhaustive]`; clippy's `exhaustive_enums` and
  `exhaustive_structs` flag the ones that are not.
- No panics in the public surface: clippy's `unwrap_used`, `expect_used`, `panic`,
  `unreachable`, `todo` and `unimplemented` are errors at gate 6, relaxed inside tests by
  `clippy.toml`.
- Errors are `thiserror` enums with stable `Display` text, because pyo3 and wasm-bindgen
  build their exceptions from that text; it is the cross-language contract, and a change
  to it is a breaking change.
- Features: `default = []`, `serde` optional and additive, and nothing else.
- `usize` is 32 bits on wasm, so a count that must agree across targets is `u32` or
  `u64`, never `usize`.
- Float maths, should a rating model need `ln`, `exp` or `sqrt`, comes from `libm`, never
  from `std`.

## Consequences

`alloc::vec::Vec`, `alloc::string::String` and `BTreeMap` in place of `HashMap`, because
`alloc` has no hasher; `clippy.toml` lists both hashed collections as disallowed types
with the reason. `#[cfg(test)] extern crate std;` so that proptest runs in the unit tests.
`core::error::Error` rather than `std::error::Error`, which is why `thiserror` is taken
without its default features. The `std_instead_of_core`, `std_instead_of_alloc` and
`alloc_instead_of_core` lints on every file, so a `std::` path that would compile under
test fails clippy. Float maths through `libm` or avoided. And a step-budget limit wherever
a timeout would have been simpler, because there is no clock to time out against.

## What would reopen this

A need for a std-only API in the core: a clock, files, threads, or a hashed map for speed.
The answer would be an additive `std` feature or a sibling crate that holds the std-only
part, not `std` in the core, so the reopening would be about where the line sits rather
than whether there is one. Or rustup dropping `wasm32v1-none`, which would change the
proof, not the decision: `wasm32-unknown-unknown` ships a standard library, so
`#![no_std]` there only drops the implicit import and an explicit `std` path or a
std-using dependency would still compile, and the proof would need another target
without `std`.

## Related pages

- [Architecture](../explanation/architecture.md)
- [API reference](../reference/api.md)
- [Configuration](../reference/configuration.md)
