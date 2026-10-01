---
id: T25
title: "The rules in Rust: the sudoku module, the proof value and Puzzle"
status: open
depends_on: [T23, T24]
parallel_with: []
branch: ticket/t25-sudoku-rules
estimated_size: L
---

# T25: The rules in Rust: the sudoku module, the proof value and Puzzle

## Context

`crates/pawdoku` holds the randomness boundary (`src/random.rs`) and one constant,
`SIDE`. This ticket builds the first engine module, `sudoku`, from
`docs/specs/sudoku.allium`: the grid, what a setter may pose, the two moves a player has,
conflicts, and when a puzzle is solved. T26 builds the solver on it, and T27 and T28 the
board.

The design was decided by the maintainer on 2026-09-30 and is recorded in the decision
T23 wrote (`docs/decisions/0014-board-imports-solver.md`, unless T23 had to renumber it).
What this ticket needs from it:

- **Well-posedness is carried by a proof value.** `SetPuzzle` requires fewer givens than
  cells and exactly one solution. `sudoku` cannot decide the second: `solution_count` is
  a black box there, and the module imports nothing. So `sudoku` defines a type,
  `WellPosed`, that holds a set of givens and their solution, and a `Puzzle` is built
  from one. Only the solver (T26) produces a `WellPosed`. No `Puzzle` can exist whose
  givens `SetPuzzle` would refuse, and `sudoku` never names `crate::solver`.
- **The proof's constructor is `pub(crate)` and fallible.** It returns a `Result` and
  never panics. It always checks that there are fewer givens than cells; that the
  solution is full, every digit in range, and free of conflict; and that every given is
  on the grid and matches the solution at its position. Uniqueness is the one thing it
  takes on its caller's word.
- **"Only the solver calls the constructor" is held by review.** Rust cannot restrict
  visibility to one sibling module. Outside the crate the rule is mechanical: nothing
  there can make a proof.
- **`Puzzle` keeps the solution privately.** It is set from the proof and cannot fail.
  It answers one question, whether one digit is the solution's at one position, yes or
  no, and that answer is `pub(crate)`, for the board's check. The digits are never
  exposed through `Puzzle`. "Solved" is the specification's definition, full and
  consistent, never a comparison with the stored solution.
- **The proof exposes what the solver found.** `WellPosed` has public read access to its
  givens and its solution, because `solver.allium`'s `SearchResult` exposes the solution
  to whoever asked for the search. Its `Debug` may show both. What is hidden is the
  solution once it is inside a `Puzzle`.

Facts that shape the work:

- **Nothing outside the crate can make a `Puzzle` until T26.** Doctests and the files
  under `crates/pawdoku/tests/` compile as outside crates. So in this ticket the
  behaviour of `WellPosed` and `Puzzle` is proved by unit tests inside the module, which
  are what the coverage floor counts, and their doc examples compile without running
  (step 5). T26 rewrites those examples to run through the solver.
- **This module's tests cannot call the solver.** They build a proof through the
  `pub(crate)` constructor from a puzzle and solution checked by hand. The fixture is
  below.
- **Two items are dead until later tickets use them.** The proof's constructor and the
  yes-or-no answer have no caller outside tests here, so `just clippy` will report each
  as dead code. Each carries
  `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` naming the ticket that
  lifts it (T26 for the constructor, T27 for the answer). The `not(test)` matters:
  `just clippy` checks the library twice, once as it ships and once with its tests, and
  in the second the tests call both items, so a bare `#[expect]` would be unfulfilled
  there and fail the gate. An expectation that stops being needed fails `just clippy`
  by itself, which is how the later ticket is told. Add one only where clippy reports
  the item.
- **The value types are unconstrained, as in the specification.** `Position` is a row
  and a column and `Given` a position and a digit, each a plain integer. A given off the
  grid or out of range is representable, because refusing it is a rule of the solver
  (`RefuseMalformedGivens`), not a property of the type. Their constructors do not fail.
- **The metrics gate was set before this code existed** (`rustqual.toml`, `clippy.toml`,
  decision 0013): no recursion; a function at most 60 lines, cognitive complexity 15,
  cyclomatic 10, nesting 4, five parameters; a struct at most 12 fields and 20 methods,
  LCOM4 at most 2; a file at most 500 code lines before its first `#[cfg(test)]`, and
  1000 for test code. A threshold that fires is met by restructuring: `src/sudoku.rs`
  plus `src/sudoku/<part>.rs`, which the boundary rule already covers (`mod.rs` is
  banned). Never by a suppression and never by moving a number. If restructuring cannot
  meet one, stop and report.
- **The boundary rule** `sudoku_imports_nothing` forbids every `crate::` path to another
  engine module in `src/sudoku`. `random` is allowed and not needed.

The fixture, checked on 2026-09-30: 30 givens, the solution is a valid grid, every given
matches it, the givens have exactly one solution, and naked and hidden singles alone
solve it. It is the example puzzle of the article `sudoku.allium` cites as its source.
Rows top to bottom, a dot for an empty cell:

```text
puzzle       solution
53..7....    534678912
6..195...    672195348
.98....6.    198342567
8...6...3    859761423
4..8.3..1    426853791
7...2...6    713924856
.6....28.    961537284
...419..5    287419635
....8..79    345286179
```

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/sudoku.allium` in
full; the decision T23 wrote; `docs/explanation/layering.md` and
`docs/explanation/architecture.md`; `docs/reference/testing.md`;
`.agents/skills/rust-change/SKILL.md` and `.agents/skills/propagate/SKILL.md`;
`crates/pawdoku/src/random.rs` as the model for a public type, an error enum and its
tests; `rustqual.toml` and `clippy.toml`. Background only, never modified: the Python
prototype at `/Users/scutting/projects/pawdoku/prototypes/board/` (`grid.py`,
`sudoku.py`). It models the rules without the proof value, so it is a guide to behaviour
and not to this design.

## Goal

`pawdoku::sudoku` exists and does what `sudoku.allium` says: the value types, the proof,
a `Puzzle` with its eighty-one cells, placing and erasing, conflicts, and solved as a
final state. Every rule, invariant, guarantee and surface clause of the module has a
named test, listed in the hand-back notes. `just check` is green with the coverage floor
held by this module's own tests and no suppression beyond the two stated expectations.

Names later tickets are written against, fixed here: `sudoku::Position`, `sudoku::Given`,
`sudoku::WellPosed`, `sudoku::Puzzle`. Every other name is this ticket's to choose and
to report (step 7).

## Non-goals

- No solver, no board, no technique. Nothing here decides whether givens are well-posed.
- No `Deserialize` on `WellPosed` and no serialisation of `Puzzle` at all: deserialising
  a proof would forge one, and a puzzle holds the solution.
- No public way to ask a `Puzzle` about its solution.
- No new dependency and no new feature flag. `thiserror`, optional `serde` and
  `proptest` for tests are the whole list (decision 0007).
- No change to `sudoku.allium` or any other module. If no clause says what the code
  needs, stop: that is a `spec-change` first.
- No threshold moved and no lint suppressed, other than the two expectations above.
- No variants: other sizes, irregular boxes, diagonals. The side is nine.

## Files touched

T25 to T28 run in sequence, so the files they share are edited by one ticket at a time.

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/sudoku.rs` | New: the module |
| `crates/pawdoku/src/sudoku/` | New only if a limit asks for a split; one file per part |
| `crates/pawdoku/src/lib.rs` | `pub mod sudoku;`, the crate overview, and where `SIDE` lives (first open point) |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type meets invariant 3; the value types serialise under `serde` |
| `crates/pawdoku/tests/sudoku.rs` | New: what an outside crate can reach here, which is the value types |
| `docs/explanation/architecture.md` | A section on how well-posedness reaches the rules: the proof, its constructor, the rule review holds |
| `docs/explanation/specifications.md` | The sentence that says `pawdoku::SIDE` mirrors `config.side`, if the constant moves |
| `docs/reference/testing.md` | The suite table and the integration-test paragraph |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T25's row set `done` |
| `tickets/T25-sudoku-rules.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t25-sudoku-rules` from `main` after T23 and T24 have
   merged. Run `just initialize` if `.pixi/` is absent; it uses the network and this
   ticket authorises it. Run `just check` and confirm it is green before any edit.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves (`docs/reference/testing.md`, Conventions) and what it
   expects. Seed it from "What the tests must cover" below. Then run
   `just plan-spec sudoku` and add any obligation the list lacks. An obligation that no
   behaviour of this module can show is struck with a one-line reason. Put the list in
   the hand-back notes under "The test list" before the first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects. A test that is green before the code exists proves nothing.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. Run `just clippy` and `just metrics` every few loops,
   so that a limit is met while the code is still small.

4. **What to build**, in the order the list will most likely take it:

   - The config: `box_side` is 3 and `side` is its square, named so that no rule carries
     a bare number.
   - `Position` and `Given`: plain values with structural equality, `Clone`, `Debug`,
     and `Serialize` and `Deserialize` under the `serde` feature.
   - A set of givens. The representation is yours (`alloc` only: no `HashMap`); two
     givens that agree are one given.
   - `WellPosed`: the givens and the solution, the `pub(crate)` constructor with one
     error for each thing it checks, and public read access to both parts.
   - `Puzzle`: set from a proof; eighty-one cells, each with its row, column, band,
     stack, digit, whether it is given and whether it conflicts; the status; `is_full`,
     `is_consistent`; placing and erasing with one error for each `requires` the rule
     states, and one for a position that names no cell; solved the moment the grid is
     complete, and final; the `pub(crate)` answer; a `Debug` that leaves the solution
     out.
   - Errors are `thiserror` enums, `#[non_exhaustive]`, with stable `Display` text in
     the style of `RandomError`: one lowercase sentence that carries the offending
     values. A unit test pins each variant's text.

5. **Doc examples.** Every public item carries one. Those on `Position`, `Given` and the
   constants run as written. Those on `WellPosed` and `Puzzle` cannot make a value from
   outside the crate yet, so each wraps its body in a hidden function that takes the
   proof or the puzzle as an argument and is never called: the example compiles, shows
   the call, and asserts nothing at run time. Say so in one line of the hand-back notes
   and name the items, so that T26 can rewrite them to run.

6. **The pages.** `docs/explanation/architecture.md` gains a section on how
   well-posedness reaches the rules, citing the decision and saying plainly that "only
   the solver calls the constructor" is a rule review holds. `docs/reference/testing.md`
   gains the suite rows, the fixture and why this module's tests build the proof by
   hand. `docs/project/repository-map.md` and `CHANGELOG.md` follow. Each page stays
   within what `docs/manifest.yml` says it owns.

7. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`, `just wasm-check`,
   `just features`, `just coverage`, `just doc` and `just check-docs`, then `just check`.
   Quote the closing lines of each and the coverage line for the module. Fill in the
   hand-back notes: the test list as it ended, as a table from clause to test name; and
   "Names later tickets need", which lists every public item and the two crate-only ones
   with their signatures. Set `status: done` here and in `tickets/README.md`, commit on
   the ticket branch, and stop before pushing.

### What the tests must cover

From `docs/specs/sudoku.allium`, by name. Each is owed at least one test.

- **Value types and config.** `Position` and `Given` are equal when their fields are.
  `box_side` is 3 and `side` is 9.
- **`SetPuzzle`.** A proof is refused for eighty-one givens; for a solution that is not
  full, has a digit out of range, or has two peers holding one digit; for a given off
  the grid; and for a given whose digit is not the solution's at its position, which is
  also how two givens that disagree about one position are refused. A proof that is
  accepted sets a puzzle that is unsolved and holds exactly those givens.
- **`LayOutGrid`.** Eighty-one cells, one to a position, each in the band and stack
  `CellsSitInTheirBox` gives it; a given's cell holds its digit and every other cell is
  empty.
- **`PlaceDigit`.** Refused on a solved puzzle, on a given cell, and for a digit outside
  1 to 9. Allowed when the digit conflicts, when the cell already holds another digit of
  the player's, and when it holds the same one.
- **`EraseDigit`.** Refused on a solved puzzle, on a given cell and on an empty cell.
- **`PuzzleSolved` and the transition.** A complete grid is solved at once. A full grid
  with a conflict is not. Solved is final: both moves are then refused.
- **Solved is not a comparison with the stored solution.** Build a proof with no givens
  and one solution grid; fill the puzzle with a different valid grid, such as the same
  grid with two digits exchanged throughout; it is solved.
- **Derived values.** `is_full`, `is_consistent` and complete; a cell's twenty peers,
  itself not among them; a conflict marks both cells; an empty cell conflicts with
  nothing.
- **The eight invariants**, by property over arbitrary sequences of placements and
  erasures, refused ones included: `TheGridIsWhole`, `OneCellToAPosition`,
  `CellsSitOnTheGrid`, `CellsSitInTheirBox`, `DigitsAreInRange`,
  `GivenCellsHoldTheirGiven`, `GivensNeverConflict`, `SolvedMeansComplete`. And the one
  stated in prose: a puzzle's givens never change.
- **`PuzzleSetting`, `OnlyWellPosedPuzzlesArePosed`.** Held by the type: no public
  constructor makes a proof. A `compile_fail` doc example shows that outside code cannot
  call the constructor. Such an example passes for any compile error, so keep it to the
  one line that names the constructor.
- **`PuzzleSolving`.** Everything the surface exposes can be read. What it provides
  agrees with the rules, and the oracle for a refusal is the rule and not the surface: a
  move is accepted exactly when every `requires` clause of its rule holds and its
  position names a cell. A `when` clause says where a move is offered and speaks of the
  cell alone. Wherever it is false the move is refused; where it is true `PlaceDigit` is
  still refused for a digit outside 1 to 9. For `EraseDigit` the two agree.
- **The crate-only answer.** Yes for the solution's digit and no for every other digit,
  at given cells too; a total function that never panics, whatever position it is asked.
- **`Debug`.** Two puzzles alike in everything but the stored solution print alike.

## Acceptance criteria

- `pawdoku::sudoku` exports `Position`, `Given`, `WellPosed` and `Puzzle`. No path in
  `src/sudoku` names another engine module.
- Nothing public makes a `WellPosed`, and the only way to make a `Puzzle` takes one. The
  yes-or-no answer is `pub(crate)`.
- `WellPosed` does not implement `Deserialize`; `Puzzle` implements neither `Serialize`
  nor `Deserialize`.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec sudoku` obligation is in the table or struck with a
  reason.
- No `unwrap`, `expect`, `panic!` or indexing that can fail outside test code; no
  `#[allow]`; the only expectations added are the two `dead_code` ones, each under
  `cfg_attr(not(test), ...)` with a reason naming T26 or T27. No `qual:allow` line.
- Line coverage of `src/sudoku` alone is at or above 90 per cent.
- `tests/api_bounds.rs` names every new public type.
- `just check` is green.

## Verification

```sh
just plan-spec sudoku
just fmt-check
just clippy
just metrics
just test
just wasm-check
just features
just coverage
just doc
just check-docs
just check
```

Expected: the obligation list the table accounts for; each recipe green with no warning;
`metrics` reporting no finding and the probe satisfied; both wasm targets under both
feature sets; the coverage total at or above the floor; `All checks passed and the
worktree is unchanged.`

## Hand-back notes

### The test list

### Names later tickets need

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **Where `SIDE` lives.** `pawdoku::SIDE` sits at the crate root today, and
  `docs/explanation/specifications.md` says it mirrors `config.side`. The specification
  states two figures, `box_side` and `side`, in `sudoku.allium`. The ticket's default:
  both become constants of `sudoku`, the root constant goes with its doctest and unit
  test moved, and the page's sentence follows. The crate is unpublished and its root
  says the items may change. The maintainer may instead keep a re-export at the root;
  a re-export in `lib.rs` is the third blind spot `docs/explanation/layering.md` names.
- **The name `WellPosed`.** The type holds both of `SetPuzzle`'s guards: one solution,
  and something left to play. The name says the first. It is fixed here because T26 to
  T28 are written against it; the maintainer may rename it at review, and a rename is
  then a change to those tickets too.
- **The `compile_fail` example** is the only mechanical proof that outside code cannot
  make a proof. If the maintainer wants a stronger one, it is a UI test and a new
  dependency, which is a T02 hand-back under decision 0007.
- **A second fixture.** One puzzle is enough for the rules. T26 needs more (a puzzle
  that takes a guess, givens with several solutions, givens with none) and finds them
  itself.
