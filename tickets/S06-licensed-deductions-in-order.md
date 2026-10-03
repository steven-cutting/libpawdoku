---
id: S06
title: "Spike: one ordered list of licensed deductions, and a run that takes the first"
status: done
depends_on: [T19]
parallel_with: []
branch: ticket/s06-licensed-deductions-in-order
estimated_size: M
---

# S06: Spike: one ordered list of licensed deductions, and a run that takes the first

## Context

`docs/specs/technique.allium` is the ground four models of a player stand on, none of
which imports another: `reach.allium` has a limit hide a deduction, `effort.allium` has
it make one dear, `lapse.allium` has it lead a player into guessing, and
`human-solving.allium` has it change what is noticed, kept in mind and got wrong. Each
has its own run and its own entry (`Rate` and `NextStep`, `Price`, `Tackle`, `Simulate`
and `Assess`), and `reach`, `effort` and `lapse` each state `OpenGrid`, `Look` and
`TakeStep` again. No solver, catalogue, rating or generator exists in code:
`crates/pawdoku/src` holds `random.rs` and nothing of play. So the question of whether
four models are needed is cheap to ask now.

The proposal, as the maintainer made it on 2026-09-29 and in this project's words:

> Given a grid (its givens, its placed digits and its candidates) and a profile, state
> every deduction the grid licenses that the profile sees, the same list every time. A
> run is then played by listing them lowest on the ladder first, then by the cells they
> touch, and taking the first. Whatever the profile, the step taken is the lowest the
> profile sees, so the same grid and the same profile give the same run. If nothing is
> wrong with this, it is a simpler alternative to the four models of a player. What has
> to be shown is that the list is never too long and never too dear to draw up.

Facts to start from, each verified at execution against the specifications as they
stand on `main`:

- **Most of it is already stated.** `SameGridSameDeductions` in `technique.allium` says
  the same digits and candidates license the same deductions every time, "with the
  same uniqueness premise and catalogue limits". So the list is drawn from more than
  the grid and the profile: `uniqueness_promised`, `max_chain_links` and
  `max_forcing_paths` change it too, and the proposal's "same grid and same profile"
  has to name all five. `Order`'s
  `systematic` is "the lowest technique on the ladder that yields anything, and back to
  the bottom after every step". `Profile.sees` is `knows`, `holds`, `takes_in` and
  `can_read` together. `reach.allium`'s `SameInputSameRun` gives the same run for the
  same input. What is new is the list as a thing a caller can read, a total order over
  it, and the claim that the other three models can go.
- **Which of equal deductions is taken is not stated.** `technique.allium` excludes "the
  order in which equal deductions are listed" and `reach.allium` leaves "which of
  several equally preferred deductions is taken" to the implementation. The proposal
  states it, which is a change to both modules.
- **The grid holds candidates.** A deduction places or strikes and never both
  (`ADeductionPlacesOrStrikes`), and ranks 5 to 25 and 28 only strike. A state of
  givens and placed digits alone has nowhere to keep a strike, so the same strike would
  be first on the list for ever. The state is `technique.allium`'s `Grid`. It is not
  `human-solving.allium`'s sheet, which that module says is not the grid.
- **The ladder is a convention.** `technique.allium` calls it "a conventional comparison
  order, not a cognitive law", and its open question on loads and ladder order says
  ranks 13 to 29 are provisional.
- **Lowest first is not shown to be the easiest route.** `reach.allium`'s open questions
  on monotonicity and on the path-dependence of the hardest step, and
  `ExtraCandidatesHideAndNeverMislead` in `technique.allium`, say that past subsets a
  strike one order takes can change the shape another pattern needed.
- **Chains are bounded and repeat themselves.** `max_chain_links` is 24 and
  `max_forcing_paths` is 9. Many witnesses can rest the same strike on different
  proofs, so a list of witnesses can be long where a list of what is placed or struck
  is short.
- **Words.** A deduction is licensed, a step is a deduction taken, and `lapse.allium`
  already uses move for a deduction or a guess. This ticket says deduction and step and
  never move. Strike, never eliminate. Given, never clue.

Read first: `tickets/CONVENTIONS.md` §11; `docs/specs/technique.allium` whole;
`docs/specs/reach.allium`, `docs/specs/effort.allium` and `docs/specs/lapse.allium`,
their headers, rules and guarantees; `docs/specs/human-solving.allium`, its header, its
`Projection` contract and its open questions; `docs/explanation/human-solving.md`;
`docs/explanation/solving-sudoku.md`; `docs/explanation/layering.md`.

## Goal

A recommendation on the ordered list and the run that takes its first entry: adopt it
in place of the four models, adopt it beneath them as the one thing they share, or not
at all. With it: the list's measured size and cost on ranks 1 to 12, a bound argued on
paper for ranks 13 to 29, and for each of the four models a statement of what it
reports that the list cannot. Nothing is installed or changed outside this file.

The starting recommendation: **adopt it beneath the models**. The list and its total
order are `technique.allium`'s, `reach.allium`'s systematic run becomes "take the
first", and `effort`, `lapse` and `human-solving` keep what only they say (a price, a
guess and its repair, chance) while losing their own statements of looking.

| Model | Entry | What it reports | Kept, moved or lost |
| --- | --- | --- | --- |
| `reach` | `Rate`, `NextStep` | how far, by which techniques, the next step | |
| `effort` | `Price` | what each step costs, escalation | |
| `lapse` | `Tackle` | lapsed marks, checks, guesses, repairs | |
| `human-solving` | `Simulate`, `Assess` | attempts drawn by chance, an assessment | |

## Non-goals

- Rust in `crates/pawdoku`, a dependency, a recipe or a workflow.
- Editing a specification. The recommendation is proposed text in the hand-back notes;
  the change itself is a follow-up made through the `spec-change` skill.
- Undoing a deduction, which is S07's, and generation, which is S08's.
- Calibrating a profile or settling the ladder past rank 12.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S06-licensed-deductions-in-order.md` | ticket | the evidence, the recommendation, the drafted follow-up; `status: done` |
| `tickets/README.md` | ticket index | S06's row set to `done` |

## Steps

1. Create the worktree on `ticket/s06-licensed-deductions-in-order` from `main`
   (README.md "How to pick up a ticket").

2. Verify, read-only, and record with sources and dates: (a) which guarantees, rules
   and derived values already state the list and the run, module by module, and which
   sentences the proposal would change; (b) for each field of `Profile` (`repertoire`,
   `capacity`, `spans`, `marking`, `order`, `fixation`, `upkeep`, `budget`, `fatigue`,
   `patience`), whether it hides entries from the list, orders the list, or has no
   place in it, and so which model it stays with; (c) how published solvers that rate
   by the lowest technique first order equal deductions, from their own documentation;
   (d) whether a run that always takes the lowest deduction ends with the lowest
   possible hardest step, argued from the specifications or refuted by a grid that
   shows otherwise; (e) whether a SAT or SMT solver can check, in a prototype, that
   a deduction is sound and that a verdict is one, and which one, by what licence.
   Fetching documentation and installing a SAT or SMT solver are separately
   authorised: stop and ask.

3. State the list and what it is drawn from: the grid's digits and candidates,
   `uniqueness_promised`, `max_chain_links`, `max_forcing_paths` and the profile. An
   entry is a technique, what it places or strikes, and one witness. Say which
   witness is kept when several rest the same placement or strikes on different
   proofs, and state the total order: rank, then what, then what, until no two
   entries are equal. Repeats are removed only within one technique: "one placement
   may be licensed by several techniques, and then each has its own deduction", so
   the list keeps one entry for each technique and output, never one for each output.
   It is still smaller than the catalogue's deductions, which `technique.allium` also
   tells apart by proof, so it is a second thing beside them and not the same; step 4
   reports the size of both.

4. Build a prototype in `ai_tmp/` or the session's scratch directory, never in a
   commit, for ranks 1 to 12, under one profile pinned in full, because `Profile.sees`
   hides by every field it reads: a repertoire of exactly ranks 1 to 12, `capacity` 4
   (the highest load among them, so that no subset is hidden), `full_marks`, every
   extent in `spans`, order `systematic` and fixation `unfixed`. Keep marks as
   `reach.allium`'s `KeepMarksTrue` keeps them: whatever a placed peer rules out is
   struck before the next look. Run it from the
   givens to the end over a stated set of grids, the four study puzzles of
   `docs/explanation/puzzle-design.md` among them, which that page marks unverified.
   Before running it, define the counters, per technique, so that another prototype
   would count the same: at the least, the patterns examined (each combination of
   cells, units and digits a technique's search tests, licensed or not) and the
   witnesses found before the rule of step 3 removes repeats. Then count, per look:
   the licensed deductions, one for each proof; the entries on the list, one for each
   technique and output; the entries when listing stops at the first rank that yields
   anything; and each counter for all three. Counts and never seconds.
   Before any count is used, check that the list is complete: on a stated sample of
   grids, compare it entry for entry with an independent exhaustive search written
   apart from the prototype, one that tries every combination of units and digits
   each technique of ranks 1 to 12 can be read across. Repeating the same answer
   twice shows only that the prototype is consistent; an empty list would pass.

5. Bound the worst case on paper, not only the grids measured. For ranks 1 to 12, the
   patterns a look can examine at most, counted as step 4 counts them, by unit and by
   subset size up to four, which the size of the grid fixes; and include among step
   4's grids some states with many candidates, early in runs from the sparsest puzzles
   in the set, so that the measured figures are compared with that bound. For ranks
   13 to 29, bound it twice over. First the list: where the count of
   witnesses grows with `max_chain_links` and `max_forcing_paths`, and what the rule
   of step 3 brings it down to. Then the search: the patterns examined per look,
   counted as step 4 counts them, as a function of the same two limits, since a chain
   search may reject a great many paths to find a short list. Where the second bound
   cannot be argued, the recommendation speaks for ranks 1 to 12 alone.

6. Complete the table above, and write the verdict against four criteria: the list is
   the same every time; it is short enough to draw up at every look; nothing a consumer
   is owed is lost; the specifications get shorter.

7. Draft the follow-up in the hand-back notes: the proposed text for
   `technique.allium` and for each model that changes, and a build ticket under the
   next free id on the day, found with `rg -n 'T2[0-9]|S0[5-9]' tickets/`. Record
   as well what S07 needs to rebuild the prototype, because `ai_tmp/` and the scratch
   directory belong to this worktree and go no further: the list and its inputs, the
   total order, the rule for repeated witnesses, the counters, the grids measured and
   each grid's counts, so that a rebuild can be checked against them.

8. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The total order is stated so that no two entries are equal, and the prototype's run
  twice over the same grid, uniqueness premise, catalogue limits and profile gives the
  same steps.
- The counters of step 4 are defined before any figure is recorded, their counts are
  in a table by rank, the worst-case bound of step 5 for ranks 1 to 12 is argued and
  compared with them, and both bounds for ranks 13 to 29 are argued or the
  recommendation is limited to ranks 1 to 12.
- The independent search of step 4 agreed with the prototype's list entry for entry
  on every sampled grid, or each difference is listed and explained.
- The hand-back notes carry everything step 7 names for a rebuild.
- The table of models is completed, and the verdict is adopt in place, adopt beneath,
  or not at all, with the reason.
- The follow-up names every specification change as a change to make first.
- `git status --porcelain` on the ticket branch lists only this file and the index.

## Verification

```sh
rg -n 'SameGridSameDeductions|ExtraCandidatesHideAndNeverMislead' docs/specs/technique.allium
rg -n 'SameInputSameRun|LimitsOnlyHide' docs/specs/reach.allium
rg -n '^rule (OpenGrid|Look|TakeStep)' docs/specs/
ls crates/pawdoku/src
git status --porcelain
```

Expected: both guarantees of `technique.allium`; both of `reach.allium`; the three
rules in `reach.allium`, `effort.allium` and `lapse.allium`; `lib.rs` and `random.rs`
and no other module; two lines naming this file and `tickets/README.md`.

## Hand-back notes

### What was verified, and how

Run on 2026-10-03 (UTC) in the Supacode worktree for this ticket, on branch
`ticket/S06-licensed-deductions-in-order` (see Deviations), from `main` at `b4eecc2`. No
tracked file changed except this one and `tickets/README.md`. Every script, download and
output went under the gitignored `ai_tmp/s06/` of this worktree and goes no further;
"What S07 needs for a rebuild" below carries what a rebuild is checked against.

The maintainer authorised on the day: read-only fetches of solver documentation, source
and licence pages and of public puzzle collections; and the install of a SAT solver into
a throwaway venv under `ai_tmp/s06/venv` (`python-sat` 1.9.dev15 and `six` 1.17.0 from
PyPI, on the pixi Python 3.14.7), never into pixi or a manifest. The maintainer also
chose the grid set (the study puzzles, puzzles derived from them, and fetched public
sets) and asked for the work to be split between separate agents, so that the search
of step 4 was written apart from the prototype.

The worktree before any step:

```text
$ git rev-parse --short HEAD
b4eecc2
$ git status --porcelain
```

How the work was divided. The definitions of step 3 and the counters of step 4 were
written first, into one scratch file, before any program existed. Seven agents then
worked apart: one built the prototype; one, given only that file and
`docs/specs/technique.allium` L284-515 and forbidden to read the prototype, wrote the
exhaustive search; one assembled the grids; one built the SAT oracle and answered (e);
one fetched the solvers' documentation and source for (c); one argued the bounds of
step 5; one read the specifications for (a) and (b). The comparison, the measurements
and the experiment of (d) were then run over their outputs. Two agents reading one
specification is as far apart as one session gets; it is not two teams.

**Step 1.** The worktree existed before the ticket ran (Deviations).

**Step 2. The five questions.** The specifications were read on 2026-10-03 at `b4eecc2`;
`technique:254` means `docs/specs/technique.allium` line 254, and `reach`, `effort`,
`lapse`, `human-solving`, `solver` and `generation` likewise. External sources were
fetched on 2026-10-03; "docs" marks a project's own documentation and "source" its code
or licence file.

**(a) Most of the list and the run is stated already; the proposal changes five
sentences, the comments on `chosen`, and one line in each of three rules.**

Already stated, module by module:

- `technique.allium`:
  - the deductions of a grid, "one deduction for each of the following and no other,
    each once, and the same ones whenever the grid is the same" (`technique:313-316`,
    `technique:342`);
  - `SameGridSameDeductions` (`technique:891-894`), which names the uniqueness premise
    and the catalogue limits beside the digits and candidates;
  - a strict ladder (`RanksAreDistinct`, `technique:802-806`);
  - `systematic` as "the lowest technique on the ladder that yields anything, and back
    to the bottom after every step" (`technique:110-113`);
  - `Profile.sees` (`technique:247-254`).
- `reach.allium`:
  - sight and the lowest seen rank (`sees`, `is_in_order`, `reach:103-113`);
  - fixation within those in order (`is_preferred`, `reach:129-135`);
  - the taking, `let taken = chosen(filter(run.grid.deductions, x => run.is_preferred(x)))`
    (`reach:238`);
  - `SameInputSameRun` (`reach:365-367`) and `LimitsOnlyHide` (`reach:369-372`).
- `lapse.allium` repeats reach's `sees`, `is_in_order`, `is_preferred` and the `let
  taken` line word for word, comments apart (`lapse:128-145`, `lapse:288`; checked with
  `diff`). The `is_favoured` that `is_preferred` reads is lapse's own (`lapse:177-200`).
- `effort.allium` has the same sight as `is_within` (`effort:93-98`) and takes the
  cheapest eligible deduction (`is_cheapest`, `effort:110-112`, `effort:185`).
- `human-solving.allium` states no list and no run of this kind: "It keeps no
  technique/Grid" (`human-solving:45-48`).

Not stated anywhere, and so new: any order among deductions (`grid.deductions` is read
only with `any`, `all`, `count` and `filter`); the list a profile sees as a value a
caller can read; and one entry for each technique and output.

For a profile with order `systematic` and fixation `unfixed`, `is_preferred` holds of
exactly the seen deductions of the lowest seen rank, and the first of a list ordered by
rank first is one of them. So "take the first" is one of the choices `chosen` already
permits there: it contradicts nothing and fixes the one thing `reach:41-43` leaves
free. (Inference from `reach:117-135`.)

Sentences the proposal changes:

| Where | Sentence now | What changes |
| --- | --- | --- |
| `technique:55-56` | "How a grid is stored, how deductions are found quickly, and the order in which equal deductions are listed." | The last clause goes. |
| `technique:313-342`, `technique:846-916` | no order and no entry | Gains the order, the entry and the rule for repeated witnesses. An addition. |
| `reach:41-43` | "Which of several equally preferred deductions is taken. It must be the same each time and is otherwise the implementation's, as the order of tied branches is in solver.allium." | Goes. |
| `reach:238`, `reach:236-237`, `reach:244-246` | `chosen(filter(...))`, "chosen of nothing is nothing", and the comment on `chosen` | The first, in the stated order, of those preferred; the comments say so of `first`. |
| `reach:429` | "the implementation's choice among tied deductions" | The tie is no longer the implementation's; the question of other orders stands. |
| `effort:44` | "Chance, and which of several equally cheap deductions is taken." | The second half goes: the first, in the stated order, of the cheapest. |
| `effort:185`, `effort:180`, `effort:191-193` | `chosen(filter(... is_cheapest ...))` and two comments on `chosen` | As reach's line and comments. `is_cheapest` itself stays (see the table of models). |
| `lapse:57-60` | "Which of several equal deductions is taken, which of several equally preferred cells is guessed in, and which of that cell's marks is tried." | Reworded, not removed: the list settles the first of the three ties only. |
| `lapse:288`, `lapse:305-306` | as `reach:238`; "chosen and filter are the black boxes reach.allium reads" | As reach's line. The comment names `solver.allium`'s black box directly, since reach no longer reads it. |

The table gives the change as it holds for the first twelve techniques. Past them the
three Excludes sentences are narrowed and not removed, as step 7 says: deductions
equal in the order stay the implementation's until their witnesses are ordered.

Not changed: `solver.allium` (its `chosen`, `solver:174-176`, loses three uses, one in
each model's `Look`; reach and effort no longer read it, and lapse still does for the
two ties of a guess, `lapse:289` and `lapse:339`); `SameGridSameDeductions`, `SameInputSameRun`, `SameInputSamePrice` and
`LimitsOnlyHide`, whose wording stands and becomes stronger, since two implementations
that each satisfy them today may differ from one another.

The three statements of looking, counted from `rule` to its closing brace:

| Rule | reach | effort | lapse |
| --- | --- | --- | --- |
| `OpenGrid` | `reach:200-207`, 8 lines | `effort:156-160`, 5 | `lapse:272-276`, 5 |
| `KeepMarksTrue` | `reach:211-225`, 15 | `effort:162-171`, 10 | none: upkeep is inside `TakeStep` |
| `Look` | `reach:229-258`, 30 | `effort:173-209`, 37 | `lapse:278-347`, 70 |
| `TakeStep` | `reach:260-284`, 25 | `effort:211-231`, 21 | `lapse:349-401`, 53 |

That is 279 lines of 1,278 in the three files. What is shared word for word is small:
the heads of `Look` and `TakeStep`; the `let taken` line in reach and lapse; the 13
lines that apply a step in reach and effort (`reach:271-283`, `effort:218-230`, no
difference by `diff`); `KeepMarksTrue`'s lines in reach and effort, comments apart.
`Look`'s ends are three different status machines and are each model's own. What can
be stated once without `technique.allium` coming to know a run (its keeper is
`external entity Keeper {}`, `technique:123-126`) is what reads the grid and the
profile alone: the order, the list, and the effect of applying one deduction.

Other readers a change touches: `docs/explanation/puzzle-design.md` L914 quotes
`reach.allium:41-43` ("which of several tied deductions is taken is the
implementation's") and becomes false, and its other line citations shift;
`generation.allium` reads reach's `Rate`, `RunResult` and guarantee names and quotes no
tie, so it stands; `tickets/S07-undoing-a-deduction.md` and
`tickets/S08-removal-by-undoing.md` read the list and its order;
`crates/pawdoku/src/solver.rs` L28-41 is the precedent for a tie stated in the Rust
module's documentation (the first cell in grid order, the lowest digit), which the
order below agrees with in spirit.

**(b) Four fields hide, two choose among what is listed, four have no place.**

| Field | In the list | Read by | Stays with |
| --- | --- | --- | --- |
| `repertoire` | hides (`knows`, `technique:247`) | reach and lapse as sight; effort to say what is tried first and what is flagged (`effort:93-98`) | the list |
| `capacity` | hides (`holds`, `technique:248`) | the same; and it divides effort's price (`effort:107`) | the list for hiding, effort for the price |
| `spans` | hides (`takes_in`, `technique:249`) | the same | the list |
| `marking` | hides (`can_read`, `technique:250`) | the same | the list |
| `order` | no place: rank is the list's first key whatever the profile, and `order` only widens what a fixation chooses among | reach and lapse (`is_in_order`); effort only inside the black box `search_effort` (`effort:101-108`) | reach and lapse |
| `fixation` | chooses within; cannot be an input of the list, because it reads the last step (`reach:119-127`, `lapse:177-200`) | reach, lapse; effort inside `search_effort` | the models: it is state of a run |
| `upkeep` | no place: it changes the candidates the list is drawn from | lapse (`lapse:381-387`) | lapse |
| `budget` | no place | effort (`effort:89`) | effort |
| `fatigue` | no place | effort's header names it and no rule reads it (`effort:40`, `effort:308`) | effort |
| `patience` | no place | lapse (`lapse:286`) | lapse |

Three things follow that the proposal did not say:

- **Repeats are removed after the profile hides, never before.** `sees` reads a
  deduction's extent, and a hidden single's extent is its unit's
  (`ExtentFollowsThePattern`, `technique:785-800`). A placement licensed by a row and
  by a box is two deductions; a profile whose spans hold a box and no line sees one of
  them. If the row's witness were kept first, the entry would be hidden from a player
  who sees the placement. The same holds of a variable load past rank 20. So the list
  is drawn from the deductions the profile sees, and the order is an order over
  deductions.
- **effort needs what the profile does not see.** Its run escalates beyond the profile
  and ends `unpriced` only where the catalogue licenses nothing (`effort:197`,
  `effort:265-274`). A list of what the profile sees is empty exactly where effort
  goes on. An order over all the grid's deductions serves both.
- **`unsystematic` states no behaviour.** "Unsystematic takes what it sees, low or
  high" (`technique:112-113`), and `chosen` takes any one member, so an implementation
  that always takes the lowest satisfies it. Once the first in a stated order is
  taken, an unsystematic, unfixed player takes exactly what a systematic one takes;
  only a fixation tells them apart.

**(c) Published solvers go lowest first, three of the five take one deduction a step,
and none states the order among equal deductions.**

| Solver | Lowest first | One or all at a time | Order among equals | Stated where |
| --- | --- | --- | --- | --- |
| HoDoKu | yes, and back to the first technique after every step | one | singles from two queues filled as candidates were struck, so it depends on the path; locked candidates by boxes, rows, columns, then unit, then digit; subsets by boxes, rows, columns, then unit, then cell or digit combinations | the technique order in docs; the rest in source only |
| Sudoku Explainer 1.2.1 | yes: the first hint of the first producer | one | each producer's loops: blocks, columns, rows; naked single alone is row by row | source only |
| SudokuWiki | yes, a numbered list | singles all at once; above singles the work is on the server and cannot be read | singles: by row then column, or by digit then box | the strategy order in docs; the rest in the page's script or nowhere |
| QQWing | yes, a fixed chain of twelve checks | one | each function's loops, unit then digit in some and digit then unit in others | source only |
| `sudoku` crate (Emerentius) | yes, a caller's list | the first strategy all, the others one | naked singles by cell; hidden singles from a cursor that resumes where the last call stopped | source only |

Sources, all fetched 2026-10-03:

- HoDoKu: `https://hodoku.sourceforge.net/en/docs_solv.php` (docs): "The solver tries
  one technique after the other until it finds a possible step." and "changing the
  order of techniques can result in completely different solutions for one and the same
  sudoku". Source read from the GitHub fork `PseudoFish/Hodoku` at `c37fe90`, not the
  SourceForge tree: `SudokuSolver.getHint`, `SimpleSolver.findNakedSingle`,
  `findHiddenSingle`, `findLockedCandidates`, `findNakedXle`, `findHiddenXle`,
  `SudokuSinglesQueue`, `SolutionStep.compareTo`.
- Sudoku Explainer: the mirror `1to9only/SudokuExplainer` at `906e247`, the 1.2.1
  import (source): `Solver.getDifficulty`, `SingleHintAccumulator` ("accumulates a
  single hint (the first that is received) and then stops"), and the `getHints` of
  `HiddenSingle`, `NakedSingle`, `Locking`, `NakedSet`, `HiddenSet`. Its own
  documentation could not be fetched (the author's host did not resolve), so nothing
  is claimed of it.
- SudokuWiki: `https://www.sudokuwiki.org/Sudoku.htm`,
  `https://www.sudokuwiki.org/Strategy_Families` and
  `https://www.sudokuwiki.org/Grading_Puzzles` (docs); the page's client script,
  `https://www.sudokuwiki.org/js/sudoku5.js?v=3.70` (source, read for behaviour only:
  its header forbids re-use; the page shows solver version 2.71.2).
- QQWing: `stephenostermiller/qqwing` at `6048c90`, `src/cpp/qqwing.cpp` (source):
  `singleSolveMove`. Its README (`doc/README` at the same commit, docs) documents the
  difficulty names and no order.
- `sudoku` crate: `Emerentius/sudoku` at `b3f8f46`, `src/strategy/solver.rs` and
  `strategies/hidden_singles.rs` (source).

The one explicit comparator found is HoDoKu's `SolutionStep.compareTo`, which sorts its
"find all steps" listing: placements first, then the most strikes first, then shorter
chains, then sums of indices. It is a display order, not the solving order, it does not separate every pair
of different steps, and a comment in its own source records that an earlier form of it
was not transitive.

So a total order stated in a specification has no published precedent to conform to
and none to contradict. The practice supports the shape (rank first, one deduction a
step as the norm, back to the bottom); the order among equals is the specification's own decision.
In two of the five the first single is not even a function of the state, which a
specification that promises the same list from the same grid cannot copy.

**(d) Not refuted for ranks 1 to 12; not shown past them.**

The claim is that a run which always takes the lowest deduction ends with the lowest
possible hardest step. The argument: let the run's hardest step have rank h. At the
look where it first took a step of rank h, nothing below h yielded anything, and the
grid was not filled. If every run that uses only ranks below h ends at the same grid
whatever its order, then no run with ranks below h fills the grid, so no order solves
with a lower hardest step. The claim therefore holds wherever runs under a repertoire
of ranks 1 to m end at the same grid in every order. The specifications do not show
that: `reach:423` leaves it open, saying that "past subsets, a strike one player takes
can change the shape another pattern needed", and `ExtraCandidatesHideAndNeverMislead`
(`technique:883-889`) says outright that it is "not a monotonicity claim". Neither
argues why subsets would be the boundary, and no argument on paper is made here.

Measured, over the 226 grids of step 4:

- Taking the last entry of the lowest rank, or a seeded-random entry of the lowest rank
  (three seeds), in place of the first: the same end, the same final digits, the same
  candidates where the run stalled, and the same hardest rank on every grid.
- For each repertoire of ranks 1 to m, m from 1 to 12, a seeded-random entry of the
  whole list at every look (three seeds: 8,136 runs): every run ended at the grid the
  systematic run under that repertoire ended at, digits and candidates alike. None
  differed.
- A seeded-random entry of the whole list under ranks 1 to 12 reached the same end grid
  with a higher hardest rank on 218, 216 and 216 of the 226 grids (three seeds) and a
  lower one on none.

So for ranks 1 to 12 no grid refutes it, and no sampled order did better than the
systematic run, while the path-dependence `reach:429` asks about is real between
orders. The scope is narrow: repertoires that are the ladder up to some rank, with
capacity 4, full marks and every span, and seeded-random orders, which are a weak test
of a claim over all orders. Nothing is shown for a profile with a missing span or a
lower capacity. For ranks 13 to 29 nothing here speaks: the prototype stops at rank
12, and the specification itself expects the answer may be no.

**(e) Yes: a SAT solver checks both, by two queries, and one did.**

The encoding is the usual 729 variables with the rules as clauses; a state adds a unit
clause against every digit that is not a cell's candidate. Call that `STATE`.

- A deduction is sound if `STATE` with the placement denied, or with the struck
  candidate asserted, has no model. The premise is the digits and candidates standing
  and nothing else. Because a wrong earlier strike would make every later query pass
  for no reason, the oracle first checks that `STATE` itself has a model.
- A verdict is one if the givens have a model and, with that model blocked, no other.
- The two uniqueness techniques need a third form, because the promise is not a clause:
  verify the promise on the original givens by the second query, then check the
  deduction against that one solution. It was not built, since ranks 1 to 12 never read
  the promise.

Used: PySAT (`python-sat` 1.9.dev15, MIT) driving its bundled CaDiCaL 1.5.3 (MIT),
installed as a binary wheel with no compiler. Also read: Z3 (`z3-solver` 5.1.0.0, MIT;
not installed, so that it runs here is not
verified) and pycosat 0.6.6 with PicoSAT (MIT; source distribution only, and no
assumptions, core or proof through the binding). MIT places no condition on private use
in a throwaway venv. One caution if a solver were ever wanted as a tool of this
repository, which this spike does not propose: the PySAT wheel bundles about twenty
solvers under their own licences, and Glucose 4.1's `LICENCE` adds to MiniSat's terms,
for the files changed since Glucose 3.0, that the parallel version "cannot be used in
any competitive event (sat competitions/evaluations) without the express permission of
the authors". The licences of the Maple family, MergeSat, Minicard and Gluecard inside
the wheel were not read.

Sources for (e), all fetched 2026-10-03:

- `https://raw.githubusercontent.com/pysathq/pysat/master/LICENSE.txt` (source) and
  `https://pypi.org/pypi/python-sat/json` (docs): MIT; version 1.9.dev15 with a wheel
  for Python 3.14 on this machine. The same licence file is in the installed wheel.
- `https://raw.githubusercontent.com/pysathq/pysat/master/solvers/prepare.py` (source):
  which archive each bundled solver is built from.
- `https://raw.githubusercontent.com/arminbiere/cadical/master/LICENSE` (source).
- `http://www.labri.fr/perso/lsimon/downloads/softwares/glucose-syrup-4.1.tgz`
  (source): its `LICENCE` file.
- `https://raw.githubusercontent.com/Z3Prover/z3/master/LICENSE.txt` (source) and
  `https://pypi.org/pypi/z3-solver/json` (docs).
- `https://raw.githubusercontent.com/conda/pycosat/main/LICENSE`, the header of
  `picosat.c` in the same repository (source), its `README.rst` and
  `https://pypi.org/pypi/pycosat/json` (docs).

The branches named `master` and `main` were read as they stood that day; no commit was
pinned for them.

What the oracle found: all 226 grids have one solution (agreeing with two exhaustive
counters written by the grid agent); all 11,499 steps of the systematic runs, all
11,437 of the last-entry runs and all 19,190 of one unsystematic run are sound; and all
127,406 entries of the lists of the 4,205 sampled states are sound.

What it is beside the crate's exact solver (`crates/pawdoku/src/solver.rs`, which gives
givens their verdict): an oracle over candidates, which the crate has no entry for, and
one that shares no code with the prototype or the crate. What it is not: it proves a
deduction sound, not that the pattern named is the technique claimed, and it never
reads the technique, the witness or the order. `CatalogueValidation`
(`technique:1110-1116`) draws the same line: "An exact evaluator may validate fixtures;
it may not supply the witness that a simulated player supposedly found."

**Step 3. The list, its entry and its order.**

Drawn from: the grid's digits and candidates, `uniqueness_promised`, `max_chain_links`,
`max_forcing_paths` and the profile. For ranks 1 to 12 only the first and the last are
read.

- **Catalogue deduction.** One for each proof, as `technique:313-333` lists them: a
  unit for full_unit; a unit and digit for cross_hatch and for hidden_single; a cell
  for naked_single; a box, digit and line for pointing; a line and digit for claiming;
  a unit and a set of k cells for a naked subset; a unit, k digits and k cells for a
  hidden subset.
- **Witness.** The deduction's unit (`technique:187-190`: none for naked_single; the
  box for pointing and the line for claiming, by `technique:452-458`), its cells and
  its digits (`technique:334-335`). The cells: for full_unit and the three singles, the
  one cell placed in; for pointing, the box's open cells that allow the digit; for
  claiming, the line's; for a subset, its k cells. The digits: the one digit for the
  singles, pointing and claiming; for a naked subset, every digit its cells allow; for
  a hidden subset, its k digits.
- **Output.** A placement (row, column, digit), or the strikes: one for each position,
  each with its digits, in order of position. A strike names only candidates that
  stand, and a position with none is not listed.
- **Entry.** A technique, an output and one witness. Among the deductions the profile
  sees, all with the same technique and the same output are one entry, which keeps the
  least witness. Two techniques with the same output keep an entry each
  (`technique:336-337`).
- **Witness order.** Unit kind (none, row, column, box), unit index, the cells as a
  list in row-major order compared element by element, then the digits likewise.
- **List order.** Rank; then a placement before strikes; then the output: a placement
  by row, column, digit; strikes as the list of (row, column, digits), compared element
  by element, a proper prefix first.

No two entries are equal: after repeats are removed no two share a technique and an
output, rank and output are the whole of the list order, and each comparison is of
integers or of lists of integers. The witness order never breaks a tie between entries;
it only says which witness an entry keeps. Over deductions the order rank, output,
witness is total as well, since two deductions of one technique differ in witness.

The pinned profile: repertoire exactly ranks 1 to 12, capacity 4, `full_marks`, every
extent, `systematic`, `unfixed`; `uniqueness_promised` true and both limits at their
defaults. It sees every deduction of ranks 1 to 12.

Reducible subsets stay licensed (`technique:1136-1138`): a naked triple that holds a
naked pair is still listed.

The run: from the givens, born kept, which is to say every open cell's candidates are
the digits no placed peer holds. At a look, a filled grid ends the run solved; the
list is drawn up; an empty list ends it stalled; otherwise the first entry is taken. A
placement sets the digit and leaves it the cell's one candidate, and the digit is
struck from every open peer before the next look (`KeepMarksTrue`). Strikes remove
their digits. Looks count from 0 and steps from 1; a stalled look is a look, and the
filled grid that ends a solved run is not.

**Step 4. The prototype, the counters and the counts.**

The counters, fixed before any program ran, for each technique at each look:

- **Patterns examined.** full_unit: the 27 units. cross_hatch and hidden_single: each
  unit and each digit the unit has not placed. naked_single: each open cell. pointing:
  each box, each digit the box has not placed, and each of the two directions.
  claiming: each line and each digit the line has not placed. Naked k: each unit and
  each set of k of its open cells. Hidden k: each unit and each set of k of the digits
  it has not placed.
- **Witnesses found.** The catalogue deductions of the technique, before repeats are
  removed.
- **Entries.** After repeats are removed.

And three ways of listing: the catalogue (every deduction), the list (every entry; the
same search, so the same patterns), and the first rank (listing stops at the first
rank that yields anything: the patterns of every rank up to and including it, and that
rank's entries alone; at a look where nothing yields, the patterns of all twelve ranks
and no entries).

The grids, 226 in all and 225 distinct, each with one solution by three counters:

| Group | Grids | Givens | Source |
| --- | --- | --- | --- |
| study | 4 | 25, 29, 32, 32 | `docs/explanation/puzzle-design.md` L536-546, L575-585, L611-621, L656-666 |
| derived | 8 (7 distinct) | 22 to 25 | each study puzzle with givens taken out one at a time, in row-major order (`_min`) or the reverse (`_minrev`), wherever one solution remains |
| seventeen | 100 | 17 | `data/puzzles2_17_clue` in tdoku's `data.zip`, lines 5 to 104 |
| unbiased | 50 | 24 to 28 | `data/puzzles1_unbiased` in the same archive, lines 1 to 50 |
| top1465 | 50 | 18 | `http://magictour.free.fr/top1465`, lines 1 to 50 |
| norvig_hardest | 11 | 22 to 28 | `https://norvig.com/hardest.txt`, all lines |
| named | 3 | 21, 22, 22 | link values on `https://www.sudokuwiki.org/Arto_Inkala_Sudoku` |

The seventeen-given grids are the states with the most candidates: 321 standing in the
64 open cells of `seventeen_049`, and of `seventeen_058`, at the first look. They are the first hundred of a
sorted file, not a random sample.

The completeness check came first. The sample is 4,205 states: every look of the
twelve study and derived grids under the systematic run; every seventh look and the
last look of every other grid under it; and every twelfth look and the last look of
every grid under one unsystematic run, whose states hold strikes a systematic run
never makes. On every one, the independent search and the prototype gave the same
entries in the same order with the same witnesses, and the same count of catalogue
deductions for each technique: 4,205 of 4,205. The independent search tries every
unit, every digit, both directions, every set of k open cells and, for hidden subsets,
every set of k digits against every set of k cells, with the predicates transcribed
from `technique:360-438` as plain sets.

The same grid, premise, limits and profile gave the same steps twice, on all 226.

The runs under the pinned profile: 11,576 looks and 11,499 steps; 149 grids solved and
77 stalled.

| Group | Grids | Solved | Stalled | Looks | Steps |
| --- | --- | --- | --- | --- | --- |
| study | 4 | 2 | 2 | 159 | 157 |
| derived | 8 | 3 | 5 | 301 | 296 |
| seventeen | 100 | 95 | 5 | 6,465 | 6,460 |
| unbiased | 50 | 23 | 27 | 1,912 | 1,885 |
| top1465 | 50 | 23 | 27 | 2,452 | 2,425 |
| norvig_hardest | 11 | 3 | 8 | 282 | 274 |
| named | 3 | 0 | 3 | 5 | 2 |

Hardest rank reached, by grids: rank 2, 45; rank 3, 36; rank 5, 38; rank 6, 12; rank
7, 23; rank 8, 58; rank 9, 5; rank 10, 6; rank 11, 1; no step at all, 2.

Counts by rank, summed over all 11,576 looks, with the most at any one look:

| Rank | Technique | Steps taken | Patterns examined (sum; most) | Witnesses found (sum; most) | Entries (sum; most) | Patterns when listing stops at the first rank (sum; most) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | full_unit | 2,970 | 312,552; 27 | 5,445; 6 | 3,985; 4 | 312,552; 27 |
| 2 | cross_hatch | 6,554 | 1,235,637; 192 | 65,594; 33 | 32,913; 16 | 1,076,016; 192 |
| 3 | naked_single | 560 | 411,879; 64 | 20,448; 10 | 20,448; 10 | 105,971; 63 |
| 4 | hidden_single | 120 | 1,235,637; 192 | 70,747; 33 | 35,001; 16 | 237,813; 189 |
| 5 | pointing | 914 | 823,758; 128 | 21,907; 12 | 21,906; 12 | 146,040; 126 |
| 6 | claiming | 121 | 823,758; 128 | 25,395; 11 | 25,395; 11 | 47,274; 126 |
| 7 | naked_pair | 110 | 2,852,801; 602 | 29,114; 24 | 25,416; 22 | 137,962; 575 |
| 8 | hidden_pair | 138 | 2,852,801; 602 | 44,433; 27 | 33,971; 20 | 96,328; 575 |
| 9 | naked_triple | 5 | 4,047,850; 1,095 | 28,353; 23 | 25,421; 20 | 46,572; 954 |
| 10 | hidden_triple | 6 | 4,047,850; 1,095 | 26,152; 23 | 23,541; 20 | 43,557; 954 |
| 11 | naked_quad | 1 | 3,774,242; 1,279 | 20,041; 14 | 18,020; 14 | 36,743; 1,021 |
| 12 | hidden_quad | 0 | 3,774,242; 1,279 | 12,760; 13 | 12,490; 10 | 36,202; 1,021 |

Per look, over all looks:

| Figure | Most | Mean | Median | 99th percentile | Sum |
| --- | --- | --- | --- | --- | --- |
| Catalogue deductions | 179 | 32.0 | 30 | 97 | 370,389 |
| List entries | 133 | 24.1 | 22 | 69 | 278,507 |
| Entries when listing stops at the first rank | 16 | 2.5 | 2 | 8 | 29,320 |
| Patterns examined, catalogue or list | 6,683 | 2,262.7 | 1,831.5 | 6,312 | 26,193,007 |
| Patterns examined, first rank | 5,775 | 200.7 | 138 | 1,768 | 2,323,030 |

The most catalogue deductions and list entries were at `study_beginner`, look 27; the
most patterns at look 0 of `seventeen_006`, `seventeen_008`, `seventeen_010` and
`seventeen_011`. The first-rank rows include the 77 stalled looks, where nothing
yields and all twelve ranks are examined: the 5,775 is the stalled look of
`top1465_002`, of `top1465_003` and of `top1465_007`. Over the 11,499 looks that
yielded a step the first-rank patterns sum to 2,078,945, with a mean of 180.8 and at
most 3,620, at `top1465_032`, look 10, which had to go on to the hidden triples.

When listing stops at the first rank, the witnesses found and the entries, by rank 1
to 12, summed over all looks with the most at one look:

```text
witnesses  5,445;6  39,853;33  898;5  165;4  2,893;11  157;4  190;5  404;9  5;1  6;1  1;1  0;0
entries    3,985;4  20,830;16  898;5  129;3  2,893;11  157;4  186;5  230;5  5;1  6;1  1;1  0;0
```

These figures are of the systematic run. The other runs of (d), under the same
profile, pass through other states: over every look of every recorded run the most
was 408 catalogue deductions, 188 list entries and 38 entries of one placement
technique, and still 6,683 patterns.

What the figures say:

- The list is short: 24 entries at a typical look and never more than 133 in a
  systematic run (188 in any run), three quarters of the catalogue's deductions.
- It is cheap to draw up, and eleven times cheaper to take the first without it:
  2,263 patterns a look for the whole list against 201 when listing stops at the first
  rank that yields (twelve times, over the looks that yield). Four fifths of the whole list's cost is the six subset techniques,
  which took 260 of the 11,499 steps.
- The run needs only the first rank. The whole list is what a caller reads: a hint
  that shows more than one step, S07's undoings, S08's draw.
- cross_hatch and hidden_single found the same placements wherever marks were kept
  and nothing had been struck, and both are on the list, as `technique:336-337` says
  they must be. A quarter of all the entries listed are that pair.

The four study puzzles, which their page marks unverified. Each has exactly one
solution. Under ranks 1 to 12:

| Puzzle | End | Steps | Hardest rank | Against the page |
| --- | --- | --- | --- | --- |
| beginner | solved | 49 | 2 | agrees: "singles only" (21 full_unit, 28 cross_hatch) |
| intermediate | solved | 54 | 6 | agrees: singles and locked candidates; its first pointing is the page's, 6 in r4c2 and r6c2 striking r2c2, r3c2 and r7c2 |
| advanced | stalled, 33 cells open | 22 | 6 | consistent: the page needs an X-Wing and an XY-Wing, ranks 13 and 16; no pair was taken before the stall |
| expert | stalled, 34 cells open | 32 | 8 | agrees on the stall and on r3c9 holding 2 and 5 there; this run took eight locked-candidate steps (seven pointing and one claiming, striking 14 candidates) where the page says six, with one naked pair and one hidden pair as the page says |

The paths the page describes past the stalls were not checked: they need ranks the
prototype does not hold.

**Step 5. The bounds.**

*Ranks 1 to 12, patterns examined.* The counters read only which cells are placed.
With every unit fully open a look examines at most 27 + 243 + 81 + 243 + 162 + 162 for
ranks 1 to 6 and 2 x 27 x (36 + 84 + 126) for the subsets: **14,202**. Only the empty
grid attains it. A grid with one solution has at least 17 givens (McGuire, Tugemann
and Civario, 2012; cited from memory, not fetched), a run only places more, and each
placed cell closes one open cell in three units. With 17 placed, ranks 1 to 6 are
exactly 27, 192, 64, 192, 128 and 128. For the subsets the sum over units of the
number of k-sets is largest when the placed cells crowd into the fewest units, and two
rows of one band cannot both be without a given (exchanging them in the solution would
give a second), so at most three rows and three columns are fully open; maximising
rows, columns and boxes apart gives 712, 1,560 and 2,208 for one technique of each
size. The total is **9,691**. It is an upper bound on every look of every run from a
puzzle with one solution, and it is not shown to be attained.

Measured against it: the most patterns at any look was 6,683, at the first look of a
17-given grid, 69% of the bound. Technique by technique the most measured was 27, 192,
64, 192, 128 and 128, which meet the bound exactly, and 602, 1,095 and 1,279 for the
pairs, triples and quads against 712, 1,560 and 2,208.

*Ranks 1 to 12, the list.* On a grid whose digits and candidates are true, a placement
technique has at most one entry for each open cell, so at most 64; the measured most
was 38. On such a grid the catalogue's deductions and the list's entries are bounded
together by 14,121: one deduction at most for each pattern, less 81 because a box and
digit point along a row or a column and never both. It is loose: the most measured was
408 and 188. On a state that contradicts itself a hidden subset's digits may have
fewer places than digits, several sets of cells then satisfy the predicate, and the
bound is larger.

*Ranks 13 to 29, the list.* An entry is a technique and an output, and "Advanced
patterns strike every eligible target for that witness" (`technique:905-906`), so an
output is a set of strikes.

| Ranks | Techniques | Witnesses at most | Entries at most | Reads the limits |
| --- | --- | --- | --- | --- |
| 13-15 | fish | 648; 1,512; 2,268 | the same | no |
| 16-18 | xy_wing, xyz_wing, w_wing | 15,390; 15,390; 262,440 | 3,240; 15,390; 131,220 (4,860 if a target need only be outside the two matching cells) | no |
| 19-21 | skyscraper, two_string_kite, empty_rectangle | 648; 2,916; 8,748 | 648; 2,916; 729 | no |
| 22 | simple_colouring | 360 components | 1,080 | no |
| 23 | remote_pair | every chain of `{a,b}` cells: about 10 million on a true grid with 17 or more placed, 4e32 on any state | at most one for each set of chain cells, under 800,000 on a true grid: not bounded by anything small, as written | `max_chain_links` |
| 24-26 | x_chain, xy_chain, alternating_inference_chain | grows as the branching to the power of the links: up to 1e26 at 23 links | 29,160; 6,480; 266,814 | `max_chain_links` |
| 27 | forcing_chain | up to nine paths, each as a chain | 1,458 | both |
| 28, 29 | unique_rectangle_type_1, bug_plus_one | 1,944; 1 | 1,944; 1 | no; both read the promise |

For ranks 24 to 26 the rule of step 3 does all the work. The strikes are "any
candidate weakly linked to BOTH endpoints" (`technique:983-984`), a function of the two
endpoints alone, so however many paths join them the entries are at most the pairs of
endpoints, whatever `max_chain_links` is. For rank 27 each proof concludes one literal,
so at most 729 placements and 729 strikes. Rank 23 is the exception: its targets are
cells that see two chain cells "an odd number of links apart" (`technique:973-978`), so
the output depends on which cells the chain holds, a chain and a longer chain holding
it can differ in output, and the entries are bounded only by the chains' sets of cells.

*Ranks 13 to 29, the search.* Step 4's counters are defined for ranks 1 to 12. Past
them a pattern is counted here by analogy, as the choice a search makes before it
tests the predicate (a digit and k base lines for a fish; a pivot and two peers for
xy_wing and xyz_wing; a pair of cells that are not peers, a digit and a unit for
w_wing; and so on), and for the chains the unit is one implication inspected. For
ranks 13 to 22, 28 and 29 the size of the grid fixes the patterns and neither limit
enters: at most 177,402 patterns a look, of which w_wing is 131,220, and at most
161,596 entries. The entry figure was first given as 35,236, on the argument that a
w_wing's strikes are fixed by its two matching cells and the digit struck. Under the
reading taken in "Step 5, extended", where a target is outside all four cells of the
witness, that is false: at look 47 of `norvig_hardest_04` the cells r5c5 and r9c6,
both holding 6 and 9, strike 9 at r7c5 and r9c5 through the conjugate pair in row 2,
and at r9c5 alone through the pair in row 7. So w_wing's entries are bounded by its
deductions, 131,220, until what a target excludes is settled. Both are loose, and the first counts a colouring component and a
bug_plus_one candidate as one pattern each: their inner tests, at most 59,049 pairs
of cells for colouring, are not in it.

For the chains there are two searches. Listing every witness path is out of reach: the
count grows as the branching to the power of `max_chain_links`. Deciding, for each pair
of endpoints, whether some path within the limit joins them is not: the static links
(`LinkMeaning`, `technique:952-963`) make a graph of at most 1,458 literals and 23,328
implications, each step allowed or not by its two literals alone, and reachability
within l links is one pass for each start over (literal, links used). That is at most
729 x 24 x 23,328, about 4.1e8 inspections a look for rank 26, linear in
`max_chain_links`, and about 3.3e7 and 1.3e7 for ranks 24 and 25. For rank 27 the paths
are "static-link paths" with "no path step that needs results of earlier nonadjacent
steps" (`technique:1002-1004`), so the same reachability serves, about 4.8e7; and
`max_forcing_paths` never binds, since a cell has at most nine candidates and a digit
at most nine places in a unit. The list needs only the second search for ranks 24 to
27. The catalogue, one deduction for each proof, cannot be drawn up for ranks 23 to 27
at all, and for rank 23 as written neither can the list.

Those search bounds stand on three readings the specification does not yet state:

1. **That a chain may pass through a candidate twice.** Only remote_pair says "no
   repeating cells" (`technique:978`); for ranks 24 to 27 nothing is said. If walks are
   admitted the pass above decides the specification's own question for ranks 26 and
   27, since a loop can be cut out of any walk. For x_chain ("at least three links",
   `technique:985`) and xy_chain ("at least three distinct cells", `technique:988`)
   cutting a loop can take a walk below the minimum, so either the minimum counts
   distinct candidates or cells, and no polynomial bound is argued here, or a walk
   meets it and every conjugate pair is an x_chain. If a chain must not hold a
   candidate in both polarities, no polynomial bound is argued either.
2. **What remote_pair's output is.** As written it is not bounded on either count. Two
   choices bound it: the witness is the whole two-coloured component, as
   simple_colouring's is (at most 20 entries); or an entry is one struck candidate.
   Under either the search is argued only where the `{a,b}` cells hold no odd cycle,
   which is so on a grid whose digits and candidates are true.
3. **An order on paths, and which witness `holds` reads.** Step 3's witness order
   covers units, cells and digits. A variable load "is the number of distinct candidate
   propositions in the proof" (`technique:598-599`) and `holds` hides by load. If the
   witness were kept before the profile hides, it would have to be one of least load,
   or an entry is hidden from a player who could hold it. If the profile hides first,
   as (b) requires, any witness the player can hold will do as the one kept; but
   whether some witness is within a capacity is then the question to answer, and for
   a chain that asks for the least load all the same. A proof of fewest links is not that witness once a chain may pass
   through a candidate twice: load counts distinct candidates, not links. At look 28 of
   `study_advanced` under the wider profile, the closure that places 4 at r2c8 has a
   shortest proof of 9 links over 9 candidates and a longer one of 11 links over 8
   (found by the Codex review of these notes; the longer path is sound link by link
   but was not re-derived here). So the least-load witness is not argued for any
   chain. Where a chain may not repeat a candidate, links and candidates rise
   together and the fewest links would do, but there the search bound of reading 1
   is the one not argued. Until this is settled, the profile must hide before a
   witness is chosen, as (b) already requires.

So, of the two outcomes the acceptance criterion allows: both bounds are argued for
ranks 13 to 22, 28 and 29, on paper alone; for ranks 23 to 27 they are not argued as
the specification stands. By the letter of the criterion the recommendation is
therefore limited to ranks 1 to 12, where it rests on measurement and bound together.
What is known past them is recorded as evidence for widening it and not as part of
it: ranks 13 to 22, 28 and 29 are bounded here and measured below, and ranks 23 to 27
wait on the three readings. One smaller gap met on the way: whether simple_colouring's wrap and trap
are one deduction or several.

**Step 5, extended. Ranks 13 to 22, 28 and 29, measured.** After the first commit of
these notes the maintainer asked, on 2026-10-03, for the ranks that were bounded on
paper alone to be run as ranks 1 to 12 were. Ranks 23 to 27 are still not built: they
wait on the three readings above.

The definitions were again fixed before any program existed, and again two agents
worked apart: one extended the prototype, one wrote an exhaustive search from the
definitions and `technique:896-971` and `technique:1018-1108`, forbidden to read the
prototype. Terms: a place of d is an open cell that allows d; d is unplaced in a unit
that holds no d; two cells see each other when they share a row, a column or a box; a
cell has exactly S when it is open and its candidates are S; a conjugate pair for d is
a unit where d is unplaced and has exactly two places. Cells are indexed row by row
from 0 to 80. Every strike technique strikes from every target that allows the digit,
and with no target there is no deduction.

| Rank | Technique | One catalogue deduction for each | Output | Witness cells; digits; role |
| --- | --- | --- | --- | --- |
| 13, 14, 15 | x_wing, swordfish, jellyfish (k = 2, 3, 4) | digit d, orientation (rows as bases with columns as covers, or columns as bases with rows as covers), set of k base lines: in each base d is unplaced and has between 2 and k places; the cover lines those places lie on, taken over all k bases, are exactly k | strike d from every cell of the k covers that is not in a base | every place of d in the bases; [d]; [orientation (0 rows as bases, 1 columns as bases), then the k base line numbers ascending] |
| 16 | xy_wing | pivot P and unordered pair of wings {W1, W2}: distinct digits a, b, c; P has exactly {a,b}; W1 and W2 each see P; one wing has exactly {a,c} and the other exactly {b,c} | strike c from every cell that sees both wings | {P, W1, W2}; [a,b,c] sorted; [index of P] |
| 17 | xyz_wing | as xy_wing but P has exactly {a,b,c} | strike c from every cell that sees P and both wings | {P, W1, W2}; [a,b,c] sorted; [index of P] |
| 18 | w_wing | unordered pair {P, Q} of cells that do not see each other, both with exactly {a,b}; linking digit a (one of the two); unit u in which a has a conjugate pair {U, V}; P, Q, U, V all distinct; and P sees U and Q sees V, or P sees V and Q sees U | strike b (the other digit) from every cell that sees both P and Q | {P, Q, U, V}; [a,b] sorted; [a, unit kind (0 row, 1 column, 2 box), unit number] |
| 19 | skyscraper | digit d, orientation (0: two rows as the parallel lines, 1: two columns), two parallel lines L1 < L2, each a conjugate pair for d; exactly one endpoint of L1 shares its perpendicular line with an endpoint of L2 (the aligned pair), and the two remaining endpoints (the unaligned ones) lie on different perpendicular lines | strike d from every cell that sees both unaligned endpoints | the four endpoints; [d]; [orientation, L1, L2] |
| 20 | two_string_kite | digit d, row r that is a conjugate pair for d {A, B}, column c that is a conjugate pair for d {C, D}, the four cells distinct, and a joined pair (X from the row pair, Y from the column pair) in one box | strike d from every cell that sees both remaining endpoints | the four cells; [d]; [r, c, index of X, index of Y] |
| 21 | empty_rectangle | variant 0: box B, digit d unplaced in B, a row R and a column C through B: every place of d in B lies in row R or column C; some place lies in R and not C; some place lies in C and not R; the cell (R, C) does not allow d; B has at least three places for d; a column K outside B's three columns that is a conjugate pair for d {P, Q} with P = (R, K) and Q = (S, K), S outside B's three rows. Variant 1 is the transpose: a row K outside B's three rows that is a conjugate pair {P, Q} with P = (K, C) and Q = (K, S), S outside B's three columns | variant 0: strike d at (S, C). Variant 1: strike d at (R, S) | B's places for d, P and Q; [d]; [variant, box number, R, C, K] |
| 22 | simple_colouring | digit d and a connected component, of two or more cells, of the graph whose nodes are the cells that allow d and whose edges join the two cells of each conjugate pair for d (in any row, column or box). Two-colour it; a component that cannot be two-coloured licenses nothing. **Reading**: wrap and trap are separate deductions. *Trap*, one per component: targets are cells outside the component that see a node of each label. *Wrap*, one per label that has two of its nodes seeing each other | trap: strike d from the targets. Wrap: strike d from every node of that label (here the targets are the label's own nodes, an exception to "outside the witness") | every node of the component; [d]; trap: [0]. Wrap: [1, least index among that label's nodes] |
| 28 | unique_rectangle_type_1 | only when `uniqueness_promised`: four open cells at the crossings of two rows and two columns, lying in exactly two boxes; three have exactly {a,b}; the fourth allows both a and b and at least one more digit | strike a and b from the fourth (one strike, digits [a,b]) | the four cells; [a,b] sorted; [index of the fourth] |
| 29 | bug_plus_one | only when `uniqueness_promised`: every open cell has exactly two candidates except one cell T with exactly three; and a digit d of T such that, with d removed from T, every unit has exactly two places for every digit unplaced in it | place d in T | [T]; [d]; [] |

Two readings were taken where the specification is open. Each is provisional and is
flagged for the follow-up, not settled:

- **A target is outside the pattern's own cells.** "Other cells" is read as excluding
  every cell of the witness. simple_colouring's wrap strikes its own nodes, and
  unique_rectangle_type_1 strikes its fourth corner, as the specification states them.
- **simple_colouring's wrap and trap are separate deductions,** one trap for each
  component and one wrap for each label that has two nodes seeing each other; a
  component that cannot be two-coloured licenses nothing.

Two more points are not open: they are what the specification says, and are recorded
as cases a build must get right.

- **The two ways round a w_wing are one deduction** (`technique:914-915`); swapping the
  two digits is another (`technique:934-935`).
- **An xy_wing's wings may see each other.** `technique:928-931` asks only that each
  wing be a peer of the pivot.

The witness gains a third part past rank 12, the role in the table's last column, a
list of integers compared after the cells and the digits, so that no two deductions of
one technique are equal in the order. Line, box and unit numbers in it run from 1 to
9. The list order is unchanged.

The profile, pinned in full: repertoire exactly ranks 1 to 22, 28 and 29; capacity
729, so that no variable load is hidden; `full_marks`; every extent; `systematic`;
`unfixed`; `uniqueness_promised` true. It is a profile the specification allows, since
a repertoire is a set, so its run is that player's own systematic run.

The counters for patterns examined, a convention fixed so that two programs count
alike: the fish, 9 digits by 2 orientations by every set of k of the 9 lines (648,
1,512 and 2,268 at every look); xy_wing and xyz_wing, each open cell as pivot with
each pair of its open peers; w_wing, each pair of open cells that do not see each
other, by 2, by 27; skyscraper 648 and two_string_kite 729 at every look;
empty_rectangle 8,748 at every look; simple_colouring, 3 for each component of two or
more cells; unique_rectangle_type_1, each rectangle in exactly two boxes with four
open corners; bug_plus_one, 1. These count choices before any pruning, so a search
that first asks whether two cells hold the same pair examines far fewer.

The completeness check: 5,219 states, being every look of the systematic run from the
look where the run under ranks 1 to 12 ended, every tenth look before it and the last,
and every ninth look and the last of one unsystematic run. On all 5,219 the two
programs gave the same entries, in the same order, with the same witnesses, and the
same count of catalogue deductions for each of the 24 techniques.

Soundness: the 189,609 sampled entries of ranks 1 to 22 are sound by the SAT oracle,
18,350 of them of ranks 13 to 22. The two uniqueness techniques are not implied by the
candidates alone, as (e) says, so they were checked against each grid's one solution:
all 474,257 catalogue deductions at every look of the systematic run agree with it,
the 1,602 of ranks 28 and 29 among them. The same steps came out twice on all 226
grids, and for every grid the run begins with the steps of the run under ranks 1 to
12.

The runs: 12,585 looks and 12,532 steps; 173 grids solved and 53 stalled. Twenty-four
grids that stalled under ranks 1 to 12 are solved, and none is lost.

| Group | Grids | Solved | Stalled | Looks | Steps |
| --- | --- | --- | --- | --- | --- |
| study | 4 | 4 | 0 | 231 | 231 |
| derived | 8 | 5 | 3 | 378 | 375 |
| seventeen | 100 | 99 | 1 | 6,612 | 6,611 |
| unbiased | 50 | 35 | 15 | 2,383 | 2,368 |
| top1465 | 50 | 25 | 25 | 2,578 | 2,553 |
| norvig_hardest | 11 | 5 | 6 | 398 | 392 |
| named | 3 | 0 | 3 | 5 | 2 |

Hardest rank reached, by grids: rank 2, 43; 3, 34; 5, 33; 6, 5; 7, 12; 8, 32; 9, 3;
10, 3; 14, 2; 16, 4; 17, 4; 18, 10; 19, 15; 20, 14; 21, 1; 22, 2; 28, 6; 29, 1; no
step, 2.

Counts by rank, summed over all 12,585 looks, with the most at any one look:

| Rank | Technique | Steps taken | Patterns examined (sum; most) | Witnesses found (sum; most) | Entries (sum; most) | Patterns when listing stops at the first rank (sum; most) |
| --- | --- | --- | --- | --- | --- | --- |
| 13 | x_wing | 8 | 8,155,080; 648 | 5,692; 5 | 5,171; 5 | 105,624; 648 |
| 14 | swordfish | 3 | 19,028,520; 1,512 | 4,218; 6 | 4,077; 6 | 234,360; 1,512 |
| 15 | jellyfish | 0 | 28,542,780; 2,268 | 3,543; 6 | 3,543; 6 | 344,736; 2,268 |
| 16 | xy_wing | 14 | 27,908,234; 7,437 | 2,522; 7 | 2,430; 6 | 452,772; 6,565 |
| 17 | xyz_wing | 15 | 27,908,234; 7,437 | 3,508; 5 | 3,445; 5 | 429,921; 6,565 |
| 18 | w_wing | 25 | 378,044,604; 82,134 | 4,074; 16 | 2,638; 8 | 5,376,726; 77,112 |
| 19 | skyscraper | 17 | 8,155,080; 648 | 1,705; 4 | 1,687; 4 | 63,504; 648 |
| 20 | two_string_kite | 18 | 9,174,465; 729 | 4,077; 8 | 4,075; 8 | 59,049; 729 |
| 21 | empty_rectangle | 1 | 110,093,580; 8,748 | 415; 2 | 414; 2 | 551,124; 8,748 |
| 22 | simple_colouring | 2 | 455,916; 87 | 36,977; 14 | 30,290; 12 | 3,228; 72 |
| 28 | unique_rectangle_type_1 | 6 | 676,191; 192 | 1,600; 2 | 1,600; 2 | 5,304; 160 |
| 29 | bug_plus_one | 1 | 12,585; 1 | 2; 1 | 2; 1 | 54; 1 |

Per look, over all looks, for the whole profile:

| Figure | Most | Mean | Median | 99th percentile | Sum |
| --- | --- | --- | --- | --- | --- |
| Catalogue deductions | 186 | 37.7 | 35 | 105 | 474,257 |
| List entries | 145 | 28.7 | 27 | 76 | 361,674 |
| Entries when listing stops at the first rank | 16 | 2.5 | 2 | 8 | 31,705 |
| Patterns examined, catalogue or list | 118,136 | 51,277.9 | 44,121 | 114,403 | 645,332,612 |
| Patterns examined, first rank | 110,661 | 813.9 | 132 | 12,583 | 10,242,793 |

The first-rank rows include the 53 stalled looks, where every rank is examined; over
the 12,532 looks that yielded a step the mean is 479.3 and the most 103,937.

What the figures say:

- **The list barely grows.** The twelve techniques add at most 24 entries at any look
  and 59,372 over all looks, about five a look; the whole list is 28.7 entries at a
  typical look and at most 145. The paper bound of 161,596 entries is looser than the
  measurement by almost four orders of magnitude.
- **The whole list becomes dear, by this count.** 51,278 patterns a look against 2,263
  for ranks 1 to 12, and 96% of them belong to the twelve techniques: w_wing alone is
  59% and empty_rectangle 17%. Both are artefacts of counting every choice before
  any pruning, which is why the convention was fixed first; they are still within the
  paper bound (w_wing at most 82,134 against 131,220; the twelve together at most
  about 111,500 against 177,402).
- **Taking the first stays cheap.** A run that stops at the first yielding rank
  examines 132 patterns at the median look and 814 at the mean, since ninety-nine
  steps in a hundred are still taken below rank 13: the twelve techniques took 110 of
  the 12,532 steps.
- **Coverage is uneven.** simple_colouring is on the list often (30,290 entries) and
  taken twice; jellyfish was never taken; empty_rectangle was taken once and
  bug_plus_one was licensed at two looks. The agreement of the two programs covers
  every state sampled, but for those the states that exercise them are few.
- **Every order still ended at one grid.** One unsystematic run under this profile
  ended at the same digits on all 226 grids and at the same candidates on the 53 that
  stalled. That is one seed, and it is not the test of (d); it is recorded because
  `reach:423` expects otherwise past subsets and this did not show it.

The study puzzles again. Under this profile all four are solved:

| Puzzle | Steps | Hardest rank | Against the page |
| --- | --- | --- | --- |
| advanced | 58 | 16 | agrees on the X-Wing exactly: 4 in rows 1 and 5, columns 3 and 9, striking r4c9 and r7c3. The run then took a naked pair and an xy_wing that strikes 4 at r2c1, not the page's xy_wing |
| expert | 70 | 19 | disagrees: the page says the ordinary advanced repertoire stalls and a contradiction chain is needed. This run went on from the page's stall with an xyz_wing (2 at r6c9), a w_wing (6 at r2c7) and a skyscraper, and solved it with no chain |

Rows for a rebuild, the fifteen grids that need no network, in the columns of the
block below under "What S07 needs":

```text
study_beginner solved 49 2 6371 4676 375 1627264 3795
derived_beginner_min solved 57 2 3284 2576 221 2308392 5490
derived_beginner_minrev solved 59 3 2774 2165 212 2514502 5775
study_intermediate solved 54 6 2862 2148 187 1869281 6047
derived_intermediate_min stalled 14 17 295 258 39 1156135 90573
derived_intermediate_minrev solved 63 6 2200 1645 147 2712172 8196
study_advanced solved 58 16 2372 1678 151 2095066 15819
derived_advanced_min stalled 20 20 438 379 51 1510181 121609
derived_advanced_minrev stalled 20 20 454 400 50 1557266 132437
study_expert solved 70 19 2805 2144 164 2869283 82040
derived_expert_min solved 71 19 2659 2011 173 3064130 54570
derived_expert_minrev solved 71 19 2659 2011 173 3064130 54570
inkala_2012 stalled 0 0 0 0 0 103787 103787
sudokuwiki_unsolvable_28 stalled 2 5 12 10 3 302013 101786
sudokuwiki_unsolvable_49 stalled 0 0 0 0 0 100279 100279
```

So for ranks 13 to 22, 28 and 29 there is now measurement as well as a paper bound,
under the two readings above, and the maintainer may widen the recommendation to them
once those two are settled. Ranks 23 to 27 are as they were. The two programs share
these definitions, so their agreement shows that each implements them and says
nothing of whether the readings are right.

**Step 6. The table of models, and the verdict.**

| Model | Entry | What it reports that the list cannot | Kept, moved or lost |
| --- | --- | --- | --- |
| `reach` | `Rate`, `NextStep` | Nothing in `RunResult` for a systematic, unfixed profile: every value is the run or read off it. Its own: the hint surface and its premise, the rating's precondition and guarantees, fixation. | For ranks 1 to 12 the tie is lost, on purpose; past them it is narrowed to deductions equal in the order. `sees`, `is_in_order` and `is_preferred` are kept as written; only `chosen` becomes the first in `technique.allium`'s order. `Look`'s ends, the two entries and the surfaces are kept. |
| `effort` | `Price` | Price, total, the dearest step, budget; escalation and `unpriced`, which need the deductions the profile does not see. | The tie among the equally cheap is lost for ranks 1 to 12 and narrowed past them. `is_cheapest` is kept: taking the first in its place would answer the open question at `effort:316`, which is not this ticket's. |
| `lapse` | `Tackle` | Lapsed marks, checks, guesses, repairs, abandonment; a list drawn from marks that may be stale (`lapse:219`). | The tie among deductions is lost for ranks 1 to 12 and narrowed past them. The two ties of a guess, the six ends and the upkeep are kept. |
| `human-solving` | `Simulate`, `Assess` | All of it: chance, the sheet against the mind, looking without finding, elementary reasoning, error, attempts and an assessment. | Kept whole. It has no `OpenGrid`, `Look` or `TakeStep` and no `technique/Grid`, so it has no statement of looking to lose. |

Against the four criteria:

1. **The list is the same every time.** Met for ranks 1 to 12. It was also so for ranks
   13 to 22, 28 and 29 once the role was added to the witness, which the specification
   does not yet have. For ranks 23 to 27 the order is not total over deductions: two
   proofs of one output are equal in it until proofs are ordered. For ranks 1 to 12
   the order is total by construction, the
   prototype gave the same steps twice on all 226 grids, and two programs written apart
   gave the same list on 4,205 states.
2. **It is short enough to draw up at every look.** Met for ranks 1 to 12: at most 188
   entries and 6,683 patterns measured, against a bound of 9,691. Met for ranks 13 to
   22, 28 and 29 under two provisional readings: they add at most 24 entries a look,
   and at most about 111,500 patterns by a count that prunes nothing, against a bound
   of 177,402. Not argued for ranks 23 to 27 until the three readings of step 5 are
   settled.
3. **Nothing a consumer is owed is lost.** Met beneath the models; not met in their
   place. A price, an escalation, a guess and its repair, and a distribution of
   attempts are each owed to someone and none is on the list.
4. **The specifications get shorter.** Not met by the change proposed, which states
   the order and nothing else once: `technique.allium` gains the order, the entry and
   the list, by an estimate of 40 to 60 lines, and the three models lose a sentence
   and a comment or two each. A wider variant, nearer the starting recommendation,
   would also state once `sees`, `is_in_order` and `is_preferred` (about 18 lines that
   lapse repeats from reach) and the effect of applying a deduction (13 lines that
   effort repeats, and lapse in part): some 30 to 40 lines out against the same
   lines in, so about even. Both figures are estimates; neither variant was drafted
   in full. In place of the models the specifications would lose three files and
   everything in the table's third column.

**The verdict: adopt beneath, and more narrowly than the starting recommendation.**
State one total order over a grid's deductions in `technique.allium`, with the entry
and the list as what a caller reads; have `reach`, `effort` and `lapse` take the first,
in that order, of whatever each already prefers. For the first twelve techniques that
removes the one freedom the three leave to the implementation and makes two
implementations agree; past them the freedom is narrowed to deductions equal in the
order and is not removed. It gives S07 and S08
the list they index into, and costs nothing the measurements can see. It does not make
the specifications shorter and it does not replace a model. The three statements of
looking stay, because what they share word for word is small and what differs is each
model's point; the one further thing that could be stated once is the effect of
applying a deduction (13 lines in reach and in effort), and it is left to the
maintainer whether that is worth a change.

Where the starting recommendation is corrected: `reach.allium`'s run does not become
"take the first" outright, since fixation still chooses among those in order; `effort`
keeps its own choice; and `human-solving` loses nothing, having no such statement.

**Step 7. The follow-up.**

Specification changes to make first, each through the `spec-change` skill, in this
order. The text is proposed; the checker and the skill shape the final form.

*`technique.allium`.*

- Excludes, `technique:55-56`, becomes: "How a grid is stored, how deductions are found
  quickly, and, past the first twelve techniques, the order of deductions that share a
  technique and an output."
- Beside `Grid.deductions`, a statement of the order:

  ```text
  -- The grid's deductions in one order, the same every time: by the rank of
  -- the technique; then a placement before strikes; then by what is placed
  -- or struck, a placement by row, column and digit, strikes as the list of
  -- their positions and digits, position by position; then by witness: the
  -- kind of unit (none, row, column, box), its index, the cells and the
  -- digits. No two deductions of the first twelve techniques are equal in
  -- it. Past them, deductions of one technique and output are equal in it
  -- until the witness of each technique is ordered (open questions below).
  in_order: ordered(deductions)

  -- first is a black box: the least of the collection in this order; where
  -- several are equal in it, one of them, the same whenever the collection
  -- is the same, and which is the implementation's. For the first twelve
  -- techniques no two are equal, so there it leaves nothing open.
  ```

  The guarantee is therefore normative for ranks 1 to 12 alone. Past them the models
  keep, among deductions equal in the order, the freedom they have today, and each
  family loses it when its witness is ordered: by the role of "Step 5, extended" for
  ranks 13 to 22, 28 and 29, and by an order on proofs for ranks 23 to 27.

- A value for what a caller reads, and the list:

  ```text
  -- What a caller is shown: a technique, what it places or strikes, and one
  -- witness. Of the deductions a profile sees, those with one technique and
  -- one output are one entry, which keeps the first of them in the order.
  -- Repeats are removed after the profile hides and never before, so that
  -- no entry is hidden behind a witness the player cannot take in.
  value Entry {
      technique: Technique
      placement: Placement?
      strikes: Set<Strike>
      unit: UnitRef?
      cells: Set<sudoku/Position>
      digits: Set<Integer>
  }

  -- listed is a black box, pinned by OneEntryForEachTechniqueAndOutput: of
  -- the grid's deductions in order that the profile sees, the first with
  -- each technique and output, as an entry, in that order.
  -- listed(grid, profile) -> List<Entry>
  ```

  Where `listed` lives is the follow-up's to shape: `Grid` has no profile and `Profile`
  is a value, so it takes both. `Catalogue` exposes `grid.in_order` beside
  `grid.deductions`, and the comment on `filter` says it keeps the order.

- Two guarantees in `Catalogue`: `OneOrder` ("The same digits, candidates, uniqueness
  premise and catalogue limits give the same deductions in the same order, and no two
  of the first twelve techniques are equal in it.") and `OneEntryForEachTechniqueAndOutput` ("The same grid and the
  same profile give the same list; no two entries share a technique and an output; two
  techniques with one output keep an entry each.").
- Open questions. Three from step 5: whether a chain may pass through a candidate
  twice, and what x_chain's and xy_chain's minimums count; what remote_pair's output
  is; the order on paths and which witness `holds` reads. Until they are answered the
  witness order is stated for ranks 1 to 12 and the order by rank and output for every
  rank. Two more are the readings of "Step 5, extended" (what "other cells" excludes,
  on which w_wing's entries depend; whether a colouring's wrap and trap are one
  deduction), with the role that orders witnesses past rank 12. The last: what full_unit and the hidden subsets license on a grid that
  contradicts itself, which `lapse.allium` can reach. On a unit with one open cell and
  a repeated digit the prototype and the independent search read full_unit
  differently, and where a hidden subset's digits have fewer places than digits the
  predicate holds of several sets of cells. The order was checked on true grids only.

*`reach.allium`.*

- Excludes, `reach:41-43`, is narrowed to deductions equal in `technique.allium`'s
  order, which the first twelve techniques never are.
- `reach:238` becomes `let taken = first(filter(run.grid.in_order, x =>
  run.is_preferred(x)))`; the comment at `reach:236-237` says that `first` of nothing
  is nothing, and the one at `reach:244-246` describes `first` and `filter`.
- `reach:429` narrows "the implementation's choice among tied deductions" to
  deductions equal in the order. It is not deleted: past rank 12 two witnesses of one
  output differ in cells and digits, which a fixation reads (`reach:119-127`), so the
  choice among them can still change a run.
- A new open question: with the first taken, an unsystematic, unfixed player takes
  what a systematic one takes; is `unsystematic` kept for the players a fixation
  distinguishes, or dropped?

*`effort.allium`.* Excludes, `effort:44`, is narrowed as reach's is; `effort:185` takes the
first in order of the cheapest eligible, and the comments at `effort:180` and
`effort:191-193` follow it. `effort:316` stays open and gains a sentence:
the order now says which of the equally cheap is taken.

*`lapse.allium`.* Excludes, `lapse:57-60`, keeps the two ties of the guess and narrows
the first clause as reach's is; `lapse:288` as reach's line; the comment at `lapse:305-306` names
`solver.allium`'s `chosen` directly.

*Not changed:* `solver.allium`, `human-solving.allium`, `generation.allium`.
`docs/explanation/puzzle-design.md` L914 is corrected in the same change, and its line
citations into the four files are refreshed.

The build ticket takes `T35`. `rg -n 'T2[0-9]|S0[5-9]' tickets/`, as step 7 has it,
cannot see ids past `T29` and `S09`. On this branch `T33` and `S10` are the highest in
use, and a search of the ticket text on every branch (`git grep -E 'T3[4-9]|S1[1-9]'`
over every ref) finds `T34` claimed by the unmerged notes of S10, for the API
reference's pages and diagrams. It is not created here: this ticket's Files touched
lists no file for it.

**T35, drafted for `tickets/T35-deductions-in-one-order.md`.**

```yaml
---
id: T35
title: "Specification: a grid's deductions in one order, and the first is taken"
status: open
depends_on: [S06]
parallel_with: []
branch: ticket/t35-deductions-in-one-order
estimated_size: M
---
```

> **Context.** S06 (hand-back notes) found that `reach`, `effort` and `lapse` each leave
> to the implementation which of several equal deductions is taken, that published
> solvers leave it to the order of their loops, and that a total order over a grid's
> deductions costs nothing measurable for ranks 1 to 12: at most 133 entries and 6,683
> patterns examined a look over 226 grids. Its bounds are argued for every rank but 23
> to 27, which wait on open questions this ticket adds; ranks 13 to 22, 28 and 29 were
> also measured, under two readings this ticket adds as open questions. The order is
> made normative for ranks 1 to 12 only. No technique, catalogue or model exists in
> `crates/pawdoku/src` yet, so the change is to the specifications and to one
> explanation page.
>
> **Goal.** `technique.allium` states the order over a grid's deductions, the entry and
> the list a profile sees, as S06's step 7 proposes; `reach`, `effort` and `lapse` take
> the first in that order of what each prefers; the sentences that left the choice
> among deductions to the implementation are narrowed to ranks past 12. `just check-specs` and `just check` are green.
>
> **Non-goals.** Rust. Dropping `Order`, `Fixation` or any model. Changing which
> deduction `effort` prefers. Answering the three open questions on chains: they are
> added, not settled. Stating once the effect of applying a deduction.
>
> **Files touched.** `docs/specs/technique.allium`, `docs/specs/reach.allium`,
> `docs/specs/effort.allium`, `docs/specs/lapse.allium`,
> `docs/explanation/puzzle-design.md` (L914 and the line citations into those four),
> the ticket and the index.
>
> **Steps.**
>
> 1. Read S06's hand-back notes, steps 3 and 7, and the `spec-change` skill.
> 2. Change `technique.allium` first: the Excludes sentence, the order, `Entry`, the two
>    guarantees and every open question step 7 lists (the three on chains, the two
>    readings of the wider repertoire with the role, and the grid that contradicts
>    itself). Say whether `catalogue_version` is
>    raised, and why.
> 3. Change `reach.allium`, then `effort.allium`, then `lapse.allium`, one commit each.
> 4. Correct `docs/explanation/puzzle-design.md` L914 and refresh its line citations.
> 5. Check the order by hand against S06's worked look: on the born-kept opening of the
>    beginner study puzzle the first entry is cross_hatch placing 6 at r1c9, and
>    cross_hatch and hidden_single each keep an entry for that placement.
>
> **Acceptance criteria.** For the first twelve techniques no specification leaves a
> tie among deductions to the implementation, and the order is stated so that no two
> of their deductions are equal. Past them each specification says that only
> deductions equal in the order are still the implementation's, and what would order
> them. The list is drawn from what the profile sees, with repeats removed after.
>
> **Verification.** `just check-specs` and `just check`, both green; and
> `rg -n "equal deductions|equally preferred deductions|equally cheap deductions|tied deductions" docs/specs/ docs/explanation/puzzle-design.md`,
> which finds six lines today; after the change each line it finds speaks only of
> deductions equal in the order.
>
> **Open points.** Whether `unsystematic` is kept. Whether fixation reads an entry's
> witness or its output. Whether the effect of applying a deduction is stated once in
> `technique.allium`.

**What S07 needs for a rebuild.** `ai_tmp/s06/` stays in this worktree. A rebuild is
checked against the following.

- *The list and its inputs, the total order, the rule for repeated witnesses, the
  counters, the profile and the run:* steps 3 and 4 above. A reviewer rebuilt ranks 1
  to 12 from those two steps and `technique:313-438` alone and reproduced the worked
  look and all 226 rows below. Two
  readings the definitions leave open were never exercised, because they arise only
  on a grid that contradicts itself: a unit with one open cell and a repeated digit
  (the prototype placed the least missing digit; the independent search listed
  nothing), and a hidden subset whose digits have fewer places than digits (the
  predicate then holds of several sets of cells).
- *The grids.* Fifteen can be rebuilt offline: the four study puzzles, the eight
  derived from them and the three named ones, whose rows below are the check that
  needs no network. The other 211 are a file, a line range and a hash, and fetching
  them again is a separately authorised action. They are not copied here, since
  whether this repository may carry other people's collections is not a spike's to
  decide. The four study puzzles are in `docs/explanation/puzzle-design.md`. The
  derived grids follow from the rule in step 4; the givens taken out, in the order
  taken, were:

  ```text
  derived_beginner_min        r1c3 r1c5 r2c6 r3c7 r3c9 r4c8 r5c6 r7c6
  derived_beginner_minrev     r9c6 r9c2 r8c8 r8c4 r7c6 r7c3 r5c6 r4c8 r2c6 r1c5
  derived_intermediate_min    r1c2 r1c5 r3c3 r3c4 r4c9 r5c7 r6c7
  derived_intermediate_minrev r9c8 r9c6 r8c5 r7c6 r7c1 r6c7 r6c3 r3c3
  derived_advanced_min        r1c1 r1c7 r3c5 r4c6 r5c6
  derived_advanced_minrev     r5c8 r4c6 r3c5 r1c7 r1c1
  derived_expert_min          r1c4 r3c8
  derived_expert_minrev       r3c8 r1c4
  ```

  The fetched files, with their sha256 as fetched on 2026-10-03:

  ```text
  https://raw.githubusercontent.com/t-dillon/tdoku/master/data.zip
    9be0601c721ac4e702e3fe097576f025fcb99b216aabfe9dbea37cac43e6bc4f
    data/puzzles2_17_clue  lines 5-104  (lines 1-4 are comments)
    2cf268735dfd40d32269b5f20ebbea067037e5976b4d016ce0b4d3c0a50deb30
    data/puzzles1_unbiased  lines 1-50
    07a35f8a9cb394315c0b85bc73fea52ce552f4181acdcd0d550aace171498f40
  http://magictour.free.fr/top1465  lines 1-50
    32837f38ece94e75678deadbe256aeafda704c8d4c2f8b5095a630ce2d0114d3
  https://norvig.com/hardest.txt  all 11 lines
    398e1e5df4f50723078e3b510e00ea7eb9e104c521a9a186b4c20cd066c0459f
  ```

  The three named grids, as the page's links give them:

  ```text
  inkala_2012               8..........36......7..9.2...5...7.......457.....1...3...1....68..85...1..9....4..
  sudokuwiki_unsolvable_28  6....894.9....61...7..4....2..61..........2...89..2.......6...5.......3.8....16..
  sudokuwiki_unsolvable_49  ..28......3..6...71......4.6...9.....5.6....9....57.6....3..1...7...6..84......2.
  ```

- *A worked look.* `study_beginner`, look 0: 128 catalogue deductions, 103 entries, 16
  entries at the first yielding rank (rank 2), 2,840 patterns for the whole list and
  174 to the first rank. The first eight steps place 6 at r1c9, 9 at r2c3, 5 at r2c4,
  4 at r1c6, 4 at r2c1, 8 at r2c7, 8 at r1c1 and 3 at r1c2.
- *Each grid's counts,* under the pinned profile. Columns: name, givens, end, steps,
  hardest rank (0 where no step was taken), then sums over the run's looks of
  catalogue deductions, list entries, entries at the first yielding rank, patterns
  examined for the whole list, and patterns examined to the first yielding rank.

  ```text
  study_beginner 32 solved 49 2 5841 4205 375 55864 3795
  derived_beginner_min 24 solved 57 2 2933 2275 221 83894 5490
  derived_beginner_minrev 22 solved 59 3 2513 1926 212 104461 5775
  study_intermediate 32 solved 54 6 2554 1870 187 56794 6047
  derived_intermediate_min 25 stalled 13 5 196 174 38 48206 7252
  derived_intermediate_minrev 24 solved 63 6 1992 1439 147 110300 8196
  study_advanced 29 stalled 22 6 507 403 59 53242 5167
  derived_advanced_min 24 stalled 19 6 306 267 50 64752 8117
  derived_advanced_minrev 24 stalled 19 7 317 280 49 68753 9680
  study_expert 25 stalled 32 8 866 738 73 87246 9598
  derived_expert_min 23 stalled 33 8 875 746 74 102798 10232
  derived_expert_minrev 23 stalled 33 8 875 746 74 102798 10232
  seventeen_001 17 solved 76 7 2264 1665 153 175608 15155
  seventeen_002 17 solved 64 3 2296 1725 139 128888 7119
  seventeen_003 17 solved 64 3 1922 1558 126 141422 6858
  seventeen_004 17 solved 64 2 1579 1206 154 129634 6957
  seventeen_005 17 solved 64 2 2173 1557 210 121982 7137
  seventeen_006 17 solved 64 2 2305 1703 173 129130 7047
  seventeen_007 17 solved 70 7 1985 1500 173 138308 8874
  seventeen_008 17 solved 64 2 2310 1707 170 130076 7047
  seventeen_009 17 solved 64 3 1729 1389 101 138126 7038
  seventeen_010 17 solved 64 2 2385 1826 161 130580 6882
  seventeen_011 17 solved 64 2 2442 1893 161 129238 6876
  seventeen_012 17 solved 72 5 2525 2023 221 139584 11130
  seventeen_013 17 solved 71 5 2389 1899 204 140513 10667
  seventeen_014 17 solved 64 3 1989 1543 137 131538 7457
  seventeen_015 17 solved 70 7 2009 1616 158 139402 9230
  seventeen_016 17 solved 70 7 2005 1613 155 139866 9230
  seventeen_017 17 solved 64 3 1893 1453 120 141700 7107
  seventeen_018 17 solved 64 2 2463 1802 177 113232 7086
  seventeen_019 17 solved 64 2 2526 1853 173 113488 7086
  seventeen_020 17 solved 68 5 2263 1751 177 139240 8688
  seventeen_021 17 solved 64 2 2311 1785 177 131008 6762
  seventeen_022 17 stalled 39 6 874 731 78 139216 10516
  seventeen_023 17 solved 68 5 1941 1467 175 141150 8424
  seventeen_024 17 stalled 39 8 827 686 90 128678 12790
  seventeen_025 17 solved 70 7 1990 1555 152 132250 9701
  seventeen_026 17 solved 64 2 1835 1446 130 132394 6792
  seventeen_027 17 solved 64 2 2963 2085 234 134454 6876
  seventeen_028 17 solved 64 2 2029 1552 161 125696 6936
  seventeen_029 17 solved 64 2 2025 1511 134 126748 6951
  seventeen_030 17 solved 64 2 2420 1706 172 124372 7014
  seventeen_031 17 solved 64 2 2059 1585 142 125594 7008
  seventeen_032 17 solved 64 2 2720 1951 194 124338 7017
  seventeen_033 17 solved 64 2 2509 1892 215 131860 6879
  seventeen_034 17 solved 66 5 2727 1970 240 133890 8186
  seventeen_035 17 solved 66 5 2696 2005 202 135202 8200
  seventeen_036 17 solved 69 5 1850 1439 159 132345 9635
  seventeen_037 17 solved 71 5 2472 1799 203 153977 11016
  seventeen_038 17 solved 64 3 1865 1410 127 141896 6784
  seventeen_039 17 solved 64 2 2378 1791 213 131512 6969
  seventeen_040 17 solved 64 2 2436 1812 218 129824 7101
  seventeen_041 17 solved 64 2 2244 1848 149 134162 6888
  seventeen_042 17 solved 65 5 2040 1564 180 123156 7510
  seventeen_043 17 solved 64 3 2538 1859 186 130250 7022
  seventeen_044 17 solved 71 5 2274 1691 188 136758 10031
  seventeen_045 17 solved 66 6 1664 1263 168 126016 7965
  seventeen_046 17 solved 64 3 1887 1509 160 124526 7041
  seventeen_047 17 solved 64 3 2106 1629 191 119916 7209
  seventeen_048 17 solved 64 3 2103 1518 180 121162 7173
  seventeen_049 17 solved 70 6 1966 1499 171 152704 10470
  seventeen_050 17 solved 64 2 2131 1662 170 123464 6876
  seventeen_051 17 solved 71 6 2486 1910 195 142549 9964
  seventeen_052 17 solved 64 2 2387 1732 190 131504 7008
  seventeen_053 17 solved 65 5 1970 1415 142 139548 7683
  seventeen_054 17 solved 76 8 1848 1381 196 172391 13540
  seventeen_055 17 stalled 45 7 1038 841 67 168142 16729
  seventeen_056 17 solved 64 3 1747 1344 132 131146 6942
  seventeen_057 17 solved 64 3 1897 1385 154 130674 6966
  seventeen_058 17 solved 69 5 1936 1492 171 121086 8982
  seventeen_059 17 stalled 42 8 790 667 73 147149 12631
  seventeen_060 17 solved 67 5 2234 1719 165 118932 8117
  seventeen_061 17 stalled 42 8 812 689 72 149150 14143
  seventeen_062 17 solved 70 5 2006 1506 172 151148 9710
  seventeen_063 17 solved 65 5 1878 1402 144 123102 7462
  seventeen_064 17 solved 74 7 2329 1804 168 175330 13611
  seventeen_065 17 solved 64 3 1824 1353 169 121132 7118
  seventeen_066 17 solved 65 5 1739 1275 138 126167 7568
  seventeen_067 17 solved 69 5 2305 1775 160 137241 9343
  seventeen_068 17 solved 64 2 2239 1617 185 118938 7068
  seventeen_069 17 solved 64 3 2100 1492 182 125292 7128
  seventeen_070 17 solved 64 2 2426 1819 214 122518 7104
  seventeen_071 17 solved 65 5 1623 1189 155 121111 7873
  seventeen_072 17 solved 65 5 1806 1344 168 121341 7950
  seventeen_073 17 solved 65 5 1808 1344 167 122301 7863
  seventeen_074 17 solved 64 3 1899 1397 145 128498 6945
  seventeen_075 17 solved 64 2 2208 1563 189 118372 7098
  seventeen_076 17 solved 64 3 1748 1390 172 134318 7088
  seventeen_077 17 solved 66 5 2390 1852 204 141790 7779
  seventeen_078 17 solved 73 8 2182 1720 146 178369 13472
  seventeen_079 17 solved 64 3 1772 1335 150 120306 7153
  seventeen_080 17 solved 64 3 2191 1554 185 121434 7067
  seventeen_081 17 solved 64 2 1885 1383 163 122726 6999
  seventeen_082 17 solved 64 2 2390 1697 191 123376 6954
  seventeen_083 17 solved 66 5 1864 1382 146 127146 7604
  seventeen_084 17 solved 68 5 2578 1789 157 138728 9069
  seventeen_085 17 solved 64 2 1946 1445 159 122908 6981
  seventeen_086 17 solved 64 2 1811 1346 165 124212 7050
  seventeen_087 17 solved 65 5 1582 1192 141 124142 7849
  seventeen_088 17 solved 70 5 2001 1515 182 146478 9878
  seventeen_089 17 solved 64 2 2037 1473 153 126042 6882
  seventeen_090 17 solved 64 2 2214 1581 188 122610 7053
  seventeen_091 17 solved 64 3 2114 1489 177 125510 7053
  seventeen_092 17 solved 64 3 1850 1364 154 126178 7111
  seventeen_093 17 solved 64 2 1972 1473 153 128338 6978
  seventeen_094 17 solved 65 5 1562 1180 138 128115 7825
  seventeen_095 17 solved 64 3 2214 1661 165 123610 7102
  seventeen_096 17 solved 64 3 2532 1823 193 125292 7046
  seventeen_097 17 solved 64 2 2209 1555 192 122044 7083
  seventeen_098 17 solved 64 2 2167 1534 169 140550 6852
  seventeen_099 17 solved 64 3 1846 1339 153 123512 7121
  seventeen_100 17 solved 67 5 2473 1784 200 138699 8427
  unbiased_001 25 solved 58 5 2452 1835 135 93866 6273
  unbiased_002 25 stalled 17 7 301 218 46 56775 6490
  unbiased_003 24 solved 57 3 2301 1714 134 95696 5585
  unbiased_004 27 solved 54 3 1762 1312 109 72679 5098
  unbiased_005 26 stalled 24 8 385 309 42 67680 10189
  unbiased_006 25 solved 56 3 2138 1513 177 95240 5046
  unbiased_007 28 stalled 19 7 350 304 24 48284 5604
  unbiased_008 27 solved 54 2 2692 1730 193 70867 4773
  unbiased_009 25 solved 56 3 2347 1781 177 85190 5166
  unbiased_010 24 stalled 8 5 133 99 26 31693 4769
  unbiased_011 26 stalled 16 7 201 178 21 59383 8646
  unbiased_012 27 solved 60 8 2378 1772 154 84034 7788
  unbiased_013 25 stalled 11 11 98 95 21 46484 12226
  unbiased_014 25 solved 62 7 2606 1937 145 103062 7763
  unbiased_015 25 stalled 41 5 1006 832 107 82582 7302
  unbiased_016 25 stalled 36 3 883 715 80 78927 5201
  unbiased_017 25 solved 58 7 2181 1646 190 81212 5973
  unbiased_018 25 stalled 22 7 448 405 52 63395 8703
  unbiased_019 25 solved 65 7 3824 2724 197 112409 11354
  unbiased_020 26 solved 58 8 1640 1189 108 81535 7089
  unbiased_021 26 solved 55 2 1870 1431 144 79167 4953
  unbiased_022 26 stalled 14 6 203 177 30 42371 5945
  unbiased_023 26 stalled 9 5 121 103 13 34376 5261
  unbiased_024 25 solved 56 3 1522 1149 107 80982 5184
  unbiased_025 25 solved 56 2 3492 2545 206 80896 5178
  unbiased_026 26 stalled 20 7 542 401 67 58791 7268
  unbiased_027 25 solved 56 3 2209 1600 157 72648 5640
  unbiased_028 26 solved 55 3 1339 975 108 72227 5013
  unbiased_029 27 stalled 9 9 95 91 22 35938 8387
  unbiased_030 24 solved 57 3 2214 1647 145 82858 5491
  unbiased_031 26 stalled 27 5 1170 818 111 69864 5241
  unbiased_032 25 stalled 30 7 864 718 80 70628 8180
  unbiased_033 27 stalled 13 8 268 216 20 45057 6884
  unbiased_034 26 stalled 24 7 397 346 46 70682 12148
  unbiased_035 25 stalled 18 10 254 215 27 59674 8294
  unbiased_036 25 stalled 21 3 536 478 47 64507 5043
  unbiased_037 27 stalled 28 7 845 725 83 61480 5203
  unbiased_038 25 stalled 30 2 863 685 81 67188 5213
  unbiased_039 24 stalled 24 6 503 402 65 64827 7023
  unbiased_040 27 solved 54 2 2617 1920 182 85897 4650
  unbiased_041 26 stalled 28 6 761 651 57 73862 5902
  unbiased_042 25 solved 59 5 1379 1043 124 88285 6539
  unbiased_043 26 solved 59 10 1675 1215 131 89147 8557
  unbiased_044 25 stalled 10 8 104 92 26 42486 9262
  unbiased_045 25 solved 56 3 2169 1606 138 74376 5346
  unbiased_046 26 stalled 35 2 1182 837 135 72084 4770
  unbiased_047 24 stalled 19 9 361 317 34 59591 8266
  unbiased_048 26 solved 55 2 3849 2855 262 71529 5058
  unbiased_049 26 solved 55 2 2229 1518 188 75023 4986
  unbiased_050 26 stalled 21 7 458 372 49 54106 5485
  top1465_001 18 stalled 20 8 276 236 48 115380 19901
  top1465_002 18 stalled 7 8 32 30 13 46425 10741
  top1465_003 18 stalled 7 8 32 30 13 46425 10741
  top1465_004 18 solved 84 8 2614 2044 173 243341 24534
  top1465_005 18 solved 75 8 2527 2011 198 182505 18043
  top1465_006 18 solved 75 8 2631 2104 198 182641 18043
  top1465_007 18 stalled 7 8 32 30 13 46469 10741
  top1465_008 18 solved 87 8 3235 2392 218 243494 26484
  top1465_009 18 solved 85 8 2718 2018 210 227595 24979
  top1465_010 18 solved 87 8 2796 2050 209 240876 26497
  top1465_011 18 solved 83 10 3095 2352 160 238790 25554
  top1465_012 18 stalled 19 10 164 147 38 105849 19321
  top1465_013 18 stalled 20 8 276 232 47 114608 19487
  top1465_014 18 solved 85 9 2181 1686 181 210780 24607
  top1465_015 18 solved 73 7 1902 1449 142 189339 14272
  top1465_016 18 stalled 22 10 219 190 45 117674 19790
  top1465_017 18 solved 86 8 2975 2083 186 230042 26893
  top1465_018 18 solved 82 8 2193 1681 162 214180 20835
  top1465_019 18 solved 86 8 2834 1998 183 229566 26887
  top1465_020 18 solved 83 8 2954 2159 207 220272 23711
  top1465_021 18 stalled 15 8 103 94 31 80543 12214
  top1465_022 18 solved 83 8 3933 2816 243 222098 23627
  top1465_023 18 stalled 18 8 183 161 29 105666 19102
  top1465_024 18 stalled 7 8 32 30 13 45953 10669
  top1465_025 18 stalled 21 8 280 233 47 120679 20412
  top1465_026 18 solved 85 8 2321 1728 171 233423 24177
  top1465_027 18 solved 85 8 3141 2223 214 233790 24172
  top1465_028 18 solved 85 8 2375 1721 170 235588 24148
  top1465_029 18 stalled 18 8 184 161 29 105666 19102
  top1465_030 18 stalled 12 7 131 112 16 72718 12361
  top1465_031 18 solved 83 8 3623 2645 223 221242 23645
  top1465_032 18 solved 74 10 3330 2642 183 181632 19163
  top1465_033 18 solved 77 8 2577 1910 175 192348 18179
  top1465_034 18 stalled 21 8 280 233 47 120679 20412
  top1465_035 18 stalled 20 8 303 253 44 109497 15728
  top1465_036 18 stalled 18 8 229 198 64 83244 11682
  top1465_037 18 solved 84 9 2739 2031 206 214940 25533
  top1465_038 18 stalled 22 8 406 352 52 115873 16045
  top1465_039 18 stalled 23 8 405 339 40 119380 19005
  top1465_040 18 solved 84 8 2941 2175 221 221081 22112
  top1465_041 18 stalled 20 8 222 195 46 100996 14232
  top1465_042 18 stalled 11 8 64 57 22 67538 13762
  top1465_043 18 stalled 24 8 328 295 66 130767 19904
  top1465_044 18 stalled 39 8 510 445 76 182266 24621
  top1465_045 18 stalled 40 8 531 453 76 177232 24192
  top1465_046 18 stalled 17 8 188 175 29 99920 15870
  top1465_047 18 stalled 39 8 589 494 78 179783 23118
  top1465_048 18 stalled 23 8 286 273 50 110046 15904
  top1465_049 18 stalled 19 8 237 205 46 109441 20177
  top1465_050 18 solved 85 8 2616 1937 220 221956 22861
  norvig_hardest_01 22 solved 66 5 2233 1668 166 112632 9285
  norvig_hardest_02 23 stalled 3 8 22 14 4 16239 5827
  norvig_hardest_03 26 stalled 11 8 153 117 25 35494 6362
  norvig_hardest_04 24 solved 62 5 1334 942 120 94396 8070
  norvig_hardest_05 22 solved 59 3 2186 1652 148 112635 5761
  norvig_hardest_06 23 stalled 1 2 5 3 1 8797 4503
  norvig_hardest_07 28 stalled 27 7 537 454 59 59821 9912
  norvig_hardest_08 28 stalled 20 8 693 607 77 49529 5062
  norvig_hardest_09 24 stalled 8 5 113 71 20 35236 5886
  norvig_hardest_10 28 stalled 9 6 99 83 20 27284 4540
  norvig_hardest_11 22 stalled 8 9 46 44 13 41749 10133
  inkala_2012 21 stalled 0 0 0 0 0 5191 5191
  sudokuwiki_unsolvable_28 22 stalled 2 5 6 6 3 15126 6158
  sudokuwiki_unsolvable_49 22 stalled 0 0 0 0 0 4726 4726
  ```

**Step 8. Verification.** Run on 2026-10-03, after the edits:

```text
$ rg -n 'SameGridSameDeductions|ExtraCandidatesHideAndNeverMislead' docs/specs/technique.allium
883:    @guarantee ExtraCandidatesHideAndNeverMislead
891:    @guarantee SameGridSameDeductions
$ rg -n 'SameInputSameRun|LimitsOnlyHide' docs/specs/reach.allium
365:    @guarantee SameInputSameRun
369:    @guarantee LimitsOnlyHide
$ rg -n '^rule (OpenGrid|Look|TakeStep)' docs/specs/
docs/specs/lapse.allium:272:rule OpenGrid {
docs/specs/lapse.allium:278:rule Look {
docs/specs/lapse.allium:349:rule TakeStep {
docs/specs/reach.allium:200:rule OpenGrid {
docs/specs/reach.allium:229:rule Look {
docs/specs/reach.allium:260:rule TakeStep {
docs/specs/effort.allium:156:rule OpenGrid {
docs/specs/effort.allium:173:rule Look {
docs/specs/effort.allium:211:rule TakeStep {
$ ls crates/pawdoku/src
board  board.rs  lib.rs  random.rs  solver  solver.rs  sudoku  sudoku.rs
$ git status --porcelain
 M tickets/README.md
 M tickets/S06-licensed-deductions-in-order.md
```

Four of the five are as expected. The `ls` is not, for the reason under Deviations.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `ticket/S06-licensed-deductions-in-order`,
  with a capital, not the `ticket/s06-licensed-deductions-in-order` the frontmatter
  names. The `branch:` field is left as written. The worktree existed before the
  ticket ran.
- **Two of the Context's facts are stale.** `crates/pawdoku/src` now holds `board`,
  `solver` and `sudoku` beside `lib.rs` and `random.rs`, so "No solver ... exists in
  code" no longer holds and the Verification's expected `ls` is not met. There is
  still no catalogue, model or generator in code, which is the Context's point.
  `SameGridSameDeductions` and the rest are as the Context quotes them.
- **Step 7's search for the next id** cannot see `T30` to `T33` or `S10`; a wider
  pattern was used and the id is `T35`.
- **The Goal's table is left as written**, as the record of the starting
  recommendation. Step 6 carries the completed table.
- **The order is stated over deductions, not only over entries.** Step 3 asks for an
  order over entries. (b) showed that repeats must be removed after the profile hides
  and that `effort` needs the deductions a profile does not see, so the proposal
  orders deductions and derives the entries. For the pinned profile the two are the
  same list.
- **A second Codex review, of the answers to the first, was answered** on 2026-10-03.
  Three of its findings were of these notes and all three held: statements that the
  tie is removed outright are qualified; the least load is said to be needed only to
  decide what a capacity hides, not of the witness kept; the drafted ticket's step
  names every open question.
- **A Codex review of both commits was asked for and answered** on 2026-10-03. Its five
  findings were checked and all five held: the order is made normative for ranks 1 to
  12 only; w_wing's entry bound is corrected; the claim that fewest links is least
  load is withdrawn; the advice on stopping at the first rank is scoped; two of four
  "readings" are restated as what the specification says.
- **Ranks 13 to 22, 28 and 29 were built and measured,** where step 4 asks for ranks 1
  to 12 and step 5 for paper. The maintainer asked for it on 2026-10-03, after the
  first commit, and chose that the open readings be picked, stated and flagged. The
  figures are in "Step 5, extended".
- **Sources read at one remove.** HoDoKu's source is a GitHub fork, not the
  SourceForge tree. Sudoku Explainer's own documentation did not resolve.
  SudokuWiki's strategies above singles run on its server. PySAT's solver page was
  read through a summarising fetch; the licence texts were read raw.

### Handed back

- **To T35** (drafted above): the specification change, which is the change to make
  first. It waits on the maintainer's word on the open points below.
- **To the owner of `docs/explanation/puzzle-design.md`:** the four study puzzles each
  have exactly one solution, so "unverified" can be narrowed to their paths. The
  advanced puzzle's X-Wing is confirmed as the page has it. The expert puzzle did not
  need its contradiction chain: ranks 1 to 22 solved it, by an xyz_wing, a w_wing and
  a skyscraper after the page's stall, so the page's claim that the repertoire stalls
  there depends on which techniques that repertoire holds. The
  intermediate puzzle's pointing is in the middle-left box (box 4), where the page
  says "lower-left box". L914 quotes a sentence T35 narrows.
- **To `effort.allium`'s next change:** its header says it reads `fatigue`
  (`effort:40`), and no rule or derived value does.
- **To S07:** the rebuild material above. The SAT oracle of (e) is the independent
  check S07's steps name; it needs `python-sat` in a scratch venv, which is a
  separately authorised install.
- **To S08:** the order its draw indexes into is the list's, and under the pinned
  profile only the first is ever taken by a run forward.
- **To whoever builds the catalogue in Rust:** for a systematic, unfixed run, and only
  after the profile has hidden what it hides, the first is found by stopping at the
  first rank that yields. Drawing up the whole list cost eleven times as many patterns
  under ranks 1 to 12 and sixty-three times as many under the wider profile, and is
  for callers that show it. The rule does not serve the other consumers: `effort`
  takes the cheapest across ranks, and a fixation in an unsystematic player can
  prefer a later rank.

### Open points settled

None is settled here: each is the maintainer's. The recommendations, with the evidence:

- **Replace price and guesses, or only the statements of looking?** Only the tie in
  each statement of looking. A price, an escalation, a guess and a repair are not on
  the list (step 6), and even the three statements of looking share too little word
  for word to be worth merging (a).
- **Do an unsystematic order and a fixation survive?** Keep both for now, as choices
  among the deductions in order: the change then decides nothing about players. But
  note that `unsystematic` without a fixation becomes the same player as `systematic`
  (b), and that taking a random entry raised the hardest rank on 216 to 218 of 226
  grids (d), so an unsystematic rating says more about the choice than the puzzle.
  Dropping `unsystematic` is a fair next question and T35 adds it to `reach.allium`.
- **Is the total order `technique.allium`'s or `reach.allium`'s?** `technique.allium`'s.
  It is a property of a grid's deductions; three models, S07 and S08 read it; and
  `effort` needs it over deductions `reach` never sees.

## Open points

None is settled; the recommendations are under "Open points settled" above, and the
questions stay here as written.

- Whether the list replaces `effort.allium`'s price and `lapse.allium`'s guesses, or
  only the three statements of looking; the recommendation says, the maintainer
  decides.
- Whether an unsystematic order and a fixation survive as orderings of the same list
  or are dropped; if dropped, `technique.allium`'s `Order` and `Fixation` go with them.
- Whether the total order is `technique.allium`'s, beside the ladder, or
  `reach.allium`'s, beside the choice it already makes.
