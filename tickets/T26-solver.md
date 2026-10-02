---
id: T26
title: "The solver in Rust: the search, its result and the proof"
status: done
depends_on: [T25]
parallel_with: []
branch: ticket/t26-solver
estimated_size: L
---

# T26: The solver in Rust: the search, its result and the proof

## Context

T25 built `pawdoku::sudoku`: the value types, the proof value `WellPosed` and `Puzzle`.
Nothing can make a proof yet, so T25 left both crate-only. This ticket builds
`pawdoku::solver` from `docs/specs/solver.allium`: the search that gives a set of givens
its verdict of none, one or many. It is the one caller of the proof's constructor, and
it makes `WellPosed` and `Puzzle` public, because outside code can now make them.

The design was decided by the maintainer on 2026-09-30 and is recorded in the decision
T23 wrote (`docs/decisions/0014-board-imports-solver.md`, unless renumbered). What this
ticket needs from it:

- **One solver, not swappable.** No trait and no generic parameter for the solver, and no
  optional-solver argument anywhere.
- **No step budget.** `solver.allium` has none: its search always concludes
  (`AlwaysConcludes`). Solving takes the givens alone. No randomness either: nothing in
  the search is left to chance (`SameGivensSameResult`).
- **Two public entries.**
  - `solver::search(givens)` returns what the `SearchResult` surface exposes: the
    verdict, the count of guesses, and the solutions found, none, one or two. It never
    fails: "Nothing is required of the givens", and the worst of them get `none`.
  - `solver::solve(givens)` is built on `search` and returns the proof or a refusal. It
    refuses three ways: the givens have no solution, they have several, or they leave
    nothing to play. The third is the proof constructor's own guard: eighty-one valid
    givens get the verdict `one` from `search`, and `SetPuzzle` still refuses them.
- **Only the solver calls the proof's constructor.** The compiler cannot hold that
  inside the crate; review does. This ticket makes the one call.

Facts that shape the work:

- **Storage is free, the search's shape is not.** The module's Excludes leave bitmasks,
  trails and copies to the implementation: "Another solver that keeps every clause here
  is the same solver." But `guesses` is exposed and the same every time, so the
  implementation makes exactly the splits the specification's search makes. It
  propagates until nothing is left: a placed digit leaves its peers' candidates, a cell
  with one candidate takes it, and a digit with one place left in a unit is placed
  there. A contradiction ends a branch at once. A settled branch that is not full is
  split on a cell with the fewest candidates, one child for each candidate. The deepest
  waiting branch is taken first. The search stops at the second solution.
- **The tie-break is the implementation's and must be fixed.** The module leaves "the
  order in which tied cells and tied branches are taken" open but says it "must be the
  same each time". Choose a rule, such as the first cell in row order and then column
  order and the lowest digit first, write it in the module's documentation, and pin it:
  a test fixes the guess count of at least one puzzle that needs a guess. Say in that
  test that the number follows from the tie-break and not from the specification.
- **No recursion.** `rustqual.toml` sets `allow_recursion = false`, and decision 0013
  says a search keeps an explicit stack. That holds for test code too.
- **The rest of the metrics gate** (`rustqual.toml`, `clippy.toml`): a function at most
  60 lines, cognitive complexity 15, cyclomatic 10, nesting 4, five parameters; a struct
  at most 12 fields and 20 methods, LCOM4 at most 2; a file at most 500 code lines
  before its first `#[cfg(test)]`, and 1000 for test code. The solver is the module
  these limits were set for. Meet them by structure: `src/solver.rs` plus
  `src/solver/<part>.rs` (propagation, the branch stack, the result), which the boundary
  rule `solver_imports_sudoku` already covers; `mod.rs` is banned. Never by a
  suppression and never by moving a number. If structure cannot meet one, stop and
  report.
- **The search is synchronous.** `SearchResult` exposes `search.status`. A caller of
  `search` is handed a search only once it has concluded, and T23 added the sentence
  to `solver.allium` that says so: such a caller "reads the status from that alone:
  being handed it says concluded". The surface's `search.status` is met by the return
  itself, so the result has no status field and no accessor. Say so in the result
  type's documentation, and mark `search.status` in the hand-back table as exposed by
  the return, with that sentence as the reason. If the sentence is not above
  `surface SearchResult` when this ticket starts, stop and report: without it the
  surface asks for a status a caller reads as a value.
- **The proof constructor's other errors cannot come from a correct search.** A solution
  that is not full, or a given that does not match it, would be a bug here. `solve` must
  still not panic. Give the refusal one conversion from the constructor's error and test
  that conversion directly; do not write a branch per impossible case that no input can
  reach, because unreachable lines count against the coverage floor.
- **Snapshots show and detect change; they prove nothing.** T29 added insta and the rule,
  which `docs/reference/testing.md` states: a snapshot shows a reviewer what a value
  looks like and makes a change to it visible in a diff. It is never the test of a
  clause, so no line of "What the tests must cover" and no row of the hand-back table
  names one. Snapshots are `.snap` files, taken from fixed inputs after the module's
  own tests are green, by tests named `snapshot_...`, from a text picture rendered in
  test code. No `Display`, method or field is added to the library to feed one.
  Here they are taken through the public API, in one integration test file of their
  own, `crates/pawdoku/tests/snapshots.rs`. T25's
  snapshots stay where they are; if this ticket's edits change one, the diff is read and
  the reason given in the hand-back notes.
- **T25 left the proof and the puzzle for this ticket.** `WellPosed`, `Puzzle` and the
  errors only they return are `pub(crate)`, with no doc examples; T25's hand-back notes
  list them. Make each public. Once `solve` calls the constructor and the types are
  public, every `dead_code` expectation naming T26 is unfulfilled, and `just clippy`
  fails until its `cfg_attr` line is removed. The one on the puzzle's answer names T27
  and stays. Each item made public gets a doc example that makes its value through
  `solve` and asserts.

The fixture from T25, with thirty givens, one solution, and solved by naked and hidden
singles alone, so its guess count is zero:

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

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/solver.allium` and
`docs/specs/sudoku.allium` in full; the decision T23 wrote; `tickets/T25-sudoku-rules.md`,
hand-back notes ("Names later tickets need"); `crates/pawdoku/src/sudoku.rs`;
`docs/explanation/layering.md`, `docs/explanation/architecture.md` and
`docs/explanation/solving-sudoku.md` (the module's cited source);
`docs/reference/testing.md`, its Snapshots section included;
`.agents/skills/rust-change/SKILL.md` and
`.agents/skills/propagate/SKILL.md`; `rustqual.toml` and `clippy.toml`. Background only,
never modified: `/Users/scutting/projects/pawdoku/prototypes/board/solver.py`, a plain
backtracking counter that keeps none of this module's clauses about propagation or
guesses.

## Goal

`pawdoku::solver` exists and does what `solver.allium` says. `search` gives any set of
givens a true verdict, the solutions and the guess count; `solve` gives the proof or one
of three refusals. A puzzle can now be made from outside the crate: solve, then set.
Every rule, invariant, guarantee and surface clause of the module has a named test or a
stated reason, listed in the hand-back notes. `just check` is green with the coverage
floor held by this module's own tests and no suppression.

Names later tickets are written against, fixed here: `solver::search` and
`solver::solve`; and `sudoku::WellPosed` and `sudoku::Puzzle` become public. Every other
name is this ticket's to choose and to report.

## Non-goals

- No deduction past singles: no locked candidates, subsets, fish or chains. Those are
  `technique.allium`'s.
- No solving from a puzzle in play. The search reads givens and nothing a player placed.
- No trait for the solver, no generic over it, no budget, no randomness.
- No benchmark, no fuzz target and no mutation job. Each is named under Handed back as a
  trigger this ticket meets (step 8).
- No change to `solver.allium` or any module. If no clause says what the code needs,
  stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No snapshot standing in for a test that asserts, and nothing added to the library so
  that a snapshot can read it.
- No change to `sudoku`'s behaviour. The edits to `src/sudoku.rs` are the visibility
  T25 deferred, the removed expectations and the doc examples.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/solver.rs` | New: the module's surface |
| `crates/pawdoku/src/solver/` | New: one file per part, as the limits ask |
| `crates/pawdoku/src/sudoku.rs` | `WellPosed`, `Puzzle` and their errors made public; each `dead_code` expectation naming T26 removed, the whole `cfg_attr` line; their doc examples, running through `solve`, and the `compile_fail` example. If T25 split the module, the file under `src/sudoku/` that holds them |
| `crates/pawdoku/src/lib.rs` | `pub mod solver;` and the crate overview |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type, `WellPosed` and `Puzzle` among them |
| `crates/pawdoku/tests/solver.rs` | New: both entries through the public API |
| `crates/pawdoku/tests/sudoku.rs` | `Puzzle` through the public API, now that one can be made: the `PuzzleSolving` surface end to end |
| `crates/pawdoku/tests/snapshots.rs` | New: the render helper and the snapshot tests of step 6 |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | The solver's place: the two entries, the one call to the constructor |
| `docs/reference/testing.md` | The suite table; the oracle; the pinned guess counts and what they depend on |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T26's row set `done` |
| `tickets/T26-solver.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t26-solver` from `main` after T25 has merged. Run
   `just initialize` if `.pixi/` is absent; it uses the network and this ticket
   authorises it. Run `just check` and confirm it is green before any edit.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec solver` and add any obligation the list lacks.
   Many of its 101 obligations name the fields of `Branch` and `BranchCell`. Where the
   implementation has that structure, test it; where it does not, strike the line with
   the reason that the module excludes how the search is stored, and name the test of
   the observable result that stands in for it. Put the list in the hand-back notes
   under "The test list" before the first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. Run `just clippy` and `just metrics` every few loops.
   A good first line is the smallest whole behaviour: malformed givens get `none` with
   no guess. Then a puzzle singles alone solve; then one that needs a guess; then two
   solutions.

4. **The oracle.** `VerdictIsTrue` ties the verdict to `sudoku.allium`'s
   `solution_count`, so it needs a count the solver did not produce. Write, in test code
   only, a plain counter that tries digits cell by cell with no propagation and stops at
   two, iterative because recursion is forbidden. A property compares it with `search`.
   Keep it fast: derive givens from the fixture's solution by removing cells, which gives
   `one` or `many` and never a long search; for `none`, spoil one given of a dense set.
   A sparse set with no solution can take a plain counter a very long time, so do not
   generate those. Find a published puzzle that needs at least one guess under singles
   alone and a set of givens with exactly two solutions, and cite where each came from.

5. **`solve` and the hand-over to `sudoku`.** Call the proof's constructor with the
   givens and the one solution when the verdict is `one`. Make public the items T25
   listed, and remove every `dead_code` expectation naming T26. Give each newly public
   item a doc example that makes its value through `solve` and asserts. Add the
   `compile_fail` example that shows outside code cannot call the proof's constructor;
   such an example passes for any compile error, so keep it to the one line that names
   the constructor. Add to `tests/sudoku.rs` what could not be written before: a puzzle
   set from outside the crate, played and solved through its public API.

6. **Snapshots**, once the list is empty. Create `crates/pawdoku/tests/snapshots.rs`.
   It holds the render helper and every snapshot test taken through the public API,
   and T27 and T28 add theirs to it. One file is deliberate. A helper module shared
   between two files under `tests/` is compiled into each, and each then reports the
   functions it does not call as dead code, which `just clippy` refuses and which no
   `#[expect]` may silence here. One file also keeps the snapshot tests apart from the
   tests that assert. The helper draws a grid as nine rows of digits with a dot for an
   empty cell, and a search result as its verdict, its guess count and each solution
   as a grid. It reads only the public API, and no line ends in a space. Then take
   these, each by a test named `snapshot_...`:

   - the `search` result for the fixture;
   - for the puzzle that needs a guess;
   - for the givens with two solutions, both grids shown;
   - for one set of malformed givens;
   - the proof `solve` gives for the fixture, givens and solution. This file holds a
     solution on purpose: the proof exposes it;
   - the `Display` text of the three refusals, in one snapshot.

   The guess counts these files show follow from the tie-break, as the pinned test
   says. Accept with `just snapshots-accept` and read each file before committing it.
   List them in the hand-back notes under "Snapshots taken".

7. **Doc examples and the pages.** Every public item this ticket adds has a doc
   example that runs and asserts (`AGENTS.md`: "a doctest on every public item"):
   `search`, `solve`, the result and each of its accessors, the verdict, the refusal,
   and the givens type if it is new. That is beside the items of step 5, which T25
   left. `missing_docs` and `just doc` do not
   notice an absent example, so the hand-back notes list each public item beside the
   doctest that covers it; `just test` runs them.

   `docs/explanation/architecture.md`: the two entries and which a caller
   wants; that the solver runs once per puzzle. `docs/reference/testing.md`: the suite
   rows, the oracle, that the pinned guess counts follow from the tie-break, and the
   snapshots taken.
   `docs/project/repository-map.md` and `CHANGELOG.md` follow.

8. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`.
   Quote the closing lines of each, the coverage line for the module, and how long
   `just test` takes. Fill in the hand-back notes: the test list as it ended, as a table
   from clause to test name or reason; "Names later tickets need", with every public
   item's signature, the givens type `search` takes and the three refusals; and under
   Handed back, the triggers this ticket meets, each for the maintainer to pick up and
   none built here:

   - S03's benchmark recipe, whose trigger is "once the solver exists".
   - T14, the mutation job S03 drafted, whose trigger is "once tests exist".
   - T16, the command-line crate S04 drafted, whose `solve` subcommand "lands with the
     solver".
   - T20 and T21, drafted in T19's hand-back notes, each of which waits for the solver
     in Rust.

   Set `status: done` here and in `tickets/README.md`, commit on the ticket branch, and
   stop before pushing.

### What the tests must cover

From `docs/specs/solver.allium`, by name.

- **`RefuseMalformedGivens`.** A given off the grid in either direction, a digit out of
  range, and two different digits to one position each get `none`, with no guess and no
  solution. Two givens that agree are one given.
- **`BeginSearch`.** Anything may be handed over: the empty set, conflicting givens,
  eighty-one givens.
- **Propagation**: `EliminateFromPeers`, `PlaceNakedSingle`, `PlaceHiddenSingle`. The
  fixture is solved with zero guesses. A puzzle a hidden single alone unlocks shows the
  third rule separately from the second.
- **Contradiction**, the three kinds `DecideBranch` reads: a cell with no candidate, a
  unit with no place for a digit, and a cell that is the only place for two digits. Each
  ends its branch with no guess beneath it. Givens in conflict are found out this way
  and get `none`.
- **Splitting.** A settled branch that is not full is split on a cell with the fewest
  candidates, one child for each candidate, and `guesses` rises by one for each split.
- **`ConcludeMany` and `ConcludeExhausted`.** `many` at the second solution, with both
  exposed and different; `one` with its one solution; `none` with no solution.
- **`AlwaysConcludes`.** The empty set of givens concludes, with `many`. A property over
  arbitrary givens, malformed ones included, always returns.
- **`VerdictIsTrue`.** The verdict agrees with the oracle: `none` for a count of 0, `one`
  for 1, `many` for 2 or more.
- **`NeverCountsPastTwo`.** No result holds more than two solutions; the empty grid
  stops at two.
- **`GuessesAreTheLastResort`.** A puzzle singles alone solve reports zero guesses.
- **`SameGivensSameResult`.** The same givens give the same verdict, solutions and guess
  count on every call, and whatever order the givens are handed over in.
- **`SearchResult`.** Everything the surface exposes can be read: the verdict, the guess
  count, and each solution's digit at every position. `search.status` is the one
  with no value of its own: under T23's sentence, being handed the result says
  concluded.
- **The invariants a result can show:** `SolvedBranchesAreSolutions` (each solution is
  full, conflict-free and holds every given), `SolutionsAreDistinct`,
  `NoMoreSolutionsThanSought`, `VerdictMatchesSolutions`. The other eight
  (`OneBranchWorksAtATime`, `TheDeepestBranchWorks`, `ChildrenSitBelowTheirParent`,
  `OnlySplitBranchesHaveChildren`, `GuessesAreOnFewestCandidates`,
  `PlacedCellsKeepOnlyTheirDigit`, `CandidatesAreDigits`,
  `ConcludedLeavesNothingOpen`) are about the search's inside: test each where the
  implementation has the structure it names, and give a reason where it does not.
- **`solve`.** The proof for the fixture, holding its givens and its solution; a refusal
  for no solution, for several, and for eighty-one givens that agree with a valid grid;
  the conversion from the constructor's error, tested directly.
- **`OnlyWellPosedPuzzlesArePosed`, from outside.** With `WellPosed` public, the
  `compile_fail` example shows that outside code cannot call its constructor.
- **The pinned tie-break.** The guess count of one puzzle that needs a guess.

## Acceptance criteria

- `pawdoku::solver` exports `search` and `solve` with the meanings above. Nothing in
  `src/solver` names an engine module other than `sudoku`.
- The only caller of the proof's constructor outside test code is in `src/solver`, read
  by inspection and stated in the hand-back notes.
- `solver` has no trait, no generic parameter for a solver, no budget argument and no
  use of `random`.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec solver` obligation is in the table or struck with a
  reason.
- No recursion, in code or tests. No `#[allow]`, no `#[expect]` added, no `qual:allow`
  line; every expectation naming T26 is gone, and the answer's, naming T27, remains.
- `pawdoku::sudoku` exports `WellPosed` and `Puzzle`. Each item T25 left crate-only for
  this ticket is public, has a doc example that runs and asserts, and is named in
  `tests/api_bounds.rs` if it is a type.
- Every public item of `solver` has a doc example that runs and asserts, and the
  hand-back notes list each beside its doctest.
- The result has no status field or accessor, its documentation says why, and
  `solver.allium` carries T23's sentence.
- Line coverage of `src/solver` alone is at or above 90 per cent.
- The six snapshots of step 6 exist as `.snap` files, each taken by a test named
  `snapshot_...` through the public API. None appears in the hand-back table as the
  test of a clause. `just snapshots-check` is green, and any snapshot of T25's that
  changed has its reason in the hand-back notes.
- `just check` is green.

## Verification

```sh
just plan-spec solver
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
`metrics` reporting no finding, no recursion among them; both wasm targets under both
feature sets; the coverage total at or above the floor; `All checks passed and the
worktree is unchanged.`

## Hand-back notes

### The test list

The list as first written on 2026-10-01, before any test or code, seeded from "What the
tests must cover" and then checked against the 101 obligations of `just plan-spec solver`.
One line is one expected test: its name, then what it expects. O is
`crates/pawdoku/tests/solver.rs`, through the public API; B is the unit tests of the
branch, the grid of candidates, under `src/solver/`; S is `src/solver.rs`.

`RefuseMalformedGivens` and `BeginSearch` (O):

1. `a_given_off_the_grid_gets_none_with_no_guess`: row 0, row 10, column 0 and column 10
   each get `none`, zero guesses and no solution.
2. `a_digit_out_of_range_gets_none_with_no_guess`: 0, 10 and 255.
3. `two_digits_to_one_position_get_none_with_no_guess`.
4. `two_givens_that_agree_are_one_given`: the fixture with a given handed over twice
   gets what the fixture gets.
5. `anything_may_be_handed_over`: the empty set, givens in conflict and eighty-one
   givens each come back with a verdict: `many`, `none` and `one`.

Propagation (O, then B for each rule alone):

1. `singles_alone_solve_the_fixture_with_no_guess`: `one`, zero guesses, the fixture's
   solution. `GuessesAreTheLastResort` as far as a result shows it.
2. `hidden_singles_solve_a_puzzle_with_no_naked_single`: givens after which no empty
   cell has one candidate, solved with zero guesses.
3. B `a_placed_digit_leaves_its_peers_candidates`: `EliminateFromPeers`.
4. B `a_cell_with_one_candidate_takes_it`: `PlaceNakedSingle`.
5. B `a_digit_with_one_place_in_a_unit_is_placed_there`: `PlaceHiddenSingle`, in a row,
   in a column and in a box, on a cell that has other candidates.
6. B `the_root_holds_each_given_placed_and_every_other_cell_open`: `LayOutRootBranch`.

Contradiction (O for the result, B for the kind):

1. `givens_in_conflict_get_none_with_no_guess`: one digit twice in a row, in a column
   and in a box.
2. `a_cell_with_no_candidate_ends_the_search_with_no_guess`.
3. `a_unit_with_no_place_for_a_digit_ends_the_search_with_no_guess`.
4. `a_cell_that_is_the_only_place_for_two_digits_ends_the_search_with_no_guess`.
5. B `each_kind_of_contradiction_is_read_from_the_cells`: the three sets of givens
   above, each showing its own kind and neither of the others before anything is placed.
6. B `two_peers_placed_with_one_digit_each_lose_it`.
7. B `a_cell_that_is_the_only_place_for_two_digits_is_not_placed`.

Splitting (O, B):

1. `a_published_puzzle_that_needs_a_guess_is_solved`: Norvig's hard puzzle gets `one`,
   its published solution, and at least one guess.
2. `the_guess_counts_follow_from_the_tie_break`: the pinned counts.
3. B `a_branch_is_split_on_the_first_cell_with_the_fewest_candidates`: one child for
   each candidate, the lowest digit first, each child its parent with the guess placed
   (`LayOutChildBranch`).

The verdict (O):

1. `the_second_solution_concludes_many_with_both_exposed`: two solutions, different.
2. `one_solution_concludes_one` and `no_solution_concludes_none`: with the fixture and
   with the fixture spoiled.
3. `the_empty_grid_concludes_many_and_stops_at_two`: `AlwaysConcludes` on the hardest
   case for a search that did not go depth first, and `NeverCountsPastTwo`.

Guarantees and invariants, by property (O):

1. `every_search_concludes`: arbitrary givens, malformed ones included, return.
2. `the_verdict_agrees_with_the_oracle`: `VerdictIsTrue`, against a plain counter.
3. `the_same_givens_get_the_same_result_in_any_order`: `SameGivensSameResult`.
4. `a_result_keeps_its_invariants`: `SolvedBranchesAreSolutions`,
   `SolutionsAreDistinct`, `NoMoreSolutionsThanSought` and `VerdictMatchesSolutions`,
   each asserted under its own name.
5. `the_result_exposes_the_verdict_the_guesses_and_every_digit`: `SearchResult`.
6. B `the_inside_of_a_search_keeps_its_invariants`: `PlacedCellsKeepOnlyTheirDigit`,
   `CandidatesAreDigits` and `GuessesAreOnFewestCandidates` on every branch of a search.

`solve` (O, S):

1. `solve_gives_the_proof_for_the_fixture`: holding its givens and its solution.
2. `solve_refuses_givens_with_no_solution`, `..._with_several_solutions` and
   `..._that_leave_nothing_to_play`.
3. `solve_gives_a_proof_exactly_when_the_verdict_is_one`: by property, eighty-one
   givens apart.
4. S `the_refusal_converts_from_the_proofs_error`: the conversion, called directly.
5. `refusal_texts_are_stable`.
6. S `two_solutions_are_sought`: `config.solutions_sought`.

From outside the crate (`tests/sudoku.rs`, `tests/api_bounds.rs`, doctests):

1. `a_puzzle_set_from_outside_is_played_and_solved`: solve, set, play, solved.
2. `the_surface_can_be_read_from_outside`: everything `PuzzleSolving` exposes.
3. The `compile_fail` example on `WellPosed`: `OnlyWellPosedPuzzlesArePosed` from outside.
4. Every new public type in the two bounds tests.

Expected to be struck, with the observable test that stands in: the obligations that
name `Branch.status`, `parent`, `depth` and `guess`, and the search's projections over
branches. The implementation is expected to keep a stack of grids and nothing else of a
branch.

Changes to the list, one line each, as the loops ran:

- The first run was against a `search` that answered `none` for everything. The three
  malformed-givens lines and `two_givens_that_agree_are_one_given` were green against
  it, as the ticket's suggested first line expects, and
  `singles_alone_solve_the_fixture_with_no_guess` failed on its assertion (no solution where
  one was expected).
- The search was then written whole, not one rule to a loop: the grid of candidates, the
  three rules, the three kinds of contradiction, the split and the stack went in
  together, because the fixture cannot go green on less than all of propagation. The
  fixture line went green with it, and every later line was green on arrival. Each was
  then shown to bite by breaking the code on purpose; "What was verified" lists the
  breaks. "Deviations" records this against step 3.
- Split in two: `one_solution_concludes_one` and `no_solution_concludes_none`.
- Added: `a_split_counts_one_guess_whatever_its_children`, because the pinned counts
  alone did not say that a split is one guess whatever the number of its children.
- Added: `the_oracle_counts_the_fixtures`, so the oracle is itself held to known counts
  before the solver is compared with it.
- Added: four tests of the candidate set, two of the unit tables, and
  `a_position_has_a_place_exactly_when_it_is_on_the_grid`, for the storage the branch
  stands on.
- Added: `a_digit_is_the_sole_place_in_the_unit_the_picture_names`, which holds each
  hidden-single picture to the one unit it claims. It caught the first box picture,
  which made the cell the only place in all three units.
- Added from outside the crate: `each_refusal_reads_as_its_text`,
  `the_moves_are_offered_and_refused_from_outside` and
  `a_puzzle_set_from_outside_does_not_print_its_solution`.
- Corrected: `a_result_keeps_its_invariants` first failed on its own assertion, which
  called a result with no solutions "two equal solutions". The test was wrong and the
  solver was not; the `proptest-regressions/` file it left was deleted.
- Corrected: the first strategy of `the_inside_of_a_search_keeps_its_invariants` spoiled
  nearly every case, so no search it made ever split. The coverage report showed the
  lines that check `GuessesAreOnFewestCandidates` never ran. The strategy now thins the
  solution until searches split, and the report shows those lines run.
- Struck, as expected: the obligations about a branch's status, parent, depth and guess
  and about which branch works next. The table gives each its reason and the test that
  stands in.

The list as it ended, from clause to test. O is `crates/pawdoku/tests/solver.rs`, S is
`src/solver.rs`, B is `src/solver/branch.rs` and C is `src/solver/candidates.rs`. No row
names a snapshot test.

| Clause | Test, or the reason there is none | `plan-spec` obligations |
| --- | --- | --- |
| `BeginSearch`: anything may be handed over, and a search begins with no guess | O `anything_may_be_handed_over` | `rule-success.BeginSearch`, `rule-entity-creation.BeginSearch.1`, `surface-provides.Solving` |
| `RefuseMalformedGivens`: a given off the grid | O `a_given_off_the_grid_gets_none_with_no_guess` | `rule-success.RefuseMalformedGivens`, `when-set.RefuseMalformedGivens.Search.verdict` |
| `RefuseMalformedGivens`: a digit out of range | O `a_digit_out_of_range_gets_none_with_no_guess` | none |
| `RefuseMalformedGivens`: two digits to one position | O `two_digits_to_one_position_get_none_with_no_guess` | none |
| Two givens that agree are one given | O `two_givens_that_agree_are_one_given` | none |
| `givens_are_well_formed`, and `OpenRootBranch` only for givens that are | B `no_root_is_opened_for_givens_that_are_not_well_formed` | `derived.Search.givens_are_well_formed`, `rule-failure.OpenRootBranch.1`, `rule-failure.RefuseMalformedGivens.1` |
| `OpenRootBranch` and `LayOutRootBranch`: each given placed, every other cell open | B `the_root_holds_each_given_placed_and_every_other_cell_open` | `rule-success.OpenRootBranch`, `rule-entity-creation.OpenRootBranch.1`, `rule-success.LayOutRootBranch`, `entity-relationship.Branch.cells`, `entity-optional.BranchCell.digit` |
| A cell's row, column, band and stack: its place in the grid and its units | B `a_position_has_a_place_exactly_when_it_is_on_the_grid`, `the_units_are_nine_rows_nine_columns_and_nine_boxes` | `entity-fields.BranchCell` |
| `row_mates`, `column_mates`, `box_mates` and `peers` | B `a_cells_mates_are_its_twenty_peers_by_row_column_and_box` | `entity-relationship.BranchCell.row_mates`, `entity-relationship.BranchCell.column_mates`, `entity-relationship.BranchCell.box_mates`, `entity-relationship.BranchCell.peers` |
| `CandidatesAreDigits`, as the type holds it | C `every_digit_is_one_to_nine_each_once`, `candidates_are_digits_whatever_is_offered`, `sets_join_part_and_overlap`, `a_single_is_a_set_of_exactly_one_digit` | none |
| `EliminateFromPeers` | B `a_placed_digit_leaves_its_peers_candidates` | `rule-success.EliminateFromPeers`, `derived.BranchCell.has_stale_candidate` |
| A placed cell loses its own digit if a peer is placed with it | B `two_peers_placed_with_one_digit_each_lose_it` | none |
| `PlaceNakedSingle`; a placed cell is not placed again | B `a_cell_with_one_candidate_takes_it` | `rule-success.PlaceNakedSingle`, `rule-failure.PlaceNakedSingle.2`, `derived.BranchCell.is_naked_single` |
| `PlaceHiddenSingle`, in a row, a column and a box; a cell with several candidates is no naked single | B `a_digit_with_one_place_in_a_unit_is_placed_there`, `a_digit_is_the_sole_place_in_the_unit_the_picture_names` | `rule-success.PlaceHiddenSingle`, `rule-failure.PlaceNakedSingle.3`, `derived.BranchCell.is_hidden_single` |
| `PlaceHiddenSingle` requires a cell that is not overdemanded | B `a_cell_that_is_the_only_place_for_two_digits_is_not_placed` | `rule-failure.PlaceHiddenSingle.3` |
| Propagation: singles alone solve the fixture; `GuessesAreTheLastResort` | O `singles_alone_solve_the_fixture_with_no_guess` | none |
| Propagation: the third rule apart from the second | O `hidden_singles_solve_a_puzzle_with_no_naked_single` | none |
| Contradiction: the three kinds `DecideBranch` reads, each alone | B `each_kind_of_contradiction_is_read_from_the_cells` | `derived.BranchCell.is_contradictory`, `transition-edge.Branch.working.contradicted` |
| Contradiction: a cell with no candidate ends its branch with no guess | O `a_cell_with_no_candidate_ends_the_search_with_no_guess` | none |
| Contradiction: a unit with no place for a digit | O `a_unit_with_no_place_for_a_digit_ends_the_search_with_no_guess` | none |
| Contradiction: a cell that is the only place for two digits | O `a_cell_that_is_the_only_place_for_two_digits_ends_the_search_with_no_guess` | none |
| Contradiction: givens in conflict are found out this way | O `givens_in_conflict_get_none_with_no_guess` | none |
| `DecideBranch`: settled and filled is solved; a full grid has nothing left to propagate; only a split has children | B `singles_alone_settle_the_fixture_into_its_solution` | `rule-success.DecideBranch`, `derived.Branch.is_settled`, `derived.Branch.is_decided`, `derived.Branch.is_filled`, `transition-edge.Branch.working.solved`, `rule-failure.PlaceHiddenSingle.2`, `projection.Branch.placed_cells` |
| Splitting: on a cell with the fewest candidates, one child for each candidate; `LayOutChildBranch`; the tie-break's two orders | B `a_branch_is_split_on_the_first_cell_with_the_fewest_candidates` | `transition-edge.Branch.working.split`, `rule-success.LayOutChildBranch`, `projection.Branch.unplaced_cells`, `projection.Branch.fewest_candidate_cells`, `derived.BranchCell.has_fewest_candidates` |
| Splitting: `guesses` rises by one for each split | O `a_split_counts_one_guess_whatever_its_children` | none |
| Splitting: a published puzzle that needs a guess | O `a_published_puzzle_that_needs_a_guess_is_solved` | none |
| The pinned tie-break | O `the_guess_counts_follow_from_the_tie_break` | none |
| `ConcludeMany`: `many` at the second solution, both exposed and different | O `the_second_solution_concludes_many_with_both_exposed` | `rule-success.ConcludeMany`, `when-set.ConcludeMany.Search.verdict`, `projection.Search.solved_branches` |
| `ConcludeExhausted`: `one` with its one solution | O `one_solution_concludes_one` | `rule-success.ConcludeExhausted`, `when-set.ConcludeExhausted.Search.verdict`, `derived.Search.is_exhausted` |
| `ConcludeExhausted`: `none` with no solution | O `no_solution_concludes_none` | none |
| `NeverCountsPastTwo`; `AlwaysConcludes` on the empty grid; `has_enough_solutions` | O `the_empty_grid_concludes_many_and_stops_at_two` | `derived.Search.has_enough_solutions` |
| `config.solutions_sought` is 2 | S `two_solutions_are_sought` | `config-default.solutions_sought` |
| `config.box_side` and `config.side`: `sudoku`'s two figures, which the solver reads and does not restate | `sudoku.rs`: `box_side_is_three_and_side_is_its_square` (T25) | `config-default.box_side`, `config-default.side` |
| `AlwaysConcludes` | O `every_search_concludes` | `transition-edge.Search.searching.concluded` |
| `VerdictIsTrue` | O `the_verdict_agrees_with_the_oracle`, with `the_oracle_counts_the_fixtures` holding the oracle itself | none |
| `SameGivensSameResult` | O `the_same_givens_get_the_same_result_in_any_order` | none |
| `SolvedBranchesAreSolutions`, `SolutionsAreDistinct`, `NoMoreSolutionsThanSought`, `VerdictMatchesSolutions` | O `a_result_keeps_its_invariants`, each asserted under its own name | `invariant.SolvedBranchesAreSolutions`, `invariant.SolutionsAreDistinct`, `invariant.NoMoreSolutionsThanSought`, `invariant.VerdictMatchesSolutions` |
| `PlacedCellsKeepOnlyTheirDigit`, `CandidatesAreDigits`, `GuessesAreOnFewestCandidates` | B `the_inside_of_a_search_keeps_its_invariants`, each asserted under its own name on every branch of a search | `invariant.PlacedCellsKeepOnlyTheirDigit`, `invariant.CandidatesAreDigits`, `invariant.GuessesAreOnFewestCandidates` |
| `OnlySplitBranchesHaveChildren` | B `the_inside_of_a_search_keeps_its_invariants` (a solved branch has no children) and `singles_alone_settle_the_fixture_into_its_solution`; `Decided::Split` is the one value that carries children | `invariant.OnlySplitBranchesHaveChildren` |
| `SearchResult`: the verdict, the guesses and each solution's digit at every position | O `the_result_exposes_the_verdict_the_guesses_and_every_digit` | `surface-exposure.SearchResult`, `entity-fields.Search` |
| `solve`: the proof for the fixture | O `solve_gives_the_proof_for_the_fixture` | none |
| `solve`: the three refusals | O `solve_refuses_givens_with_no_solution`, `solve_refuses_givens_with_several_solutions`, `solve_refuses_givens_that_leave_nothing_to_play` | none |
| `solve`: a proof exactly when the verdict is `one`, eighty-one givens apart | O `solve_gives_a_proof_exactly_when_the_verdict_is_one` | none |
| `solve`: the conversion from the constructor's error | S `the_refusal_converts_from_the_proofs_error` | none |
| Each refusal's `Display` text is stable | S `refusal_texts_are_stable`; O `each_refusal_reads_as_its_text` | none |
| `OnlyWellPosedPuzzlesArePosed`, from outside | the `compile_fail` example on `sudoku::WellPosed`, which names `WellPosed::vouch` and nothing else | none |
| `PuzzleSolving` end to end, from outside the crate | `tests/sudoku.rs`: `a_puzzle_set_from_outside_is_played_and_solved`, `the_moves_are_offered_and_refused_from_outside`, `the_surface_can_be_read_from_outside`, `a_puzzle_set_from_outside_does_not_print_its_solution` | none |
| Invariant 3 for the new public types | `tests/api_bounds.rs`: `public_types_are_send_sync_and_static`, `public_types_are_clone_and_debug` | none |
| `search.status`, and its transitions | struck: a result has no status value. `search` returns only a concluded search, and the sentence at `solver.allium` lines 638 to 641 says being handed it says concluded. `every_search_concludes` stands in; `verdict` is a method and so always present | `transition-rejected.Search.status`, `transition-terminal.Search.status`, `when-presence.Search.verdict`, `rule-failure.ConcludeMany.1`, `rule-failure.ConcludeExhausted.1` |
| `Guess`, and a branch's `parent`, `guess` and `depth` | struck: the module excludes how the search is stored. A branch is its cells; a child is its parent with one cell placed, and nothing records the guess, the parent or the depth. `a_branch_is_split_on_the_first_cell_with_the_fewest_candidates` reads each child's guess from where it parts from its parent | `value-equality.Guess`, `entity-fields.Guess`, `entity-fields.Branch`, `entity-optional.Branch.parent`, `entity-optional.Branch.guess`, `rule-failure.LayOutRootBranch.1`, `rule-failure.LayOutChildBranch.1`, `invariant.ChildrenSitBelowTheirParent` |
| `Branch.status` as a value; rules that require a working branch | struck: a branch has no status field. Waiting is being on the stack, working is being inside `Branch::decide`, and the three ends are the three values of `Decided`, tested above. Nothing can propagate a waiting or an ended branch, because `decide` takes the branch by value | `transition-rejected.Branch.status`, `transition-terminal.Branch.status`, `derived.BranchCell.is_live`, `derived.Branch.is_laid_out`, `rule-failure.EliminateFromPeers.1`, `rule-failure.PlaceNakedSingle.1`, `rule-failure.PlaceHiddenSingle.1`, `rule-failure.DecideBranch.1` |
| Which branch works next: `is_ready`, `is_next`, `is_due`, `is_idle`, `AttendWaitingBranch`, and the projections over branches | struck: the waiting branches are a stack and the search takes its top. `the_guess_counts_follow_from_the_tie_break` stands in: the counts are those of a depth-first search that takes the lowest digit first, and the break "the highest digit is taken first" fails the tests. The empty grid concluding in 47 guesses is what breadth first would not do | `entity-relationship.Search.branches`, `projection.Search.working_branches`, `projection.Search.waiting_branches`, `projection.Search.ready_branches`, `derived.Search.is_idle`, `derived.Branch.is_ready`, `derived.Branch.is_next`, `derived.Branch.is_due`, `transition-edge.Branch.waiting.working`, `rule-success.AttendWaitingBranch`, `rule-failure.AttendWaitingBranch.1`, `rule-failure.AttendWaitingBranch.2`, `invariant.OneBranchWorksAtATime`, `invariant.TheDeepestBranchWorks` |
| Abandoning what still waits: `waiting -> abandoned`, `ConcludedLeavesNothingOpen` | struck: the stack is dropped when `search` returns, so nothing is left open and nothing is marked. `the_empty_grid_concludes_many_and_stops_at_two` stands in: no branch is taken up after the second solution, or the result would hold a third and the guess count would not be 47 | `transition-edge.Branch.waiting.abandoned`, `invariant.ConcludedLeavesNothingOpen` |
| A surface's actor | struck: neither surface declares one | `surface-actor.Solving`, `surface-actor.SearchResult` |

101 obligations: 62 mapped to a test, 39 struck with a reason.

`search.status` is exposed by the return. The sentence T23 added is in
`docs/specs/solver.allium` above `surface SearchResult`, at lines 638 to 641: "A caller
that is handed a search only once it has concluded reads the status from that alone:
being handed it says concluded." `SearchResult` has no status field and no accessor, and
its documentation quotes the sentence.

### Names later tickets need

**The solver** (`pawdoku::solver`), every item with a doc example that runs:

| Item | Signature |
| --- | --- |
| `search` | `#[must_use] pub fn search(givens: impl IntoIterator<Item = Given>) -> SearchResult` |
| `solve` | `pub fn solve(givens: impl IntoIterator<Item = Given>) -> Result<WellPosed, SolveError>` |
| `SearchResult` | `Debug, Clone, PartialEq, Eq`; `#[non_exhaustive]`, fields private; no serde; no status |
| `SearchResult::verdict` | `pub const fn verdict(&self) -> Verdict` |
| `SearchResult::guesses` | `pub const fn guesses(&self) -> u32` |
| `SearchResult::solutions` | `pub fn solutions(&self) -> &[Grid<u8>]`: none, one or two, in the order found |
| `Verdict` | `Debug, Clone, Copy, PartialEq, Eq`; `#[non_exhaustive]`; `NoSolution`, `OneSolution`, `ManySolutions` |
| `SolveError` | `Debug, Clone, PartialEq, Eq, thiserror::Error`; `#[non_exhaustive]`; `NoSolution`, `ManySolutions`, `NotPosed(WellPosedError)`, with `From<WellPosedError>` |

**The givens type.** Both entries take `impl IntoIterator<Item = Given>`: a
`BTreeSet<Given>`, a `Vec<Given>`, an array or any iterator of givens by value. A
borrowed collection is handed over as `givens.iter().copied()`. Both collect into a
`BTreeSet<Given>`, so the order does not matter and a given handed over twice is one
given. No new type was added for givens. The maintainer chose this shape on 2026-10-01.

**The three refusals of `solve`**, with their `Display` text:

| Refusal | When | Text |
| --- | --- | --- |
| `SolveError::NoSolution` | the verdict is `none`, malformed givens included | `the givens have no solution` |
| `SolveError::ManySolutions` | the verdict is `many` | `the givens have more than one solution` |
| `SolveError::NotPosed(WellPosedError::NothingLeftToPlay { givens: 81 })` | the verdict is `one` and every cell is given | the constructor's own: `81 givens leave nothing to play: a puzzle has fewer givens than its 81 cells` |

`NotPosed` is transparent: its text and its source are the wrapped error's. The other
five `WellPosedError` variants would arrive in it too, by the same conversion, and no
givens reach them. The maintainer chose the wrapped shape on 2026-10-01.

**The tie-break**, confirmed by the maintainer on 2026-10-01: among the empty cells with the fewest
candidates, the first in grid order (topmost row, then leftmost column); among the
children of a split, the lowest digit first. It is the rule the ticket suggested, and it
is stated in the documentation of `pawdoku::solver` under "The tie-break".

**`sudoku`, public now.** The paths T25 fixed exist: `sudoku::WellPosed` and
`sudoku::Puzzle`. `proof` and `puzzle` are private modules and `src/sudoku.rs` re-exports
`WellPosed`, `WellPosedError`, `Cell`, `MoveError`, `Puzzle` and `Status`. Signatures
are as T25's notes list them, with `pub` for `pub(crate)` and `#[must_use]` on the
getters, on `Puzzle::set`, `cell`, `is_full` and `is_consistent`.

| Item | Change |
| --- | --- |
| `sudoku::Grid<T>` | `pub type Grid<T> = [[T; 9]; 9]`, public, as the maintainer chose: rows top to bottom, so row `r`, column `c` is `grid[r - 1][c - 1]` |
| `WellPosed`, `WellPosed::givens`, `WellPosed::solution` | public; `solution` returns `&Grid<u8>` |
| `WellPosed::vouch` | stays `pub(crate)`; `solver::solve` is its one caller outside test code |
| `WellPosed::into_parts` | stays `pub(super)` |
| `WellPosedError` | public |
| `Puzzle` and `set`, `givens`, `status`, `cell`, `cells`, `is_full`, `is_consistent`, `place`, `erase` | public; `place` and `erase` gained an `# Errors` section that states the refusal order |
| `Puzzle::is_solution_digit` | stays `pub(crate)` with its `dead_code` expectation, which names T27 |
| `Status`, `Cell` and its seven getters, `MoveError` | public |
| `sudoku::LINE`, `sudoku::in_range` | `pub(crate)` now, because the solver uses them; `at`, `at_mut`, `grid_positions` and `Position`'s `band`, `stack` and `is_peer_of` stay private to `sudoku` |

For T27: a board opens from givens by calling `solver::solve(givens)` and
`Puzzle::set(proof)`, and maps `SolveError` into its own refusal. The solution a test
needs is `proof.solution()`, read before the proof is given to `Puzzle::set`.

**Every public item beside the doctest that covers it.** `just test` runs them as
`crates/pawdoku/src/<file> - <path> (line n)`.

| Public item | Doctest |
| --- | --- |
| `pawdoku` (crate overview) | `src/lib.rs - (line 17)`: solve, then set |
| `solver` (module) | `solver.rs - solver` |
| `solver::search` | `solver.rs - solver::search` |
| `solver::solve` | `solver.rs - solver::solve` |
| `solver::SolveError` | `solver.rs - solver::SolveError` |
| `solver::Verdict` | `solver/result.rs - solver::result::Verdict` |
| `solver::SearchResult` | `solver/result.rs - solver::result::SearchResult` |
| `SearchResult::verdict`, `guesses`, `solutions` | `solver/result.rs - solver::result::SearchResult::verdict`, `::guesses`, `::solutions` |
| `sudoku` (module) | `sudoku.rs - sudoku` |
| `sudoku::Grid` | `sudoku.rs - sudoku::Grid` |
| `sudoku::WellPosed` | `sudoku/proof.rs - sudoku::proof::WellPosed`, and the `compile_fail` example beside it |
| `WellPosed::givens`, `WellPosed::solution` | `sudoku/proof.rs - sudoku::proof::WellPosed::givens`, `::solution` |
| `sudoku::WellPosedError` | `sudoku/proof.rs - sudoku::proof::WellPosedError` |
| `sudoku::Status` | `sudoku/puzzle.rs - sudoku::puzzle::Status` |
| `sudoku::Cell` | `sudoku/puzzle.rs - sudoku::puzzle::Cell` |
| `Cell::row`, `column`, `band`, `stack`, `digit`, `is_given`, `is_conflicting` | `sudoku/puzzle.rs - sudoku::puzzle::Cell::row` and the six beside it |
| `sudoku::MoveError` | `sudoku/puzzle.rs - sudoku::puzzle::MoveError` |
| `sudoku::Puzzle` | `sudoku/puzzle.rs - sudoku::puzzle::Puzzle` |
| `Puzzle::set`, `givens`, `status`, `cell`, `cells`, `is_full`, `is_consistent`, `place`, `erase` | `sudoku/puzzle.rs - sudoku::puzzle::Puzzle::set` and the eight beside it |

Each of the `sudoku` examples makes its value through `solve`. `WellPosedError`'s makes
its value by handing `solve` eighty-one givens. The variants of the errors and the
variants of `Verdict` and `Status` are shown in their type's example and have none of
their own.

### Snapshots taken

All six are in `crates/pawdoku/tests/snapshots/`, taken by tests in
`crates/pawdoku/tests/snapshots.rs` through the public API after the list was empty, and
each file was read before it was committed. The render helpers draw from
`SearchResult::verdict`, `guesses` and `solutions`, and from `WellPosed::givens` and
`solution`; nothing was added to the library for them. The guess counts and the order of
the two solutions follow from the tie-break.

| Test | File | Shows |
| --- | --- | --- |
| `snapshot_the_search_of_the_fixture` | `snapshots__tests__snapshot_the_search_of_the_fixture.snap` | The fixture's givens as a grid, then verdict `OneSolution`, 0 guesses and the one solution |
| `snapshot_the_search_of_a_puzzle_that_needs_a_guess` | `snapshots__tests__snapshot_the_search_of_a_puzzle_that_needs_a_guess.snap` | Norvig's puzzle, verdict `OneSolution`, 50 guesses and its solution |
| `snapshot_the_search_of_givens_with_two_solutions` | `snapshots__tests__snapshot_the_search_of_givens_with_two_solutions.snap` | The fixture without one given, verdict `ManySolutions`, 1 guess and both grids |
| `snapshot_the_search_of_malformed_givens` | `snapshots__tests__snapshot_the_search_of_malformed_givens.snap` | Three givens listed, one off the grid and one with the digit 12, then verdict `NoSolution`, 0 guesses and no solution |
| `snapshot_the_proof_of_the_fixture` | `snapshots__tests__snapshot_the_proof_of_the_fixture.snap` | The proof's givens and its solution, each as a grid. This file holds a solution on purpose: the proof exposes it |
| `snapshot_the_three_refusals` | `snapshots__tests__snapshot_the_three_refusals.snap` | The `Display` text of the three refusals, one line each |

None of T25's four snapshots under `src/sudoku/snapshots/` changed.

### What was verified, and how

Done on 2026-10-01 in the supplied Supacode worktree, on `69025ae`, the merge of T25.
The worktree was clean and `.pixi/` and `.tools/bin` were present, so `just initialize`
was not run. `just check` was green before any edit. Commands used rustup's cargo
through `PATH="$HOME/.cargo/bin:$PATH"`.

`just plan-spec solver` printed 101 obligations and an empty `diagnostics` array; the
table above accounts for each.

Closing lines of each recipe, from the last run:

| Recipe | Closing output |
| --- | --- |
| `just fmt-check` | `cargo fmt --all --check` (no diagnostic) |
| `just clippy` | ``Finished `dev` profile [unoptimized + debuginfo] target(s)``, no warning |
| `just metrics` | `::notice::Quality score: 100.0% (297 functions analyzed)`, no finding, no recursion among them, and the probe satisfied |
| `just test` | `141 tests run: 141 passed, 0 skipped`; `test result: ok. 56 passed; 0 failed` and `ok. 1 passed` (the `compile_fail` example) for the doctests |
| `just snapshots-check` | `info: no unreferenced snapshots found`; `info: no snapshots to review` |
| `just wasm-check` | both feature sets on `wasm32-unknown-unknown` and on `wasm32v1-none`, each `Finished` |
| `just features` | `--no-default-features` and `--features serde`, each `Finished` |
| `just coverage` | `TOTAL` lines `99.77%` (1737 lines, 4 missed); `Finished report saved to target/llvm-cov/lcov.info` |
| `just doc` | `Generated .../target/doc/pawdoku/index.html` |
| `just check-docs` | `Validated 41 pages and 42 canonical topics.` |
| `just check` | `The worktree matches the check baseline.`; `All checks passed and the worktree is unchanged.` |

`just test` takes about 1.2 seconds of wall time once built, 0.3 of them in nextest; it
took about the same before this ticket. The slowest solver test is
`the_inside_of_a_search_keeps_its_invariants`, at 0.4 seconds under coverage and less
without. No test costs a second, so nothing is reported under "Speed".

Coverage of the module alone, from the same report:

| File | Lines | Missed | Cover |
| --- | --- | --- | --- |
| `solver.rs` | 52 | 0 | 100.00% |
| `solver/branch.rs` | 502 | 0 | 100.00% |
| `solver/candidates.rs` | 65 | 0 | 100.00% |
| `solver/result.rs` | 12 | 0 | 100.00% |
| `solver/search.rs` | 17 | 0 | 100.00% |

The two tables in `branch.rs` are built when the crate is compiled, so nothing ran their
builders under instrumentation until `the_units_are_nine_rows_nine_columns_and_nine_boxes`
called them again and compared. The four missed lines are three in `random.rs`, which
this ticket did not touch, and the one T25 reported in `sudoku/puzzle.rs`.

**The guess counts were checked against a second implementation.** Before the Rust was
written, a model of `solver.allium` was written in Python in `ai_tmp/`, from the
specification's text, with sets for candidates and the rules fired one at a time. It
gave 0 guesses for the fixture, 50 for Norvig's puzzle, 1 for the two-solution givens
and 47 for the empty grid, and the Rust gave the same four when it first ran. Then 160
further sets of givens, thinned from the two solutions at six densities and a quarter of
them spoiled, were put to both: the verdict, the guess count and the solutions agreed on
all 160 (118 `many` with 1,723 guesses between them, 27 `none`, 15 `one`). The model, the
cases and the test that read them are scratch and are not in the commit. Norvig's
solution and the uniqueness of each fixture were also counted by a third, plain counter
in the same scratch file.

**The published puzzle** was read from <https://norvig.com/sudoku.html> on 2026-10-01,
with the maintainer's leave for the one fetch: the essay's `grid2` and the solution it
prints. `tests/solver.rs` cites it. The two-solution givens and the hidden-single puzzle
were made from the fixture and say so beside their constants.

**The tests that were green on arrival were shown to bite.** Fifteen deliberate breaks,
each made, run through `just test` and reverted:

| Break | Tests that failed |
| --- | --- |
| Hidden singles are never placed | `hidden_singles_solve_a_puzzle_with_no_naked_single`, and two snapshots |
| Naked singles are never placed | `the_guess_counts_follow_from_the_tie_break`, and a snapshot |
| A unit with no place for a digit is not a contradiction | `each_kind_of_contradiction_is_read_from_the_cells` |
| A cell that is the only place for two digits is not a contradiction | `each_kind_of_contradiction_is_read_from_the_cells` |
| A cell with no candidate is not a contradiction | `each_kind_of_contradiction_is_read_from_the_cells` |
| A hidden single is placed in an overdemanded cell | `a_cell_that_is_the_only_place_for_two_digits_is_not_placed` |
| The highest digit is taken first | `the_second_solution_concludes_many_with_both_exposed`, `the_result_exposes_the_verdict_the_guesses_and_every_digit`, and a snapshot |
| The split is on the last cell with the fewest candidates | `a_branch_is_split_on_the_first_cell_with_the_fewest_candidates` |
| The split is on the first empty cell, fewest or not | `a_branch_is_split_on_the_first_cell_with_the_fewest_candidates` |
| Three solutions are sought | `two_solutions_are_sought` |
| A digit out of range is accepted | `no_root_is_opened_for_givens_that_are_not_well_formed` |
| Two digits to one position are accepted | `no_root_is_opened_for_givens_that_are_not_well_formed` |
| A guess is counted for each child | `a_split_counts_one_guess_whatever_its_children`, and two snapshots |
| A placed digit stays among its peers' candidates | the run did not finish in five minutes and was stopped |
| `WellPosed::vouch` is public | the `compile_fail` example, which then compiles |

Those runs stop at the first failures, so each row is what failed before the run
stopped, not everything that would have. Snapshot tests appear because they run in the
same suite and a changed result changes a picture; no clause rests on them.

Acceptance criteria read against the code:

- **The one caller.** Outside test code `WellPosed::vouch` is called once, at
  `crates/pawdoku/src/solver.rs` in `solve`. The other calls are in `#[cfg(test)]` code
  under `src/sudoku/`, as T25 left them.
- **What `src/solver` names.** `crate::sudoku` and nothing else of the engine: no
  `crate::random`, no trait, no generic parameter for a solver, no budget argument. The
  one generic is the givens' `impl IntoIterator`. `just metrics` holds the boundary rule
  `solver_imports_sudoku`.
- **No recursion**, in code or tests: the search and the oracle each keep a list, and
  `just metrics` reports no finding.
- **Suppressions.** No `#[allow]`, no `qual:allow` and no `#[expect]` added, on an item
  or on a crate. The twenty expectations naming T26 are gone, and the one on
  `Puzzle::is_solution_digit`, naming T27, remains.
- **No panic in the library.** Outside test code `src/solver` has no `unwrap`, `expect`,
  `panic!` or run-time indexing. The two tables are indexed while they are built, at
  compile time; a running search reads them through `get` and iterators.

### Deviations, and why

- **The loops were not one line each.** The first test was red on its assertion, and
  then the whole search went in at once, because no part of propagation turns the
  fixture green alone. Most lines were therefore green on arrival. The deliberate breaks
  above are how each was shown to prove something, as T25 did.
- **The two new integration test files keep their tests in a `#[cfg(test)] mod tests`**,
  unlike the three older ones, which open with a crate-level
  `#![expect(clippy::tests_outside_test_module, ...)]`. The first commit followed the
  older files; review round 1 held it to the criterion "no `#[expect]` added", and the
  module is what meets it. The snapshot files are therefore named
  `snapshots__tests__snapshot_...`.
- **`README.md` and `crates/pawdoku/README.md` are edited**, one sentence each, though
  the ticket does not list them: each said the crate held the randomness boundary and
  the value types alone. T25's review asked for the same fix to the same sentences and
  the maintainer accepted it. Revert them if the lane should not have.
- **`sudoku::LINE` and `sudoku::in_range` are `pub(crate)`.** T25's notes left widening a
  helper to this ticket. The solver uses these two and no other.
- **The verdict's variants are `NoSolution`, `OneSolution` and `ManySolutions`**, not
  the specification's bare `none`, `one` and `many`. The maintainer chose them on
  2026-10-01, after the first commit: a variant named `None` reads as Rust's
  `Option::None`. The refusal's second variant was renamed from `SeveralSolutions` to
  `ManySolutions` to match. Four snapshots changed with the names and with nothing else:
  their `verdict:` line now reads `OneSolution`, `ManySolutions` or `NoSolution`.
- **`guesses` is a `u32`**, not a `usize`, so that its width is the same on every target
  and a binding sees one type.
- **No serde on the new types.** Nothing asks for one yet, as T25 decided for `WellPosed`.
- **The second two-solution set.** The ticket asks for one set with exactly two
  solutions. There are two: the fixture without one given, which the snapshot shows, and
  the fixture's solution with four cells emptied, which gives a split whose two children
  are both solved at once.
- **The branch** is `ticket/T26-solver`, as Supacode made it, not the `branch:` field's
  `ticket/t26-solver`.
- **Commands outside the recipes.** A Python model and its scripts in `ai_tmp/`; one
  temporary file under `tests/` that read the model's cases, run through `just test` and
  deleted; and one `cargo llvm-cov report`, which printed nothing and was replaced by
  reading `just coverage`. None is needed to reproduce anything here.
- **Choices the ticket left open**, for review: the result's solutions are a slice of
  grids, in the order found; the refusal is named `SolveError` and its third variant
  `NotPosed`; the parts are `candidates.rs`, `branch.rs`, `search.rs` and `result.rs`.

### Review round 1

An adversarial review by Codex, run locally on 2026-10-01 over `69025ae..45cf0d8`, with
the rename in `8356c7c` in its sight. It was asked to try to make the batch propagation
passes and the place of the contradiction checks change a verdict, a solution or a guess
count. It reported that a one-rule-at-a-time model and a batch model of its own agreed
on 1,000 inputs and that it found no propagation or guess-count defect. Its sandbox was
read-only, so it ran no recipe. Two findings, both accepted:

- **The oracle could be handed givens it cannot count in any reasonable time.** The
  property `the_verdict_agrees_with_the_oracle` drew from `arbitrary()`, which can make
  sparse, well-formed givens with no solution. Codex's example is five givens that leave
  row 9 no place for a 1: the plain counter fills eight rows every way before it finds
  out. The run would have hung, rarely. The oracle's third strategy is now `malformed()`:
  arbitrary givens with one added that is certainly off the grid or out of range, which
  the oracle counts as none without a search. Its thinned sets are denser too, from six
  cells in ten kept. `docs/reference/testing.md` said such inputs were not generated,
  which was untrue of that arm, and now says what is enforced. `just test` with
  `PROPTEST_CASES=30000` ran the property in 0.4 seconds. The properties that call the
  solver alone still take arbitrary givens.
- **The two crate-level `#![expect]` lines** broke the criterion as written. Both files
  now keep their tests in a `#[cfg(test)] mod tests` and carry no expectation. The six
  snapshot files were renamed by `just snapshots-accept` and their pictures did not
  change. The oracle's loop lost a level of nesting to fit under the module.

Also in this round, unasked: `every_search_concludes` asserted that the verdict was one
of its three variants, which no result can fail. It now asserts only what returning
shows, with a comment that says so.

The three older integration test files still open with the expectation. Whether they
follow these two is the maintainer's: it is outside this ticket's files.

### Review round 2

Pull request 27, reviewed at `6017049` by Codex and by Copilot on 2026-10-01. Three
comments, two findings.

- **`snapshots.rs` does not hold every public-API snapshot: accepted** (Copilot, twice).
  `docs/reference/testing.md` and `docs/project/repository-map.md` said it did, and
  `tests/random.rs` keeps `snapshot_the_randomness_boundary`, which T29 put there. Both
  pages now say the file holds the solver's and the proof's snapshot tests and name the
  one that stays in `random.rs`. The ticket's own step 6 uses the same wording; it is
  left as written, being the instruction and not a description of the repository.
- **Keep the generic `Grid<T>` alias private: declined** (Codex, P1). The finding:
  `Grid<Rc<()>>` is not `Send`, so a public generic alias breaks invariant 3. A type
  alias defines no type. `[[T; 9]; 9]` is an array a caller can write with or without
  the name, and it has every bound its element has. The invariant is about the types
  the crate defines and hands out, and the only grid it hands out is `Grid<u8>`, which
  `tests/api_bounds.rs` holds to the bounds. The maintainer chose the public generic
  alias on 2026-10-01, and T27 may want `Grid<Option<u8>>` for a board's digits. The
  alias's documentation now says it names a plain array and which grid the API hands out.

No re-review was asked, at the maintainer's instruction.

### Handed back

T26 is done here and in the ticket index.

Triggers this ticket meets, each for the maintainer to pick up and none built here:

- **S03's benchmark recipe**, whose trigger is "once the solver exists". `solver::search`
  on Norvig's puzzle and on the empty grid are the two natural first benchmarks.
- **T14**, the mutation job S03 drafted, whose trigger is "once tests exist". The fifteen
  breaks above are a hand-made sample of what it would do.
- **T16**, the command-line crate S04 drafted, whose `solve` subcommand "lands with the
  solver". `solver::solve` and `solver::search` are what it wraps.
- **T20 and T21**, drafted in T19's hand-back notes, each of which waits for the solver
  in Rust.
- **rustqual's pin.** Decision 0013 and `docs/how-to/maintain-dependencies.md` hold the
  pin at 1.8.3 "until the solver has landed under it". It has: `just metrics` reports no
  finding with the solver in the crate, with no suppression and no threshold moved.

Still open, for the maintainer:

- **`docs/tutorials/first-change.md`** is still T30's.
- **No fuzz target** is triggered: S03 adopts cargo-fuzz "when a parser exists", and
  the solver is not one.

## Open points

- **The tie-break.** The ticket suggests row order, then column order, then the lowest
  digit. Any fixed rule satisfies the module. The choice changes guess counts, which a
  consumer may come to record, so the rule chosen is stated in the module's
  documentation and in the hand-back notes for the maintainer to confirm.
  **Confirmed on 2026-10-01:** the first cell in grid order, the lowest digit first.
- **What `search` exposes of the search's inside.** The surface exposes the verdict, the
  guesses and the solutions, and the ticket builds exactly that. Branches, depth and the
  guess taken at each split are not exposed. `generation.allium` may later ask for more;
  that is its ticket's.
- **The third refusal's place.** "Nothing left to play" is `SetPuzzle`'s guard, decided
  to live in the proof's constructor. `solve` reports it as a refusal of its own beside
  the two verdicts. If that reads oddly from a binding, the maintainer may want it named
  differently; the behaviour is fixed.
- **Speed.** No figure is promised and none is measured here. If `just test` slows by
  more than a few seconds, say so in the hand-back notes with the test that costs it.
