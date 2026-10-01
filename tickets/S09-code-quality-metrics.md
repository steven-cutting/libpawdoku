---
id: S09
title: "Spike: code-quality metrics as a gate, module boundaries, complexity, coupling and cohesion"
status: done
depends_on: [T11]
parallel_with: []
branch: ticket/s09-code-quality-metrics
estimated_size: S
---

# S09: Spike: code-quality metrics as a gate, module boundaries, complexity, coupling and cohesion

## Context

The gate CONVENTIONS.md §4 fixes measures correctness, coverage against a floor, lints,
features and targets. It says nothing about the shape of the code: how deeply a
function nests, how many things a type does, which module leans on which. The one
structural rule the handbook states, the import direction in
`docs/explanation/layering.md`, is enforced by review alone; that page's "Enforcement"
section says so in its first sentence. The maintainer asked on 2026-09-29 which tools
should enforce the metrics the project cares about — cohesion, coupling, connascence,
LCOM, afferent and efferent coupling, cyclomatic complexity and WMC — naming cargo-pup,
cargo-coupling, rustqual, ast-metrics, rust-code-analysis and clippy as candidates, and
asked for "something like Tach": a file that declares which module may depend on which,
checked mechanically, failing the build on a violation.

The maintainer settled the shape on the same day: a spike ticket only, in S03's shape;
whatever is adopted is a **required gate in `check`**, which reopens the list S03
recorded as frozen and so needs a decision record; all four metric families are wanted,
module boundaries first; and thresholds are **set now**, before the solver lands, so
that they shape it rather than being fitted to it.

Every gate recipe is deterministic, offline, read-only outside ignored paths, pinned
through pixi (`pyproject.toml`) or `tools.txt`, and runs on the pinned stable toolchain
with no rustc driver and no nightly (CONVENTIONS.md §1 fact 3, §13, decision 0009).
`tools.txt` is installed with `cargo binstall --disable-strategies compile`, so a crate
there needs release binaries. `.tools/bin` also holds the Allium checker, installed by a
pinned, checksummed download (`bg-install-allium`, decision 0005), which is the
precedent for a binary that is neither on conda-forge nor on crates.io.

Facts to start from, each verified at execution (as reasoned on 2026-09-29 from the
tools' documentation, source, crates.io and the GitHub API; nothing had yet been run):

- **Two metric families have no honest Rust tooling.** Connascence is a review concept;
  no tool names it, and cargo-coupling's own documentation lists it as a blind spot. WMC
  is computed only by rust-code-analysis, whose last release predates edition 2024. The
  proxy for both is what the other tools measure: per-type method count and cyclomatic
  complexity for WMC; the module-boundary rules for the coupling connascence describes.
- **LCOM is a class metric.** Rust has no classes, so a tool maps it to a `struct` plus
  the methods its `impl` blocks attach. rustqual (LCOM4 per struct, `[srp]`) and
  ast-metrics (LCOM4 with struct, enum, union and trait as the class) both do this.
- **Tach is Python-only.** Its `[[modules]]` entries name Python dotted paths; Rust files
  appear in its documentation only as cache inputs. It is on conda-forge (0.35.1), so it
  would install, and would then read nothing here.
- **The clippy baseline costs nothing.** Clippy is gate 6. Its size and complexity lints
  take thresholds in `clippy.toml`: `cognitive_complexity`
  (`cognitive-complexity-threshold`, default 25), `excessive_nesting`
  (`excessive-nesting-threshold`, default 0, which is off), `too_many_lines`
  (`too-many-lines-threshold`, 100), `too_many_arguments`
  (`too-many-arguments-threshold`, 7), `type_complexity`, `large_enum_variant`,
  `struct_excessive_bools`, `fn_params_excessive_bools`, `large_types_passed_by_value`.
  Which lint group each sits in decides whether the workspace table already enables it;
  two recollections disagreed (restriction against nursery for `cognitive_complexity`),
  so the group is verified against the pinned toolchain below. Nursery lints stay out,
  not by an invariant but because they move between releases and would break a pinned
  gate on every toolchain bump.
- **No viable Rust tool has release binaries or a conda-forge package**, so `tools.txt`
  as written cannot install any of them. That is a T00 follow-up decision the spike
  must make explicit, not a detail.

| Tool | Metrics | Fails on threshold | Needs build or nightly | Install path | Verdict to start from |
| --- | --- | --- | --- | --- | --- |
| rustqual 1.8.3 (MIT, released 2026-09-24, one author, first release 2026-04) | cognitive, cyclomatic, nesting, function length; LCOM4 per struct, fan-out; instability, fan-in, fan-out, cycles, Stable Dependencies Principle per module; declared layers, forbidden edges and path patterns in `rustqual.toml` | yes, exit 1; text, json, sarif, github | syn only, stable | `cargo install --locked`; no binaries; not on conda-forge | candidate for the gate: the one tool covering all four families and the Tach-like config |
| cargo-coupling 0.4.0 (MIT, 2026-09-08) | module coupling strength and distance, afferent and efferent checks, cycles, god modules; volatility from git history | yes, `--check` with `--min-grade`, `--max-circular`, `--fail-on` | syn plus `cargo metadata`, stable | `cargo install --locked`; no binaries | alternative for coupling numbers; `--no-git` for determinism; no LCOM, no complexity |
| ast-metrics 0.43.1 (MIT, Go, 2026-09-15) | cyclomatic, maintainability index, afferent and efferent coupling, instability, LCOM4 | yes, `.ast-metrics.yaml` and `--fail-on` | tree-sitter, no build | GitHub release binaries for macOS and Linux; not a cargo tool | alternative; coarser Rust semantics; needs the checksummed-download route |
| clippy at 1.98.1 | the size and complexity lints above | yes, gate 6 | already in the gate | none | adopt now: the baseline every external tool must justify itself beyond |
| cargo-modules 0.27.0 (MPL-2.0) | module tree, `dependencies --acyclic`, `orphans --deny` | yes, those two | rust-analyzer crates, slow source build | `cargo install --locked`; no binaries | not needed if rustqual covers cycles |
| archaven 1.1.0 (MIT or Apache-2.0), cargo-archtest-cli 0.2.6 (AGPL-3.0), archunit 0.0.1 (MIT) | declared module access rules as a `#[test]` or an `architecture.json` | yes, through the test | syn, stable | a dev-dependency (T02 hand-back, decision 0007) | fallback for the boundary rules alone; each is young and tiny |
| cargo-pup 0.1.8 (Apache-2.0, 2026-06-09) | ArchUnit-style rules, function length, imports | yes | rustc driver pinned to `nightly-2026-01-22`; writes `.pup/` | none | not at all: nightly, a driver, and writes to the tree |
| rust-code-analysis 0.0.25 (MPL-2.0, 2023-01-13) | cyclomatic, cognitive, Halstead, MI, WMC | no, report only | tree-sitter grammar from 2022 | dead release | not at all |
| Tach 0.35.1 | Python module boundaries | yes | none | pixi | not at all: Python-only |
| cargo-crap 0.6.1 (MIT, 2026-09-29) | CRAP, cyclomatic weighted by coverage | yes, `--threshold` | an lcov file first | binstall works | on a trigger: `coverage` already writes `lcov.info` |

Read first: CONVENTIONS.md §1, §4, §11, §13; `docs/reference/quality-gates.md`;
`docs/explanation/layering.md`; `docs/decisions/0009-rust-quality-gate.md` ("What would
reopen this"); `tickets/S03-bench-fuzz-mutants.md` as the shape and the precedent for
what enters `check`; `Cargo.toml`'s lint table and `clippy.toml`.

## Goal

A verdict per tool (adopt now, adopt on a named trigger, or not at all), the thresholds
that become guardrails, what enters `check`, and a drafted follow-up for whatever is
adopted. Nothing is installed or changed outside this file and the ticket index.

The starting recommendation: **the clippy baseline now**, as `clippy.toml` thresholds
and one added workspace lint; **rustqual now, as gate `metrics`**, with a committed
`rustqual.toml` whose architecture rules encode the table in
`docs/explanation/layering.md` and whose complexity, cohesion and coupling thresholds
are the guardrails, on the condition that the spike proves on a scratch copy that a
planted violation fails, that output is byte-stable across two runs, and that the
install route is settled; **cargo-coupling and cargo-crap on a trigger**; **cargo-pup,
rust-code-analysis, Tach and cargo-modules not at all**. If rustqual is judged too young
for a required gate, the fallback is archaven as a dev-dependency architecture test for
the boundary rules alone, with rustqual demoted to an advisory report.

Thresholds only. No baseline file, no `--fail-on-regression`, no `--compare`: `check-clean`
fails anything that rewrites a baseline, and a gate cannot depend on how deep the clone
is. cargo-coupling's volatility axis reads git history and is off (`--no-git`) for the
same reason.

## Non-goals

- Adding a recipe, a pin, a workflow, a dependency, a configuration file or a decision
  record: the follow-up does that, with the `Justfile`, `tools.txt` and `pyproject.toml`
  changes as T00 follow-ups, any dependency as a T02 hand-back with the decision 0007
  note, and the gate list reopened by a decision record.
- Changing the coverage floor or the gate order.
- Refactoring anything a threshold flags today. The spike records what fires; the
  follow-up decides whether the threshold or the code moves.
- A metric for the specifications. `analyse-specs` is that gate.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S09-code-quality-metrics.md` | ticket | the evidence, the verdicts, the thresholds, the drafted follow-up; `status: done` |
| `tickets/README.md` | ticket index | S09's row in the spike table |

## Steps

1. Create the worktree on `ticket/s09-code-quality-metrics` from `main` after T11 has
   merged (README.md "How to pick up a ticket").

2. Verify, read-only, against each tool's documentation, crates.io's API, `pixi search`
   and the GitHub API, and record with sources: version, release date, MSRV against
   1.98.1, conda-forge and release-binary availability, whether it needs a rustc driver
   or nightly, whether it exits non-zero on a threshold, and whether it reads the
   intra-crate `mod` graph or only crate-to-crate dependencies. Verify the lint group of
   each clippy lint in the Context against the pinned toolchain
   (`cargo clippy -- -W help` lists every lint under its group).

3. Make a scratch copy with `git archive HEAD` in the session's scratch directory and
   install rustqual, and cargo-coupling if time allows, there only (`cargo install
   --locked --root <scratch>/tools`), never into `.tools/` or `.pixi/`. Run each on the
   copy twice and `diff` the output. Record run time and every file the tool wrote.

4. Write the `rustqual.toml` that encodes `docs/explanation/layering.md`. The table is
   a directed acyclic graph with sibling bans, not a linear stack: the four models of a
   player never import each other, `generation` imports `reach` but `reach` never
   imports `solver`, and `random` may be imported by anything. Check whether ranked
   layers can express that or whether per-module rules are needed. Because the crate
   has two modules today, probe the rules on a stub crate with one file per module of
   the table. Plant two violations, one at a time: a `use crate::effort::...` line in
   `reach`, and an inline fully qualified `crate::effort::Price` with no `use` line,
   because a tool that reads only `use` statements misses the second. Confirm exit 1
   and the finding text for each. Confirm the globs treat `src/technique.rs` and
   `src/technique/` as one module, the two forms `mod_module_files` leaves.

5. In the scratch copy, set the proposed clippy thresholds in `clippy.toml` and add
   `cognitive_complexity = "warn"` to the workspace table; run `just clippy`'s command
   and record what fires on `random.rs` today. Lower every threshold to 1 once to prove
   each lint is live.

6. Give the verdicts: per tool, what enters `check`, the thresholds, and the install
   route.

7. Draft the follow-up, T22, in the hand-back notes: the `metrics` recipe and its place
   in the gate order, the pin and its install route, the `pyproject.toml` recipes entry,
   the `rustqual.toml` and `clippy.toml` contents, the `quality-gates.md` row, the
   `layering.md` "Enforcement" rewrite, the decision record that reopens the gate list,
   and the CI job it joins.

8. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Every row of the table has its version, install path and behaviour verified with a
  source and a date, and the two clippy lint groups are settled against the toolchain.
- The two planted violations each fail the scratch run, with the finding quoted, or the
  tool's failure to catch one is recorded as a finding of its own.
- Every tool has a verdict of adopt now, adopt on a named trigger, or not at all, and
  "what enters `check`" is stated with its thresholds.
- The follow-up draft lists every frozen-file change as a T00 follow-up, every
  dependency as a T02 hand-back, and names the decision record.
- `git status --porcelain` on the ticket branch lists only this file and
  `tickets/README.md`.

## Verification

```sh
cargo clippy -- -W help 2>/dev/null | grep -E '^ +clippy::[a-z]+ ' | awk '/clippy::cognitive-complexity(,| |$)/ {print "cognitive-complexity", $1} /clippy::excessive-nesting(,| |$)/ {print "excessive-nesting", $1} /clippy::too-many-lines(,| |$)/ {print "too-many-lines", $1}'
grep -n 'disable-strategies' Justfile
grep -n 'Enforcement' -A3 docs/explanation/layering.md | head -5
git status --porcelain
```

Expected: `cognitive-complexity clippy::restriction`, `excessive-nesting
clippy::complexity` (and `clippy::all`, which contains the complexity group),
`too-many-lines clippy::pedantic`; the `install-tools` recipe's `--disable-strategies compile`; the
sentence "There is no import-boundary checker here"; two lines naming this file and
`tickets/README.md`.

## Hand-back notes

### What was verified, and how

Run on 2026-09-29 (US Pacific) in the Supacode worktree for this ticket, on branch
`code-quality-checks` (see Deviations), from `main` at `661e5be` after the S06 to S08
spikes had merged. Commands stamped 2026-09-30 UTC belong to the same run. No tracked
file changed except this one and `tickets/README.md`. `just initialize` installed the
ignored `.pixi/` and `.tools/` in this worktree so that the gate could run; being a
secondary worktree, it skipped `install-hooks`. Outside the worktree, tools went only
into the session's scratch directory. Every experiment ran in scratch copies: one made
with `git archive HEAD`, and one stub crate written for step 4.

The maintainer authorised, by approving the plan for this spike: read-only
documentation and crates.io API reads, `pixi search`, `gh api` reads of release
metadata, and `cargo install --locked` of rustqual and cargo-coupling into the scratch
directory only.

The worktree before any step:

```text
$ git rev-parse --short HEAD
661e5be
$ git status --porcelain
$ rustup show active-toolchain
1.98.1-aarch64-apple-darwin (overridden by '.../rust-toolchain.toml')
```

**Step 1.** The worktree existed before the ticket ran (Deviations).

**Step 2. The table, verified.** crates.io's API on 2026-09-29 (`max_stable_version`,
`updated_at`; no crate states a `rust_version`):

```text
rustqual 1.8.3 2026-09-24
cargo-coupling 0.4.0 2026-09-08
cargo-modules 0.27.0 2026-08-03
cargo_pup 0.1.8 2026-06-09
cargo-crap 0.6.1 2026-09-29
rust-code-analysis-cli 0.0.25 2023-01-13
archaven 1.1.0 2026-06-04
cargo-archtest-cli 0.2.6 2026-09-22
archunit 0.0.1 2026-09-20
```

`pixi search` on conda-forge found none of rustqual, cargo-coupling, ast-metrics,
cargo-modules, cargo-pup or cargo-crap, and found `tach-0.35.1`. The GitHub API's
latest release: `SaschaOnTour/rustqual` v1.8.3 with zero assets;
`nwiizo/cargo-coupling` v0.4.0 with zero assets; `regexident/cargo-modules` has no
release at all; `ast-metrics/ast-metrics` v0.43.1 with nine assets, among them
`ast-metrics_Darwin_arm64` and `ast-metrics_Linux_x86_64`. So the Context's last fact
holds: no Rust candidate can go into `tools.txt` as the recipe stands.

The clippy lint groups at 1.98.1, from `cargo clippy -- -W help`:

```text
cognitive-complexity -> clippy::restriction
excessive-nesting -> clippy::complexity
too-many-lines -> clippy::pedantic
too-many-arguments -> clippy::complexity
type-complexity -> clippy::complexity
large-enum-variant -> clippy::perf
struct-excessive-bools -> clippy::pedantic
fn-params-excessive-bools -> clippy::pedantic
large-types-passed-by-value -> clippy::pedantic
large-stack-frames -> clippy::nursery
module-name-repetitions -> clippy::restriction
```

The recollection that `cognitive_complexity` is nursery was wrong; it is restriction,
so the pedantic group does not enable it and the workspace table must name it. Every
other lint in the list is already live through `pedantic` or the default groups;
`excessive_nesting` is live but silent until its threshold is set. `large_stack_frames`
is nursery and stays out.

rustqual's configuration was read from its book (`book/reference-configuration.md`,
`book/architecture-rules.md`, in the crate's registry source at 1.8.3). Its dimensions
are IOSP (the author's "integration operation, segregation principle"), complexity,
DRY, SRP, coupling, test quality and architecture, each with its own findings, and a
weighted "quality score" over them. The architecture dimension is **off unless
`[architecture] enabled = true`** is written; `--init` does not write it. Every other
dimension has an `enabled` key except IOSP, which has none: the config parser lists the
accepted top-level tables, and `iosp` is not among them.

**Step 3. Scratch installs and runs.** Both built from source with the pinned toolchain
(the first attempt, from a directory without `rust-toolchain.toml`, failed because the
machine has no default toolchain, which is itself a fact for the install route):

```text
$ cargo install --locked --root <scratch>/tools rustqual
   (64 crates compiled)
    Finished `release` profile [optimized] target(s) in 37.65s
   Installed package `rustqual v1.8.3` (executables `cargo-qual`, `rustqual`)
$ cargo install --locked --root <scratch>/tools cargo-coupling
   (101 crates compiled)
    Finished `release` profile [optimized] target(s) in 15.60s
   Installed package `cargo-coupling v0.4.0` (executable `cargo-coupling`)
```

rustqual on the tree as it is, with no configuration, twice; the two outputs were
identical (`diff -q` printed nothing) and the run took under a second:

```text
$ rustqual crates/pawdoku
═══ Summary ═══
  Functions: 34    Quality Score: 76.3%    8 findings
  IOSP:        100.0%  (12I, 3O, 19T)
  Complexity:   88.2%  (4 magic numbers)
  DRY:          97.1%  (1 dead types)
  SRP:         100.0%
  Coupling:    100.0%
  Test Quality: 91.2%  (3 no SUT)
  Architecture:100.0%
═══ 8 Findings ═══
  src/lib.rs:23  DEAD_TYPE  testonly const SIDE
  src/lib.rs:30  TQ_NO_SUT  a_grid_is_nine_by_nine  in a_grid_is_nine_by_nine
  src/random.rs:167  MAGIC_NUMBER  30
  src/random.rs:168  MAGIC_NUMBER  27
  src/random.rs:169  MAGIC_NUMBER  31
  src/random.rs:175  MAGIC_NUMBER  11
  src/random.rs:282  TQ_NO_SUT  exhausted_text_is_stable  in exhausted_text_is_stable
  src/random.rs:290  TQ_NO_SUT  out_of_range_text_is_stable  in out_of_range_text_is_stable
exit 1
```

The four magic numbers are the multiplier constants of the shipped generator, the dead
type is a test-only constant, and the three "no SUT" findings are tests that assert on
`Display` text. None is a metric this ticket asked for; all eight are the author's
opinions. With the gate configuration below (opinion dimensions off, magic numbers off)
the same tree reports `Quality Score: 100.0%`, `All quality checks passed! ✓`, exit 0,
and a coupling table of `lib` and `random` at instability 0.00. Neither tool wrote a
file into the copy.

cargo-coupling on the same copy, twice, identical output:

```text
$ cargo-coupling coupling --no-git --check crates/pawdoku
Analysis complete: 2 files, 2 modules
Coupling Quality Gate
Grade: B (100%)  ✅ PASSED
  Critical issues: 0 / High issues: 0 / Medium issues: 0 / Representative cycle paths: 0
exit 0
```

It runs `cargo metadata` first and falls back to "basic analysis" when it cannot, so it
too needs rustup's cargo on `PATH`; that is what every recipe has.

**Step 4. The boundary rules.** A stub crate with one file per module of the layering
table (`sudoku`, `technique` with a `technique/catalogue.rs` child, `solver`, `reach`,
`effort`, `lapse`, `human_solving`, `board`, `generation`, `random`), each importing
exactly what the table allows. Three rule families were tried.

- *Ranked layers* (`[architecture.layers] order = [...]`, one layer per module) cannot
  express the table. A rank order lets any module import any lower rank, so `effort`
  ranked above `reach` may import it, which the table forbids; and `random`, which
  anything may import, must sit at rank 0 or every import of it is a false violation
  (with `random` ranked last, the clean stub reported `layer reach ↛ random via
  crate::random::Draw`). `unmatched_behavior` accepts only `"strict_error"` and
  `"composition_root"`; the book's `"warn"` is rejected, and only by `--explain`: a
  normal run with the bad value silently skipped the dimension and reported
  `Architecture:100.0%`. So the configuration must be proved by a planted violation,
  never by a green run.
- *Forbidden edges* (`[[architecture.forbidden]] from = "src/reach{.rs,/**}" to =
  "src/effort{.rs,/**}"`, fifty-seven pairs generated from the table) catch the
  `use` line and miss the inline path:

  ```text
  --- clean stub
  No findings.                                              exit 0
  --- use crate::effort::Price; added to src/reach.rs
  src/reach.rs:1  ARCHITECTURE  forbidden import crate::effort::Price: layering: reach imports sudoku, technique
                                                            exit 1
  --- pub fn sneak() -> crate::effort::Price { crate::effort::price(...) } added, no use line
  No findings.                                              exit 0
  --- use crate::solver::solve added to src/technique/catalogue.rs
  src/technique/catalogue.rs:2  ARCHITECTURE  forbidden import crate::solver::solve: layering
                                                            exit 1
  ```

  The last line also shows that `src/technique/**` and `src/technique.rs` are one
  module to the glob.
- *Path patterns* (`[[architecture.pattern]] forbid_path_prefix = ["crate::effort"]
  forbidden_in = ["src/reach{.rs,/**}"]`) catch both:

  ```text
  --- inline path only
  src/reach.rs:2  ARCHITECTURE  path "crate::effort::Price": layering: reach never imports effort
  src/reach.rs:2  ARCHITECTURE  path "crate::effort::price": layering: reach never imports effort
                                                            exit 1
  --- use line only
  src/reach.rs:1  ARCHITECTURE  path "crate::effort::Price": layering: reach never imports effort
                                                            exit 1
  ```

Globs resolve against the target directory, not the working directory: run from the
workspace root as `rustqual crates/pawdoku --config rustqual.toml`, a rule over
`src/random{.rs,/**}` fired on a planted `crate::sudoku::Grid` in `random.rs`, and the
same rule written `crates/pawdoku/src/random{.rs,/**}` matched nothing. The draft
below uses the `src/` form.

So the rule that lands is one `[[architecture.pattern]]` per module, whose
`forbid_path_prefix` lists every `crate::<module>` it may not name. Ten rules encode
the whole table, nine for the specification modules and one for `random`, which
imports nothing, both import forms fail, and the `reason` is the table's row. What it
cannot see: a path reached through `super::` from a child module, and a re-export in
`lib.rs` (`crate::Price` for `crate::effort::Price`). `unreachable_pub` and
`unnameable_types` in the workspace table keep re-exports deliberate, and `lib.rs` is the
one file to review for them.

**Step 5. The clippy thresholds.** In the scratch copy, `cognitive_complexity = "warn"`
was added to `[workspace.lints.clippy]` and these keys to `clippy.toml`:

```toml
cognitive-complexity-threshold = 15
excessive-nesting-threshold = 4
too-many-lines-threshold = 60
too-many-arguments-threshold = 5
```

`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` then
reported one error, in test code:

```text
error: this block is too nested
   --> crates/pawdoku/src/random.rs:382:52
382 |                   if let Some(draws) = self.0.take() {
```

The block is an `if let` in a method of a local `impl SeqAccess for Fields` declared
inside the test helper `deserialise_replay`, itself inside `mod tests`: module,
helper function, local impl, method, block. A local item inside a function costs two
levels production code rarely pays. At `excessive-nesting-threshold = 5` the tree is
clean. With every threshold set to 1 the lints proved live: eleven
cognitive-complexity errors (the highest, 8, is the test
`clones_replay_the_same_draws_and_debug_names_the_type` at `random.rs:345`) and ten
too-many-lines errors (the longest function, 22 lines). Nothing exceeded five
arguments.

**Step 6. Verdicts.**

| Tool | Verdict | Where |
| --- | --- | --- |
| clippy thresholds | **adopt now** | gate 6, no new tool |
| rustqual | **adopt now**, as gate `metrics`, with the configuration below | `check`, between `clippy` and `features`; the `rust` CI job |
| cargo-coupling | on a trigger: when `crates/pawdoku/src` has ten modules, as a second opinion on the coupling numbers, `--no-git`, by hand or advisory | not `check` |
| cargo-crap | on a trigger: when `solver.rs` exists, reading `target/llvm-cov/lcov.info` after `coverage` | `coverage`'s CI job, advisory first |
| ast-metrics | not now; the alternative if rustqual is dropped, by the checksummed-download route | — |
| archaven | not now; the fallback for the boundary rules alone, as a dev-dependency test | — |
| cargo-archtest-cli | not at all: the boundary rules alone, in a second rule file (`architecture.json`) beside `rustqual.toml`, and AGPL-3.0; archaven is the fallback for the same job | — |
| archunit | not at all: a single 0.0.1 release (2026-09-20), the boundary rules alone | — |
| cargo-modules | not at all: no release, a slow rust-analyzer build, and rustqual reports cycles | — |
| cargo-pup | not at all: nightly, a rustc driver, writes `.pup/` | — |
| rust-code-analysis | not at all: dead since 2023-01 | — |
| Tach | not at all: Python-only | — |

What enters `check`: the clippy thresholds (gate 6, unchanged in kind) and one recipe,
`metrics`, running rustqual over `crates/pawdoku` against a committed configuration.
Thresholds, as guardrails set before the solver exists:

| Guardrail | Value | Where |
| --- | --- | --- |
| cognitive complexity per function | 15 | both: `clippy.toml` and `[complexity] max_cognitive` |
| cyclomatic complexity per function | 10 | `[complexity] max_cyclomatic` |
| nesting depth | 5 in clippy (a test `impl` costs a level); 4 in rustqual | `clippy.toml`; `[complexity] max_nesting_depth` |
| function length | 60 lines | both |
| parameters | 5 | both: `too-many-arguments-threshold`; `[srp] max_parameters` |
| LCOM4 per struct | 2 | `[srp] lcom4_threshold` |
| fields and methods per struct | 12 and 20 | `[srp] max_fields`, `max_methods` |
| instability per module that something imports | 0.8 | `[coupling] max_instability` |
| fan-in and fan-out per module | 15 and 12 | `[coupling]` |
| Stable Dependencies Principle | on | `[coupling] check_sdp` |
| module boundaries | the layering table | ten `[[architecture.pattern]]` rules |

The two nesting numbers differ because the tools count differently, and the
repository's tests live in a `mod tests` inside the file (decision 0009).

The instability cap does not reject the outer modules. `effort`, `lapse`,
`human_solving`, `board` and `generation` are imported by nothing, so each has `Ca = 0`
and the textbook `Ce / (Ca + Ce) = 1.0`. rustqual does not hold a module nothing
imports to the cap: on the stub crate of step 4 with the `[coupling]` table below, the
five read `1.00` and the run exited 0. The cap bites a module something imports; with
five imports planted in `random` (one importer, so `1/6`, `I = 0.83`) the same run
exited 1:

```text
    board                  0    1  1.00
    effort                 0    2  1.00
    generation             0    4  1.00
    human_solving          0    2  1.00
    lapse                  0    2  1.00
    ...                                            exit 0
--- five imports planted in random
    random                 1    5  0.83  ⚠ exceeds threshold
  ⚠ SDP violation: reach (I=0.75) depends on random (I=0.83)
                                                   exit 1
```

So the cap guards `sudoku`, `technique`, `solver`, `reach` and `random`, the modules
others lean on, which is where a fall in stability costs something; the outer modules
are held by the boundary rules and fan-out instead.

The honest risks of rustqual, stated for the decision record:

- It is five months old, by one author, and went from 1.0 to 1.8.3 in that time. A
  pin in `tools.txt` freezes it. Nothing bumps that pin today: there is no Dependabot
  or Renovate configuration, `docs/how-to/maintain-dependencies.md` says `tools.txt`
  moves by hand, and S01 found Dependabot cannot read the file at all (only a Renovate
  regex manager can). So a bump is a deliberate edit, one release at a time, with
  `just check` run on it; the automation S01 recommends would make it a pull request
  the gate tests, but T22 does not wait on it.
- IOSP cannot be disabled. It reports nothing on this tree, and a finding from it would
  fail the gate on an opinion the project never adopted. `// qual:allow(iosp) reason:
  "..."` is the tool's suppression (`rustqual --explain allow`: IOSP takes the bare
  form only), one line with a reason, which is the form AGENTS.md permits. But
  `max_suppression_ratio` defaults to 0.05 of the function count, which over today's
  34 functions is one line; the second surfaces as a warning, which fails only under
  `--fail-on-warnings`, so the drafted recipe passes it. What the ratio counts, tried
  on an eleven-function crate with one line of each kind: `#[expect(...)]` does not
  count (exit 0), while `#[allow(...)]` and `// qual:allow(...)` each count (9.1%
  against 5.0%, exit 1), so the ratio charges exactly the forms AGENTS.md does not
  prefer. A `qual:allow` that suppresses nothing is its own finding
  (`ORPHAN_SUPPRESSION`). The current tree passes with the flag. If IOSP findings
  arrive with the solver, the follow-up asks the author for an `enabled` key, and
  until then the ratio is the measure.
- A misconfigured architecture section is silent, and so is a wrong target: rustqual
  over a directory that does not exist, or a crate with no source, exits 0. The
  follow-up commits a probe: `metrics` also runs rustqual over a fixture crate under
  `tests/fixtures/metrics-violation/` that breaks each of the ten rules once, and
  requires exit 1 (0 is no findings, 2 a configuration it could not read) and that
  the set of rules named in the `github` output (`architecture/pattern/<name>`)
  equals the set of `name =` lines in `rustqual.toml`. A bare `!` would not do: it
  turns any non-zero status, a bad configuration included, into a pass.
- The install route. Rustqual has no binaries, so `tools.txt` needs the compile
  strategy. Its source build took 38 seconds here with the registry warm; CI pays that
  once per cache key. A source build also needs a toolchain: the first scratch install
  failed because this machine has no default, and `rust-toolchain.toml` is honoured
  only when cargo runs from inside the repository, which `install-tools` does.

**Step 7. The follow-up, drafted.** One ticket, **T22, "Metrics gate: clippy thresholds
and rustqual"**, depends on S09. One ticket rather than two because the clippy change
and the rustqual configuration state the same thresholds and must land together, or
the two will drift.

- **`clippy.toml`** (T02's file): the four keys of step 5 with `excessive-nesting-threshold
  = 5`. **`Cargo.toml`**: `cognitive_complexity = "warn"` in `[workspace.lints.clippy]`.
- **`rustqual.toml`** at the repository root, committed, run with `--config` because the
  target is `crates/pawdoku` and the tool's default is a file in the target directory:

  ```toml
  [architecture]
  enabled = true

  [complexity]
  enabled = true
  max_cognitive = 15
  max_cyclomatic = 10
  max_nesting_depth = 4
  max_function_lines = 60
  detect_magic_numbers = false

  [srp]
  enabled = true
  max_parameters = 5
  lcom4_threshold = 2
  max_fields = 12
  max_methods = 20

  [coupling]
  enabled = true
  # Applies to modules something imports; rustqual exempts the rest (step 6).
  max_instability = 0.8
  max_fan_in = 15
  max_fan_out = 12
  check_sdp = true

  [duplicates]
  enabled = false

  [boilerplate]
  enabled = false

  [test_quality]
  enabled = false

  [structural]
  enabled = false

  # One rule per module: everything it may not name, from
  # docs/explanation/layering.md. random is in no other rule's list, since anything
  # may import it, and its own rule forbids all nine: it imports nothing.
  [[architecture.pattern]]
  name = "sudoku_imports_nothing"
  forbid_path_prefix = ["crate::technique", "crate::solver", "crate::reach", "crate::effort", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/sudoku{.rs,/**}"]
  reason = "layering: sudoku may import only random"

  [[architecture.pattern]]
  name = "technique_imports_sudoku"
  forbid_path_prefix = ["crate::solver", "crate::reach", "crate::effort", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/technique{.rs,/**}"]
  reason = "layering: technique may import only sudoku and random"

  [[architecture.pattern]]
  name = "solver_imports_sudoku"
  forbid_path_prefix = ["crate::technique", "crate::reach", "crate::effort", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/solver{.rs,/**}"]
  reason = "layering: solver may import only sudoku and random"

  [[architecture.pattern]]
  name = "reach_imports_sudoku_and_technique"
  forbid_path_prefix = ["crate::solver", "crate::effort", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/reach{.rs,/**}"]
  reason = "layering: reach may import only sudoku, technique and random"

  [[architecture.pattern]]
  name = "effort_imports_sudoku_and_technique"
  forbid_path_prefix = ["crate::solver", "crate::reach", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/effort{.rs,/**}"]
  reason = "layering: effort may import only sudoku, technique and random"

  [[architecture.pattern]]
  name = "lapse_imports_sudoku_and_technique"
  forbid_path_prefix = ["crate::solver", "crate::reach", "crate::effort", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/lapse{.rs,/**}"]
  reason = "layering: lapse may import only sudoku, technique and random"

  [[architecture.pattern]]
  name = "human_solving_imports_sudoku_and_technique"
  forbid_path_prefix = ["crate::solver", "crate::reach", "crate::effort", "crate::lapse", "crate::board", "crate::generation"]
  forbidden_in = ["src/human_solving{.rs,/**}"]
  reason = "layering: human_solving may import only sudoku, technique and random"

  [[architecture.pattern]]
  name = "board_imports_sudoku"
  forbid_path_prefix = ["crate::technique", "crate::solver", "crate::reach", "crate::effort", "crate::lapse", "crate::human_solving", "crate::generation"]
  forbidden_in = ["src/board{.rs,/**}"]
  reason = "layering: board may import only sudoku and random"

  [[architecture.pattern]]
  name = "generation_imports_sudoku_solver_technique_reach"
  forbid_path_prefix = ["crate::effort", "crate::lapse", "crate::human_solving", "crate::board"]
  forbidden_in = ["src/generation{.rs,/**}"]
  reason = "layering: generation may import only sudoku, solver, technique, reach and random"

  [[architecture.pattern]]
  name = "random_imports_nothing"
  forbid_path_prefix = ["crate::sudoku", "crate::technique", "crate::solver", "crate::reach", "crate::effort", "crate::lapse", "crate::human_solving", "crate::board", "crate::generation"]
  forbidden_in = ["src/random{.rs,/**}"]
  reason = "layering: random imports nothing"
  ```

  Today only `random` exists, so `random_imports_nothing` is the one rule with a file
  to guard, and it is live from the day T22 lands: a `crate::sudoku::Grid` planted in
  the real `random.rs` failed the gate
  (`architecture/pattern/random_imports_nothing`, exit 1). The other nine name modules
  that do not exist yet; the tool accepts a glob that matches nothing, which is what
  lets the guardrail precede the code.
- **`Justfile`** (T00 follow-up): a `metrics` recipe after `clippy`,

  ```text
  metrics:
      #!/bin/sh
      set -eu
      rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github
      # The probe: the fixture breaks each boundary rule once, so rustqual must exit 1
      # (0 is no findings, 2 a configuration it could not read) and name every rule.
      status=0
      out=$(rustqual tests/fixtures/metrics-violation --config rustqual.toml --format github 2>&1) || status=$?
      want=$(sed -n 's/^name = "\(.*\)"$/\1/p' rustqual.toml | sort)
      got=$(printf '%s\n' "$out" | sed -n 's|.*architecture/pattern/\([a-z_]*\) .*|\1|p' | sort -u)
      if [ "$status" -ne 1 ] || [ "$want" != "$got" ]; then
          printf '%s\n' "$out" >&2
          printf 'metrics: probe exited %s, want 1\nrules named:\n%s\nrules fired:\n%s\n' "$status" "$want" "$got" >&2
          exit 1
      fi
  ```

  the probe being what proves the architecture section alive. Tried in the scratch
  copy as a script with this layout: the clean tree exits 0; a missing fixture, a
  mistyped glob in one rule (the output lists the rules that fired, one short), and
  `[architecture] enabled = false` each exit 1; a malformed `rustqual.toml` exits 2 at
  the first line. `want` reads every `name =` line, which holds while the pattern rules
  are the only tables with a `name`; and `install-tools` changed by one of two routes,
  the maintainer's choice, which also decides where the pin lives:
  - *Route A, one list:* `install-tools` drops `--disable-strategies compile` for every
    line, and **`tools.txt`** (T00 follow-up) gains `rustqual@1.8.3`, its comment
    amended to say a source build is allowed and why. Every future line may then
    compile too.
  - *Route B, two lists:* `install-tools` keeps the binary-only loop over `tools.txt`
    unchanged and gains a second loop, with the compile strategy allowed, over a new
    **`tools-source.txt`** holding `rustqual@1.8.3`. `tools.txt` does not change: a
    source-only pin left in it would fail the binary-only loop before the second loop
    ran.
- **`pyproject.toml`** (T00 follow-up): `"metrics"` in the recipes list after
  `"clippy"`, so `bg-*`'s aggregate runs it.
- **`.github/workflows/ci.yml`**: `just metrics` in the `rust` job after `clippy`.
- **`tests/fixtures/metrics-violation/`**: a crate with a `lib.rs` and one file per
  module of the table, `random` included, each file breaking its own rule once,
  alternating the `use` line and the inline `crate::x::Y` form so both stay proved; its
  manifest carries an empty `[workspace]` so cargo never adopts it, and it is excluded
  from `taplo.toml` if the manifest trips a lint. Tried in the scratch copy: exit 1 with exactly ten
  findings, one per rule. Adding the tenth rule before its fixture file also showed
  the probe working: the recipe failed, listing nine rules fired against ten named.
- **`docs/reference/quality-gates.md`**: a row for `metrics` between gates 6 and 7 and
  the renumbering; **`docs/reference/commands.md`** the recipe;
  **`docs/explanation/layering.md`**: "Enforcement" rewritten to name the rule file and
  what it cannot see; **`docs/reference/configuration.md`**: `rustqual.toml` and the
  new `clippy.toml` keys.
- **`docs/decisions/0013-metrics-gate.md`**: reopens the gate list of decision 0009 for
  one recipe; records the thresholds and why each number; records rustqual's age, the
  IOSP dimension that cannot be switched off, and the compile-strategy exception to the
  binstall rule; names ast-metrics and archaven as the alternatives and what would
  swap them in. **`docs/decisions/README.md`**: its row, and "the next decision this
  repository takes" moved to 0014. **`docs/manifest.yml`** (T00 follow-up, frozen at
  T00): the page's entry, `kind: decision`, or `check-docs` fails; the manifest
  requires every page.
- **The pages that state the binary-only rule**, whichever route is chosen, since each
  would otherwise say something false: `AGENTS.md` ("the one tool conda-forge lacks is
  pinned in `tools.txt`"; `just check-agents` after), `docs/how-to/maintain-dependencies.md`
  (`install-tools` "refuses a source build"), `docs/operations/troubleshooting.md` (the
  section "`just install-tools` refuses to build from source"),
  `docs/explanation/security-model.md` (cargo-hack as "the one line in `tools.txt`";
  rustqual arrives as crates.io source under `--locked` instead of a release download),
  `docs/project/repository-map.md` (`tools.txt` as "the one tool conda-forge lacks", and
  the new file under route B) and `docs/reference/commands.md`'s `install-tools` row
  ("refuses to build from source").
- **`tickets/README.md`**: T22's row, depending on S09.

**Step 8.** `status: done`; committed on this branch; not pushed.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `code-quality-checks`, not the
  `ticket/s09-code-quality-metrics` the frontmatter names, as in earlier tickets. The
  `branch:` field is left as written.
- **The ticket was written and executed in one sitting**, so the Context records what
  the research believed before anything ran, and the hand-back what running showed.
  Two beliefs changed: `cognitive_complexity` is restriction, not nursery; and
  forbidden edges are not the rule that lands, path patterns are, because only they
  see an inline path.
- **A stub crate was built for step 4** because the tree has two modules and a boundary
  rule over two modules proves nothing.
- **`just initialize` ran in this worktree** so that `just check` could run before the
  hand-back, as the change workflow requires. It changed no tracked file.

## Open points

- Whether the maintainer accepts a required gate on a five-month-old tool by one author,
  with IOSP undisablable, or prefers the archaven fallback for the boundary rules and
  rustqual as an advisory report until it has a year of releases. The spike recommends
  the gate; the research that fed it, done independently the same day from the tools'
  documentation and source, recommended the other way: clippy as the only required
  gate, rustqual and cargo-coupling as advisory reports, and cargo-modules as a
  possible second gate for cycles and orphans after a trial. The two readings weigh the
  same facts differently, so both cases are set out here.

  For the required gate:

  - The tool is pinned by version, deterministic (two runs, identical output), offline,
    fast (under a second on this tree) and writes nothing. Those are the gate's
    properties, and every one was observed rather than read.
  - A guardrail that only reports is a guardrail nobody reads until the code has
    already grown past it. The maintainer chose thresholds set before the solver
    exists precisely so that they shape it; an advisory report cannot do that.
  - The failure mode of a young tool is a false finding, and a false finding costs one
    suppression line with a reason, which AGENTS.md already permits. It is not a wrong
    build.
  - The boundary rules have no other standalone home. archaven is younger still
    (created 2026-05, four stars, no commit since June) and adds a dev-dependency to
    the core crate; cargo-modules checks cycles only.

  For advisory first:

  - Five months, one author, and eight minor versions since 1.0. A version bump may
    rename a rule, change a default or tighten a heuristic, and every bump is then a
    gate failure to triage rather than a report to glance at. The pin defers this; it
    does not remove it.
  - IOSP cannot be disabled and the suppression budget is one line today. If the
    solver's natural shape offends IOSP in three places, the gate is failing on an
    opinion the project never adopted, and the only exits are to widen
    `max_suppression_ratio` (a threshold moved to fit the code, which is what the
    maintainer wanted to avoid) or to wait on an upstream `enabled` key.
  - A misconfiguration is silent. The fixture probe in the `metrics` recipe covers the
    architecture section; it does not prove the complexity, SRP or coupling sections are
    running with the thresholds written, and a green gate would look identical to one
    that had quietly stopped measuring.
  - The gate list was frozen at T00 and reopening it is a decision record. Reopening
    it for a tool that may need replacing in a year spends that decision twice.

  A middle path the decision record can take: land T22 with `metrics` in `check`, the
  rustqual pin held at 1.8.3 until the solver has landed under it, then bumped by hand
  one release at a time (nothing bumps `tools.txt` until S01's follow-up adds a
  Renovate regex manager for it), and a stated exit, that if two consecutive bumps
  each need a suppression or a threshold change, `metrics` moves out of `check` and
  into an advisory job on the `audit.yml` pattern, with the boundary rules alone kept
  as an archaven test. Whichever way, the clippy thresholds are in the gate
  regardless; nothing in either case argues against them.
- Route A or route B for `install-tools` (step 7): the compile strategy for every line
  of `tools.txt`, or a second list, `tools-source.txt`, that alone may compile. The
  choice also decides where the rustqual pin lives. The spike has no preference; the
  decision record does.
- Whether the nesting guardrail should be 4 in both tools, which means moving the one
  test block at `random.rs:382` (test code, so T02's or the next Rust ticket's), or 5
  in clippy as drafted.

All three were answered by the maintainer on 2026-09-29 and are carried into
`T22-metrics-gate.md`: the required gate with the stated exit; route B, a
`tools-source.txt` beside `tools.txt`; and nesting 4 in both tools, so T22 moves the test
block at `random.rs:382`.
