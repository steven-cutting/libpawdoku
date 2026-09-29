---
id: S06
title: "Spike: one ordered list of licensed deductions, and a run that takes the first"
status: open
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
  the same digits and candidates license the same deductions every time. `Order`'s
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

3. State the list. An entry is a technique, what it places or strikes, and one
   witness. Say which witness is kept when several rest the same placement or strikes
   on different proofs, and state the total order: rank, then what, then what, until no
   two entries are equal.

4. Build a prototype in `ai_tmp/` or the session's scratch directory, never in a
   commit, for ranks 1 to 12 with full marks and every span. Run it from the givens to
   the end over a stated set of grids, the four study puzzles of
   `docs/explanation/puzzle-design.md` among them, which that page marks unverified.
   Count, per look: the entries on the whole list, the entries when listing stops at
   the first rank that yields anything, and the operations each took. Counts and never
   seconds.

5. Bound the list for ranks 13 to 29 on paper: where the count of witnesses grows with
   `max_chain_links` and `max_forcing_paths`, and what the rule of step 3 brings it
   down to.

6. Complete the table above, and write the verdict against four criteria: the list is
   the same every time; it is short enough to draw up at every look; nothing a consumer
   is owed is lost; the specifications get shorter.

7. Draft the follow-up in the hand-back notes: the proposed text for
   `technique.allium` and for each model that changes, and a build ticket under the
   next free id on the day, found with `rg -n 'T2[0-9]|S0[5-9]' tickets/`.

8. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The total order is stated so that no two entries are equal, and the prototype's run
  over the same grid twice gives the same steps.
- The counts of step 4 are in a table by rank, and the bound of step 5 is argued.
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

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the list replaces `effort.allium`'s price and `lapse.allium`'s guesses, or
  only the three statements of looking; the recommendation says, the maintainer
  decides.
- Whether an unsystematic order and a fixation survive as orderings of the same list
  or are dropped; if dropped, `technique.allium`'s `Order` and `Fixation` go with them.
- Whether the total order is `technique.allium`'s, beside the ladder, or
  `reach.allium`'s, beside the choice it already makes.
