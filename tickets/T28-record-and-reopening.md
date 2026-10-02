---
id: T28
title: "The record and reopening: a board written down whole and had again"
status: done
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

The list as first written on 2026-10-02, before any test or code, seeded from "What the
tests must cover" and then checked against `just plan-spec board`, whose three
obligations T27 marked as this ticket's are `contract-signature.Recording.write`,
`contract-signature.Recording.reopen` and `surface-actor.Reopening`. One line is one
expected test: its name, then what it expects. B is `crates/pawdoku/tests/board.rs`,
through the public API; U is a unit test in `crates/pawdoku/src/board/record.rs`, where
a malformed record can be built field by field.

Writing and reopening (B):

1. `a_fresh_board_is_written_and_reopened`: the reopened board shows the full view of
   the one written.
2. `a_record_is_written_from_a_board_at_any_point`: fresh, in play, with moves undone,
   and solved.
3. `writing_changes_nothing`: the full view is the same before and after, and writing
   twice gives equal records.
4. `a_board_in_play_reopens_to_the_same_board`: every kind of move, the full view equal.
5. `undone_moves_reopen_undone_and_are_re_taken_the_same`: the same moves undone, and
   redo answers the same on both boards.
6. `checks_reopen_with_their_indices_and_their_answers`: a check answered each way.
7. `a_solved_board_reopens_solved`: solved, nothing offered, every move read back.
8. `two_reopenings_of_one_record_are_independent_boards`: `ARecordIsEnoughOnItsOwn`; a
   move on one does not show on the other, nor on the board written.
9. `a_check_after_moves_since_discarded_reopens_and_keeps_its_after_move`: the two
   boards a player can reach, place-check-undo and three placements, a check, three
   undos and a written mark.
10. `a_reopened_board_writes_the_same_record`.
11. `a_reopened_board_is_the_same_board`: the property of step 6, from a fresh board, a
    board with moves undone and a solved board.

Refusals (U):

1. `givens_with_no_solution_are_refused`.
2. `givens_with_several_solutions_are_refused`.
3. `givens_that_leave_nothing_to_play_are_refused`.
4. `a_move_on_a_given_is_refused`.
5. `a_move_with_a_digit_out_of_range_is_refused`.
6. `a_placement_of_the_standing_digit_is_refused`.
7. `an_erasure_of_an_empty_cell_is_refused`.
8. `a_mark_written_where_a_digit_stands_is_refused`.
9. `a_mark_struck_that_is_not_there_is_refused`.
10. `a_move_after_the_puzzle_is_solved_is_refused`.
11. `more_undone_moves_than_moves_are_refused`.
12. `undone_moves_on_a_solved_puzzle_are_refused`.
13. `a_check_on_a_given_is_refused`.
14. `a_check_off_the_grid_is_refused`.
15. `a_check_with_a_digit_out_of_range_is_refused`.
16. `a_check_after_no_move_is_refused`: an `after_move` of 0.
17. `checks_in_a_record_with_no_move_are_refused`: a check sound in itself.
18. `refusals_come_in_a_fixed_order`: a record wrong in several ways reports the first.
19. `a_refused_reopening_leaves_the_record_as_it_was`: an error, no board, and the
    record still equal to a copy taken before.
20. `reopening_refusal_texts_are_stable`: each variant's `Display` text.
21. `a_record_holds_nothing_of_the_solution`: the `Debug` of a fresh board's record
    shows no row of the solution.

From outside the crate:

1. `Record` and the reopening error in the two bounds tests of `tests/api_bounds.rs`,
   and `Record` in the `serde` one.
2. A doc example on each of the four public items.

Expected to be struck: `surface-actor.Reopening`, since the surface names no actor, as
T27 struck `surface-actor.Playing`.

Changes to the list, one line each, as the loops ran:

- The first run was against a board with no `write` and no `reopen`, so the ten B tests
  failed to compile. `Record`, writing and the refusal type were then written whole,
  with a reopening that opened a board from the givens and did nothing else, so that
  every test compiled. From there the tests failed on their assertions: thirteen were
  seen to fail before the run stopped, and all were green once reopening made the
  moves, took back the undone ones and put the checks back.
- Corrected: `refusals_come_in_a_fixed_order` failed once on its own expectation, an
  undone count of 1 where the record said 2. The test was wrong and the code was not.
- Added: U `a_move_off_the_grid_is_refused`, the one refusal of a move the ticket's
  list leaves out.
- Added: U `a_record_built_by_hand_reopens`, so that the refusal tests are not the only
  use of the hand-built record: the helper builds a sound one too.
- Added: U `a_check_that_cannot_be_read_back_is_accepted`, a record no board wrote: a
  check of a digit no move placed, after more moves than a `usize` can count. It
  reopens, keeps its `after_move`, and its answer is worked out from the solution.
- Added: U `a_refusal_names_its_source`, for the three variants that carry another
  refusal.
- Added: `a_board_and_a_puzzle_do_not_serialise` in `tests/api_bounds.rs`, which makes
  an acceptance criterion a compile-time fact and not an inspection.
- Changed after coverage: taking back first asked whether the puzzle was solved and
  then mapped an undo's refusal through a closure no record could reach. The guard
  went, and the board's own refusal of an undo on a solved puzzle is the refusal, so
  no line of the file is unreachable.
- Struck, as expected: `surface-actor.Reopening`.
- What the gates taught. Clippy refuses a block nested five deep, so the test that
  looks for the solution in what a record prints gathers the rows first. The metrics
  gate counts cohesion by the fields a method names, and a constructor names none, so
  what a record keeps of a move and of a check are `From` conversions and not methods.

The list as it ended, from clause to test. B is `crates/pawdoku/tests/board.rs`, U is a
unit test in `crates/pawdoku/src/board/record.rs`, A is `crates/pawdoku/tests/api_bounds.rs`,
and P is the property, `a_reopened_board_is_the_same_board` in B. No row names a
snapshot test.

| Clause | Test, or the reason there is none | `plan-spec` obligations |
| --- | --- | --- |
| `Recording.write`: from a fresh board, a board in play, a board with moves undone and a solved board | B `a_record_is_written_from_a_board_at_any_point`, B `a_fresh_board_is_written_and_reopened`; P | `contract-signature.Recording.write` |
| `WritingChangesNothing` | B `writing_changes_nothing`: the full view before and after, and two writes equal; P, after any script | none |
| `Recording.reopen`, `ReopeningIsExact`: the same givens, digits, notes shown and waiting, moves and checks | B `a_record_is_written_from_a_board_at_any_point`, on the full view at the four points; B `a_board_in_play_reopens_to_the_same_board`, with every kind of move, upkeep and a note that waits; B `a_reopened_board_writes_the_same_record` | `contract-signature.Recording.reopen` |
| The same moves undone, and undo and redo answering the same | B `undone_moves_reopen_undone_and_are_re_taken_the_same` | none |
| The same checks with the same answers | B `checks_reopen_with_their_indices_and_their_answers`; U `a_record_built_by_hand_reopens` | none |
| A solved board | B `a_solved_board_reopens_solved` | none |
| `AReopenedBoardIsTheSameBoard` | P, from a fresh board, a board with moves undone and a solved board: the full view equal after reopening, then what each board answers and the full view equal after every step of a second script | none |
| `ARecordIsEnoughOnItsOwn` | B `two_reopenings_of_one_record_are_independent_boards`: reopened after the board written is dropped, and a move and a check on one reopening show on neither the other nor the record. By shape: `Board::reopen` takes the record and nothing else. U `a_record_holds_nothing_of_the_solution` | none |
| A record of a puzzle the rules would not pose is refused | U `givens_with_no_solution_are_refused`, `givens_with_several_solutions_are_refused`, `givens_that_leave_nothing_to_play_are_refused` | none |
| A move on a given cell | U `a_move_on_a_given_is_refused` | none |
| A digit out of range | U `a_move_with_a_digit_out_of_range_is_refused`, for a placement and a mark | none |
| A placement of the digit already standing | U `a_placement_of_the_standing_digit_is_refused` | none |
| An erasure of an empty cell | U `an_erasure_of_an_empty_cell_is_refused` | none |
| A mark written where a digit stands | U `a_mark_written_where_a_digit_stands_is_refused` | none |
| A mark struck that is not there | U `a_mark_struck_that_is_not_there_is_refused`, one never written and one upkeep struck | none |
| A move off the grid | U `a_move_off_the_grid_is_refused` | none |
| A move after the puzzle is solved | U `a_move_after_the_puzzle_is_solved_is_refused`, for a move on the grid and one off it | none |
| An undone count greater than the count of moves | U `more_undone_moves_than_moves_are_refused`, `usize::MAX` among them | none |
| Undone moves in a record whose moves solve the puzzle | U `undone_moves_on_a_solved_puzzle_are_refused` | none |
| A check on a given, off the grid, with a digit out of range, with an `after_move` of 0 | U `a_check_on_a_given_is_refused`, `a_check_off_the_grid_is_refused`, `a_check_with_a_digit_out_of_range_is_refused`, `a_check_after_no_move_is_refused` | none |
| A check, sound in itself, in a record that holds no move | U `checks_in_a_record_with_no_move_are_refused` | none |
| Which refusal is reported first | U `refusals_come_in_a_fixed_order` | none |
| What cannot be read back is accepted | B `a_check_after_moves_since_discarded_reopens_and_keeps_its_after_move`, the two boards a player can reach; U `a_check_that_cannot_be_read_back_is_accepted`, a record no board wrote. Documented on `Record` | none |
| Refusal leaves nothing behind | By the type: `Board::reopen` returns a board or an error, never both, and borrows the record. U `a_refused_reopening_leaves_the_record_as_it_was` | none |
| The stable text of each refusal | U `reopening_refusal_texts_are_stable`, U `a_refusal_names_its_source` | none |
| `serde`: `Record` meets the serialisation bounds | A `streams_and_value_types_serialise_under_the_serde_feature` | none |
| Without `serde` the crate compiles with no serde in its surface | `just features` and `just wasm-check`, which build without the feature; every serde derive in `record.rs` is behind `cfg_attr(feature = "serde", ...)` | none |
| `Board` and `Puzzle` do not serialise | A `a_board_and_a_puzzle_do_not_serialise`, at compile time | none |
| `Record` and `ReopenError` meet invariant 3 | A, both bounds tests | none |
| `surface-actor.Reopening` | Struck: `Reopening` names no actor and no context | `surface-actor.Reopening` |

**What a `Record` holds, by inspection of its fields.** Four: `givens`, a set of
`Given`; `moves`, a list in which each entry is one of `Place { target, digit }`,
`Erase { target }`, `WriteMark { target, digit }` and `StrikeMark { target, digit }`;
`undone`, a count; and `checks`, a list of `{ after_move, target, digit }`. Nothing
derived: no cell's digit, no note, no `digit_before`, no `struck_from`, no index and no
`is_right`. Nothing read from the solution: writing reads the puzzle's givens, the
journal and the checks, and never calls `is_solution_digit`.

### Names later tickets need

**Public** (`pawdoku::board`), each with a doc example that runs and asserts.

| Item | Signature | Doctest |
| --- | --- | --- |
| `Record` | `Debug, Clone, PartialEq, Eq`; `#[non_exhaustive]`, fields private, no method; `Serialize` and `Deserialize` under `serde` | `board::record::Record` |
| `Board::write` | `pub fn write(&self) -> Record` | `board::Board::write` |
| `Board::reopen` | `pub fn reopen(record: &Record) -> Result<Self, ReopenError>` | `board::Board::reopen` |
| `ReopenError` | `Debug, Clone, PartialEq, Eq, thiserror::Error`; `#[non_exhaustive]`; `TooManyUndone { undone, moves }`, `ChecksWithoutMoves { checks }`, `Givens { source: SolveError }`, `Move { index, source: PlayError }`, `MoveAfterSolved { index }`, `UndoneWhenSolved { undone }`, `Check { index, source: PlayError }`, `CheckAfterNoMove { index }`; every count and index a `usize` | `board::record::ReopenError` |

The choices this ticket made:

- **Writing and reopening are named for the contract**: `write` and `reopen`, as the
  seven operations are named for their rules. `Board` has 19 methods of the 20 allowed.
- **Reopening borrows the record**, so one record reopens as often as a caller likes.
- **A record has no accessor.** Outside the crate it can be written, cloned, compared,
  printed, reopened and, under `serde`, serialised. A consumer that wants to read a
  game reopens it and reads the board.
- **A move in a record is an enum with one variant to a kind**, each carrying what the
  kind needs. A record cannot say a placement has no digit or an erasure has one, so no
  refusal exists for it. Under serde's defaults it is externally tagged, which every
  format can carry: `{"Place": {"target": {"row": 1, "column": 3}, "digit": 4}}`.
- **The givens are a set**, as the puzzle keeps them, so two records of one board are
  equal and a reopened board writes the record it was reopened from.
- **`after_move` and the undone count are `usize`**, as `Check::after_move` is. serde
  writes one as a `u64`. Zero stays writable, and its refusal is tested.
- **The order of refusals**, fixed by `refusals_come_in_a_fixed_order` and stated under
  `# Errors` on `Board::reopen`: `TooManyUndone`, `ChecksWithoutMoves`, `Givens`; then
  the first move that fails, `MoveAfterSolved` before `Move`; `UndoneWhenSolved`; then
  the first check that fails, off the grid, a given, a digit out of range, and last
  `CheckAfterNoMove`. The two counts come before the search, so a record that fails
  them costs no search.
- **A refusal's text closes with its cause.** `Givens`, `Move` and `Check` carry the
  solver's or the board's refusal as their source, and print it too: "move 3 of the
  record does not read back: the cell at row 1, column 1 holds a given". Bindings build
  an exception from the text alone, so the text has to say why.

**Inside**, for whoever changes the record next. `src/board/record.rs` holds `Record`,
the private `Written` (a move) and `Asked` (a check), `ReopenError`, and reopening as
four free functions in the order they run: `counted`, `replay`, `take_back`,
`ask_again`. Reopening calls `Board::open`, the four public moves, `Board::undo` and
`Check::new`; it reads the solution only through `Puzzle::is_solution_digit`.

### Snapshots taken

All four are in `crates/pawdoku/tests/snapshots/`, taken by tests in
`crates/pawdoku/tests/snapshots.rs` after the list was empty, and each file was read
before it was committed. Nothing was added to the library for them. None holds anything
read from the solution: a record has none to hold, and the reopened board's picture
shows, as T27's does, only a check's answer. The record is that of the board
`THE_SCRIPTED_GAME` leaves, played by `played`.

| Test | File | Shows |
| --- | --- | --- |
| `snapshot_the_record_of_the_scripted_game_as_json` | `snapshots__tests__snapshot_the_record_of_the_scripted_game_as_json.snap` | The record as JSON, by `assert_json_snapshot!`, compiled only under `serde`: the four fields in order, thirty givens, eleven moves each tagged with its kind, an undone count of 1 and two checks. A picture of what a consumer would store, and no promise of a format |
| `snapshot_the_record_of_the_scripted_game` | `snapshots__tests__snapshot_the_record_of_the_scripted_game.snap` | The same record's pretty `Debug`, which needs no feature |
| `snapshot_the_board_reopened_from_the_record` | `snapshots__tests__snapshot_the_board_reopened_from_the_record.snap` | The board reopened from that record, drawn by `render_board` |
| `snapshot_the_reopening_refusals` | `snapshots__tests__snapshot_the_reopening_refusals.snap` | The `Display` text of each of the eight `ReopenError` variants |

The reopened board's picture was laid beside T27's
`snapshot_the_board_after_the_scripted_game` by hand: below the header insta writes, the
two files are the same, line for line. No test compares them.

The refusals are made by hand in the snapshot test, where T27's were each got by asking
a board. No malformed record can be built outside the crate without a format crate,
since a record's fields are private; the unit tests show a record that earns each.

No earlier snapshot changed.

### What was verified, and how

Done on 2026-10-02 in the supplied Supacode worktree, on `cd27d19`, the merge of T27.
The worktree's branch is `ticket/T28-record-and-reopening`, as Supacode named it.
`.pixi/` was present. `just check` was green before any edit. Commands used rustup's
cargo through `PATH="$HOME/.cargo/bin:$PATH"`.

`just plan-spec board` printed 78 obligations. T27's table accounts for 75; the three
it marked as this ticket's are in the table above.

Closing lines of each recipe, from the last run:

| Recipe | Closing output |
| --- | --- |
| `just fmt-check` | `cargo fmt --all --check` (no diagnostic) |
| `just clippy` | ``Finished `dev` profile [unoptimized + debuginfo] target(s)``, no warning |
| `just metrics` | `::notice::Quality score: 100.0% (579 functions analyzed)`, no finding, and the probe satisfied |
| `just test` | `247 tests run: 247 passed, 0 skipped`; `test result: ok. 111 passed; 0 failed` and `ok. 1 passed` (the `compile_fail` example) for the doctests |
| `just snapshots-check` | `info: no unreferenced snapshots found`; `info: no snapshots to review` |
| `just wasm-check` | both feature sets on `wasm32-unknown-unknown` and on `wasm32v1-none`, each `Finished` |
| `just features` | `--no-default-features` and `--features serde`, each `Finished` |
| `just coverage` | `TOTAL` lines `99.86%` (2797 lines, 4 missed); `Finished report saved to target/llvm-cov/lcov.info` |
| `just doc` | `Generated .../target/doc/pawdoku/index.html` |
| `just check-docs` | `Validated 41 pages and 42 canonical topics.` |
| `just check` | `The worktree matches the check baseline.`; `All checks passed and the worktree is unchanged.` |

The exactness property runs 128 cases in about half a second, and about two under
coverage.

Coverage of `src/board`, from the same report:

| File | Lines | Missed | Cover |
| --- | --- | --- | --- |
| `board.rs` | 164 | 0 | 100.00% |
| `board/cell.rs` | 46 | 0 | 100.00% |
| `board/check.rs` | 30 | 0 | 100.00% |
| `board/error.rs` | 85 | 0 | 100.00% |
| `board/journal.rs` | 96 | 0 | 100.00% |
| `board/moves.rs` | 31 | 0 | 100.00% |
| `board/note.rs` | 97 | 0 | 100.00% |
| `board/reading.rs` | 24 | 0 | 100.00% |
| `board/record.rs` | 487 | 0 | 100.00% |

The four missed lines are the ones T27 reported, three in `random.rs` and one in
`sudoku/puzzle.rs`; this ticket touched neither.

**The tests were shown to bite.** Thirteen deliberate breaks, each made, run through
`just test` and reverted. Those runs stop at the first failures, so a row is what failed
before the run stopped. P is the property.

| Break | Tests that failed |
| --- | --- |
| Reopening does not take the undone moves back | `undone_moves_on_a_solved_puzzle_are_refused`, `refusals_come_in_a_fixed_order`, before the run reached B |
| An undo refused on a solved puzzle is passed over | `undone_moves_on_a_solved_puzzle_are_refused`, `refusals_come_in_a_fixed_order` |
| Reopening does not put the checks back | `a_record_built_by_hand_reopens`, `a_check_that_cannot_be_read_back_is_accepted` |
| A reopened check's answer is always yes | `a_check_that_cannot_be_read_back_is_accepted` |
| A reopened check keeps no more than the count of moves that stand | `a_check_that_cannot_be_read_back_is_accepted` |
| No move is refused for following the one that solves | `a_move_after_the_puzzle_is_solved_is_refused` |
| The two counts are not asked | `more_undone_moves_than_moves_are_refused`, `checks_in_a_record_with_no_move_are_refused`, `refusals_come_in_a_fixed_order` |
| A check after no move is accepted | `a_check_after_no_move_is_refused` |
| A check on a given is accepted | `a_check_on_a_given_is_refused`, `refusals_come_in_a_fixed_order` |
| A check's digit is not held to its range | `a_check_with_a_digit_out_of_range_is_refused` |
| The moves are made latest first | Five of the refusal tests, on the index reported |
| Writing says no move is undone | P, `undone_moves_reopen_undone_and_are_re_taken_the_same`, `a_record_is_written_from_a_board_at_any_point`, and two more |
| Writing leaves the checks out | P, `checks_reopen_with_their_indices_and_their_answers`, and four more |

The `proptest-regressions` file the breaks left was deleted; it recorded failures of
code that was broken on purpose.

Acceptance criteria read against the code:

- **The exports.** `pawdoku::board` exports `Record` and `ReopenError`; `Board::write`
  writes and `Board::reopen` returns a `Result`.
- **Nothing derived, nothing of the solution.** Stated above, under the table.
- **`Board` and `Puzzle` do not serialise.** Neither carries a serde derive, and
  `a_board_and_a_puzzle_do_not_serialise` stops compiling the day either does.
- **No recursion**, no `#[allow]`, no `#[expect]` added, no `qual:allow` line. No struct
  in `src/board` exceeds 20 methods: `Board` has 19, `Journal` 10.
- **`Cargo.toml` and `Cargo.lock` are unchanged**, both manifests.
- **`record.rs` has about 175 code lines** before its first `#[cfg(test)]`, of the 500
  allowed, and `tests/board.rs` is inside the thousand a test file is allowed.

### Deviations, and why

- **Eight variants where step 5 lists six.** The ticket asks for one variant for each
  way a record fails to read back and then names six. Two ways it describes elsewhere
  fit none of them. Moves undone on a solved puzzle is not a move after the puzzle is
  solved: every move reads back and the count is what is wrong, so it is
  `UndoneWhenSolved`. And a check asked after no move is the one fault of a check that
  the board has no refusal for, so it is `CheckAfterNoMove` beside `Check`, which
  carries the board's own refusal for the other three.
- **One line outside the list: `crates/pawdoku/src/board/journal.rs`.** Writing needs
  each move's kind, target and digit as the kind's own fields. `Board::moves` hands out
  a digit that may be absent whatever the kind, and turning that back into an enum
  would need an arm for a placement with no digit, which never happens. So the journal
  gained `made`, which lends the moves as it keeps them, and writing reads those.
- **Two sentences outside the list**, one in `README.md` and one in
  `crates/pawdoku/README.md`, which say what the crate holds today. T26 and T27 made the
  same two edits.
- **The refusals' snapshot is made by hand**, as "Snapshots taken" says.
- **Step 3, one line at a time.** The ten B tests were written before any code and
  failed to compile; the twenty-odd U tests were written with the types they build
  records from, against a reopening that did nothing, and failed on their assertions.
  Reopening was then written in one piece and not a refusal at a time. The breaks
  above are what shows each test holds its own clause.
- **The reading order in the ticket is not the checking order.** The ticket describes
  reopening as givens, moves, undone moves, checks. The two counts are asked first,
  because they need no board and a record that fails them should not cost a search.

### Handed back

- **`Record` is the value the bindings carry.** S04's drafted tickets (T15, the
  WebAssembly crate, first) take a game across their boundary as a `Record` through the
  `serde` feature, and build an exception from `ReopenError`'s text. A `Board` does not
  cross.
- **Records across versions**, the first open point, is untouched: no version field,
  no promise, and `Record`'s documentation says so. The serialised shape is serde's
  default for the four fields and for the move's enum, so a renamed field or variant is
  a change to what consumers have stored; the JSON snapshot is where it would show.
  The maintainer decides before the first release.
- **Fuzzing**, the second open point, is the maintainer's call. What stands in for it
  here: the refusal tests, the order test, and a property whose records are all sound.
  Nothing generates malformed records at random; a fuzz target, or a proptest strategy
  over the record's private fields, would.
- **A check's digit**, the third open point, stands as the ticket leaves it. Reopening
  refuses the four impossible checks it can recognise and accepts the rest, and
  `Record` documents it.
- **Reopening costs one search**, as opening does, and makes every move again. Its
  cost was not measured: the tests that reopen the solved fixture's fifty-one moves
  take about twenty milliseconds each, process start included, and that is all that
  is known.

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
