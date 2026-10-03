//! Every public type crosses a binding boundary one day, so each is
//! `Send + Sync + 'static`, `Clone` and `Debug` (`AGENTS.md`, invariant 3), and the
//! boundary trait is usable as a trait object.

#![expect(
    clippy::tests_outside_test_module,
    reason = "an integration test file is its own test module"
)]

use pawdoku::board::{
    Board, BoardCell, Check, Move, MoveKind, Note, PlayError, Record, ReopenError,
};
use pawdoku::generation::{GenerateError, Tier};
use pawdoku::random::{RandomError, RandomStream, ReplayStream, SeededStream};
use pawdoku::solver::{SearchResult, SolveError, Verdict};
use pawdoku::sudoku::{
    Cell, Given, Grid, MoveError, Position, Puzzle, Status, WellPosed, WellPosedError,
};

const fn assert_send_sync<T: Send + Sync + 'static>() {}
const fn assert_clone_debug<T: Clone + core::fmt::Debug>() {}
fn assert_dyn_compatible(_: &mut dyn RandomStream) {}

#[test]
fn public_types_are_send_sync_and_static() {
    assert_send_sync::<SeededStream>();
    assert_send_sync::<ReplayStream>();
    assert_send_sync::<RandomError>();
    assert_send_sync::<Position>();
    assert_send_sync::<Given>();
    assert_send_sync::<Grid<u8>>();
    assert_send_sync::<WellPosed>();
    assert_send_sync::<WellPosedError>();
    assert_send_sync::<Puzzle>();
    assert_send_sync::<Status>();
    assert_send_sync::<Cell>();
    assert_send_sync::<MoveError>();
    assert_send_sync::<SearchResult>();
    assert_send_sync::<Verdict>();
    assert_send_sync::<SolveError>();
    assert_send_sync::<Board>();
    assert_send_sync::<BoardCell>();
    assert_send_sync::<Note>();
    assert_send_sync::<Move>();
    assert_send_sync::<MoveKind>();
    assert_send_sync::<Check>();
    assert_send_sync::<PlayError>();
    assert_send_sync::<Record>();
    assert_send_sync::<ReopenError>();
    assert_send_sync::<Tier>();
    assert_send_sync::<GenerateError>();
}

#[test]
fn public_types_are_clone_and_debug() {
    assert_clone_debug::<SeededStream>();
    assert_clone_debug::<ReplayStream>();
    assert_clone_debug::<RandomError>();
    assert_clone_debug::<Position>();
    assert_clone_debug::<Given>();
    assert_clone_debug::<Grid<u8>>();
    assert_clone_debug::<WellPosed>();
    assert_clone_debug::<WellPosedError>();
    assert_clone_debug::<Puzzle>();
    assert_clone_debug::<Status>();
    assert_clone_debug::<Cell>();
    assert_clone_debug::<MoveError>();
    assert_clone_debug::<SearchResult>();
    assert_clone_debug::<Verdict>();
    assert_clone_debug::<SolveError>();
    assert_clone_debug::<Board>();
    assert_clone_debug::<BoardCell>();
    assert_clone_debug::<Note>();
    assert_clone_debug::<Move>();
    assert_clone_debug::<MoveKind>();
    assert_clone_debug::<Check>();
    assert_clone_debug::<PlayError>();
    assert_clone_debug::<Record>();
    assert_clone_debug::<ReopenError>();
    assert_clone_debug::<Tier>();
    assert_clone_debug::<GenerateError>();
}

#[test]
fn the_boundary_is_a_trait_object() {
    let mut stream = SeededStream::new(1);
    assert_dyn_compatible(&mut stream);
}

#[cfg(feature = "serde")]
#[test]
fn streams_and_value_types_serialise_under_the_serde_feature() {
    fn assert_serde<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    assert_serde::<SeededStream>();
    assert_serde::<ReplayStream>();
    assert_serde::<Position>();
    assert_serde::<Given>();
    assert_serde::<Record>();
    assert_serde::<Tier>();
}

/// A board and a puzzle are never serialised: a record is the one value that crosses.
/// The call below names an implementation by inference, which is ambiguous, and so
/// fails to compile, the day either type implements one of serde's two traits.
#[cfg(feature = "serde")]
#[test]
fn a_board_and_a_puzzle_do_not_serialise() {
    trait Unless<Marker> {
        fn holds() {}
    }
    impl<T> Unless<()> for T {}
    impl<T: serde::Serialize> Unless<u8> for T {}
    impl<T: serde::de::DeserializeOwned> Unless<u16> for T {}

    <Board as Unless<_>>::holds();
    <Puzzle as Unless<_>>::holds();
}
