---
title: "Quality gates"
kind: "reference"
audience: [contributor, maintainer, agent]
canonical_for: [quality_gate_reference]
requires: []
---

# Quality gates

`just check` runs the gates in the order below and snapshots the worktree between each
one. A recipe that modifies a file Git does not ignore fails the run, because checks are
read-only; an ignored path is outside the snapshot, which is why every build output below
is ignored.

| Order | Gate | Proves |
| --- | --- | --- |
| 1 | `check-toolchain` | `cargo` is rustup's proxy, and the toolchain `rust-toolchain.toml` pins is the active one. |
| 2 | `lock-check` | `Cargo.toml` agrees with `Cargo.lock`, and `pyproject.toml` with `pixi.lock`. |
| 3 | `lint` | The whole hook gate passes over every file. |
| 4 | `fmt-check` | Every Rust file is as rustfmt would write it. |
| 5 | `toml-check` | Every TOML file is as taplo would write it, and passes taplo's lint. |
| 6 | `clippy` | Every lint in the workspace table is clean over every crate target and feature, for the host platform, with warnings as errors. |
| 7 | `metrics` | Every function is within the complexity, length, nesting and parameter thresholds and none that holds logic calls itself, every struct within the cohesion and size thresholds, every file within the length threshold, every module within the coupling thresholds, and no module names one the layering table forbids it; and the probe below proves the boundary rules are live. |
| 8 | `features` | Every feature combination compiles. |
| 9 | `wasm-check` | The core compiles for `wasm32-unknown-unknown`, the target wasm-bindgen uses, and for `wasm32v1-none`, which has no standard library: the proof that the core is `no_std`. |
| 10 | `test-doc` | Every doc example compiles and passes. |
| 11 | `coverage` | Every unit and integration test passes, and line coverage is at or above the floor. Doctests are gate 10's. |
| 12 | `doc` | rustdoc is warning-free under `--cfg docsrs`, intra-doc links and missing docs included. |
| 13 | `deny` | Every dependency that ships has an allowed licence, no banned crate is in that graph, and every source is crates.io. Development dependencies are outside the graph. |
| 14 | `deps-unused` | No crate declares a dependency it does not use. |
| 15 | `check-docs` | The documentation contract holds. |
| 16 | `check-agents` | The agent contract holds. |
| 17 | `check-specs` | Every specification reports an empty `diagnostics` array. |
| 18 | `analyse-specs` | Every specification reports an empty `findings` array too. |
| 19 | `check-clean` | The run changed nothing. |

Gates 4 and 5 also run inside `lint`, as two of its hooks. The duplication is deliberate:
a regression names itself in the list of gates rather than being one line inside `lint`.

Gates 17 and 18 cost the gate something real: the pinned `allium` binary lives in the
gitignored `.tools/bin/`, which is a per-worktree install, so a worktree that has never
run `just initialize` fails `just lint` and `just check` until `just install-allium`
puts one there. The alternative was a gate that skipped itself whenever its tool was
absent, which asserts nothing.
[Decision 0005](../decisions/0005-project-managed-allium-cli.md) is the record.

Neither gate trusts the tool's exit code, because neither exit code means what this
project means by clean. `allium check` exits 0 on an `info` diagnostic —
`allium.field.unused` is one — and `allium analyse` keys its status on findings alone and
ignores diagnostics entirely, so a module that does not parse passes it with the `error`
sitting in the JSON it has just printed. `bg-run-allium` runs the subcommand,
prints its output whole, and asserts what the contract actually says: every module reports
an empty `diagnostics` array and an empty `findings` array. A diagnostic may be waived
only where the checker itself is wrong, on the terms in
[Work with the specifications](../how-to/work-with-the-specs.md); a finding cannot be
waived at all.

Gate 7 does not trust a green run either. rustqual skips its architecture section without
a word when the section is switched off or misconfigured, and exits 0 over a directory that
does not exist; either way it reports every boundary as met. And its version decides what
it reports, so `just metrics` first fails unless `rustqual --version` matches the pin in
`tools-source.txt`, rather than let a missing `.tools/bin/rustqual` fall through to
another on `PATH`. Then it runs rustqual twice. The first run is over `crates/pawdoku`, with warnings as failures. The
second, the probe, is over `tests/fixtures/metrics-violation/`, a directory shaped like a
crate that breaks every boundary rule once, half with a `use` line and half with an inline
`crate::` path, and one from a child module, `src/technique/catalogue.rs`, so the `/**`
arm of the rules' globs is proved beside the `.rs` arm. The probe passes only if rustqual exits 1 there (0 is no findings, 2 a
configuration it could not read) and the rules it names are exactly the `name =` lines of
`rustqual.toml`, each once, so a rule that stopped matching shows up as the one missing
from the list, and a rule that matches more than the fixture breaks shows up twice.
The probe proves the boundary rules alone. Nothing proves the complexity, cohesion and
coupling sections are running with the numbers written, and
[Decision 0013](../decisions/0013-metrics-gate.md) records that as a risk it accepted.
Like gates 17 and 18, gate 7 needs a per-worktree binary: `just install-tools` builds
`.tools/bin/rustqual` from source, and a worktree without it fails the gate.

One check is still deliberately missing from the table. `check-links-online` needs the
network, and a check that can fail because a third party is down is not a gate. It is
listed in [Commands](commands.md).
`audit` is missing for the same reason: it reads the RustSec advisory database, and runs
in a workflow of its own.

## What the hook gate contains

`just lint` runs `.pre-commit-config.yaml` over every file. This is the read-only
configuration, and it is the one installed as the pre-commit hook.

| Hook | Checks |
| --- | --- |
| `fmt-check`, `toml-check`, `deps-unused` | Gates 4, 5 and 14, when a Rust file, a TOML file or a manifest is among the files. Each runs through its `just` recipe: a git hook does not inherit the `Justfile`'s `PATH`, so routing through the recipe is what lets it find the environment's tools, and it keeps each tool's arguments in one place. |
| `validate-docs`, `validate-agents` | The two contracts, so a hook catches them before the aggregate does. |
| `check-specs`, `analyse-specs` | The specifications, through `allium`. Needs the pinned binary; see above. |
| `editorconfig-checker` | Whitespace, line endings, final newlines. |
| `markdownlint-cli2` | Markdown structure. |
| `typos` | Spelling, excluding the lockfiles. |
| `lychee` | Link targets, offline. |
| `shellcheck` | `scripts/initialize.sh`. |
| `actionlint` | Every GitHub Actions workflow, its structure only — see below. |
| `ripsecrets` | Credential material, with its output suppressed so a match is never logged. |
| Builtin `check-*` | Large files, case conflicts, merge markers, JSON, TOML, YAML, private keys, shebangs. |

The four contract and specification hooks run their `bg-*` scripts through
`pixi run --frozen`, because a git hook does not inherit the `Justfile`'s `PATH`, and a
bare `pixi run` could rewrite `pixi.lock` from inside a commit. Clippy is deliberately not
a hook: a cold build over every target blocks a commit for minutes, so it is gate 6 and a
CI step instead.

Third-party hooks are pinned to commit SHAs with a version comment beside each.

One gap is worth knowing about rather than being surprised by. `actionlint` analyses a
`run:` block by handing it to `shellcheck`, and it reports nothing at all when it cannot
find `shellcheck` on its own `PATH`. Under `prek` each hook gets its own environment, so
the `shellcheck` hook one row up is not the one `actionlint` can see, and the shell
embedded in a workflow goes unread. Every `run:` in this repository's workflows and its
setup action is a single line, nearly all of them one `just` recipe, so there is no
embedded shell to miss today. A block of more than a line
added to a workflow deserves `shellcheck` by hand until the gap is closed.

## The mutating counterpart

`.pre-commit-fix.yaml` holds the hooks that write: `cargo fmt`, `taplo fmt`,
`cargo shear --fix`, end-of-file and trailing-whitespace repair, and
`markdownlint --fix`. It is never installed as a hook and runs only from `just fix`.
`cargo clippy --fix` is not among them: it needs a full build and permission to touch a
dirty tree, so it is a line in the `fix` recipe itself.

## In continuous integration

`.github/workflows/ci.yml` runs the same recipes in five gate jobs, each on Ubuntu and
each starting with the repository's composite setup action, which installs the pixi
environment, the toolchain, the tools `tools.txt` and `tools-source.txt` pin, and the hook
environments.

| Job | Runs |
| --- | --- |
| `rust` | `check-toolchain`, `lock-check`, `fmt-check`, `toml-check`, `clippy`, `metrics`, `features`, `test`, `doc`, `deps-unused` |
| `coverage` | `coverage`, then uploads `lcov.info` as an artifact kept for seven days |
| `wasm` | `wasm-check` |
| `deny` | `deny` |
| `documents` | `install-allium`, `lint`, `check-docs`, `check-agents`, `check-specs`, `analyse-specs` |

A sixth job, `check`, needs all five and runs no recipe: it fails when any of them failed,
was cancelled or was skipped. It exists so that branch protection can name one check that
never changes. A gate job added later joins its `needs`, and protection stays as it is.

The mapping is not one to one. `rust` runs `test`, where the gate runs `test-doc` and
`coverage`; `lint` runs in `documents`; and no job runs `check-clean`, since each job
runs its recipes directly rather than through `just check`. Past the composite setup action,
nothing in CI runs a command that does not exist in the `Justfile`.

`.github/workflows/audit.yml` runs `just audit` on every pull request, weekly, and by
hand. It is never a required check: a fetch of the advisory database can fail for reasons
that have nothing to do with the diff.

## On `main`

`main` is protected, and one check, `check`, must pass before a branch merges into it.
That is the aggregate job above, which fails unless `rust`, `coverage`, `wasm`, `deny` and
`documents` all succeeded, so a gate job is added by joining its `needs` and the
protection itself never changes. `audit` is never required.

The branch is not required to be up to date with `main` first, and no review is required —
neither earns its cost on a repository with one author. Force pushes and deletion are
refused. Administrators are not bound by the rule, so the direct push remains available
when it is genuinely wanted; the protection is there to stop an unproved merge, not to stop
the author.

There is no deployment: nothing publishes on a push to `main`.

## Related pages

- [Commands](commands.md)
- [Quality philosophy](../explanation/quality-philosophy.md)
- [Documentation contract](documentation-contract.md)
- [Agent contract](agent-contract.md)
- [Decision 0009](../decisions/0009-rust-quality-gate.md)
- [Decision 0013](../decisions/0013-metrics-gate.md)
