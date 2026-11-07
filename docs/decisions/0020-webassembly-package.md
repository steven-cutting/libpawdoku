---
title: "Decision 0020: The WebAssembly package"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_webassembly_package]
requires: []
---

# Decision 0020: The WebAssembly package

## Context

The game this engine serves is a Svelte static site, built by Vite with no WebAssembly
plugin, and it is to reach the engine through WebAssembly. Until now it could not: the
workspace held the core alone, and the game restated the clauses it needs from the
engine's specifications with nothing to hold its copy to this repository's text, so the
two could drift without a test noticing.

Spike S04 designed the crate and the package and measured the lint table against
wasm-bindgen's expansion. Spike S12 then decided how the engine's wider surface reaches
TypeScript, which later tickets build on this crate. This record covers the crate, the
package and how both are built and released.

## Decision

**A sibling crate, `crates/pawdoku-wasm`, built with wasm-bindgen and packed by
wasm-pack for the `web` target.** The `web` output's `init()` loads the `.wasm` through
`new URL(..., import.meta.url)`, which Vite rewrites at build time, so the game needs
no plugin. The crate is a `cdylib` and an `rlib`, keeps `[lints] workspace = true`, and
stays off crates.io (`publish = false`): the npm package is its release.

**The package is `@steven-cutting/pawdoku-wasm` on GitHub Packages**, the house npm
registry for the `@steven-cutting` scope, made public after its first publish.

**The package ships the specification modules.** `scripts/wasm_pkg.py` copies every
module in `docs/specs/` into the package's `specs/` and gives the package an `exports`
map with three keys: `"."` with `types` before `default`, the `.wasm`, and
`"./specs/*"`. A consumer resolves `@steven-cutting/pawdoku-wasm/specs/sudoku.allium`
through the package's own `exports`, so the game's test can hold each clause it
restates equal to the text the engine was built from, at the version it installed.
That ends the shared-truth risk, rather than describing it.

**The wasm-bindgen CLI is pinned and held equal to the lockfile.** `pyproject.toml`
pins `wasm-bindgen-cli` and `wasm-pack`, and `just wasm-guard`, which `wasm-build` and
`wasm-test` both depend on, compares `wasm-bindgen --version` with the version
`Cargo.lock` resolves and stops on a mismatch, naming both. wasm-pack runs with
`--mode no-install`, under which it neither downloads a wasm-bindgen nor runs
`cargo install`, and prefers a matching one on `PATH`: wasm-pack 0.15.1's
`InstallMode::install_permitted` is false for that mode, and both the download and the
`cargo install` return before fetching anything when it is false. `-- --locked` reaches
cargo, but it is not what holds the lockfile: wasm-pack 0.15.1 runs `cargo metadata`
without `--locked` before it builds, which rewrites a `Cargo.lock` that disagrees with a
manifest, so the build then finds the two agreeing. `wasm-build` therefore depends on
`lock-check` and stops on a stale lockfile before wasm-pack runs.

**What crosses.** This first version exports the randomness boundary alone: a
`RandomStream` class with `seeded(seed)`, `replay(draws)`, `nextDraw()` and a read-only
`index`, and a function `randomVersion()`. The seed and the index cross as a `bigint`,
because a `u64` does not fit a `number`, and a script as a `Float64Array` or an array of
numbers. Errors cross as a JavaScript `Error` whose `message` is the core error's
`Display` text, the text invariant 3 keeps stable. The random version is a function,
not a constant, because wasm-bindgen 0.2.130 refuses a constant: "#[wasm_bindgen] will
not work on constants unless you are defining a #[wasm_bindgen(typescript_custom_section)]".
Python's binding keeps the constant's name, `RANDOM_VERSION`.

**No live randomness callback.** `RandomError` has no variant for a callback that
throws or returns a value outside `[0, 1)`, and a binding cannot add one, so the class
holds one of the two streams the core ships and nothing else. A consumer that needs one
asks the core for the variant first.

**A panic reaches the console.** A start function installs a panic hook that writes the
panic's message through an imported `console.error`, so the panic decision 0008 rules
out shows its message rather than `RuntimeError: unreachable`. The archived
`console_error_panic_hook` crate is not used.

**Tests in Node, examples in `js` fences.** `tests/web.rs` runs under
`wasm-bindgen-test-runner` in Node, as gate 20, `wasm-test`; it starts
`#![cfg(target_arch = "wasm32")]` so the host build compiles it to nothing. A Rust
doctest runs on the host, where a wasm-bindgen import panics with "cannot call
wasm-bindgen imported functions on non-wasm targets", so the crate's examples are
JavaScript in `js` fences, which rustdoc shows and does not run. That is an exception
to decision 0009's doctest on every public item, for this crate alone.

**`metrics` stays on the core**, and `coverage` measures `crates/pawdoku/src/**` alone.
The binding is glue: its module boundaries are not the core's layering, and a measured
crate would need rules of its own.

**`release-npm.yml` publishes on a `pawdoku-wasm-v*` tag**, in the `release` environment
after decision 0019's guards, with the run's own `GITHUB_TOKEN` and `packages: write`,
and npm from the pinned Node rather than `actions/setup-node`. An existence check lets
the publish run only on npm's `E404`, so a re-run skips a version already there.

Turned down:

- **The `bundler` target.** It needs a WebAssembly plugin in the consumer's build, which
  the game does not have.
- **npm's public registry.** The house registry for the scope is GitHub Packages.
- **wasm-pack's default mode.** It downloads a wasm-bindgen of its own when none on
  `PATH` matches, a binary no lockfile pins.
- **`wasm-opt`.** Off until a measurement says the size is worth it; spike S12 measured
  it, and the ticket that builds the wider surface turns it on.
- **npm provenance.** Undocumented for GitHub Packages.

## Consequences

The ones that hurt.

- **Two pins that must move together.** `wasm-bindgen` in `Cargo.lock` and
  `wasm-bindgen-cli` in `pyproject.toml`; the guard fails the gate until both agree.
- **wasm-pack still reaches the network for one thing.** It asks crates.io for its own
  latest version in the background on every run. Offline that check fails with a
  warning, "failed to get wasm-pack version", and the build goes on; it fetches no tool.
- **wasm-bindgen exports the start function** to JavaScript as `start()`. Calling it
  again installs the same hook again, which is harmless, but it is on the surface.
- **A class whose doctests cannot run.** Its examples are read, not run; `tests/web.rs`
  is what holds them true, and a wrong example is a review finding, not a red gate.
- **A release build in the `wasm` job.** Fat LTO makes the job longer, so its timeout
  is twenty minutes.
- **The approval fails open until the environment exists.** GitHub creates a missing
  environment, with no protection rules, for any workflow that names it, and nothing
  else stands between a tag and the run's own token. So the `release` environment and
  the tag ruleset are a prerequisite of the first tag, read back before it is pushed.
- **A token to read the package.** GitHub Packages asks for one to install even a
  public package, so the game's CI needs the repository granted read access.

## What would reopen this

The game moving off Vite to a bundler with a WebAssembly plugin; GitHub Packages
documenting provenance; wasm-bindgen exporting constants; a consumer that needs a live
randomness callback.

## Related pages

- [Architecture](../explanation/architecture.md)
- [Testing](../reference/testing.md)
- [Quality gates](../reference/quality-gates.md)
- [Maintenance](../operations/maintenance.md)
- [Decision 0008](0008-no-std-core.md)
- [Decision 0009](0009-rust-quality-gate.md)
- [Decision 0019](0019-releases.md)
