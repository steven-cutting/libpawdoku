---
id: T42
title: "The engine in TypeScript: the Board class, play from a record, search and generation in @steven-cutting/pawdoku-wasm"
status: open
depends_on: [T15, T41, S12]
parallel_with: []
branch: ticket/t42-typescript-surface
estimated_size: L
---

# T42: The engine in TypeScript: the Board class, play from a record, search and generation in @steven-cutting/pawdoku-wasm

## Context

T15 built `crates/pawdoku-wasm` and published `@steven-cutting/pawdoku-wasm 0.1.0` to
GitHub Packages with one class, `RandomStream`, the random version, and the engine's
specification modules under `specs/`. This ticket gives the package the engine itself,
so the game can play: the board, play from a record, the solver's search and the basic
generator.

**How the game drives the engine: both ways** (the maintainer, 2026-10-31). Asked to
choose between a long-lived `Board` object with methods, stateless functions with plain
values in and out and the game holding the record, or both, the maintainer chose both,
with the stateless functions implemented in Rust so that TypeScript and Python share
them. So:

- **The `Board` class** wraps `pawdoku::board::Board`. The game opens one from givens or
  reopens one from a record, holds it for as long as the puzzle is in play, calls its
  seven operations, reads its view, and writes its record to keep the game between
  visits. One solver run per puzzle opened.
- **Play from a record**, `board.allium`'s `PlayingFromARecord` (T40), built in the core
  as free functions in `pawdoku::board` (T41). The maintainer settled on 2026-10-31 that
  an operation hands back the record after it **and** the view, and for a check the
  check it added, and that opening from givens is one of the functions. Each call is
  handed a record or givens and gives back a record and a view; nothing is held. Each
  call on a record reopens it, which costs a search and a replay of every move (S12
  measured it).
- **Both return the same plain values**, the core's wire shape (T41), so a value read
  from a held board equals the one play from a record gives, and a Python binding
  will give the same.

**A record names its format version** (the maintainer, 2026-10-31; T43 builds it in the
core before T41). Every record the package hands out names `RECORD_VERSION`, which
crosses as `RANDOM_VERSION` did in T15 (a constant if wasm-bindgen exported one there,
otherwise a function), and a record of another version, or of none, is refused by
reopening: `Board.reopen` throws the core's `ReopenError` `Display` text for it, and
every function of play from a record throws the text of T41's error, which carries that
refusal; either way the message names both versions. No promise is made that a later
package reads a record an earlier one wrote. What the game does with a refused record
is the game's: the G ticket below discards it and tells the player. Whether this crate
reads a record's version before the rest of it, so that a record of a later shape is
refused with the version text rather than a deserialisation error, is T43's Open point;
step 1 reads its answer and steps 4, 5 and 7 carry it.

**What the surface exposes, and what it does not.** Exactly what the specifications'
surfaces expose: `Playing` and `PlayingFromARecord` for the board, `SearchResult` for
the search, `GenerationResult` for the generator. Not the solution grid: the board's
`TheSolutionIsNeverShown` forbids it, and `GenerationResult` does not expose it even
though the Rust `generate` returns a `WellPosed` that holds it. Not `Puzzle`, `WellPosed`
or `solve`: a `Board` is the playable abstraction (decision 0014), and `Board.open` puts
the givens to the solver itself. `search` does expose solutions, because `solver.allium`'s
`SearchResult` does: it is a setter's tool, and the game decides where it uses it.

**What S12 decided.** This ticket depends on S12's hand-back for: how plain values are
typed in the declarations (wasm-bindgen's own `typescript_custom_section` with
`unchecked_return_type` and `unchecked_param_type`, which need no new dependency, or
tsify, which would have gone to T02 with a decision-0007 note); how errors cross (an
`Error` with the `Display` text as its message, and whether its `name` is the Rust
error type, which needs `js-sys`); the record's TypeScript name (proposed `BoardRecord`,
because `Record` shadows TypeScript's built-in `Record<K, V>`); whether the published
package gets a Node test of its own and a `tsc` check of a typed consumer; whether
`wasm-opt` is turned on; and how the game frees a `Board`. Where S12's "Amendments to
T41 and T42" says otherwise than this ticket, follow S12 and say so under Deviations.

**Seeds and draws.** As S04 settled for every binding: a seed is a `bigint`; a
`RandomStream` is passed to `generate` and advanced in place; the generation result
carries what replays it (T41, from S12's question 3): the tier, the seed when there is
one, the stream's start index, `draws_taken`, both versions and the givens. `Draw`
records are `human-solving.allium`'s and are carried by its results when they are built;
nothing built today returns them.

**Where puzzles are made is still open.** `generation.allium`'s first open question and
decision 0012 leave open whether puzzles are generated on the device, ahead of time, or
both. This ticket exports `generate` and generation from a seed because the core has
them and a page, a test or a build step can use them; exporting them answers nothing.
The game calling them on the device would be an answer, and it is the maintainer's to
record in the specification first.

Read first: `AGENTS.md`; T15 and its hand-back notes; S12's hand-back notes; T40 and T41
and their "Names" sections; T43 and its "Open points settled"; `docs/specs/board.allium`
(`Playing`, `PlayingFromARecord`, `Reopening`, `Recording`), `docs/specs/solver.allium`
(`SearchResult`), `docs/specs/generation.allium` (`GenerationResult`);
`crates/pawdoku-wasm/src/lib.rs`; `crates/pawdoku/src/board.rs`, `board/from_record.rs`,
`board/view.rs`, `board/record.rs`, `generation.rs`;
`docs/decisions/0020-webassembly-package.md` (or the number T15 took) and the play-from-a-record record (T40's).

## Goal

`@steven-cutting/pawdoku-wasm 0.2.0` exports a typed `Board` class, the stateless
functions of play from a record, `search`, `generate` and generation from a seed, and
the three versions: the record's, the generation's and the random stream's; every plain
value it returns is the core's wire shape with a named TypeScript type; every refusal
throws an `Error` whose message is the core's `Display` text; a value read from a held board equals the one play from a record gives; and the
game can install it and play. `just check` is green.

## Non-goals

- Any behaviour the specifications do not state. If one is found missing, stop and hand
  it to T40's successor as a specification change.
- The solution, `Puzzle`, `WellPosed`, `solve`, hints, rating.
- A live randomness callback (S04 Step 4).
- npm provenance (undocumented for GitHub Packages, S04 question (d)).
- The game's side: drafted below, separately authorised.
- Python: T17 mirrors this surface on the same core values.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `crates/pawdoku-wasm/Cargo.toml` | crate manifest | `pawdoku` with `features = ["serde"]`; `serde-wasm-bindgen`; `js-sys` if S12 chose named errors; version `0.2.0` by the release run, not by hand |
| `crates/pawdoku-wasm/src/lib.rs` | crate | `mod` lines; the crate documentation, with a `js` example of each way |
| `crates/pawdoku-wasm/src/board.rs` | crate | New: the `Board` class (step 4) |
| `crates/pawdoku-wasm/src/from_record.rs` | crate | New: the stateless functions and the record version (step 5) |
| `crates/pawdoku-wasm/src/solver.rs` | crate | New: `search` (step 6) |
| `crates/pawdoku-wasm/src/generation.rs` | crate | New: `generate`, generation from a seed, the generation version (step 6) |
| `crates/pawdoku-wasm/src/error.rs` | crate | New: one conversion from each core error to a thrown value (step 3) |
| `crates/pawdoku-wasm/src/types.rs` | crate | New: the TypeScript declarations of the plain values, if S12 chose wasm-bindgen's own attributes (step 3) |
| `crates/pawdoku-wasm/tests/web.rs` | crate tests | The tests of step 7, or new `tests/*.rs` beside it, each `#![cfg(target_arch = "wasm32")]` |
| `crates/pawdoku-wasm/tests/package/` | crate tests | If S12 recommended it: a Node module that loads `pkg/` with `initSync` and plays, and a strict TypeScript consumer with its `tsconfig.json` (step 8) |
| `crates/pawdoku-wasm/README.md` | crate | How to load the module, both ways, a seed, errors, freeing a board |
| `crates/pawdoku-wasm/CHANGELOG.md` | crate | Under `[Unreleased]`, Added: the package's new exports; the file is the crate's own, which T15 created |
| `Cargo.toml` | workspace manifest | `serde-wasm-bindgen = "0.6.5"`, and `js-sys = "0.3.107"` if needed, each with its decision-0007 note (step 2); a tsify line only if T02 accepted it |
| `Cargo.lock` | generated | The resolution |
| `Justfile` | T00 follow-up | `wasm-package-test`, if S12 recommended the package test (step 8). Frozen; step 1 |
| `pyproject.toml` | T00 follow-up | `typescript = "==6.0.2"` if the consumer is type-checked; `binaryen = "==121"` if `wasm-opt` is turned on; each with a comment |
| `pixi.lock` | generated | `pixi lock` after the pins |
| `.github/workflows/ci.yml` | workflow | The `wasm` job runs `just wasm-package-test` after `just wasm-build`, if added |
| `docs/explanation/architecture.md` | explanation | "What crosses each boundary": the WebAssembly row names both ways, the plain values and the errors |
| `docs/reference/testing.md` | reference | Where this crate's new tests live, and the package test |
| `docs/reference/commands.md` | reference | A row for `just wasm-package-test`, outside `just check`, if added |
| `docs/reference/quality-gates.md` | reference | The `wasm` job's new step, if added |
| `README.md` | maintainer docs | "How the game will use it": both ways, in two sentences |
| `CHANGELOG.md` | maintainer docs | Under `[Unreleased]`: this ticket's repository entries only, the `wasm-package-test` recipe and the CI step if added; the package's entries are in the crate's own file |
| `tickets/README.md` | ticket index | T42's row set `done` |
| `tickets/T42-typescript-surface.md` | ticket | `status:`, hand-back notes, the G ticket as handed back |

## Steps

1. Create the worktree on `ticket/t42-typescript-surface` from `main` after T15, T41 and
   S12 have merged, and after T02 has accepted any dependency S12 handed it. Run
   `just initialize` and `just install-tools` if needed (network, authorised here) and
   `just check`, quoted. Confirm with the maintainer that the `Justfile` may be edited
   here if step 8 adds a recipe. Write S12's decisions, T41's names, and T43's name for
   the value that reads a record's version alone, at the top of the hand-back notes
   before any code.

2. **Dependencies**, a T02 hand-back carried here: `serde-wasm-bindgen = "0.6.5"` in
   `[workspace.dependencies]`, with its decision-0007 note (it ships; `MIT`, already
   allowed; one maintainer and no release since 2024-02-27, which is the risk the note
   names, and the alternative of `JsValue` built by hand), and `js-sys = "0.3.107"` only
   if S12 chose named errors. Re-check both versions on crates.io on the day and quote
   them. The crate's `pawdoku` dependency gains `features = ["serde"]`.

3. **Errors and types.** In `src/error.rs`, one conversion per core error type
   (`PlayError`, `ReopenError`, `SolveError`, `GenerateError`, `RandomError`, and T41's
   error for play from a record) to the thrown value S12 chose, whose message is always
   `error.to_string()`. In `src/types.rs`, if S12 chose wasm-bindgen's own attributes,
   the TypeScript interfaces of every plain value in one `typescript_custom_section`,
   each field as the core's wire shape writes it, each with a doc comment; and the
   record's interface under the name S12 chose. Every export that takes or returns a
   plain value names its interface with `unchecked_param_type` or
   `unchecked_return_type`.

4. **The `Board` class**, in `src/board.rs`: `#[wasm_bindgen] #[non_exhaustive]
   #[derive(Clone, Debug)]` around `pawdoku::board::Board`, with the in-crate bounds
   assertion T15's `RandomStream` has. Methods, camelCase through `js_name`:
   `Board.open(givens)` and `Board.reopen(record)`, static, throwing the solver's or
   reopening's refusal (`Board.reopen` reads the record through T43's version-only value
   in the core before deserialising the rest, and throws that value's refusal when the
   version differs; this crate writes no reading of its own); `view()`, the core's view
   of the board; `readingAfter(index)`;
   `write()`, the record; and the seven operations, `place(position, digit)`,
   `erase(position)`, `writeMark(position, digit)`, `strikeMark(position, digit)`,
   `check(position)`, which returns the check, `undo()` and `redo()`, each throwing the
   board's refusal and changing nothing when it does. Values cross through
   `serde_wasm_bindgen` both ways. A `js` example on every method.

5. **Play from a record**, in `src/from_record.rs`: one export for each of T41's
   functions, under the camelCase of T41's names, taking and returning plain values and
   throwing the refusal; opening from givens is one of them, and an operation's export
   returns the record and the view, with the check it added for a check. Every export
   that takes a record reads it as `Board.reopen` does (step 4), version first through
   T43's value, and wraps that value's refusal in T41's error's reopening variant before
   throwing it, so that the message is T41's error's text whichever reading refused. If
   T41 offers one function that takes the operation value, this crate
   offers that one export, and the operation's TypeScript type is a union with one
   member per kind. The record version crosses here, as step 6 says the generation
   version does.

6. **Search and generation.** `search(givens)` returns the core's search result.
   `generate(tier, stream)` takes the `RandomStream` class by `&mut` and returns T41's
   generation result from a stream; generation from a seed takes a tier and a `bigint`
   and returns T41's generation result from a seed. The generation version crosses as
   the random version did in T15 (a constant if wasm-bindgen exported one there,
   otherwise a function).

7. **Tests**, wasm-bindgen-test in Node, `#![cfg(target_arch = "wasm32")]`: the
   scripted game played on a held `Board` and played from records gives equal views,
   records and readings at every step, compared as JavaScript values; every refusal's
   message equals the core's `Display` text for that refusal, and a refused operation
   leaves the board as it was; a record naming another version is refused by
   `Board.reopen` with the `ReopenError` text naming both versions, and by play from a
   record with the `Display` text of T41's error wrapping that refusal, and so is a
   record of another version with a field renamed, with those texts and not a
   deserialisation error; the record
   version equals `pawdoku::board::RECORD_VERSION`; `search` on the fixture gives one
   solution and the guesses the core gives; generation from seed 0 in tier 3 equals the
   core's snapshot of it, and its `draws_taken` is 23 when its first grid attempt
   succeeds; a seed of `2n ** 63n` survives the round trip; `generate` advances the
   `RandomStream` it is handed by `draws_taken`; and `generate` in tier 3, handed
   `RandomStream.replay` of five zeros, throws
   `the replay stream is exhausted at draw 5` and leaves that stream's `index` at `5n`,
   because the draws given before the refusal stay taken (`generation.allium`'s
   `AStreamThatRunsOut`; the core's doctest on `GenerateError`). Plant one wrong
   expected value, see it fail, and quote it.

8. **The published package**, if S12 recommended it: `crates/pawdoku-wasm/tests/package/`
   holds a Node module that imports `pkg/pawdoku_wasm.js`, calls `initSync` with the
   bytes of `pkg/pawdoku_wasm_bg.wasm`, opens a board, plays two moves, writes and
   reopens it, and plays the same from records; and a TypeScript consumer type-checked
   with `tsc --noEmit --strict` against `pkg/pawdoku_wasm.d.ts`, which uses every export
   with its declared types. A `wasm-package-test` recipe after `wasm-build` runs both
   with the pinned `nodejs` and `typescript`; CI's `wasm` job runs it after `just
   wasm-build`. If `wasm-opt` is turned on, set it under
   `[package.metadata.wasm-pack.profile.release]` at S12's level with the `binaryen`
   pin, and quote the `.wasm`'s size before and after.

9. **Pages and README**, as Files touched lists: the crate's README is what a consumer
   reads on the package page, so it shows `init()` in the browser and `initSync` in a
   test, both ways of playing, a seed as a `bigint`, a caught refusal, and freeing a
   board as S12 recommended.

10. **The pull request.** `just check-docs`, then `just check`, quoted; the `wasm` job's
    artefact inspected, quoting the declarations of `Board`, one function and one value.
    Commit; ask before pushing and opening it.

11. **Release 0.2.0**, after the merge, as T15 released 0.1.0 (T13's "Cut a release"):
    `cargo release minor -p pawdoku-wasm` on a `release/pawdoku-wasm-v0.2.0` branch, the
    pull request, the tag on `main`, the push, the approval. Quote
    `gh api /users/steven-cutting/packages/npm/pawdoku-wasm/versions`.

12. **Hand back the G ticket.** Update the draft below with what this ticket built, the
    names as exported and the version published, and copy it into "The G ticket" under
    Hand-back notes. Set `status: done` and the index row in a closing pull request.

**Authorisation**, each asked for on its own (CONVENTIONS.md §11): each push and pull
request; the tag push; approving the release deployment. Editing the game's repository
is never this ticket's.

**The G ticket, drafted for the game's `tickets/` and separately authorised.** Updated
on 2026-10-31 from S04's Step 6 draft to the surface above. It was read against the
game's clone at `add73be` (HEAD on 2026-10-31, the same commit S04 found on
2026-09-28); whoever opens it re-reads every path at the game's HEAD on the day.

> **Title.** "Consume the engine: `@steven-cutting/pawdoku-wasm` pinned, its
> specifications held equal by test, and a puzzle played through it."
>
> **Part A, after `pawdoku-wasm` 0.1.0 (T15)**, which ships the specifications:
>
> - `package.json` and `package-lock.json`: `"@steven-cutting/pawdoku-wasm":
>   "<exact version>"`, an exact pin as `@steven-cutting/biscuit-games` is. The scope's
>   registry line is already in `.npmrc`, and `.github/workflows/ci.yml` already grants
>   `packages: read`; the package must grant the game's repository Read (T15 step 12).
> - `tests/engine.ts` (new), shaped like `tests/platform.ts`: `PACKAGE`,
>   `enginePath(subpath)` and `engineFile(subpath)` through the same `createRequire`
>   resolver and the same assertion that the path is inside `node_modules`.
> - `tests/engineRestated.ts` (new): `ENGINE_RESTATED`, empty until `pawdoku.allium`
>   restates an engine clause, with `tests/restated.ts`'s header adapted.
> - `tests/engineSpecs.test.ts` (new): one assertion per module the game restates from,
>   resolved through `specs/<module>.allium`, never a fixed count of modules, because
>   the engine adds modules (T37 adds a tenth); one test per restated clause holding the
>   flattened text equal; and the pin equal to the installed version. The `clauses` and
>   `flatten` helpers move from `tests/platformSpecs.test.ts` into a shared
>   `tests/allium.ts`, or are duplicated if the game prefers self-contained files.
> - The engine's modules leave the game's `docs/specs/` (at `add73be` it still holds
>   eight of them beside `pawdoku.allium`), with their manifest entries and links, as
>   libpawdoku's T06 hand-back lists; `docs/explanation/specifications.md`,
>   `docs/how-to/work-with-the-specs.md` and `.agents/skills/spec-change/SKILL.md` say
>   the engine's text is held by test to what the package ships.
>
> **Part B, after `pawdoku-wasm` 0.2.0 (T42)**, which ships the engine:
>
> - `src/lib/engine/` (new): one module that calls the package's `init()` once, in the
>   browser only (from `onMount`, or behind `browser` from `$app/environment`), never
>   during prerender, because adapter-static prerenders with SSR on; and an adapter that
>   holds one `Board` per puzzle in play and frees it when the puzzle is left, as the
>   package's README says.
> - Keeping a game: `JSON.stringify(board.write())` through the existing `StoragePort`
>   (`src/lib/ports/storage.ts`), and `Board.reopen(JSON.parse(stored))` on return. A
>   record names its format version, and a record of another version is refused with a
>   message naming both, the core's `ReopenError` text; libpawdoku promises nothing
>   across versions. The game discards a record that does not reopen and tells the
>   player, and that behaviour is stated in `pawdoku.allium` first.
> - A new puzzle: a seed drawn from the platform's cryptographic source in the range
>   the specifications give a seed, as a `bigint`, handed to generation from a seed; the
>   game keeps the seed, the tier and both versions with the puzzle, which is what
>   replays it. Whether the game generates on the device at all is libpawdoku's
>   `generation.allium` open question and decision 0012's; the game asks before it
>   ships this, and the answer is recorded there first.
> - The stateless functions are there for a caller that holds no board (a web worker,
>   a test, a resumed game rendered before it is reopened); the game says where it uses
>   them, if anywhere.
> - Tests: a vitest file per adapter, loading the module in `beforeAll` by reading
>   `pawdoku_wasm_bg.wasm` through `enginePath` and calling `initSync`, as S04
>   question (c) found the vitest maintainers recommend; the browser path is proved by a
>   built preview (`vite build`, then `vite preview`) in which a puzzle is played, the
>   page reloaded, and the puzzle found as it was left.
> - `pawdoku.allium`: whatever the game now does with the engine (keeping a game,
>   discarding a record that does not reopen, where a puzzle comes from) is stated there
>   before it is built, under the game's own spec-first rule.
>
> **Its proof.** `npm ci` resolves the package from `npm.pkg.github.com`; `npm test` is
> green; a one-word edit to a restated clause in a scratch branch fails the restatement
> test; a puzzle played in the preview survives a reload. It closes libpawdoku's T06
> item 12, CONVENTIONS.md §1 fact 6 and §13, and makes the shared-truth paragraph of
> libpawdoku's `docs/explanation/specifications.md` past tense, which is a one-line
> follow-up in this repository once the game's test lands.

**Adversarial review**, by the procedure in `tickets/README.md`, "Adversarial
review":

- **R1**, before step 10, the pull request: the surface and its declarations. Focus: where the declarations and the values can differ.
- **R2**, before step 11, the release: the same focus over what the pull request merged.

## Acceptance criteria

- The package's declarations name a type for every parameter and return value; no
  export returns `any`.
- A held board and play from records give equal values at every step of the scripted
  game, through JavaScript.
- Every refusal throws with the core's `Display` text. A refusal by the `Board` class
  or by play from a record changes nothing. A refused `generate(tier, stream)` leaves
  the `RandomStream` past the draws it was given, because they stay taken:
  `generation.allium`'s `AStreamThatRunsOut` when the stream runs out, and its
  `DrawsInOrder` for every other refusal.
- Nothing exported returns a digit read from the solution except a check's answer and
  `search`'s solutions.
- `just check` green; `@steven-cutting/pawdoku-wasm 0.2.0` published by
  `release-npm.yml` with no token beyond the run's own.
- The G ticket is in the hand-back notes, updated to what was published.
- `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just wasm-check
just wasm-test
just wasm-build
rg -n ': any\b|=> any\b' crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts
rg -n 'export (class|function|interface|type)' crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts
ls -l crates/pawdoku-wasm/pkg/pawdoku_wasm_bg.wasm
just check-docs
just check
gh api /users/steven-cutting/packages/npm/pawdoku-wasm/versions --jq '.[].name'
git status --porcelain
```

Expected: the checks and tests green; no `any` in the declarations; the class, the
functions and the interfaces listed; the `.wasm`'s size, quoted; `All checks passed and
the worktree is unchanged.`; `0.2.0` and `0.1.0`; only paths in Files touched.

## Hand-back notes

### Decisions fixed by S12, T40, T41 and T43

Written before any code, from S12's "The recommendations" and "Amendments to T41 and
T42", T41's "Names T42 and T17 need" and T43's "Open points settled". Where they say
otherwise than this ticket's steps, they are followed, and Deviations says so.

- **Types (S12, 1):** wasm-bindgen's own `typescript_custom_section`, with
  `unchecked_param_type` and `unchecked_return_type` on every export that takes or
  returns a plain value; no tsify, nothing for T02. A TypeScript name is the core's Rust
  name, except `Record`, which is `BoardRecord`, written out in full with `version:
  string`. A `u64` field is `bigint`, an `Option` is `| null`, a cell's note is
  `note?: number[]`.
- **Errors (S12, 2):** a `js_sys::Error` whose `name` is the Rust type's and whose
  message is always `error.to_string()`, for `PlayError`, `ReopenError`, `SolveError`,
  `GenerateError` and `RandomError`. `FromRecordError` throws the name of the stage that
  refused: `ReopenError` from its reopening variant, `PlayError` from its playing one,
  `FromRecordError` for the index that names no move and any later variant.
  `GenerationRefused` always throws `name` `GenerateError`, with `tier` (a string) and
  `draws_taken` (a `bigint`) as own properties that are not enumerable. `types.rs`
  exports `type EngineErrorName` and `interface GenerationRefused extends Error`. A
  value of the wrong shape throws `name` `Error` with serde's text.
- **Two serialisers (S12, step 3):** board values and the search through
  `serialize_missing_as_null(true).serialize_maps_as_objects(true)`; the generation
  result adds `serialize_large_number_types_as_bigints(true)`. The default `to_value`
  is used nowhere.
- **Numbers in (S12, step 4):** `digit` and `index` parameters are `JsValue`, read
  through `serde_wasm_bindgen::from_value` to `u8` and `usize`, declared `number`,
  never a Rust `u8` or `usize` parameter, which wraps. A seed stays a `u64` and so a
  `bigint`, which reduces modulo 2^64 silently.
- **Tests (S12, 5):** wasm-bindgen-test in Node, and a package test: a Node module over
  `pkg/` through `initSync`, a strict `tsc` consumer, the snapshots against the
  interfaces and the live values, and a declaration module (`declarations.ts`). No
  browser gate.
- **Size (S12, 6):** `wasm-opt = ['-O3']`, `binaryen = "==121"`, and `wasm-build`'s
  guard requires `wasm-opt version 121`; `opt-level` stays the default.
- **Lifetime (S12, 7):** one `Board` held for the life of a puzzle in play, `free()`
  called once when it is left.
- **Play from a record (T41):** `pawdoku::board::{open, play, read, reading_after}`;
  `Operation` tagged by `kind` with `position` and `digit`; `Played { record, view,
  check }`, `check` absent unless the operation was `CheckCell`; `BoardView::of`,
  `ReadingAfter::of(&Board, move_index)`, `CellReading`; `FromRecordError::{Reopen,
  Play, NoSuchMove}`, the first two transparent; `open` refuses with `SolveError`,
  `read` with `ReopenError`. Only `Operation` and `Record` deserialise.
- **Generation (T41):** `pawdoku::generation::{generate_from_seed,
  generate_from_stream}`, each `-> Result<GenerationResult, GenerationRefused>`;
  `GenerationResult { tier, seed, start_index, draws_taken, generation_version,
  random_version, givens }`; `GenerationRefused` read through `tier()` and
  `draws_taken()`, with no `match`.
- **The record's version (T43):** `pawdoku::board::RecordVersion`, under the core's
  `serde` feature, read from a record before the rest of it; `readable()` refuses with
  `ReopenError::Version`, naming both versions. This crate writes no reading of its own.
  `RECORD_VERSION` is `"record-1"`.

### What was verified, and how

Two segments of this run, from `4023b35`: steps 1 to 9 to the R1 checkpoint `003c1d9`;
then R1's two rounds (`105c2ad`, `8cfb839`) and step 10 as far as it runs without a
push. R2, the CI artefact and everything after step 10 are left (Handed back).

**Step 1.** `.pixi/` and `.tools/bin` were present, so neither install ran. `just check`
at the start (`4023b35`) closed with `All checks passed and the worktree is unchanged.`
The frozen files are edited under the run's leave. The decisions above were written
before any code.

**Step 2.** On crates.io on 2026-11-07: `serde-wasm-bindgen` 0.6.5, last released
2024-02-27, and `js-sys` 0.3.107, released 2026-10-30. `cargo update --workspace` added
`serde-wasm-bindgen` alone; `js-sys` was already in `Cargo.lock` under `wasm-bindgen`.
The crate also names `serde`, a workspace dependency already, for the two serialisers'
`Serialize` bound. On conda-forge, `binaryen` 121 and `typescript` 6.0.2, as S12 read
them; `pixi lock` added both and moved nothing else. `just lock-check` passed.

**Steps 3 to 6.** `src/error.rs`, `src/types.rs`, `src/board.rs`, `src/from_record.rs`,
`src/solver.rs` and `src/generation.rs`, as Decisions says. `cargo clippy` over the crate
with the workspace's lints is clean, with two `#[expect]`s beyond T15's (Deviations). The generated
declarations name a type for every parameter and return: `sed '/export interface
InitOutput/,$d' crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts | rg -n ': any\b|=> any\b'`
prints nothing, while the ticket's own grep counts 15 lines, all in `InitOutput`, as S12
said. `rg -n 'export (class|function|interface|type)'` lists the 21 interfaces and types,
`Board`, `RandomStream`, and the functions `generate`, `generateFromSeed`,
`generationVersion`, `open`, `play`, `randomVersion`, `read`, `readingAfter`,
`recordVersion`, `search`, `initSync` and T15's `start`.

**Step 7.** `tests/surface.rs`, 17 tests in Node beside `web.rs`'s 6: `just wasm-test`
ended `test result: ok. 17 passed; 0 failed` and `ok. 6 passed; 0 failed`. The code of
steps 3 to 6 was written before these tests (Deviations), so each was made to bite by a
planted change, each reverted:

- A wrong expected value, `draws_taken` 24 for seed 0 in tier 3:
  `test generation_from_seed_zero_in_tier_3_is_the_cores_snapshot ... FAIL`,
  `left: JsValue(23n)`, `right: JsValue(24n)`, `16 passed; 1 failed`.
- The version-first read removed from `board::record`, so a record went to `Record`'s
  deserialiser whole: `a_record_of_another_version_is_refused_naming_both_either_way`
  failed with ``left: Err(("Error", "missing field `undone`"))`` against
  `right: Err(("ReopenError", "the record is of format version \"record-9\", and this
  engine reads \"record-1\""))`.

**Step 8.** `wasm-opt = ["-O3"]` replaces T15's `false`. `just wasm-build` before it
wrote a `.wasm` of 351,425 bytes, 100,666 gzipped at `-9`; after it, 231,591 bytes and
85,804 gzipped, 14.8% smaller gzipped, with `[INFO]: found wasm-opt at ".../.pixi/envs/default/bin/wasm-opt"`
and `Optimizing wasm binaries with wasm-opt`. `just wasm-package-test` closed with
`snapshots: 10 snapshots written to target/wasm-package-test/snapshots.ts`, a silent
`tsc`, and `package test: 16 checks passed`. Planted in a copy of `pkg/` and the package
test in a scratch copy outside the tree, each widening or narrowing of the declarations failed `tsc` (first
error quoted):

| Planted in the `.d.ts` | First error |
| --- | --- |
| `Place`'s `digit?: number` | `declarations.ts(183,69): error TS2322: Type 'true' is not assignable to type 'false'.` |
| `{ kind: "Fly" }` added to `Operation` | `declarations.ts(55,3): error TS1360`, the table of operations |
| `"Six"` added to `Tier` | `declarations.ts(64,3): error TS1360`, the table of tiers |
| `is_given?: boolean` | `declarations.ts(183,15): error TS2322`, the shapes line |
| `undone: number \| string` | `declarations.ts(183,63): error TS2322` |
| `undone?: number` | `declarations.ts(183,63): error TS2322` |
| `Position`'s `column?: number` | `declarations.ts(183,3): error TS2322` |
| `BoardCell`'s `note` removed | `consumer.ts(92,52): error TS2339: Property 'note' does not exist on type 'BoardCell'.`, and the snapshots' excess property among 100 |
| `draws_taken: number` | `consumer.ts(61,27): error TS2365: Operator '+' cannot be applied to types 'number' and 'bigint'.` |

With the `.d.ts` restored, `tsc` exited 0. A wrong live value planted in the copy of
`play.mjs`, the reading after move 5 for the snapshot's move 6, failed:
`package test: the live value of the_reading_after_a_move is its snapshot:
AssertionError: Expected values to be strictly equal`.

**Step 9.** The crate's README, the root README's "How the game will use it",
`architecture.md`, `testing.md`, `commands.md`, `quality-gates.md` and both changelogs.
`just check-docs`: `Validated 48 pages and 49 canonical topics.`; `just lint` passed every
hook. Before the R1 checkpoint commit, `just check` over the staged tree closed with
`All checks passed and the worktree is unchanged.`, exit 0; its first run had stopped
at `clippy` on the two `@throws` lines (Deviations).

**R1.** Two rounds, both answered (Adversarial review). After them `tests/surface.rs`
holds 20 tests and the package test 19 checks: `just wasm-test` passed 20 and 6, and
`just wasm-package-test` closed with `package test: 19 checks passed`, on `8cfb839`.

**Step 10, locally.** The `wasm` job's artefact is CI's (Handed back), so the
declarations were read from the `pkg/` that `just wasm-build` wrote from `8cfb839`'s
source; the one later change to the crate, the module documentation of `types.rs`,
writes nothing into `pkg/`. After R1's fixes the `.wasm` is 232,932 bytes, 86,331 gzipped at `-9`. With
`InitOutput` and what follows it removed, the declarations hold no `any`. The class
`Board`, its documentation left out:

```ts
export class Board {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    check(position: Position): Check;
    erase(position: Position): void;
    static open(givens: Given[]): Board;
    place(position: Position, digit: number): void;
    readingAfter(index: number): ReadingAfter;
    redo(): void;
    static reopen(record: BoardRecord): Board;
    strikeMark(position: Position, digit: number): void;
    undo(): void;
    view(): BoardView;
    writeMark(position: Position, digit: number): void;
    write(): BoardRecord;
}
```

One function, `play`:

```ts
export function play(record: BoardRecord, operation: Operation): Played;
```

One value, `GenerationResult`:

```ts
export interface GenerationResult {
    /** The tier asked for. */
    tier: Tier;
    /** The seed it was generated from, or `null` when it was generated from a stream. */
    seed: bigint | null;
    /** The stream's index when generation began. */
    start_index: bigint;
    /** How many draws generation took. */
    draws_taken: bigint;
    /** The generator's version, `generationVersion()`. */
    generation_version: string;
    /** The random stream's version, `randomVersion()`. */
    random_version: string;
    /** The puzzle's givens. The solution is never shown. */
    givens: Given[];
}
```

`just check-docs` closed with `Validated 48 pages and 49 canonical topics.` before the
step's commit. `just check` on that commit, `f9a04ac`, passed 20 tests in
`surface.rs` and 6 in `web.rs` among the rest and closed with `All checks passed and the
worktree is unchanged.`, exit 0; the commit that quotes it changed this ticket alone.

### Deviations, and why

- **One branch.** This ticket runs on `implement-typescript-surface-fullsend` as the
  run's brief sets, not on `ticket/t42-typescript-surface`, and no worktree was created.
  Step 1's conditions are satisfied by order on this branch.
- **The package stays at 0.1.0.** 0.2.0 comes from a release run (step 11), which is not
  authorised here, and 0.1.0 has not been published (T15 is open), so, as the second
  Open point allows, the first release would carry this surface at 0.1.0.
- **S12's amendments over the ticket's text:** named errors through `js-sys`; two
  serialisers; numbers read through serde; `wasm-opt` on with its guard; the package test
  and its declaration module; and the Verification grep replaced by S12's, since the
  ticket's matches `InitOutput` always.
- **T43's version-only reader.** `Board.reopen` and every function of play from a record
  read a record through `pawdoku::board::RecordVersion` first, in one place,
  `board::record`; this crate writes no reading of a version.
- **T41's names.** The exports are the camelCase of T41's: `open`, `play`, `read`,
  `readingAfter`; generation from a seed is `generateFromSeed`, the core's
  `generate_from_seed`. One `play` taking the operation value, as T41 offers one.
- **Tests after the code.** Steps 3 to 6 were written before `tests/surface.rs`, against
  the brief's tests-first rule; each test arrived green and was then made to bite by the
  planted changes quoted above, as T41 did for its second segment. The test list is
  `tests/surface.rs`'s 20 tests and the package test's 19 checks (Verification).
- **One test file, `tests/surface.rs`**, rather than one per module: each `tests/*.rs` is
  its own wasm binary, and the tests share the fixture and the script.
- **`RandomStream` throws `RandomError` by name.** T15's class threw a `JsError`, named
  `Error`; S12's step 3 lists `RandomError` among the named errors, so `web.rs` now holds
  the name too, and the crate's changelog entry for `RandomStream`, unreleased, says so.
- **The record's moves and checks are `RecordMove` and `RecordCheck`.** The core's types
  for them, `Written` and `Asked`, are private, so no Rust name exists to keep.
- **An operation of the wrong shape is refused before the record is reopened.** `play`
  reads the operation through serde before the core reopens the record, so a record that
  would not reopen and an operation of the wrong shape together throw serde's `Error`.
  T41's order, a record refused first, holds among the engine's refusals.
- **`view()`, `write()` and the other value returns are `Result`s** for the serialiser's
  error, which the core's values never raise; a panic would be the alternative.
- **The declaration module holds each interface by its whole shape**, `Same<Interface,
  { ...fields }>`, rather than S12's optional-keys line and a read-back per field: one
  equality catches a field added, removed, made optional or widened, which the planted
  table shows. The union tables and `@ts-expect-error` lines are as S12 wrote them.
- **`play.mjs` imports `declarations.ts`** through Node 26's type stripping, so the tables
  `tsc` checks are the values put to the engine; every type import there is `import type`.
- **`wasm-package-test` does not depend on `wasm-build`**, so CI builds once; it stops,
  saying so, when `pkg/` is absent. **The `wasm-opt` guard is in `wasm-build`'s body**,
  not in `wasm-guard`, so `wasm-test` in `just check` needs no binaryen. **`tsc` runs with
  `CONDA_PREFIX` set** to the pixi environment, because conda-forge's `tsc` script finds
  itself through it.
- **Two `#[expect(clippy::doc_markdown)]`**, on `generate` and `generate_from_seed`, whose
  `@throws {GenerationRefused}` lines S12 asks for: the JSDoc type takes no backticks.
- **T00 follow-ups:** the `Justfile` (`wasm-package-test`, `wasm-build`'s guard) and
  `pyproject.toml` (`binaryen`, `typescript`), under the run's leave.
- **Step 10 without the pull request.** The declarations were read from the local
  `pkg/`, not from the `wasm` job's artefact, which needs a push; the commit was made,
  and nothing was pushed.
- **A held board reads its arguments without running the caller's code uncaught** (R1):
  a position from its own fields, a digit, an index and `generate`'s tier as primitives,
  an object or a function there refused unread with `Error`. The ticket's steps read
  every value through `serde_wasm_bindgen`; that left a `Board` or a `RandomStream`
  locked after a throwing getter.
- **`play` refuses an array position as the held board does.** The final code review
  found that `play` read an operation's `position` given as an array `[1, 3]` as a
  cell, through serde's internally tagged reading, while `board.place([1, 3], 4)`, `open`
  and `Board.open` refused it with ``missing field `row` ``, so a refusal differed
  between the two ways (`TheSameBoardFromEitherWay`). `types::read_operation` now refuses
  an array in the operation's own `position` field with `read_held_position`'s refusal;
  tested red first as a fourth case of
  `a_board_reads_a_position_as_play_does_from_its_own_fields`.
- **A record's refusals are mapped once, in `board::record`.** The final code review
  found the same mapping of an unread record written three times, in `Board.reopen`,
  `read` and a `reopened` helper that wrapped a version's refusal in `FromRecordError`
  with no difference a caller sees; `board::record` now throws the `ReopenError` itself,
  and `a_record_of_another_version_is_refused_naming_both_either_way` holds the same
  name and text at all four sites, unchanged.
- **R2 is not run.** Its range is what the pull request merged, and nothing has merged.

### Handed back

What remains, none of it authorised in this run:

- **Step 10:** the `wasm` job's artefact, inspected for the declarations of `Board`, one
  function and one value, which needs the push; the push and the pull request.
- **R2**, before step 11, over what the pull request merged.
- **Step 11:** the release, by `cargo release` on its branch, its pull request, the tag
  on `main`, the push and the deployment approval, and the `gh api` read of the package's
  versions. No `pawdoku-wasm-v*` tag is pushed until the `release` environment, with its
  required reviewer, and the tag ruleset exist and read back with the two `gh api` reads
  in `docs/operations/maintenance.md` (T15's R2); until then GitHub would run
  `release-npm.yml` in an environment it creates with no approval rule. The version is
  0.1.0 if T15's has not been published by then, otherwise 0.2.0 (Open points settled).
- **Step 12:** the G ticket, below, brought to the names and the version published;
  `status: done` and the index row, in a closing pull request. `status` stays `open`.
- **Acceptance** not yet met: the package published by `release-npm.yml`, the G ticket
  updated to what was published.

### The G ticket

**Unreleased.** Written against the names as this branch exports them, at `8cfb839`
with the R1 fixes; nothing here has been published, and step 12 brings it to what is.
Part A is the draft under Steps, unchanged. Part B, as built:

> **Part B, after the release that carries the engine** (T42; 0.1.0 if T15's release
> has not gone out first, otherwise 0.2.0):
>
> - `src/lib/engine/` (new): one module that calls the package's default export,
>   `init()`, once, in the browser only (from `onMount`, or behind `browser` from
>   `$app/environment`), never during prerender; and an adapter that holds one `Board`
>   per puzzle in play, opened by `Board.open(givens)` or `Board.reopen(record)`, and
>   calls `board.free()` once when the puzzle is left. The seven operations are
>   `place(position, digit)`, `erase(position)`, `writeMark(position, digit)`,
>   `strikeMark(position, digit)`, `check(position)`, which returns the `Check`,
>   `undo()` and `redo()`; the game redraws from `view()`, a `BoardView`, and reads
>   history through `readingAfter(index)`. A position is `{ row, column }`, each 1 to 9.
> - Keeping a game: `JSON.stringify(board.write())`, a `BoardRecord`, through the
>   existing `StoragePort`, and `Board.reopen(JSON.parse(stored))` on return. A record
>   names its format version, `recordVersion()` (`"record-1"` today), and a record of
>   another version throws an `Error` named `ReopenError` whose message names both;
>   libpawdoku promises nothing across versions. The game discards a record that does
>   not reopen and tells the player, stated in `pawdoku.allium` first.
> - A new puzzle: a seed drawn from the platform's cryptographic source as a `bigint`,
>   handed to `generateFromSeed(tier, seed)` with `tier` one of `"One"` to `"Five"`; it
>   returns a `GenerationResult` whose `givens` open the board, and the game keeps
>   `seed`, `tier`, `generation_version` and `random_version` with the puzzle, which is
>   what replays it. A refusal throws a `GenerationRefused`, `name` `"GenerateError"`,
>   with `tier` and `draws_taken`. Whether the game generates on the device at all is
>   libpawdoku's `generation.allium` open question and decision 0012's; the game asks
>   before it ships this.
> - Every refusal is an `Error` whose `name` is one of `EngineErrorName` and whose
>   message is the engine's text; a value of the wrong shape throws `name` `"Error"`.
> - The stateless functions, `open(givens)`, `play(record, operation)`, `read(record)`
>   and `readingAfter(record, moveIndex)`, are there for a caller that holds no board;
>   each reopens the record it is handed. The game says where it uses them, if anywhere.
> - Tests: a vitest file per adapter, loading the module in `beforeAll` with
>   `initSync({ module: bytes })` over `pawdoku_wasm_bg.wasm` read through
>   `enginePath`; the browser path is proved by a built preview in which a puzzle is
>   played, the page reloaded, and the puzzle found as it was left.
> - `pawdoku.allium`: keeping a game, discarding a record that does not reopen, and
>   where a puzzle comes from, stated there before they are built.
>
> **Its proof**, as drafted under Steps.

### Adversarial review

- **R1, round 1**, before step 10, on the range from the start (`4023b35`) to the
  checkpoint `003c1d9`, focus where the declarations and the values can differ: needs
  attention, one finding.
  - [medium] A throwing position getter leaves a held `Board` unusable
    (`crates/pawdoku-wasm/src/board.rs`, lines 171 to 176). Valid, and wider than
    reported. Reproduced against the built package: after
    `board.erase({ get row() { throw new RangeError("row") }, column: 3 })` threw its
    `RangeError`, `board.write()` and `board.place(...)` threw `recursive use of an
    object detected which would lead to unsafe aliasing in rust`. serde-wasm-bindgen
    0.6.5 reads fields through an indexing getter that does not catch, and its refusal's
    text formats the value through more of the caller's code, while the glue holds the
    `Board`'s borrow, which a throw across WebAssembly never releases; so every method
    taking an argument was reached, `readingAfter` and a hostile digit among them. Fixed
    in `105c2ad` by `types::read_held`, which copied an object through `JSON.stringify`
    and `JSON.parse`, both catching, and refused a function unread; tested red first in
    `tests/surface.rs` and through the glue in `tests/package/play.mjs`.
  - The reviewer's note that its read-only sandbox blocked the recipes is an environment
    note, not a finding; they ran here.
- **R1, round 2**, on the range from the start to `105c2ad`, the same focus: needs
  attention, two findings. Round 2 is the procedure's limit.
  - [medium] The JSON copy changes valid positions (`crates/pawdoku-wasm/src/types.rs`,
    lines 277 to 279). Two claims, judged apart. That a class with prototype getters is
    refused held but read by `play`: invalid; `play` reads an operation through
    `Object.entries`, own fields only, and refuses the same instance with
    ``missing field `row` ``, as the copy did. That a position whose `toJSON` names
    another cell moves the held board there: valid; a held `place` of
    `{ row: 1, column: 3, toJSON() { return { row: 1, column: 4 } } }` recorded a move at
    column 4, while `play` refused the value. Fixed in `8cfb839`: `read_held_position`
    reads a position from its own fields through a catching `Object.entries`, refusing a
    function in any field, `toJSON` among them, and an object in `row` or `column`,
    unread; a digit and an index are read only as primitives. Tested red first in both
    places. Left: the record path also read an array `[1, 3]` as a position, refused
    since by the final code review (Deviations), and looks into unknown fields, which
    the held path does not; neither is a value `Position` declares.
  - [medium] `generate` reads its tier while the `RandomStream` is borrowed
    (`crates/pawdoku-wasm/src/generation.rs`, lines 41 to 45). Valid; reproduced with
    `{ get Three() { throw new RangeError("tier") } }`. Fixed in `8cfb839`: the tier is
    read as a primitive, an object refused unread, and the stream stays usable; tested
    with a throwing getter and a `Proxy`, after which `index` is `0n` and `nextDraw` and
    a valid `generate` match a fresh stream.
  - Not a finding: `types.rs`'s module documentation still named one reader for every
    export; corrected in step 10's commit.
- **R1, round 3**, model `gpt-6-astra` (thread `01a1b790-d149-7651-b3af-bc06945d8a1c`),
  over the branch against `main`: needs attention, one finding. Its fix is not yet
  reviewed by Codex.
  - [medium] `play` deserialises the operation, not the position it validated
    (`crates/pawdoku-wasm/src/types.rs`, lines 325 to 330). Valid; reproduced red in
    `tests/surface.rs`. `Object.assign([1, 4], { row: 1, column: 3 })` was placed at
    column 3 by a held `place` and at column 4 by `play`, and an operation whose
    `position` getter answered an object and then `[1, 4]`, or whose `digit` getter
    answered 4 and then 5, was played as its second answer. Fixed in `40530a8`:
    `read_operation` reads the operation's own fields once, its `position` through
    `read_held_position` and its `digit` through `read_held`, and serde reads a plain
    copy of `kind`, `position` and `digit`. Left: a `digit` or `position` the operation's
    kind does not take, which serde ignored, is now read and may be refused; no
    declared `Operation` carries one. Closed by the code review below.
  - The reviewer's note that its read-only sandbox blocked the recipes is an environment
    note, not a finding; `just wasm-test`, `just wasm-build` and `just wasm-package-test`
    ran here.
- **R2** is not run (Deviations).
- **Code review** (Claude Code's `/code-review high`, over the branch against `main`,
  2026-11-07; the findings on this ticket's files, answered in the working tree and not
  yet committed, so no commit is named).
  - [medium] `play` refuses an operation that a held `Board` accepts
    (`crates/pawdoku-wasm/src/types.rs`, `read_operation`). Valid, and the item round 3
    left: `{ kind: "Erase", position, digit: null }` and `{ kind: "Undo", position: null }`
    were refused with serde's `Error`, while `board.erase(position)` ignores an extra
    argument and the core's `Operation`, internally tagged without `deny_unknown_fields`,
    ignores the extra field, against `lib.rs`'s "same values, same refusals, both ways".
    Fixed: `read_operation` lists the own fields once, reads `kind` from that list, and
    pre-reads `position` only for `Place`, `Erase`, `WriteMark`, `StrikeMark` and
    `CheckCell` and `digit` only for `Place`, `WriteMark` and `StrikeMark`; a `kind`
    that is missing or not a string is forwarded alone for serde to refuse. Tested red
    first in `tests/surface.rs` (`play_leaves_a_field_its_kind_does_not_take_unread_as_a_held_board_does`,
    which against the old code was refused with `invalid type: unit value, expected u8`,
    the `null` digit read as a `u8`), green with the fix.
  - [low] `wasm-build` pinned `wasm-opt version 121` beside `pyproject.toml`'s
    `binaryen = "==121"`. Valid. The recipe now reads the pin from `pyproject.toml`, as
    `wasm-guard` reads its own from `Cargo.lock`, and refuses when the pin is missing.
    Both refusals were exercised: the missing pin through `just wasm-build` with
    `pyproject.toml` edited to a range and restored; the wrong version and a missing
    `wasm-opt` by running the recipe's own lines, taken from `just --show wasm-build`,
    in `sh` with a stub `wasm-opt` answering `wasm-opt version 120` and with none on
    `PATH`, because editing the pin to `==120` stales `pixi.lock` so `lock-check`
    refuses first, and pixi's `just` shim puts the environment's `wasm-opt` ahead of a
    stub. Each refused, naming both versions, and the real `wasm-opt` passed.
    The `just wasm-build` row of `docs/reference/commands.md` names the pin, not 121.
  - [low] Two boards in `tests/package/play.mjs`'s `live` table were never freed.
    Valid; a `freeing(board, use)` helper frees each in `finally`.
  - [low] `Stream::index` in `src/lib.rs` repeated `as_dyn`'s match. Valid; a shared
    `as_dyn_ref` borrows the trait object and `index` calls it.
  - Left alone: `BoardView::of` clones a `Reading` per move (`crates/pawdoku/src/board/view.rs`),
    a stated trade so a view's moves can read back; replacing `Vec<Move>` with a wire-only
    summary changes the core's public surface, so it is the maintainer's call.

### Open points settled

- **Whether `Board`'s operations return the view:** no, as proposed. Each returns nothing,
  `check` the check it added, and the game calls `view()` to redraw, which costs no search
  on a held board.
- **Package version:** 0.1.0 here, and 0.1.0 at a first release if T15's has not been
  published by then (Deviations).

## Open points

- **Whether `Board`'s operations return the view.** Proposed: no; each returns nothing
  (or the check, for `check`), and the game calls `view()` when it redraws, which on a
  held board costs no search. Play from a record returns the view with the record
  because a second call would reopen again.
- **Package version.** 0.2.0 is a minor bump before 1.0, as T13's policy allows for an
  addition. If T15's 0.1.0 has not been published when this ticket is done, this
  release is the first, at 0.1.0, and the notes say so.
