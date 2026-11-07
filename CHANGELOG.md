# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The WebAssembly crate's place in the repository (decision 0020); the crate and the
  npm package keep their own changelog, `crates/pawdoku-wasm/CHANGELOG.md`.
  `just wasm-build` packs `@steven-cutting/pawdoku-wasm` with wasm-pack in
  `--mode no-install` once `just lock-check` has passed, because wasm-pack would
  otherwise rewrite a stale `Cargo.lock`; then `scripts/wasm_pkg.py` adds the
  specification modules under `specs/`, the crate's changelog and an `exports` map.
  `just wasm-test`, gate 20, runs the crate's tests in Node; both first run
  `just wasm-guard`, which stops unless the wasm-bindgen CLI is the version `Cargo.lock`
  resolves. `pyproject.toml` pins wasm-pack, wasm-bindgen-cli and Node. CI's `wasm` job
  runs both recipes and keeps the package as an artifact, and `release-npm.yml`
  publishes it to GitHub Packages on a `pawdoku-wasm-v*` tag, after the release guards.
  `just coverage` now measures `pawdoku` alone, which is what its floor is over.
- Play from a record, `pawdoku::board::{open, play, read, reading_after}`, from
  `board.allium`'s `PlayingFromARecord`: a caller that holds no board hands each call a
  record and one `Operation`, and is handed back a `Played`, the record after it, a
  `BoardView` of the board and, after a check, the `Check` it added; or a
  `FromRecordError`. Each call is the held board's own operations composed, so both ways
  give the same values, and nothing is kept between calls. `BoardView::of` and
  `ReadingAfter::of` give the same values of a held board; `Board` gains no method.
  Under the `serde` feature every value `Playing`, `SearchResult` and the generation
  result expose serialises in one wire shape (snake_case fields, PascalCase enum
  values, an absent digit `null`), and `Operation` also deserialises, tagged by `kind`.
  `generation::generate_from_seed` and `generate_from_stream` give a
  `GenerationResult`, what replays a puzzle and never its solution, or a
  `GenerationRefused`. `Record`'s wire shape is unchanged.
- The release machinery (decision 0019). cargo-release, pinned in `tools.txt`, cuts a
  release on the maintainer's machine through `[workspace.metadata.release]` and
  `pawdoku`'s `[package.metadata.release]`, which moves this file's unreleased entries
  under the new version; it never publishes and never pushes. `release.yml` publishes
  `pawdoku` to crates.io through trusted publishing on a `pawdoku-v*` tag, in the
  `release` environment, after the release guards, a composite action every release
  workflow shares, refuse a tag off `main` or one that disagrees with the manifest. A
  `semver` job in CI runs `just semver`, cargo-semver-checks against the last
  `pawdoku-v*` tag, pinned in `pyproject.toml`; it reports and never blocks before 1.0.
  The crate ships `LICENSE`. "Cut a release" in the maintenance page has the procedure.
- The API reference hosted on GitHub Pages at
  <https://steven-cutting.github.io/libpawdoku/>, republished from `main` on every push
  by a workflow of its own. `just site` builds what is served: `just doc`, and a root
  page that sends a reader to the crate (decision 0016).
- The basic generator, `pawdoku::generation`, from the basic way of `generation.allium`.
  `generate(tier, stream)` makes a puzzle from a `Tier`, one of five construction
  settings, and a stream of draws, and gives the solver's proof of it or a
  `GenerateError`. The same tier and the same draws give the same puzzle, named by
  `GENERATION_VERSION`; it draws from wherever the stream stands. A puzzle can now be
  made, opened as a board and played with this crate and a seed. A tier rates nothing.
- `generation.allium` states the basic generator in full: a solution grid drawn from
  eleven givens through the randomness boundary, givens removed one position at a time
  in the order one of five tiers names, each removal kept only while the solver's
  verdict stays one. The designed generator beside it stays a skeleton (decision 0015).
- The record, `pawdoku::board::Record`, from `board.allium`'s `Recording` and
  `Reopening`. `Board::write` writes a board down at any point as a plain value the
  caller keeps, and `Board::reopen` makes the same board from it: moves, undone moves
  and checks included. A record holds the givens, the moves, how many are undone and
  the checks, and nothing worked out from them or read from the solution. A record that
  does not read back is refused with a `ReopenError`, never a panic. Under the `serde`
  feature a `Record` serialises; a `Board` and a `Puzzle` do not. No promise is made yet
  that a record reopens across versions of the engine.
- The board in play, `pawdoku::board`, from `board.allium`. A `Board` opens from givens
  in one call, owns its puzzle and hands out values of its own: `BoardCell`, `Note`,
  `Move` and `Check`. It places and erases digits, writes and strikes marks, keeps the
  peers' notes up after a placement, takes the latest move back and re-takes it, reads
  back the board as it stood after any move, and answers a check of one cell yes or no.
  A refused operation changes nothing and says why with a `PlayError`.
- The solver, `pawdoku::solver`, from `solver.allium`. `search` gives any givens a
  verdict of none, one or many, the solutions found and the count of guesses; `solve`
  gives the proof a puzzle is set from, or one of three refusals. With it
  `sudoku::WellPosed`, `sudoku::Puzzle`, `Cell`, `Status`, `Grid` and the two errors
  become public, so a puzzle can be made from outside the crate: solve, then set.
- The first engine module, `pawdoku::sudoku`, from `sudoku.allium`: the constants
  `BOX_SIDE` and `SIDE`, and the value types `Position` and `Given`. Inside the crate
  until the solver arrives: the proof value `WellPosed`, and a `Puzzle` set from it with
  its eighty-one cells, placing and erasing, conflicts, and solved as a final state.
  `SIDE` moves there from the crate root, which no longer exports a constant.
- File snapshots through insta for review and change detection, with a randomness-boundary
  picture, a read-only `snapshots-check` gate that rejects orphans, and cargo-insta recipes
  to review and accept intended changes. Snapshot tests prove no specification clause.
- `just plan-spec <module>` prints Allium's JSON test obligations as a suggestion list outside the gate.
- A Cargo workspace with `crates/pawdoku` as its one member, and `rust-toolchain.toml`
  pinning Rust 1.98.1 with its components and both WebAssembly targets.
- The Apache-2.0 licence, in `LICENSE` at the root, with no per-file headers.
- A `Justfile` whose `just check` runs nineteen read-only gates and then proves the
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
- Eighteen architecture decision records, from the engine as a library of its own to a
  record that names its format version.
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

### Changed

- `pawdoku` is publishable: its manifest no longer inherits the workspace's
  `publish = false`, which stays for every crate that writes `publish.workspace = true`.
  `rust-version` no longer follows the toolchain pin; it rises only on purpose, in a
  minor release (decision 0019).
- `board.allium` states play from a record, the contract and surface
  `PlayingFromARecord`: a board opened from givens, an operation put to a record, a
  record read, and the board as it stood after one move of a record, each in a call that
  keeps nothing and hands back the record with what `Playing` shows, the readings after
  a move on asking. It adds no rule: each call is reopening, one of `Playing`'s
  operations and writing, composed (decision 0021).
- Every record names its format version, `pawdoku::board::RECORD_VERSION`, `"record-1"`,
  and `Board::reopen` refuses a record of another version, or of none, with
  `ReopenError::Version`, before any other refusal; no compatibility across versions is
  promised. A record's serialised shape gains a first field, `version`. Under the
  `serde` feature, `RecordVersion` reads a record's version alone, for the bindings
  (decision 0022). `board.allium`'s `config` gains `record_version`, and `Recording`
  the invariant `ARecordNamesItsVersion`.

[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/
