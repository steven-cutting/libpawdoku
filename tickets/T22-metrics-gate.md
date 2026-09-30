---
id: T22
title: "Metrics gate: clippy thresholds and rustqual as gate 7, with the boundary rules from the layering table"
status: done
depends_on: [S09]
parallel_with: []
branch: ticket/t22-metrics-gate
estimated_size: M
---

# T22: Metrics gate: clippy thresholds and rustqual as gate 7, with the boundary rules from the layering table

## Context

S09 (`S09-code-quality-metrics.md`, done 2026-09-29) weighed eleven tools for the four
metric families the maintainer asked for, complexity, cohesion, coupling and module
boundaries, and recommended two things that enter `check`: thresholds for the size and
complexity lints clippy already runs, and rustqual 1.8.3 as a new recipe, `metrics`,
against a committed `rustqual.toml` whose ten `[[architecture.pattern]]` rules encode the
import table in `docs/explanation/layering.md`. Every threshold is a guardrail set before
the solver exists, so that it shapes the solver rather than being fitted to it. S09's
hand-back notes hold the evidence: the tool is deterministic (two runs, identical
output), offline, under a second on this tree, writes nothing, and exits 1 on a planted
violation in either import form, the `use` line and the inline `crate::x::Y` path.

The maintainer answered S09's three open points on 2026-09-29, and this ticket builds
on the answers:

1. **rustqual is a required gate in `check`, with a stated exit.** The pin is held at
   1.8.3 until the solver has landed under it, then bumped by hand one release at a time
   (nothing bumps a tool pin automatically today; S01's follow-up may add a Renovate regex
   manager). If two consecutive bumps each need a suppression or a threshold change,
   `metrics` leaves `check` for an advisory job on the `audit.yml` pattern, and the
   boundary rules alone are kept as an archaven dev-dependency test. The exit lives in
   decision 0013's "What would reopen this".
2. **Route B for the install.** `tools.txt` stays binary-only and unchanged. A second
   list, `tools-source.txt`, holds the one pin that must compile, `rustqual@1.8.3`, and
   `install-tools` gains a second loop over it that allows the compile strategy.
3. **Nesting is 4 in both tools.** S09 drafted 5 for clippy because one test block,
   the `if let` inside a local `impl SeqAccess for Fields` in the helper
   `deserialise_replay` at `crates/pawdoku/src/random.rs:382`, sits five levels deep.
   This ticket moves that block instead of the threshold.

Facts from S09 that decide the shape here:

- rustqual's architecture dimension is off unless `[architecture] enabled = true` is
  written, and a misconfigured section (a bad `unmatched_behavior` value, a mistyped
  glob, a target directory that does not exist) is **silent**: the run exits 0 and
  reports `Architecture:100.0%`. So the recipe carries a probe: a fixture crate that
  breaks every boundary rule once, which must exit 1 and name every rule. A green run
  proves nothing on its own.
- Globs in `rustqual.toml` resolve against the target directory, so the rules are
  written `src/<module>{.rs,/**}` and the tool is run from the repository root as
  `rustqual crates/pawdoku --config rustqual.toml`. The brace form treats
  `src/technique.rs` and `src/technique/` as one module, the two forms
  `mod_module_files` leaves.
- The `[coupling] max_instability` cap applies only to a module something imports;
  rustqual exempts a module with no importers, so the outer modules (`effort`, `lapse`,
  `human_solving`, `board`, `generation`, each `I = 1.00` by the textbook formula) pass.
- IOSP, the author's own dimension, cannot be disabled and has no `enabled` key. It
  reports nothing on this tree. Its suppression is `// qual:allow(iosp) reason: "..."`,
  one line with a reason, and `max_suppression_ratio` (default 0.05 of the function
  count) charges `#[allow]` and `qual:allow` lines but not `#[expect]`.
- `cognitive_complexity` is in clippy's `restriction` group at 1.98.1, so the workspace
  table must name it; `excessive_nesting`, `too_many_lines` and `too_many_arguments` are
  already live through `complexity` and `pedantic`, and `excessive_nesting` is silent
  until `clippy.toml` sets its threshold.
- A source build needs a toolchain. This machine has no rustup default, and
  `rust-toolchain.toml` is honoured only when cargo runs from inside the repository,
  which `install-tools` does. S09 installed rustqual with `cargo install --locked`
  directly, in 38 seconds with the registry warm; it did not run cargo-binstall's
  compile fallback, so whether that fallback carries `--locked` through to
  `cargo install` is checked in step 2, not assumed.
- Inserting `metrics` as gate 7 renumbers gates 7 to 18 to 8 to 19. Every page that cites
  a gate by number is listed under Files touched; the list came from a grep and the
  step that edits it re-runs the grep.
- The gate list was frozen at T00 (decision 0009 states it and S03 recorded it as
  frozen). Reopening it for one recipe is a decision record, 0013, and decision 0009
  gains a sentence pointing at it, because the decisions index says a superseded record
  is marked rather than silently bypassed.

Read first: `tickets/CONVENTIONS.md` §1, §4, §13; `tickets/S09-code-quality-metrics.md`
steps 4 to 7 of the hand-back notes, which hold the tested configuration, the recipe and
the fixture layout; `docs/reference/quality-gates.md`; `docs/explanation/layering.md`;
`docs/decisions/0009-rust-quality-gate.md`; `docs/decisions/0012-generation-and-dev-time-judges.md`
as the shape of the newest decision; `Justfile` (`install-tools`, `clippy`, `check`);
`tools.txt`; `pyproject.toml` (`[tool.biscuit-games-tooling] recipes`);
`.github/actions/setup/action.yml` (the `.tools` cache key hashes `tools.txt`);
`.github/workflows/ci.yml` (the `rust` job); `clippy.toml`; `Cargo.toml`'s lint table.

## Goal

`just check` runs nineteen gates, the new one `metrics` at position 7 between `clippy` and
`features`: rustqual 1.8.3 over `crates/pawdoku` against the committed `rustqual.toml`,
with `--fail-on-warnings`, followed by the fixture probe, which must exit 1 and name all
ten boundary rules. `clippy.toml` carries the four thresholds and the workspace table
names `cognitive_complexity`. The pin lives in `tools-source.txt`, installed by
`install-tools`'s second loop, and CI's `.tools` cache key hashes both lists. Decision
0013 records the gate, every threshold and why its number, rustqual's risks and the exit.
Every page that stated the binary-only rule or a gate number is true again. The tree is
green at the thresholds with no suppression added.

The thresholds, as S09 set them and the maintainer confirmed:

| Guardrail | Value | Where |
| --- | --- | --- |
| cognitive complexity per function | 15 | `clippy.toml` `cognitive-complexity-threshold`; `[complexity] max_cognitive` |
| cyclomatic complexity per function | 10 | `[complexity] max_cyclomatic` |
| nesting depth | 4 | `clippy.toml` `excessive-nesting-threshold`; `[complexity] max_nesting_depth` |
| function length | 60 lines | `clippy.toml` `too-many-lines-threshold`; `[complexity] max_function_lines` |
| parameters | 5 | `clippy.toml` `too-many-arguments-threshold`; `[srp] max_parameters` |
| LCOM4 per struct | 2 | `[srp] lcom4_threshold` |
| fields and methods per struct | 12 and 20 | `[srp] max_fields`, `max_methods` |
| instability, modules something imports | 0.8 | `[coupling] max_instability` |
| fan-in and fan-out per module | 15 and 12 | `[coupling] max_fan_in`, `max_fan_out` |
| Stable Dependencies Principle | on | `[coupling] check_sdp` |
| module boundaries | the layering table | ten `[[architecture.pattern]]` rules |

## Non-goals

- Adopting cargo-coupling, cargo-crap, ast-metrics or archaven. S09 gave each a trigger
  or a fallback role; none fires here. archaven is named in 0013 as the exit's fallback
  only.
- Changing any threshold to fit the code. If a threshold fires on the tree today, the
  code moves (as the nesting block does) or the ticket stops and reports; the numbers
  are the maintainer's.
- Changing the coverage floor, the order of the existing gates, or `tools.txt`.
- A baseline file, `--fail-on-regression` or `--compare`: `check-clean` fails anything
  that rewrites a baseline, and a gate cannot depend on how deep the clone is.
- Bumping rustqual past 1.8.3, or wiring any automation to bump it.
- Metrics over the specifications. `analyse-specs` is that gate.

## Files touched

Frozen files (T00's) are marked as the T00 follow-ups they are, and the class tells the
reviewer why each is touched. No dependency is added, so there is no T02 hand-back.

| Path | Class | Change |
| --- | --- | --- |
| `rustqual.toml` | new, gate config | S09 step 7's configuration verbatim: dimensions, thresholds, the ten pattern rules with their `reason` lines. taplo will rewrap the long arrays at 100 columns; that is the committed form. |
| `tools-source.txt` | new, pin (T00 follow-up class) | Header comment stating what may live here and why a source build is allowed for it; one line, `rustqual@1.8.3`. |
| `tests/fixtures/metrics-violation/` | new, fixture | A crate-shaped directory: `src/lib.rs` declaring ten modules, one file per module of the layering table (`random` included), each breaking its own rule exactly once, alternating the `use` form and the inline path form. Step 5 decides whether a `Cargo.toml` is needed. |
| `Justfile` | T00 follow-up | `metrics` recipe after `clippy` (S09's text, below); `install-tools` gains the second loop over `tools-source.txt`; the header comment at line 6 names both lists. |
| `pyproject.toml` | T00 follow-up | `"metrics"` after `"clippy"` in `[tool.biscuit-games-tooling] recipes`. |
| `clippy.toml` | T02's config | The four threshold keys. |
| `Cargo.toml` | T02's lint table | `cognitive_complexity = "warn"` in `[workspace.lints.clippy]`, with a one-line comment that it is a restriction lint pedantic does not enable. |
| `crates/pawdoku/src/random.rs` | code | The local `impl SeqAccess for Fields` and its struct move out of `deserialise_replay` to `mod tests` level, so no block is deeper than four; behaviour and coverage unchanged. |
| `.github/workflows/ci.yml` | CI | `- run: just metrics` after `just clippy` in the `rust` job. |
| `.github/actions/setup/action.yml` | CI | The `.tools` cache key becomes `hashFiles('tools.txt', 'tools-source.txt')`, and the comment names both. |
| `.gitignore` | T00 follow-up | The comment at line 36, which names `tools.txt` alone, names both lists. |
| `scripts/initialize.sh` | script | The comment at line 25, which names `tools.txt` alone, names both lists. |
| `docs/decisions/0013-metrics-gate.md` | new decision | See step 9. |
| `docs/decisions/README.md` | decision index | Row for 0013; "The next decision this repository takes is 0014." |
| `docs/decisions/0009-rust-quality-gate.md` | decision | "seventeen gates" and "Gates 4 to 13" corrected to the current count and range; one sentence under "What would reopen this" or after "Decision" saying 0013 reopened the list for `metrics`. |
| `docs/decisions/0011-tool-manager.md` | decision | Line 47, "`tools.txt` remains as the escape hatch": one sentence saying `tools-source.txt` is the second, for a crate with no binary, per decision 0013. The other `tools.txt` mentions in 0004, 0005 and 0011 stay true. |
| `CHANGELOG.md` | maintainer docs | Under Unreleased, Added: the `metrics` gate and the clippy thresholds; the "seventeen read-only gates" bullet at line 15 becomes the current count, and the `.tools/bin` bullet at line 24 names rustqual beside cargo-hack. |
| `docs/manifest.yml` | T00 follow-up | Entry for the 0013 page, `kind: decision`, `canonical_for: [decision_metrics_gate]`. |
| `docs/reference/quality-gates.md` | reference | Row for `metrics` at 7, "Proves" text; gates 7 to 18 renumbered; the prose citations at lines 27 ("gate 9's"), 40 ("Gates 16 and 17"), 71 ("Gates 4, 5 and 13") updated; a paragraph on the probe and why a green run alone is not proof. |
| `docs/reference/commands.md` | reference | `just metrics` row; every "Gate N" in the recipe table renumbered from `features` on; `just install-tools` row no longer says "refuses to build from source" and names both lists; `just initialize` row likewise. |
| `docs/reference/configuration.md` | reference | Rows for `rustqual.toml` and `tools-source.txt`; the `clippy.toml` row gains the thresholds; the `tools.txt` row drops "Today one line" wording if it becomes false; the owner table at line 113 gains the source list. |
| `docs/explanation/layering.md` | explanation | "Enforcement" rewritten: the rule file, one rule per module, both import forms caught, the probe, and what it cannot see (`super::` paths from a child module, re-exports through `lib.rs`), with `unreachable_pub` and `unnameable_types` as the guard for the second. |
| `docs/explanation/security-model.md` | explanation | Line 48: rustqual arrives as crates.io source built under `--locked`, beside cargo-hack's release download. |
| `docs/how-to/maintain-dependencies.md` | how-to | Lines 74 to 77: the two lists, what each may hold, and that the source list is bumped one release at a time with `just check` run on it; line 28's gate number. |
| `docs/how-to/develop-locally.md` | how-to | Lines 23, 76 and 127: both lists. |
| `docs/operations/troubleshooting.md` | operations | The section "`just install-tools` refuses to build from source" restated for two lists; a section for a rustqual source build failing with no toolchain (run from the repository root). |
| `docs/operations/maintenance.md` | operations | Line 27: both lists move by hand. |
| `docs/project/repository-map.md` | project | Tree gains `tools-source.txt`, `rustqual.toml` and `tests/fixtures/`; the `tools.txt` line and the `.tools/bin` row name both. |
| `docs/reference/api.md` | reference | Lines 20 and 36: gate numbers. |
| `docs/decisions/0005-project-managed-allium-cli.md`, `0006-apache-2-0.md`, `0007-dependency-policy.md`, `0008-no-std-core.md` | decisions | Gate numbers only (0005 line 77; 0006 line 44; 0007 line 34; 0008 line 29). `0008` line 38 cites gate 6, which does not move. |
| `.agents/skills/project-check/SKILL.md` | skill | Line 9: "gates 3, 16 and 17" renumbered; `.tools/bin/rustqual` joins the prerequisites list. |
| `AGENTS.md` | agent contract | Line 83: "the one tool conda-forge lacks is pinned in `tools.txt`" becomes the two lists and what each may do; the invariant list is unchanged. |
| `README.md` | maintainer docs | Lines 18 and 59: both lists. |
| `tickets/CONVENTIONS.md` | ticket conventions | §2 (`tools.txt`, "frozen at T00") and §4 (the `Justfile`, "complete; frozen at T00") are records of what T00 was told to build, and line 527 cites gate 6, which does not move. Not rewritten: a dated one-line note under each frozen block points at this ticket and decision 0013. |
| `tickets/README.md` | ticket index | T22's row set `done`. |
| `tickets/T22-metrics-gate.md` | ticket | `status:`, hand-back notes. |

## Steps

1. Create the worktree on `ticket/t22-metrics-gate` from `main` (README.md "How to pick
   up a ticket"). Run `just initialize` if `.pixi/` is absent; it is a network operation
   the ticket authorises. Record `git rev-parse --short HEAD` and an empty
   `git status --porcelain`.

2. **The pin and the install route.** Write `tools-source.txt`:

   ```text
   # Tools that must be built from source: crates on crates.io with no release binary
   # and no conda-forge package, installed by `just install-tools` into the gitignored
   # .tools/bin with cargo-binstall's compile strategy allowed, one name@version per
   # line. tools.txt is binary-only and stays so; a pin here costs a source build at
   # bootstrap, which decision 0013 accepts for the one tool below and no other by
   # default. Bumped by hand, one release at a time, with `just check` run on it.
   rustqual@1.8.3
   ```

   Change `install-tools` to two loops, the first unchanged:

   ```text
   install-tools:
       grep -v '^#' tools.txt | xargs cargo binstall --root .tools --no-confirm --locked --disable-strategies compile
       grep -v '^#' tools-source.txt | xargs cargo binstall --root .tools --no-confirm --locked
   ```

   Run `just install-tools` from the worktree root and quote the lines cargo-binstall
   prints for rustqual, including which strategy it took and the build time. Confirm
   `.tools/bin/rustqual --version` prints 1.8.3 and that `cargo-qual` is beside it. If
   cargo-binstall's compile fallback does not pass `--locked` to `cargo install` (check
   its `-v` output), record that and use `cargo install --locked --root .tools
   rustqual@1.8.3` in the second loop instead, with the reason in a comment.

3. **The clippy thresholds and the nesting block.** Add to `clippy.toml`:

   ```toml
   cognitive-complexity-threshold = 15
   excessive-nesting-threshold = 4
   too-many-lines-threshold = 60
   too-many-arguments-threshold = 5
   ```

   Add `cognitive_complexity = "warn"` to `[workspace.lints.clippy]`. Run `just clippy`;
   expect exactly one error, `this block is too nested` at `random.rs:382`. Move the
   `Fields` struct and its `impl SeqAccess` out of `deserialise_replay` to the level of
   `mod tests`, keeping the helper's signature and every call site. Run `just clippy`
   and `just coverage`; both green, the coverage figure unchanged or higher. Quote the
   before and after.

4. **`rustqual.toml`.** Copy S09 step 7's configuration to the repository root. Run
   `just toml-check`; if taplo rewraps the `forbid_path_prefix` arrays, accept its form
   (run `just fix` or `taplo fmt rustqual.toml`) and commit that. Run
   `rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github`
   from the root; expect exit 0 and `Quality Score: 100.0%`. Then plant
   `use crate::sudoku::Grid;` in `random.rs`, run again, and quote the
   `architecture/pattern/random_imports_nothing` finding and exit 1; revert.

5. **The fixture.** Create `tests/fixtures/metrics-violation/src/lib.rs` with
   `mod sudoku; mod technique; ...` for all ten modules, and one file per module that
   names exactly one forbidden module of its rule: `use crate::<forbidden>::Item;` in
   the odd files and an inline `crate::<forbidden>::item()` in a function body in the
   even files, so both forms stay proved. `random.rs` breaks `random_imports_nothing`.
   First try rustqual over the directory **without** a `Cargo.toml`; if it scans and
   reports ten findings, no manifest is written and taplo, cargo and `deny` never see
   the fixture. If it needs a manifest, write the smallest `Cargo.toml` with
   `[package] name = "metrics-violation"` and an empty `[workspace]` table so cargo
   never adopts it, run `just toml-check`, `just deps-unused` and `just deny`, and
   record what each did with it. Either way, quote the run: exit 1, ten findings, one
   per rule.

6. **The recipe.** Add after `clippy` in the `Justfile`:

   ```text
   # Gate 7: complexity, cohesion, coupling and the module boundaries of
   # docs/explanation/layering.md, through rustqual against rustqual.toml. The
   # probe follows because a misconfigured architecture section is silent: the
   # fixture breaks each boundary rule once, so rustqual must exit 1 (0 is no
   # findings, 2 a configuration it could not read) and name every rule.
   metrics:
       #!/bin/sh
       set -eu
       rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github
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

   Add `"metrics"` after `"clippy"` in `pyproject.toml`'s recipes list. Prove the probe
   live three ways and quote each: `just metrics` green on the tree; with
   `[architecture] enabled = false` it fails naming ten rules and none fired; with one
   fixture file's violation removed it fails naming ten against nine. Revert each. Check
   the `want` line still holds: it reads every `name =` line in `rustqual.toml`, which is
   correct while the pattern rules are the only tables with a `name` key; leave a
   comment saying so.

7. **CI.** `- run: just metrics` after `just clippy` in the `rust` job of `ci.yml`. The
   `.tools` cache key in `.github/actions/setup/action.yml` becomes
   `hashFiles('tools.txt', 'tools-source.txt')`, and its comment names both lists;
   without this a pin bump would restore a stale `.tools/bin` (cargo-binstall skips a
   matching version, and would not see the mismatch). Run `actionlint` through `just
   lint`.

8. **The pages.** Re-run the two greps that produced the Files touched list and edit
   every hit that becomes false, then re-run them and quote the result:

   ```sh
   grep -rnE '[Gg]ates? [0-9]+' docs AGENTS.md .agents README.md
   grep -rn 'tools.txt' --exclude-dir=.pixi --exclude-dir=.tools --exclude-dir=target --exclude-dir=.git --exclude-dir=tickets --exclude-dir=ai_tmp .
   ```

   The renumbering: `features` 8, `wasm-check` 9, `test-doc` 10, `coverage` 11, `doc`
   12, `deny` 13, `deps-unused` 14, `check-docs` 15, `check-agents` 16, `check-specs`
   17, `analyse-specs` 18, `check-clean` 19. Gates 1 to 6 do not move. Then the
   Enforcement rewrite in `layering.md`, the `quality-gates.md` row and probe paragraph,
   and the reference rows; each page stays within what `docs/manifest.yml` says it owns.
   `just check-docs` and `just check-agents` after.

9. **Decision 0013**, `docs/decisions/0013-metrics-gate.md`, in the shape of 0012:
   frontmatter with `canonical_for: [decision_metrics_gate]`; Context (what the gate
   measured before, the maintainer's four families, S09); Decision (the `metrics`
   recipe at gate 7, the probe, the clippy thresholds, every threshold with one line on
   why the number, `tools-source.txt` as the one exception to the binary-only rule and
   why compile is acceptable for it); Consequences, the ones that hurt (a young tool by
   one author, eight minor versions since 1.0; IOSP undisablable with a suppression
   budget of one line today; a source build at bootstrap; a green run that could hide a
   dead section, which is why the probe exists and what it does not prove, the
   complexity, SRP and coupling sections); What would reopen this (the exit: two
   consecutive bumps each needing a suppression or a threshold change move `metrics` to
   an advisory job and keep the boundary rules as an archaven test; ast-metrics by the
   checksummed-download route as the other alternative; an upstream `enabled` key for
   IOSP; rustqual on conda-forge or with release binaries, which empties
   `tools-source.txt`). Add the manifest entry, the README row, and the pointer sentence
   in 0009.

10. `just check` green, nineteen gates. `git status --porcelain` empty after it. Quote
    the gate list `bg-project-check run` prints.

11. Hand-back notes: what ran, the quoted outputs of every step, every file rustqual
    wrote (expected none), the clippy and rustqual run times cold and warm, and the
    deviations. Set `status: done`, set the index row, commit on the ticket branch.
    Stop before pushing.

## Acceptance criteria

- `just check` is green and prints nineteen gates with `metrics` seventh.
- `rustqual --version` from `.tools/bin` prints 1.8.3; `tools.txt` is byte-identical to
  `main`; `tools-source.txt` has one pin.
- The probe fails under each of the two sabotage runs of step 6 with the stated
  message, and the tree passes with none.
- A planted `use crate::sudoku::Grid;` in `random.rs` fails `just metrics` naming
  `random_imports_nothing`; the fixture fails with exactly ten findings, one per rule.
- `just clippy` is green with `excessive-nesting-threshold = 4` and no `#[expect]` or
  `#[allow]` added anywhere; `just coverage` is at or above its prior figure.
- No `qual:allow` line exists in `crates/`.
- `grep -rnE '[Gg]ates? [0-9]+' docs AGENTS.md .agents README.md` shows every citation
  agreeing with the nineteen-gate table, and `grep -rn 'refuses to build from source\|refuses a source build\|the one tool conda-forge lacks' AGENTS.md docs README.md`
  prints nothing.
- Decision 0013 exists, is in the manifest and the decisions index, states every
  threshold and the exit, and 0009 points at it; the index says the next decision is
  0014.
- `hashFiles` in the setup action names both lists.
- `just check-docs`, `just check-agents` and `just lint` pass.

## Verification

```sh
.tools/bin/rustqual --version
cat tools-source.txt | grep -v '^#'
diff <(git show main:tools.txt) tools.txt && echo "tools.txt unchanged"
grep -n 'excessive-nesting-threshold' clippy.toml
grep -n 'cognitive_complexity' Cargo.toml
just clippy
just metrics
grep -c '^name = ' rustqual.toml
grep -n 'just metrics' .github/workflows/ci.yml
grep -n "hashFiles('tools.txt', 'tools-source.txt')" .github/actions/setup/action.yml
grep -rnE '[Gg]ates? [0-9]+' docs AGENTS.md .agents README.md
grep -rn 'refuses to build from source\|refuses a source build\|the one tool conda-forge lacks' AGENTS.md docs README.md || echo "binary-only rule restated"
grep -n '0013\|0014' docs/decisions/README.md docs/manifest.yml docs/decisions/0009-rust-quality-gate.md
grep -rn 'qual:allow' crates || echo "no suppressions"
just check
git status --porcelain
```

Expected: `rustqual 1.8.3`; `rustqual@1.8.3`; `tools.txt unchanged`; the threshold and the
lint each on one line; both recipes green with no finding; `10`; one line each in the
two workflow files; every gate citation matching the nineteen-gate table; `binary-only
rule restated`; 0013 in all three files and 0014 as the next number; `no suppressions`;
`check` green; nothing from `git status`.

## Hand-back notes

### What ran, and how

Executed on 2026-09-29 in the Supacode worktree `S09-code-quality-metrics`, whose branch
the maintainer had renamed to `ticket/t22-metrics-gate` (see Deviations). It sits on
`f7a1530`, this ticket's own commit over `main` at `9bedca8`. The maintainer authorised
`just initialize` and the new `just install-tools` (network), and read-only GitHub API
lookups of rustqual's releases and cargo-quickinstall's tags.

**Step 1.** `git rev-parse --short HEAD` printed `f7a1530`, and `git status --porcelain`
printed nothing. `just initialize` exited 0. As a secondary worktree it skipped
`install-hooks`, and it changed no tracked file.

**Step 2. The pin and the install route.** `tools-source.txt` is the ticket's text, and
`install-tools` gained the second line as written. The recipe's rustqual line was run
once by hand with `-v` to see the strategy; these are the lines that matter:

```text
cargo-binstall:  INFO resolve: Resolving package: 'rustqual@=1.8.3'
cargo-binstall: DEBUG has_release_artifact{... tag: "v1.8.3" ...}: ... nodes: [] ...
cargo-binstall: DEBUG Failed to download signature, skipping verification: ... cargo-quickinstall/releases/download/rustqual-1.8.3/rustqual-1.8.3-aarch64-apple-darwin.tar.gz.sig: HTTP status client error (404 Not Found)
cargo-binstall:  WARN The package rustqual v1.8.3 will be installed from source (with cargo)
cargo-binstall: DEBUG Running `/Users/scutting/.rustup/toolchains/1.98.1-aarch64-apple-darwin/bin/cargo install rustqual --version 1.8.3 --locked --root .tools`
    Finished `release` profile [optimized] target(s) in 37.46s
   Installed package `rustqual v1.8.3` (executables `cargo-qual`, `rustqual`)
```

So the compile fallback does pass `--locked` through to `cargo install`, and the second
line stays as the ticket wrote it. `just install-tools` afterwards:

```text
cargo-binstall:  INFO cargo-hack v0.6.45 is already installed, use --force to override
cargo-binstall:  INFO rustqual v1.8.3 is already installed, use --force to override
$ .tools/bin/rustqual --version
rustqual 1.8.3
$ ls .tools/bin
allium  cargo-hack  cargo-qual  rustqual
```

The log also showed cargo-binstall's default order: the crate's own release
(`crate-meta-data`), then a cargo-quickinstall build (`quick-install`), then compile.
The maintainer asked whether compiling could be avoided at all. It cannot today. None of
rustqual's 35 GitHub releases (v0.3.8 to v1.8.3) has an asset, because its `release.yml`
publishes to crates.io only. cargo-quickinstall has no `rustqual` tag. The crate is not
on conda-forge (S09) or Homebrew (`brew info rustqual`: no formula). rustqual's own
`book/getting-started.md` and `book/ci-integration.md` install with
`cargo install rustqual`, which compiles too. cargo-quickinstall's builds are signed
(cargo-hack 0.6.45 has a `.sig` beside every archive), and cargo-binstall checks the
signature when one exists. The maintainer chose to keep the default strategies: rustqual
compiles today, and a signed quickinstall build would be taken if one appeared. The pages
and decision 0013 say so.
*Follow-up, 2026-09-30 (PR #20 review):* the defaults also put `crate-meta-data` first,
which would take an unsigned release asset from rustqual's own repository ahead of the
signed build. The maintainer chose to disable it: the `tools-source.txt` pass now passes
`--disable-strategies crate-meta-data`, leaving a signed quickinstall build or the
crates.io source build, as the security model says.
*Follow-up, 2026-09-30 (Codex, PR #20):* CI's setup action sets `GITHUB_TOKEN` for the
whole `install-tools` step, so the source build's build scripts inherited it. The
`tools-source.txt` pass now runs under `env -u GITHUB_TOKEN -u GH_TOKEN` with
`--no-discover-github-token`; the `tools.txt` pass keeps the token for its rate limit.
*Follow-up, 2026-09-30 (Copilot, PR #20):* the probe compared the rules it saw after
`sort -u`, so a rule firing twice passed. It now compares without deduplicating, so each
rule must fire exactly once, as the fixture's files say; each does today.
*Follow-up, 2026-09-30 (Copilot, PR #20):* both `install-tools` passes use `xargs -r`, so
an emptied list, which 0013 foresees for `tools-source.txt`, skips its pass instead of
running `cargo binstall` with no crate under GNU xargs. 0013's cost sentence now says
each of the five gate jobs compiles rustqual on a cold cache, as its Consequences do.

**Step 3. The clippy thresholds and the nesting block.** With the four keys in
`clippy.toml` and `cognitive_complexity = "warn"` in the lint table, `just clippy`
reported exactly one error:

```text
error: this block is too nested
   --> crates/pawdoku/src/random.rs:382:52
382 |                   if let Some(draws) = self.0.take() {
error: could not compile `pawdoku` (lib test) due to 1 previous error
```

`Fields` and its `impl SeqAccess` moved out of `deserialise_replay` to the level of
`mod tests`, each under `#[cfg(feature = "serde")]`. The impl names serde's traits by
path and imports `IntoDeserializer` and `SeqDeserializer` inside its method, so no
module-level `use` needs a `cfg`. The helper's signature and both call sites are
unchanged. After the move:

```text
$ just clippy
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
$ just coverage            # before, then after
TOTAL  219  5  97.72%  23  0  100.00%  139  3  97.84%  ...
TOTAL  219  5  97.72%  23  0  100.00%  139  3  97.84%  ...
```

Coverage is unchanged, and no `#[expect]` or `#[allow]` was added.

**Step 4. `rustqual.toml`.** S09 step 7's configuration went in verbatim, under a
four-line header comment. `just toml-check` failed on the long `forbid_path_prefix`
arrays; `taplo fmt rustqual.toml` rewrapped them one path per line, and that is the
committed form. The ten `name = "..."` lines survived the rewrap, so the probe's `want`
still reads them (`grep -c '^name = ' rustqual.toml` prints `10`).

```text
$ rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github
::notice::Quality score: 100.0% (35 functions analyzed)
exit 0
```

With `use crate::sudoku::Grid;` planted in `random.rs`, through `just metrics`:

```text
::warning file=src/random.rs,line=434::architecture/pattern/random_imports_nothing — path "crate::sudoku::Grid": layering: random imports nothing
::error::Quality analysis: 1 finding(s) (0 IOSP violation(s)), 98.0% quality score
error: recipe `metrics` failed with exit code 1
```

Reverted.

**Step 5. The fixture.** The bare directory, with no `Cargo.toml`, scans, so no manifest
was written, and taplo, cargo and `deny` never see the fixture. Odd files (`sudoku`,
`solver`, `effort`, `human_solving`, `generation`) use a `use` line, and even files use
an inline path:

```text
$ rustqual tests/fixtures/metrics-violation --config rustqual.toml --format github
::warning file=src/board.rs,line=6::architecture/pattern/board_imports_sudoku — path "crate::generation::item": ...
::warning file=src/effort.rs,line=3::architecture/pattern/effort_imports_sudoku_and_technique — path "crate::lapse::item": ...
::warning file=src/generation.rs,line=3::architecture/pattern/generation_imports_sudoku_solver_technique_reach — path "crate::effort::item": ...
::warning file=src/human_solving.rs,line=3::architecture/pattern/human_solving_imports_sudoku_and_technique — path "crate::board::item": ...
::warning file=src/lapse.rs,line=6::architecture/pattern/lapse_imports_sudoku_and_technique — path "crate::effort::item": ...
::warning file=src/random.rs,line=6::architecture/pattern/random_imports_nothing — path "crate::sudoku::item": ...
::warning file=src/reach.rs,line=6::architecture/pattern/reach_imports_sudoku_and_technique — path "crate::effort::item": ...
::warning file=src/solver.rs,line=3::architecture/pattern/solver_imports_sudoku — path "crate::technique::item": ...
::warning file=src/sudoku.rs,line=3::architecture/pattern/sudoku_imports_nothing — path "crate::technique::item": ...
::warning file=src/technique.rs,line=6::architecture/pattern/technique_imports_sudoku — path "crate::solver::item": ...
::error::Quality analysis: 10 finding(s) (0 IOSP violation(s)), 65.0% quality score
exit 1
```

That is ten findings, one per rule. `just lint` passes with the fixture tracked
(typos, editorconfig-checker and ripsecrets see it).

**Step 6. The recipe.** The ticket's text, with the `want` comment folded into the
recipe's header comment. `"metrics"` follows `"clippy"` in `pyproject.toml`. The three
proofs:

```text
$ just metrics                                  # the tree
::notice::Quality score: 100.0% (35 functions analyzed)
exit 0

$ just metrics                                  # [architecture] enabled = false
::notice::Quality score: 100.0% (35 functions analyzed)
::notice::Quality score: 100.0% (20 functions analyzed)
metrics: probe exited 0, want 1
rules named:
board_imports_sudoku
... (all ten)
rules fired:

error: recipe `metrics` failed with exit code 1

$ just metrics                                  # board.rs's violation removed
::error::Quality analysis: 9 finding(s) (0 IOSP violation(s)), 66.8% quality score
metrics: probe exited 1, want 1
rules named:
board_imports_sudoku
... (all ten)
rules fired:
effort_imports_sudoku_and_technique
... (nine, board_imports_sudoku missing)
error: recipe `metrics` failed with exit code 1
```

Each was reverted, and `just metrics` was green again after.

**Step 7. CI.** `- run: just metrics` follows `just clippy` in the `rust` job. The
`.tools` cache key is `hashFiles('tools.txt', 'tools-source.txt')`, and its comment names
both lists and what a key missing one would cost. `just lint` (actionlint included)
passes.

**Step 8. The pages.** The two greps were re-run after the edits. Every gate citation
agrees with the nineteen-row table: `metrics` 7, `features` 8, `wasm-check` 9,
`test-doc` 10, `coverage` 11, `doc` 12, `deny` 13, `deps-unused` 14, `check-docs` 15,
`check-agents` 16, `check-specs` 17, `analyse-specs` 18, `check-clean` 19. Gates 1 to 6
are unchanged, so `0008` line 38 and `quality-gates.md`'s "gate 6" stay. Every
remaining `tools.txt` hit is either about that list alone, and true, or names both
lists. The acceptance grep:

```text
$ grep -rn 'refuses to build from source\|refuses a source build\|the one tool conda-forge lacks' AGENTS.md docs README.md || echo "binary-only rule restated"
binary-only rule restated
```

`just check-docs` (40 pages, 41 canonical topics) and `just check-agents` pass.

**Step 9. Decision 0013** is in the shape of 0012, with the manifest entry, the index row
("the next decision this repository takes is 0014") and 0009's pointer. Writing it
turned up a fact S09 did not state: every threshold S09 chose is rustqual's documented
default at 1.8.3 (`book/reference-configuration.md`). The record says so and gives each
number's reason beside the default.

**Step 10.**

```text
$ just check
==> just check-toolchain
==> just lock-check
==> just lint
==> just fmt-check
==> just toml-check
==> just clippy
==> just metrics
==> just features
==> just wasm-check
==> just test-doc
==> just coverage
==> just doc
==> just deny
==> just deps-unused
==> just check-docs
==> just check-agents
==> just check-specs
==> just analyse-specs
==> just check-clean
All checks passed and the worktree is unchanged.
just check  82.31s user 11.14s system 468% cpu 19.928 total
```

**Files rustqual wrote:** none. `git status --porcelain --ignored` after every run showed
only the already-ignored `.lycheecache`, `.pixi/`, `.tools/`, `ai_tmp/` and `target/`.

**Run times.** Everything was measured on this machine. `just metrics` took 0.31 s cold
and 0.13 s warm; it has no cache, so the difference is the filesystem. `just clippy`
took 3.98 s cold, in a fresh target directory with its dependencies built, and 0.13 s
warm. rustqual's source build took 37.46 s with the registry warm.

### Deviations, and why

- **Branch and base.** The maintainer chose to rename the Supacode branch
  `S09-code-quality-metrics` to `ticket/t22-metrics-gate`, rather than cut a new one
  from `main`. `main` does not carry this ticket file, because `f7a1530` was never pushed.
  So the build sits on `f7a1530`, and the ticket and its build go out together. The
  worktree directory keeps its Supacode name.
- **The rustqual install was first run by hand with `-v`**, as the same command line the
  recipe runs, to read the strategy and the `cargo install` line. `just install-tools`
  then ran through the recipe.
- **A header comment on `rustqual.toml`**, four lines above S09's text: what the file is
  for, where globs resolve, and that IOSP always runs. The configuration itself is
  verbatim.
- **Lines the Files touched table did not list**, each made false by this change:
  `pyproject.toml`'s comment on cargo-binstall ("For tools.txt only"); the CI job table
  and the setup-action sentence in `quality-gates.md`; the `.tools/bin` sentence at
  `troubleshooting.md` line 80, and that page's heading "`just install-tools` refuses to
  build from source", renamed so the acceptance grep holds; `0011` line 78, where
  emptying `tools.txt` no longer removes cargo-binstall; `CHANGELOG.md`'s "Eleven
  architecture decision records", already stale at twelve, now thirteen; and one
  sentence in `docs/decisions/README.md` saying 0013 reopens 0009's list.
- **Left unchanged on purpose:** `0004` line 44, "lists the seventeen recipes". It sits
  in a paragraph that still describes `uv.lock`, which decision 0011 superseded, so the
  paragraph records its day rather than today. It is outside the acceptance grep.
- **0009's pointer is in its Context**, where the count and the range are, not under
  "What would reopen this".
- **Counts.** `quality-gates.md` has nineteen rows with `check-clean`. `CHANGELOG.md`
  and `0009` count the recipes `pyproject.toml` lists, which is eighteen now.

### For the maintainer

- **rustqual defaults nobody chose are live.** Because `[srp]` and `[complexity]` are
  enabled and the recipe passes `--fail-on-warnings`, rustqual's other defaults gate
  too. They are: SRP-002, a file over 300 production lines (the book's "warn 300, hard
  800"); a per-struct fan-out of 10; an SRP composite score of 0.6; and the A20
  error-handling check. The 300-line file limit is the one likely to meet the solver.
  Decision 0013 lists them as unchosen, and the numbers were left alone, as the
  non-goals require.
  *Follow-up, 2026-09-29:* the maintainer chose them. `rustqual.toml` now writes every
  key that can fail the gate: the file limit is 500 code lines, test code has its own
  looser limits, recursion stays forbidden, and the error-handling and `unsafe` checks
  are off because clippy and the lint table own those rules. Decision 0013 has a row
  for each.
- **Cold CI cost.** Every gate job runs the setup action, so on a new `.tools` key all
  five compile rustqual in parallel, once. Limiting the build to the `rust` job would
  mean a setup-action input, which is not this ticket's change.
- **cargo-binstall telemetry.** Whenever the `quick-install` strategy is tried, as it
  is for rustqual on every fresh install, cargo-binstall reports the crate, version and
  target to cargo-quickinstall's stats server (its `--help`). `--disable-telemetry` would
  stop it; the maintainer's choice of the default strategies left it on, and the later
  removal of `crate-meta-data` keeps `quick-install`, so it is still on.
- **The probe's message** reads "probe exited 1, want 1" when the exit is right but a
  rule is missing. That is the ticket's text; the two lists printed beneath it show
  which rule is missing.
- `tickets/CONVENTIONS.md` §2 and §4 carry the dated note the open point defaults to.

## Open points

- Whether `tests/fixtures/metrics-violation/` carries a `Cargo.toml` at all. Step 5
  settles it by trying the bare directory first; both outcomes are written there so the
  agent does not stop on it.
- Whether cargo-binstall's compile fallback honours `--locked` end to end. Step 2 checks
  its verbose output and names the `cargo install --locked` fallback if not; neither
  reading changes the pin or the file.
- Whether `tickets/CONVENTIONS.md` §2 and §4, both frozen at T00, take the dated note the
  Files touched row describes or are left untouched. The ticket defaults to the note;
  the maintainer may strike it at review.

All three were settled in the run: the fixture needs no `Cargo.toml` (step 5);
cargo-binstall's compile fallback passes `--locked` to `cargo install` (step 2); and
§2 and §4 carry the dated note, which the maintainer may still strike at review.
