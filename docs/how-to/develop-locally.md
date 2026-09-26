---
title: "Develop locally"
kind: "how-to"
audience: [contributor, maintainer, agent]
canonical_for: [local_development]
requires: []
---

# Develop locally

## Prerequisites

| Tool | Why |
| --- | --- |
| `rustup` | Installs the exact toolchain `rust-toolchain.toml` pins, and provides the only cargo the gate accepts. `~/.cargo/bin` must come ahead of `~/.pixi/bin` on `PATH`, or run `pixi global uninstall rust`. |
| `pixi`, 0.81.0 or newer | Installs every other tool, and the Python environment the hook gate runs on, from `pixi.lock` into the gitignored `.pixi/envs/default`. |
| `just` | The task runner, and the only supported interface to the checks. `pixi global install just` provides one. Any `just` launches the recipes, because the pinned one in the environment runs every nested call and CI; `pixi run --frozen just <recipe>` is the escape hatch when no global `just` is installed. |
| `gh` | Only for the pin-bump commands in [Maintain dependencies](maintain-dependencies.md). |

Nothing else: not uv, not cargo-binstall. `rust-toolchain.toml` names the exact
toolchain, its components and its two wasm targets, and rustup installs them the first
time cargo runs here. `pyproject.toml` names every other tool and `pixi.lock`
pins it, with `tools.txt` the one-line exception for what conda-forge lacks.
[Configuration](../reference/configuration.md) has each figure; this page states none.
The page covers Ubuntu and macOS; Windows gains a section the day the command-line crate
brings it into CI.

## First run

Two tools are installed once per machine, and everything after them comes from this
repository's pins.

The first is rustup. No default toolchain is needed: `rust-toolchain.toml` chooses the
toolchain here, and prek builds its one Rust hook with the pinned toolchain rustup has
already installed. The one-time command is upstream's:

```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --profile minimal
```

The `stable` default it sets is only for use outside this repository; here
`rust-toolchain.toml` overrides it, and `--default-toolchain none` works as well. Any
route that ends with `rustup` on `PATH` and `command -v cargo` printing
`~/.cargo/bin/cargo` is equivalent.

The second, and last, is pixi, with the installer its documentation gives:

```console
curl -fsSL https://pixi.sh/install.sh | sh
```

Copy the command from pixi's installation page on the day rather than from here if the
two differ. `requires-pixi` in `pyproject.toml` is the floor, and a pixi below it refuses
the manifest. These two are the only curl-pipes in this handbook. Then
`pixi global install just`, if nothing else on the machine provides a just.

One hazard catches almost everyone once. A cargo from `pixi global`, Homebrew's `rust`
formula or a Linux distribution ignores `rust-toolchain.toml` and builds with whatever
release it happens to be, so the lint set and the target list silently differ from CI's.
`just check-toolchain` refuses it before anything else runs, with one of two lines:

```text
rustup is not installed; see docs/how-to/develop-locally.md
cargo resolves to /home/you/.pixi/bin/cargo, not rustup's proxy
```

The remedy is `~/.cargo/bin` ahead of `~/.pixi/bin` on `PATH`, or
`pixi global uninstall rust`. The repository's own pixi environment holds no cargo and is
not the hazard; a `pixi global` rust is.

```console
just initialize
```

This installs the pixi environment from `pixi.lock`, installs the pinned toolchain,
refuses a cargo that is not rustup's, installs cargo-hack from `tools.txt` into
`.tools/bin` and then the Allium checker, syncs both lockfiles, normalises formatting,
and installs the pre-commit hook with every hook environment. Run it once per clone. It
needs the network for the pixi environment, the toolchain, cargo-hack, the Allium
checker, `cargo fetch` and the hook clones. It never stages, commits, tags or pushes.

The hook is installed from the primary checkout only, because every worktree shares one
`.git/hooks`. In a secondary worktree `just initialize` says so and skips it, and the
first `just lint` there clones and builds any hook environment the primary checkout has
not already built, which needs the network too. `just check` runs `lint` as its third
gate and adds no network need of its own, so once the hook environments exist the whole
gate is offline.

`.pixi/envs/default/bin` and `.tools/bin` are both gitignored, and the `Justfile`
prepends them to `PATH`, in that order, for every recipe. They are not on your shell's
`PATH`, so a bare `cargo nextest` from a shell needs
`PATH="$PWD/.pixi/envs/default/bin:$PWD/.tools/bin:$PATH"`, a recipe, or
`pixi run --frozen cargo nextest`.

## Every day

```console
just build
just test
just doc
```

`just build` compiles the workspace with every feature. `just test` runs the unit and
integration tests through nextest, then the doctests. `just doc` builds the API
documentation into `target/doc/`, with warnings as errors, as CI does.

## Before handing work back

```console
just fix       # formats and applies the safe automatic repairs
just check     # the whole gate, read-only
```

Of the two, only `just fix` modifies files; outside the pair, `just lock` and
`just format` write too. Every check is read-only, and `just check` proves it by comparing
the worktree before and after each recipe.

## Keeping the workspace current

After pulling, re-sync so the installed dependencies match the lockfiles:

```console
just sync
```

It installs a moved `pixi.lock` exactly as committed and fetches what `Cargo.lock` names.
After `tools.txt` has moved, run `just install-tools` as well; it skips a version that is
already there.

## Related pages

- [Commands](../reference/commands.md)
- [Test and debug](test-and-debug.md)
- [Troubleshooting](../operations/troubleshooting.md)
- [Configuration](../reference/configuration.md)
