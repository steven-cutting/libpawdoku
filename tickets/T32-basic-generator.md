---
id: T32
title: "The basic generator in Rust: the solution grid, removal in order and the five tiers"
status: done
depends_on: [T31, T28]
parallel_with: []
branch: ticket/t32-basic-generator
estimated_size: L
---

# T32: The basic generator in Rust: the solution grid, removal in order and the five tiers

## Context

T31 wrote the basic generator into `docs/specs/generation.allium`: a published method
that draws a full grid, then takes givens away one position at a time in a stated order,
keeping each removal only while the solver's verdict stays one. This ticket builds it as
`pawdoku::generation`. It is the first thing in the crate that makes a puzzle, and the
yardstick the designed generator (T20, not yet a ticket file) will be compared against.

**This ticket's words may be a day old.** It says tier, bound, floor, order of removal
and grid attempt, the words T31 proposed. T31's hand-back notes, under "Names T32 needs"
and "Open points settled", hold the words and the answers the maintainer chose. Where
they differ from this ticket, they win, and this ticket is read with them.

The method, as T31 was asked to specify it. Read `generation.allium` for what it says
now.

| Tier | Givens left | Floor per row and column | Order of removal |
| --- | --- | --- | --- |
| 1 | 51 to 60 (the paper says "more than 50") | 5 | drawn |
| 2 | 36 to 49 | 4 | drawn |
| 3 | 32 to 35 | 3 | every other cell |
| 4 | 28 to 31 | 2 | alternating rows |
| 5 | 22 to 27 | 0 | row order |

1. **The solution grid.** Eleven givens that do not conflict are drawn and put to the
   solver. A solution it finds is the grid. A verdict of none, or a drawn position with
   no digit left, ends the attempt; attempts are counted against a figure.
2. **The bound.** One draw picks a count inside the tier's range.
3. **Removal.** Each of the 81 positions is visited once, in the tier's order. A removal
   is refused if it would leave fewer givens than the bound, or leave the position's row
   or column below the floor. Otherwise the givens without it go to the solver: a
   verdict of one and the given is gone for good, many and it stays for good.
4. **The result.** The givens left, which are well-posed by construction.

**Decided by the maintainer on 2026-10-02:** the basic generator is public API. The
shape below is the ticket writer's recommendation; the maintainer asks for one where a
choice is about Rust and not about the product. Report what was built under "Names later
tickets need".

- **One entry.** `generation::generate(tier, stream)` takes a `Tier` and a
  `&mut dyn RandomStream` and returns `Result<WellPosed, GenerateError>`. If T31 made
  the figure for grid attempts the caller's, it is a third argument.
- **It draws from wherever the stream stands.** The entry takes a stream and not a
  seed, because the boundary is a trait and a test supplies its draws through the fake.
  A stream a caller has already drawn from gives another puzzle than a fresh one from
  the same seed. The documentation of `generate` says so, and says what replays a
  puzzle: the tier, the versions, the seed and the index the stream stood at
  (`RandomStream::index`), or simply a fresh stream for each puzzle.
- **The proof comes from the solver**, and `WellPosed::vouch` keeps its one caller
  (decision 0014). Two shapes do it. Removal can ask `solver::search` for each verdict
  and put the givens left to `solver::solve` once at the end. Or each visit can ask
  `solver::solve` and keep the proof of the last removal that went through, which saves
  the closing search. Choose the one that leaves fewer lines no input can reach, and
  say which under "Names later tickets need". A consumer opens a board with
  `Board::open(proof.givens().iter().copied())`; `board` is not touched.
- **`Tier` is a public enum** of five, `#[non_exhaustive]`, that can say its own range
  and floor, so a caller can tell whether a result met its range.
- **`GenerateError` is a `thiserror` enum**, `#[non_exhaustive]` as every public enum
  is (`AGENTS.md`, invariant 3), with stable `Display` text: the stream ran
  out (it wraps the `RandomError`, transparently, as `SolveError::NotPosed` wraps its
  cause); grid attempts were spent; and, if the shape chosen needs it, one conversion
  from `SolveError` for a refusal no input reaches.
- **`GENERATION_VERSION`** is a public constant equal to the specification's
  `config.generation_version`, as `random::RANDOM_VERSION` equals `random_version`.

Facts that shape the work:

- **A correct removal always has a proof.** The first position visited is always
  removed: 80 givens are above every bound, eight in a row are above every floor, and a
  full grid less one cell has one solution. So "nothing left to play" cannot arrive at
  the end, and neither can the solver's other refusals. `generate` must still not
  panic. Whichever shape is chosen, give what cannot happen one path and test that path
  directly; do not write a branch per impossible case, because unreachable lines count
  against the coverage floor. T26 did the same for the proof constructor's errors.
- **A stream can run out.** `RandomStream::next_draw` returns a `Result`. The library's
  `SeededStream` never fails; the `ReplayStream` fake does when its script ends. Every
  draw's error is carried out of `generate`, at whichever draw it happens.
- **How a draw becomes an index is T31's to settle, and it has two answers.** The
  binary64 product of the draw and `n`, truncated, is not always the floor of the exact
  product: for the draw `6004799503160661 / 2^53` and `n` of 3 the first gives 2 and the
  second 1. T31 proposed the first. If that stands, the index is one multiplication and
  a truncating cast, which is floor for a product that is never negative (`f64::floor`
  may not be nameable under `no_std`); if the exact reading was chosen, compute it from
  the draw's bits in integers. Either way put it in one function, with its `#[expect]`
  lines and reasons where a cast needs them, as `random.rs` does for its own cast. Two
  tests hold it: the largest draw a stream can give picks the last of `n` for every `n`
  from 1 to 81, and the draw above picks what T31's reading says.
- **What the solver does decides the grid.** Eleven givens nearly always have many
  solutions, and `solver::search` stops at two. Which two it reaches follows from the
  rule `pawdoku::solver` documents under "The tie-break", and from what its propagation
  strikes before a guess, since a guess is on a cell with the fewest candidates. A
  change to either changes what a seed gives, with nothing in this module touched.
  Which of two solutions is the grid is T31's rule. The tests that pin what seeds give
  say all this, and so does the documentation of `GENERATION_VERSION`.
- **Removal in a fixed order draws nothing.** It is a function of a solution grid, an
  order, a bound and a floor. Build it so, and test it apart from the grid, on the
  solution of the fixture below.
- **One call is many searches.** Up to 81 for removal, one more if the proof is taken
  at the end, and one for each grid attempt. Keep each property's case count small, and quote how long `just test`
  takes before and after.
- **Draws come through the fake.** `docs/reference/testing.md`: "A test that needs
  randomness scripts its draws with `ReplayStream`". A property hands the fake draws
  that proptest made. The one test that must drive `SeededStream` is the one that pins
  what a seed gives; say so in that test and on the page.
- **No recursion.** `rustqual.toml` sets `allow_recursion = false`. That holds for test
  code too.
- **The rest of the metrics gate** (`rustqual.toml`, `clippy.toml`): a function at most
  60 lines, cognitive complexity 15, cyclomatic 10, nesting 4, five parameters; a struct
  at most 12 fields and 20 methods, LCOM4 at most 2; a file at most 500 code lines
  before its first `#[cfg(test)]`, and 1000 for test code. Meet them by structure:
  `src/generation.rs` plus `src/generation/<part>.rs` (the orders, the grid, removal);
  `mod.rs` is banned. Never by a suppression and never by moving a number.
- **The boundary rule exists.** `generation_imports_sudoku_solver_technique_reach` in
  `rustqual.toml` already covers `src/generation{.rs,/**}`. The module names
  `crate::sudoku`, `crate::solver` and `crate::random` and nothing else.
- **Snapshots show and detect change; they prove nothing** (`docs/reference/testing.md`,
  Snapshots). They are taken from fixed inputs after the module's own tests are green,
  by tests named `snapshot_...` in `crates/pawdoku/tests/snapshots.rs`, from a text
  picture rendered in test code. Nothing is added to the library to feed one.

The fixture from T25, whose solution is a ready solution grid for the tests of removal:

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

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/generation.allium`
and `docs/specs/solver.allium` in full; `tickets/T31-basic-generator-spec.md`, Context
and hand-back notes; the decision record T31 wrote; `tickets/T26-solver.md`, hand-back
notes ("Names later tickets need"); `crates/pawdoku/src/random.rs`,
`crates/pawdoku/src/solver.rs` and `crates/pawdoku/src/sudoku.rs`;
`docs/explanation/layering.md` and `docs/explanation/architecture.md`;
`docs/reference/testing.md`, its Conventions and Snapshots sections;
`.agents/skills/rust-change/SKILL.md` and `.agents/skills/propagate/SKILL.md`;
`rustqual.toml` and `clippy.toml`.

## Goal

`pawdoku::generation` exists and does what `generation.allium` says of the basic
generator. `generate` gives a tier and a stream of draws a well-posed puzzle, the same
puzzle for the same draws, or a refusal that says why. A puzzle can now be made, opened
as a board and played with nothing but this crate and a seed. Every rule, invariant,
guarantee and surface clause the basic way has is matched by a named test or a stated
reason, listed in the hand-back notes. `just check` is green with the coverage floor
held by this module's own tests and no suppression beyond the one cast.

Names later tickets are written against, fixed here: `generation::generate`,
`generation::Tier`, `generation::GenerateError` and `generation::GENERATION_VERSION`.
Every other name is this ticket's to choose and to report.

## Non-goals

- Nothing of the designed way: no symmetry, no technique contract, no `Rate` run.
- No rating and no claim that a tier is harder than another.
- No retry to reach a tier's range, unless T31 settled that there is one.
- No swap of digits, rows, columns or stacks after removal.
- No change to the behaviour of `sudoku`, `solver`, `board` or `random`.
- No change to `generation.allium` or any module. If no clause says what the code
  needs, stop: that is a `spec-change` first.
- No new dependency and no new feature flag.
- No benchmark, no fuzz target, no mutation job, no command line and no binding. Each
  is named under Handed back as a trigger this ticket meets.
- No snapshot standing in for a test that asserts, and nothing added to the library so
  that a snapshot can read it.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/generation.rs` | New: the module's surface |
| `crates/pawdoku/src/generation/` | New: one file per part, as the limits ask |
| `crates/pawdoku/src/lib.rs` | `pub mod generation;` and the crate overview |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type |
| `crates/pawdoku/tests/generation.rs` | New: `generate` through the public API |
| `crates/pawdoku/tests/snapshots.rs` | The snapshot tests of step 6 and their render helper |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | The generator's place: one entry, draws through the boundary, the proof from the solver |
| `docs/explanation/layering.md` | The sentence T31 left saying the Rust module is not yet built |
| `docs/project/purpose-and-scope.md` | The generation bullet: the basic generator is built |
| `docs/reference/testing.md` | The suite table; the generator's tests; the one test that drives `SeededStream`; the snapshots taken |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `crates/pawdoku/README.md` | What the crate can now do, if the page lists it |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T32's row set `done` |
| `tickets/T32-basic-generator.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t32-basic-generator` from `main` after T31 has merged.
   Run `just initialize` if `.pixi/` is absent; it uses the network and this ticket
   authorises it. Run `just check` and confirm it is green before any edit; note how
   long `just test` takes. Read T31's hand-back notes and write down, at the top of this
   ticket's hand-back notes, each word or answer that differs from this ticket.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec generation` and add any obligation the list
   lacks. Obligations that belong to the designed way cannot exist yet; if one appears,
   stop and report. Where an obligation names a structure the implementation does not
   have, strike the line with the reason that the module excludes how the search is
   stored, and name the test of the observable result that stands in for it. Put the
   list in the hand-back notes under "The test list" before the first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. Run `just clippy` and `just metrics` every few loops.
   A good first line is the smallest whole behaviour: row order is the 81 positions,
   each once, row by row. Then the other two fixed orders; then a choice among `n`;
   then removal on the fixture's solution in row order with a bound and no floor; then
   the floor; then the drawn order; then the bound's draw; then a grid attempt; then
   `generate`.

4. **Each test must be able to fail.** When a line goes green on arrival, break the code
   on purpose, see the test fail, and restore it. List the breaks in the hand-back
   notes, as T26 did.

5. **Pin what a seed gives.** One test drives `SeededStream` from seed zero through
   `generate` for each tier and compares the givens with a literal in the test, beside
   an assertion on `GENERATION_VERSION`. One seed is a tripwire and not a proof: a
   change in the solver can move other seeds' grids and leave seed zero's alone. So a
   second test folds the givens of seeds 0 to 99, in one tier, into one number by a
   rule written out in the test, and pins that number. Say in both tests that the
   givens follow from the solver's tie-break, from what its propagation strikes and
   from this module's draws, and that whoever changes any of them changes the constant
   in the same commit.

6. **Snapshots**, once the list is empty, in `crates/pawdoku/tests/snapshots.rs`, each
   by a test named `snapshot_...`, through the public API:

   - for each of the five tiers, the puzzle seed zero gives: the givens as a grid, the
     solution as a grid, and the count of givens. These files hold solutions on purpose:
     the proof exposes them;
   - the `Display` text of every refusal, in one snapshot.

   Reuse the file's render helpers for a grid. Accept with `just snapshots-accept` and
   read each file before committing it. List them in the hand-back notes under
   "Snapshots taken".

7. **Doc examples and the pages.** Every public item this ticket adds has a doc example
   that runs and asserts (`AGENTS.md`: "a doctest on every public item"): the module,
   `generate`, `Tier` and each of its methods, `GenerateError`, `GENERATION_VERSION`.
   `missing_docs` and `just doc` do not notice an absent example, so the hand-back notes
   list each public item beside the doctest that covers it.

   The crate overview in `lib.rs` gains the fourth module and shows a puzzle made from a
   seed and opened as a board. `docs/explanation/architecture.md`: the one entry, what
   it draws and in what order, why the proof is the solver's.
   `docs/reference/testing.md`: the suite rows, how removal is tested apart from the
   grid, the test that drives `SeededStream` and why it is the exception, and the
   snapshots. `docs/explanation/layering.md` and `docs/project/purpose-and-scope.md`:
   the basic generator is built. `docs/project/repository-map.md` and `CHANGELOG.md`
   follow.

8. **Measure what the yardstick gives.** For seeds 0 to 99 in each tier, record the
   fewest and the most givens, how many results fell within the tier's range, and the
   most grid attempts any seed took. Take the figures from a test that asserts what
   `generation.allium` guarantees over those seeds; do not assert the figures
   themselves, which follow from the solver. Quote them in the hand-back notes as a
   table. They are the first row of any later comparison. The source paper reports
   every attempt succeeding for its level 5 under row order, and 13.33 per cent under a
   drawn order.

9. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`. Quote the closing lines of each,
   the coverage line for the module, and how long `just test` takes beside the figure
   from step 1. Fill in the hand-back notes: the test list as it ended, as a table from
   clause to test name or reason; "Names later tickets need", with every public item's
   signature and each refusal's text; and under Handed back, the triggers this ticket
   meets, each for the maintainer to pick up and none built here:

   - S08's control arm. Removal at random now exists in Rust, without `Rate`.
   - T15 to T17, the bindings and the command line S04 drafted: `generate` is the first
     entry that needs the randomness boundary carried across.
   - T14, the mutation job S03 drafted.
   - S03's benchmark recipe: one `generate` per tier is a natural benchmark.
   - T20: what the designed way can reuse of the solution grid and of removal.

   Set `status: done` here and in `tickets/README.md`, commit on the ticket branch, and
   stop before pushing.

### What the tests must cover

From the basic way in `docs/specs/generation.allium`. T31 named the clauses; use its
names in the table. By subject:

- **The orders.** Each of the three fixed orders is the 81 positions, each once. Row
  order begins (1,1), (1,2), (1,3). Alternating rows runs (1,8), (1,9), (2,9), (2,8) at
  its first turn. Every other cell begins (1,1), (1,3), (1,5), (1,7), (1,9), (2,8),
  (2,6), (2,4), (2,2), (3,1), takes 41 positions on its first pass, and then the rest
  as T31 settled. The drawn order visits each position once whatever the draws, and a
  scripted set of draws gives the order worked out by hand. It takes one draw for every
  visit, 81 in all, whether or not removal reached the bound early.
- **A choice among `n`.** A draw of zero picks the first. The largest draw a stream can
  give picks the last, for every `n` from 1 to 81. The draw `6004799503160661 / 2^53`
  among three picks what T31's reading says, and the test says which reading that is.
- **Which solution is the grid.** Eleven scripted givens with more than one solution
  give the grid T31's rule names, not the other.
- **A grid attempt.** Scripted draws place eleven givens, no two in conflict. A script
  that leaves a drawn position no digit ends the attempt. Eleven givens that agree and
  have no solution end the attempt: the digits 1 to 8 along a row and a 9 in the box of
  the ninth cell, off that row, leave the ninth cell nothing. The next attempt then
  begins.
- **Spent attempts.** The refusal T31 specified, with the count.
- **A stream that runs out.** The error carries the `RandomError`, whether the script
  ends in the grid, at the bound or in a drawn order.
- **The bound.** It lies in the tier's range, and both ends of the range can be drawn.
- **Removal.** A removal that would go below the bound is refused. One that would leave
  a row below the floor is refused; a column likewise. A given whose removal gives many
  solutions stays. A given whose removal keeps one solution goes. No position is asked
  about twice.
- **The result, by property, for every tier.** Every given sits in the solution at its
  position; `solver::search` gives the givens the verdict of one and the same solution;
  the count is at least the low end of the tier's range; every row and column holds at
  least the floor.
- **Replay.** The same draws give the same givens. Two calls on one stream give two
  puzzles, and the second is had again from a stream brought to the index the second
  call began at, and not from a fresh stream of the same seed. The pinned tests of
  step 5.
- **`generate` from outside.** A puzzle from a seed opens as a board and can be played.
  Each refusal reads as its text. The one path for what cannot happen, tested directly.
  Every new public type is in the two bounds tests.

## Acceptance criteria

- `pawdoku::generation` exports `generate`, `Tier`, `GenerateError` and
  `GENERATION_VERSION` with the meanings above. Nothing in `src/generation` names an
  engine module other than `sudoku`, `solver` and `random`.
- `Tier` and `GenerateError` are both `#[non_exhaustive]`. The bounds tests cannot see
  the attribute, so it is read by inspection and stated in the hand-back notes.
- The only caller of the proof's constructor outside test code is still in
  `src/solver`, read by inspection and stated in the hand-back notes.
- `generate` cannot panic on any tier and any stream, a stream that runs out included.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec generation` obligation of the basic way is in the
  table or struck with a reason.
- No recursion, in code or tests. No `#[allow]` and no `qual:allow` line. The only
  `#[expect]` added is on the cast that turns a draw into an index, with its reason.
- Every public item of `generation` has a doc example that runs and asserts, and the
  hand-back notes list each beside its doctest.
- Both pinned tests name `GENERATION_VERSION`, and the constant equals the default of
  `config.generation_version` in `docs/specs/generation.allium`.
- The documentation of `generate` says it draws from wherever the stream stands and
  what a caller records to have a puzzle again.
- Line coverage of `src/generation` alone is at or above 90 per cent.
- The six snapshots of step 6 exist as `.snap` files, each taken by a test named
  `snapshot_...` through the public API. None appears in the hand-back table as the
  test of a clause. `just snapshots-check` is green, and any earlier snapshot that
  changed has its reason in the hand-back notes.
- The table of step 8 is in the hand-back notes.
- `just check` is green.

## Verification

```sh
just plan-spec generation
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

### Where T31's answers differ from this ticket

Read on 2026-10-03, before any edit, from T31's "Names T32 needs" and "Open points
settled".

- **The word "visit"** is new: the one look a position gets. This ticket says "each of
  the 81 positions is visited once" and means the same.
- **The figure for grid attempts is config's**, 100, and not a caller's, so `generate`
  keeps two arguments.
- **The orders have names**: `drawn`, `every_other`, `s_path` and `row_by_row`, where
  this ticket says drawn, every other cell, alternating rows and row order.
- **A draw's index** is the first reading: the binary64 product truncated. The worked
  draw among three picks index 2.
- **Which of two solutions is the grid**: the one with the lower digit at the first
  position, row by row, where the two differ, and not the first the search found.
- **A short seeding** takes `2k + 1` draws and is not put to the solver.
- **No retry** to reach a tier's range.
- Everything else agrees.

Settled with the maintainer on 2026-10-03, before the first test:

- **The specification binds and the paper cross-checks it.** The paper was fetched once
  and read beside `generation.allium`: Table 1 and Table 2, the eleven givens of its
  Section 5.2.1, the four sequences and their assignment in Table 7, and the two
  restrictions. Each is as T31 wrote it down. Nothing was found that the module lacks.
- **The tiers are `Tier::One` to `Tier::Five`.**

### The test list

Written before any code, on 2026-10-03. One line a test: its name, and what it expects.
Changes made while working it are under "Changes to the list" below.

The orders (`src/generation/order.rs`):

1. `row_by_row_is_every_position_once_a_row_at_a_time`: 81 positions, each once; it
   begins (1,1), (1,2), (1,3) and step 10 is (2,1).
2. `s_path_turns_at_the_end_of_each_row`: 81 positions, each once; steps 8 to 11 are
   (1,8), (1,9), (2,9), (2,8).
3. `every_other_takes_the_odd_steps_of_the_s_path_then_the_even`: 81 positions, each
   once; it begins (1,1), (1,3), (1,5), (1,7), (1,9), (2,8), (2,6), (2,4), (2,2), (3,1);
   the first 41 are the positions whose row and column sum to an even number; step 42
   on is (1,2), (1,4), (1,6), (1,8), (2,9), (2,7).
4. `each_tier_has_the_order_the_paper_assigns_it`: drawn for tiers 1 and 2, every other
   for 3, the S path for 4, row order for 5.

A choice among `n` (`src/generation/choice.rs`):

1. `a_draw_of_zero_picks_the_first`: index 0 for every `n` from 1 to 81.
2. `the_largest_draw_picks_the_last_and_never_n`: `1 - 2^-53` gives `n - 1` for every
   `n` from 1 to 81.
3. `the_index_is_the_binary64_product_truncated`: `6004799503160661 / 2^53` among 3 is
   2, the binary64 reading, where exact arithmetic would say 1.

Removal, on the fixture's solution, with no draw (`src/generation/removal.rs`):

1. `removal_begins_from_the_whole_grid_and_the_first_visit_empties_its_position`.
2. `a_removal_that_would_go_below_the_bound_is_refused`: row order, a bound of 78, no
   floor: three positions go and every later visit is refused by the bound.
3. `a_removal_that_would_leave_a_row_below_the_floor_is_refused`.
4. `a_removal_that_would_leave_a_column_below_the_floor_is_refused`.
5. `the_bound_is_read_before_the_floor`: a visit that breaks both is refused by the
   bound.
6. `a_given_whose_removal_gives_many_solutions_stays`.
7. `a_given_whose_removal_keeps_one_solution_goes`.
8. `every_position_is_visited_and_none_is_asked_about_twice`: all 81 visits run after
   the bound is reached, and what was reached comes back.
9. `removal_stops_at_the_first_draw_the_order_cannot_give`: the order's error is
   carried out.

The drawn order (`src/generation/order.rs`):

1. `the_drawn_order_visits_each_position_once_whatever_the_draws`, by property.
2. `scripted_draws_give_the_order_worked_out_by_hand`: the last of 81, the first of
   80, the forty-first of 79, then zeros: (9,9), (1,1), (5,6), then row order.
3. `the_drawn_order_takes_a_draw_for_every_visit_the_last_included`: 81 draws.
4. `a_stream_that_runs_out_in_a_drawn_order_says_so`.

The bound (`src/generation.rs`):

1. `the_bound_is_drawn_within_the_tiers_range_fewest_first`: a draw of zero gives the
   fewest and the largest draw the most, for every tier.
2. `each_tier_says_its_figures`: the fifteen figures of config, and the number.
3. `the_version_is_the_specifications`: `"generation-1"`.

A grid attempt (`src/generation/grid.rs`):

1. `scripted_draws_seed_eleven_givens_no_two_in_conflict`: a position among the empty
   ones in row order, then a digit among those allowed, lowest first.
2. `a_position_with_no_digit_left_ends_the_seeding_short`: `2k + 1` draws, no search.
3. `eleven_givens_with_no_solution_fail_the_attempt`: 1 to 8 along row 1 and a 9 at
   (2,9): 22 draws.
4. `the_next_attempt_begins_after_a_failed_one`: the draws a failed attempt took stay
   taken, and the grid is the second attempt's.
5. `of_two_solutions_the_grid_is_the_lower_row_by_row`: eleven scripted givens for
   which the search finds the higher grid first.
6. `a_found_grid_is_full_and_holds_every_seeded_given`.
7. `a_hundred_failed_attempts_are_a_refusal`: `GridAttemptsSpent { attempts: 100 }`,
   and the draws taken are those of the hundred attempts and no more.
8. `a_stream_that_runs_out_in_the_grid_says_so`: not a failed attempt.

`generate`, inside the crate (`src/generation.rs`):

1. `refusal_texts_are_stable`.
2. `the_refusal_converts_from_the_solvers`: the one path for what cannot happen,
   called directly with each `SolveError`.
3. `the_refusal_converts_from_the_streams`.

`generate` from outside (`tests/generation.rs`):

1. `every_tier_gives_a_puzzle_that_keeps_the_guarantees`, by property over draws from
   the fake: every given sits in the solution; `search` gives one and the same
   solution; the count is at least the tier's fewest; every row and column holds the
   floor; at most 80 givens.
2. `the_same_draws_give_the_same_givens`, by property.
3. `the_draws_taken_follow_from_the_grid_attempts_and_the_tier`: 23 draws for tiers 3
   to 5 and 104 for tiers 1 and 2 when the first attempt finds a grid.
4. `two_calls_on_one_stream_give_two_puzzles_and_the_index_replays_the_second`.
5. `a_stream_that_runs_out_at_the_bound_says_so`, and in the grid and in a drawn
   order, each at its draw.
6. `spent_attempts_are_refused_with_the_count`.
7. `a_puzzle_from_a_seed_opens_as_a_board_and_is_played_to_its_end`.
8. `each_refusal_reads_as_its_text`.
9. `a_stream_that_breaks_its_word_cannot_make_generate_panic`: draws of one and above,
   below zero and NaN.
10. `seed_zero_gives_the_pinned_puzzles`, the one test that drives `SeededStream`
    for a pin: five literals beside `GENERATION_VERSION`.
11. `a_hundred_seeds_fold_to_the_pinned_number`.
12. `a_hundred_seeds_keep_the_guarantees_in_tier_n`, one for each tier: the figures
    of step 8, printed and not asserted.
13. `api_bounds.rs`: `Tier` and `GenerateError` in both bounds tests, `Tier` in the
    serde test.

#### Changes to the list

Each made while working the list, with its reason.

- **Added** `a_draw_the_boundary_forbids_still_picks_one_of_the_n` (`choice.rs`). The
  acceptance criteria say `generate` cannot panic on any stream, and a stream of a
  caller's own may give a draw outside `[0, 1)`; the index is held to the last of `n`.
- **Added** `a_fixed_order_takes_no_draw_and_the_drawn_order_takes_them_all`
  (`order.rs`), when the four orders were put behind one function: it holds that each
  `Order` gives its own order and that only the drawn one reads the stream.
- **Added** `any_draws_seed_givens_that_agree` (`grid.rs`), a property: the hand-worked
  cases show three seedings, and "the givens never conflict" is said of all.
- **Corrected** the second hand-worked seeding in
  `scripted_draws_seed_eleven_givens_no_two_in_conflict`. The list expected 5 at (1,5)
  after a 5 had been seeded at (5,5), in the same column. The code was right and the
  expectation was wrong; the test now says why (1,5) takes 6 and (1,8) takes 9.
- **Merged** the three stream-runs-out lines of `tests/generation.rs` into
  `a_stream_that_runs_out_says_so_at_whichever_draw`: one table of five scripts read
  better than three tests of one line each. The unit tests keep one for each part.
- **Renamed** `a_puzzle_from_a_seed_opens_as_a_board_and_is_played_to_its_end` to
  `a_generated_puzzle_...`: its draws come through the fake, as the convention asks, so
  no seed is in it. The crate overview's doctest is the one that starts from a seed.
- **Renamed** the third conversion test, first named for the boundary with an `s`
  after it, to `the_refusal_converts_from_the_streams`: the `typos` hook rewrites the
  first name.
- **Struck** nothing.

#### From clause to test

Unit tests are under `crates/pawdoku/src/`, and the others in
`crates/pawdoku/tests/generation.rs`. No snapshot test is in this table.

| Clause | Test, or reason |
| --- | --- |
| `Tier`, config `tier_n_fewest_givens`, `tier_n_most_givens`, `tier_n_floor` (15) | `each_tier_says_its_figures` |
| config `generation_version` | `the_version_is_the_specifications`; both pins name it |
| config `random_version` | `seed_zero_draws_the_golden_stream` in `tests/random.rs`, before this ticket; both pins name it |
| config `seeded_givens` | `scripted_draws_seed_eleven_givens_no_two_in_conflict` |
| config `grid_attempt_limit` | `a_hundred_failed_attempts_are_a_refusal` |
| config `box_side`, `side` | `box_side_is_three_and_side_is_its_square` in `sudoku.rs`, before this ticket |
| config `step_budget`, `candidate_limit`, `catalogue_version` | Struck: the designed way's. The basic way reads none of them |
| `index_among` | `a_draw_of_zero_picks_the_first`, `the_largest_draw_picks_the_last_and_never_n`, `the_index_is_the_binary64_product_truncated` |
| `BeginGeneration`, `order_of`, `RemovalOrder` | `each_tier_has_the_order_the_paper_assigns_it` |
| `OpenFirstGridAttempt` | `scripted_draws_seed_eleven_givens_no_two_in_conflict`: the first attempt takes the stream's first draws |
| `SettleGridAttempt`, `drawn_seeding` | `scripted_draws_seed_eleven_givens_no_two_in_conflict`, `any_draws_seed_givens_that_agree` |
| `SettleGridAttempt`, a short seeding | `a_position_with_no_digit_left_ends_the_seeding_short`, for its draws and for no grid coming of it. That no search is made is read from `found_grid`, which returns before `search`: a test cannot see a search that was made and discarded |
| `SettleGridAttempt`, a verdict of none | `eleven_givens_with_no_solution_fail_the_attempt` |
| `found_grid`, a verdict of many | `of_two_solutions_the_grid_is_the_lower_row_by_row` |
| `found_grid`, `TheSolutionGridIsFull` | `a_found_grid_is_full_and_holds_every_seeded_given` |
| `FollowGridAttempt`, another attempt | `the_next_attempt_begins_after_a_failed_one` |
| `FollowGridAttempt`, refused; `GridAttemptsStayWithinTheLimit`; `EndsAreEarned`, refused | `a_hundred_failed_attempts_are_a_refusal`, `spent_attempts_are_refused_with_the_count` |
| `FollowGridAttempt`, the bound; `TheBoundIsInTheTiersRange` | `the_bound_is_drawn_within_the_tiers_range_fewest_first` |
| `BeginVisit`, `order_position` | `row_by_row_is_every_position_once_a_row_at_a_time`, `s_path_turns_at_the_end_of_each_row`, `every_other_takes_the_odd_steps_of_the_s_path_then_the_even`, `a_fixed_order_takes_no_draw_and_the_drawn_order_takes_them_all` |
| `BeginVisit`, `nth_unvisited` | `scripted_draws_give_the_order_worked_out_by_hand`, `the_drawn_order_takes_a_draw_for_every_visit_the_last_included` |
| `NoPositionIsVisitedTwice`, `VisitsAreNumberedOnce`, `RemovalIsBounded` | The three fixed-order tests, each once; `the_drawn_order_visits_each_position_once_whatever_the_draws`. A visit has no number in the code: its step is its place in the order |
| `DecideVisit`, refused by the bound; `GivensStayAtOrAboveTheBound` | `a_removal_that_would_go_below_the_bound_is_refused`; by property, `every_tier_gives_a_puzzle_that_keeps_the_guarantees`, which works the bound out again |
| `DecideVisit`, refused by the floor; `RowsAndColumnsKeepTheFloor` | `a_removal_that_would_leave_a_row_below_the_floor_is_refused`, `a_removal_that_would_leave_a_column_below_the_floor_is_refused`; by property, as above |
| `DecideVisit`, the order of its ends | `the_bound_is_read_before_the_floor` |
| `DecideVisit`, kept | `a_given_whose_removal_gives_many_solutions_stays` |
| `DecideVisit`, emptied; `EmptiedPositionsStayEmpty`, `UnemptiedPositionsKeepTheirGiven` | `a_given_whose_removal_keeps_one_solution_goes`, `removal_begins_from_the_whole_grid_and_the_first_visit_empties_its_position`: each compares the whole set of givens left |
| `FinishRemoval`; `EndsAreEarned`, finished; `EveryPositionOnce` | `every_position_is_visited_and_none_is_asked_about_twice` |
| `GivensSitInTheSolutionGrid`, `GivensLeftAreWellPosed`, `OneSolutionAndItIsTheGrid`, `TheRestrictionsHold` | `every_tier_gives_a_puzzle_that_keeps_the_guarantees`, by property; `a_hundred_seeds_keep_the_guarantees_in_tier_1` to `_5` |
| `OneSolutionAndItIsTheGrid`, handed to `SetPuzzle` | `a_generated_puzzle_opens_as_a_board_and_is_played_to_its_end` |
| `AlwaysEnds` | The two properties and the five hundred-seed tests return; `spent_attempts_are_refused_with_the_count` is the other end. Its counts of solver calls, at most one an attempt and one a visit, are read from `found_grid` and `decide`, by inspection: the solver is a function, and no test counts its calls |
| `DrawsInOrder` | `the_draws_taken_follow_from_the_grid_attempts_and_the_tier`, `any_draws_seed_givens_that_agree`, `spent_attempts_are_refused_with_the_count` |
| `AStreamThatRunsOut` | `a_stream_that_runs_out_says_so_at_whichever_draw`, `a_stream_that_runs_out_in_the_grid_says_so`, `a_stream_that_runs_out_in_a_drawn_order_says_so`, `removal_stops_at_the_first_draw_the_order_cannot_give` |
| `SameDrawsSameGivens` | `the_same_draws_give_the_same_givens`, `two_calls_on_one_stream_give_two_puzzles_and_the_index_replays_the_second`, `seed_zero_gives_the_pinned_puzzles`, `a_hundred_seeds_fold_to_the_pinned_number` |
| `OnlyTheBoundary` | Reason: the module is `no_std` and names no source of chance but `RandomStream`; the same-draws property would fail on any other |
| `ATierClaimsNothing` | Reason: nothing to run. The module has no rating, and `Tier`'s documentation says so |
| `Generating` provides `Generate(tier)` | Every test of `generate` |
| `GenerationResult` exposes | The tier is the caller's own; the status is `Ok` or `Err`; `draws_taken` is the stream's index after the call less its index before, `after.wrapping_sub(before)` since a `SeededStream`'s index wraps, in `the_draws_taken_follow_...`, which also begins one call at index 7; the givens are the proof's, in every test of `generate` |
| The refusals' text | `refusal_texts_are_stable`, `each_refusal_reads_as_its_text` |
| What cannot happen | `the_refusal_converts_from_the_solvers`, the conversion called directly |
| The public types' bounds | `public_types_are_send_sync_and_static`, `public_types_are_clone_and_debug`, `streams_and_value_types_serialise_under_the_serde_feature` |
| No panic on any stream | `a_stream_that_breaks_its_word_cannot_make_generate_panic`, `a_draw_the_boundary_forbids_still_picks_one_of_the_n` |

**The 87 obligations of `just plan-spec generation`**, by kind, with an empty
`diagnostics` array:

| Kind | Count | Where |
| --- | --- | --- |
| `config_default` | 24 | The config rows above; three struck as the designed way's |
| `rule_success`, `rule_failure`, `rule_entity_creation` | 14 | The rule rows above |
| `invariant` | 13 | The invariant rows above |
| `surface_*` | 4 | The two surface rows above |
| `enum_comparable` | 2 | `Tier` and the private `Order` derive `PartialEq`; `each_tier_has_the_order_the_paper_assigns_it` compares both |
| `transition_edge` | 9 | `Generation`: found, refused, finished, in the rule rows. `GridAttempt`: failed and found. `Visit`: its four ends, `DecideVisit`'s rows |
| `transition_rejected`, `transition_terminal` | 6 | Struck: the module excludes how a generation is stored. No status is kept, so none can be moved wrongly: `generate` returns at an end, a grid attempt is a function's result, and `Decision` is made once for a visit. The tests of the ends stand in |
| `entity_fields`, `entity_relationship`, `when_presence`, `projection`, `derived` | 15 | Struck, for the same reason: `Generation`, `GridAttempt` and `Visit` are not stored. The observable results stand in: the givens and the solution of the proof, the count of draws, and `every_position_is_visited_and_none_is_asked_about_twice` for `is_due_a_visit` and `is_fully_visited` |

#### The breaks

Every test was first seen to fail against a stub, but for those that arrived green with
code an earlier line asked for. So the code was then broken on purpose, one way at a
time, and `just test` run. Each break failed at least the tests named; the run stops at
the first failures, so later tests that would also fail are not listed.

| Break | A test that failed |
| --- | --- |
| The grid is the first solution found, not the lower | `of_two_solutions_the_grid_is_the_lower_row_by_row` |
| The index is not held to the last of `n` | `a_draw_the_boundary_forbids_still_picks_one_of_the_n` |
| The S path visits row by row | `a_fixed_order_takes_no_draw_and_the_drawn_order_takes_them_all` |
| A hundred and one grid attempts | `a_hundred_failed_attempts_are_a_refusal` |
| Removal stops when it reaches the bound | `every_position_is_visited_and_none_is_asked_about_twice` |
| The floor is off by one | The row and the column floor tests |
| The refusal's text changed | `refusal_texts_are_stable` |
| The digit's draw taken when no digit is left | `a_position_with_no_digit_left_ends_the_seeding_short` and two more |
| The bound not read before the floor | `the_bound_is_read_before_the_floor` and two more |
| The bound drawn before the grid | `the_draws_taken_follow_...`, both pins, `spent_attempts_are_refused_with_the_count` and two more |
| Emptied on many and kept on one | Seven tests of `removal.rs` |
| The drawn order skips its last draw | Five tests of `order.rs` |
| The solver's refusal not carried as its own text | `the_refusal_converts_from_the_solvers` |
| The bound counted from the most | `the_bound_is_drawn_within_the_tiers_range_fewest_first` |

### Names later tickets need

All in `pawdoku::generation`.

```rust
pub const GENERATION_VERSION: &str = "generation-1";

#[non_exhaustive]
pub enum Tier { One, Two, Three, Four, Five }
// Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash; serde under the feature.

impl Tier {
    pub const fn number(self) -> u8;            // 1 to 5
    pub const fn fewest_givens(self) -> usize;  // 51, 36, 32, 28, 22
    pub const fn most_givens(self) -> usize;    // 60, 49, 35, 31, 27
    pub const fn floor(self) -> usize;          // 5, 4, 3, 2, 0
}

#[non_exhaustive]
pub enum GenerateError {
    Stream(RandomError),                   // #[from], transparent
    GridAttemptsSpent { attempts: u32 },
    NotProved(SolveError),                 // #[from], transparent
}
// Debug, Clone, PartialEq, thiserror::Error. Not Eq: RandomError holds an f64.

pub fn generate(tier: Tier, stream: &mut dyn RandomStream) -> Result<WellPosed, GenerateError>;
```

- **The refusals' text.** `Stream` reads as the `RandomError` it carries, for the fake
  `the replay stream is exhausted at draw {index}`. `GridAttemptsSpent` reads
  `no solution grid was found in 100 grid attempts`. `NotProved` reads as the
  `SolveError` it carries, and no input reaches it.
- **The proof's shape.** Removal asks `solver::search` for each verdict, and the givens
  left go to `solver::solve` once, at the end. This is the first of the two shapes the
  ticket offers. It leaves no line that no input reaches: what cannot happen is one `?`
  into `NotProved`, and the conversion is tested directly. The other shape needs a place
  for "no proof yet" that is never empty at the end, and runs the proof constructor's
  whole check up to 81 times.
- **Both enums are `#[non_exhaustive]`**, read by inspection: `generation.rs` lines 122
  and 255.
- **The proof's constructor has one caller outside test code**, read by inspection:
  `src/solver.rs` line 198. Every other `vouch(` under `src/` is in `sudoku`'s tests and
  its test fixture.
- **`src/generation` names `crate::sudoku`, `crate::solver` and `crate::random`** and no
  other engine module, read by search; `just metrics` holds the rule.
- **A tier's order of removal is not public.** It is `Tier`'s private `order()`.
- **A count is a `usize`**, so it compares with `proof.givens().len()` as it is.
- **`Tier` has an order**, by number, so that it can key a map. It claims nothing.
- **Inside the module**, for T20: `choice::index_among(u, n)`; `order::visits(order,
  stream)`, which gives positions one at a time and is the only thing that draws
  during removal; `grid::draw_grid(stream)`; and `removal::remove(solution, visits,
  limits)`, which draws nothing and takes any order.

**Every public item beside its doctest.**

| Item | Its example asserts |
| --- | --- |
| The module | A puzzle from seed 7 in tier 5 has at least the tier's fewest givens, and each is the solution's digit |
| `GENERATION_VERSION` | It is `"generation-1"` |
| `Tier` | Tier 2's number, range and floor |
| `Tier::number` | 1 and 5 |
| `Tier::fewest_givens` | 22 for tier 5 |
| `Tier::most_givens` | 27 for tier 5, and how a result is held against its range |
| `Tier::floor` | 5 and 0 |
| `GenerateError` | A script of five draws is refused with the boundary's error and reads as its text |
| `generate` | A puzzle is made and set; a second call on the stream begins at index 23 and gives another; the seed from index zero gives the first again |
| The crate overview | A puzzle from a seed is opened as a board, played one cell and checked |

### Snapshots taken

Six files under `crates/pawdoku/tests/snapshots/`, each read before it was staged. No
earlier snapshot changed.

- `snapshots__tests__snapshot_the_puzzle_seed_zero_gives_in_tier_1.snap` to `..._5.snap`:
  the givens as a grid, the solution as a grid, and the count of givens beside the
  tier's range. They hold solutions on purpose.
- `snapshots__tests__snapshot_the_generators_refusals.snap`: the `Display` text of the
  three refusals, each made by hand.

### What the yardstick gives

Seeds 0 to 99, each a fresh `SeededStream`, in each tier, from
`a_hundred_seeds_keep_the_guarantees_in_tier_1` to `_5`, which assert the guarantees and
print these figures. Read on 2026-10-03 with `NEXTEST_SUCCESS_OUTPUT=immediate just test`.

| Tier | Fewest givens | Most givens | Within the tier's range | Most grid attempts |
| --- | --- | --- | --- | --- |
| 1 | 51 | 60 | 100 of 100 | 1 |
| 2 | 36 | 49 | 100 of 100 | 1 |
| 3 | 32 | 35 | 100 of 100 | 1 |
| 4 | 28 | 31 | 100 of 100 | 1 |
| 5 | 22 | 28 | 98 of 100 | 1 |

- **No grid attempt failed.** Every seed took 104 draws in tiers 1 and 2 and 23 in tiers
  3 to 5, which is one attempt. Three hundred further seeds, tried while looking for the
  two-solution case, gave no short seeding and no seeding without a solution either.
  Nothing here is a reason to move `grid_attempt_limit`.
- **The source paper** reports every attempt succeeding for its level 5 under row order;
  here 98 of 100 met the range, and the two that did not ended at 28 givens.
- **Of the 300 further seeds, 13** had the solver find the higher of two grids first, so
  the rule for which solution is the grid decides about one grid in twenty-three.

### What was verified, and how

On 2026-10-03, in the supplied Supacode worktree, from `19473d7`.

- **Before any edit.** `just check` ended
  `All checks passed and the worktree is unchanged.` `just test`, warm: 250 tests in
  0.86 s by nextest's summary, 2.1 s in all.
- **`just plan-spec generation`**: 87 obligations and an empty `diagnostics` array.
- **`just fmt-check`**, **`just clippy`**: exit 0, no warning.
- **`just metrics`**: `Quality score: 100.0% (702 functions analyzed)`, no finding.
- **`just test`**: `309 tests run: 309 passed, 0 skipped`, then 121 doctests and the one
  `compile_fail`. Warm: 1.0 s by nextest's summary, 2.4 s in all, 0.3 s more than
  before. The slowest new test takes 0.4 s.
- **`just snapshots-check`**: `no unreferenced snapshots found`, `no snapshots to review`.
- **`just wasm-check`**, **`just features`**: exit 0, both targets and both feature sets.
- **`just coverage`**: total lines 99.83 per cent. `generation.rs` 100, `choice.rs` 100,
  `order.rs` 100, `removal.rs` 100, `grid.rs` 99.56: its one missed line is a test's own
  `panic!` for givens that turn out not to have two solutions.
- **`just doc`**, **`just check-docs`**: exit 0; `Validated 42 pages and 43 canonical topics.`
- **`just check`**: `All checks passed and the worktree is unchanged.`
- **The paper**, fetched once with the maintainer's leave and read beside the module.

### Deviations, and why

- **The branch is `ticket/T32-basic-generator`**, as Supacode named it, and not
  `ticket/t32-basic-generator`.
- **The loops were by part, not by line.** Each part's tests were written first and seen
  to fail against a stub that compiled, then its code written: the three fixed orders
  one at a time, then the choice, removal (the bound and the verdict, then the floor),
  the drawn order, the tiers and the bound, the grid, and `generate`. Within a part
  several lines went red and green together. The breaks above cover what arrived green.
- **Seven tests drive `SeededStream`, not one.** The two pins, and the five that step 8
  asks for over seeds 0 to 99. `docs/reference/testing.md` names all seven.
- **`index_among` holds its result to the last of `n`.** The specification says a draw
  is in `[0, 1)` and does not say what a draw outside it chooses; the trait cannot stop
  a caller's own stream giving one, and this ticket says `generate` cannot panic. For
  every draw the specification speaks of, nothing changes.
- **The one `#[expect]` names three lints**, on the one expression that turns a draw
  into an index: `n` goes to `f64` and the product comes back.
- **`tests/generation.rs` is a `#[cfg(test)] mod tests`**, as `tests/snapshots.rs` is,
  so that clippy's leave to `unwrap` in tests reaches its helpers.
- **The figures of step 8 were read with `NEXTEST_SUCCESS_OUTPUT=immediate just test`**,
  which makes nextest show what a passing test prints. No recipe was added.
- **A temporary test** tried 300 seeds for a seeding whose higher solution is found
  first. Its givens are now a literal in `grid.rs`, and the temporary test is gone.
- **`crates/pawdoku/README.md`** gained a clause: the page lists what the crate holds.
- **Nothing was pushed.**

- **Paths T33 also edits.** This ticket edited `CHANGELOG.md` and
  `docs/project/repository-map.md`, which T33 edits too. As this ticket's Open points
  say, each edit is a row or a sentence, and whichever pull request merges second
  merges `main` first.
- **A Codex adversarial review was run on 2026-10-03**, at the maintainer's request,
  over the two commits, and answered in a third. It found no defect in the code and
  seven in the tests and the words; all seven were taken.
  1. The guarantees property unwrapped a puzzle, and a script of a hundred short
     seedings is refused. It now takes that refusal as a right end and no other.
  2. No test can see how often the solver is asked, so "a short seeding is not searched"
     and the call counts of `AlwaysEnds` were claimed and not proved. The table above
     and `docs/reference/testing.md` now say they are read from the code. Making them
     testable would put the solver behind a parameter, which decision 0014 decided
     against.
  3. The stream-runs-out tests read the fake's index, which does not move on a second
     asking. `a_stream_that_runs_out_says_so_at_whichever_draw` now counts the askings,
     and `removal_stops_at_the_first_draw_the_order_cannot_give` counts the visits asked
     for.
  4. The fixed orders were held at their ends and turns only. Each is now compared, all
     81 steps, with the order written out in the test a row at a time.
  5. `Tier`'s documentation said the count of givens is bounded within the range. It
     is the bound that is drawn within it.
  6. `generate`'s documentation said a stream already drawn from gives another puzzle.
     It gives other draws, and a stream that repeats itself gives the same puzzle.
  7. `draws_taken` is counted from where the stream stood, not from zero. The table
     says so, `generate`'s documentation says how to read it, and the test begins one
     call at index 7.

  With it, the second assertion of `Tier::most_givens`'s doctest, which could not fail,
  was replaced by one that can.

- **Copilot's review of pull request 37**, on 2026-10-03, recommended approval with
  one finding, which was taken: a `SeededStream`'s index wraps past `u64::MAX`, so the
  draws a call took are `after.wrapping_sub(before)` and not a plain subtraction.
  `generate`'s documentation and the `GenerationResult` row above now say so. Nothing
  in the code changed: `generate` reads no index.

### Handed back

Triggers this ticket meets, none built here.

- **S08's control arm.** Removal in a drawn order now exists in Rust, as tiers 1 and 2,
  with a bound and a floor and without `Rate`. `removal::remove` takes any order and any
  limits, so a control with no bound and no floor is `Limits { bound: 0, floor: 0 }`.
- **T15 to T17**, the bindings and the command line. `generate` is the first entry that
  needs the randomness boundary carried across: it takes a `&mut dyn RandomStream`, and
  a caller that wants replay records the stream's index beside the seed.
- **T14**, the mutation job. The fourteen breaks above are a hand-made sample of it.
- **S03's benchmark recipe.** One `generate` for each tier is a natural benchmark; a
  hundred of them take 0.1 s to 0.4 s under the test profile.
- **T20.** The designed way can reuse `grid::draw_grid` for its solution grid and
  `choice::index_among` for its draws as they are. `removal::remove` is one position at
  a time and accepts on the verdict alone, so the designed way reuses its shape, an
  order handed in and a decision for each visit, and not the function.
- **`grid_attempt_limit`.** T31 said this ticket's measurement may move it. No attempt
  failed in 800 seeds, so 100 is far above what is used and nothing argues for another.
- **`human-solving.allium`'s `floor(u*n)`**, as T31 handed back: `index_among` here is
  the binary64 reading, private to `generation`. If that module chooses the other, the
  two differ.

## Open points

- **The tiers' names in Rust.** A tier is a number in the specification and a variant
  needs a name. The ticket suggests the numbers spelt out, with a method that gives the
  number. The maintainer may want others; the behaviour is fixed.
- **Whether `Tier` serialises.** A caller that wants to replay a puzzle records the
  tier, the seed, the two versions, the figure for grid attempts if it is the caller's,
  and the index the stream stood at unless every puzzle has a fresh stream. The ticket
  suggests that `Tier` derives the `serde` traits under the existing feature, as
  `Position` and `Given` do, and that nothing else here does.
- **What `generate` returns beside the proof.** The ticket returns the proof alone: the
  givens, the solution, and through `Tier` whether the range was met. The bound that
  was drawn and the count of grid attempts are not returned. A comparison may later ask
  for them; that is its ticket's.
- **Speed.** No figure is promised. If `just test` slows by more than a few seconds,
  say so in the hand-back notes with the test that costs it.
- **Paths T33 also edits.** T33, the API reference on GitHub Pages, may be open beside
  this ticket and edits `CHANGELOG.md` and `docs/project/repository-map.md`. T33's Open
  points record the deviation from CONVENTIONS.md §11: each shared edit is a row or a
  sentence, and whichever pull request merges second merges `main` first.
