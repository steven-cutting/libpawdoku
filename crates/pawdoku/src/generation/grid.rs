//! The solution grid: eleven givens drawn, put to the solver, one grid attempt after
//! another until one finds a grid or the attempts are spent.

use super::choice::index_among;
use super::order::row_by_row;
use crate::random::{RandomError, RandomStream};
use crate::solver::search;
use crate::sudoku::{BOX_SIDE, Given, Grid, Position, SIDE};
use alloc::vec::Vec;

/// How many givens a grid attempt draws before it asks the solver for a grid:
/// `config.seeded_givens`, the source paper's figure.
const SEEDED_GIVENS: usize = 11;

/// The most grid attempts a generation may fail before it is refused:
/// `config.grid_attempt_limit`.
pub(super) const GRID_ATTEMPT_LIMIT: u32 = 100;

/// Whether two positions share a row, a column or a box.
fn share_a_unit(a: Position, b: Position) -> bool {
    let band = |line: u8| line.saturating_sub(1) / BOX_SIDE;
    let share_a_box = band(a.row()) == band(b.row()) && band(a.column()) == band(b.column());
    a.row() == b.row() || a.column() == b.column() || share_a_box
}

/// The digits no seeded given holds in the row, column or box of `position`, lowest
/// first.
fn allowed_digits(seeded: &[Given], position: Position) -> Vec<u8> {
    let is_taken = |digit: u8| {
        let holds = |given: &Given| given.digit() == digit;
        let beside = |given: &Given| share_a_unit(given.position(), position);
        seeded.iter().any(|given| holds(given) && beside(given))
    };
    (1..=SIDE).filter(|&digit| !is_taken(digit)).collect()
}

/// `drawn_seeding`: the givens a grid attempt draws, one after another from an empty
/// grid until there are [`SEEDED_GIVENS`] of them or one cannot be drawn.
///
/// Each takes two draws. The first chooses its position among the positions that hold
/// no given yet, in row order. The second chooses its digit among those no earlier
/// given holds in that position's row, column or box, lowest first. When no digit is
/// left the second draw is not taken and the seeding stops there, short. So the givens
/// never conflict.
fn seed(stream: &mut dyn RandomStream) -> Result<Vec<Given>, RandomError> {
    let mut empty: Vec<Position> = row_by_row().collect();
    let mut seeded = Vec::new();
    for _ in 0..SEEDED_GIVENS {
        let position = empty.remove(index_among(stream.next_draw()?, empty.len()));
        let allowed = allowed_digits(&seeded, position);
        if allowed.is_empty() {
            break;
        }
        let chosen = allowed.get(index_among(stream.next_draw()?, allowed.len()));
        seeded.extend(chosen.map(|&digit| Given::new(position, digit)));
    }
    Ok(seeded)
}

/// `found_grid`: the solution grid the solver's search of the seeded givens gives, or
/// nothing when its verdict is none.
///
/// A verdict of many gives two solutions, in no order the specification states, and
/// the grid is the one that holds the lower digit at the first position, in row
/// order, where the two differ. A grid compares row by row and each row from the
/// left, so that one is the lesser of the two.
///
/// A short seeding is not put to the solver and gives nothing.
fn found_grid(seeded: &[Given]) -> Option<Grid<u8>> {
    let is_full = seeded.len() == SEEDED_GIVENS;
    let found = is_full.then(|| search(seeded.iter().copied()))?;
    found.solutions().iter().min().copied()
}

/// `SettleGridAttempt`: one grid attempt, which finds a grid or fails.
fn attempt(stream: &mut dyn RandomStream) -> Result<Option<Grid<u8>>, RandomError> {
    Ok(found_grid(&seed(stream)?))
}

/// `FollowGridAttempt`, as far as the grid: one grid attempt after another until one
/// finds a grid, or nothing when [`GRID_ATTEMPT_LIMIT`] of them have failed.
///
/// The draws failed attempts took stay taken. A stream that has no draw to give ends
/// the drawing there with its own error, which is not a failed attempt.
pub(super) fn draw_grid(stream: &mut dyn RandomStream) -> Result<Option<Grid<u8>>, RandomError> {
    for _ in 0..GRID_ATTEMPT_LIMIT {
        let found = attempt(stream)?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{attempt, draw_grid, found_grid, seed};
    use crate::random::{RandomError, RandomStream, ReplayStream};
    use crate::solver::{Verdict, search};
    use crate::sudoku::{Given, Grid, Position};
    use alloc::collections::BTreeSet;
    use alloc::vec;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    /// A draw that picks index `index` among `n`: the middle of its share of `[0, 1)`.
    fn pick(index: u8, n: u8) -> f64 {
        (f64::from(index) + 0.5) / f64::from(n)
    }

    fn replaying(script: Vec<f64>) -> ReplayStream {
        ReplayStream::new(script).unwrap()
    }

    /// Givens as (row, column, digit), to lay beside a list worked out by hand.
    fn triples(givens: &[Given]) -> Vec<(u8, u8, u8)> {
        let triple = |given: &Given| {
            (
                given.position().row(),
                given.position().column(),
                given.digit(),
            )
        };
        givens.iter().map(triple).collect()
    }

    /// Whether two cells share a row, a column or a box, written from `sudoku.allium`.
    fn share_a_unit(a: Position, b: Position) -> bool {
        let band = |line: u8| (line - 1) / 3;
        let share_a_box = band(a.row()) == band(b.row()) && band(a.column()) == band(b.column());
        a.row() == b.row() || a.column() == b.column() || share_a_box
    }

    /// Whether no two of the givens share a position, or a digit and a unit.
    fn agree(givens: &[Given]) -> bool {
        let pairs = givens.iter().enumerate();
        let mut pairs = pairs.flat_map(|(at, a)| givens[at + 1..].iter().map(move |b| (a, b)));
        pairs.all(|(a, b)| {
            let apart = a.position() != b.position();
            apart && !(a.digit() == b.digit() && share_a_unit(a.position(), b.position()))
        })
    }

    /// The draws of a seeding that ends short after nine givens: 1 to 8 along row 1
    /// from zeros, then a 9 at (2,9), which is the tenth of the 73 empty positions and
    /// takes the last of its seven digits, and then (1,9), the first empty position,
    /// which has no digit left. Nineteen draws.
    fn a_short_seeding() -> Vec<f64> {
        let mut script = vec![0.0; 16];
        script.extend([pick(9, 73), pick(6, 7), 0.0]);
        script
    }

    /// The draws of a full seeding with no solution: the nine givens above, and then
    /// (2,1) and (2,2), each the second empty position, so (1,9) is never drawn and is
    /// left with no digit. Twenty-two draws.
    fn a_seeding_with_no_solution() -> Vec<f64> {
        let mut script = vec![0.0; 16];
        script.extend([pick(9, 73), pick(6, 7)]);
        script.extend([pick(1, 72), 0.0, pick(1, 71), 0.0]);
        script
    }

    /// What zeros seed: the first empty position each time and its lowest digit.
    const SEEDED_BY_ZEROS: [(u8, u8, u8); 11] = [
        (1, 1, 1),
        (1, 2, 2),
        (1, 3, 3),
        (1, 4, 4),
        (1, 5, 5),
        (1, 6, 6),
        (1, 7, 7),
        (1, 8, 8),
        (1, 9, 9),
        (2, 1, 4),
        (2, 2, 5),
    ];

    /// Whether a grid is a full grid with no conflict: every row, column and box holds
    /// nine different digits from 1 to 9.
    fn is_a_solution_grid(grid: &Grid<u8>) -> bool {
        let cell = |row: usize, column: usize| grid[row][column];
        let full = |unit: Vec<u8>| unit.into_iter().collect::<BTreeSet<u8>>() == (1..=9).collect();
        (0..9).all(|at| {
            let (top, left) = (at / 3 * 3, at % 3 * 3);
            full((0..9).map(|column| cell(at, column)).collect())
                && full((0..9).map(|row| cell(row, at)).collect())
                && full((0..9).map(|k| cell(top + k / 3, left + k % 3)).collect())
        })
    }

    #[test]
    fn scripted_draws_seed_eleven_givens_no_two_in_conflict() {
        let mut stream = replaying(vec![0.0; 30]);
        let seeded = seed(&mut stream).unwrap();
        assert_eq!(triples(&seeded), SEEDED_BY_ZEROS);
        assert_eq!(stream.index(), 22);

        // The last of 81 positions is (9,9), and the last of its nine digits is 9. The
        // last of the 80 left is (9,8), and the last of its eight digits is 8.
        let mut script = vec![pick(80, 81), pick(8, 9), pick(79, 80), pick(7, 8)];
        script.extend([0.0; 4]);
        let mut stream = replaying(script);
        let seeded = seed(&mut stream);
        assert_eq!(seeded, Err(RandomError::Exhausted { index: 8 }));
        let mut script = vec![pick(80, 81), pick(8, 9), pick(79, 80), pick(7, 8)];
        script.extend([pick(40, 79), pick(4, 9)]);
        script.extend([0.0; 16]);
        let seeded = seed(&mut replaying(script)).unwrap();
        // (5,5) is the forty-first of the 79 left; its row, column and box hold no
        // given yet, so its fifth digit is 5. Zeros then run along row 1 with the
        // lowest digit each time: (1,5) is in the column of that 5, so it takes 6 and
        // (1,6) takes the 5; and (1,8), with 1 to 7 in its row and the 8 of (9,8) in
        // its column, has only 9 left.
        assert_eq!(triples(&seeded)[..3], [(9, 9, 9), (9, 8, 8), (5, 5, 5)]);
        assert_eq!(triples(&seeded)[3..7], SEEDED_BY_ZEROS[..4]);
        assert_eq!(
            triples(&seeded)[7..],
            [(1, 5, 6), (1, 6, 5), (1, 7, 7), (1, 8, 9)]
        );
        assert!(agree(&seeded));
    }

    #[test]
    fn a_position_with_no_digit_left_ends_the_seeding_short() {
        let mut script = a_short_seeding();
        script.extend([0.0; 10]);
        let mut stream = replaying(script.clone());
        let seeded = seed(&mut stream).unwrap();
        let mut expected = SEEDED_BY_ZEROS[..8].to_vec();
        expected.push((2, 9, 9));
        assert_eq!(triples(&seeded), expected);
        // Two draws for each of the nine givens and one for the position that stopped it.
        assert_eq!(stream.index(), 19);

        // A short seeding is not put to the solver, whatever it would say: none for
        // these nine givens, and many for the first eight, which are still no grid.
        assert_eq!(
            search(seeded.iter().copied()).verdict(),
            Verdict::NoSolution
        );
        let without_the_nine = &seeded[..8];
        assert_eq!(
            search(without_the_nine.iter().copied()).verdict(),
            Verdict::ManySolutions
        );
        assert_eq!(found_grid(without_the_nine), None);
        assert_eq!(attempt(&mut replaying(script)), Ok(None));
    }

    #[test]
    fn eleven_givens_with_no_solution_fail_the_attempt() {
        let mut stream = replaying(a_seeding_with_no_solution());
        let seeded = seed(&mut stream).unwrap();
        assert_eq!(seeded.len(), 11);
        assert_eq!(triples(&seeded)[8..], [(2, 9, 9), (2, 1, 4), (2, 2, 5)]);
        assert!(agree(&seeded));
        assert_eq!(
            search(seeded.iter().copied()).verdict(),
            Verdict::NoSolution
        );

        let mut stream = replaying(a_seeding_with_no_solution());
        assert_eq!(attempt(&mut stream), Ok(None));
        assert_eq!(stream.index(), 22);
    }

    #[test]
    fn a_found_grid_is_full_and_holds_every_seeded_given() {
        let mut stream = replaying(vec![0.0; 22]);
        let grid = attempt(&mut stream).unwrap().unwrap();
        assert!(is_a_solution_grid(&grid));
        for (row, column, digit) in SEEDED_BY_ZEROS {
            assert_eq!(grid[usize::from(row - 1)][usize::from(column - 1)], digit);
        }
        assert_eq!(stream.index(), 22);
    }

    #[test]
    fn the_next_attempt_begins_after_a_failed_one() {
        let mut script = a_seeding_with_no_solution();
        script.extend(a_short_seeding());
        script.extend([0.0; 22]);
        let mut stream = replaying(script);
        let grid = draw_grid(&mut stream).unwrap().unwrap();
        assert_eq!(Some(grid), attempt(&mut replaying(vec![0.0; 22])).unwrap());
        // The draws the two failed attempts took stay taken.
        assert_eq!(stream.index(), 22 + 19 + 22);
    }

    #[test]
    fn a_hundred_failed_attempts_are_a_refusal() {
        let hundred: Vec<f64> = (0..100).flat_map(|_| a_short_seeding()).collect();
        let mut spent = hundred.clone();
        spent.extend([0.0; 30]);
        let mut stream = replaying(spent);
        assert_eq!(draw_grid(&mut stream), Ok(None));
        // The draws of the hundred attempts and no more: no hundred-and-first is begun.
        assert_eq!(stream.index(), 1900);

        // Ninety-nine failed attempts are not a refusal.
        let mut script = hundred[19..].to_vec();
        script.extend([0.0; 22]);
        let mut stream = replaying(script);
        assert!(draw_grid(&mut stream).unwrap().is_some());
        assert_eq!(stream.index(), 99 * 19 + 22);
    }

    #[test]
    fn a_stream_that_runs_out_in_the_grid_says_so() {
        for (script, index) in [(vec![], 0), (vec![0.0; 5], 5), (vec![0.0; 21], 21)] {
            assert_eq!(
                draw_grid(&mut replaying(script)),
                Err(RandomError::Exhausted { index })
            );
        }
        // In a later attempt too: the failed attempt before it is not what is reported.
        let mut script = a_short_seeding();
        script.extend([0.0; 3]);
        assert_eq!(
            draw_grid(&mut replaying(script)),
            Err(RandomError::Exhausted { index: 22 })
        );
    }

    proptest! {
        #[test]
        fn any_draws_seed_givens_that_agree(script in prop::collection::vec(0.0..1.0_f64, 22)) {
            let mut stream = replaying(script);
            let seeded = seed(&mut stream).unwrap();
            prop_assert!(agree(&seeded));
            let count = u64::try_from(seeded.len()).unwrap();
            let draws = if count < 11 { 2 * count + 1 } else { 22 };
            prop_assert_eq!(stream.index(), draws);
        }
    }

    /// Eleven givens that agree and have many solutions, of which the solver's search
    /// reaches the higher grid first. Found by trying seeds; the search's order follows
    /// from the solver's tie-break, which the first two assertions pin for this case.
    #[test]
    fn of_two_solutions_the_grid_is_the_lower_row_by_row() {
        let seeded = [
            (4, 8, 9),
            (8, 7, 5),
            (4, 5, 5),
            (9, 3, 5),
            (2, 5, 6),
            (8, 1, 7),
            (5, 8, 6),
            (8, 5, 8),
            (6, 4, 2),
            (7, 6, 2),
            (3, 9, 1),
        ]
        .map(|(row, column, digit)| Given::new(Position::new(row, column), digit));
        assert!(agree(&seeded));
        let found = search(seeded);
        let [first, second] = found.solutions() else {
            panic!("these givens have many solutions");
        };
        let cells = first.iter().flatten().zip(second.iter().flatten());
        let (in_first, in_second) = cells.clone().find(|(a, b)| a != b).unwrap();
        assert!(in_second < in_first);

        let grid = found_grid(&seeded).unwrap();
        assert_eq!(&grid, second);
        assert_ne!(&grid, first);
    }
}
