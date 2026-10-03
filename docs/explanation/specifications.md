---
title: "Specifications"
kind: "explanation"
audience: [contributor, maintainer, agent]
canonical_for: [specification_model]
requires: []
---

# Specifications

This library's behaviour is written down, in a formal language, before it is built. The
nine Allium modules under `docs/specs/` are the source of truth for what the engine
does. This handbook, the code and the tests all answer to them.

The procedure is in [Work with the specifications](../how-to/work-with-the-specs.md).
This page is why.

## What a specification is for

A figure looks like a detail and is not. The side of the grid is stated once, in
`sudoku.allium`'s `config` block, as `side`, the square of `box_side`.
`pawdoku::sudoku::SIDE` and `pawdoku::sudoku::BOX_SIDE` mirror the two once, and a
doctest and a unit test in `crates/pawdoku/src/sudoku.rs` hold the mirror, so a grid of
the wrong size fails on the number rather than on a reviewer's eye.
Left to prose, a figure like that drifts. Stated as a contract with a named guarantee, it
is testable.

## What the modules are

Nine are the library's. `sudoku.allium` states the rules of classic Sudoku — the grid, what a setter
may pose, what a player may do and when a puzzle is solved — and nothing about how any
of it looks, so it imports nothing; a consumer that draws the rules imports this module
beside its own.
`board.allium` states the puzzle in play, which the rules leave out: a note in every
cell, the four moves a player makes and their exact undo and redo, the board as it stood
after any move, the check of one cell against the solution, and the record a board is
written to and had again from. It imports the rules and the solver: every placement and
erasure still goes to the rules' own moves, and the check reads the one solution the
solver finds. It states a `contract`, as `human-solving.allium` does, which names what
writing a record and reopening it must satisfy and no form for either.
`solver.allium` states what `sudoku.allium` leaves a black box: the search that gives a
set of givens its verdict — no solution, one or many — and the work that search may be
seen to do, and nothing about how it is stored or made fast. It imports the rules and is
not imported by them.

Five more say how a person solves a puzzle, which `solver.allium` excludes.
`technique.allium` states the named techniques once — when each holds on a grid of digits
and candidates and what it places or strikes — and the profile of a player: what they
know, how much they hold in mind, how much of the grid they take in, and whether they
keep marks. Four models stand on it and never import each other, because
each is a different answer to what a limit does. In `reach.allium` a limit hides a
deduction, so a puzzle is within a player's reach or it is not, and the next thing that
player would find is a hint pitched at them. In `effort.allium` a limit makes a deduction
dear, and a puzzle is priced end to end. In `lapse.allium` a limit lets marks fall
behind until the player guesses, and a guess is the one thing that can be wrong.
In `human-solving.allium` a limit changes what is noticed, kept in mind and got wrong,
by chance: it specifies independently tunable cognition, experience, aids, fallible
microsteps and seeded repeated assessments, and the projection from its finer
description of a player to `technique.allium`'s profile, so that one player can be put
through all four and the results compared.
`technique.allium` defines 29 techniques, including bounded chains and explicit
uniqueness premises, and records how eight of the nine questions it once left open
were resolved; partial marking and the loads past subsets remain open.
`lapse.allium` records the same of its own seven, each answer deliberately simpler than
`human-solving.allium`'s, and leaves one open: whether a player stuck under a guess with
no contradiction in sight should withdraw it. `effort.allium` remains coarse and retains its open questions,
which name the answer `human-solving.allium` gives where it gives one. Its [owning explanation](human-solving.md) states the
research basis and provisional parameter choices. These are behavioural contracts
for a future simulator; passing the Allium gates is not an empirical validation.

The ninth, `generation.allium`, is the first module written here rather than carried
from the game, and the only one that stands on a model: it says how a setter that is a
program finds givens, drawing a solution grid through the randomness boundary and
removing givens by symmetry orbit until what is left is well-posed and solvable within a
technique contract, under a step budget. That is the designed way, and it is a skeleton,
scope, config and open questions with no trigger, because every figure it would state is
a product decision nobody has taken; [Puzzle design objectives](puzzle-design.md) says
what it is for and [decision 0012](../decisions/0012-generation-and-dev-time-judges.md)
why it is the engine's. Beside it the module states the basic way in full, with its
rules, invariants and surfaces: a published method that draws a solution grid from eleven
givens and removes givens one position at a time in the order its tier names, keeping
each removal only while the solver's verdict stays one
([decision 0015](../decisions/0015-basic-generator.md)).

The game's root module, `pawdoku.allium`, stays in the game. Allium has no
cross-repository import, so the game restates the clauses it needs and holds them equal
to this repository's text by a test on its own side. Until the WebAssembly package ships
the modules inside it, that text is shared truth across two repositories and can drift
silently: a change to a module here is a change the game has to see.

## Open questions are a feature

An `open question` block records a product decision nobody has made yet. They are
recorded rather than resolved on purpose: an unwritten gap gets filled in by whoever
writes the code first, silently and invisibly, while a written one has to be answered by
someone entitled to answer it.

A low count is the normal state of a settled module, not a reason to stop using the
construct. A change that reaches a decision nobody has taken should add one rather than
guess.

The open questions in `effort.allium` and `human-solving.allium` were carried from the
game as they stood; the modules are where they live and where they will be answered.

## What a specification is not

It is not a design document, and it does not choose a language, a framework, a storage
mechanism or a layout. A module states what any implementation must satisfy and describes
no scheme for satisfying it. How is this repository's business; `AGENTS.md` decides that.

## Related pages

- [Work with the specifications](../how-to/work-with-the-specs.md)
- [Modelling a human Sudoku solver](human-solving.md)
- [Decision 0002](../decisions/0002-specs-are-the-source-of-truth.md)
