//! The puzzle in play: its cells, the two moves, conflicts and the solved state.

use super::proof::WellPosed;
use super::{Given, Grid, LINE, Position, SIDE, at, at_mut, grid_positions, in_range};
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Whether a puzzle is still to be solved. Solved is final.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum Status {
    /// The grid is not yet complete.
    Unsolved,
    /// Every cell holds a digit and none conflicts.
    Solved,
}

/// One cell of a puzzle as it stands: where it sits, what it holds and how it stands
/// with its peers. A value, read from the puzzle and never written back.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) struct Cell {
    position: Position,
    digit: Option<u8>,
    is_given: bool,
    is_conflicting: bool,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
impl Cell {
    /// The cell's row, counted from the top.
    pub(crate) const fn row(self) -> u8 {
        self.position.row
    }

    /// The cell's column, counted from the left.
    pub(crate) const fn column(self) -> u8 {
        self.position.column
    }

    /// The band of the cell's box, counted from the top.
    pub(crate) const fn band(self) -> u8 {
        self.position.band()
    }

    /// The stack of the cell's box, counted from the left.
    pub(crate) const fn stack(self) -> u8 {
        self.position.stack()
    }

    /// The digit the cell holds, or nothing while it is empty.
    pub(crate) const fn digit(self) -> Option<u8> {
        self.digit
    }

    /// Whether the setter wrote the cell's digit, which is then never the player's to
    /// change.
    pub(crate) const fn is_given(self) -> bool {
        self.is_given
    }

    /// Whether the cell holds a digit that one of its peers holds too. A conflict is
    /// two peers holding one digit, and both cells are in it.
    pub(crate) const fn is_conflicting(self) -> bool {
        self.is_conflicting
    }
}

/// Why a move was refused: one variant for each `requires` clause of `PlaceDigit` and
/// `EraseDigit`, and one for a position that names no cell.
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
pub(crate) enum MoveError {
    /// The position is off the grid, so there is no cell to move on.
    #[error("row {}, column {} names no cell of the grid", .position.row, .position.column)]
    NoSuchCell {
        /// The position that was named.
        position: Position,
    },
    /// The puzzle is solved, and no move changes a solved puzzle.
    #[error("the puzzle is solved, and solved is final")]
    AlreadySolved,
    /// The cell holds a given, which is never the player's to change.
    #[error("the cell at row {}, column {} holds a given", .position.row, .position.column)]
    GivenCell {
        /// The given cell's position.
        position: Position,
    },
    /// The digit to place is not from 1 to 9.
    #[error("the digit {digit} is outside 1 to 9")]
    DigitOutOfRange {
        /// The digit that was offered.
        digit: u8,
    },
    /// The cell to erase holds no digit.
    #[error("the cell at row {}, column {} is already empty", .position.row, .position.column)]
    EmptyCell {
        /// The empty cell's position.
        position: Position,
    },
}

/// A puzzle in play: the givens it was set with, its eighty-one cells and whether it
/// is solved.
///
/// A puzzle is set from a proof and from nothing else, so none exists whose givens
/// `SetPuzzle` would refuse. It keeps the proof's solution and never hands it out.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
#[derive(Clone)]
#[non_exhaustive]
pub(crate) struct Puzzle {
    givens: BTreeSet<Given>,
    digits: Grid<Option<u8>>,
    solution: Grid<u8>,
    status: Status,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
impl Puzzle {
    /// `SetPuzzle` and `LayOutGrid` in one step: the puzzle, unsolved, with a cell at
    /// every position, each given's cell holding its digit and every other cell empty.
    pub(crate) fn set(proof: WellPosed) -> Self {
        let (givens, solution) = proof.into_parts();
        let mut digits = [[None; LINE]; LINE];
        for given in &givens {
            if let Some(cell) = at_mut(&mut digits, given.position) {
                *cell = Some(given.digit);
            }
        }
        Self {
            givens,
            digits,
            solution,
            status: Status::Unsolved,
        }
    }

    /// What the setter supplied. No move changes it.
    pub(crate) const fn givens(&self) -> &BTreeSet<Given> {
        &self.givens
    }

    /// Whether the puzzle is solved.
    pub(crate) const fn status(&self) -> Status {
        self.status
    }

    /// The cell at `position` as it stands, or nothing where the grid has no cell.
    pub(crate) fn cell(&self, position: Position) -> Option<Cell> {
        let digit = at(&self.digits, position)?;
        Some(Cell {
            position,
            digit,
            is_given: self.is_given(position),
            is_conflicting: self.conflicts(position),
        })
    }

    /// Every cell as it stands, a row at a time from the top.
    pub(crate) fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        grid_positions().filter_map(|position| self.cell(position))
    }

    /// Whether every cell holds a digit. Counted, as the specification counts it.
    pub(crate) fn is_full(&self) -> bool {
        let filled = self.digits.iter().flatten().filter(|digit| digit.is_some());
        filled.count() == LINE * LINE
    }

    /// Whether no cell conflicts with a peer.
    pub(crate) fn is_consistent(&self) -> bool {
        !grid_positions().any(|position| self.conflicts(position))
    }

    /// `PlaceDigit`: writes `digit` in the cell at `position`.
    pub(crate) fn place(&mut self, position: Position, digit: u8) -> Result<(), MoveError> {
        let cell = self.players_cell(position)?;
        if !in_range(digit) {
            return Err(MoveError::DigitOutOfRange { digit });
        }
        *cell = Some(digit);
        self.settle();
        Ok(())
    }

    /// `EraseDigit`: empties the cell at `position`.
    pub(crate) fn erase(&mut self, position: Position) -> Result<(), MoveError> {
        let cell = self.players_cell(position)?;
        if cell.is_none() {
            return Err(MoveError::EmptyCell { position });
        }
        *cell = None;
        Ok(())
    }

    /// The cell at `position`, to write: what both moves require before anything else.
    fn players_cell(&mut self, position: Position) -> Result<&mut Option<u8>, MoveError> {
        let is_given = self.is_given(position);
        let cell = at_mut(&mut self.digits, position).ok_or(MoveError::NoSuchCell { position })?;
        if self.status == Status::Solved {
            return Err(MoveError::AlreadySolved);
        }
        if is_given {
            return Err(MoveError::GivenCell { position });
        }
        Ok(cell)
    }

    /// `PuzzleSolved`: a puzzle whose grid has become complete is solved, at once.
    fn settle(&mut self) {
        if self.is_full() && self.is_consistent() {
            self.status = Status::Solved;
        }
    }

    /// Whether the cell at `position` holds a digit that one of its peers holds too.
    fn conflicts(&self, position: Position) -> bool {
        let holds = |position| at(&self.digits, position).flatten();
        holds(position).is_some()
            && grid_positions()
                .any(|peer| position.is_peer_of(peer) && holds(peer) == holds(position))
    }

    /// Whether a given sits at `position`.
    fn is_given(&self, position: Position) -> bool {
        self.givens.iter().any(|given| given.position == position)
    }
}

impl Puzzle {
    /// Whether `digit` is the solution's at `position`: the one question a puzzle
    /// answers about its solution, yes or no. It is no wherever the grid has no cell.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the board's check is its one caller, and T27 builds the board"
        )
    )]
    pub(crate) fn is_solution_digit(&self, position: Position, digit: u8) -> bool {
        at(&self.solution, position) == Some(digit)
    }
}

/// Shows the puzzle as it was set and as it stands, a row to a string and a dot for an
/// empty cell. The solution is left out: a puzzle never hands it out.
impl fmt::Debug for Puzzle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let held = |position| at(&self.digits, position).flatten();
        let given = |position| held(position).filter(|_| self.is_given(position));
        formatter
            .debug_struct("Puzzle")
            .field("status", &self.status)
            .field("givens", &picture(given))
            .field("digits", &picture(held))
            .finish_non_exhaustive()
    }
}

/// A grid drawn a row to a string: each cell's digit, or a dot where `digit_at` gives
/// none.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "nothing reaches it until T26: the solver makes the proof and `Puzzle` becomes public"
    )
)]
fn picture(digit_at: impl Fn(Position) -> Option<u8>) -> Vec<String> {
    let cell = |position| {
        digit_at(position)
            .and_then(|digit| char::from_digit(u32::from(digit), 10))
            .unwrap_or('.')
    };
    let row = |row| (1..=SIDE).map(move |column| cell(Position::new(row, column)));
    (1..=SIDE).map(|line| row(line).collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{EXCHANGED, PUZZLE, SOLUTION, are_peers, fixture, givens, grid};
    use super::super::proof::{WellPosed, WellPosedError};
    use super::super::{at, grid_positions};
    use super::{Cell, Given, MoveError, Position, Puzzle, Status};
    use alloc::collections::{BTreeMap, BTreeSet};
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use core::fmt::Write;
    use proptest::prelude::*;

    /// A cell the fixture leaves to the player; its solution digit is 4, and the 5 of
    /// [`GIVEN`] is in its row and its box.
    const FREE: Position = Position::new(1, 3);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// Four positions that name no cell, one past each edge of the grid.
    const OFF_GRID: [Position; 4] = [
        Position::new(0, 1),
        Position::new(1, 0),
        Position::new(10, 1),
        Position::new(1, 10),
    ];

    /// Places the digit `rows` pictures in every cell that is not given, but for `skip`.
    fn play(puzzle: &mut Puzzle, rows: [&str; 9], skip: Option<Position>) {
        for given in givens(rows) {
            let position = given.position();
            let is_given = puzzle.cell(position).unwrap().is_given();
            if !is_given && Some(position) != skip {
                puzzle.place(position, given.digit()).unwrap();
            }
        }
    }

    /// The fixture played through to its solution.
    fn solved() -> Puzzle {
        let mut puzzle = Puzzle::set(fixture());
        play(&mut puzzle, SOLUTION, None);
        puzzle
    }

    fn digit_at(puzzle: &Puzzle, position: Position) -> Option<u8> {
        puzzle.cell(position).unwrap().digit()
    }

    fn cells(puzzle: &Puzzle) -> Vec<Cell> {
        puzzle.cells().collect()
    }

    #[test]
    fn an_accepted_proof_sets_an_unsolved_puzzle_with_exactly_those_givens() {
        let puzzle = Puzzle::set(fixture());
        assert_eq!(puzzle.status(), Status::Unsolved);
        assert_eq!(puzzle.givens(), &givens(PUZZLE));
    }

    #[test]
    fn the_grid_is_eighty_one_cells_one_to_a_position() {
        let puzzle = Puzzle::set(fixture());
        let positions: Vec<Position> = puzzle
            .cells()
            .map(|cell| Position::new(cell.row(), cell.column()))
            .collect();
        let expected: Vec<Position> = (1..=9)
            .flat_map(|row| (1..=9).map(move |column| Position::new(row, column)))
            .collect();
        assert_eq!(positions.len(), 81);
        assert_eq!(positions, expected);
    }

    #[test]
    fn cells_sit_in_the_band_and_stack_of_their_box() {
        const BOX_OF_LINE: [u8; 9] = [1, 1, 1, 2, 2, 2, 3, 3, 3];
        let puzzle = Puzzle::set(fixture());
        for cell in puzzle.cells() {
            assert_eq!(cell.band(), BOX_OF_LINE[usize::from(cell.row() - 1)]);
            assert_eq!(cell.stack(), BOX_OF_LINE[usize::from(cell.column() - 1)]);
        }
    }

    #[test]
    fn a_given_cell_holds_its_digit_and_every_other_cell_is_empty() {
        let pictured = grid(PUZZLE);
        let puzzle = Puzzle::set(fixture());
        for cell in puzzle.cells() {
            let digit = pictured[usize::from(cell.row() - 1)][usize::from(cell.column() - 1)];
            assert_eq!(cell.digit(), digit);
            assert_eq!(cell.is_given(), digit.is_some());
        }
        assert_eq!(puzzle.cells().filter(|cell| cell.is_given()).count(), 30);
    }

    #[test]
    fn placing_puts_the_digit_in_the_cell() {
        let mut puzzle = Puzzle::set(fixture());
        let before = cells(&puzzle);
        assert_eq!(puzzle.place(FREE, 4), Ok(()));
        assert_eq!(digit_at(&puzzle, FREE), Some(4));
        let changed = cells(&puzzle)
            .into_iter()
            .zip(before)
            .filter(|(after, before)| after != before)
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn placing_is_refused_on_a_solved_puzzle() {
        let mut puzzle = solved();
        assert_eq!(puzzle.place(FREE, 4), Err(MoveError::AlreadySolved));
        assert_eq!(puzzle.place(FREE, 7), Err(MoveError::AlreadySolved));
    }

    #[test]
    fn placing_is_refused_on_a_given_cell() {
        let mut puzzle = Puzzle::set(fixture());
        for digit in [5, 6] {
            assert_eq!(
                puzzle.place(GIVEN, digit),
                Err(MoveError::GivenCell { position: GIVEN })
            );
        }
        assert_eq!(digit_at(&puzzle, GIVEN), Some(5));
    }

    #[test]
    fn placing_is_refused_for_a_digit_outside_one_to_nine() {
        let mut puzzle = Puzzle::set(fixture());
        for digit in [0, 10, u8::MAX] {
            assert_eq!(
                puzzle.place(FREE, digit),
                Err(MoveError::DigitOutOfRange { digit })
            );
        }
        assert_eq!(digit_at(&puzzle, FREE), None);
    }

    #[test]
    fn placing_is_refused_where_no_cell_is() {
        let mut puzzle = Puzzle::set(fixture());
        for position in OFF_GRID {
            assert_eq!(
                puzzle.place(position, 4),
                Err(MoveError::NoSuchCell { position })
            );
        }
    }

    #[test]
    fn a_conflicting_digit_may_be_placed() {
        let mut puzzle = Puzzle::set(fixture());
        assert_eq!(puzzle.place(FREE, 5), Ok(()));
        assert_eq!(digit_at(&puzzle, FREE), Some(5));
    }

    #[test]
    fn a_cell_holding_the_players_digit_takes_a_new_one() {
        let mut puzzle = Puzzle::set(fixture());
        puzzle.place(FREE, 4).unwrap();
        assert_eq!(puzzle.place(FREE, 6), Ok(()));
        assert_eq!(digit_at(&puzzle, FREE), Some(6));
    }

    #[test]
    fn the_same_digit_may_be_placed_again() {
        let mut puzzle = Puzzle::set(fixture());
        puzzle.place(FREE, 4).unwrap();
        assert_eq!(puzzle.place(FREE, 4), Ok(()));
        assert_eq!(digit_at(&puzzle, FREE), Some(4));
    }

    #[test]
    fn erasing_empties_the_cell() {
        let mut puzzle = Puzzle::set(fixture());
        let before = cells(&puzzle);
        puzzle.place(FREE, 4).unwrap();
        assert_eq!(puzzle.erase(FREE), Ok(()));
        assert_eq!(digit_at(&puzzle, FREE), None);
        assert_eq!(cells(&puzzle), before);
    }

    #[test]
    fn erasing_is_refused_on_a_solved_puzzle() {
        let mut puzzle = solved();
        assert_eq!(puzzle.erase(FREE), Err(MoveError::AlreadySolved));
    }

    #[test]
    fn erasing_is_refused_on_a_given_cell() {
        let mut puzzle = Puzzle::set(fixture());
        assert_eq!(
            puzzle.erase(GIVEN),
            Err(MoveError::GivenCell { position: GIVEN })
        );
        assert_eq!(digit_at(&puzzle, GIVEN), Some(5));
    }

    #[test]
    fn erasing_is_refused_on_an_empty_cell() {
        let mut puzzle = Puzzle::set(fixture());
        assert_eq!(
            puzzle.erase(FREE),
            Err(MoveError::EmptyCell { position: FREE })
        );
    }

    #[test]
    fn erasing_is_refused_where_no_cell_is() {
        let mut puzzle = Puzzle::set(fixture());
        for position in OFF_GRID {
            assert_eq!(
                puzzle.erase(position),
                Err(MoveError::NoSuchCell { position })
            );
        }
    }

    #[test]
    fn move_error_texts_are_stable() {
        let position = Position::new(2, 3);
        let texts = [
            (
                MoveError::NoSuchCell {
                    position: Position::new(0, 10),
                },
                "row 0, column 10 names no cell of the grid",
            ),
            (
                MoveError::AlreadySolved,
                "the puzzle is solved, and solved is final",
            ),
            (
                MoveError::GivenCell { position },
                "the cell at row 2, column 3 holds a given",
            ),
            (
                MoveError::DigitOutOfRange { digit: 10 },
                "the digit 10 is outside 1 to 9",
            ),
            (
                MoveError::EmptyCell { position },
                "the cell at row 2, column 3 is already empty",
            ),
        ];
        for (error, text) in texts {
            assert_eq!(error.to_string(), text);
        }
    }

    #[test]
    fn a_complete_grid_is_solved_at_once() {
        let mut puzzle = Puzzle::set(fixture());
        play(&mut puzzle, SOLUTION, Some(FREE));
        assert_eq!(puzzle.status(), Status::Unsolved);
        puzzle.place(FREE, 4).unwrap();
        assert_eq!(puzzle.status(), Status::Solved);
    }

    #[test]
    fn a_full_grid_with_a_conflict_is_not_solved() {
        let mut puzzle = Puzzle::set(fixture());
        play(&mut puzzle, SOLUTION, Some(FREE));
        puzzle.place(FREE, 5).unwrap();
        assert!(puzzle.cells().all(|cell| cell.digit().is_some()));
        assert_eq!(puzzle.status(), Status::Unsolved);
        puzzle.place(FREE, 4).unwrap();
        assert_eq!(puzzle.status(), Status::Solved);
    }

    #[test]
    fn solved_is_final() {
        let mut puzzle = solved();
        let before = cells(&puzzle);
        assert_eq!(puzzle.status(), Status::Solved);
        assert!(puzzle.place(FREE, 5).is_err());
        assert!(puzzle.erase(FREE).is_err());
        assert_eq!(puzzle.status(), Status::Solved);
        assert_eq!(cells(&puzzle), before);
    }

    #[test]
    fn solved_is_not_a_comparison_with_the_stored_solution() {
        let proof = WellPosed::vouch(BTreeSet::new(), grid(SOLUTION)).unwrap();
        let mut puzzle = Puzzle::set(proof);
        play(&mut puzzle, EXCHANGED, None);
        assert_ne!(grid(EXCHANGED), grid(SOLUTION));
        assert_eq!(
            cells(&puzzle)
                .iter()
                .map(|cell| cell.digit())
                .collect::<Vec<_>>(),
            grid(EXCHANGED).into_iter().flatten().collect::<Vec<_>>()
        );
        assert_eq!(puzzle.status(), Status::Solved);
    }

    #[test]
    fn full_counts_the_filled_cells() {
        let mut puzzle = Puzzle::set(fixture());
        assert!(!puzzle.is_full());
        play(&mut puzzle, SOLUTION, Some(FREE));
        assert!(!puzzle.is_full());
        puzzle.place(FREE, 5).unwrap();
        assert!(puzzle.is_full());
    }

    #[test]
    fn consistent_means_no_cell_conflicts() {
        let mut puzzle = Puzzle::set(fixture());
        assert!(puzzle.is_consistent());
        puzzle.place(FREE, 5).unwrap();
        assert!(!puzzle.is_consistent());
        puzzle.place(FREE, 4).unwrap();
        assert!(puzzle.is_consistent());
    }

    #[test]
    fn a_conflict_marks_both_cells() {
        let mut puzzle = Puzzle::set(fixture());
        puzzle.place(FREE, 5).unwrap();
        let conflicting: Vec<Position> = puzzle
            .cells()
            .filter(|cell| cell.is_conflicting())
            .map(|cell| Position::new(cell.row(), cell.column()))
            .collect();
        assert_eq!(conflicting, [GIVEN, FREE]);
    }

    #[test]
    fn an_empty_cell_conflicts_with_nothing() {
        let mut puzzle = Puzzle::set(fixture());
        assert!(!puzzle.cells().any(Cell::is_conflicting));
        puzzle.place(FREE, 5).unwrap();
        assert!(
            puzzle
                .cells()
                .all(|cell| cell.digit().is_some() || !cell.is_conflicting())
        );
        puzzle.erase(FREE).unwrap();
        assert!(!puzzle.cells().any(Cell::is_conflicting));
    }

    #[test]
    fn a_refused_move_leaves_the_puzzle_as_it_was() {
        let mut puzzle = Puzzle::set(fixture());
        puzzle.place(FREE, 4).unwrap();
        let before = cells(&puzzle);
        let empty = Position::new(1, 4);
        assert!(puzzle.place(GIVEN, 1).is_err());
        assert!(puzzle.place(FREE, 0).is_err());
        assert!(puzzle.place(OFF_GRID[0], 1).is_err());
        assert!(puzzle.erase(GIVEN).is_err());
        assert!(puzzle.erase(empty).is_err());
        assert!(puzzle.erase(OFF_GRID[0]).is_err());
        assert_eq!(cells(&puzzle), before);
        assert_eq!(puzzle.status(), Status::Unsolved);
    }

    #[test]
    fn the_surface_exposes_the_status_and_every_cell() {
        let mut puzzle = Puzzle::set(fixture());
        puzzle.place(FREE, 5).unwrap();
        assert_eq!(puzzle.status(), Status::Unsolved);
        assert!(!puzzle.is_full());
        assert!(!puzzle.is_consistent());
        let read = |position| {
            let cell = puzzle.cell(position).unwrap();
            (
                (cell.row(), cell.column(), cell.band(), cell.stack()),
                (cell.digit(), cell.is_given(), cell.is_conflicting()),
            )
        };
        assert_eq!(read(FREE), ((1, 3, 1, 1), (Some(5), false, true)));
        assert_eq!(read(GIVEN), ((1, 1, 1, 1), (Some(5), true, true)));
        assert_eq!(
            read(Position::new(9, 9)),
            ((9, 9, 3, 3), (Some(9), true, false))
        );
        assert_eq!(
            read(Position::new(5, 5)),
            ((5, 5, 2, 2), (None, false, false))
        );
        assert_eq!(puzzle.cells().count(), 81);
        assert!(
            OFF_GRID
                .iter()
                .all(|&position| puzzle.cell(position).is_none())
        );
    }

    #[test]
    fn the_answer_is_yes_for_the_solutions_digit_and_no_for_every_other() {
        let solution = grid(SOLUTION);
        let puzzle = Puzzle::set(fixture());
        for position in grid_positions() {
            let right = at(&solution, position).unwrap();
            for digit in (0..=10).chain([u8::MAX]) {
                assert_eq!(
                    puzzle.is_solution_digit(position, digit),
                    Some(digit) == right
                );
            }
        }
    }

    #[test]
    fn two_puzzles_alike_but_for_the_solution_print_alike() {
        let set =
            |solution| Puzzle::set(WellPosed::vouch(BTreeSet::new(), grid(solution)).unwrap());
        let (mut one, mut other) = (set(SOLUTION), set(EXCHANGED));
        assert_eq!(format!("{one:?}"), format!("{other:?}"));
        for puzzle in [&mut one, &mut other] {
            puzzle.place(FREE, 4).unwrap();
        }
        assert_eq!(format!("{one:?}"), format!("{other:?}"));
        assert_eq!(format!("{one:#?}"), format!("{other:#?}"));
        assert!(format!("{one:?}").starts_with("Puzzle"));
        assert!(!format!("{:?}", Puzzle::set(fixture())).contains("534678912"));
    }

    /// One thing a player may try, allowed or not.
    #[derive(Debug, Clone, Copy)]
    enum Move {
        Place(Position, u8),
        Erase(Position),
        /// Place the solution's digit in the first cell, in grid order, that does not
        /// hold it: what lets a script run on to a solved puzzle.
        PlaceNextRight,
    }

    impl Move {
        /// The move as it is made on `puzzle`: where, and the digit to place or none
        /// to erase.
        fn on(self, puzzle: &Puzzle) -> (Position, Option<u8>) {
            match self {
                Self::Place(position, digit) => (position, Some(digit)),
                Self::Erase(position) => (position, None),
                Self::PlaceNextRight => grid_positions()
                    .find_map(|position| {
                        let right = at(&grid(SOLUTION), position)?;
                        (digit_at(puzzle, position) != right).then_some((position, right))
                    })
                    .unwrap_or((FREE, Some(4))),
            }
        }
    }

    /// Makes `step` on the puzzle and says where, with what digit, and whether it was
    /// accepted.
    fn make(puzzle: &mut Puzzle, step: Move) -> (Position, Option<u8>, bool) {
        let (position, digit) = step.on(puzzle);
        let made = match digit {
            Some(digit) => puzzle.place(position, digit),
            None => puzzle.erase(position),
        };
        (position, digit, made.is_ok())
    }

    /// The rules as `sudoku.allium` states them, kept apart from the implementation: a
    /// move is accepted exactly when its position names a cell and every `requires`
    /// clause of its rule holds.
    #[derive(Debug, Clone)]
    struct Model {
        givens: BTreeSet<Position>,
        digits: BTreeMap<Position, u8>,
        solved: bool,
    }

    impl Model {
        fn of(givens: &BTreeSet<Given>) -> Self {
            Self {
                givens: givens.iter().map(|given| given.position()).collect(),
                digits: givens
                    .iter()
                    .map(|given| (given.position(), given.digit()))
                    .collect(),
                solved: false,
            }
        }

        fn names_a_cell(position: Position) -> bool {
            (1..=9).contains(&position.row()) && (1..=9).contains(&position.column())
        }

        fn is_players(&self, position: Position) -> bool {
            Self::names_a_cell(position) && !self.solved && !self.givens.contains(&position)
        }

        fn conflicts(&self, position: Position, digit: u8) -> bool {
            let mut cells = self.digits.iter();
            cells.any(|(&peer, &other)| are_peers(position, peer) && digit == other)
        }

        /// `PlaceDigit` for a digit and `EraseDigit` for none, then `PuzzleSolved`.
        fn make(&mut self, position: Position, digit: Option<u8>) -> bool {
            let allowed = self.is_players(position)
                && match digit {
                    Some(digit) => (1..=9).contains(&digit),
                    None => self.digits.contains_key(&position),
                };
            if let (true, Some(digit)) = (allowed, digit) {
                self.digits.insert(position, digit);
            } else if allowed {
                self.digits.remove(&position);
            }
            let mut cells = self.digits.iter();
            let consistent = !cells.any(|(&cell, &digit)| self.conflicts(cell, digit));
            self.solved = self.solved || (self.digits.len() == 81 && consistent);
            allowed
        }
    }

    fn any_position() -> impl Strategy<Value = Position> {
        (0_u8..=10, 0_u8..=10).prop_map(|(row, column)| Position::new(row, column))
    }

    fn any_move() -> impl Strategy<Value = Move> {
        prop_oneof![
            4 => (any_position(), 0_u8..=10).prop_map(|(position, digit)| Move::Place(position, digit)),
            3 => any_position().prop_map(Move::Erase),
            3 => Just(Move::PlaceNextRight),
        ]
    }

    /// A puzzle part of the way to its solution, and moves to make on it: the fixture
    /// with all but a few of its cells played right, so that a script may solve it and
    /// then go on.
    fn any_game() -> impl Strategy<Value = (Puzzle, Vec<Move>)> {
        (0_usize..=51, prop::collection::vec(any_move(), 0..40)).prop_map(|(played, moves)| {
            let mut puzzle = Puzzle::set(fixture());
            for _ in 0..played {
                make(&mut puzzle, Move::PlaceNextRight);
            }
            (puzzle, moves)
        })
    }

    /// The eight invariants of `sudoku.allium`, each under its own name.
    fn hold_invariants(puzzle: &Puzzle) -> Result<(), TestCaseError> {
        let cells = cells(puzzle);
        let position = |cell: &Cell| Position::new(cell.row(), cell.column());
        let positions: BTreeSet<Position> = cells.iter().map(position).collect();
        let givens: Vec<&Cell> = cells.iter().filter(|cell| cell.is_given()).collect();
        prop_assert_eq!(cells.len(), 81, "TheGridIsWhole");
        prop_assert_eq!(positions.len(), cells.len(), "OneCellToAPosition");
        for cell in &cells {
            let on_the_grid = (1..=9).contains(&cell.row()) && (1..=9).contains(&cell.column());
            prop_assert!(on_the_grid, "CellsSitOnTheGrid");
            let in_its_box = (cell.band() - 1) * 3 < cell.row()
                && cell.row() <= cell.band() * 3
                && (cell.stack() - 1) * 3 < cell.column()
                && cell.column() <= cell.stack() * 3;
            prop_assert!(in_its_box, "CellsSitInTheirBox");
            let in_range = cell.digit().is_none_or(|digit| (1..=9).contains(&digit));
            prop_assert!(in_range, "DigitsAreInRange");
        }
        for given in &givens {
            let holds_its_given =
                cell_given(**given).is_some_and(|it| puzzle.givens().contains(&it));
            prop_assert!(holds_its_given, "GivenCellsHoldTheirGiven");
            let conflicts = givens.iter().any(|other| {
                are_peers(position(given), position(other)) && given.digit() == other.digit()
            });
            prop_assert!(!conflicts, "GivensNeverConflict");
        }
        let complete = cells
            .iter()
            .all(|cell| cell.digit().is_some() && !cell.is_conflicting());
        prop_assert!(
            puzzle.status() != Status::Solved || complete,
            "SolvedMeansComplete"
        );
        Ok(())
    }

    /// The given a cell would be, if it holds a digit.
    fn cell_given(cell: Cell) -> Option<Given> {
        Some(Given::new(
            Position::new(cell.row(), cell.column()),
            cell.digit()?,
        ))
    }

    proptest! {
        #[test]
        fn the_invariants_hold_after_every_move((mut puzzle, moves) in any_game()) {
            hold_invariants(&puzzle)?;
            for step in moves {
                make(&mut puzzle, step);
                hold_invariants(&puzzle)?;
            }
        }

        #[test]
        fn a_puzzles_givens_never_change((mut puzzle, moves) in any_game()) {
            for step in moves {
                make(&mut puzzle, step);
                prop_assert_eq!(puzzle.givens(), &givens(PUZZLE));
            }
        }

        #[test]
        fn a_move_is_accepted_exactly_when_its_rule_allows_it((mut puzzle, moves) in any_game()) {
            let mut model = Model::of(puzzle.givens());
            for placed in puzzle.cells().filter(|cell| !cell.is_given()).filter_map(cell_given) {
                model.make(placed.position(), Some(placed.digit()));
            }
            for step in moves {
                let (position, digit, accepted) = make(&mut puzzle, step);
                let allowed = model.make(position, digit);
                prop_assert_eq!(accepted, allowed, "{:?}", step);
                let digits: BTreeMap<Position, u8> = puzzle
                    .cells()
                    .filter_map(cell_given)
                    .map(|given| (given.position(), given.digit()))
                    .collect();
                prop_assert_eq!(&digits, &model.digits);
                prop_assert_eq!(puzzle.status() == Status::Solved, model.solved);
            }
        }

        #[test]
        fn a_move_is_refused_wherever_it_is_not_offered((mut puzzle, moves) in any_game()) {
            for step in moves {
                let (position, digit) = step.on(&puzzle);
                let unsolved = puzzle.status() == Status::Unsolved;
                let offered = puzzle.cell(position).filter(|cell| unsolved && !cell.is_given());
                let expected = match digit {
                    Some(digit) => offered.is_some() && (1..=9).contains(&digit),
                    None => offered.is_some_and(|cell| cell.digit().is_some()),
                };
                let (.., accepted) = make(&mut puzzle, step);
                prop_assert_eq!(accepted, expected, "{:?}", step);
            }
        }

        #[test]
        fn the_answer_is_no_wherever_no_cell_is(row: u8, column: u8, digit: u8) {
            let position = Position::new(row, column);
            let right = Model::names_a_cell(position)
                .then(|| at(&grid(SOLUTION), position))
                .flatten()
                .flatten();
            let answer = Puzzle::set(fixture()).is_solution_digit(position, digit);
            prop_assert_eq!(answer, right == Some(digit));
        }
    }

    /// One row of a picture: a character for each cell.
    fn draw(row: &[Cell], character: impl Fn(Cell) -> char) -> String {
        row.iter().copied().map(character).collect()
    }

    /// A picture of a puzzle from what `PuzzleSolving` exposes and nothing else: the
    /// digits, which cells are given and which conflict, side by side, then the status.
    fn render(puzzle: &Puzzle) -> String {
        let mut picture = String::from("digits     given      conflicting\n");
        for row in cells(puzzle).chunks(9) {
            let digits = draw(row, |cell| {
                cell.digit().map_or('.', |digit| char::from(b'0' + digit))
            });
            let given = draw(row, |cell| if cell.is_given() { 'x' } else { '.' });
            let conflicting = draw(row, |cell| if cell.is_conflicting() { 'x' } else { '.' });
            writeln!(picture, "{digits}  {given}  {conflicting}").unwrap();
        }
        writeln!(picture, "status: {:?}", puzzle.status()).unwrap();
        writeln!(picture, "is_full: {}", puzzle.is_full()).unwrap();
        writeln!(picture, "is_consistent: {}", puzzle.is_consistent()).unwrap();
        picture
    }

    // Pictures for review and change detection; the tests above own correctness.
    #[test]
    fn snapshot_the_fixture_as_set() {
        insta::assert_snapshot!(render(&Puzzle::set(fixture())));
    }

    #[test]
    fn snapshot_the_fixture_after_a_short_script() {
        let mut puzzle = Puzzle::set(fixture());
        let mut picture = String::new();
        let mut show = |caption: &str, puzzle: &Puzzle| {
            writeln!(picture, "{caption}\n{}", render(puzzle)).unwrap();
        };
        puzzle.place(FREE, 4).unwrap();
        show("place 4 at row 1, column 3", &puzzle);
        puzzle.place(Position::new(1, 4), 5).unwrap();
        show(
            "place 5 at row 1, column 4: it conflicts with two givens",
            &puzzle,
        );
        puzzle.place(FREE, 2).unwrap();
        show(
            "place 2 at row 1, column 3: over the player's own 4",
            &puzzle,
        );
        puzzle.erase(Position::new(1, 4)).unwrap();
        show("erase row 1, column 4", &puzzle);
        insta::assert_snapshot!(picture.trim_end());
    }

    #[test]
    fn snapshot_the_puzzles_debug() {
        insta::assert_snapshot!(format!("{:#?}", Puzzle::set(fixture())));
    }

    #[test]
    fn snapshot_every_error_text() {
        let (here, there) = (Position::new(2, 3), Position::new(2, 8));
        let given = Given::new(here, 7);
        let mut picture = String::from("WellPosedError (Display)\n");
        for error in [
            WellPosedError::NothingLeftToPlay { givens: 81 },
            WellPosedError::SolutionNotFull { position: here },
            WellPosedError::SolutionDigitOutOfRange {
                position: here,
                digit: 10,
            },
            WellPosedError::SolutionConflict {
                first: here,
                second: there,
                digit: 4,
            },
            WellPosedError::GivenOffGrid {
                given: Given::new(Position::new(0, 10), 7),
            },
            WellPosedError::GivenMismatch { given, solution: 2 },
        ] {
            writeln!(picture, "{error}").unwrap();
        }
        writeln!(picture, "\nMoveError (Display)").unwrap();
        for error in [
            MoveError::NoSuchCell {
                position: Position::new(0, 10),
            },
            MoveError::AlreadySolved,
            MoveError::GivenCell { position: here },
            MoveError::DigitOutOfRange { digit: 10 },
            MoveError::EmptyCell { position: here },
        ] {
            writeln!(picture, "{error}").unwrap();
        }
        insta::assert_snapshot!(picture);
    }
}
