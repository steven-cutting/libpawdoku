//! The proof value: how well-posedness reaches the rules without the rules asking the
//! solver.

use super::{Given, Grid, LINE, Position, at, grid_positions, in_range};
use alloc::collections::BTreeSet;

/// The solution with every cell's digit read out, or the first cell, in grid order,
/// that is empty or holds something out of range.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
fn filled(solution: &Grid<Option<u8>>) -> Result<Grid<u8>, WellPosedError> {
    let mut digits = [[0; LINE]; LINE];
    for (position, cell) in grid_positions().zip(digits.iter_mut().flatten()) {
        let digit = at(solution, position)
            .flatten()
            .ok_or(WellPosedError::SolutionNotFull { position })?;
        if !in_range(digit) {
            return Err(WellPosedError::SolutionDigitOutOfRange { position, digit });
        }
        *cell = digit;
    }
    Ok(digits)
}

/// Refuses a solution in which two peers hold one digit, naming the first such pair in
/// grid order.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
fn consistent(solution: &Grid<u8>) -> Result<(), WellPosedError> {
    let holds = |position| at(solution, position);
    let conflict = grid_positions().find_map(|first| {
        let second = grid_positions()
            .find(|&second| first.is_peer_of(second) && holds(first) == holds(second))?;
        Some((first, second, holds(first)?))
    });
    match conflict {
        Some((first, second, digit)) => Err(WellPosedError::SolutionConflict {
            first,
            second,
            digit,
        }),
        None => Ok(()),
    }
}

/// Refuses a given that is off the grid or whose digit is not the solution's there.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
fn matches(given: Given, solution: &Grid<u8>) -> Result<(), WellPosedError> {
    match at(solution, given.position) {
        None => Err(WellPosedError::GivenOffGrid { given }),
        Some(digit) if digit == given.digit => Ok(()),
        Some(solution) => Err(WellPosedError::GivenMismatch { given, solution }),
    }
}

/// Why givens and a solution were refused as a proof.
///
/// The `Display` text of each variant is stable API: bindings build their exceptions
/// from it.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum WellPosedError {
    /// The givens leave the player nothing to do.
    #[error("{givens} givens leave nothing to play: a puzzle has fewer givens than its 81 cells")]
    NothingLeftToPlay {
        /// How many givens there were.
        givens: usize,
    },
    /// The solution leaves a cell empty.
    #[error("the solution has no digit at row {}, column {}", .position.row, .position.column)]
    SolutionNotFull {
        /// The first empty cell, in grid order.
        position: Position,
    },
    /// The solution holds something that is not a digit from 1 to 9.
    #[error(
        "the solution's digit at row {}, column {} is {digit}, outside 1 to 9",
        .position.row,
        .position.column
    )]
    SolutionDigitOutOfRange {
        /// The first offending cell, in grid order.
        position: Position,
        /// What it holds.
        digit: u8,
    },
    /// Two peers of the solution hold one digit.
    #[error(
        "the solution holds {digit} at row {}, column {} and at row {}, column {}, which are peers",
        .first.row,
        .first.column,
        .second.row,
        .second.column
    )]
    SolutionConflict {
        /// The earlier of the two cells, in grid order.
        first: Position,
        /// Its first peer, in grid order, that holds the same digit.
        second: Position,
        /// The digit both hold.
        digit: u8,
    },
    /// A given names a position the grid does not have.
    #[error(
        "the given {} at row {}, column {} is off the grid",
        .given.digit,
        .given.position.row,
        .given.position.column
    )]
    GivenOffGrid {
        /// The given.
        given: Given,
    },
    /// A given's digit is not the solution's at its position.
    #[error(
        "the given at row {}, column {} is {}, and the solution's digit there is {solution}",
        .given.position.row,
        .given.position.column,
        .given.digit
    )]
    GivenMismatch {
        /// The given.
        given: Given,
        /// The solution's digit at the given's position.
        solution: u8,
    },
}

/// The proof that a set of givens is well-posed: the givens, and the one solution they
/// have.
///
/// `SetPuzzle` admits only givens that leave something to play and have exactly one
/// solution. This module cannot count solutions, so a puzzle is set from this value
/// and from nothing else, and whoever makes one vouches for the uniqueness.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) struct WellPosed {
    givens: BTreeSet<Given>,
    solution: Grid<u8>,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
impl WellPosed {
    /// Makes the proof, on its caller's word that `solution` is the only solution
    /// `givens` have.
    ///
    /// Only the solver calls this, a rule review holds: nothing else can know that the
    /// solution is the only one. Everything else is checked here.
    pub(crate) fn vouch(
        givens: BTreeSet<Given>,
        solution: Grid<Option<u8>>,
    ) -> Result<Self, WellPosedError> {
        if givens.len() >= LINE * LINE {
            return Err(WellPosedError::NothingLeftToPlay {
                givens: givens.len(),
            });
        }
        let solution = filled(&solution)?;
        consistent(&solution)?;
        givens
            .iter()
            .try_for_each(|&given| matches(given, &solution))?;
        Ok(Self { givens, solution })
    }

    /// The givens the proof is about.
    pub(crate) const fn givens(&self) -> &BTreeSet<Given> {
        &self.givens
    }

    /// The one solution of the givens: rows top to bottom, and in each row the columns
    /// left to right.
    pub(crate) const fn solution(&self) -> &Grid<u8> {
        &self.solution
    }

    /// The givens and the solution, for the puzzle that is set from them. A proof is
    /// taken apart only here and made only by [`Self::vouch`].
    pub(super) fn into_parts(self) -> (BTreeSet<Given>, Grid<u8>) {
        (self.givens, self.solution)
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{PUZZLE, SOLUTION, fixture, givens, grid};
    use super::{Given, Grid, Position, WellPosed, WellPosedError};
    use alloc::collections::BTreeSet;
    use alloc::string::ToString;

    /// The fixture's solution with one cell rewritten.
    fn solution_with(position: Position, cell: Option<u8>) -> Grid<Option<u8>> {
        let mut solution = grid(SOLUTION);
        solution[usize::from(position.row() - 1)][usize::from(position.column() - 1)] = cell;
        solution
    }

    #[test]
    fn a_proof_is_refused_for_eighty_one_givens() {
        assert_eq!(
            WellPosed::vouch(givens(SOLUTION), grid(SOLUTION)),
            Err(WellPosedError::NothingLeftToPlay { givens: 81 })
        );
    }

    #[test]
    fn a_proof_is_refused_for_a_solution_that_is_not_full() {
        let position = Position::new(2, 3);
        assert_eq!(
            WellPosed::vouch(givens(PUZZLE), solution_with(position, None)),
            Err(WellPosedError::SolutionNotFull { position })
        );
    }

    #[test]
    fn a_proof_is_refused_for_a_solution_digit_out_of_range() {
        let position = Position::new(9, 4);
        for digit in [0, 10, u8::MAX] {
            assert_eq!(
                WellPosed::vouch(givens(PUZZLE), solution_with(position, Some(digit))),
                Err(WellPosedError::SolutionDigitOutOfRange { position, digit })
            );
        }
    }

    #[test]
    fn a_proof_is_refused_for_two_peers_holding_one_digit() {
        // Row 1 of the solution is 534678912 and its first box is 534, 672, 198.
        // Each rewrite repeats the 5 of row 1, column 1: along its row, down its
        // column, and inside its box where neither the row nor the column is shared.
        for second in [
            Position::new(1, 9),
            Position::new(9, 1),
            Position::new(2, 2),
        ] {
            assert_eq!(
                WellPosed::vouch(BTreeSet::new(), solution_with(second, Some(5))),
                Err(WellPosedError::SolutionConflict {
                    first: Position::new(1, 1),
                    second,
                    digit: 5
                })
            );
        }
    }

    #[test]
    fn a_proof_is_refused_for_a_given_off_the_grid() {
        for position in [
            Position::new(0, 1),
            Position::new(1, 0),
            Position::new(10, 1),
            Position::new(1, 10),
        ] {
            let given = Given::new(position, 5);
            assert_eq!(
                WellPosed::vouch([given].into(), grid(SOLUTION)),
                Err(WellPosedError::GivenOffGrid { given })
            );
        }
    }

    #[test]
    fn a_proof_is_refused_for_a_given_that_is_not_the_solutions() {
        // The solution holds 5 at row 1, column 1.
        for digit in [6, 0, 10] {
            let given = Given::new(Position::new(1, 1), digit);
            assert_eq!(
                WellPosed::vouch([given].into(), grid(SOLUTION)),
                Err(WellPosedError::GivenMismatch { given, solution: 5 })
            );
        }
    }

    #[test]
    fn two_givens_that_disagree_about_one_position_are_refused() {
        let here = Position::new(1, 1);
        let disagreeing: BTreeSet<Given> = [Given::new(here, 5), Given::new(here, 6)].into();
        assert_eq!(disagreeing.len(), 2);
        assert_eq!(
            WellPosed::vouch(disagreeing, grid(SOLUTION)),
            Err(WellPosedError::GivenMismatch {
                given: Given::new(here, 6),
                solution: 5
            })
        );
    }

    #[test]
    fn a_proof_gives_back_its_givens_and_its_solution() {
        let proof = fixture();
        assert_eq!(proof.givens(), &givens(PUZZLE));
        assert_eq!(proof.givens().len(), 30);
        assert_eq!(proof.solution().map(|row| row.map(Some)), grid(SOLUTION));
    }

    #[test]
    fn proof_error_texts_are_stable() {
        let here = Position::new(2, 3);
        let given = Given::new(here, 7);
        let texts = [
            (
                WellPosedError::NothingLeftToPlay { givens: 81 },
                "81 givens leave nothing to play: a puzzle has fewer givens than its 81 cells",
            ),
            (
                WellPosedError::SolutionNotFull { position: here },
                "the solution has no digit at row 2, column 3",
            ),
            (
                WellPosedError::SolutionDigitOutOfRange {
                    position: here,
                    digit: 10,
                },
                "the solution's digit at row 2, column 3 is 10, outside 1 to 9",
            ),
            (
                WellPosedError::SolutionConflict {
                    first: here,
                    second: Position::new(2, 8),
                    digit: 4,
                },
                "the solution holds 4 at row 2, column 3 and at row 2, column 8, which are peers",
            ),
            (
                WellPosedError::GivenOffGrid {
                    given: Given::new(Position::new(0, 10), 7),
                },
                "the given 7 at row 0, column 10 is off the grid",
            ),
            (
                WellPosedError::GivenMismatch { given, solution: 2 },
                "the given at row 2, column 3 is 7, and the solution's digit there is 2",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }
}
