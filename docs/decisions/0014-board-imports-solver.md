---
title: "Decision 0014: The board imports the solver, and well-posedness is a proof value"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_board_imports_solver]
requires: []
---

# Decision 0014: The board imports the solver, and well-posedness is a proof value

## Context

The crate holds the randomness boundary and no engine module. The first three to be
built are `sudoku`, `solver` and `board`, and they share one design, so it is written
down here once, before any of them, for the tickets that build them to cite.

A board answers a check: whether one cell's digit is the solution's, yes or no
(`CheckCell` in `docs/specs/board.allium`). Only the solver finds a solution. Yet until
this decision the specifications kept the two apart: `board.allium` said the solver "is
not imported", [Layering](../explanation/layering.md) gave `board` the row "imports
`sudoku`", and the boundary rule `board_imports_sudoku` in `rustqual.toml` forbade
`crate::solver` under `src/board`. The migration of `board.allium` left the question
open on purpose, to be settled when the Rust board was planned. A second question sat
beside it: `SetPuzzle` in `sudoku.allium` admits only givens with exactly one solution,
and `sudoku` imports nothing, so the rules cannot ask the solver whether givens are
well-posed.

The maintainer decided both, and the rest of the shared design, on 2026-09-30.

## Decision

1. **Build order.** `sudoku`, then `solver`, then `board`. The grid and the rules are
   `sudoku.allium`'s, and the board is the play layer that stands on them.
2. **The board imports the solver**, and the specification says so first:
   `board.allium` carries a second `use` line, and the layering table and the boundary
   rule, now `board_imports_sudoku_and_solver`, follow it. The edge is safe because
   `board` is a leaf: nothing imports it, so the import closes no cycle. The layering
   page's load-bearing claim is that every model of a player is testable without the
   others and without a generator; `board` is not a model of a player, and no model
   gains an import.
3. **Well-posedness is carried by a proof value.** `sudoku` defines a type, `WellPosed`,
   holding a set of givens and their solution. Only `solver` produces it, and a `Puzzle`
   is built from it, so no `Puzzle` exists whose givens `SetPuzzle` would refuse, and
   `sudoku` never names `crate::solver`. The proof's constructor is `pub(crate)` and
   fallible: it returns a `Result` and never panics. It always checks that there are
   fewer givens than cells; that the solution is full, every digit in range, and free of
   conflict; and that every given is on the grid and matches the solution at its
   position. Uniqueness is the one thing it takes on the solver's word. Rust cannot
   restrict visibility to one sibling module, so "only the solver calls the constructor"
   is held by review inside the crate; outside the crate it is mechanical.
4. **`Puzzle` keeps the solution privately.** It answers one question, yes or no:
   whether one digit is the solution's at one position. That answer is `pub(crate)`, for
   the board's check, and the digits themselves are never exposed through `Puzzle`.
   "Solved" stays the specification's definition, full and consistent, and is not a
   comparison with the stored solution. The solver runs once for each puzzle, as it is
   set.
5. **One solver, not swappable.** There is no trait and no generic parameter for the
   solver, and no optional-solver argument. `solver.allium` has no step budget, because
   its search always concludes, so solving takes the givens alone.
6. **The shape of the API.** `solver` has two public entries: `search`, which returns
   what the `SearchResult` surface exposes (the verdict, the guesses and the solutions
   found, at most two), and `solve`, built on it, which returns the proof or a refusal.
   Whoever holds the proof can read its givens and its solution. A `Puzzle` is set from
   the proof and cannot fail. A `Board` opens from givens in one call and reopens from a
   record in one call, each running the solver itself. The two-step path, solve and then
   set, stays public for code that wants a puzzle without a board.
7. **`Board` hides its puzzle.** A board is the playable abstraction: it owns its
   `Puzzle` and hands out values of its own, never the puzzle, so no question reaches a
   puzzle in play without the board recording it.
8. **The record is a plain value the caller keeps.** There is no storage effect. A record
   holds the givens, the moves, how many are undone, and each check's `after_move`,
   target and digit. Reopening runs the solver, reads the moves forward through the
   board's own guards, derives everything else again and refuses a record that does not
   read back.

What this asked of the specifications is comment text and one `use` line; no entity,
rule, invariant or surface changed. `board.allium` imports `solver.allium` and says why,
and no longer says how undo and redo put a digit back or that reopening re-makes no
move. `sudoku.allium` says that whoever sets a puzzle may bring the solver's verdict and
the one solution with it. `solver.allium` says that a caller handed a search only once
it has concluded has no status to read. Keeping the solution with the puzzle is an
implementation choice the modules already permit: each excludes how anything is stored,
and `solution_digit_at` is a function of the givens whatever keeps its answer.

## Consequences

The layering table no longer shows the board standing on the rules alone. The page now
has to say why a second import is safe, where before the row showed that there was none.

"Only the solver calls the proof's constructor" is a rule review holds, not the
compiler. Any module in the crate can name a `pub(crate)` function; the constructor's
own checks bound the damage, since it can be handed a wrong claim of uniqueness and
nothing else that is wrong.

Every puzzle costs one search as it is set, and every reopening costs one more, because
a record does not carry the solution. A consumer that opens many boards pays for each.

A `Puzzle` cannot be made without the solver, so `sudoku`'s own tests build the proof by
hand, from givens and a solution the test states, through the crate-only constructor.

Nothing outside the crate can ask a `Puzzle` about its solution. Code on the two-step
path reads the digits from the proof before it sets the puzzle, or does without them.

The specifications say less than they did. They no longer say how undo puts a digit
back, so the build may put it back through the puzzle's own placing and erasing, and
`Puzzle` keeps one way to write a cell; that rests on placing and erasing doing nothing
but set a cell's digit, and is looked at again if either ever gains another effect. And
a search's status is something no caller reads: the `SearchResult` surface still exposes
it, and the function that returns a search returns only a concluded one.

The game restates clauses of these three modules and holds its text equal to this
repository's by test, so each changed sentence is a change the game must take.

## What would reopen this

A second solver, or a need to swap one in, which would bring back the trait or the
parameter that decision 5 leaves out. A module that must import the board, since the
edge is safe only while the board is a leaf. A consumer that needs the solution through
a `Puzzle`, which would reopen what decision 4 keeps private. Or a measured cost of
solving at reopening that a stored solution would remove, which would put the solution
in the record and reopen decision 8.

## Related pages

- [Layering and dependency direction](../explanation/layering.md)
- [Architecture](../explanation/architecture.md)
- [Specifications](../explanation/specifications.md)
- [Decision 0013: The metrics gate](0013-metrics-gate.md)
