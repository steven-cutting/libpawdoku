//! The metrics probe's fixture: one file per module of the layering table in
//! docs/explanation/layering.md, each naming one module its rule in rustqual.toml
//! forbids. `just metrics` requires rustqual to fail here and to name every rule.
//! Odd files break their rule with a `use` line and even files with an inline path,
//! so both forms stay proved. Never compiled: no manifest declares it.

pub mod sudoku;
pub mod technique;
pub mod solver;
pub mod reach;
pub mod effort;
pub mod lapse;
pub mod human_solving;
pub mod board;
pub mod generation;
pub mod random;
