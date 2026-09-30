//! Breaks `random_imports_nothing` once, with an inline path naming `sudoku`.

pub fn item() {}

pub fn sneak() {
    crate::sudoku::item();
}
