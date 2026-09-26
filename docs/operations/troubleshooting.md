---
title: "Troubleshooting"
kind: "operations"
audience: [contributor, maintainer, operator, agent]
canonical_for: [troubleshooting]
requires: []
---

# Troubleshooting

Symptoms as headings, causes and fixes as bodies.

## `just check` reports that a recipe changed the worktree

The recipe is the defect, not your change. Checks are read-only; anything that writes
belongs in `.pre-commit-fix.yaml` and runs from `just fix`. The report names which paths
moved. Move the offending hook, or add the generated path to the ignore rules.
Here the usual paths are a build or tool directory that escaped the ignore rules —
`target/`, `.tools/` or `.pixi/` — or a rewritten `Cargo.lock` or `pixi.lock`, which
means a recipe resolved dependencies instead of reading the lockfile.

## `docs validation: frontmatter <field> disagrees with the manifest`

Almost always list order. The comparison between `docs/manifest.yml` and a page's
frontmatter is order-sensitive, so `[maintainer, contributor]` fails against
`["contributor", "maintainer"]`. Copy the order from the manifest.

If the field is `title`, check for a stray difference in punctuation — the level-one
heading must match it byte for byte as well.

## `docs validation: not reachable from docs/README.md`

The page exists and is registered, but nothing links to it. Add it to
[the documentation map](../README.md), or to a page that is already reachable.

## `agent validation: unexpected managed file`

Something appeared under `.agents/`, `.claude/` or `.codex/` that is neither a declared
skill nor `.claude/settings.json`. If it is local tool state, add it to `.gitignore` —
the inventory reads Git, so an ignored file is invisible to it. If it is real content, it
belongs in `.agents/skills/` with bridges, or somewhere else entirely.

## `agent validation: must stay a thin pointer to the canonical skill`

A bridge under `.claude/` or `.codex/` has grown content, or its frontmatter has drifted
from the canonical skill. Regenerate it: the canonical frontmatter verbatim, one blank
line, then the fixed pointer sentence and nothing else. The body is compared against a
template rather than measured, so a clause of your own fails however short it is —
[Agent contract](../reference/agent-contract.md#what-a-bridge-must-be) carries the text.

## Coverage fails but everything is tested

Distinguish two cases. If a real path is untested, add the test. If the uncovered branch
cannot be reached by any input — a bounds check after a modulo, a fallback after an
exhaustive assignment — delete the branch. Do not lower the threshold.

On a crate this small the denominator is small too, so one untested branch in
`random.rs` is enough to breach the floor on its own. Test it or delete it; the number
stays where it is.

## `cargo resolves to <path>, not rustup's proxy`

`just check-toolchain`, the first gate, found a `cargo` on `PATH` that is not rustup's.
A cargo from pixi, Homebrew or a distribution ignores `rust-toolchain.toml` and builds
with whatever version it is, so the gate stops before anything compiles. Put
`~/.cargo/bin` ahead of the other directory on `PATH`, or remove the other cargo — for a
global pixi install, `pixi global uninstall rust` — until `command -v cargo` prints the
proxy in `~/.cargo/bin`.

## `rustup is not installed`

The same gate, one step earlier: there is no `rustup` on `PATH` at all. Install rustup
as [Develop locally](../how-to/develop-locally.md) describes, then run
`just install-toolchain`.

## A recipe cannot find `allium`, `prek` or a `bg-*` script

The pixi environment under `.pixi/` and the binaries under `.tools/bin/` are installed
per worktree, and a fresh worktree has neither. `pixi install --frozen` restores the
environment, `just install-tools` the `tools.txt` binaries and `just install-allium` the
`allium` checker; `just initialize` does all three. A worktree that was renamed or moved
needs `pixi install --frozen` again even though `.pixi/` is there: the `bg-*` scripts'
shebangs name the Python at the worktree's old absolute path.

## `just lock-check` or `pixi install --locked` fails on `pixi.lock`

`pixi.lock` is stale against `pyproject.toml`: a pin moved in the manifest and the
lockfile was not solved again. Run `just lock`, read the diff, and commit the lockfile.

Look at `git status` first. On a stale lock, `lock-check` fails and also leaves
`pixi.lock` rewritten from a solve restricted to the packages already on this machine,
which can pin older versions than the channels offer. Discard that with
`git restore pixi.lock` before running `just lock`.

## `cargo` reports that `Cargo.lock` needs to be updated but `--locked` was passed

Every recipe passes `--locked`, so a manifest edit that the lockfile does not reflect
fails every build until the lockfile is solved again. Run `just lock`, read the diff, and
commit `Cargo.lock` with the manifest. A new dependency is a decision before it is an
edit: [Decision 0007](../decisions/0007-dependency-policy.md) says what one has to
justify.

## `just lint` clones or downloads

Every hook environment should already be prepared, so `lint`, and `just check` around
it, never reaches the network. If it does, prek's cache was cleared, or this is a
secondary worktree whose primary checkout never ran `just initialize`. Run
`just install-hooks` from the primary checkout; it prepares every hook environment and
warms the link checker. `just initialize` is where the network belongs, and `just check`
must not need it.

## A `stable` toolchain appears that nobody installed

`just install-hooks` runs the link checker once to warm it, and the lychee hook's
checkout pins `stable`: its script installs lychee through cargo-binstall, and rustup
installs the toolchain that checkout names on the way. It is harmless and costs disk.
`rustup toolchain uninstall stable` removes it, until the next `just install-hooks`.

## The ripsecrets hook fails to install: the package believes it is in a workspace

`PREK_HOME` points inside a Cargo workspace, this checkout included. ripsecrets is a Rust
hook, built in prek's cache, and a build under a workspace root is taken for a member of
it. Point `PREK_HOME` at a directory outside any checkout, as CI does with the runner's
temporary directory, and run `just install-hooks` again.

## `just wasm-check` cannot find `core` for `wasm32v1-none`

The target is not installed. `just install-toolchain` installs every target
`rust-toolchain.toml` names, through rustup; a cargo that is not rustup's cannot, which
`just check-toolchain` would also have reported.

## `just install-tools` refuses to build from source

`--disable-strategies compile` is deliberate: a tool that cannot be installed from a
release binary fails loudly rather than compiling at bootstrap. The pin in `tools.txt`
names a version with no release binary for this host; choose one that has one. To see
which cargo-binstall is running, ask it with `cargo binstall -V -v`: `cargo --list -v`
names the last one on `PATH`, not the one cargo runs.

## prek behaves as if a pin had not moved

A hook whose `rev` changed still runs its old environment. Clear prek's cache with
`pixi run --frozen prek cache clean`, then run `just install-hooks` again to prepare
every environment from the current configuration.

## Related pages

- [Test and debug](../how-to/test-and-debug.md)
- [Quality gates](../reference/quality-gates.md)
- [Maintenance](maintenance.md)
