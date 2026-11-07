---
id: T41
title: "Play from a record in Rust, and the plain values every binding hands out"
status: open
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

### The test list

### What was verified, and how

### Deviations, and why

### Names T42 and T17 need

### Handed back

### Adversarial review

### Open points settled

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
