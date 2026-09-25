---
name: rust-change
description: Implement or review a change to the engine with the trait boundary, no_std, tests and documentation evidence the invariants require.
---

This skill will implement or review a change to the engine: read `AGENTS.md` and the
architecture page, design the trait boundary first, derive tests from the clause, and
treat documentation examples as tests. It is new in this repository, and ticket T05
writes its full procedure. Until then, follow `AGENTS.md` and prove the work with
`just clippy` and then `just check`.
