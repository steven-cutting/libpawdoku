//! WebAssembly bindings for [`pawdoku`], the classic-sudoku engine, packed by wasm-pack
//! as the npm package `@steven-cutting/pawdoku-wasm` (decision 0020).
//!
//! The package gives a game the engine two ways, which hand back the same plain values:
//!
//! - **A held board.** The [`Board`] class opens on givens or reopens from a record, is
//!   held for as long as the puzzle is in play, makes the seven operations, shows its
//!   view and writes its record. Free it once when the puzzle is left.
//! - **Play from a record.** [`open`], [`play`], [`read`] and [`reading_after`] are each
//!   handed a record or givens and give back a record and a view; nothing is held, and
//!   each call on a record reopens it.
//!
//! Beside them: [`search`], the solver's setter's tool; [`generate`] from a
//! [`RandomStream`] and [`generate_from_seed`]; and the three versions a kept game or a
//! seed needs, [`record_version`], [`generation_version`] and [`random_version`]. The
//! package also ships the engine's specification modules under `specs/`, so a consumer
//! can hold the clauses it restates equal to this repository's text.
//!
//! A plain value is the core's wire shape, with a TypeScript interface of the same name
//! (the record's is `BoardRecord`). A refusal throws a JavaScript `Error` whose `name`
//! is the engine's error type and whose `message` is the engine's `Display` text; a
//! value of the wrong shape throws an `Error` named `Error`. The examples below are
//! JavaScript: a Rust doctest runs on the host, where a wasm-bindgen import panics, so
//! the crate's behaviour is proved in Node by its tests instead.
//!
//! ```js
//! import init, { Board, play, generateFromSeed } from "@steven-cutting/pawdoku-wasm";
//!
//! await init();
//! const { givens, seed } = generateFromSeed("Three", 2024n);
//!
//! // Held: one board for the puzzle's life.
//! const board = Board.open(givens);
//! board.place({ row: 1, column: 3 }, 4);
//! const kept = JSON.stringify(board.write());
//! board.free();
//!
//! // From a record: values in, values out.
//! const { record, view } = play(JSON.parse(kept), { kind: "Undo" });
//! ```

mod board;
mod error;
mod from_record;
mod generation;
mod solver;
mod types;

pub use board::Board;
pub use from_record::{open, play, read, reading_after, record_version};
pub use generation::{generate, generate_from_seed, generation_version};
pub use solver::search;

use pawdoku::random::{RandomStream as CoreStream, ReplayStream, SeededStream};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::error::thrown;

/// The stream a [`RandomStream`] holds: one of the two the engine ships, so the class is
/// `Send`, `Sync`, `Clone` and `Debug`, which a boxed trait object could not be. Closed
/// on purpose: a third source of draws is a change to the engine first.
#[derive(Clone, Debug)]
enum Stream {
    Seeded(SeededStream),
    Replay(ReplayStream),
}

impl Stream {
    /// The stream as the engine's trait, which every engine entry point that draws takes.
    fn as_dyn(&mut self) -> &mut dyn CoreStream {
        match self {
            Self::Seeded(stream) => stream,
            Self::Replay(stream) => stream,
        }
    }

    /// The stream as the engine's trait, borrowed shared, for what reads and draws nothing.
    fn as_dyn_ref(&self) -> &dyn CoreStream {
        match self {
            Self::Seeded(stream) => stream,
            Self::Replay(stream) => stream,
        }
    }

    /// The index the next draw will carry, read without a mutable borrow.
    fn index(&self) -> u64 {
        self.as_dyn_ref().index()
    }
}

/// A stream of draws in `[0, 1)`, the engine's one source of randomness.
///
/// [`seeded`](Self::seeded) gives the engine's own generator, the one
/// [`random_version`] names, so the same seed gives the same draws in every binding;
/// [`replay`](Self::replay) gives a script, the fake a test supplies its draws through.
/// Every draw carries an index, from zero.
///
/// ```js
/// const stream = RandomStream.seeded(0n);
/// stream.index;      // 0n
/// stream.nextDraw(); // 0.8833108082136426
/// stream.index;      // 1n
/// ```
#[wasm_bindgen]
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct RandomStream {
    stream: Stream,
}

const _: () = {
    const fn assert_bounds<T: Send + Sync + 'static + Clone + core::fmt::Debug>() {}
    assert_bounds::<RandomStream>();
};

impl RandomStream {
    /// The stream as the engine's trait, for [`generate`] to draw from in place.
    pub(crate) fn as_dyn(&mut self) -> &mut dyn CoreStream {
        self.stream.as_dyn()
    }
}

#[wasm_bindgen]
impl RandomStream {
    /// The engine's own generator, started from `seed` at index zero. In JavaScript the
    /// seed is a `bigint`, every one from `0n` to `2n ** 64n - 1n`.
    ///
    /// ```js
    /// const stream = RandomStream.seeded(2024n);
    /// ```
    #[must_use]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "wasm-bindgen exports cannot be const"
    )]
    pub fn seeded(seed: u64) -> Self {
        Self {
            stream: Stream::Seeded(SeededStream::new(seed)),
        }
    }

    /// A stream that replays `draws` in order from index zero, then throws when asked
    /// for one more. In JavaScript the script is a `Float64Array`, or an array of
    /// numbers.
    ///
    /// # Errors
    ///
    /// Throws a `RandomError` whose message is `RandomError::OutOfRange`'s text for the
    /// first value outside `[0, 1)`, `NaN` included.
    ///
    /// ```js
    /// const stream = RandomStream.replay(new Float64Array([0.25, 0.5]));
    /// stream.nextDraw(); // 0.25
    /// RandomStream.replay([0.5, 1]);
    /// // throws RandomError: draw 1 of the replay script is 1, outside [0, 1)
    /// ```
    pub fn replay(draws: Vec<f64>) -> Result<Self, JsValue> {
        ReplayStream::new(draws)
            .map(|stream| Self {
                stream: Stream::Replay(stream),
            })
            .map_err(|refusal| thrown(&refusal))
    }

    /// The next draw, a number in `[0, 1)`; the index moves on by one.
    ///
    /// # Errors
    ///
    /// Throws a `RandomError` whose message is `RandomError::Exhausted`'s text when a replay
    /// script has no draw left; the index stays where it was. A seeded stream never
    /// throws.
    ///
    /// ```js
    /// const stream = RandomStream.replay([0.25]);
    /// stream.nextDraw(); // 0.25
    /// stream.nextDraw();
    /// // throws RandomError: the replay stream is exhausted at draw 1
    /// ```
    #[wasm_bindgen(js_name = nextDraw)]
    pub fn next_draw(&mut self) -> Result<f64, JsValue> {
        self.stream
            .as_dyn()
            .next_draw()
            .map_err(|refusal| thrown(&refusal))
    }

    /// The index the next draw will carry, which is how many draws were taken; a
    /// `bigint` in JavaScript, read-only.
    ///
    /// ```js
    /// const stream = RandomStream.seeded(0n);
    /// stream.nextDraw();
    /// stream.index; // 1n
    /// ```
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn index(&self) -> u64 {
        self.stream.index()
    }
}

/// The name of the generator [`RandomStream::seeded`] uses, `"seeded-stream-1"`, the
/// engine's `RANDOM_VERSION`. Keep it beside a seed: a seed replays only under the
/// generator it was drawn from. A function, not a constant, because wasm-bindgen
/// exports no constant.
///
/// ```js
/// randomVersion(); // "seeded-stream-1"
/// ```
#[wasm_bindgen(js_name = randomVersion)]
#[must_use]
pub fn random_version() -> String {
    pawdoku::random::RANDOM_VERSION.into()
}

#[wasm_bindgen]
extern "C" {
    /// JavaScript's `console.error`.
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// Runs when the module is instantiated: a panic, which the engine never means to raise
/// (decision 0008), is written to `console.error` with its message and location before
/// the module traps, rather than reaching JavaScript as a bare
/// `RuntimeError: unreachable`.
#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|panic| console_error(&panic.to_string())));
}
