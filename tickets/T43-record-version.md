---
id: T43
title: "A record names its format version: the clause in board.allium, then the field, the refusal and the decision"
status: done
depends_on: [T40]
parallel_with: [T13, T15, S12]
branch: ticket/t43-record-version
estimated_size: M
---

# T43: A record names its format version: the clause in board.allium, then the field, the refusal and the decision

## Context

Play from a record (T40, T41) makes a record a consumer's state between calls, and a
game keeps it between visits, so a record written by one release of the engine may be
handed to another. Nothing says what happens then. `Record`'s documentation
(`crates/pawdoku/src/board/record.rs` lines 51 to 54, "# Across versions") says no
promise is made yet that a record written by one version of the engine reopens in
another, and that a record carries no version. `docs/explanation/architecture.md`, "The
record", says the same. `board.allium` says nothing either way.

**Settled by the maintainer, 2026-10-31.** Asked what a saved record promises across
versions of the engine, the maintainer chose: **version it, and promise nothing.** A
record names its format version. A record of another version is refused with a clear
message, and no compatibility is promised. The answer is a specification change and a
change to the core, so it is this ticket, carried by the `spec-change` skill: the
clause first, then the Rust.

**The precedent.** The engine already names what replay depends on with version
strings, each raised by whoever changes what it names:

- `docs/specs/generation.allium`'s config, `generation_version: String =
  "generation-1"` (line 350), whose comment says it is raised whenever anything changes
  in that module or in the solver, and `random_version: String = "seeded-stream-1"`
  (line 357), whose comment says only what it names;
- in Rust, `pub const GENERATION_VERSION: &str = "generation-1"` at
  `crates/pawdoku/src/generation.rs` line 95 and `pub const RANDOM_VERSION: &str =
  "seeded-stream-1"` at `crates/pawdoku/src/random.rs` line 31, each with a doctest
  pinning its value and a sentence naming the test that detects a change; the rule for
  raising `RANDOM_VERSION` is stated there, in its documentation, and not in the
  specification.

A record's version follows that pattern: one string in the module's config, one
constant in Rust, raised in the same commit as any change to what a record holds or
what reopening reads from it.

**What `board.allium` says of a record today.** `external entity Record {}` (line 126)
is empty on purpose: its comment says what a record is made of is nobody's here, and
that `Recording` says what writing one and reopening it must satisfy. `contract
Recording` (line 630) has two signatures, `write` and `reopen`, and two invariants,
`ReopeningIsExact` and `WritingChangesNothing`; its comment names no form and no place.
`surface Reopening` (line 774) fulfils it, with `AReopenedBoardIsTheSameBoard` and
`ARecordIsEnoughOnItsOwn`. The board's `config` (line 244) holds one figure, `side`.

So the clause belongs in `Recording`, as a statement about what `write` gives and what
`reopen` refuses, and the version string in the board's `config`. `external entity
Record` stays empty: naming the version a record was written under is something
`Recording` requires of the operations, not a field the specification lays out. Its
comment gains one sentence pointing at the new invariant.

**What the core does today.** `Record` (`record.rs` line 87) is `#[non_exhaustive]` with
four private fields, `givens`, `moves`, `undone` and `checks`, and derives `Serialize`
and `Deserialize` under the `serde` feature. `ReopenError` (`record.rs` line 223) has
eight variants, each with stable `Display` text that bindings build their exceptions
from. `Board::reopen` (`crates/pawdoku/src/board.rs` line 881) documents the order its
refusals are reported in: the first that applies is the one returned. A record's
serialised shape is pinned for review by
`crates/pawdoku/tests/snapshots/snapshots__tests__snapshot_the_record_of_the_scripted_game_as_json.snap`,
and the refusals' texts by `snapshot_the_reopening_refusals` in
`crates/pawdoku/tests/snapshots.rs`.

**Why this ticket follows T40.** Both edit `board.allium`, and T40 adds play from a
record beside `Recording`. Running after it keeps one owner of the module at a time.
T41 builds on the versioned record and runs after this ticket.

**Releases.** T13 publishes `pawdoku 0.1.0` soon after its first pull request merges,
and does not wait for this ticket. Adding a private field to `Record` and a variant to
`ReopenError` breaks no Rust API, because both are `#[non_exhaustive]` and the field is
private, but it changes the record's serialised shape. If 0.1.0 has been published when
this ticket merges, a record written by 0.1.0 has no version and is refused, which is a
breaking change for a consumer that kept one; it ships as 0.2.0, which semver allows
before 1.0. If 0.1.0 has not been published, the record is versioned from the first
release and nothing kept is affected.

One file this ticket edits is frozen under CONVENTIONS.md §11, `docs/manifest.yml`, for
the decision record's entry; step 1 asks before any edit.

Read first: `AGENTS.md`; `.agents/skills/spec-change/SKILL.md` and the `tend` skill;
`docs/explanation/specifications.md` and `docs/how-to/work-with-the-specs.md`;
`docs/specs/board.allium` whole, above all `external entity Record`, `config`, `contract
Recording` and `surface Reopening`; `docs/specs/generation.allium`'s config, lines 340
to 358; `crates/pawdoku/src/board/record.rs` whole and `Board::reopen` in
`crates/pawdoku/src/board.rs`; `crates/pawdoku/src/random.rs` and
`crates/pawdoku/src/generation.rs` for the two constants; `crates/pawdoku/tests/board.rs`,
`crates/pawdoku/tests/api_bounds.rs` and `crates/pawdoku/tests/snapshots.rs`;
`docs/explanation/architecture.md`, "The record"; T40's ticket and hand-back notes;
decision 0014 (`docs/decisions/0014-board-imports-solver.md`), the shape of a
spec-change record.

## Goal

`board.allium` states that a record names the format version it was written under and
that reopening refuses a record naming another, with both versions in the refusal, and
says no promise is made that a later engine reads an earlier version. The core builds
it: `pawdoku::board::RECORD_VERSION`, a version field in every record written, a
refusal with stable `Display` text that `Board::reopen` reports before any other, and,
under the `serde` feature, a small value that reads a record's version alone, so that
every binding shares that reading. Both
specification gates are clean with no waiver, `just check` is green, and a decision
record says why.

## Non-goals

- Reading an earlier version: no migration, no compatibility promise, no list of
  versions an engine accepts beyond its own.
- A format, or anything about where a record is kept: still the caller's.
- Any other change to what a record holds or to what reopening checks.
- The bindings: T41 builds play from a record on this record, T42 exports
  `RECORD_VERSION` to TypeScript and T17 to Python, and each reads a record's version
  through the value step 3 adds, which is this ticket's and not theirs.
- Any change to `sudoku.allium`, `solver.allium` or `generation.allium`.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `docs/specs/board.allium` | specification | `config` gains `record_version`; `contract Recording` states the version (step 2); three comments gain the version as their one exception: `external entity Record`'s, `Recording`'s preamble, and the Scope's "The record" bullet |
| `crates/pawdoku/src/board/record.rs` | core | `RECORD_VERSION`; the private version field; `ReopenError::Version`; the check; the version-only value under `serde`; "# Across versions" and the doctests rewritten; unit tests (steps 3 to 5) |
| `crates/pawdoku/src/board.rs` | core | The re-exports of `RECORD_VERSION` and the version-only value; `Board::reopen`'s list of refusals gains the version, first; the module documentation's paragraph on the record |
| `crates/pawdoku/tests/board.rs` | core tests | A record of another version is refused through the public API (step 4) |
| `crates/pawdoku/tests/api_bounds.rs` | core tests | The version-only value's bounds, under the `serde` feature |
| `crates/pawdoku/tests/snapshots.rs` | core tests | `snapshot_the_reopening_refusals` gains the version refusal (step 6) |
| `crates/pawdoku/tests/snapshots/*.snap` | snapshots | The record's two snapshots and the refusals', re-accepted (step 6) |
| `docs/explanation/architecture.md` | explanation | "The record": the version, the refusal, and no promise across versions (step 7) |
| `docs/reference/testing.md` | reference | The table's `board/record.rs` row: the version among the ways a record is refused; its `tests/board.rs` row: a record of another version refused through the public API; the doctests row: `RECORD_VERSION`'s value (step 7) |
| `docs/project/terminology.md` | project | The "Record, reopen" row: a record names its format version |
| `docs/decisions/0022-a-record-names-its-version.md` | decision | New record (step 8); the number is the next free one on the day |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | T00 follow-up | The record's entry. Frozen under CONVENTIONS.md §11; step 1 |
| `CHANGELOG.md` | maintainer docs | Under `[Unreleased]`, Changed: the record names its format version (step 7) |
| `tickets/README.md` | ticket index | T43's row set `done` |
| `tickets/T43-record-version.md` | ticket | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t43-record-version` from `main` after T40 has merged
   (`tickets/README.md`, "How to pick up a ticket"). Run `just initialize` if `.pixi/` is
   absent, `just install-tools` if `.tools/bin` lacks cargo-hack or rustqual, and
   `just install-allium` if `.tools/bin/allium` is absent; each uses the network and this
   ticket authorises it. Run `just check`, `just check-specs`, `just analyse-specs` and
   `just plan-spec board`, and quote their closing lines and the count of obligations.
   The maintainer has confirmed that `docs/manifest.yml` may be edited here, for the
   record's entry in the same commit as the record, and has answered the first three
   Open points; the answers are under "Open points settled".

2. **The clause**, written with the `tend` skill in the module's voice:

   - **`config`** gains `record_version: String = "record-1"`, with a comment in the
     shape of `generation.allium`'s `generation_version`: it names what a record holds
     and what reopening reads from it, and no format, and is raised whenever either
     changes, in this module or in the core.
   - **`contract Recording`**: `write`'s comment says the record names
     `config.record_version`; `reopen`'s comment says a record that names any other
     version, or none, is refused before anything else in it is read, and the refusal
     names both the version the record names and the one this engine reads. One new
     `@invariant`, proposed `ARecordNamesItsVersion`, says both, with its reason in the
     comment: the record is a consumer's state between calls, and a consumer told which
     version it holds can say so to a player. A comment says, in the module's words,
     that no promise is made that an engine reads a record of an earlier version.
   - **`external entity Record`**: stays `{}`. Its comment gains one sentence: beyond
     the version `Recording` requires it to name, what a record is made of is nobody's
     here.
   - **Two comments that say a record has no stated form**, `Recording`'s preamble ("It
     names no form and no place", lines 627 and 628) and the Scope's "The record" bullet
     ("nothing of the form a record takes", lines 53 and 54), each gain the same one
     exception, so that no sentence of the module contradicts the new clause: beyond
     the version a record names, nothing is said of its form.

   `ReopeningIsExact` and `ARecordIsEnoughOnItsOwn` stand as written: a record of this
   engine's version still carries everything reopening needs. Run `just check-specs` and
   `just analyse-specs` after each clause, and fix every diagnostic and finding at its
   root; no `allium-ignore`. Then `just plan-spec board`, and list the obligations the
   clause adds for step 4's test list.

3. **The Rust**, following the `rust-change` skill:

   - `pub const RECORD_VERSION: &str = "record-1";` in `record.rs`, re-exported from
     `pawdoku::board` beside `Record` and `ReopenError`, with a doc comment in the shape
     of `GENERATION_VERSION`'s: what it names, when it is raised, that whoever changes
     what a record holds changes it in the same commit, and that the record's JSON
     snapshot is what shows such a change to a reviewer. A doctest pins its value.
   - `Record` gains a private field holding the version it was written under. `Record::of`
     writes `RECORD_VERSION` into it. Under the `serde` feature the field is first in the
     serialised shape and, when absent, deserialises as empty
     (`#[serde(default)]`). In a self-describing format with named fields, such as JSON,
     a record written before this field existed then deserialises and is refused by
     `Board::reopen` with the version refusal. In a positional format, which hands a
     struct's fields over as a sequence in order, a new first field shifts every other,
     so such a record fails to deserialise in the consumer's format before `reopen` is
     reached; the core cannot change that, and the third Open point is the same caveat
     for later shapes. The field's name on the wire is an Open point.
   - `ReopenError` gains one variant, proposed `Version`, holding the version the
     record names and the version this engine reads, with stable `Display` text that
     names both. Proposed: `the record is of format version "record-0", and this engine
     reads "record-1"`, and, when the record names none, `the record names no format
     version, and this engine reads "record-1"`. A unit test pins each text, as the
     other variants' are pinned.
   - `Board::reopen` checks the version first, before the counts, because nothing else
     in a record of another version can be trusted to mean what this engine reads. Its
     list of refusals in order gains the version at the head.
   - **Reading the version alone.** Under the `serde` feature, a public
     `#[non_exhaustive]` value in `record.rs`, re-exported from `pawdoku::board`, proposed
     `RecordVersion`, that deserialises a record's `version` field and ignores every
     other, and that gives the same `ReopenError::Version` refusal `Board::reopen` gives
     when the version it read is not `RECORD_VERSION`. It takes a record that names none,
     and a record of a later shape whose other fields it could not read. T42 and T17 read
     through it before the rest of a record, so that a record of another shape is refused
     by the version refusal's text and not by the format's error. It is a reading, not a
     second check: `Board::reopen` still checks the record it is handed.
   - The "# Across versions" paragraph of `Record`'s documentation is rewritten: a
     record names its format version, `RECORD_VERSION`; reopening refuses a record that
     names another or none; no promise is made that a later engine reads an earlier
     version. The doctest above it still holds; add one that shows the version refusal.

4. **The tests, written first**, one at a time from a test list written in the
   hand-back notes, each naming the clause or obligation it proves, and revisited after
   every red-green loop:

   - a record of another version is refused with `ReopenError::Version`, naming both
     versions, through `Board::reopen`; and that refusal comes first when the record
     also fails in another way;
   - under `serde`, a record's serialised form with the version field removed
     deserialises and is refused with the version refusal naming no version; this needs
     a deserialiser that hands the fields over by name, such as a hand-written
     `serde::de::MapAccess` in the shape of the existing `Fields` helper, with no format
     crate, because the sequence form cannot leave out a first field;
   - under `serde`, a record's serialised form with another version deserialises and is
     refused naming that version;
   - under `serde`, the version-only value reads a record's serialised form with its
     other fields renamed or missing, and gives the version refusal naming what it read,
     or no version; given a record of this engine's version it gives no refusal;
   - every record `write` gives names `RECORD_VERSION`, and the round trip through
     `write` and `reopen`, and through the serialised form, still holds:
     `ReopeningIsExact` and `WritingChangesNothing` stand;
   - `ReopenError` is still `Send + Sync + 'static`, `Clone` and `Debug`
     (`tests/api_bounds.rs` already asserts it; quote it green).

   Unit tests go in `record.rs`'s `tests` module beside the existing ones, which build
   records by hand; the public-API test goes in `tests/board.rs`. Two existing unit
   tests in `record.rs` change with the field, and the change is said in the notes:
   `a_record_holds_what_it_states_and_no_answer`, whose list of stated fields gains the
   version's; and the `Fields` sequence helper behind
   `a_malformed_record_deserialises_and_is_refused_at_reopening`, which hands a
   record's four fields over in order and must now hand over the version first. That
   test's malformed record must still be refused for its own fault, so the helper hands
   over `RECORD_VERSION`, and a second case handing over another version is refused
   with the version refusal.

5. Run `just fmt-check`, `just clippy`, `just metrics` and `just test`, and quote each
   closing line.

6. **Snapshots**, through the existing harness and its rule (a snapshot shows a value
   to a reviewer and detects a change; it never proves a clause). Add the version
   refusal to `snapshot_the_reopening_refusals`. Run `just snapshots-accept`; read the
   changed `snapshot_the_record_of_the_scripted_game_as_json`,
   `snapshot_the_record_of_the_scripted_game` and `snapshot_the_reopening_refusals`
   `.snap` files whole, and quote the JSON one in the notes. Say in the commit message
   and the notes that the record's serialised shape changed, and how.

7. **The pages.** `docs/explanation/architecture.md`, "The record": replace the
   sentence that says no promise is made yet with what is now so: a record names its
   format version, reopening refuses another first, and no promise is made across
   versions; name the decision. `docs/project/terminology.md`'s "Record, reopen" row: a
   record names its format version. `CHANGELOG.md` under `[Unreleased]`, Changed: every
   record names its format version, `RECORD_VERSION`, and `Board::reopen` refuses a
   record of another version, or of none, with `ReopenError::Version`; a record's
   serialised shape gains the field. If `pawdoku 0.1.0` has been published by then,
   the entry says a record kept from 0.1.0 is refused, and that the release carrying it
   is 0.2.0, and that in a positional format such a record fails to deserialise rather
   than being refused. `docs/reference/testing.md`'s table of where tests
   live: the `board/record.rs` row gains the version among the ways a record is
   refused, the `tests/board.rs` row gains a record of another version refused through
   the public API, and the doctests row names `RECORD_VERSION`'s value beside
   `RANDOM_VERSION`'s.
   `docs/reference/api.md` names neither version constant on 2026-10-31 and is not
   changed; check it with `rg -n 'RANDOM_VERSION|GENERATION_VERSION' docs/reference/api.md`
   and hand back any line that has appeared since.

8. **The decision record**, `docs/decisions/0022-a-record-names-its-version.md` in the
   shape of 0014, `canonical_for: [decision_record_version]`; use the next free number on
   the day (0019, 0020 and 0021 are projected for T13, T15 and T40) and rename the file,
   the title, the index row and the manifest entry if it is taken. Context: play from a
   record makes the record a consumer's state; the maintainer's question and answer of
   2026-10-31. Decision: a record names its format version; reopening refuses another
   first, naming both; no compatibility is promised; the version is a string in the
   shape of `generation_version` and `random_version`. Consequences, including the ones
   that hurt: a game discards a record of another version and must say so to the
   player; any change to what a record holds raises the version and orphans every kept
   record; a record kept from a release without the field is refused, and in a
   positional format fails to deserialise rather than being refused. What would reopen
   it: a consumer that needs records to survive an upgrade, which would bring a promise
   and a reader for each earlier version. Register it in `docs/decisions/README.md` and
   `docs/manifest.yml`, and move "The next decision this repository takes is" on.

9. `just plan-spec board` again, with the new count of obligations quoted;
   `just check-docs`, then `just check`, quoted. Set `status: done` and the index row.
   Commit, and ask before pushing.

**Adversarial review**, by the procedure in `tickets/README.md`, "Adversarial
review":

- **R1**, after step 2 and before step 3: the clause, before any Rust. Focus: the refusal's order and what a record must name.
- **R2**, after step 8 and before step 9: the Rust, before `status: done`. Focus: the refusal's order and the wire shape.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` report no diagnostics and no findings for
  every module, with no waiver.
- `board.allium`'s `config` holds `record_version`; `Recording` states the version and
  its refusal; `external entity Record` is still `{}`; no other rule, invariant,
  surface or contract changed, and the module's `use` lines are unchanged.
- `pawdoku::board::RECORD_VERSION` exists with a doctest; every record written names
  it; `Board::reopen` refuses a record of another version, or of none, with
  `ReopenError::Version` before any other refusal, and never panics.
- Under the `serde` feature, the version-only value is public, re-exported from
  `pawdoku::board`, meets invariant 3 with a doctest, and gives the same refusal as
  `Board::reopen`.
- Each test of step 4 passes; each `Display` text is pinned; the coverage floor holds.
- The three snapshots are re-accepted, and the notes quote the JSON one.
- The decision record is in the manifest and the index.
- `just check` is green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just check-specs
just analyse-specs
just plan-spec board
rg -n 'record_version|ARecordNamesItsVersion' docs/specs/board.allium
rg -n 'RECORD_VERSION|RecordVersion|Version \{' crates/pawdoku/src/board/record.rs crates/pawdoku/src/board.rs
cargo test -p pawdoku --features serde --locked record
just snapshots-check
git diff --stat main -- docs/specs crates/pawdoku/tests/snapshots
just check-docs
just check
git status --porcelain
```

Expected: every module clean; the obligations, with the count; the config line and the
invariant; the constant, its re-export and the variant; the record's tests passing;
snapshots unchanged after acceptance; one specification file and three snapshot files
changed; the documents gate green; `All checks passed and the worktree is unchanged.`;
only paths in Files touched.

## Hand-back notes

### What was verified, and how

Run on 2026-11-07 on the branch `implement-typescript-surface-fullsend`, from
`a3cb5db6baee0a456cec93bbd6d59a1dbc8df6b3`, with allium 3.6.1 from `.tools/bin`.

- **Before any clause.** `just check`: "All checks passed and the worktree is
  unchanged." `just check-specs`: "allium check: 9 specifications, no diagnostics and
  no findings." `just analyse-specs`: "allium analyse: 9 specifications, no diagnostics
  and no findings." `just plan-spec board`: 88 obligations.
- **After the clause.** Both specification gates read as before, every module clean,
  with no waiver; each was run after the config line, after `Recording`, and after the
  three comments. `just plan-spec board`: 89 obligations. The one added is
  `config-default.record_version`. `allium plan` derives nothing from a prose
  `@invariant`, so `ARecordNamesItsVersion` reaches the test list from its text.
- `external entity Record` is still `{}`; no rule, top-level invariant, surface or
  `use` line changed, and `contract PlayingFromARecord` is untouched.

### The test list

Written before any Rust, from the clause and the obligations; revisited after every
red-green loop.

1. `config-default.record_version`: `RECORD_VERSION` is `"record-1"` (a doctest).
2. `ARecordNamesItsVersion`, the write half: every record `write` gives names
   `RECORD_VERSION`.
3. `ARecordNamesItsVersion`, the refusal: a record of another version is refused through
   `Board::reopen` with `ReopenError::Version`, naming both versions.
4. `ARecordNamesItsVersion`, the order: that refusal comes first when the record also
   fails in another way.
5. Under `serde`, a record's serialised form with the version field removed, handed over
   by name through a hand-written `MapAccess`, deserialises and is refused naming no
   version.
6. Under `serde`, a record's serialised form with another version deserialises and is
   refused naming that version.
7. Under `serde`, the version-only value reads a record's form with its other fields
   renamed or missing and gives the version refusal naming what it read, or none; given
   this engine's version it gives no refusal.
8. `ReopeningIsExact` and `WritingChangesNothing` stand: the round trip through `write`
   and `reopen`, and through the serialised form, still holds.
9. Each `Display` text of the new variant is pinned.
10. `ReopenError` and the version-only value are `Send + Sync + 'static`, `Clone` and
    `Debug` (`tests/api_bounds.rs`).

**The Rust, test first (steps 3 to 5).** Each item of the list above was written red
and turned green, and the list revisited after each loop; it gained no item. Where each
is proved:

1. `RECORD_VERSION`'s doctest.
2. `every_record_written_names_this_version` (`record.rs`): fresh, in play and solved.
3. `a_record_of_another_version_is_refused` (`record.rs`), on a record sound in every
   other way; `a_record_of_another_version_is_refused_first` (`tests/board.rs`, under
   `serde`) through the public API.
4. `refusals_come_in_a_fixed_order` (`record.rs`) now opens with a record naming none,
   then `"record-0"`; the public-API test's record also fails its count and its givens.
5. `a_record_with_no_version_field_deserialises_and_is_refused` (`record.rs`), through a
   hand-written `MapAccess`, `Named`, which hands fields over by name.
6. `a_serialised_record_of_another_version_is_refused_naming_it` (`record.rs`), with
   the same fields naming this engine's version reaching the givens.
7. `the_version_alone_is_read_from_any_shape` (`record.rs`): a later shape a `Record`
   cannot be read from, a record naming none, and this engine's version; and
   `RecordVersion`'s doctest.
8. The existing round trips through `write` and `reopen` stand unchanged
   (`tests/board.rs`, the record section and the exactness property). No test
   round-trips the serialised form: that needs a format crate, which the core's tests do
   not take, so the JSON snapshot shows it and the deserialising tests above hold the
   by-name and positional readings.
9. `reopening_refusal_texts_are_stable` gains both texts; `Record`'s "# Across versions"
   doctest pins the first.
10. `public_types_are_send_sync_and_static` and `public_types_are_clone_and_debug`
    (`tests/api_bounds.rs`) still name `ReopenError`, green; the new
    `the_version_reading_is_send_sync_clone_and_debug` holds `RecordVersion` to the
    same bounds and to `DeserializeOwned`.

The two existing tests the ticket names changed as it says:
`a_record_holds_what_it_states_and_no_answer` lists `version` first among the stated
fields; the `Fields` sequence helper hands over five fields, the version first, and
`a_malformed_record_deserialises_and_is_refused_at_reopening` reads it twice: with
`RECORD_VERSION`, refused for its own fault, `TooManyUndone`; with `"record-0"`, refused
for the version.

**Step 5's gates**, on 2026-11-07. `just fmt-check`: `cargo fmt --all --check`, no
output. `just clippy`: "Finished `dev` profile [unoptimized + debuginfo] target(s)".
`just metrics`: "::notice::Quality score: 100.0% (724 functions analyzed)", the probe
passing. `just test`: "316 tests run: 316 passed, 0 skipped", and the doctests "124
passed; 0 failed".

**Step 6, the snapshots.** `just snapshots-accept` accepted three, and each was read
whole. `snapshot_the_record_of_the_scripted_game` gains `version: "record-1",` as the
first field of `Record { ... }`; `snapshot_the_reopening_refusals` gains two lines at
its head:

```text
a record of another format version: the record is of format version "record-0", and this engine reads "record-1"
a record that names no format version: the record names no format version, and this engine reads "record-1"
```

The JSON one, `snapshot_the_record_of_the_scripted_game_as_json`, changes in one line:
the record's serialised shape gains a first field, `version`, and nothing else in it
moves. Its head:

```json
{
  "version": "record-1",
  "givens": [
    {
      "position": {
        "row": 1,
        "column": 1
      },
      "digit": 5
    },
```

and the rest, thirty givens, eleven moves, `"undone": 1` and two checks, is as it was.

**Step 7.** `rg -n 'RANDOM_VERSION|GENERATION_VERSION' docs/reference/api.md` finds
nothing on 2026-11-07, so `api.md` is unchanged. `pawdoku 0.1.0` is not published (T13
has not run on this branch), so the changelog entry carries no 0.2.0 sentence.
`just check-docs`: "Validated 46 pages and 47 canonical topics."

**Step 8.** Decision 0022, as the brief projects it; the index row, the manifest entry
(a T00 follow-up, confirmed), and "The next decision this repository takes is 0023."

**Step 9.** `just plan-spec board`: 89 obligations, as after the clause; the Rust adds
none, as a clause's obligations come from the specification alone. `just check-docs`:
"Validated 46 pages and 47 canonical topics." `just check`, run after the ticket's last
commit, is quoted in the commit range's closing report rather than here, since these notes
are part of that commit; it was green at `4471852`: "All checks passed and the worktree is
unchanged."

### Deviations, and why

- **One branch.** The work is on `implement-typescript-surface-fullsend`, after S12 and
  T40, not on `ticket/t43-record-version` from `main`; step 1's worktree and the merge
  condition are met by order on that branch. `.pixi/` and `.tools/bin` were present, so
  no install ran.
- **The refusal in `reopen`'s comment comes first.** The version sentence opens
  `reopen`'s comment, ahead of `SetPuzzle`, so the text reads in the order of the
  refusals.
- **`PlayingFromARecord`'s preamble is unchanged.** Its "It names no form, as Recording
  names none" still holds: a version is not a form, and `Recording`'s preamble now says
  beyond the version it names no form. Its refusal invariant already sends a record that
  does not reopen to `reopen`'s refusal, which now begins with the version.

- **The variant's fields.** `ReopenError::Version { named: Option<String>, reads:
  &'static str }`. `named` is `None` when the record names none, so a binding tells the
  two texts apart without parsing; an empty `version` string reads as none, which is
  what `#[serde(default)]` gives a missing field. `reads` is always `RECORD_VERSION`.
  The `Display` text quotes both with `{:?}`, so a hostile version string prints
  escaped.
- **`require` in `record.rs` is no longer a `const fn`.** `ReopenError` now holds a
  `String`, whose destructor cannot run in a constant function.
- **`RecordVersion`** derives `Debug`, `Clone`, `PartialEq`, `Eq` and `Deserialize`, not
  `Serialize`: it only reads. Two methods: `named()`, the version or `None`, and
  `readable()`, `Ok` or the same refusal `Board::reopen` gives. Both go through one
  private function, `versioned`, so the refusal is the same code path.
- **The public-API test is under `serde`.** A record's fields are private, so a record of
  another version reaches `Board::reopen` from outside the crate only by deserialising.
  Its helper, `Arriving`, is a second hand-written `MapAccess`, smaller than `Named`.

### Handed back

- **To T41.** A record names `RECORD_VERSION`, and `Board::reopen` refuses another, or
  none, with `ReopenError::Version { named, reads }` before any other refusal, so it is
  the first refusal of `play`, `read` and `reading_after` with nothing to add.
  `ReopenError` now holds a `String`, so a function returning it cannot be `const`.
  `Board` keeps its method count; the new code is free and associated functions.
- **To T42 and T17.** Read a record through `pawdoku::board::RecordVersion` (under
  `serde`) before the rest of it: `named()` gives the version or `None`, and `readable()`
  gives `Ok` or the same refusal `Board::reopen` gives, whose `Display` text is the one to
  throw. By name it ignores every other field; in order it takes the first element as the
  version and reads every later one past as `IgnoredAny`, which needs a format that says
  what each value is. A positional format that does not is the limit decision 0022
  records. Every later version keeps the field's name, `version`, and its type, a string.
- **To T13.** The changelog entry under `[Unreleased]`, Changed, carries no 0.2.0
  sentence, since `pawdoku 0.1.0` was not published when it was written; if it is by the
  time this reaches a release, the entry gains it as step 7 says.

### Adversarial review

- **R1, round 1**, after step 2, on the range from the start to the checkpoint
  `1adf64d`, focus the refusal's order and what a record must name: approved with no
  findings. The reviewer's note that its sandbox could not run `just check` is an
  environment note, not a finding.
- **R2, round 1**, after step 8, on the range to the checkpoint `333dee6`, focus the
  refusal's order and the wire shape. One finding, medium, judged valid: `RecordVersion`'s
  derived `Deserialize` read one element of a record handed over as a sequence and left
  the rest, so a format that refuses a sequence left part read (serde's own
  `SeqDeserializer` does; the finding said a JSON array does too, which was not checked
  here, as no format crate is in the workspace) refused the whole record before
  `readable()` could give the version refusal. A red test,
  `the_version_alone_is_read_from_a_record_handed_over_in_order`, showed one of five
  fields read. Answered in `4471852`: `RecordVersion` gains a hand-written `Deserialize`
  that by name defers to a private derived reading, so field names and ignored fields
  behave as before, and in order reads the first element as the version (a missing one
  reads as none) and every later one past as `IgnoredAny`. The test reads a full record
  of this version through the counting `Fields` helper, all five handed over and readable,
  and a later shape through serde's exhaustion-checking `SeqDeserializer`, refused naming
  `"record-9"`. `RecordVersion`'s documentation and decision 0022's Consequences state the
  limit: reading past needs a format that says what each value is, or one that hands over
  no more than the version. `docs/reference/testing.md`'s `board/record.rs` row names the
  in-order case. `just check` was green at `4471852`.
- **R2, round 2**, Codex, on the range from `a3cb5db`. One finding, medium, judged
  valid: `RecordVersion`'s `Deserialize` asked for the struct `RecordVersion`, while a
  record is serialised under the name `Record`, so a format that checks a struct's name
  on reading would refuse every record, of this version or a later one, before
  `readable()` could give the version refusal. serde's value deserialisers ignore the
  name, which is why the tests missed it. A red test,
  `the_version_alone_is_read_from_a_format_that_checks_the_name`, reads through a
  deserialiser that refuses any struct name but `Record`; `Record` itself read through
  it, and `RecordVersion` was refused. Answered in `6177365`: the reading asks for
  `Record`, and its documentation says so; the test reads a record of this version as
  readable and a later shape refused naming `"record-9"`. `just wasm-test` stayed
  green. The reviewer's note that its sandbox could not run `just check` is an
  environment note, not a finding.
- **Round 3** (2026-11-07, the maintainer's OK for a third round over fixes landed
  after the two-round limit; here `6177365`), Codex `gpt-6-astra` (confirmed in Codex's log, thread
  `01a1b790-d5f6-7aa2-9028-29a273c7337b`), with `--base main`, focus the core's late
  fixes `6177365` and `64f0463`: refusal order, the version reader on every record shape,
  and invariant 3. Approved with no findings: it found the version refused before shape
  and content, the reading asked for under `Record`, the public types' bounds and
  `GenerationRefused`'s `source()` as the contracts state them. Nothing was valid,
  invalid or the maintainer's, so no commit answers it. The run was short, about 36
  seconds, and did not report attacking each record shape one by one. Its note that its
  read-only sandbox could not run `just check` is an environment note; `just check` ran
  here at `79c080b`.

### Open points settled

Answered by the maintainer on 2026-11-06:

- **The version string, and the field's name:** `"record-1"`, and `version`, first among
  the serialised record's fields, as proposed.
- **Whether the refusal names the engine's crate version too:** no, format versions only,
  as proposed.
- **A record of a later shape may not reach `Board::reopen`:** the alternative, over the
  proposal. The core reads the version alone: a small value under the `serde` feature,
  shared by T42 and T17, so the bindings do not each write that reading. Files touched,
  Non-goals, step 3, step 4 and the acceptance criteria above are amended to match, and
  so are T42's steps 1, 4, 5 and 7.

## Open points

Each is the maintainer's; the proposal is the ticket writer's, of 2026-10-31.

- **The version string, and the field's name.** Proposed: `"record-1"`, in the shape of
  `"generation-1"`, and `version` as the field's name in the serialised record, first
  among its fields. The alternative names the format in full, such as
  `"pawdoku-record-1"`, which says what it is when a record is read out of context.
- **Whether the refusal names the engine's crate version too.** Proposed: no, the
  format version only. The crate version changes with every release while the format
  may not, so naming it would suggest a record of an earlier release is refused when it
  is not. A consumer that wants the crate version in its message has it from its own
  dependency.
- **A record of a later shape may not reach `Board::reopen`.** A record written by a
  later engine whose shape renamed or dropped a field may fail to deserialise in an
  earlier one, so the consumer sees its format's error, not the version refusal. The
  field's being first and its type staying a string help a consumer read it alone.
  Proposed: nothing more in the core now; the bindings (T42, T17) read the version
  before the rest of a record and throw the version refusal's text when it differs, and
  every later version keeps the field's name and type. The alternative is a small value
  in the core, under `serde`, that deserialises the version alone, so every binding
  shares that reading.
- **Paths other tickets also edit.** T41 runs after this ticket and edits `board.rs`,
  `tests/api_bounds.rs`, `tests/snapshots.rs`, the snapshot directory,
  `docs/explanation/architecture.md`, `docs/reference/testing.md` and `CHANGELOG.md`; it
  builds on the versioned record. T13, T15 and T37 to T39 may be open at the same time
  and share `CHANGELOG.md`, `docs/decisions/README.md`, `docs/manifest.yml`,
  `docs/explanation/architecture.md` and `docs/reference/testing.md`; T37 also shares
  `docs/project/terminology.md`, and T38 and T39 share
  `crates/pawdoku/tests/api_bounds.rs`, `crates/pawdoku/tests/snapshots.rs` and
  `crates/pawdoku/tests/snapshots/`. Each shared edit is a row, an entry, a paragraph,
  a test or a snapshot; whichever pull request merges second merges `main` first and
  takes the next decision number, the deviation from CONVENTIONS.md §11 that T31 and
  T33 recorded.
