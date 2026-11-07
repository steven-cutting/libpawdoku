---
id: S12
title: "Spike: the engine's TypeScript surface, typed plain values, errors, tests, size and lifetime"
status: done
depends_on: [S04]
parallel_with: [T13, T15, T40, T43]
branch: ticket/s12-typescript-surface
estimated_size: M
---

# S12: Spike: the engine's TypeScript surface, typed plain values, errors, tests, size and lifetime

## Context

The game reaches the engine through WebAssembly. S04 designed the crate and the package,
and T15 builds them with one class, `RandomStream`, and the random version. Nothing yet
says how the board, the solver, play and the generator cross. T42 builds that surface,
in `crates/pawdoku-wasm`, once T41 has built what it stands on in the core. This spike
gathers the evidence T41 and T42 need and ends in a recommendation for each question
below.

**What is already decided, and by whom.**

- **The game drives the engine both ways** (the maintainer, 2026-10-31). A long-lived
  `Board` class, opened from givens or reopened from a record, whose methods are the
  seven operations of `board.allium`'s `Playing` surface and whose readings come back as
  plain values; and a set of stateless functions that take plain values in and give
  plain values out, the caller keeping the record. The stateless functions are
  implemented in Rust, in the core, so that TypeScript and Python share one
  implementation. They are a new surface, so `board.allium` states them first (T40) and
  the core builds them (T41); this crate exposes both (T42).
- **Why a stateless call costs a search.** A `Board` holds its puzzle's solution and,
  by `TheSolutionIsNeverShown`, never serialises (`crates/pawdoku/tests/api_bounds.rs`
  holds it under the `serde` feature). A `Record` holds no solution. So a call that is
  handed a record reopens it: `Board::reopen` puts the givens to the solver and makes
  every move again (`docs/explanation/architecture.md`, "The record"). That cost is
  measured here (question 6).
- **What S04 settled for every binding** (S04 Step 4): the seed is a `u64`, a `bigint`
  in TypeScript; the library's `SeededStream` is the default and `ReplayStream` the
  fake; no live callback; a binding class holds the private `Stream` enum and asserts
  its bounds in-crate; errors carry their `Display` text, which invariant 3 makes
  stable.
- **What the specifications expose.** `board.allium`'s `Playing` exposes the board's
  status, `is_full`, `is_consistent`, `can_undo`, `can_redo`; each cell's row, column,
  digit, `is_given`, `is_conflicting` and its note while shown; each move's index, kind,
  target, digit and `is_undone`, and for every cell `digit_after` and `note_after`; and
  each check's index, `after_move`, target, digit and `is_right`. `solver.allium`'s
  `SearchResult` exposes the verdict, the count of guesses and the solved branches'
  cells. `generation.allium`'s `GenerationResult` exposes the tier, the status,
  `draws_taken` and, once finished, the givens; not the solution grid, though the Rust
  `generate` returns `WellPosed`, which holds it. `TheSolutionIsNeverShown` holds for
  the board.
- **What a record promises across versions** (the maintainer, 2026-10-31). A record
  names its format version, a record of another version is refused with a clear
  message, and no compatibility is promised. T43 (`tickets/T43-record-version.md`)
  states it in `board.allium` and builds it in the core; it is not this spike's to
  weigh. The record this spike's probe serialises may not carry the field yet, and no
  question below depends on it.

**What is known without a probe.** A value returned through `serde_wasm_bindgen` is a
`JsValue`, which the generated declarations type as `any`, so plain values lose their
types unless something names them. wasm-bindgen 0.2.130's macro accepts
`unchecked_return_type`, `unchecked_param_type`, `return_description`,
`param_description` and `typescript_custom_section` (all five parsed in
`wasm-bindgen-macro-support-0.2.130/src/parser.rs`, read in the local cargo registry on
2026-10-31), which together let a binding name a hand-written TypeScript interface for a
`JsValue` with no new dependency. tsify 0.5.8 (`madonoharu/tsify`, released 2026-08-23,
`MIT OR Apache-2.0`) derives the interfaces from the Rust types instead, and is a new
dependency; tsify-next 0.5.6 (2025-04-20) is the fork that carried it while it was
quiet. TypeScript 6.0.2 is on conda-forge (`pixi search typescript`, 2026-10-31), so a
type check needs no npm. Only these types derive serde today: `Record`, `Position`,
`Given`, `Tier`, `SeededStream` and `ReplayStream`. `Record`'s serialised shape is
snake_case and pinned by
`crates/pawdoku/tests/snapshots/snapshots__tests__snapshot_the_record_of_the_scripted_game_as_json.snap`.

**What S04's rule on draws becomes for the generator.** S04 Step 4, point 2, says every
engine result that consumed draws carries the seed, the random version, the ordered
`Draw` records and the cursor. `Draw` is `human-solving.allium`'s value, held by its
plans, effects and trace steps, none of which is built. `generation.allium` states the
basic way's replay "over the draws taken, which a seed stands for only when the stream
begins at index zero", and its `GenerationResult` exposes `draws_taken`, not draws.
`pawdoku::generation::generate` returns no draws, and `architecture.md` lines 300 to 303
say what replays a puzzle: the tier, `GENERATION_VERSION`, `RANDOM_VERSION`, the seed
and the index the stream stood at. So a generation result carries those, and `Draw`
records stay the rule for human-solving's results when they are built. This spike
confirms nothing more is needed (question 3); if something is, it is a spec change and
a core change, handed back, never a binding feature.

Read first: `AGENTS.md`; S04's questions (c) and (f), Steps 4 and 5; T15, T40 and T41;
`docs/specs/board.allium` (`Playing`, `Recording`, `Reopening`),
`docs/specs/solver.allium` (`SearchResult`), `docs/specs/generation.allium`
(`Generating`, `GenerationResult`, Scope's Replay); `crates/pawdoku/src/board.rs`,
`board/record.rs`, `random.rs`, `generation.rs`, `solver/result.rs`;
`docs/explanation/architecture.md`; `docs/decisions/0007-dependency-policy.md` and
`0008-no-std-core.md`.

## Goal

A recommendation for each of the nine questions below, with the evidence beside it, and
the amendments T41 and T42 need, in the hand-back notes. Nothing is created outside this
file and the index row. The probes are throwaway crates in the session's scratch
directory, each with the workspace's three lint tables copied into its own `[lints]`,
`rust-toolchain.toml` copied beside it, and an empty `[workspace]`, as S04's were; one
of them depends on the core by path, read-only.

## Non-goals

- Building any of it: T41 and T42 do.
- Changing the core, a specification, the `Justfile`, a manifest or a workflow. A need
  found here is a hand-back.
- The CLI and Python crates (T16, T17); this spike notes what T17 should mirror.
- The game's side: the G ticket is drafted in T42.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S12-typescript-surface.md` | ticket | The evidence and the recommendations; `status: done` |
| `tickets/README.md` | ticket index | S12's row set `done` |

## Steps

1. Create the worktree on `ticket/s12-typescript-surface` from `main`. The network uses
   this spike needs are authorised by this ticket and each is read-only: the crates.io,
   npm and conda-forge registries; the documentation of wasm-bindgen, wasm-pack, tsify,
   TypeScript, Node and binaryen; `gh api` reads; cargo fetches for the probes; and
   `pixi exec` of `typescript`, `wasm-pack`, `wasm-bindgen-cli`, `nodejs` and `binaryen`
   at the versions T15 pins, outside the repository's environment. Record each host used
   beside the fact it supports. Probes, their outputs and any report a sub-agent writes
   go in `ai_tmp/s12/` or the session's scratch directory; no sentence below cites them.

2. **Build the probe.** A cdylib on wasm-bindgen 0.2.130, serde-wasm-bindgen 0.6.5 and
   the core by path with `features = ["serde"]`, exporting: a `Board` class holding
   `pawdoku::board::Board` with `open(givens)`, `place`, `undo`, `cells()` and `write()`;
   `search(givens)`; and `generate(tier, stream)` taking a `RandomStream` class by
   `&mut`. Its plain values are small `#[derive(Serialize, Deserialize)]` structs in the
   probe standing in for T41's. Pack it with `wasm-pack build --target web --mode
   no-install` and keep the `.d.ts` and `.js` it writes.

3. **The questions.** Answer each with the evidence: the generated declarations quoted,
   the compiler's or `tsc`'s output quoted, a measured number with how it was measured.

   1. **Typing plain values.** Compare three ways, each over the probe: (a)
      `typescript_custom_section` interfaces with `unchecked_return_type` and
      `unchecked_param_type` on the exports; (b) tsify 0.5.8; (c) an exported class per
      value. Write a small consumer module in strict TypeScript that opens a board,
      plays, reads the cells and writes the record, and quote `tsc --noEmit --strict`
      over it with each. For (a), say how the hand-written interfaces are held equal to
      the serde shape so that they cannot drift: for example, a test that type-checks
      the record JSON snapshot against the record's interface, and a `wasm-test` case
      that compares each value's keys with the interface's. For (b), its maintenance
      record since 2024 and whether it works with serde-wasm-bindgen's output as
      configured. For (c), the cost of a wasm object per cell. Recommend one. **If the
      recommendation needs a new dependency, it is handed to T02 with the decision-0007
      note the record asks for, and not added.** Also say what the record's TypeScript
      name should be: `Record` shadows TypeScript's built-in `Record<K, V>` in any module
      that imports it, so the ticket writer proposes `BoardRecord`; confirm or replace.
   2. **Errors.** `JsError::new(&error.to_string())` gives an `Error` whose `name` is
      `"Error"` and whose message is the `Display` text. The other shape is a
      `js_sys::Error` whose `name` is the Rust error type (`PlayError`, `ReopenError`,
      `SolveError`, `GenerateError`, `RandomError`, and T41's error for play from a
      record), so that a game can branch on the kind without parsing a message, at the
      cost of `js-sys`. Show both from the probe, quote what each looks like when
      caught, and say what the declarations can say about a throw (nothing, or a
      `@throws` line from a doc comment). Recommend one, and say what T17 should mirror
      in Python (one exception class with the text, or one per error type).
   3. **Seeds and draws.** Prove that `generate(tier, stream)` with the `RandomStream`
      class passed by `&mut` advances the JavaScript object's stream, that `index` after
      minus `index` before equals the draws a generation took (23 for tiers 3 to 5 when
      the first grid attempt succeeds, 104 for tiers 1 and 2, `architecture.md` lines 296
      to 298), and that a seed beyond `Number.MAX_SAFE_INTEGER` survives the round trip
      as a `bigint`. Say what a stateless generation result must carry for
      `generation.allium`'s Replay to hold (the ticket writer proposes: the tier, the
      seed, the stream's index at the start, `draws_taken`, `GENERATION_VERSION`,
      `RANDOM_VERSION` and the givens, and never the solution), and whether `Draw`
      records are needed for anything built today.
   4. **Casing.** serde's default is the Rust field names, snake_case, which is
      `Record`'s shape today, the specifications' names (`is_given`, `after_move`,
      `draws_taken`) and Python's idiom. TypeScript's idiom is camelCase, which
      `#[serde(rename_all = "camelCase")]` gives, and which would change `Record`'s
      shape and its snapshot. One shape serves both bindings, because the values are
      the core's. The ticket writer proposes snake_case for every plain value, with the
      class methods in camelCase through `js_name`. Confirm or replace, with what the
      declarations look like under each.
   5. **Where tests run.** `wasm-bindgen-test-runner` runs the Rust exports through glue
      it generates itself, not through the `web` glue in `pkg/` that the game imports.
      Prove whether a Node module can import `pkg/pawdoku_wasm.js`, call
      `initSync({ module })` with the bytes read from disk, and exercise the probe; if
      so, that is a test of the published package with no browser. Then say what only a
      browser would prove: the default `init()` fetches the `.wasm` through
      `new URL(..., import.meta.url)`, which Node's `fetch` cannot do for a `file:` URL,
      and which the game's Vite build rewrites. Check whether `chromedriver` and
      `geckodriver` are on conda-forge for both platforms (`geckodriver` 0.37.1 is, for
      `linux-64`, on 2026-10-31) and what a headless-browser gate would cost a `check`
      run. Recommend: Node only here and the browser on the game's side, or a browser
      job here, and why.
   6. **Size and speed.** On the probe built `--release` with the workspace's
      `[profile.release]` (`lto = "fat"`, `codegen-units = 1`): the `.wasm`'s bytes and
      its gzip and brotli sizes with `wasm-opt` off, and with binaryen 121's `wasm-opt`
      at `-Oz` and `-O3`; and with `opt-level = "s"` and `"z"` for the wasm crate's
      profile. In Node, the time to generate one puzzle of tier 1 and of tier 3 from ten
      seeds; to open a board; and to reopen a record of 0, 50 and 200 moves, which is
      the cost of one stateless call. Recommend whether `wasm-opt` is turned on and at
      what level (it needs `binaryen = "==121"` pinned in `pyproject.toml`, a T00
      follow-up for T42), and whether the stateless cost is acceptable for a call per
      move.
   7. **Lifetime.** A wasm-bindgen class owns memory in the module and has `free()`.
      Quote whether the 0.2.130 glue registers a `FinalizationRegistry` without a flag,
      and whether it implements `Symbol.dispose` so that TypeScript's `using` frees a
      `Board`. Recommend how the game holds a board: for the life of a puzzle in play,
      freed when the puzzle is left.
   8. **The readings after a move.** `Playing` exposes, for every move and every cell,
      `digit_after` and `note_after`. Carried inside every move, that is 81 cells times
      the number of moves on each read. Propose a shape that exposes the same facts
      without that, for both the class and the stateless functions; the ticket writer
      proposes a separate `reading_after(move_index)` giving the 81 cells as they stood.
      Say whether the shape states anything `board.allium` does not, which would be a
      spec change for T40 to carry.
   9. **What the declarations say.** Quote the `.d.ts` the recommended combination
      writes for the `Board` class, one stateless function and one plain value, with
      their doc comments, and say whether it is good enough for the game to use without
      a hand-written wrapper.

4. **Write the recommendations** as the amendments T41 and T42 take, each naming the
   section of that ticket it changes, and anything handed to T02, T40, a T00 follow-up,
   T17 or the specifications.

5. Set `status: done` and the row in `tickets/README.md`, and commit. Stop before
   pushing.

**Adversarial review**, by the procedure in `tickets/README.md`, "Adversarial
review":

- **R1**, after step 4 and before step 5: the recommendations as drafted. Focus: evidence against claim, whether each recommendation follows from what its probe showed.

## Acceptance criteria

- Each of the nine questions has a recommendation, the evidence quoted beside it, its
  sources and dates.
- No new dependency is recommended without the decision-0007 note, handed to T02.
- Every proposal of the ticket writer's above is confirmed or replaced, with the reason.
- `git status --porcelain` lists this file and `tickets/README.md` only.

## Verification

```sh
git status --porcelain
rg -n '^### ' tickets/S12-typescript-surface.md
```

Expected: the two files; the hand-back sections below, filled.

## Hand-back notes

### What was verified, and how

Run on the night of 2026-11-06 to 2026-11-07 on the branch `implement-typescript-surface-fullsend`,
from `3ce08de52677812198c7edaae851841193bb72d9`, on an Apple M5 Pro under macOS. Every
timing below is that one machine in Node, so it is an upper-end desktop figure; no phone
was measured. The probes were throwaway crates in a scratch directory, each with the
workspace's three lint tables, `rust-toolchain.toml` and an empty `[workspace]`, one
depending on `crates/pawdoku` by path with `features = ["serde"]`; nothing in the core
changed. T40, T41, T43 and T15 have not landed, so the plain values are small structs in
the probe, the record carries no version, and `RandomStream` is the probe's own. The
figures for the plain values therefore stand in for T41's, and T41's are what T42 measures.

**Versions, each read on 2026-11-06.** One has moved since the tickets were written.

| Tool | Ticket's | On the day | Source |
| --- | --- | --- | --- |
| `wasm-bindgen`, `wasm-bindgen-cli` | 0.2.130 | 0.2.130 | `crates.io/api/v1/crates/<name>`; `pixi search` for conda-forge, `linux-64` and `osx-arm64` |
| `js-sys` | 0.3.107 | 0.3.107 | crates.io |
| `wasm-bindgen-test` | 0.3.80 | 0.3.80 | crates.io |
| `serde-wasm-bindgen` | 0.6.5 | 0.6.5, last released 2024-02-27 | crates.io |
| `wasm-pack` | 0.15.1 | 0.15.1, on crates.io and on conda-forge for both platforms | crates.io; `pixi search` |
| `binaryen` | 121 | 121, both platforms | `pixi search` |
| `typescript` | 6.0.2 | 6.0.2, both platforms | `pixi search` |
| `nodejs` | 26.12.1 (T15) | **26.13.0**, both platforms; the probes used it | `pixi search` |
| `tsify` | 0.5.8 | 0.5.8; `tsify-next` still 0.5.6 | crates.io |

Rust was 1.98.1 by rustup, with `wasm32-unknown-unknown` and `wasm32v1-none` installed.
Tools ran through `pixi exec` at those versions, with `brotli` 1.2.1 for the sizes, outside
the repository's environment.

**The probe built as the ticket asks.** `wasm-pack build --target web --mode no-install`
wrote the `.d.ts` and `.js` quoted below. It exports `Board` with all seven operations,
`open`, `reopen`, `view`, `cells`, `readingAfter` and `write`; four stateless functions;
`search`; `generate(tier, stream)` and `generateFromSeed(tier, seed)`; and `RandomStream`.

#### 1. Typing plain values

The same strict consumer (open, place, undo, read the cells, write the record) under
`tsc --noEmit --strict --target es2022 --lib esnext,dom --module esnext --moduleResolution bundler`:

| Way | `tsc` | What it costs |
| --- | --- | --- |
| (a) `typescript_custom_section` with `unchecked_return_type` and `unchecked_param_type` | clean, no output | Interfaces written by hand; held below, within the limit stated there |
| (b) tsify 0.5.8 over the core's `Record` | `pawdoku_wasm.d.ts(83,27): error TS2314: Generic type 'Record' requires 2 type argument(s).` | A new dependency, and the wire values have to be declared twice, or the core has to take it on |
| (c) a class per value | clean, no output | 81 `free()` calls per read, and a value nothing can store |

(a) also refuses misuse. A second module, each line meant to fail:

```text
a-wrong.ts(4,14): error TS2322: Type 'number | null' is not assignable to type 'number'.
a-wrong.ts(5,26): error TS2551: Property 'isGiven' does not exist on type 'BoardCell'. Did you mean 'is_given'?
a-wrong.ts(6,13): error TS2345: Argument of type '{ row: number; }' is not assignable to parameter of type 'Position'.
a-wrong.ts(7,36): error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.
a-wrong.ts(8,18): error TS2345: Argument of type '"Six"' is not assignable to parameter of type 'Tier'.
a-wrong.ts(9,32): error TS2322: Type '"Fly"' is not assignable to type '"Place" | "Erase" | "WriteMark" | "StrikeMark" | "CheckCell" | "Undo" | "Redo"'.
a-wrong.ts(10,30): error TS2345: Argument of type '{ kind: "Place"; position: { row: number; column: number; }; }' is not assignable to parameter of type 'Operation'.
```

**Holding (a)'s interfaces to the serde shape, run rather than proposed.** Three links. The
first two put the interface and the live value on the same text, and hold one direction only;
the third holds the other:

1. **The interface against the snapshot.** A small script strips the insta header from
   `snapshots__tests__snapshot_the_record_of_the_scripted_game_as_json.snap` and writes
   `export const record = <the JSON> satisfies BoardRecord;`. `satisfies` on a literal does
   the excess-property check, so a field the interface has lost, or the JSON has gained, shows.
   As written it is clean. With the interface's `undone` renamed:

   ```text
   record-drift1.ts(315,3): error TS2353: Object literal may only specify known properties, and '"undone"' does not exist in type 'BoardRecord'.
   ```

   With a `version` field added to the JSON, which is what T43 will do:

   ```text
   record-drift2.ts(3,3): error TS2353: Object literal may only specify known properties, and '"version"' does not exist in type 'BoardRecord'.
   ```

2. **The live value against the snapshot.** In Node, `JSON.stringify(Board.reopen(snap).write())
   === JSON.stringify(snap)` printed `true`, key order included, for the snapshot's 11 moves,
   1 undone and 2 checks. That holds `serde_wasm_bindgen`'s shape equal to `serde_json`'s,
   given the serialiser setting question 4 fixes.

A snapshot holds `seed: 0` as a number where the interface says `bigint`, so the generation
value goes through a mapped type, also run:

```ts
type Wire<T> = { [K in keyof T]: T[K] extends bigint ? number : T[K] extends bigint | null ? number | null : T[K] };
```

```text
gen-snapshot.ts(4,162): error TS2353: Object literal may only specify known properties, and 'solution' does not exist in type 'Wire<GenerationOutcome>'.
```

That line is the planted `solution` field; the same literal without it is clean.

**Those two links are one-way.** `satisfies` refuses an interface narrower than the value and
never one wider, and showing every member in a snapshot does not change that. Seven widenings
were planted in the declarations, with the record's snapshot and a literal of all seven
operations satisfying them. The snapshot guard stayed clean on all seven, and a consumer could
then type-check a call serde refuses. The third link, a module type-checked against the same
declarations, stopped each:

| Planted in the `.d.ts` | Snapshot guard | Declaration module, first error |
| --- | --- | --- |
| `Place`'s `digit?: number` | clean | `TS2322`, the optional-keys line |
| `\| { kind: "Fly" }` added to `Operation` | clean | `TS1360`, the table of operations |
| `"Six"` added to `Tier` | clean | `TS1360`, the table of tiers |
| `is_given?: boolean` | clean | `TS2322`, the optional-keys line |
| `undone: number \| string` | clean | `TS2322`, the read-back line |
| `undone?: number` | clean | `TS2322`, the optional-keys line |
| `Position`'s `column?: number` | clean | `TS2322`, the optional-keys line |

**The third link, the declarations against the engine.** The module has four parts, each run above:

```ts
type OptionalKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? K : never }[keyof T];
type Same<A, B> = (<X>() => X extends A ? 1 : 2) extends (<X>() => X extends B ? 1 : 2) ? true : false;

// One value per declared member: a new member is a missing key.
export const operations = { Place: { kind: "Place", position: here, digit: 4 }, /* the other six */ }
  satisfies { [K in Operation["kind"]]: Extract<Operation, { kind: K }> };
// Optionality stated exactly.
export const optional: [Same<OptionalKeys<BoardCell>, "note">, Same<OptionalKeys<BoardRecord>, never>] = [true, true];
// A field read back into its narrow type.
const undone: number = record.undone;
// What must not type-check; an unused directive is itself an error.
// @ts-expect-error a Place needs its digit
export const noDigit: Operation = { kind: "Place", position: here };
// @ts-expect-error no such operation
export const fly: Operation = { kind: "Fly" };
```

```text
guard.ts(25,24): error TS2322: Type 'true' is not assignable to type 'false'.
guard.ts(36,1): error TS2578: Unused '@ts-expect-error' directive.
```

That is `Place`'s `digit?: number`. The Node module then puts each table value to the engine,
which is what ties a declared member to the engine: the seven operations each reached it and
came back as a result or a `PlayError`, and the two the declarations must not allow did not:

```text
op Fly Error | unknown variant `Fly`, expected one of `Place`, `Erase`, `WriteMark`, `StrikeMark`, `CheckCell`, `Undo`, `Redo`
op PlaceNoDigit Error | missing field `digit`
tier Six Error unknown variant `Six`, expected one of `One`, `Two`, `Three`, `Four`, `Five`
```

**What is still not held.** The third link holds what it names: an interface with no optional-keys
line or read-back is as open as before, so T42's module has one for every exported value and its
list is reviewed by hand. A type says nothing of a range, so `digit: number` admits 260, which serde
refuses at run time (question 4). So these interfaces are guarded rather than proved equal: drift
in a listed field, member or optionality fails the gate.

**(b) tsify's record since 2024.** `madonoharu/tsify`, `MIT OR Apache-2.0`, not archived, 515
stars, 33 open issues, last pushed 2026-11-01 (`gh api repos/madonoharu/tsify`, read 2026-11-07). The newest 100 commits
reach back to 2024-01, so that count is capped and early months may be short; they show no commit in
2025-06 to 2025-09, 2025-11 to 2025-12 or 2026-03 to 2026-07, and 31 in 2026-08. Releases 0.5.5
(2025-04-13), 0.5.6 (2025-10-14), 0.5.7 (2026-08-20) and 0.5.8 (2026-08-23). It is
maintained in bursts. Three things decide against it:

- **It reads the tokens of its own item, and the values are the core's.** Wrapping the core's
  type wrote `export type BoardRecord = Record;`, the `TS2314` above. Through the wrapper it
  now recommends, `tsify::Ts<Record>` fails to compile:
  `` error[E0277]: the trait bound `Record: Tsify` is not satisfied ``.
- **The core cannot derive it.** A `no_std` crate with `tsify` 0.5.8 (`default-features =
  false, features = ["js"]`) on `wasm32v1-none` fails with
  `` error[E0463]: can't find crate for `std` `` in `slab` and `futures-core`, which is
  invariant 5. So the wire values would be declared a second time in the wasm crate and again
  for Python, which is the drift T41 exists to prevent, and those copies would need a guard of
  their own.
- **The obvious attributes are deprecated in the version in hand.** `#[tsify(into_wasm_abi)]`
  warns `into_wasm_abi/from_wasm_abi are deprecated as they cause memory leaks
  (https://github.com/madonoharu/tsify/issues/65). Consider using tsify::Ts instead.`; `-D
  warnings` would stop on it. Issue 65 is open, last updated 2026-08-21.

It does work with `serde-wasm-bindgen` where it can see the type: its `js` feature uses that
crate, and the `.d.ts` it wrote for a value of its own was correct, with `digit: number |
undefined` and, under `large_number_types_as_bigints`, `seed: bigint | undefined` and
`moves: bigint` for a `usize`. Only the declarations were read; values from it were not run.

**(c) a class per value.** Reading the 81 cells and every field, median of 2001 reads on the
`opt-level = 3` release build: plain values 0.0205 ms, classes 0.0224 ms with each freed. So
speed does not decide it. Ownership does: 20000 reads whose cells were not freed grew the
module's memory by 65,273,856 bytes, and `JSON.stringify` of a class value gives
`{"__wbg_ptr":1116800}`, a pointer, which is not a record a game can keep. Plain values also
cannot be classes for the stateless way, whose whole point is values in and out.

**The record's name.** A module that imports an interface named `Record` and writes
`Record<string, number>` gets `error TS2315: Type 'Record' is not generic.`

#### 2. Errors

Caught and quoted from the probe, for `PlayError::Solved`:

| | `name` | `message` | `instanceof Error` | `String(e)` |
| --- | --- | --- | --- | --- |
| `JsError::new` | `Error` | `the puzzle is solved, and solved is final` | `true` | `Error: the puzzle is solved, and solved is final` |
| `js_sys::Error` with `set_name` | `PlayError` | the same | `true` | `PlayError: the puzzle is solved, and solved is final` |

Both have `Error.prototype` as their prototype and a `stack`. `set_name` sets an own property
on an `Error`, not a subclass, so a consumer branches on `e.name` and there is no
`instanceof PlayError`. The named shape, every refusal from the release build:

```text
PlayError | the cell at row 1, column 1 holds a given | true
SolveError | the givens have more than one solution | true
ReopenError | the record's undone count, 2, exceeds its move count, 0 | true
Error | missing field `moves` | true
PlayError | the cell at row 1, column 1 holds a given | true
GenerateError | the replay stream is exhausted at draw 5 | true
RandomError | draw 0 of the replay script is 1.5, outside [0, 1) | true
```

The fourth line is a value of the wrong shape, read by serde, not a refusal of the engine's.
With names the two can be told apart; with `JsError` they cannot. The fifth is
`applyToRecord`, whose probe names the stage that refused.

**The cost of `js-sys` is one manifest line.** `cargo tree -i js-sys` shows it already built
for `serde-wasm-bindgen`. The `.wasm` with names is smaller: 336,697 bytes and 96,039 gzipped
against 338,194 and 96,296 with `JsError` (both `opt-level = 3`, `wasm-opt` off).

**What the declarations can say:** nothing of their own. A `@throws` line in a Rust doc
comment passes through as written (question 9 quotes it), and `tsc` checks none of it.

#### 3. Seeds and draws

```text
seed round trip 2^63+5                       true
seed MAX_SAFE+2                              9007199254740993n
RandomStream.seeded(2^63+5).index type       bigint
draws by seed 0..9 tier One                  104 104 104 104 104 104 104 104 104 104
draws by seed 0..9 tier Two                  104 104 104 104 104 104 104 104 104 104
draws by seed 0..9 tier Three                23 23 23 23 23 23 23 23 23 23
draws by seed 0..9 tier Four                 23 23 23 23 23 23 23 23 23 23
draws by seed 0..9 tier Five                 23 23 23 23 23 23 23 23 23 23
class path: start_index,taken,seed           3 23 null
class path: replay same givens               true
fromSeed equals stream at 0                  true
exhausted                                    the replay stream is exhausted at draw 5
index after exhausted                        5n
```

So `&mut` advances the JavaScript object's stream, which stays usable; `index` after minus
before equalled `draws_taken` in all fifty; and a stream advanced three draws, replayed from
the same seed by drawing `start_index` times and generating, gave the same givens, which is
`generation.allium`'s Replay at an index other than zero. A seed outside `u64` wraps silently
(`2n ** 64n` reads `0n`, `-1n` reads `18446744073709551615n`), as wasm-bindgen does for a
`u64`; the result carries the wrapped seed, so replay still holds. A `number` for a seed
throws `expected a bigint argument, found number`.

**A refused generation, and its `draws_taken`.** `GenerationResult` exposes `generation.draws_taken`
with no `when`, where its givens each carry `when generation.status = finished`. A caller holding a
stream reads the figure from `index`. A caller of generation from a seed cannot: the stream is
private and dropped on the refusal, and the thrown error's own properties were `stack`, `message` and
`name`. Nor does the text stand in for it, since `GridAttemptsSpent` counts attempts, and an attempt
takes 22 draws or, stopping short after k givens, 2k + 1 (`DrawsInOrder`). Three scripts through `RandomStream.replay`, each with the same message:

```text
100 short seedings             | GenerateError | no solution grid was found in 100 grid attempts | index 1900n
100 full seedings, no solution | GenerateError | no solution grid was found in 100 grid attempts | index 2200n
50 and 50                      | GenerateError | no solution grid was found in 100 grid attempts | index 2050n
```

No seed was found that refuses. Tier 3, seeds 0 to 99,999: no refusal, the first grid attempt failed
for 617 and the most draws taken was 67. So the seed path's refusal has not been seen, and at that
rate it is out of practical reach (an inference from one rate, not a proof). The clause is the
clause all the same, and it exposes `generation.tier` with no `when` either, so a second probe carried
both on the refusal: a stand-in for T41's `#[non_exhaustive]` `thiserror` enum, in a `no_std` crate of its
own so that `non_exhaustive` binds, built by the helper both exports share, and thrown with the two as own
properties of the `Error`:

```text
spent, 1900             | GenerateError | no solution grid was found in 100 grid attempts | tier Three | draws_taken 1900n bigint | index 1900n
spent, tier 1           | GenerateError | no solution grid was found in 100 grid attempts | tier One | draws_taken 1900n bigint | index 1900n
exhausted, 5 zeros      | GenerateError | the replay stream is exhausted at draw 5 | tier Five | draws_taken 5n bigint | index 5n
exhausted, from index 3 | GenerateError | the replay stream is exhausted at draw 5 | tier Two | draws_taken 2n bigint | index 5n
JSON.stringify(e), by Reflect.set, draws_taken alone  TypeError: Do not know how to serialize a BigInt
JSON.stringify(e), by Object.defineProperty           {"name":"GenerateError"}
```

`JsValue::from(u64)` crosses as a `bigint` with no serialiser. The text is the core's unchanged, and
the `.wasm` grew 1,159 bytes (336,697 to 337,856, `opt-level = 3`, `wasm-opt` off). The enum's crate is
clean under `cargo clippy -- -D warnings` with the workspace's lint tables, builds for `wasm32v1-none`, and
satisfies the bounds of invariant 3. Matched from another crate with no wildcard arm, it fails:
`` error[E0004]: non-exhaustive patterns: `&_` not covered ``.

A wildcard arm has no `tier` or `draws_taken` to give, so a throw through it could not meet the interface
declared for the refusal. So the enum answers through `tier()` and `draws_taken()`, `const fn`s that match inside
its own crate, and the binding reads them with no match at all: the four refusals above came back line for line
through that shape, at the same 337,856 bytes, and the crate stayed clean under clippy and on `wasm32v1-none`. A
later variant then fails to compile in the core until it answers both, which is where that question belongs.

#### 4. Casing, and what the serialiser has to be

Under either casing the interfaces are hand-written, so the declarations say whatever the
serde shape is: `is_given: boolean`, `after_move: number` and `draws_taken: bigint` under
snake_case, `isGiven`, `afterMove` and `drawsTaken` under `rename_all = "camelCase"`. The
mixture reads `board.writeMark(here, 4)` beside `cell.is_given`; a slip is a `tsc` error that
names the field (`Did you mean 'is_given'?`). The game's `eslint.config.js` at `add73be` has
no `camelcase` or `naming-convention` rule to object.

**`serde_wasm_bindgen`'s defaults break the "one wire shape"; this was not in the ticket.**
Measured from the probe:

```text
default            digit in obj:true digit:undefined small:number moves:number
default JSON       {"small":7,"moves":7}
default u64::MAX   18446744073709551615 can't be represented as a JavaScript number
null, numbers      digit:null small:number moves:number
null, bigints      digit:null small:bigint moves:bigint
flatten, default            [object Map]
flatten, maps as objects    [object Object] {"row":1,"column":2,"digit":null}
JSON.stringify(result)      TypeError: Do not know how to serialize a BigInt
```

Four facts follow. `Option::None` is `undefined` by default where `serde_json` writes `null`.
A `u64` above `Number.MAX_SAFE_INTEGER` **throws** by default: `2n ** 53n - 1n` crossed as a number, and `2n ** 53n` threw `9007199254740992 can't be represented as a JavaScript number`. The bigint setting also turns every `usize` into a
`bigint`, because serde writes a `usize` as a `u64`: that would make `undone`, `index` and
`after_move` bigints, and `JSON.stringify(board.write())` would throw. And `#[serde(flatten)]`
yields a JavaScript `Map`. So one serialiser cannot serve both the board's values and the
generator's.

With `serialize_missing_as_null` and `serialize_maps_as_objects` for the board's values, and
with `serialize_large_number_types_as_bigints` as well for the generation value alone,
`JSON.stringify` of a record parses back to a board that shows the same view (`true`), and a
seed of `2n ** 63n + 5n` arrives as `bigint`. Only `u64` fields are bigints, and only the
generation value holds them.

**A number parameter wraps, and that makes the two ways disagree; nor was this in the
ticket.** A `u8` or `usize` parameter of an export takes whatever JavaScript number it is
given:

```text
demoU8(265)    9        demoSerdeU8(265)   invalid value: integer `265`, expected u8
demoU8(-247)   9        demoSerdeU8(-1)    invalid type: floating point `-1.0`, expected u8
demoU8(9.9)    9        demoSerdeU8(9.9)   invalid type: floating point `9.9`, expected u8
demoUsize(1.5) 1        demoSerdeUsize(1.5) invalid type: floating point `1.5`, expected usize
demoUsize(4294967297) 1 demoSerdeUsize(4294967297) invalid value: integer `4294967297`, expected usize
```

The first build's `place(position, digit: u8)` **put a 4 in the cell when handed 260**, while
`applyToRecord` with the same digit refused. Read through serde, with `unchecked_param_type =
"number"` keeping the declaration as `digit: number`, both refuse with the same text:

```text
board.place(FREE,260)     invalid value: integer `260`, expected u8
applyToRecord digit 260   invalid value: integer `260`, expected u8
board.place digit 0       the digit 0 is outside 1 to 9
applyToRecord digit 0     the digit 0 is outside 1 to 9
```

Other findings for the operation value: `#[serde(tag = "kind")]` round-trips
(`{"kind":"Undo"}`, `{"kind":"Place","position":{"row":1,"column":3},"digit":4}`) and compiles
under `no_std` with `serde`'s `alloc` feature on `wasm32v1-none` (`Finished`), so it is open to
the core. An unknown kind says `` unknown variant `Fly`, expected one of `Place`, `Erase`,
`WriteMark`, `StrikeMark`, `CheckCell`, `Undo`, `Redo` ``. An extra key in a value is ignored,
a missing one refused.

#### 5. Where tests run

A Node module imported `pkg/pawdoku_wasm.js`, called `initSync({ module: bytes })` and exercised
every export: the runs quoted throughout this section are that. The glue's signature is
`export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;`. The
default `init()` in Node throws, as the ticket supposed:

```text
await init()  TypeError: fetch failed
init() cause: not implemented... yet...
```

The glue's line is `module_or_path = new URL('pawdoku_wasm_bg.wasm', import.meta.url);`. Once, by
hand and not as a gate, the same package was loaded in headless Chrome 156 from a local server;
`init()` succeeded there, `generateFromSeed('Three', 2n ** 63n)` gave a `bigint` seed, and a board
opened with 81 cells. So the default path was seen to work once, on this date.

What only a browser proves is that fetch path, and the path the game actually ships is the one
Vite rewrites, which no browser job in this repository would exercise. On conda-forge, `chromedriver` on 2026-11-06 and the others as stated: `chromedriver` has no package on either platform, `chromium` none on `linux-64` (2026-11-07), and
`python-chromedriver-binary` 159.0.8147 (11.86 MiB, `linux-64`) brings no browser; `geckodriver`
0.37.1 (1.79 MiB) is on both, and `firefox` 159.0.1 on both (104.67 MiB `linux-64`, 170.63 MiB
`osx-arm64`; 2026-11-07). A gate is therefore over 100 MiB per fresh environment, two more pins and a CI job;
its run time was not measured. The Node route costs: `tsc` over the consumer 1 s, the Node check
under 0.1 s, `typescript` 3.13 MiB.

#### 6. Size and speed

The probe's `.wasm`, `lto = "fat"`, `codegen-units = 1`, with named errors. The optimised rows are
`wasm-opt` 121 with `--all-features`:

| `opt-level` | `wasm-opt` | Bytes | gzip -9 | brotli -q 11 |
| --- | --- | --- | --- | --- |
| 3 | off | 336,697 | 96,039 | 75,503 |
| 3 | `-O3` | 230,141 | 83,814 | 66,644 |
| 3 | `-Oz` | 227,931 | 83,952 | 66,573 |
| `s` | off | 310,585 | 85,602 | 69,927 |
| `s` | `-O3` | 191,608 | 71,991 | 60,104 |
| `s` | `-Oz` | 189,806 | 72,170 | 60,148 |
| `z` | off | 328,520 | 84,038 | 68,968 |
| `z` | `-O3` | 169,558 | 67,137 | 56,788 |
| `z` | `-Oz` | 166,124 | 67,623 | 57,219 |

Through wasm-pack itself, `wasm-opt = ['-O3']` with no extra flag wrote 229,788 bytes, 83,657 gzip,
66,383 brotli, so binaryen 121 reads this toolchain's output without a feature flag. The glue is
28,225 bytes, 6,102 gzipped. **A trap:** with the metadata on and no `wasm-opt` on `PATH`, `--mode
no-install` does not fail:

```text
[INFO]: Skipping wasm-opt as no downloading was requested
[INFO]: ✨   Done in 0.44s
```

Time per call, median of 101 unless said, in Node:

| Build | Generate, tier 1 / tier 3, ten seeds | `applyToRecord`, fixture, 200 moves | `Board.reopen`, slowest puzzle, 200 moves |
| --- | --- | --- | --- |
| 3, off | 0.85 / 1.24 ms | 0.83 ms | 1.28 ms |
| 3, `-O3` | 0.69 / 0.97 ms | 0.80 ms | 1.36 ms |
| 3, `-Oz` | 0.73 / 1.22 ms | 0.83 ms | 1.37 ms |
| `s`, `-O3` | 0.83 / 1.35 ms | 1.51 ms | 1.95 ms |
| `z`, `-O3` | 1.22 / 1.87 ms | 1.64 ms | 2.63 ms |
| `z`, `-Oz` | 0.98 / 1.38 ms | 1.40 ms | 1.87 ms |

Differences of a tenth of a millisecond between the `opt-level = 3` rows are noise. The `s` and `z`
rows are 1.4 to 2 times slower on replay. The stateless cost itself, on `opt-level = 3`, `-O3`:

| Record | Moves | `Board.open` | `Board.reopen` | `applyToRecord` (one stateless call), median and p95 | Held: two moves, a view, two undos |
| --- | --- | --- | --- | --- | --- |
| Fixture, no guess | 0 | 0.032 ms | 0.036 ms | 0.077, 0.117 ms | 0.026 ms |
| | 50 | | 0.197 ms | 0.259, 0.320 ms | 0.043 ms |
| | 200 | | 0.682 ms | 0.771, 0.851 ms | 0.090 ms |
| `NEEDS_A_GUESS`, 50 guesses | 0 | 0.914 ms | 0.925 ms | 0.938, 1.006 ms | 0.020 ms |
| | 50 | | 1.007 ms | 1.070, 1.181 ms | 0.038 ms |
| | 200 | | 1.321 ms | 1.470, 1.674 ms | 0.090 ms |
| Tier 5, seed 0, 3 guesses | 0 | 0.097 ms | 0.102 ms | 0.133, 0.144 ms | 0.024 ms |
| | 200 | | 0.668 ms | 0.798, 0.842 ms | 0.090 ms |

That table is one run of the build, so its figures differ from the row above by the noise between runs.
`NEEDS_A_GUESS` is the 17-given puzzle of `crates/pawdoku/tests/solver.rs` and the slowest measured. Tier 5
seeds 0 to 9 took 3, 0, 0, 0, 0, 0, 3, 1, 0 and 0 guesses, so a generated puzzle costs about what the fixture
does. A cold start: `initSync` 2.03 ms, the first `Board.open` 1.64 ms, the first `applyToRecord`
2.10 ms, the first generation 1.46 ms. A record of 200 moves is 10,765 bytes of JSON.

#### 7. Lifetime

Quoted from the 0.2.130 glue, built with no flag:

```text
const BoardFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_board_free(ptr, 1));
if (Symbol.dispose) Board.prototype[Symbol.dispose] = Board.prototype.free;
```

The `.d.ts` declares `[Symbol.dispose](): void;`
on each class. `using held = Board.open(givens)` type-checks under the flags above and frees the board
when its block ends: read after it, the board throws. Under `--target es2020 --lib es2020,dom` the
declarations themselves fail, `error TS2318: Cannot find global type 'Disposable'.`, so a consumer's
`lib` needs `esnext` or `esnext.disposable`, and with no `dom` the file fails on `Response`, `URL` and
`WebAssembly`.

```text
use after free                     Attempt to use a moved value
free twice                         null pointer passed to rust
memory: start, after 20000 open+free   1245184 1245184
memory: 2000 held, then freed          2621440 2621440
approx bytes per open board        688
bytes per 200-move board           5767
never freed, 100000 opened: memory MB  68.3
never freed, 200000 opened: memory MB  135.4
never freed, 300000 opened: memory MB  202.5
```

Freed memory is reused and the module's memory never shrinks. The last three lines ran in one
synchronous loop, in which a `FinalizationRegistry` callback has no chance to run, so they show what
a board costs unfreed and not that the registry fails. A registry promises no timing in any case.

#### 8. The readings after a move

Carrying the 81 cells inside every move, against reading one on asking, on the fixture at 200 moves:

| | JSON bytes | Time |
| --- | --- | --- |
| The view, with no readings | 24,099 | 0.087 ms |
| One reading | 3,502 | 0.019 ms held; 0.660 ms by `readRecordAfter` |
| The 200 readings, if carried | 700,193 | 3.758 ms for the view and all of them, held |
| The 200 readings, one call each by record | | 137.7 ms |

The probe's reading, read from a game: `{"row":1,"column":4,"digit":null,"note":[4]}`. A given reads
`{"row":1,"column":1,"digit":5,"note":[]}`, an undone move is still readable, and index 4 of 3 moves
is refused. A move's index counts from 1 (`crates/pawdoku/src/board/moves.rs`).
`Move::note_after` gives the note "whether it was shown then or waited beneath a digit", and
`Playing` exposes `move.note_after(cell)` with no `when`, where a cell's own note is exposed only
`when cell.shows_note`. So a reading's note is an array in every cell, and a view's is absent while
a digit stands: both are as `board.allium` states them.

#### 9. What the declarations say

Quoted from the probe's `pawdoku_wasm.d.ts`, the `Board` class in part, one stateless function, one
plain value:

```ts
/** A cell as `Playing` exposes it. */
export interface BoardCell {
    row: number;
    column: number;
    digit: number | null;
    is_given: boolean;
    is_conflicting: boolean;
    /** Its note's digits while the note is shown. */
    note?: number[];
}

export class Board {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The 81 cells as they stand.
     */
    cells(): BoardCell[];
    /**
     * Checks the player's digit against the solution.
     */
    check(position: Position): BoardCheck;
    /**
     * Opens a board on `givens`.
     *
     * @throws {Error} `SolveError`, its message the solver's refusal, when the givens
     * do not have exactly one solution.
     * @param givens - The puzzle's givens.
     */
    static open(givens: Given[]): Board;
}

/**
 * Puts one operation to a record: it reopens the record, makes the operation and writes
 * the board down again. Nothing is held between calls.
 *
 * @throws {Error} When the record does not reopen or the operation is refused.
 * @param record - The record, unchanged.
 * @param operation - One of the seven.
 * @returns The record after it and the view of the board after it.
 */
export function applyToRecord(record: BoardRecord, operation: Operation): Played;
```

No exported signature says `any`. The same file's `InitOutput`, wasm-bindgen's description of the raw
exports, does, on 15 lines (`readonly applyToRecord: (a: any, b: any) => [number, number, number];`),
and T42's Verification grep matches them; every match lies at or after line 227, where `InitOutput`
begins, and none before. Under the workspace's lints, `cargo clippy --target wasm32-unknown-unknown
-- -D warnings` over the probe gives 22 of `# Errors` missing on exports returning `Result`, 2 of
`could be a const fn` and 1 of `more than 3 bools in a struct`, the view.

**The ticket's Verification block**, run before the final commit:

```text
$ git status --porcelain
 M tickets/README.md
 M tickets/S12-typescript-surface.md
$ rg -n '^### ' tickets/S12-typescript-surface.md
255:### What was verified, and how
824:### The recommendations
868:### Amendments to T41 and T42
1012:### Handed back
1051:### Deviations, and why
1058:### Adversarial review
1072:### Open points settled
```

### The recommendations

1. **Typing: (a), wasm-bindgen's own attributes, with no new dependency.** The interfaces are held by
   the three links above, within the limit stated there: the snapshots hold what the live values show,
   and a declaration module holds members and optionality, which the snapshots cannot. tsify is not recommended, because the values are the core's and it cannot
   see them. Nothing goes to T02 for it. The record's TypeScript name is **`BoardRecord`, confirmed**:
   `TS2315` is what `Record` does. Every other value keeps the core's Rust name; `BoardMove` and `BoardCheck` above are the probe's stand-ins.
2. **Errors: `js_sys::Error` named for the Rust type**, message always `error.to_string()`. The game can
   tell a refusal from a value of the wrong shape, and which refusal, without reading a message; the cost
   is one manifest line for a crate already built. Declarations say it only in `@throws`. **T17 mirrors it
   with one exception class per error type**, under one base class, each carrying the text.
3. **Seeds and draws: the writer's list is confirmed, with `seed` null on the class path.** Tier, seed,
   start index, `draws_taken`, both versions and the givens, never the solution. A seedless path exists
   because a `SeededStream` holds its state and not its seed. No `status` field: a refusal is thrown,
   which is what `SearchResult` already does with its status, and a result exists only once finished.
   **A refusal carries `tier` and `draws_taken`**, on both paths, because `GenerationResult` exposes both with no
   `when`, and generation from a seed has no stream left to read. That is a core change for T41, and no
   change to the specification. **`Draw` records are needed by nothing built today.**
4. **Casing: snake_case fields and camelCase methods, confirmed**, with one addition the writer did not
   state: enum values stay PascalCase, serde's default, because `Record`'s moves and `Tier` already are
   and a change would move the record's snapshot for no consumer. **And two serialiser settings**,
   question 4's, which are what make one wire shape true.
5. **Tests: Node here, the browser on the game's side.** `initSync` in Node tests the published package
   with no browser. The one thing a browser adds is the fetch path that Vite rewrites in the game, which a
   job here would not exercise; the G ticket's built preview does. **No browser gate is recommended.**
6. **Size and speed: `wasm-opt` on at `-O3`, `opt-level` left at the default.** It saves 12.9% gzipped
   (96,039 to 83,657) and 12.1% with brotli for no measured loss, and `-Oz` is within 150 bytes of it once compressed.
   `opt-level = "s"` or `"z"` would save 12 to 17 KB more gzipped at 1.4 to 2 times the replay, and because
   `[profile.release]` is shared with the crates T16 and T17 will add, it would need a profile of its own, which was
   not tried; not worth it. **The stateless cost is acceptable for a call
   per move**: at most 1.7 ms at p95 at 200 moves on the slowest puzzle measured, on this machine, and well
   under a frame's 16 ms. It is not for a call per frame, and a phone was not measured. T42 pins
   `binaryen = "==121"` and guards it, because of the silent skip.
7. **Lifetime: hold one `Board` for the life of a puzzle in play and call `free()` once when it is left.**
   The registry is a safety net with no timing and `using` suits a short scope such as a test or a worker
   call. Neither may be combined with a second `free()`, which throws.
8. **Readings: a separate `reading_after(move_index)`, confirmed**, in both ways: `readingAfter(index)` on
   the class and a stateless function taking a record and an index. Each gives the 81 cells in row order,
   flat, with `digit` and `note`. No batch: a caller that scrubs through a game holds a board, 3.8 ms against
   137.7 ms for all 200. **It is a spec change for T40 on the stateless side**, below.
9. **Declarations: good enough to use with no hand-written wrapper.** Every parameter and return is named,
   doc comments and `@param`, `@returns`, `@throws` come through, and misuse is a `tsc` error. A consumer's
   `lib` needs `esnext` and `dom`.

### Amendments to T41 and T42

**T41, "What S12 decides first"** (Context). All three are settled: casing as recommendation 4, readings as 8,
the generation result as 3.

**T41, step 3, the wire shape:**

- Field names are the specifications', snake_case, and enum values serde's default PascalCase: `"Unsolved"`,
  `"OneSolution"`, `"WriteMark"`. `Record`'s shape does not change for casing, so the Non-goal stands.
- An absent digit is `null`, so `Option<u8>` carries no `skip_serializing_if`: `digit: Option<u8>` on a cell, a
  move and a reading. A cell's `note` alone is absent while not shown, with `skip_serializing_if =
  "Option::is_none"`; an empty note that is shown is `[]`.
- **No `#[serde(flatten)]` in any wire value.** It gives a `Map` under `serde-wasm-bindgen`'s defaults, and it
  would change how every binding has to read it. The flat cell is written with its own `row` and `column`.
- The reading is `{ move_index, cells: [{ row, column, digit, note }] }`, 81 cells in row order. `note` is always
  an array and includes a note waiting beneath a digit, as `Move::note_after` gives it. `move_index` counts from 1,
  and an undone move still has a reading; 0 and anything past the last are the refusal.
- `#[expect(clippy::struct_excessive_bools, reason = "...")]` on the view, which holds what `Playing` exposes: the
  lint fires on its four bools.

**T41, step 4, the operation and the error:**

- The operation is `#[serde(tag = "kind")]` with variants named as `Playing`'s, `Place`, `Erase`, `WriteMark`,
  `StrikeMark`, `CheckCell`, `Undo`, `Redo`, and fields `position` and `digit`. So on the wire `{ "kind": "Undo" }`
  and `{ "kind": "Place", "position": { ... }, "digit": 4 }`, which gives TypeScript a discriminated union.
- What a call hands back is `{ record, view, check }` with `check` absent unless the operation was `CheckCell`.
- The two wrapping variants of the error are `#[error(transparent)]`, so a refusal's text is the same whichever way
  the board is driven. Step 4 leaves that open, and T42 step 7's comparison of the two ways reads it.

**T41, step 6, the generation result.** The fields and wire names are `tier`, `seed: Option<u64>`, `start_index: u64`,
`draws_taken: u64`, `generation_version`, `random_version`, `givens`, in that order. `seed` is `None` from a stream, because
neither stream knows one. `draws_taken` is `index` after `wrapping_sub` `index` before, as the index wraps. There is no
`status`: `Err` is the refusal.

That `Err` is a new public `#[non_exhaustive]` `thiserror` enum, on both paths, so a refusal has one shape, as invariant 3 requires of an
error: `GenerationRefused`, with the one variant `Refused { tier: Tier, draws_taken: u64, error: GenerateError }`. The variant's `Display` is
`#[error("{error}")]`, since `#[error(transparent)]` takes one field, so the text stays the core's. Both paths are built on one function that
takes a tier and a stream and reads the index on `Err` as on `Ok`. `generate` is unchanged. The enum has two public `#[must_use] const fn`
accessors, `tier(&self) -> Tier` and `draws_taken(&self) -> u64`, each matching inside the crate, so that no binding has to match a
`#[non_exhaustive]` enum to read them and a later variant cannot be added without answering both.
No seed is known to reach the refusal (question 3), so it is tested through that function with a script, and the seed path's preservation is
held by sharing it, not by a test of its own. Step 7 gains: a hundred short seedings give `draws_taken` 1900; a hundred full seedings with no
solution, 2200, with the same text; five zeros, 5; a stream drawn three times first counts only its own, 2 of a script of five; and each of the
five tiers comes back as the tier asked for. `tests/api_bounds.rs` takes the new type under step 9.

**T41, step 8, the snapshots**, because T42's live comparison and `satisfies` see only what they show, and neither of those
catches a widened declaration, which is T42 step 8's declaration module. The view's snapshot must show all four move
kinds, an undone move, a cell with a note, a cell without one, an absent digit and a check. Add a snapshot of the JSON of all
seven operations, and of a generation result from a stream (where `seed` is `null`) beside the one from a seed. Show a search
result of each verdict.

**T41, Open points.** "Where the readings after a move live on the wire": settled as above, with T40 stating the stateless
form first.

**T42, Context, "What S12 decided"**: (a) wasm-bindgen's attributes; named errors, needing `js-sys`; `BoardRecord`; the package
test and the `tsc` check, both yes; `wasm-opt` on at `-O3`; a board freed once when its puzzle is left.

**T42, step 2 and Files touched.** `js-sys = "0.3.107"` is needed, re-checked on the day. `typescript = "==6.0.2"` and
`binaryen = "==121"` are both added to `pyproject.toml`. `nodejs` is whatever T15 pinned, which should be 26.13.0.

**T42, step 3, errors and types:**

- `src/error.rs` builds a `js_sys::Error`, `set_name` to the Rust type's name, for `PlayError`, `ReopenError`, `SolveError`,
  `GenerateError` and `RandomError`. T41's error takes the name of the stage that refused: `ReopenError` from its reopening
  variant and `PlayError` from its playing one, with its own type's name for the index that names no move and for any variant
  added later. So a game branches the same way on either way of driving the board.
- `src/types.rs` also exports `type EngineErrorName`, the union of those names, so a branch on `e.name` is checked.
- Every refusal of T41's `GenerationRefused` is thrown one way, with no `match` and so no wildcard arm: `Display` text as the message,
  `name` always `GenerateError`, and `tier` and `draws_taken` from T41's accessors as own properties of the `Error`, `tier` a string
  through the board's serialiser and `draws_taken` a `bigint` by `JsValue::from(u64)`. Both are set with `Object.defineProperty` and so
  are not enumerable: set with `Reflect.set`, `JSON.stringify(e)` throws on the `bigint`. No other name is thrown for it and
  `EngineErrorName` gains none. `src/types.rs` exports `interface GenerationRefused extends Error { name: "GenerateError"; tier: Tier;
  draws_taken: bigint }`, which then holds of every such throw, and the `@throws` lines of both generation exports name it. `tsc` checks
  neither, as question 2 says of every throw.
- **Two serialisers**, in one helper each. Board values and the search use `Serializer::new().serialize_missing_as_null(true)
  .serialize_maps_as_objects(true)`. The generation result adds `.serialize_large_number_types_as_bigints(true)`, and nothing
  else may, because it turns a `usize` into a `bigint` and `JSON.stringify(board.write())` would then throw. The default `to_value`
  is used nowhere.
- A TypeScript name is the core's Rust name, except `Record`, which is `BoardRecord`. Its interface is written out in full and not
  left opaque, so that the guard of step 8 has something to hold; `version: string`, not a literal, because a record of another
  version is the case the refusal exists for.
- Interfaces for `u64` fields say `bigint`, and for `Option` say `| null`. A cell's `note` is `note?: number[]`.

**T42, step 4, the class.** `digit` and `index` parameters are `JsValue`, read through `serde_wasm_bindgen::from_value` to `u8`
and `usize`, with `unchecked_param_type = "number"`, and **never** a Rust `u8` or `usize` parameter, which wraps: `place(here, 260)`
put a 4 on the board in the probe. Each export returning `Result` needs its `# Errors` section, for clippy.

**T42, step 5.** The operation's type is the discriminated union on `kind`. T41's reading after a move crosses as an export taking
a record and an index, with the index read through serde as in step 4.

**T42, step 6.** `generate` and generation from a seed use the bigint serialiser, and both throw T41's refusal as step 3 says. The
README states that a seed reduces modulo 2^64 silently.

**T42, step 7, added tests:**

- `place(here, 260)`, `writeMark(here, 260)`, `readingAfter(1.5)` and `readingAfter(2 ** 32 + 1)` each throw and leave the board as
  it was, and play from a record throws the same text for the same digit.
- A refusal's `name`: each error type's, `ReopenError` and `PlayError` from play from a record, and `Error` for a wrong shape.
- A record through `JSON.stringify` and `JSON.parse` reopens and shows the same view, which fails if a `bigint` reaches it.
- A generation result's `seed`, `start_index` and `draws_taken` are `bigint`, `seed` null from a stream, and its `index`
  replay holds: a stream advanced by `start_index` draws from the same seed gives the same givens.
- `typeof` for a view's `index`, `after_move` and the record's `undone` is `number`.
- A refused `generate`: `RandomStream.replay` of five zeros throws with `draws_taken` `5n` and `index` `5n`; of a hundred short
  seedings (the script of `generation/grid.rs`'s `a_hundred_failed_attempts_are_a_refusal`), with `name` `GenerateError`, the core's
  text, `draws_taken` `1900n` and `index` `1900n`; a stream drawn three times first gives `draws_taken` `2n` and `index` `5n`. Each
  has `tier` equal to the tier asked for, run under more than one tier. `JSON.stringify` of each refusal does not throw. No test throws from generation from a seed, because no seed is known to refuse;
  T41 tests the function the two share.

A comparison of generation results "as JavaScript values" needs a replacer, because `JSON.stringify` throws on a `bigint`.

**T42, step 8, the package test, recommended.** Its contents, in addition to the Node module and the strict consumer:

- `crates/pawdoku-wasm/tests/package/` also holds a script that strips each wire-shape snapshot T41 accepted of its insta header and
  writes a scratch `.ts` of `export const value = <JSON> satisfies <Interface>;`, one per snapshot, with `Wire<T>` (question 1) for the
  generation result; and the Node module checks each snapshot's live value against that JSON, by `JSON.stringify` for the board's values
  and through a replacer for the generation's. Files touched gains the script.
- `crates/pawdoku-wasm/tests/package/declarations.ts`, in Files touched, is question 1's third link, because the snapshots hold no
  widening: a table keyed by every exported union (`Operation["kind"]`, `Tier`, `Status`, `MoveKind`, `Verdict`) with one value per
  member, an `OptionalKeys` equality for every exported value naming exactly its optional fields, each field read back into its
  narrow type, and `@ts-expect-error` lines for a missing required field and an unknown member of each union that a game passes in.
  The Node module puts the table of operations and of tiers to the engine and fails on a `name` of `Error`, serde's refusal of the
  shape; an engine refusal such as `PlayError` passes, since the shape was read. Plant a widening of each kind, as question 1's table
  did, see `tsc` fail on each, and quote them.
- `tsc` runs with `--target es2022 --lib esnext,dom --module esnext --moduleResolution bundler`, in the `tsconfig.json`.
- `[package.metadata.wasm-pack.profile.release] wasm-opt = ['-O3']`, replacing T15's `false`. No feature flag is needed.
- **`wasm-build`'s guard gains `wasm-opt --version`**, which must say `wasm-opt version 121`, because wasm-pack in `--mode no-install`
  skips the step and succeeds when the tool is absent. It stands beside T15's `wasm-bindgen --version` guard.
- Quote the `.wasm` before and after; this probe's were 336,697 and 229,788 bytes, T41's values will differ.

**T42, step 9, the README** says also: a generation result holds `bigint`s, so `JSON.stringify` throws on it, and a game stores
`String(seed)` and reads back with `BigInt(...)`; a refused generation has `e.tier` and `e.draws_taken` on either path; `using` needs `esnext` in `lib`; `free()` once, a second throws
`null pointer passed to rust`; a thrown `name` of `Error` means a value of the wrong shape, whose text is serde's and not stable.

**T42, Verification.** `rg -n ': any\b|=> any\b' crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts` matches `InitOutput` always (15 lines in the
probe), so it cannot be expected empty. Replace it with `sed '/export interface InitOutput/,$d' crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts
| rg -n ': any\b|=> any\b'`, which the probe leaves empty.

**T42, Acceptance criteria.** "Every refusal throws with the core's `Display` text" stands for the engine's refusals. Add: a value of the
wrong shape throws too, with `name` `Error` and serde's text, and leaves everything unchanged.

**T42, the G ticket, Part B.** Keeping a game: wrap `Board.reopen(JSON.parse(stored))` as one `try`, because a record that does not reopen
can throw `ReopenError` or `Error`. A new puzzle: the seed is kept as `String(seed)`, as question 4 shows. The adapter calls `free()` exactly
once when the puzzle is left. The vitest and preview proofs stand, and the preview is what proves the fetch path Vite rewrites.

### Handed back

- **To T40, which must carry this in `board.allium`.** Question 8's shape adds to the contract as T40 drafts it. Regrouping the readings on a held
  board adds nothing, because `Playing` exposes them already; the stateless side is what is new. The whole of it:
  1. **What a call hands back excludes the readings.** T40 step 3 says a call gives "what `Playing` exposes", which read literally includes
     `move.digit_after` and `move.note_after` for every move and cell: 700,193 bytes at 200 moves. The signatures, and the Scope's Includes bullet,
     say that what comes back is what `Playing` exposes of the board, its status, cells, moves and checks, without the readings after its moves,
     and that those are given on asking. The third signature, reading a record, says the same.
  2. **A fourth signature.** It takes a record and a move's index and gives, for every cell of the board, `digit_after` and `note_after` as that
     move holds them. The index is `move.index`, counted from 1, and a move undone is still one of the record's moves. `note_after` is a note shown
     or waiting beneath a digit, as `Playing` already exposes it with no `when`, where a cell's own note carries `when cell.shows_note`.
  3. **A third refusal.** T40's refusal invariant names two; this adds an index that names no move, and nothing is handed back then either. A
     record that does not reopen is refused first, since there is no board to ask until it has.
  4. **The surface's guarantee and the other invariants reach it.** "The same readings" in the surface `PlayingFromARecord` is the fourth signature
     against `Move`'s reading on a held board. "What is shown" and "nothing kept between calls" cover it as they stand, and `TheSolutionIsNeverShown`
     holds, since a reading holds no solution digit a player's board did not hold.
  5. **T40's own text.** "Its three signatures and four invariants" under Acceptance criteria, and "with three signatures" in step 3, become four
     signatures, with a deviation recorded; decision 0021's Decision says the readings are given on asking. T41 step 5 already builds this fourth
     function, and without the clause it would be behaviour no specification states.

  Nothing else in `Playing` changes, and no `open question` is touched. **The generation refusal the rounds added is not T40's:** it is T41's core
  change to what `GenerationResult` already exposes, and T40's Non-goals exclude `generation.allium`. Nor is anything here about the record's version,
  which is T43's.
- **To T15.** `nodejs` moved to 26.13.0 on conda-forge (both platforms; T15 Context's table and step 6 say 26.12.1), to be pinned and noted under
  this run's rule. The `wasm-opt = false` metadata stays until T42 changes it.
- **To T00 follow-ups, carried by T42.** `typescript = "==6.0.2"`, `binaryen = "==121"`, the `wasm-package-test` recipe and `wasm-build`'s guard.
- **To T02.** Nothing new. `serde-wasm-bindgen` and `js-sys` are already T42's step 2, each with its decision-0007 note, and tsify is not added.
- **To T17.** One exception class per error type under one base, with the `Display` text, and `tier` and `draws_taken` attributes of the
  generation refusal's. The same core values in snake_case. A `dict` has no
  bigint problem, since Python's `int` is exact, but T17 should still read a `u8` through a checked conversion rather than a wrapping one, and a test
  should hold that a digit of 260 is refused the same both ways.
- **To T41, a core change the specification already asks for.** `generation.allium`'s `GenerationResult` exposes `tier`, `status` and `draws_taken` with no `when`.
  A thrown refusal gives the status, the stream's `index` gives the draws only to a caller who holds the stream, and the tier is only what the caller remembers
  of its own call. T41 step 6 above closes that on both paths. Context says a need here is a spec change and a core change, handed back; this one needs the core change alone.
- **To the specifications, beyond T40: a wording, and no behaviour.** The comment above `GenerationResult` says "A refused generation shows nothing but that it
  was refused", which reads tighter than the `exposes` beneath it. T41 follows `exposes`, which is the clause, and takes the comment to mean no givens and no grid.
  This spike changes no specification, so the comment stays until whoever next edits that module, or the maintainer, says which is meant; if it is the comment,
  T41's new value goes and `tier` and `draws_taken` leave the surface's refused case. No `open question` is touched.

### Deviations, and why

- **One branch.** This ticket ran on `implement-typescript-surface-fullsend` as the run's brief set, not on `ticket/s12-typescript-surface`, and
  no worktree was created. Step 1's condition is satisfied by order on this branch.
- **`nodejs` 26.13.0**, not 26.12.1, as the version table says.
- **Measured beyond the ticket:** `wasm-opt` at `-Os` and `-O2` were also run and said nothing further, so they are left out of the table.

### Adversarial review

**R1, round 1**, model `gpt-6-astra`, over `3ce08de..ee0dc08`, focus evidence against claim. The round's verdict was needs-attention, with two findings, both of medium severity and both valid. Each was answered in `7a97793`.

1. **Snapshot assignability does not establish declaration equality. Valid.** The notes said drift shows both ways and that showing every member closed the gap. Seven planted widenings, among them an optional `digit` on `Place` and an added `Fly` operation, left the snapshot guard clean with every operation shown. Question 1 now says the guard is one-way, T42 step 8 gains a declaration module that stopped all seven, and the notes state what that module still does not hold.
2. **The seed-only path cannot expose refusal draw counts. Valid.** `GenerationResult` exposes `draws_taken` with no `when`, the thrown error held only `stack`, `message` and `name`, and the same text came with 1900, 2200 and 2050 draws. No seed was found that refuses: 100,000 in tier 3, with 617 failed first attempts. Recommendation 3 and T41 step 6 now carry the figure on the refusal on both paths, with T42 steps 3, 6, 7 and 9 and T17 following, probed. The comment above `GenerationResult` reads tighter than its `exposes`, and that wording is handed back above; the finding's "no failing seed" stands as it said.

**R1, round 2**, model `gpt-6-astra`, over `3ce08de..f01204d`, same focus. The round's verdict was needs-attention, with two findings, both of medium severity and both valid. Each was answered in `d175e9d`. The rounds stop here at the limit of two.

1. **Preserve the tier on refused generation results. Valid.** `GenerationResult` exposes `generation.tier` with no `when`, as it does `draws_taken`, and the refusal as amended left it out. The caller named the tier in its call, but the clause does not rest on that, and round 1's argument was the clause. Recommendation 3, T41 step 6, T42 steps 3, 7 and 9, T17 and the hand-backs now carry it, probed through the stream path under four tier settings. The finding's "both generation paths" stands in the contract; a test of the seed path's refusal does not, because no seed is known to refuse, so the two share one function and T41 tests that.
2. **Specify the new public error as a thiserror enum. Valid.** Invariant 3 says errors are `thiserror` enums, every error in `crates/pawdoku/src` is one, and the amendment's value was a struct. T41 step 6 now names `GenerationRefused`, a `#[non_exhaustive]` enum whose one variant `Refused` holds the fields and the `Display` attribute. The probe moved to it, in a crate of its own so that `non_exhaustive` binds: clippy clean under the workspace's lints, built for `wasm32v1-none`, and the wildcard arm T42 step 3 now states is required.

**After the rounds.** The last triage reported one gap: T42 step 3's wildcard arm threw `name` `GenerationRefused` with no `tier` or `draws_taken`, while its interface declared `name` `GenerateError` and both properties. That wildcard is gone: T41's enum answers through two accessors and T42 step 3 throws one shape, probed in question 3. Round 2's sentence above records the state of that round.

### Open points settled

- **A new dependency for typing.** None: wasm-bindgen's own attributes, question 1. T42 does not wait on T02 for this.
- **A browser gate.** None here: Node and `tsc`, question 5, with the game's preview carrying the browser.

## Open points

- **A new dependency for typing.** If question 1 recommends tsify, its decision-0007
  note goes to T02 and the maintainer weighs it; T42 then waits for that dependency.
- **A browser gate.** If question 5 recommends one, it brings a webdriver pin and a CI
  job; the maintainer decides whether the cost is worth it here or the game's tests
  carry it.
