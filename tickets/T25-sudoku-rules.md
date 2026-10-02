---
id: T25
title: "The rules in Rust: the sudoku module, the proof value and Puzzle"
status: done
depends_on: [T23, T24, T29]
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
- **The proof exposes what the solver found.** `WellPosed` gives read access to its
  givens and its solution, because `solver.allium`'s `SearchResult` exposes the solution
  to whoever asked for the search. Its `Debug` may show both. What is hidden is the
  solution once it is inside a `Puzzle`.

Facts that shape the work:

- **`WellPosed` and `Puzzle` stay crate-only until T26.** Doctests and the files under
  `crates/pawdoku/tests/` compile as outside crates, and nothing outside the crate can
  make a proof until T26's `solve` exists. A public item needs a doc example that runs
  and asserts (`AGENTS.md`, `docs/reference/testing.md`), and neither type could have
  one here. So this ticket declares both `pub(crate)`, with the errors only they return.
  T26 makes them public, and `solve` lets each have a doc example that runs. Their names
  are fixed now all the same. Their behaviour is proved by unit tests inside the module,
  which are what the coverage floor counts.
- **This module's tests cannot call the solver.** They build a proof through the
  `pub(crate)` constructor from a puzzle and solution checked by hand. The fixture is
  below.
- **The crate-only items are dead until later tickets use them.** Outside tests nothing
  here calls the proof's constructor, `Puzzle` and its methods, or the yes-or-no answer,
  so `just clippy` will report them as dead code. Each item clippy reports carries
  `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` naming the ticket that
  lifts it: T26 for everything `solve` and the public API will reach, T27 for the
  answer. Put the answer's expectation on the answer itself, so that it stays when T26
  removes the others. The `not(test)` matters: `just clippy` checks the library twice,
  once as it ships and once with its tests, and in the second the tests call these
  items, so a bare `#[expect]` would be unfulfilled there and fail the gate. An
  expectation that stops being needed fails `just clippy` by itself, which is how the
  later ticket is told. Add one only where clippy reports the item.
- **Snapshots show and detect change; they prove nothing.** T29 added insta and the rule,
  which `docs/reference/testing.md` states: a snapshot shows a reviewer what a value
  looks like and makes a change to it visible in a diff. It is never the test of a
  clause, so no line of "What the tests must cover" and no row of the hand-back table
  names one. Snapshots are `.snap` files, taken from fixed inputs after the module's
  own tests are green, by tests named `snapshot_...`, from a text picture rendered in
  test code. No `Display`, method or field is added to the library to feed one.
  Here the snapshots sit inside the module, because `Puzzle` is crate-only until T26.
  insta writes each file to a `snapshots/` directory beside the test's own file.
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
`tickets/T29-snapshot-harness.md`, the rule and its hand-back notes;
`crates/pawdoku/src/random.rs` as the model for a public type, an error enum and its
tests; `rustqual.toml` and `clippy.toml`. Background only, never modified: the Python
prototype at `/Users/scutting/projects/pawdoku/prototypes/board/` (`grid.py`,
`sudoku.py`). It models the rules without the proof value, so it is a guide to behaviour
and not to this design.

## Goal

`pawdoku::sudoku` exists and does what `sudoku.allium` says: the value types, the proof,
a `Puzzle` with its eighty-one cells, placing and erasing, conflicts, and solved as a
final state. Every rule, invariant, guarantee and surface clause of the module has a
named test, listed in the hand-back notes. The puzzle as its surface shows it is
snapshotted for review. `just check` is green with the coverage floor
held by this module's own tests and no suppression beyond the stated `dead_code`
expectations.

Names later tickets are written against, fixed here: `sudoku::Position`, `sudoku::Given`,
`sudoku::WellPosed`, `sudoku::Puzzle`. The last two are crate-only here and public from
T26. Every other name is this ticket's to choose and to report (step 8).

## Non-goals

- No solver, no board, no technique. Nothing here decides whether givens are well-posed.
- No `Deserialize` on `WellPosed` and no serialisation of `Puzzle` at all: deserialising
  a proof would forge one, and a puzzle holds the solution.
- No public way to ask a `Puzzle` about its solution.
- No public `WellPosed` or `Puzzle`, and no public error that only they return. T26
  makes them public.
- No new dependency and no new feature flag. `thiserror`, optional `serde`, and
  `proptest` and `insta` for tests are the whole list (decision 0007, T29).
- No snapshot standing in for a test that asserts, and nothing added to the library so
  that a snapshot can read it.
- No change to `sudoku.allium` or any other module. If no clause says what the code
  needs, stop: that is a `spec-change` first.
- No threshold moved and no lint suppressed, other than the `dead_code` expectations
  above.
- No variants: other sizes, irregular boxes, diagonals. The side is nine.

## Files touched

T25 to T28 run in sequence, so the files they share are edited by one ticket at a time.

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/sudoku.rs` | New: the module |
| `crates/pawdoku/src/sudoku/` | New only if a limit asks for a split; one file per part |
| `crates/pawdoku/src/snapshots/` | New: the `.snap` files of step 5; under `src/sudoku/` instead if the module is split and the tests sit there |
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
   - `WellPosed`, crate-only: the givens and the solution, the `pub(crate)` constructor
     with one error for each thing it checks, and read access to both parts.
   - `Puzzle`, crate-only: set from a proof; eighty-one cells, each with its row, column, band,
     stack, digit, whether it is given and whether it conflicts; the status; `is_full`,
     `is_consistent`; placing and erasing with one error for each `requires` the rule
     states, and one for a position that names no cell; solved the moment the grid is
     complete, and final; the `pub(crate)` answer; a `Debug` that leaves the solution
     out.
   - Errors are `thiserror` enums, `#[non_exhaustive]`, with stable `Display` text in
     the style of `RandomError`: one lowercase sentence that carries the offending
     values. A unit test pins each variant's text.

5. **Snapshots**, once the list is empty. Write one render helper in the module's test
   code that draws a puzzle as text from what `PuzzleSolving` exposes: nine rows of
   digits with a dot for an empty cell, in the fixture's own layout; which cells are
   given; which conflict; and the status with `is_full` and `is_consistent`. No line
   ends in a space. Then take these, each by a test named `snapshot_...`:

   - the fixture as set;
   - the fixture after a short fixed script: a placement, a placement that conflicts
     with a given, a placement over the player's own digit, and an erasure;
   - `Puzzle`'s `Debug` for the fixture, which a reviewer can see holds no solution;
   - the `Display` text of every error the module defines, one line each, in one
     snapshot.

   Accept them with `just snapshots-accept` and read each file before committing it:
   a picture that looks wrong is a bug to fix with a test that asserts, not a snapshot
   to accept. List them in the hand-back notes under "Snapshots taken".

6. **Doc examples.** Every public item carries one that runs and asserts: `Position`,
   `Given`, the constants and any public error. `WellPosed`, `Puzzle` and their errors
   are crate-only here. They carry doc comments and no examples, because a doc example
   cannot reach a crate-only item. List in the hand-back notes every item T26 is to make
   public, so that it can write their examples.

7. **The pages.** `docs/explanation/architecture.md` gains a section on how
   well-posedness reaches the rules, citing the decision and saying plainly that "only
   the solver calls the constructor" is a rule review holds. `docs/reference/testing.md`
   gains the suite rows, the fixture, why this module's tests build the proof by
   hand, and the snapshots taken. `docs/project/repository-map.md` and
   `CHANGELOG.md` follow. Each page stays
   within what `docs/manifest.yml` says it owns.

8. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`.
   Quote the closing lines of each and the coverage line for the module. Fill in the
   hand-back notes: the test list as it ended, as a table from clause to test name; and
   "Names later tickets need", which lists every public item and every crate-only one
   with their signatures, each crate-only item marked with the ticket that makes it
   public (T26) or calls it (T27). Set `status: done` here and in `tickets/README.md`, commit on
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
- **`PuzzleSetting`, `OnlyWellPosedPuzzlesArePosed`.** Held by the type: a `Puzzle` is
  made only from a proof, and the proof only through its constructor, which refuses
  what `SetPuzzle` refuses. Outside the crate neither type can be named yet. T26 adds
  the `compile_fail` example that shows outside code cannot call the constructor, once
  the type is public.
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

- `pawdoku::sudoku` exports `Position`, `Given` and the constants. `WellPosed` and
  `Puzzle` exist, `pub(crate)`. No path in `src/sudoku` names another engine module.
- Nothing makes a `WellPosed` but its `pub(crate)` constructor, and the only way to make
  a `Puzzle` takes one. The yes-or-no answer is `pub(crate)`.
- Every public item has a doc example that runs and asserts.
- `WellPosed` does not implement `Deserialize`; `Puzzle` implements neither `Serialize`
  nor `Deserialize`.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec sudoku` obligation is in the table or struck with a
  reason.
- No `unwrap`, `expect`, `panic!` or indexing that can fail outside test code; no
  `#[allow]`; the only expectations added are `dead_code` ones on items clippy reports,
  each under `cfg_attr(not(test), ...)` with a reason naming T26 or T27, and the
  answer's on the answer alone. No `qual:allow` line.
- Line coverage of `src/sudoku` alone is at or above 90 per cent.
- `tests/api_bounds.rs` names every new public type.
- The four snapshots of step 5 exist as `.snap` files, each taken by a test named
  `snapshot_...`. None appears in the hand-back table as the test of a clause, and no
  public or crate-only item exists only to feed one. `just snapshots-check` is green.
- `just check` is green.

## Verification

```sh
just plan-spec sudoku
just fmt-check
just clippy
just metrics
just test
just snapshots-check
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

The list as first written on 2026-10-01, before any test or code, seeded from "What the
tests must cover" and then checked against the 48 obligations of `just plan-spec sudoku`.
One line is one expected test: its name, then what it expects.

Value types and config:

1. `box_side_is_three_and_side_is_its_square`: `BOX_SIDE` is 3, `SIDE` is 9 and is
   `BOX_SIDE * BOX_SIDE`.
2. `positions_are_equal_when_their_fields_are`: equal for the same row and column,
   unequal when either differs.
3. `givens_are_equal_when_their_fields_are`: equal for the same position and digit,
   unequal when either differs.
4. `two_givens_that_agree_are_one_given`: a set made from the same given twice holds one.

`SetPuzzle`, through the proof:

1. `a_proof_is_refused_for_eighty_one_givens`: every cell of the solution as a given is
   refused, as leaving nothing to play.
2. `a_proof_is_refused_for_a_solution_that_is_not_full`: one empty cell in the solution
   is refused, naming the cell.
3. `a_proof_is_refused_for_a_solution_digit_out_of_range`: a 0 or a 10 in the solution
   is refused.
4. `a_proof_is_refused_for_two_peers_holding_one_digit`: a full, in-range solution with
   a repeat in a row, in a column or in a box is refused.
5. `a_proof_is_refused_for_a_given_off_the_grid`: a given at row 0 or column 10 is
   refused.
6. `a_proof_is_refused_for_a_given_that_is_not_the_solutions`: a given whose digit
   differs from the solution's at its position is refused.
7. `two_givens_that_disagree_about_one_position_are_refused`: by the same check, since
   at most one of them can match.
8. `a_proof_gives_back_its_givens_and_its_solution`: both parts are read back as given.
9. `an_accepted_proof_sets_an_unsolved_puzzle_with_exactly_those_givens`.
10. `proof_error_texts_are_stable`: every variant's `Display` text, pinned.

`LayOutGrid`:

1. `the_grid_is_eighty_one_cells_one_to_a_position`: every position from row 1, column
   1 to row 9, column 9, each once, in grid order.
2. `cells_sit_in_the_band_and_stack_of_their_box`: rows 1 to 3 are band 1, 4 to 6 band
   2, 7 to 9 band 3, and columns likewise.
3. `a_given_cell_holds_its_digit_and_every_other_cell_is_empty`.

`PlaceDigit`:

1. `placing_puts_the_digit_in_the_cell`.
2. `placing_is_refused_on_a_solved_puzzle`.
3. `placing_is_refused_on_a_given_cell`.
4. `placing_is_refused_for_a_digit_outside_one_to_nine`: 0 and 10.
5. `placing_is_refused_where_no_cell_is`: a position off the grid.
6. `a_conflicting_digit_may_be_placed`.
7. `a_cell_holding_the_players_digit_takes_a_new_one`.
8. `the_same_digit_may_be_placed_again`.

`EraseDigit`:

1. `erasing_empties_the_cell`.
2. `erasing_is_refused_on_a_solved_puzzle`.
3. `erasing_is_refused_on_a_given_cell`.
4. `erasing_is_refused_on_an_empty_cell`.
5. `erasing_is_refused_where_no_cell_is`.
6. `move_error_texts_are_stable`: every variant's `Display` text, pinned.

`PuzzleSolved` and the transition:

1. `a_complete_grid_is_solved_at_once`: the placement that completes the grid solves it,
   with no further call.
2. `a_full_grid_with_a_conflict_is_not_solved`.
3. `solved_is_final`: after refused moves of both kinds the status is still solved and
   no cell has changed.
4. `solved_is_not_a_comparison_with_the_stored_solution`: a proof with no givens, the
   puzzle filled with the same grid with 1 and 2 exchanged throughout, is solved.

Derived values:

1. `full_counts_the_filled_cells`: false with one cell empty, true with none.
2. `consistent_means_no_cell_conflicts`.
3. `a_cell_has_twenty_peers_and_is_not_among_them`: for every cell of the grid.
4. `a_conflict_marks_both_cells`: and no third cell.
5. `an_empty_cell_conflicts_with_nothing`.
6. `a_refused_move_leaves_the_puzzle_as_it_was`.

Invariants, by property over arbitrary sequences of placements and erasures, refused
ones included, some of which run on to a solved puzzle:

1. `the_invariants_hold_after_every_move`: `TheGridIsWhole`, `OneCellToAPosition`,
   `CellsSitOnTheGrid`, `CellsSitInTheirBox`, `DigitsAreInRange`,
   `GivenCellsHoldTheirGiven`, `GivensNeverConflict` and `SolvedMeansComplete`, each
   asserted under its own name after every step.
2. `a_puzzles_givens_never_change`: the givens after every step are the givens it was
   set with.

`PuzzleSolving`:

1. `the_surface_exposes_the_status_and_every_cell`: status, `is_full`, `is_consistent`
   and all seven things of each cell can be read.
2. `a_move_is_accepted_exactly_when_its_rule_allows_it`: by property, against a model
   written from the rules' `requires` clauses and nothing of the implementation.
3. `a_move_is_refused_wherever_it_is_not_offered`: by property, the surface's `when`
   clause false means refused; true means an erasure is accepted and a placement is
   accepted exactly when the digit is in range.

The crate-only answer, and `Debug`:

1. `the_answer_is_yes_for_the_solutions_digit_and_no_for_every_other`: at every cell,
   given cells included.
2. `the_answer_is_no_wherever_no_cell_is`: by property over any position and digit; it
   never panics.
3. `two_puzzles_alike_but_for_the_solution_print_alike`.

From outside the crate (`tests/sudoku.rs`, `tests/api_bounds.rs`):

1. `the_side_is_the_square_of_the_box_side`.
2. `a_position_off_the_grid_and_a_given_out_of_range_are_representable`.
3. `value_types_compare_by_their_fields`.
4. The value types added to the two bounds tests and the serde test.

Obligations of `just plan-spec sudoku` struck, with the reason:

- `surface-actor.PuzzleSetting`, `surface-actor.PuzzleSolving`: neither surface declares
  an actor, so there is no restriction to show.
- `rule-failure.SetPuzzle.2` (`solution_count = 1`): this module cannot decide it; the
  proof takes uniqueness on its caller's word, and T26 owns the refusal.
- `rule-failure.PuzzleSolved.1` (`status = unsolved`): no move reaches a solved puzzle,
  so the rule cannot be offered a solved one; `solved_is_final` shows what can be seen.

Every other obligation maps to a line above; the closing table says which.

Changes to the list, one line each, as the loops ran:

- Value types 2 to 4 were taken in one loop after line 1: their tests were written
  together, failed to compile for want of `Position` and `Given`, and went green with
  the two types.
- `two_givens_that_agree_are_one_given` needed no code of its own: a `BTreeSet` and the
  derived ordering are the set. Kept, because the clause is the ticket's.
- Proof lines 1 to 7 were run first against a constructor that accepted everything, so
  each failed on its assertion; the checks then turned them green. Lines 8 and 10 were
  red by compilation only.
- The move and solved lines were run against moves that wrote the digit and checked
  nothing, and the guards were added one at a time (the solved transition, no such
  cell, solved, given, digit range, empty cell), each run showing the next failures.
- nextest's default profile stops at the first failure, so a red run names the first
  failing tests and not every one; the guard-by-guard runs are how each was seen.
- Five lines were green as soon as the unguarded moves existed (`placing_puts_...`,
  `a_conflicting_digit_...`, `a_cell_holding_the_players_digit_...`,
  `the_same_digit_...`, `erasing_empties_...`), and the property tests, the answer and
  the `Debug` test went green with the code they were written beside. Each was then
  shown to bite by breaking the code on purpose; "What was verified" lists the breaks.
- Added: `a_refused_move_leaves_the_puzzle_as_it_was` gained erasures and an off-grid
  position, because the refusal order put the position first.
- Moved: `a_cell_has_twenty_peers_and_is_not_among_them` sits in `src/sudoku.rs`, with
  the peer relation it tests, after the module was split.
- Added outside the crate: `two_givens_that_agree_are_one_given` in `tests/sudoku.rs`,
  with two disagreeing givens staying two, since that is what the proof then refuses.
- No line was struck.

The list as it ended, from clause to test. Tests are in `crates/pawdoku/src/sudoku.rs`
(S), `src/sudoku/proof.rs` (P), `src/sudoku/puzzle.rs` (Z) and `tests/sudoku.rs` (O).
No row names a snapshot test.

| Clause | Test | `plan-spec` obligations |
| --- | --- | --- |
| `config`: `box_side` is 3, `side` its square | S `box_side_is_three_and_side_is_its_square`; O `the_side_is_the_square_of_the_box_side` | `config-default.box_side`, `config-default.side` |
| `Position` equal when its fields are | S `positions_are_equal_when_their_fields_are`; O `value_types_compare_by_their_fields` | `value-equality.Position`, `entity-fields.Position` |
| `Given` equal when its fields are | S `givens_are_equal_when_their_fields_are`; O `value_types_compare_by_their_fields` | `value-equality.Given`, `entity-fields.Given` |
| Two givens that agree are one given | S and O `two_givens_that_agree_are_one_given` | none |
| The value types are unconstrained | O `a_position_off_the_grid_and_a_given_out_of_range_are_representable` | none |
| `SetPuzzle`: fewer givens than cells | P `a_proof_is_refused_for_eighty_one_givens` | `rule-failure.SetPuzzle.1` |
| `SetPuzzle`: the solution is full | P `a_proof_is_refused_for_a_solution_that_is_not_full` | none (the proof's own check) |
| `SetPuzzle`: every solution digit in range | P `a_proof_is_refused_for_a_solution_digit_out_of_range` | none |
| `SetPuzzle`: the solution free of conflict | P `a_proof_is_refused_for_two_peers_holding_one_digit` | none |
| `SetPuzzle`: every given on the grid | P `a_proof_is_refused_for_a_given_off_the_grid` | none |
| `SetPuzzle`: every given matches the solution | P `a_proof_is_refused_for_a_given_that_is_not_the_solutions`, `two_givens_that_disagree_about_one_position_are_refused` | none |
| The proof exposes what the solver found | P `a_proof_gives_back_its_givens_and_its_solution` | none |
| `SetPuzzle` succeeds: unsolved, exactly those givens | Z `an_accepted_proof_sets_an_unsolved_puzzle_with_exactly_those_givens` | `rule-success.SetPuzzle`, `rule-entity-creation.SetPuzzle.1`, `entity-fields.Puzzle`, `surface-provides.PuzzleSetting` |
| `SetPuzzle`: exactly one solution | struck: taken on the caller's word; T26 | `rule-failure.SetPuzzle.2` |
| `LayOutGrid`: eighty-one cells, one to a position | Z `the_grid_is_eighty_one_cells_one_to_a_position` | `rule-success.LayOutGrid`, `entity-relationship.Puzzle.cells` |
| `LayOutGrid`: band and stack | Z `cells_sit_in_the_band_and_stack_of_their_box` | `entity-fields.Cell` |
| `LayOutGrid`: given cells hold their digit, others empty | Z `a_given_cell_holds_its_digit_and_every_other_cell_is_empty` | `entity-optional.Cell.digit` |
| `PlaceDigit` succeeds | Z `placing_puts_the_digit_in_the_cell` | `rule-success.PlaceDigit` |
| `PlaceDigit` requires an unsolved puzzle | Z `placing_is_refused_on_a_solved_puzzle` | `rule-failure.PlaceDigit.1` |
| `PlaceDigit` requires a cell that is not given | Z `placing_is_refused_on_a_given_cell` | `rule-failure.PlaceDigit.2` |
| `PlaceDigit` requires a digit from 1 to 9 | Z `placing_is_refused_for_a_digit_outside_one_to_nine` | `rule-failure.PlaceDigit.3` |
| `PlaceDigit`: a position that names no cell | Z `placing_is_refused_where_no_cell_is` | none |
| `PlaceDigit` allows a conflict | Z `a_conflicting_digit_may_be_placed` | none |
| `PlaceDigit` over the player's own digit | Z `a_cell_holding_the_players_digit_takes_a_new_one` | none |
| `PlaceDigit` of the same digit | Z `the_same_digit_may_be_placed_again` | none |
| `EraseDigit` succeeds | Z `erasing_empties_the_cell` | `rule-success.EraseDigit` |
| `EraseDigit` requires an unsolved puzzle | Z `erasing_is_refused_on_a_solved_puzzle` | `rule-failure.EraseDigit.1` |
| `EraseDigit` requires a cell that is not given | Z `erasing_is_refused_on_a_given_cell` | `rule-failure.EraseDigit.2` |
| `EraseDigit` requires a digit to erase | Z `erasing_is_refused_on_an_empty_cell` | `rule-failure.EraseDigit.3` |
| `EraseDigit`: a position that names no cell | Z `erasing_is_refused_where_no_cell_is` | none |
| A refused move changes nothing | Z `a_refused_move_leaves_the_puzzle_as_it_was` | none |
| `PuzzleSolved`: a complete grid is solved at once | Z `a_complete_grid_is_solved_at_once` | `rule-success.PuzzleSolved`, `transition-edge.Puzzle.unsolved.solved`, `derived.Puzzle.is_complete` |
| A full grid with a conflict is not solved | Z `a_full_grid_with_a_conflict_is_not_solved` | `derived.Puzzle.is_complete` |
| Solved is final | Z `solved_is_final`, with the two `..._on_a_solved_puzzle` tests | `transition-terminal.Puzzle.status`, `transition-rejected.Puzzle.status` |
| `PuzzleSolved` requires an unsolved puzzle | struck: no move reaches a solved puzzle | `rule-failure.PuzzleSolved.1` |
| Solved is not a comparison with the stored solution | Z `solved_is_not_a_comparison_with_the_stored_solution` | none |
| `is_full` | Z `full_counts_the_filled_cells` | `derived.Puzzle.is_full`, `projection.Puzzle.filled_cells` |
| `is_consistent` | Z `consistent_means_no_cell_conflicts` | `derived.Puzzle.is_consistent`, `projection.Puzzle.conflicting_cells` |
| A cell's twenty peers, itself not among them | S `a_cell_has_twenty_peers_and_is_not_among_them` | `entity-relationship.Cell.peers` |
| A conflict marks both cells | Z `a_conflict_marks_both_cells` | `derived.Cell.is_conflicting` |
| An empty cell conflicts with nothing | Z `an_empty_cell_conflicts_with_nothing` | `derived.Cell.is_conflicting` |
| `TheGridIsWhole`, `OneCellToAPosition`, `CellsSitOnTheGrid`, `CellsSitInTheirBox`, `DigitsAreInRange`, `GivenCellsHoldTheirGiven`, `GivensNeverConflict`, `SolvedMeansComplete` | Z `the_invariants_hold_after_every_move`, each asserted under its own name | the eight `invariant.*` |
| A puzzle's givens never change | Z `a_puzzles_givens_never_change` | none (prose) |
| `PuzzleSetting`, `OnlyWellPosedPuzzlesArePosed` | held by the type: `Puzzle::set` takes a `WellPosed` and nothing else makes a puzzle, and `WellPosed::vouch` is the one place a `WellPosed` is made; the P rows above are what it refuses. T26 adds the `compile_fail` example | `surface-provides.PuzzleSetting` |
| `PuzzleSolving` exposes | Z `the_surface_exposes_the_status_and_every_cell` | `surface-exposure.PuzzleSolving` |
| `PuzzleSolving` provides: accepted exactly when the rule allows | Z `a_move_is_accepted_exactly_when_its_rule_allows_it` | `surface-provides.PuzzleSolving` |
| `PuzzleSolving` provides: refused wherever `when` is false | Z `a_move_is_refused_wherever_it_is_not_offered` | `surface-provides.PuzzleSolving` |
| A surface's actor | struck: neither surface declares one | `surface-actor.PuzzleSetting`, `surface-actor.PuzzleSolving` |
| The crate-only answer | Z `the_answer_is_yes_for_the_solutions_digit_and_no_for_every_other`, `the_answer_is_no_wherever_no_cell_is` | none |
| `Debug` leaves the solution out | Z `two_puzzles_alike_but_for_the_solution_print_alike` | none |
| Each error's `Display` text is stable | P `proof_error_texts_are_stable`; Z `move_error_texts_are_stable` | none |
| Invariant 3 for the public types; serde | `tests/api_bounds.rs`: `public_types_are_send_sync_and_static`, `public_types_are_clone_and_debug`, `streams_and_value_types_serialise_under_the_serde_feature` | none |

48 obligations, 44 mapped to a test, 4 struck with a reason.

### Names later tickets need

**Public now** (`pawdoku::sudoku`), each with a doc example that runs:

| Item | Signature |
| --- | --- |
| `BOX_SIDE` | `pub const BOX_SIDE: u8 = 3` |
| `SIDE` | `pub const SIDE: u8 = BOX_SIDE * BOX_SIDE` |
| `Position` | `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`; `Serialize, Deserialize` under `serde`; `#[non_exhaustive]`, fields private |
| `Position::new` | `pub const fn new(row: u8, column: u8) -> Self` |
| `Position::row`, `Position::column` | `pub const fn row(self) -> u8`, `pub const fn column(self) -> u8` |
| `Given` | the same derives and attributes as `Position` |
| `Given::new` | `pub const fn new(position: Position, digit: u8) -> Self` |
| `Given::position`, `Given::digit` | `pub const fn position(self) -> Position`, `pub const fn digit(self) -> u8` |

**Crate-only**, with the ticket that changes each. All are `#[non_exhaustive]`, carry
doc comments and no examples. `Grid<T>` is `[[T; 9]; 9]`: rows top to bottom, and in
each row the columns left to right.

| Item, by its path today | Signature | Ticket |
| --- | --- | --- |
| `sudoku::Grid<T>` | `pub(crate) type Grid<T> = [[T; LINE]; LINE]` | T26: public, or written out, since `WellPosed::solution` returns it |
| `sudoku::proof::WellPosed` | `Debug, Clone, PartialEq, Eq`; no serde | T26 makes it public |
| `WellPosed::vouch` | `pub(crate) fn vouch(givens: BTreeSet<Given>, solution: Grid<Option<u8>>) -> Result<Self, WellPosedError>` | T26 calls it, from `solve` alone; it stays `pub(crate)` |
| `WellPosed::givens` | `pub(crate) const fn givens(&self) -> &BTreeSet<Given>` | T26 public |
| `WellPosed::solution` | `pub(crate) const fn solution(&self) -> &Grid<u8>` | T26 public |
| `WellPosed::into_parts` | `pub(super) fn into_parts(self) -> (BTreeSet<Given>, Grid<u8>)` | stays: `Puzzle::set` is its one caller |
| `sudoku::proof::WellPosedError` | `Debug, Clone, PartialEq, Eq, thiserror::Error`; variants `NothingLeftToPlay { givens: usize }`, `SolutionNotFull { position }`, `SolutionDigitOutOfRange { position, digit }`, `SolutionConflict { first, second, digit }`, `GivenOffGrid { given }`, `GivenMismatch { given, solution: u8 }` | T26 public, and the source of its refusal's one conversion |
| `sudoku::puzzle::Puzzle` | `Clone`, a hand-written `Debug`; no `PartialEq`, no serde | T26 public |
| `Puzzle::set` | `pub(crate) fn set(proof: WellPosed) -> Self` | T26 public |
| `Puzzle::givens` | `pub(crate) const fn givens(&self) -> &BTreeSet<Given>` | T26 public |
| `Puzzle::status` | `pub(crate) const fn status(&self) -> Status` | T26 public |
| `Puzzle::cell` | `pub(crate) fn cell(&self, position: Position) -> Option<Cell>` | T26 public |
| `Puzzle::cells` | `pub(crate) fn cells(&self) -> impl Iterator<Item = Cell> + '_`, in grid order | T26 public |
| `Puzzle::is_full`, `Puzzle::is_consistent` | `pub(crate) fn is_full(&self) -> bool`, likewise | T26 public |
| `Puzzle::place` | `pub(crate) fn place(&mut self, position: Position, digit: u8) -> Result<(), MoveError>` | T26 public |
| `Puzzle::erase` | `pub(crate) fn erase(&mut self, position: Position) -> Result<(), MoveError>` | T26 public |
| `Puzzle::is_solution_digit` | `pub(crate) fn is_solution_digit(&self, position: Position, digit: u8) -> bool` | T27 calls it; it stays `pub(crate)` |
| `sudoku::puzzle::Status` | `Debug, Clone, Copy, PartialEq, Eq`; `Unsolved`, `Solved` | T26 public |
| `sudoku::puzzle::Cell` | `Debug, Clone, Copy, PartialEq, Eq`; `const fn` getters `row`, `column`, `band`, `stack` (`u8`), `digit` (`Option<u8>`), `is_given`, `is_conflicting` (`bool`) | T26 public |
| `sudoku::puzzle::MoveError` | `Debug, Clone, PartialEq, Eq, thiserror::Error`; variants `NoSuchCell { position }`, `AlreadySolved`, `GivenCell { position }`, `DigitOutOfRange { digit }`, `EmptyCell { position }` | T26 public |

What T26 does with them:

- **The paths.** The two parts are declared `pub(crate) mod proof;` and
  `pub(crate) mod puzzle;` in `src/sudoku.rs`, with no re-export, so today the paths
  are `sudoku::proof::WellPosed` and `sudoku::puzzle::Puzzle`. T26 makes the two
  modules private and adds `pub use proof::{WellPosed, WellPosedError};` and
  `pub use puzzle::{Cell, MoveError, Puzzle, Status};`, which gives the names this
  ticket fixed, `sudoku::WellPosed` and `sudoku::Puzzle`. "Deviations" says why the
  re-export is not here already.
- **The expectations.** Twenty `cfg_attr(not(test), expect(dead_code, ...))` name T26:
  seven in `src/sudoku.rs` (the private `impl Position` block, `LINE`, `Grid`, `at`,
  `at_mut`, `grid_positions`, `in_range`), six in `src/sudoku/proof.rs` (`filled`,
  `consistent`, `matches`, `WellPosedError`, `WellPosed`, its `impl`), and seven in
  `src/sudoku/puzzle.rs` (`Status`, `Cell`, its `impl`, `MoveError`, `Puzzle`, its first
  `impl`, `picture`). Each fails `just clippy` by itself once it is not needed. The one
  on `Puzzle::is_solution_digit` names T27 and sits on that method alone, in an `impl`
  block of its own.
- **`#[must_use]`.** The crate-only getters carry none, because clippy asks it of public
  functions only. It will ask once they are public.
- **Doc examples.** Every item in the second table that T26 makes public needs one,
  made through `solve`.
- **The helpers are private to `sudoku`.** `at`, `at_mut`, `grid_positions`, `in_range`
  and `Position`'s `band`, `stack` and `is_peer_of` are visible in `src/sudoku.rs` and
  the files under `src/sudoku/` only. If the solver wants the peer relation or the grid
  order, widening one to `pub(crate)` is T26's choice.
- **The refusal order of a move.** A position that names no cell is refused first, then
  a solved puzzle, then a given cell, then the digit's range or the empty cell: the
  rules' `requires` clauses in the order they are written. T27's board sees the first
  that fails.

### Snapshots taken

All four are in `crates/pawdoku/src/sudoku/snapshots/`, taken by tests in
`src/sudoku/puzzle.rs` after the list was empty, and each file was read before it was
committed. The render helper draws from `status`, `is_full`, `is_consistent` and the
cells' getters, and nothing was added to the library for it.

| Test | File | Shows |
| --- | --- | --- |
| `snapshot_the_fixture_as_set` | `pawdoku__sudoku__puzzle__tests__snapshot_the_fixture_as_set.snap` | The digits, the given cells and the conflicting cells side by side, then the status, `is_full` and `is_consistent` |
| `snapshot_the_fixture_after_a_short_script` | `..._snapshot_the_fixture_after_a_short_script.snap` | The same picture after each of four moves: place 4 at row 1, column 3; place 5 at row 1, column 4, which conflicts with the givens at row 1, column 1 and row 2, column 6; place 2 over the 4; erase row 1, column 4 |
| `snapshot_the_puzzles_debug` | `..._snapshot_the_puzzles_debug.snap` | `Puzzle`'s pretty `Debug` for the fixture: the status, the givens and the digits as rows, and `..` where the solution is left out |
| `snapshot_every_error_text` | `..._snapshot_every_error_text.snap` | The `Display` text of the six `WellPosedError` and five `MoveError` variants, one line each |

### What was verified, and how

Done on 2026-10-01 in the supplied Supacode worktree, on `b44dc59`, the merge of T29.
The worktree was clean. `.pixi/` and `.tools/bin` were absent: `pixi install --locked`
installed the environment, and the pinned `allium`, `cargo-hack` and `rustqual` 1.8.3
were copied from sibling worktrees' `.tools/bin` in place of `just install-tools`.
`just check` was green before any edit. Commands used rustup's cargo through
`PATH="$HOME/.cargo/bin:$PATH"`.

`just plan-spec sudoku` printed 48 obligations and an empty `diagnostics` array; the
table above accounts for each.

Closing lines of each recipe, from the last run:

| Recipe | Closing output |
| --- | --- |
| `just fmt-check` | `cargo fmt --all --check` (no diagnostic) |
| `just clippy` | ``Finished `dev` profile [unoptimized + debuginfo] target(s)``, no warning |
| `just metrics` | `::notice::Quality score: 100.0% (159 functions analyzed)`, no finding, and the probe satisfied |
| `just test` | `79 tests run: 79 passed, 0 skipped`; `test result: ok. 21 passed; 0 failed` (doctests) |
| `just snapshots-check` | `info: no unreferenced snapshots found`; `info: no snapshots to review` |
| `just wasm-check` | both feature sets on `wasm32-unknown-unknown` and on `wasm32v1-none`, each `Finished` |
| `just features` | `--no-default-features` and `--features serde`, each `Finished` |
| `just coverage` | `TOTAL` lines `99.63%` (1089 lines, 4 missed); `Finished report saved to target/llvm-cov/lcov.info` |
| `just doc` | `Generated .../target/doc/pawdoku/index.html` |
| `just check-docs` | `Validated 41 pages and 42 canonical topics.` |
| `just check` | `The worktree matches the check baseline.`; `All checks passed and the worktree is unchanged.` |

Coverage of the module alone, from the same report:

| File | Lines | Missed | Cover |
| --- | --- | --- | --- |
| `sudoku.rs` | 86 | 0 | 100.00% |
| `sudoku/fixture.rs` | 27 | 0 | 100.00% |
| `sudoku/proof.rs` | 176 | 0 | 100.00% |
| `sudoku/puzzle.rs` | 664 | 1 | 99.85% |

The one missed line is in test code: the fallback of the property tests' "place the
next right digit" move, taken only on a puzzle that already holds its whole solution.
Whether a run's scripts reach it varies, so the figure moves by a line: the `just check`
run on the committed tree reported 99.72% (3 missed) and counted that line as covered.

**The tests that were green on arrival were shown to bite.** Seven deliberate breaks,
each made, run through `just test` and reverted:

| Break | Tests that failed |
| --- | --- |
| An empty cell conflicts with its empty peers | `a_conflict_marks_both_cells`, `an_empty_cell_conflicts_with_nothing`, `consistent_means_no_cell_conflicts` |
| The answer reads the cell's digit, not the solution | `the_answer_is_yes_for_the_solutions_digit_and_no_for_every_other` |
| `Debug` prints the solution | `two_puzzles_alike_but_for_the_solution_print_alike` |
| Placing the digit a cell already holds is refused | `a_move_is_accepted_exactly_when_its_rule_allows_it`, `a_move_is_refused_wherever_it_is_not_offered` |
| A box is not a unit | `a_proof_is_refused_for_two_peers_holding_one_digit` |
| A given cell is the player's until the puzzle is solved | both properties above, `a_refused_move_leaves_the_puzzle_as_it_was`, and the two `..._on_a_given_cell` tests |
| Solved compares the grid with the stored solution | `solved_is_not_a_comparison_with_the_stored_solution` |

Those runs stop at the first failures, so each row is what failed before the run
stopped, not everything that would have. The `proptest-regressions/` file the breaks
left behind records failures of broken code and was deleted, not committed.

Acceptance criteria read against the code: no path under `src/sudoku` names another
engine module, and `just metrics` holds it; `WellPosed` is made only by `vouch`, and
`Puzzle::set` takes one; neither type derives a serde trait; outside test code there is
no `unwrap`, `expect`, `panic!`, `#[allow]`, `qual:allow` or indexing, every lookup
going through `get`; and the only expectations added are the twenty-one `dead_code`
ones, each under `cfg_attr(not(test), ...)`.

### Deviations, and why

- **The module is split**, as the ticket allows when a limit asks. With every
  `dead_code` expectation in place, `just metrics` reported
  `SRP module length: src/sudoku.rs has 524 lines` against the cap of 500. It is now
  `src/sudoku.rs` (the figures, the value types and the grid helpers),
  `src/sudoku/proof.rs`, `src/sudoku/puzzle.rs`, and `src/sudoku/fixture.rs`, a
  `#[cfg(test)]` module holding the fixture both parts' tests share. The snapshots are
  therefore under `src/sudoku/snapshots/`.
- **`sudoku::WellPosed` and `sudoku::Puzzle` are not yet paths.** With the types in
  child modules, the names would need `pub(crate) use` lines in `src/sudoku.rs`, and
  rustc reports a crate-only re-export nothing uses as `unused_imports`. The ticket
  allows `dead_code` expectations and no other, so the re-exports wait for T26, when
  they are `pub use` and need none. Until then the paths are `sudoku::proof::WellPosed`
  and `sudoku::puzzle::Puzzle`.
- **Twenty-one expectations, private helpers among them.** Nothing outside tests reaches
  the proof or the puzzle, so clippy reports every item they alone use, down to
  `grid_positions`. Each carries the expectation, as the ticket says. `Position`'s
  private helpers sit in an `impl` block of their own so that one expectation covers
  them and none touches its public methods.
- **`docs/reference/configuration.md` is edited**, though the ticket does not list it:
  its sentence named `pawdoku::SIDE`. The maintainer approved the edit in the session.
- **`CHANGELOG.md` says the constant moved** inside the Added entry, not under a
  Changed heading: the ticket names Added, and the crate is unreleased.
- **The branch** is `ticket/T25-sudoku-rules`, as Supacode made it; the maintainer chose
  to keep it and record the difference from the `branch:` field.
- **The loops were not all one line each.** The value types went in one loop of three
  lines, and several lines were green on arrival; "Changes to the list" says which, and
  the deliberate breaks are how those were shown to prove something.
- **Two commands outside the recipes.** `cargo fmt --all` was run once before
  `just format` was used throughout, and one `cargo nextest run --no-fail-fast` was
  tried for the deliberate breaks and replaced by `just test`. Neither is needed to
  reproduce anything here.
- **Choices the ticket left open**, for review: the fields are `u8`; the constructor is
  named `vouch`, so that a call to it reads as the claim it is and is easy to find in
  review; one `MoveError` serves both moves; `Puzzle` has no `is_complete` accessor,
  because the surface exposes `is_full` and `is_consistent` and no more; `Puzzle` has
  no `PartialEq`, because a derived one would compare the stored solution; and
  `WellPosed` has no `Serialize`, because nothing asks for one yet. The maintainer
  confirmed `u8` in review round 1, below.

### Review round 1

Pull request 26, reviewed at `8041afa` by Codex and by Copilot on 2026-10-01. They
raised the same two findings, in four comments.

- **Signed integers for `Position` and `Given`: declined**, by the maintainer's
  decision. The finding: `sudoku.allium` gives both an `Integer`, a `u8` cannot hold
  -1, and so a malformed value never reaches `RefuseMalformedGivens`. It does reach it.
  A row, a column or a digit may be 0 or anything from 10 to 255, so a position off the
  grid on either side and a digit out of range on either side can all be made, and
  `a_proof_is_refused_for_a_given_off_the_grid` and
  `placing_is_refused_where_no_cell_is` refuse (0, 1), (1, 0), (10, 1) and (1, 10).
  What a `u8` leaves out is the negatives, and any finite type leaves something out.
  The docs of `Position` and `Given` now say which values each admits.
- **Stale pages: accepted for the two READMEs, deferred for the tutorial.** `README.md`
  and `crates/pawdoku/README.md` said the crate held the randomness boundary alone;
  both now name the figures and the value types of `sudoku`. Neither page was in this
  ticket's list; the edit is a sentence in each. The tutorial stays handed back, by the
  maintainer's decision, and has an owner: `tickets/T30-first-change-tutorial.md`,
  drafted in this round and added to the index.

`just check` was green on the tree this round committed.

### Handed back

T25 is done here and in the ticket index.

One page outside this ticket's list still describes the crate as it was:

- `docs/tutorials/first-change.md` walks a reader through adding `BOX_SIDE` beside
  `SIDE` in `lib.rs`. Both now exist, in `sudoku`, so the exercise cannot be followed.
  It needs a new exercise, which is a choice for the maintainer and not a sentence to
  correct. T30 owns it.

For T26: the list under "Names later tickets need", the re-exports, the expectations
to remove, and the `compile_fail` example this ticket leaves to it.

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
- **The `compile_fail` example**, which T26 writes, is the only mechanical proof that
  outside code cannot make a proof. If the maintainer wants a stronger one, it is a UI
  test and a new dependency, which is a T02 hand-back under decision 0007.
- **A second fixture.** One puzzle is enough for the rules. T26 needs more (a puzzle
  that takes a guess, givens with several solutions, givens with none) and finds them
  itself.
