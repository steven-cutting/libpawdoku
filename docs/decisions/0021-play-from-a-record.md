---
title: "Decision 0021: Play from a record, beside the board"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_play_from_a_record]
requires: []
---

# Decision 0021: Play from a record, beside the board

## Context

The game reaches the engine through WebAssembly, and Python will reach it through pyo3.
On 2026-10-31 the maintainer was asked how a consumer drives a board, with three
choices: a long-lived `Board` object with methods; a small set of stateless functions
taking plain values in and giving plain values out, the caller holding the record; or
both. The answer was both, with the stateless functions built in Rust, so that
TypeScript and Python use the same ones.

The first way needed nothing new: `board.allium`'s `Playing` surface is a board and its
operations, and `pawdoku::board::Board` builds it. The second was behaviour a consumer
sees that no clause stated. `board.allium` said how a board is written down and had
again (`contract Recording`, `surface Reopening`) and what may be done to a board
(`Playing`), but not that an operation may be put to a record. Decision 0002 makes that
a specification change first.

Three further questions were put with a proposal each, and each proposal was taken:

- **The name.** The contract and the surface are both `PlayingFromARecord`, and the
  behaviour is play from a record.
- **What a call hands back.** The record after the operation and the view: what
  `Playing` exposes of the board after it, so that one call is enough to redraw. For a
  check, the check it added comes back too.
- **Opening from givens.** It is one of the stateless set, so a caller that never holds
  a board can start one.

A fourth, whether a record reopens across versions of the engine, was answered then too:
a record names its format version and no compatibility is promised. That is its own
clause and its own record, after this one.

The spike on the TypeScript surface then measured the view against the readings after
each move. Carried inside every call, the readings of 200 moves came to 700,193 bytes of
JSON against 24,099 for the view without them, and a caller redrawing reads none of
them.

## Decision

1. **Both ways.** A consumer may hold a `Board` and put operations to it, or hold a
   record and put each operation to it in a call that keeps nothing.
2. **The stateless way is specified in `board.allium` as a composition.** The contract
   `PlayingFromARecord` has four signatures: `open` from givens, `play` one operation on
   a record, `read` a record, and `reading_after`, the board as it stood after one move
   of a record. Each is clauses that stand already, `SetPuzzle` and `OpenBoard`,
   `Recording`'s `write` and `reopen`, and one of `Playing`'s seven operations under
   its guard, and its invariants say so: playing from a record is holding the board,
   step for step; a refusal hands back nothing else; what is shown is what `Playing`
   shows; nothing is kept between calls. The surface of the same name stands whether or
   not a board is open, as `Reopening` does, and guarantees that a caller playing from
   records and one holding a board read the same at every step.
3. **Each call hands back the record and the view**, and the readings after a move are
   given on asking, one move at a time, through `reading_after`. That is the spike's
   finding, carried into the clause.
4. **Built in the core.** The bindings crates depend on the engine and never on each
   other, so a stateless function written in one binding would be written again in the
   next. In the core every binding shares it.
5. **No format in the core.** The values cross as `Record` already does, through the
   `serde` feature. Where a record is kept stays the caller's.

## Consequences

Every stateless call reopens its record: one search for the givens, and every move made
again under the board's guards. A caller that plays a long game from records pays that
on each move, and a caller that scrubs through a game's readings pays it on each reading;
the spike measured 137.7 ms for 200 readings by record against 3.8 ms on a held board.
A caller for whom that matters holds a board.

There are two ways to keep consistent. The surface's guarantees say they agree, and the
tests that build the second way hold them equal step for step.

The record becomes a consumer's state between calls, which a game may keep across a
release of the engine. That made the version question pressing, and it is answered by
the record naming its format version.

The contract's signatures return a `Board` and name the record and the view in prose.
A value holding the record, the board and the check was drafted, and the pinned checker
reports a value named only as a return type as unused. The prose is the clause; a
consumer's value is the binding's to name.

## What would reopen this

A record that carries the solution, which would remove the search from every call and
which `TheSolutionIsNeverShown` forbids today. A consumer that needs neither way. Or a
measured cost of reopening on every call that a consumer cannot meet by holding a board.

## Related pages

- [Decision 0014: The board imports the solver, and well-posedness is a proof value](0014-board-imports-solver.md)
- [Specifications](../explanation/specifications.md)
- [Architecture](../explanation/architecture.md)
