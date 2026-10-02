//! What the module's tests share: the fixture puzzle, checked by hand, and the helpers
//! that read a picture of a grid. This module cannot call the solver, so its tests
//! build the proof themselves.

use super::proof::WellPosed;
use super::{Given, Grid, Position, at, grid_positions};
use alloc::collections::BTreeSet;

/// The example puzzle of the article `sudoku.allium` cites, rows top to bottom, a
/// dot for an empty cell: 30 givens, one solution, checked by hand on 2026-09-30.
pub(super) const PUZZLE: [&str; 9] = [
    "53..7....",
    "6..195...",
    ".98....6.",
    "8...6...3",
    "4..8.3..1",
    "7...2...6",
    ".6....28.",
    "...419..5",
    "....8..79",
];

/// The one solution of [`PUZZLE`].
pub(super) const SOLUTION: [&str; 9] = [
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

/// Reads a picture of a grid: a digit for a filled cell and a dot for an empty one.
pub(super) fn grid(rows: [&str; 9]) -> Grid<Option<u8>> {
    rows.map(|row| {
        let mut cells = row
            .bytes()
            .map(|cell| cell.checked_sub(b'0').filter(|&d| d <= 9));
        core::array::from_fn(|_| cells.next().unwrap())
    })
}

/// The filled cells of a picture, as givens.
pub(super) fn givens(rows: [&str; 9]) -> BTreeSet<Given> {
    let cells = grid(rows);
    grid_positions()
        .filter_map(|position| Some(Given::new(position, at(&cells, position)??)))
        .collect()
}

/// The fixture's proof, built by hand: this module cannot call the solver.
pub(super) fn fixture() -> WellPosed {
    WellPosed::vouch(givens(PUZZLE), grid(SOLUTION)).unwrap()
}

/// [`SOLUTION`] with 1 and 2 exchanged throughout: another valid grid.
pub(super) const EXCHANGED: [&str; 9] = [
    "534678921",
    "671295348",
    "298341567",
    "859762413",
    "416853792",
    "723914856",
    "962537184",
    "187429635",
    "345186279",
];

/// Peers by the specification's words, with nothing of the implementation: another
/// position that shares a row, a column or a box.
pub(super) fn are_peers(a: Position, b: Position) -> bool {
    let same_box =
        (a.row() - 1) / 3 == (b.row() - 1) / 3 && (a.column() - 1) / 3 == (b.column() - 1) / 3;
    a != b && (a.row() == b.row() || a.column() == b.column() || same_box)
}
