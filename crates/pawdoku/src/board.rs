//! The board: one puzzle as it is being played.
//!
//! This module is `docs/specs/board.allium` in Rust, less the record, which is a later
//! change. It stands on [`sudoku`](crate::sudoku), the rules, and on
//! [`solver`](crate::solver), which finds the one solution a check is answered from.
//! Nothing imports it.
//!
//! A [`Board`] is the thing a player plays on. It owns its puzzle and hands out values
//! of its own, never the puzzle: a [`BoardCell`] for what a cell shows, a [`Move`] for
//! each move on the record, a [`Check`] for each question asked, a [`Note`] for a
//! cell's marks. To the rules it adds what a player expects:
//!
//! - **Notes.** Every cell that is not a given has a note, the marks the player has
//!   written in it. A note is shown while its cell is empty and waits beneath a digit.
//! - **Moves.** Placing a digit, erasing one, writing a mark and striking one are the
//!   four moves, and each is recorded with what it displaced. [`Board::undo`] takes
//!   back the latest move that stands and [`Board::redo`] re-takes the latest undone;
//!   a new move discards every undone one.
//! - **Upkeep.** A placed digit is struck from the note of every peer that holds it.
//!   Erasing the digit gives nothing back; undoing the placement gives back exactly
//!   what it struck.
//! - **Reading back.** Each [`Move`] can say what any cell held once it had been made,
//!   whether it stands or is undone, and reading takes nothing back.
//! - **The check.** [`Board::check`] asks whether one cell's digit is the solution's
//!   and hears yes or no. Nothing else a board hands out is read from the solution.
//!
//! The five operations on a cell take a [`Position`]; undo and redo take nothing. Each
//! refuses with a [`PlayError`], and a refused operation changes nothing.
//!
//! # A board and its puzzle
//!
//! A board opens from givens, in one call, and from nothing else. [`Board::open`] puts
//! the givens to the solver and sets the puzzle from its proof, so no board takes a
//! [`Puzzle`] that may already have been played on: such a
//! puzzle would hold digits no move recorded.
//!
//! `board.allium` gives every puzzle that is set a board. That is this module's view,
//! of the puzzles set through it. A puzzle made the two-step way, solved and then set,
//! has no board, and that is not a defect: it is the rules' own boundary, which
//! `sudoku.allium` keeps. Each board owns its puzzle and no second board can reach it,
//! so no two boards share one.
//!
//! **Not final.** This is a draft: it may change as implementation continues.
//!
//! ```
//! use pawdoku::board::Board;
//! use pawdoku::sudoku::{Given, Position, Status};
//!
//! // Thirty givens, rows top to bottom, a dot for an empty cell.
//! let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
//!     .bytes()
//!     .zip(0_u8..)
//!     .filter(|(cell, _)| cell.is_ascii_digit())
//!     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
//!
//! let mut board = Board::open(givens)?;
//! assert_eq!(board.status(), Status::Unsolved);
//!
//! // The third cell of the first row is empty, and its digit is 4.
//! let here = Position::new(1, 3);
//! board.place(here, 4)?;
//! assert!(board.check(here)?.is_right());
//!
//! board.undo()?;
//! assert_eq!(board.cell(here).and_then(|cell| cell.digit()), None);
//! assert!(board.can_redo());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod cell;
mod check;
mod error;
mod journal;
mod moves;
mod note;
mod reading;

pub use cell::BoardCell;
pub use check::Check;
pub use error::PlayError;
pub use moves::{Move, MoveKind};
pub use note::Note;

use crate::solver::{SolveError, solve};
use crate::sudoku::{Given, MoveError, Position, Puzzle, Status, in_range};
use alloc::vec::Vec;
use journal::{Journal, Made};
use note::Notes;
use reading::Reading;

/// A puzzle in play: the puzzle, a note in every cell, every move made and every check
/// asked.
///
/// A board owns its puzzle and never hands it out. What a player can see is read
/// through the board, as values, and what a player does goes through the board, so
/// that every placement and erasure is recorded, kept up and reversible.
///
/// ```
/// use pawdoku::board::Board;
/// use pawdoku::sudoku::{Given, Position};
///
/// // Thirty givens, rows top to bottom, a dot for an empty cell.
/// let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
///     .bytes()
///     .zip(0_u8..)
///     .filter(|(cell, _)| cell.is_ascii_digit())
///     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
///
/// let mut board = Board::open(givens)?;
/// let here = Position::new(1, 3);
/// board.write_mark(here, 4)?;
/// board.place(here, 4)?;
///
/// assert_eq!(board.moves().count(), 2);
/// assert_eq!(board.cells().filter(|cell| cell.holds_players_digit()).count(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Board {
    puzzle: Puzzle,
    notes: Notes,
    journal: Journal,
    checks: Vec<Check>,
}

impl Board {
    /// `SetPuzzle`, `OpenBoard` and `LayOutBoard` in one call: solves `givens`, sets
    /// the puzzle from the proof, and opens a board on it with every note empty, no
    /// move and no check.
    ///
    /// The givens are read as a set, as [`solve`] reads them. The search runs once,
    /// here, and each check reads what it found.
    ///
    /// # Errors
    ///
    /// What [`solve`] refuses, a board refuses:
    ///
    /// - [`SolveError::NoSolution`] when the givens have no solution;
    /// - [`SolveError::ManySolutions`] when they have more than one;
    /// - [`SolveError::NotPosed`] when they have one and leave nothing to play.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::solver::SolveError;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let board = Board::open(givens)?;
    /// assert_eq!(board.cells().count(), 81);
    /// assert_eq!(board.moves().count(), 0);
    /// assert_eq!(board.checks().count(), 0);
    ///
    /// // No givens at all have many solutions, so there is no board to open.
    /// assert_eq!(Board::open([]).unwrap_err(), SolveError::ManySolutions);
    /// # Ok::<(), SolveError>(())
    /// ```
    pub fn open(givens: impl IntoIterator<Item = Given>) -> Result<Self, SolveError> {
        Ok(Self {
            puzzle: Puzzle::set(solve(givens)?),
            notes: Notes::empty(),
            journal: Journal::new(),
            checks: Vec::new(),
        })
    }

    /// Whether the puzzle is solved. Solved is the rules' definition, every cell full
    /// and none in conflict, and it is final.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position, Status};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let board = Board::open(givens)?;
    /// assert_eq!(board.status(), Status::Unsolved);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn status(&self) -> Status {
        self.puzzle.status()
    }

    /// Whether every cell holds a digit.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let board = Board::open(givens)?;
    /// // Fifty-one cells are still empty.
    /// assert!(!board.is_full());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.puzzle.is_full()
    }

    /// Whether no cell conflicts with a peer.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// assert!(board.is_consistent());
    ///
    /// // A second 5 in row 1 conflicts with the given at its head.
    /// board.place(Position::new(1, 3), 5)?;
    /// assert!(!board.is_consistent());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.puzzle.is_consistent()
    }

    /// Whether [`undo`](Self::undo) has a move to take back: the puzzle is unsolved
    /// and a move stands.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// assert!(!board.can_undo());
    /// board.place(Position::new(1, 3), 4)?;
    /// assert!(board.can_undo());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.status() == Status::Unsolved && self.journal.standing() > 0
    }

    /// Whether [`redo`](Self::redo) has a move to re-take: the puzzle is unsolved and
    /// a move is undone.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// board.place(Position::new(1, 3), 4)?;
    /// assert!(!board.can_redo());
    /// board.undo()?;
    /// assert!(board.can_redo());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.status() == Status::Unsolved && self.journal.has_undone()
    }

    /// The cell at `position` as the board shows it, or nothing where the grid has no
    /// cell.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let board = Board::open(givens)?;
    ///
    /// let cell = board.cell(Position::new(9, 9)).expect("row 9, column 9 is a cell");
    /// assert_eq!(cell.digit(), Some(9));
    /// // Row 0 and column 10 are off the grid.
    /// assert!(board.cell(Position::new(0, 1)).is_none());
    /// assert!(board.cell(Position::new(1, 10)).is_none());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn cell(&self, position: Position) -> Option<BoardCell> {
        let cell = self.puzzle.cell(position)?;
        Some(BoardCell::new(cell, self.notes.at(position)))
    }

    /// Every cell as the board shows it, a row at a time from the top.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let board = Board::open(givens)?;
    ///
    /// assert_eq!(board.cells().count(), 81);
    /// let first_row: Vec<Option<u8>> = board.cells().take(9).map(|cell| cell.digit()).collect();
    /// assert_eq!(first_row[..3], [Some(5), Some(3), None]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn cells(&self) -> impl Iterator<Item = BoardCell> + '_ {
        let cells = self.puzzle.cells();
        cells.map(|cell| {
            let position = Position::new(cell.row(), cell.column());
            BoardCell::new(cell, self.notes.at(position))
        })
    }

    /// Every move on the record, in the order made, standing and undone alike. Each
    /// carries its reading: the board as it stood once that move had been made.
    ///
    /// The readings are worked out as the moves are handed out, from the board as it
    /// was opened forward through the moves, so reading back takes nothing back.
    ///
    /// ```
    /// use pawdoku::board::{Board, MoveKind};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    /// board.place(here, 4)?;
    /// board.erase(here)?;
    /// board.undo()?;
    ///
    /// let moves: Vec<_> = board.moves().collect();
    /// assert_eq!(moves.len(), 2);
    /// assert_eq!((moves[0].kind(), moves[0].is_undone()), (MoveKind::Place, false));
    /// assert_eq!((moves[1].kind(), moves[1].is_undone()), (MoveKind::Erase, true));
    /// // The undone erasure still reads back the board it left.
    /// assert_eq!(moves[1].digit_after(here), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn moves(&self) -> impl Iterator<Item = Move> + '_ {
        self.journal.moves(Reading::opening(self.puzzle.givens()))
    }

    /// Every check asked, in the order asked. A check is never taken back.
    ///
    /// The checks are handed out and nothing is counted from them: whether wrong
    /// answers or checks made are counted, and shown, is an open question of
    /// `board.allium`.
    ///
    /// ```
    /// use pawdoku::board::Board;
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    /// board.place(here, 4)?;
    /// let asked = board.check(here)?;
    ///
    /// // Undo takes back the placement and leaves the check.
    /// board.undo()?;
    /// assert_eq!(board.checks().collect::<Vec<_>>(), [asked]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn checks(&self) -> impl Iterator<Item = Check> + '_ {
        self.checks.iter().copied()
    }

    /// `Place`: writes `digit` in the cell at `position`, and strikes it from the note
    /// of every peer that holds it.
    ///
    /// The placement joins the record, remembering what the cell held and which notes
    /// lost the digit, and discards every undone move. The cell's own note is kept
    /// beneath the digit. A digit that conflicts with a peer may be placed, as on
    /// paper; the placement that completes the grid solves the puzzle at once.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::NoSuchCell`] when `position` is off the grid;
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::GivenCell`] when the cell holds a given;
    /// - [`PlayError::DigitOutOfRange`] when `digit` is not from 1 to 9;
    /// - [`PlayError::DigitAlreadyStands`] when the cell already holds `digit`.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let (here, beside) = (Position::new(1, 3), Position::new(1, 4));
    /// board.write_mark(beside, 4)?;
    ///
    /// board.place(here, 4)?;
    /// assert_eq!(board.cell(here).and_then(|cell| cell.digit()), Some(4));
    /// // Upkeep: the 4 is struck from the note of the cell beside it.
    /// assert!(board.cell(beside).and_then(|cell| cell.note()).is_some_and(|note| note.is_empty()));
    ///
    /// // The same digit again changes nothing, so it is not a move.
    /// let refusal = PlayError::DigitAlreadyStands { position: here, digit: 4 };
    /// assert_eq!(board.place(here, 4), Err(refusal));
    /// assert_eq!(board.moves().count(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn place(&mut self, position: Position, digit: u8) -> Result<(), PlayError> {
        let cell = cell_in_play(self, position)?;
        require(!cell.is_given(), PlayError::GivenCell { position })?;
        require(in_range(digit), PlayError::DigitOutOfRange { digit })?;
        let changes = cell.digit() != Some(digit);
        require(changes, PlayError::DigitAlreadyStands { position, digit })?;
        let holders = self.cells().filter(|peer| peer.kept_note().contains(digit));
        let struck = holders.filter(|peer| peer.is_peer_of(cell));
        let made = Made::Place {
            target: position,
            digit,
            before: cell.digit(),
            struck: struck.map(BoardCell::position).collect(),
        };
        perform(&mut self.puzzle, &mut self.notes, &made)?;
        self.journal.record(made);
        Ok(())
    }

    /// `Erase`: empties the cell at `position`, which must hold a digit of the player's
    /// own.
    ///
    /// The erasure joins the record, remembering the digit, and discards every undone
    /// move. The note the digit stood on is shown again. The peers' notes get nothing
    /// back: an erasure is not the reverse of a placement, and only undo is.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::NoSuchCell`] when `position` is off the grid;
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::NoPlayersDigit`] when the cell is empty or holds a given.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    ///
    /// board.place(here, 4)?;
    /// board.erase(here)?;
    /// assert_eq!(board.cell(here).and_then(|cell| cell.digit()), None);
    ///
    /// // There is nothing left to erase, and a given is never the player's.
    /// assert_eq!(board.erase(here), Err(PlayError::NoPlayersDigit { position: here }));
    /// let given = Position::new(1, 1);
    /// assert_eq!(board.erase(given), Err(PlayError::NoPlayersDigit { position: given }));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn erase(&mut self, position: Position) -> Result<(), PlayError> {
        let cell = cell_in_play(self, position)?;
        let digit = cell.players_digit();
        let before = digit.ok_or(PlayError::NoPlayersDigit { position })?;
        let made = Made::Erase {
            target: position,
            before,
        };
        perform(&mut self.puzzle, &mut self.notes, &made)?;
        self.journal.record(made);
        Ok(())
    }

    /// `WriteMark`: writes the mark `digit` in the note of the cell at `position`,
    /// which must be empty and not a given.
    ///
    /// Writing a mark is a move: it joins the record and discards every undone move.
    /// Nothing asks the mark to be true. As on paper, a player may write a digit a
    /// placed peer rules out.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::NoSuchCell`] when `position` is off the grid;
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::MarksNotAccepted`] when the cell holds a digit, the setter's or
    ///   the player's;
    /// - [`PlayError::DigitOutOfRange`] when `digit` is not from 1 to 9;
    /// - [`PlayError::MarkAlreadyWritten`] when the note already holds the mark.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    ///
    /// board.write_mark(here, 4)?;
    /// assert!(board.cell(here).and_then(|cell| cell.note()).is_some_and(|note| note.contains(4)));
    /// assert_eq!(board.moves().count(), 1);
    ///
    /// let refusal = PlayError::MarkAlreadyWritten { position: here, digit: 4 };
    /// assert_eq!(board.write_mark(here, 4), Err(refusal));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_mark(&mut self, position: Position, digit: u8) -> Result<(), PlayError> {
        let cell = cell_in_play(self, position)?;
        require(
            cell.accepts_marks(),
            PlayError::MarksNotAccepted { position },
        )?;
        require(in_range(digit), PlayError::DigitOutOfRange { digit })?;
        let is_new = !cell.kept_note().contains(digit);
        require(is_new, PlayError::MarkAlreadyWritten { position, digit })?;
        let made = Made::WriteMark {
            target: position,
            digit,
        };
        perform(&mut self.puzzle, &mut self.notes, &made)?;
        self.journal.record(made);
        Ok(())
    }

    /// `StrikeMark`: strikes the mark `digit` from the note of the cell at `position`,
    /// which must be empty and not a given.
    ///
    /// Striking a mark is a move: it joins the record and discards every undone move.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::NoSuchCell`] when `position` is off the grid;
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::MarksNotAccepted`] when the cell holds a digit, the setter's or
    ///   the player's;
    /// - [`PlayError::MarkNotThere`] when the note does not hold the mark, which is so
    ///   for anything that is not a digit from 1 to 9.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    /// board.write_mark(here, 4)?;
    ///
    /// board.strike_mark(here, 4)?;
    /// assert!(board.cell(here).and_then(|cell| cell.note()).is_some_and(|note| note.is_empty()));
    /// assert_eq!(board.moves().count(), 2);
    ///
    /// let refusal = PlayError::MarkNotThere { position: here, digit: 4 };
    /// assert_eq!(board.strike_mark(here, 4), Err(refusal));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn strike_mark(&mut self, position: Position, digit: u8) -> Result<(), PlayError> {
        let cell = cell_in_play(self, position)?;
        require(
            cell.accepts_marks(),
            PlayError::MarksNotAccepted { position },
        )?;
        let is_there = cell.kept_note().contains(digit);
        require(is_there, PlayError::MarkNotThere { position, digit })?;
        let made = Made::StrikeMark {
            target: position,
            digit,
        };
        perform(&mut self.puzzle, &mut self.notes, &made)?;
        self.journal.record(made);
        Ok(())
    }

    /// `Undo`: takes back the latest move that stands, and leaves the board exactly as
    /// it stood before that move: every digit and every note, shown or waiting.
    ///
    /// A placement taken back leaves its cell as it was, digit or none, and gives the
    /// digit back to the notes upkeep struck it from and to no others. An erasure
    /// taken back puts the digit back, a written mark is struck and a struck mark
    /// written. The move stays on the record, undone, until it is re-taken or a new
    /// move discards it. Checks are not moves and are never taken back.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::Solved`] when the puzzle is solved, because solved is final;
    /// - [`PlayError::NothingToUndo`] when no move stands.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let (here, beside) = (Position::new(1, 3), Position::new(1, 4));
    /// board.write_mark(beside, 4)?;
    /// board.place(here, 4)?;
    ///
    /// board.undo()?;
    /// assert_eq!(board.cell(here).and_then(|cell| cell.digit()), None);
    /// // The mark upkeep struck is back in the note it was struck from.
    /// assert!(board.cell(beside).and_then(|cell| cell.note()).is_some_and(|note| note.contains(4)));
    ///
    /// board.undo()?;
    /// assert_eq!(board.undo(), Err(PlayError::NothingToUndo));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn undo(&mut self) -> Result<(), PlayError> {
        require(self.status() == Status::Unsolved, PlayError::Solved)?;
        let latest = self.journal.latest_standing();
        let made = latest.ok_or(PlayError::NothingToUndo)?;
        revert(&mut self.puzzle, &mut self.notes, made)?;
        self.journal.take_back();
        Ok(())
    }

    /// `Redo`: re-takes the earliest undone move, and leaves the board exactly as it
    /// stood after that move was made.
    ///
    /// Nothing has moved since the move was taken back, because a new move would have
    /// discarded it, so what it did then is what it does now.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::NothingToRedo`] when no move is undone.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    /// board.place(here, 4)?;
    /// board.undo()?;
    ///
    /// board.redo()?;
    /// assert_eq!(board.cell(here).and_then(|cell| cell.digit()), Some(4));
    /// assert_eq!(board.redo(), Err(PlayError::NothingToRedo));
    ///
    /// // A new move discards what was undone.
    /// board.undo()?;
    /// board.write_mark(here, 2)?;
    /// assert_eq!(board.redo(), Err(PlayError::NothingToRedo));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn redo(&mut self) -> Result<(), PlayError> {
        require(self.status() == Status::Unsolved, PlayError::Solved)?;
        let next = self.journal.next_to_redo();
        let made = next.ok_or(PlayError::NothingToRedo)?;
        perform(&mut self.puzzle, &mut self.notes, made)?;
        self.journal.retake();
        Ok(())
    }

    /// `CheckCell`: asks whether the digit the player has placed in the cell at
    /// `position` is the solution's there, and hears yes or no.
    ///
    /// The check is kept on the board, in the order asked, and handed back: the digit
    /// it holds is the player's own as it stood, and its answer is
    /// [`Check::is_right`]. It is not a move. It changes no cell and no note, discards
    /// no undone move, and undo never takes it back. No digit of the solution is told,
    /// here or anywhere on a board.
    ///
    /// # Errors
    ///
    /// The first of these that applies, in this order:
    ///
    /// - [`PlayError::NoSuchCell`] when `position` is off the grid;
    /// - [`PlayError::Solved`] when the puzzle is solved;
    /// - [`PlayError::NoPlayersDigit`] when the cell is empty or holds a given.
    ///
    /// ```
    /// use pawdoku::board::{Board, PlayError};
    /// use pawdoku::sudoku::{Given, Position};
    ///
    /// // `givens` are the thirty givens of the example in the module's documentation.
    /// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
    /// #     .bytes()
    /// #     .zip(0_u8..)
    /// #     .filter(|(cell, _)| cell.is_ascii_digit())
    /// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
    /// let mut board = Board::open(givens)?;
    /// let here = Position::new(1, 3);
    ///
    /// // There is nothing to ask about an empty cell.
    /// assert_eq!(board.check(here), Err(PlayError::NoPlayersDigit { position: here }));
    ///
    /// board.place(here, 2)?;
    /// assert!(!board.check(here)?.is_right());
    /// board.place(here, 4)?;
    /// assert!(board.check(here)?.is_right());
    /// assert_eq!(board.checks().count(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn check(&mut self, position: Position) -> Result<Check, PlayError> {
        let cell = cell_in_play(self, position)?;
        let digit = cell.players_digit();
        let digit = digit.ok_or(PlayError::NoPlayersDigit { position })?;
        let is_right = self.puzzle.is_solution_digit(position, digit);
        let index = self.checks.len() + 1;
        let check = Check::new(index, self.journal.standing(), position, digit, is_right);
        self.checks.push(check);
        Ok(check)
    }
}

/// Passes when a `requires` clause `holds`, and refuses with `refusal` when it does not.
const fn require(holds: bool, refusal: PlayError) -> Result<(), PlayError> {
    if holds { Ok(()) } else { Err(refusal) }
}

/// The cell at `position` on a board still in play: what every operation on a cell
/// requires before its own clauses. A position off the grid is refused first, then a
/// solved puzzle.
fn cell_in_play(board: &Board, position: Position) -> Result<BoardCell, PlayError> {
    let cell = board.cell(position);
    let cell = cell.ok_or(PlayError::NoSuchCell { position })?;
    require(board.status() == Status::Unsolved, PlayError::Solved)?;
    Ok(cell)
}

/// Puts `digit` in the puzzle's cell at `position`, or empties the cell: the rules'
/// own two moves, and the only way the board writes a digit.
fn put(puzzle: &mut Puzzle, position: Position, digit: Option<u8>) -> Result<(), MoveError> {
    match digit {
        Some(digit) => puzzle.place(position, digit),
        None => puzzle.erase(position),
    }
}

/// Does what `made` does, to the puzzle and to the notes. The digit goes first, so a
/// move the rules refuse leaves the notes alone.
fn perform(puzzle: &mut Puzzle, notes: &mut Notes, made: &Made) -> Result<(), MoveError> {
    if let Some(change) = made.digit_change() {
        put(puzzle, made.target(), change.after)?;
    }
    made.mark(notes);
    Ok(())
}

/// Takes back what `made` did, to the puzzle and to the notes: the cell's digit is put
/// back to what the move displaced, and the notes to what they held.
fn revert(puzzle: &mut Puzzle, notes: &mut Notes, made: &Made) -> Result<(), MoveError> {
    if let Some(change) = made.digit_change() {
        put(puzzle, made.target(), change.before)?;
    }
    made.unmark(notes);
    Ok(())
}
