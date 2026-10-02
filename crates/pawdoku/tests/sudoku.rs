//! The rules from outside the crate: the two figures, the value types, and a puzzle
//! made the way outside code makes one, by solving and then setting, and played through
//! the `PuzzleSolving` surface to its end.

#![expect(
    clippy::tests_outside_test_module,
    reason = "an integration test file is its own test module"
)]

extern crate alloc;

use alloc::collections::BTreeSet;
use pawdoku::solver::{SolveError, solve};
use pawdoku::sudoku::{
    BOX_SIDE, Cell, Given, MoveError, Position, Puzzle, SIDE, Status, WellPosed,
};

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

/// The rules' fixture, rows top to bottom, a dot for an empty cell: thirty givens.
const FIXTURE: &str =
    "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

/// A cell the fixture leaves to the player; the solution holds 4 there, and the given 5
/// at [`GIVEN`] is in its row and its box.
const FREE: Position = Position::new(1, 3);

/// A cell the fixture gives: a 5.
const GIVEN: Position = Position::new(1, 1);

/// The fixture's givens, in grid order.
fn fixture_givens() -> Vec<Given> {
    let cells = FIXTURE.bytes().zip(0_u8..);
    cells
        .filter(|(cell, _)| cell.is_ascii_digit())
        .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
        .collect()
}

/// The fixture's proof, made the one way outside code can make it.
fn fixture() -> Result<WellPosed, SolveError> {
    solve(fixture_givens())
}

/// The digit a full grid holds at `position`.
fn digit_at(solution: &[[u8; 9]; 9], position: Position) -> u8 {
    solution[usize::from(position.row() - 1)][usize::from(position.column() - 1)]
}

#[test]
fn a_puzzle_set_from_outside_is_played_and_solved() {
    let proof = fixture().unwrap();
    let solution = *proof.solution();
    let mut puzzle = Puzzle::set(proof.clone());
    assert_eq!(puzzle.status(), Status::Unsolved);
    assert_eq!(puzzle.givens(), proof.givens());

    let free: Vec<Position> = puzzle
        .cells()
        .filter(|cell| !cell.is_given())
        .map(|cell| Position::new(cell.row(), cell.column()))
        .collect();
    assert_eq!(free.len(), 51);
    for (placed, &position) in free.iter().enumerate() {
        assert_eq!(
            puzzle.status(),
            Status::Unsolved,
            "after {placed} placements"
        );
        assert!(!puzzle.is_full());
        assert_eq!(
            puzzle.place(position, digit_at(&solution, position)),
            Ok(())
        );
        assert!(puzzle.is_consistent());
    }
    // The placement that completes the grid solves it, with no further call.
    assert_eq!(puzzle.status(), Status::Solved);
    assert!(puzzle.is_full());
    let digits: Vec<Option<u8>> = puzzle.cells().map(Cell::digit).collect();
    let expected: Vec<Option<u8>> = solution.iter().flatten().copied().map(Some).collect();
    assert_eq!(digits, expected);

    // Solved is final: both moves are refused and nothing changes.
    assert_eq!(puzzle.place(FREE, 4), Err(MoveError::AlreadySolved));
    assert_eq!(puzzle.erase(FREE), Err(MoveError::AlreadySolved));
    assert_eq!(puzzle.status(), Status::Solved);
}

#[test]
fn the_moves_are_offered_and_refused_from_outside() {
    let mut puzzle = Puzzle::set(fixture().unwrap());
    assert_eq!(puzzle.place(FREE, 5), Ok(()));
    assert_eq!(puzzle.place(FREE, 4), Ok(()));
    assert_eq!(puzzle.erase(FREE), Ok(()));
    assert_eq!(
        puzzle.erase(FREE),
        Err(MoveError::EmptyCell { position: FREE })
    );
    assert_eq!(
        puzzle.place(GIVEN, 4),
        Err(MoveError::GivenCell { position: GIVEN })
    );
    assert_eq!(
        puzzle.erase(GIVEN),
        Err(MoveError::GivenCell { position: GIVEN })
    );
    assert_eq!(
        puzzle.place(FREE, 10),
        Err(MoveError::DigitOutOfRange { digit: 10 })
    );
    let nowhere = Position::new(0, 10);
    assert_eq!(
        puzzle.place(nowhere, 4),
        Err(MoveError::NoSuchCell { position: nowhere })
    );
    assert!(puzzle.cell(nowhere).is_none());
}

#[test]
fn the_surface_can_be_read_from_outside() {
    let mut puzzle = Puzzle::set(fixture().unwrap());
    puzzle.place(FREE, 5).unwrap();
    assert_eq!(puzzle.status(), Status::Unsolved);
    assert!(!puzzle.is_full());
    assert!(!puzzle.is_consistent());
    let read = |position| {
        let cell = puzzle.cell(position).unwrap();
        (
            (cell.row(), cell.column(), cell.band(), cell.stack()),
            (cell.digit(), cell.is_given(), cell.is_conflicting()),
        )
    };
    assert_eq!(read(FREE), ((1, 3, 1, 1), (Some(5), false, true)));
    assert_eq!(read(GIVEN), ((1, 1, 1, 1), (Some(5), true, true)));
    assert_eq!(
        read(Position::new(9, 9)),
        ((9, 9, 3, 3), (Some(9), true, false))
    );
    assert_eq!(
        read(Position::new(5, 5)),
        ((5, 5, 2, 2), (None, false, false))
    );
    assert_eq!(puzzle.cells().count(), 81);
    assert_eq!(puzzle.cells().filter(|cell| cell.is_given()).count(), 30);
}

/// A puzzle hands out nothing of its solution: its `Debug` leaves it out, though the
/// proof it was set from exposes it.
#[test]
fn a_puzzle_set_from_outside_does_not_print_its_solution() {
    let proof = fixture().unwrap();
    assert_eq!(proof.solution()[0], [5, 3, 4, 6, 7, 8, 9, 1, 2]);
    let printed = format!("{:?}", Puzzle::set(proof));
    assert!(printed.starts_with("Puzzle"));
    assert!(!printed.contains("534678912"));
}
