---
name: distill
description: "Extract an Allium specification from an existing codebase. Use when the user has existing code and wants to distil behaviour into a spec, reverse engineer a specification from implementation, generate a spec from code, turn implementation into a behavioural specification, or document what a codebase does in Allium terms."
---

This skill will extract an Allium specification from existing engine code, within the
rules `AGENTS.md` sets for the specifications. It is vendored from the `juxt/allium`
skill of the same name, which Pawdoku carries, and ticket T05 writes its full procedure.
Until then, follow `AGENTS.md` and prove the work with `just analyse-specs` and then
`just check`.
