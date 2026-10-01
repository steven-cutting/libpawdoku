//! Breaks `technique_imports_sudoku` once, from a child module, with an inline path
//! naming `solver`: the rule's `/**` arm holds a child to its parent's row.

pub fn sneak() {
    crate::solver::item();
}
