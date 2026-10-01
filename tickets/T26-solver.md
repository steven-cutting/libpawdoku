---
id: T26
title: "The solver in Rust: the search, its result and the proof"
status: open
depends_on: [T25]
parallel_with: []
branch: ticket/t26-solver
estimated_size: L
---

# T26: The solver in Rust: the search, its result and the proof

## Context

T25 built `pawdoku::sudoku`: the value types, the proof value `WellPosed` and `Puzzle`.
Nothing can make a proof yet, so nothing outside the crate can make a puzzle. This ticket
builds `pawdoku::solver` from `docs/specs/solver.allium`: the search that gives a set of
givens its verdict of none, one or many. It is the one caller of the proof's constructor.

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
- **The search is synchronous.** A caller sees a search only once it has concluded, so
  `SearchResult`'s `status` is always `concluded` in Rust and needs no field. Record that
  reading in the hand-back notes.
- **The proof constructor's other errors cannot come from a correct search.** A solution
  that is not full, or a given that does not match it, would be a bug here. `solve` must
  still not panic. Give the refusal one conversion from the constructor's error and test
  that conversion directly; do not write a branch per impossible case that no input can
  reach, because unreachable lines count against the coverage floor.
- **T25 left two things for this ticket.** The constructor carries a `dead_code`
  expectation, under `cfg_attr(not(test), ...)`, naming T26; now that `solve` calls it,
  the expectation is unfulfilled and `just clippy` fails until the line is removed. And
  the doc
  examples on `WellPosed` and `Puzzle` compile without running; T25's hand-back notes
  name them, and they can now run through `solve`.

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
`docs/reference/testing.md`; `.agents/skills/rust-change/SKILL.md` and
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
`solver::solve`. Every other name is this ticket's to choose and to report.

## Non-goals

- No deduction past singles: no locked candidates, subsets, fish or chains. Those are
  `technique.allium`'s.
- No solving from a puzzle in play. The search reads givens and nothing a player placed.
- No trait for the solver, no generic over it, no budget, no randomness.
- No benchmark, no fuzz target and no mutation job. Each is named under Handed back as a
  trigger this ticket meets (step 7).
- No change to `solver.allium` or any module. If no clause says what the code needs,
  stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No change to `sudoku`'s behaviour. The edits to `src/sudoku.rs` are the removed
  expectation and the doc examples.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/solver.rs` | New: the module's surface |
| `crates/pawdoku/src/solver/` | New: one file per part, as the limits ask |
| `crates/pawdoku/src/sudoku.rs` | The constructor's `dead_code` expectation removed, the whole `cfg_attr` line; the doc examples T25 named rewritten to run through `solve` |
| `crates/pawdoku/src/lib.rs` | `pub mod solver;` and the crate overview |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type |
| `crates/pawdoku/tests/solver.rs` | New: both entries through the public API |
| `crates/pawdoku/tests/sudoku.rs` | `Puzzle` through the public API, now that one can be made: the `PuzzleSolving` surface end to end |
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
   givens and the one solution when the verdict is `one`. Remove the
   `dead_code` expectation on the constructor. Rewrite the doc examples T25 named so that
   each makes its value through `solve` and asserts. Add to `tests/sudoku.rs` what could
   not be written before: a puzzle set from outside the crate, played and solved through
   its public API.

6. **The pages.** `docs/explanation/architecture.md`: the two entries and which a caller
   wants; that the solver runs once per puzzle. `docs/reference/testing.md`: the suite
   rows, the oracle, and that the pinned guess counts follow from the tie-break.
   `docs/project/repository-map.md` and `CHANGELOG.md` follow.

7. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`, `just wasm-check`,
   `just features`, `just coverage`, `just doc` and `just check-docs`, then `just check`.
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
  count, and each solution's digit at every position.
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
  line; the constructor's expectation is gone.
- The doc examples on `WellPosed` and `Puzzle` run and assert.
- Line coverage of `src/solver` alone is at or above 90 per cent.
- `just check` is green.

## Verification

```sh
just plan-spec solver
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
`metrics` reporting no finding, no recursion among them; both wasm targets under both
feature sets; the coverage total at or above the floor; `All checks passed and the
worktree is unchanged.`

## Hand-back notes

### The test list

### Names later tickets need

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **The tie-break.** The ticket suggests row order, then column order, then the lowest
  digit. Any fixed rule satisfies the module. The choice changes guess counts, which a
  consumer may come to record, so the rule chosen is stated in the module's
  documentation and in the hand-back notes for the maintainer to confirm.
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
