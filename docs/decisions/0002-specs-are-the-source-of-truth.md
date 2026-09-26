---
title: "Decision 0002: Specifications decide behaviour"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_spec_first]
requires: []
---

# Decision 0002: Specifications decide behaviour

*Carried from Pawdoku's decision 0003 at `78d03cdf`, and restated for the library the game's engine moved to. Pawdoku's own record stands where it is.*

## Context

This library's behaviour is written in Allium before the code that implements it: the
eight engine modules under `docs/specs/`, which moved here from the game with it. The
question is whether they remain authoritative once implementation starts, or become a
design document that quietly falls behind.

A solver is a good argument for the former. The rule that decides whether a technique
applies has a well-known trap somewhere in it, and the difference between a correct
implementation and a plausible one is a few lines. Left in prose, that detail is lost in
the first refactor.

## Decision

The specifications are the source of truth for behaviour. `AGENTS.md` states the split:
when deciding *what* the engine should do, the specifications win; when deciding *how* to
build it, `AGENTS.md` wins.

In practice: behaviour changes in the specification first, then in the tests, then in the
code. No rule, guard or threshold the specifications state is re-decided in code. A test
is never weakened to make it pass — the specification is corrected and the tests are
re-derived.

## Consequences

Contracts become testable obligations rather than intentions. A surface's `@guarantee`
clauses name what it owes, and a test asserts each one directly, including the edge cases
a plausible implementation gets wrong.

Unresolved product decisions stay visible. They are recorded as `open question` blocks and
answered by someone entitled to answer them rather than by whoever writes the code first.

The costs are real. Every behaviour change is two edits, not one. The project-managed
`allium` binary — [Decision 0005](0005-project-managed-allium-cli.md) — holds every
module to parsing and analysing cleanly, but nothing mechanical holds the code to a
clause, so drift between a clause and its implementation is still caught by review.

The modules are read in two repositories. The game restates the clauses it needs and
holds them equal to this repository's text by test on its own side, so a clause changed
here without a matching change there is a drift no gate in this repository sees. Until
the specifications ship inside the package the game consumes, that text is shared truth
held equal by hand.

## What would reopen this

If the specifications stopped being maintained — if a behaviour change landed in code
without a matching edit and nobody noticed — the honest response would be to distil them
back from the implementation and restart, or to abandon the approach outright rather than
keep a document that lies.

## Related pages

- [Specifications](../explanation/specifications.md)
- [Work with the specifications](../how-to/work-with-the-specs.md)
