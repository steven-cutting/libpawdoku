---
name: code-review
description: Review a change in this repository against its invariants, specifications, tests, and documentation contract.
---

# Review a change

1. Read `AGENTS.md` and the specification module the change touches. Review the whole diff, not the summary of it.
2. Check the invariants in `AGENTS.md` one at a time. Specs decide behaviour, randomness behind the boundary and no other effect, `Send + Sync` and `#[non_exhaustive]` on every public type, no `unsafe`, both wasm targets, pedantic clippy with `expect` only, the coverage floor intact, `Cargo.lock` the pin.
3. Check the change against the specification it implements. A rule, guard or threshold decided in code rather than in `docs/specs/` is a finding even when the behaviour looks right.
4. Check the test evidence. A test that only asserts a function was called is not evidence; a test that supplies draws through the fake and asserts what the engine did with them is. A public item without a doctest that asserts is a finding.
5. Check the boundaries. Nothing outside `crates/pawdoku/src/random.rs` may name a source of draws, nothing in the core may reach `std`, and no test may replace the boundary with a global or a thread-local.
6. Check documentation ownership. A durable fact belongs on the page that owns its topic in `docs/manifest.yml`, added there rather than restated.
7. Check scope. Unrelated refactors, new dependencies and speculative abstractions are findings in themselves.
8. Report findings by severity with `file:line`, what breaks, and the smallest fix. Run `just check` before concluding.
