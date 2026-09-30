//! Breaks `technique_imports_sudoku` once, with an inline path naming `solver`.

pub fn item() {}

pub fn sneak() {
    crate::solver::item();
}
