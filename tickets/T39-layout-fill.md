---
id: T39
title: "A layout filled in Rust: trials over drawn grids, the check, and the figures beside the paper's"
status: open
depends_on: [T38]
parallel_with: []
branch: ticket/t39-layout-fill
estimated_size: L
---

# T39: A layout filled in Rust: trials over drawn grids, the check, and the figures beside the paper's

## Context

T37 wrote `docs/specs/layout.allium` and T38 built its run: `layout::check` says whether
a repertoire of the four techniques solves a set of givens without guessing. This ticket
builds the other half, the filling: a layout of positions and a repertoire go in, and
trials draw solution grids through the randomness boundary, read each at the layout and
put the givens to the run, until one passes or the trial limit is spent. It is the
generate-and-test search of Nishikawa and Toda's paper, built faithfully so that the
engine's second generator can be set beside the basic one and beside the paper's own
figures. The last step measures what the yardsticks give.

**This ticket's words may be a day old.** It says layout, filling, trial, repertoire and
trial limit, the words T37 proposed, and `check`, `Run` and `Repertoire`, the names T38
fixed. T37's and T38's hand-back notes hold the words and the answers the maintainer
chose. Where they differ from this ticket, they win, and this ticket is read with them:
in particular whether the trial limit is the caller's, whether a layout of fewer than 17
positions or of 81 is refused at the asking, and what a refusal by trials shows.

The filling, as T37 was asked to specify it. Read `layout.allium` for what it says now.

1. **The asking.** A layout, a set of positions each on the grid and each once; a
   repertoire; a stream of draws; and, if T37 made it the caller's, the trial limit. A
   layout that can never be filled is refused before any draw, as T37 settled.
2. **A trial.** A solution grid is drawn exactly as the basic way draws one
   (`generation.allium`'s grid attempts: eleven givens, two draws each, the solver,
   the lower of two solutions, at most a hundred failed attempts, and then a refusal).
   The candidate givens are the grid's digits at the layout's positions and nothing
   else. They go to a run under the repertoire. Solved passes the trial; stalled fails
   it.
3. **The ends.** A passed trial finishes the filling with its givens; the solver's proof
   of them cannot refuse, since a solved run shows one solution. A failed trial opens
   the next, until the trial limit is spent, when the filling is refused by trials
   with the count.
4. **The draws.** Every draw is a grid attempt's: 22 for a full seeding, `2k + 1` for a
   short one; the run takes none and nothing else of the filling draws.
5. **What comes back.** The proof of the givens, which exposes the solution, and the
   count of trials.

**Decided by the maintainer on 2026-10-05:** the problem is positions in and digits out,
solved by this search and not by the paper's encoding; the filling is public API. The
shape below is the ticket writer's recommendation; the maintainer asks for one where a
choice is about Rust and not about the product. Report what was built under "Names
later tickets need".

- **`Layout` is a public set of positions**, `#[non_exhaustive]`, built from any
  positions and refusing what the specification refuses at the asking: a position off
  the grid, a position twice, fewer than the figure, or 81. It says its count and
  iterates its positions in row order. It serialises under the `serde` feature as
  `Tier` does, so a caller can record what was asked for.
- **One entry.** `layout::fill(layout, repertoire, trial_limit, stream)` takes a
  `&Layout`, a `Repertoire`, the limit as a `u32` if T37 made it the caller's, and a
  `&mut dyn RandomStream`, and returns `Result<Filled, FillError>`. If T37 put the limit
  in config, it is a constant and `fill` has three arguments.
- **`Filled` carries the proof and the count**, and is `#[non_exhaustive]`. `proof()`
  gives the `WellPosed` of the givens, from `solver::solve` once at the end, so the
  proof's constructor keeps its one caller (decision 0014); `trials()` gives how many
  trials the filling took, which the comparison needs. A consumer opens a board with
  `Board::open(filled.proof().givens().iter().copied())`.
- **`FillError` is a `thiserror` enum**, `#[non_exhaustive]`, with stable `Display`
  text: the stream ran out (it wraps the `RandomError` transparently, as
  `GenerateError::Stream` does); grid attempts were spent in a trial, with the trial's
  number; trials were spent, with the count; and one conversion from `SolveError` for a
  refusal no input reaches, as `GenerateError::NotProved` is. The refusals of the layout
  and of the repertoire are their constructors', not `fill`'s, unless T37 settled
  otherwise.
- **It draws from wherever the stream stands**, as `generate` does, and its
  documentation says what replays a filling: the layout, the repertoire, the limit,
  `LAYOUT_VERSION`, `GENERATION_VERSION`, `RANDOM_VERSION`, the seed and the index the
  stream stood at; or a fresh stream for each filling. `catalogue_version` is the
  specification's fourth name and has no Rust constant yet; say so in one sentence.

Facts that shape the work:

- **The grid attempt is `generation`'s, reused and not copied.** `generation::grid::draw_grid`
  (`src/generation/grid.rs`, `pub(super)`) draws a grid attempt after attempt and gives
  `Ok(None)` when a hundred have failed. Its module is private: `generation.rs` declares
  `mod grid;`, so widening the function alone does not reach it. Widen `draw_grid`, and
  `GRID_ATTEMPT_LIMIT` if the refusal's text wants the figure, to `pub(crate)` in
  `grid.rs`, and `mod grid;` to `pub(crate) mod grid;` in `generation.rs`, keeping the
  path `generation::grid::draw_grid`; change nothing else in `generation`: its tests,
  its pins and `GENERATION_VERSION` stay as they are, because no draw of the basic way
  changes. T32's notes say T20 may reuse `draw_grid` as it is;
  this is the first module to do so from outside.
- **A trial never draws for itself.** The layout is the caller's and the run draws
  nothing, so a filling's draws are its grid attempts' alone. Build the trial so, and
  test the count: a filling whose every trial finds its grid at once takes 22 draws a
  trial, and one refused by trials after n trials of one attempt each took 22n.
- **A passed trial's proof cannot refuse.** The run's `SolvedMeansOne` guarantee says
  the givens have one solution. `solve` is still asked, and its refusal carried as one
  `?` into a variant no input reaches, tested directly, as `generate` does.
- **What a seed gives depends on the solver, the grid attempt and the run.** Eleven
  givens nearly always have many solutions, which two the search reaches follows from
  the solver's tie-break and propagation, which deduction a run takes follows from
  T38's order of choice, and the draws are the grid attempt's. A change to any of them
  changes what a seed gives; the pins say so and name `LAYOUT_VERSION` and
  `GENERATION_VERSION`.
- **One filling is many searches.** A trial is a grid attempt (one or more solver
  searches) and a run; a filling is up to the limit of them. Keep each property's case
  count and limit small, and quote how long `just test` takes before and after.
- **Draws come through the fake.** `docs/reference/testing.md`: "A test that needs
  randomness scripts its draws with `ReplayStream`". A property hands the fake draws
  that proptest made, a script long enough for the limit's worth of trials. The tests
  that must drive `SeededStream` are the pins and the measurements, and they say so.
- **The limits of the metrics gate** are as T38 states them. Meet them by structure:
  `src/layout/<part>.rs` for the layout, the trial and the filling; no recursion.
- **The boundary rule.** `layout_imports_sudoku_solver_technique_generation` covers
  `src/layout{.rs,/**}`; this ticket adds `crate::generation` and `crate::random` to
  what the module names, and nothing else.
- **The dead-code expectations T38 left** on items only this ticket calls come off here.
- **Snapshots show and detect change; they prove nothing**, as every ticket since T29
  says.
- **A layout is drawn in a test as the paper drew its instances**: a count n, then n
  distinct cells at random. A test makes one from a `SeededStream` by choosing, for
  each of n picks, an index among the positions not yet chosen in row order, as
  `generation`'s drawn order does; write that helper in the test and not in the library.
- **The paper's 30 layouts** of its Tables 2 and 3, Sudokus with 17 givens drawn at
  random from Royle's collection, are copied here as the paper prints them, nine runs
  of nine digits separated by periods, 0 for an empty cell, with the paper's running
  time in seconds for its exact method under naked singles alone, a dash where it did
  not end within several hours. A test forgets the digits and keeps the positions, as
  the paper did. The paper proved each timed one not solvable by naked singles alone
  with any digits; of the others it says nothing. One time is in question: see the
  Open point "852 or 849" before embedding the table.

   | Grid | Time |
   | --- | --- |
   | `090600000.000080300.000000010.060000800.000205000.000041000.000300702.401000000.500000000` | – |
   | `050608000.300000070.000000000.000400601.700100500.200000000.061000000.000070020.000090000` | 926 |
   | `000600370.801000000.000200000.070010060.000004500.200080000.060700000.000050800.000000000` | – |
   | `005000060.000780000.000000000.200000407.001300000.000000800.000601030.040070000.580000000` | – |
   | `031000000.000400006.000000200.600059000.000010030.400000000.000200800.050000010.700600000` | 1,243 |
   | `000000025.000601000.090000000.805000600.000020000.000000300.040250000.300000790.000800000` | 1,255 |
   | `600500300.000000010.000000000.000000596.010024000.000000000.704000800.000210000.300900000` | – |
   | `000010600.050000030.000080000.700500020.000002000.008000000.530900000.000400807.000000100` | – |
   | `000530800.700600000.400000000.100024000.000000630.000000000.050301000.000000042.080000000` | 1,070 |
   | `070060030.500400100.000000000.400501000.300000076.000800000.001000500.060020000.000000000` | – |
   | `000700380.501000000.000200000.000000506.070400000.000000900.300056000.080010040.000000000` | – |
   | `603001020.000800500.200000000.050040700.000003000.000200000.040750000.000000031.000000000` | 994 |
   | `000030001.007500000.600000000.810002000.000600350.400000000.003000760.040080000.000000000` | – |
   | `500300000.000000801.004000600.000600430.710000000.000500000.200000060.000078000.000010000` | 6,564 |
   | `080071000.000040600.000000000.040000008.000600010.200500000.603000500.500200000.000080000` | 1,134 |
   | `400000076.000081000.000000000.000630004.500000200.017000000.320400000.000000810.000000000` | – |
   | `020540000.040000006.000000010.080700500.900020000.000006000.603000000.000300200.100000000` | 26,835 |
   | `500060107.030200000.400000000.280000030.000007000.000010000.000800020.000400600.001000000` | – |
   | `400010000.060000020.000000000.000500270.301400000.008000000.000600100.070002000.100000003` | – |
   | `008000200.400050000.000600070.000082000.060000050.000300000.950100000.000000306.000000800` | – |
   | `010000300.000042000.000090000.000800100.205000000.600000004.000310000.900000020.000700050` | 10,090 |
   | `050000200.000700010.600080000.012000050.000600040.000030000.900000308.000001000.000000600` | 852 |
   | `600050043.200000007.000400010.070200000.000060200.010000000.500000800.000730000.000000000` | – |
   | `304050600.000200000.600000000.080000072.000031000.000000000.120700000.000000340.000000009` | 889 |
   | `040000300.000072000.000010800.200000010.050700000.000050000.000800400.701000000.600300000` | – |
   | `400080000.000500100.000000200.000034070.001000000.060000000.750000030.000001640.000200000` | 1,021 |
   | `000042500.100000070.000000000.400700000.000000208.000000650.025000000.000830000.060100000` | 869 |
   | `600800000.000090500.000000020.025000700.090000300.000400001.100300008.000050060.000000000` | 849 |
   | `200400000.000000031.000000007.000702500.301000000.900800000.080000400.000030090.070000000` | – |
   | `007000010.400700000.000800030.200000400.000010000.000300000.000002709.530000000.080000600` | – |

- **The paper's random instances cannot be had offline.** Its 100 position sets of
  Section 7.2 are on the authors' site, and fetching them is a network action; the
  measurement below draws its own and says they are not the paper's. The paper confirmed
  every one of its 100 strategy-solvable before timing, which this search cannot do, so
  a layout this search does not fill may be unfillable or merely unfilled.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/layout.allium` and
`docs/specs/generation.allium` in full; `tickets/T37-layout-spec.md` and
`tickets/T38-technique-run.md`, Context and hand-back notes; the decision record T37
wrote; `tickets/T32-basic-generator.md` whole, as the nearest ticket in shape and the
source of the figures the comparison stands beside; `crates/pawdoku/src/layout.rs` and
`crates/pawdoku/src/layout/`; `crates/pawdoku/src/generation.rs` and
`crates/pawdoku/src/generation/grid.rs`; `crates/pawdoku/src/random.rs`;
`crates/pawdoku/tests/generation.rs`, as the shape of the tests; `docs/explanation/architecture.md`,
"The generator" and the section T38 added; `docs/reference/testing.md`;
`.agents/skills/rust-change/SKILL.md` and `.agents/skills/propagate/SKILL.md`;
`rustqual.toml` and `clippy.toml`.

## Goal

`pawdoku::layout` is whole: `fill` gives a layout, a repertoire and a stream of draws a
well-posed puzzle whose givens sit at exactly the layout's positions and which the
repertoire solves without guessing, the same puzzle for the same draws, or a refusal
that says why. A puzzle shaped to a layout can now be made, opened as a board and played
with nothing but this crate and a seed. Every clause of the filling in `layout.allium`
is matched by a named test or a stated reason, listed in the hand-back notes. The
comparison figures are in the hand-back notes, beside the basic generator's and the
paper's. `just check` is green with the coverage floor held by this module's own tests
and no suppression beyond the one cast, if one is needed.

Names later tickets are written against, fixed here: `layout::Layout`, `layout::fill`,
`layout::Filled` and `layout::FillError`. Every other name is this ticket's to choose
and to report.

## Non-goals

- Nothing of the designed way: no symmetry scheme (a caller who wants a symmetric
  layout hands one over), no ceiling, no required or forbidden set, no shortcut
  suppression, no pacing.
- No criterion for the generation step and no transformation of a grid between trials:
  each trial is a fresh grid attempt, as the specification states. If a measurement
  suggests a criterion, it is handed back and not built.
- No claim that a layout cannot be filled; a refusal by trials is a count.
- No rating and no claim that one layout's puzzles are harder than another's.
- No change to the behaviour of `sudoku`, `solver`, `board`, `random` or `generation`;
  the one edit to `generation` is the visibility of its grid attempt.
- No change to `layout.allium` or any module. If no clause says what the code needs,
  stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No fetch of the paper's instances or of Royle's collection: the figures stand on what
  is embedded here and drawn here.
- No benchmark, no fuzz target, no mutation job, no command line and no binding. Each
  is named under Handed back as a trigger this ticket meets.
- No snapshot standing in for a test that asserts.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/layout.rs` | `fill`, `Filled`, `FillError`, and the module documentation now covering the filling |
| `crates/pawdoku/src/layout/` | New files for the layout and the trial; the dead-code expectations T38 left come off |
| `crates/pawdoku/src/lib.rs` | The crate overview: a puzzle filled to a layout |
| `crates/pawdoku/src/generation.rs` | `mod grid` to `pub(crate)`; nothing else |
| `crates/pawdoku/src/generation/grid.rs` | `draw_grid`, and `GRID_ATTEMPT_LIMIT` if the refusal's text wants it, to `pub(crate)`; nothing else |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type |
| `crates/pawdoku/tests/layout.rs` | `fill` through the public API; the pins; the measurements |
| `crates/pawdoku/tests/snapshots.rs` | The snapshot tests of step 6 |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | The `layout` section: the filling, its draws, the proof from the solver, what a seed gives |
| `docs/explanation/layering.md` | The sentence that the module is now built whole, and that it names `generation` for the grid attempt |
| `docs/project/purpose-and-scope.md` | The generation bullet: the second generator is built |
| `docs/reference/testing.md` | The suite table; the filling's tests; the tests that drive `SeededStream` and why; the measurements; the snapshots |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `crates/pawdoku/README.md` | What the crate can now do, if the page lists it |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T39's row set `done` |
| `tickets/T39-layout-fill.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t39-layout-fill` from `main` after T38 has merged.
   Run `just initialize` if `.pixi/` is absent, and `just install-allium` and
   `just install-tools` if `.tools/bin/allium` or `.tools/bin/rustqual` is; each uses
   the network and this ticket authorises it. Run `just check` and confirm it is green
   before any edit; note how long `just test` takes. Read T37's and T38's hand-back
   notes and write down, at the top of this ticket's hand-back notes, each word or
   answer that differs from this ticket.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec layout` and add any obligation of the filling
   the list lacks; obligations of the run are T38's and are struck with that reason.
   Where an obligation names a structure the implementation does not have, strike the
   line with the reason that the module excludes how a trial is stored, and name the
   test of the observable result that stands in for it. Put the list in the hand-back
   notes under "The test list" before the first test is written.

3. **Work the list one line at a time**, as T38 and T32 did: write the test, see it fail
   for the expected reason, write the least code, tidy, re-read the list, note each
   change with its reason. Run `just clippy` and `just metrics` every few loops. A good
   first line is the smallest whole behaviour: a layout refuses a position twice. Then
   the layout's other refusals and its order; then reading a grid at a layout; then a
   trial on a scripted grid attempt that passes, and one that fails; then the next
   trial after a failed one; then spent trials; then spent grid attempts inside a
   trial; then `fill`.

4. **Each test must be able to fail.** When a line goes green on arrival, break the code
   on purpose, see the test fail, and restore it. List the breaks in the hand-back
   notes. Breaks worth making: a trial reading one position too many; a passed trial
   not ending the filling; the limit off by one; a draw taken outside the grid
   attempt; the proof taken from the run's grid instead of the solver.

5. **Pin what a seed gives.** Two layouts are written out in the test, each as 81
   characters with `x` for a position in the layout: a layout of 30 positions with
   half-turn symmetry, and one of 45 drawn at random by the helper above from seed 1.
   One test drives `SeededStream` from seed zero through `fill` on each under the four
   techniques, with the limit the test names, and compares the givens and the count of
   trials with literals in the test, beside an assertion on `LAYOUT_VERSION` and
   `GENERATION_VERSION`. If seed zero is refused by trials on either, raise the limit or
   choose another layout and say so in the test: a pin of a refusal pins nothing. A second test folds the
   givens of seeds 0 to 99 on the first layout into one number by a rule written out in
   the test, and pins that number. Say in both tests that the givens follow from the
   solver's tie-break, from what its propagation strikes, from the grid attempt's draws
   and from the run's order of choice, and that whoever changes any of them changes the
   constant in the same commit.

6. **Snapshots**, once the list is empty, in `crates/pawdoku/tests/snapshots.rs`, each
   by a test named `snapshot_...`, through the public API:

   - the puzzle seed zero gives on the symmetric layout of step 5 under the four
     techniques: the layout as a picture, the givens as a grid, the solution as a grid,
     and the count of trials. The file holds a solution on purpose: the proof exposes
     it;
   - the `Display` text of every refusal, in one snapshot.

   Reuse the file's render helpers. Accept with `just snapshots-accept` and read each
   file before committing it. List them under "Snapshots taken".

7. **Doc examples and the pages.** Every public item this ticket adds has a doc example
   that runs and asserts: `fill`, `Layout` and each of its methods, `Filled` and each of
   its methods, `FillError`. The hand-back notes list each beside its doctest.

   The crate overview in `lib.rs` shows a puzzle filled to a small symmetric layout from
   a seed and opened as a board. `docs/explanation/architecture.md`: the `layout`
   section gains the filling, in the shape of "The generator": the entry, the draws and
   their count, that the grid attempt is `generation`'s, that the proof is the solver's
   and the one path it cannot take, what a seed gives and the four names.
   `docs/reference/testing.md`: the suite rows, how a trial is tested on a scripted
   grid attempt, the tests that drive `SeededStream` and why they are the exception,
   the measurements and what they do not show, and the snapshots.
   `docs/explanation/layering.md` and `docs/project/purpose-and-scope.md`: the second
   generator is built. `docs/project/repository-map.md` and `CHANGELOG.md` follow.

8. **Measure what the yardsticks give.** Three tables, each from a test that asserts
   what `layout.allium` guarantees of every filling and every run and prints the
   figures; do not assert the figures themselves, which follow from the solver, the
   grid attempt and the run. Read them with `NEXTEST_SUCCESS_OUTPUT=immediate just test`
   and quote them in the hand-back notes. Keep the whole of `just test` within a few
   seconds; if a table needs more, shrink its counts and say so.

   1. **Random layouts in the paper's six buckets.** For each bucket (20 to 29, 30 to
      39, 40 to 49, 50 to 59, 60 to 69, 70 to 79 positions), a small number of layouts
      the ticket names, each drawn from `SeededStream` by the helper above with a count
      drawn within the bucket; each filled from a fresh `SeededStream` under the four
      techniques and again under naked singles alone, with a trial limit the test
      names. Record, per bucket and repertoire: how many were filled, the fewest, the
      most and the median trials of those that were, and how many were refused. Say in
      the notes that the layouts are not the paper's and were not confirmed fillable,
      so a refusal may be an unfillable layout or an unfilled one, and that the
      paper's finding to set it beside is that generate-and-test filled none of its
      instances with 50 or more positions in 600 seconds.
   2. **The paper's 30 layouts** under naked singles alone, with the same limit, each
      from a fresh `SeededStream`: how many were filled and the trials spent. The
      figure to set beside it is the paper's: 14 proved unfillable under naked singles
      by its exact method, 16 left undecided. Filling one of the 16 is a result, to be
      reported with its givens; filling one of the 14 timed ones would contradict a
      proof and is a defect to find before anything else is reported.
   3. **The basic generator's puzzles.** For seeds 0 to 99 in each tier,
      `generation::generate` and then `check` under naked singles alone, under naked
      and hidden singles, and under the four: how many of the hundred each repertoire
      solves. This is the first comparison of the two generators' output, and it says
      what the basic way's tiers ask of a solver who never guesses.

9. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`. Quote the closing lines of each,
   the coverage line for the module, and how long `just test` takes beside the figure
   from step 1. Fill in the hand-back notes: the test list as it ended, as a table from
   clause to test name or reason; "Names later tickets need", with every public item's
   signature and each refusal's text; the three tables; and under Handed back, the
   triggers this ticket meets, each for the maintainer to pick up and none built here:

   - T20: a layout is what the designed way's symmetry scheme produces, and `fill` is a
     way to fill one; what T20 can reuse.
   - T15 to T17, the bindings and the command line S04 drafted: `fill` and `check` are
     entries that need the layout and the repertoire carried across.
   - T14, the mutation job; S03's benchmark recipe: one `fill` per bucket is a natural
     benchmark.
   - `docs/explanation/puzzle-design.md`: what the comparison says of the technique
     contract's shape, for the maintainer to carry to the page if anything.
   - Any criterion for the generation step the measurements suggest, as an open
     question's evidence and not a build.

   Set `status: done` here and in `tickets/README.md`, commit on the ticket branch, and
   stop before pushing.

### What the tests must cover

From the filling in `docs/specs/layout.allium`. T37 named the clauses; use its names in
the table. By subject:

- **The layout.** A position off the grid is refused, and so is a position twice;
  fewer than the figure, and 81, are refused as T37 settled; a layout says its count and gives its positions in row order;
  it serialises and comes back equal under the feature. Each refusal reads as its text.
- **Reading a grid.** The candidate givens of a trial are the grid's digits at the
  layout's positions, each once, and nothing else; on the fixture's solution and a
  layout written in the test.
- **A trial**, on scripted draws that seed a known grid: a layout the fixture's solution
  fills under the four passes and ends the filling with those givens; a layout it does
  not fill fails and the next trial begins on the next draws; the draws a trial took
  are its grid attempt's, 22 for one full seeding, and the run took none.
- **Spent trials.** A script of n failing trials is refused with the count n, and took
  22n draws when every attempt seeded fully.
- **Spent grid attempts.** A script of a hundred short seedings inside a trial is
  refused as grid attempts spent, naming the trial, and counts as no trial.
- **A stream that runs out.** The error carries the `RandomError`, whether the script
  ends in the first trial's seeding or in a later one's.
- **The proof.** A finished filling's proof has the trial's givens and the trial's grid
  as its solution; the one path for what cannot happen, tested directly.
- **The result, by property, over draws from the fake**, on several layouts written in
  the test and under each named repertoire: a finished filling's givens sit at exactly
  the layout's positions; each is the solution's digit there; `check` of them under
  the repertoire ends solved with the same grid; `solver::search` gives the verdict of
  one and the same solution; a refusal by trials, when the script's trials all fail, is
  the only other end.
- **Replay.** The same draws give the same givens and the same count of trials. Two
  calls on one stream give two puzzles, and the second is had again from a stream
  brought to the index the second call began at. The pinned tests of step 5.
- **`fill` from outside.** A filled puzzle opens as a board and is played to its end.
  Each refusal reads as its text. A stream that breaks its word with draws outside
  `[0, 1)` cannot make `fill` panic. Every new public type is in the two bounds tests,
  `Layout` in the serde test.
- **The measurements** of step 8, each a test that asserts the guarantees and prints.

## Acceptance criteria

- `pawdoku::layout` exports `Layout`, `fill`, `Filled` and `FillError` beside T38's
  items, with the meanings above. Nothing in `src/layout` names an engine module other
  than `sudoku`, `solver`, `generation` and `random`.
- `Layout`, `Filled` and `FillError` are all three `#[non_exhaustive]`; read by
  inspection and stated in the hand-back notes.
- `src/generation` differs from `main` only in the visibility of the `grid` module,
  `draw_grid` and, if needed, `GRID_ATTEMPT_LIMIT`; its tests, snapshots and pins are
  unchanged and `GENERATION_VERSION` is unchanged.
- The only caller of the proof's constructor outside test code is still in
  `src/solver`, read by inspection and stated in the hand-back notes.
- `fill` cannot panic on any layout, repertoire, limit or stream, a stream that runs
  out included.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec layout` obligation of the filling is in the table or
  struck with a reason; the run's obligations are struck as T38's.
- No recursion, in code or tests. No `#[allow]` and no `qual:allow` line. No
  `#[expect]` beyond what the house already has in `generation`, and T38's dead-code
  expectations are gone.
- Every public item this ticket adds has a doc example that runs and asserts, and the
  hand-back notes list each beside its doctest.
- Both pinned tests name `LAYOUT_VERSION` and `GENERATION_VERSION`.
- The documentation of `fill` says it draws from wherever the stream stands and what a
  caller records to have a puzzle again.
- Line coverage of `src/layout` alone is at or above 90 per cent.
- The two snapshots of step 6 exist as `.snap` files, each taken by a test named
  `snapshot_...` through the public API. None appears in the hand-back table as the
  test of a clause. `just snapshots-check` is green, and any earlier snapshot that
  changed has its reason in the hand-back notes.
- The three tables of step 8 are in the hand-back notes, each with the sentence that
  says what it does not show, and no filling of a paper layout the paper proved
  unfillable was reported without being found a defect first.
- `just check` is green.

## Verification

```sh
just plan-spec layout
just fmt-check
just clippy
just metrics
just test
NEXTEST_SUCCESS_OUTPUT=immediate just test
just snapshots-check
just wasm-check
just features
just coverage
just doc
just check-docs
just check
git diff main -- crates/pawdoku/src/generation.rs crates/pawdoku/src/generation
git status --porcelain
```

Expected: the obligation list the table accounts for; each recipe green with no warning;
`metrics` reporting no finding, no recursion among them; the three tables printed; both
wasm targets under both feature sets; the coverage total at or above the floor; a diff
of `generation` that changes visibility and nothing else; `All checks passed and the
worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### Where T37's and T38's answers differ from this ticket

### The test list

#### Changes to the list

#### From clause to test

#### The breaks

### Names later tickets need

### Snapshots taken

### What the yardsticks give

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- **The limits the measurements use.** The ticket leaves the count of layouts per
  bucket and the trial limit to the executing agent, within a few seconds of
  `just test`; a maintainer who wants fuller tables says the figures and accepts the
  time, or asks for the measurement to become a recipe outside `just test`, which is
  a `Justfile` change and a follow-up.
- **What `Filled` carries beside the proof and the count.** The draws taken are read
  from the stream's index by a caller, as `generate`'s are; the ticket adds nothing for
  them. A comparison that wants the grid attempts per trial asks for it in its own
  ticket.
- **852 or 849.** Added on 2026-10-06, after a review. T37's Context says the exact
  method ended in 852 to 26,835 seconds, but the table above lists 849 for
  `600800000.000090500…` as the lowest time, and 852 for
  `050000200.000700010…`. One transcription of the paper's Tables 2 and 3 is wrong,
  and only the paper settles which. No proposal: the maintainer checks the paper
  before T39 embeds the table, and the correction lands in whichever ticket is wrong.
- **Whether `Layout` offers the named symmetries.** A caller builds a symmetric layout
  by hand today. A constructor that mirrors a half-turn, or any scheme, is the designed
  way's open question on symmetry and is not built here.
- **Speed.** No figure is promised. If `just test` slows by more than a few seconds,
  say so in the hand-back notes with the test that costs it, and shrink the
  measurements before anything else.
