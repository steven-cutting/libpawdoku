---
id: T40
title: "Specification: play from a record, a board's operations on a record handed over and handed back"
status: open
depends_on: [T28]
parallel_with: [T13, T15, S12]
branch: ticket/t40-play-from-a-record-spec
estimated_size: S
---

# T40: Specification: play from a record, a board's operations on a record handed over and handed back

## Context

The game reaches the engine through WebAssembly (T15, T42), and Python will reach it
through pyo3 (T17). On 2026-10-31 the maintainer decided how a consumer drives a board,
asked with three choices: a long-lived `Board` object with methods; a small set of
stateless functions taking plain values in and giving plain values out, the caller
holding the record; or both. **The answer was both**, with the stateless functions
**implemented in Rust, so that TypeScript and Python use the same ones.**

The first way needs nothing new: `board.allium`'s `Playing` surface is a board and its
operations, and `pawdoku::board::Board` builds it (T27, T28). The second way is new
behaviour a consumer sees. A caller hands over a record and one of the board's
operations and is handed back the record after it, with what `Playing` would have shown;
or hands over a record alone and is handed what `Playing` shows. Nothing is held between
calls. `board.allium` says how a board is written down and had again (`contract
Recording`, `surface Reopening`) and what may be done to a board (`Playing`), but no
clause says that an operation may be put to a record. AGENTS.md invariant 1 says no rule
is re-decided in code and a behaviour the code needs that the specifications do not
state is a specification change first. This ticket is that change. T41 builds it in the
core, where both bindings reach it, because the bindings crates depend on the engine
and never on each other (`docs/project/repository-map.md` lines 11 to 13).

**What it composes, and why it adds no rule of its own.** Every part already has a
clause. Reopening a record puts its givens to `SetPuzzle` and makes the board whole
(`Recording.reopen`, `ReopeningIsExact`); an operation on that board is one of
`Playing`'s seven, under their guards; writing the board down changes nothing
(`WritingChangesNothing`); and a record carries everything reopening needs
(`ARecordIsEnoughOnItsOwn`). So the new clauses state a composition and its guarantees:
that it is the same as reopening, operating and writing; that a refusal, by reopening
or by the operation's guard, hands nothing back but the refusal; that what is shown is
what `Playing` exposes of the reopened board, so `TheSolutionIsNeverShown` and
`ACheckNeverTellsTheDigit` hold for it too; and that nothing is kept between calls.

**What it costs.** A board holds its puzzle's solution, which is why a `Board` does not
serialise and a `Record` holds none (`docs/explanation/architecture.md`, "What crosses
each boundary"). So every call reopens the record: one search for the givens, and every
move made again. S12 measures the cost; the specification states no budget, because the
search always concludes (`solver.allium`'s `AlwaysConcludes`) and the moves are finite.

**What does not need a clause.** The solver and the generator are already stateless.
`solver.allium`'s `Solving` takes givens and gives a `SearchResult`.
`generation.allium`'s `Generating` takes a tier and a stream of draws, and a stream begun
from a seed at index zero is what its Replay names; its `GenerationResult` exposes the
tier, the status, `draws_taken` and the givens, and not the solution. A binding that
generates from a seed and hands back those values states nothing new. If S12 finds that
the readings after a move need a shape `Playing` does not state (its question 8), that
shape is added here, before T41 builds it.

**An open question this change makes pressing.** With the stateless way the record is a
consumer's state between calls, and a game may keep it across a release of the engine.
`Record`'s documentation (`crates/pawdoku/src/board/record.rs` lines 51 to 54) says no
promise is made yet that a record written by one version of the engine reopens in
another, and a record carries no version. `board.allium` says nothing either way. That
is a product decision nobody has taken, so this ticket adds an `open question` block for
it and does not answer it.

Read first: `AGENTS.md`; `docs/explanation/specifications.md`, "Open questions are a
feature"; `docs/how-to/work-with-the-specs.md`; `docs/specs/board.allium` whole, above
all its Scope, `contract Recording`, `surface Playing` and `surface Reopening`;
`docs/specs/sudoku.allium`'s `PuzzleSetting`; the `.agents/skills/spec-change/SKILL.md`
and `tend` skills; decision 0014 (`docs/decisions/0014-board-imports-solver.md`), the
shape of a spec-change record; `docs/explanation/architecture.md`, "The board" and "The
record".

## Goal

`board.allium` states play from a record: what a caller hands over, what it is handed
back, the guarantees above, and a new open question on records across versions of the
engine. Both specification gates are clean with no waiver. A decision record says why
both ways exist and why the second lives in the core. T41 can build the second way from
this text alone.

## Non-goals

- No Rust: T41 builds it.
- No change to any existing rule, invariant, surface or contract of `board.allium`
  beyond the Scope's Includes and the line that counts the boundaries; no change to
  `sudoku.allium`, `solver.allium` or `generation.allium`.
- No format, and nothing about where a record is kept: still the caller's.
- No answer to the open question on checks, or to the new one on versions.
- No statement of a binding's names, casing or types: those are T41's and T42's.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `docs/specs/board.allium` | specification | Scope's Includes gains play from a record, and the boundaries become three; the new contract and surface (step 3); one `open question` block (step 4) |
| `docs/decisions/0021-play-from-a-record.md` | decision | New record (step 5); the number is the next free one on the day |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | T00 follow-up | The record's entry. Frozen under CONVENTIONS.md §11; step 1 |
| `docs/project/terminology.md` | project | A row for play from a record |
| `docs/explanation/specifications.md` | explanation | If it lists `board.allium`'s surfaces or counts its open questions, the count; nothing else |
| `CHANGELOG.md` | maintainer docs | Under `[Unreleased]`, Changed: the surface; the count of decision records |
| `tickets/README.md` | ticket index | T40's row set `done` |
| `tickets/T40-play-from-a-record-spec.md` | ticket | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t40-play-from-a-record-spec` from `main`. Run `just
   initialize` if `.pixi/` is absent and `just install-allium` if `.tools/bin/allium` is;
   each uses the network and this ticket authorises it. Run `just check-specs`,
   `just analyse-specs` and `just plan-spec board`, and quote their closing lines and the
   count of obligations. Confirm with the maintainer that `docs/manifest.yml` may be
   edited here, for the record's entry in the same commit; without it, stop.

2. **Put the Open points to the maintainer before writing a clause**, each with its
   proposal. Record each answer, dated, under "Open points settled". If S12 has
   finished, read its question 8 first and carry any shape it found unstated.

3. **The clauses**, written with the `tend` skill, in the module's voice and vocabulary
   (never pencil mark, never candidate for what a player writes):

   - **Scope.** An Includes bullet: play from a record, a board's operations put to a
     record a caller hands over, the record after it handed back with what `Playing`
     shows, and nothing kept between calls. The bullet "Two boundaries" becomes three,
     naming the new one. Excludes' "How a record is kept" stands as written.
   - **A contract** beside `Recording`, proposed name `PlayingFromARecord`, with three
     signatures: one that opens from givens and gives the record of the board opened;
     one that takes a record and one operation and gives the record after it; and one
     that takes a record and gives the board it reopens to, for reading. An operation is
     one of `Playing`'s seven, `Place`, `Erase`, `WriteMark`, `StrikeMark`, `CheckCell`,
     `Undo` and `Redo`, with the cell and digit each takes. How an operation is named as
     a value is the module's to choose under Allium 3; say in the hand-back notes what
     was chosen and why.
   - **Its invariants**, each named and each with its reason in a comment:
     - the record handed back is the record `write` gives of the board `reopen` gives,
       after the operation made through `Playing` under its guards: the same as holding
       the board, step for step;
     - a refusal hands back the refusal and no record: a record that does not reopen is
       refused as `reopen` refuses it, and an operation `Playing` does not offer is
       refused by its guard; the record handed over is a value and is unchanged;
     - what is shown is what `Playing` exposes of the reopened board, nothing more, so
       `TheSolutionIsNeverShown` and `ACheckNeverTellsTheDigit` hold;
     - nothing is kept between calls, and the outcome of a call depends on what is handed
       over and nothing else (`ARecordIsEnoughOnItsOwn` already says the record suffices).
   - **A surface**, proposed name `PlayingFromARecord`, that `fulfils` the contract and
     stands whether or not a board is open, as `Reopening` does. Its guarantees say the
     same of a caller: playing from records step by step and holding one board give
     records equal at every step, and the same readings, moves and checks.

   Run `just check-specs` and `just analyse-specs` after each clause, and fix every
   diagnostic and finding at its root; no `allium-ignore`.

4. **The open question**, under the existing one, in the module's voice: whether a
   record written by one version of the engine is promised to reopen in a later one,
   whether a record carries anything that names its version, and what a consumer is told
   when one does not reopen. Say in its comment that play from a record makes the record
   a consumer's state between calls, and that `Record`'s documentation makes no promise
   today.

5. **The decision record**, `docs/decisions/0021-play-from-a-record.md` in the shape of
   0014, `canonical_for: [decision_play_from_a_record]`; use the next free number on the
   day (0019 and 0020 are projected for T13 and T15; 0017 is T34's on its branch and
   0018 T37's) and rename the file, the title, the index row and the manifest entry if
   it is taken. Context: the maintainer's question and answer of 2026-10-31, with the
   three choices as asked. Decision: both ways; the stateless way specified in
   `board.allium` as a composition and built in the core, so that every binding shares
   it and none depends on another; no format in the core, values through the `serde`
   feature as `Record` already is. Consequences, including the ones that hurt: a search
   and a replay of every move on each stateless call; two ways to keep consistent, held
   equal by T41's tests; the version question now pressing. What would reopen it: a
   record that carries the solution (which `TheSolutionIsNeverShown` forbids today), or
   a consumer that needs neither way.

6. **The pages**, as Files touched lists. Then `just plan-spec board` again and record
   the new count of obligations, and the obligations the new clauses add, for T41's
   test list.

7. `just check-docs`, then `just check`, quoted. Set `status: done` and the index row.
   Commit, and ask before pushing.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` report no diagnostics and no findings for
  every module, with no waiver.
- `board.allium` states the contract, its four invariants, the surface and the new open
  question; its existing rules, invariants, surfaces and contract are unchanged.
- The module's `use` lines are unchanged.
- The decision record is in the manifest and the index.
- The obligations `just plan-spec board` adds are listed in the hand-back notes.
- `just check` is green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just check-specs
just analyse-specs
just plan-spec board
rg -n '^(contract|surface|open question|use)' docs/specs/board.allium
git diff --stat main -- docs/specs
just check-docs
just check
git status --porcelain
```

Expected: every module clean; the obligations; two contracts, three surfaces, two open
questions and two `use` lines; one specification file changed; the documents gate green;
`All checks passed and the worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

### Names T41 needs

### Deviations, and why

### Handed back

### Open points settled

## Open points

Each is the maintainer's; the proposal is the ticket writer's, of 2026-10-31.

- **The name.** Proposed: play from a record, `PlayingFromARecord` for the contract and
  the surface. Other words in use are taken: replay is `human-solving.allium`'s and
  `ExactReplay`'s; session suggests something held, which this is not.
- **What a call hands back beside the record.** Proposed: what `Playing` exposes of the
  board after the operation, so one call is enough to redraw; and for `CheckCell`, the
  check it added, which is the last of the board's checks. The other choice hands back
  the record alone and leaves the reading to a second call, which reopens a second time.
- **Whether opening from givens belongs to the surface.** Proposed: yes, so a caller that
  never holds a board can start one; it is `OpenBoard` and `write`, composed. The other
  choice leaves the first record to the `Board` way, so a stateless caller needs both.
- **Records across versions of the engine.** Not answered: step 4 adds it as an open
  question. If the maintainer answers it now, the answer is a specification change of its
  own, with its own clause and probably a field in the record, and it is not this
  ticket's.
- **Paths other tickets also edit.** T37, open, edits `docs/decisions/README.md`,
  `docs/manifest.yml`, `docs/explanation/specifications.md`, `docs/project/terminology.md`
  and `CHANGELOG.md`, and takes decision 0018; T13 and T15 take decision records too.
  Each shared edit is a row, an entry or a sentence; whichever merges second merges
  `main` first and takes the next number, the deviation from CONVENTIONS.md §11 that T31
  and T33 recorded.
