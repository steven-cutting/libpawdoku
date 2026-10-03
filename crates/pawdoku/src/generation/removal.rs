//! Removal: every position visited once, in an order, and its given taken away while
//! the bound, the floor and the solver's verdict allow.

use super::order::row_by_row;
use crate::random::RandomError;
use crate::solver::{Verdict, search};
use crate::sudoku::{Given, Grid, Position};
use alloc::collections::BTreeSet;

/// What removal will not go below: `Generation.bound` and `Generation.floor`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Limits {
    /// The fewest givens removal may leave in all.
    pub(super) bound: usize,
    /// The fewest givens removal may leave in any row or column.
    pub(super) floor: usize,
}

/// How a visit ends: `Visit.status`, without `pending`.
#[derive(Debug, PartialEq, Eq)]
enum Decision {
    RefusedByBound,
    RefusedByFloor,
    Kept,
    /// The position is emptied, and these are the givens without it.
    Emptied(BTreeSet<Given>),
}

/// `DecideVisit`: what a visit to `position` makes of the givens as they stand. One
/// rule and four ends, in this order; a refused removal asks the solver nothing.
fn decide(givens: &BTreeSet<Given>, position: Position, limits: Limits) -> Decision {
    if givens.len() <= limits.bound {
        return Decision::RefusedByBound;
    }
    let beside = |line: fn(Position) -> u8| {
        let in_line = |given: &&Given| line(given.position()) == line(position);
        givens.iter().filter(in_line).count()
    };
    if beside(Position::row) <= limits.floor || beside(Position::column) <= limits.floor {
        return Decision::RefusedByFloor;
    }
    let without: BTreeSet<Given> = givens
        .iter()
        .copied()
        .filter(|given| given.position() != position)
        .collect();
    match search(without.iter().copied()).verdict() {
        Verdict::OneSolution => Decision::Emptied(without),
        _ => Decision::Kept,
    }
}

/// The solution grid as a given for every position.
fn every_given(solution: &Grid<u8>) -> BTreeSet<Given> {
    let digits = solution.iter().flatten().copied();
    row_by_row()
        .zip(digits)
        .map(|(position, digit)| Given::new(position, digit))
        .collect()
}

/// Removal from the whole of `solution`: a visit for each position `visits` gives, in
/// that order, and the givens left when it has given its last.
///
/// It draws nothing itself. A drawn order takes a draw for each position it gives, and
/// the error of a stream that has none is carried out from the visit it stopped.
pub(super) fn remove(
    solution: &Grid<u8>,
    visits: impl Iterator<Item = Result<Position, RandomError>>,
    limits: Limits,
) -> Result<BTreeSet<Given>, RandomError> {
    let mut givens = every_given(solution);
    for visit in visits {
        if let Decision::Emptied(without) = decide(&givens, visit?, limits) {
            givens = without;
        }
    }
    Ok(givens)
}

#[cfg(test)]
mod tests {
    use super::super::order::row_by_row;
    use super::{Decision, Limits, decide, remove};
    use crate::random::RandomError;
    use crate::sudoku::{Given, Grid, Position};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    /// The solution of the rules' fixture, `docs/reference/testing.md`.
    const SOLUTION: [&str; 9] = [
        "534678912",
        "672195348",
        "198342567",
        "859761423",
        "426853791",
        "713924856",
        "961537284",
        "287419635",
        "345286179",
    ];

    const NO_LIMITS: Limits = Limits { bound: 0, floor: 0 };

    fn solution() -> Grid<u8> {
        SOLUTION.map(|row| {
            let mut digits = row.bytes().map(|digit| digit - b'0');
            core::array::from_fn(|_| digits.next().unwrap())
        })
    }

    fn at(row: u8, column: u8) -> Position {
        Position::new(row, column)
    }

    /// The solution's given at a position.
    fn given(row: u8, column: u8) -> Given {
        let digit = solution()[usize::from(row - 1)][usize::from(column - 1)];
        Given::new(at(row, column), digit)
    }

    /// Every given of the solution but those at `emptied`.
    fn all_but(emptied: &[(u8, u8)]) -> BTreeSet<Given> {
        let kept = row_by_row().filter(|at| !emptied.contains(&(at.row(), at.column())));
        kept.map(|at| given(at.row(), at.column())).collect()
    }

    fn visiting(positions: &[(u8, u8)]) -> impl Iterator<Item = Result<Position, RandomError>> {
        positions.iter().map(|&(row, column)| Ok(at(row, column)))
    }

    fn row_of(givens: &BTreeSet<Given>, row: u8) -> Vec<u8> {
        let in_row = givens.iter().filter(|given| given.position().row() == row);
        in_row.map(|given| given.position().column()).collect()
    }

    fn column_of(givens: &BTreeSet<Given>, column: u8) -> Vec<u8> {
        let in_column = givens
            .iter()
            .filter(|given| given.position().column() == column);
        in_column.map(|given| given.position().row()).collect()
    }

    #[test]
    fn removal_begins_from_the_whole_grid_and_the_first_visit_empties_its_position() {
        let untouched = remove(&solution(), visiting(&[]), NO_LIMITS).unwrap();
        assert_eq!(untouched, all_but(&[]));
        assert_eq!(untouched.len(), 81);

        // Eighty givens are above every bound, eight in a row above every floor, and a
        // full grid less one cell has one solution.
        let strictest = Limits {
            bound: 60,
            floor: 5,
        };
        for first in [(1, 1), (5, 5), (9, 9)] {
            let left = remove(&solution(), visiting(&[first]), strictest).unwrap();
            assert_eq!(left, all_but(&[first]));
        }
    }

    #[test]
    fn a_removal_that_would_go_below_the_bound_is_refused() {
        let limits = Limits {
            bound: 78,
            floor: 0,
        };
        let left = remove(&solution(), row_by_row().map(Ok), limits).unwrap();
        assert_eq!(left, all_but(&[(1, 1), (1, 2), (1, 3)]));
        assert_eq!(decide(&left, at(1, 4), limits), Decision::RefusedByBound);
    }

    #[test]
    fn a_removal_that_would_leave_a_row_below_the_floor_is_refused() {
        let limits = Limits { bound: 0, floor: 5 };
        let along_row_one: Vec<(u8, u8)> = (1..=9).map(|column| (1, column)).collect();
        let left = remove(&solution(), visiting(&along_row_one), limits).unwrap();
        assert_eq!(row_of(&left, 1), [5, 6, 7, 8, 9]);
        assert_eq!(left.len(), 77);
        assert_eq!(decide(&left, at(1, 5), limits), Decision::RefusedByFloor);
    }

    #[test]
    fn a_removal_that_would_leave_a_column_below_the_floor_is_refused() {
        let limits = Limits { bound: 0, floor: 5 };
        let down_column_one: Vec<(u8, u8)> = (1..=9).map(|row| (row, 1)).collect();
        let left = remove(&solution(), visiting(&down_column_one), limits).unwrap();
        assert_eq!(column_of(&left, 1), [5, 6, 7, 8, 9]);
        assert_eq!(left.len(), 77);
        assert_eq!(decide(&left, at(5, 1), limits), Decision::RefusedByFloor);
    }

    #[test]
    fn the_bound_is_read_before_the_floor() {
        let both = Limits {
            bound: 81,
            floor: 9,
        };
        assert_eq!(
            decide(&all_but(&[]), at(1, 1), both),
            Decision::RefusedByBound
        );
    }

    /// Rows 3 and 6 of columns 5 and 6 hold 4 and 2 one way round and can as well hold
    /// them the other: `TWO_SOLUTIONS_DENSE` in `tests/solver.rs`. With three of the
    /// four emptied the fourth still fixes them; without it there are two solutions.
    #[test]
    fn a_given_whose_removal_gives_many_solutions_stays() {
        let rectangle = [(3, 5), (3, 6), (6, 5), (6, 6)];
        let three_gone = all_but(&rectangle[..3]);
        assert_eq!(decide(&three_gone, at(6, 6), NO_LIMITS), Decision::Kept);

        let left = remove(&solution(), visiting(&rectangle), NO_LIMITS).unwrap();
        assert_eq!(left, three_gone);
    }

    #[test]
    fn a_given_whose_removal_keeps_one_solution_goes() {
        let two_gone = all_but(&[(3, 5), (3, 6)]);
        let without = all_but(&[(3, 5), (3, 6), (6, 5)]);
        assert_eq!(
            decide(&two_gone, at(6, 5), NO_LIMITS),
            Decision::Emptied(without)
        );
    }

    #[test]
    fn every_position_is_visited_and_none_is_asked_about_twice() {
        let limits = Limits {
            bound: 78,
            floor: 0,
        };
        let mut visited = Vec::new();
        let visits = row_by_row().inspect(|&at| visited.push(at)).map(Ok);
        let left = remove(&solution(), visits, limits).unwrap();
        assert_eq!(left.len(), 78);
        assert_eq!(visited, row_by_row().collect::<Vec<Position>>());
    }

    #[test]
    fn removal_stops_at_the_first_draw_the_order_cannot_give() {
        let ran_out = RandomError::Exhausted { index: 5 };
        let visits = [Ok(at(1, 1)), Err(ran_out.clone()), Ok(at(1, 2))];
        assert_eq!(
            remove(&solution(), visits.into_iter(), NO_LIMITS),
            Err(ran_out)
        );
    }
}
