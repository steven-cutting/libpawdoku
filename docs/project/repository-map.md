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
├── tools.txt               The one tool conda-forge lacks, installed into .tools/bin
├── rustfmt.toml            Rust formatting
├── clippy.toml             Clippy's thresholds; the lint table itself is in Cargo.toml
├── taplo.toml              TOML formatting and lint
├── deny.toml               Licences, bans, sources and advisories
├── .config/nextest.toml    The test runner's profiles
├── crates/
│   └── pawdoku/            The engine
│       ├── src/
│       │   ├── lib.rs      The crate root: no_std with alloc, every public item
│       │   └── random.rs   The randomness boundary and its fake
│       └── tests/          Integration tests: the API bounds and the boundary
├── docs/                   This handbook, plus specs/
│   └── specs/              The Allium modules: behaviour is decided here
├── scripts/                The first-run script
├── .agents/skills/         Canonical agent procedures
├── allium-skill-reference/ Reference material the Allium skills link to
├── .github/                CI, the audit workflow and the composite setup action
├── AGENTS.md               Engineering conventions and the agent working agreement
├── Justfile                Every supported command
├── pyproject.toml          The pixi manifest: every other tool, Python and prek
├── pixi.lock               The pin for everything pyproject.toml names
└── target/ .pixi/ .tools/ ai_tmp/   Gitignored; never committed
```

## What each part is responsible for

| Directory | Responsibility |
| --- | --- |
| `crates/pawdoku/src/` | Pure behaviour. Given the same input it returns the same output, always; the one effect, randomness, sits behind the trait `random.rs` defines. Every public item is documented, and its example is a test. |
| `crates/pawdoku/tests/` | Integration tests: the bounds every public type must meet, and the randomness boundary through its fake. Unit tests sit in the module they test, a stated deviation recorded in decision 0009. |
| `docs/specs/` | The Allium specifications. Behaviour is decided here, not in code. |
| `scripts/` | `initialize.sh`, the first-run script. The two validators, the allium installer and runner, the gate runner and the ripsecrets wrapper are console scripts of `biscuit-games-tooling`, pinned in `pyproject.toml`. |
| `.agents/skills/` | Canonical agent procedures. The two bridge trees, `.claude/skills/` and `.codex/skills/`, point at them and add nothing. |
| `allium-skill-reference/` | Vendored reference material for the Allium skills, kept outside `.agents/` and ignored by every linter (decision 0010). |
| `.pixi/envs/default/bin` | Every tool `pyproject.toml` pins and the tooling package's console scripts, installed from `pixi.lock`. First on `PATH` inside every recipe, and nowhere else. |
| `.tools/bin` | cargo-hack from `tools.txt`, and the Allium checker. On `PATH` inside every recipe, after the pixi environment. |
| `target/` | Cargo's build output. `just coverage` writes its report under `target/llvm-cov/`. |

Which of these may import which is not a matter of taste; see
[Layering and dependency direction](../explanation/layering.md).

## Related pages

- [Architecture](../explanation/architecture.md)
- [Commands](../reference/commands.md)
