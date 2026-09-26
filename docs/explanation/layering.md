---
title: "Layering and dependency direction"
kind: "explanation"
audience: [contributor, maintainer, agent]
canonical_for: [dependency_boundaries]
requires: []
---

# Layering and dependency direction

This page will state which module may depend on which, and why the direction in the code
mirrors the import graph of the specifications, so that a lower module never reaches up
into a higher one. That graph has eight modules; `board.allium` imports `sudoku.allium`
alone and nothing imports it. It is a shorter adaptation of the Pawdoku page of the same
path, written by lane T08.
