---
id: T32
title: "The basic generator in Rust: the solution grid, removal in order and the five tiers"
status: open
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

### The test list

### Names later tickets need

### Snapshots taken

### What the yardstick gives

### What was verified, and how

### Deviations, and why

### Handed back

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
