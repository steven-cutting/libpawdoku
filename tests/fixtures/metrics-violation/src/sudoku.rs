//! Breaks `sudoku_imports_nothing` once, with a `use` line naming `technique`.

use crate::technique::item as reached;

pub fn item() {}

pub fn sneak() {
    reached();
}
