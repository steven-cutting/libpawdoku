# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- A Cargo workspace with `crates/pawdoku` as its one member, and `rust-toolchain.toml`
  pinning Rust 1.98.1 with its components and both WebAssembly targets.
- The Apache-2.0 licence, in `LICENSE` at the root, with no per-file headers.
- A `Justfile` whose `just check` runs seventeen read-only gates and proves the worktree
  unchanged, `just fix` as the one mutating recipe, and a pre-commit hook run by `prek`.
- The GitHub repository, public, with `main` protected by one required check, and
  continuous integration that runs five gate jobs behind that check and audits the
  RustSec advisories weekly.
- The `no_std` crate with `alloc`, holding the randomness boundary and its replay fake,
  under workspace lint tables that forbid unsafe code and turn on clippy pedantic.
- The dotfiles every hook reads, and a pixi environment, pinned by `pixi.lock`, that
  installs every tool the gate runs.
- An agent contract in `AGENTS.md`, with fourteen skills under `.agents/skills/` and thin
  bridges for Claude, Codex and Copilot.
- The seven engine specifications migrated from Pawdoku, with the two explanation pages
  they cite, and an eighth, `board.allium`, for a puzzle as it is being played.
- A handbook of twenty-five pages under `docs/`, held to the documentation contract and
  reachable from its map.
- Eleven architecture decision records, from the engine as a library of its own to pixi
  as the tool manager.

[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/
