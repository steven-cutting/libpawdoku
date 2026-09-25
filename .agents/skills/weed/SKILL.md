---
name: weed
description: "Weed the Allium garden. Find where Allium specifications and implementation code have diverged, and help resolve the divergences. Use when the user wants to check spec-code alignment, compare specs against implementation, audit for spec drift or violations, sync specs with code or code with specs, or verify whether the implementation matches what the spec says."
---

This skill will find where the specifications and the engine have diverged and resolve
each divergence the way `AGENTS.md` says the specifications win. It is vendored from the
`juxt/allium` skill of the same name, which Pawdoku carries, and ticket T05 writes its
full procedure. Until then, follow `AGENTS.md` and prove the work with
`just analyse-specs` and then `just check`.
