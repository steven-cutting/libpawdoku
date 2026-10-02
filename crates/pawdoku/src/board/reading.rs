//! Reading back: the board as it stood once one move and no later one had been made.

use super::journal::Made;
use super::note::{Note, Notes, held, slot};
use crate::sudoku::{Given, Grid, LINE, Position};

/// Every cell's digit and every cell's note at one point in the moves. It starts as
/// the board was opened and goes forward a move at a time; it never goes back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Reading {
    digits: Grid<Option<u8>>,
    notes: Notes,
}

impl Reading {
    /// The board before any move: each given in its cell, every other cell empty and
    /// every note empty.
    pub(super) fn opening<'a>(givens: impl IntoIterator<Item = &'a Given>) -> Self {
        let mut digits = [[None; LINE]; LINE];
        for given in givens {
            if let Some(cell) = slot(&mut digits, given.position()) {
                *cell = Some(given.digit());
            }
        }
        Self {
            digits,
            notes: Notes::empty(),
        }
    }

    /// Goes forward by one move: what the move did when it was made, done here.
    pub(super) fn apply(&mut self, made: &Made) {
        let cell = slot(&mut self.digits, made.target());
        if let (Some(cell), Some(change)) = (cell, made.digit_change()) {
            *cell = change.after;
        }
        made.mark(&mut self.notes);
    }

    /// The digit the cell at `position` held, or nothing where it was empty and where
    /// the grid has no cell.
    pub(super) fn digit_at(&self, position: Position) -> Option<u8> {
        held(&self.digits, position).flatten()
    }

    /// The note the cell at `position` kept, shown or waiting beneath a digit.
    pub(super) fn note_at(&self, position: Position) -> Note {
        self.notes.at(position)
    }
}
