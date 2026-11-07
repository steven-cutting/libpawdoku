# @steven-cutting/pawdoku-wasm

WebAssembly bindings for `pawdoku`, the classic-sudoku engine behind Pawdoku, a Biscuit
Games game. The Rust crate `pawdoku-wasm` is built with wasm-bindgen and packed by
wasm-pack for the web target into the npm package `@steven-cutting/pawdoku-wasm`, an ES
module, its `.wasm`, and TypeScript declarations.

It exports the engine: a `Board` class to hold a puzzle in play, the stateless
functions of play from a record, the solver's `search`, the basic generator from a seed
or from a `RandomStream`, and the three versions a kept game needs. The package also
ships the engine's specification modules, so a consumer can hold the rules it restates
equal to the engine's own text.

## Install

The package is published to GitHub Packages, the registry for the `@steven-cutting`
scope. Point the scope at it in the project's `.npmrc`, with a token that can read
packages:

```ini
@steven-cutting:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

Then install it:

```sh
npm install @steven-cutting/pawdoku-wasm
```

## Load the module

In the browser, call `init()` once before anything else. It fetches the `.wasm` beside
the module through `new URL(..., import.meta.url)`, which a bundler such as Vite rewrites
at build time, so no WebAssembly plugin is needed. Call it in the browser only, never
while a page is prerendered.

```js
import init from "@steven-cutting/pawdoku-wasm";

await init();
```

In Node, where `init()` cannot fetch, as in a test, hand `initSync` the bytes:

```js
import { readFileSync } from "node:fs";
import { initSync } from "@steven-cutting/pawdoku-wasm";

const wasm = import.meta.resolve("@steven-cutting/pawdoku-wasm/pawdoku_wasm_bg.wasm");
initSync({ module: readFileSync(new URL(wasm)) });
```

## Play: a held board

Hold one `Board` for as long as a puzzle is in play. It is opened from givens or
reopened from a record, makes the seven operations, shows its view and writes its
record. One search is made when it opens, and none after.

```js
import { Board } from "@steven-cutting/pawdoku-wasm";

const board = Board.open(givens); // [{ position: { row: 1, column: 1 }, digit: 5 }, ...]
const here = { row: 1, column: 3 };
board.writeMark(here, 2);
board.place(here, 4);
const { is_right } = board.check(here); // whether the digit is right, never the digit
board.undo();
const view = board.view();              // status, flags, 81 cells, moves, checks
const { cells } = board.readingAfter(1); // the board as the first move left it

localStorage.setItem("game", JSON.stringify(board.write()));
board.free();
```

The operations are `place`, `erase`, `writeMark`, `strikeMark`, `check`, `undo` and
`redo`. Each returns nothing but `check`, which returns the check it added; call `view()`
to redraw.

**Free a board once, when its puzzle is left.** A board's memory is the WebAssembly
module's, which JavaScript's garbage collector does not see. Call `free()` exactly once;
a board used after it, or freed twice, throws (`null pointer passed to rust`). For a
short scope such as a test or a worker call, `using board = Board.open(givens)` frees it
when the block ends; that needs `esnext` in the TypeScript `lib`.

## Play: from a record

For a caller that holds no board, such as a web worker or a resumed game drawn before it
is reopened, four functions take a record or givens and give back plain values. Nothing
is held between calls, and each call on a record reopens it, which costs a search and a
replay of every move: fine for a call per move, not for a call per frame.

```js
import { open, play, read, readingAfter } from "@steven-cutting/pawdoku-wasm";

let { record, view } = open(givens);
({ record, view } = play(record, { kind: "Place", position: { row: 1, column: 3 }, digit: 4 }));
const { check } = play(record, { kind: "CheckCell", position: { row: 1, column: 3 } });
const again = read(record);                // the view alone
const { cells } = readingAfter(record, 1); // the board as the first move left it
```

An operation is one of `Place`, `Erase`, `WriteMark`, `StrikeMark`, `CheckCell`, `Undo`
and `Redo`, by its `kind`. What comes back has the record after the call and the view,
and the check it added after a `CheckCell`. A value read from a held board equals the
one play from a record gives.

## Keeping a game

A record is plain JSON with no `bigint`: keep it with `JSON.stringify` and reopen it with
`Board.reopen(JSON.parse(stored))`, in one `try`. A record names its format version,
`recordVersion()`, and a record of another version is refused with a message naming
both; no promise is made that a later package reads a record an earlier one wrote.

## Search and generation

```js
import { RandomStream, generate, generateFromSeed, generationVersion, randomVersion, search }
  from "@steven-cutting/pawdoku-wasm";

const { verdict, guesses, solutions } = search(givens); // a setter's tool

const puzzle = generateFromSeed("Three", 2024n); // a tier from "One" to "Five"; a bigint seed
puzzle.givens;                                   // the puzzle; never its solution
localStorage.setItem("seed", String(puzzle.seed));
const seed = BigInt(localStorage.getItem("seed"));

const stream = RandomStream.seeded(2024n);
const next = generate("Three", stream);          // advances the stream by next.draws_taken
```

A seed is a `bigint` from `0n` to `2n ** 64n - 1n`; one outside that range is reduced
modulo `2n ** 64n` silently, and the result carries the seed as reduced. A generation
result holds `bigint`s (`seed`, `start_index`, `draws_taken`), so `JSON.stringify`
throws on it: keep the seed as `String(seed)` and read it back with `BigInt(...)`.
The same tier and seed give the same puzzle under the same `generationVersion()` and
`randomVersion()`; keep both with the seed.

## Errors

A refusal throws a JavaScript `Error` whose `message` is the engine's own text and
whose `name` is the engine's error type: `PlayError`, `ReopenError`, `SolveError`,
`GenerateError`, `RandomError` or `FromRecordError`, the union `EngineErrorName`. A
refused call changes nothing.

```js
try {
  board.place({ row: 1, column: 1 }, 4);
} catch (error) {
  error.name;    // "PlayError"
  error.message; // "the cell at row 1, column 1 holds a given"
}
```

Play from a record throws the name of the stage that refused, `ReopenError` or
`PlayError`, as a held board does. A refused generation, on either path, is a
`GenerateError` with `error.tier` and `error.draws_taken` (a `bigint`); a stream keeps
the draws it gave before the refusal. A thrown `name` of `Error` means a value of the
wrong shape, such as a digit of `260` or a missing field; its text is serde's and is
not stable. A position handed to a board's method is read as `play` reads one, from
its own fields: a getter on its prototype is not seen, and a function in it, `toJSON`
among them, is refused with an `Error`. A board's method and `generate` refuse an
object for a digit, an index or a tier with an `Error`, unread, and a getter or a
`Proxy` trap on a position that throws comes back as its own exception; the board, or
the stream, is unchanged and still usable.

## The specifications

The engine's behaviour is specified in Allium modules before it is built. The package
ships every module under `specs/`, and its `exports` map makes each one reachable by
name, so a test can read the text the engine was built from:

```js
const url = import.meta.resolve("@steven-cutting/pawdoku-wasm/specs/sudoku.allium");
```

## Licence

Apache-2.0. The licence text ships in the package as `LICENSE`, and the package's
changes are in its `CHANGELOG.md`. The source is at
<https://github.com/steven-cutting/libpawdoku>, under `crates/pawdoku-wasm`.
