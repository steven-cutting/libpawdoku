//! The randomness boundary from outside the crate: determinism, the golden draws that
//! pin `RANDOM_VERSION`, the replay fake, and the range every draw keeps.

#![expect(
    clippy::tests_outside_test_module,
    reason = "an integration test file is its own test module"
)]

use core::fmt::Write;
use pawdoku::random::{RandomError, RandomStream, ReplayStream, SeededStream};
use proptest::prelude::*;

// A picture for review and change detection; the assertions below own correctness.
#[test]
fn snapshot_the_randomness_boundary() {
    let mut picture = String::from("SeededStream(seed = 0)\n");
    let mut stream = SeededStream::new(0);
    for _ in 0..8 {
        let index = stream.index();
        let draw = stream.next_draw().unwrap();
        writeln!(picture, "draw {index}: {draw:.17}").unwrap();
    }
    writeln!(picture, "\nRandomError (Display)").unwrap();
    for error in [
        RandomError::Exhausted { index: 8 },
        RandomError::OutOfRange {
            index: 2,
            value: 1.0,
        },
    ] {
        writeln!(picture, "{error}").unwrap();
    }
    insta::assert_snapshot!(picture);
}

#[test]
fn the_same_seed_draws_the_same_stream() {
    let mut a = SeededStream::new(2024);
    let mut b = SeededStream::new(2024);
    assert_eq!(a.index(), 0);
    for _ in 0..1000 {
        assert_eq!(
            a.next_draw().unwrap().to_bits(),
            b.next_draw().unwrap().to_bits()
        );
    }
    assert_eq!(a.index(), 1000);
    assert_eq!(b.index(), 1000);
}

/// The first eight draws of seed zero, from the `SplitMix64` reference algorithm.
///
/// `ExactReplay` promises the same draws for the same seed and `random_version`. A
/// changed generator changes these bits, and whoever changes them must change
/// `RANDOM_VERSION` in the same commit.
#[test]
fn seed_zero_draws_the_golden_stream() {
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
    assert_eq!(pawdoku::random::RANDOM_VERSION, "seeded-stream-1");
    let mut stream = SeededStream::new(0);
    for expected in GOLDEN {
        assert_eq!(stream.next_draw().unwrap().to_bits(), expected);
    }
}

fn replay_three(stream: &mut dyn RandomStream) {
    for (index, expected) in (0_u64..).zip([0.25_f64, 0.5, 0.75]) {
        assert_eq!(stream.index(), index);
        assert_eq!(stream.next_draw().map(f64::to_bits), Ok(expected.to_bits()));
    }
    assert_eq!(stream.next_draw(), Err(RandomError::Exhausted { index: 3 }));
    assert_eq!(stream.index(), 3);
}

#[test]
fn the_fake_replays_its_script_then_is_exhausted() {
    let mut stream = ReplayStream::new(vec![0.25, 0.5, 0.75]).unwrap();
    replay_three(&mut stream);
}

#[test]
fn the_fake_behaves_the_same_behind_a_trait_object() {
    let mut boxed: Box<dyn RandomStream> =
        Box::new(ReplayStream::new(vec![0.25, 0.5, 0.75]).unwrap());
    replay_three(boxed.as_mut());
}

#[test]
fn every_draw_is_in_the_unit_interval() {
    for seed in [0, 1, u64::MAX] {
        let mut stream = SeededStream::new(seed);
        for _ in 0..10_000 {
            assert!((0.0..1.0).contains(&stream.next_draw().unwrap()));
        }
    }
}

proptest! {
    #[test]
    fn fresh_streams_agree_bit_for_bit(seed in any::<u64>(), n in 1usize..256) {
        let mut a = SeededStream::new(seed);
        let mut b = SeededStream::new(seed);
        for _ in 0..n {
            let (u, v) = (a.next_draw().unwrap(), b.next_draw().unwrap());
            prop_assert_eq!(u.to_bits(), v.to_bits());
            prop_assert!((0.0..1.0).contains(&u));
        }
    }
}
