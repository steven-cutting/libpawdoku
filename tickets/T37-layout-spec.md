---
id: T37
title: "Specification: a layout filled by search, and the run without guessing that checks it"
status: open
depends_on: [T32]
parallel_with: []
branch: ticket/t37-layout-spec
estimated_size: L
---

# T37: Specification: a layout filled by search, and the run without guessing that checks it

## Context

The engine makes a puzzle one way. `generation.allium`'s basic way, built as
`pawdoku::generation` by T32, draws a solution grid from eleven givens and removes givens
one position at a time while the solver's verdict stays one. It asks nothing of how the
puzzle is solved. The designed way beside it, which accepts a candidate on a
`reach.allium` `Rate` run within a technique contract, is a skeleton: it waits on
`technique` and `reach` in Rust, which do not exist, and on the spikes S06 to S08.

On 2026-10-05 the maintainer asked for a second published method beside the basic
generator, from Nishikawa and Toda's paper on strategy-solvable Sudoku clues, so that the
generators can be compared: follow the paper closely; do not build its CSP encoding;
instead "let the algorithm explore clue removals and test via solver rather than encode
everything in one giant SAT formula", that is, "use search/optimization instead of exact
CSP to satisfy the strategy constraints". This ticket specifies it; T38 and T39 build it.

**The source.** Kohei Nishikawa and Takahisa Toda, "Exact Method for Generating
Strategy-Solvable Sudoku Clues", Algorithms 2020, 13(7), 171, and arXiv 2005.14098 (v1,
28 May 2020), read on 2026-10-05 in the HTML rendering at
<https://ar5iv.labs.arxiv.org/html/2005.14098>; the record is
<https://arxiv.org/abs/2005.14098>. Everything this ticket needs from it is written out
below, so the paper need not be fetched; fetching it is a network action and separately
authorised. The paper's words are strategy, clue,
clue position and clue cell; this project's are technique, given and position, and, new
here, layout.

**What the paper does, and what it does not.** Its one method is exact. The problem it
names the strategy-solvable Sudoku clues problem, SSC, is: given a set of clue positions
and a set of strategies, decide whether digits exist for those positions such that the
strategies alone complete the grid, and give them. It encodes an instance as a constraint
satisfaction problem over a bounded number of solving steps and hands it to a CSP solver
(Sugar over MiniSat), which either finds the digits or proves that none exist. The paper
proposes no search method. The search it describes is the one it beats, the
generate-and-test program of Zama and Sasano (2016, in Japanese, known only through this
paper), "which repeats the followings until the test is passed or the number of trials
exceeds a predetermined limit. (1) Generate clues in specified positions. (2) Test whether
all the other cells are completed using specified strategies only." It names three
drawbacks of that method: "the lower the density of solutions (i.e. strategy-solvable
clues) over the whole search space becomes, the harder it becomes to find a solution";
"if a set of clue positions happens to be not strategy-solvable, it is unable to
recognize it no matter how much time passes"; and "even for a strategy-solvable set of
clue positions, there is no guarantee for being able to find a solution in a finite
amount of time". Of the generation step it says only that "since the test step can be
done quickly, the key is to devise a criterion for the generation step so that the
number of trials is as small as possible", and does not say what Zama and Sasano's
criterion is.

So this ticket specifies the paper's problem, its framework and its three strategies,
solved by the generate-and-test search the paper describes and not by its encoding. The
maintainer chose that on 2026-10-05, shown the difference between the two shapes:
positions in and digits out, which is the paper's problem and what its experiments
measure, against removal from a full grid with a technique test in place of the verdict,
which is the basic way's shape and not the paper's. Where the paper is under-specified
the module says which reading is its own, as `generation.allium` does for its source.

**Decided by the maintainer on 2026-10-05.**

1. **Positions in, digits out.** The asking is a layout, the set of positions that will
   hold givens, and a set of techniques. Each trial draws a solution grid through the
   randomness boundary, keeps its digits at the layout's positions and puts those givens
   to the test; trials repeat until one passes or a limit is reached.
2. **Any non-empty subset of the paper's three strategies**: naked single, hidden single
   and locked candidates, which in `technique.allium` are `naked_single`,
   `hidden_single`, and `pointing` with `claiming`. The paper itself runs with all three
   (its Section 7.2) and with naked singles alone (its Section 7.3).
3. **The test is a run without guessing, specified now** against `technique.allium`'s
   definitions of those techniques, and built now, rather than waiting for `reach` in
   Rust. When `reach`'s `Rate` exists the run is a candidate to be replaced by it.
4. **A module of its own**, not a third way inside `generation.allium`. Decision 0015
   said a second published method "would ask whether `generation.allium` is still one
   module"; the answer is that it is not.

**Two choices of the ticket writer's that go past those words**, each an Open point
below: the run is a public surface, `Check(givens, repertoire)`, and not a private part
of the filling, because it is the direct way to classify the basic generator's puzzles by
the techniques that solve them, which is the comparison the maintainer asked for, and
because it checks the study puzzles of `docs/explanation/puzzle-design.md`; and the
module imports `generation.allium` for its grid attempt instead of restating it, as the
basic way reaches the solver through black boxes pinned to `solver.allium`.

**The paper's framework (its Section 3), and what each part becomes here.**

| Paper | Here |
| --- | --- |
| A state is a pair of functions: the digit placed in each cell, 0 for none, and the set of candidates of each cell. Every state satisfies S1, no cell has an empty candidate set, and S2, a digit is not a candidate of a cell that holds another digit or whose peer holds that digit: the normal form. | `technique.allium`'s `Grid`, `GridCell.digit` and `GridCell.candidates`. S2 is `Grid.is_kept`: no candidate stands that a placed peer rules out. S1 is `not is_contradictory`, which `DeductionsAreSound` keeps true of a true grid. |
| An initial state places the clue cells, leaves the others open, and rules out exactly the candidates S2's antecedent names, no other. | A grid born kept on the givens: `technique.allium`'s `LayOutGrid`, "every other cell allows what the start does not rule out". |
| A transition applies strategies. What is placed stays placed and what is ruled out stays ruled out. And once a transition leaves the candidates unchanged, the state never changes again. | A step: one licensed deduction whose technique is in the repertoire, taken, with upkeep after a placement so the grid is kept again. A run that finds nothing licensed is stalled and ends there. |
| A final state has a digit in every cell; its Corollary 1, every final state is a Sudoku solution. | A run is solved when the grid is filled; by `DeductionsAreSound` the filled grid is the one solution of the givens. |
| Definition 3.1: an initial state is strategy-solvable when a sequence of transitions reaches a final state. Definition 3.2, SSC: decide whether some assignment of digits to the clue positions is strategy-solvable. | Solvable by a repertoire: the run from the givens ends solved. A filling: find givens for a layout that are solvable by the repertoire. |
| Proposition 2: a sequence whose every transition rules out a candidate has at most 648 transitions, since 729 candidates stand at most and 81 at least; so a bound of 649 steps suffices for the encoding. | The reason the run always ends: each step places a digit or strikes a candidate that stood, and a grid holds at most 81 of the one and 729 of the other. No step bound is a figure of this module. |
| The transition applies every applicable strategy at once and asks only whether the candidates changed. | One deduction at a time. For a repertoire that holds `hidden_single`, or holds neither `pointing` nor `claiming`, the end is the same whatever the order: a deduction licensed on a grid is still licensed, or its effect already holds, after any other sound step. The paper's two settings both qualify. This is stated as a guarantee with its reason and its condition, and the implementation chooses which licensed deduction to take, the same way each time, as `reach.allium` allows. |
| The CSP encoding (its Section 4), the step bound K and the incremental Algorithm 1 (its Section 5), and the application to the fewest strategy-solvable clues (its Section 6). | Not carried. |
| Generate-and-test, Zama and Sasano's, as the paper describes it. | Carried as the search. Each trial draws a solution grid as `generation.allium`'s grid attempt does, reads its digits at the layout, and puts them to the run. The paper says nothing of a criterion for the generation step; independent trials, each from a fresh grid attempt, are this module's own reading. |
| The experiments' time limit of 600 seconds. | A limit on trials, a count and never a clock. |
| The exact method recognises an unsolvable set of positions. | The search never does. A refusal by spent trials says nothing of the layout, and the module says so as a guarantee. |
| Equivalent transformations of a grid between trials (the basic generator's source calls this Operator 5). | Not carried, and named in Excludes; whether a trial may transform a grid is an open question of the module. |

**The strategies (its Section 2.2) and the catalogue.** The paper's definitions, in its
words, and what licenses each here. Rows and columns are numbered from 0 in the paper and
from 1 here; "group" is unit, "block" is box.

| Paper | Here |
| --- | --- |
| "A naked single is a strategy that places n in (i,j) if no other candidate but n remains at (i,j)." | `naked_single`, rank 3: `GridCell.is_naked_single`, one candidate. |
| "A hidden single is a strategy that places n in (i,j) if there is a group G having (i,j) such that no cell in G minus (i,j) has n as a candidate." | `hidden_single`, rank 4: `Unit.hides_single(d)`, in a row, a column or a box, read from candidates. `cross_hatch`, rank 2, is the same placement read by sight and is not named here: on a kept grid every cross-hatch is a hidden single. `full_unit`, rank 1, is a naked single and a hidden single both, on a kept grid, and is not named either. |
| "Let A, B be groups such that the intersection of A and B has 3 cells. A locked candidate is a strategy that rules out n over all cells in one difference set B minus A if no cell in the other difference set A minus B has n as a candidate." | `pointing`, rank 5, where A is the box and B the line, and `claiming`, rank 6, where A is the line and B the box: `Unit.points(d)` and `Unit.claims(d)`. The paper's one definition covers both directions. |

One reading here is the module's own, and the module says so: the catalogue's `points`
and `claims` require two or more places for a digit still to be placed in the unit, and a
place outside to strike, where the paper's definition has no such clause. When
`hidden_single` is in the repertoire the end is the same on a kept, true grid: with one
place the deduction is the hidden single, which places the digit, and upkeep then
strikes what the paper's locked candidate would have struck; a placed digit has no
places under S2; and a strike with nothing to strike changes no state. Without
`hidden_single` the paper's locked candidate strikes where the catalogue's does not,
and the Open point on which repertoires are allowed is where that is settled. The
catalogue's definitions are followed as they stand, so that a `Rate` run under a
profile holding the same four techniques makes the same deductions.

The paper's formulation of a naked single (its Formula 8) also places when no candidate
remains, which it notes leads to a contradiction and "no substantial problem". It cannot
arise here: every candidate set of givens is read from a solution grid, so the grid is
true and `DeductionsAreSound` keeps S1.

**The paper's measurements**, for whoever compares; T39 measures what can be measured
beside them.

- Of Royle's collection of 49,151 Sudokus with 17 givens, 37,373 are solvable by the
  three strategies, and none by naked singles alone.
- For 4 by 4 Sudoku, every arrangement of 3 and of 4 clue positions was checked against
  brute force: none with 3 is strategy-solvable, exactly 704 with 4 are, and all 704 are
  solvable by naked singles alone. This cannot be reproduced here: the engine's grid is
  9 by 9 (`crates/pawdoku/src/sudoku.rs`, `Grid<T>` is `[[T; 9]; 9]`).
- Its Section 7.2: 100 sets of positions, each made by choosing n from 20 to 79 at random
  and then n distinct cells at random, all confirmed strategy-solvable; by count of
  positions, 19 in 20 to 29, 30 in 30 to 39, 20 in 40 to 49, 10 in 50 to 59, 11 in 60 to
  69 and 10 in 70 to 79. Both methods allowed the three strategies; the limit was 600
  seconds. The exact method solved 95, all but the sparsest within about one minute, and
  every unsolved instance had 25 or fewer positions. Zama and Sasano's generate-and-test
  solved 65, "much faster even in near 20 cells", and none with 50 or more positions.
- Its Section 7.3: the positions of 30 Sudokus drawn at random from Royle's collection,
  digits forgotten, under naked singles alone. Within several hours the exact method
  ended for 14 of the 30, each proved not strategy-solvable, in 852 to 26,835 seconds
  (T39's table reads 849; see its Open points); the other 16 did not end. The 30 are
  printed in the paper's Tables 2 and 3, and T39 carries them.

**Facts that shape the work**, each verified on 2026-10-05 against `main` at `f580c02`.

- **No technique machinery exists in Rust.** `crates/pawdoku/src/` holds `sudoku`,
  `solver`, `board`, `generation` and `random`; nothing names a technique, a deduction
  or a profile beyond the solver's own singles propagation. T35, the ordered list of
  deductions S06 drafted, and T20, the designed way's triggers, are drafts in spike
  notes and not files. So the run this module states is new behaviour with no owning
  module, and it is stated in full, as entities and rules, not as a black box alone.
- **`Rate` is not the test.** `reach.allium`'s `Rate(givens, profile)` requires well-posed
  givens (its rule at line 177 requires `solution_count = 1`), and whether a candidate is
  well-posed is not known before the test; a solved run shows it. The run here is stated
  so that it needs no such premise, and the module says, in an open question, what it
  would take for the run to become a `Rate` run once `reach` exists in Rust.
- **The solver cannot be the test either.** `solver.allium` guesses when propagation
  stalls (`GuessesAreTheLastResort`), and a verdict "explains nothing". Solvable by a
  repertoire is a stronger claim than well-posed, and a weaker tool than the solver
  decides it.
- **Soundness is the catalogue's.** `technique.allium`'s `DeductionsAreSound` (line 870)
  says of a grid whose start is part of a solution, whose placed digits are that
  solution's and whose every open cell allows the solution's digit, that every licensed
  deduction places the solution's digit or strikes only digits the solution does not use
  there. Every candidate here is read from a solution grid, so every run starts true and
  stays true; that is what makes a solved run's grid the one solution.
- **The four techniques are monotone, on one condition.** `is_naked_single` needs one
  candidate; `hides_single` needs one place; `points` and `claims` need two or more
  places in one line or box and a place outside. After any sound step, candidates only
  shrink and placements only grow, so the one candidate or the one place stays
  (soundness keeps the solution's digit), or the digit is placed and the deduction is
  moot; a place outside is struck or still stands; and places that lay in one line
  still do, or the digit is placed, or they shrink to one. That last case is the
  condition: one place is a hidden single, and only `hidden_single` takes it. With
  `hidden_single` in the repertoire the set of placements a run can reach does not
  depend on the order deductions are taken in, and neither does whether it ends solved
  or stalled; the same holds of a repertoire with neither locked candidate, where no
  deduction loses its licence. Without `hidden_single` and with a locked candidate it
  does not: a strike from another box can take a box's places for a digit from two to
  one, and the pointing that was licensed before is licensed no more, so the order
  decides whether its strike was made. The paper's two settings, all three strategies
  and naked singles alone, both qualify. `reach.allium`'s open question on monotonicity
  (line 423) is about the whole catalogue and stays open; this is a claim about four
  techniques.
- **The grid attempt is `generation.allium`'s.** `SettleGridAttempt` and
  `FollowGridAttempt` draw eleven givens, two draws each, put them to the solver, take
  the lower of two solutions row by row, count failed attempts against
  `config.grid_attempt_limit` and refuse when it is spent; a full seeding takes 22 draws
  and a short one `2k + 1`. In Rust it is `generation::grid::draw_grid`, which T32's
  notes say T20 may reuse as it is; it is `pub(super)` in a private `mod grid;` today,
  and a sibling module needs a visibility change to the `grid` module and `draw_grid`,
  T39's only edit to `generation`. Both rules are keyed to a `Generation` with a tier,
  and `FollowGridAttempt`'s found branch draws the bound, a 23rd draw, and moves the
  generation to `removing`, so pinning them by name does not by itself give a trial of
  22 draws and no tier; the Open point on the grid attempt asks how to settle that.
- **`generation.allium` says two ways.** Its Scope opens "The module states two ways of
  doing it, side by side"; `docs/how-to/work-with-the-specs.md` line 28 and
  `docs/explanation/layering.md` lines 36 to 46 say the same and that nothing imports
  it. Decision 0015's "What would reopen this" names "a second published method worth
  keeping beside this one, which would ask whether `generation.allium` is still one
  module".
- **Words already taken.** Pattern is `technique.allium`'s, for the cells and digits a
  deduction is read from; geometry is `human-solving.allium`'s, for the board; level is
  the research report's and band is `sudoku.allium`'s; clue and strategy are the paper's
  and are used only in the Source note. Repertoire is `technique.allium`'s word for the
  set of techniques a profile knows, and is taken as it is. Run, step, solved and
  stalled are `reach.allium`'s, and `docs/project/terminology.md` says each model has a
  run of its own; this module's run is another. Layout is free: `puzzle-design.md`
  says "given layout" for the arrangement of givens, and no module uses the word. The
  proposed word for the paper's set of clue positions is **layout**; the first Open point
  asks the maintainer. Trial and check are not free: trial is banned as a synonym for
  guess by `terminology.md`, `solver.allium` and `lapse.allium`, and check is
  `board.allium`'s `Check` and `lapse.allium`'s; the Open point "Trial and check are
  words already taken" asks the maintainer.
- **Seventeen is `sudoku.allium`'s figure.** `SetPuzzle` requires fewer than 81 givens
  and exactly one solution, and says "the fewest givens that can pass is 17, which is a
  fact about this rule". A layout of fewer positions can never be filled, and a layout of
  81 leaves nothing to play; whether the asking refuses both is an Open point.
- **Replay has four names.** `generation.allium` states replay over the draws taken,
  for one implementation of the solver, under `generation_version`, `random_version`
  and the config. A filling's givens depend on the grid attempt, so on
  `generation_version`; on the four techniques' definitions, so on `catalogue_version`;
  on the stream, so on `random_version`; and on this module, so on a version of its
  own. `human-solving.allium`'s `ExactReplay` is the precedent for naming several.
- **The metrics probe mirrors the layering table.** `rustqual.toml` carries one
  `[[architecture.pattern]]` per module, and `just metrics` requires the fixture under
  `tests/fixtures/metrics-violation/src/` to fire every rule exactly once (the recipe
  compares the rules named with the rules fired). A new module needs a rule and a stub,
  whether or not its Rust exists yet. Odd files there break their rule with a `use`
  line and even files with an inline path; `layout.rs` is the eleventh.
- **`puzzle-design.md` cites `generation.allium` by line number** in its closing table,
  thirteen times. Any edit above an open question moves those lines; T31's step 9 is
  the procedure.
- **`.tools/` may be absent in the worktree.** `just check-specs`, `analyse-specs`,
  `plan-spec` and `metrics` need `.tools/bin/allium` and `.tools/bin/rustqual`, which
  `just install-allium` and `just install-tools` fetch; both use the network and this
  ticket authorises them once.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `.agents/skills/spec-change/SKILL.md`
and `.agents/skills/tend/SKILL.md`; `tickets/T31-basic-generator-spec.md` whole, as the
shape of a specification ticket and its hand-back; `docs/specs/generation.allium`,
`docs/specs/solver.allium` and `docs/specs/sudoku.allium` in full; `docs/specs/technique.allium`
lines 284 to 520 (the grid, the units and the predicates that license the first twelve
techniques), 715 to 838 (`LayOutGrid` and the grid's invariants) and 870 to 882
(`DeductionsAreSound`); `docs/specs/reach.allium`, its header, `OpenGrid` and
`KeepMarksTrue` (lines 195 to 240) and its `Rating` surface (lines 340 to 372);
`docs/decisions/0015-basic-generator.md`; `docs/decisions/README.md` ("Writing a new
one"); `docs/explanation/puzzle-design.md` lines 271 to 283 and the technique-contract
row of its closing table; `docs/explanation/layering.md`; `rustqual.toml` and
`tests/fixtures/metrics-violation/src/lib.rs`.

## Goal

`docs/specs/layout.allium` states the method in full: what is asked, how a trial draws
its grid and reads the layout, how the run takes licensed deductions of the repertoire
until the grid is filled or nothing is licensed, when a filling finishes and when it is
refused, what comes back, and what is guaranteed of it. A reader can tell this way of
finding givens from `generation.allium`'s two, and every open question of the designed
way still stands. Decision 0018 records the module and the maintainer's decisions. Every
page that lists the modules lists ten. Both specification gates end
`10 specifications, no diagnostics and no findings.` with no waiver, `just metrics`
fires the new rule once, and `just check` is green.

## Non-goals

- No Rust under `crates/`. T38 builds the run and T39 the filling.
- Nothing of the designed way: no symmetry, no ceiling, no required or forbidden set,
  no shortcut suppression, no pacing, no redundant-givens floor. T20 writes those.
- No open question of the designed way answered. Where this module has an answer of its
  own for its own asking, the question in `generation.allium` gains a clause saying so
  and keeps asking for the designed way, as T31 did for the basic way.
- No CSP, no SAT, no encoding, no step bound K, no fewest-givens application.
- No criterion for the generation step beyond a fresh grid attempt, and no
  transformation of a grid between trials.
- No rating, no grading and no claim that a layout's puzzles are harder than another's.
- No edit to `technique.allium`, `reach.allium` or `solver.allium`: the run is stated
  against their clauses as they stand. If a clause is found wanting, stop and hand it
  back.
- No edit to a done ticket, to S06, S07, S08 or S11, or to the T20 and T35 drafts.

## Files touched

| Path | Change |
| --- | --- |
| `docs/specs/layout.allium` | New: the module, steps 3 to 7 |
| `docs/specs/generation.allium` | One sentence in Scope naming the third way and where it lives; a clause on two open questions (step 8); under (b) of the Open point on the grid attempt, the grid attempt split out of `Generation` |
| `docs/decisions/0018-layout-by-search.md` | New decision record (step 9); the number is the next free one on the day |
| `docs/decisions/README.md` | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | Entry for the record. Frozen under CONVENTIONS.md §11; see Open points |
| `docs/how-to/work-with-the-specs.md` | A row for `layout.allium`; the `generation.allium` row no longer says nothing imports it; any count of modules |
| `docs/explanation/specifications.md` | The paragraph at 71 to 84 gains the tenth module; "The ninth" stays true of `generation.allium` |
| `docs/explanation/layering.md` | A row for `layout`; `layout` added to the "Imported by" cells of `sudoku`, `solver`, `technique` and `generation`; the prose at 36 to 46, which says nothing imports `generation` |
| `docs/README.md` | The sentence at 23 to 26. Frozen under CONVENTIONS.md §11; see Open points |
| `docs/project/terminology.md` | Rows for layout, filling, trial, and the run as this module uses the word |
| `docs/explanation/puzzle-design.md` | Line numbers in the closing table that cite `generation.allium`; the technique-contract row's "Where it lands" cell gains `layout.allium`; nothing else |
| `rustqual.toml` | The rule `layout_imports_sudoku_solver_technique_generation`; `"crate::layout"` in each of the ten existing rules; "all nine" to "all ten" in the comment (step 10) |
| `tests/fixtures/metrics-violation/src/layout.rs` | New: the stub that fires the rule once |
| `tests/fixtures/metrics-violation/src/lib.rs` | `pub mod layout;` and "the other nine" to "the other ten" in its comment |
| `CHANGELOG.md` | Under Unreleased: the module specified; the count of decision records |
| `tickets/README.md` | T37's index row set `done` |
| `tickets/T37-layout-spec.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t37-layout-spec` from `main` after T32 has merged
   (README.md "How to pick up a ticket"). Run `just initialize` if `.pixi/` is absent
   and `just install-allium` and `just install-tools` if `.tools/bin/allium` or
   `.tools/bin/rustqual` is; each uses the network and this ticket authorises it. Run
   `just check-specs`, `just analyse-specs` and `just metrics` before any edit and quote
   their closing lines.

2. **Put the Open points to the maintainer before writing a clause.** Each has a proposed
   answer. They are product decisions and the specification is where they are recorded,
   so do not pick one silently. Record each answer, with its date, under "Open points
   settled". The word the maintainer chooses for the layout renames the module, its
   file, its Rust module and every path in this ticket; the words chosen for the trial
   and the check rename their rules, surfaces, vocabulary and terminology rows likewise.

3. **The header.** Follow the `spec-change` and `tend` skills, in the shape of
   `generation.allium`'s header.

   - **Scope.** How a setter that is a program fills a layout: a set of positions, given
     by the caller, that will hold givens, and a repertoire of techniques; digits are
     found by trials, each a solution grid drawn through the randomness boundary and
     read at the layout, and a trial passes when a run without guessing, taking only the
     repertoire's deductions, fills the grid. Say that this is the problem Nishikawa and
     Toda state and that the method is the generate-and-test search their paper
     describes, not their exact one; say in a line what the exact method can do that
     this cannot. Say that `generation.allium` states two other ways and that none of
     the three is built on another; this module imports `generation.allium` for its
     grid attempt alone.
   - **Source.** The paper, as the Context gives it: title, authors, journal and arXiv
     id, the address and the date read. Then the two tables of Context in a line each:
     what of the framework and the strategies is carried, what is replaced, and what is
     not carried, the reading that is this module's own included.
   - **Includes.** The layout and what is asked of it; the trial; the run and its two
     ends; the limits (the trial limit and, through the import, the grid attempt limit;
     counts, never a clock); replay over the four names; the check as an asking of its
     own.
   - **Excludes.** Rating, hints and any claim of difficulty; deciding well-posedness,
     which a solved run shows and the solver proves; any claim that a layout cannot be
     filled; any criterion for the generation step, and any transformation of a grid
     between trials; the exact method; a required or forbidden technique, a ceiling and
     anything else of the designed way's contract; the techniques past the four;
     symmetry, which a caller puts into the layout it hands over; where puzzles are
     made; chance beyond the boundary; how a grid, a run or a trial is stored.
   - **Dependencies.** `sudoku.allium` for Position and Given and what a setter may
     pose; `solver.allium` for the proof of a finished filling; `technique.allium` for
     the grid, its cells and units, the catalogue and licensed deductions;
     `generation.allium` for the grid attempt and its draws.
   - **Vocabulary.** The words of the four modules stand. To them: layout; filling, one
     asking of this module, and fill, its verb; trial; repertoire as `technique.allium`
     has it, here a non-empty set of the four; run, step, solved and stalled as
     `reach.allium` has them, this module's run being its own and no model's; check, the
     asking of a run on its own; technique contract, which is here the first of the two
     shapes `generation.allium` asks about, a repertoire and the run ends solved. Say
     that clue and strategy are the paper's words and are used nowhere else.

4. **The config.** `fewest_positions`, 17, with a comment that it is `sudoku.allium`'s
   figure and McGuire, Tugemann and Civario's theorem, and that it is not a caller's to
   change; `trial_limit`, as the Open point settles it, with a comment on what a trial
   is and that the figure is a count; `layout_version`, `"layout-1"`, with a comment in
   the shape of `generation_version`'s: raised whenever anything changes, in this
   module or in the run, so that the same draws give other givens or a filling takes
   another number of trials; and the three names restated as the precedent restates
   them, `catalogue_version` from `technique`, `generation_version` and `random_version`
   from `generation`, with a sentence on what each one covers for a filling. Run
   `just check-specs`.

5. **The run.** Write it with the `tend` skill at the level `reach.allium` is written,
   and run `just check-specs` after each block. The module must say, in whatever
   constructs fit:

   - **What is asked.** Givens and a repertoire. The givens are refused when two share a
     position, when two peers hold one digit, or when there are 81; the repertoire when
     it is empty or names a technique outside the four. Say what a refusal at the asking
     is and that it takes no draw and opens no grid.
   - **The grid.** Opened born kept on the givens, as `technique/Grid.created` with this
     run as keeper, `uniqueness_promised` false, exactly as `reach.allium`'s `OpenGrid`
     does. Nothing here promises uniqueness: no uniqueness technique is in the four.
   - **A step.** While the grid is unfilled and some deduction of `grid.deductions` has a
     technique in the repertoire, one of them is taken. Which one is the implementation's
     and the same each time, as `reach.allium` says of tied deductions; the end does not
     depend on it (below). A placement puts the digit in its cell, leaves that cell with
     the digit as its one candidate, and then upkeep strikes every candidate the placed
     digit rules out of its peers, so the grid is kept before the next look; state this
     as `reach.allium`'s `KeepMarksTrue` states it, as this run's own rule, since
     `KeepMarksTrue` keeps only a `reach` run's grid. A strike takes the deduction's
     strikes out of their cells' candidates. Each step is numbered from 1 within its
     run.
   - **The end.** Solved when the grid is filled; stalled when it is unfilled and no
     deduction of the repertoire is licensed. Both are final. Nothing is guessed and
     nothing is put back.
   - **What comes back.** Solved or stalled; the count of steps; the digits placed, so a
     caller can see how far a stalled run got.

   And it must guarantee, each as a named invariant or guarantee, with its reason in
   the comment:

   - every run ends, because a step places a digit or strikes a candidate that stood,
     and a grid holds at most 81 of the one and 729 of the other (the paper's
     Proposition 2);
   - a run of givens read from a solution grid never places another digit than that
     solution's and never strikes that solution's digit, by `DeductionsAreSound`;
   - a solved run's givens are well-posed and the filled grid is their one solution, by
     the same guarantee and the paper's Corollary 1, so the solver's proof may be taken
     of them and `sudoku.allium`'s `SetPuzzle` accepts them;
   - whether a run ends solved or stalled, and which digits it places, does not depend
     on which licensed deduction is taken first, for a repertoire that holds
     `hidden_single` or holds neither `pointing` nor `claiming`, by the monotonicity
     argument in Context, stated in the comment for each of the four techniques; for any
     other repertoire the Open point on repertoires either removes it from the asking
     or the module says that the end may depend on the order and only the same-input
     guarantee stands;
   - the same givens and the same repertoire give the same steps and the same end;
   - stalled says the four techniques, as the catalogue licenses them, do not fill this
     grid from these givens; it says nothing of other techniques and nothing of whether
     the givens are well-posed (`reach.allium`'s `StalledIsNotUnsolvable`, restated for
     this run);
   - a run rates nothing: solved under a repertoire is a fact about four techniques, not
     a difficulty.

   Then one surface for the asking, `Checking`, providing `Check(givens, repertoire)`
   and carrying the guarantees, and one for what comes back, `CheckResult`, in the shape
   of `generation.allium`'s `Generating` and `GenerationResult`. Whether `Checking` is
   public is the Open point; if the maintainer keeps the run private, `Checking` is
   still the surface a trial asks through, and the module says no other caller reaches
   it.

6. **The filling.** The module must say:

   - **What is asked.** A layout, a repertoire and a stream of draws. The layout is a set
     of positions, each on the grid, each once; it is refused with fewer than
     `config.fewest_positions` positions or with 81, as the Open point settles, and the
     repertoire as step 5 says. A refusal at the asking takes no draw.
   - **A trial.** Numbered from 1 within its filling. It draws a solution grid as
     `generation.allium`'s grid attempts do, attempt after attempt until one finds a
     grid or `generation/config.grid_attempt_limit` have failed; pin a black box to
     `SettleGridAttempt` and `FollowGridAttempt` by name, with the draws each attempt
     takes, as `generation.allium` pins `verdict_is_one` to the solver, and as the Open
     point on the grid attempt settles: no tier and no bound draw. Spent grid
     attempts refuse the filling, with nothing to show, as they refuse the basic way's
     generation. A grid found is read at the layout: the candidate givens are the grid's
     digit at each of the layout's positions, and nothing else. The givens go to a run
     under the repertoire. Solved passes the trial; stalled fails it.
   - **The ends.** A passed trial finishes the filling with its givens and its grid. A
     failed trial opens the next, unless `config.trial_limit` trials have failed, when
     the filling is refused by trials with the count and, as the Open point settles,
     nothing else. One rule and its ends, in the shape of `FollowGridAttempt`, since
     `allium analyse` refuses two rules that write one status.
   - **What comes back.** The status; the count of trials; the draws taken, counted as
     `generation.allium` counts them; and, of a finished filling, the givens and the
     solution grid.
   - **The draws.** Every draw a filling takes is a grid attempt's, through
     `generation.allium`'s `draw_at` and `index_among`, which this module reads through
     the import and does not restate; the run takes none, and no draw chooses anything
     of this module's own. So a filling whose every trial finds its grid at once takes
     22 draws a trial, as the Open point on the grid attempt settles.

   And it must guarantee, each as a named invariant or guarantee:

   - every filling ends, finished or refused: trials are counted against
     `config.trial_limit`, grid attempts against `generation/config.grid_attempt_limit`,
     and every run ends;
   - the givens of a finished filling sit at exactly the layout's positions, one each,
     and each is the solution grid's digit there;
   - they are solvable by the repertoire, hence well-posed, and the one solution is the
     grid, so they may be handed to `SetPuzzle` as they are;
   - a refusal by trials says nothing of the layout: another stream, or more trials,
     may fill it, and the module never says a layout cannot be filled (the paper's
     second drawback, stated as a limit of this method);
   - no trial is numbered twice and a trial's grid is never reused;
   - the draws are taken in the order the grid attempts take them and no others are
     taken, so the count depends on the grid attempts alone;
   - a stream that runs out ends the filling with the boundary's own refusal, as
     `generation.allium`'s `AStreamThatRunsOut` says;
   - the same layout, the same repertoire, the same config, the same four versions and
     the same draws give the same status, the same givens and the same count of trials,
     for one implementation of the solver, and a seed stands for the draws exactly when
     the stream begins at index zero;
   - no draw is taken from anywhere but the boundary;
   - a filling rates nothing, and a layout claims nothing of difficulty.

   Then the surfaces `Filling`, providing `Fill(layout, repertoire)` with the guarantees,
   and `FillResult`, in the shape of `Generating` and `GenerationResult`.

7. **The open questions.** The module carries at most three, each a product decision:
   whether the run becomes a `Rate` run under a profile holding the four techniques
   once `reach` exists in Rust, and what then of `Rate`'s premise that the givens are
   well-posed, which a candidate does not yet have; whether a trial may transform the
   grid it drew, by relabelling digits or permuting rows within a band, columns within a
   stack, bands or stacks, to put another candidate to the run without another search,
   which the paper's source for the basic generator calls equivalent propagation and
   which would make a trial cheaper and the draws fewer; and whether a stalled trial
   should report how far its run got, which is the raw material for the criterion the
   paper says Zama and Sasano devised. Write nothing that answers any of them.

8. **`generation.allium`.** Add one sentence to its Scope, after "neither is built on the
   other", naming the third way and `layout.allium`, and saying it imports this module
   for its grid attempt. Then, as T31 did for the basic way, add a clause to two open
   questions and leave their text asking for the designed way: the technique contract's
   shape (the question at line 833), where this module's answer for its own asking is a
   repertoire of four techniques and a run that ends solved, no ceiling, no required
   set and no forbidden set; and what a spent budget reports (the question at line
   841), where this module's answer is a refusal with the count of trials and nothing to
   show, the inner work bounded by the grid attempt limit and by the run's own end. Do
   not touch the other five.

9. **The decision record**, in the shape of 0015: frontmatter with
   `canonical_for: [decision_layout_by_search]`; the title "Decision 0018: A second
   published method, by search, in a module of its own". Use the next free number on the
   day and rename the file, the title and the index row if 0018 is taken.

   - **Context:** what the maintainer asked for and why; what the paper's method is and
     that it proposes no search; that the designed way still waits on two modules and
     three spikes; that decision 0015 said a second method would reopen the module
     question.
   - **Decision:** the four decisions of Context, the two choices of the ticket writer's
     as the maintainer settled them, and each answer to the Open points.
   - **Consequences, the ones that hurt included:** two modules now say how a setter
     finds givens, and a reader must know which does what; the catalogue's four
     techniques are implemented in the run's Rust, and again when `technique` lands, so
     a change to a definition moves both and `catalogue_version` names it; the search
     can never say a layout is unfillable, so a refusal by trials is weaker than the
     paper's answer and the comparison with the paper's figures is indicative only; the
     grid attempt is shared, so a change to it is a new `generation_version` and moves
     this module's puzzles too; replay names four versions; the pin that chooses which
     deduction a run takes is the implementation's and is a new `layout_version` when
     it changes, though the end does not.
   - **What would reopen this:** `reach` in Rust, which would make the run a `Rate`
     run or show why not; a comparison that shows the search never fills a dense layout
     within any budget a caller can afford, which would ask for the exact method and the
     dependency it needs; a consumer that wants the fewest-givens application; T20
     finding the designed way can take a layout from this module.
   - **Related pages:** Puzzle design objectives, Layering, Specifications, decisions
     0012 and 0015.

   The docs validator refuses a page without its manifest entry, so the record, its
   entry in `docs/manifest.yml` (audience `contributor`, `maintainer` and `agent`) and
   its row in `docs/decisions/README.md` go in one commit. Move the index's "next
   decision" sentence on by one.

10. **The layering rule and its stub.** In `rustqual.toml`, after the `generation` rule,
    add `layout_imports_sudoku_solver_technique_generation`, forbidding
    `crate::reach`, `crate::effort`, `crate::lapse`, `crate::human_solving` and
    `crate::board` in `src/layout{.rs,/**}`, with the reason in the shape of its
    neighbours. `reach` is forbidden until the open question of step 7 is answered the
    other way; say so in the reason. Nothing may import `layout`, so add
    `"crate::layout"` to the `forbid_path_prefix` of each of the ten existing rules, and
    change the comment above them that says `random`'s rule "forbids all nine" to "all
    ten". Add `tests/fixtures/metrics-violation/src/layout.rs` in the shape of
    `generation.rs` there, breaking the rule once with a `use` line naming `effort`,
    `pub mod layout;` in that `lib.rs`, and its comment's "the other nine" changed to
    "the other ten". Run `just metrics` and quote the probe's closing lines.

11. **The pages.** Each stays within what `docs/manifest.yml` says it owns. The three
    that describe the specifications (`specifications.md`, `work-with-the-specs.md` and
    `docs/README.md`) say what the module states and make no claim about what is built.
    `layering.md` adds the row and the four "Imported by" cells, and its prose says
    `generation` is now imported by `layout`, for the grid attempt alone, and that
    `layout` is a leaf, so the claim under "Why the direction matters" still holds; its
    sentence that the Rust module is not yet built is T38's to change. `terminology.md`
    gains the rows step 3 settled. `puzzle-design.md`: print the text at every
    `generation.allium` line its closing table cites and correct each number that moved,
    and add `layout.allium` to the "Where it lands" cell of the technique-contract row,
    in one clause; change nothing else on the page. Search `docs/`, `README.md`,
    `AGENTS.md` and `crates/pawdoku/README.md` for "two ways", "nine" of the modules and
    "nothing imports it", and record each further hit under Deviations.

12. `CHANGELOG.md`, under Unreleased: `layout.allium` specified, in two or three lines;
    the count of decision records.

13. Run `just check-specs`, `just analyse-specs`, `just plan-spec layout`,
    `just check-docs`, `just metrics` and `just check`. Quote the closing lines of each
    and the count of obligations `plan-spec` prints, which T38 and T39 start from. Fill
    in the hand-back notes, with "Names T38 and T39 need" in the shape of T31's "Names
    T32 needs": the words, the enumerations, the entities, the rules, the black boxes,
    the invariants, the surfaces and the config figures. Set `status: done` here and in
    `tickets/README.md`, commit on the ticket branch with short imperative subjects,
    and stop before pushing.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` each end
  `10 specifications, no diagnostics and no findings.`, and no `allium-ignore` directive
  exists under `docs/specs/`.
- `layout.allium` states everything steps 5 and 6 list, and each guarantee there is a
  named invariant or guarantee with its reason in a comment.
- The words clue and strategy appear in the module only in its Source note; level,
  difficulty and band appear nowhere in it; pattern appears only as `technique.allium`
  uses it.
- The module's `use` lines name `sudoku`, `solver`, `technique` and `generation` and
  no other module, and nothing it states names `reach`.
- The module carries at most the three open questions of step 7.
- `generation.allium` keeps seven `open question` blocks, two of which carry a clause
  for this module; the text that asks for the designed way is still there in each; its
  rules, invariants, surfaces and config are unchanged, unless the maintainer chose (b)
  in the Open point on the grid attempt; `generation_version` is unchanged, because no
  draw gives other givens.
- `rustqual.toml` names eleven rules, every rule but `layout`'s own forbids
  `crate::layout`, and `just metrics` ends with the probe fired once for each.
- No page under `docs/` says the engine finds givens in two ways only or that nothing
  imports `generation`, decision records of their day excepted.
- Every `generation.allium` line cited in `puzzle-design.md` holds the text its row
  describes.
- The decision record exists, is in the manifest and the index, and states the four
  decisions, the two choices and each settled Open point.
- `just plan-spec layout` prints obligations, and their count is in the hand-back notes.
- `just check` is green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just check-specs
just analyse-specs
just plan-spec layout
just metrics
rg -n '^open question' docs/specs/layout.allium docs/specs/generation.allium | cut -c1-80
rg -n '^use ' docs/specs/layout.allium
rg -n -w -i 'clue|strategy|strategies|level|difficulty|band' docs/specs/layout.allium
rg -n -i 'two ways|nothing imports' docs README.md crates/pawdoku/README.md
rg -n 'generation\.allium:[0-9]+' docs/explanation/puzzle-design.md
rg -c '"crate::layout"' rustqual.toml
just check-docs
just check
git status --porcelain
```

Expected: two runs ending `10 specifications, no diagnostics and no findings.`; a list of
obligations with an empty `diagnostics` array; the probe fired once for eleven rules; at
most three open questions in `layout.allium` and seven in `generation.allium`; four `use`
lines; the paper's words only in the Source note's lines; "two ways" only where a page
says `generation.allium` states two and "nothing imports" only of leaves; each cited
line checked by hand; `10`, one `"crate::layout"` for each rule but `layout`'s own; the
documents gate green with one more page than before;
`All checks passed and the worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Names T38 and T39 need

### Handed back

### Open points settled

## Open points

Each is the maintainer's. The proposal beside it is the ticket writer's, made on
2026-10-05; the grid attempt's three choices and the point on trial and check were added
on 2026-10-06, after a review of the ticket.

- **The word for the paper's set of clue positions, and the module's name.** Proposed:
  layout, and `docs/specs/layout.allium`, with `pawdoku::layout` to follow. Pattern is
  `technique.allium`'s and geometry is `human-solving.allium`'s, so neither is free;
  arrangement and placement are words the pages already use of other things.
- **Whether the run is a public surface.** Proposed: yes, `Check(givens, repertoire)`,
  for the three reasons in Context, and because a private run leaves T38 a module
  nothing calls until T39. The other choice keeps the same run behind `Fill` alone and
  drops the comparison of the basic generator's puzzles by repertoire from T39.
- **Whether the module imports `generation.allium` for the grid attempt.** Proposed:
  yes, so the eleven-given draw has one statement and one Rust function. But pinning a
  black box to `SettleGridAttempt` and `FollowGridAttempt` by name is not enough as they
  stand: both are keyed to a `Generation` with a tier, and `FollowGridAttempt`'s found
  branch draws the bound, a 23rd draw, and moves the generation to `removing`. So a
  tier-less trial of 22 draws cannot be had from them while `generation.allium`'s rules
  stay unchanged. Three choices. (a) Restate a tier-less grid attempt in `layout.allium`
  and let T39 copy `draw_grid` or call it, two statements of one draw. (b) Split
  `generation.allium`'s grid attempt out of `Generation` into an entity of its own that
  both modules use; this relaxes the acceptance that its rules are unchanged, and grows
  its row in Files touched and step 8, though `generation_version` stays, because no
  draw changes. (c) Pin a black box to the seeding and the settling alone, and say in
  prose that the bound draw is not taken. The Rust already takes 22 draws a trial
  whichever is chosen: `draw_grid(stream)` (`generation/grid.rs:85`) takes no tier, and
  the bound is drawn by `draw_bound` in `generate` (`generation.rs:352`), so no choice
  asks for a refactor and (c) is the cheapest in the specification. The ticket writer
  proposes (b), as the one statement and one function this point argues for, with (c)
  named as the cheapest; the maintainer decides.
- **Trial and check are words already taken.** Trial is banned as a synonym for guess:
  `docs/project/terminology.md` line 49 ("Never trial or assumption"),
  `solver.allium` line 58 and `lapse.allium` line 67, and `layout.allium` imports
  `solver.allium`. Check is `board.allium`'s `Check`, the player's question about one
  cell, built as `board::Check`, and `lapse.allium`'s check of the marks, so
  `layout::check` would be a third meaning. Attempt is no way out: it is
  `generation.allium`'s grid attempt and `human-solving.allium`'s attempt. No proposal:
  naming is the maintainer's. T38 and T39 already read their words from T37's hand-back
  notes.
- **Whether `trial_limit` is the caller's or a figure in config.** Proposed: the
  caller's, with no default, as the designed way's `step_budget` and `candidate_limit`
  are, because the paper's own limit is a clock the engine cannot carry, so no figure is
  the paper's, and because the comparison T39 makes varies it. The other choice is a
  fixed figure like `grid_attempt_limit`, with 100 suggested and no measurement behind
  it, which keeps `Fill` to a layout, a repertoire and a stream.
- **Whether the asking refuses a layout of fewer than 17 positions or of 81.** Proposed:
  yes, both, with a refusal of its own that takes no draw. Neither can ever be filled:
  17 is the fewest givens any well-posed puzzle has, and 81 leaves nothing to play. The
  paper's problem takes any set and its method answers no; this search would spend
  every trial to say nothing.
- **Whether pointing and claiming are chosen apart or only together.** Proposed: apart,
  as the catalogue names them; a caller wanting the paper's locked candidates names both.
  The other choice is an enumeration of the paper's three, with locked candidates one
  thing.
- **Which repertoires are allowed.** Proposed: any non-empty set of the four in which
  `pointing` or `claiming` appears only beside `hidden_single`. Then the end of a run
  never depends on the order deductions are taken in, and the catalogue's locked
  candidates make the same deductions as the paper's, as Context argues; the paper's
  two settings are both allowed, and what is refused is a set the paper never runs.
  The other choice allows every non-empty set and scopes the guarantee, so that a
  caller who asks for naked singles with pointing is told the end may depend on the
  order.
- **What a refusal by trials shows.** Proposed: the count of trials and nothing else.
  The other choice reports the last trial's givens and how far its run got, which is
  the designed way's question of "the best candidate so far" and is not answered here.
- **Whether a check refuses givens that conflict.** Proposed: yes, since a grid born
  kept on them would be false and `DeductionsAreSound` promises nothing of it; the
  refusal names the position. The other choice opens the grid anyway and lets the run
  stall or contradict.
- **The two frozen files.** `docs/manifest.yml` and `docs/README.md` are frozen under
  CONVENTIONS.md §11. T31 edited both with the maintainer's leave. This ticket lists
  both; if leave is not given for `docs/README.md`, hand that edit back as a `main`
  follow-up and say so under Deviations.
- **The decision record.** It is in scope because decision 0015 said this method would
  reopen the module question, and the answer wants a record. The maintainer may strike
  it as more than a minimal ask; the Source note in the module then carries the reason
  alone.
- **The ids.** T37 to T39 are taken because T34 is claimed by S10's follow-up on its
  branch, T35 by S06's draft, and T36 conditionally by S11's amendments, which say they
  "lift out whole as a ticket of their own under T36" if the maintainer lands the order
  alone. If the maintainer folds those amendments into T35, these three may be
  renumbered T36 to T38 before they are picked up.
- **Paths T35 also edits.** T35, the ordered list of deductions, is drafted to edit
  `docs/explanation/puzzle-design.md` and `technique.allium`, and S11's amendments add
  pages to it. CONVENTIONS.md §11 forbids two open lanes sharing a path. This ticket
  follows the deviation T31 recorded for T33: each shared edit is a row, a cell or a
  sentence, and whichever pull request merges second merges `main` first.
