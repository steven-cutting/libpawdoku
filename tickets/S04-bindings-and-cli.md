---
id: S04
title: "Spike: bindings and CLI groundwork, pawdoku-cli, pawdoku-py, pawdoku-wasm"
status: done
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
matrix belongs in S02's `release.yml`; maturin is a pixi dependency in the root
`pyproject.toml` when the crate lands (decision 0011), and that root file is already the
pixi manifest, so how pixi treats a second `pyproject.toml` under `crates/pawdoku-py/`
(pixi-build, or a separate manifest pixi never reads) is a real question for step 3
(b), not a hypothetical, because the bindings crates live in this workspace (settled
with the maintainer in D01, as decision 0001 assumed). wasm-bindgen with
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
| `pawdoku-py` | pyo3 `abi3-py311`, maturin through pixi | PyPI, trusted publishing | Python supplies a seed; the fake is exposed for tests | likely, for pyo3's macros | maturin's wheel matrix in `release.yml`; pixi and maturin in CI regardless of D01 |

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
| `tickets/README.md` | ticket index | S04's row set to `done` (added in the first review round; see Deviations) |

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
   (b) how pixi treats a second `pyproject.toml` under `crates/pawdoku-py/` beside the
   root manifest (a pixi-build package, a workspace member, or a file pixi never reads
   and only maturin does); (c) wasm-bindgen
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

Run on 2026-09-28 (US Pacific) in the Supacode worktree for this ticket, on branch
`S04-bindings-and-cli` (see Deviations), from `main` at `c1a1415` after S03 had merged.
Commands stamped 2026-09-29 UTC belong to the same run. No tracked file changed except
this one. The worktree already held the ignored `.pixi/` and `.tools/` from an earlier
`just initialize`. Outside it, nothing was installed: the three probe crates for question
(f) were built in the session's scratch directory, and their dependencies went into
cargo's own registry cache.

The maintainer authorised these network uses on the day, each read-only: documentation
(the pyo3, maturin, pixi, wasm-bindgen, wasm-pack, Vite, npm, GitHub, cargo-deny,
cargo-dist and getrandom pages named below), the crates.io, PyPI, npm and anaconda.org
registry APIs, `pixi search` against conda-forge, `gh api` reads, and the cargo fetches
the probe crates needed. Every host used is named beside the fact it supports.

The worktree before any step:

```text
$ git rev-parse --short HEAD origin/main
c1a1415
c1a1415
$ git status --porcelain
$ git rev-parse 'HEAD^{tree}'
be9d3e35fed4951ae1aadb6f9762c7ff7de56290
```

**Step 1.** The worktree existed before the ticket ran (Deviations).

**Step 2. What the core guarantees today: every quote matches the tree, with one count
wrong in the ticket.** The five commands, run at `c1a1415`:

```text
$ grep -n -E 'unsafe_code|exhaustive|missing_docs' Cargo.toml
24:unsafe_code = "forbid"
26:missing_docs = "warn"
58:exhaustive_enums = "warn"
59:exhaustive_structs = "warn"
$ grep -n -B1 -A1 'wrappers' deny.toml
40-deny = [
41:  { crate = "getrandom", reason = "core takes a seed; only the CLI may source entropy", wrappers = [
42-    "pawdoku-cli",
43-  ] },
44:  { crate = "rand", reason = "same", wrappers = [
45-    "pawdoku-cli",
$ grep -n 'panic' Cargo.toml
48:panic = "warn"
77:# panic stays "unwind": pyo3 turns panics into exceptions and needs it.
$ grep -n -E 'pub trait|fn ' crates/pawdoku/src/random.rs
82:pub trait RandomStream {
98:    fn next_draw(&mut self) -> Result<f64, RandomError>;
111:    fn index(&self) -> u64;
155:    pub const fn new(seed: u64) -> Self {
246:    pub fn new(draws: alloc::vec::Vec<f64>) -> Result<Self, RandomError> {
$ ls docs/specs
board.allium      generation.allium     lapse.allium  solver.allium  technique.allium
effort.allium     human-solving.allium  reach.allium  sudoku.allium
```

The `random.rs` listing is trimmed to the public signatures; the other `fn` lines are
the two implementations, the serde `try_from` and the unit tests. So:

- The forbid line is at `Cargo.toml` line 24, in `[workspace.lints.rust]`, and
  `exhaustive_enums` and `exhaustive_structs` are `warn` in the clippy table, which
  `just clippy` makes errors.
- The two `wrappers = ["pawdoku-cli"]` entries are there; taplo splits each array over
  three lines, which is why the grep shows them that way. `grep -c` counts 2.
- There is no `panic = "abort"`. Line 48 is clippy's `panic` lint, and line 77 is the
  comment that keeps the release profile on `unwind`. Cargo's default is `unwind`, so
  the guarantee is a comment and the absence of a line, not a setting.
- The boundary is `pub trait RandomStream` with `fn next_draw(&mut self) ->
  Result<f64, RandomError>` and `fn index(&self) -> u64`, dyn-compatible;
  `SeededStream::new(seed: u64)` is `const`; `ReplayStream::new(draws: Vec<f64>) ->
  Result<Self, RandomError>` rejects a draw outside `[0, 1)`;
  `RANDOM_VERSION = "seeded-stream-1"`; `RandomError` is `#[non_exhaustive]` with
  `Exhausted { index }` and `OutOfRange { index, value }`, texts pinned by tests.
- **Nine modules, not seven.** T12 added `board.allium` and T19 added
  `generation.allium` after this ticket was written; `docs/explanation/specifications.md`
  line 29 already says "Nine are the library's." Every "seven" in this ticket's Context
  and Steps reads nine below.

Decision 0008 against the tree, guarantee by guarantee: `no_std` with `alloc`
(`lib.rs` lines 9 and 11); forbid through the workspace table (above); both wasm targets
in `rust-toolchain.toml` and `just wasm-check`; `rand` and `getrandom` banned (above);
`Send + Sync + 'static`, `Clone` and `Debug` asserted in `crates/pawdoku/tests/api_bounds.rs`;
`#[non_exhaustive]` on all three public types; the panic lints in the clippy table;
`thiserror` errors with tested `Display` text; features `serde` only (0008 says
`default = []`, the manifest has no `default` key, and the two are the same empty set);
no `usize` in the public API. Every guarantee holds, so there is no T02 hand-back from
this step. Two wording gaps are the record's, not the tree's, and go to T09 (Handed
back): 0008 does not itself say `panic = "unwind"` (only `Cargo.toml` line 77 does), and
its Decision list does not carry the "no clock, threads, filesystem or environment"
sentence this ticket attributes to it (0003 lines 21-24 and `AGENTS.md` invariant 2 do).

**Step 3. The six questions.** Sources were read on 2026-09-28. "docs" marks an official
page, and "source" marks code, a manifest, a template or an API record. The versions
below are the newest non-yanked release on each registry on that day, read from
`https://crates.io/api/v1/crates/<name>` (source), `https://pypi.org/pypi/maturin/json`
(source) and `https://registry.npmjs.org/wasm-pack` (source):

| Crate | Version | Released | MSRV | Licence | Repository |
| --- | --- | --- | --- | --- | --- |
| pyo3 | 0.29.2 | 2026-08-05 | 1.83 | MIT OR Apache-2.0 | `pyo3/pyo3` |
| maturin | 1.15.0 | 2026-08-24 | 1.89 | MIT OR Apache-2.0 | `pyo3/maturin` (PyPI and crates.io agree) |
| wasm-bindgen | 0.2.129 | 2026-09-25 | 1.81 | MIT OR Apache-2.0 | `wasm-bindgen/wasm-bindgen` (moved from `rustwasm`) |
| js-sys | 0.3.106 | 2026-09-25 | 1.81 | MIT OR Apache-2.0 | same tree |
| wasm-bindgen-test | 0.3.79 | 2026-09-25 | 1.81 | MIT OR Apache-2.0 | same tree |
| wasm-pack | 0.15.0 | 2026-05-15 | none stated | MIT OR Apache-2.0 | `wasm-bindgen/wasm-pack` (moved from `drager` and `rustwasm`) |
| serde-wasm-bindgen | 0.6.5 | 2024-02-27 | none stated | MIT | `RReverser/serde-wasm-bindgen` |
| console_error_panic_hook | 0.1.7 | 2021-10-11 | none stated | Apache-2.0/MIT | `rustwasm/console_error_panic_hook` |
| clap | 4.6.7 | 2026-09-14 | 1.85 | MIT OR Apache-2.0 | `clap-rs/clap` |
| cargo-dist | 0.32.0 | 2026-05-22 | 1.74 | MIT OR Apache-2.0 | `axodotdev/cargo-dist` |
| getrandom | 0.4.3 | 2026-06-17 | 1.85 | MIT OR Apache-2.0 | `rust-random/getrandom` |
| rand | 0.10.3 | 2026-09-20 | 1.85 | MIT OR Apache-2.0 | `rust-random/rand` |

Every MSRV is under the 1.98.1 pin. `@steven-cutting/pawdoku-wasm` does not exist on
the public npm registry (HTTP 404 from `registry.npmjs.org`), and GitHub Packages was not
queried, because a read there needs a token.

**(a) pyo3 0.29.2 and maturin 1.15.0: `abi3-py311` is one GIL-enabled wheel per
platform; `extension-module` is deprecated; `generate-ci` publishes with `uv publish`
and a token unless told to use trusted publishing.**

- **Versions.** pyo3 0.29.2, released 2026-08-05, MSRV 1.83 (`rust_version` on the
  crates.io record, source; `package.rust-version = "1.83"` in the tag's `Cargo.toml`,
  source), "CPython 3.8 or greater" (the README, docs). maturin 1.15.0 on PyPI
  (2026-08-24, `requires_python >=3.7`, source), on crates.io (MSRV 1.89, source) and
  as GitHub release `v1.15.0` (`gh api repos/PyO3/maturin/releases/latest`, source).
- **`abi3-py311`.** The guide's features page (`pyo3.rs/v0.29.2/features.html`, docs)
  lists `abi3-py38` to `abi3-py314` as "extensions of the `abi3` feature to specify the
  exact minimum Python version which the multiple-version-wheel will support", and the
  building page (`pyo3.rs/v0.29.2/building-and-distribution.html`, docs) says "package
  vendors only need to build and distribute a single copy (for each OS / architecture),
  and users can install it on all Python versions from the minimum version and up", with
  the host constraint "PyO3 is only able to link your extension module to abi3 version
  up to and including your host Python version", which is the error question (f) hit
  with the system's 3.9. One qualification from maturin's bindings page
  (`maturin.rs/bindings.html`, docs): "Free-threaded CPython 3.14 does not support the
  `abi3t` stable ABI, so maturin builds a version-specific `cp314-cp314t` wheel for it
  instead", and the generated CI does exactly that (below). So it is one `abi3` wheel
  plus one `cp314t` wheel per target, not one file.
- **`extension-module` is deprecated.** The features page lists it under "Deprecated
  features for extension module authors": "Deprecated, users should remove this feature
  and upgrade to `maturin >= 1.9.4` or `setuptools-rust >= 1.12`" (docs). The building
  page: "PyO3 uses an environment variable `PYO3_BUILD_EXTENSION_MODULE` to disable
  linking to `libpython`. This should only be set when building a library for
  distribution. `maturin >= 1.9.4` and `setuptools-rust >= 1.12` will set this for you
  automatically", and "Projects are encouraged to migrate off the feature, as it caused
  major development pain due to the lack of linking" (docs). maturin's side is
  `src/compile.rs` lines 877-880 at `v1.15.0`: `if bridge_model.is_pyo3() &&
  !bridge_model.is_bin() { build_command.env("PYO3_BUILD_EXTENSION_MODULE", "1"); }`
  (source). The ticket's Context and this spike's first (f) design both assumed the
  feature; the corrected design is in (f) and Step 5. The FAQ's other `cargo test`
  caveat stays: for a `cdylib` crate "the compiler won't be able to find your crate"
  from `tests/` (`pyo3.rs/v0.29.2/faq.html`, docs).
- **`#[pyclass]` must be `Send` and `Sync`.** The class page
  (`pyo3.rs/v0.29.2/class.html`, docs): "Python objects may be created and destroyed by
  different Python threads; therefore `#[pyclass]` objects must be `Send`" and "may be
  accessed by multiple Python threads simultaneously; therefore `#[pyclass]` objects
  must be `Sync`". `unsendable` is "Required if your struct is not `Send`… your class
  will panic when accessed by another thread" (docs), which invariant 3 makes
  unnecessary: the class holds a private enum over the library's two streams, which is
  `Send + Sync` by construction (Step 4).
- **Errors.** "PyO3 will automatically convert a `Result<T, E>` returned by a
  `#[pyfunction]` into a `PyResult<T>` as long as there is an implementation of
  `std::from::From<E> for PyErr`" (`pyo3.rs/v0.29.2/function/error-handling.html`,
  docs), and a custom class comes from `create_exception!(module, MyError,
  pyo3::exceptions::PyException);` (`pyo3.rs/v0.29.2/exception.html`, docs). The guide
  does not say the message is the `Display` text; that mapping is the `From` impl the
  bindings crate writes, and invariant 3 is what makes it write `error.to_string()`.
- **`forbid(unsafe_code)` in pyo3's own tests.** pyo3's macros do emit `unsafe`
  (`pyo3-macros-backend/src/module.rs` line 543, `pyclass.rs` line 2305, source), with
  `call_site` spans since PR #4574 (merged 2024-09-25, "fix unintentional `unsafe_code`
  trigger", source); the changelog's 0.22.4 entry is "Fix regression in 0.22.3 failing
  compiles under `#![forbid(unsafe_code)]`. #4574" (source), and
  `tests/ui/forbid_unsafe.rs` at `v0.29.2` is a `//@check-pass` test under
  `#![forbid(unsafe_code)]` (source). Question (f) confirms it on this lint table.
- **`pyproject.toml` beside the crate.** `maturin new` writes `[build-system] requires
  = ["maturin>=1.15,<2.0"]`, `build-backend = "maturin"`, a `[project]` with
  `requires-python` and `dynamic = ["version"]` (`src/templates/pyproject.toml.j2`,
  source). `[tool.maturin]` takes `features`, `manifest-path`, `module-name` ("including
  with a dotted import path such as `my_package._native`"), `python-source`, and
  `locked` ("Require Cargo.lock is up to date") (`maturin.rs/config.html`, docs). In a
  workspace, `project_layout.rs` checks an explicit `--manifest-path`, then the
  `manifest-path` key, then `rust/Cargo.toml`, then "Cargo.toml in current directory"
  (source), so a `pyproject.toml` beside `crates/pawdoku-py/Cargo.toml` needs no key;
  and `source_distribution/mod.rs` takes the crate's `Cargo.lock` or else the
  workspace root's, and with `locked` set, "Cargo.lock is required by
  `--locked`/`--frozen` but it's not found" is an error (source). So the sdist carries
  the workspace lockfile, which is invariant 8 in another registry.
- **What `maturin generate-ci github` emits.** There is no template file; the workflow
  is rendered by `src/ci/github/render.rs` and pinned by
  `src/ci/github/__snapshot__/github_default.yml` (identical at `v1.15.0` and `main`,
  source). Jobs `linux` (ubuntu-22.04: x86_64, x86, aarch64, armv7, s390x, ppc64le,
  `manylinux: auto`), `musllinux` (four targets), `windows` (windows-latest x64 and
  x86, windows-11-arm), `macos` (macos-15-intel, macos-latest), `sdist` (emitted
  whenever a `pyproject.toml` exists, `src/ci/mod.rs` line 278), and `release`. Each
  build step is `PyO3/maturin-action@v1` with `args: --release --out dist
  --find-interpreter`; with an abi3 feature the `--find-interpreter` goes and a second
  step "Build free-threaded wheels" with `-i python3.14t` is added
  (`__snapshot__/github_abi3.yml`, source). `--zig` is off unless configured and
  applies to the manylinux job only. The release job (`github_default.yml` lines
  156-181, source) runs on a tag, `needs` the five build jobs, takes `id-token: write`
  ("Use to sign the release artifacts"), `contents: write` and `attestations: write`,
  runs `actions/attest@v4` over `wheels-*/*`, then `uv publish 'wheels-*/*'` with
  `UV_PUBLISH_TOKEN: ${{ secrets.PYPI_API_TOKEN }}`. Not maturin-action, not
  `pypa/gh-action-pypi-publish`, and a token by default. With
  `[tool.maturin.generate-ci.github] trusted-publishing = true` and
  `publishing-environment = "release"` the step becomes `uv publish
  --trusted-publishing always 'wheels-*/*'` with no token (`render.rs`, source;
  `maturin.rs/distribution.html` "Using PyPI's trusted publishing", docs). The
  `--pytest`, `--zig` and `--skip-attestation` flags are "[deprecated: use
  [tool.maturin.generate-ci.github] in pyproject.toml]" (`tests/cmd/generate-ci.stdout`,
  source). Its action pins are `@v1`, `@v6`, `@v7` tags, which the house replaces with
  SHAs, and its trigger is every push to `main` and every tag, which the house narrows
  to `pawdoku-py-v*` (Step 5).
- **PyPI trusted publishing.** A GitHub publisher is the repository owner, the
  repository, the workflow filename and an optional environment ("strongly
  recommended"), with `id-token: write` "at either the job level (strongly recommended)
  or workflow level (discouraged)" (`docs.pypi.org/trusted-publishers/`, docs). A
  first publish needs no token: a "pending" publisher "will create the project when
  used for the first time", with the caveat that it "does not create a project or
  reserve a project's name until it is actually used to publish"
  (`docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/`, docs). So,
  unlike crates.io in S02, the Python crate never needs a token at all.

**(b) pixi 0.81.0 never reads `crates/pawdoku-py/pyproject.toml` from the repository
root, and fails on it from inside that directory; it is maturin's file alone.** pixi
0.81.0 was the latest release (2026-09-15, `gh api
repos/prefix-dev/pixi/releases/latest`, source); `pixi.sh/latest/` now redirects to
`pixi.prefix.dev/latest/`, where the pages below were read.

- **Discovery.** The manifest reference's "Manifest discovery" table
  (`pixi.prefix.dev/latest/reference/pixi_manifest/`, docs): priority 6
  `--manifest-path`, 5 `pixi.toml` and 4 `pyproject.toml` "In your current working
  directory", 3 "Iterate through all parent directories. The first discovered manifest
  is used", 1 `$PIXI_PROJECT_MANIFEST` inside `pixi shell` or `pixi run`. Subdirectories
  are never searched, so every recipe, which runs from the repository root, finds the
  root `pyproject.toml` and stops. The `--manifest-path` flag is "The path to
  `pixi.toml`, `pyproject.toml`, or the workspace directory"
  (`crates/pixi_cli/src/cli_config.rs` lines 29-31, source).
- **From inside `crates/pawdoku-py/`.** pixi's own snapshot test covers this shape: a
  nested `pyproject.toml` with no `tool.pixi` table under a pyproject-based workspace
  (`tests/data/workspace-discovery/nested-pyproject-workspace/nested-non-pixi-pyproject/`,
  source) is expected to fail with `× Missing field \`tool.pixi\``
  (`crates/pixi_manifest/src/snapshots/…nested-non-pixi-pyproject.snap`, source);
  `discovery.rs` lines 565-566 skip such a file only when a package manifest was
  already found below it (source). So a developer who runs `pixi run` from inside the
  crate directory gets that error, and the recipes never do. Not a pixi-build package,
  not a workspace member: "a file pixi never reads".
- **What would make pixi read it.** Listing it as a `path` entry under
  `pypi-dependencies`, when "Pixi will use this section to build and install the
  project if it is added as a pypi path dependency"
  (`pixi.prefix.dev/latest/python/pyproject_toml/`, docs), building it with `uv`, which
  would need Rust in the pixi environment. Not wanted.
- **pixi-build and workspaces.** "`pixi-build` is a preview flag, and will change until
  it is stabilized" (manifest reference, docs); the build tutorial lists "Known
  limitations: … Limited set of build-backends" (docs). A Rust backend exists and makes
  conda packages, not wheels. `[project]` became `[workspace]` in 0.43.0 (2025-03-20,
  the CHANGELOG, source) and is still an alias; multiple packages in one pixi workspace
  exist only through pixi-build. None of this applies to a maturin crate.
- **maturin from pixi.** conda-forge's `maturin` 1.15.0 has run dependencies `python`,
  `tomli >=1.1.0`, `openssl` and the platform's libc; `${{ compiler('rust') }}` is a
  build requirement of the feedstock only (`api.anaconda.org/package/conda-forge/maturin`
  and `conda-forge/maturin-feedstock` `recipe/recipe.yaml`, source), so the pin adds no
  `rust` to the environment. The PyPI wheels are `py3-none-<platform>` with no Rust in
  `requires_dist` (source), a second route the house does not need. Either way maturin
  drives rustup's cargo from `PATH` (decision 0011).

**(c) wasm-bindgen 0.2.129 and wasm-pack 0.15.0: `--target web` is what a plugin-free
Vite site imports; wasm-pack writes no `exports` and has no post-build hook, so `pkg/`
is amended after the build with `npm pkg set` or a few lines of Python.**

- **Versions and homes.** wasm-bindgen 0.2.129, js-sys 0.3.106, wasm-bindgen-test 0.3.79
  and wasm-bindgen-cli 0.2.129 were all released on 2026-09-25 (crates.io, source); the
  crate's MSRV is 1.81 and the CLI's 1.86 (source). The project moved from `rustwasm` to
  the `wasm-bindgen` organisation: `gh api repos/rustwasm/wasm-bindgen` redirects to
  `wasm-bindgen/wasm-bindgen` (source), `rustwasm.github.io/wasm-bindgen/` is HTTP 404
  and the guide lives at `wasm-bindgen.github.io/wasm-bindgen/` (other). wasm-pack
  0.15.0 (2026-05-15) lives at `wasm-bindgen/wasm-pack`; its release notes say "The
  0.14.0 npm package shipped with the old `drager/wasm-pack` release URL and was never
  republished after the repository moved" (source), so the ticket's `drager` reference
  is a year out of date. `rustwasm/console_error_panic_hook` is **archived** (last push
  2022-12-04, last release 2021-10-11, source). `RReverser/serde-wasm-bindgen` is not
  archived (last push 2025-10-01) but its last release is 0.6.5 of 2024-02-27 (source).
- **CLI and crate must match exactly.** The guide: "Make sure to replace "X.Y.Z" with
  the same version of `wasm-bindgen` that you already have in `Cargo.toml`!"
  (`wasm-bindgen-test/usage.md` at 0.2.129, docs). The CLI enforces it:
  "Currently the bindgen format is unstable enough that these two schema versions must
  exactly match" (`crates/cli-support/src/wit/mod.rs` lines 2809-2830, source).
  wasm-pack reads the version from `Cargo.lock` (`src/lockfile.rs`
  `wasm_bindgen_version`, source) and, in its default `--mode normal`, installs a
  matching CLI itself, a download `check` must never make; hence the
  `wasm-bindgen-cli = "==0.2.129"` pixi pin held equal to the lockfile, a guard that
  compares the two, and `--mode no-install` (Step 5). The first review round read the
  installer at wasm-pack `v0.15.0` (commit `8320269d`, source):
  `download_prebuilt_or_cargo_install` (`src/install/mod.rs` lines 55-89) first takes
  `which wasm-bindgen` if `check_version` says it equals the lockfile's version, and
  otherwise tries `download_prebuilt` and then `cargo_install`, each handed
  `install_permitted`; `InstallMode::install_permitted` (`src/install/mode.rs`) is
  false for `no-install` alone; `step_install_wasm_bindgen` (`src/command/build.rs`
  lines 461-470) reads the version with `lockfile.require_wasm_bindgen()` and passes
  `self.mode.install_permitted()`. Under `no-install`, binary-install 0.4.1's
  `Cache::_download` (`src/lib.rs` lines 123-128, source) returns a binary already in
  wasm-pack's cache and otherwise `Ok(None)` without fetching, which ends in "Not able
  to find or install a local wasm-bindgen." So `no-install` never downloads, but on its
  own it would take a right-version CLI from the cache, not pixi's; the guard is what
  makes the `PATH` branch the one that wins. Arguments after `--` reach
  `cargo_build_wasm` as `extra_options` (`build.rs` lines 197-198 and 412-418).
- **Type mapping, from the guide at 0.2.129 (docs).** "`u64`, `i64`, `u128`, and
  `i128` will be represented as `BigInt` in JavaScript" (`reference/types/numbers.md`);
  an incoming BigInt "too large or too small for the target integer type … will wrap
  around". `Box<[T]>` and `Vec<T>` of numbers cross as "a JavaScript `TypedArray` of the
  appropriate type", copied each way (`boxed-number-slices.md`, `boxed-slices.md`);
  `&[f64]` is accepted as a parameter as a view over Wasm memory and cannot be returned
  (`number-slices.md`). `Result<T, E>`: "whenever `Err(error)` is encountered an
  exception is thrown in JS with `error`", with `E: Into<JsValue>`
  (`reference/types/result.md`). Exported structs need no `Clone`; public fields need
  `Copy` for automatic getters or `getter_with_clone`, and private fields with
  `#[wasm_bindgen(getter)]` methods need neither (`exported-rust-types.md`,
  `getter_with_clone.md`). `js_sys::Function::call0` returns `Result<JsValue, JsValue>`,
  and "If calling the imported function throws an exception, then `Err` will be
  returned with the exception that was raised" (`catch.md`). The probe in (f) compiled
  every one of these shapes.
- **`--target web`.** wasm-pack's build page (docs): `bundler` "Outputs JS that is
  suitable for interoperation with a Bundler like Webpack"; `web` "Outputs JS that can
  be natively imported as an ES module in a browser, but the WebAssembly must be
  manually instantiated and loaded". The wasm-bindgen deployment page: the bundler
  output "assumes a model where the Wasm module itself is natively an ES module. This
  model, however, is not natively implemented in any JS implementation at this time"
  and "Currently the only known bundler known to be fully compatible with
  `wasm-bindgen` is webpack" (docs). The `web` output's `init()` defaults its module
  path to `new URL('<name>_bg.wasm', import.meta.url)` and fetches it
  (`crates/cli-support/src/js/mod.rs` lines 1572-1594 and 1898 at 0.2.129, source), and
  accepts a `URL`, `Response`, `ArrayBuffer` or compiled `WebAssembly.Module` in the
  object form `init({ module_or_path })`; the positional form is deprecated (source).
  Vite: "During the production build, Vite will perform necessary transforms so that
  the URLs still point to the correct location even after bundling and asset hashing"
  for `new URL(url, import.meta.url)`, and "This pattern does not work if you are using
  Vite for Server-Side Rendering" (`docs/guide/assets.md`, docs). G's `vite.config.ts`
  has no wasm plugin, so `web` is the target it can import without one; the game calls
  `init()` from `onMount` or behind `browser`, never during prerender, because
  adapter-static prerenders with SSR on ("You must ensure SvelteKit's `ssr` option isn't
  set to `false`", `adapter-static.md`, docs).
- **vitest.** No jsdom-specific issue was found. The vitest maintainer's closing comment
  on issue #2910 (2024-01-23, source): "explicitly loading wasm module seems most
  simplest and obvious way at least for Vitest", with the pattern `readFile` the
  `_bg.wasm`, `WebAssembly.compile`, then `init(module)` in `beforeAll`
  (`hi-ogawa/argon2-wasm-bindgen/test/basic.test.ts`, source). G's restatement test
  (Step 6) needs no `init()` at all: it reads `specs/*.allium` as text.
- **What wasm-pack writes, and what it does not.** At v0.15.0, `to_web` produces
  `name` (`@<scope>/<crate>` with `--scope`), `"type": "module"`, `version`, `license`,
  `repository` (`{ "type": "git", "url": <Cargo's repository verbatim> }`), `files`
  (`<name>_bg.wasm`, `<name>.js`, `<name>.d.ts`), `main`, `types`, `sideEffects:
  ["./snippets/*"]`, plus description, homepage, keywords and collaborators from
  `Cargo.toml` (`src/manifest/mod.rs` line 792 and `npm/esmodules.rs`, source). **No
  `exports` and no `module` field**; the build page's table saying `module` and
  `sideEffects: false` is stale against the source (the `type`/`main` change is commit
  `473dbf8f`, 2023-11-14). `--scope`, `--out-dir` (default `pkg`), `--out-name`,
  `--no-pack` and extra cargo arguments after `--` exist (`src/command/build.rs`,
  source). **No post-build hook**: the build page's footnote says "If you need to
  include additional assets in the pkg directory and your NPM package, we intend to
  have a solution for your use case soon" (docs). `[package.metadata.wasm-pack.profile.
  release]` is honoured, with `wasm-opt` and the `wasm-bindgen` sub-keys
  (`cargo-toml-configuration.md`, docs); that page contradicts itself on the release
  `wasm-opt` default, so T15 sets `wasm-opt = false` explicitly.
- **Amending `pkg/package.json`.** `npm pkg set` "Sets a `value` in your `package.json`
  … The same syntax used to retrieve values from your package can also be used to
  define new properties", with `[]` appending to arrays and `[key]` selecting into
  objects (`docs.npmjs.com/cli/v11/commands/npm-pkg`, docs). Run in the scratch
  directory on npm 11.17.0 (other): `npm pkg set 'files[]=specs' 'exports[.]=./y.js'
  'exports[./specs/*]=./specs/*'` produced `"files":["y_bg.wasm","y.js","specs"]` and
  `"exports":{".":"./y.js","./specs/*":"./specs/*"}`. **Encapsulation**: "When the
  `"exports"` field is defined, all subpaths of the package are encapsulated and no
  longer available to importers" (Node's `packages.md`, docs), so the map must also
  export `./<name>_bg.wasm` (for a consumer that loads the bytes itself, as vitest
  does) and carry `types` under `"."`. The full map T15 writes: `"."` with `types` and
  `default`, `"./pawdoku_wasm_bg.wasm"`, and `"./specs/*"`. Because the amendment is
  a JSON edit, Step 5 does it with the pixi environment's Python rather than adding
  Node to the toolchain; the publish job, which has Node for `npm publish`, could use
  `npm pkg set` instead.
- **Publishing.** `wasm-pack publish` "creates a tarball from the pkg directory **and**
  publishes it to the NPM registry. Underneath, these commands use `npm pack` and `npm
  publish`" (docs); its source is `npm publish` in `pkg/` with no `--registry` flag,
  and it prompts interactively when `pkg/` is missing (`src/npm.rs`,
  `src/command/publish/mod.rs`, source). So the release job runs `npm publish` in
  `pkg/` itself, with the registry from `actions/setup-node`'s `.npmrc`.
- **Not verified**, as the researcher recorded: that `JsError: Into<JsValue>` was not
  read on docs.rs, though the probe's `Result<Vec<f64>, JsError>` export compiled, which
  is the same fact; that Vite 8.2.1 no longer lowers `import.meta.url` when
  pre-bundling a `web` dependency, an issue from Vite 2 (vitejs/vite#7287, 2022-03-12,
  other); the effective release `wasm-opt` default.

**(d) GitHub Packages: `packages: write` on the publish job with the run's token; the
`repository` field links the package; provenance is documented for `registry.npmjs.org`
only.** GitHub's docs were read from `github/docs` at `f4e8afc6` (2026-09-28) and the
rendered pages.

- **Publishing from Actions.** The tutorial's workflow (`publish-nodejs-packages.md`,
  docs): job `permissions: contents: read, packages: write`; `actions/setup-node` with
  `registry-url: 'https://npm.pkg.github.com'` and `scope`, which writes an `.npmrc`
  with `//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}`, the scope line and
  `always-auth=true`; then `npm publish` with `NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN
  }}`. "All workflows accessing registries that support granular permissions should use
  the `GITHUB_TOKEN` instead of a personal access token", and the npm registry supports
  granular permissions (`publishing-and-installing-a-package-with-github-actions.md`,
  `about-permissions-for-github-packages.md`, docs). No secret is stored, which is what
  CONVENTIONS.md §11 asks.
- **`repository` must match.** "The `repository` field must match the URL for your
  GitHub repository. For example, if your repository URL is `github.com/my-org/test`
  then the repository field should be `https://github.com/my-org/test.git`"
  (`verify_repository_field.md`, docs); another page shows the form without `.git`.
  wasm-pack copies Cargo's `repository` verbatim, and the workspace's is
  `https://github.com/steven-cutting/libpawdoku` (no `.git`). Whether both forms are
  accepted is **not verified**; T15 tries the inherited value first and, on a
  rejection, sets `repository` in `pkg/package.json` in the same amendment step.
  "Package names and scopes must only use lowercase letters", and the scope is the
  account: `@steven-cutting/pawdoku-wasm` (`working-with-the-npm-registry.md`, docs).
- **Provenance.** GitHub's docs mention `--provenance` only in the `registry.npmjs.org`
  workflow (`publish-nodejs-packages.md` line 59, gated on artifact attestations, with
  `id-token: write`); the GitHub Packages workflow has neither. npm's page "Generating
  provenance statements" says "publish your packages to the npm registry" and every
  example uses `registry-url: 'https://registry.npmjs.org'` (docs); the npm trusted
  publishers page's OIDC audience is `npm:registry.npmjs.org` (docs). `npm/provenance`'s
  README says another registry "can distribute public keys using the same hostname
  scheme" (other), which describes what a registry could do, not what GitHub Packages
  does. **Verdict: undocumented for GitHub Packages, documented only for
  `registry.npmjs.org`**; T15's job has no `id-token: write` and no `--provenance`, and
  gains them the day GitHub documents it. Nothing publishes from `ci.yml` (§10).
- **Read access for G.** "For packages scoped to a personal account or an organization,
  to ensure that a GitHub Actions workflow has access to your package, you must give
  explicit access to the repository where the workflow is stored. The specified
  repository does not need to be the repository where the source code for the package
  is kept" (`configuring-a-packages-access-control-and-visibility.md`, docs): Package
  settings, "Manage Actions access", **Add repository**, role Read. A package published
  with `repository` set inherits the linked repository's permissions "if you link the
  repository to the package before you publish the package" (docs). Visibility is set
  separately; "When you first publish a package, the default visibility is private"
  (docs). If the package is public, "any workflow running in any repository can
  download the package" with its own token (docs); either way G's `packages: read`
  and the `.npmrc` line suffice, and a developer installing locally needs a classic
  token with `read:packages`, as G's `develop-locally.md` already says.

**(e) clap 4.6.7 is fine; cargo-dist is active but is the wrong tool here, because it
owns `release.yml`, checks it on every pull request, and publishes nothing to
crates.io; the CLI's binaries ride S02's cargo-release flow with two pinned actions.**

- **clap.** 4.6.7, released 2026-09-14, MSRV 1.85, `MIT OR Apache-2.0`, edition 2024
  (crates.io, source; `rust-version = "1.85"` in the workspace `Cargo.toml`, source).
  Its crate docs still say "MSRV, currently 1.74" (`src/lib.rs` line 30, source),
  which is stale. `derive` is optional and pulls `clap_derive` (`_features/index.html`,
  docs); default features are `std` ("Not Currently Used"), `color`, `help`, `usage`,
  `error-context` and `suggestions`, of which `color` brings `anstream` and the
  Windows-only crates and `suggestions` brings `strsim`. With defaults and `derive` the
  probe in (f) locked fifteen crates under clap on macOS. `wrap_help` and `env` are
  off by default and stay off.
- **cargo-dist.** The crate is `cargo-dist`, the binary `dist` ("formerly known as
  `cargo-dist`", README, source). crates.io's newest is 0.32.0 (2026-05-22, MSRV 1.74),
  and `/crates/cargo-dist/0.33.0` is HTTP 404, while GitHub has release `v0.33.0`
  (published 2026-09-11) (source), so the current version is a GitHub release only.
  conda-forge does not carry it (`api.anaconda.org`, 404, source), so it would go
  through `tools.txt` like cargo-hack. It is maintained: `archived: false`, pushed
  2026-09-25, 37 non-Dependabot commits in six months; Astral's fork is archived and
  "0.29.0 includes all of the new features from Astral's fork" (source). Three facts
  decide against it:
  1. **The filename.** `const GITHUB_CI_FILE: &str = "release.yml";`
     (`cargo-dist/src/backend/ci/github.rs` line 43, source). "since 0.3.0 we will
     actually consider it an error for there to be any edits or out of date information
     in release.yml" (`book/src/ci/customizing.md`, docs); `dist plan`, `dist build`
     and `dist manifest` run `check_integrity`, "currently equivalent to `dist generate
     --check`" (`cargo-dist/src/lib.rs`, source); and "By default we run the "plan"
     step of your release CI on every pull-request" (`quickstart/rust.md`, docs). So a
     hand-written `release.yml` (S02's T13) and dist's cannot share the name, and the
     escape hatches are `tag-namespace` ("It also renames `release.yaml` to
     `owo-release.yml`", docs) or `allow-dirty = ["ci"]`, which the book discourages.
  2. **Nothing for crates.io.** `publish-jobs` accepts "homebrew", "npm" or a custom
     reusable workflow (`reference/config.md`, docs); "dist intentionally doesn't
     handle these steps … publishing to crates.io" (`cargo-release-guide.md`, docs). So
     cargo-release stays regardless, and dist would be a second release tool.
  3. **Its shape.** The plan job installs dist with `curl … | sh` from GitHub Releases
     (`axolotlsay`'s rendered `release.yml`, source); the default tag glob
     `**[0-9]+.[0-9]+.[0-9]+*` matches every crate's tag, and a library tag produces
     "a very minimal build-less Announcement (and therefore Github Release)"
     (`workspace-guide.md`, docs); a plan job on every pull request is a workflow
     outside `check` that fails on a stale generated file (§10). Its root permissions
     are `contents: write` (source), which is also what the alternative needs.
- **The alternative, recommended.** cargo-release 1.1.6 (2026-09-16, MSRV 1.92,
  source), as S02 chose, cuts the tag `pawdoku-cli-v<version>`; `release-cli.yml`,
  triggered by that tag, runs `taiki-e/create-gh-release-action` (v1.11.0, 2026-04-17)
  then `taiki-e/upload-rust-binary-action` (v1.30.2, 2026-04-17; `archived: false`,
  pushed 2026-09-02, source) over a three-runner matrix, each pinned by SHA (its README:
  "consider using the `@v<major>.<minor>.<patch>` tag or their hash to pin the
  version", source), with `locked: true` (the default is `false`), `bin: pawdoku`,
  `archive: pawdoku-cli-$target` (binstall's default templates key on the package
  `{ name }`, the action's default on the binary `$bin-$target`; the two coincide
  only when package and binary share a name, so the archive is named explicitly and
  `[package.metadata.binstall]` stays absent), `tar: unix`, `zip: windows`,
  `checksum: sha256`, and `permissions: contents: write` on the job. cargo-binstall
  users then get the CLI with `cargo binstall pawdoku-cli` once it is on crates.io
  (S02's open question), which is why the archive name follows binstall's default
  layout. One release tool, one hand-written workflow per registry, no generated file
  and no installer script.
- **Windows.** `windows-latest` is Windows Server 2025 (labels `windows-latest`,
  `windows-2025`, `windows-2025-vs2026`), image 20260922.246.2, with Rust 1.98.1,
  Rustup 1.29.1, Bash 5.3.15 and Git (`actions/runner-images`
  `images/windows/Windows2025-VS2026-Readme.md`, source). `just` and `pixi` are not
  listed. Every tool `pyproject.toml` pins has a `win-64` build on conda-forge at the
  pinned version (source), so the blocker for a pixi-based Windows job is the
  repository's own "win-64 never: bg-install-allium and the Justfile are POSIX-only",
  not the tool supply (Step 5).
- **cargo-deny `wrappers` means the direct parent.** "This field allows specific crates
  to have a direct dependency on the banned crate but denies all transitive dependencies
  on it" (`checks/bans/cfg.html`, docs); the check walks `direct_dependents` and
  warns "direct parent '{}' of banned crate '{}' was not marked as a wrapper" before
  the `banned` error (`src/bans.rs` lines 608-657, `src/bans/diags.rs`, source). rand
  0.10.3's default features include `sys_rng = ["dep:getrandom", "getrandom/sys_rng"]`
  (source), so a CLI on `rand` would make `rand` the direct parent of `getrandom` and
  fail today's `deny.toml`. The CLI therefore depends on `getrandom` alone (Step 4).
  The `unused-wrapper` warnings, two of them, are already recorded by T02
  ("It also printed two `unused-wrapper` warnings, one for each `"pawdoku-cli"` entry.
  All ten are kept, not silenced", T02's notes).
- **getrandom on wasm.** "the `wasm32-unknown-unknown` and `wasm64-unknown-unknown`
  targets … are not automatically supported", "If no implementation is available, a
  compilation error will be raised" (docs.rs and the README, docs). `just wasm-check`
  is `-p pawdoku` today and gains `-p pawdoku-wasm`, never `--workspace`, so the CLI is
  never checked for wasm (Step 5). `getrandom::u64()` exists since 0.3.0 ("`u32` and
  `u64` functions for generating random values of the respective type", the CHANGELOG,
  source) and is what the probe called.

**(f) Neither pyo3's nor wasm-bindgen's macro output trips `forbid(unsafe_code)`, or
any other lint in the workspace table; wasm-bindgen refuses `const fn`, which
`missing_const_for_fn` then demands. Proved, not read.** Two throwaway crates were built
in the session's scratch directory on the pinned 1.98.1 (the repository's
`rust-toolchain.toml` copied beside each), each with the three `[workspace.lints.*]`
tables copied verbatim into its own `[lints]` tables, `crate-type = ["cdylib", "rlib"]`
and an empty `[workspace]` so cargo treated it alone.

- **`probe-wasm`**, depending on wasm-bindgen 0.2.129, js-sys 0.3.106, serde-wasm-bindgen
  0.6.5 and console_error_panic_hook 0.1.7: a `#[wasm_bindgen]` struct holding a `u64`,
  a `#[wasm_bindgen(constructor)]`, a `#[wasm_bindgen(getter)]` returning `u64`, a
  method taking `Vec<f64>` and returning `Result<Vec<f64>, JsError>`, a method taking
  `&js_sys::Function` and returning its `call0` result (`Result<JsValue, JsValue>`), and
  a `#[wasm_bindgen(start)]` function that installs the panic hook. As first written,
  with the constructor and getter `const fn` as the core writes them, both `cargo check`
  and `cargo clippy` failed in the macro itself, on the host and on
  `wasm32-unknown-unknown`:

  ```text
  error: can only #[wasm_bindgen] non-const functions
    --> src/lib.rs:17:9
     |
  17 |     pub const fn new(seed: u64) -> Self {
     |         ^^^^^
  ```

  With `const` removed, `cargo clippy --all-targets -- -D warnings` and
  `cargo clippy --target wasm32-unknown-unknown -- -D warnings` each failed on exactly
  two errors, both `clippy::missing_const_for_fn` on those two functions ("this could be
  a `const fn`"), and on nothing else: `forbid(unsafe_code)`, `missing_docs`,
  `unreachable_pub`, `unnameable_types`, the pedantic group and the rest of the table
  are silent on wasm-bindgen's expansion. The two lints contradict each other on any
  exported function whose body could be `const`, so the wasm crate's manifest keeps
  `[lints] workspace = true` and each such function carries
  `#[expect(clippy::missing_const_for_fn, reason = "wasm-bindgen exports cannot be const")]`
  (invariant 6). In practice few exports qualify: a constructor that boxes a stream or
  calls into the core is not `const` anyway. Because the research for question (c)
  found three 2026 reports of `forbid(unsafe_code)` tripping on wasm-bindgen
  expansions, and `wasm-bindgen-macro-support`'s `codegen.rs` at 0.2.129 does emit
  `unsafe` sixty times with no `allow_internal_unsafe`, the probe then gained the one
  form it lacked, an import block: `#[wasm_bindgen] extern "C" { #[wasm_bindgen(js_namespace
  = console, js_name = error)] fn console_error(message: &str); }` and a
  `std::panic::set_hook` that calls it. Both the `extern "C"` and the `unsafe extern
  "C"` spellings compiled under the table with the same two `const` errors and nothing
  else, and with the two `#[expect]` attributes in place `cargo clippy --target
  wasm32-unknown-unknown -- -D warnings` and `cargo clippy --all-targets -- -D
  warnings` printed `Finished` and exited 0; `cargo build --release --target
  wasm32-unknown-unknown` linked the cdylib. Why it passes is inference, not a read
  source: rustc appears not to report the `unsafe_code` lint inside an external macro's
  expansion unless the offending tokens carry the caller's span, which is the mechanism
  pyo3's fix in question (a) relies on (PR #4574 changed spans, not lint levels); the
  rustc source was not read. The three reports name older or different macro forms and
  are recorded, not followed.
  The compiled result on 0.2.129 and 1.98.1 is the evidence this spike stands on, and
  T15's first `just clippy` re-proves it on the real crate. The import block also
  settles the panic hook: `console_error_panic_hook` is archived (question (c)), and
  ten lines in the crate do what it does with no dependency, so T15 does not add it.
- **`probe-py`**, depending on pyo3 0.29.2 with `abi3-py311`: a `#[pyclass(frozen)]`
  struct deriving `Debug` and `Clone`, a `#[new]` constructor, a `#[getter]`, a
  `#[pyfunction]` returning `Err(PyValueError::new_err("the replay stream is exhausted at
  draw 3"))` and a `#[pymodule]`. `cargo check` passed with one warning from the macro,
  and `cargo clippy --all-targets -- -D warnings` made it the only error:

  ```text
  error: use of deprecated associated constant `pyo3::impl_::deprecated::HasAutomaticFromPyObject::<true>::MSG`: The `FromPyObject` implementation for `#[pyclass]` types which implement `Clone` is changing to an opt-in option. Use `#[pyclass(from_py_object)]` to opt-in to the `FromPyObject` derive now, or `#[pyclass(skip_from_py_object)]` to skip the `FromPyObject` implementation.
   --> src/lib.rs:7:1
    |
  7 | #[pyclass(frozen)]
    | ^^^^^^^^^^^^^^^^^^
  ```

  With `#[pyclass(frozen, skip_from_py_object)]`, `cargo clippy --all-targets --
  -D warnings` printed `Finished` and exited 0. So pyo3's expansion trips nothing in the
  table either, and every `#[pyclass]` that derives `Clone` (invariant 3 wants `Clone`)
  writes `skip_from_py_object` or `from_py_object` explicitly. The Python crate too keeps
  `[lints] workspace = true`.
- **The negative control.** A function with `unsafe { core::mem::transmute::<i8, u8>(-1) }`
  appended to each probe made `cargo check` fail with
  `error: usage of an \`unsafe\` block` and `= note: requested on the command line with
  \`-F unsafe-code\``, so the copied table was live in both runs.
- **`extension-module` and `cargo test`.** The probe first named `pyo3/extension-module`
  in its manifest, as the ticket's Context assumed, before question (a) showed the
  feature deprecated in 0.29 in favour of the `PYO3_BUILD_EXTENSION_MODULE` variable
  maturin sets. With the feature on, `cargo test --no-run` failed to link (`Undefined
  symbols for architecture arm64: "_PyBaseObject_Type"` and the rest of the C API),
  which is the "major development pain" the guide describes. With the feature off,
  `cargo test --no-run` first failed in `pyo3-build-config` because the `python3` on
  `PATH` was the system's 3.9.6:

  ```text
  error: cannot set a minimum Python version 3.11 higher than the interpreter version 3.9 (the minimum Python version is implied by the abi3-py311 feature)
  ```

  With `PYO3_PYTHON` pointed at the worktree's pixi Python (3.14.7) it linked, and the
  test binary then aborted on launch because `dyld` could not find `libpython3.14.dylib`
  on its search path. `cargo clippy --all-targets -- -D warnings` stayed green with the
  feature off and with `--features pyo3/extension-module` passed on the command line.
  Three consequences for T17: the crate's manifest never names `extension-module`, and
  maturin sets the variable when it builds the wheel, so `--all-features` in the gate
  changes nothing; the recipes that build the crate set `PYO3_PYTHON` to the pixi
  environment's interpreter, because the pin is 3.11 or newer and a stray system Python
  is older; and the crate's tests are Python tests run by pytest against a `maturin
  develop` build, with `just test` excluding the crate (`--exclude pawdoku-py`), because
  a Rust test binary that embeds the interpreter needs the loader to find `libpython`
  and a `no_std` engine has nothing to gain from testing through Python twice.
- **`probe-cli`**, a third probe the ticket did not name, depending on clap 4.6.7
  (`derive`) and getrandom 0.4.3: a `#[derive(Parser)]` struct with `--seed
  Option<u64>`, `getrandom::u64()` when the flag is absent, and output through
  `writeln!` on `std::io::stdout().lock()`. `cargo clippy --all-targets -- -D warnings`
  printed `Finished` and exited 0, so clap's derive output trips nothing in the table
  either, and `getrandom::u64()` is the one call the CLI needs. `cargo run` printed
  `seed = 16009227737182265439` then `random_version = seeded-stream-1`, and
  `cargo run -- --seed 7` printed `seed = 7`. The control: the same binary with one
  `println!` failed clippy with `error: use of \`println!\`` under
  `clippy::print_stdout`, so the CLI writes through the locked handle and needs no
  `#[expect]` for its output.
- **What the dependency trees bring.** `cargo tree -e normal` and `cargo metadata` on the
  probes (source): wasm-bindgen pulls cfg-if, once_cell, bumpalo, proc-macro2, quote, syn
  3, unicode-ident and wasm-bindgen-shared; js-sys adds futures-core, futures-task,
  futures-util, pin-project-lite and slab; serde-wasm-bindgen adds serde. pyo3 pulls
  libc, once_cell, pyo3-ffi, pyo3-macros, pyo3-macros-backend, pyo3-build-config, heck,
  target-lexicon, portable-atomic, proc-macro2, quote, syn 2 and unicode-ident. clap
  pulls clap_builder, clap_derive, clap_lex, anstream, anstyle and its parse, query and
  wincon crates, colorchoice, is_terminal_polyfill, once_cell_polyfill, strsim,
  utf8parse, heck, proc-macro2, quote, syn 3 and, for Windows, windows-sys and
  windows-link; getrandom pulls cfg-if, libc and, for UEFI, r-efi. Every licence is
  already in `deny.toml`'s allow list: `MIT OR Apache-2.0` throughout, `MIT` alone for
  serde-wasm-bindgen, slab and strsim, `Apache-2.0/MIT` for console_error_panic_hook,
  `(MIT OR Apache-2.0) AND Unicode-3.0` for unicode-ident, `Apache-2.0 WITH
  LLVM-exception` for target-lexicon, and `MIT OR Apache-2.0 OR LGPL-2.1-or-later` for
  r-efi, which the allowed `MIT` satisfies. No new licence entry is needed (Step 5).

The Goal's table said "likely, for generated code" and "likely, for pyo3's macros" under
"Own `[lints]`". Both are wrong on today's versions: neither crate needs its own table.
The corrected table is under Step 5.

**Step 4. The randomness crossing, in the trait's names.** The boundary is `pub trait
RandomStream { fn next_draw(&mut self) -> Result<f64, RandomError>; fn index(&self) ->
u64; }`, dyn-compatible, with two implementations the library ships:
`SeededStream::new(seed: u64)`, the generator `RANDOM_VERSION` names, and
`ReplayStream::new(draws: Vec<f64>) -> Result<Self, RandomError>`, the fake. Every
engine entry point that draws takes `&mut dyn RandomStream` (the shape
`tests/random.rs` already exercises through `Box<dyn RandomStream>`), so a binding
never re-implements the trait: it holds one of the library's streams and lends it as
`&mut dyn RandomStream` (below). Three things are the same in every binding, and they
are what makes `ExactReplay` (`human-solving.allium` lines 1093-1101,
`generation.allium` lines 34-38) hold across a browser, Python and a terminal:

1. **The default is the library's generator.** A caller that gives a seed and nothing
   else gets `SeededStream::new(seed)`, whose draws are bit-identical on every IEEE 754
   target (`random.rs` lines 120-125), so a seed recorded in the game replays in Python.
2. **The name travels with the seed.** Each binding exports `RANDOM_VERSION`
   (`"seeded-stream-1"`) as a constant, and every result that consumed draws carries the
   seed, the name and the count of draws consumed (`index()`), which is what the
   guarantee asks a consumer to record.
3. **The fake is a list of draws.** `ReplayStream::new` takes a `Vec<f64>`, and a list
   is the one shape every language has: a JavaScript array or `Float64Array`, a Python
   `list[float]`. A test scripts its draws and hands the list across, as the
   specification says ("a test supplies the draws through the boundary's fake").
   `RandomError::OutOfRange` comes back for a value outside `[0, 1)` and
   `RandomError::Exhausted` when the script runs out, each as the exception the
   language builds from the `Display` text.

The ticket asked for a fourth shape, a live callback ("a JavaScript function that
supplies draws", "a callable" in Python). It is **not recommended**, and the reason is
in the trait: `RandomError` is `#[non_exhaustive]` with `Exhausted` and `OutOfRange`
only, so a binding that wrapped a `js_sys::Function` (whose `call0` returns
`Result<JsValue, JsValue>`, proved in the probe) or a Python callable has no variant to
return when the callback throws, returns a non-number, or returns 1.0. A binding cannot
add a variant to the core's enum, and swallowing the failure into a panic breaks
decision 0008's "no panics in the public surface". Nothing in the specifications asks
for a callback: `ExactReplay` names a seed and a script. If a consumer ever needs one,
the core gains a variant such as `RandomError::Source { index, message }` first, which is
a T02 hand-back and a spec sentence, and this is recorded as an open point below.

**What a binding's class holds.** Not a `Box<dyn RandomStream>`: a trait object keeps
only the bounds written on it, so a boxed stream is neither `Send` nor `Sync` unless the
box says so, and it can never be `Clone` or `Debug`, because the trait has only
`next_draw` and `index`. A class holding one would break invariant 3 for the binding's
own public type (first review round, Deviations). Each binding instead holds a private

```rust
#[derive(Clone, Debug)]
enum Stream {
    Seeded(SeededStream),
    Replay(ReplayStream),
}

impl Stream {
    fn as_dyn(&mut self) -> &mut dyn RandomStream {
        match self {
            Self::Seeded(stream) => stream,
            Self::Replay(stream) => stream,
        }
    }
}
```

which is `Send + Sync + 'static` by construction, because both variants are
(`tests/api_bounds.rs`), and whose `as_dyn` is what the class passes to every engine
entry point. The enum is closed on purpose: a third source is a new variant, which is
the same core change the live callback would need first. Each binding proves its class's
bounds at compile time with an in-crate item beside the class, the helpers
`api_bounds.rs` already uses, so `just clippy` and `just wasm-check` fail the day a
field breaks them:

```rust
const _: () = {
    const fn assert_bounds<T: Send + Sync + 'static + Clone + core::fmt::Debug>() {}
    assert_bounds::<RandomStream>();
};
```

It sits in `src/lib.rs`, not `tests/`, because `pawdoku-py` is a `cdylib` whose
`tests/` cannot link it (question (a), pyo3's FAQ) and whose Rust tests `just test`
excludes. A scratch crate on the core, checked offline on 2026-09-28 with clippy's
`pedantic` group and `-D warnings`, compiled both blocks clean, and the same assertion
over a struct holding `Box<dyn RandomStream + Send + Sync>` failed with
``the trait bound `Boxed: Clone` is not satisfied`` and ``doesn't implement `Debug` ``.

- **`pawdoku-wasm`.** One exported class, `RandomStream`, `#[derive(Clone, Debug)]`,
  holding a `Stream`, with two static constructors:
  `RandomStream.seeded(seed: bigint)` calling `SeededStream::new`, and
  `RandomStream.replay(draws: Float64Array | number[])` calling `ReplayStream::new` and
  throwing on `Err`. `nextDraw(): number` calls `next_draw` and throws
  `Error("the replay stream is exhausted at draw 3")` on `Err`; the `index` getter
  returns `bigint`. `RANDOM_VERSION` is exported as a constant. **The seed is a
  `bigint`**, because wasm-bindgen maps `u64` to and from `BigInt` and nothing else
  does the whole range; the specification's `seed: Integer -- 0..2147483647`
  (`human-solving.allium` line 193) fits a `Number`, so a game writes
  `RandomStream.seeded(BigInt(seed))` and never meets the difference. The two-`u32`
  shape the ticket offers is rejected: it is a second way to say one number, and it is
  not the trait's. Errors cross as `JsError::new(&error.to_string())`, so the message is
  the `Display` text invariant 3 pins. A `#[wasm_bindgen(start)]` function installs a
  panic hook that writes the message through an imported `console.error` (question
  (f)), so the one panic decision 0008 rules out, if it ever happens, reaches the
  console with its message instead of `RuntimeError: unreachable`.
  Public engine values (`serde` feature) cross as plain objects through
  `serde_wasm_bindgen::to_value`, behind the crate's own `serde` feature that turns on
  `pawdoku/serde`.
- **`pawdoku-py`.** One `#[pyclass(name = "RandomStream", skip_from_py_object)]`,
  `#[derive(Clone, Debug)]`, holding a `Stream`; not `frozen`, because `next_draw`
  takes `&mut self`. A `#[pyclass]` must be `Send` and `Sync` (pyo3 rejects a
  non-`Send` payload unless the class is `unsendable`), and the enum is both because
  both library streams are, so the trait itself stays as it is. Two `#[staticmethod]`
  constructors, `RandomStream.seeded(seed: int)` and `RandomStream.replay(draws: list[float])`; a
  method `next_draw() -> float`; a property `index -> int`; the module attribute
  `RANDOM_VERSION`. A Python `int` converts to `u64` on the way in, and pyo3 raises
  `OverflowError` for a negative or too-large value before any Rust code runs. Errors
  are one exception class, `pawdoku.PawdokuError`, declared with `create_exception!`
  and raised with the `Display` text as its message; a `match` on the error's variants
  would break on a new one, and the text is the contract. The fake is exposed as
  `replay` so a Python test can script draws exactly as `tests/random.rs` does.
- **`pawdoku-cli`.** A `--seed <u64>` flag (clap parses the integer and reports
  overflow itself). Absent, the CLI is the one crate `deny.toml` lets source entropy: it
  depends on `getrandom` directly and fills a `u64`, then constructs
  `SeededStream::new(seed)`. Every run prints `seed`, `random_version` and the draws
  consumed beside its result, so the same command with `--seed <printed>` is the
  replay, on this machine or in Python. It depends on `getrandom` **directly**, because
  cargo-deny's `wrappers` allow only the direct parent (question (e)): a CLI on `rand`
  would make `rand` the direct parent of `getrandom` and fail the ban. The CLI never
  depends on `rand` in any case: it needs one `u64`, not a distribution.

**Step 5. Layout, recipes, `deny.toml`, pages.** Three sibling crates, each depending on
the engine and none on another, each `[lints] workspace = true` (question (f)). Every
crate under `crates/` is a workspace member automatically (`Cargo.toml` line 3,
`members = ["crates/*"]`), which is why several recipes need scoping the day the first
sibling lands (below).

```text
crates/pawdoku-wasm/
├── Cargo.toml          cdylib and rlib; wasm-bindgen, js-sys; serde-wasm-bindgen behind
│                       `serde = ["dep:serde-wasm-bindgen", "pawdoku/serde"]`;
│                       publish.workspace = true (npm, not crates.io);
│                       [package.metadata.wasm-pack.profile.release] wasm-opt = false
├── src/lib.rs          RandomStream, RANDOM_VERSION, the in-crate panic hook through an
│                       imported console.error (no console_error_panic_hook, question
│                       (f)), the engine's entry points as they arrive
└── tests/web.rs        wasm-bindgen-test in Node, run by `just wasm-test` (T15 decides
                        whether the first version needs it)
crates/pawdoku-py/
├── Cargo.toml          cdylib; pyo3 with abi3-py311 only (never extension-module,
│                       questions (a) and (f)); publish = false (PyPI, not crates.io)
├── pyproject.toml      [build-system] requires = ["maturin>=1.15,<2.0"]; [project] name
│                       "pawdoku", requires-python >= 3.11, dynamic = ["version"];
│                       [tool.maturin] python-source = "python", module-name =
│                       "pawdoku._pawdoku", locked = true; [tool.maturin.generate-ci.github]
│                       trusted-publishing = true, publishing-environment = "release"
├── src/lib.rs          RandomStream, PawdokuError, RANDOM_VERSION, the #[pymodule]
├── python/pawdoku/
│   ├── __init__.py     re-exports from ._pawdoku
│   ├── _pawdoku.pyi    the type stubs
│   └── py.typed
└── tests/test_random.py   pytest against `maturin develop`
crates/pawdoku-cli/
├── Cargo.toml          [[bin]] name = "pawdoku"; clap (derive), getrandom;
│                       publish.workspace = true until S02's follow-up says otherwise
├── src/main.rs         clap derive, a `solve` subcommand, output through a locked
│                       stdout handle so `print_stdout` never fires
└── tests/cli.rs        std::process::Command against the built binary, no new dev-dep
```

Generated paths that Git must ignore (T03 hand-back, `.gitignore`):
`crates/pawdoku-wasm/pkg/`, `crates/pawdoku-py/python/pawdoku/*.so` and `*.pyd` (what
`maturin develop` drops beside `__init__.py`), `.pytest_cache/` and `dist/` (maturin's
wheel output). `__pycache__/` is already there (line 2).

**Recipes, each a T00 follow-up on `main` (CONVENTIONS.md §11), in the order the gate
meets them:**

- `wasm-check` gains a third line, `cargo hack check -p pawdoku-wasm --feature-powerset
  --target wasm32-unknown-unknown --locked`. Not `wasm32v1-none`: wasm-bindgen needs
  `std`, and that target stays the core's proof alone.
- `coverage` narrows to `-p pawdoku` on its `llvm-cov nextest` line; the two `report`
  lines take no package filter. The floor's comment says
  `crates/pawdoku/src/**` (`Justfile` line 12) and the command says `--workspace`, so a
  bindings crate with Python or JavaScript tests would otherwise sink the number the
  floor measures. Invariant 7 names the core's lines, and the recipe should say so.
- `test` gains `--exclude pawdoku-py` on both lines (question (f): a Rust test binary
  that embeds the interpreter needs the loader to find `libpython`, and the crate's
  tests are Python's). A `py-test` recipe runs `maturin develop` then `pytest
  crates/pawdoku-py/tests`. Whether `py-test` joins `check` is T17's call; it costs the
  gate a wheel build.
- `export PYO3_PYTHON := justfile_directory() / ".pixi/envs/default/bin/python"` at the
  top of the `Justfile`, so `build`, `clippy`, `features`, `doc` and `test` compile
  `pawdoku-py` against the pinned 3.14 and never against a system Python older than the
  `abi3-py311` floor (question (f)).
- `wasm-build`, in three parts, in this order:
  1. **A version guard.** Read the `wasm-bindgen` version from `Cargo.lock` offline
     (for example `cargo pkgid --locked -p wasm-bindgen`, whose output ends in
     `@0.2.129`) and compare it with `wasm-bindgen --version` on `PATH`; on a mismatch,
     stop with a message naming both versions. The guard runs before wasm-pack, so a
     skewed pin fails with nothing fetched.
  2. **`wasm-pack build crates/pawdoku-wasm --mode no-install --target web --scope
     steven-cutting --release -- --locked`.** `--mode no-install` forbids wasm-pack's
     installer, and `-- --locked` reaches `cargo build`, so a stale lockfile fails here
     as it does in every other gate (invariant 8; CI's `wasm` job and
     `release-npm.yml` both run this recipe). With the guard passed, wasm-pack's first
     branch takes the pixi CLI on `PATH` and never consults its own cache (question
     (c), read from the installer's source in the first review round).
  3. Copy `docs/specs/*.allium` into `crates/pawdoku-wasm/pkg/specs/` and amend
     `pkg/package.json` (`files` gains `specs`, `exports` gains `"./specs/*":
     "./specs/*"`) so the game resolves a module through the package's `exports`, the
     way `tests/platform.ts` insists on. The amendment is a few lines of Python run
     through the pixi environment, so the recipe needs no Node locally; CI's publish job
     has Node for `npm publish`.

  `wasm-opt = false` under `[package.metadata.wasm-pack.profile.release]` until a size
  problem exists.
- `wheel`: `maturin build --release --manifest-path crates/pawdoku-py/Cargo.toml --out
  dist`; `locked = true` in `[tool.maturin]` makes every maturin build honour the
  workspace `Cargo.lock` (question (a)), and maturin sets
  `PYO3_BUILD_EXTENSION_MODULE` itself. `py-develop`: `maturin develop --manifest-path
  crates/pawdoku-py/Cargo.toml`, into the pixi environment, before `py-test`.
- `cli`: `cargo run -p pawdoku-cli --locked --`, the developer's tool.
- `check-clean` is unchanged: `pkg/`, `dist/` and the `.so` are ignored, so the recipes
  above leave the tree clean.
- `deny`: nothing. `cargo deny --locked check licenses bans sources` already covers a
  member the moment it joins, and question (f) showed no new licence. The
  `unused-wrapper` warning T02 kept for `getrandom` ends when `pawdoku-cli` depends on
  it; the one for `rand` stays, because the CLI never uses `rand`. The entry's
  `wrappers` can then go, leaving `rand` banned outright (T02 hand-back, one line).

**Pins, each a T00 follow-up in `pyproject.toml` and `pixi.lock` (decision 0011):**
`pixi search <name> --platform <platform>` on 2026-09-28 (source) found each on
conda-forge for both `linux-64` and `osx-arm64`, and none depends on `rust`, which is
the condition decision 0011 and CONVENTIONS.md §13 set ("The pixi environment must never
gain `rust`"):

- `maturin = "==1.15.0"` (build `py311h…_2`, a `noarch: python` package whose run
  dependencies are `python`, `tomli >=1.1.0` and `openssl`; the `py311` in the build
  string is the build's Python, not a pin, so it resolves against the manifest's
  `python = "3.14.*"`; T17 proves the solve with `pixi lock`).
- `pytest = "==9.1.1"` (`noarch: python`).
- `wasm-pack = "==0.15.0"` (no run dependencies).
- `wasm-bindgen-cli = "==0.2.129"`, held equal to the `wasm-bindgen` crate version in
  `Cargo.lock`, because wasm-bindgen requires the CLI and the crate to match and
  wasm-pack in its default mode downloads a CLI when none matches on `PATH`, which
  `check` must never do. Dependabot and Renovate (T18) then move the two together, or
  `wasm-build`'s guard fails on the mismatch before wasm-pack runs, and
  `--mode no-install` would refuse to fetch even without the guard, which is the right
  failure.
- `binaryen = "==121"` if `wasm-opt` is wanted, or `wasm-opt = false` under
  `[package.metadata.wasm-pack.profile.release]` in the crate's manifest so wasm-pack
  neither runs nor downloads it. T15 starts with `wasm-opt = false`, because the first
  version's size is not the problem it solves, and the pin is one line when it is.
- No `nodejs`: the local recipe amends `package.json` with Python, and the publish job
  installs Node with `actions/setup-node`, which also writes the registry `.npmrc`.

**Continuous integration** (`.github/workflows/ci.yml`, `.github/actions/setup`):

- `wasm` gains `just wasm-build` after `just wasm-check`, and uploads `pkg/` as an
  artefact for a reviewer to inspect. The job keeps `contents: read`; publishing is the
  release workflow's.
- **`windows-latest` is a job of its own, not a matrix entry.** CONVENTIONS.md §10 and
  the `ci.yml` comment say `windows-latest` joins `rust` the day the CLI exists. It
  cannot join as written: the setup action installs pixi with `platforms = ["linux-64",
  "osx-arm64"]`, the manifest comment says "win-64 never", and the `Justfile` is
  POSIX-only. So T16 adds a `windows` job that runs rustup's toolchain from
  `rust-toolchain.toml` and plain cargo (`cargo build -p pawdoku-cli --locked`,
  `cargo test -p pawdoku-cli --locked`, `cargo run -p pawdoku-cli --locked -- --help`)
  with no pixi and no `just`, joins `check`'s `needs`, and proves the one thing Ubuntu
  cannot: that the binary builds, links `getrandom` and runs where the game's players'
  machines are. Today's `windows-latest` is Windows Server 2025 with Rust 1.98.1 and
  rustup 1.29.1 preinstalled (question (e)); `rust-toolchain.toml` keeps the pin honest
  when the image moves.
- `rust`, `coverage`, `deny` and `documents` are unchanged; the recipes they run gain
  their scoping above.
- The `check` aggregate gains `windows` in `needs`, which is the one change branch
  protection never sees (§10).

**Release** (S02's `release.yml`, T13): one workflow file per registry, each triggered
by its crate's tag and carrying only the permission that registry needs, rather than one
file with four jobs and `if:` guards on the tag name: `release.yml` (`pawdoku-v*`,
`id-token: write`, crates.io) stays as T13 drafts it; `release-npm.yml`
(`pawdoku-wasm-v*`, `packages: write`, GitHub Packages); `release-pypi.yml`
(`pawdoku-py-v*`, `id-token: write`, PyPI, maturin's matrix); `release-cli.yml`
(`pawdoku-cli-v*`, `contents: write`, GitHub Releases). A tag can then only reach the
job it names, and a reviewer reads one registry's steps at a time. The ticket's Context
put the wheel matrix "in S02's `release.yml`"; the split is the deviation and this
paragraph is the reason. Each binding crate carries its own
`[package.metadata.release]` with `pre-release-replacements` into the root
`CHANGELOG.md` under a heading per crate (S02's open point, settled below), and
`shared-version` stays `false`.

**Pages that change**, each in the ticket that lands the crate:

- `docs/explanation/architecture.md` lines 27 ("today the only member") and 33-36
  ("Three siblings join when ticket S04 opens them … None exists yet."), and the table
  "What crosses each boundary" at lines 67-71, which gains a row per shipped crate
  saying what crosses in the trait's names (a `bigint` seed and a `Float64Array` script;
  an `int` seed and a `list[float]`; `--seed` or `getrandom`) and, for the wasm row,
  that the package carries `specs/`.
- `docs/project/repository-map.md` lines 11-13 ("one member today"), the tree at lines
  15-39 (a directory per crate, and `.github/workflows/release-*.yml`), and the
  responsibility table at lines 43-50 (a row per crate).
- `docs/reference/commands.md`: a row per new recipe (`wasm-build`, `wheel`, `py-test`,
  `cli`), the list at line 23 of recipes outside `just check`, and the count at line 15
  ("Nine recipes reach the network") if `wheel` or `wasm-build` ever fetches; as drafted
  neither does, because pixi and `cargo fetch` have already run.
- `docs/reference/testing.md`: where each crate's tests live (wasm-bindgen-test, pytest,
  `tests/cli.rs`), a stated exception to decision 0009's three places.
- `docs/reference/quality-gates.md`: the `windows` job and `wasm-build`.
- `docs/explanation/specifications.md` lines 80-84: the shared-truth paragraph becomes
  past tense the day `@steven-cutting/pawdoku-wasm` ships `specs/` and the game's test
  reads them (Step 6).
- `docs/manifest.yml` has no owner for "bindings" or "release"; `system_architecture`
  (architecture.md) owns the crossing and `command_reference` owns the recipes, so no
  page is added, and a consumer-facing how-to for the npm package is the game's to
  write, not this repository's.

The Goal's table, corrected on today's evidence:

| Crate | Toolchain | Registry | Randomness | Own `[lints]` | CI addition |
| --- | --- | --- | --- | --- | --- |
| `pawdoku-wasm` | wasm-bindgen 0.2.129 and js-sys, wasm-pack 0.15.0 `--target web`, serde-wasm-bindgen 0.6.5 behind `serde`; no console_error_panic_hook | GitHub Packages npm, `@steven-cutting/pawdoku-wasm`, `specs/` inside, `exports` amended after the build; no provenance until documented | `seeded(bigint)` or `replay(Float64Array)`; `SeededStream` is the default; no callback | no; `#[expect(clippy::missing_const_for_fn)]` on exports that could be `const` | third `wasm-check` line and `just wasm-build` in `wasm`; `release-npm.yml` |
| `pawdoku-cli` | clap 4.6.7 derive; binaries by `release-cli.yml` with taiki-e's two actions, SHA-pinned; cargo-dist rejected (question (e)) | GitHub Releases; crates.io optional | `--seed <u64>`, else `getrandom` (direct dependency), printed with the result and `RANDOM_VERSION` | no | a `windows` job, not a matrix entry; `release-cli.yml` |
| `pawdoku-py` | pyo3 0.29.2 `abi3-py311` (no `extension-module`), maturin 1.15.0 through pixi | PyPI, trusted publishing from the first release (a pending publisher) | `seeded(int)` or `replay(list[float])`; `SeededStream` is the default; no callable | no; `skip_from_py_object` on each `Clone` class | `PYO3_PYTHON` export, `test --exclude`, `py-test`; `release-pypi.yml` from `maturin generate-ci`, re-pinned and narrowed |

**Step 6. G's side: the transport and the restatement test.** Read at `78d03cdf` as the
ticket pins, with `git -C /Users/scutting/projects/pawdoku show 78d03cdf:<path>`; G's
HEAD on 2026-09-28 was `add73be7`, six commits on, and nothing below was re-derived
against it (T06's hand-back list already is). What G has today:

- `.npmrc` is the one line the Verification block quotes,
  `@steven-cutting:registry=https://npm.pkg.github.com`, so a second package in the
  scope needs no registry change.
- `package.json` is `"name": "pawdoku"`, private, with
  `"@steven-cutting/biscuit-games": "1.1.0"` as an exact pin, and `package-lock.json`
  resolves it from `https://npm.pkg.github.com/download/@steven-cutting/biscuit-games/…`.
  There is no wasm Vite plugin and no `optimizeDeps` entry in `vite.config.ts`; the test
  project is vitest 4.1.10 in `jsdom` over `tests/**/*.test.ts`.
- `tests/platform.ts` resolves a shipped file with
  `createRequire(resolve(process.cwd(), 'package.json')).resolve('@steven-cutting/biscuit-games/<subpath>')`,
  "Through the package's own `exports` subpaths rather than a path into `node_modules`,
  so a file the package renames fails here rather than reading as nothing", and throws
  when the resolved path is outside `node_modules`. This is why Step 5's `wasm-build`
  amends `exports` and not only `files`: a `files` entry ships the modules, and only an
  `exports` entry lets this resolver reach them.
- `tests/platformSpecs.test.ts` parses `surface|contract Name {` blocks and their
  `@guarantee|@invariant Name` comment bodies into a `Scope.Name` map, flattens the
  `--` comment lines, and holds each `RESTATED` clause equal to the platform's text; it
  also holds six config figures equal across the platform module, `src/lib/config.ts`,
  `pawdoku.allium` and every `docs/specs/**/*.allium`, and asserts the pinned version
  equals the installed one. `tests/restated.ts` is `RESTATED = []` today. Its header
  says the point: "It is meant to be over-sensitive. A reworded comma fails it".
- `.github/workflows/ci.yml` already carries `packages: read`, with the comment "Every
  job installs @steven-cutting/biscuit-games from GitHub Packages with the run's own
  token", so a second scoped package installs the same way once this repository's
  package grants G's repository read access (question (d)).
- `docs/specs/` at `78d03cdf` still holds seven engine modules beside `pawdoku.allium`;
  T06's hand-back (its items 1 to 13) has G delete them once T12 has landed, and its
  item 12 says "The restatement test itself, and how the engine's text reaches G (the
  wasm package shipping `specs/`), are S04's." This step is that answer.

**The G ticket, drafted for G's `tickets/` and separately authorised** (CONVENTIONS.md
§11: editing G is never this repository's work). Title: **"Consume the engine:
`@steven-cutting/pawdoku-wasm` pinned, its `specs/` held equal by test"**. It depends on
T15 having published a first version. Files:

- **`package.json` and `package-lock.json`.** `"@steven-cutting/pawdoku-wasm":
  "<exact version>"` under `dependencies`, an exact pin like the platform's. Nothing
  else in the manifest changes: the scope's registry line is already in `.npmrc`.
- **`tests/engine.ts`** (new), beside `tests/platform.ts` and shaped like it:
  `PACKAGE = '@steven-cutting/pawdoku-wasm'`, `enginePath(subpath)` and
  `engineFile(subpath)` through the same `createRequire` resolver with the same
  `node_modules` assertion, an `EngineModule` union of the nine module names, and an
  `EngineRestatement` interface with the same fields as `Restatement`.
- **`tests/engineRestated.ts`** (new), `ENGINE_RESTATED: readonly EngineRestatement[]`,
  empty until `pawdoku.allium` restates an engine clause, with the header comment
  `tests/restated.ts` carries adapted to the engine.
- **`tests/engineSpecs.test.ts`** (new). `it.each` over the nine modules asserts
  `enginePath('specs/<module>')` contains `node_modules`; one test per
  `ENGINE_RESTATED` clause holds the flattened body equal, reusing the `clauses` and
  `flatten` helpers (which move from `platformSpecs.test.ts` into a shared
  `tests/allium.ts`, or are duplicated if G prefers each test file self-contained); and
  a version test holds `package.json`'s pin equal to the installed package's `version`.
  No figure test: the engine's modules state no config figure the game restates today
  (the six figures are the platform's), and one is added the day a module does.
- **`docs/explanation/specifications.md`** in G (T06's item 5 rewrites it): the
  sentence on shared truth becomes "held equal by `tests/engineSpecs.test.ts` to the
  text `@steven-cutting/pawdoku-wasm` ships under `specs/`".
- **`docs/how-to/work-with-the-specs.md`** step 2 (T06's item 4): "and a restated engine
  clause is held to the package's text the same way".
- **`.agents/skills/spec-change/SKILL.md`** lines 10 and 13 (T06's item 9): the engine
  as the second owner whose text is held by test.
- Nothing in `.github/workflows/`: the reusable workflow installs with the run's token
  and `packages: read` is already granted.

Its proof: `npm ci` resolves the package from `npm.pkg.github.com` (the lockfile's
`resolved` URL says so); `npm test` is green with the nine path assertions; a deliberate
one-word edit to a copied clause in a scratch branch fails the restatement test, as it
does for the platform's. It closes T06's item 12 and, with T15, the shared-truth note
in this repository's `docs/explanation/specifications.md` lines 80-84 and CONVENTIONS.md
§1 fact 6 and §13.

**Step 7. The three follow-up drafts, and the order.** **`pawdoku-wasm` first**, as the
ticket proposed, and the evidence sharpened the reason: it is the only crate whose
absence costs something today. The shared-truth risk (CONVENTIONS.md §13, T06 item 12,
`specifications.md` lines 80-84) is open until the package ships `specs/`, the game has
no engine to call and its `RandomPort` still draws from `crypto` with no seeded stream
(G's `src/lib/ports/random.ts` at `78d03cdf`), and every question in Step 3 that turned
out harder than the ticket assumed (`exports`, the CLI pin, provenance, the panic hook)
sits in this crate. **The CLI second**: it is the cheapest crate (one binary, no
registry, a Windows job), the developer's tool for running the solver once a solver
exists, and its release path is settled by (e). **Python last**, when a consumer exists:
T19's S05 names it as the judge pipeline's home, which is the first concrete consumer,
and PyPI's pending publisher means nothing about publishing has to be prepared before
then. Each draft below is for a file that is not created here, because this spike's
Files touched lists no ticket file; the agent that opens one copies its block and adds
the index row. Each depends on S04 and, for the release workflow, on T13's
`[workspace.metadata.release]` table having landed or being written in the same pull
request.

**T15, drafted for `tickets/T15-wasm-bindings.md`.**

```yaml
---
id: T15
title: "WebAssembly bindings: pawdoku-wasm, @steven-cutting/pawdoku-wasm with specs/ inside"
status: open
depends_on: [S04, T13]
parallel_with: [T16, T17]
branch: ticket/t15-wasm-bindings
estimated_size: M
---
```

> **Context.** S04 (hand-back notes, Steps 3 to 6) designed the crate: wasm-bindgen
> 0.2.129 and js-sys, wasm-pack 0.15.0 `--target web`, the package
> `@steven-cutting/pawdoku-wasm` on GitHub Packages with the nine specification modules
> under `specs/` and an `exports` map the game's resolver can reach. The lint table was
> proved on a probe crate; the `exports` shape, the CLI pin and the panic hook were each
> settled there. The game is waiting for it: T06's item 12 and CONVENTIONS.md §13 name
> the shared-truth risk this crate ends.
>
> **Goal.** `just wasm-build` produces `crates/pawdoku-wasm/pkg/` with the JavaScript,
> the `.wasm`, the types, `specs/*.allium` and an amended `package.json`; `just check`
> is green with the crate in the workspace; `release-npm.yml` publishes
> `@steven-cutting/pawdoku-wasm 0.1.0` on the tag `pawdoku-wasm-v0.1.0`; the game's
> repository can install it in CI.
>
> **Non-goals.** Engine entry points beyond `RandomStream` and `RANDOM_VERSION`: the
> crate exports what the core has, and grows with it. npm provenance (question (d):
> undocumented for GitHub Packages). `wasm-opt` (off until a size problem exists).
> The game's side (the G ticket in S04's Step 6). A `wasm32v1-none` check of this crate.
>
> **Files touched.**
>
> - **`crates/pawdoku-wasm/Cargo.toml`** (new). `crate-type = ["cdylib", "rlib"]`;
>   `publish.workspace = true`; `[lints] workspace = true`; dependencies `pawdoku`,
>   `wasm-bindgen`, `js-sys` and optional `serde-wasm-bindgen` behind
>   `serde = ["dep:serde-wasm-bindgen", "pawdoku/serde"]`, every version
>   `workspace = true`; `[package.metadata.wasm-pack.profile.release] wasm-opt = false`
>   with a comment; `[package.metadata.release]` with the CHANGELOG replacements under
>   this crate's heading (S02's shape, `pawdoku-wasm-v{{version}}`).
> - **`crates/pawdoku-wasm/src/lib.rs`** (new). The `RandomStream` class of S04's Step 4
>   (`seeded(bigint)`, `replay(Float64Array)`, `nextDraw()`, the `index` getter),
>   holding the private `Stream` enum, with the in-crate `const _` bounds assertion
>   beside it, `RANDOM_VERSION`, the `#[wasm_bindgen(start)]` hook writing panics
>   through an imported `console.error`, `#[expect(clippy::missing_const_for_fn, reason = …)]` on
>   any export that could be `const`, a doctest on each public item.
> - **`crates/pawdoku-wasm/tests/web.rs`** (new, optional): wasm-bindgen-test in Node
>   for `seeded(0n)`'s first draw equal to `tests/random.rs`'s golden value, run by a
>   `wasm-test` recipe. T15 decides whether the first version carries it; if not, the
>   golden check moves to G's test on the published package.
> - **`Cargo.toml`** (T02 hand-back, with the decision 0007 note for each): `wasm-bindgen
>   = "0.2.129"`, `js-sys = "0.3.106"`, `serde-wasm-bindgen = "0.6.5"` and, for tests,
>   `wasm-bindgen-test = "0.3.79"` in `[workspace.dependencies]`; every licence is
>   already allowed (question (f)).
> - **`Justfile`** (T00 follow-up on `main`): the third `wasm-check` line
>   (`-p pawdoku-wasm` on `wasm32-unknown-unknown` only); `coverage` narrowed to `-p
>   pawdoku`; the `wasm-build` recipe of S04's Step 5 (the `Cargo.lock`-against-`PATH`
>   version guard, then wasm-pack with `--mode no-install --target web --scope
>   steven-cutting --release -- --locked`, then the specs copy and the `package.json`
>   amendment through `scripts/wasm_pkg.py` run by the pixi Python, adding `files: specs`, and
>   `exports` with `"."` (`types`, `default`), `"./pawdoku_wasm_bg.wasm"` and
>   `"./specs/*"`); a `wasm-test` recipe if `tests/web.rs` exists. `check-clean` is
>   unchanged.
> - **`pyproject.toml` and `pixi.lock`** (T00 follow-up on `main`): `wasm-pack =
>   "==0.15.0"` and `wasm-bindgen-cli = "==0.2.129"`, the second held equal to
>   `Cargo.lock`'s `wasm-bindgen`, each with a one-line comment; then `pixi lock`.
> - **`scripts/wasm_pkg.py`** (new): reads `pkg/package.json`, adds the three things
>   above and, if GitHub Packages rejects the inherited `repository` (question (d)),
>   rewrites it with `.git`. Ten lines, stdlib only, no `bg-` script.
> - **`.gitignore`** (T03 hand-back): `crates/pawdoku-wasm/pkg/`.
> - **`.github/workflows/ci.yml`**: the `wasm` job gains `just wasm-build` after `just
>   wasm-check` and uploads `crates/pawdoku-wasm/pkg/` with `actions/upload-artifact`
>   (the SHA `ci.yml` pins), `retention-days: 7`. Permissions stay `contents: read`.
> - **`.github/workflows/release-npm.yml`** (new). `on: push: tags:
>   ['pawdoku-wasm-v*']`; top-level `permissions: contents: read`; one job `publish` with
>   `environment: release` and `permissions: contents: read, packages: write`;
>   concurrency on the tag, no cancelling; `actions/checkout` (pinned,
>   `persist-credentials: false`), the composite setup action, `just check-toolchain`,
>   T13's two guards (the commit is on `main`; the tag equals `pawdoku-wasm-v` plus the
>   manifest version), `just wasm-build`, `actions/setup-node` (pinned) with
>   `registry-url: https://npm.pkg.github.com` and `scope: '@steven-cutting'`, an
>   existence check against the registry that skips a version already published, then
>   `npm publish` in `crates/pawdoku-wasm/pkg` with `NODE_AUTH_TOKEN: ${{
>   secrets.GITHUB_TOKEN }}`. No `id-token`, no `--provenance` (question (d)).
> - **`docs/explanation/architecture.md`** lines 27 and 33-36, and the crossing table
>   (S04 Step 5). **`docs/project/repository-map.md`**: the crate, `scripts/wasm_pkg.py`
>   and the workflow. **`docs/reference/commands.md`**: `wasm-build` (and `wasm-test`)
>   rows; both outside `just check`. **`docs/reference/testing.md`**: where this crate's
>   tests live. **`docs/reference/quality-gates.md`**: the `wasm` job's new step.
>   **`docs/explanation/specifications.md`** lines 80-84: the shared-truth paragraph,
>   present tense, naming the package and G's test. **`docs/operations/maintenance.md`**:
>   a "Cut a wasm release" entry beside T13's, and the package-settings steps.
> - **`CHANGELOG.md`**: a `pawdoku-wasm` heading under `[Unreleased]` (S02's open point,
>   settled by S04: one root CHANGELOG, one heading per crate).
> - **`tickets/README.md`**: the T15 row.
>
> **Steps.**
>
> 1. Land the crate, the follow-ups and the pages through their pull requests, with
>    `just check` green and the `wasm` job's artefact inspected: `pkg/package.json`
>    carries `files`, `exports` and `repository`; `pkg/specs/` holds nine modules.
> 2. Prove the lint table on the real crate: `just clippy` green, and a planted
>    `unsafe {}` in `src/lib.rs` failing it, quoted.
> 3. Prove `wasm-build`'s two refusals, each quoted: a shim `wasm-bindgen` earlier on
>    `PATH` that reports another version stops the recipe at the guard before wasm-pack
>    runs; and a `Cargo.toml` edit that forces a re-resolve fails on `-- --locked`.
> 4. Prove the resolver: in a scratch directory, `npm install ./crates/pawdoku-wasm/pkg`,
>    then resolve `@steven-cutting/pawdoku-wasm/specs/sudoku.allium` and the `.wasm`
>    subpath with `import.meta.resolve` from a one-line Node script, and check that both
>    paths fall inside `node_modules`.
> 5. Release 0.1.0 as T13 does: a `release/pawdoku-wasm-v0.1.0` branch, `cargo release
>    -p pawdoku-wasm --no-tag --execute`, a pull request, then `cargo release tag -p
>    pawdoku-wasm --execute` on `main` and the push of the tag.
> 6. After the first publish: make the package public, add the game's repository under
>    "Manage Actions access" with Read (question (d)), and read both settings back with
>    `gh api`.
>
> **Authorisation**, each asked for on its own (CONVENTIONS.md §11): each push and pull
> request; the tag push; approving the `release` environment's deployment, which is the
> publish; changing the package's visibility; granting the game's repository access.
>
> **Acceptance and proof.** `just check` green; `gh api
> /users/steven-cutting/packages/npm/pawdoku-wasm/versions` lists 0.1.0; the
> `release-npm.yml` run is green with no token beyond the run's own; the scratch
> install resolves `specs/sudoku.allium` and the `.wasm` through `exports`; both
> `wasm-build` refusals of step 3 fail as stated; the G ticket can proceed.
>
> **Open points.** Whether `tests/web.rs` ships in 0.1.0 or the golden check lives only
> in G. Whether the inherited `repository` (no `.git`) is accepted by GitHub Packages
> (question (d), verified at the first publish). The `wasm-opt` decision when size
> matters.

**T16, drafted for `tickets/T16-cli.md`.**

```yaml
---
id: T16
title: "Command line: pawdoku-cli, --seed or getrandom, a Windows job, binaries on GitHub Releases"
status: open
depends_on: [S04, T13]
parallel_with: [T15, T17]
branch: ticket/t16-cli
estimated_size: S
---
```

> **Context.** S04 (hand-back notes) designed the crate: clap 4.6.7 with `derive`,
> `getrandom` 0.4.3 as a direct dependency because cargo-deny's `wrappers` allow only the
> direct parent, output through a locked stdout handle so `print_stdout` never fires, and
> cargo-dist rejected in favour of S02's cargo-release flow plus two pinned actions
> (question (e)). A probe binary proved clap's derive and `getrandom::u64()` under the
> lint table. It is the developer's tool for running the engine on a grid from a
> terminal; until the solver exists, its one subcommand exercises the randomness
> boundary.
>
> **Goal.** `just cli -- draw --seed 7` prints the seed, `random_version` and the first
> draws; without `--seed` it prints a seed from the operating system that reproduces the
> run. A `windows` job builds and runs the binary in CI. `release-cli.yml` attaches
> binaries for Linux, macOS and Windows to a GitHub Release on `pawdoku-cli-v*`.
>
> **Non-goals.** The grid-string input format: `sudoku.allium` decides it (S04's Open
> points), and the `solve` subcommand lands with the solver. crates.io for the CLI
> (S02's open question; `publish.workspace = true` until then). Installers, Homebrew,
> a Windows signature. Running `just` or pixi on Windows: the manifest says "win-64
> never".
>
> **Files touched.**
>
> - **`crates/pawdoku-cli/Cargo.toml`** (new). `[[bin]] name = "pawdoku"`;
>   `publish.workspace = true`; `[lints] workspace = true`; `pawdoku`, `clap` with
>   `derive`, `getrandom`, all `workspace = true`; `[package.metadata.release]` with the
>   CHANGELOG replacements under this crate's heading.
> - **`crates/pawdoku-cli/src/main.rs`** (new): clap derive with a `draw` subcommand
>   (`--seed <u64>`, `--count <n>`), `getrandom::u64()` when `--seed` is absent, every
>   line through `writeln!` on `std::io::stdout().lock()`, exit codes from
>   `std::process::ExitCode`.
> - **`crates/pawdoku-cli/tests/cli.rs`** (new): `std::process::Command` on
>   `env!("CARGO_BIN_EXE_pawdoku")`: `--seed 7` prints `seed = 7` and the golden first
>   draw from `tests/random.rs`; two runs without `--seed` print different seeds; each
>   printed seed reproduces its run.
> - **`Cargo.toml`** (T02 hand-back, decision 0007 notes): `clap = { version = "4.6.7",
>   features = ["derive"] }` and `getrandom = "0.4.3"` in `[workspace.dependencies]`.
> - **`deny.toml`** (T02 hand-back): drop `wrappers` from the `rand` entry, leaving `rand`
>   banned outright, so its `unused-wrapper` warning ends with `getrandom`'s.
> - **`Justfile`** (T00 follow-up on `main`): the `cli` recipe (`cargo run -p pawdoku-cli
>   --locked -- "$@"`).
> - **`.github/workflows/ci.yml`**: a `windows` job on `windows-latest` with no pixi and
>   no `just`: checkout (pinned, `persist-credentials: false`), `rustup show` (installs
>   the pin from `rust-toolchain.toml`), `cargo build -p pawdoku-cli --locked`, `cargo
>   test -p pawdoku-cli --locked`, `cargo run -p pawdoku-cli --locked -- draw --seed 7`;
>   `timeout-minutes: 15`; added to `check`'s `needs`. The comment at lines 28-33 is
>   rewritten: the day has come, and the job is its own because the toolchain manifest
>   is POSIX-only.
> - **`.github/workflows/release-cli.yml`** (new). `on: push: tags: ['pawdoku-cli-v*']`;
>   top-level `permissions: contents: read`; a `release` job (`environment: release`,
>   `permissions: contents: write`) running T13's two guards then
>   `taiki-e/create-gh-release-action` (SHA of v1.11.0) with the CHANGELOG section; a
>   `binaries` job (`needs: release`, `permissions: contents: write`) over a matrix of
>   `ubuntu-latest` (`x86_64-unknown-linux-gnu`), `macos-latest`
>   (`aarch64-apple-darwin`) and `windows-latest` (`x86_64-pc-windows-msvc`), each
>   running `taiki-e/upload-rust-binary-action` (SHA of v1.30.2) with `bin: pawdoku`,
>   `package: pawdoku-cli`, `archive: pawdoku-cli-$target`, `locked: true`, `tar: unix`,
>   `zip: windows`, `checksum: sha256`, `token: ${{ github.token }}`. No installer.
> - **`docs/explanation/architecture.md`**: the CLI row of the crossing table and the
>   `getrandom` sentence. **`docs/project/repository-map.md`**: the crate and the
>   workflow. **`docs/reference/commands.md`**: the `cli` row. **`docs/reference/
>   quality-gates.md`**: the `windows` job. **`docs/reference/testing.md`**:
>   `tests/cli.rs`. **`docs/operations/maintenance.md`**: "Cut a CLI release".
> - **`CHANGELOG.md`**: a `pawdoku-cli` heading. **`tickets/README.md`**: the T16 row.
>
> **Steps.** Land the crate and the follow-ups with `just check` green and the
> `windows` job green; `just deny` prints no `unused-wrapper` warning; release 0.1.0 as
> T13 does, on `pawdoku-cli-v0.1.0`; download one archive from the Release and run the
> binary with `--seed 7`.
>
> **Authorisation**, each on its own: pushes and pull requests; the tag push; approving
> the `release` deployment, which creates the Release and its assets.
>
> **Acceptance and proof.** `just check` green; `check`'s `needs` names `windows` and
> branch protection still names `check` alone; the Release holds three archives and
> three checksums; `cargo deny check bans` in a scratch copy with `rand` added to the
> CLI fails on `getrandom`'s ban (the negative control for the direct-parent rule).
>
> **Open points.** The grid-string format (to `sudoku.allium`). Whether the CLI goes to
> crates.io, and with it `[package.metadata.binstall]` (the archive name already fits
> binstall's default templates).

**T17, drafted for `tickets/T17-python-bindings.md`.**

```yaml
---
id: T17
title: "Python bindings: pawdoku-py, one abi3 wheel per platform, PyPI by trusted publishing"
status: open
depends_on: [S04, T13, S05]
parallel_with: [T15, T16]
branch: ticket/t17-python-bindings
estimated_size: M
---
```

> **Context.** S04 (hand-back notes) designed the crate: pyo3 0.29.2 with `abi3-py311`
> and never the deprecated `extension-module` feature, maturin 1.15.0 from conda-forge
> through pixi, a `pyproject.toml` beside the crate that pixi never reads (question (b)),
> `PYO3_PYTHON` pointed at the pixi interpreter, Python tests by pytest, and a release
> workflow generated by `maturin generate-ci github` with `trusted-publishing = true`
> then re-pinned and narrowed by hand (question (a)). A probe crate proved pyo3's macros
> under the lint table. Its first consumer is S05's development-time judge pipeline
> (T19), which wants `RunResult`, `PricedRun`, `TackledRun` and `AssessmentSummary`
> serialisable; this ticket opens when that consumer is ready, which is why it depends on
> S05.
>
> **Goal.** `just wheel` builds an abi3 wheel; `just py-test` passes pytest against
> `maturin develop`; `release-pypi.yml` publishes `pawdoku 0.1.0` to PyPI from a
> pending trusted publisher, with no token at any point.
>
> **Non-goals.** A Rust test binary in the crate (question (f)). Free-threaded wheels
> beyond what maturin's generator emits. A conda-forge package. The judge pipeline
> itself (S05).
>
> **Files touched.**
>
> - **`crates/pawdoku-py/Cargo.toml`** (new). `crate-type = ["cdylib"]`; `publish =
>   false`; `[lints] workspace = true`; `pawdoku` (with `serde` when S05 needs it) and
>   `pyo3` with `abi3-py311`, `workspace = true`; `[package.metadata.release]` with the
>   CHANGELOG replacements and a `pre-release-replacements` entry that moves the
>   version in `pyproject.toml` too, unless `dynamic = ["version"]` reads Cargo's.
> - **`crates/pawdoku-py/pyproject.toml`** (new): `[build-system] requires =
>   ["maturin>=1.15,<2.0"]`, `build-backend = "maturin"`; `[project] name = "pawdoku"`,
>   `requires-python = ">=3.11"`, `dynamic = ["version"]`, licence and classifiers;
>   `[tool.maturin] python-source = "python"`, `module-name = "pawdoku._pawdoku"`,
>   `locked = true`; `[tool.maturin.generate-ci.github] trusted-publishing = true`,
>   `publishing-environment = "release"`. No `[tool.pixi]` table, on purpose, with a
>   comment saying so and why (question (b)).
> - **`crates/pawdoku-py/src/lib.rs`** (new): the `RandomStream` class of S04's Step 4
>   (`seeded(int)`, `replay(list[float])`, `next_draw()`, `index`), holding the private
>   `Stream` enum, with the in-crate `const _` bounds assertion beside it, `PawdokuError`
>   from `create_exception!`, `RANDOM_VERSION`, `#[pyclass(skip_from_py_object)]` on
>   every `Clone` class, the `#[pymodule]`.
> - **`crates/pawdoku-py/python/pawdoku/__init__.py`, `_pawdoku.pyi`, `py.typed`**
>   (new). **`crates/pawdoku-py/tests/test_random.py`** (new): the golden first draw of
>   seed zero, `replay` exhaustion raising `PawdokuError` with the `Display` text, an
>   out-of-range script raising on construction.
> - **`Cargo.toml`** (T02 hand-back, decision 0007 note): `pyo3 = { version = "0.29.2",
>   features = ["abi3-py311"] }`. Licences already allowed (question (f)).
> - **`Justfile`** (T00 follow-up on `main`): `export PYO3_PYTHON :=
>   justfile_directory() / ".pixi/envs/default/bin/python"`; `test` gains `--exclude
>   pawdoku-py` on both lines; `coverage` is already `-p pawdoku` after T15 (or becomes
>   so here); new `wheel`, `py-develop` and `py-test` recipes.
> - **`pyproject.toml` and `pixi.lock`** (T00 follow-up on `main`): `maturin =
>   "==1.15.0"` and `pytest = "==9.1.1"` under `[tool.pixi.dependencies]`, then `pixi
>   lock`; the solve proves `maturin`'s `python` dependency is satisfied by `3.14.*`.
> - **`.gitignore`** (T03 hand-back): `crates/pawdoku-py/python/pawdoku/*.so`, `*.pyd`,
>   `.pytest_cache/`, `dist/`.
> - **`.github/workflows/ci.yml`**: the `rust` job gains `just py-test` after `just test`
>   (the pixi environment has maturin and pytest; the step is the one wheel build in the
>   gate). `just check`'s recipe list in `pyproject.toml` gains `py-test` if the
>   maintainer wants it in the local gate too.
> - **`.github/workflows/release-pypi.yml`** (new): the output of `maturin generate-ci
>   github -m crates/pawdoku-py/Cargo.toml`, then edited: trigger narrowed to `push:
>   tags: ['pawdoku-py-v*']`, every `@vN` action pin replaced by a SHA, the platform
>   matrix cut to what the house supports (linux x86_64 and aarch64, macOS aarch64,
>   Windows x64) unless S05 wants more, `sdist` kept, T13's two guards added before the
>   build, the release job's `environment: release`, `id-token: write` and `uv publish
>   --trusted-publishing always` kept, `contents: write` and `attestations: write`
>   removed unless attestation is wanted. The edited file is recorded against the
>   generator's output in the notes, so the next `generate-ci` diff is readable.
> - **`docs/explanation/architecture.md`**: the Python row. **`docs/project/
>   repository-map.md`**: the crate and the workflow. **`docs/reference/commands.md`**:
>   `wheel`, `py-develop`, `py-test`. **`docs/reference/testing.md`**: pytest and why no
>   Rust tests. **`docs/reference/quality-gates.md`**: the `py-test` step.
>   **`docs/decisions/0004-hook-runner-and-checkers.md`**: the sentence "installed and
>   run through `uv`" is superseded by 0011; a one-line note pointing at it, or a T09
>   follow-up. **`docs/operations/maintenance.md`**: "Cut a Python release".
> - **`CHANGELOG.md`**: a `pawdoku-py` heading. **`tickets/README.md`**: the T17 row.
>
> **Steps.** Land the crate and the follow-ups with `just check` green; `just wheel`
> produces `pawdoku-0.1.0-cp311-abi3-<platform>.whl`; configure the pending publisher
> on PyPI (owner `steven-cutting`, repository `libpawdoku`, workflow
> `release-pypi.yml`, environment `release`, project `pawdoku`) before the tag; release
> as T13 does on `pawdoku-py-v0.1.0`; `pip install pawdoku` in a fresh virtual
> environment and `python -c "import pawdoku; print(pawdoku.RANDOM_VERSION)"`.
>
> **Authorisation**, each on its own: pushes and pull requests; the PyPI account and the
> pending publisher; the tag push; approving the `release` deployment, which is the
> publish; the project name `pawdoku` on PyPI, which S02's name check did not cover.
>
> **Acceptance and proof.** `just check` green; the PyPI project page lists 0.1.0 with
> the trusted-publisher badge; the workflow run used no secret; `pytest` passes locally
> and in CI.
>
> **Open points.** Whether `py-test` joins `just check` locally or only CI. Whether the
> free-threaded `cp314t` wheel maturin's generator adds is kept. Whether the PyPI name
> `pawdoku` is free (checked on the day).

**Step 8.** `status: done`, and one commit on the ticket branch. Nothing is pushed.

**The Verification block.** Run after the notes were written, with
`PATH="$HOME/.cargo/bin:$PWD/.tools/bin:$PATH"` so that `cargo hack` is the `.tools/bin`
binary (CONVENTIONS.md §13):

```text
$ grep -c 'wrappers' deny.toml
2
$ grep -n 'unsafe_code' Cargo.toml
24:unsafe_code = "forbid"
$ cargo hack check -p pawdoku --feature-powerset --target wasm32-unknown-unknown --locked 2>&1 | tail -1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
$ git -C /Users/scutting/projects/pawdoku show 78d03cdf:.npmrc
@steven-cutting:registry=https://npm.pkg.github.com
$ git status --porcelain
 M tickets/S04-bindings-and-cli.md
```

The first `cargo hack` run of the session, before anything was cached, printed
`Compiling serde_derive v1.0.229`, `Checking pawdoku v0.1.0` and `Finished … in 1.49s`;
the line above is the re-run over the same `target/`.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `S04-bindings-and-cli`, not the
  `ticket/s04-bindings-and-cli` the frontmatter names, as in earlier tickets. The
  `branch:` field is left as written.
- **Nine modules, not seven.** The ticket was written on 2026-09-23 with seven engine
  modules; T12 added `board.allium` and T19 `generation.allium`. Step 2, step 3(c) and
  the wasm-pack copy in the Context read nine here, and `specifications.md` line 29
  already agrees.
- **Four of the Context's premises changed**, each recorded where it was found:
  - pyo3's `extension-module` feature is deprecated; maturin sets
    `PYO3_BUILD_EXTENSION_MODULE` (question (a)). The first design of question (f)
    assumed the feature and was corrected in the same step.
  - `maturin generate-ci github` has no template file and publishes with `uv publish`
    and a token unless `trusted-publishing = true` (question (a)).
  - wasm-pack lives at `wasm-bindgen/wasm-pack`, not `drager`; wasm-bindgen at
    `wasm-bindgen/wasm-bindgen`; `console_error_panic_hook` is archived and is not used
    (questions (c) and (f)).
  - cargo-dist is not adopted for the CLI's binaries (question (e)): it owns
    `release.yml`, checks it on every pull request, and publishes nothing to crates.io.
- **Two of the Context's attributions were to the wrong record.** Decision 0008 does not
  itself state `panic = "unwind"` (only `Cargo.toml` line 77 does) nor the "no clock,
  threads, filesystem or environment" sentence (0003 and `AGENTS.md` do); decision 0011
  says nothing about maturin (D02's notes do, and 0004 still says `uv`). The guarantees
  hold in the tree; the wording gaps go to T09 (Handed back).
- **The Goal's table is left as written**, as the record of what was believed on
  2026-09-23. Step 5 carries the corrected table: both "Own `[lints]`" cells are "no".
- **The release shape differs from the ticket's.** The Context puts maturin's wheel
  matrix "in S02's `release.yml`"; Step 5 recommends one workflow per registry, each
  with its own tag pattern and least permission, and says why.
- **`windows-latest` is a job of its own**, not the matrix entry §10 and the `ci.yml`
  comment foresaw, because the pixi manifest and the `Justfile` are POSIX-only (Step 5).
- **The live-callback shape is not recommended.** Step 4 was asked to design it for
  wasm and Python; the trait's `#[non_exhaustive]` error enum has no variant for a
  failing callback, and the specification asks for a seed and a script. It is an open
  point below rather than a design.
- **More than the ticket named was run**, each with the maintainer's authorisation of
  2026-09-28: three probe crates (`probe-wasm`, `probe-py`, `probe-cli`) built and
  linted in the session's scratch directory, including `cargo test --no-run` and a
  release `wasm32-unknown-unknown` build, with the cargo fetches they needed;
  `pixi search` for the six conda-forge packages Step 5 pins; and the registry API
  reads. The documentation research for questions (a) to (e) was fanned out to three
  sub-agents; every fact they returned is cited above by its own URL, date and class,
  and their reports stay in the gitignored `ai_tmp/`, which no sentence above cites.
  The first review round added read-only `gh api` reads of wasm-pack `v0.15.0`'s
  `src/install/mod.rs`, `src/install/mode.rs` and `src/command/build.rs`, and of
  binary-install `v0.4.1`'s `src/lib.rs`, each authorised on 2026-09-28 and cited in
  question (c).
- **Not verified, stated in place and gathered here:** whether `npm publish
  --provenance` works against GitHub Packages (undocumented either way); whether
  GitHub Packages accepts the inherited `repository` without `.git`; wasm-pack's
  effective release `wasm-opt` default; whether Vite 8.2.1 still lowers
  `import.meta.url` when pre-bundling a `web`-target dependency; and the pixi solve of
  `maturin`, `pytest`, `wasm-pack` and `wasm-bindgen-cli` against the manifest, which
  `pixi lock` would prove and this ticket may not run (it rewrites `pixi.lock`). Each
  is a first step of the ticket it belongs to.
- **Two files, not one.** Files touched first named this ticket alone, and the first
  commit touched only it. S01, S02 and S03 each set their `tickets/README.md` row to
  `done` after the first review round, so the row is set to `done` in this ticket's
  first review round, and the Files touched table carries it.
- **The first review round (Codex adversarial, 2026-09-28) corrected two designs.**
  - Step 4 had each binding's class hold a `Box<dyn RandomStream>` (`+ Send + Sync` in
    Python). A trait object keeps only the bounds written on it, and `Clone` cannot be
    written there, so neither class could be `Clone` or `Debug` and the wasm one was not
    `Send + Sync`: invariant 3 broken for the binding's own public type. The probes had
    used other payloads, so they never showed it. Each class now holds a private
    `Stream` enum over `SeededStream` and `ReplayStream`, lent to the engine through
    `as_dyn`, with an in-crate compile-time assertion of the class's bounds; the T15
    and T17 drafts carry both.
  - Step 5's `wasm-build` ran wasm-pack in its default mode, which installs a CLI when
    none on `PATH` matches, and without `--locked`, so a skewed pin could fetch a tool
    outside pixi and a stale lockfile was rewritten rather than refused: invariant 8 broken in the CI `wasm` job
    and `release-npm.yml`, and the claims that nothing is downloaded and that a mismatch
    fails loudly were untrue. The recipe now guards the `PATH` CLI against
    `Cargo.lock`'s version, runs `--mode no-install` and passes `-- --locked`; question
    (c) cites the installer's source for why that suffices; T15 proves both refusals.

### Handed back

- **To T15, T16 and T17** (drafted above, T15 ready now, T16 when a solver or the
  `draw` subcommand justifies a binary, T17 when S05 opens): the crates, the recipes,
  the pins, the workflows and the pages, each listed in its draft.
- **To T00 follow-ups on `main`** (frozen files, CONVENTIONS.md §11): in the
  `Justfile`, the third `wasm-check` line, `coverage -p pawdoku`, `test --exclude
  pawdoku-py`, the `PYO3_PYTHON` export, and the `wasm-build`, `wasm-test`, `wheel`,
  `py-develop`, `py-test` and `cli` recipes; in `pyproject.toml` and `pixi.lock`, the
  pins `wasm-pack = "==0.15.0"`, `wasm-bindgen-cli = "==0.2.129"`, `maturin =
  "==1.15.0"` and `pytest = "==9.1.1"`, and `py-test` in the recipe list if it joins
  `check`. Nothing in `tools.txt`: every new tool is on conda-forge and none depends on
  `rust`.
- **To T02:** `wasm-bindgen`, `js-sys`, `serde-wasm-bindgen`, `wasm-bindgen-test`,
  `pyo3`, `clap` and `getrandom` in `[workspace.dependencies]`, each with its decision
  0007 note in the draft that needs it; the `rand` entry's `wrappers` dropped from
  `deny.toml` when the CLI lands; no licence entry (question (f)).
- **To T03:** the `.gitignore` lines for `pkg/`, the `.so` and `.pyd` files,
  `.pytest_cache/` and `dist/`.
- **To T09:** decision 0008's Decision list should say `panic` stays `unwind` and why,
  and carry the no-clock sentence it is cited for; decision 0004's "installed and run
  through `uv`" is superseded by 0011 and should say so; and the T13 draft's
  `0012-releases.md` collides with T19's 0012, so the release record is 0013 or later.
- **To T13:** S02's "To S04" bullet, answered. Each binding crate carries its own
  `[package.metadata.release]` with replacements into the one root `CHANGELOG.md`, under
  a heading per crate; tags stay `<crate>-v<version>`; registry mechanics are one
  workflow per registry (Step 5), all sharing the `release` environment, whose
  deployment rule must therefore admit the four tag patterns and not `pawdoku-v*`
  alone; the two guards T13 writes are reused verbatim by the three siblings, so T13
  might place them in a composite action.
- **To T18:** the `wasm-bindgen-cli` pixi pin must move with `Cargo.lock`'s
  `wasm-bindgen`, or `wasm-build`'s guard fails on the version mismatch before wasm-pack
  runs; the three new workflows bring action pins to watch; `crates/pawdoku-py/pyproject.toml` is a second Python
  manifest the updater may try to read.
- **To T19 and S05:** the Python crate is the judge pipeline's home, T17 depends on S05,
  and S05 should say which engine values it needs serialisable so T17 turns on `serde`.
- **To the G ticket** (drafted in Step 6, separately authorised): the pin, `tests/
  engine.ts`, `tests/engineRestated.ts`, `tests/engineSpecs.test.ts`, and T06's items
  4, 5 and 9 as they touch the engine.
- **To the specifications:** the CLI's input format (Open points), and, if a live
  randomness callback is ever wanted, a `RandomError` variant for a failing source,
  which is a spec sentence in `human-solving.allium`'s boundary wording before it is a
  T02 change.
- **To the index:** nothing. S04's row in `tickets/README.md` was set to `done` in the
  first review round (Deviations).

### Open points settled

None of this ticket's own. The one question below stays open; what was learned about it
is recorded there. Two questions the ticket did not list were settled on the way and are
recorded in their steps: the live callback is not offered (Step 4), and neither bindings
crate needs its own `[lints]` table (question (f)).

## Open points

- Whether the CLI reads the grid-string format `sudoku.allium` describes or a JSON
  shape shared with the wasm crate; a specification question for `sudoku.allium`'s
  open points, not decided here. Learned on 2026-09-28: `sudoku.allium` at `c1a1415`
  describes no grid string and carries no `open question` block (grep for `string` and
  `open question`), so the question has nothing to hang on yet. T16's `draw` subcommand
  needs no grid; a `solve` subcommand waits for the module to state the format, and the
  81-character string every sudoku tool reads is the obvious candidate.
- Whether a live randomness source (a JavaScript function, a Python callable) is ever
  wanted. Not offered by the bindings (Step 4): `RandomError` has no variant for a
  source that fails, and the specification's fake is a script of draws. If a consumer
  asks, the core gains the variant first.
