---
title: "Decision 0001: The engine is a library of its own"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_engine_as_a_library]
requires: []
---

# Decision 0001: The engine is a library of its own

## Context

The classic-sudoku engine was written in the game Pawdoku, as seven Allium modules that
specify the grid, the solver, the technique catalogue, reach, effort, lapse and the
human-solving model, with no code behind them yet. Read clause by clause, the seven carry
almost no game-only wording: a handful of lines name the game's Play surface or its
randomness port, and the game's root module is the only module that names a surface at
all. Everything else is compute: what a valid grid is, which technique applies where, how
hard a puzzle is for a person.

That compute has more than one consumer. The game reaches it through WebAssembly in a
browser. A command line wants it for batch analysis and for rating puzzles. Python
bindings want it for the analysis and modelling the human-solving specification invites.
Each is a sibling crate later; only the library is built by the founding tickets, and
every decision taken for it keeps the other three cheap.

## Decision

The engine is a library of its own: the crate `pawdoku` in `crates/pawdoku` of this
repository, a Cargo workspace from day one, so that the command-line, Python and
WebAssembly crates arrive as siblings without a restructuring. The seven modules move here
with it, adapted so that nothing game-only remains; the game's root module stays in the
game.

Two alternatives were considered. A TypeScript engine inside the game was the path of
least ceremony: one implementation for the one surface that exists today. It lost because
the solver and the human-solving model are compute the command line and any analysis also
need, and a second implementation in a second language drifts from the first the day it
is written. A Rust crate inside the game's repository would have kept one repository. It
lost because the game is rendered from the house template with a Node toolchain and a
managed file set, and a crate there would be gated by neither: no clippy, no coverage
floor, no `no_std` proof, and every Rust file a stranger to the template's inventory.

## Consequences

Two repositories hold shared truth. Allium has no cross-repository import, so the game
restates the clauses it needs and holds them equal to this repository's text by test on
its own side. Until the WebAssembly package ships the specifications inside itself, a
clause can change here and drift there with no gate on either side to notice. That is the
consequence that hurts first.

A second house toolchain. This repository carries its own CI, hooks, pins and handbook, in
the house shape but with the frontend tools replaced by Rust ones, and a fix to a shared
checker arrives here as a moved pin rather than as a template update.

A game release now couples to an engine release. A behaviour change the game needs is a
change here, a version, and a bump there, where before it was one commit.

## What would reopen this

The game remaining the only consumer after the bindings spikes close, which would make
the second repository pure overhead; or a consumer that needs the engine's source rather
than a package, which a separate library serves badly.

## Related pages

- [Purpose and scope](../project/purpose-and-scope.md)
- [Architecture](../explanation/architecture.md)
- [Specifications](../explanation/specifications.md)
