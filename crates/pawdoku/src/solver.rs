//! The solver: the verdict on a set of givens, which is none, one or many solutions,
//! and what those solutions are.
//!
//! This module is `docs/specs/solver.allium` in Rust. It stands on [`sudoku`](crate::sudoku)
//! alone and is the search that module leaves out: `sudoku` admits only givens with
//! exactly one solution and cannot count solutions itself.
//!
//! There are two entries, and one solver behind both. Neither takes a budget, because
//! every search concludes, and neither draws on randomness, because nothing in a search
//! is left to chance.
//!
//! - [`search`] gives any givens their verdict, with the solutions found and the count
//!   of guesses. It never fails. Call it to learn about givens: whether they can be
//!   solved, whether the answer is open, and how much guessing it took.
//! - [`solve`] is built on it and gives the proof a puzzle is set from, or a refusal.
//!   Call it to set a puzzle: `Puzzle::set(solve(givens)?)`.
//!
//! # How the search goes
//!
//! Propagation runs out before anything is guessed. A placed digit leaves its peers'
//! candidates, a cell with one candidate takes it, and a digit with one place left in a
//! unit is placed there. A contradiction ends a branch at once: a cell with no
//! candidate, a unit with no place for a digit, or a cell that is the only place for
//! two digits. A branch that settles with cells still empty is split on a cell with
//! the fewest candidates, into one child for each candidate. The deepest waiting branch
//! is taken first, and the search stops at the second solution.
//!
//! # The tie-break
//!
//! The specification leaves two orders to the implementation and asks only that each be
//! the same every time. This is the rule here:
//!
//! - **Tied cells.** Among the empty cells with the fewest candidates, the split is on
//!   the first in grid order: the topmost row, and in that row the leftmost column.
//! - **Tied branches.** Among the children of one split, the one that guessed the
//!   lowest digit is taken first.
//!
//! The verdict does not depend on the rule, and nor does a puzzle's one solution. Two
//! things do: [`SearchResult::guesses`] for givens that need a guess, and which two
//! solutions are found, and in which order, when there are many. A change to the rule
//! changes those for every consumer that recorded them.
//!
//! **Not final.** This is a draft: it may change as implementation continues.
//!
//! ```
//! use pawdoku::solver::{Verdict, search, solve};
//! use pawdoku::sudoku::{Given, Position, Puzzle};
//!
//! // Thirty givens, rows top to bottom, a dot for an empty cell.
//! let givens: Vec<Given> =
//!     "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!         .bytes()
//!         .zip(0_u8..)
//!         .filter(|(cell, _)| cell.is_ascii_digit())
//!         .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
//!         .collect();
//!
//! let result = search(givens.clone());
//! assert_eq!(result.verdict(), Verdict::One);
//!
//! let puzzle = Puzzle::set(solve(givens)?);
//! assert_eq!(puzzle.givens().len(), 30);
//! # Ok::<(), pawdoku::solver::SolveError>(())
//! ```

mod branch;
mod candidates;
mod result;
mod search;

pub use result::{SearchResult, Verdict};

use crate::sudoku::{Given, WellPosed, WellPosedError};
use alloc::collections::BTreeSet;

/// Why [`solve`] gave no proof: the givens are not ones a puzzle can be set from.
///
/// The `Display` text of each variant is stable API: bindings build their exceptions
/// from it.
///
/// ```
/// use pawdoku::solver::{SolveError, solve};
/// use pawdoku::sudoku::{Given, Position};
///
/// let fives = [1, 2].map(|column| Given::new(Position::new(1, column), 5));
/// let refusal = solve(fives).unwrap_err();
/// assert_eq!(refusal, SolveError::NoSolution);
/// assert_eq!(refusal.to_string(), "the givens have no solution");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SolveError {
    /// The verdict is none: the givens are malformed, they conflict, or nothing
    /// completes them.
    #[error("the givens have no solution")]
    NoSolution,
    /// The verdict is many: the givens leave the answer open.
    #[error("the givens have more than one solution")]
    SeveralSolutions,
    /// The givens have one solution and the proof's constructor still refused them.
    ///
    /// For any givens a caller can hand over, the reason is
    /// [`WellPosedError::NothingLeftToPlay`]: eighty-one givens that agree with a valid
    /// grid have one solution and leave the player nothing to do. The constructor's
    /// other refusals are about a solution that is wrong, which a search does not
    /// produce; they arrive here too, by the same conversion, so that a defect in the
    /// search is a refusal and never a panic.
    #[error(transparent)]
    NotPosed(#[from] WellPosedError),
}

/// Searches for the solutions of `givens` and gives the verdict: `BeginSearch`, and
/// every rule it sets off, run to the search's conclusion.
///
/// Nothing is required of the givens. They may be empty, conflict, or name positions
/// the grid does not have; the worst of them get [`Verdict::None`]. They are read as a
/// set, so the order they come in does not matter and a given handed over twice is one
/// given.
///
/// The result is the same every time for the same givens. The search stops at the
/// second solution and never looks for a third.
///
/// ```
/// use pawdoku::solver::{Verdict, search};
/// use pawdoku::sudoku::{Given, Position};
///
/// let givens: Vec<Given> =
///     "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
///         .bytes()
///         .zip(0_u8..)
///         .filter(|(cell, _)| cell.is_ascii_digit())
///         .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
///         .collect();
///
/// // Singles alone solve this one: nothing is guessed.
/// let result = search(givens.iter().copied());
/// assert_eq!(result.verdict(), Verdict::One);
/// assert_eq!(result.guesses(), 0);
/// assert_eq!(result.solutions()[0][0], [5, 3, 4, 6, 7, 8, 9, 1, 2]);
///
/// // A digit that is not from 1 to 9 has no solution, and nothing is searched.
/// let malformed = givens.into_iter().chain([Given::new(Position::new(9, 1), 0)]);
/// assert_eq!(search(malformed).verdict(), Verdict::None);
/// ```
#[must_use]
pub fn search(givens: impl IntoIterator<Item = Given>) -> SearchResult {
    search::run(&givens.into_iter().collect())
}

/// Solves `givens` and gives the proof that they are well-posed, which is what a
/// [`Puzzle`](crate::sudoku::Puzzle) is set from.
///
/// The proof holds the givens and their one solution. This is the only place a proof
/// is made, so a proof in hand means the solver found exactly one solution.
///
/// # Errors
///
/// Three refusals, and no other outcome for any givens:
///
/// - [`SolveError::NoSolution`] when the givens have no solution;
/// - [`SolveError::SeveralSolutions`] when they have more than one;
/// - [`SolveError::NotPosed`] when they have one and leave nothing to play, because
///   every cell is given.
///
/// ```
/// use pawdoku::solver::{SolveError, solve};
/// use pawdoku::sudoku::{Given, Position, WellPosedError};
///
/// let givens: Vec<Given> =
///     "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
///         .bytes()
///         .zip(0_u8..)
///         .filter(|(cell, _)| cell.is_ascii_digit())
///         .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
///         .collect();
///
/// let proof = solve(givens)?;
/// assert_eq!(proof.givens().len(), 30);
/// assert_eq!(proof.solution()[8], [3, 4, 5, 2, 8, 6, 1, 7, 9]);
///
/// // No givens at all have many solutions.
/// assert_eq!(solve([]), Err(SolveError::SeveralSolutions));
///
/// // The whole solution as givens has one solution, and nothing left to play.
/// let every_cell = proof.solution().iter().zip(1..).flat_map(|(row, r)| {
///     row.iter().zip(1..).map(move |(&digit, c)| Given::new(Position::new(r, c), digit))
/// });
/// assert_eq!(
///     solve(every_cell),
///     Err(SolveError::NotPosed(WellPosedError::NothingLeftToPlay { givens: 81 }))
/// );
/// # Ok::<(), SolveError>(())
/// ```
pub fn solve(givens: impl IntoIterator<Item = Given>) -> Result<WellPosed, SolveError> {
    let givens: BTreeSet<Given> = givens.into_iter().collect();
    match search::run(&givens).solutions() {
        [] => Err(SolveError::NoSolution),
        [solution] => Ok(WellPosed::vouch(givens, solution.map(|row| row.map(Some)))?),
        _ => Err(SolveError::SeveralSolutions),
    }
}

#[cfg(test)]
mod tests {
    use super::search::SOLUTIONS_SOUGHT;
    use super::{SolveError, search};
    use crate::sudoku::{Given, Position, WellPosedError};
    use alloc::string::ToString;

    #[test]
    fn two_solutions_are_sought() {
        assert_eq!(SOLUTIONS_SOUGHT, 2);
        assert_eq!(search([]).solutions().len(), SOLUTIONS_SOUGHT);
    }

    /// No givens reach the constructor's other errors through `solve`, since a search
    /// hands it a full, consistent solution that holds every given. The conversion is
    /// what carries them if a defect ever does, so it is tested by itself.
    #[test]
    fn the_refusal_converts_from_the_proofs_error() {
        let position = Position::new(2, 3);
        let given = Given::new(position, 7);
        for error in [
            WellPosedError::NothingLeftToPlay { givens: 81 },
            WellPosedError::SolutionNotFull { position },
            WellPosedError::SolutionDigitOutOfRange {
                position,
                digit: 10,
            },
            WellPosedError::SolutionConflict {
                first: position,
                second: Position::new(2, 8),
                digit: 4,
            },
            WellPosedError::GivenOffGrid { given },
            WellPosedError::GivenMismatch { given, solution: 2 },
        ] {
            let refusal = SolveError::from(error.clone());
            assert_eq!(refusal, SolveError::NotPosed(error.clone()));
            assert_eq!(refusal.to_string(), error.to_string());
        }
    }

    #[test]
    fn refusal_texts_are_stable() {
        let texts = [
            (SolveError::NoSolution, "the givens have no solution"),
            (
                SolveError::SeveralSolutions,
                "the givens have more than one solution",
            ),
            (
                SolveError::NotPosed(WellPosedError::NothingLeftToPlay { givens: 81 }),
                "81 givens leave nothing to play: a puzzle has fewer givens than its 81 cells",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }
}
