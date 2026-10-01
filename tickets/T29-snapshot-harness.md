---
id: T29
title: "The snapshot harness: insta, its recipes and the rule for what a snapshot is for"
status: open
depends_on: [T24]
parallel_with: []
branch: ticket/t29-snapshot-harness
estimated_size: M
---

# T29: The snapshot harness: insta, its recipes and the rule for what a snapshot is for

## Context

T25 to T28 build the first engine modules. The maintainer decided on 2026-10-01 that each
of them also takes snapshots, with [insta](https://insta.rs/), at the interface of the
layer or object it builds. This ticket runs before T25 and makes the harness ready: the
dependency, the tool, the recipes, the gate and the rule. T25 to T28 then only take
snapshots.

**The rule.** A snapshot has two purposes and no other:

1. **To show a reviewer what a value looks like.** It is print-based debugging that is
   kept: a person reading a pull request sees the grid, the moves or the result without
   running anything.
2. **Cheap change detection.** A change to what an interface hands out shows as a diff
   in a committed file, as Chromatic shows a changed component in a front end.

**A snapshot never verifies correctness.** Nobody checked the first snapshot against the
specification; it records what the code printed on the day. So:

- No clause of a specification is proved by a snapshot. Each clause keeps its own test
  that asserts, and a ticket's table from clause to test never names a snapshot test.
- Snapshot tests are written after the behaviour's own tests are green, never as the
  failing test of a red-green loop. Their names begin `snapshot_`, so that a reader and
  a filter can tell them apart.
- A snapshot is deterministic: a fixed input and a fixed script. Never a property-test
  input.
- A snapshot is a text picture rendered **in test code** from what the public surface
  hands out. No `Display`, method or field is added to the library to feed one. A small
  value, such as an error or a record, may be shown through its `Debug` or its
  serialised form.
- Snapshots are files (`.snap`), never inline in Rust source. A file shows as its own
  diff in a pull request, and accepting it rewrites no source.
- A changed snapshot is a question and not a failure of the code: was the change meant?
  The one who changed it reads the diff, accepts it with the recipe, and gives the
  reason in the pull request or the hand-back notes.

How the maintainer answered the design questions:

- **Review and acceptance go through `cargo-insta`**, pinned in the pixi environment. A
  mismatch fails the gate, the gate writes no file, and a `.snap` file no test refers
  to is rejected.
- **The record of T28 is snapshotted as JSON**, through insta's `json` feature. This
  ticket turns the feature on so that T28 need not touch a manifest.

Facts that shape the work:

- **insta is the fourth crate.** `Cargo.toml` says "The only three crates the core may
  use (decision 0007)". Decision 0007 allows an addition "through one reviewer", with
  "the reason recorded in the manifest beside the range". insta is a development
  dependency: it never ships, and `deny.toml` already leaves development dependencies
  out of its graph.
- **The gate runner aborts on a changed worktree.** `just check` snapshots the worktree
  between recipes (that word is the runner's own and older than this ticket). By
  default insta writes a `.snap.new` file beside a snapshot that does not match. Under
  `just test` or `just coverage` that file would abort the run with a misleading
  message. So the recipes that run tests set `INSTA_UPDATE=no`, and the pending-file
  patterns are ignored as well.
- **The `Justfile`, the hook configuration, `.gitignore`, `.editorconfig` and the CI
  workflow are frozen** (CONVENTIONS.md §11). T22 and T24 reopened the `Justfile`; this
  is a follow-up of the same class, and each such row below is marked.
- **The hooks read every text file.** typos and editorconfig-checker will read `.snap`
  files, and `.editorconfig` asks for two-space indentation and no trailing whitespace.
- **The crate is `no_std`; its tests are not.** The crate root has
  `#[cfg(test)] extern crate std;`, and integration tests are ordinary std crates, so
  insta is usable in both places. `just features` and `just wasm-check` compile no
  development dependency.

Claims from insta's documentation as read on 2026-10-01, and for the `cargo insta test`
flags from memory of the tool. **None was run.** Step 2 checks each before anything is
built on it:

- insta's newest release is 1.48.0, and `cargo-insta` 1.48.0 is on conda-forge for
  linux-64 and osx-arm64.
- `INSTA_UPDATE=no` writes no file on a mismatch and fails the test. `always` overwrites
  the `.snap` file. The default, `auto`, is `no` in CI and `new` (a `.snap.new` file)
  elsewhere.
- `cargo insta test` accepts `--check`, `--unreferenced reject` and
  `--test-runner nextest`; `cargo insta review` and `cargo insta accept` handle pending
  snapshots.
- A test in `src/foo.rs` stores its files in `src/snapshots/`, and one in `tests/foo.rs`
  in `tests/snapshots/`.
- `assert_snapshot!` and `assert_debug_snapshot!` need no feature;
  `assert_json_snapshot!` needs `json`. The default features are `colors`, `json` and
  `yaml`.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §4, §5 and §11;
`docs/decisions/0007-dependency-policy.md`; `docs/how-to/maintain-dependencies.md`;
`docs/reference/testing.md` and `docs/reference/quality-gates.md`; the `Justfile`
(`test`, `coverage`, `metrics`); `pyproject.toml`; `.pre-commit-config.yaml` and
`.editorconfig`; `crates/pawdoku/tests/random.rs`; `tickets/T22-metrics-gate.md` as the
model for a ticket that adds a pinned tool and a gate.

## Goal

A test in this crate can take a snapshot, and the repository knows what to do with one.
`just test`, `just coverage` and `just check` fail when a snapshot does not match and
write nothing. `just snapshots-check` is a gate that also rejects an orphaned `.snap`
file. `just snapshots-review` and `just snapshots-accept` are how a changed snapshot is
read and accepted. `docs/reference/testing.md` states the rule above, and one snapshot
at the randomness boundary proves the harness end to end.

Names later tickets are written against, fixed here: the recipes `snapshots-check`,
`snapshots-review` and `snapshots-accept`, and the test-name prefix `snapshot_`.

## Non-goals

- No snapshot of any engine module. `sudoku`, `solver` and `board` do not exist yet;
  T25 to T28 take theirs.
- No test replaced by a snapshot. `tests/random.rs` keeps every assertion it has, the
  golden draws included.
- No inline snapshot, and no insta feature beyond `json`: no `yaml`, `redactions`,
  `filters`, `glob` or `colors`.
- No render helper for grids. The first ticket with a grid to show writes it.
- No dependency other than insta and what it brings, and no change to what ships.
- No change to the coverage floor or to how coverage is counted.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `Cargo.toml` | T02 follow-up | `insta` in `[workspace.dependencies]`, with its reason; the comment above the table corrected |
| `crates/pawdoku/Cargo.toml` | T02 follow-up | `insta` under `[dev-dependencies]` |
| `Cargo.lock` | T02 follow-up | insta and what it brings |
| `pyproject.toml` | T00 follow-up | `cargo-insta`, pinned exactly; `snapshots-check` in the recipe list |
| `pixi.lock` | T00 follow-up | Follows the manifest |
| `Justfile` | T00 follow-up | `INSTA_UPDATE=no` on `test` and `coverage`; the three new recipes |
| `.gitignore` | T00 follow-up | `*.snap.new` and `*.pending-snap` |
| `.editorconfig` | T03 follow-up | A `[*.snap]` section, only if step 5 shows it is needed |
| `.pre-commit-config.yaml`, `_typos.toml` | T03 follow-up | An exclusion for `.snap` files, only if step 5 shows one is needed |
| `.github/workflows/ci.yml` | T04 follow-up | `just snapshots-check` in the `rust` job |
| `crates/pawdoku/tests/random.rs` | code | One snapshot test, added beside the tests that assert |
| `crates/pawdoku/tests/snapshots/` | code | New: the `.snap` file that test writes |
| `docs/reference/testing.md` | reference | A "Snapshots" section with the rule; the framework paragraph; the suite table |
| `docs/reference/quality-gates.md` | reference | The new gate's row, and `check-clean`'s number |
| `docs/reference/commands.md` | reference | The three recipes; the `test` and `coverage` rows |
| `docs/reference/configuration.md` | reference | `INSTA_UPDATE`, and the `pyproject.toml` row if it lists the tools |
| `docs/how-to/test-and-debug.md` | how-to | How to read and accept a changed snapshot |
| `docs/how-to/maintain-dependencies.md` | how-to | Only if it counts or lists the crates |
| `docs/project/repository-map.md` | project | The `tests/` line: the `snapshots/` directory |
| `AGENTS.md` | agent contract | One sentence beside "Tests live in three places" |
| `CHANGELOG.md` | maintainer docs | Under Unreleased, Added |
| `tickets/README.md` | ticket index | T29's row set `done` |
| `tickets/T29-snapshot-harness.md` | ticket | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t29-snapshot-harness` from `main` after T24 has
   merged. Run `just initialize` if `.pixi/` is absent; it uses the network and this
   ticket authorises it. Run `just check` and confirm it is green before any edit.

2. **Check the claims**, in two parts, and write each result in the hand-back notes
   with the versions found.

   - Before any edit: the versions and the platforms, read from crates.io and
     conda-forge. If `cargo-insta` is not on conda-forge for both platforms, it goes
     in `tools.txt` as a release binary instead.
   - After step 3 has installed the tool and before step 4 builds on it: the flags,
     from the tool's own `--help`, and what `INSTA_UPDATE=no` does, from a trial run
     on a snapshot made to mismatch. If a flag has another name, use the one the
     tool prints. If no mode fails a mismatch without writing a file, stop and
     report: the gate depends on it.

   The two registry lookups here, and `just lock` and `just sync` in step 3, use the
   network. With `just initialize` in step 1, those are the network uses this ticket
   authorises, and it authorises no other.

3. **The dependency and the tool.**

   - `Cargo.toml`: `insta` in `[workspace.dependencies]` as a full `x.y.z` caret range,
     with `default-features = false` and `features = ["json"]`. Beside it, the reason:
     snapshots for review and change detection, tests only, and `json` for the record's
     snapshot in T28. Correct the comment that says only three crates: there are four,
     of which `serde` and `thiserror` may ship and `proptest` and `insta` serve the
     tests.
   - `crates/pawdoku/Cargo.toml`: `insta = { workspace = true }` under
     `[dev-dependencies]`.
   - `pyproject.toml`: `cargo-insta`, pinned exactly to the release that matches the
     crate.
   - Run `just lock`, which relocks `Cargo.lock` and `pixi.lock`, then `just sync`, as
     `docs/how-to/maintain-dependencies.md` describes. Read both lockfile diffs. Every
     crate insta brings is recorded in the hand-back notes with its licence.

4. **The recipes.** In the `Justfile`:

   - `test` and `coverage` run their nextest line with `INSTA_UPDATE=no`. A comment says
     why: a mismatch must fail, and a pending file would trip the worktree check.
   - `snapshots-check` runs every snapshot test through `cargo insta test`, with
     nextest as the runner and every feature, in the mode that writes nothing, and
     rejects a `.snap` file no test refers to. It keeps `--locked` if the tool passes
     it through; if it cannot, say so in the hand-back notes.
   - `snapshots-review` opens `cargo insta review`. It is for a person at a terminal.
   - `snapshots-accept` reruns the tests and accepts every pending snapshot without
     asking, so that an agent with no terminal can use it. It is the only recipe here
     that writes, and its comment says the diff is read before the commit.
   - Add `snapshots-check` to the recipe list in `pyproject.toml`, last, after
     `analyse-specs`. It becomes gate 19 and `check-clean` gate 20, and no other gate
     is renumbered.
   - `.github/workflows/ci.yml`: `just snapshots-check` in the `rust` job, after
     `just test`.

5. **Hygiene.**

   - `.gitignore`: `*.snap.new` and `*.pending-snap`, with a comment in the style of
     its neighbours: a run outside the recipes may leave them, and one Git can see
     aborts the gate.
   - Take the proof snapshot of step 6, then run `just lint`. If editorconfig-checker
     objects to insta's own file layout, give `.snap` files a section in
     `.editorconfig`, as Markdown has, that unsets only the property it trips on. If
     typos objects to a word insta writes, exclude `.snap` files in `_typos.toml` with
     the reason. Change neither file if the gate is already green, and never exclude
     the files from ripsecrets.
   - A picture has no trailing whitespace and ends in a newline. That is a rule for
     every later render helper, and the testing page says so.

6. **The proof.** In `crates/pawdoku/tests/random.rs`, one test named
   `snapshot_the_randomness_boundary`. It renders as text the first eight draws of seed
   zero and the `Display` text of each `RandomError` variant, and takes one
   `assert_snapshot!`. Then show the harness works, and quote each result:

   - `just test` and `just snapshots-check` are green.
   - Change one character of the committed `.snap` file. `just test`,
     `just coverage` and `just snapshots-check` each fail and name the snapshot.
     After each one, `git status --short --ignored` shows the edit and, under the
     snapshot directories, no `.snap.new` and no `.pending-snap` file. Plain
     `git status` cannot show this, because step 5 ignores both patterns, and the
     gate's own worktree check leaves ignored paths out for the same reason.
   - `just snapshots-accept` restores the file, and `git diff` is empty.
   - Add a `.snap` file no test refers to. `just snapshots-check` fails and names it;
     `just test` does not. Remove it.

7. **The pages.**

   - `docs/reference/testing.md`: a "Snapshots" section that states the rule under
     Context in full, since this page owns the topic; where the files live; that
     coverage counts a snapshot test like any other, which is why no clause may rest on
     one; and the row in the suite table. The Framework paragraph names insta as a
     development dependency.
   - `docs/reference/quality-gates.md`, `docs/reference/commands.md`,
     `docs/reference/configuration.md` and `docs/how-to/test-and-debug.md` follow, each
     within what `docs/manifest.yml` says it owns.
   - `AGENTS.md`: after the sentence on the three places tests live, one sentence:
     snapshots show a reviewer a value and detect change; they never prove a clause.
     Run `just check-agents`.
   - `docs/project/repository-map.md` and `CHANGELOG.md`.

8. Run `just fmt-check`, `just toml-check`, `just clippy`, `just test`,
   `just snapshots-check`, `just coverage`, `just deny`, `just deps-unused`,
   `just lock-check`, `just check-agents`, `just check-docs` and `just lint`, then
   `just check`. Quote the closing lines of each and how long `just snapshots-check`
   takes. Fill in the hand-back notes, set `status: done` here and in
   `tickets/README.md`, commit on the ticket branch, and stop before pushing.

## Acceptance criteria

- `insta` is a development dependency of `pawdoku` and of nothing that ships, with
  default features off and `json` on. `Cargo.toml` states its reason.
- `cargo-insta` is pinned exactly, and `just lock-check` passes.
- With a snapshot that does not match, `just test`, `just coverage`,
  `just snapshots-check` and `just check` each fail, and none leaves a file Git can see.
- With a `.snap` file no test refers to, `just snapshots-check` fails.
- `just check` runs twenty gates, `snapshots-check` the nineteenth, and is green. CI's
  `rust` job runs it.
- `tests/random.rs` has one snapshot test, named with the `snapshot_` prefix, and every
  test that was there still asserts what it did.
- `docs/reference/testing.md` states both purposes, that a snapshot never proves a
  clause, and how one is accepted. `AGENTS.md` carries the one sentence.
- No hook is excluded from `.snap` files without a reason beside the exclusion.
- No `#[allow]`, no `#[expect]` and no `qual:allow` line is added.

## Verification

```sh
just lock-check
just fmt-check
just toml-check
just clippy
just test
just snapshots-check
just coverage
just deny
just deps-unused
just check-agents
just check-docs
just lint
just check
```

Expected: each recipe green with no warning; the coverage total at or above the floor;
`Validated AGENTS.md, 2 adapters, and 14 skills.`; twenty gates; `All checks passed and
the worktree is unchanged.`

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **A decision record.** The rule is stated on the testing page, which owns the topic.
  If the maintainer wants "snapshots are for review and change detection, never for
  correctness" recorded as a decision as well, it is the next free number and one more
  row in the index; this ticket writes none.
- **Where the gate sits.** `snapshots-check` is appended so that no gate is renumbered.
  It belongs beside `coverage` by subject; moving it there renumbers gates 12 to 18 on
  every page that cites one.
- **A third run of the tests.** `coverage` runs every test, and `snapshots-check` runs
  them again to find orphans. While the suite takes seconds that is cheap. If it stops
  being cheap, the orphan check is what the gate needs and the rerun is not; say so in
  the hand-back notes with the time measured.
- **Inline snapshots.** Refused here, for the reasons under Context. If a one-line
  value ever reads better inline, that is a change to the rule on the testing page.
- **Coverage.** A snapshot test runs code, so llvm-cov counts it. The rule keeps a
  clause from resting on one, and nothing mechanical does. If the maintainer wants it
  mechanical, the coverage recipe can leave the `snapshot_` tests out by a nextest
  filter; that is a change to how the floor is counted, and this ticket does not make
  it.
