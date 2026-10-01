# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A Cargo workspace with `crates/pawdoku` as its one member, and `rust-toolchain.toml`
  pinning Rust 1.98.1 with its components and both WebAssembly targets.
- The Apache-2.0 licence, in `LICENSE` at the root, with no per-file headers.
- A `Justfile` whose `just check` runs eighteen read-only gates and then proves the
  worktree unchanged, `just fix` for the safe automatic repairs, and a pre-commit hook
  run by `prek`.
- The GitHub repository, public, with `main` protected by one required check, and
  continuous integration that runs five gate jobs behind that check and audits the
  RustSec advisories weekly.
- The `no_std` crate with `alloc`, holding the randomness boundary and its replay fake,
  under workspace lint tables that forbid unsafe code and turn on clippy pedantic.
- The dotfiles every hook reads, and a pixi environment, pinned by `pixi.lock`, that
  installs the gate's other tools, with cargo-hack, rustqual and the Allium checker
  outside it in `.tools/bin`.
- An agent contract in `AGENTS.md`, with fourteen skills under `.agents/skills/` and thin
  bridges for Claude, Codex and Copilot.
- The seven engine specifications migrated from Pawdoku, with the two explanation pages
  they cite, and an eighth, `board.allium`, for a puzzle as it is being played.
- A handbook of twenty-five pages under `docs/`, held to the documentation contract and
  reachable from its map.
- Fourteen architecture decision records, from the engine as a library of its own to the
  board's import of the solver.
- A `metrics` gate, seventh in `just check`: rustqual, built from source from its pin in
  `tools-source.txt`, holds thresholds for complexity, cohesion and coupling and the
  module boundaries of the layering table, with a probe that proves the boundary rules
  live. Clippy gains thresholds for cognitive complexity, nesting depth, function length
  and parameter count.
- The board may import the solver: `board.allium` gains a second `use` line, the layering
  table follows it, and the boundary rule is renamed `board_imports_sudoku_and_solver`
  and no longer forbids `crate::solver`.
- Comment sentences changed in `board.allium`, `sudoku.allium` and `solver.allium`: why
  the solver is imported, the proof a puzzle may be set with, undo and redo no longer
  said to run no rule, and a search's status, read from being handed the search.

[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/
