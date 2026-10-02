//! The search itself: branches taken deepest first from a stack until two solutions
//! are in or nothing waits.

use super::branch::{Branch, Decided};
use super::result::SearchResult;
use crate::sudoku::Given;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// One solution shows the givens can be solved and a second shows they are not
/// well-posed. No verdict needs a third, so none is looked for:
/// `config.solutions_sought`.
pub(super) const SOLUTIONS_SOUGHT: usize = 2;

/// Runs the search for `givens` to its conclusion.
///
/// The waiting branches are a stack. A split pushes its children on top, so the branch
/// taken next is always the deepest there is, and among a split's children the one
/// that guessed the lowest digit. Malformed givens open no branch at all. Whatever
/// still waits when the second solution arrives is left unopened.
pub(super) fn run(givens: &BTreeSet<Given>) -> SearchResult {
    let mut search = SearchResult {
        guesses: 0,
        solutions: Vec::new(),
    };
    let mut waiting: Vec<Branch> = Branch::root(givens).into_iter().collect();
    while search.solutions.len() < SOLUTIONS_SOUGHT
        && let Some(branch) = waiting.pop()
    {
        match branch.decide() {
            Decided::Contradicted => {}
            Decided::Solved(solution) => search.solutions.push(solution),
            Decided::Split(children) => {
                search.guesses += 1;
                waiting.extend(children.into_iter().rev());
            }
        }
    }
    search
}
