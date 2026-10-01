//! Breaks `effort_imports_sudoku_and_technique` once, with a `use` line naming `lapse`.

use crate::lapse::item as reached;

pub fn item() {}

pub fn sneak() {
    reached();
}
