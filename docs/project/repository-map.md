---
title: "Repository map"
kind: "project"
audience: [contributor, maintainer, agent]
canonical_for: [repository_layout]
requires: []
---

# Repository map

The repository root is a Cargo workspace with one member today, the engine at
`crates/pawdoku`. The command-line, Python and WebAssembly crates will sit beside it
under `crates/` as siblings, each depending on the engine and none on another.

```text
.
├── Cargo.toml              The workspace: members, shared dependencies, lints, profiles
├── Cargo.lock              The pin for every crate; every gate passes --locked
├── rust-toolchain.toml     The exact toolchain, its components and its two wasm targets
├── tools.txt               Tools conda-forge lacks, from release binaries into .tools/bin
├── tools-source.txt        Tools conda-forge lacks with no release binary, built into .tools/bin
├── rustfmt.toml            Rust formatting
├── clippy.toml             Clippy's thresholds; the lint table itself is in Cargo.toml
├── rustqual.toml           The metrics gate: thresholds and the module-boundary rules
├── taplo.toml              TOML formatting and lint
├── deny.toml               Licences, bans, sources and advisories
├── .config/nextest.toml    The test runner's profiles
├── crates/
│   └── pawdoku/            The engine
│       ├── src/
│       │   ├── lib.rs      The crate root: no_std with alloc, every public item
│       │   ├── board.rs    The board in play: opening, the moves, undo and redo, the check, writing and reopening
│       │   ├── board/      The board's parts: the cell, the note, the move, the journal, the reading, the check, the refusal, the record
│       │   ├── generation.rs  The basic generator: the tiers, the one entry and its refusal
│       │   ├── generation/ The generator's parts: a choice among n, the orders, the solution grid, removal
│       │   ├── guide.rs    The engine map: a page of the API reference with its figures and their styles, compiled only when rustdoc builds
│       │   ├── random.rs   The randomness boundary and its fake
│       │   ├── solver.rs   The solver's two entries, search and solve, and its refusal
│       │   ├── solver/     The solver's parts: candidates, the branch, the search, the result
│       │   ├── sudoku.rs   The rules: the grid's figures and the value types
│       │   └── sudoku/     The rules' parts: the proof, the puzzle, the test fixture
│       │       └── snapshots/  Committed .snap pictures of the puzzle and its errors
│       └── tests/          Integration tests: the API bounds, the boundary, the rules, the solver, the board, the generator
│           └── snapshots/  Committed .snap pictures of public values
├── tests/fixtures/         The metrics probe's fixture, never compiled
├── docs/                   This handbook, plus specs/
│   └── specs/              The Allium modules: behaviour is decided here
├── scripts/                The first-run script
├── .agents/skills/         Canonical agent procedures
├── allium-skill-reference/ Reference material the Allium skills link to
├── .github/                CI, the audit and Pages workflows and the composite setup action
├── AGENTS.md               Engineering conventions and the agent working agreement
├── Justfile                Every supported command
├── pyproject.toml          The pixi manifest: every other tool, Python and prek
├── pixi.lock               The pin for everything pyproject.toml names
└── target/ .pixi/ .tools/ ai_tmp/   Gitignored; never committed
```

## What each part is responsible for

| Directory | Responsibility |
| --- | --- |
| `crates/pawdoku/src/` | Pure behaviour. Given the same input it returns the same output, always; the one effect, randomness, sits behind the trait `random.rs` defines. `sudoku.rs` and `sudoku/` are the rules of `sudoku.allium`: `proof.rs` the proof a puzzle is set from, `puzzle.rs` the puzzle in play, `fixture.rs` what their tests share. `solver.rs` and `solver/` are the search of `solver.allium`: `candidates.rs` a set of digits, `branch.rs` one grid of candidates with the propagation rules and the split, `search.rs` the stack of waiting branches, `result.rs` what a search hands back. `board.rs` and `board/` are the board in play of `board.allium`: `cell.rs` a cell as the board shows it, `note.rs` a cell's marks and the grid of them, `moves.rs` a move as the board hands it out, `journal.rs` the moves as the board keeps them, `reading.rs` the board as it stood after a move, `check.rs` a check, `error.rs` the refusal, `record.rs` a board written down, reopening and its refusal. `generation.rs` and `generation/` are the basic generator of `generation.allium`: `choice.rs` how a draw chooses among `n`, `order.rs` the four orders of removal, `grid.rs` the solution grid and its grid attempts, `removal.rs` the visits and what each decides. Every public item is documented, and its example is a test. |
| `crates/pawdoku/tests/` | Integration tests: the bounds every public type must meet, the randomness boundary through its fake, the rules with a puzzle solved and set from outside, the solver's two entries (`solver.rs`), the board's rules clause by clause (`board_rules.rs`), and a whole game, arbitrary scripts and the record written and reopened, all through the board (`board.rs`), and the generator through its one entry, with the two tests that pin what seeds give (`generation.rs`). `snapshots.rs` holds the snapshot tests of the solver, the proof, the board, the record and the generator, and no test that asserts; the randomness boundary's one snapshot stays in `random.rs`. `snapshots/` holds the committed pictures, for review and change detection. Unit tests sit in the module they test, a stated deviation recorded in decision 0009. |
| `docs/specs/` | The Allium specifications. Behaviour is decided here, not in code. |
| `scripts/` | `initialize.sh`, the first-run script. The two validators, the allium installer and runner, the gate runner and the ripsecrets wrapper are console scripts of `biscuit-games-tooling`, pinned in `pyproject.toml`. |
| `.agents/skills/` | Canonical agent procedures. The two bridge trees, `.claude/skills/` and `.codex/skills/`, point at them and add nothing. |
| `allium-skill-reference/` | Vendored reference material for the Allium skills, kept outside `.agents/` and ignored by every linter (decision 0010). |
| `.pixi/envs/default/bin` | Every tool `pyproject.toml` pins and the tooling package's console scripts, installed from `pixi.lock`. First on `PATH` inside every recipe, and nowhere else. |
| `.tools/bin` | cargo-hack from `tools.txt`, rustqual from `tools-source.txt`, and the Allium checker. On `PATH` inside every recipe, after the pixi environment. |
| `tests/fixtures/metrics-violation/` | A directory shaped like a crate, with no manifest, that breaks every module-boundary rule once. `just metrics` requires rustqual to fail on it; nothing compiles it. |
| `target/` | Cargo's build output. `just coverage` writes its report under `target/llvm-cov/`. |

Which of these may import which is not a matter of taste; see
[Layering and dependency direction](../explanation/layering.md).

## Related pages

- [Architecture](../explanation/architecture.md)
- [Commands](../reference/commands.md)
