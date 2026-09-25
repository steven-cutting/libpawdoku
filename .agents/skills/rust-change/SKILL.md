---
name: rust-change
description: Implement or review a change to the engine with the trait boundary, no_std, tests and documentation evidence the invariants require.
---

# Change the engine

1. Read `AGENTS.md` and `docs/explanation/architecture.md`. Identify which module of `crates/pawdoku` the change lands in and whether it touches the public API; a public item is a contract the Python and WebAssembly crates will build on, so its shape is part of the change.
2. Find the clause. Every behaviour has an owning module under `docs/specs/`; read the rule, its guards and its outcomes before writing code. If no clause says what you need, this is a `spec-change` first, not a judgement call in code.
3. Put any effect behind the trait boundary. The only effect the engine has is randomness, reached through the trait `crates/pawdoku/src/random.rs` defines and its fake. A change that wants a clock, a thread, the filesystem or the environment has found a design problem, not a missing import: the core is `#![no_std]` and cannot name them. Limits are step budgets, never timeouts.
4. Derive the test from the clause before implementing, and confirm it fails first. A unit test lives in the module beside the code it tests; an integration test under `crates/pawdoku/tests/` drives the public API through the fake; a public item carries a doctest that asserts, because doc examples are tests. A test already green proves nothing about the new behaviour.
5. Import from `core` and `alloc`, never `std`; `#[cfg(test)] extern crate std;` is the one exception and it stays under `cfg(test)`. Every public type is `Send + Sync + 'static`, `Clone` and `Debug`; public enums and structs are `#[non_exhaustive]`; errors are `thiserror` enums whose `Display` text is stable.
6. Silence a lint only with `#[expect(lint, reason = "...")]` on the item it names, never a blanket `allow`, and never write `unsafe`; the crate forbids it.
7. Run `just fmt-check`, `just clippy`, `just test` and `just wasm-check`, then `just check` before handoff. A bare `std::` path already fails `clippy` and `test` on the host; `wasm-check` is what catches a dependency that needs std, or a `cfg`-gated path that only ever compiled there.
8. Report by `file:line`: what changed, which clause it implements, and which test proves it.
