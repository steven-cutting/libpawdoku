---
name: elicit
description: "Run a structured discovery session to build an Allium specification through conversation. Use when the user wants to create a new spec from scratch, elicit or gather requirements, capture domain behaviour, specify a feature or system, define what a system should do, or is describing functionality and needs help shaping it into a specification."
---

This skill will run a structured discovery session that ends in an Allium specification,
within the rules `AGENTS.md` sets for the specifications. It is vendored from the
`juxt/allium` skill of the same name, which Pawdoku carries, and ticket T05 writes its
full procedure. Until then, follow `AGENTS.md` and prove the work with
`just check-specs` and then `just check`.
