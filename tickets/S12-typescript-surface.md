---
id: S12
title: "Spike: the engine's TypeScript surface, typed plain values, errors, tests, size and lifetime"
status: open
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

### The recommendations

### Amendments to T41 and T42

### Handed back

### Open points settled

## Open points

- **A new dependency for typing.** If question 1 recommends tsify, its decision-0007
  note goes to T02 and the maintainer weighs it; T42 then waits for that dependency.
- **A browser gate.** If question 5 recommends one, it brings a webdriver pin and a CI
  job; the maintainer decides whether the cost is worth it here or the game's tests
  carry it.
