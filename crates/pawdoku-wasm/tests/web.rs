//! The randomness boundary as JavaScript receives it, run in Node by
//! `wasm-bindgen-test-runner` (`just wasm-test`). On the host this file compiles to
//! nothing: a wasm-bindgen import panics off wasm32, so the crate's behaviour is
//! proved here and its examples are `js` fences rustdoc does not run.

#![cfg(target_arch = "wasm32")]

use pawdoku::random::RandomError;
use pawdoku_wasm::{RandomStream, random_version};
use wasm_bindgen::{JsCast, JsError, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen]
extern "C" {
    /// JavaScript's `Error`, read for its message, the one thing a caller sees.
    type Error;
    #[wasm_bindgen(method, getter)]
    fn message(this: &Error) -> String;
}

/// The message a thrown error carries, as `error.message` reads it in JavaScript.
fn message_of(error: JsError) -> String {
    JsValue::from(error).unchecked_into::<Error>().message()
}

/// The first eight draws of seed zero, `crates/pawdoku/tests/random.rs`'s golden bits.
const GOLDEN: [u64; 8] = [
    0x3FEC_4415_072F_63B9,
    0x3FDB_9E27_9AA8_6E58,
    0x3F9B_1174_6200_2500,
    0x3FEF_1177_150E_4990,
    0x3FBB_3989_6A51_A870,
    0x3FD4_F2E7_C31D_1FA8,
    0x3FC6_414D_5F0F_A298,
    0x3FE8_B082_6759_22D5,
];

#[wasm_bindgen_test]
fn seed_zero_draws_the_golden_stream() {
    let mut stream = RandomStream::seeded(0);
    for expected in GOLDEN {
        let draw = stream.next_draw().map_err(message_of);
        assert_eq!(draw.map(f64::to_bits), Ok(expected));
    }
}

#[wasm_bindgen_test]
fn the_index_starts_at_zero_and_advances_by_one_per_draw() {
    let mut stream = RandomStream::seeded(2024);
    for expected in 0..16 {
        assert_eq!(stream.index(), expected);
        assert!(stream.next_draw().is_ok());
    }
    assert_eq!(stream.index(), 16);
}

#[wasm_bindgen_test]
fn a_replay_gives_its_script_in_order_then_throws_the_exhausted_text() {
    let mut stream = RandomStream::replay(vec![0.25, 0.5, 0.75]).map_err(message_of);
    assert_eq!(stream.as_ref().map(RandomStream::index), Ok(0));
    let Ok(stream) = stream.as_mut() else { return };
    for (index, expected) in (0_u64..).zip([0.25_f64, 0.5, 0.75]) {
        assert_eq!(stream.index(), index);
        let draw = stream.next_draw().map_err(message_of);
        assert_eq!(draw.map(f64::to_bits), Ok(expected.to_bits()));
    }
    let exhausted = stream.next_draw().map_err(message_of);
    assert_eq!(
        exhausted,
        Err(RandomError::Exhausted { index: 3 }.to_string())
    );
    assert_eq!(stream.index(), 3);
}

#[wasm_bindgen_test]
fn a_replay_outside_the_unit_interval_throws_the_out_of_range_text() {
    for (script, index, value) in [
        (vec![0.5, 1.0], 1, 1.0),
        (vec![-0.25], 0, -0.25),
        (vec![0.0, 0.5, f64::NAN], 2, f64::NAN),
    ] {
        let refused = RandomStream::replay(script).map(|_| ()).map_err(message_of);
        assert_eq!(
            refused,
            Err(RandomError::OutOfRange { index, value }.to_string())
        );
    }
}

#[wasm_bindgen_test]
fn a_clone_draws_on_from_where_it_was_taken() {
    let mut stream = RandomStream::seeded(7);
    assert!(stream.next_draw().is_ok());
    let mut copy = stream.clone();
    for _ in 0..4 {
        let (a, b) = (
            stream.next_draw().map_err(message_of),
            copy.next_draw().map_err(message_of),
        );
        assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
    }
    assert_eq!(copy.index(), 5);
}

#[wasm_bindgen_test]
fn the_random_version_is_the_engines() {
    assert_eq!(random_version(), pawdoku::random::RANDOM_VERSION);
    assert_eq!(random_version(), "seeded-stream-1");
}
