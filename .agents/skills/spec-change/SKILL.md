---
name: spec-change
description: Change an Allium specification and carry the change through the tests and the implementation.
---

This skill will change an Allium module under `docs/specs/` and carry the change through
the randomness boundary, the tests and the engine, following the order `AGENTS.md` sets.
It is carried from the Pawdoku skill of the same name, and ticket T05 writes its full
procedure. Until then, follow `AGENTS.md` and prove the work with `just check-specs` and
then `just check`.
