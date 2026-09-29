---
id: S08
title: "Spike: removal by undoing deductions, against removal and a rating"
status: open
depends_on: [S06, S07]
parallel_with: []
branch: ticket/s08-removal-by-undoing
estimated_size: L
---

# S08: Spike: removal by undoing deductions, against removal and a rating

## Context

`docs/specs/generation.allium` is a skeleton: a solution grid drawn through the
randomness boundary, givens removed one orbit of a symmetry scheme at a time, and a
candidate set of givens kept only when `solver.allium`'s verdict is one and
`reach.allium`'s `Rate` for the target profile ends solved within the technique
contract. It says which givens are acceptable and nothing of which to remove next.
T20, drafted in `tickets/T19-puzzle-design-objectives.md`, is to write its triggers.
This spike weighs one answer to "which next" before T20 is written.

The proposal, as the maintainer made it on 2026-09-29 and in this project's words. It
came in two parts. The first:

> 1. Draw a solution grid.
> 2. Choose the profile to remove as.
> 3. Draw a cell, then draw a technique from the profile's repertoire, and undo that
>    technique's deduction there, with further draws where the technique needs them
>    (the other cells of its pattern, say).
> 4. After every removal, ask for the verdict. If it is many, the digit goes back.
> 5. After a stated count of removals, put the givens left to a run by another model
>    of a player, under a profile of like skill. If the puzzle asks too little of that
>    profile, go back to 3 and remove more.

The second joins it to S06 and S07:

> With S07's list of undoings, removal is a draw from that list. A run forward takes
> the first deduction on S06's list and is the same every time; removal draws from the
> list of undoings and is as the seed has it.

Facts to start from, each verified at execution against the specifications as they
stand on `main` and against S06's and S07's hand-back notes:

- **A technique undone is not a technique needed.** A run takes the lowest deduction
  anywhere on the grid, so it may never meet the deduction that was undone.
  `generation.allium`'s open questions on the technique contract's shape ("a solved run
  does not show that a required technique was needed, only that it was used") and on
  shortcut suppression already say so. So the run of part 5 is the acceptance and not
  a last look, and the technique undone is at most a ceiling on what the run asks.
- **Drawing a cell and then a technique mostly finds nothing.** On a full grid no cell
  has a candidate, so only singles can be undone. Drawing from S07's list, as the
  second part says, draws only from what can be undone.
- **The verdict can be many part of the way.** `solver.allium` judges the givens alone.
  Partway through removal, the grid still has strikes that no deduction from the givens
  left has earned yet: they are assumed, not derived. So the givens left can have
  more than one solution after undoing any technique, not only a uniqueness one, and
  part 4 can put a digit back at any removal. A run of licensed deductions from a
  born-kept grid to a full one keeps every solution (`DeductionsAreSound` is stated
  for any solution), so a verdict of one is owed only when removal ends on a
  born-kept grid and no uniqueness technique was undone. How often part 4 rejects on
  the way is measured, not assumed. Acceptance asks for the verdict whatever removal
  did.
- **Removal must end on givens.** A grid is born kept, so removal is finished only
  when every candidate the givens left allow is back. Removal can arrive at a grid
  where nothing more can be undone and strikes still stand.
- **More removed is not shown to be harder.** `reach.allium`'s monotonicity question is
  open, and `generation.allium` says the count of givens is "never a rating". And a
  puzzle that asks too much has no way back in the proposal but to put a given back.
- **Cells against orbits.** `generation.allium` removes an orbit at a time. The
  maintainer's decision of 2026-09-29: this spike removes one cell at a time, and says
  what removal by orbit would ask of S07's list. One cell at a time is removal under
  the scheme `none`, one of the schemes `generation.allium`'s symmetry question still
  weighs, so every figure here, and the recommendation, speak for `none` alone.
  Removal at random is measured the same way, so the comparison is like for like.
- **Draws and limits.** Every draw comes through the randomness boundary, a choice
  among `n` is `floor(u * n)` over a list in a stated order, which S06's total order
  gives, and every draw is recorded (`human-solving.allium`'s `ExactReplay`). A limit
  is `step_budget` or `candidate_limit`, figures and never a clock.
- **Words.** Candidate is two things here: `solver.allium`'s digit a cell may still
  hold, and `generation.allium`'s set of givens put to acceptance. This ticket says
  candidate for the first and candidate givens for the second. The proposal's levels
  are the research report's; the specifications say technique contract and rating.

Read first: `tickets/CONVENTIONS.md` §11; `tickets/S06-licensed-deductions-in-order.md`
and `tickets/S07-undoing-a-deduction.md` with their hand-back notes;
`docs/specs/generation.allium` whole; `docs/specs/solver.allium` and
`docs/specs/reach.allium`, their headers and guarantees;
`docs/explanation/puzzle-design.md`;
`docs/decisions/0012-generation-and-dev-time-judges.md`; the T20 draft in
`tickets/T19-puzzle-design-objectives.md` under "Handed back".

## Goal

A recommendation for T20 between removal by undoing and removal followed by a rating,
under the scheme `none`: adopt removal by undoing, adopt it as one way of choosing
what to remove, or not at all. Whether it carries over to any other scheme is stated
as a condition on step 2(e)'s answer, not measured. With it: what each yields,
measured on the same seeds and profiles, and a proposed answer to three of
`generation.allium`'s open questions (the technique contract's shape, shortcut
suppression, redundant givens).
Nothing is installed or changed outside this file.

The starting recommendation: **adopt it as a way of choosing what to remove**, under
the scheme `none` until removal by orbit is shown to work.
Acceptance stays as `generation.allium` states it, the verdict and one `Rate`; undoing
decides only which given goes next, and is kept if it reaches a contract in fewer
candidate givens than removal at random.

| Way of removing | Accepted per 100 seeds | Each counter per accepted | Candidate givens per accepted | Run's hardest step below the hardest undone | Ended with strikes standing | Givens left |
| --- | --- | --- | --- | --- | --- | --- |
| at random, then the verdict and `Rate` | | | | not applicable | not applicable | |
| by undoing, broad reading | | | | | | |
| by undoing, strict reading | | | | | | |

## Non-goals

- Writing `generation.allium`'s triggers, guarantees or fixtures, which is T20's.
- Editing a specification; proposed text goes in the hand-back notes.
- Rust in `crates/pawdoku`, a dependency, a recipe or a workflow.
- Removal by orbit beyond saying what it would ask.
- Pacing, which is T21's, and any judge, which is S05's and development-time only
  (decision 0012).
- Where puzzles are made, which stays open.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S08-removal-by-undoing.md` | ticket | the evidence, the recommendation, the revised T20 draft; `status: done` |
| `tickets/README.md` | ticket index | S08's row set to `done` |

## Steps

1. Create the worktree on `ticket/s08-removal-by-undoing` from `main` after S06 and S07
   have merged (README.md "How to pick up a ticket").

2. Verify, read-only, and record with sources and dates: (a) how published generators
   aim at a named technique, from their own documentation; (b) where the verdict is
   still owed when every deduction undone is sound, and what asking for it after every
   removal costs against asking once at acceptance; (c) what removal does on reaching
   a grid where nothing more can be undone: take the last undoing back, begin again
   from the solution grid, or put a given back; (d) what ends removal, since a stated
   count of removals takes no account of the profile; (e) what removal by orbit asks:
   an undoing for every cell of the orbit on one grid, or one after another. Fetching
   a source is separately authorised: stop and ask.

3. Build the three ways of removing in the prototype S07 left, in `ai_tmp/` or the
   session's scratch directory and never in a commit, for ranks 1 to 12. All three
   draw through one seeded stream and record their draws. Before running them, define
   what each charges, because `generation.allium`'s step (a look at a position, the
   removal of an orbit, a call to acceptance) does not reach the work removal by
   undoing adds. Count for all three: looks, cells removed, calls to acceptance, and
   the verdicts and `Rate` runs those calls make. Count for the two ways of undoing as
   well: the list of undoings drawn up (with S06's counters), the earlier grids tried
   (S07's), each verdict asked after a removal, and each digit put back.

4. Run each over the same seeds against the same technique contracts, a profile and a
   ceiling at ranks 4, 6, 8 and 12, and complete the table above.

5. Show replay: the same seed, profile and config give the same givens, run twice.

6. Write the proposed answers to the three open questions, and the revised T20 draft,
   in the hand-back notes. Every change to a specification is named as a change to
   make first.

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The counters of step 3 are defined before any figure is recorded, and the table is
  completed for the three ways of removing on the same seeds.
- The recommendation names the scheme `none` as the only one measured.
- Replay is shown with the two runs' givens quoted.
- The verdict is adopt, adopt as a way of choosing, or not at all, with the reason.
- Each of the three open questions has a proposed answer or a reason it stays open.
- The revised T20 draft is in the hand-back notes.
- `git status --porcelain` on the ticket branch lists only this file and the index.

## Verification

```sh
rg -n '^open question' docs/specs/generation.allium | cut -c1-80
rg -n 'step_budget|candidate_limit' docs/specs/generation.allium
rg -n 'T20' tickets/T19-puzzle-design-objectives.md
git status --porcelain
```

Expected: the seven open questions of `generation.allium`; both config figures; the
T20 draft under "Handed back"; two lines naming this file and `tickets/README.md`.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Who the second player of part 5 is: another profile put to `Rate`, or another of the
  four models. S06's recommendation may leave only the first.
- Whether candidate givens that ask too much are given a digit back or thrown away.
- Whether the undoings drawn are part of what replay promises, so that
  `generation_version` is raised when S07's list changes.
