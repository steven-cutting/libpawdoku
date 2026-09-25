//! The Pawdoku engine: classic sudoku, specified in `docs/specs/` and built here.
//!
//! This crate is `no_std` with `alloc`: it has no clock, no threads, no filesystem and
//! no source of randomness of its own. What it needs from the outside world it takes
//! through traits, so that it runs the same in a browser, in Python and on the command
//! line. See `docs/explanation/architecture.md`.

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

/// The side of a classic sudoku grid: nine cells to a row, a column and a box.
///
/// ```
/// assert_eq!(pawdoku::SIDE, 9);
/// ```
pub const SIDE: u8 = 9;

#[cfg(test)]
mod tests {
    use super::SIDE;

    #[test]
    fn a_grid_is_nine_by_nine() {
        assert_eq!(usize::from(SIDE) * usize::from(SIDE), 81);
    }
}
