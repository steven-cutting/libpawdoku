---
id: T43
title: "A record names its format version: the clause in board.allium, then the field, the refusal and the decision"
status: open
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
it: `pawdoku::board::RECORD_VERSION`, a version field in every record written, and a
refusal with stable `Display` text that `Board::reopen` reports before any other. Both
specification gates are clean with no waiver, `just check` is green, and a decision
record says why.

## Non-goals

- Reading an earlier version: no migration, no compatibility promise, no list of
  versions an engine accepts beyond its own.
- A format, or anything about where a record is kept: still the caller's.
- Any other change to what a record holds or to what reopening checks.
- The bindings: T41 builds play from a record on this record, T42 exports
  `RECORD_VERSION` to TypeScript and T17 to Python.
- Any change to `sudoku.allium`, `solver.allium` or `generation.allium`.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `docs/specs/board.allium` | specification | `config` gains `record_version`; `contract Recording` states the version (step 2); three comments gain the version as their one exception: `external entity Record`'s, `Recording`'s preamble, and the Scope's "The record" bullet |
| `crates/pawdoku/src/board/record.rs` | core | `RECORD_VERSION`; the private version field; `ReopenError::Version`; the check; "# Across versions" and the doctests rewritten; unit tests (steps 3 to 5) |
| `crates/pawdoku/src/board.rs` | core | The re-export of `RECORD_VERSION`; `Board::reopen`'s list of refusals gains the version, first; the module documentation's paragraph on the record |
| `crates/pawdoku/tests/board.rs` | core tests | A record of another version is refused through the public API (step 4) |
| `crates/pawdoku/tests/api_bounds.rs` | core tests | Only if the version brings a new public type: its bounds |
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
   Confirm with the maintainer that `docs/manifest.yml` may be edited here, for the
   record's entry in the same commit as the record; without it, stop. Put the first
   three Open points to the maintainer, each with its proposal, and record each answer,
   dated, under "Open points settled" before writing a clause.

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

## Acceptance criteria

- `just check-specs` and `just analyse-specs` report no diagnostics and no findings for
  every module, with no waiver.
- `board.allium`'s `config` holds `record_version`; `Recording` states the version and
  its refusal; `external entity Record` is still `{}`; no other rule, invariant,
  surface or contract changed, and the module's `use` lines are unchanged.
- `pawdoku::board::RECORD_VERSION` exists with a doctest; every record written names
  it; `Board::reopen` refuses a record of another version, or of none, with
  `ReopenError::Version` before any other refusal, and never panics.
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
rg -n 'RECORD_VERSION|Version \{' crates/pawdoku/src/board/record.rs crates/pawdoku/src/board.rs
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

### The test list

### Deviations, and why

### Handed back

### Open points settled

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
