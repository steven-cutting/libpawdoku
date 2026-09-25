---
name: propagate
description: "Generate tests from Allium specifications. Use when the user wants to propagate tests, generate test files from a spec, write tests for a specification, create property-based tests, produce state machine tests, check test coverage against spec obligations, or understand what tests a specification requires."
---

This skill will generate tests from the Allium specifications and place them where
`AGENTS.md` says the engine's tests live. It is vendored from the `juxt/allium` skill of
the same name, which Pawdoku carries, and ticket T05 writes its full procedure. Until
then, follow `AGENTS.md` and prove the work with `just test` and then `just check`.
