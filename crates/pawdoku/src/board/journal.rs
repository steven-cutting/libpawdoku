//! The moves as the board keeps them: what each one did, in the order made, and how
//! many of them stand.

use super::moves::{Move, MoveKind};
use super::note::Notes;
use super::reading::Reading;
use crate::sudoku::Position;
use alloc::vec::Vec;

/// One move as it was made, with what it displaced, so that it can be taken back and
/// re-taken exactly. Each kind carries what it needs and nothing else, which is
/// `MovesCarryWhatTheirKindNeeds` held by the type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Made {
    /// A digit placed. `before` is what the cell held, and `struck` the peers whose
    /// note lost the digit to upkeep.
    Place {
        target: Position,
        digit: u8,
        before: Option<u8>,
        struck: Vec<Position>,
    },
    /// A digit erased. `before` is the digit that stood.
    Erase { target: Position, before: u8 },
    /// A mark written in the target's note.
    WriteMark { target: Position, digit: u8 },
    /// A mark struck from the target's note.
    StrikeMark { target: Position, digit: u8 },
}

/// What a placement or an erasure does to its cell: the digit the cell held before the
/// move, and the digit it holds once the move is made, each nothing for an empty cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DigitChange {
    pub(super) before: Option<u8>,
    pub(super) after: Option<u8>,
}

impl Made {
    /// Which of the four moves this is.
    pub(super) const fn kind(&self) -> MoveKind {
        match self {
            Self::Place { .. } => MoveKind::Place,
            Self::Erase { .. } => MoveKind::Erase,
            Self::WriteMark { .. } => MoveKind::WriteMark,
            Self::StrikeMark { .. } => MoveKind::StrikeMark,
        }
    }

    /// The cell the move was made on.
    pub(super) const fn target(&self) -> Position {
        match self {
            Self::Place { target, .. }
            | Self::Erase { target, .. }
            | Self::WriteMark { target, .. }
            | Self::StrikeMark { target, .. } => *target,
        }
    }

    /// The digit placed, written or struck, and nothing for an erasure.
    pub(super) const fn digit(&self) -> Option<u8> {
        match self {
            Self::Place { digit, .. }
            | Self::WriteMark { digit, .. }
            | Self::StrikeMark { digit, .. } => Some(*digit),
            Self::Erase { .. } => None,
        }
    }

    /// What the move does to its target's digit, for a placement and an erasure. A move
    /// on a note changes no digit.
    pub(super) const fn digit_change(&self) -> Option<DigitChange> {
        match self {
            Self::Place { digit, before, .. } => Some(DigitChange {
                before: *before,
                after: Some(*digit),
            }),
            Self::Erase { before, .. } => Some(DigitChange {
                before: Some(*before),
                after: None,
            }),
            Self::WriteMark { .. } | Self::StrikeMark { .. } => None,
        }
    }

    /// What the move does to the notes: upkeep strikes a placed digit from the peers
    /// recorded, a written mark is written and a struck mark struck.
    pub(super) fn mark(&self, notes: &mut Notes) {
        match self {
            Self::Place { digit, struck, .. } => {
                for &peer in struck {
                    notes.strike(peer, *digit);
                }
            }
            Self::Erase { .. } => {}
            Self::WriteMark { target, digit } => notes.write(*target, *digit),
            Self::StrikeMark { target, digit } => notes.strike(*target, *digit),
        }
    }

    /// The exact reverse of [`Self::mark`]: the placed digit goes back to the notes
    /// upkeep struck it from and no others, a written mark is struck and a struck mark
    /// written.
    pub(super) fn unmark(&self, notes: &mut Notes) {
        match self {
            Self::Place { digit, struck, .. } => {
                for &peer in struck {
                    notes.write(peer, *digit);
                }
            }
            Self::Erase { .. } => {}
            Self::WriteMark { target, digit } => notes.strike(*target, *digit),
            Self::StrikeMark { target, digit } => notes.write(*target, *digit),
        }
    }
}

/// Every move on the record, in the order made. The first `standing` of them stand and
/// the rest are undone, so the undone moves are always the latest
/// (`UndoneMovesAreTheLatest`) and a move's index is its place in the list
/// (`MoveIndicesRunFromOne`, `MoveIndicesAreDistinct`).
#[derive(Debug, Clone)]
pub(super) struct Journal {
    made: Vec<Made>,
    standing: usize,
}

impl Journal {
    /// The record of a board just opened: no move.
    pub(super) const fn new() -> Self {
        Self {
            made: Vec::new(),
            standing: 0,
        }
    }

    /// How many moves stand.
    pub(super) const fn standing(&self) -> usize {
        self.standing
    }

    /// Whether any move is undone.
    pub(super) const fn has_undone(&self) -> bool {
        self.standing < self.made.len()
    }

    /// A new move joins the record: every undone move is discarded, and the move takes
    /// the next index, one past the standing count.
    pub(super) fn record(&mut self, made: Made) {
        self.made.truncate(self.standing);
        self.made.push(made);
        self.standing = self.made.len();
    }

    /// The move undo takes back: the latest that stands.
    pub(super) fn latest_standing(&self) -> Option<&Made> {
        self.made.get(self.standing.checked_sub(1)?)
    }

    /// The move redo re-takes: the earliest that is undone, which comes right after the
    /// standing ones.
    pub(super) fn next_to_redo(&self) -> Option<&Made> {
        self.made.get(self.standing)
    }

    /// Marks the latest standing move undone. It keeps its place on the record.
    pub(super) const fn take_back(&mut self) {
        self.standing = self.standing.saturating_sub(1);
    }

    /// Marks the earliest undone move standing again.
    pub(super) const fn retake(&mut self) {
        if self.has_undone() {
            self.standing += 1;
        }
    }

    /// Every move as the board hands it out, each with the board as it stood once that
    /// move and no later one had been made. The readings are worked out here, from
    /// `opening` forward through the moves, and nothing is taken back to make them.
    pub(super) fn moves(&self, opening: Reading) -> impl Iterator<Item = Move> + '_ {
        let numbered = (1..).zip(&self.made);
        numbered.scan(opening, |reading, (index, made)| {
            reading.apply(made);
            let is_undone = index > self.standing;
            Some(Move::new(index, made, is_undone, reading.clone()))
        })
    }
}
