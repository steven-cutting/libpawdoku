---
title: "Purpose and scope"
kind: "project"
audience: [user, contributor, maintainer, agent]
canonical_for: [project_purpose, project_non_goals]
requires: []
---

# Purpose and scope

`pawdoku` is the classic-sudoku engine behind Pawdoku, a Biscuit Games game. It is a Rust
library crate at `crates/pawdoku`, `no_std` with `alloc`, licensed Apache-2.0, in a Cargo
workspace that will hold its bindings as sibling crates. What this repository decides is
the rules and the reasoning about them; what a player sees is a consumer's to decide.

## What it does

It states the rules of classic Sudoku (`sudoku.allium`): a grid, its givens, the
conflicts a placement may leave and when a puzzle is solved. It states the search that
says whether a set of givens is well-posed, with exactly one solution (`solver.allium`),
and a catalogue of 29 named techniques a person solves with (`technique.allium`). On top
of those sit four models of a player: three without chance (`reach.allium`,
`effort.allium`, `lapse.allium`) and one with it (`human-solving.allium`). The models
exist for two things a consumer asks for: a difficulty rating that means something to a
person, and a hint pitched at the player who asked. `board.allium` keeps what a puzzle in
play needs beyond the rules: the notes, the moves, undo and the check.

What the engine does is stated first in `docs/specs/`, one rule at a time, and built
second; see [Specifications](../explanation/specifications.md). A behaviour that no
module states is not the engine's yet, however obvious it looks.

## What it deliberately does not do

- **No surface.** Nothing here draws, and no module says how a puzzle looks.
  `sudoku.allium` leaves whether a surface refuses, flags or allows a conflicting
  placement to the consumer: the rules allow it, and nothing here draws.
- **No persistence.** Nothing is remembered between calls. `board.allium` says what a
  record of a puzzle in play holds, as a plain value; a consumer keeps what it wants kept,
  where it wants it kept.
- **No puzzle generation yet.** How a setter finds givens is excluded from
  `sudoku.allium`. Generation is a planned module, `generation.allium`, specified before
  it is built and today a skeleton of scope, config and open questions; whether puzzles
  are made on the device, ahead of time, or both is open
  ([decision 0012](../decisions/0012-generation-and-dev-time-judges.md)). What a
  generated puzzle is held to is in
  [Puzzle design objectives](../explanation/puzzle-design.md).
- **No variants.** Other sizes, irregular boxes, diagonals, cages and overlapping grids
  are each a different game, and `sudoku.allium` excludes them by name.
- **No clock, threads, filesystem or network.** The core is `no_std`, so they are
  unnameable rather than merely avoided. Randomness is the one effect, reached through
  the randomness boundary: a stream of draws the caller supplies. See
  [Architecture](../explanation/architecture.md) and
  [Security model](../explanation/security-model.md).

## Who it is for

Consumers, not players. The game reaches the engine through a WebAssembly crate; a
command line runs it from a shell; Python reaches it through a bindings crate. Each
decides what a player is shown, and none re-decides a rule. Today only the library
exists; the three are sibling crates to come in this workspace
([decision 0001](../decisions/0001-engine-as-a-library.md)).

## Related pages

- [Repository map](repository-map.md)
- [Terminology](terminology.md)
- [Architecture](../explanation/architecture.md)
- [Specifications](../explanation/specifications.md)
