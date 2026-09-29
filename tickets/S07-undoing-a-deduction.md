---
id: S07
title: "Spike: undoing a deduction, technique by technique"
status: open
depends_on: [S06]
parallel_with: []
branch: ticket/s07-undoing-a-deduction
estimated_size: M
---

# S07: Spike: undoing a deduction, technique by technique

## Context

`docs/specs/technique.allium` says when each of twenty-nine techniques holds on a grid
and what it places or strikes. Nothing says what it would be to take one back: given a
grid, which earlier grids license a deduction that leads to it. S08 proposes to make
givens that way, so whether each technique can be undone has to be settled first.

The proposal, as the maintainer made it on 2026-09-29 and in this project's words:

> For each technique in the catalogue, find out whether its deduction can be undone.
> That is, whether the list S06 states has a counterpart that runs the other way: given
> a grid and a profile, every undoing the profile's repertoire allows, each one the
> reverse of a deduction the earlier grid licenses. Whether that counterpart exists,
> for which techniques, is what has to be shown.

Facts to start from, each verified at execution against the specifications as they
stand on `main`:

- **Undoing is two operations.** A deduction places or strikes and never both
  (`ADeductionPlacesOrStrikes`). The four singles (ranks 1 to 4), `bug_plus_one`, a
  forcing chain's positive conclusion and an alternating inference chain's closure
  place; every other technique strikes. To undo a placement is to take a digit out of
  a cell. To undo a strike is to give a cell a candidate back, which changes no digit.
- **Undoing is not one grid.** Taking a digit out leaves open which candidates its cell
  and its peers hold afterwards. So an undoing is stated by what it must lead to: the
  earlier grid is any grid on which S06's list holds the deduction being undone and
  on which taking that deduction, with upkeep after a placement, gives exactly the
  later grid, digit for digit and candidate for candidate.
- **Upkeep is not a strike.** "A placed digit leaves its peers' candidates by upkeep,
  which is the keeper's and not a strike of the deduction's." Undoing a placement has
  to say what happens to the candidates upkeep took.
- **The two uniqueness techniques rest on a premise.** `unique_rectangle_type_1` and
  `bug_plus_one` hold only where `Grid.uniqueness_promised` is true, and that premise
  is "supplied by the keeper, never inferred". A grid being undone towards givens
  that have no verdict yet cannot promise it.
- **A grid is born kept.** A grid opened on a set of givens holds "every candidate the
  start allows and no other", and a placed cell keeps its digit as its one candidate.
  So undoing has reached a set of givens only when the grid is exactly the born-kept
  grid of the digits left: every open cell holds every candidate its placed peers
  allow. An undoing that leaves a strike standing has not got there.
- **Two readings of the reverse.** The broad one undoes any deduction the earlier grid
  licenses and the profile sees. The strict one undoes only the deduction a run would
  take there, the first on S06's list. The strict one makes the run forward retrace
  the undoing step for step; the broad one does not.
- **Words.** `technique.allium` has no word for this. This ticket says undoing, the
  reverse of one deduction, and earlier grid, the grid an undoing leads to. Whether
  those are the words the specifications take is an open point. Never move; never
  eliminate.

Read first: `tickets/CONVENTIONS.md` §11; `tickets/S06-licensed-deductions-in-order.md`
and its hand-back notes; `docs/specs/technique.allium` whole, its catalogue guarantees
above all (`BasicFish`, `Wings`, `SingleDigitPatterns`, `SimpleColouring`,
`RemotePairs`, `AlternatingChains`, `BoundedForcingChains`, `UniquenessPatterns`);
`docs/explanation/solving-sudoku.md`.

## Goal

A table with a row for each of the twenty-nine techniques, and a recommendation on the
list of undoings: adopt it, adopt it for a named part of the catalogue, or not at all.
Nothing is installed or changed outside this file.

The starting recommendation: **adopt it for placements only**. Every placing technique
through rank 4 can be undone; a strike is not undone one at a time but all at once, by
opening the grid afresh on the givens left, which is what `OpenGrid` already does.

| Rank | Technique | Places or strikes | Can be undone | What undoing does | What it leaves open | Caveat |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `full_unit` | places | | | | |
| 2 | `cross_hatch` | places | | | | |
| … | one row for each technique to rank 29 | | | | | |

## Non-goals

- A generator, which is S08's.
- Rust in `crates/pawdoku`, a dependency, a recipe or a workflow.
- Editing a specification; proposed text goes in the hand-back notes.
- Removal by orbit. This spike undoes one cell at a time.
- The techniques `technique.allium` defers.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S07-undoing-a-deduction.md` | ticket | the evidence, the recommendation, the drafted follow-up; `status: done` |
| `tickets/README.md` | ticket index | S07's row set to `done` |

## Steps

1. Create the worktree on `ticket/s07-undoing-a-deduction` from `main` after S06 has
   merged (README.md "How to pick up a ticket").

2. Verify, read-only, and record with sources and dates: (a) what has been published
   on making a puzzle by undoing deductions or by aiming at a named technique;
   (b) for each placing technique, which candidates the emptied cell and its peers may
   hold so that the earlier grid still licenses the placement; (c) for each striking
   technique, which candidates may be given back so that the earlier grid still
   licenses the strike, and whether the several strikes of one deduction are undone
   together or one by one; (d) whether either uniqueness technique can be undone
   soundly with `uniqueness_promised` false; (e) what undoing a placement does to the
   candidates upkeep struck. Fetching a source is separately authorised: stop and ask.

3. State the two readings, broad and strict, and what each guarantees: that a run from
   the earlier grid ends solved, that it retraces the undoing, and that its hardest
   step is the hardest technique undone.

4. Rebuild S06's prototype from S06's hand-back notes, in `ai_tmp/` or the session's
   scratch directory and never in a commit; S06's own copy stayed in its worktree.
   Check the rebuild against the grids and counts S06 recorded before going on. Then
   extend it for ranks 1 to 12. From a solution grid, undo one cell at a time and
   count at each grid: the earlier grids tried, the undoings found under each reading,
   and how both change as the digits left fall towards 30.

   The earlier grids an undoing may lead to are too many to list, since the candidates
   given back to a cell's peers can be any of a great many sets. So completeness is
   shown for what is undone, not for every earlier grid: on a stated sample of grids,
   an independent search (exhaustive over small cases, or the SAT or SMT solver S06
   weighed) finds every deduction that some earlier grid would undo, each named by its
   technique and what it places or strikes, and the prototype must find the same set.
   Two undoings that differ only in their witness are one here, because the witness
   depends on which earlier grid is chosen. The count of distinct earlier grids is
   reported as found and never as complete.

5. Check every undoing found, by the prototype's own search or by the SAT or SMT
   solver S06 weighed: the earlier grid meets `DeductionsAreSound`'s premise for the
   solution grid (every placed digit is the solution's, every open cell allows the
   solution's digit), its digits and candidates together admit that solution alone,
   the deduction undone is on its list, and taking that deduction, with upkeep after
   a placement, gives exactly the later grid. Whether the digits alone are well-posed
   is a verdict on givens and S08's to measure.

6. Complete the table for ranks 13 to 29 on paper, from the catalogue guarantees.

7. Draft the follow-up in the hand-back notes: the proposed text and the module it
   belongs to, and a build ticket under the next free id on the day if one is wanted.
   Record as well what S08 needs to rebuild the prototype: S06's record, extended by
   the two readings, how an earlier grid is proposed and tested, and the grids
   measured with each grid's counts.

8. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The table has twenty-nine rows and no empty cell.
- Both readings are stated with what each guarantees, and both are counted.
- The rebuild reproduced S06's recorded counts, or each difference is explained.
- On every sampled grid the prototype found the same deductions, by technique and by
  what each places or strikes, as the independent search of step 4, or each
  difference is listed.
- Every undoing the prototype found passed the check of step 5, or is listed as a
  defect.
- The hand-back notes carry everything step 7 names for a rebuild.
- The verdict is adopt, adopt for a named part, or not at all, with the reason.
- `git status --porcelain` on the ticket branch lists only this file and the index.

## Verification

```sh
rg -n 'ADeductionPlacesOrStrikes|uniqueness_promised' docs/specs/technique.allium
rg -n 'born kept' docs/specs/technique.allium
rg -c '^\| [0-9]+ \| `' tickets/S07-undoing-a-deduction.md
git status --porcelain
```

Expected: the invariant and the field; the sentence that a grid is born kept; 29; two
lines naming this file and `tickets/README.md`.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the strict reading is too narrow to arrive at givens whose run needs rank 5
  or above; step 4 measures it, the maintainer decides what follows.
- Whether strikes need undoing at all, or only placements, with a run from the givens
  left to say what the puzzle asks.
- The words. Undoing and earlier grid are this ticket's; the specification that takes
  them, if one does, names them in its Vocabulary.
