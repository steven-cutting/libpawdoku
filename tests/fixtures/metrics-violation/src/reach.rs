//! Breaks `reach_imports_sudoku_and_technique` once, with an inline path naming `effort`.

pub fn item() {}

pub fn sneak() {
    crate::effort::item();
}
