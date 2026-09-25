---
id: S04
title: "Spike: bindings and CLI groundwork, pawdoku-cli, pawdoku-py, pawdoku-wasm"
status: open
depends_on: [T11, T06]
parallel_with: []
branch: ticket/s04-bindings-and-cli
estimated_size: M
---

# S04: Spike: bindings and CLI groundwork, pawdoku-cli, pawdoku-py, pawdoku-wasm

## Context

CONVENTIONS.md §1 decision 1 names three sibling crates that join the workspace later:
`crates/pawdoku-cli`, `crates/pawdoku-py` (pyo3 and maturin) and `crates/pawdoku-wasm`
(wasm-bindgen). Every build decision was taken to keep them cheap, and decision 0008
(`docs/decisions/0008-no-std-core.md`, T09) is specified to record what the core
guarantees:
no clock, threads, filesystem or environment; randomness through the boundary
`crates/pawdoku/src/random.rs` defines, a seeded stream a caller supplies; every public
type `Send + Sync + 'static`, `Clone`, `Debug` and `#[non_exhaustive]`; `thiserror`
errors with stable `Display` text, because pyo3 and wasm-bindgen build their exceptions
from it (§7 invariant 3); the core checked on `wasm32-unknown-unknown` and
`wasm32v1-none` under every feature combination; `panic = "unwind"` kept because pyo3
turns panics into exceptions. T02's `deny.toml` is specified to ban `getrandom` and
`rand` with `wrappers = ["pawdoku-cli"]`, so the CLI is the one crate that may source
entropy;
`[workspace.lints.rust] unsafe_code = "forbid"` cannot be lowered by a member, so a
bindings crate whose macro output trips a workspace lint declares its own `[lints]`
table instead of `workspace = true`.

Why the wasm crate matters most: Allium has no cross-repository import (§1 fact 6), so
G will restate the clauses it needs and hold them equal to this repository's text by
test, as it holds the platform's figures today in `tests/platformSpecs.test.ts`. Until
the wasm package ships `docs/specs/` inside it, that text is shared truth across two
repositories and drifts silently (§13); §9's last paragraph hands the question here.
The game is a Svelte static site with no server (G's `README.md` at `78d03cdf` lines
7-9), so the target is `wasm-pack build --target web`, and the house npm registry is
GitHub Packages for the `@steven-cutting` scope (G's `.npmrc`, one line), where a
`@steven-cutting/pawdoku-wasm` package would live.

Facts to start from, each verified at execution: pyo3 with `abi3-py311` builds one
wheel per platform rather than per Python; maturin reads a `pyproject.toml` beside the
crate and can generate its own workflow (`maturin generate-ci github`), whose wheel
matrix belongs in S02's `release.yml`; a root `pyproject.toml` already exists as a
virtual project (decision 0004), so uv sees two projects, and the bindings crates live in
this workspace (settled with the maintainer in D01, as decision 0001 assumed), so that is
a real question for step 3 (b), not a hypothetical. wasm-bindgen with
`serde-wasm-bindgen` behind the core's `serde` feature turns public types into plain
JavaScript objects; `console_error_panic_hook` routes a panic to the console;
wasm-pack writes the `package.json` it publishes, so shipping `specs/` is a step after
the build (copy the seven modules into `pkg/specs/`, add them to `files`). clap for the
CLI and cargo-dist for its binaries; `windows-latest` joins the `rust` job the day the
CLI exists (§10).

Read first: CONVENTIONS.md §1 (decisions 1 and 5, facts 6 and 8), §7 invariants 2 and 3,
§9, §10 (last paragraph), §11; decisions 0007 and 0008;
`docs/explanation/architecture.md` (T08); `crates/pawdoku/src/random.rs` and
`tests/random.rs` (T02); `Cargo.toml` and `deny.toml`; T06's hand-back notes (what G
must restate); G's `tests/platformSpecs.test.ts` at `78d03cdf`.

## Goal

A workspace layout, a paragraph of design per crate, how the randomness boundary
crosses each language boundary, the CI and release additions each crate brings, and a
drafted build ticket per crate, all in the hand-back notes. Nothing is created outside
this file. The starting recommendation: **`pawdoku-wasm` first**, because it ends the
shared-truth risk and the game is waiting for it; the CLI second, as the developer's
tool for running the solver on a grid string; Python last, when a consumer exists.

| Crate | Toolchain | Registry | Randomness | Own `[lints]` | CI addition |
| --- | --- | --- | --- | --- | --- |
| `pawdoku-wasm` | wasm-bindgen, wasm-pack `--target web`, serde-wasm-bindgen behind `serde` | GitHub Packages npm, `@steven-cutting/pawdoku-wasm`, `specs/` inside | JavaScript supplies a seed or the stream; the library's generator, named by `random_version`, is the default | likely, for generated code | a `wasm-pack build` step in `wasm`; publish in `release.yml` |
| `pawdoku-cli` | clap, cargo-dist | GitHub Releases binaries; crates.io optional | the one crate allowed `getrandom`: `--seed`, else OS entropy printed with the result | no | `windows-latest` in `rust`; cargo-dist's workflow |
| `pawdoku-py` | pyo3 `abi3-py311`, maturin, uv | PyPI, trusted publishing | Python supplies a seed; the fake is exposed for tests | likely, for pyo3's macros | maturin's wheel matrix in `release.yml`; uv in CI regardless of D01 |

## Non-goals

- Creating any crate, manifest, `pyproject.toml`, `package.json` or workflow; each
  follow-up does its own.
- Release cadence and lockstep versus independent versions: S02, with this crate list.
- API design beyond what crossing a boundary forces (a seed as an integer, an error
  as a string).
- Changing the core, `deny.toml`, the workspace lints or the toolchain; a need found
  here is a T02 hand-back or a T00 follow-up.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S04-bindings-and-cli.md` | ticket | the design, the answers, the drafted follow-ups; `status: done` |

## Steps

1. Create the worktree on `ticket/s04-bindings-and-cli` from `main` after T11 and T06
   have merged (README.md "How to pick up a ticket").

2. Confirm what the core guarantees today, from the tree, and quote it:

   ```sh
   grep -n -E 'unsafe_code|exhaustive|missing_docs' Cargo.toml
   grep -n -B1 -A1 'wrappers' deny.toml
   grep -n 'panic' Cargo.toml
   grep -n -E 'pub trait|fn ' crates/pawdoku/src/random.rs
   ls docs/specs
   ```

   The forbid line, two `wrappers = ["pawdoku-cli"]` entries, no `panic = "abort"`,
   the trait's signature, seven modules. A guarantee decision 0008 states that the tree
   does not hold is a T02 hand-back.

3. Verify, read-only, against current documentation and record with sources: (a) pyo3
   and maturin versions, `abi3-py311`, and what `maturin generate-ci github` emits;
   (b) how uv treats a second `pyproject.toml` under `crates/pawdoku-py/` beside the
   root virtual project (a `[tool.uv.workspace]` member, or excluded); (c) wasm-bindgen
   and wasm-pack versions, whether `--target web` output is what SvelteKit's static
   adapter imports, and how `pkg/package.json`'s `files` is extended after the build;
   (d) whether GitHub Packages accepts an npm package with provenance from
   `release.yml` and what `packages: write` that needs (never in `ci.yml`, §10);
   (e) clap and cargo-dist versions and whether cargo-dist's generated workflow coexists
   with `release.yml`; (f) whether `forbid(unsafe_code)` at workspace level trips pyo3's
   or wasm-bindgen's generated code on current versions (if so, the crate's own
   `[lints]` drops to `deny` with a reason).

4. Design the randomness crossing per crate, in the trait's actual names from
   `random.rs`: for wasm, a constructor taking a `u64` seed (BigInt or two `u32`) and,
   optionally, a JavaScript function that supplies draws, so a game test can use a
   fake; for Python, the same two shapes as a class and a callable; for the CLI, a
   `--seed` flag and, absent one, entropy from `getrandom` printed alongside the result
   so a run is reproducible. Say how `ExactReplay` (§9) holds across all three: the
   library's generator, named by `random_version`, is the default in every binding.

5. Write the layout (`crates/pawdoku-wasm/{Cargo.toml,src/lib.rs}`,
   `crates/pawdoku-py/{Cargo.toml,pyproject.toml,src/lib.rs,python/pawdoku/__init__.py}`,
   `crates/pawdoku-cli/{Cargo.toml,src/main.rs}`), the recipes each needs as T00
   follow-ups, the `deny.toml` licence entries the new trees bring (T02 hand-back), and
   the pages that change (`docs/explanation/architecture.md`,
   `docs/project/repository-map.md`, `docs/reference/commands.md`: T07 and T08).

6. Write G's side for the wasm crate: the npm pin, a test that reads `specs/*.allium`
   from `node_modules` and holds G's restated clauses equal, closing T06's hand-back
   and the shared-truth note in `docs/explanation/specifications.md`. A G ticket,
   separately authorised; draft its title and files.

7. Draft three follow-up build tickets, ids `T15` (wasm), `T16` (CLI), `T17` (Python),
   each with files, the T00 and T02 hand-backs it depends on, the CI change, the
   release change (S02's `release.yml`), and its proof.

8. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (f) are recorded with sources and dates; step 2's quotes match the
  tree.
- The randomness crossing is written per crate in the trait's actual names.
- The table is completed and one order of adoption is recommended with the reason.
- Three follow-up drafts and one G ticket draft exist in the hand-back notes.
- `git status --porcelain` on the ticket branch lists only this file.

## Verification

```sh
grep -c 'wrappers' deny.toml
grep -n 'unsafe_code' Cargo.toml
cargo hack check -p pawdoku --feature-powerset --target wasm32-unknown-unknown --locked 2>&1 | tail -1
git -C /Users/scutting/projects/pawdoku show 78d03cdf:.npmrc
git status --porcelain
```

Expected: `2`; `unsafe_code = "forbid"`; a finished check; the one-line scope registry;
one line naming this file.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the CLI reads the grid-string format `sudoku.allium` describes or a JSON
  shape shared with the wasm crate; a specification question for `sudoku.allium`'s
  open points, not decided here.
