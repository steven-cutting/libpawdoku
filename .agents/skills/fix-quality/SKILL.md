---
name: fix-quality
description: Diagnose and repair a failing quality gate at its root instead of suppressing the finding.
---

This skill will find the root cause of a failing gate and repair it rather than suppress
the finding, gate by gate in the order `AGENTS.md` and the gate list give. It is carried
from the Pawdoku skill of the same name, and ticket T05 writes its full procedure. Until
then, follow `AGENTS.md` and prove the work with `just fix` and then `just check`.
