# @steven-cutting/pawdoku-wasm

WebAssembly bindings for `pawdoku`, the classic-sudoku engine behind Pawdoku, a Biscuit
Games game. The Rust crate `pawdoku-wasm` is built with wasm-bindgen and packed by
wasm-pack for the web target into the npm package `@steven-cutting/pawdoku-wasm`, an ES
module, its `.wasm`, and TypeScript declarations.

This first version exports the engine's randomness boundary: the `RandomStream` class,
which gives the engine's own seeded generator or replays a script of draws, and
`randomVersion()`, the name of that generator. A seed and the name replay the same draws
here, in Python and on the command line. The package also ships the engine's
specification modules, so a consumer can hold the rules it restates equal to the
engine's own text.

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

## Use

Call `init()` once before anything else. It fetches the `.wasm` beside the module
through `new URL(..., import.meta.url)`, which a bundler such as Vite rewrites at build
time, so no WebAssembly plugin is needed.

```js
import init, { RandomStream, randomVersion } from "@steven-cutting/pawdoku-wasm";

await init();

const stream = RandomStream.seeded(2024n); // the seed is a bigint
const draw = stream.nextDraw();            // a number in [0, 1)
stream.index;                              // 1n, the index the next draw carries
randomVersion();                           // "seeded-stream-1"

const script = RandomStream.replay(new Float64Array([0.25, 0.5]));
```

A refused call throws an `Error` whose `message` is the engine's own text: a script
value outside `[0, 1)`, or a draw past the end of a script.

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
