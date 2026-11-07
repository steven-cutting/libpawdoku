//! WebAssembly bindings for [`pawdoku`], the classic-sudoku engine, packed by wasm-pack
//! as the npm package `@steven-cutting/pawdoku-wasm` (decision 0020).
//!
//! This first version exports the engine's randomness boundary alone: the
//! [`RandomStream`] class, which gives the engine's own seeded generator or replays a
//! script of draws, and [`random_version`], the name of that generator. A seed and the
//! name are what replay a puzzle in a browser, in Python and on the command line alike.
//! The package also ships the engine's specification modules under `specs/`, so a
//! consumer can hold the clauses it restates equal to this repository's text.
//!
//! Errors cross as a JavaScript `Error` whose `message` is the engine's `Display` text.
//! The examples below are JavaScript: a Rust doctest runs on the host, where a
//! wasm-bindgen import panics, so the crate's behaviour is proved in Node by
//! `tests/web.rs` instead.
//!
//! ```js
//! import init, { RandomStream, randomVersion } from "@steven-cutting/pawdoku-wasm";
//!
//! await init();
//! const stream = RandomStream.seeded(2024n);
//! const draw = stream.nextDraw(); // a number in [0, 1)
//! console.log(randomVersion(), stream.index); // "seeded-stream-1" 1n
//! ```

use pawdoku::random::{RandomStream as CoreStream, ReplayStream, SeededStream};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

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

    /// The index the next draw will carry, read without a mutable borrow.
    fn index(&self) -> u64 {
        match self {
            Self::Seeded(stream) => stream.index(),
            Self::Replay(stream) => stream.index(),
        }
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
    /// Throws an `Error` whose message is `RandomError::OutOfRange`'s text for the
    /// first value outside `[0, 1)`, `NaN` included.
    ///
    /// ```js
    /// const stream = RandomStream.replay(new Float64Array([0.25, 0.5]));
    /// stream.nextDraw(); // 0.25
    /// RandomStream.replay([0.5, 1]);
    /// // throws Error: draw 1 of the replay script is 1, outside [0, 1)
    /// ```
    pub fn replay(draws: Vec<f64>) -> Result<Self, JsError> {
        ReplayStream::new(draws)
            .map(|stream| Self {
                stream: Stream::Replay(stream),
            })
            .map_err(|error| JsError::new(&error.to_string()))
    }

    /// The next draw, a number in `[0, 1)`; the index moves on by one.
    ///
    /// # Errors
    ///
    /// Throws an `Error` whose message is `RandomError::Exhausted`'s text when a replay
    /// script has no draw left; the index stays where it was. A seeded stream never
    /// throws.
    ///
    /// ```js
    /// const stream = RandomStream.replay([0.25]);
    /// stream.nextDraw(); // 0.25
    /// stream.nextDraw();
    /// // throws Error: the replay stream is exhausted at draw 1
    /// ```
    #[wasm_bindgen(js_name = nextDraw)]
    pub fn next_draw(&mut self) -> Result<f64, JsError> {
        self.stream
            .as_dyn()
            .next_draw()
            .map_err(|error| JsError::new(&error.to_string()))
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
