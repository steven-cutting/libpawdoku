---
title: "Decision 0013: The metrics gate"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_metrics_gate]
requires: []
---

# Decision 0013: The metrics gate

## Context

Until this decision the gate measured the code's shape in one place. Clippy ran
`too_many_lines`, from its `pedantic` group, and `too_many_arguments`, from its
`complexity` group, at clippy's defaults of 100 lines and 7 arguments, and nothing else
looked at size, cohesion or coupling. The import
direction [Layering](../explanation/layering.md) sets out was enforced by review alone,
because rustc allows a cycle between modules of one crate.

The maintainer asked for four families of metric as a required gate: complexity,
cohesion, coupling and module boundaries. They asked for them as guardrails set before
the solver exists, so that the numbers shape the solver rather than being fitted to it.
Spike S09 weighed eleven tools on 2026-09-29 and recommended two things: thresholds for
the lints clippy already runs, and rustqual as a new recipe. The maintainer answered its
open points the same day: a required gate with a stated exit, a second pin list for a
tool that must be compiled, and a nesting limit of 4 in both tools.

The gate list was set at T00 and [decision 0009](0009-rust-quality-gate.md) records its
Rust recipes. Adding one reopens that list, which is what this record does.

## Decision

**Clippy's thresholds.** `clippy.toml` sets four, and the workspace lint table names
`cognitive_complexity`, a restriction lint the `pedantic` group does not enable.
`excessive_nesting` is already live through the `complexity` group but silent until its
threshold is set.

**A `metrics` recipe at gate 7**, between `clippy` and `features`, in `just check` and in
the `rust` CI job. It runs rustqual over `crates/pawdoku` against the committed
`rustqual.toml`, with `--fail-on-warnings`, then a probe. rustqual's dimensions are on for
complexity, SRP (cohesion and size), coupling and architecture; duplicates, boilerplate,
test quality and the structural checks are off, as is the magic-number check, because
each is the author's opinion rather than a metric the maintainer asked for. The
architecture dimension holds ten `[[architecture.pattern]]` rules, one per module of the
layering table. Each forbids every `crate::` path its module may not name, which catches
an inline path as well as a `use` line.

**The probe.** rustqual reports a misconfigured architecture section as a clean one, so
the recipe also runs it over `tests/fixtures/metrics-violation/`, a directory shaped like
a crate, with no manifest, that breaks each rule once, one of them from a child module so
the globs' `/**` arm is proved. The gate fails unless that run
exits 1 and names each rule `rustqual.toml` declares exactly once.
[Quality gates](../reference/quality-gates.md) describes it.

**The thresholds.** Every key that can fail the gate is written in `rustqual.toml` and
has a row below; none is left to rustqual's default. Most rows keep the default rustqual
documents at 1.8.3, now as a choice, and three depart from it: the file length and the
two limits on test code. They were chosen before the solver and not tuned to the tree,
which passes them all.

The four clippy can also check carry the same number in `clippy.toml`, but the two tools
do not count alike. rustqual measures a function from its opening brace to its closing
one, comments and blank lines included, where clippy's `too_many_lines` skips both; so
rustqual fires first on a commented function. rustqual also keeps no metrics for a
function with no logic and no call to the crate's own functions, so it never checks
that function's length, and there clippy is the only guard.

| Guardrail | Value | Why this number |
| --- | --- | --- |
| Cognitive complexity per function | 15 | The default of the metric's originator, SonarSource, and rustqual's; clippy's is 25. The highest on the tree is 8. |
| Cyclomatic complexity per function | 10 | McCabe's own recommended ceiling for the measure, and rustqual's default. |
| Nesting depth | 4 | rustqual's default. Clippy counted one block at five, inside a local `impl` in a test helper; the `impl` moved to the test module's level rather than the limit moving. |
| Function length | 60 lines | rustqual's default: a function that fits on one screen. Clippy's 100 is too loose to shape anything; the longest function on the tree is 22 lines. |
| Test function length | 100 lines | Looser than production on purpose: a table of cases or a grid literal is long without being complex. rustqual's default would hold tests to 60. |
| Recursion | forbidden | rustqual's default, kept as a choice: a recursive call in a function with logic is an IOSP finding. A search keeps an explicit stack, which the step budget of invariant 2 then bounds directly. |
| Closures, iterator chains and `?` | not counted as logic | rustqual's defaults. All three are idiomatic Rust, and IOSP counting them would push the code away from the idiom. |
| Parameters | 5 | rustqual's default. A sixth is usually a struct waiting to be named; clippy's default is 7. |
| LCOM4 per struct | 2 | rustqual's default: one separable cluster of methods is tolerated before cohesion counts against a struct. |
| Fields and methods per struct | 12 and 20 | rustqual's defaults, the sizes over which the SRP score weighs a struct more heavily. |
| Fan-out per struct | 10 | rustqual's default: the count of other types a struct's methods reach, an input to the SRP score. It is not the per-module fan-out below. |
| SRP score per struct | 0.6 | rustqual's default, with its default weights of 0.4 for LCOM4, 0.25 for fields, 0.15 for methods and 0.2 for fan-out. A struct at or over it is reported. |
| File length | 500 code lines | Counted before the first `#[cfg(test)]`, without comments, doc comments or blank lines. Rust has no norm here and clippy no lint; a Rust module keeps a type beside its impls, so rustqual's 300 would force splits that serve the tool. The longest file on the tree is 110. |
| Test file length | 1,000 code lines | Applies to files under `tests/` and to test-only modules. Looser for the reason the test function limit is. |
| Independent clusters per file | 2 | rustqual's default: a file is reported at three groups of private functions, of five statements or more each, that never call one another. |
| Instability of a module something imports | 0.8 | rustqual's default. A module nothing imports is exempt, so the outer modules pass at 1.0; the cap guards the modules others lean on. |
| Fan-in and fan-out per module | 15 and 12 | rustqual's defaults. With ten modules planned, fan-out binds only once the crate has more. |
| Stable Dependencies Principle | on | rustqual's default: a module may not depend on one less stable than itself. |
| Module boundaries | the layering table | The table is the rule; each pattern's `reason` is its row. |
| Suppression budget | 5% of functions | rustqual's default (`max_suppression_ratio`). Small enough that a suppression stays an exception. |
| Headroom of a suppression pin | 10% | rustqual's default (`pin_headroom`). A suppression that names a limit, as `file_length=600` does, is itself a finding when the limit sits more than 10% above the value it covers, so a pin cannot absorb later growth. |

Two of rustqual's checks are switched off because another gate owns the rule. Its check
on `unsafe` belongs to `unsafe_code = "forbid"`, and its check on `unwrap`, `expect` and
`panic!` belongs to clippy's `unwrap_used`, `expect_used` and `panic`. Left on, a
justified `#[expect]` on one of those lints would need a second suppression in rustqual's
syntax, charged to the budget above.

**`tools-source.txt`, the one exception to the binary-only rule.** rustqual publishes no
release binary, for any of its versions, and is on neither conda-forge nor Homebrew. So
`tools.txt` stays binary-only and unchanged, and a second list, `tools-source.txt`,
holds `rustqual@1.8.3`. `just install-tools` reads it in a second pass that allows
cargo-binstall's compile strategy; cargo-binstall passes `--locked` through to
`cargo install`, so the build takes the versions rustqual's own lockfile names.
cargo-binstall tries a signed cargo-quickinstall build first and would take one that
appeared for the pin; `--only-signed` makes the signature a requirement rather than a
check made when one exists, and still leaves the source build. The pass disables the `crate-meta-data` strategy, which
cargo-binstall would otherwise try before either: it takes a release asset from the
crate's own repository, which rustqual would publish unsigned, so a release appearing
upstream cannot bypass both the signature and the registry checksum. Compiling is
acceptable for this one tool because it is paid once per worktree (38 seconds on the
maintainer's machine) and, in CI, once per pin by each of the five gate jobs, in
parallel, on the cold cache; `.tools` is then cached on the hash of both lists. A line
joins this list only on a decision record that accepts the same cost.

**The pin** is held at 1.8.3 until the solver has landed under it, then bumped by hand,
one release at a time, with `just check` run on each. Nothing bumps a tool pin
automatically today. `just metrics` fails unless the rustqual it finds reports the pinned
version, so a missing `.tools/bin/rustqual` cannot fall through to another on `PATH`.

## Consequences

The ones that hurt.

- **A young tool by one author.** rustqual was five months old at 1.8.3, with eight minor
  versions since 1.0. A bump may rename a rule, change a default or tighten a heuristic,
  and each bump is then a gate failure to triage rather than a report to read. The pin
  defers that cost; it does not remove it.
- **IOSP cannot be switched off.** It is the author's own dimension, it has no `enabled`
  key, and it reports nothing on this tree today. If the solver's natural shape offends
  it, the gate fails on an opinion the project never adopted. One case is known and
  chosen: recursion is a finding, so the solver's search keeps an explicit stack. An
  IOSP suppression is one line with a reason, `// qual:allow(iosp) reason: "..."`, which
  is the form `AGENTS.md` permits. But the suppression budget is 5% of the function count, one line today, and
  it charges `#[allow]` and `qual:allow` lines while `#[expect]` goes free.
- **A source build at bootstrap.** `just initialize` compiles a Rust program, which needs
  a toolchain and the network. On a cold CI cache all five gate jobs build it in
  parallel.
- **A green run can hide a dead section.** The probe proves the architecture rules are
  live. Nothing proves the complexity, SRP and coupling sections are running with the
  numbers written; a green gate would look the same if one had quietly stopped
  measuring.
- **Three blind spots in the boundary rules.** A path relative to the module
  (`super::super::solver`) names no `crate::` prefix, an alias of the crate
  (`use crate as root;` then `root::solver`) replaces the prefix, and a re-export in
  `lib.rs` hides the module it came from. [Layering](../explanation/layering.md) says what guards each.

## What would reopen this

**The exit.** If two consecutive bumps of the rustqual pin each need a suppression or a
threshold change, `metrics` leaves `just check` for an advisory job on the pattern of
`audit.yml`, and the boundary rules alone are kept as a test through archaven, a
development dependency.

ast-metrics is the other alternative, installed by a checksummed download as the Allium
checker is: it ships release binaries, and it would replace rustqual for complexity and
coupling if rustqual were dropped.

An upstream `enabled` key for IOSP would remove the second consequence above. rustqual
publishing release binaries, or arriving on conda-forge, would move its pin into
`tools.txt` or `pyproject.toml` and empty `tools-source.txt`.

## Related pages

- [Decision 0009: The Rust quality gate](0009-rust-quality-gate.md)
- [Quality gates](../reference/quality-gates.md)
- [Layering and dependency direction](../explanation/layering.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
