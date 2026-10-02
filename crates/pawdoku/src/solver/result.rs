//! What a search has to show for itself: the `SearchResult` surface.

use crate::sudoku::Grid;
use alloc::vec::Vec;

/// How many solutions a set of givens has, as far as any verdict needs to know: none,
/// one, or more than one.
///
/// Givens are well-posed exactly when their verdict is [`Verdict::OneSolution`].
///
/// ```
/// use pawdoku::solver::{Verdict, search};
/// use pawdoku::sudoku::{Given, Position};
///
/// // No givens at all leave every grid open.
/// assert_eq!(search([]).verdict(), Verdict::ManySolutions);
///
/// // Two fives in one row cannot both stand.
/// let fives = [1, 2].map(|column| Given::new(Position::new(1, column), 5));
/// assert_eq!(search(fives).verdict(), Verdict::NoSolution);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Verdict {
    /// The givens have no solution: they are malformed, they conflict, or nothing
    /// completes them.
    NoSolution,
    /// The givens have exactly one solution, which the result holds.
    OneSolution,
    /// The givens have two solutions or more. The result holds two of them and the
    /// search looked for no third.
    ManySolutions,
}

/// A search that has concluded: its verdict, the solutions it found and the guesses it
/// made.
///
/// This is everything the `SearchResult` surface of `solver.allium` exposes. The
/// surface also exposes the search's status, and a result has no field and no accessor
/// for it: [`search`](crate::solver::search) hands a result over only once the search
/// has concluded, so, in the specification's words, a caller "reads the status from
/// that alone: being handed it says concluded".
///
/// ```
/// use pawdoku::solver::{Verdict, search};
/// use pawdoku::sudoku::{Given, Position};
///
/// let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
///     .bytes()
///     .zip(0_u8..)
///     .filter(|(cell, _)| cell.is_ascii_digit())
///     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
///
/// let result = search(givens);
/// assert_eq!(result.verdict(), Verdict::OneSolution);
/// assert_eq!(result.guesses(), 0);
/// assert_eq!(result.solutions().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SearchResult {
    pub(super) guesses: u32,
    pub(super) solutions: Vec<Grid<u8>>,
}

impl SearchResult {
    /// The verdict on the givens, read from how many solutions the search found: none,
    /// one, or the two it stops at.
    ///
    /// ```
    /// use pawdoku::solver::{Verdict, search};
    ///
    /// assert_eq!(search([]).verdict(), Verdict::ManySolutions);
    /// ```
    #[must_use]
    pub const fn verdict(&self) -> Verdict {
        match self.solutions.as_slice() {
            [] => Verdict::NoSolution,
            [_] => Verdict::OneSolution,
            _ => Verdict::ManySolutions,
        }
    }

    /// How many times the search split a branch on a guess: the work it did, as far as
    /// it can be seen from outside. Zero when propagation alone decided the givens.
    ///
    /// The same givens always get the same count. Which count that is, for givens that
    /// need a guess, follows from the order tied cells and tied digits are taken in,
    /// which the [module](crate::solver) states.
    ///
    /// ```
    /// use pawdoku::solver::search;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // A conflict is found out by propagation, with nothing guessed.
    /// let fives = [1, 2].map(|column| Given::new(Position::new(1, column), 5));
    /// assert_eq!(search(fives).guesses(), 0);
    ///
    /// // An empty grid cannot be filled without guessing.
    /// assert!(search([]).guesses() > 0);
    /// ```
    #[must_use]
    pub const fn guesses(&self) -> u32 {
        self.guesses
    }

    /// The solutions the search found, in the order it found them: none, one or two,
    /// and never more. Each is a full grid, rows top to bottom and in each row the
    /// columns left to right, so the digit at row `r`, column `c` is
    /// `solution[r - 1][c - 1]`.
    ///
    /// ```
    /// use pawdoku::solver::search;
    ///
    /// let result = search([]);
    /// let [first, second] = result.solutions() else {
    ///     panic!("an empty grid has more than one solution");
    /// };
    /// assert_ne!(first, second);
    /// assert!(first.iter().flatten().all(|digit| (1..=9).contains(digit)));
    /// ```
    #[must_use]
    pub fn solutions(&self) -> &[Grid<u8>] {
        &self.solutions
    }
}
