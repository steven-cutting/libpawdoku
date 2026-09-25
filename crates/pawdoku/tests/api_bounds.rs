//! Every public type crosses a binding boundary one day, so each is
//! `Send + Sync + 'static`, `Clone` and `Debug` (`AGENTS.md`, invariant 3), and the
//! boundary trait is usable as a trait object.

#![expect(
    clippy::tests_outside_test_module,
    reason = "an integration test file is its own test module"
)]

use pawdoku::random::{RandomError, RandomStream, ReplayStream, SeededStream};

const fn assert_send_sync<T: Send + Sync + 'static>() {}
const fn assert_clone_debug<T: Clone + core::fmt::Debug>() {}
fn assert_dyn_compatible(_: &mut dyn RandomStream) {}

#[test]
fn public_types_are_send_sync_and_static() {
    assert_send_sync::<SeededStream>();
    assert_send_sync::<ReplayStream>();
    assert_send_sync::<RandomError>();
}

#[test]
fn public_types_are_clone_and_debug() {
    assert_clone_debug::<SeededStream>();
    assert_clone_debug::<ReplayStream>();
    assert_clone_debug::<RandomError>();
}

#[test]
fn the_boundary_is_a_trait_object() {
    let mut stream = SeededStream::new(1);
    assert_dyn_compatible(&mut stream);
}

#[cfg(feature = "serde")]
#[test]
fn streams_serialise_under_the_serde_feature() {
    fn assert_serde<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    assert_serde::<SeededStream>();
    assert_serde::<ReplayStream>();
}
