//! The rules of classic sudoku: the grid, what a setter may pose, the two moves a
//! player has, conflicts, and when a puzzle is solved.
//!
//! This module is `docs/specs/sudoku.allium` in Rust, and like that module it imports
//! nothing: every other engine module stands on it. [`Position`] and [`Given`] are the
//! specification's value types, plain and unconstrained, and [`BOX_SIDE`] and [`SIDE`]
//! are its two figures.
//!
//! `SetPuzzle` admits only givens with exactly one solution, and this module cannot
//! count solutions. So a [`Puzzle`] is set from a proof value, [`WellPosed`], and from
//! nothing else, and the proof is made by the solver alone: `pawdoku::solver::solve`
//! gives one for givens it finds exactly one solution to. Setting a puzzle is therefore
//! two steps, solve and then set, and the second cannot fail.
//! `docs/explanation/architecture.md` says how well-posedness reaches the rules.
//!
//! A puzzle in play is read through [`Cell`] values and its [`Status`], and changed by
//! the two moves, [`Puzzle::place`] and [`Puzzle::erase`], which refuse with a
//! [`MoveError`].
//!
//! **Not final.** This is a draft: it may change as implementation continues.
//!
//! ```
//! use pawdoku::solver::solve;
//! use pawdoku::sudoku::{Given, Position, Puzzle, Status};
//!
//! // Thirty givens, rows top to bottom, a dot for an empty cell.
//! let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!     .bytes()
//!     .zip(0_u8..)
//!     .filter(|(cell, _)| cell.is_ascii_digit())
//!     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
//!
//! let mut puzzle = Puzzle::set(solve(givens)?);
//! assert_eq!(puzzle.status(), Status::Unsolved);
//!
//! // The third cell of the first row is empty, and its digit is 4.
//! puzzle.place(Position::new(1, 3), 4)?;
//! assert!(puzzle.is_consistent());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(test)]
mod fixture;
mod proof;
mod puzzle;

pub use proof::{WellPosed, WellPosedError};
pub use puzzle::{Cell, MoveError, Puzzle, Status};

/// The side of a box: three cells across and three down, and three boxes to a band and
/// to a stack.
///
/// It mirrors `config.box_side` in `sudoku.allium`, named so that no rule carries a
/// bare number, not so that it can be tuned.
///
/// ```
/// assert_eq!(pawdoku::sudoku::BOX_SIDE, 3);
/// ```
pub const BOX_SIDE: u8 = 3;

/// The side of the grid: nine cells to a row, a column and a box, and the digits run
/// from 1 to this.
///
/// It mirrors `config.side` in `sudoku.allium`, the square of [`BOX_SIDE`].
///
/// ```
/// use pawdoku::sudoku::{BOX_SIDE, SIDE};
///
/// assert_eq!(SIDE, 9);
/// assert_eq!(SIDE, BOX_SIDE * BOX_SIDE);
/// ```
pub const SIDE: u8 = BOX_SIDE * BOX_SIDE;

/// Where a cell sits: rows count from the top and columns from the left, both from 1
/// to [`SIDE`].
///
/// A position is a plain value, as in the specification: one that is off the grid can
/// be made, because refusing it is a rule's business and not a property of the type.
/// Two positions are equal when their rows and their columns are.
///
/// The specification's `Integer` is a `u8` here. A row or a column may be anything
/// from 0 to 255, so a position can lie off the grid on either side, at 0 or at 10 and
/// above; one below 0 cannot be made.
///
/// ```
/// use pawdoku::sudoku::Position;
///
/// let position = Position::new(2, 7);
/// assert_eq!(position.row(), 2);
/// assert_eq!(position.column(), 7);
/// assert_eq!(position, Position::new(2, 7));
/// assert_ne!(position, Position::new(7, 2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Position {
    row: u8,
    column: u8,
}

impl Position {
    /// The position at `row` and `column`, whether or not the grid has such a cell.
    ///
    /// ```
    /// use pawdoku::sudoku::Position;
    ///
    /// assert_eq!(Position::new(0, 10).column(), 10);
    /// ```
    #[must_use]
    pub const fn new(row: u8, column: u8) -> Self {
        Self { row, column }
    }

    /// The row, counted from the top.
    ///
    /// ```
    /// use pawdoku::sudoku::Position;
    ///
    /// assert_eq!(Position::new(3, 5).row(), 3);
    /// ```
    #[must_use]
    pub const fn row(self) -> u8 {
        self.row
    }

    /// The column, counted from the left.
    ///
    /// ```
    /// use pawdoku::sudoku::Position;
    ///
    /// assert_eq!(Position::new(3, 5).column(), 5);
    /// ```
    #[must_use]
    pub const fn column(self) -> u8 {
        self.column
    }
}

impl Position {
    /// Which band a row is in, or which stack a column is: lines 1 to 3 are box 1,
    /// 4 to 6 box 2 and 7 to 9 box 3, as `CellsSitInTheirBox` says.
    const fn box_index(line: u8) -> u8 {
        line.saturating_sub(1) / BOX_SIDE + 1
    }

    /// The band of the box this position is in, counted from the top.
    const fn band(self) -> u8 {
        Self::box_index(self.row)
    }

    /// The stack of the box this position is in, counted from the left.
    const fn stack(self) -> u8 {
        Self::box_index(self.column)
    }

    /// Whether `other` shares a row, a column or a box with this position and is not
    /// this position.
    const fn is_peer_of(self, other: Self) -> bool {
        let shares_a_box = self.band() == other.band() && self.stack() == other.stack();
        let is_another = self.row != other.row || self.column != other.column;
        (self.row == other.row || self.column == other.column || shares_a_box) && is_another
    }
}

/// One digit the setter has written in before play begins.
///
/// A given is a plain value, as in the specification: its position may be off the grid
/// and its digit out of range, because refusing either is a rule's business. Two givens
/// are equal when their positions and their digits are, so a set holds a given once.
///
/// The digit is a `u8`, as a position's row and column are: it may be out of range on
/// either side, at 0 or at 10 and above, and one below 0 cannot be made.
///
/// ```
/// use pawdoku::sudoku::{Given, Position};
///
/// let given = Given::new(Position::new(1, 1), 5);
/// assert_eq!(given.position(), Position::new(1, 1));
/// assert_eq!(given.digit(), 5);
/// assert_eq!(given, Given::new(Position::new(1, 1), 5));
/// assert_ne!(given, Given::new(Position::new(1, 1), 6));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct Given {
    position: Position,
    digit: u8,
}

impl Given {
    /// The given that writes `digit` at `position`, whatever either is.
    ///
    /// ```
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// assert_eq!(Given::new(Position::new(9, 9), 0).digit(), 0);
    /// ```
    #[must_use]
    pub const fn new(position: Position, digit: u8) -> Self {
        Self { position, digit }
    }

    /// Where the digit is written.
    ///
    /// ```
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// assert_eq!(Given::new(Position::new(4, 6), 2).position(), Position::new(4, 6));
    /// ```
    #[must_use]
    pub const fn position(self) -> Position {
        self.position
    }

    /// The digit written there.
    ///
    /// ```
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// assert_eq!(Given::new(Position::new(4, 6), 2).digit(), 2);
    /// ```
    #[must_use]
    pub const fn digit(self) -> u8 {
        self.digit
    }
}

/// The side of the grid as a length.
pub(crate) const LINE: usize = SIDE as usize;

/// One value for every position of the grid: rows top to bottom, and in each row the
/// columns left to right.
///
/// Rows and columns count from 1 and an array counts from 0, so what the grid holds at
/// row `r`, column `c` is `grid[r - 1][c - 1]`. A solution is a `Grid<u8>`: a digit in
/// every cell.
///
/// ```
/// use pawdoku::solver::solve;
/// use pawdoku::sudoku::{Given, Grid, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let proof = solve(givens)?;
/// let solution: &Grid<u8> = proof.solution();
///
/// // Row 1, column 3.
/// assert_eq!(solution[0][2], 4);
/// assert_eq!(solution.len(), 9);
/// # Ok::<(), pawdoku::solver::SolveError>(())
/// ```
pub type Grid<T> = [[T; LINE]; LINE];

/// What a grid holds at `position`, or nothing where the grid has no such cell.
fn at<T: Copy>(grid: &Grid<T>, position: Position) -> Option<T> {
    let row = usize::from(position.row).checked_sub(1)?;
    let column = usize::from(position.column).checked_sub(1)?;
    grid.get(row)?.get(column).copied()
}

/// The place a grid keeps for `position`, or nothing where the grid has no such cell.
fn at_mut<T>(grid: &mut Grid<T>, position: Position) -> Option<&mut T> {
    let row = usize::from(position.row).checked_sub(1)?;
    let column = usize::from(position.column).checked_sub(1)?;
    grid.get_mut(row)?.get_mut(column)
}

/// Every position from row 1, column 1 to row [`SIDE`], column [`SIDE`], each once, a
/// row at a time.
fn grid_positions() -> impl Iterator<Item = Position> {
    (1..=SIDE).flat_map(|row| (1..=SIDE).map(move |column| Position::new(row, column)))
}

/// Whether `digit` is one a cell may hold: from 1 to [`SIDE`].
pub(crate) const fn in_range(digit: u8) -> bool {
    1 <= digit && digit <= SIDE
}

#[cfg(test)]
mod tests {
    use super::fixture::are_peers;
    use super::{BOX_SIDE, Given, Position, SIDE};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    #[test]
    fn box_side_is_three_and_side_is_its_square() {
        assert_eq!(BOX_SIDE, 3);
        assert_eq!(SIDE, 9);
        assert_eq!(SIDE, BOX_SIDE * BOX_SIDE);
    }

    #[test]
    fn positions_are_equal_when_their_fields_are() {
        assert_eq!(Position::new(4, 7), Position::new(4, 7));
        assert_ne!(Position::new(4, 7), Position::new(5, 7));
        assert_ne!(Position::new(4, 7), Position::new(4, 8));
        assert_eq!(Position::new(4, 7).row(), 4);
        assert_eq!(Position::new(4, 7).column(), 7);
    }

    #[test]
    fn givens_are_equal_when_their_fields_are() {
        let here = Position::new(1, 2);
        assert_eq!(Given::new(here, 3), Given::new(here, 3));
        assert_ne!(Given::new(here, 3), Given::new(here, 4));
        assert_ne!(Given::new(here, 3), Given::new(Position::new(2, 1), 3));
        assert_eq!(Given::new(here, 3).position(), here);
        assert_eq!(Given::new(here, 3).digit(), 3);
    }

    #[test]
    fn two_givens_that_agree_are_one_given() {
        let here = Position::new(1, 2);
        let givens: BTreeSet<Given> = [Given::new(here, 3), Given::new(here, 3)].into();
        assert_eq!(givens.len(), 1);
    }

    #[test]
    fn a_cell_has_twenty_peers_and_is_not_among_them() {
        for cell in super::grid_positions() {
            let peers: Vec<Position> = super::grid_positions()
                .filter(|&other| cell.is_peer_of(other))
                .collect();
            assert_eq!(peers.len(), 20);
            assert!(!peers.contains(&cell));
            assert!(peers.iter().all(|&peer| are_peers(cell, peer)));
        }
    }
}
