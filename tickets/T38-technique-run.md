---
id: T38
title: "The run without guessing in Rust: a kept grid, the four techniques and solved or stalled"
status: open
depends_on: [T37]
parallel_with: []
branch: ticket/t38-technique-run
estimated_size: L
---

# T38: The run without guessing in Rust: a kept grid, the four techniques and solved or stalled

## Context

T37 wrote `docs/specs/layout.allium`: how a setter that is a program fills a layout of
positions with givens that a repertoire of techniques suffices to solve, by trials over
drawn grids, each trial checked by a run that takes only licensed deductions of the
repertoire and never guesses. This ticket builds the run and its asking, `Check`, as the
first half of `pawdoku::layout`; T39 builds the trials and `Fill`. The run is the test
every trial depends on, so it comes first and stands on its own: a caller can already ask
whether a set of givens is solvable by naked singles alone, or by the paper's three
strategies, which is what the comparison between the basic generator and the filled
layouts will ask of every puzzle.

**This ticket's words may be a day old.** It says layout, repertoire, run, step, solved,
stalled and check, the words T37 proposed. T37's hand-back notes, under "Names T38 and
T39 need" and "Open points settled", hold the words and the answers the maintainer chose.
Where they differ from this ticket, they win, and this ticket is read with them. If the
maintainer kept the run private, `check` is `pub(crate)` and the integration tests below
move into the module; everything else stands.

The run, as T37 was asked to specify it. Read `layout.allium` for what it says now.

1. **The grid.** Opened born kept on the givens: a placed cell keeps its digit as its one
   candidate, and every other cell allows every digit no placed peer holds
   (`technique.allium`'s `LayOutGrid`). No uniqueness is promised.
2. **A step.** While the grid is unfilled and some licensed deduction has a technique in
   the repertoire, one is taken. A placement puts the digit in its cell and upkeep
   strikes it from the cell's peers, so the grid is kept again; a strike takes its
   candidates out. Which licensed deduction is taken is the implementation's, the same
   each time; the end does not depend on it.
3. **The four techniques**, licensed exactly as `technique.allium`'s predicates say, on a
   kept grid: `naked_single`, an open cell with one candidate, placed; `hidden_single`,
   a unit in which a digit still to be placed has one place, placed there; `pointing`,
   a box whose places for such a digit lie in one row or one column that allows the
   digit outside the box, struck from the rest of that line; `claiming`, a row or column
   whose places lie in one box that allows the digit off the line, struck from the rest
   of that box. Two or more places are required for pointing and claiming; one place is
   the hidden single.
4. **The end.** Solved when the grid is filled; stalled when it is unfilled and nothing of
   the repertoire is licensed. Nothing is guessed, nothing is put back, and a run always
   ends: each step places a digit or strikes a candidate that stood.
5. **What comes back.** Solved or stalled, the count of steps, and the digits placed.

**Decided by the maintainer on 2026-10-05:** the run is specified against
`technique.allium`'s definitions and built now, without waiting for `technique` or
`reach` in Rust. The shape below is the ticket writer's recommendation; the maintainer
asks for one where a choice is about Rust and not about the product. Report what was
built under "Names T39 needs".

- **One module, `pawdoku::layout`**, as `src/layout.rs` and `src/layout/<part>.rs`.
  T39 adds the filling to it. Its surface after this ticket:
  `layout::Technique`, `layout::Repertoire`, `layout::check`, `layout::Run`,
  `layout::Outcome`, `layout::CheckError` and `layout::LAYOUT_VERSION`.
- **`Technique` is a public enum** of four, `#[non_exhaustive]`: `NakedSingle`,
  `HiddenSingle`, `Pointing`, `Claiming`, in the catalogue's ladder order, with a
  method that gives its rank (3, 4, 5, 6) so that the lowest can be taken first and a
  caller can tell which it is. It is a different type from any later `technique`
  module's catalogue entry and says so in its documentation; when that module lands it
  is the one to keep.
- **`Repertoire` is a public set of those four**, built from any techniques and refusing
  an empty set; with `contains`, an iterator of what it holds, and two constants or
  constructors for the sets the paper runs: the three strategies (all four here) and
  naked singles alone. It serialises under the `serde` feature as `Tier` does, so a
  caller can record what a puzzle was checked against.
- **`check(givens, repertoire)` takes `impl IntoIterator<Item = Given>`**, as
  `solver::search` does, and returns `Result<Run, CheckError>`. It refuses givens that
  are two to a position, in conflict, or 81, and takes no draw and opens no grid when it
  does; reuse `sudoku`'s refusal for malformed givens where one exists with the right
  text, and add a variant only where none does.
- **`Run` is the result**: `outcome()` gives `Outcome::Solved` or `Outcome::Stalled`, a
  public `#[non_exhaustive]` enum; `steps()` the count of steps taken; `grid()` the
  digits placed as a `Grid<u8>`, 0 for an open cell, so a caller can see how far a
  stalled run got and a solved run's grid is the one solution. Nothing else is exposed:
  not the deductions, not the candidates. A run is a plain value, `Clone`, `Debug`,
  `PartialEq`, `Send + Sync + 'static`.
- **`CheckError` is a `thiserror` enum**, `#[non_exhaustive]`, with stable `Display`
  text: the empty repertoire, and the refusals of the givens.
- **`LAYOUT_VERSION`** is a public constant equal to the specification's
  `config.layout_version`, as `generation::GENERATION_VERSION` equals its own. It names
  the run too: which deduction a run takes among several is pinned here, and a change
  to that pin is a new name, though the end does not change.

Facts that shape the work:

- **The run keeps its own candidates.** No `technique` module exists, and the solver's
  `Candidates` (`src/solver/candidates.rs`, a `u16` bit set, `pub(super)`) is the
  solver's own. Write the run's candidate set in the module, in the same shape, unless
  `just metrics` reports the copy as duplication; then widen the solver's to
  `pub(crate)` instead, say so under Deviations, and change nothing else in `solver`.
  Do not reach into `solver`'s branches or search.
- **A licensed deduction is a predicate on the grid as it stands.** `technique.allium`
  lines 284 to 520 give each: `is_naked_single`, `hides_single(d)`, `has_places(d)`,
  `points_along_row(d)`, `points_along_column(d)`, `claims(d)`. Build each as a function
  of the grid and the unit, read from candidates, and test each on a grid worked out by
  hand against the predicate's text, the case that licenses and a case that does not.
  A kept grid is the premise of every one: a digit a unit has placed has no places,
  "whatever stale marks still allow".
- **The order of choice is the module's own.** The specification leaves it to the
  implementation and asks only that it be the same each time and that the end not
  depend on it. Take the lowest-ranked technique that licenses anything; among its
  deductions, the first in row order of the cell placed or of the unit read, rows
  before columns before boxes, lowest digit first. Put the choice in one function so a
  test can drive the loop with another order: the inner loop takes the chooser as
  `removal::remove` takes an order, and `check` hands it this one.
- **The end does not depend on the order, for the repertoires the guarantee covers,
  and a test holds it.** For a repertoire that holds `hidden_single` or no locked
  candidate, the four techniques are monotone (T37's Context says why), so a run that
  takes licensed deductions in any order ends the same way with the same digits
  placed. For a repertoire with a locked candidate and no `hidden_single`, T37 settles
  whether `Repertoire` accepts it at all; if it does, the only promise is that `check`
  is the same each time. A property drives the loop with an order proptest made and
  compares it with `check`'s for the covered repertoires, and asserts no more than
  determinism for the rest.
- **Soundness comes from the catalogue.** Every givens set a trial puts to a run is read
  from a solution grid, and `DeductionsAreSound` then promises every step. A caller of
  `check` may hand over any givens, well-posed or not; the run still ends, still never
  panics, and promises nothing of a grid that is not true. Say so in `check`'s
  documentation.
- **A run always ends, and the bound is small.** At most 81 placements and 648 strikes,
  so at most 729 steps; the loop needs no budget and no clock, and a test asserts the
  bound over every run it makes.
- **No recursion.** `rustqual.toml` sets `allow_recursion = false`, for test code too.
- **The rest of the metrics gate** (`rustqual.toml`, `clippy.toml`): a function at most
  60 lines, cognitive complexity 15, cyclomatic 10, nesting 4, five parameters; a struct
  at most 12 fields and 20 methods, LCOM4 at most 2; a file at most 500 code lines
  before its first `#[cfg(test)]`, and 1000 for test code. Meet them by structure:
  `src/layout.rs` plus `src/layout/<part>.rs` (the repertoire, the candidates and the
  grid, the licences, the run); `mod.rs` is banned. Never by a suppression and never by
  moving a number.
- **The boundary rule exists.** T37 added `layout_imports_sudoku_solver_technique_generation`
  to `rustqual.toml`, covering `src/layout{.rs,/**}`. This ticket's module names
  `crate::sudoku` and `crate::solver` (the proof, in tests and in T39) and nothing else
  of the engine; `crate::random` arrives with T39.
- **Nothing calls the run but tests until T39.** The public items are reachable from
  outside, so nothing is dead. Anything `pub(crate)` that only T39 will use carries
  `cfg_attr(not(test), expect(dead_code))` with its reason, as the house does, and T39
  removes it.
- **Snapshots show and detect change; they prove nothing** (`docs/reference/testing.md`,
  Snapshots). They are taken from fixed inputs after the module's own tests are green,
  by tests named `snapshot_...` in `crates/pawdoku/tests/snapshots.rs`, from a text
  picture rendered in test code. Nothing is added to the library to feed one.
- **The study puzzles are unverified claims on a page.** `docs/explanation/puzzle-design.md`
  carries four grids with the report's claims about each: the beginner one "resolves
  entirely through naked singles", the intermediate one needs singles and locked
  candidates, the advanced one an X-Wing and an XY-Wing, the expert one a contradiction
  chain; the page says none of it has been verified here. S06's notes (on its branch)
  found all four well-posed and the expert one solvable within ranks 1 to 22. This
  ticket runs each under the repertoires below and records the outcome; it asserts
  nothing the page claims, and hands the page what it found. The grids, from the page,
  a dot for an empty cell:

   ```text
   beginner (32 givens)   intermediate (32)      advanced (29)          expert (25)
   ..792..5.              .3.47..5.              8..92.35.              ..94.....
   .2..16.37              ..2.8....              ......8.7              ..8..9...
   6..3..4.9              ..51.3..7              ....84...              7.4.38.9.
   ......29.              ..9...2.4              ..3..8.2.              .6.....7.
   ...6.1...              2.79.83.1              .1.562.3.              ..1.4.8..
   .78......              1.3...5..              .5.4..9..              .7.....1.
   9.2..5..3              3..7.49..              ...81....              .9.75.3.6
   56.87..4.              ....9.6..              5.9......              ...1..5..
   .8..496..              .5..31.4.              .87.59..3              .....69..
   ```

   Read each grid again from the page before using it, and say under Deviations if
   any differs from what is copied here.

The fixture from T25, whose solution serves as the true grid for hand-worked cases:

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

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/layout.allium` in
full and `docs/specs/technique.allium` lines 284 to 520, 715 to 760 and 870 to 882;
`docs/specs/reach.allium` lines 195 to 240 (`OpenGrid`, `KeepMarksTrue`);
`tickets/T37-layout-spec.md`, Context and hand-back notes; the decision record T37
wrote; `tickets/T32-basic-generator.md`, hand-back notes, as the shape of a Rust
ticket's notes; `tickets/T26-solver.md`, hand-back notes ("Names later tickets need");
`crates/pawdoku/src/sudoku.rs`, `crates/pawdoku/src/solver.rs` and
`crates/pawdoku/src/solver/candidates.rs`; `crates/pawdoku/src/generation/removal.rs`,
for a loop that takes its order from outside; `docs/explanation/layering.md` and
`docs/explanation/architecture.md`; `docs/reference/testing.md`, its Conventions and
Snapshots sections; `.agents/skills/rust-change/SKILL.md` and
`.agents/skills/propagate/SKILL.md`; `rustqual.toml` and `clippy.toml`.

## Goal

`pawdoku::layout` exists with the run and its asking, and does what `layout.allium` says
of them. `check` gives a set of givens and a repertoire a run that ended solved or
stalled, with its count of steps and the digits it placed, or a refusal that says why.
A puzzle made by `generation::generate` can be asked, with nothing but this crate,
whether naked singles alone solve it, or the paper's three strategies. Every clause of
the run in `layout.allium` is matched by a named test or a stated reason, listed in the
hand-back notes. `just check` is green with the coverage floor held by this module's own
tests and no suppression beyond the dead-code ones T39 removes.

Names later tickets are written against, fixed here: `layout::Technique`,
`layout::Repertoire`, `layout::check`, `layout::Run`, `layout::Outcome`,
`layout::CheckError` and `layout::LAYOUT_VERSION`. Every other name is this ticket's to
choose and to report.

## Non-goals

- Nothing of the filling: no layout type, no trial, no draw, no `random`. T39.
- No technique beyond the four, and no `technique` module: the catalogue stays the
  specification's until its own ticket.
- No rating, no difficulty and no ordering of puzzles by what solves them.
- No change to the behaviour of `sudoku`, `solver`, `board`, `generation` or `random`;
  the one allowed edit to `solver` is the visibility of its candidate set, and only if
  `just metrics` asks for it.
- No change to `layout.allium` or any module. If no clause says what the code needs,
  stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No benchmark, no fuzz target, no mutation job, no command line and no binding.
- No edit to `docs/explanation/puzzle-design.md`: what the study puzzles showed is
  handed back, not written there.
- No snapshot standing in for a test that asserts.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/layout.rs` | New: the module's surface, `check`, `Run`, `Outcome`, `CheckError`, `LAYOUT_VERSION` |
| `crates/pawdoku/src/layout/` | New: one file per part, as the limits ask |
| `crates/pawdoku/src/lib.rs` | `pub mod layout;` and the crate overview |
| `crates/pawdoku/src/solver/candidates.rs` | Visibility only, and only if `just metrics` asks for it |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type |
| `crates/pawdoku/tests/layout.rs` | New: `check` through the public API |
| `crates/pawdoku/tests/snapshots.rs` | The snapshot tests of step 6 and their render helper |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | A section for `layout`: the run, what it reads and what it promises; the filling is T39's |
| `docs/explanation/layering.md` | The sentence T37 left saying the Rust module is not yet built |
| `docs/project/purpose-and-scope.md` | The generation bullet: the run is built, the filling is specified |
| `docs/reference/testing.md` | The suite table; the run's tests; the order-independence property; the snapshots taken |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `crates/pawdoku/README.md` | What the crate can now do, if the page lists it |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T38's row set `done` |
| `tickets/T38-technique-run.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t38-technique-run` from `main` after T37 has merged.
   Run `just initialize` if `.pixi/` is absent, and `just install-allium` and
   `just install-tools` if `.tools/bin/allium` or `.tools/bin/rustqual` is; each uses
   the network and this ticket authorises it. Run `just check` and confirm it is green
   before any edit; note how long `just test` takes. Read T37's hand-back notes and
   write down, at the top of this ticket's hand-back notes, each word or answer that
   differs from this ticket.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec layout` and add any obligation of the run the
   list lacks; obligations of the filling, the trial and `Fill` are T39's, and are struck
   with that reason. Where an obligation names a structure the implementation does not
   have, strike the line with the reason that the module excludes how a run is stored,
   and name the test of the observable result that stands in for it. Put the list in
   the hand-back notes under "The test list" before the first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. Run `just clippy` and `just metrics` every few loops.
   A good first line is the smallest whole behaviour: a repertoire of the four, refusing
   the empty set. Then a grid born kept on the fixture; then a naked single licensed and
   taken, with upkeep; then the hidden single in each kind of unit; then pointing along
   a row and a column; then claiming; then the end, solved and stalled; then `check` and
   its refusals; then the properties; then the study puzzles.

4. **Each test must be able to fail.** When a line goes green on arrival, break the code
   on purpose, see the test fail, and restore it. List the breaks in the hand-back
   notes, as T32 did. Breaks worth making: pointing licensed with one place; a hidden
   single read from sight rather than candidates (a stale candidate left standing);
   upkeep skipped after a placement; claiming striking inside the line; the run stopping
   one step early; a technique outside the repertoire taken.

5. **Hold the end against the order.** One property drives the inner loop with an order
   proptest made, choosing among the licensed deductions of the repertoire at each
   step, and compares the end and the digits placed with `check`'s run on the same
   givens, for givens from `generation::generate` through the fake and for the fixture
   and the study puzzles, under the repertoires the guarantee covers. For any other
   repertoire the specification allows, the property asserts only that two runs by
   `check` agree. Say in the test why it holds and where it does not, in one sentence
   per technique, as T37's Context argues it.

6. **Snapshots**, once the list is empty, in `crates/pawdoku/tests/snapshots.rs`, each
   by a test named `snapshot_...`, through the public API:

   - the fixture's run under naked and hidden singles: the outcome, the count of steps
     and the grid reached;
   - the run of a puzzle that stalls under naked singles alone, likewise, so a reviewer
     sees what a stalled grid looks like;
   - the `Display` text of every refusal, in one snapshot.

   Reuse the file's render helpers for a grid. Accept with `just snapshots-accept` and
   read each file before committing it. List them in the hand-back notes under
   "Snapshots taken".

7. **Doc examples and the pages.** Every public item this ticket adds has a doc example
   that runs and asserts (`AGENTS.md`: "a doctest on every public item"): the module,
   `check`, `Technique` and its method, `Repertoire` and each of its methods, `Run` and
   each of its methods, `Outcome`, `CheckError`, `LAYOUT_VERSION`. `missing_docs` and
   `just doc` do not notice an absent example, so the hand-back notes list each public
   item beside the doctest that covers it.

   The crate overview in `lib.rs` gains the fifth module and shows a generated puzzle
   checked under naked singles. `docs/explanation/architecture.md`: a section for the
   module in the shape of "The generator": the entry, what it reads, that it keeps its
   own candidates until `technique` lands, the order of choice and why the end does not
   depend on it, that soundness is the catalogue's and holds of true givens. The filling
   is named as T39's. `docs/reference/testing.md`: the suite rows, how the licences are
   tested on hand-worked grids, the order property and the snapshots.
   `docs/explanation/layering.md` and `docs/project/purpose-and-scope.md`: the run is
   built. `docs/project/repository-map.md` and `CHANGELOG.md` follow.

8. **What the study puzzles showed.** Run each of the four under naked singles alone,
   under naked and hidden singles, and under all four, from a test that asserts what
   `layout.allium` guarantees of every run and prints the outcome and the count of
   steps; do not assert the page's claims. Quote the twelve outcomes in the hand-back
   notes as a table beside what the page says of each puzzle, and under Handed back say
   which claims the run bears out, which it cannot speak to (a technique outside the
   four), and which it contradicts, for the maintainer to carry to the page.

9. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`. Quote the closing lines of each,
   the coverage line for the module, and how long `just test` takes beside the figure
   from step 1. Fill in the hand-back notes: the test list as it ended, as a table from
   clause to test name or reason; "Names T39 needs", with every public item's signature
   and each refusal's text, and the `pub(crate)` items T39 is expected to call; and
   under Handed back, what the study puzzles showed and any trigger met. Set
   `status: done` here and in `tickets/README.md`, commit on the ticket branch, and stop
   before pushing.

### What the tests must cover

From the run in `docs/specs/layout.allium`. T37 named the clauses; use its names in the
table. By subject:

- **The repertoire.** The empty set is refused with its text. A repertoire holds what
  it was given and nothing else, and says so for each of the four. The two named sets
  hold the four and the one. It serialises and comes back equal under the feature.
- **Born kept.** On the fixture, every placed cell has its digit as its one candidate,
  and every open cell's candidates are exactly the digits no placed peer holds. Nothing
  is struck that the start does not rule out.
- **The licences**, each on a grid worked out by hand and written in the test, the case
  that licenses and one that does not: a naked single, and a cell with two candidates
  that is none; a hidden single in a row, in a column and in a box, and a digit with two
  places in the unit that is none; pointing along a row and along a column, a box with
  places in two rows that is none, and a box whose line allows nothing outside that is
  none; claiming from a row and from a column, a line with places in two boxes that is
  none. A digit the unit has placed licenses nothing, whatever the candidates say.
- **A step.** A placement puts the digit, leaves the cell one candidate and strikes it
  from every peer; a strike removes exactly the named candidates and nothing else; the
  count of steps rises by one. A technique outside the repertoire is never taken,
  though licensed.
- **The end.** The fixture ends solved under naked and hidden singles, with the solution
  as its grid; a puzzle the solver needs a guess for (Norvig's hard puzzle, which
  `tests/snapshots.rs` and `tests/solver.rs` carry, or one found through `generate`)
  ends stalled under the four, with an unfilled grid; a stalled run's grid holds every given and
  every digit placed. The run ends solved in at most 81 placements and in at most 729
  steps in all, asserted of every run the tests make.
- **`check` and its refusals.** Two givens at one position, two peers with one digit,
  81 givens and the empty repertoire each read as their text; a refusal takes nothing
  and gives no run. `check` cannot panic on any givens.
- **By property, over puzzles from `generate` through the fake**, in each tier: every
  digit a run places is the proof's solution's at that position; a solved run's grid is
  the solution; a solved run's givens get `solver::search`'s verdict of one; the same
  givens and repertoire give an equal run; a run under a larger repertoire never ends
  stalled where a smaller one ended solved; adding a given from the solution to solved
  givens keeps them solved; the end and the digits placed are the same under an order
  proptest chose, for the repertoires the guarantee covers, and `check` is the same
  each time for the rest (step 5).
- **The study puzzles.** Four grids, three repertoires, twelve outcomes recorded (step
  8); the guarantees asserted of each run.
- **`check` from outside.** A puzzle from `generate` is checked under each named
  repertoire and the outcome is read; `Run` is read through every method. Every new
  public type is in the two bounds tests, `Repertoire` and `Technique` in the serde test.

## Acceptance criteria

- `pawdoku::layout` exports `Technique`, `Repertoire`, `check`, `Run`, `Outcome`,
  `CheckError` and `LAYOUT_VERSION` with the meanings above. Nothing in `src/layout`
  names an engine module other than `sudoku` and `solver`.
- `Technique`, `Outcome` and `CheckError` are `#[non_exhaustive]`. The bounds tests
  cannot see the attribute, so it is read by inspection and stated in the hand-back
  notes.
- `check` cannot panic on any givens and any repertoire.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec layout` obligation of the run is in the table or
  struck with a reason; the filling's obligations are struck as T39's.
- The order property of step 5 exists and passes, and the inner loop takes its order
  from outside.
- No recursion, in code or tests. No `#[allow]` and no `qual:allow` line. The only
  `#[expect]` lines are the dead-code ones for items T39 calls, each with its reason.
- Every public item of `layout` has a doc example that runs and asserts, and the
  hand-back notes list each beside its doctest.
- `LAYOUT_VERSION` equals the default of `config.layout_version` in
  `docs/specs/layout.allium`, and a test says so.
- Line coverage of `src/layout` alone is at or above 90 per cent.
- The three snapshots of step 6 exist as `.snap` files, each taken by a test named
  `snapshot_...` through the public API. None appears in the hand-back table as the
  test of a clause. `just snapshots-check` is green, and any earlier snapshot that
  changed has its reason in the hand-back notes.
- The table of step 8 is in the hand-back notes, and `puzzle-design.md` is unchanged.
- `just check` is green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just plan-spec layout
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
rg -n 'crate::(reach|effort|lapse|human_solving|board|generation|random)' crates/pawdoku/src/layout.rs crates/pawdoku/src/layout/
git status --porcelain
```

Expected: the obligation list the table accounts for; each recipe green with no warning;
`metrics` reporting no finding, no recursion among them; both wasm targets under both
feature sets; the coverage total at or above the floor; no engine module named in
`layout` but `sudoku` and `solver`; `All checks passed and the worktree is unchanged.`;
only paths in Files touched.

## Hand-back notes

### Where T37's answers differ from this ticket

### The test list

#### Changes to the list

#### From clause to test

#### The breaks

### Names T39 needs

### Snapshots taken

### What the study puzzles showed

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- **The names of the four in Rust.** The ticket suggests `Technique` with the
  catalogue's names in camel case and a method for the rank; a later `technique` module
  may want the name, and then this enum is the one that goes. The maintainer may want
  the paper's word for the enum instead; the behaviour is fixed either way.
- **What `Run` exposes.** The ticket returns the outcome, the count of steps and the
  digits placed. The deductions taken, each with its technique, would serve a later
  comparison of routes; they are not returned here, and a ticket that wants them adds
  them.
- **The candidate set.** Written in the module, or the solver's widened, as
  `just metrics` decides; the ticket's preference is the module's own, so that the two
  modules share nothing but the public rules.
- **Speed.** No figure is promised. If `just test` slows by more than a few seconds,
  say so in the hand-back notes with the test that costs it.
