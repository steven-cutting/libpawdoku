---
id: T40
title: "Specification: play from a record, a board's operations on a record handed over and handed back"
status: done
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

**Settled by the maintainer, 2026-10-31.** Four questions this ticket raised were put
to the maintainer, each with the ticket writer's proposal, and each proposal was taken:

- **The name.** The contract and the surface are both `PlayingFromARecord`, and the
  behaviour is play from a record.
- **What a call hands back.** An operation hands back the record after it **and** the
  view: what `Playing` exposes of the board after the operation, so one call is enough
  to redraw. For `CheckCell`, the check it added comes back too, which is the last of
  the board's checks.
- **Opening from givens.** It is part of the stateless set, so a caller that never
  holds a board can start one; it is `OpenBoard` and `write`, composed.
- **Records across versions of the engine.** A record names its format version, a
  record of another version is refused with a clear message, and no compatibility is
  promised. That is a specification change of its own, with its own clause and a field
  in the record, and it is T43's (`tickets/T43-record-version.md`), which runs after
  this ticket.

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

**Records across releases, answered and left to T43.** With the stateless way the
record is a consumer's state between calls, and a game may keep it across a release of
the engine. `Record`'s documentation (`crates/pawdoku/src/board/record.rs` lines 51 to
54) says no promise is made yet that a record written by one version of the engine
reopens in another, and a record carries no version; `board.allium` says nothing either
way. The maintainer has answered that (above): a record names its format version, and
nothing is promised across versions. T43 writes that clause into `Recording` and builds
it, so this ticket states nothing about versions, and adds no `open question` block.

Read first: `AGENTS.md`; `docs/explanation/specifications.md`, "Open questions are a
feature"; `docs/how-to/work-with-the-specs.md`; `docs/specs/board.allium` whole, above
all its Scope, `contract Recording`, `surface Playing` and `surface Reopening`;
`docs/specs/sudoku.allium`'s `PuzzleSetting`; the `.agents/skills/spec-change/SKILL.md`
and `tend` skills; decision 0014 (`docs/decisions/0014-board-imports-solver.md`), the
shape of a spec-change record; `docs/explanation/architecture.md`, "The board" and "The
record".

## Goal

`board.allium` states play from a record, as `PlayingFromARecord`: what a caller hands
over, what it is handed back, and the guarantees above. Both specification gates are
clean with no waiver. A decision record says why both ways exist and why the second
lives in the core. T41 can build the second way from this text alone.

## Non-goals

- No Rust: T41 builds it.
- No change to any existing rule, invariant, surface or contract of `board.allium`
  beyond the Scope's Includes and the line that counts the boundaries; no change to
  `sudoku.allium`, `solver.allium` or `generation.allium`.
- No format, and nothing about where a record is kept: still the caller's.
- No answer to the open question on checks.
- Nothing about a record's version: T43 states it, after this ticket.
- No statement of a binding's names, casing or types: those are T41's and T42's.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `docs/specs/board.allium` | specification | Scope's Includes gains play from a record, and the boundaries become three; the new contract and surface (step 3) |
| `docs/decisions/0021-play-from-a-record.md` | decision | New record (step 4); the number is the next free one on the day |
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

2. **Before writing a clause**, read the maintainer's answers under "Settled by the
   maintainer, 2026-10-31" in Context; they are not asked again. If S12 has finished,
   read its question 8 first and carry any shape it found unstated.

3. **The clauses**, written with the `tend` skill, in the module's voice and vocabulary
   (never pencil mark, never candidate for what a player writes):

   - **Scope.** An Includes bullet: play from a record, a board's operations put to a
     record a caller hands over, the record after it handed back with what `Playing`
     shows, and nothing kept between calls. The bullet "Two boundaries" becomes three,
     naming the new one. Excludes' "How a record is kept" stands as written.
   - **A contract** beside `Recording`, named `PlayingFromARecord` (settled), with
     three signatures: one that opens from givens and gives the record of the board
     opened and what `Playing` exposes of it; one that takes a record and one operation
     and gives the record after it and what `Playing` exposes of the board after it,
     with, for `CheckCell`, the check it added; and one that takes a record and gives
     the board it reopens to, for reading. Opening from givens is one of the three
     (settled). An operation is one of `Playing`'s seven, `Place`, `Erase`, `WriteMark`,
     `StrikeMark`, `CheckCell`, `Undo` and `Redo`, with the cell and digit each takes.
     How an operation, and what a call gives back, are named as values is the module's
     to choose under Allium 3; say in the hand-back notes what was chosen and why.
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
   - **A surface**, also named `PlayingFromARecord` (settled), that `fulfils` the
     contract and stands whether or not a board is open, as `Reopening` does. Its
     guarantees say the same of a caller: playing from records step by step and holding
     one board give records equal at every step, and the same readings, moves and
     checks.

   Write the contract and the surface's empty shells first and run `just check-specs`:
   Allium 3's validity rules require contract names to be unique in a module and say
   nothing of a contract and a surface sharing one. If the checker refuses the one name
   for both, stop and put the clash to the maintainer rather than choosing another name.

   Run `just check-specs` and `just analyse-specs` after each clause, and fix every
   diagnostic and finding at its root; no `allium-ignore`.

4. **The decision record**, `docs/decisions/0021-play-from-a-record.md` in the shape of
   0014, `canonical_for: [decision_play_from_a_record]`; use the next free number on the
   day (0019 and 0020 are projected for T13 and T15; 0017 is T34's on its branch and
   0018 T37's) and rename the file, the title, the index row and the manifest entry if
   it is taken. Context: the maintainer's question and answer of 2026-10-31, with the
   three choices as asked, and the three answers on the name, what a call hands back and
   opening from givens. Decision: both ways; the stateless way specified in
   `board.allium` as a composition and built in the core, so that every binding shares
   it and none depends on another; each call hands back the record and the view; no
   format in the core, values through the `serde` feature as `Record` already is.
   Consequences, including the ones that hurt: a search and a replay of every move on
   each stateless call; two ways to keep consistent, held equal by T41's tests; the
   version question, made pressing by the record becoming a consumer's state, and
   answered by T43. What would reopen it: a record that carries the solution (which
   `TheSolutionIsNeverShown` forbids today), or a consumer that needs neither way.

5. **The pages**, as Files touched lists. Then `just plan-spec board` again and record
   the new count of obligations, and the obligations the new clauses add, for T41's
   test list.

6. `just check-docs`, then `just check`, quoted. Set `status: done` and the index row.
   Commit, and ask before pushing.

**Adversarial review**, by the procedure in `tickets/README.md`, "Adversarial
review":

- **R1**, after step 3 and before step 4: the clauses, clean under both gates, before the decision record is written. Focus: what T41 would have to guess.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` report no diagnostics and no findings for
  every module, with no waiver.
- `board.allium` states the contract `PlayingFromARecord`, its three signatures and four
  invariants, and the surface of the same name; it adds no `open question`; its
  existing rules, invariants, surfaces and contract are unchanged.
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

Expected: every module clean; the obligations; two contracts, three surfaces, one open
question and two `use` lines; one specification file changed; the documents gate green;
`All checks passed and the worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

Run on 2026-11-07 on the branch `implement-typescript-surface-fullsend`, from
`d501d180ea0ae13d10679d10d26f2955126df813`, with allium 3.6.1 from `.tools/bin`.

- **Before any clause.** `just check-specs`: "allium check: 9 specifications, no
  diagnostics and no findings." `just analyse-specs`: "allium analyse: 9 specifications,
  no diagnostics and no findings." `just plan-spec board`: 78 obligations.
- **The stop in step 3.** A scratch copy of the modules, with `board.allium` given an
  empty-bodied contract `PlayingFromARecord` and a surface of the same name that
  `fulfils` it, checked and analysed with no diagnostic and no finding. The checker takes
  the one name for both, so no clash went to the maintainer.
- **After the clauses, and after the review's answer.** Both gates read as before, every
  module clean, with no waiver. `just plan-spec board`: 88 obligations.
- `rg -n '^(contract|surface|open question|use)' docs/specs/board.allium`: two contracts
  (`Recording`, `PlayingFromARecord`), three surfaces (`Playing`, `Reopening`,
  `PlayingFromARecord`), one open question, the two `use` lines unchanged.
- `just check-docs` and `just check`, run at this ticket's last commit.

**The obligations the clauses add**, ten, for T41's test list:

- `contract-signature.PlayingFromARecord.open`, `.play`, `.read` and `.reading_after`;
- `entity-fields.Operation` and `value-equality.Operation`;
- `entity-fields.Reading` and `value-equality.Reading`;
- `enum-comparable.OperationKind`;
- `surface-actor.PlayingFromARecord`.

`allium plan` derives no obligation from a prose `@invariant` or `@guarantee`, so the
contract's four invariants and the surface's two guarantees reach T41's test list from
their text alone. Each is a test of its own: below.

### Names T41 needs

- **`enum OperationKind`**: `place`, `erase`, `write_mark`, `strike_mark`,
  `check_cell`, `undo`, `redo`, `Playing`'s seven.
- **`value Operation`** `{ kind, cell: sudoku/Position?, digit: Integer? }`. A cell for
  every kind but `undo` and `redo`; a digit for `place`, `write_mark` and `strike_mark`;
  nothing else. Every operation is well formed: a value lacking what its kind takes is
  no operation, and reading a value from outside as an operation comes before any call
  and is outside the contract. T41's enum with one variant per kind, each carrying what
  it needs and nothing else, is this value, and its tests need no malformed-operation
  refusal.
- **`value Reading`** `{ cell: sudoku/Position, digit: Integer?, note: Set<Integer> }`:
  `Move`'s `digit_after` and `note_after` for one cell, the note present whether it was
  shown or waited beneath a digit.
- **`contract PlayingFromARecord`**, four signatures:
  - `open(givens: Set<sudoku/Given>) -> Board`: `SetPuzzle` and `OpenBoard`;
  - `play(record: Record, operation: Operation) -> Board`: reopen, then the operation
    through `Playing`; the board after it;
  - `read(record: Record) -> Board`: the board the record reopens to;
  - `reading_after(record: Record, move_index: Integer) -> List<Reading>`: every cell in
    row order, as the move of that index holds it; `move.index`, from 1, undone moves
    included.
- **What a call hands back.** The signatures return `Board`, and the contract's comment
  names the parts: for `open` and `play`, the record `write` gives of that board, which
  the caller keeps, and the board as `Playing` exposes it less `move.digit_after` and
  `move.note_after`; for `play` with `check_cell`, the check it added, the last of the
  board's checks. `read` hands back the view alone. `reading_after` hands back the
  readings the other three leave out. The binding's value is T41's to name.
- **The refusal order** (`ARefusalHandsBackNothingElse`). A refusal hands back the
  refusal and no record, board or reading, and the record handed over is unchanged.
  `open`: givens are refused as `SetPuzzle` refuses them. `play` and `reading_after`: a
  record that does not reopen is refused first, as `reopen` refuses it; then, for `play`,
  an operation `Playing` does not offer that board, by its guard; for `reading_after`, an
  index that names no move. `read`: the reopen refusal alone.
- **The invariants, one test each.** `PlayingFromARecordIsHoldingTheBoard`;
  `ARefusalHandsBackNothingElse`; `WhatIsShownIsWhatPlayingShows`, which carries
  `TheSolutionIsNeverShown` and `ACheckNeverTellsTheDigit`; `NothingIsKeptBetweenCalls`.
- **`surface PlayingFromARecord`**, no context, `fulfils PlayingFromARecord`, with
  `RecordsKeepStepWithAHeldBoard` (equal records at every step against a held board) and
  `TheSameBoardFromEitherWay` (the same view, moves, checks, operations offered,
  readings and refusals).

### Deviations, and why

- **One branch.** The work is on `implement-typescript-surface-fullsend`, with S12
  before it, not on `ticket/t40-play-from-a-record-spec` from `main`; step 1's
  worktree and the merge conditions are met by order on that branch. `.pixi/` and
  `.tools/bin/allium` were present, so neither install ran.
- **`docs/manifest.yml`.** Frozen under CONVENTIONS.md §11; edited for the record's entry
  in the same commit as the record, under the maintainer's confirmation of 2026-11-06 for
  this run, as a T00 follow-up.
- **Four signatures, not three, from S12's carry.** S12's question 8 measured the
  readings after every move at 700,193 bytes of JSON at 200 moves against 24,099 for
  the view. So what a call hands back excludes the readings, in the signatures and the
  Scope's Includes bullet; a fourth signature, `reading_after`, gives them one move at a
  time; and the refusal invariant gains a third case, an index that names no move,
  after the record's. Step 3's "three signatures" and the Acceptance criteria's "three
  signatures and four invariants" read four signatures. Decision 0021's Decision says
  the readings come on asking. The generation refusal S12's rounds added is T41's, not
  this ticket's.
- **The hand-back as `Board`, not as a value.** A value `Played { record, board, check? }`
  was drafted for what `open` and `play` give. allium 3.6.1 reports
  `allium.definition.unused` for a value named only as a bare return type: `Set<Played>`
  clears it and `Played?` does not. Rather than bend the type or waive the finding, the
  calls return `Board`, as `Recording.reopen` does, and the contract's comment names
  the record, the view and the check. The maintainer's lead kept this after R1.
- **`OperationKind` and two values beside `MoveKind`.** The Non-goals forbid changing
  any existing clause; they did not forbid new declarations the contract's signatures
  need. `Position` is `sudoku.allium`'s, reached through the existing `use` line, since a
  caller with no board holds no `BoardCell`.
- **`docs/explanation/specifications.md`.** It said `board.allium` states "a
  `contract`", which the second made untrue; the sentence now counts two and names
  play from a record. It neither lists surfaces nor counts open questions.
- **The decision index.** 0021 is taken out of order under this run's numbering: 0017 to
  0020 are held for records still to land, which the index now says, and the "next
  decision" sentence reads 0022.

### Handed back

- **To T41.** The names above, and three points: the contract returns `Board`, so the
  value a call hands back is T41's to name, holding the record, the view without the
  readings and, for a check, the check; an operation is decoded before the call and the
  core's enum cannot be malformed, so no malformed-operation refusal is tested; and the
  six prose clauses carry no `plan` obligation, so the test list takes them from the
  text.
- **To whoever next relies on the checker for a value.** A value named only as a bare
  return type of a contract signature reads as unused in allium 3.6.1, where the same
  value inside `Set<>` or `List<>` does not. Not filed upstream; this run has no
  authority to contact anyone.
- **To T43.** `board.allium` now has two contracts. The version clause is `Recording`'s,
  and a record of another version is refused where `reopen` refuses, so it is the first
  refusal of `play`, `read` and `reading_after` too, with nothing to add to
  `PlayingFromARecord`.

### Adversarial review

- **R1, round 1**, after step 3, on the range from the start to the checkpoint `59d91bb`,
  focus what T41 would have to guess. One finding, medium, judged valid: `Operation`'s
  comment promised to refuse a value carrying more or less than its kind takes, as an
  operation `Playing` does not offer, but no guard of `Playing` can refuse it (`Undo`
  with a surplus digit passes its guard; `Place` with no cell cannot evaluate its own),
  and its place in the refusal order was unstated. It also ran against S12's reading of
  the wire, where a missing field is refused by serde before the core and a surplus one
  is ignored. Answered in `53f58ed` by the reviewer's alternative: every `Operation` is
  well formed, only `Playing`'s guard refuses one, and reading a value from outside as
  an operation is outside the contract, as a record's form is. No invariant, surface or
  signature changed. The reviewer's note that its sandbox could not run `just check` is
  an environment note, not a finding.
- **R1, round 2**, on the range to `53f58ed`: approved with no findings. Its residual
  point stands as a note to T41: the value's fields stay optional, because Allium values
  carry no invariants and sum types are for entities, so the restriction is prose.

### Open points settled

- **How an operation and what a call gives back are named.** An operation is a value
  with a kind of its own enum, a position and a digit; a call returns `Board` with the
  hand-back in prose, for the checker reason above.
- **Where the readings after a move live.** On asking, through `reading_after`, from
  S12's measurement.

## Open points

The name, what a call hands back, opening from givens and records across versions were
settled by the maintainer on 2026-10-31 (Context) and are not asked again; the last is
T43's.

- **Paths other tickets also edit.** T37, open, edits `docs/decisions/README.md`,
  `docs/manifest.yml`, `docs/explanation/specifications.md`, `docs/project/terminology.md`
  and `CHANGELOG.md`, and takes decision 0018; T13 and T15 take decision records too.
  T43 edits `docs/specs/board.allium`, `docs/decisions/README.md`, `docs/manifest.yml`,
  `docs/project/terminology.md` and `CHANGELOG.md` after this ticket has merged, and
  takes the decision number after this ticket's. Each shared edit is a row, an entry or
  a sentence; whichever merges second merges `main` first and takes the next number,
  the deviation from CONVENTIONS.md §11 that T31 and T33 recorded.
