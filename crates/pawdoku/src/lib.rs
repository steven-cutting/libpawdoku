//! The Pawdoku engine: classic sudoku, specified in `docs/specs/` and built here.
//!
//! This crate is `no_std` with `alloc`: it has no clock, no threads, no filesystem and
//! no source of randomness of its own. What it needs from the outside world it takes
//! through traits, so that it runs the same in a browser, in Python and on the command
//! line. Randomness is the one effect, and [`random`] is its boundary: a seeded stream
//! of draws the caller supplies. See `docs/explanation/architecture.md`.
//!
//! Behaviour arrives one module to a specification module. [`sudoku`] is the first: the
//! rules, from `docs/specs/sudoku.allium`, beneath every module to come. It holds the
//! grid's two figures, the value types, and the puzzle in play with its two moves.
//! [`solver`] is the second, from `docs/specs/solver.allium`: the search that gives a set
//! of givens its verdict of none, one or many solutions. A puzzle is made in two steps,
//! [`solver::solve`] and then [`sudoku::Puzzle::set`], because only the solver can show
//! that givens have exactly one solution. [`board`] is the third, from
//! `docs/specs/board.allium`: one puzzle as it is being played. A [`board::Board`] opens
//! from givens in one call and is what a player plays on: it keeps a note in every
//! cell and every move made, takes the latest move back and re-takes it, reads back
//! the board as it stood after any move, and answers a check of one cell yes or no. It
//! is written down as a [`board::Record`], a plain value the caller keeps, and reopened
//! from one as the same board.
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
//! board.place(Position::new(1, 3), 4)?;
//! assert!(board.check(Position::new(1, 3))?.is_right());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! **Not final.** This is a draft: the public items, their names and their signatures
//! may change as implementation continues.

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod board;
pub mod random;
pub mod solver;
pub mod sudoku;
