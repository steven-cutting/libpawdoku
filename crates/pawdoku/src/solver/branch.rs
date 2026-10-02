//! One branch of a search: a grid of candidates, the rules that propagate through it,
//! and what the search decides of it once they have run out.

use super::candidates::Candidates;
use crate::sudoku::{BOX_SIDE, Given, Grid, LINE, Position, SIDE, in_range};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// The cells of the grid, counted a row at a time from the top left.
const CELLS: usize = LINE * LINE;

/// The side of a box as a length.
const BOX: usize = BOX_SIDE as usize;

/// The kinds of unit a cell is in: its row, its column and its box.
const KINDS: usize = 3;

/// The cells of every unit, each cell as its place in the grid: the nine rows, then
/// the nine columns, then the nine boxes.
const UNITS: [[usize; LINE]; KINDS * LINE] = units();

/// The other cells of a cell's row, of its column and of its box: the specification's
/// `row_mates`, `column_mates` and `box_mates`.
type Mates = [[usize; LINE - 1]; KINDS];

/// Every cell's mates, in grid order.
const MATES: [Mates; CELLS] = mates();

/// The row, the column and the box a cell is in, as places in [`UNITS`].
const fn units_of(cell: usize) -> [usize; KINDS] {
    let (row, column) = (cell / LINE, cell % LINE);
    let boxed = row / BOX * BOX + column / BOX;
    [row, LINE + column, 2 * LINE + boxed]
}

// The two tables are built when the crate is compiled, so their indexing cannot fail
// in a running search.
const fn units() -> [[usize; LINE]; KINDS * LINE] {
    let mut units = [[0; LINE]; KINDS * LINE];
    let mut cell = 0;
    while cell < CELLS {
        let (row, column) = (cell / LINE, cell % LINE);
        let [in_row, in_column, in_box] = units_of(cell);
        units[in_row][column] = cell;
        units[in_column][row] = cell;
        units[in_box][row % BOX * BOX + column % BOX] = cell;
        cell += 1;
    }
    units
}

const fn mates() -> [Mates; CELLS] {
    let mut mates = [[[0; LINE - 1]; KINDS]; CELLS];
    let mut next = 0;
    while next < CELLS * KINDS {
        let (cell, kind) = (next / KINDS, next % KINDS);
        let unit = UNITS[units_of(cell)[kind]];
        let (mut from, mut to) = (0, 0);
        while from < LINE {
            if unit[from] != cell {
                mates[cell][kind][to] = unit[from];
                to += 1;
            }
            from += 1;
        }
        next += 1;
    }
    mates
}

/// Where a position's cell is in grid order, or nothing where the grid has no cell.
fn place_of(position: Position) -> Option<usize> {
    let on_grid = |line: u8| (1..=SIDE).contains(&line).then(|| usize::from(line - 1));
    Some(on_grid(position.row())? * LINE + on_grid(position.column())?)
}

/// One cell of a branch: the specification's `BranchCell`, without its position, which
/// is its place in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    /// The digits the cell may still hold. A placed cell keeps its digit as its one
    /// candidate, and loses even that if a peer is placed with it.
    candidates: Candidates,
    /// Absent until the cell is placed: by a given, a single or a guess.
    digit: Option<u8>,
}

impl Cell {
    /// A cell that may hold any digit.
    const OPEN: Self = Self {
        candidates: Candidates::ALL,
        digit: None,
    };

    /// A cell placed with `digit`, its one candidate.
    fn placed(digit: u8) -> Self {
        Self {
            candidates: Candidates::only(digit),
            digit: Some(digit),
        }
    }
}

/// What the search decides of a branch once it is taken up: `DecideBranch`.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Decided {
    /// A contradiction ended the branch, and nothing is guessed beneath it.
    Contradicted,
    /// Propagation ran out with every cell placed: the grid is a solution.
    Solved(Grid<u8>),
    /// Propagation ran out with cells left to place. The children, one for each
    /// candidate of the cell guessed on, lowest digit first.
    Split(Vec<Branch>),
}

/// One grid of candidates. A guess makes a child for each candidate of a cell and
/// changes nothing in the parent, so nothing is ever undone.
///
/// A branch here is its cells and no more. Its status is where it is: waiting on the
/// search's stack, working inside [`Branch::decide`], or ended in what that returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Branch {
    cells: [Cell; CELLS],
}

impl Branch {
    /// `LayOutRootBranch`: every given's cell placed and every other cell open. Nothing
    /// for givens that are not well-formed, which `RefuseMalformedGivens` refuses: a
    /// position off the grid, a digit out of range, or two digits to one position.
    pub(super) fn root(givens: &BTreeSet<Given>) -> Option<Self> {
        let mut cells = [Cell::OPEN; CELLS];
        for given in givens {
            let cell = cells.get_mut(place_of(given.position())?)?;
            let digit = Some(given.digit()).filter(|&digit| in_range(digit))?;
            if cell.digit.is_some_and(|held| held != digit) {
                return None;
            }
            *cell = Cell::placed(digit);
        }
        Some(Self { cells })
    }

    /// Works the branch to its end: propagates until nothing is left, stopping at once
    /// on a contradiction, and then reads the settled grid as a solution or splits it.
    pub(super) fn decide(mut self) -> Decided {
        while !self.is_contradictory() {
            if !self.propagate() {
                return self
                    .solution()
                    .map_or_else(|| Decided::Split(self.children()), Decided::Solved);
            }
        }
        Decided::Contradicted
    }

    /// Whether some cell is contradictory, in any of the three ways the specification
    /// reads: no candidate, a unit with no place for a digit, or two digits with no
    /// other place.
    fn is_contradictory(&self) -> bool {
        self.has_cell_without_candidate()
            || self.has_unit_without_place()
            || self.has_overdemanded_cell()
    }

    fn has_cell_without_candidate(&self) -> bool {
        self.cells.iter().any(|cell| cell.candidates.is_empty())
    }

    /// Whether some unit has a digit none of its cells can hold.
    fn has_unit_without_place(&self) -> bool {
        UNITS
            .iter()
            .any(|unit| self.candidates_among(unit) != Candidates::ALL)
    }

    /// Whether some cell is the only place for two digits.
    fn has_overdemanded_cell(&self) -> bool {
        let mut cells = self.cells.iter().zip(&MATES);
        cells.any(|(&cell, mates)| self.sole_places(cell, mates).count() > 1)
    }

    /// One pass of each propagation rule over every cell. Says whether anything
    /// changed: a branch that nothing changes in has settled.
    fn propagate(&mut self) -> bool {
        let rules = [
            Self::without_stale_candidates,
            Self::with_naked_single_placed,
            Self::with_hidden_single_placed,
        ];
        let mut changed = false;
        for rule in rules {
            changed |= self.rewrite(rule);
        }
        changed
    }

    /// Rewrites every cell by `rule`, each read from the branch as it stood before the
    /// pass, and says whether any cell changed.
    fn rewrite(&mut self, rule: fn(&Self, Cell, &Mates) -> Cell) -> bool {
        let before = self.clone();
        for (cell, mates) in self.cells.iter_mut().zip(&MATES) {
            *cell = rule(&before, *cell, mates);
        }
        *self != before
    }

    /// `EliminateFromPeers`: the cell without the digits its peers are placed with.
    fn without_stale_candidates(&self, cell: Cell, mates: &Mates) -> Cell {
        let candidates = cell.candidates.without(self.placed_digits(mates));
        Cell { candidates, ..cell }
    }

    /// `PlaceNakedSingle`: an unplaced cell with one candidate takes it.
    fn with_naked_single_placed(_: &Self, cell: Cell, _: &Mates) -> Cell {
        let single = cell.candidates.single().filter(|_| cell.digit.is_none());
        single.map_or(cell, Cell::placed)
    }

    /// `PlaceHiddenSingle`: an unplaced cell that is the only place for one digit, and
    /// for no second, takes it, whatever else the cell could have held.
    fn with_hidden_single_placed(&self, cell: Cell, mates: &Mates) -> Cell {
        let sole = self.sole_places(cell, mates);
        let single = sole.single().filter(|_| cell.digit.is_none());
        single.map_or(cell, Cell::placed)
    }

    /// The digits a cell's mates are placed with: `placed_peer_digits`.
    fn placed_digits(&self, mates: &Mates) -> Candidates {
        let cells = mates.iter().flatten().filter_map(|&at| self.cells.get(at));
        let digits = cells.filter_map(|cell| cell.digit).map(Candidates::only);
        digits.fold(Candidates::NONE, Candidates::with)
    }

    /// Every digit some cell at `places` may still hold.
    fn candidates_among(&self, places: &[usize]) -> Candidates {
        let cells = places.iter().filter_map(|&at| self.cells.get(at));
        cells.fold(Candidates::NONE, |all, cell| all.with(cell.candidates))
    }

    /// The candidates of `cell` that no other cell of one of its units can hold: the
    /// digits for which `is_sole_place` is true.
    fn sole_places(&self, cell: Cell, mates: &Mates) -> Candidates {
        let elsewhere = mates.iter().map(|unit| self.candidates_among(unit));
        let sole = elsewhere.map(|others| cell.candidates.without(others));
        sole.fold(Candidates::NONE, Candidates::with)
    }

    /// The grid's digits when every cell is placed, and nothing otherwise. Of a
    /// settled branch this is a solution: no two peers hold one digit.
    fn solution(&self) -> Option<Grid<u8>> {
        let mut digits = [[0; LINE]; LINE];
        for (digit, cell) in digits.iter_mut().flatten().zip(&self.cells) {
            *digit = cell.digit?;
        }
        Some(digits)
    }

    /// `LayOutChildBranch` for each candidate of the cell the branch is split on: the
    /// first unplaced cell, in grid order, with the fewest candidates. Lowest digit
    /// first.
    fn children(&self) -> Vec<Self> {
        let cells = self.cells.iter().enumerate();
        let unplaced = cells.filter(|(_, cell)| cell.digit.is_none());
        let fewest = unplaced.min_by_key(|(_, cell)| cell.candidates.count());
        let guesses = fewest
            .into_iter()
            .flat_map(|(at, cell)| cell.candidates.digits().map(move |digit| (at, digit)));
        guesses.map(|(at, digit)| self.child(at, digit)).collect()
    }

    /// The branch with `digit` placed in the cell at `at`: a guess.
    fn child(&self, at: usize, digit: u8) -> Self {
        let mut child = self.clone();
        if let Some(cell) = child.cells.get_mut(at) {
            *cell = Cell::placed(digit);
        }
        child
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Branch, CELLS, Candidates, Cell, Decided, MATES, Mates, UNITS, mates, place_of, units,
    };
    use crate::sudoku::{Given, Position};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;
    use proptest::prelude::*;

    /// The rules' fixture: thirty givens, one solution.
    const FIXTURE: [&str; 9] = [
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

    /// The one solution of [`FIXTURE`].
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

    /// Norvig's hard puzzle, which singles cannot solve; `tests/solver.rs` cites it.
    const NEEDS_A_GUESS: [&str; 9] = [
        "4.....8.5",
        ".3.......",
        "...7.....",
        ".2.....6.",
        "....8.4..",
        "....1....",
        "...6.3.7.",
        "5..2.....",
        "1.4......",
    ];

    /// Row 1, column 1 has no candidate: 1 to 3 in its row, 4 to 6 in its column and
    /// 7 to 9 in its box.
    const NO_CANDIDATE: [&str; 9] = [
        "...123...",
        ".78......",
        ".9.......",
        "4........",
        "5........",
        "6........",
        ".........",
        ".........",
        ".........",
    ];

    /// Row 1 has no place for a 1, and every cell has a candidate.
    const NO_PLACE: [&str; 9] = [
        "........2",
        "1........",
        "...1.....",
        "......1..",
        ".........",
        ".........",
        ".......1.",
        ".........",
        ".........",
    ];

    /// Row 1, column 1 is the only place for a 1 in its row and for a 2 in its column.
    const OVERDEMANDED: [&str; 9] = [
        ".........",
        "....1.2..",
        "...2...1.",
        ".1.......",
        "..2......",
        ".........",
        "..1......",
        ".2.......",
        ".........",
    ];

    /// Row 1, column 1 is the only place for a 1 in its row, and in no other unit: the
    /// rest of the row is filled or sees a 1 in its column.
    const SOLE_IN_ROW: [&str; 9] = [
        "...234567",
        ".........",
        ".........",
        ".1.......",
        ".........",
        ".........",
        "..1......",
        ".........",
        ".........",
    ];

    /// [`SOLE_IN_ROW`] turned on its diagonal: the only place for a 1 in column 1.
    const SOLE_IN_COLUMN: [&str; 9] = [
        ".........",
        "...1.....",
        "......1..",
        "2........",
        "3........",
        "4........",
        "5........",
        "6........",
        "7........",
    ];

    /// Row 1, column 1 is the only place for a 1 in its box, and in no other unit: the
    /// rest of the box is filled or sees the 1 in row 2.
    const SOLE_IN_BOX: [&str; 9] = [
        ".56......",
        "........1",
        "234......",
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
        ".........",
    ];

    fn givens(rows: [&str; 9]) -> BTreeSet<Given> {
        let cell = |row: u8, column: u8, cell: u8| {
            cell.is_ascii_digit()
                .then(|| Given::new(Position::new(row, column), cell - b'0'))
        };
        let row = |(row, line): (u8, &str)| {
            let cells = (1..).zip(line.bytes());
            cells
                .filter_map(move |(column, held)| cell(row, column, held))
                .collect::<Vec<_>>()
        };
        (1..).zip(rows).flat_map(row).collect()
    }

    fn root(rows: [&str; 9]) -> Branch {
        Branch::root(&givens(rows)).unwrap()
    }

    /// The root of a picture with `EliminateFromPeers` run out and nothing placed.
    fn eliminated(rows: [&str; 9]) -> Branch {
        let mut branch = root(rows);
        while branch.rewrite(Branch::without_stale_candidates) {}
        branch
    }

    fn at(branch: &Branch, row: u8, column: u8) -> Cell {
        branch.cells[place_of(Position::new(row, column)).unwrap()]
    }

    fn digits(candidates: Candidates) -> Vec<u8> {
        candidates.digits().collect()
    }

    fn sole_places(branch: &Branch, row: u8, column: u8) -> Vec<u8> {
        let place = place_of(Position::new(row, column)).unwrap();
        digits(branch.sole_places(branch.cells[place], &MATES[place]))
    }

    /// Peers by the specification's words: another cell sharing a row, a column or a box.
    fn are_peers(a: usize, b: usize) -> bool {
        let same_box = row_of(a) / 3 == row_of(b) / 3 && column_of(a) / 3 == column_of(b) / 3;
        a != b && (row_of(a) == row_of(b) || column_of(a) == column_of(b) || same_box)
    }

    fn row_of(cell: usize) -> usize {
        cell / 9
    }

    fn column_of(cell: usize) -> usize {
        cell % 9
    }

    #[test]
    fn a_position_has_a_place_exactly_when_it_is_on_the_grid() {
        assert_eq!(place_of(Position::new(1, 1)), Some(0));
        assert_eq!(place_of(Position::new(1, 9)), Some(8));
        assert_eq!(place_of(Position::new(2, 1)), Some(9));
        assert_eq!(place_of(Position::new(9, 9)), Some(80));
        for off in [(0, 1), (1, 0), (10, 1), (1, 10), (u8::MAX, u8::MAX)] {
            assert_eq!(place_of(Position::new(off.0, off.1)), None);
        }
    }

    #[test]
    fn the_units_are_nine_rows_nine_columns_and_nine_boxes() {
        // The tables are built when the crate is compiled; building them again here
        // shows the same functions give the same tables in a running program.
        assert_eq!(units(), UNITS);
        assert_eq!(mates(), MATES);
        for (index, unit) in UNITS.iter().enumerate() {
            let distinct: BTreeSet<usize> = unit.iter().copied().collect();
            assert_eq!(distinct.len(), 9);
            let rows: BTreeSet<usize> = unit.iter().map(|&cell| row_of(cell)).collect();
            let columns: BTreeSet<usize> = unit.iter().map(|&cell| column_of(cell)).collect();
            let expected = match index / 9 {
                0 => (1, 9),
                1 => (9, 1),
                _ => (3, 3),
            };
            assert_eq!((rows.len(), columns.len()), expected);
        }
        for cell in 0..CELLS {
            assert_eq!(UNITS.iter().filter(|unit| unit.contains(&cell)).count(), 3);
        }
    }

    #[test]
    fn a_cells_mates_are_its_twenty_peers_by_row_column_and_box() {
        for (cell, mates) in MATES.iter().enumerate() {
            let [row, column, boxed]: &Mates = mates;
            assert!(row.iter().all(|&mate| row_of(mate) == row_of(cell)));
            assert!(
                column
                    .iter()
                    .all(|&mate| column_of(mate) == column_of(cell))
            );
            assert!(boxed.iter().all(|&mate| are_peers(cell, mate)));
            let peers: BTreeSet<usize> = mates.iter().flatten().copied().collect();
            let expected: BTreeSet<usize> =
                (0..CELLS).filter(|&other| are_peers(cell, other)).collect();
            assert_eq!(peers.len(), 20);
            assert_eq!(peers, expected);
        }
    }

    #[test]
    fn the_root_holds_each_given_placed_and_every_other_cell_open() {
        let branch = root(FIXTURE);
        let given: BTreeSet<Given> = givens(FIXTURE);
        for row in 1..=9 {
            for column in 1..=9 {
                let position = Position::new(row, column);
                let cell = at(&branch, row, column);
                match given.iter().find(|given| given.position() == position) {
                    Some(given) => assert_eq!(cell, Cell::placed(given.digit())),
                    None => assert_eq!(cell, Cell::OPEN),
                }
            }
        }
        assert_eq!(digits(at(&branch, 1, 1).candidates), [5]);
        assert_eq!(at(&branch, 1, 1).digit, Some(5));
        assert_eq!(
            digits(at(&branch, 1, 3).candidates),
            [1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert_eq!(at(&branch, 1, 3).digit, None);
    }

    #[test]
    fn no_root_is_opened_for_givens_that_are_not_well_formed() {
        let here = Position::new(4, 4);
        let malformed = [
            [Given::new(Position::new(0, 4), 5)],
            [Given::new(Position::new(4, 10), 5)],
            [Given::new(here, 0)],
            [Given::new(here, 10)],
        ];
        for givens in malformed {
            assert_eq!(Branch::root(&givens.into()), None);
        }
        let two_digits = [Given::new(here, 5), Given::new(here, 6)];
        assert_eq!(Branch::root(&two_digits.into()), None);
        // Givens in conflict are well-formed: propagation finds them out.
        let conflicting = [Given::new(here, 5), Given::new(Position::new(4, 5), 5)];
        assert!(Branch::root(&conflicting.into()).is_some());
        assert!(Branch::root(&BTreeSet::new()).is_some());
    }

    #[test]
    fn a_placed_digit_leaves_its_peers_candidates() {
        let lone = [Given::new(Position::new(5, 5), 7)];
        let mut branch = Branch::root(&lone.into()).unwrap();
        assert!(branch.rewrite(Branch::without_stale_candidates));
        let centre = place_of(Position::new(5, 5)).unwrap();
        for (place, cell) in branch.cells.iter().enumerate() {
            let expected = match place {
                _ if place == centre => Candidates::only(7),
                _ if are_peers(place, centre) => Candidates::ALL.without(Candidates::only(7)),
                _ => Candidates::ALL,
            };
            assert_eq!(cell.candidates, expected, "cell {place}");
        }
        // Nothing is stale now, so a second pass changes nothing.
        assert!(!branch.rewrite(Branch::without_stale_candidates));
    }

    #[test]
    fn two_peers_placed_with_one_digit_each_lose_it() {
        for second in [(1, 9), (9, 1), (2, 2)] {
            let conflicting =
                [(1, 1), second].map(|(row, column)| Given::new(Position::new(row, column), 5));
            let mut branch = Branch::root(&conflicting.into()).unwrap();
            assert!(!branch.is_contradictory());
            branch.rewrite(Branch::without_stale_candidates);
            assert!(at(&branch, 1, 1).candidates.is_empty());
            assert!(at(&branch, second.0, second.1).candidates.is_empty());
            assert_eq!(at(&branch, 1, 1).digit, Some(5));
            assert!(branch.has_cell_without_candidate());
            assert!(matches!(
                Branch::root(&conflicting.into()).unwrap().decide(),
                Decided::Contradicted
            ));
        }
    }

    #[test]
    fn a_cell_with_one_candidate_takes_it() {
        // Row 1, column 1 sees 1 to 3, 4 to 6, and 7 and 8: only 9 is left to it.
        let rows = [
            "...123...",
            ".78......",
            ".........",
            "4........",
            "5........",
            "6........",
            ".........",
            ".........",
            ".........",
        ];
        let mut branch = eliminated(rows);
        assert_eq!(digits(at(&branch, 1, 1).candidates), [9]);
        assert_eq!(at(&branch, 1, 1).digit, None);
        // Its row, its column and its box each have another place for a 9, so the cell
        // is no hidden single: only `PlaceNakedSingle` places it.
        assert!(sole_places(&branch, 1, 1).is_empty());
        assert!(branch.rewrite(Branch::with_naked_single_placed));
        assert_eq!(at(&branch, 1, 1), Cell::placed(9));
        // A placed cell is not a naked single: a second pass has nothing to do.
        assert!(!branch.rewrite(Branch::with_naked_single_placed));
    }

    #[test]
    fn a_digit_with_one_place_in_a_unit_is_placed_there() {
        for rows in [SOLE_IN_ROW, SOLE_IN_COLUMN, SOLE_IN_BOX] {
            let mut branch = eliminated(rows);
            let before = at(&branch, 1, 1);
            // The cell has other candidates, so it is no naked single.
            assert!(before.candidates.count() > 1);
            assert_eq!(before.digit, None);
            assert!(!branch.clone().rewrite(Branch::with_naked_single_placed));
            assert_eq!(sole_places(&branch, 1, 1), [1]);
            assert!(branch.rewrite(Branch::with_hidden_single_placed));
            assert_eq!(at(&branch, 1, 1), Cell::placed(1));
        }
    }

    #[test]
    fn a_digit_is_the_sole_place_in_the_unit_the_picture_names() {
        let others_with = |branch: &Branch, unit: usize| {
            let mates = &MATES[0][unit];
            branch.candidates_among(mates).contains(1)
        };
        // Which of row, column and box still has another place for the 1.
        let expected = [
            (SOLE_IN_ROW, [false, true, true]),
            (SOLE_IN_COLUMN, [true, false, true]),
            (SOLE_IN_BOX, [true, true, false]),
        ];
        for (rows, elsewhere) in expected {
            let branch = eliminated(rows);
            assert_eq!([0, 1, 2].map(|unit| others_with(&branch, unit)), elsewhere);
        }
    }

    #[test]
    fn each_kind_of_contradiction_is_read_from_the_cells() {
        let kinds = |branch: &Branch| {
            [
                branch.has_cell_without_candidate(),
                branch.has_unit_without_place(),
                branch.has_overdemanded_cell(),
            ]
        };
        // Before anything is struck no cell is contradictory.
        for rows in [NO_CANDIDATE, NO_PLACE, OVERDEMANDED, FIXTURE] {
            assert!(!root(rows).is_contradictory());
        }
        assert_eq!(kinds(&eliminated(NO_CANDIDATE)), [true, false, false]);
        assert_eq!(kinds(&eliminated(NO_PLACE)), [false, true, false]);
        assert_eq!(kinds(&eliminated(OVERDEMANDED)), [false, false, true]);
        assert_eq!(kinds(&eliminated(FIXTURE)), [false, false, false]);
        for rows in [NO_CANDIDATE, NO_PLACE, OVERDEMANDED] {
            assert!(eliminated(rows).is_contradictory());
            assert_eq!(root(rows).decide(), Decided::Contradicted);
        }
    }

    #[test]
    fn a_cell_that_is_the_only_place_for_two_digits_is_not_placed() {
        let mut branch = eliminated(OVERDEMANDED);
        assert_eq!(sole_places(&branch, 1, 1), [1, 2]);
        branch.rewrite(Branch::with_hidden_single_placed);
        assert_eq!(at(&branch, 1, 1).digit, None);
    }

    /// The root of a picture, propagated until it settles or is contradictory, as the
    /// search does inside `decide`.
    fn settled(rows: [&str; 9]) -> Branch {
        let mut branch = root(rows);
        while !branch.is_contradictory() && branch.propagate() {}
        branch
    }

    #[test]
    fn singles_alone_settle_the_fixture_into_its_solution() {
        let branch = settled(FIXTURE);
        assert!(!branch.is_contradictory());
        assert_eq!(branch, root(SOLUTION));
        assert!(branch.children().is_empty());
        let solution = root(SOLUTION).solution().unwrap();
        assert_eq!(solution[0], [5, 3, 4, 6, 7, 8, 9, 1, 2]);
        assert_eq!(root(FIXTURE).decide(), Decided::Solved(solution));
        assert_eq!(root(FIXTURE).solution(), None);
    }

    /// The cell a child guessed on and the digit it guessed: where it parts from its
    /// parent.
    fn guess(parent: &Branch, child: &Branch) -> (usize, u8) {
        let cells = parent.cells.iter().zip(&child.cells).enumerate();
        let changed: Vec<(usize, u8)> = cells
            .filter(|(_, (before, after))| before != after)
            .filter_map(|(place, (_, after))| Some((place, after.digit?)))
            .collect();
        assert_eq!(changed.len(), 1);
        changed[0]
    }

    #[test]
    fn a_branch_is_split_on_the_first_cell_with_the_fewest_candidates() {
        let parent = settled(NEEDS_A_GUESS);
        assert!(!parent.is_contradictory());
        let unplaced = || {
            parent
                .cells
                .iter()
                .enumerate()
                .filter(|(_, cell)| cell.digit.is_none())
        };
        let fewest = unplaced()
            .map(|(_, cell)| cell.candidates.count())
            .min()
            .unwrap();
        let (first, cell) = unplaced()
            .find(|(_, cell)| cell.candidates.count() == fewest)
            .unwrap();
        assert!(fewest >= 2);

        let children = parent.children();
        let guesses: Vec<(usize, u8)> =
            children.iter().map(|child| guess(&parent, child)).collect();
        // One child for each candidate of that cell, the lowest digit first.
        let expected: Vec<(usize, u8)> = cell
            .candidates
            .digits()
            .map(|digit| (first, digit))
            .collect();
        assert_eq!(guesses, expected);
        // A child is its parent with the guess placed, its one candidate.
        for (child, (place, digit)) in children.iter().zip(guesses) {
            assert_eq!(child.cells[place], Cell::placed(digit));
        }
        assert_eq!(root(NEEDS_A_GUESS).decide(), Decided::Split(children));
    }

    /// The invariants a cell can show, whatever its branch is doing.
    fn cells_keep_their_invariants(branch: &Branch) -> Result<(), TestCaseError> {
        for cell in &branch.cells {
            let all_digits = cell.candidates.digits().count() == cell.candidates.count() as usize;
            prop_assert!(all_digits, "CandidatesAreDigits");
            let only_its_digit = cell
                .digit
                .is_none_or(|digit| cell.candidates.digits().all(|held| held == digit));
            prop_assert!(only_its_digit, "PlacedCellsKeepOnlyTheirDigit");
        }
        Ok(())
    }

    /// `GuessesAreOnFewestCandidates` for the children of a settled branch.
    fn guesses_are_on_fewest_candidates(
        parent: &Branch,
        children: &[Branch],
    ) -> Result<(), TestCaseError> {
        let unplaced = parent.cells.iter().filter(|cell| cell.digit.is_none());
        let fewest = unplaced.map(|cell| cell.candidates.count()).min();
        for child in children {
            let (place, digit) = guess(parent, child);
            let cell = parent.cells[place];
            prop_assert!(cell.digit.is_none() && Some(cell.candidates.count()) == fewest);
            prop_assert!(cell.candidates.contains(digit));
        }
        Ok(())
    }

    /// Cells of the fixture's solution, thinned until a search has to split, and one
    /// time in four with one digit changed: givens with one solution, many or none.
    fn any_givens() -> impl Strategy<Value = BTreeSet<Given>> {
        let kept = (0.0..0.6)
            .prop_flat_map(|chance| prop::collection::vec(prop::bool::weighted(chance), 81));
        let spoiling = prop_oneof![3 => Just(0_u8), 1 => 1_u8..=8];
        (kept, any::<prop::sample::Index>(), spoiling).prop_map(|(kept, which, shift)| {
            let cells = givens(SOLUTION).into_iter().zip(kept);
            let mut thinned: Vec<Given> = cells
                .filter_map(|(given, kept)| kept.then_some(given))
                .collect();
            let spoiled = which.index(thinned.len().max(1));
            if let Some(given) = thinned.get_mut(spoiled) {
                *given = Given::new(given.position(), (given.digit() + shift - 1) % 9 + 1);
            }
            thinned.into_iter().collect()
        })
    }

    proptest! {
        /// Walks a whole search as `search::run` does, one branch at a time from a
        /// stack, and looks inside every branch after every pass.
        #[test]
        fn the_inside_of_a_search_keeps_its_invariants(givens in any_givens()) {
            let mut waiting: Vec<Branch> = Branch::root(&givens).into_iter().collect();
            let mut solved = 0;
            while solved < 2 && let Some(mut branch) = waiting.pop() {
                cells_keep_their_invariants(&branch)?;
                while !branch.is_contradictory() && branch.propagate() {
                    cells_keep_their_invariants(&branch)?;
                }
                let children = branch.children();
                match branch.clone().decide() {
                    Decided::Contradicted => prop_assert!(branch.is_contradictory()),
                    Decided::Solved(_) => {
                        solved += 1;
                        prop_assert!(children.is_empty());
                    }
                    Decided::Split(decided) => {
                        guesses_are_on_fewest_candidates(&branch, &children)?;
                        prop_assert!(children.len() >= 2);
                        prop_assert_eq!(&decided, &children);
                        waiting.extend(children.into_iter().rev());
                    }
                }
            }
        }
    }
}
