---
id: S11
title: "Spike: the chain techniques, ranks 23 to 27: what a chain is, what is listed and what capacity hides"
status: open
depends_on: [S06]
parallel_with: [S07, S08]
branch: ticket/s11-chain-techniques
estimated_size: L
---

# S11: Spike: the chain techniques, ranks 23 to 27: what a chain is, what is listed and what capacity hides

## Context

S06 proposed one total order over a grid's deductions and measured it for ranks 1 to
22, 28 and 29. It could not speak for the five techniques between: `remote_pair`
(rank 23), `x_chain` (24), `xy_chain` (25), `alternating_inference_chain` (26) and
`forcing_chain` (27). Its hand-back notes say why: `docs/specs/technique.allium` leaves
unsaid three things the list and its cost depend on, so neither the size of the list
nor the cost of the search could be argued, and the order was made normative for ranks
1 to 12 alone. These five are the only techniques that read the two catalogue limits,
`max_chain_links` (24) and `max_forcing_paths` (9).

The request, as the maintainer made it on 2026-10-03 and in this project's words:

> A follow-up spike for ranks 23 to 27. It finds the published work that bears on
> them, academic papers among it; it checks what it builds with a SAT solver; and it
> ends in product decisions, which later tickets then take as given.

Facts to start from, each verified at execution against the specifications as they
stand on `main` and against S06's hand-back notes:

- **Listing every witness was not shown to be practical; listing the outputs is
  bounded.** The count of chain paths is bounded only by the branching to the power
  of the links, at most about 1e26 at 23 links. That is an upper bound and not a
  count any grid was shown to reach, but no smaller one was argued, so S06 did not
  draw up the catalogue's deductions, one for each proof, for these ranks. The
  outputs are of three kinds. An endpoint strike, for `x_chain`, `xy_chain` and
  `alternating_inference_chain`, strikes "any candidate weakly linked to BOTH
  endpoints", a function of the two endpoints alone, so such entries are at most the
  pairs of endpoints: 29,160, 6,480 and 265,356 on paper. A closure, which
  `AlternatingChains` gives to `alternating_inference_chain`, strikes or places one
  candidate: at most 1,458 more, 266,814 in all for rank 26. A `forcing_chain`
  concludes one literal: at most 1,458.
- **Whether a chain may pass through a candidate twice is not said.** Only
  `RemotePairs` says "no repeating cells". If a chain may repeat, whether some chain
  within the limit joins two endpoints is one pass over a graph of at most 1,458
  literals and 23,328 implications (`LinkMeaning`), about 4.1e8 inspections a look at
  worst for rank 26. If it may not hold a candidate in both polarities, the question
  is a path with forbidden pairs and no polynomial bound was argued.
- **The minimums of two techniques depend on that answer.** `x_chain` needs "at least
  three links" and `xy_chain` "at least three distinct cells". If a chain may repeat,
  a conjugate pair walked there, back and there again has three links, so every
  conjugate pair has the shape of an `x_chain`; it is a deduction only where some
  candidate outside it is weakly linked to both ends.
- **`remote_pair` has no useful small bound as written.** Its targets see two chain
  cells "an odd number of links apart", so its strikes depend on which cells the
  chain holds. S06's bound on its chains, on a true grid with 17 or more cells
  placed, is about ten million a look, loose and not a measurement, and its entries
  are bounded only by the chains' sets of cells.
- **A proof of fewest links is not a proof of least load.** A variable load "is the
  number of distinct candidate propositions in the proof", `Profile.holds` hides a
  deduction whose load is past `capacity`, and the witness an entry keeps decides its
  load. A review of S06 found a closure with a shortest proof of 9 links over 9
  candidates and a longer one of 11 links over 8. So "keep the shortest" can hide an
  entry from a player who could hold it, and no search for the least load is known
  to this project.
- **The specification already doubts these loads.** Its open question on loads and
  ladder order says a variable load "puts a long chain beyond every capacity Cowan
  suggests and so hides it from an expert", and asks whether "a chain [should] be held
  link by link".
- **`forcing_chain` has gaps of its own.** Whether a root reaches itself by an empty
  path; whether "up to config.max_forcing_paths" counts one path for each root or for
  each incompatible literal; and which witness is kept when a proof has several paths.
  On a grid of side 9, `max_forcing_paths` never binds: a cell has at most nine
  candidates and a digit at most nine places in a unit.
- **Two readings from ranks 13 to 22 are still open.** What "other cells" excludes, on
  which the entries of `w_wing` depend; and whether a colouring's wrap and trap are
  one deduction or several. They need no building and are decided here with the
  rest, because one specification change states them all.
- **What exists to build on.** S06's prototype, grids and oracle stayed in S06's
  worktree. Its notes carry the definitions, the order, the counters, the grids and
  their counts for ranks 1 to 22, 28 and 29, and 53 of its 226 grids stalled under
  that repertoire: those stalled states are the candidates for testing these five
  techniques. Which of the five apply there was not established and is this spike's
  to find.
- **Words.** Deduction, step, strike, given. A chain is the specification's word. A
  walk is a chain that may pass through a candidate twice, a word this ticket needs
  and the specification does not have. Never move, never eliminate, never clue.

Read first: `tickets/CONVENTIONS.md` §11; `tickets/S06-licensed-deductions-in-order.md`,
its hand-back notes whole, "Step 5" and "Step 5, extended" above all;
`docs/specs/technique.allium`, its catalogue guarantees `ProofsMatchTheirOutputs`,
`LinkMeaning`, `RemotePairs`, `AlternatingChains`, `BoundedForcingChains` and
`AdvancedExamples`, its config, and its open question on loads and ladder order;
`docs/specs/reach.allium` and `docs/specs/effort.allium`, their open questions;
`docs/explanation/solving-sudoku.md`; `docs/explanation/human-solving.md`, its evidence
table.

## Goal

A decision on each row of the table below, made by the maintainer on evidence this
spike gathers: the published work, a prototype of the five techniques under each
reading that can be built, and a SAT solver's word on what the prototype finds. With
the decisions: the proposed specification text, and the follow-up tickets drafted so
that each takes the decisions as given. No tracked file changes but this one and its
row in `tickets/README.md`; programs, downloads and a solver are scratch, under
`ai_tmp/` or the session's scratch directory, each where a step authorises it.

| Id | Decision | Options to weigh | Starting recommendation |
| --- | --- | --- | --- |
| D1 | May a chain pass through a candidate twice? | a walk; no candidate twice in either polarity; no cell twice | a walk, for ranks 24 to 27 |
| D2 | What do the minimums of `x_chain` and `xy_chain` count? | links walked; distinct candidates or cells | distinct candidates and distinct cells |
| D3 | What is the output and the witness of `remote_pair`? | one for each chain, as written; the whole two-coloured component; one struck candidate | the component, as `simple_colouring` has it |
| D4 | What does an entry of a chain technique hold? | as written: for an endpoint strike every target of its endpoints, for a closure or a forcing chain its one candidate; or one struck candidate or one placement throughout | as written |
| D5 | Which witness does an entry keep, and what load does `holds` read? | hide first, then keep the first witness the player can hold, which needs to know whether any is within capacity; keep the fewest links and read its load; no load at all for chains; held link by link | undecided: the evidence of step 5 says |
| D6 | `forcing_chain`: may a root reach itself, how are paths counted, which witness is kept? | a root reaches itself by an empty path, or does not; one path counted for each root, or for each incompatible literal; each path the shortest, or the paths of least load together, or any the player can hold; with whatever step 2 and step 3 add | undecided |
| D7 | Do `max_chain_links` and `max_forcing_paths` stay at 24 and 9? | stay; a lower figure the measurements support | stay, with what binds and what never does recorded |
| D8 | What do "other cells" exclude, for ranks 13 to 22? | every cell of the witness; only the cells the proof says are one of two | undecided: S06 measured the first only |
| D9 | Are a colouring's wrap and trap one deduction? | separate; one for each component | separate, as S06 measured |

## Non-goals

- Rust in `crates/pawdoku`, a dependency, a recipe or a workflow.
- Editing a specification. The decisions become proposed text in the hand-back notes;
  the change itself is a follow-up made through the `spec-change` skill.
- The techniques `technique.allium` defers: grouped chains, almost locked sets,
  forcing nets, finned fish.
- Settling the ladder's order past rank 12, or calibrating a profile. What the
  measurements show of either is recorded and handed back.
- Undoing a chain, which is S07's, and generation, which is S08's.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S11-chain-techniques.md` | ticket | the evidence, the decisions, the drafted follow-ups; `status: done` |
| `tickets/README.md` | ticket index | S11's row set to `done` |

## Steps

1. Create the worktree on `ticket/s11-chain-techniques` from `main` after S06 has
   merged (`tickets/README.md`, "How to pick up a ticket").

2. Verify, read-only, and record with sources and dates: (a) every sentence of
   `technique.allium` that bears on D1 to D9, quoted with its lines, and for each
   decision whether the specification is silent, says one thing, or says two; (b)
   which sentences of `reach.allium`, `effort.allium`, `lapse.allium` and
   `human-solving.allium` read a chain's load, links or paths, and so change with a
   decision; (c) what S06's notes hand to this ticket, checked against the notes as
   merged. After step 3 and before step 5, write out the options of D6 and of any
   other decision the reading has added to, so that step 5 builds each from a stated
   definition. The SAT solver of step 6 checks what the options entail; it does not
   choose among them.

3. Find the published work, academic papers first. Fetching is separately authorised:
   stop and ask. For each source record the full citation, where it was read (a URL
   and the date, or "abstract only"), tagged "paper", "docs" or "source", and one
   sentence on what it settles for which decision. A claim taken from memory and not
   read is marked so and decides nothing. The questions to put to the literature:

   - **Chains as implication graphs.** Static links are binary implications between
     literals, as in 2-satisfiability and in failed-literal probing: what is known of
     reachability, of the shortest proof, and of what such chains can and cannot
     derive beside unit propagation (D1, D4, D6).
   - **Paths with forbidden pairs, and simple paths of a required length or parity.**
     The complexity of each, in general and on graphs of this shape (D1, D2, D3).
   - **The fewest distinct nodes on a walk or on several paths from one root.** Whether
     the least load is a known problem and what it costs (D5).
   - **Resolution rules and confluence.** The work that treats Sudoku techniques as
     rules of a resolution theory, with chains, whips and braids, and proves when
     the order of application cannot change the end (D1, and S06's question (d)).
   - **Rating by technique.** The published evaluations of difficulty measures, and
     what they say of chain length against human difficulty (D5, D7).
   - **Working memory and chains.** What the cognitive literature the specifications
     already cite says of holding a sequence link by link against holding it whole
     (D5).
   - **How published solvers define these five.** Their own documentation and source,
     as S06's question (c) read them: whether a chain may repeat, what a remote pair
     strikes, how a forcing chain's paths are counted, and what bound each sets
     (D1 to D7).

   Leads to start from, none verified: Berthier's work on resolution rules and
   pattern-based constraint satisfaction; Pelánek's evaluation of difficulty rating;
   Lynce and Ouaknine on Sudoku as a SAT problem; Aspvall, Plass and Tarjan on
   2-satisfiability; Gabow, Maheshwari and Osterweil on paths with forbidden pairs;
   Cowan on working-memory capacity, which the specifications cite; McGuire, Tugemann
   and Civario on the least number of givens.

4. Rebuild S06's prototype for ranks 1 to 22, 28 and 29 from S06's hand-back notes, in
   `ai_tmp/` or the session's scratch directory and never in a commit. Check the
   rebuild against the fifteen grids S06 records that need no network, row for row,
   before going on. Fetching S06's other grids again and installing a SAT solver into
   a throwaway environment under scratch, never into pixi or a manifest, are
   separately authorised: stop and ask.

5. Fix the definitions, then build. Before any program exists, write down for each of
   the five techniques, under each option of D1 to D6 that can be built: the catalogue
   deduction, the output, the witness and its order, and the counters, counted as
   S06 counts them (for a chain, one implication inspected). Then build each, and
   have a second program written apart from the same definitions and
   `technique.allium` alone. Where the witnesses cannot be listed, the second program
   lists them exhaustively under a small limit on links, stated, and the two are
   compared there entry for entry; under the full limit they are compared by output.
   Run both over S06's grids under a profile pinned in full, its repertoire all
   twenty-nine ranks. Count, for each technique and each option, at each look: the
   entries, the patterns examined, the links of the witness kept and its load, and
   where each limit binds. Counts and never seconds. Record the grids that solve and
   the hardest rank of each.

6. Check with a SAT solver, which S06's question (e) weighed, and say what each check
   shows and does not:

   - **Soundness.** Every entry found is sound against the digits and candidates
     standing: with a placement denied, or a struck candidate asserted, the state has
     no model. A state with no model at all is reported as a failure, not as
     soundness.
   - **A ceiling.** For a stated sample of states, every candidate that no completion
     of the state uses, by one query each. Every strike of a chain technique lies
     within that set; the share of it each technique and each option reaches is
     recorded, and so is what a chain of unbounded length could reach and the limits
     cut off.
   - **The static graph.** That each implication the prototype uses holds of the
     state, by one query for each implication over a sample.
   - **What it does not show.** That a pattern named is the technique claimed, that a
     witness is the least, or that the list is complete: those are the second
     program's. `CatalogueValidation` allows an exact evaluator to validate and
     forbids it to supply a witness.

7. Complete the table: for each of D1 to D9, each option with what the literature
   says, what it costs by the counters, what it lists, what it hides under a capacity
   of 4 and of 8, and which sentences of which specification it changes. Write a
   recommendation for each, and say where two decisions cannot be made apart.

8. Put the decisions to the maintainer: stop and ask, one decision at a time, each
   with its evidence and its recommendation. Record each answer with its date under
   "Open points settled", one line for each decision, opening with its id in bold. A decision the maintainer defers is recorded as deferred,
   with what it blocks. No decision is taken by this ticket's agent.

9. Draft the follow-ups in the hand-back notes, each written to take the decisions as
   given and to cite them by id:

   - the proposed text for `technique.allium`, and for each model a decision touches;
   - a specification-change ticket, under the next free id on the day, found by
     searching ticket text on every ref and not only this branch; or, if S06's drafted
     specification change has not been created yet, the amendments to that draft;
   - what S07 needs for the five rows of its table, what S08 and the designed
     generator of T19's hand-back need of a chain in a run, and what the ticket that
     builds the catalogue in Rust needs: the search each technique takes, its bound,
     and the fixtures `AdvancedExamples` asks for;
   - what a rebuild needs: the definitions as decided, the counters, the grids and
     each grid's counts.

10. Set `status: done`, run the Verification, whose last command is run before the
    commit, and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (c) of step 2 are recorded with lines and dates.
- Every source of step 3 carries a citation, where and when it was read, its tag and
  the decision it bears on; nothing unread decides anything.
- The rebuild of step 4 reproduced S06's fifteen recorded rows, or each difference is
  explained.
- The definitions of step 5, D6's options among them, were fixed before any figure was
  recorded, and the counts are in a table by technique and option. Which of the five
  techniques apply on the stalled states is recorded.
- The two programs agreed entry for entry under the small limit and output for output
  under the full one, on every sampled state, or each difference is listed and
  explained.
- Every entry found passed the soundness check of step 6, or is listed as a defect;
  the ceiling and each technique's share of it are recorded, with what a chain of
  unbounded length reaches beside the limits; the sampled implications each held, or
  are listed.
- The table of decisions is complete: no option without its cost, its list, what it
  hides at a capacity of 4 and of 8, and the sentences it changes.
- Each of D1 to D9 is answered by the maintainer with a date, or recorded as deferred
  with what it blocks.
- The follow-ups cite the decisions by id and name every specification change as a
  change to make first.
- The hand-back notes carry everything step 9 names for a rebuild.
- Before the commit, `git status --porcelain` on the ticket branch lists only this file
  and the index; after it, the worktree is clean.

## Verification

```sh
rg -n 'guarantee (LinkMeaning|RemotePairs|AlternatingChains|BoundedForcingChains)' docs/specs/technique.allium
rg -n 'max_chain_links|max_forcing_paths' docs/specs/technique.allium
rg -c '^\| D[1-9] \|' tickets/S11-chain-techniques.md
rg -c '^- \*\*D[1-9]\.\*\* .*(20[0-9]{2}-[0-9]{2}-[0-9]{2}|deferred)' tickets/S11-chain-techniques.md
git status --porcelain
```

Expected: the four guarantees; the two limits, in the config and wherever a guarantee
reads them; at least 18, the Goal's nine rows and the completed table's, a check of
shape only; 9, one line under "Open points settled" for each decision, opening with
its id in bold and carrying the date of the maintainer's answer or the word deferred;
and, run before the commit, two lines naming this file and `tickets/README.md`.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the ladder's order past rank 12 should be settled with these decisions or
  after them; `technique.allium` holds both in one open question.
- Whether D8 and D9 belong here or in the specification change S06 drafted; they are
  here so that one change states every reading, and may be taken out.
- Whether the profile of step 5 should also be run at a capacity of 4, the expert
  preset's, so that what the decisions hide from the preset is measured and not
  only argued.
