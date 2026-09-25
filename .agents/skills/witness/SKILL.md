---
name: witness
description: "Independently witness that an Allium loop's convergence claim is true and was reached honestly. Use when the user wants to verify a loop's self-report, confirm tests really pass and no generated test was weakened, produce a convergence certificate or witness record, gate CI on a trustworthy signal, or check that an autonomous run did not cheat its way to green."
---

This skill will independently confirm that a loop's claim of convergence is true: the
tests pass and none was weakened, as `AGENTS.md` requires. It is vendored from the
`juxt/allium` skill of the same name, which Pawdoku carries, and ticket T05 writes its
full procedure. Until then, follow `AGENTS.md` and prove the work with `just test` and
then `just check`.
