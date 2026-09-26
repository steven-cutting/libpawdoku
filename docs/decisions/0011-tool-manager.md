---
title: "Decision 0011: Tool manager"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_tool_manager]
requires: []
---

# Decision 0011: Tool manager

## Context

Decision 0004 keeps the Python hook runner and the shared checkers, and its consequences
named the installers of the day: uv for the Python side, cargo-binstall for the cargo
tools, each with its own lockfile or pin list, beside rustup for the compiler. A fresh
machine installed four tools before the first-run script could start, and the tool pin
list named versions rather than hashes, trusting each tool's GitHub release naming.

The foundation ticket stopped on that trust. taplo 0.10.0 publishes no binstall metadata
and its release archives are in a shape binstall cannot unpack, so the pinned, current
version could not be installed without compiling it, which the install command refuses by
design. The other five tools installed only because binstall's default naming guess
matched their releases.

pixi resolves the same tools from conda-forge, where every one of them except cargo-hack
is packaged at the pinned version, and records the whole environment, Python and the
checkers included, in one hashed lockfile. conda-forge also packages the Rust compiler,
but without the `wasm32v1-none` standard library that proves the core is `no_std`, and
without the llvm tools the coverage floor needs; and only rustup's cargo proxy honours
`rust-toolchain.toml`.

## Decision

pixi owns every tool binary and the Python environment. `pyproject.toml` is the manifest:
its `[tool.pixi.*]` tables pin Python, just, prek, cargo-binstall, cargo-nextest,
cargo-llvm-cov, cargo-deny, cargo-shear and taplo exactly, and pin `biscuit-games-tooling`
as a PyPI git dependency at a release tag. `pixi.lock` is the pin; `pixi install --locked`
is the first thing the first-run script does; `pixi lock --check` is the gate's lockfile
check; the environment lives in the gitignored `.pixi/`. The `Justfile` puts the
environment's `bin` first on every recipe's `PATH`, then `.tools/bin`, and calls every
tool by name; nothing runs through `pixi run` inside a recipe.

rustup keeps the compiler. `rust-toolchain.toml` remains the single owner of the Rust
version, its components and its targets; the pixi environment holds no `rust` package,
and `check-toolchain` keeps refusing any cargo that is not rustup's proxy.

`tools.txt` remains as the escape hatch for a tool conda-forge does not carry. It lists
`cargo-hack` alone; cargo-binstall, itself a pixi dependency, installs it into
`.tools/bin`, beside the Allium checker the tooling package puts there. The hooks that
prek pins (typos, lychee, shellcheck, actionlint, markdownlint-cli2, editorconfig-checker,
ripsecrets) keep their pins in the hook configuration.

## Consequences

Of decision 0004's consequences, these are superseded: uv is not a prerequisite and not a
resolver here; `uv.lock`, `.venv/` and `.python-version` do not exist; CI carries no
`setup-uv` step; no recipe begins with `uv run --frozen`. These stand: the hook runner is
prek, the checkers are the six console scripts of `biscuit-games-tooling` at a pinned
tag, `pyproject.toml` is where the package reads its configuration, and the interim
`runes` sentence in `AGENTS.md` waits for the same follow-up.

Contributors need rustup, pixi and just (any just launches the recipes; the pinned one in
the environment runs every nested call and CI). CI installs pixi with one action, restores
the environment from a cache keyed on the lockfile's hash, and needs one more cache only
for what pixi cannot own. Every pin except the compiler's is a hash in `pixi.lock`, and a
moved pin is one `pixi update` and a committed lockfile.

The cost is a package manager that Rust contributors do not usually carry, needed for
the gate and not for `cargo build`; one environment per worktree, not relocatable; a
version that conda-forge lags on occasionally (cargo-shear is one minor version behind
crates.io today); and cargo-hack still arriving by a second mechanism until conda-forge
packages it.

## What would reopen this

cargo-hack appearing on conda-forge, which empties `tools.txt` and removes cargo-binstall;
that is a pin move, not a new decision. conda-forge shipping the `wasm32v1-none` standard
library and llvm-tools for its Rust package, which would make a pixi-owned compiler
possible and would reopen the question of who owns the Rust pin. pixi's PyPI git
resolution failing for the tooling package, which would return the Python side to uv
without moving the tools. A house-wide move of every Biscuit Games repository to pixi,
which would make the manifest shape a template concern.

## Related pages

- [Decision 0004: Hook runner and checkers](0004-hook-runner-and-checkers.md)
- [Develop locally](../how-to/develop-locally.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
