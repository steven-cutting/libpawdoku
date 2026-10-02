//! The rules from outside the crate. Until the solver can make a proof, what an outside
//! crate can reach of `sudoku` is its two figures and its two value types.

#![expect(
    clippy::tests_outside_test_module,
    reason = "an integration test file is its own test module"
)]

extern crate alloc;

use alloc::collections::BTreeSet;
use pawdoku::sudoku::{BOX_SIDE, Given, Position, SIDE};

#[test]
fn the_side_is_the_square_of_the_box_side() {
    assert_eq!(BOX_SIDE, 3);
    assert_eq!(SIDE, BOX_SIDE * BOX_SIDE);
    assert_eq!(usize::from(SIDE) * usize::from(SIDE), 81);
}

#[test]
fn value_types_compare_by_their_fields() {
    let here = Position::new(4, 7);
    assert_eq!(here, Position::new(4, 7));
    assert_ne!(here, Position::new(7, 4));
    assert_eq!(Given::new(here, 2), Given::new(here, 2));
    assert_ne!(Given::new(here, 2), Given::new(here, 3));
    assert_ne!(Given::new(here, 2), Given::new(Position::new(7, 4), 2));
}

/// Refusing a given that is off the grid or out of range is a rule's business, not a
/// property of the type, so both can be made and read back.
#[test]
fn a_position_off_the_grid_and_a_given_out_of_range_are_representable() {
    let nowhere = Position::new(0, SIDE + 1);
    let given = Given::new(nowhere, u8::MAX);
    assert_eq!((nowhere.row(), nowhere.column()), (0, 10));
    assert_eq!((given.position(), given.digit()), (nowhere, u8::MAX));
}

#[test]
fn two_givens_that_agree_are_one_given() {
    let here = Position::new(1, 1);
    let givens = BTreeSet::from([Given::new(here, 5), Given::new(here, 5)]);
    assert_eq!(givens.len(), 1);
    let disagreeing = BTreeSet::from([Given::new(here, 5), Given::new(here, 6)]);
    assert_eq!(disagreeing.len(), 2);
}
