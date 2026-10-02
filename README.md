# libpawdoku

The classic-sudoku engine behind Pawdoku, a Biscuit Games game: a Rust library.

The engine is a solver, a catalogue of the techniques a person solves with, a difficulty
rating and hints, built on a model of a human solver. Every rule comes from the
specifications in `docs/specs/`, and the code is built from them one module at a time;
today the crate holds the randomness boundary and the first of the rules of sudoku, the
grid's two figures and its value types, and the other modules follow them. The core is
`no_std`, so the same code will run in a browser, in Python and on the command line.

## Prerequisites

Four tools, and nothing else: not uv, not cargo-binstall.

- **rustup**, with no default toolchain needed, because `rust-toolchain.toml` chooses the
  one used here. `~/.cargo/bin` must come ahead of `~/.pixi/bin` on `PATH`.
- **pixi**, 0.81.0 or newer, which installs the pixi environment from `pixi.lock`. Three
  tools are not in it: `just initialize` installs cargo-hack from `tools.txt`, builds
  rustqual from `tools-source.txt` and downloads the Allium checker, all into
  `.tools/bin`, and no lockfile names any of them.
- **just**, from `pixi global install just`. Any just launches the recipes, because the
  pinned one in the environment runs every nested call and CI.
- **gh**, only for the pin-bump commands.

A cargo from `pixi global`, Homebrew or a Linux distribution ignores
`rust-toolchain.toml`, and `just check-toolchain` refuses it.
[Develop locally](docs/how-to/develop-locally.md) has the detail and the remedy.

## Quick start

```console
just initialize
just check
```

`just initialize` installs the pixi environment, the pinned toolchain, and cargo-hack and
the Allium checker into `.tools/bin`. From the primary checkout it also installs the
pre-commit hook, which every worktree shares; in a secondary worktree it says so and
skips that step. It never stages, commits, tags or pushes.

## Check your work

```console
just fix      # the safe automatic repairs; unlike any check, it modifies files
just check    # every gate, read-only, proving the worktree is unchanged
```

`just --list` prints every recipe. Each one is described in
[Commands](docs/reference/commands.md).

## Layout

```text
crates/pawdoku/      The engine; src/random.rs is its one effect boundary
docs/                The handbook
docs/specs/          Nine Allium modules — the source of truth for behaviour
tickets/             The work that set this repository up, one ticket per branch
pyproject.toml       The pixi manifest, pinned by pixi.lock
rust-toolchain.toml  The compiler pin
tools.txt            cargo-hack: conda-forge lacks it, and it installs from a release binary
tools-source.txt     rustqual: conda-forge lacks it, and it has no binary, so it is built
rustqual.toml        The metrics gate's thresholds and module-boundary rules
.pixi/               The pixi environment, ignored by Git
.tools/              cargo-hack, rustqual and the Allium checker, outside any lockfile, ignored by Git
target/              Build output, ignored by Git
```

## Documentation

Start at [the documentation map](docs/README.md).

- [Purpose and scope](docs/project/purpose-and-scope.md) — what the engine is, and is not
- [Make your first change](docs/tutorials/first-change.md) — clone to green gate
- [Architecture](docs/explanation/architecture.md) — how a portable engine with one effect fits together
- [Specifications](docs/explanation/specifications.md) — why behaviour is written down first
- [API reference](docs/reference/api.md) — the rustdoc, and how it is built

Engineering conventions and the agent working agreement are in [AGENTS.md](AGENTS.md).

## How the game will use it

The game will consume the engine through a WebAssembly package that also ships the
specification text, so the game can hold the clauses it restates equal to this
repository's by test. That package, a command line and Python bindings are planned as
sibling crates in this workspace, and spike S04 decides their shape.

## Boundaries

Behaviour is decided in `docs/specs/`, not in code. When the two disagree, the
specification is right and the code is a defect. Changing what the engine does means
changing a specification first.

The crate is `publish = false` and has no release yet; spike S02 decides how the first
one is cut. The library is licensed Apache-2.0, in [LICENSE](LICENSE) at the root, with
no per-file headers.
