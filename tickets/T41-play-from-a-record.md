---
id: T41
title: "Play from a record in Rust, and the plain values every binding hands out"
status: done
depends_on: [T40, S12, T43]
parallel_with: [T13, T15]
branch: ticket/t41-play-from-a-record
estimated_size: L
---

# T41: Play from a record in Rust, and the plain values every binding hands out

## Context

T40 wrote play from a record into `board.allium`: a caller hands over a record and one
of `Playing`'s operations, and is handed back the record after it with what `Playing`
shows, or a refusal; nothing is kept between calls. The maintainer asked on 2026-10-31
for both ways of driving a board, a long-lived `Board` and these stateless functions,
and for the stateless functions to be **implemented in Rust, so that TypeScript and
Python share them**. This ticket builds them in the core, `crates/pawdoku`, because the
bindings crates depend on the engine and never on one another
(`docs/project/repository-map.md` lines 11 to 13), so the core is the one place both can
reach. T42 exposes them, and the `Board` class, to TypeScript; T17 will expose them to
Python.

**Settled by the maintainer, 2026-10-31**, on T40's questions: the contract and the
surface are named `PlayingFromARecord`; an operation on a record hands back the record
after it **and** the view of the board after it, and for `CheckCell` the check it added
as well; and opening from givens is one of the stateless functions. T40 states them and
this ticket builds them as stated. The maintainer also answered what a saved record
promises across versions of the engine: a record names its format version, a record of
another version is refused with a clear message, and no compatibility is promised. T43
(`tickets/T43-record-version.md`) builds that before this ticket, so the record every
function here takes and hands back is the versioned one, and a record of another
version is refused by `Board::reopen` like any record that does not reopen.

**One set of plain values for both ways and both languages.** A binding hands out what
`Playing` exposes as plain values: a JavaScript object, a Python `dict`. Today only
`Record`, `Position`, `Given`, `Tier`, `SeededStream` and `ReplayStream` derive serde, and
`BoardCell`, `Move`, `Check`, `Note`, `Status`, `MoveKind`, `SearchResult` and `Verdict`
do not. If each binding wrote its own serialisable copies, the two bindings would drift
from each other and from the surface. So the core gains, under its existing optional
`serde` feature, `Serialize` for every value the surfaces expose, in one wire shape, and
the stateless functions return those same values. The `Board` class in T42 returns them
too, so playing from records step by step and holding a board give the same values, and
a test here holds that.

**What the core still does not do.** It gains no format crate and no format: values
serialise through whatever a consumer brings, as `Record` already does
(`docs/explanation/architecture.md`, "The record"). It keeps nothing between calls. It
never serialises a `Board` or a `Puzzle`, which `tests/api_bounds.rs` holds under the
`serde` feature, because a board holds its solution.

**What S12 decides first.** This ticket depends on S12's hand-back for: the casing of
the wire shape (the ticket writer proposed snake_case, `Record`'s today, the
specifications' names and Python's idiom); the shape of the readings after a move
(proposed: a separate reading per move index, never carried inside every move); and what
a generation result carries for `generation.allium`'s Replay (proposed: the tier, the
seed when there is one, the stream's index at the start, `draws_taken`,
`GENERATION_VERSION`, `RANDOM_VERSION` and the givens, never the solution). Where S12's
"Amendments to T41 and T42" changes a choice below, follow S12 and say so under
Deviations. T40's hand-back notes name the contract, the surface and the operation as the
specification spells them; the Rust names follow them.

**The metrics gate shapes where the code goes.** `Board` has nineteen methods
(`architecture.md`, "The board") and the gate caps a type at twenty. A view of a board
and a reading after a move are therefore associated functions of the values they make,
not methods of `Board`, and the stateless functions are free functions in a new
submodule of `board`, which `rustqual.toml`'s rule for `crate::board` already covers.

**Paths other open tickets edit.** T38 and T39, open, edit `crates/pawdoku/src/lib.rs`,
`docs/explanation/architecture.md`, `docs/reference/testing.md` and `CHANGELOG.md`; T39
may edit `crates/pawdoku/src/generation.rs` or `generation/grid.rs`, depending on the
answer to T37's Open point on the grid attempt. T43, which merges before this ticket
starts, edits `crates/pawdoku/src/board.rs`, `board/record.rs`, `tests/board.rs`,
`tests/snapshots.rs`, the record's snapshots, `docs/explanation/architecture.md`,
`docs/reference/testing.md` and `CHANGELOG.md`; this ticket starts from its result. Each shared edit here is a section, a
paragraph or a few items; whichever pull request merges second merges `main` first, the
deviation from CONVENTIONS.md §11 that T31 and T33 recorded.

Read first: `AGENTS.md`; `.agents/skills/rust-change/SKILL.md` and the `tdd` approach
the house uses (a test list first, revisited after every loop); T40's ticket and
hand-back notes; S12's hand-back notes; `docs/specs/board.allium` (`Recording`,
`Playing`, `Reopening`, and T40's contract and surface); `docs/specs/solver.allium`
(`SearchResult`); `docs/specs/generation.allium` (`GenerationResult`, Scope's Replay);
`crates/pawdoku/src/board.rs` and every file under `board/`; `solver/result.rs`;
`generation.rs`; `random.rs`; `tests/api_bounds.rs`, `tests/board.rs`,
`tests/snapshots.rs`; `docs/explanation/architecture.md`; `docs/reference/testing.md`;
`rustqual.toml`.

## Goal

`pawdoku::board` offers play from a record: opening from givens, an operation on a
record, and a reading of a record and of the board after one of its moves, each returning
plain values or a refusal with stable `Display` text. Under the `serde` feature every
value `Playing`, `SearchResult` and `GenerationResult` expose serialises in one wire
shape, and the stateless functions return exactly the values a held `Board` gives. A
generation from a seed returns a serialisable result that carries what replays it and
never the solution. `just check` is green and the coverage floor holds.

## Non-goals

- Any binding: T42 for TypeScript, T17 for Python.
- A format crate, a format, or a version field in `Record` (T43 adds it).
- Serialising a `Board`, a `Puzzle` or a `WellPosed`.
- Any change to what a board does: every operation is `Board`'s own.
- `Draw` records: `human-solving.allium`'s, built when that module is.
- Changing `Record`'s wire shape, unless S12's casing answer requires it; then the
  change and the snapshot's are called out in the CHANGELOG.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `crates/pawdoku/src/board.rs` | core | `mod` lines and re-exports for the new items; the module's documentation |
| `crates/pawdoku/src/board/from_record.rs` | core | New: the stateless functions, the operation value, what a call hands back, and its error (steps 3 to 5) |
| `crates/pawdoku/src/board/view.rs` | core | New: the view of a board and the reading after a move (step 3) |
| `crates/pawdoku/src/board/cell.rs` | core | The wire shape of `BoardCell` |
| `crates/pawdoku/src/board/check.rs` | core | The wire shape of `Check` |
| `crates/pawdoku/src/board/moves.rs` | core | The wire shapes of `Move`, without its reading, and `MoveKind` |
| `crates/pawdoku/src/board/note.rs` | core | The wire shape of `Note`: its digits |
| `crates/pawdoku/src/sudoku/puzzle.rs` | core | The wire shape of `Status` |
| `crates/pawdoku/src/solver/result.rs` | core | The wire shapes of `SearchResult` and `Verdict` |
| `crates/pawdoku/src/generation.rs` | core | The generation result and generation from a seed (step 6) |
| `crates/pawdoku/src/lib.rs` | core | The crate documentation's paragraph on the board and a doc example of play from a record |
| `crates/pawdoku/tests/from_record.rs` | core tests | New: the integration tests of step 7 |
| `crates/pawdoku/tests/api_bounds.rs` | core tests | Bounds for every new public type; `Board` and `Puzzle` still do not serialise |
| `crates/pawdoku/tests/snapshots.rs` | core tests | Snapshots of the wire shape (step 8) |
| `crates/pawdoku/tests/snapshots/` | snapshots | The new `.snap` files step 8 accepts |
| `docs/explanation/architecture.md` | explanation | A section "Play from a record"; "What crosses each boundary" names the wire shape |
| `docs/reference/testing.md` | reference | "The board's tests" gains `tests/from_record.rs`; "Snapshots" lists the new ones |
| `CHANGELOG.md` | maintainer docs | Under `[Unreleased]`, Added |
| `tickets/README.md` | ticket index | T41's row set `done` |
| `tickets/T41-play-from-a-record.md` | ticket | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t41-play-from-a-record` from `main` after T40, S12
   and T43 have merged. Run `just initialize` and `just install-tools` if `.pixi/` or
   `.tools/bin` is incomplete; each uses the network and this ticket authorises it. Run
   `just check` and quote its last line. Read S12's "Amendments to T41 and T42" and
   T40's "Names T41 needs", and write the decisions they fix at the top of your
   hand-back notes before any code.

2. **The test list.** Run `just plan-spec board` and take the obligations T40 listed for
   its new clauses as suggestions. Write the test list in the hand-back notes, one line
   per test, each naming the clause it proves, before the first test is written; revisit
   it after every red-green loop, and record what was added and why. Tests come first,
   one at a time.

3. **The view and the reading.** In `board/view.rs`, public, `#[non_exhaustive]`,
   `#[derive(Clone, Debug, PartialEq, Eq)]`, `Serialize` under the `serde` feature:
   - a view of a board, made by an associated function from `&Board`, holding exactly
     what `Playing` exposes: the status, `is_full`, `is_consistent`, `can_undo`,
     `can_redo`, the 81 cells, the moves and the checks;
   - a reading after a move, made from `&Board` and a move index, holding each cell's
     digit and note as they stood once that move and no later one had been made, read
     through `Move::digit_after` and `Move::note_after`; an index that names no move is
     a refusal, not a panic.

   The wire shape, in the casing S12 chose: a cell is flat (row, column, digit,
   `is_given`, `is_conflicting`, and its note's digits while it is shown, absent
   otherwise); a move is its index, kind, target, digit and `is_undone`, never its
   reading; a check is its index, `after_move`, target, digit and `is_right`; a note is
   the list of its digits in order. Whether each comes from a derive on the existing
   value or a serialised form (`#[serde(into = ...)]` or `serialize_with`) is the
   implementer's choice per type, with the reason in a comment: a private field never
   reaches the wire under its private name. Nothing derived reads the solution, so
   `TheSolutionIsNeverShown` holds by construction; a test holds it too.

4. **The operation and what a call hands back.** In `board/from_record.rs`: an
   operation value, a public `#[non_exhaustive]` enum with one variant for each of
   `Playing`'s seven (place a digit, erase, write a mark, strike a mark, check a cell,
   undo, redo), each carrying the position and digit it needs and nothing else, with
   `Serialize` and `Deserialize`; what a call hands back, the record after it and the
   view of the board after it, and for a check the check it added, with `Serialize`;
   and the error, a `thiserror` enum, `#[non_exhaustive]`, with stable `Display` text:
   one variant wrapping `ReopenError` and one wrapping `PlayError`, each
   `#[error(transparent)]` or with a sentence that says which stage refused, and one
   for a move index that names no move. A unit test pins each variant's text, as
   `board/error.rs` does.

5. **The functions**, free functions in `board/from_record.rs`, re-exported from
   `pawdoku::board`, each with a doc example: open from givens, giving the record and
   the view of the board opened, or the solver's refusal; apply an operation to a
   record, which reopens it with `Board::reopen`, makes the operation with the board's
   own method, and writes it with `Board::write`; read a record, giving its view; and
   read a record after one of its moves. Every one takes values and returns values; none
   panics; the record handed over is borrowed and unchanged. The names are T40's.

6. **The generator's result.** In `generation.rs`, a public `#[non_exhaustive]`
   result value, `Serialize` under `serde`, holding the tier, the givens, `draws_taken`,
   the stream's index at the start, `GENERATION_VERSION` and `RANDOM_VERSION`, and the
   seed when there is one (or exactly what S12's question 3 settled); and never the
   solution, which `GenerationResult` does not expose. Two ways to get one: from a tier
   and a seed, beginning a `SeededStream` at index zero, which is the stateless call; and
   from a tier and a `&mut dyn RandomStream`, reading the index before and after, which
   is the class path. `generate` itself is unchanged, and both are built on it.

7. **The tests**, in `tests/from_record.rs` and in-module where they test one function:
   - for every scripted game in `tests/board.rs` and `tests/snapshots.rs`, playing from
     records step by step and holding one board give equal records and equal views at
     every step, and equal readings after every move;
   - each refusal: a record that does not reopen, refused with the `ReopenError` inside;
     an operation the guard refuses, with the `PlayError` inside; a move index past the
     last; and the record handed over equal to itself after each;
   - the view never holds a digit at an empty cell, the reading after a move equals
     `Move::digit_after` and `Move::note_after` for every cell, and a check's answer
     comes back only through the checks;
   - a proptest over random sequences of operations, from the fixture's givens, holding
     the two ways equal after every operation, with any refusal refused by both;
   - the generation result: from a seed, the same as `generate` with
     `SeededStream::new(seed)`; `draws_taken` 23 for tiers 3 to 5 and 104 for tiers 1 and
     2 when the first grid attempt succeeds, as `architecture.md` lines 296 to 298 say;
     from a stream begun elsewhere, the start index recorded;
   - under the `serde` feature, each wire shape round-trips where it deserialises
     (`Record`, the operation) and serialises to the fields above.

8. **Snapshots**, through the existing harness and its rule (a snapshot shows a value to
   a reviewer and detects a change; it never proves a clause): the JSON of the view of
   the scripted game, of a reading after one of its moves, of what applying one operation
   hands back, of a search result, and of a generation result from seed 0 in tier 3. Run
   `just snapshots-accept`, read every new `.snap`, and quote one in the notes.

9. **The bounds and the pages.** `tests/api_bounds.rs` asserts `Send + Sync + 'static`,
   `Clone` and `Debug` for every new public type, and that `Board` and `Puzzle` still do
   not serialise. Then the pages: `architecture.md` gains "Play from a record" (what it
   composes, what it costs, why it lives in the core, the one wire shape) and a sentence
   in "What crosses each boundary"; `testing.md` and the CHANGELOG as listed.

10. Run `just fmt-check`, `just clippy`, `just metrics`, `just test` and
    `just snapshots-check`, then `just check`, and quote each closing line, with the
    coverage figure.

11. If R2 changed anything, run step 10's recipes again and quote them. Set
    `status: done` and the index row. Commit, and ask before pushing.

**Adversarial review**, by the procedure in `tickets/README.md`, "Adversarial
review":

- **R1**, after steps 3 to 6 and before step 7's proptests: the public values and their wire shape. Focus: what a binding would be stuck with.
- **R2**, after step 10 and before step 11: the ticket whole.

## Acceptance criteria

- Play from a record exists as T40 states it, every function with a doc example, and
  every new public type `#[non_exhaustive]`, `Clone`, `Debug`, `Send + Sync + 'static`.
- Holding a board and playing from records give equal records, views and readings at
  every step of every scripted game and of the proptest.
- Every refusal is a `Result`, never a panic, with a pinned `Display` text.
- Under `serde`, every value the three surfaces expose serialises in one shape. No
  serialised value of the board or of the generation result holds a digit read from the
  solution except a check's answer; a search result's solutions do, because
  `solver.allium`'s `SearchResult` exposes them.
- `Board` still has at most twenty methods, `just metrics` is green, and the coverage
  floor holds.
- `just check` green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just fmt-check
just clippy
just metrics
just test
just snapshots-check
cargo test -p pawdoku --features serde --test from_record --locked
rg -n 'pub fn' crates/pawdoku/src/board/from_record.rs crates/pawdoku/src/board/view.rs
rg -n -c 'fn ' crates/pawdoku/src/board.rs
just coverage
just check
git status --porcelain
```

Expected: each recipe green; the new tests passing; the public functions named as T40's
hand-back says; the coverage figure at or above 90; `All checks passed and the worktree
is unchanged.`; only paths in Files touched.

## Hand-back notes

### Decisions fixed by S12 and T40

Written before any code, from S12's "Amendments to T41 and T42" and T40's "Names T41
needs".

- **The four functions are T40's**: `open`, `play`, `read` and `reading_after`, free
  functions in `board/from_record.rs`, re-exported from `pawdoku::board`. `play` takes one
  operation value, not one function per operation (this ticket's Open point, settled for
  one entry).
- **The operation** is `Operation`, one variant per `OperationKind`: `Place`, `Erase`,
  `WriteMark`, `StrikeMark`, `CheckCell`, `Undo`, `Redo`, serde's default PascalCase,
  `#[serde(tag = "kind")]`, with fields `position` and `digit` (S12). T40's value names
  the cell `cell`; the Rust field is `position`, as every `Board` method and S12's wire
  name it.
- **What a call hands back** is `Played { record, view, check }`, `check` present only
  after `CheckCell` and absent from the wire otherwise (S12). `open` hands back the same
  value with no check. `read` hands back the view alone.
- **The view** is `BoardView`, made by `BoardView::of(&Board)`: `status`, `is_full`,
  `is_consistent`, `can_undo`, `can_redo`, `cells`, `moves`, `checks`, in that order,
  with `#[expect(clippy::struct_excessive_bools)]` (S12).
- **The reading** is `ReadingAfter { move_index, cells }`, made by
  `ReadingAfter::of(&Board, move_index)`, 81 `CellReading { row, column, digit, note }`
  in row order (S12). `CellReading` is T40's `value Reading`; the name differs because
  `board/reading.rs` already holds the private `Reading` the moves read through.
  `move_index` counts from 1; 0 and anything past the last move are refused.
- **The error** is `FromRecordError`: `Reopen(ReopenError)` and `Play(PlayError)`, both
  `#[error(transparent)]` (S12), and `NoSuchMove { move_index }`. `open` refuses with
  `SolveError` and `read` with `ReopenError`, each the stage's own type, since neither
  has a second stage.
- **The wire shape** (S12): snake_case fields, PascalCase enum values; an absent digit is
  `null`; a cell's `note` is absent while not shown and `[]` when shown and empty; a
  reading's `note` is always a list; no `#[serde(flatten)]` anywhere. A move never
  carries its reading. Every `Serialize` sits under `cfg_attr(feature = "serde", ...)`.
- **The generation result** is `GenerationResult { tier, seed, start_index, draws_taken,
  generation_version, random_version, givens }`, in that order; `seed` is `None` from a
  stream; `draws_taken` is the index after `wrapping_sub` the index before; no status.
  The refusal is `GenerationRefused::Refused { tier, draws_taken, error }`, `Display`
  `"{error}"`, with `const fn` accessors `tier` and `draws_taken` (S12). The two entries
  are `generate_from_seed(tier, seed)` and `generate_from_stream(tier, stream)`, both
  built on one private function that reads the index on `Err` as on `Ok`. `generate` is
  unchanged.
- **The record is versioned** (T43): every function takes and hands back the record
  `Board::write` writes, and a record of another version is refused by `Board::reopen`
  inside `FromRecordError::Reopen`.

### The test list

Written before the first test, from `just plan-spec board` (89 obligations; T40's
contract and surface invariants do not appear in its output, so they are taken from
the specification's text) and from the ticket's steps. Each line names the test and
the clause it proves; T1 to T22 are its labels. Segment 1 builds those marked (1),
segment 2 the rest.

The view and the reading, `board/view.rs`:

- T1 (1) `a_view_holds_what_playing_exposes`: status, the four flags, 81 cells, the moves
  and the checks equal what `Board` hands out (`WhatIsShownIsWhatPlayingShows`).
- T2 (1) `the_reading_after_a_move_is_digit_after_and_note_after`: 81 cells in row order,
  each equal to `Move::digit_after` and `Move::note_after`, a note beneath a digit
  included (`reading_after`, T40's `Reading`).
- T3 (1) `an_undone_move_still_has_a_reading` (`reading_after`: "a move undone is still
  one of the record's moves").
- T4 (1) `an_index_that_names_no_move_is_refused`: 0, one past the last, `usize::MAX`
  (`ARefusalHandsBackNothingElse`).

The functions, `board/from_record.rs`:

- T5 (1) `each_refusal_says_what_was_refused`: `NoSuchMove`'s text pinned, and the two
  wrapping variants' text equal to the wrapped refusal's (invariant 3, S12).
- T6 (1) `open_hands_back_the_record_and_view_of_the_board_opened`, and refuses givens as
  `Board::open` does (`PlayingFromARecordIsHoldingTheBoard` for `open`).
- T7 (1) `play_makes_each_operation_as_a_held_board_does`: all seven kinds, equal record
  and view, a check only after `CheckCell` (`PlayingFromARecordIsHoldingTheBoard`).
- T8 (1) `a_record_that_does_not_reopen_is_refused_first`: before the guard and before
  the index, the record handed over unchanged (`ARefusalHandsBackNothingElse`).
- T9 (1) `an_operation_the_guard_refuses_is_refused_with_its_play_error`, the record
  unchanged.
- T10 (1) `read_gives_the_view_of_the_board_reopened`, and the reopen refusal alone.
- T11 (1) `reading_after_reads_the_board_reopened`: equal to `ReadingAfter::of` on the
  held board (`TheSameBoardFromEitherWay`, the readings).

The generation result, `generation.rs`:

- T12 (1) `from_a_seed_is_generate_with_a_seeded_stream`: the same givens, `seed` named,
  `start_index` 0, `draws_taken` the stream's index after.
- T12a (1) `a_grid_found_at_once_takes_23_draws_or_104_where_the_order_is_drawn`: from
  a stream of zeros, whose first grid attempt succeeds, tiers 3 to 5 take 23 draws and
  tiers 1 and 2 take 104 (`architecture.md`, "The generator"). Added in the loop: T12
  holds `draws_taken` to the seeded stream's index, and seed 0's first attempt is not
  known to succeed in every tier, so the figures are held from zeros.
- T13 (1) `from_a_stream_records_where_it_began_and_only_its_own_draws`: `seed` `None`;
  five zeros refused with `draws_taken` 5; a script of five drawn three times first,
  2 and `start_index` 3.
- T14 (1) `a_hundred_failed_attempts_are_refused_with_every_draw_counted`: a hundred short
  seedings 1900, a hundred seedings with no solution 2200, each with the core's text.
- T15 (1) `each_tier_comes_back_as_asked`, on the result and on the refusal.
- T16 (1) `the_result_holds_no_solution`: its `Debug` prints no row of the solution
  (`TheSolutionIsNeverShown` for the generation result).

Step 7, `tests/from_record.rs` and the wire shape:

- T17 (2) Every scripted game of `tests/board.rs` and `tests/snapshots.rs`, played from
  records and on a held board: equal records, views and readings at every step
  (`RecordsKeepStepWithAHeldBoard`, `TheSameBoardFromEitherWay`).
- T18 (2) Each refusal through the public API, the record unchanged after each.
- T19 (2) The view holds no digit at an empty cell; a check's answer only through the
  checks (`TheSolutionIsNeverShown`, `ACheckNeverTellsTheDigit`).
- T20 (2) Proptest: random operations from the fixture, the two ways equal after each,
  any refusal refused by both (`TheSameBoardFromEitherWay`).
- T21 (2) Under `serde`: `Operation` and `Record` round-trip; every value serialises to
  exactly the fields above, read through a test-only capturing `Serializer` (no format
  crate is added, so serialised output is read field by field in the test).
- T22 (2) `NothingIsKeptBetweenCalls`: the same call twice, with other calls between,
  hands back equal values.

Revisited in segment 2, as built in `tests/from_record.rs`: T17 is
`every_scripted_game_is_the_same_game_either_way`, over four games restated as
operations (`THE_SCRIPTED_GAME`, `every_kind_of_move`, `in_play` and the whole game);
T18 is three tests, the guard's refusals on a fresh and a solved board, the index, and,
under `serde`, a record that does not reopen; T19 is
`a_view_shows_no_digit_the_player_has_not_placed`, with the serialised digits held to
the board's in T21; T20 is `any_script_is_the_same_either_way`, 128 cases; T21 split
into seven tests, one per value, after clippy's cognitive-complexity limit refused one;
T22 is `nothing_is_kept_between_calls`. Added in the loop: a solved board in T18 and
the property's prefill of 48 to 51 cells, because `Operation` has no "place the right
digit" step and a script would otherwise never reach a solved board.

### What was verified, and how

Step 1: `.pixi/` and `.tools/bin` were present, so neither install ran. `just check` at
the start (`e0a814c`) closed with `All checks passed and the worktree is unchanged.`,
coverage 99.33% of lines. Step 2: `just plan-spec board` lists 89 obligations.

Steps 3 to 6 were built test-first, a seam at a time, each test seen to fail (the build
refused the missing names) before its code was written: the view and the reading
(T1 to T4), the functions and the error (T5 to T11), the generation result and its
refusal (T12 to T16). One test was added to the list in these loops, T12a, for the
reason given there; two expected counts in the tests were corrected (the scripted
moves number 7 and 6, not 8 and 7). The wire shapes were printed once through
`serde_json` in a scratch crate outside the workspace, never committed, and read
against S12's amendments; the
tests that assert them are T21, in segment 2.

Segment 2, steps 7 to 10. The tests of step 7 arrived green, since segment 1's code
was built first; three were shown to bite by breaking the code on purpose: `play`
dropping a wrong check (three tests failed, the scripted games, the property and the
view's) and the cell's wire name `is_conflicting` renamed (the view's wire test
failed). The `proptest-regressions` file the break left was deleted. The wire shape is
read through a test-only capturing serializer and a deserializer over the same values,
in `tests/from_record.rs`; no format crate and no dependency was added. The
deserializer goes through `MapAccessDeserializer` for an externally tagged move of
`Record`, which `MapDeserializer` would refuse.

Step 8: nine JSON snapshots under the `serde` feature, accepted with
`just snapshots-accept` and each read: the view of `THE_SCRIPTED_GAME` (all four kinds
of move, an undone one, a shown note, a note waiting, an empty cell, a conflict, a check
each way), the reading after its sixth move, what a check on it hands back, the seven
operations, a search of each verdict, and the generation result from seed 0 in tier 3
and from a stream. One, the stream's, begins:

```text
{
  "tier": "Three",
  "seed": null,
  "start_index": 23,
  "draws_taken": 23,
  "generation_version": "generation-1",
  "random_version": "seeded-stream-1",
  "givens": [
```

Step 9: `tests/api_bounds.rs` takes the eight new types in both bound tests, a
`Serialize` bound for every wire value and `Serialize + DeserializeOwned` for
`Operation`; the `Board` and `Puzzle` test is unchanged. Each new accessor has a
doctest, as every older one does (29 of them). `lib.rs` gains a sentence and a doc
example of play from a record; `architecture.md` "Play from a record" and a paragraph in
"What crosses each boundary"; `testing.md` the file, the unit tests and the snapshots;
the CHANGELOG an entry under Added. Short doc lines segment 1 left in `generation.rs`
were reflowed.

Step 10, each closing line: `just fmt-check` `cargo fmt --all --check` with no diff;
`just clippy` `Finished`; `just metrics` `Quality score: 100.0% (887 functions
analyzed)`; `just test` `360 tests run: 360 passed, 0 skipped`, doctests `168 passed`;
`just snapshots-check` `no snapshots to review`, `no unreferenced snapshots found`;
`cargo test -p pawdoku --features serde --test from_record --locked` `16 passed`.
`rg -n 'pub fn'` lists `open`, `play`, `read`, `reading_after` and the two `of`
functions with the slice accessors; `rg -c 'fn ' board.rs` counts 24 lines, private
functions included, and `Board` keeps nineteen methods. `just check` closed with
`All checks passed and the worktree is unchanged.`, coverage 99.79% of lines (99.33%
at the start, 98.21% at the R1 checkpoint).

Step 11: R2 approved with no findings and changed nothing, so step 10's recipes were
not run again before the closing commit; `just check` ran once more on that commit.
Nothing was pushed.

### Deviations, and why

- **One branch.** The work is on `implement-typescript-surface-fullsend`, after S12, T40,
  T43 and T13; step 1's worktree and the merge condition are met by order on that
  branch.
- **`Operation`'s field is `position`, not T40's `cell`.** S12's amendment names the
  wire field `position`, and every `Board` method names the parameter so; the spec's
  value types it as `sudoku/Position`, which this is.
- **`CellReading`, not T40's `Reading`.** `board/reading.rs` holds the private
  `Reading` every move reads back through; a public value of the same name would shadow
  it in `board.rs`. T40's `cell: Position` is written flat as `row` and `column` on the
  wire (S12), and the value also offers `position()`.
- **A reading comes wrapped**, as `ReadingAfter { move_index, cells }` (S12), where T40's
  signature returns `List<Reading>`. The cells are that list, in row order.
- **`open` refuses with `SolveError` and `read` with `ReopenError`**, not with
  `FromRecordError`, since each has one stage only; `play` and `reading_after` have two
  and take `FromRecordError`.
- **`ReadingAfter::of` on a held board refuses with `FromRecordError::NoSuchMove`**, so
  the index's refusal is one value and one text whichever way the board is driven,
  which T42's class needs (S12, T42 step 3). Its text names the board: "no move of the
  board has the index N".
- **`Status`, `Verdict` and `MoveKind` gain `Serialize` only.** Nothing reads them back;
  `Operation` and `Record` are the values that deserialise.
- **The generation result's entries are named `generate_from_seed` and
  `generate_from_stream`**; neither the ticket nor S12 named them.
- **The scripts are restated in `tests/from_record.rs`**, each naming its source, since
  an integration test file cannot reach another's helpers and `tests/common/` is not in
  Files touched.
- **A record that does not reopen is made by deserialising, in the integration tests**,
  so that test is under `serde`; outside the crate there is no other way to make one.
- **More snapshots than step 8 lists**, as S12's amendment asks: the seven operations,
  a search of each verdict and a generation result from a stream.

### Names T42 and T17 need

Final after R2, and unreleased: no `pawdoku` release carries them yet.

- In `pawdoku::board`, re-exported from `board/from_record.rs`:
  `open(givens: impl IntoIterator<Item = Given>) -> Result<Played, SolveError>`,
  `play(record: &Record, operation: Operation) -> Result<Played, FromRecordError>`,
  `read(record: &Record) -> Result<BoardView, ReopenError>` and
  `reading_after(record: &Record, move_index: usize) -> Result<ReadingAfter,
  FromRecordError>`.
- `Operation`: `Place { position, digit }`, `Erase { position }`,
  `WriteMark { position, digit }`, `StrikeMark { position, digit }`,
  `CheckCell { position }`, `Undo`, `Redo`; `Serialize` and `Deserialize`, tagged by
  `kind`. On the wire `position` is an object `{ "row", "column" }`, as `Position`
  serialises everywhere.
- `Played`, with `record()`, `view()` and `check() -> Option<&Check>`; on the wire
  `record`, `view` and `check`, the last absent unless the operation was `CheckCell`.
- `BoardView::of(&Board)`, with `status()`, `is_full()`, `is_consistent()`,
  `can_undo()`, `can_redo()`, `cells() -> &[BoardCell]`, `moves() -> &[Move]` and
  `checks() -> &[Check]`. On the wire a cell is flat (`row`, `column`, `digit`,
  `is_given`, `is_conflicting`, `note` while shown); a move and a check name their cell
  `target`, an object like `position`.
- `ReadingAfter::of(&Board, move_index: usize) -> Result<ReadingAfter, FromRecordError>`,
  with `move_index()` and `cells() -> &[CellReading]`; `CellReading` with `position()`,
  `row()`, `column()`, `digit() -> Option<u8>` and `note() -> Note`, flat on the wire,
  `note` always a list.
- `FromRecordError::{Reopen(ReopenError), Play(PlayError), NoSuchMove { move_index }}`,
  the first two transparent and each with `From`, so `FromRecordError::from(refusal)` on
  `RecordVersion::readable()`'s refusal gives the text `ReopenError` gives. `NoSuchMove`
  reads `no move of the board has the index {move_index}`.
- In `pawdoku::generation`: `generate_from_seed(tier: Tier, seed: u64)` and
  `generate_from_stream(tier: Tier, stream: &mut dyn RandomStream)`, each
  `-> Result<GenerationResult, GenerationRefused>`. `GenerationResult` has `tier()`,
  `seed() -> Option<u64>`, `start_index() -> u64`, `draws_taken() -> u64`,
  `generation_version()`, `random_version()` and `givens() -> &BTreeSet<Given>`.
  `GenerationRefused::Refused { tier, draws_taken, error: GenerateError }`, with
  `tier()` and `draws_taken()`, and `Display` the `GenerateError`'s own text.
- Only `Operation` and `Record` deserialise. `Played`, `BoardView`, `ReadingAfter`,
  `CellReading`, `GenerationResult`, `Status`, `Verdict` and `MoveKind` serialise only,
  and `GenerationRefused` does not serialise; a binding throws its text.

### Handed back

- **To T42.** Three `u64` fields cross in the generation result (`seed`,
  `start_index`, `draws_taken`), and the seed is a `u64` argument, so the binding
  chooses how a `u64` crosses; the core does not. An operation's cell is the nested
  `position` while the view's cells and the readings are flat: the TypeScript types
  follow that shape as written. `read` refuses with `ReopenError` itself, not
  `FromRecordError`; both give the same text for a reopening refusal, since the variant
  is transparent. `Operation` is `#[non_exhaustive]`, so a binding's own `match` needs a
  wildcard arm.
- **To T15.** This ticket adds "Play from a record" and a paragraph in "What crosses
  each boundary" to `architecture.md`, rows for `tests/from_record.rs`, the new unit
  tests and the snapshots to `testing.md`, and an entry under `[Unreleased]`, Added, in
  `CHANGELOG.md`; T15 edits the same three files after it.

### Adversarial review

- **R1, round 1**, after steps 3 to 6, on the range from the start to the checkpoint
  `3afcdc3`, focus what a binding would be stuck with: the run handed back no output,
  so it gave no verdict and no findings, and nothing was folded in. It is not counted
  as an approval; the wire shape it was to read was read again by R2, on the whole
  range.
- **R2, round 1**, after step 10, on the range from the start to the checkpoint
  `af7b83a`, the ticket whole: approved with no findings. The reviewer found the
  record's replay, the order of the refusals and the isolation of calls as the contract
  states them, and the clause tests standing apart from the snapshots. Its note that
  its read-only sandbox could not run `just check` is an environment note, not a
  finding; `just check` ran here, as step 10 and step 11 say.

### Open points settled

Settled by the implementer, each from S12's amendments or recorded above:

- **Where the readings after a move live on the wire:** a separate `ReadingAfter` per
  move index, as proposed and as S12 settled; a move never carries its reading.
- **One function per operation, or one taking the operation value:** one, `play`,
  taking `Operation`, as proposed. A binding may still offer one method per operation.
- **Paths other open tickets edit:** T43 and T13 came before on the one branch; T38
  and T39 are not on it, and T15 follows, as Handed back says.

## Open points

- **Where the readings after a move live on the wire.** Proposed, pending S12: a
  separate reading per move index, so a move never carries 81 cells. If S12 finds a
  better shape, it is T40's to state first if it states anything new.
- **One function per operation, or one taking the operation value.** Proposed: one
  taking the value, so a binding has one entry and the value deserialises from a plain
  object; a binding can still offer one method per operation on its side. This is the
  implementer's call, recorded in the notes; it changes no clause.
- **Paths other open tickets edit.** See Context: T38 and T39 share `lib.rs`,
  `architecture.md`, `testing.md` and `CHANGELOG.md`, and T39 may share
  `generation.rs`. T43 edits `board.rs`, `board/record.rs`, the test files, the
  record's snapshots and the same three pages, and has merged before this ticket
  starts. T15, if open at the same time, shares `architecture.md`,
  `testing.md` and `CHANGELOG.md`.
