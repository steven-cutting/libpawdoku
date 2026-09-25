---
id: S03
title: "Spike: benchmarks, fuzzing and mutation testing"
status: open
depends_on: [T11]
parallel_with: []
branch: ticket/s03-bench-fuzz-mutants
estimated_size: S
---

# S03: Spike: benchmarks, fuzzing and mutation testing

## Context

The gate CONVENTIONS.md §4 fixes measures correctness (nextest, doctests, proptest
through T02's dev-dependency), coverage against a floor, lints, features and targets.
It measures nothing about speed, says nothing about inputs no test author thought of,
and cannot tell a test that asserts from one that merely runs. The tools for those are
benchmarks, fuzzing and mutation testing, and none belongs in a gate that must be
deterministic, offline and read-only on the worktree (CONVENTIONS.md §1 fact 3, §13
"`--locked` and the snapshot"). This spike decides what, if anything, is added and
where it runs: never in `check`, either in a scheduled CI job, or by hand through a
recipe.

Facts to start from, each verified at execution against the tool's documentation or
`pixi search` (as reasoned on 2026-09-23):

- **Benchmarks.** divan is attribute-style (`#[divan::bench]`), small, and prints a
  table; criterion keeps baselines on disk and compares runs, at the cost of a larger
  tree and files under `target/criterion`. Both are dev-dependencies, so adding either
  is a T02 hand-back with the decision 0007 note (§11, "No dependency is added outside
  T02"), and a `bench` recipe is a `Justfile` change, a T00 follow-up on `main`. A bench
  target under `crates/pawdoku/benches/` is its own crate root with `std`, so the
  `no_std` core is untouched.
- **Fuzzing.** cargo-fuzz drives libFuzzer on a nightly toolchain, invoked as
  `cargo +nightly fuzz run <target>`. Its `fuzz/` crate sits outside the workspace
  (`[workspace] exclude = ["fuzz"]`), so it enters neither `cargo hack`, `cargo shear`,
  coverage nor the lockfile; `taplo fmt --check` still walks `fuzz/Cargo.toml` unless
  `taplo.toml` excludes it. Nightly is a maintainer prerequisite beside the 1.98.1
  pin, never the pin (§2). First targets are the parsers: a grid-string import,
  anything that turns bytes into a `Sudoku`.
- **Mutation testing.** cargo-mutants (27 on 2026-09-23) runs on stable, writes
  `mutants.out/`, which T00's `.gitignore` already ignores (§5), and takes minutes to
  hours. It is a scheduled job and a report, never a gate: a surviving mutant is a
  finding for a person.
- What enters `check`: nothing. The gate list is frozen at T00 and the three are
  non-deterministic (fuzzing), slow (mutants) or write files (criterion baselines).

Read first: CONVENTIONS.md §4, §5, §10 (`audit.yml` is the precedent for a scheduled,
non-required job), §11, §13; `docs/reference/testing.md` and
`docs/reference/quality-gates.md` (T08); `crates/pawdoku/Cargo.toml` as T02 left it;
`.gitignore` for the `mutants.out*/` line; `taplo.toml` for its `exclude` list.

## Goal

A recommendation per tool (adopt now, adopt on a named trigger, or not at all), where
each runs, and a drafted follow-up for whatever is adopted. Nothing is installed or
changed outside this file. The starting recommendation: **divan** as a dev-dependency
with a `just bench` recipe once the solver exists (a benchmark of `SIDE` is noise);
**cargo-fuzz** with one target the day a parser lands, by hand on the maintainer's
nightly and as a short-budget scheduled job (`-max_total_time=300`) if the runner has
one; **cargo-mutants** as a weekly scheduled job that uploads `mutants.out/` as an
artefact and is never required.

| Tool | Deterministic | Writes to the tree | Toolchain | Where it runs | Adopt |
| --- | --- | --- | --- | --- | --- |
| divan | yes (timing varies) | `target/` only | stable | `just bench` by hand | when a solver exists |
| criterion | yes (timing varies) | `target/criterion` baselines | stable | `just bench` by hand | only if baselines are wanted |
| cargo-fuzz | no | `fuzz/corpus`, `fuzz/artifacts` (ignored) | nightly | by hand; scheduled with a time budget | when a parser exists |
| cargo-mutants | yes | `mutants.out/` (ignored) | stable | weekly scheduled job, artefact upload | once tests exist |

## Non-goals

- Adding a dependency, a recipe, a workflow or a `fuzz/` directory: the follow-up does
  that, with the recipe and the tool pin (a pixi dependency in `pyproject.toml` when
  conda-forge carries cargo-mutants, checked with `pixi search`, otherwise a `tools.txt`
  line) as T00 follow-ups, because the `Justfile` and `tools.txt` are frozen and the
  manifest is T00's.
- A performance target for the solver; a specification question for `effort.allium`
  and `solver.allium`, not a tooling one.
- Changing the coverage floor or the gate order.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S03-bench-fuzz-mutants.md` | ticket | the evidence, the recommendations, the drafted follow-up; `status: done` |

## Steps

1. Create the worktree on `ticket/s03-bench-fuzz-mutants` from `main` after T11 has
   merged (README.md "How to pick up a ticket").

2. Verify, read-only, against each tool's current documentation and record with
   sources: (a) divan's and criterion's versions and MSRV against 1.98.1; (b) whether
   divan runs under `cargo bench` from a `no_std` crate's `benches/` target with
   `harness = false`; (c) cargo-fuzz's nightly requirement, the layout `cargo fuzz
   init` generates, and whether it writes into the workspace manifest;
   (d) cargo-mutants' version, whether it honours `--locked` and
   `.config/nextest.toml`, its `--in-place` and `--jobs` flags, and what `mutants.toml`
   can exclude (test modules, the fake in `random.rs`); (e) whether `ubuntu-latest`
   can install a nightly with `rustup toolchain install nightly --profile minimal`
   inside a job without touching `rust-toolchain.toml`.

3. Size the workload on the tree as it is: the functions cargo-mutants would mutate
   (`cargo mutants --list` from a scratch install under the session's scratch
   directory, by `pixi exec` if conda-forge carries it or by a binstall there, never
   `.tools/` or `.pixi/`) and the tests nextest lists. A crate with ten functions
   is not worth a weekly job yet; the recommendation says when it becomes worth it.

4. Write, per adopted tool, the exact changes the follow-up needs: the dev-dependency
   for `[workspace.dependencies]` and the crate (T02 hand-back), the `benches/<name>.rs`
   skeleton, the `bench` and `mutants` recipes and the cargo-mutants pin, a pixi
   dependency or a `tools.txt` line (T00 follow-up),
   the `taplo.toml` and `.gitignore` edits (T02 and T03 hand-backs), the `[workspace]
   exclude` line, and a `mutants.yml` workflow (weekly `schedule`, `workflow_dispatch`,
   `contents: read`, `actions/upload-artifact` of `mutants.out/`, never required)
   modelled on `audit.yml`.

5. Draft the follow-up in the hand-back notes: id `T14`, one ticket or one per tool
   (say which and why), each with the trigger that makes it worth picking up ("when
   `solver.rs` exists", "when the first parser lands").

6. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The table is completed with the versions and behaviours found.
- Each tool has a verdict of adopt now, adopt on a named trigger, or not at all, and
  "what enters `check`" is stated as nothing, with the reason.
- The follow-up draft lists every frozen-file change as a T00 follow-up and every
  dependency as a T02 hand-back.
- `git status --porcelain` on the ticket branch lists only this file.

## Verification

```sh
grep -n 'mutants.out' .gitignore
grep -n 'exclude' taplo.toml
rustup toolchain list
cargo nextest list --workspace --locked 2>/dev/null | tail -1
git status --porcelain
```

Expected: the `mutants.out*/` line; taplo's exclude list (`target/**`, `.tools/**`,
`.pixi/**`, `ai_tmp/**`; no `fuzz/**` today); the
installed toolchains (a nightly is not required for this spike); the test count; one
line naming this file.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether a nightly on the maintainer's machine is acceptable as a prerequisite for
  fuzzing alone, or whether fuzzing waits for a stable path (verify whether cargo-fuzz
  now has one).
- Whether mutation results feed the coverage-floor question (a surviving mutant in a
  covered line is what the floor cannot see); a finding for
  `docs/explanation/quality-philosophy.md`, T08's page, if the maintainer agrees.
