---
id: T28
title: "The record and reopening: a board written down whole and had again"
status: open
depends_on: [T27]
parallel_with: []
branch: ticket/t28-record-and-reopening
estimated_size: M
---

# T28: The record and reopening: a board written down whole and had again

## Context

T27 built the board in play. This ticket builds the last part of
`docs/specs/board.allium`: `contract Recording` and `surface Reopening`. A board can be
written down as a record, and a record reopened to the same board, moves, undone moves
and checks included. The library has no persistence (`AGENTS.md`: "no UI, no persistence
and no server"), so the record is a value the caller is handed and keeps wherever it
likes.

The design was decided by the maintainer on 2026-09-30 and is recorded in the decision
T23 wrote (`docs/decisions/0014-board-imports-solver.md`, unless renumbered). What this
ticket needs from it:

- **The record is a plain value; there is no storage effect.** Randomness stays the
  engine's only effect.
- **A board reopens from a record in one call**, which runs the solver itself, as
  opening from givens does.
- **The record is minimal.** It holds the givens; the moves in the order made, each with
  its kind, its target and its digit; how many of them are undone; and each check's
  `after_move`, target and digit. It holds nothing that can be worked out from those:
  no cell's digit, no note, no `digit_before`, no `struck_from`, no `is_right`, and
  nothing read from the solution.
- **Reopening reads the record back.** It puts the givens to `solver::solve`, opens a
  board, reads the moves forward through the board's own guards, takes back the undone
  ones, and works out each check's answer from the solution the solver found. A record
  that does not read back is refused.

What the specification says, after T23's rewording of the `reopen` comment: the moves and
checks a reopened board holds are the record's own, with the indices they had and the
same moves undone; reopening adds none and discards none; how the board is made whole is
not said; a record that no board could have been written to is refused.
`ARecordIsEnoughOnItsOwn` says the solution is not in the record and the solver finds it
again.

Facts that shape the work:

- **A record can be anything.** Its fields are private, but under the `serde` feature a
  caller can deserialise whatever it likes. Deserialising checks nothing; reopening
  checks everything. So `reopen` is fallible and never panics.
- **The undone moves must read back too.** Redo has to be able to re-take them. Reading
  every move forward and then taking back the last few gives the board the record was
  written from, because an undone move was made after the standing ones and nothing has
  moved since. A record whose undone count exceeds its moves is refused. One whose moves
  solve the puzzle before the last of them, or solve it with moves still undone, is
  refused too: no board reaches those states.
- **One thing cannot be read back.** A check records the digit that stood in its cell
  when it was asked. After an undo and a new move, the moves that stood then may be
  gone, so `after_move` can exceed the count of moves the record holds, and the digit
  cannot be compared with any state the record can rebuild. Reopening holds a check to
  what it can: the target is on the grid and not a given, the digit is in range, and
  `after_move` is at least 1. A check needs a player's digit in its cell, and only a
  standing move puts one there, so no board writes a check after no move. For the same
  reason a record that holds a check holds at least one move: the placement the check
  asked about was made, undo keeps a move on the record, and an undone move leaves it
  only when a new move takes its place. So a record with checks and no moves is
  refused, whatever the checks say. Nothing more can be asked of the moves: they may
  all be undone, and none need be a placement, since a mark can discard the placement
  that was checked. Say so in the type's documentation. `is_right` is worked
  out from the solution, so a record cannot lie about it.
- **No format crate.** After T29 the workspace has four dependencies, two that may
  ship and two for tests, and it gains none here (decision 0007). `Record` derives
  `Serialize` and `Deserialize` under the existing `serde` feature. Tests that need a
  malformed record build one field by field inside the module.
  `crates/pawdoku/src/random.rs` shows how a test drives a derived `Deserialize` with
  serde's own value deserialisers if one is wanted.
- **Snapshots show and detect change; they prove nothing.** T29 added insta and the rule,
  which `docs/reference/testing.md` states: a snapshot shows a reviewer what a value
  looks like and makes a change to it visible in a diff. It is never the test of a
  clause, so no line of "What the tests must cover" and no row of the hand-back table
  names one. Snapshots are `.snap` files, taken from fixed inputs after the module's
  own tests are green, by tests named `snapshot_...`, from a text picture rendered in
  test code. No `Display`, method or field is added to the library to feed one.
  Here the record is shown as JSON, through the `json` feature T29 turned on for
  insta. That is a picture of the stored shape for a reviewer and no promise of a
  format: the library still has no format crate. An earlier ticket's snapshot that
  changes has its diff read and its reason given in the hand-back notes.
- **The metrics gate** (`rustqual.toml`, `clippy.toml`): a struct at most 20 methods and
  12 fields; a file at most 500 code lines before its first `#[cfg(test)]`; a function
  at most 60 lines, cognitive complexity 15, cyclomatic 10, nesting 4, five parameters;
  no recursion. Writing and reopening add to `Board`'s method count, which T27 left
  below the cap. Put the record in its own file under `src/board/`. Never a suppression
  and never a moved number; if structure cannot meet a limit, stop and report.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/board.allium` in
full, `contract Recording` and `surface Reopening` twice; the decision T23 wrote; the
hand-back notes of `tickets/T27-board-in-play.md` ("Names later tickets need");
`crates/pawdoku/src/board.rs` and its parts; `crates/pawdoku/src/random.rs` (how
`ReplayStream` validates on the way in from serde, through `TryFrom`);
`docs/explanation/architecture.md`; `docs/project/terminology.md`, the "Record, reopen"
row; `docs/project/purpose-and-scope.md`, "No persistence";
`.agents/skills/rust-change/SKILL.md` and `.agents/skills/propagate/SKILL.md`.
Background only, never modified:
`/Users/scutting/projects/pawdoku/prototypes/board/board.py`, whose record holds the
board whole; this ticket's record does not.

## Goal

A board can be written to a `Record` at any point, solved or not, and writing changes
nothing. A `Record` reopens to a board that `Playing` cannot tell from the one written:
the same is exposed, the same is offered, and every move, reading, undo, redo and check
answers the same. A record that does not read back is refused with an error that says
why. Under the `serde` feature a record serialises. `just check` is green with the
coverage floor held and no suppression.

The name later tickets are written against, fixed here: `board::Record`. Every other
name is this ticket's to choose and to report.

## Non-goals

- No storage: no file, no key, no trait for a store. Where a record is kept is the
  caller's.
- No format and no format crate. The record is a Rust value and, under `serde`, whatever
  a consumer's serialiser makes of it.
- No promise that a record written by one version of the engine reopens in another
  (first open point). No version field is added.
- No serialisation of `Board` or `Puzzle`, and nothing of the solution in a record.
- No fuzz target (second open point).
- No change to `board.allium` or any module. If no clause says what the code needs,
  stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No snapshot standing in for a test that asserts. `ReopeningIsExact` is proved by the
  property of step 6 and never by two snapshot files that match.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/board/record.rs` | New: `Record`, writing, reopening and its errors; the file's name is the agent's |
| `crates/pawdoku/src/board.rs` | The module declaration and the two entries on `Board` |
| `crates/pawdoku/tests/api_bounds.rs` | `Record` and the new error meet invariant 3; `Record` serialises under `serde` |
| `crates/pawdoku/tests/board.rs` | Writing and reopening through the public API |
| `crates/pawdoku/tests/snapshots.rs` | The snapshot tests of step 7 |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | "What crosses each boundary": the record is the value a consumer keeps |
| `docs/project/terminology.md` | The "Record, reopen" row, if the minimal record makes any of it untrue |
| `docs/reference/testing.md` | The suite table |
| `docs/project/repository-map.md` | The `src/` line, if it lists the module's parts |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T28's row set `done` |
| `tickets/T28-record-and-reopening.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t28-record-and-reopening` from `main` after T27 has
   merged. Run `just initialize` if `.pixi/` is absent; it uses the network and this
   ticket authorises it. Run `just check` and confirm it is green before any edit.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec board` and add the obligations T27 marked as
   this ticket's. Put the list in the hand-back notes under "The test list" before the
   first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. A good order is: a fresh board written and reopened;
   then moves; then undone moves; then checks; then a solved board; then each refusal.

4. **`Record`.** A public struct with private fields, `Clone`, `Debug`, equality,
   `#[non_exhaustive]`, and `Serialize` and `Deserialize` under `serde`. Its contents are
   the ones under Context and no more. Writing takes the board by shared reference.

5. **Reopening.** One call from a record to a board or an error. The error is a
   `thiserror` enum, `#[non_exhaustive]`, with one variant for each way a record fails
   to read back and stable `Display` text in the style of `RandomError`: the solver's
   refusal of the givens, carried as its source; a move the board's guards refuse, with
   its index; more undone moves than moves; a move after the puzzle is solved; a check
   that fails what a check is held to, with its index; checks in a record that holds no
   move. A unit test pins each variant's text.

6. **The exactness property.** A property plays an arbitrary sequence of the board's
   seven operations on the fixture below, writes the board, reopens the record, and
   compares everything `Playing` exposes, using T27's full-view helper: the whole
   view, moves and checks included, and not the picture alone. Then it plays a
   second arbitrary sequence on both boards and compares after every step. Run it from
   a fresh board, from a board with moves undone, and from a solved board.

7. **Snapshots**, once the list is empty. Take these in
   `crates/pawdoku/tests/snapshots.rs`, each by a test named `snapshot_...`:

   - the record of the board T27's named script leaves, as JSON, by
     `assert_json_snapshot!`. The test is compiled only under the `serde` feature;
     `just test` and `just snapshots-check` run with every feature, so it runs in the
     gate. A reviewer sees what a consumer would store, and a renamed or reordered
     field shows in the diff;
   - the same record's `Debug`, which needs no feature;
   - the board reopened from that record, drawn by T27's render helper. It is its own
     file. A reviewer can lay it beside T27's picture of the scripted board, and the
     two should read alike, but no test compares the files;
   - the `Display` text of every reopening error, in one snapshot.

   Accept with `just snapshots-accept` and read each file before committing it. List
   them in the hand-back notes under "Snapshots taken".

8. **Doc examples and the pages.** Every public item this ticket adds has a doc
   example that runs and asserts (`AGENTS.md`: "a doctest on every public item"):
   `Record`, writing, reopening, and the reopening error. `missing_docs` and `just doc` do not
   notice an absent example, so the hand-back notes list each public item beside the
   doctest that covers it; `just test` runs them.

   `docs/explanation/architecture.md`: the record in "What crosses each
   boundary", and that deserialising checks nothing while reopening checks everything.
   `docs/project/terminology.md`: re-read the "Record, reopen" row against the minimal
   record and edit it only if a sentence is no longer true. `docs/reference/testing.md`,
   with the snapshots taken, and `CHANGELOG.md` follow.

9. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`.
   Quote the closing lines of each and the coverage line for `src/board`. Fill in the
   hand-back notes: the test list as it ended, as a table from clause to test name or
   reason; "Names later tickets need", with the signatures of writing and reopening and
   the error; and under Handed back, that `Record` is a value S04's drafted bindings
   tickets (T15, the WebAssembly crate, first) carry across their boundary through the
   `serde` feature. Set `status: done`
   here and in `tickets/README.md`, commit on the ticket branch, and stop before
   pushing.

The fixture: thirty givens, one solution, rows top to bottom.

```text
puzzle       solution
53..7....    534678912
6..195...    672195348
.98....6.    198342567
8...6...3    859761423
4..8.3..1    426853791
7...2...6    713924856
.6....28.    961537284
...419..5    287419635
....8..79    345286179
```

### What the tests must cover

From `docs/specs/board.allium`, by name.

- **`Recording.write`.** A record can be written from a fresh board, a board in play, a
  board with moves undone and a solved board.
- **`WritingChangesNothing`.** Everything `Playing` exposes is the same before and after
  writing, and writing twice gives equal records.
- **`Recording.reopen` and `ReopeningIsExact`.** The reopened board has the same givens,
  digits, notes shown and waiting, moves in the same order with the same indices and the
  same ones undone, and the same checks with the same answers. Every reading, undo, redo
  and check then answers as it would have on the written board.
- **`AReopenedBoardIsTheSameBoard`.** The exactness property of step 6.
- **`ARecordIsEnoughOnItsOwn`.** Reopening takes the record and nothing else. Two
  reopenings of one record are independent boards: a move on one does not show on the
  other.
- **A record of a puzzle the rules would not pose is refused:** givens with no solution,
  with several, and with nothing left to play.
- **A record that does not read back is refused**, one test for each: a move on a given
  cell; a digit out of range; a placement of the digit already standing; an erasure of
  an empty cell; a mark written where a digit stands; a mark struck that is not there; a
  move after the puzzle is solved; an undone count greater than the count of moves;
  undone moves in a record whose moves solve the puzzle; a check on a given, on a
  position off the grid, with a digit out of range, or with an `after_move` of 0; a
  check, sound in itself, in a record that holds no move.
- **What cannot be read back is accepted and documented.** A check whose `after_move`
  exceeds the moves the record holds reopens, and keeps its `after_move`, so long as
  the record holds a move. Two boards a player can reach show it: place, check, undo,
  which leaves one move, undone; and three placements, a check, three undos and one
  written mark, which leaves one move that is no placement and a check after three.
- **Refusal leaves nothing behind.** A refused reopening returns an error and no board.
- **`serde`.** Under the feature, `Record` meets the serialisation bounds; without it,
  the crate compiles with no serde in its public surface.

## Acceptance criteria

- `pawdoku::board` exports `Record`, a way to write one from a board, and a way to
  reopen one that returns a `Result`.
- `Record` holds nothing derived and nothing read from the solution, read by inspection
  of its fields and stated in the hand-back notes.
- `Board` and `Puzzle` implement neither `Serialize` nor `Deserialize`.
- Every public item this ticket adds has a doc example that runs and asserts, and
  the hand-back notes list each beside its doctest.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec board` obligation T27 marked T28's is in the table or
  struck with a reason.
- No recursion. No `#[allow]`, no `#[expect]`, no `qual:allow` line. No struct in
  `src/board` exceeds 20 methods, by `just metrics`.
- `Cargo.toml` and `Cargo.lock` are unchanged.
- Line coverage of `src/board` is at or above 90 per cent.
- The four snapshots of step 7 exist as `.snap` files, each taken by a test named
  `snapshot_...`. None appears in the hand-back table as the test of a clause, and
  nothing of the solution is in any of them. `just snapshots-check` is green, and any
  earlier snapshot that changed has its reason in the hand-back notes.
- `just check` is green.

## Verification

```sh
just plan-spec board
just fmt-check
just clippy
just metrics
just test
just snapshots-check
just wasm-check
just features
just coverage
just doc
just check-docs
just check
```

Expected: the obligation list the table accounts for; each recipe green with no warning;
`metrics` reporting no finding; both wasm targets under both feature sets, which is what
proves the record compiles with and without `serde`; the coverage total at or above the
floor; `All checks passed and the worktree is unchanged.`

## Hand-back notes

### The test list

### Names later tickets need

### Snapshots taken

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **Records across versions.** A consumer keeps a record between visits, so a record
  will outlive the engine version that wrote it. The specification is silent on whether
  a later version must reopen it, and that is a product decision. The ticket builds
  nothing for it, adds no version field, and says in the type's documentation that no
  promise across versions is made yet. The maintainer decides before the first release;
  S02's release ticket (T13) is where it would bite.
- **Whether reopening meets the fuzzing trigger.** S03 recommended a fuzz target "the
  day a parser lands". Reopening takes input a caller controls and must refuse it
  without panicking, which is the property a fuzz target tests. Whether that counts as
  the trigger is the maintainer's call; the property test of step 6 and the refusal
  tests are what this ticket provides.
- **A check's digit.** A record can state any in-range digit for a check whose moves
  have since been discarded, and reopening cannot tell. So "a record that no board
  could have been written to is refused" holds for the moves and not fully for the
  checks: reopening refuses every impossible check it can recognise (an `after_move`
  of 0, a given or off-grid target, a digit out of range, checks with no move) and
  accepts the rest. Storing more would make the
  record larger than the minimal form the maintainer chose. If the gap matters, it is a
  question for the specification: what a check keeps.
- **`after_move` as a count.** It is an integer that is never negative, so an unsigned
  type makes a negative count impossible to write. The agent chooses the type. Zero
  stays writable whatever the type, and its refusal is tested either way.
