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

These are decisions about *how* this library is built. Decisions about *what* it does
belong in the specifications under `docs/specs/`, and unresolved ones are recorded there
as `open question` blocks — see [Specifications](../explanation/specifications.md).

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
| [0012](0012-generation-and-dev-time-judges.md) | Generation is the engine's; judges are development-time only |
| [0013](0013-metrics-gate.md) | The metrics gate |
| [0014](0014-board-imports-solver.md) | The board imports the solver, and well-posedness is a proof value |
| [0015](0015-basic-generator.md) | A basic generator from a published method, as the yardstick |
| [0016](0016-api-reference-on-github-pages.md) | The API reference on GitHub Pages |

Decision 0004 is superseded in part by decision 0011, which names the consequences of
0004 it replaces and the ones that stand. Decision 0013 reopens the gate list decision
0009 records, for one recipe.

## The numbering

This series is the repository's own and starts at 0001. Four entries were carried from
Pawdoku's decision records at `78d03cdf` when the engine moved here (0002, 0003, 0004 and
0005); each says so under its heading and keeps the topic slug it had, because the slug
is what a cross-repository reference names, and the game restates the engine's clauses
and cites these records by slug. Nothing here is rendered from a template, so no frozen
inventory of numbers applies. The next decision this repository takes is 0017.

## Writing a new one

Copy the shape of an existing entry: context, the decision, the consequences including
the ones that hurt, and what would reopen it. Add the file, add a manifest entry, add a
row above. A decision nobody can find is not recorded.

## Related pages

- [Documentation map](../README.md)
- [Architecture](../explanation/architecture.md)
