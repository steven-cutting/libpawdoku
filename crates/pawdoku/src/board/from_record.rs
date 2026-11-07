//! Play from a record: `board.allium`'s `PlayingFromARecord`, where a caller holds no
//! board and hands each call a record.
//!
//! Every function here takes values and returns values, and nothing is kept between
//! calls. Each composes what a held [`Board`] already does: [`Board::open`] or
//! [`Board::reopen`], the board's own operation under its own guards, and
//! [`Board::write`]. So a caller that plays from records and a caller that holds one
//! board see the same records, views, readings and refusals, step for step.

use super::{Board, BoardView, Check, PlayError, ReadingAfter, Record, ReopenError};
use crate::solver::SolveError;
use crate::sudoku::{Given, Position};

/// One of `Playing`'s seven operations, as a caller hands it over with a record:
/// `board.allium`'s `Operation`, one variant for each `OperationKind`.
///
/// Each variant carries what the board's operation of that kind takes and nothing
/// else, so every operation is one `Playing` offers in form, and only the board's
/// guard refuses it. Under the `serde` feature it serialises and deserialises tagged
/// by `kind`, as `{ "kind": "Undo" }` or `{ "kind": "Place", "position": { "row": 1,
/// "column": 3 }, "digit": 4 }`.
///
/// ```
/// use pawdoku::board::{Operation, open, play};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let opened = open(givens)?;
/// let place = Operation::Place { position: Position::new(1, 3), digit: 4 };
/// let played = play(opened.record(), place)?;
/// assert_eq!(played.view().moves().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind")
)]
#[non_exhaustive]
pub enum Operation {
    /// [`Board::place`]: `digit` in the cell at `position`.
    Place {
        /// The cell to place in.
        position: Position,
        /// The digit to place.
        digit: u8,
    },
    /// [`Board::erase`]: the player's digit in the cell at `position`.
    Erase {
        /// The cell to erase.
        position: Position,
    },
    /// [`Board::write_mark`]: the mark `digit` in the note at `position`.
    WriteMark {
        /// The cell whose note takes the mark.
        position: Position,
        /// The mark.
        digit: u8,
    },
    /// [`Board::strike_mark`]: the mark `digit` from the note at `position`.
    StrikeMark {
        /// The cell whose note loses the mark.
        position: Position,
        /// The mark.
        digit: u8,
    },
    /// [`Board::check`]: whether the player's digit at `position` is the solution's.
    CheckCell {
        /// The cell to check.
        position: Position,
    },
    /// [`Board::undo`]: the latest move that stands, taken back.
    Undo,
    /// [`Board::redo`]: the earliest undone move, re-taken.
    Redo,
}

impl Operation {
    /// Makes the operation on `board` through the board's own method, and hands back
    /// the check it added when it is a check.
    fn make(self, board: &mut Board) -> Result<Option<Check>, PlayError> {
        match self {
            Self::Place { position, digit } => board.place(position, digit).map(|()| None),
            Self::Erase { position } => board.erase(position).map(|()| None),
            Self::WriteMark { position, digit } => board.write_mark(position, digit).map(|()| None),
            Self::StrikeMark { position, digit } => {
                board.strike_mark(position, digit).map(|()| None)
            }
            Self::CheckCell { position } => board.check(position).map(Some),
            Self::Undo => board.undo().map(|()| None),
            Self::Redo => board.redo().map(|()| None),
        }
    }
}

/// What [`open`] and [`play`] hand back: the record the caller keeps for the next call,
/// the board after the call as `Playing` exposes it, and, after a check, the check it
/// added.
///
/// Under the `serde` feature it serialises as `{ record, view, check }`, with `check`
/// left out unless the operation was [`Operation::CheckCell`].
///
/// ```
/// use pawdoku::board::{Operation, open, play};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let here = Position::new(1, 3);
/// let opened = open(givens)?;
/// let placed = play(opened.record(), Operation::Place { position: here, digit: 4 })?;
/// assert_eq!(placed.check(), None);
///
/// let checked = play(placed.record(), Operation::CheckCell { position: here })?;
/// assert!(checked.check().is_some_and(|check| check.is_right()));
/// assert_eq!(checked.view().checks().len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
pub struct Played {
    record: Record,
    view: BoardView,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    check: Option<Check>,
}

impl Played {
    /// The record [`Board::write`] gives of the board after the call. The caller keeps
    /// it and hands it to the next call.
    #[must_use]
    pub const fn record(&self) -> &Record {
        &self.record
    }

    /// The board after the call, as `Playing` exposes it.
    #[must_use]
    pub const fn view(&self) -> &BoardView {
        &self.view
    }

    /// The check the call added, when the operation was [`Operation::CheckCell`]: the
    /// last of the view's checks. Nothing otherwise.
    #[must_use]
    pub const fn check(&self) -> Option<&Check> {
        self.check.as_ref()
    }

    /// What `board` shows and writes, with the check that was added, if one was.
    fn of(board: &Board, check: Option<Check>) -> Self {
        Self {
            record: board.write(),
            view: BoardView::of(board),
            check,
        }
    }
}

/// Why a call of play from a record was refused: the record did not reopen, the board
/// refused the operation, or the index names no move.
///
/// A refused call hands back nothing else, and the record handed over is unchanged. A
/// record that does not reopen is refused first, since until it reopens there is no
/// board to ask. The `Display` text of each variant is stable API, and the two that
/// wrap a stage's refusal show that refusal's text unchanged, so a refusal reads the
/// same whichever way the board is driven.
///
/// ```
/// use pawdoku::board::{FromRecordError, Operation, PlayError, open, play, reading_after};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let record = open(givens)?.record().clone();
///
/// let refusal = play(&record, Operation::Undo).unwrap_err();
/// assert_eq!(refusal, FromRecordError::Play(PlayError::NothingToUndo));
/// assert_eq!(refusal.to_string(), PlayError::NothingToUndo.to_string());
///
/// let refusal = reading_after(&record, 1).unwrap_err();
/// assert_eq!(refusal.to_string(), "no move of the board has the index 1");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FromRecordError {
    /// The record did not reopen, refused as [`Board::reopen`] refuses it.
    #[error(transparent)]
    Reopen(#[from] ReopenError),
    /// The board the record reopens to refused the operation, by its guard.
    #[error(transparent)]
    Play(#[from] PlayError),
    /// No move of the board has the index asked for: moves count from 1, undone moves
    /// included.
    #[error("no move of the board has the index {move_index}")]
    NoSuchMove {
        /// The index that was asked for.
        move_index: usize,
    },
}

/// `PlayingFromARecord.open`: opens a board from `givens`, as [`Board::open`] does, and
/// hands back its record and its view.
///
/// # Errors
///
/// What [`Board::open`] refuses, as it refuses it: a [`SolveError`] when the givens
/// have no solution, more than one, or leave nothing to play.
///
/// ```
/// use pawdoku::board::open;
/// use pawdoku::solver::SolveError;
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let opened = open(givens)?;
/// assert_eq!(opened.view().cells().len(), 81);
/// assert!(opened.view().moves().is_empty());
///
/// assert_eq!(open([]).unwrap_err(), SolveError::ManySolutions);
/// # Ok::<(), SolveError>(())
/// ```
pub fn open(givens: impl IntoIterator<Item = Given>) -> Result<Played, SolveError> {
    let board = Board::open(givens)?;
    Ok(Played::of(&board, None))
}

/// `PlayingFromARecord.play`: reopens `record`, makes `operation` on the board through
/// the board's own method, and hands back the board's record and view after it, with
/// the check it added for a check.
///
/// The record handed over is borrowed and unchanged, refused or not.
///
/// # Errors
///
/// The first of these that applies, in this order:
///
/// - [`FromRecordError::Reopen`] when the record does not reopen;
/// - [`FromRecordError::Play`] when the board refuses the operation.
///
/// ```
/// use pawdoku::board::{FromRecordError, Operation, PlayError, open, play};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let here = Position::new(1, 3);
/// let record = open(givens)?.record().clone();
///
/// let placed = play(&record, Operation::Place { position: here, digit: 4 })?;
/// assert_eq!(placed.view().cells()[2].digit(), Some(4));
/// let undone = play(placed.record(), Operation::Undo)?;
/// assert!(undone.view().can_redo());
///
/// // A given is never the player's to erase.
/// let given = Position::new(1, 1);
/// let refusal = FromRecordError::Play(PlayError::NoPlayersDigit { position: given });
/// assert_eq!(play(&record, Operation::Erase { position: given }), Err(refusal));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn play(record: &Record, operation: Operation) -> Result<Played, FromRecordError> {
    let mut board = Board::reopen(record)?;
    let check = operation.make(&mut board)?;
    Ok(Played::of(&board, check))
}

/// `PlayingFromARecord.read`: the view of the board `record` reopens to.
///
/// # Errors
///
/// What [`Board::reopen`] refuses, as it refuses it: a [`ReopenError`].
///
/// ```
/// use pawdoku::board::{Board, BoardView, read};
/// use pawdoku::sudoku::{Given, Position};
///
/// // `givens` are the thirty givens of the example in the module's documentation.
/// # let givens = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
/// #     .bytes()
/// #     .zip(0_u8..)
/// #     .filter(|(cell, _)| cell.is_ascii_digit())
/// #     .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'));
/// let mut board = Board::open(givens)?;
/// board.write_mark(Position::new(1, 3), 4)?;
///
/// assert_eq!(read(&board.write())?, BoardView::of(&board));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn read(record: &Record) -> Result<BoardView, ReopenError> {
    let board = Board::reopen(record)?;
    Ok(BoardView::of(&board))
}

/// `PlayingFromARecord.reading_after`: every cell of the board `record` reopens to, as
/// the move of index `move_index` left it, counted from 1, undone moves included.
///
/// # Errors
///
/// The first of these that applies, in this order:
///
/// - [`FromRecordError::Reopen`] when the record does not reopen;
/// - [`FromRecordError::NoSuchMove`] when no move has that index.
///
/// ```
/// use pawdoku::board::{Board, ReadingAfter, reading_after};
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
/// board.erase(Position::new(1, 3))?;
///
/// let reading = reading_after(&board.write(), 1)?;
/// assert_eq!(reading.cells()[2].digit(), Some(4));
/// assert_eq!(reading, ReadingAfter::of(&board, 1)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn reading_after(record: &Record, move_index: usize) -> Result<ReadingAfter, FromRecordError> {
    let board = Board::reopen(record)?;
    ReadingAfter::of(&board, move_index)
}

#[cfg(test)]
mod tests {
    use super::{FromRecordError, Operation, open, play, read, reading_after};
    use crate::board::journal::{Journal, Made};
    use crate::board::{Board, BoardView, Check, PlayError, ReadingAfter, Record, ReopenError};
    use crate::solver::SolveError;
    use crate::sudoku::{Given, Position};
    use alloc::collections::BTreeSet;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    /// The rules' fixture: thirty givens, one solution, rows top to bottom.
    const FIXTURE: &str =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";

    /// A cell the fixture leaves to the player. Its solution digit is 4.
    const FREE: Position = Position::new(1, 3);

    /// A second cell left to the player, in the row of [`FREE`].
    const BESIDE: Position = Position::new(1, 4);

    /// A cell the fixture gives: a 5.
    const GIVEN: Position = Position::new(1, 1);

    /// The filled cells of a picture read a row after a row, as givens.
    fn givens(picture: &str) -> BTreeSet<Given> {
        let cells = picture.bytes().zip(0_u8..);
        cells
            .filter(|(cell, _)| cell.is_ascii_digit())
            .map(|(cell, at)| Given::new(Position::new(at / 9 + 1, at % 9 + 1), cell - b'0'))
            .collect()
    }

    /// Makes `operation` on a held board, as `Playing` offers it.
    fn on_a_held_board(
        board: &mut Board,
        operation: Operation,
    ) -> Result<Option<Check>, PlayError> {
        match operation {
            Operation::Place { position, digit } => board.place(position, digit).map(|()| None),
            Operation::Erase { position } => board.erase(position).map(|()| None),
            Operation::WriteMark { position, digit } => {
                board.write_mark(position, digit).map(|()| None)
            }
            Operation::StrikeMark { position, digit } => {
                board.strike_mark(position, digit).map(|()| None)
            }
            Operation::CheckCell { position } => board.check(position).map(Some),
            Operation::Undo => board.undo().map(|()| None),
            Operation::Redo => board.redo().map(|()| None),
        }
    }

    /// Every kind of operation, each one the board before it offers: marks written and
    /// struck, a placement checked, erased, placed again, undone and redone.
    const EVERY_KIND: [Operation; 9] = [
        Operation::WriteMark {
            position: FREE,
            digit: 2,
        },
        Operation::WriteMark {
            position: BESIDE,
            digit: 7,
        },
        Operation::StrikeMark {
            position: BESIDE,
            digit: 7,
        },
        Operation::Place {
            position: FREE,
            digit: 9,
        },
        Operation::CheckCell { position: FREE },
        Operation::Erase { position: FREE },
        Operation::Place {
            position: FREE,
            digit: 4,
        },
        Operation::Undo,
        Operation::Redo,
    ];

    /// A record that does not reopen: on the fixture's givens, it erases a given.
    fn that_does_not_reopen() -> Record {
        let board = Board::open(givens(FIXTURE)).unwrap();
        let mut journal = Journal::new();
        journal.record(Made::Erase {
            target: GIVEN,
            before: 5,
        });
        Record::of(&board.puzzle, &journal, &[])
    }

    #[test]
    fn each_refusal_says_what_was_refused() {
        let no_move = FromRecordError::NoSuchMove { move_index: 3 };
        assert_eq!(no_move.to_string(), "no move of the board has the index 3");
        // The two stages' refusals read as their own, whichever way the board is driven.
        let played = PlayError::NothingToUndo;
        assert_eq!(
            FromRecordError::from(played.clone()).to_string(),
            played.to_string()
        );
        let reopened = Board::reopen(&that_does_not_reopen()).unwrap_err();
        assert_eq!(
            FromRecordError::from(reopened.clone()).to_string(),
            reopened.to_string()
        );
    }

    #[test]
    fn open_hands_back_the_record_and_view_of_the_board_opened() {
        let board = Board::open(givens(FIXTURE)).unwrap();
        let opened = open(givens(FIXTURE)).unwrap();
        assert_eq!(opened.record(), &board.write());
        assert_eq!(opened.view(), &BoardView::of(&board));
        assert_eq!(opened.check(), None);
        // Givens the solver refuses are refused as Board::open refuses them.
        assert_eq!(open([]).unwrap_err(), SolveError::ManySolutions);
    }

    #[test]
    fn play_makes_each_operation_as_a_held_board_does() {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        let mut record = open(givens(FIXTURE)).unwrap().record().clone();
        for operation in EVERY_KIND {
            let asked = on_a_held_board(&mut board, operation).unwrap();
            let played = play(&record, operation).unwrap();
            assert_eq!(played.record(), &board.write());
            assert_eq!(played.view(), &BoardView::of(&board));
            assert_eq!(played.check(), asked.as_ref());
            assert_eq!(
                played.check().is_some(),
                matches!(operation, Operation::CheckCell { .. })
            );
            record = played.record().clone();
        }
        // The game is played out: the redone placement stands and the check is kept.
        assert_eq!(board.cell(FREE).unwrap().digit(), Some(4));
        assert_eq!(read(&record).unwrap().checks().len(), 1);
    }

    #[test]
    fn a_record_that_does_not_reopen_is_refused_first() {
        let record = that_does_not_reopen();
        let kept = record.clone();
        let refusal = FromRecordError::Reopen(Board::reopen(&record).unwrap_err());
        // Undo would also be refused, by the guard, and index 1 names no move; the
        // record's refusal comes before either.
        assert_eq!(play(&record, Operation::Undo).unwrap_err(), refusal);
        assert_eq!(reading_after(&record, 1).unwrap_err(), refusal);
        assert_eq!(read(&record), Err(Board::reopen(&record).unwrap_err()));
        assert!(matches!(
            refusal,
            FromRecordError::Reopen(ReopenError::Move { .. })
        ));
        assert_eq!(record, kept);
    }

    #[test]
    fn an_operation_the_guard_refuses_is_refused_with_its_play_error() {
        let record = Board::open(givens(FIXTURE)).unwrap().write();
        let kept = record.clone();
        let refusals = [
            (
                Operation::Place {
                    position: GIVEN,
                    digit: 6,
                },
                PlayError::GivenCell { position: GIVEN },
            ),
            (
                Operation::Erase { position: FREE },
                PlayError::NoPlayersDigit { position: FREE },
            ),
            (Operation::Undo, PlayError::NothingToUndo),
            (Operation::Redo, PlayError::NothingToRedo),
        ];
        for (operation, refusal) in refusals {
            assert_eq!(
                play(&record, operation),
                Err(FromRecordError::Play(refusal))
            );
        }
        assert_eq!(record, kept);
    }

    #[test]
    fn read_gives_the_view_of_the_board_reopened() {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        for operation in EVERY_KIND {
            on_a_held_board(&mut board, operation).unwrap();
            assert_eq!(read(&board.write()), Ok(BoardView::of(&board)));
        }
    }

    #[test]
    fn reading_after_reads_the_board_reopened() {
        let mut board = Board::open(givens(FIXTURE)).unwrap();
        for operation in EVERY_KIND {
            on_a_held_board(&mut board, operation).unwrap();
        }
        let record = board.write();
        let indices: Vec<usize> = board.moves().map(|made| made.index()).collect();
        assert_eq!(indices.len(), 6);
        for move_index in indices {
            assert_eq!(
                reading_after(&record, move_index),
                ReadingAfter::of(&board, move_index)
            );
        }
        assert_eq!(
            reading_after(&record, 7),
            Err(FromRecordError::NoSuchMove { move_index: 7 })
        );
    }
}
