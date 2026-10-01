---
id: T25
title: "The rules in Rust: the sudoku module, the proof value and Puzzle"
status: open
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

### Names later tickets need

### Snapshots taken

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
- **The `compile_fail` example**, which T26 writes, is the only mechanical proof that
  outside code cannot make a proof. If the maintainer wants a stronger one, it is a UI
  test and a new dependency, which is a T02 hand-back under decision 0007.
- **A second fixture.** One puzzle is enough for the rules. T26 needs more (a puzzle
  that takes a guess, givens with several solutions, givens with none) and finds them
  itself.
