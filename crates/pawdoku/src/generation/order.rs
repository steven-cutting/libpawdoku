//! The orders of removal: which position is visited at every step.

use alloc::boxed::Box;

use super::choice::index_among;
use crate::random::{RandomError, RandomStream};
use crate::sudoku::{Position, SIDE};
use alloc::vec::Vec;

/// The four orders of removal: the specification's `RemovalOrder`, and the source
/// paper's four sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Order {
    /// "Randomizing globally".
    Drawn,
    /// "Jumping one cell".
    EveryOther,
    /// "Wandering along S".
    SPath,
    /// "Left to Right then Top to Bottom".
    RowByRow,
}

/// The visits of an order, one after another: `BeginVisit`. Only the drawn order
/// reads the stream, and only it can fail.
pub(super) fn visits<'a>(
    order: Order,
    stream: &'a mut dyn RandomStream,
) -> Box<dyn Iterator<Item = Result<Position, RandomError>> + 'a> {
    match order {
        Order::Drawn => Box::new(drawn(stream)),
        Order::EveryOther => Box::new(every_other().map(Ok)),
        Order::SPath => Box::new(s_path().map(Ok)),
        Order::RowByRow => Box::new(row_by_row().map(Ok)),
    }
}

/// Row order, `grid_positions` in the specification: every position, row by row from
/// the top and each row from the left.
pub(super) fn row_by_row() -> impl Iterator<Item = Position> {
    (1..=SIDE).flat_map(|row| (1..=SIDE).map(move |column| Position::new(row, column)))
}

/// The S path: row 1 from the left, row 2 from the right, and so on down, turning at
/// the end of every row.
pub(super) fn s_path() -> impl Iterator<Item = Position> {
    let column = |row: u8, step: u8| if row % 2 == 1 { step } else { SIDE + 1 - step };
    (1..=SIDE)
        .flat_map(move |row| (1..=SIDE).map(move |step| Position::new(row, column(row, step))))
}

/// Every other cell: the odd-numbered steps of the S path in order, then its
/// even-numbered steps in order. The first pass is the source paper's figure; the
/// second is the specification's own reading.
pub(super) fn every_other() -> impl Iterator<Item = Position> {
    s_path().step_by(2).chain(s_path().skip(1).step_by(2))
}

/// The drawn order: every visit takes one draw, which chooses among the positions not
/// yet visited, in row order. The last visit draws too, with one position left and
/// nothing to decide: 81 draws in all.
///
/// A visit whose draw the stream cannot give is that error.
pub(super) fn drawn(
    stream: &mut dyn RandomStream,
) -> impl Iterator<Item = Result<Position, RandomError>> + '_ {
    let mut unvisited: Vec<Position> = row_by_row().collect();
    core::iter::from_fn(move || {
        let draw = (!unvisited.is_empty()).then(|| stream.next_draw())?;
        Some(draw.map(|u| unvisited.remove(index_among(u, unvisited.len()))))
    })
}

#[cfg(test)]
mod tests {
    use super::{Order, drawn, every_other, row_by_row, s_path, visits};
    use crate::random::{RandomError, RandomStream, ReplayStream};
    use crate::sudoku::Position;
    use alloc::collections::BTreeSet;
    use alloc::vec;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    /// A draw that picks index `index` among `n`: the middle of its share of `[0, 1)`.
    fn pick(index: u8, n: u8) -> f64 {
        (f64::from(index) + 0.5) / f64::from(n)
    }

    /// The whole drawn order for a script of draws, or the error that stopped it.
    fn drawn_from(script: Vec<f64>) -> (Result<Vec<Position>, RandomError>, u64) {
        let mut stream = ReplayStream::new(script).unwrap();
        let order = drawn(&mut stream).collect();
        (order, stream.index())
    }

    /// The positions as (row, column) pairs, to lay beside a list worked out by hand.
    fn pairs(order: impl Iterator<Item = Position>) -> Vec<(u8, u8)> {
        order.map(|at| (at.row(), at.column())).collect()
    }

    /// Whether an order is the 81 positions of the grid, each once.
    fn is_every_position_once(order: &[(u8, u8)]) -> bool {
        let distinct: BTreeSet<&(u8, u8)> = order.iter().collect();
        let on_the_grid =
            |&(row, column): &(u8, u8)| (1..=9).contains(&row) && (1..=9).contains(&column);
        order.len() == 81 && distinct.len() == 81 && order.iter().all(on_the_grid)
    }

    /// The S path written out a row at a time, apart from the code it is laid beside:
    /// the odd rows from the left and the even rows from the right.
    fn the_s_path_by_hand() -> Vec<(u8, u8)> {
        let mut path = Vec::new();
        for row in 1..=9 {
            let mut columns: Vec<u8> = (1..=9).collect();
            if row % 2 == 0 {
                columns.reverse();
            }
            path.extend(columns.into_iter().map(|column| (row, column)));
        }
        path
    }

    #[test]
    fn row_by_row_is_every_position_once_a_row_at_a_time() {
        let order = pairs(row_by_row());
        let mut by_hand = Vec::new();
        for row in 1..=9 {
            for column in 1..=9 {
                by_hand.push((row, column));
            }
        }
        assert_eq!(order, by_hand);
        assert!(is_every_position_once(&order));
        assert_eq!(order[..3], [(1, 1), (1, 2), (1, 3)]);
        assert_eq!(order[9], (2, 1));
        assert_eq!(order[80], (9, 9));
    }

    #[test]
    fn s_path_turns_at_the_end_of_each_row() {
        let order = pairs(s_path());
        assert_eq!(order, the_s_path_by_hand());
        assert!(is_every_position_once(&order));
        assert_eq!(order[7..11], [(1, 8), (1, 9), (2, 9), (2, 8)]);
        assert_eq!(order[17..19], [(2, 1), (3, 1)]);
        assert_eq!(order[80], (9, 9));
    }

    #[test]
    fn every_other_takes_the_odd_steps_of_the_s_path_then_the_even() {
        let order = pairs(every_other());
        // All 81 steps: the S path's first, third, fifth and on, then its second,
        // fourth and on, each pass in the path's own order.
        let path = the_s_path_by_hand();
        let steps = |parity: usize| {
            let numbered = path.iter().copied().enumerate();
            numbered
                .filter(move |(at, _)| at % 2 == parity)
                .map(|(_, position)| position)
        };
        assert_eq!(order, steps(0).chain(steps(1)).collect::<Vec<(u8, u8)>>());
        assert!(is_every_position_once(&order));
        let first_pass = [
            (1, 1),
            (1, 3),
            (1, 5),
            (1, 7),
            (1, 9),
            (2, 8),
            (2, 6),
            (2, 4),
            (2, 2),
            (3, 1),
        ];
        assert_eq!(order[..10], first_pass);
        let is_even = |&(row, column): &(u8, u8)| (row + column) % 2 == 0;
        assert!(order[..41].iter().all(is_even));
        assert!(!order[41..].iter().any(is_even));
        let second_pass = [(1, 2), (1, 4), (1, 6), (1, 8), (2, 9), (2, 7)];
        assert_eq!(order[41..47], second_pass);
    }

    #[test]
    fn scripted_draws_give_the_order_worked_out_by_hand() {
        // The last of 81 is (9,9). The first of the 80 left is (1,1). Of the 79 left,
        // (1,2) has index 0, so index 40 is the forty-second position of the grid in
        // row order, (5,6). Zeros then take what is left in row order.
        let mut script = vec![pick(80, 81), pick(0, 80), pick(40, 79)];
        script.extend([0.0; 78]);
        let order = pairs(drawn_from(script).0.unwrap().into_iter());
        assert_eq!(order[..3], [(9, 9), (1, 1), (5, 6)]);
        let rest = pairs(row_by_row()).into_iter();
        let rest: Vec<(u8, u8)> = rest.filter(|at| !order[..3].contains(at)).collect();
        assert_eq!(order[3..], rest);
    }

    #[test]
    fn the_drawn_order_takes_a_draw_for_every_visit_the_last_included() {
        let (order, draws_taken) = drawn_from(vec![0.0; 100]);
        assert_eq!(order.unwrap().len(), 81);
        assert_eq!(draws_taken, 81);
    }

    #[test]
    fn a_stream_that_runs_out_in_a_drawn_order_says_so() {
        let (order, draws_taken) = drawn_from(vec![0.0; 80]);
        assert_eq!(order, Err(RandomError::Exhausted { index: 80 }));
        assert_eq!(draws_taken, 80);
    }

    #[test]
    fn a_fixed_order_takes_no_draw_and_the_drawn_order_takes_them_all() {
        let all = |order: Order, script: Vec<f64>| {
            let mut stream = ReplayStream::new(script).unwrap();
            let visited: Result<Vec<Position>, RandomError> = visits(order, &mut stream).collect();
            (pairs(visited.unwrap().into_iter()), stream.index())
        };
        assert_eq!(all(Order::RowByRow, vec![]), (pairs(row_by_row()), 0));
        assert_eq!(all(Order::SPath, vec![]), (pairs(s_path()), 0));
        assert_eq!(all(Order::EveryOther, vec![]), (pairs(every_other()), 0));
        assert_eq!(all(Order::Drawn, vec![0.0; 81]), (pairs(row_by_row()), 81));
    }

    proptest! {
        #[test]
        fn the_drawn_order_visits_each_position_once_whatever_the_draws(
            script in prop::collection::vec(0.0..1.0_f64, 81),
        ) {
            let order = pairs(drawn_from(script).0.unwrap().into_iter());
            prop_assert!(is_every_position_once(&order));
        }
    }
}
