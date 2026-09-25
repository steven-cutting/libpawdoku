---
title: "Architecture decisions"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_index]
requires: []
---

# Architecture decisions

A record of what was chosen, what it cost, and what would have to change for the choice
to be revisited. Each entry is numbered and never renumbered; a decision that is
superseded is marked rather than deleted, because the reasoning is what makes the
successor legible.

These are decisions about *how* the engine is built. Decisions about *what* it does belong
in the specifications under `docs/specs/`, and unresolved ones are recorded there as
`open question` blocks — see [Specifications](../explanation/specifications.md).

## The record

| Number | Decision |
| --- | --- |
| [0001](0001-engine-as-a-library.md) | The engine is a library of its own |
| [0002](0002-specs-are-the-source-of-truth.md) | Specifications decide behaviour |
| [0003](0003-effects-behind-traits.md) | Effects behind traits |
| [0004](0004-hook-runner-and-checkers.md) | Hook runner and checkers |
| [0005](0005-project-managed-allium-cli.md) | A project-managed Allium binary |
| [0006](0006-apache-2-0.md) | Apache-2.0 |
| [0007](0007-dependency-policy.md) | Dependency policy for a library |
| [0008](0008-no-std-core.md) | A pure no_std core |
| [0009](0009-rust-quality-gate.md) | The Rust quality gate |
| [0010](0010-skill-reference-material.md) | Skill reference material beside the skills |
| [0011](0011-tool-manager.md) | Tool manager |

## The numbering

The series is this repository's own and starts at 0001; it does not continue Pawdoku's.
Several entries were carried from Pawdoku's decision records when the engine moved here,
and each of those says under its heading which Pawdoku record it came from and keeps the
topic slug it had, because the slug is what a cross-repository reference names. A new
decision takes the next free number; the next decision is 0012.

## Writing a new one

Copy the shape of an existing entry: context, the decision, the consequences including
the ones that hurt, and what would reopen it. Add the file, add a manifest entry, add a
row above. A decision nobody can find is not recorded.

## Related pages

- [Documentation map](../README.md)
- [Architecture](../explanation/architecture.md)
