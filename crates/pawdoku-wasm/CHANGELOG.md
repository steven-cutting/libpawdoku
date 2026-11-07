# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This is the changelog of `pawdoku-wasm` and of the npm package
`@steven-cutting/pawdoku-wasm` it is packed into. The engine's own changes, and the
repository's, are in the root `CHANGELOG.md`.

## [Unreleased]

### Added

- The crate `pawdoku-wasm`, WebAssembly bindings for `pawdoku` built with wasm-bindgen,
  and the npm package `@steven-cutting/pawdoku-wasm`, packed by wasm-pack for the web
  target and published to GitHub Packages (decision 0020).
- `RandomStream`, the engine's randomness boundary as a class: `RandomStream.seeded(seed)`
  takes a `bigint` seed and gives the engine's own generator, `RandomStream.replay(draws)`
  takes a `Float64Array` script and throws when a value is outside `[0, 1)`, `nextDraw()`
  gives the next draw and throws when a script is exhausted, and `index` is the index
  the next draw will carry, as a `bigint`. Each error is named `RandomError`, and its
  message is the engine's own text.
- `randomVersion()`, the name of the generator `seeded` uses, `"seeded-stream-1"`.
- A panic, which the engine never means to raise, is written to `console.error` with its
  message before the module traps.
- The `Board` class: `Board.open(givens)` and `Board.reopen(record)`, `view()`,
  `readingAfter(index)`, `write()`, and the seven operations `place`, `erase`,
  `writeMark`, `strikeMark`, `check`, `undo` and `redo`. A board is freed once, with
  `free()`, when its puzzle is left.
- Play from a record: `open(givens)`, `play(record, operation)`, `read(record)` and
  `readingAfter(record, moveIndex)`, which hold nothing between calls and hand back the
  values a held board gives; and `recordVersion()`, `"record-1"`.
- `search(givens)`, the solver's search with its verdict, guesses and solutions.
- `generate(tier, stream)`, which advances a `RandomStream` in place,
  `generateFromSeed(tier, seed)`, and `generationVersion()`, `"generation-1"`. A
  generation result carries what replays it, its counts as `bigint`s.
- TypeScript interfaces for every plain value, in the engine's own field names, the
  record's named `BoardRecord`; `Operation` is a union on `kind`.
- A refusal throws an `Error` named for the engine's error type, `EngineErrorName`, with
  the engine's text; a refused generation also carries `tier` and `draws_taken`.
- The `.wasm` is optimised by binaryen's `wasm-opt` at `-O3`.
- The package ships the engine's specification modules under `specs/`, reachable as
  `@steven-cutting/pawdoku-wasm/specs/<module>.allium` through its `exports` map, with
  this changelog and the Apache-2.0 licence.

[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/crates/pawdoku-wasm/
