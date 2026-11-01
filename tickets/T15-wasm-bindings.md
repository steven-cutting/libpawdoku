---
id: T15
title: "WebAssembly bindings: the pawdoku-wasm crate, and @steven-cutting/pawdoku-wasm with specs/ inside"
status: open
depends_on: [S04, T13]
parallel_with: [S12, T40, T41, T43]
branch: ticket/t15-wasm-bindings
estimated_size: L
---

# T15: WebAssembly bindings: the pawdoku-wasm crate, and @steven-cutting/pawdoku-wasm with specs/ inside

## Context

S04 (`tickets/S04-bindings-and-cli.md`, `status: done`, merged as pull request #17)
designed this crate and drafted this ticket in its hand-back notes, "Step 7", on
2026-09-28. That draft is the starting point; this file is the draft reconciled with
`main` on 2026-10-31. Read S04's question (c) and (d), Step 4 and Step 5 for the evidence;
it is not repeated here.

**The design, as S04 left it.** A sibling crate, `crates/pawdoku-wasm`, built with
wasm-bindgen and packed by wasm-pack with `--target web`, because the game is a Svelte
static site whose Vite configuration has no wasm plugin, and the `web` output's `init()`
loads the `.wasm` through `new URL(..., import.meta.url)`, which Vite rewrites at build
time. The package is `@steven-cutting/pawdoku-wasm` on GitHub Packages, the house npm
registry for the `@steven-cutting` scope. It ships the engine's specification modules
under `specs/` and an `exports` map, so the game can hold the clauses it restates equal
to this repository's text by test, resolving them through the package's own `exports`
as its `tests/platform.ts` insists. That ends the shared-truth risk CONVENTIONS.md §1
fact 6 and §13 name, and that `docs/explanation/specifications.md` lines 85 to 89
describe. The crate keeps `[lints] workspace = true`: S04's probe crate proved that
wasm-bindgen's expansion trips nothing in the workspace lint table except
`clippy::missing_const_for_fn` on an export that could be `const`, which wasm-bindgen
refuses, so such an export carries
`#[expect(clippy::missing_const_for_fn, reason = "wasm-bindgen exports cannot be const")]`.

**This ticket is the crate and the package; it exports the randomness boundary alone.**
How the board, the solver, play from a record and the generator reach TypeScript is
S12's spike and the build tickets after it: T40 specifies play from a record, T41
builds it in the core for every binding, and T42 builds the TypeScript surface in this
crate. The maintainer decided on 2026-10-31 that the game drives the engine both ways: a
long-lived `Board` class with methods, and stateless functions, plain values in and
out, implemented in Rust so that TypeScript and Python share them. Nothing of that is
this ticket's, but two things here are shaped for it: the crate is a library with a
`src/lib.rs` that T42 grows, and the package and its release are the ones T42 ships a
second version through.

**Settled by the maintainer, 2026-10-31.** Two questions that bear on this ticket were
answered, each with the ticket writer's proposal: the package is **public**, made so
after its first publish (step 12); and each binding crate keeps **its own changelog**,
so this crate has `crates/pawdoku-wasm/CHANGELOG.md` and the package ships it (steps 3
and 6), while the root `CHANGELOG.md` stays the core's and the repository's. Neither is
asked again.

**Versions, re-verified on 2026-10-31 at 20:41 UTC.** Sources: the crates.io API
(`https://crates.io/api/v1/crates/<name>`, newest non-yanked version), the npm registry
(`https://registry.npmjs.org/wasm-pack`), and conda-forge through `pixi search <name>
--platform linux-64` and `--platform osx-arm64` with pixi 0.83.0.

| Tool or crate | S04, 2026-09-28 | 2026-10-31 | Where |
| --- | --- | --- | --- |
| `wasm-bindgen` | 0.2.129 | **0.2.130** (2026-10-30, `rust_version` 1.81) | crates.io |
| `js-sys` | 0.3.106 | **0.3.107** (2026-10-30) | crates.io |
| `wasm-bindgen-test` | 0.3.79 | **0.3.80** (2026-10-30) | crates.io |
| `wasm-bindgen-cli` | 0.2.129 | **0.2.130** (2026-10-30, `rust_version` 1.86) on crates.io; **0.2.130** on conda-forge for both platforms, depending on nothing but `__glibc` on `linux-64` | crates.io; `pixi search` |
| `wasm-pack` | 0.15.0 | **0.15.1** (2026-10-30) on crates.io, on npm and on conda-forge for both platforms; conda-forge's depends on `libgcc` and `__glibc` only | crates.io; npm; `pixi search` |
| `serde-wasm-bindgen` | 0.6.5 | 0.6.5, unchanged (its last release is still 2024-02-27) | crates.io |
| `nodejs` | 26.10.0 | **26.12.1** on conda-forge for both platforms, no `rust` dependency | `pixi search` |
| `binaryen` | 121 | 121, unchanged, on conda-forge for both platforms | `pixi search` |
| `console_error_panic_hook` | 0.1.7, archived | 0.1.7, unchanged; not used (S04 question (f)) | crates.io |

The conda-forge feedstock's `recipe/recipe.yaml` for `wasm-bindgen-cli` (head
`e391fdb5`, 2026-10-30) still lists `bin/wasm-bindgen-test-runner` beside
`bin/wasm-bindgen` (read with `gh api repos/conda-forge/wasm-bindgen-cli-feedstock/contents/recipe/recipe.yaml`).
`@steven-cutting/pawdoku-wasm` is still HTTP 404 on the public npm registry; GitHub
Packages was not queried, because a read there needs a token. None of these moves
anything in the design, but every pin in this ticket is the 2026-10-31 one, and step 1
checks them again on the day. **Not re-read:** wasm-pack's installer, whose
`--mode no-install` behaviour S04's first review round read at `v0.15.0`
(`src/install/mod.rs`, `src/install/mode.rs`, `src/command/build.rs`); step 1 re-reads
it at `v0.15.1`.

**What `main` has gained since S04, and what each means here.**

- **The gate list** in `pyproject.toml` is nineteen recipes (`docs/reference/
  quality-gates.md`'s table numbers them 1 to 19, with `check-clean` as 20). T22 added
  `metrics` as gate 7; T29 added `snapshots-check` last, as gate 19. `wasm-test` joins
  last, after `snapshots-check`, as gate 20, so no gate is renumbered; `check-clean`
  becomes 21.
- **`metrics`** runs rustqual over `crates/pawdoku` alone (`Justfile` line 182), and its
  architecture rules are the core's layering. This ticket leaves it there: the binding
  crate is glue, its module boundaries are not `docs/explanation/layering.md`'s, and a
  measured crate would need rules of its own. Recorded in the decision record, and an
  Open point.
- **`snapshots-check`, `test`, `coverage`, `clippy`, `features` and `doc` all pass
  `--workspace`.** Every member under `crates/` joins them automatically (`Cargo.toml`
  line 3, `members = ["crates/*"]`). On the host, `tests/web.rs` must compile to nothing:
  it starts `#![cfg(target_arch = "wasm32")]`, the usual wasm-bindgen pattern, and
  `wasm-bindgen-test` is a `[target.'cfg(target_arch = "wasm32")'.dev-dependencies]`
  entry. `coverage` narrows to `-p pawdoku`, because invariant 7's floor is over
  `crates/pawdoku/src/**`, as the floor's comment at `Justfile` line 13 already says.
- **`doc` documents this crate too, and `pages.yml` publishes what `doc` builds**
  (decision 0016). The hosted reference therefore gains a `pawdoku_wasm` page. That is
  accepted: it documents what the binding exports, and the root `index.html` still sends
  a reader to `pawdoku/`.
- **Doctests.** `AGENTS.md` asks for a doctest on every public item. A Rust doctest runs
  on the host, where a wasm-bindgen import panics with "cannot call wasm-bindgen
  imported functions on non-wasm targets", so a doctest that reaches `JsError::new` or
  any `JsValue` would fail. The crate's examples are written in `js` fences, which
  rustdoc shows and does not run, and its behaviour is proved by `tests/web.rs`. That is
  a stated exception to decision 0009's three places, owned by
  `docs/reference/testing.md`. The panic on the host is a claim this ticket proves in
  step 5, not a fact read from source.
- **The release machinery is T13's.** T13 lands `[workspace.metadata.release]`, the
  cargo-release pin, the `release` environment with four tag patterns including
  `pawdoku-wasm-v*`, the tag ruleset, and `.github/actions/release-guards/action.yml`, in
  its first pull request. This ticket depends on that pull request alone, not on
  `pawdoku` being on crates.io. If T13's first pull request has not merged, stop.
- **`js-sys` and `serde-wasm-bindgen` are not needed yet.** S04's draft added both. The
  `RandomStream` class takes a `Vec<f64>`, which wasm-bindgen maps to a `Float64Array`,
  and the panic hook's `console.error` is an import block; neither needs `js-sys`, and
  nothing here serialises. cargo-shear fails the gate on a dependency nothing uses, so
  each is added by the ticket whose code first uses it (T42, if S12 recommends it).
- **One changelog per binding crate** (the maintainer, 2026-10-31, on T13's question).
  The root `CHANGELOG.md` stays the core's and the repository's, so this crate gets its
  own `crates/pawdoku-wasm/CHANGELOG.md`, which its `[package.metadata.release]`
  rewrites with the two replacements T13's step 3 writes for the core, and which the
  npm package ships. The root file takes only this ticket's repository entries.
- **Decision records.** 0019 is projected for T13's; this ticket's is the next free
  number after it on the day, 0020 if nothing else has landed.

**Two departures from S04's draft, each for a stated reason.**

- **No `actions/setup-node` in `release-npm.yml`.** The pixi environment pins `nodejs`
  for `wasm-test`, and conda-forge's Node ships `npm`, so the publish job uses that `npm`
  and writes the two registry lines `actions/setup-node` would write, into
  `crates/pawdoku-wasm/pkg/.npmrc` (`npm pack` never includes an `.npmrc`). One Node pin
  and no second action. If the environment's `npm` cannot publish, fall back to
  `actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0` (the latest
  release on 2026-10-31; its tag ref is a commit, so lightweight) with `registry-url`
  and `scope`, and say so under Deviations.
- **`RANDOM_VERSION` crosses as a function**, `randomVersion(): string`, unless step 4
  shows that wasm-bindgen 0.2.130 exports a `const` item. S04 wrote "exported as a
  constant" without proving it, and its probe exported no constant. Python's attribute
  keeps the constant's name; the difference is recorded in the record.

Frozen files under CONVENTIONS.md §11 that this ticket edits: the `Justfile` and
`docs/manifest.yml`. Each is a T00 follow-up carried here, as T22, T24, T29, T33 and
T13 did; step 1 asks.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §1, §10, §11, §13; S04's questions
(c), (d) and (f), Steps 4 to 6, Deviations and "Handed back"; T13 and its hand-back
notes; `crates/pawdoku/src/random.rs` and `crates/pawdoku/tests/random.rs` (the golden
values); `crates/pawdoku/tests/api_bounds.rs`; the `Justfile`; `pyproject.toml`;
`.github/workflows/ci.yml` and `.github/actions/setup/action.yml`;
`docs/explanation/architecture.md`; `docs/reference/testing.md`;
`docs/reference/quality-gates.md`; `docs/decisions/0007-dependency-policy.md`,
`0008-no-std-core.md` and `0009-rust-quality-gate.md`.

## Goal

`just wasm-build` produces `crates/pawdoku-wasm/pkg/` holding the JavaScript, the
`.wasm`, the TypeScript declarations, `specs/*.allium` and an amended `package.json`;
`just check` is green with the crate in the workspace and runs `wasm-test`;
`release-npm.yml` publishes `@steven-cutting/pawdoku-wasm 0.1.0` on the tag
`pawdoku-wasm-v0.1.0`; and the game's repository can install it in CI. The package
exports `RandomStream` and the random version, which is enough for the game to replay a
seed and, more to the point, to read the specification modules it ships.

## Non-goals

- The engine's TypeScript surface: the board, the solver, play from a record and the
  generator. S12, T40, T41 and T42.
- npm provenance: undocumented for GitHub Packages (S04 question (d)).
- `wasm-opt`: off until S12 measures a size worth fixing.
- A browser test: S12 asks whether one is needed.
- A `wasm32v1-none` check of this crate: wasm-bindgen needs `std`, and that target
  stays the core's proof alone.
- A live randomness callback: not offered (S04 Step 4); the core gains a
  `RandomError` variant first if a consumer ever asks.
- The game's side. The G ticket is drafted in T42.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `crates/pawdoku-wasm/Cargo.toml` | crate manifest | New (step 3) |
| `crates/pawdoku-wasm/README.md` | crate | New: what the package is, how to install it from GitHub Packages, `init()`, and the `specs/` subpath; wasm-pack copies it into `pkg/` |
| `crates/pawdoku-wasm/src/lib.rs` | crate | New: `RandomStream`, the random version, the panic hook, the bounds assertion (step 4) |
| `crates/pawdoku-wasm/tests/web.rs` | crate tests | New, mandatory: wasm-bindgen-test in Node (step 5) |
| `crates/pawdoku-wasm/CHANGELOG.md` | crate | New: the crate's own, in the Keep a Changelog shape of the root file, with `[Unreleased]` holding the crate and the package, and an `[Unreleased]` link that each release rewrites to compare from its `pawdoku-wasm-v*` tag (step 3) |
| `Cargo.toml` | workspace manifest | `wasm-bindgen = "0.2.130"` and `wasm-bindgen-test = "0.3.80"` in `[workspace.dependencies]`, each with the decision-0007 note (step 2) |
| `Cargo.lock` | generated | The resolution of the two |
| `Justfile` | T00 follow-up | `wasm-check`'s third line; `coverage -p pawdoku`; the `wasm-build` and `wasm-test` recipes (step 6). Frozen; step 1 |
| `pyproject.toml` | T00 follow-up | `wasm-pack = "==0.15.1"`, `wasm-bindgen-cli = "==0.2.130"`, `nodejs = "==26.12.1"`, each with a comment; `wasm-test` appended to the recipe list (step 6) |
| `pixi.lock` | generated | `pixi lock` after the pins |
| `scripts/wasm_pkg.py` | script | New: copies the specifications into `pkg/specs/` and the crate's `CHANGELOG.md` into `pkg/`, and amends `pkg/package.json` (step 6) |
| `.gitignore` | dotfile | `crates/pawdoku-wasm/pkg/` |
| `.github/workflows/ci.yml` | workflow | The `wasm` job: `wasm-test` and `wasm-build` after `wasm-check`, the `pkg/` artefact, `timeout-minutes: 20` (step 7) |
| `.github/workflows/release-npm.yml` | workflow | New: publish on a `pawdoku-wasm-v*` tag (step 8) |
| `docs/decisions/0020-webassembly-package.md` | decision | New record (step 9); the number is the next free one on the day |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | T00 follow-up | The record's entry. Frozen; step 1 |
| `docs/explanation/architecture.md` | explanation | Line 27 ("today the only member"); lines 36 to 39 ("Three siblings join … None exists yet."); the table "What crosses each boundary" at 323 to 336, its WebAssembly row in the boundary's names |
| `docs/explanation/specifications.md` | explanation | Lines 85 to 89: the shared-truth paragraph says the package ships the modules and the game's test can hold its restatements to them; past tense once the G ticket lands is the G ticket's note, not this edit |
| `docs/project/repository-map.md` | project | Lines 11 to 13 ("one member today"); the tree (the crate, `scripts/wasm_pkg.py`, `release-npm.yml`); the responsibility table |
| `docs/project/purpose-and-scope.md` | project | Lines 63 to 65: the WebAssembly crate exists |
| `docs/reference/commands.md` | reference | Rows for `wasm-build` and `wasm-test`; `wasm-test` among `just check`'s recipes, `wasm-build` outside it |
| `docs/reference/testing.md` | reference | "Layout": where this crate's tests live, the `cfg` gate on `tests/web.rs`, and the `js`-fence exception to doctests |
| `docs/reference/quality-gates.md` | reference | Gate 20, `wasm-test`; `check-clean` becomes 21; the `wasm` job's row at 140 |
| `docs/operations/maintenance.md` | operations | Under T13's "Cut a release", a wasm release, which rewrites `crates/pawdoku-wasm/CHANGELOG.md`; the package settings: public, and the game's repository granted Read under Actions access; the rule that `wasm-bindgen-cli`'s pin moves with `Cargo.lock`'s `wasm-bindgen` |
| `docs/explanation/security-model.md` | explanation | The publishing bullets: `release-npm.yml`, `packages: write`, the run's own token |
| `AGENTS.md` | agent contract | Line 67 ("its only member today"); the publishing paragraph T13 rewrote gains `release-npm.yml` |
| `README.md` | maintainer docs | Lines 86 to 89, "How the game will use it": the package exists |
| `CHANGELOG.md` | maintainer docs | Under `[Unreleased]`: this ticket's repository entries only, the recipes, the CI job, `release-npm.yml` and `scripts/wasm_pkg.py`; the package's entries are in the crate's own file |
| `tickets/README.md` | ticket index | T15's row set `done`, in the closing pull request |
| `tickets/T15-wasm-bindings.md` | ticket | Hand-back notes; `status:` |

## Steps

1. Create the worktree on `ticket/t15-wasm-bindings` from `main` after T13's first pull
   request has merged (`tickets/README.md`, "How to pick up a ticket"). Run
   `just initialize` if `.pixi/` is absent and `just install-tools` if `.tools/bin`
   lacks cargo-hack, rustqual or cargo-release; each uses the network and this ticket
   authorises it. Run `just check` and quote its last line.

   Confirm with the maintainer that the two frozen files, the `Justfile` and
   `docs/manifest.yml`, may be edited here; the recipes are what CI runs and the record
   needs its entry in the same commit, so without leave for both, stop and say so.

   Re-check every row of the Context's version table on the day, with the same sources,
   and quote each. A version that moved is the one this ticket pins, and the move is
   recorded under Deviations. Then read wasm-pack's installer at the pinned tag
   (`gh api repos/wasm-bindgen/wasm-pack/contents/src/install/mod.rs?ref=v0.15.1` and
   the same for `src/install/mode.rs` and `src/command/build.rs`) and quote that
   `--mode no-install` still forbids a download and still prefers a matching
   `wasm-bindgen` on `PATH`. If it does not, stop: `wasm-build`'s design rests on it.

2. **The dependencies**, classed as a T02 hand-back carried here. In
   `[workspace.dependencies]`: `wasm-bindgen = "0.2.130"`, and `wasm-bindgen-test =
   "0.3.80"` for tests, each with a comment that is its decision-0007 note: what it is
   for, that it ships (wasm-bindgen) or serves tests only (wasm-bindgen-test), that its
   licence is `MIT OR Apache-2.0` and already allowed, and, for wasm-bindgen, that the
   pixi pin of `wasm-bindgen-cli` must equal it, which `wasm-build`'s guard enforces.
   Run `cargo update -p wasm-bindgen --precise 0.2.130` if the resolver picks another,
   and quote `cargo tree -p pawdoku-wasm -e normal --target wasm32-unknown-unknown`.

3. **The manifest**, `crates/pawdoku-wasm/Cargo.toml`: `name = "pawdoku-wasm"`,
   `version = "0.1.0"`, a description, and every package field `.workspace = true` as
   `crates/pawdoku/Cargo.toml` writes them, `publish.workspace = true` included, so the
   crate stays off crates.io (the npm package is its release); `[lib] crate-type =
   ["cdylib", "rlib"]`; `[lints] workspace = true`; `[dependencies] pawdoku = { path =
   "../pawdoku" }` and `wasm-bindgen = { workspace = true }`;
   `[target.'cfg(target_arch = "wasm32")'.dev-dependencies] wasm-bindgen-test = {
   workspace = true }`; `[package.metadata.wasm-pack.profile.release] wasm-opt = false`,
   with a comment that S12 measures whether it is worth turning on; and
   `[package.metadata.release]` with the two replacements of T13's step 3, on
   `CHANGELOG.md` relative to the crate (the crate's own file, not `../../CHANGELOG.md`)
   and with the tag prefix `pawdoku-wasm-v` in the compare and tag links in place of
   `pawdoku-v`. Write `crates/pawdoku-wasm/CHANGELOG.md` in the root file's shape: the
   same header, a bare `## [Unreleased]` holding this crate's and this package's
   entries, and the link line
   `[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/crates/pawdoku-wasm/`,
   the crate's history until its first tag, as the root file's
   `[Unreleased]: …/commits/main/` was the repository's; the first release rewrites it to
   a compare link from `pawdoku-wasm-v0.1.0`. Quote `cargo release -p pawdoku-wasm` (a
   dry run) showing its replacements on that file and none on the root `CHANGELOG.md`.
   The workspace's `[profile.release]` (`lto = "fat"`, `codegen-units = 1`) applies to
   the wasm build too.

4. **The crate**, `src/lib.rs`, from S04's Step 4:

   - A private `#[derive(Clone, Debug)] enum Stream { Seeded(SeededStream),
     Replay(ReplayStream) }` with `fn as_dyn(&mut self) -> &mut dyn RandomStream`.
   - `#[wasm_bindgen] #[non_exhaustive] #[derive(Clone, Debug)] pub struct RandomStream`
     holding a `Stream`, beside the in-crate compile-time assertion:
     `const _: () = { const fn assert_bounds<T: Send + Sync + 'static + Clone +
     core::fmt::Debug>() {} assert_bounds::<RandomStream>(); };`. The Rust type and the
     core's trait share a name; import the trait under an alias, or name the class with
     `js_name = RandomStream` on a Rust type called something else, and say which.
   - Two static constructors: `seeded(seed: u64)` (a `bigint` in TypeScript) calling
     `SeededStream::new`, and `replay(draws: Vec<f64>)` (a `Float64Array`) calling
     `ReplayStream::new` and throwing on `Err`. A method `nextDraw(): number` and a getter
     `index: bigint`. Errors cross as `JsError::new(&error.to_string())`, so the message
     is the `Display` text invariant 3 pins.
   - The random version: first try `#[wasm_bindgen] pub const RANDOM_VERSION: &str`
     and quote the compiler's answer; if wasm-bindgen refuses it, export
     `#[wasm_bindgen(js_name = randomVersion)] pub fn random_version() -> String`
     returning `pawdoku::random::RANDOM_VERSION`.
   - A `#[wasm_bindgen(start)]` function that installs a panic hook writing the panic's
     message through an imported `console.error`, so the one panic decision 0008 rules
     out reaches the console with its message rather than `RuntimeError: unreachable`.
   - `#[expect(clippy::missing_const_for_fn, reason = "wasm-bindgen exports cannot be
     const")]` only where clippy asks for it.
   - A doc comment on every public item, with a `js` example for each export.

5. **The tests**, `tests/web.rs`, starting `#![cfg(target_arch = "wasm32")]` and run in
   Node by `wasm-bindgen-test-runner` (no `run_in_browser`):
   `RandomStream.seeded(0n)`'s first draws equal `crates/pawdoku/tests/random.rs`'s golden
   values; `index` advances by one per draw and starts at `0n`; `replay` of a value
   outside `[0, 1)` throws the `Display` text of `RandomError::OutOfRange`; a script's
   exhaustion throws the text of `RandomError::Exhausted`; and the random version
   equals `pawdoku::random::RANDOM_VERSION`. Then prove the doctest claim of Context:
   add, in a scratch commit you will drop, a Rust doctest on `RandomStream::replay` that
   unwraps an out-of-range script, run `just test-doc`, and quote the panic message; if it
   does not panic, the exception is not needed, and the crate gains Rust doctests
   instead, recorded under Deviations.

6. **The recipes and pins.** In `pyproject.toml`: `wasm-pack = "==0.15.1"`,
   `wasm-bindgen-cli = "==0.2.130"` (its comment says it is held equal to `Cargo.lock`'s
   `wasm-bindgen`) and `nodejs = "==26.12.1"` (for `wasm-test` and the publish job's
   `npm`), then `pixi lock`, and quote that both platforms solved; `wasm-test` appended
   to the recipe list after `snapshots-check`. In the `Justfile`:

   - `wasm-check` gains `cargo hack check -p pawdoku-wasm --feature-powerset --target
     wasm32-unknown-unknown --locked`, with its comment extended: this crate is checked
     on the wasm-bindgen target only.
   - `coverage`'s first line narrows `--workspace` to `-p pawdoku`; the two `report`
     lines take no package filter.
   - `wasm-build`, after `wasm-check`, in three parts: **a version guard** that reads
     `wasm-bindgen`'s version from `Cargo.lock` offline (`cargo pkgid --locked -p
     wasm-bindgen` ends in `@0.2.130`) and compares it with `wasm-bindgen --version`,
     stopping with both versions named on a mismatch, before anything else runs; then
     **`wasm-pack build crates/pawdoku-wasm --mode no-install --target web --scope
     steven-cutting --release -- --locked`**; then **`python scripts/wasm_pkg.py`**, run
     by the pixi environment's Python.
   - `wasm-test`, after `wasm-build`: the same guard, then
     `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p
     pawdoku-wasm --target wasm32-unknown-unknown --locked`. The guard is one shell
     function or a third recipe both depend on; say which, and why.

   `scripts/wasm_pkg.py`, stdlib only and run with `-I`: copy every `docs/specs/*.allium`
   into `crates/pawdoku-wasm/pkg/specs/` (every module the directory holds on the day,
   not a fixed list); amend `pkg/package.json` so `files` gains `specs`, and `exports`
   is `"."` with `types` and `default`, `"./pawdoku_wasm_bg.wasm"`, and `"./specs/*"`;
   copy `crates/pawdoku-wasm/CHANGELOG.md` into `pkg/` and add `CHANGELOG.md` to
   `files`, so the package ships its own changelog; and if GitHub Packages refused the
   inherited `repository` at the first publish (S04 question (d)), rewrite it with
   `.git`. It writes inside `pkg/` only, which Git
   ignores, so `check-clean` is unchanged.

7. **CI.** The `wasm` job gains `just wasm-test` and `just wasm-build` after `just
   wasm-check`, then `actions/upload-artifact` at the SHA `ci.yml` already pins for
   `lcov`, uploading `crates/pawdoku-wasm/pkg/` with `retention-days: 7`, and its
   `timeout-minutes` goes from 10 to 20, because a release build with fat LTO now runs in
   it. Permissions stay `contents: read`. Quote the job green on the pull request, and
   inspect the artefact: `package.json` carries `files`, `exports` and `repository`;
   `specs/` holds every module.

8. **`release-npm.yml`.** `on: push: tags: ['pawdoku-wasm-v*']`; top-level
   `permissions: contents: read`; one job, `publish`, with `environment: release` and
   `permissions: contents: read, packages: write`; concurrency on the tag, never
   cancelled. Steps: `actions/checkout` at `ci.yml`'s SHA with
   `persist-credentials: false`; the setup action; `just check-toolchain`; T13's
   `./.github/actions/release-guards` with `package: pawdoku-wasm`; `just wasm-build`;
   a step that writes `pkg/.npmrc` with `@steven-cutting:registry=https://npm.pkg.github.com`
   and `//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}` (the variable stays a
   literal for npm to expand; it is never written out); an existence check (`npm view
   @steven-cutting/pawdoku-wasm@<version> version`) that skips the publish when the
   version is there; then `npm publish` in `crates/pawdoku-wasm/pkg` with
   `NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}` on that step only. No `id-token`, no
   `--provenance`. `actionlint` green through `just lint`.

9. **The record and the pages.** The record, `docs/decisions/0020-webassembly-package.md`
   in the shape of 0016 with `canonical_for: [decision_webassembly_package]` (next free
   number on the day): wasm-bindgen and wasm-pack `--target web`; GitHub Packages and the
   scope; the specification modules inside the package and the `exports` map, and why
   (the shared-truth risk); the CLI pin held equal to the lockfile and `--mode
   no-install`; the seed as a `bigint`, errors as the `Display` text, no live callback,
   no `console_error_panic_hook`; tests in Node, examples in `js` fences;
   `metrics` left on the core; the random version as a function, if it is one. Register
   it. Then every page in Files touched.

10. **The first pull request.** `just check-docs`, then `just check`, quoted. Commit,
    then ask before pushing and opening the pull request. Prove, and quote:
    - `just clippy` green, and a planted `unsafe {}` in `src/lib.rs` failing it.
    - `just wasm-test` green, and a planted wrong golden value failing it.
    - `wasm-build`'s two refusals: a shim `wasm-bindgen` earlier on `PATH` that reports
      another version stops the recipe at the guard before wasm-pack runs; and a
      `Cargo.toml` edit that forces a re-resolve fails on `-- --locked`.
    - The resolver: in a scratch directory outside the repository,
      `npm install <worktree>/crates/pawdoku-wasm/pkg`, then a one-line Node module that
      resolves `@steven-cutting/pawdoku-wasm/specs/sudoku.allium` and the `.wasm`
      subpath with `import.meta.resolve`, both inside `node_modules`.

11. **Release 0.1.0**, after the first pull request has merged, as T13's "Cut a
    release" says: a `release/pawdoku-wasm-v0.1.0` branch, `cargo release -p
    pawdoku-wasm --no-tag --execute`, a pull request, then `cargo release tag -p
    pawdoku-wasm --execute` on `main` and the push of the tag. Approve the deployment.

12. **After the first publish**: make the package **public** (the maintainer,
    2026-10-31), and add the game's repository under the package's "Manage Actions
    access" with the Read role, as S04 drafted it. The maintainer changes the
    visibility in the package's settings on GitHub, under "Danger Zone", as GitHub's
    page "Configuring a package's access control and visibility" describes. That page
    is understood to warn that a package made public cannot be made private again; it
    was not re-read on 2026-10-31, so re-read it on the day, quote the steps and the
    warning, and say so when asking. Read both settings back (`gh api
    /users/steven-cutting/packages/npm/pawdoku-wasm --jq .visibility` gives `public`),
    and record in `docs/operations/maintenance.md` that the package is public. Set
    `status: done` and the row in `tickets/README.md` in a closing pull request with the
    proof.

**Authorisation**, each asked for on its own (CONVENTIONS.md §11): each push and pull
request; the tag push; approving the `release` environment's deployment, which is the
publish; the package's visibility; granting the game's repository access.

## Acceptance criteria

- `just check` green, running `wasm-test` as gate 20; the `wasm` job green with the
  artefact inspected as step 7 says.
- `crates/pawdoku-wasm` keeps `[lints] workspace = true`, and every `#[expect]` in it
  names `clippy::missing_const_for_fn` with the reason.
- `just coverage` measures `crates/pawdoku/src/**` alone, and the floor holds.
- `tests/web.rs` runs in Node and fails on a planted wrong value; the host build of the
  workspace compiles it to nothing.
- Both `wasm-build` refusals fail as stated, and the resolver check finds both subpaths
  inside `node_modules`.
- `gh api /users/steven-cutting/packages/npm/pawdoku-wasm/versions` lists 0.1.0, the
  `release-npm.yml` run is green with no token beyond the run's own, and the package's
  visibility reads back `public`.
- The record is in the manifest and the index; no page says the WebAssembly crate does
  not exist yet.
- `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
grep -n -E 'wasm-bindgen|wasm-bindgen-test' Cargo.toml
grep -n -E 'wasm-pack|wasm-bindgen-cli|nodejs|wasm-test' pyproject.toml
cargo pkgid --locked -p wasm-bindgen
wasm-bindgen --version
just wasm-check
just wasm-test
just wasm-build
ls crates/pawdoku-wasm/pkg crates/pawdoku-wasm/pkg/specs
python3 -I -c 'import json; p=json.load(open("crates/pawdoku-wasm/pkg/package.json")); print(p["name"], p["version"], p["files"], p["exports"], p["repository"])'
cargo release -p pawdoku-wasm
just coverage
rg -n 'only member|None exists yet|one member today' AGENTS.md docs README.md
just check-docs
just check
git status --porcelain
```

Expected: the two workspace pins; the three pixi pins and `wasm-test` last in the
recipe list; `@0.2.130` from both; three finished checks; the tests passing; a `pkg/`
holding `pawdoku_wasm.js`, `pawdoku_wasm_bg.wasm`, `pawdoku_wasm.d.ts`, `package.json`,
`README.md`, `CHANGELOG.md` and every module under `specs/`;
`@steven-cutting/pawdoku-wasm 0.1.0` with `specs` and `CHANGELOG.md` in `files` and the
three `exports` keys; a dry run whose replacements touch
`crates/pawdoku-wasm/CHANGELOG.md` and not the root file, ending "aborting release due
to dry run"; the floor met; no stale sentence;
`All checks passed and the worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- **Whether the inherited `repository` is accepted.** wasm-pack copies Cargo's
  `https://github.com/steven-cutting/libpawdoku`, without `.git`; GitHub's page shows
  both forms. Verified at the first publish; `scripts/wasm_pkg.py` rewrites it if not.
- **`metrics` over the binding.** Left on the core (Context). If the maintainer wants the
  crate measured, rustqual runs a second time over `crates/pawdoku-wasm` with the
  complexity thresholds and no architecture rules, a second probe-less line in the
  recipe.
- **Whether T15 should wait on T13 at all.** It waits on T13's first pull request only,
  for the `release` environment, the tag ruleset, the guards' composite action and the
  cargo-release pin, so that those have one owner. The alternative is that this ticket
  writes its own guards and environment rule, which makes two owners of the release
  machinery; not proposed.
- **Paths later tickets also edit.** T42 edits `crates/pawdoku-wasm/**`, `Cargo.toml`,
  the `Justfile`, `pyproject.toml`, `docs/explanation/architecture.md`,
  `docs/reference/testing.md` and `CHANGELOG.md` after this ticket, never at the same
  time. T41 and T43 edit `docs/explanation/architecture.md`, `docs/reference/testing.md`
  and `CHANGELOG.md`, and T43 `docs/decisions/README.md` and `docs/manifest.yml`, and
  either may be open at the same time; T13 edits `AGENTS.md`, `ci.yml` and
  several pages before it. Each shared edit is a row, a sentence or a recipe, and
  whichever pull request merges second merges `main` first, the deviation from
  CONVENTIONS.md §11 that T31 and T33 recorded.
