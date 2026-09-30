//! Holds the child that breaks `technique_imports_sudoku`, so the probe proves the
//! rule's `/**` arm; this file breaks nothing, so the rule's `.rs` arm is not proved.

pub mod catalogue;

pub fn item() {}
