---
title: "Decision 0001: The engine is a library of its own"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_engine_as_a_library]
requires: []
---

# Decision 0001: The engine is a library of its own

This record will explain why the sudoku engine left the game for a Cargo workspace of
its own, with the core crate first and the command line, Python and WebAssembly crates
as siblings later. It is a new record, with no Pawdoku counterpart, written by lane T09.
