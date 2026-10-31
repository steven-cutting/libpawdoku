---
id: S11
title: "Spike: the chain techniques, ranks 23 to 27: what a chain is, what is listed and what capacity hides"
status: done
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
| D7 | Do `max_chain_links` and `max_forcing_paths` stay at 24 and 9? | stay; `max_chain_links` at 16, 12 or 8, each with `max_forcing_paths` at 9 and at a lower figure if step 5 finds one that binds | stay, with what binds and what never does recorded |
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
   where each limit binds. Then run again, with the same counters, under every pair
   of limits D7 names: where a limit binds says nothing of how a run goes on without
   the chains it cut, since each strike not made changes the grids after it. Counts
   and never seconds. Record the grids that solve and
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
- Every pair of limits D7 names was run, with the grids solved and stalled under each.
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

Everything below was done on 2026-10-03 in the worktree of this ticket, at HEAD `0682b06`,
clean before and after every step. `docs/specs/technique.allium` is 1,166 lines and its
last commit is `42330fe` (2026-09-28); S06 ran from `b4eecc2`, and the file has not changed
since, so S06's line citations hold. `technique:n` is a line of that file, and `effort:n`
and `reach:n` are lines of `docs/specs/effort.allium` and `docs/specs/reach.allium`.
"Definition choice N" is the numbered choice of the table under step 5. **Fact** marks
what a source says or a command printed; **inference** marks reasoning no command checked.
Every program, download, list and solver stayed under the gitignored `ai_tmp/s11/` of the
worktree and is part of no commit; these notes carry what a reader needs and do not depend
on it. Counts, never seconds.

#### Step 2(a): what `technique.allium` says on D1 to D9

The file was read whole. Every quotation was copied by line number by a script; a second,
adversarial pass then checked 576 of 576 fenced lines verbatim at the cited lines, searched
every inline quotation, and searched the file for sentences the first reading missed
(chain, path, link, repeat, distinct, endpoint, load, capacity, "other cells", wrap, trap,
forcing, root, loop, component, witness, canonical, duplicate, "each once"). The second
pass corrected 2 misquotes, 3 citations and 3 verdicts. The verdicts as corrected:

| Id | Verdict | The sentences, with their lines |
| --- | --- | --- |
| D1 | one thing for rank 23; one thing for the two ends of a closure at rank 26; silent on every other repeat for ranks 24 to 27 | "At most config.max_chain_links peer links; no repeating cells." (978, of `RemotePairs` alone). "Also permit a path from a literal to its negation: strike its candidate if the start is positive, place it if negative." (992-993): a literal and its negation are one candidate in two polarities (159-162), so a closure holds its candidate twice, at its two ends. "Continuous loops may expose subpaths with these outputs; no separate loop name or further inference rule is introduced" (998-999) gives a loop no deduction of its own and neither admits nor forbids a repeat inside an open path. `InferencePath.literals` is a `List<Literal>` (171) |
| D2 | one thing each, in two units; silent on walked against distinct for `x_chain` | "X-Chain restricts all nodes to one digit, with at least three links." (985); "at least three distinct cells" (988). No fixture sits at `xy_chain`'s minimum |
| D3 | one thing: one deduction for each chain, striking every target of every odd pair | 973-978 ("an odd number of links apart", 975-976), with "every eligible target for that witness" (905-906); the fixture at 1094-1095 names one target and does not pin the whole set of strikes. "Odd" includes one, so consecutive pairs count as written |
| D4 | one thing for the output; one sentence that reads two ways for what one deduction is | every target for the witness (905-906, 983-984); one candidate for a closure (992-993); one literal for a forcing chain (1009, 1012). "Placement proofs yield one deduction per forced placement" (906-907) can be read as one for each placement or one for each proof; the general definition (35-36, 68, 82-84, 182) says one for each place a technique holds. "One placement may be licensed by several techniques, and then each has its own deduction" (336-337) is silent on several proofs within one technique |
| D5 | one thing as written (each witness its own deduction with its own load), by inference from the general definitions and not by a sentence on chains; doubted by the file's own open question | "A variable load is the number of distinct candidate propositions in the proof (at least one)." (598-599); `holds(x): x.load <= capacity` (248). "It holds one deduction for each of the following and no other, each once" (314-315) is followed by a list of ranks 1 to 12 only (317-333); ranks 13 to 29 come in by 338-341, which hand "Advanced patterns and every proof/output" to the Catalogue. No catalogue guarantee from 896 to 1116 says "one deduction for each" of anything past rank 12. The open question at 1162 asks "should a chain be held link by link" |
| D6 | silent on all three parts | 1001-1016: "forcing_chain admits up to config.max_forcing_paths static-link paths, each of at most config.max_chain_links. No nested guesses and no path step that needs results of earlier nonadjacent steps." (1002-1004). Nothing says what "reach" needs, what a path is counted for, or which witness is kept. The fixture's "Omitting either path" (1103) leans to one path for each root |
| D7 | one thing for the figures (24 and 9), and that "Changing either changes catalogue_version." (527-530); silent on the unit of a forcing path's bound | one figure bounds three units: peer links between cells (978), links between candidate nodes (982, 995-996), and the steps of a forcing path (1003) |
| D8 | silent | "other cells" four times (929, 934, 941, 944) and nowhere else, with no stated antecedent |
| D9 | silent; what leans, leans to one deduction for each component | two named outcomes (967-968, 969) and no "one deduction" or "two". "Disconnected components cannot share a label assignment as proof" (970), "Duplicate witnesses from reversed paths or swapped labels have one canonical representation" (914-915) and 905-906 make the proof a thing with labels over a component. Nothing quoted leans to two; the ticket's starting recommendation ("separate") is therefore against the lean of the text |

What follows from the corrections (inference):

- D1's option "no candidate twice in either polarity", read of a whole path, contradicts
  992-993 as written. It was built only with the exception written into it: the last
  literal of a path may be the negation of its first.
- Under a walk, a witness of fewest links never holds a literal twice (the stretch between
  two visits can be cut out), though it can hold a candidate in both polarities; a walk
  that does so contains a closure. So the reading added a fourth state to D1's three
  options: no literal twice.
- A sentence saying what is listed "each once" past rank 12 does not exist. The proposed
  text cannot lean on 314-315 for ranks 13 to 29.
- A sentence no first reading quoted, 1142-1145, confirms that the file's word "chain"
  covers ranks 19 to 21 as well as 23 to 26; it bears on no verdict.
- Decisions the text does not let be made apart: D1 with D2; D3 with D9 and with D5; D4
  with D5; D6 with D1 and D7; D8 with D4 (984's "excluding the endpoints").

#### Step 2(b): what the player models read of a chain

Read whole: `docs/specs/reach.allium` (429 lines), `effort.allium` (318), `lapse.allium`
(531), `human-solving.allium` (1,284). Read in part, with searches over the whole page:
`docs/explanation/human-solving.md` and `docs/explanation/puzzle-design.md`. 267 quoted
fragments were checked against the cited lines by the second pass; 1 range, 2 lists of
lines (one wrong, one incomplete), 1 misquote and 4 option labels were corrected. `generation.allium`, `board.allium`
and `solver.allium` were not read.

Fact: none of reach, effort or lapse names `max_chain_links`, `max_forcing_paths`,
`catalogue_version`, `reasoning_depth`, "link", or "path" in a chain's sense
(`reach.allium` 429 has "Path-dependence" of a run's hardest step); "chain" is in the
three only at
`effort.allium` 71, 312 and 314. Each reads a chain only through `technique.allium`'s
`holds` (248), `sees` (254), the variable load (597-599) and `grid.deductions`. Inference:
a decision reaches them in three ways only: which entries `grid.deductions` holds (D1 to
D4, D6 to D9); what `x.load` is (D5 directly, the others through the witness kept); and
whether `holds` reads a load of that kind at all (D5). "Extension" below means the sentence
stands and what it evaluates to changes; "text" means the sentence needs rewriting.

| Id | `reach.allium` | `effort.allium` | `lapse.allium` | `human-solving.allium` | `human-solving.md` | `puzzle-design.md` |
| --- | --- | --- | --- | --- | --- | --- |
| D1 | extension (entries) | extension; 71, 312 what "past its limits" covers | extension; 433-441 soundness to keep | silent on repetition (920-925, 956-962); 54-55 text if "walk" is added; 965-970 extension | 247 text (what a link counts) | line drift only |
| D2 | extension | extension | extension | 956-962 counts consecutive implications (analogy); 965-970 extension | 247 text (clarify) | none |
| D3 | extension; 238 what one step strikes | extension; load as a component's; 314 "chains" list | extension | 916-917 extension | 284 text if put beside colouring | 949 note |
| D4 | extension; 238, 383-390, 405-407 | extension; 110-112 more or fewer entries | extension; 444-454 kept | 916-917 extension | none | 918 by load |
| D5 | 100-103, 383-390 text under two options; 369-372 conflict under "hide first"; rest extension | 100-108 text under two options; open question 314 answered or rewritten; 29-30 | extension; 154-159 when a contradiction is noticed | 772-775 text under "link by link"; 839-852, 920-925 are the stated model for it | 96 text; 146-149 none | 918, 920 text under "no load"; 950 extension |
| D6 | extension | extension | extension | 944-950, 1010-1011 silent, none | 248 text | 925 text |
| D7 | extension; open question 423 | 69-71, open question 312 extension | extension; open question 531 (how often its case arises) | 108, 485 and "smaller" (the two 24s); 93, 1135-1137 extension | 247 text; 130 text if the range follows; 248 | 925 "short"; 924 line drift |
| D8 | extension only (ranks 13 to 22) | extension only | extension only | none | none | none |
| D9 | extension only; 238 | extension only; load of each | extension only | none | 283 text if wrap and trap become one | none |
| any | none | none | none | 195, 208, 475, 1094-1097: `catalogue_version` changes | 257-258 text (`technique-2`) | 929 extension |

The sentences that read a chain's load, links or paths, and what the decisions as taken do
to each (the reading is inference; the quotations are fact):

- **Load, changed by D5 (held link by link).** `reach.allium` 7-9 ("cannot hold"), 100-103
  ("small enough to hold") and 383-390 ("of a size they can hold") need their words looked
  at, since a chain's load is no longer the count of its propositions. `effort.allium`
  29-30 and 100-108 hold the only arithmetic on a chain's load in the four models,
  `base_figure(x.technique) * x.load / profile.capacity`, and its open question at 314
  ("Pricing a variable load ... Is a long chain priced by its length, by what is held at
  once, or by its base figure alone?") is D5's question put to effort: D5's answer answers
  it or rewrites it and cannot leave it as it stands. `human-solving.allium` 772-775 (the
  projection of capacity) is the sentence that sends a capacity of 4 to `holds`;
  839-852 and 920-925 already describe grouping "a witnessed link, or a consecutive
  implication path" into one chunk, the stated model for holding link by link.
  `human-solving.md` 96 (the Cowan row) says the evidence does not establish "a mapping
  from one cell to one chunk". `reach.allium` 369-372 (`LimitsOnlyHide`) would have been
  contradicted only by the option "hide first, then keep the first witness the player can
  hold", which was not chosen.
- **Links, changed by D7 (`max_chain_links` 16).** `human-solving.allium` 108,
  `reasoning_depth: Integer -- 0..24 implications per reasoning episode`, and
  `human-solving.md` 130 give a range that reaches past 16; `technique:527-528` calls the
  catalogue limits "independent of a player's smaller reasoning/memory limits" and
  `human-solving.md` 247 reads "Catalogue chain limit | 24 | Links per catalogue proof
  path; distinct from the profile's smaller reasoning bound." So the figure at 247 changes,
  and either the range narrows or "smaller" is reworded. No sentence says the range was set
  from the limit. `human-solving.allium` 956-962 is the only sentence in the four models
  that bounds a chain by links for a player ("Reasoning depth bounds a chain: consecutive
  implications each taking the previous conclusion as a premise"). `effort.allium` 69-71
  and its open question 312 ("a chain past technique.allium's limits") stand as written;
  what they cover grows.
- **Paths, changed by D6.** `human-solving.md` 248, "Catalogue forcing-path limit | 9 |
  Paths in a bounded forcing proof.", must say what a path is counted for (each
  incompatible literal's path). `puzzle-design.md` 925 calls contradiction reasoning
  "short" on the strength of both limits.
- **Entries, changed by D3 and D9.** `effort.allium` 314 lists "colouring, chains, forcing
  chains and BUG+1"; `human-solving.md` 283-284 names the wrap and trap and the remote
  pair; `puzzle-design.md` 949 puts `remote_pair` with "the single-digit `chains` (rank
  13 to 23)" though a remote pair runs on two digits, a fact about that row whatever is
  decided.
- **Every decision that changes the text changes `catalogue_version`** from `technique-2`
  (527-536): `human-solving.allium` 195, 208, 475 and 1095 name it; `human-solving.md`
  257-258 and `puzzle-design.md` 929 restate it; `puzzle-design.md` 924 and 925 cite
  `technique.allium` by line and drift with any edit above those lines.
- **Open questions touched:** `reach.allium` 423 (does a richer profile always reach at
  least as far) and 429; `effort.allium` 312, 314 and 316; `lapse.allium` 531 (whether a
  player stuck under a standing guess withdraws it; the decisions change how often that
  arises); `human-solving.allium` 1272 and 1278 by inference
  only. None is answered by a decision but `effort.allium` 314.

#### Step 2(c): what S06's notes hand over, checked against the notes as merged

`tickets/S06-licensed-deductions-in-order.md` (1,701 lines) was read whole and every
figure this ticket's Context takes from it was found again; about 190 quoted strings were
checked by the second pass, which corrected 1 false search claim and 2 line ranges. No
figure is wrong. Exact in S06: 29,160; 6,480; 266,814; 1,458 for `forcing_chain`; 1e26 at
23 links; 1,458 literals and 23,328 implications; 4.1e8; about ten million; 9 links over 9
and 11 over 8; 53 of 226; fifteen offline grids; the limits 24 and 9.

Handed over and used: the definitions of ranks 1 to 22, 28 and 29 (S06 L545-563,
L875-909); the witness order, the role and the list order (L564-568, L906-909); the
counters (L593-609, L916-924; for chains "one implication inspected", L789); the two
pinned profiles (L576-578, L911-914); the 226 grids with the sources and hashes of the 211
fetched ones (L611-621, L1317-1330); each grid's row under ranks 1 to 12 (L1350-1575) and
fifteen rows under the wider profile (L1024-1038); the SAT encoding and the soundness
query (L481-487); the drafted specification change T35, not created (no `T34` to `T39`
file on any ref, no T35 row in `tickets/README.md`).

Mismatches found, none of them a wrong figure:

- **M1.** 265,356 and the closure's 1,458 are not in S06; S06 gives 266,814 with no split.
  The split is this ticket's and sums to S06's total.
- **M2.** Four statements rest on the specification alone and not on S06: the closure as
  `AlternatingChains`'; the Cowan quotation and "held link by link"; the empty path; how a
  forcing chain's paths are counted.
- **M3.** "A path with forbidden pairs" is this ticket's framing; S06 says only that no
  polynomial bound is argued.
- **M4.** One horn of S06's reading is missing from the Context: if the minimum counts
  distinct candidates or cells, "no polynomial bound is argued here" (S06 L824-825). That
  is the pair this ticket recommended to start, and the pair decided. Step 5 states a
  polynomial search for it (the capped-set search, below).
- **M5.** The closure of 9 links over 9 candidates and 11 over 8 was a reviewer's finding
  that S06 did not re-derive. It was re-derived here by both programs (below).
- **M6.** "The grids and their counts for ranks 1 to 22, 28 and 29" overstates: under the
  wider profile S06 records rows for fifteen grids only, does not name the 53 stalled
  grids, and records no stalled state. They came from the rebuild of step 4.
- **M7.** S06's fifteen wider rows have nine fields, with no givens column, against the
  ten its heading promises.
- **M8.** "One pass over a graph" against S06's "one pass for each start"; the 4.1e8
  already includes the 729 starts.
- **M9.** The Context's "was not shown to be practical" is weaker than S06's "out of
  reach" and "cannot be drawn up ... at all".
- **M10.** Figures of S06's the Context leaves out: `remote_pair`'s 4e32 on any state and
  under 800,000 on a true grid; at most 20 entries under the component reading, and its
  odd-cycle condition; 3.3e7, 1.3e7 and 4.8e7 for ranks 24, 25 and 27.
- **M11.** Two open questions of S06 have no row in this ticket: the role that orders
  witnesses past rank 12, and what is licensed on a grid that contradicts itself.
- **M12.** S06 calls T35 "The build ticket"; it is a specification change.
- **M13.** The frontmatter's branch is `ticket/s11-chain-techniques`; the worktree is on
  `ticket/S11-chain-techniques`.

#### Step 3: the published work

Every source below was read on 2026-10-03, so the date is given once. Fetching published
work for this step was authorised by the maintainer on the day ("Open points settled").
Seven records were made, one for each question of step 3, each by a first reading and a
second pass made apart that fetched the copy again and found every quotation again; a
completeness critic then read the seven side by side, closed six gaps and added rows I11,
C1 to C9 and two pages of A8, which have had no second pass. "Paper", "docs" and "source"
are the ticket's tags. "By inference" marks a record's reasoning, not the source's. A
source read as an abstract only says so. A citation's title keeps its own words.

Papers and monographs:

| Key | Citation | Where read | Tag | Bears on | What it settles |
| --- | --- | --- | --- | --- | --- |
| I1 | Aspvall, Plass and Tarjan, "A linear-time algorithm for testing the truth of certain quantified Boolean formulas", Information Processing Letters 8(3), 1979, 121-123 | `https://mathweb.ucsd.edu/~sbuss/CourseWeb/Math268_2007WS/2SAT.pdf`, whole | paper | D1 | The implication graph of binary clauses is closed under contraposition and satisfiability is decided by strong components in linear time; by inference a walk makes "does a chain join these two literals" plain reachability |
| I2 | Lynce and Ouaknine, "Sudoku as a SAT problem", AIMATH 2006 | `https://www.mpi-sws.org/~joel/publications/sudoku05.pdf`, whole | paper | D6 | One literal assumed and unit propagation run to a conflict finishes every one of 24,260 grids of 17 givens that unit propagation alone does not: an upper bound on `forcing_chain`, not a measure of the static-link restriction |
| I3 | Simonis, "Sudoku as a constraint problem", CP Workshop on Modeling and Reformulating CSPs, 2005 | `https://ai.dmi.unibas.ch/_files/teaching/fs13/ki/material/ki10-sudoku-inference.pdf`, section 5.5, the summary after Table 5, the conclusion | paper | D6 | Shaving with the strongest propagation solves all the paper's grid sets; by inference shaving is a forcing contradiction, so an upper bound only |
| I4 | Bacchus and Winter, "Effective preprocessing with hyper-resolution and equality reduction", SAT 2003, LNCS, 2004, 341-355 | `https://www.cs.toronto.edu/~fbacchus/Papers/BSAT2003.pdf`, sections 1 to 2.3 | paper | D6 | What one hyper-binary resolution step yields, unit propagation from one literal also reaches; by inference a verity over a cell or a unit has that shape |
| I5 | Heule, Järvisalo and Biere, "Efficient CNF simplification based on binary implication graphs", SAT 2011, LNCS, 201-215 | `https://cs.utexas.edu/~marijn/publications/unhiding.pdf`, sections 1 to 3, 6.1, 6.2 | paper | D1, D4 | Over binary clauses a literal is failed exactly when a path joins it to its complement, with no condition of simplicity on the path; by inference that is the closure of rank 26 read as a walk |
| I6 | Heule and van Maaren, "Look-ahead based SAT solvers", Handbook of Satisfiability, 2009, 155-184 | `https://www.cs.utexas.edu/~marijn/publications/p01c05_lah.pdf`, section 5.4.1 | paper | D6 | A literal reached from both truth values of one variable is a "necessary" assignment; by inference a two-root verity over static links is a second witness of a closure |
| I7 = L1 | Buresh-Oppenheim and Mitchell, "Minimum witnesses for unsatisfiable 2CNFs", SAT 2006, LNCS, 42-47, DOI 10.1007/11814948_6 | `https://www2.cs.sfu.ca/~mitchell/papers/sat06-2res.pdf`, whole | paper | D1, D5, D6 | The least proof of a unit over binary clauses is measured by distinct links, the fewest links do not give it, and a breadth-first search is said to find it in polynomial time; it counts links, not candidates |
| I8 | Kleine Büning, Wojciechowski and Subramani, "On the computational complexity of read once resolution decidability in 2CNF formulas", arXiv:1610.04523v1, 2016 | `https://arxiv.org/pdf/1610.04523`, sections 1, 2, 4, 6, 7 | paper (preprint) | D1 | Refuting a 2CNF with each variable used at most once is NP-complete; the hardness is for a refutation, not for one closure |
| I9 | Eppstein, "Nonrepetitive paths and cycles in graphs with application to Sudoku", arXiv:cs/0507053, 2005 | `https://arxiv.org/pdf/cs/0507053`, sections 1, 2.1, 3.2 to 3.5, openings of 2.2 and 2.3 | paper (preprint) | D1, D6 | The one paper read that builds Sudoku chain rules on graph search defines its chains as walks, calls simplicity "unimportant for our Sudoku applications", and proves the simple directed variant NP-complete |
| I10 | Eppstein, "Solving single-digit Sudoku subproblems", arXiv:1202.5074, FUN 2012, LNCS, 142-153 | `https://arxiv.org/pdf/1202.5074`, section 1, the close of 5, the opening of 6 | paper (preprint) | D6 | A solver with the walk rules of I9 and a reduction to 2-SAT had 36 grids it could not finish without backtracking |
| I11 | McGuire, Tugemann and Civario, "There is no 16-Clue Sudoku: Solving the Sudoku Minimum Number of Clues Problem via Hitting Set Enumeration", arXiv:1201.0749v2; Experimental Mathematics 23(2), 2014, 190-217 | `https://arxiv.org/pdf/1201.0749`, abstract, contents and page 3 | paper (preprint, part) | none | The premise "17 or more cells placed"; bears on no decision |
| A1 | Kováč, "Complexity of the path avoiding forbidden pairs problem revisited", Discrete Applied Mathematics 161(10-11), 2013, 1506-1512; arXiv:1111.3996v3 | `https://arxiv.org/pdf/1111.3996`, whole | paper | D1 | Reports, of other papers, that a path holding at most one node of each forbidden pair is NP-hard to find in general and polynomial under Yinnone's skew-symmetry condition; by inference `LinkMeaning` with polarity pairs meets it |
| A2 | Kolman and Pangrác, "On the complexity of paths avoiding forbidden pairs", Discrete Applied Mathematics 157(13), 2009, 2871-2876 | `https://kam.mff.cuni.cz/kamserie/clanky/2008/s888.pdf` (preprint), whole | paper | D1 | The problem stays NP-complete with one forbidden pair for each node; by inference the skew symmetry and not the pairing is what would make "no candidate twice" tractable |
| A3 | Goldberg and Karzanov, "Maximum skew-symmetric flows and matchings", Mathematical Programming 100(3), 2004, 537-568; arXiv:math/0304290 | `https://ar5iv.labs.arxiv.org/html/math/0304290`, sections 1, 2, 7 | paper | D1 | Reports, without proof, that on a skew-symmetric graph a path free of a symmetric pair of arcs is found in time linear in the arcs; its pairs are of arcs, not nodes |
| A4 | Goldberg and Karzanov, "Path problems in skew-symmetric graphs", report, Stanford, 1993 | `https://www.alexander-karzanov.net/Publications/93_path_probl_skew.pdf`, a truncated file, pages 1 to 7 | paper (part) | D1 | Nothing beyond A3 |
| A5 | LaPaugh and Papadimitriou, "The even-path problem for graphs and digraphs", Networks 14(4), 1984, 507-513 | `https://collaborate.princeton.edu/en/publications/the-evenpath-problem-for-graphs-and-digraphs/`, abstract only | paper | D3, D1 | A simple path of even length is linear-time in an undirected graph and NP-complete in a directed one |
| A6 | Chauhan, Datta, Gupta and Sharma, "The Even-Path Problem in Directed Single-Crossing-Minor-Free Graphs", MFCS 2024, LIPIcs 306, 43:1-43:16 | `https://drops.dagstuhl.de/storage/00lipics/lipics-vol306-mfcs2024/LIPIcs.MFCS.2024.43/LIPIcs.MFCS.2024.43.pdf`, whole | paper | D1, D3 | States both claims of A5 from a full text; its Figure 3 supposes a graph with an even walk and no even simple path |
| A7 | Arkin, Papadimitriou and Yannakakis, "Modularity of cycles and paths in graphs", Journal of the ACM 38(2), 1991, 255-274 | `https://researchconnect.suny.edu/en/publications/modularity-of-cycles-and-paths-in-graphs/`, abstract only | paper | D3 | Whether all paths between two nodes of an undirected graph have one length modulo a fixed number is decided in linear time; by inference "an odd number of links apart" can belong to the two cells and not to the chain |
| A8 | Alon, Yuster and Zwick, "Color-coding", Journal of the ACM 42(4), 1995, 844-856 | abstract at `https://projects.csail.mit.edu/jacm/Abstracts/alonyz95.ltx`; pages 1 and 2 of `https://www.tau.ac.il/~nogaa/PDFS/col5.pdf` | paper (abstract and two pages) | D1, D2 | A simple directed path of exactly k links is found in time exponential in k alone, and the problem is NP-complete when k is part of the input |
| A9 | German, "Layer-Based Width for PAFP", arXiv:2605.12457v1, 2026 | `https://arxiv.org/pdf/2605.12457`, whole | paper (preprint) | none | A third statement of Yinnone's condition; bears on no decision |
| L2 | Kumar, "Computing a Minimum Color Path in Edge-Colored Graphs", SEA 2019 | `https://par.nsf.gov//servlets/purl/10107247`, pages 1 to 5 of 18 | paper | D5 | A path over the fewest distinct labels is a named problem, NP-hard in its edge-coloured form; nothing found treats labels that sit on two nodes each |
| L3 | Charikar, Chekuri, Cheung, Dai, Goel, Guha and Li, "Approximation Algorithms for Directed Steiner Problems", Journal of Algorithms 33, 1999, 73-91 | `https://chekuri.cs.illinois.edu/papers/dir-steiner-talg.pdf`, pages 73-76, 88-89 | paper | D6, D5 | A root joined to k terminals at least cost is hard to approximate, and joining each terminal by a shortest path costs at most k times the least; by inference several paths counted by their union are the node-weighted form |
| L4 | Dreyfus and Wagner, "The Steiner Problem in Graphs", Networks 1, 1972, 195-207 | `https://web.vu.lt/mif/s.jukna/tropical/Dreyfus-Wagner.pdf`, pages 195-197, 204-205 | paper | D6 | An exact least-cost tree joining k chosen nodes costs time exponential in k alone |
| L5 | Feldmann and Marx, "The Complexity Landscape of Fixed-Parameter Directed Steiner Network Problems", arXiv:1707.06808 | `https://arxiv.org/abs/1707.06808`, abstract only | paper | D6 | The directed rooted form of L4 is stated as fixed-parameter tractable in the number of terminals |
| L6 | Alekhnovich, Buss, Moran and Pitassi, "Minimum Propositional Proof Length is NP-Hard to Linearly Approximate", Journal of Symbolic Logic 66, 2001, 171-191 | `https://mathweb.ucsd.edu/~sbuss/ResearchWeb/kproveApprox/jslnearfinal.pdf`, pages 1-3, 6-8 | paper | D5, D6 | The least resolution refutation is hard even to approximate for Horn clauses: a contrast with I7, not a verdict on a chain |
| L7 | Klein and Ravi, "A Nearly Best-Possible Approximation Algorithm for Node-Weighted Steiner Trees", Brown CS-92-54, 1992 | `https://cs.brown.edu/research/pubs/techreports/reports/CS-92-54.html`, abstract only | paper | D5 | Counting nodes and not links in a tree that joins chosen nodes is a named problem |
| L8 | Halperin and Krauthgamer, "Polylogarithmic inapproximability", STOC 2003, 585-594 | `https://cris.tau.ac.il/en/publications/polylogarithmic-inapproximability/`, abstract only | paper | D6 | A sharper form of L3's hardness; changes no decision |
| L9 | Hassin, Monnot and Segev, "Approximation algorithms and hardness results for labeled connectivity problems", Journal of Combinatorial Optimization 14(4), 2007, 437-453 | `https://ideas.repec.org/a/spr/jcomop/v14y2007i4d10.1007_s10878-007-9044-x.html`, abstract only | paper | D5 | The minimum label path between two nodes is a named problem with published hardness results |
| L10 | Umans, "The Minimum Equivalent DNF Problem and Shortest Implicants", FOCS 1998, 556-563 | `https://authors.library.caltech.edu/records/081v8-b0580`, abstract only | paper | none | Nothing for any decision |
| L11 | "Redundancy in Logic I: CNF Propositional Formulae", arXiv:cs/0211031 (the author's surname is left out; see "Deviations, and why") | `https://arxiv.org/abs/cs/0211031`, abstract only | paper | none | Nothing for any decision |
| B1 | Berthier, "From Constraints to Resolution Rules, Part II: chains, braids, confluence and T&E", CISSE 08; Springer 2010, 171-176; arXiv:1304.3210 | `https://arxiv.org/pdf/1304.3210`, whole | paper | D1, S06 (d) | A chain is "2n different candidates" and an xy-chain "n different bivalue rc-cells"; confluence is stated and proved for the braid theories of each length |
| B2 | Berthier, *Pattern-Based Constraint Satisfaction and Logic Puzzles*, Lulu 2012; arXiv:1304.1628 | `https://arxiv.org/pdf/1304.1628`, sections 1.4.3, 4.5, 5.1 to 5.10.4, parts of 6, 8.6, 9.2, 9.3 | paper (self-published monograph) | D1, D2, D4, D5, D6, D7, S06 (d) | The general chain allows a repeat; theorems 5.15 to 5.17 prove a strike by a bivalue-chain with a repeat can be made by a shorter one without; whips of length 9 or less solve 99.99% of ten million generated grids; whip theories are shown by a grid not to be confluent |
| B3 | Berthier, *The Hidden Logic of Sudoku*, second edition, Lulu 2007 | `Publications/2007-HLS2.pdf` of `github.com/denis-berthier/CSP-Rules-V2.1` at commit `7eefd715`, sections XII.1, XII.2, XIV.1, parts of XVI, XX.2.2, XXII.2 | paper (self-published monograph) | D1, D2, D3 | Its xy-chain and single-digit chain take cells that are all different and count length in cells; theorems XIV.1 to XIV.5 prove the no-repeat xy-chain loses no reach; a remote pair "is subsumed by the general xy-pattern" |
| B4 | Berthier, *Constraint Resolution Theories*, Lulu 2011 | `Publications/2011-CRT.pdf`, same repository and commit; section 5.8's headings and theorem statements | paper (self-published monograph) | D1 | Nothing beyond B2 |
| B5 = T4 | Berthier, "Unbiased Statistics of a Constraint Satisfaction Problem – a Controlled-Bias Generator", CISSE 09; Springer 2010, DOI 10.1007/978-90-481-9112-3_16; arXiv:1111.4083 (pages unsettled) | `https://arxiv.org/pdf/1111.4083`, whole | paper | D1, D7 | A whip takes "different candidates"; with singles the only other rule, whips of length 7 or less cover about 99.9% of minimal grids, validated against another rating and against search effort, not against people |
| B7 | Berthier, "From Constraints to Resolution Rules, Part I: Conceptual Framework", arXiv:1304.3208 | `https://arxiv.org/pdf/1304.3208`, introduction only | paper | none | Defers confluence to Part II; settles nothing |
| T1 | Pelánek, "Difficulty Rating of Sudoku Puzzles: An Overview and Evaluation", arXiv:1403.7373v1, 2014 | `https://arxiv.org/pdf/1403.7373`, whole | paper | D5, D7 | The published evaluation of technique ratings against human time tests one number for a grid (Sudoku Explainer's, r 0.70 and 0.86) and never isolates chain length |
| T2 | Pelánek, "Difficulty Rating of Sudoku Puzzles by a Computational Model", FLAIRS 2011, 434-439 | `https://www.fi.muni.cz/~xpelanek/publications/flairs-sudoku.pdf`, whole | paper | D5 | Names chaining once and sets it aside |
| T3 | Pelánek, "Human Problem Solving: Sudoku Case Study", FIMU-RS-2011-01, 2011 | `https://www.fi.muni.cz/~xpelanek/publications/sudoku-tr.pdf`, whole | paper (technical report) | D5 | The one data set with a time for each placement was used only for the order in which cells were filled |
| T8 = M6 | Lee, Goodwin and Johnson-Laird, "The psychological puzzle of Sudoku", Thinking & Reasoning 14(4), 2008, 342-364 | `https://modeltheory.org/papers/2008sudoku.pdf`, whole | paper (human subjects) | D5, D6 | A deduction resting on more units takes novices longer, up to five units, and recorded candidates make it faster; it measures no chain |
| T9 | Ercsey-Ravasz and Toroczkai, "The Chaos Within Sudoku", arXiv:1208.0370v1, 2012 | `https://arxiv.org/pdf/1208.0370`, whole | paper | none | Names no technique and no chain |
| T10 | Behrens, "Improving Research Methods for Problem Solving: The Example of Sudoku", doctoral thesis, TU Darmstadt, 2024 | `https://www.tu-darmstadt.de/cogsci/studying_cogsci/doctoral_theses/doctoral_thesis_thea_behrens.en.jsp`, abstract only | paper (thesis) | none | Speaks of basic tactics and a modelled step duration, never of a chain |
| M1 | Cowan, "The magical number 4 in short-term memory", Behavioral and Brain Sciences 24(1), 2001, 87-114, with commentary and response to 185 | `https://www.cambridge.org/core/services/aop-cambridge-core/content/view/44023F1147D4A1D44BDC0AD226838496/S0140525X01003922a.pdf/the-magical-number-4-in-short-term-memory-a-reconsideration-of-mental-storage-capacity.pdf` (DOI 10.1017/S0140525X01003922), whole, 99 pages | paper | D5 | The limit of about four chunks is on what is in the focus of attention at one time, and a chunk can grow "little by little"; by inference it is no ceiling on the items a serially built sequence passes through |
| M2 | Halford, Phillips and Wilson, "Processing capacity limits are not explained by storage limits", commentary in M1's volume, 123-124 | the same PDF as M1, at the same address (it carries the target article's DOI on every page) | paper | D5 | The limit on what is related in one step is held apart from the limit on what is stored |
| M3 | Cowan, "The magical mystery four", Current Directions in Psychological Science 19(1), 2010, 51-57 | `https://pmc.ncbi.nlm.nih.gov/articles/PMC2864034/`, whole | paper | D5 | The figure of 3 to 5 chunks is seen when rehearsal and grouping are prevented; practical limits vary with the processes a task allows |
| M4 | Halford, Cowan and Andrews, "Separating cognitive capacity from knowledge", Trends in Cognitive Sciences 11(6), 2007, 236-242 | `https://pmc.ncbi.nlm.nih.gov/articles/PMC2613182/`, whole | paper | D5 | As a rule of an analysis method and not a measurement, the processing complexity of a task of several steps is that of its most complex step, unless the variables must be related together; by inference this fits "held link by link" |
| M5 | Goodwin and Johnson-Laird, "Reasoning about relations", Psychological Review 112(2), 2005, 468-493 | `https://www.modeltheory.org/papers/2005reasoningrelations.pdf`, whole | paper | D5 | Reports Halford's account that a transitive series of n entities is done in steps whose most complex integrates three |
| M7 | Johnson-Laird, "Mental models and human reasoning", PNAS 107(43), 2010, 18243-18250 | `https://pmc.ncbi.nlm.nih.gov/articles/PMC2972923/`, whole | paper | D5, D6 | The model theory's predictor of difficulty is the count of models held ("more than three models causes trouble"); under the rival theory "some evidence has corroborated" that the number of steps in a proof predicts difficulty |
| M8 | Risko and Gilbert, "Cognitive offloading", Trends in Cognitive Sciences 20(9), 2016, 676-688 | `https://samgilbert.net/pubs/Risko2016TiCS.pdf`, whole | paper | D5 | People mark positions in sequential tasks and offload more as memory load rises; it measures nothing about chains |
| M9 | Halford, Wilson and Phillips, "Processing capacity defined by relational complexity", Behavioral and Brain Sciences 21(6), 1998, 803-831 | PubMed record 10191879, abstract only | paper | D5 | The theory's two ways past the limit, serial segmentation and chunking; corroborates M4 |
| M10 | Halford, Baker, McCredden and Bain, "How many variables can humans process?", Psychological Science 16(1), 2005, 70-76 | PubMed record 15660854, abstract only | paper | D5 | The limit of four variables was measured on a task in which decomposition is constrained |
| M11 | Risko and Dunn, "Storing information in-the-world", Consciousness and Cognition 36, 2015, 61-74 | PubMed record 26092219, abstract only | paper | D5 | The use of external storage "was related" to the count of items to be remembered |

Solvers' documentation and source. Rows marked "S06's copy" were read from the copy S06
fetched on 2026-10-03, at the commit S06 records:

| Key | Citation | Where read | Tag | Bears on | What it settles |
| --- | --- | --- | --- | --- | --- |
| H1 | Hobiger, HoDoKu, `solver/ChainSolver.java`, mirror `PseudoFish/Hodoku` at `c37fe90` (an unofficial clone with changes) | `raw.githubusercontent.com/PseudoFish/Hodoku/c37fe90…/src/solver/ChainSolver.java` | source | D1, D2, D3, D5, D7 | Its X-chain, XY-chain and remote pair never pass through a cell twice, need three links (seven for a remote pair), keep the shorter chain for each set of strikes; a remote pair is one step for each chain |
| H2 | same, `solver/TablingSolver.java` | same commit; lines 1500-1690 and 1900-2600 not read | source | D1, D4, D5, D6 | AICs are "no cell twice", enforced after a search that ignores repeats; a forcing chain's paths get no repeat test and are one for each root (verity) or one for each incompatible literal (contradiction) |
| H3 | same, `solver/TableEntry.java`, `sudoku/Chain.java` | same commit | source | D2, D5 | Distance is "number of links used to get to the current entry"; a path's length is its node count with a penalty for complex nodes |
| H4 | same, `sudoku/Options.java`, `sudoku/SolutionStep.java` | S06's copy of the two files of the mirror `PseudoFish/Hodoku` at `c37fe90`, byte-identical to a fresh fetch | source | D5, D7 | Default limits are 20 for chains and 10 for nice loops; base scores take no account of length; found steps sort by placement, most strikes, then chain length |
| H5 | Hobiger, "HoDoKu: Solving Techniques - Chains and Loops" | `https://hodoku.sourceforge.net/en/tech_chains.php` | docs | D2, D3, D4, D6 | A remote pair can be reproduced by one or two simple-colour steps; a struck candidate may be part of the chain; a chain's every link "relies only on the step immediately before it" |
| H6 | Hobiger, "HoDoKu: Solving Techniques - Last Resort" | `https://hodoku.sourceforge.net/en/tech_last.php` | docs | D6 | "For every Forcing Chain Verity a complementary Forcing Chain Contradiction exists." |
| H7 | Hobiger, "HoDoKu: User Manual (Chapter 3: Configuring the Solver)" | S06's copy of `https://hodoku.sourceforge.net/en/docs_solv.php` | docs | D2, D5, D7 | The unit is links ("an XY-Chain with 5 cells contains 9 links"); the limit applies to X-chains, XY-chains and remote pairs only; one chain is kept for each set of strikes, the shorter |
| T6 | Hobiger, HoDoKu manual, "Rating and difficulty levels" (`docs_cre.php`) | S06's copy of `https://hodoku.sourceforge.net/en/docs_cre.php`, and the live page | docs | D5 | A grid's score is the sum of its steps' scores and the author disclaims it as a measure of difficulty |
| E1 | Juillerat, Sudoku Explainer 1.2.1, `Chaining.java`, mirror `1to9only/SudokuExplainer` at `906e247` | raw file at that commit; `getOnToOff` and `getOffToOn` not read | source | D1 to D7 | One search over literals adds each literal once (by inference no literal twice on a path, and nothing forbids a candidate in both polarities); multiple chains are one for each root; no bound on length is set |
| E2 = T5 | same, `ChainingHint.java`, `CycleHint.java`, `ForcingChainHint.java`, `BinaryChainingHint.java`, `CellChainingHint.java`, `RegionChainingHint.java`, `Potential.java` | same commit | source | D1, D5, D6 | A chain's "length" is the count of distinct literals among the target's ancestors, summed path by path, and the rating gains 0.1 at roughly geometric thresholds of it |
| E3 | same, `Solver.java` | S06's copy of the file of `1to9only/SudokuExplainer` at `906e247` | source | D6 | The static producers are the first and the "multiple" one; Nishio, dynamic and nested chains are nets |
| E4 | same, hint texts `ForcingChain.html`, `X-Cycle.html` | same commit | docs | D3, D4 | A forcing chain concludes one literal; a cycle's strikes are where the digit appears "regardless of the chosen configuration" |
| K1 | Sukaku Explainer, `Chaining.java`, `ChainingHint.java`, `SudokuMonster/SukakuExplainer` at `362854e` | raw files at that commit | source | none | The same code with variant links |
| W1 | Stuart, SudokuWiki, "Remote Pairs" | `https://www.sudokuwiki.org/Remote_Pairs` | docs | D3 | A remote pair is defined by the parity of the least distance between two cells, whichever path, and "has been entirely subsumed into the various Chaining strategies, particularly Simple Colouring" |
| W2 | Stuart, SudokuWiki, "X-Cycles" and "X-Cycles (Part 2)" | `https://www.sudokuwiki.org/X_Cycles`, `https://www.sudokuwiki.org/X_Cycles_Part_2` | docs | D1, D4, D5 | "Loops can be of any length but they don't re-use any candidate"; "shorter chains always take priority over longer ones" |
| W3 | Stuart, SudokuWiki, "XY-Chains" | `https://www.sudokuwiki.org/XY_Chains` | docs | D2, D5, D7 | Length is "the number of cells, not the chain links", from three; an older search stopped at twelve cells |
| W4 | Stuart, SudokuWiki, "Alternating Inference Chains" | `https://www.sudokuwiki.org/Alternating_Inference_Chains` | docs | D4 | A continuous loop strikes on the units seen by two or more of its cells |
| W5 | Stuart, SudokuWiki, "Digit", "Cell", "Unit" and "Nishio Forcing Chains" | `https://www.sudokuwiki.org/Digit_Forcing_Chains`, `https://www.sudokuwiki.org/Cell_Forcing_Chains`, `https://www.sudokuwiki.org/Unit_Forcing_Chains`, `https://www.sudokuwiki.org/Nishio_Forcing_Chains` | docs | D6 | A verity takes one chain for each root, a contradiction two chains from one root; a unit forcing chain may have a root that "is its own assertion" |
| W6 | Stuart, SudokuWiki solver page and client script | S06's copies of `https://www.sudokuwiki.org/Sudoku.htm` and `https://www.sudokuwiki.org/js/sudoku5.js?v=3.70`, the addresses S06's notes record | docs / source | D5 | "find the best and shortest chains - not necessarily the first one it happens to find"; the chain code runs on the server and cannot be read |
| T7 | Stuart, SudokuWiki, "Grading Puzzles" | S06's copy of `https://www.sudokuwiki.org/Grading_Puzzles`, and the live page | docs | D5 | A chain scores a base for its technique plus its "chain length", linearly, the unit unstated |
| P1 | Sudopedia (mirror), "X-Chain" | `http://sudopedia.enjoysudoku.com/X-Chain.html` | docs | D2, D4 | The shortest useful X-Chain must involve at least four cells |
| P2 | Sudopedia (mirror), "XY-Chain" | `http://sudopedia.enjoysudoku.com/XY-Chain.html` | docs | D2, D4 | "The shortest XY-Chain is an XY-Wing with only 3 cells." |
| P3 | Sudopedia (mirror), "Remote Pairs" | `http://sudopedia.enjoysudoku.com/Remote_Pairs.html` | docs | D3 | One chain of an even number of cells, with strikes from "each pair of cells with opposite parity, except those directly linked" |
| P4 | Sudopedia (mirror), "Nice Loop" | `http://sudopedia.enjoysudoku.com/Nice_Loop.html` | docs | D4 | Each of the three discontinuity rules acts on one candidate at one cell |
| P5 | Sudopedia (mirror), "Alternating Inference Chain" | `http://sudopedia.enjoysudoku.com/Alternating_Inference_Chain.html` | docs | D1 | States no rule on repeats and no minimum |
| P6 | Sudopedia (mirror), "Forcing Chain", "Verity", "Chain" | `http://sudopedia.enjoysudoku.com/Forcing_Chain.html`, `http://sudopedia.enjoysudoku.com/Verity.html`, `http://sudopedia.enjoysudoku.com/Chain.html` | docs | D6 | A forcing chain is "a single stream of implications", no branching; a verity's roots are a literal and its negation or all the candidates of a cell |
| Q1 | Ostermiller, QQWing, `qqwing.cpp` at `6048c90` | S06's copy of `src/cpp/qqwing.cpp` of `stephenostermiller/qqwing` at that commit | source | none | No chain technique |
| X1 | Emerentius, `sudoku` crate, `strategies.rs` at `b3f8f46` | S06's copy, from the repository `Emerentius/sudoku` at that commit | source | none | No chain technique |
| B6 | Berthier, CSP-Rules-V2.1, `README.md` at `7eefd715` | `raw.githubusercontent.com/denis-berthier/CSP-Rules-V2.1/master/README.md` | docs | D2, D4, D5 | A chain's length is "the number of CSP-Variables its definition involves" and the simplest available rule is applied first; the program's source was not read |
| L12 | `tickets/S06-licensed-deductions-in-order.md`, hand-back notes, lines 834-849 | this worktree at `0682b06` | source (a local record) | D5, D1 | The one recorded state where a closure has a shortest proof of 9 links over 9 candidates and a longer one of 11 over 8 |
| C1 | Hobiger, "HoDoKu: Solving Techniques - Coloring" | `https://hodoku.sourceforge.net/en/tech_col.php` | docs | D9 | After colouring "the grid is scanned for one of two possible contradictions", the trap and the wrap, each described apart |
| C2 | Hobiger, HoDoKu, `solver/ColoringSolver.java` at `c37fe90` | raw file at that commit, lines 186-295 | source | D9 | For each colour pair it builds a wrap step and then a separate trap step, each carrying every strike of its kind |
| C3 | Stuart, SudokuWiki, "Simple Colouring" | `https://www.sudokuwiki.org/Simple_Colouring` | docs | D9 | The wrap and the trap are two numbered rules |
| C4 | Sudopedia (mirror), "Simple Colors", "Color Wrap", "Color Trap" | `http://sudopedia.enjoysudoku.com/Simple_Colors.html`, `http://sudopedia.enjoysudoku.com/Color_Wrap.html`, `http://sudopedia.enjoysudoku.com/Color_Trap.html` | docs | D9 | "Color Wrap belongs to Simple Colors, together with Color Trap": two named techniques on one cluster |
| C5 | Hobiger, "HoDoKu: Solving Techniques - Wings" | `https://hodoku.sourceforge.net/en/tech_wings.php` | docs | D8 | For a W-Wing the candidate goes from every cell that sees both bivalue cells; no exclusion of the pattern's other cells is stated |
| C6 | Hobiger, "HoDoKu: Solving Techniques - Single Digit Patterns" | `https://hodoku.sourceforge.net/en/tech_sdp.php` | docs | D8 | For a skyscraper every candidate that sees both of the two cells is struck; for a kite, the one that sees the two other cells |
| C7 | Hobiger, HoDoKu, `solver/WingSolver.java`, `solver/SingleDigitPatternSolver.java` at `c37fe90` | raw files at that commit; lines 288-420, and 560-620 and 700-756 | source | D8 | The W-Wing and the skyscraper strike every peer of both named cells that holds the digit; no line read takes the link cells or the aligned endpoints out |
| C8 | Sudopedia (mirror), "W-Wing", "Skyscraper", "2-String Kite", "XY-Wing" | `http://sudopedia.enjoysudoku.com/W-Wing.html`, `http://sudopedia.enjoysudoku.com/Skyscraper.html`, `http://sudopedia.enjoysudoku.com/2-String_Kite.html`, `http://sudopedia.enjoysudoku.com/XY-Wing.html` | docs | D8 | Each states the target as any cell seen by the two named cells and states no exclusion |
| C9 | Stuart, SudokuWiki, "W-Wing Strategy" | `https://www.sudokuwiki.org/W_Wing_Strategy`, searched lines only | docs | D8 | "There are two cells that can see both ends and contains 3"; no exclusion in the lines read |

Where the records overlap or disagree (fact, by reading them side by side). On D1 they
pull two ways and both are right about what they read: nothing in the SAT and graph papers
contradicts the walk, and every published definition of a concrete chain shown to a
player excludes repeats (B1, B3, H1, H2, W2). One record says of the Context's "no
polynomial bound was argued" that the literature argues one, through Yinnone 1997, which
no one here has opened.

**Not opened, or recalled only: these decide nothing.**

- Known only through a source that was read: Gabow, Maheshwari and Osterweil, "On Two
  Problems in the Generation of Program Test Paths", IEEE TSE SE-2(3), 1976, 227-231, a
  lead of this ticket, closed access, three routes tried by the record and two by the
  critic (through A1, A2; D1). Yinnone,
  "On paths avoiding forbidden pairs of vertices in a graph", Discrete Applied Mathematics
  74(1), 1997, 85-92, HTTP 403 to every fetcher (through A1, A2, A9; D1, second option).
  Goldberg and Karzanov, Combinatorica 16(3), 1996. Tutte 1967; Nedev 1999; Bodlaender,
  Jansen and Kratsch 2011; Blum 1990. Van Gelder, "Toward leaner binary-clause reasoning
  in a satisfiability solver", 2005, closed access. Subramani 2004. Bacchus 2002 (two
  papers); Le Berre 2001. Halford, Wilson and Phillips 1998, body text. Maybery, Bain and
  Halford 1986; Ericsson and Kintsch 1995; Gobet and Simon 1996, 1998. Rips 1994, *The
  Psychology of Proof*, and Braine and O'Brien 1998, *Mental Logic* (the evidence that
  steps in a proof predict difficulty; D5, D7). Leone, Mills and Vaswani 2008; Křivánek
  2011; Fowler 2009. Berthier's proofs, the unread chapters of B2, B3 and B4, later
  editions of B2, and his Basic User Manual.
- Recalled, with no source opened: that a walk of a required parity or within a stated
  number of links is found by breadth-first search over (node, parity) or (node, links
  used); that a fewest-arc walk repeats no node; that a longest simple path is NP-hard;
  that hyper-binary closure detects every failed literal; that singleton arc consistency
  is the constraint-side name for shaving; that the smallest unsatisfiable core is hard;
  how SudoRules' typed chains treat repeats; the range of the Sudoku Explainer scale; that
  HoDoKu's SourceForge tree and the fork agree in the chain code; who coined "remote
  pair"; that solving guides describe chains as traced with alternating colours.

Gaps:

1. **The least load over distinct candidates** (D5): no source states the problem or its
   cost. The nearest result counts distinct links (I7 = L1).
2. **A human study of chain length** (D5, D7): not found by four searches. No measurement
   of a chain against people was read, and no human evidence for any figure of D7.
3. **The cost of a walk** (D1): that reachability within a number of links is one pass
   rests on textbook facts for which no source was opened.
4. **Confluence for singles with chains alone** (S06's question (d)): no source.
5. **Abstract only:** A5, A7, L5, L7 to L11, M9 to M11, T10. D3's inference that parity
   belongs to two cells leans on A5 and A7.
6. **Part read:** B2, B3, B4, A3, A4, A8, I3 to I6, I8 to I11, L2 to L4, L6, H2, E1, C7,
   C9, each with the parts named above.
7. **Solvers:** SudokuWiki's chain code is on the server; HoDoKu was read from an
   unofficial clone; Sudoku Explainer's documentation site did not resolve; HoDoKu's
   `doColoring` was not read, so that its colour pair is a whole component is not
   established (D9).
8. **Citations unsettled:** the pages of B5 = T4; the page range of Goldberg and Karzanov
   1996; the fuller venue of I2.
9. **Inferences handed to steps 5 and 6, none a published result:** that an endpoint
   strike of k links is a closure of k + 2; that a two-root forcing chain is a closure;
   that a static-link `forcing_chain` is inside failed-literal probing; that the
   specification's links meet Yinnone's condition; that a remote pair's component has no
   odd cycle.
10. **D8 and D9 have docs and source only**, added by the critic, with no second pass.

#### Step 4: the rebuild of S06's prototype, blind, then the cross-check

**The environment.** A throwaway venv under `ai_tmp/s11/venv`, made with `uv` 0.11.18
from the pixi interpreter (Python 3.14.7, conda-forge), `include-system-site-packages`
false; `python-sat` 1.9.dev15 (the version S06 used) and its dependency `six` 1.17.0;
solver name `cadical153`. pixi, `pyproject.toml` and every manifest are untouched. The
prototype and both chain programs use the standard library only; the solver is imported
by the SAT oracle alone. S06's other 211 grids were not fetched again: the cross-check
below read S06's scratch in its own worktree and copied its grid file. The install, the
reading of S06's scratch and the copy were each authorised by the maintainer on the day
("Deviations, and why" and "Open points settled").

**Blind.** The prototype for ranks 1 to 22, 28 and 29 was written from four sources and
nothing else: S06's ticket (whole), `technique.allium` L150-1166, the four study puzzles
of `docs/explanation/puzzle-design.md`, and this ticket. Nothing under any worktree's
`ai_tmp/s06` was opened. The fifteen offline grids were built again from the page and
S06's notes (four study puzzles, eight derived by S06's rule, three named); a solution
counter written for the purpose found one solution for each, the given counts S06 records,
and that S06's rule, applied afresh, takes out exactly the givens S06 lists.

**Fact: all fifteen rows reproduce under each profile, field for field, on the first run.**
Ranks 1 to 12 at capacity 4: 15 of 15 rows, 150 fields. Ranks 1 to 22, 28 and 29 at
capacity 729: 15 of 15 rows, 135 fields (that block of S06's notes has no givens column,
M7). The worked look of `study_beginner` reproduces (128 catalogue deductions, 103
entries, 16 at rank 2, 2,840 patterns, 174 to the first rank), as do the first eight
steps and each statement S06's notes make of the study puzzles. No definition was changed
after a figure was seen. Eleven ambiguities in S06's notes were each settled by a sentence
of the notes or the specification before the run, or recorded as unsettled by the fifteen
rows (which rank is "first" under a profile that hides; a `two_string_kite` with four
cells in one box; `full_unit` on a unit that repeats a digit).

**Then the cross-check, with S06's scratch in view.** The seven files of the blind phase
were snapshotted with their SHA-256 and verified unchanged afterwards; nothing under
S06's scratch was written to. `grids.txt` (226 lines) was copied byte for byte from S06's
worktree, sha256 `49d96f93ea1da57559492887481ab5837f3464c76e18f7753028e6fb076990a3`, and
its `PROVENANCE.md`,
`277bfa9d1a988a8298fbb714df146fd76ec1697cd67c5bb566bd1746a698d009`. The five raw sources
S06's ticket hashes (L1320-1330) have those hashes:

| Source | sha256 found, equal to S06's |
| --- | --- |
| `https://raw.githubusercontent.com/t-dillon/tdoku/master/data.zip` | `9be0601c721ac4e702e3fe097576f025fcb99b216aabfe9dbea37cac43e6bc4f` |
| `data/puzzles2_17_clue` | `2cf268735dfd40d32269b5f20ebbea067037e5976b4d016ce0b4d3c0a50deb30` |
| `data/puzzles1_unbiased` | `07a35f8a9cb394315c0b85bc73fea52ce552f4181acdcd0d550aace171498f40` |
| `http://magictour.free.fr/top1465` | `32837f38ece94e75678deadbe256aeafda704c8d4c2f8b5095a630ce2d0114d3` |
| `https://norvig.com/hardest.txt` | `398e1e5df4f50723078e3b510e00ea7eb9e104c521a9a186b4c20cd066c0459f` |

Not done: the grids were not built again from the raw files, so what is verified is that
the raw files are the ones hashed and that `grids.txt` is S06's file.

| | S06's stored result | Compared with the rebuild's | Size | Same |
| --- | --- | --- | --- | --- |
| A | the ticket's rows, S06 L1350-1575 | summary under ranks 1 to 12 | 226 rows, 2,260 fields | 226 of 226 |
| C | each grid's run under ranks 1 to 12 | each grid's run | 226 grids, 11,576 looks, 11,499 steps, 1,494,044 fields | 226 of 226 |
| C2 | the whole list at every look | the list of each look | 11,576 looks, 278,507 entries | all |
| D | sums by rank | sums by rank | 2,712 rows | all |
| E | summary under the wider profile | summary | 226 rows, 3,616 fields | 226 of 226 |
| F | each grid's run under the wider profile | each grid's run | 226 grids, 12,585 looks, 12,532 steps, 2,983,577 fields | 226 of 226 |
| F2 | the whole list at every look | the list of each look | 12,585 looks, 361,674 entries | all |
| G | sums by rank | sums by rank | 5,424 rows | all |

Each run comparison holds the end, steps, hardest rank, givens and final digits; for each
step its technique, output and kept witness (unit, cells, digits, role); for each look the
digits, every cell's candidates, the totals and each technique's counters. So the kept
witness and the role of all 640,181 entries were checked against S06's program, which the
fifteen rows could not do. A negative control (the wider lists against the narrower run)
reports 21,941 differences. Totals reproduced: 173 solved and 53 stalled under the wider
profile, 149 and 77 under ranks 1 to 12; **the 53 grids that stall are S06's 53**, with
the same steps, hardest rank, open cells, candidates and digits at the stall. Over the 53:
960 steps, 2,697 open cells and 9,091 candidates at the stalls.

**The one difference, of notation.** Where no step was taken (`inkala_2012` and
`sudokuwiki_unsolvable_49`, both profiles, 4 fields) S06's JSON holds `null` for the
hardest rank and the rebuild 0; S06's notes and its own summaries say 0. The rebuild
follows the notes. No other difference of any kind, and nothing in the rebuild was changed
after the blind phase.

Found by reading both programs, exercised by neither profile: S06's programs hide nothing
(no capacity, extent or marking), so the agreement says nothing of `Profile.sees` or of
the loads the rebuild reads for ranks 21, 22 and 29 (the witness's cells; the component's
nodes; every candidate in an open cell); and without the uniqueness promise S06's program
examines no pattern of ranks 28 and 29 where the rebuild still counts them. Agreement of
two programs is not correctness of either; S06's soundness check was not repeated and
carries over because the lists are equal entry for entry (inference).

#### Step 5: the definitions, fixed before any figure

The definitions were written as one file for two programmers who never speak, reviewed
(forty-five findings, each answered in the file's last section; no program was written or
run for the review), and fixed: SHA-256
`dcd25dfdd8578f53649e830f8915bc29dcab0b0f7dffc43bb7a9e9819cd0b328`, last modified
2026-10-03 16:47:23. The first file of either chain program was created at 17:08:10. The
file was never edited afterwards (the hash was taken again when the results were
assembled, when they were checked and after the decided run, equal each time); every later
finding is one of fifteen errata kept apart, below. A file time shows when a file was
last written, not what was in a head; that the order is as it looks is inference.

**The graph.** Cells are indexed row by row, `i = 9*(r-1) + (c-1)`. A node is a standing
candidate of an open cell, `n = 9*i + (d-1)`; a literal is `2n` (asserted) or `2n + 1`
(denied), and literals are ordered by that number. A weak link joins two distinct nodes
that are different digits of one cell or one digit in two peer cells. A strong link joins
the two candidates of a cell that has exactly two, or the two places of an unplaced digit
in a unit that has exactly two; every strong link is also weak. A link is its pair of
nodes and its kind, counted once whatever units give it. A weak link `{u, v}` licenses
`+u -> -v` and `+v -> -u`; a strong link `-u -> +v` and `-v -> +u` (`technique:952-963`).
So polarity alternates at every step, a path from a negative to a positive literal begins
and ends strong, the mirror of a path (reversed and negated) is a path, and there are at
most 1,458 literals and 23,328 implications. `x_chain` reads one digit's nodes and its
unit strong links; `xy_chain` the cells of exactly two candidates, their cell strong links
and weak links between peers on one digit; ranks 26 and 27 read everything. Out-neighbours
are always visited in ascending literal number.

**The counters**, each a function of the graph, the limit and the keys and not of a
program. `implications`: for each start literal of the technique, one breadth-first pass
to the depth the limit allows, counting every implication that leaves a literal reached
with at least one link to spare (starts: rank 24, each digit's negative literals; rank
25, the negative literals of cells of two candidates; ranks 26 and 27, every literal). It
is the same under every option. `patterns` equals it for ranks 24 to 27, so S06's sums
stay defined. `extensions`, `witnesses`, `deductions`, `witnesses_fewest` and `node_sets`
are stated only where every witness is listed (the small limit). `combinations` counts
the cases of rank 27. Two reference counters carry the cost of the dear options:
`dfs_inspections` (the directed no-repeat search) and `set_states` (the least-load
search). A program's own cost is `work` and is compared with nothing. Two counters are of
entries: `one_link`, the entries of rank 26 whose kept witness has one link; and
`interior`, the struck candidates that are nodes of the kept witness, defined to count
the strikes the alternative of definition choice 25 would drop. `interior` is in the
definitions and in no table of results, so no figure for it is carried here.

**The option keys** (the first value of each is the baseline the runs were held against,
except where "decided" says otherwise):

| Key | Values, with their meaning | Decided |
| --- | --- | --- |
| D1 | `walk`: any path. `no_literal_twice`. `no_candidate_twice`: no node twice in either sign, except that the last literal may be the negation of the first. `no_cell_twice`: that, and no cell entered twice, the first and last cell excepted. Applied to each path of a forcing chain apart | `walk` |
| D2 | `distinct`: an `x_chain` holds at least four distinct candidates, an `xy_chain` at least three distinct cells. `links_walked`: three links walked; for `xy_chain` three cell visits (five links), which contradicts 988 and was built only to be measured | `distinct` |
| D3 | `component`: one deduction for each two-coloured connected component, of at least four cells, of the graph of cells holding exactly `{a, b}`; targets see two cells of unlike colour within `max_chain_links` of each other. `per_chain`: one for each chain, as written. `per_strike`: one for each struck candidate | `component` |
| D3a | `adjacent_yes`: a pair one link apart counts, as written. `adjacent_no`: odd and at least three | `adjacent_yes` |
| D3l | `both_digits`: a remote pair's load counts both candidates of every cell. `one_per_cell` | not asked; under D5 as decided a remote pair reads 3 under either (inference) |
| D4 | `as_written`: an endpoint strike holds every target of its two endpoints; a closure and a forcing chain one candidate. `single_output`: one struck candidate or one placement throughout | `as_written` |
| D4c | `closures_all`: every closure is listed. `closures_residual`: a closure is dropped where a listed endpoint strike covers its candidate | `closures_all` |
| D5 | `fewest_links`: keep the witness of fewest links and hide it if its whole load is over capacity. `first_holdable`, `every_witness`, `least_load`: list an output when some witness is within capacity. `no_load`: reads 1. `link_by_link`: keep the fewest links and read `min(whole, paths + 2)`, which is 3 for any single chain of two links or more | `link_by_link` |
| D6a | `root_no`: no terminal is the root itself, by any path. `root_yes`: a root reaches itself by the path of no links | `root_no` |
| D6b | `per_root`: a contradiction counts 1 against `max_forcing_paths`, a verity its roots. `per_literal`: a contradiction counts its incompatible literals, a verity its roots | `per_literal` |
| D6c | `shortest_each`: the witness of least total links over all paths, each path the fewest. `least_load_together`. `any_holdable` | `shortest_each` |
| D6k | `kinds_five`: the five kinds of contradiction of 1005-1009. `kinds_three` | `kinds_five` |
| D6x | `rank27_all`: rank 27 lists everything it proves. `rank27_residual`: only what rank 26 does not list at that look | `rank27_all` |
| D7 | `(L, m)`: `max_chain_links` in 24, 16, 12, 8; `max_forcing_paths` 9 or lower | (16, 9) |
| D8 | `witness_cells`: a target is outside every cell of the witness (S06's reading). `named_cells`: outside only the two cells the sentence says the target sees; changes ranks 18, 19 and 20 only | `named_cells` |
| D9 | `separate`: one trap for each component, one wrap for each label with two nodes that see each other. `component`: one for each component. `per_target`: one for each trap target and wrap pair, the joining conjugate path as proof | `separate` |

**The five techniques.** A deduction of these ranks has no unit and extent
`multiple_units`; its cells and digits are those of the witness's nodes; a target counts
toward the load only if it is a node of the witness (definition choice 29). A witness with no output
is no deduction. An entry is one technique and one output, keeping one witness.

| Rank | Catalogue deduction and witness | Output | Canonical form and role | Whole load | Links against `L` |
| --- | --- | --- | --- | --- | --- |
| 23 `remote_pair`, `component` | a connected, two-colourable component of at least four cells that each hold exactly `{a, b}`, joined where cells are peers | strike `a` and `b` from every open cell outside the component that sees two cells of unlike colour at distance at most `L` | a set, nothing to settle; role `[m, cells ascending, a, b]` | `2m` under `both_digits`, `m` under `one_per_cell` | distance between the two cells seen; `links` recorded as `m - 1`; `patterns` is twice the joined pairs |
| 23, `per_chain` | a run of distinct such cells `c0..ck`, `3 <= k <= L` | for positions `p < q` with `q - p` odd, every open cell outside the chain that sees both | the chain and its reverse are one, `c0 < ck`; role `[k, c0, ..., ck]` | `2(k+1)` or `k+1` | `k` peer links |
| 24 `x_chain` | an endpoint strike on one digit: a path from `-s` to `+t`, `s` and `t` distinct, admitted by D1, meeting D2's minimum | strike every node other than `s` and `t` weakly linked to both in the whole graph; an interior node of the chain may be a target | the path and its mirror are one; it starts at the endpoint of lesser node number; role `[k, literals]` | distinct nodes on the path | `k`, always odd; at most 23 under 24, 15 under 16 |
| 25 `xy_chain` | the same over cells of two candidates, with the two endpoint digits equal | the same | the same | the same | the same; a chain through `q` cells once each has `2q - 1` links |
| 26 `alternating_inference_chain` | an endpoint strike over the whole graph, with no minimum; or a closure, a path from a literal to its negation | endpoint strike: as rank 24. Closure: strike the candidate if the start is positive, place it if negative | closure: the lesser of the path and its mirror as lists of literal numbers; role `[k, literals]` | distinct nodes | `k`, odd |
| 27 `forcing_chain` | contradiction: a root literal and one path to each terminal of a kind (p and not-p; two digits in one cell; one digit in peer cells; every candidate of a cell false; every place of a unit's unplaced digit false). Verity: roots that exhaust a case (both values of a candidate; all candidates of a cell; all places of a digit in a unit), one path from each to one conclusion | contradiction: the negation of the root. Verity: the conclusion | paths in ascending literal number of terminal or root; role `[total, code, anchor, P, then each path's links and literals]`, contradictions coded before verities | distinct nodes over all paths | each path apart, at most `L`, either parity; `links` is the total, `longest` the longest path |

Within one technique witnesses are compared by role alone, as lists of integers, element
by element, a proper prefix first: fewest links first (for rank 27, least total), and the
order is total. The list is ordered as S06 orders it: rank, a placement before strikes,
then the output. The profile hides first; repeats are then removed.

Inference stated in the definitions, and checked by one pair of runs (below): **the
duality lemma.**
Every verity has a contradiction with the same output, the same paths mirrored, the same
total and load, and a lesser code. So no verity is ever the witness an entry keeps (by the
numbering, not as a finding about grids); under `per_root` no `max_forcing_paths` of 1 or
more can bind; and the specification's one fixture for rank 27, a verity, is a member of
the catalogue and never a kept witness. Two more: `walk` with `distinct` still lists a
conjugate pair as an `x_chain` where a detour of five links over four candidates exists;
and rank 26, having no minimum, lists a chain of one strong link (what `pointing` or
`claiming` strikes).

**The searches.** Under `walk` with no minimum (rank 26, each path of rank 27): a
breadth-first pass from each start gives the fewest links; the kept path is the least list
of literal numbers among the walks of that length, built forward against the distance to
the goal. With D2's minimum (ranks 24 and 25): a capped-set search over states (literal,
the distinct nodes or cells seen so far, capped at four nodes or three cells), which is
polynomial; this is the search S06 left unargued (M4). The least-load question
("is some witness within capacity `c`?") is a set search over states (literal, set of
nodes) to depth `min(D, 2c - 1)`, exact but not polynomially bounded, under budgets; by a
lemma proved in the definitions a witness of load at most `c` exists exactly when one
exists within `2c - 1` links. The no-repeat keys: exhaustive under the small limit; under
the full limit a directed iterative deepening, budgeted. Rank 27: every (root, kind,
anchor) and (kind, anchor, conclusion) is a case; a case is admitted when its count is
within `max_forcing_paths`, D6a holds and every path exists. Rank 23: components by
breadth-first search. Eight budgets, each a count of a named thing with a stated scope
(never a timeout); a look where a search budget is hit is marked, and a grid is
**undecided** from its first marked look, a third class beside solved and stalled.

The eight budgets, as the definitions state them. A figure is the most of the thing
counted that one scope may reach; "hit" in these notes means a scope reached it:

| Budget | What is counted | Scope | Figure | On a hit |
| --- | --- | --- | --- | --- |
| `B_dfs` | implications read by the directed no-repeat search (`dfs_inspections`) | one query | 2,000,000 | the query is unknown |
| `B_dfs_look` | the same | all queries of one rank at one look | 50,000,000 | every query of that rank not yet answered at the look is unknown |
| `B_set` | distinct states (literal, set of nodes) of the least-load search (`set_states`) | one start at one capacity threshold | 200,000 | that start's outputs not yet answered are unknown, and treated as hidden |
| `B_set_look` | the same | all set searches of one look, all ranks | 20,000,000 | every question not yet answered at the look is unknown |
| `B_prod` | partial unions formed by the search over several paths' node sets (the least-load keys of D6c) | one case | 1,000,000 | the case is undecided; its output is unknown unless another case answers |
| `B_prod_look` | the same | all cases of one look | 20,000,000 | every output not yet answered is unknown |
| `B_rp` | the pairs of rank 23's `patterns` under `per_chain` | all pair graphs of one look | 5,000,000 | rank 23 lists nothing at that look |
| `B_small` | the extensions a small-limit listing would make, counted before listing | one state and technique | 50,000,000 | the state is listed at 5 links for that technique, or not at all, and named |

**Runs.** The baseline profile is S06's wider profile with all twenty-nine ranks. Chain
looks are lazy (definition choice 19): ranks 23 to 29 are searched only at looks where ranks 1 to 22
list nothing after hiding. `R0` is the run of ranks 1 to 22, 28 and 29 alone; `V` is the
grids that stall in `R0` or take a step of rank 28 or 29 in it; a chain run is over `V`
and the result over 226 is `R0`'s rows outside `V` and the chain run's inside. The
carry-over was checked on every grid and not assumed: with all 29 ranks over all 226
grids, 172 of 172, 171 of 171 and 171 of 171 grids outside `V` are the same run as in `R0`
at capacity 729, 8 and 4. The **54 sampled states** are the 53 states where `R0` stalls at
capacity 729 and look 28 of `study_advanced`; the static table makes every option's list
at those same states, like for like. A census searched ranks 23 to 27 at every look of the
baseline at capacity 729 on the fifteen offline grids.

**The definition choices**, each something neither the ticket nor the specification
states. Three were stated inside an option put to the maintainer and accepted with it:
7's "four" (in D2's option), 9 (in D5's) and 28 (in D6's). For 7 the alternative, three
distinct candidates, was never weighed, so it stays open and Q3 of the proposed text
asks it. No other was put. Those marked
*open* change a list, a load or what is hidden under the configuration as decided and are
not settled by these notes; the unmarked ones are not settled either, and change counters,
the witness shown, or nothing:

| No. | Choice made | Alternative, and whether measured |
| --- | --- | --- |
| 1 | a placed cell contributes no node | its digit is a node; no list changes on a kept grid |
| 2 | literals ordered by cell, digit, positive before negative | negative first; the witness shown only |
| 3 | one link for each pair of nodes and kind | one for each unit; counters only |
| 4 | counters are functions of the graph, limit and keys | each program's own count |
| 5 | D1 applies to each path of a forcing chain apart | across all paths of a proof; not measured |
| 6 | `no_cell_twice` is `no_candidate_twice` and no cell entered twice | no two literals in one cell, which forbids every `xy_chain` |
| 7 | *open:* `distinct` for `x_chain` is at least four distinct candidates | three distinct candidates, or three distinct links; not measured; three candidates would list more. "Four" was in the D2 option as put and accepted; the alternative was never weighed (Q3) |
| 8 | `no_load` reads 1 | 0 |
| 9 | `link_by_link` reads `min(whole, paths + 2)` | 2, or the candidates of one link; stated in the D5 question and accepted |
| 10 | under `per_literal` a verity counts its roots | it counts 1; no output changes |
| 11 | the key D3l; baseline `both_digits` | `one_per_cell`; run at capacity 8 and 4 |
| 12 | *open:* a component needs at least four cells | one holding a chain of four cells, which drops stars; any size; not measured |
| 13 | *open:* under `component` the limit bounds the distance between the two cells seen | it bounds the component's size, or does not apply; not measured |
| 14 | an endpoint strike has two distinct endpoints | a path from a literal to its negation listed under both; no list changes |
| 15 | witnesses of ranks 23 to 27 are compared by role alone, links first | cells and digits first, as ranks 13 to 22 |
| 16 | *open:* a forcing chain's fewest links is the least total over its paths | the least longest path; not measured |
| 17 | the eight budgets (the table under "The searches"); an unknown is treated as hidden and reported | other figures |
| 18 | *open:* the loads of ranks 21, 22 and 29 are the rebuild's readings | other readings; they are no row of the table of decisions |
| 19 | lazy chain looks, with one census. **A deviation from step 5** | search every rank at every look |
| 20 | `max_forcing_paths` run at 9, at 2 under `per_literal`, and at 3 to 8 only where a look shows it binds | run every figure |
| 21 | the small limit is 7 links, falling back to 5 over a stated count | 5 or 9 |
| 22 | the implication sample: every tenth implication of each of the 54 states, and every one on a kept witness | all |
| 23 | the hand examples are synthetic states, given to no SAT check | true grids |
| 24 | the provisional baseline keys | each alternative is a run |
| 25 | *open:* targets are read from the whole graph and may be interior nodes of the chain | targets outside the chain only (Berthier's form, B2); not built. The counter `interior` was defined to count the strikes it would drop and is in no table of results, so no count is carried. Not put to the maintainer: Q9 |
| 26 | `closures_residual` and `rank27_residual` are judged against what is listed at the same look | against the catalogue before hiding |
| 27 | the file formats | any other |
| 28 | rank 27's codes put contradictions before verities, so a contradiction is kept on a tie | verities first; stated in the D6 question and accepted |
| 29 | *open:* a target counts toward the load only if it is a node of the witness | the struck candidates count; not measured; moot for ranks 23 to 27 under D5 as decided |
| 30 | *open:* under `root_no` the root is never a terminal, by any path, where the question put was of an empty path | it may be one by a path of two links or more; not measured |
| 31 | a grid is undecided from its first marked look | leave out the marked look only |
| 32 | `D9=per_target`: a wrap strikes the label's nodes on the path only | the whole label |
| 33 | a dedicated, unbudgeted query on look 28 of `study_advanced` | rely on a listing |
| 34 | the static table on the 54 states | read "what it hides" from runs |
| 35 | fall back to a subset if `V` is over 80 grids | never used: `V` is 54 or 55 |

**The errata**, kept apart from the definitions, which were not edited:

| No. | Raised by | What |
| --- | --- | --- |
| 1 | second program | under the node keys of D1 fewest links is not always least load (an endpoint strike and a closure of one output with equal links); no list changes at capacity 4, 8 or 729 |
| 2 | second program | under `rank27_residual` the cases of an output rank 26 lists are dropped from rank 27's listings everywhere |
| 3 | second program | the path of no links is written as a line of rank 27's listing under `root_yes` |
| 4 | second program | `witnesses_fewest` and `node_sets` are not defined for rank 27 and are null |
| 5 | engine | the order of one look's set searches across thresholds is not stated; the order used is given |
| 6 | engine | a budget hit made only by an evidence question (`fits4`, `fits8`: is another witness within 4 or 8 candidates) marks the look though no list can change: one grid, `unbiased_044` |
| 7 | engine | what "past the limit" means for rank 23 under `component`, where the limit shrinks an output and removes no witness |
| 8 | engine | the columns of outputs past a limit under a no-repeat key are upper bounds at 24 |
| 9 | engine | the recorded closure of S06: a witness of 9 links over 8 candidates exists, which S06's record does not mention |
| 10 | engine | under `least_load_together` at the full limit the load recorded can be above the least |
| 11 | the comparison | under the small limit, a rank 27 witness under D6c that either program marks not exact is compared by output: 16 entry lines |
| 12 | the run phase | a run with a look marked by an evidence question alone was made a second time without the question, and both are quoted: 24 runs |
| 13 | the run phase | outputs past a limit that a player could hold were computed in four companion runs only |
| 14 | the SAT checks | the implication query cannot fail on a premise the one completion denies; a local form is asked beside it |
| 15 | the SAT checks | the checks hold more than the definitions name (ranks 18 to 22 under D8 and D9, 225 ceiling states, the no-limit lists); no no-limit list under a no-repeat key |

#### The counts, by technique and option

One program, the engine, made every run: 139 runs, all with exit status 0 (95 of the
matrix and the pairs of D7, 12 of `R0` alone, 3 with all 29 ranks over all 226 grids, the
census, 24 companions made without the evidence questions of erratum 12, and 4 more). No
output is unknown in any run. Every entry of ranks 23 to 27 at every chain look was held
against its grid's one solution: 127 runs, 7,406 grid runs, 20,491 looks, 734,725 entries,
0 bad. One budget was hit in any run: the set-search budget, by the evidence question at
rank 27 alone, at two looks of `unbiased_044` in 25 runs; that grid is the one undecided
grid below, and it ends solved.

Two kinds of evidence, not to be mixed. A **run** takes the first entry and looks again,
so two runs part at the first step one takes and the other does not; its "hidden" counts
outputs at its own chain looks. The **static table** makes each option's list at the same
54 states and is like for like.

The baseline in the runs (ranks 23 / 24 / 25 / 26 / 27 where five figures are given):

| Capacity | `R0`: solved / stalled | `V` | With chains: solved / stalled / undecided | The same without the evidence questions | Chain looks | Steps taken | Entries at chain looks | Hidden by capacity at chain looks |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 729 | 173 / 53 | 54 | 204 / 21 / 1 | 205 / 21 / 0 | 224 | 0 / 6 / 36 / 98 / 62 | 0 / 6 / 307 / 4,477 / 7,285 | 0 / 0 / 0 / 0 / 0 |
| 8 | 171 / 55 | 55 | 190 / 36 / 0 | | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 519 / 254 | 0 / 0 / 125 / 1,888 / 4,106 |
| 4 | 171 / 55 | 55 | 172 / 54 / 0 | | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 / 2 / 122 / 1,344 / 1,569 |

In the baseline run at capacity 729 the kept witnesses have, median and greatest: rank 24,
5 and 7 links, load 6 and 8; rank 25, 13 and 23 links, load 14 and 24; rank 26, 11 and 23
links, load 11 and 24; rank 27, 14 and 50 links over all paths, load 13 and 32. Cost over
its 224 chain looks: implications 108,982 / 108,917 / 4,052,736 / 4,055,316 for ranks 24
to 27, with 110,359,566 cases of rank 27; rank 23 examines 1,442 patterns under
`component` and 3,516 under `per_chain`. The directed search of the no-repeat keys reads
15,999,823 and 25,411,113 implications at ranks 26 and 27 under `no_candidate_twice`, and
12,688,437 and 34,598,391 under `no_cell_twice`.

The same counters by option, each run at capacity 729, 24 links and 9 paths, over its
own chain looks. A cell reads: `implications` in all; entries; links of the kept
witness, median and greatest; load read (the whole load), median and greatest. A key
not named reads as the baseline at the ranks it was run for (`no_literal_twice`,
`links_walked`, the two together, `per_literal` at 9 paths, `kinds_three`):

| Option | Rank 24 | Rank 25 | Rank 26 | Rank 27 |
| --- | --- | --- | --- | --- |
| baseline | 108,982; 6; 5, 7; 6, 8 | 108,917; 307; 13, 23; 14, 24 | 4,052,736; 4,477; 11, 23; 11, 24 | 4,055,316; 7,285; 14, 50; 13, 32 |
| `D1=no_candidate_twice` | 108,202; 5; 5, 7; 6, 8 | 106,936; 294; 13, 23; 14, 24 | 4,094,506; 3,693; 11, 23; 11, 24 | 4,097,310; 7,272; 14, 50; 13, 32 |
| `D1=no_cell_twice` | 108,202; 5; 5, 7; 6, 8 | 106,936; 294; 13, 23; 14, 24 | 4,094,506; 3,606; 11, 23; 11, 24 | 4,097,310; 7,268; 14, 50; 13, 32 |
| `D4=single_output` | 112,600; 6; 5, 7; 6, 8 | 115,689; 376; 13, 23; 14, 24 | 4,292,658; 3,794; 11, 23; 11, 24 | not carried |
| `D4c=closures_residual` | 107,761; 6; 5, 7; 6, 8 | 108,712; 307; 13, 23; 14, 24 | 4,006,669; 4,193; 11, 23; 12, 24 | not carried |
| `D6a=root_yes` | not carried | not carried | not carried | 4,055,316; 7,290; 14, 50; 13, 32 |
| `D6x=rank27_residual` | not carried | not carried | not carried | 4,055,316; 3,822; 17, 50; 16, 32 |

The totals differ between runs because each run has its own chain looks (224 under the
baseline, 230 under `single_output`, 221 under `closures_residual`). "Not carried"
marks a rank the scratch results do not tabulate under that option's own heading. The
rows of every key at capacity 8 and 4, of the keys of D5 and D6c, and of the pairs of D7
stayed in the scratch results and are not carried: see "Deviations, and why".

**Only three things change which grids solve**: `D5=no_load`, `D5=link_by_link`, and
`max_forcing_paths` 2 under `D6b=per_literal`. Every other key of D1 to D6, D8 and D9,
every link limit under `per_root`, and `max_forcing_paths` 3 and 4 under `per_literal` end
with the same grids solved as the baseline at that capacity. The runs that differ:

| Option | Capacity | Solved / stalled / undecided | Grids whose end differs from the baseline at that capacity |
| --- | --- | --- | --- |
| `D5=link_by_link` | 4 | 191 / 35 / 0 | 19, all solved here |
| `D5=link_by_link` | 8 | 204 / 21 / 1 (205 / 21 / 0 without the evidence questions) | 15, all solved here |
| `D5=no_load` | 4 | 204 / 21 / 1 (205 / 21 / 0) | 33, all solved here |
| `D5=no_load` | 8 | 204 / 21 / 1 (205 / 21 / 0) | 15, all solved here |
| `D6b=per_literal`, `max_forcing_paths` 2, at each of 24, 16, 12 and 8 links | 729 | 191 / 35 / 0 | 14, all stalled here |

The static table: entries listed, and in brackets outputs hidden by the capacity, summed
over the 54 sampled states at 24 links, at capacity 729, 8 and 4. Rank 23 lists nothing
on any of the 54 states under any key. A key set not named lists what the baseline lists
(`D2=links_walked` alone, every key of D3, `D6b=per_literal` at 9 paths):

| Key set | Rank 24: 729; 8; 4 | Rank 25: 729; 8; 4 | Rank 26: 729; 8; 4 | Rank 27: 729; 8; 4 |
| --- | --- | --- | --- | --- |
| baseline | 8; 8; 5 (3) | 85; 11 (74); 0 (85) | 1,121; 295 (826); 8 (1,113) | 1,286; 171 (1,115); 44 (1,242) |
| `D1=no_literal_twice`, alone or with `D2=links_walked` | 7; 7; 4 (3) | as baseline | as baseline | as baseline |
| `D1=no_candidate_twice` | 7; 7; 4 (3) | 79; 11 (68); 0 (79) | 930; 275 (655); 7 (923) | as baseline |
| `D1=no_cell_twice` | 7; 7; 4 (3) | 79; 11 (68); 0 (79) | 915; 270 (645); 7 (908) | 1,286; 170 (1,116); 44 (1,242) |
| `D4=single_output` | 7; 7; 4 (3) | 101; 12 (89); 0 (101) | 854; 266 (588); 7 (847) | as baseline |
| `D4c=closures_residual` | as baseline | as baseline | 1,057; 289 (1,039); 8 (1,549) | as baseline |
| `D5=every_witness`, `first_holdable`, `least_load` | as baseline | as baseline | 1,121; 314 (807); 8 (1,113) | as baseline |
| `D5=link_by_link` | 8; 8; 8 | 85; 85; 85 | 1,121; 1,121; 1,121 | 1,286; 1,286; 795 (491) |
| `D5=no_load` | 8; 8; 8 | 85; 85; 85 | 1,121; 1,121; 1,121 | 1,286; 1,286; 1,286 |
| `D6a=root_yes` | as baseline | as baseline | as baseline | 1,287; 171 (1,116); 44 (1,243) |
| `D6b=per_literal`, `max_forcing_paths` 2 | as baseline | as baseline | as baseline | 854; 141 (713); 44 (810) |
| `D6c=any_holdable`, `least_load_together` | as baseline | as baseline | as baseline | 1,286; 184 (1,102); 44 (1,242) |
| `D6k=kinds_three` | as baseline | as baseline | as baseline | 1,286; 165 (1,121); 44 (1,242) |
| `D6x=rank27_residual` | as baseline | as baseline | as baseline | 432; 55 (965); 39 (1,240) |

No output is unknown in any of the 312 static cells (26 key sets, three capacities, four
link limits). The candidates struck at capacity 729 over the 54 states: 7, 101, 590 and
961 for ranks 24 to 27 under the baseline; the no-repeat keys strike fewer at ranks 25 and
26 (93; 476 and 469) and the same at rank 27.

D8 and D9 touch ranks 18 to 22 and so `R0` itself; they are measured by the engine's runs
alone, over every look of all 226 grids. In `R0` at capacity 729: `w_wing` lists 2,638
entries under `witness_cells` and 2,767 under `named_cells`; `skyscraper` 1,687 and 3,422;
`two_string_kite` 4,075 and 4,131; the patterns examined are the same, nothing of ranks 18
to 20 is hidden at capacity 8 or 4 under either key, and no grid's end and no count of
steps at ranks 18 to 20 changes. `simple_colouring` lists 30,290 entries under `separate`,
29,704 under `component` and 60,358 under `per_target`; at capacity 4 `R0` hides 1,413,
1,366 and 327 catalogue deductions of rank 22 under the three, at capacity 8 137, 109 and
3 (loads that are the rebuild's readings, definition choice 18). `component` ends as the baseline at
every capacity; `per_target` changes `R0` at capacity 8 and 4 (172 solved and 54 stalled
against 171 and 55) and no grid's end with chains.

The census (the baseline at capacity 729, every look of the fifteen offline grids: 785
looks, 26 of them chain looks) is the only place the five ranks were counted at looks
where a lower rank lists something: entries 19 / 1,431 / 1,542 / 28,373 / 35,031 for
ranks 23 to 27. All 19 entries of `remote_pair` are at looks that are not chain looks.

**Which of the five apply on the 53 stalled states** (one look at each state where `R0`
stalls at capacity 729, 24 links, static): `remote_pair` on 0, `x_chain` on 3, `xy_chain`
on 9, `alternating_inference_chain` on 31, `forcing_chain` on 47; at least one of the five
on 47, with 0 / 3 / 84 / 1,032 / 1,221 entries. **None of the five has any output on six
states: `top1465_030`, `norvig_hardest_02`, `norvig_hardest_06`, `inkala_2012`,
`sudokuwiki_unsolvable_28` and `sudokuwiki_unsolvable_49`.** The counts of states are the
same under the four keys of D1; the entries differ (rank 26: 1,032 under `walk` and
`no_literal_twice`, 854 under `no_candidate_twice`, 844 under `no_cell_twice`). At
capacity 8 with loads held whole the five list something on 38 states and are wholly
hidden on 9; at capacity 4 they list something on 1 and are wholly hidden on 46.

#### The agreement of the two programs

The second program was written apart, from the definitions and `technique.allium` alone:
plain Python, eleven files, with other algorithms wherever the definitions allow (one
plain depth-first enumeration of every admitted path under the small limit, with entries
selected from the complete listing; one forward pass carrying least lists under the full
limit). It lists a state; it does not run a grid, and it does not build D8 or D9.

The two were compared on 64 states (the 54 sampled states, the seven hand examples of the
definitions and three near misses), under 37 key sets at capacity 729, 8 and 4: 119 cells
under the small limit and 136 under the full limit, the pairs of D7 among them; 854,075
records. **The small limit is 7 links** for one path (8 cells for rank 23): it covers the
minimums, the `xy_chain` fixture of four cells, a closure of each target of a strike of
three or five links, and the five-link detour; a count made before listing showed it can
be listed, and no state fell back to 5. It is the right size for comparing programs and
too small to be evidence for D5.

| | Small limit, 7 links | Full limit |
| --- | --- | --- |
| Cells | 119 | 136 |
| Graph files equal / compared | 7,616 / 7,616 | 8,704 / 8,704 |
| Lists (cell, state, rank) with equal outputs / compared | 38,080 / 38,080 | 43,520 / 43,520 |
| Entry lines in those lists | 116,814 | 194,091 |
| Entry lines compared by output only (a witness not marked exact) | 16 | 2,745 |
| Witness listing files equal / compared | 48,768 / 48,768 (31,155,814 lines) | 8,704 / 8,704 (rank 23's listing only, 190 lines) |
| Counters equal / compared | 373,184 / 373,184 | 243,712 / 243,712 |
| Records not equal in the final run | 0 | 0 |

- **Under the small limit: witness for witness** on every listing file and every counter,
  and entry for entry (rank, output and role) on all but 16 of 116,814 entry lines. The 16
  are rank 27 under `D6c=least_load_together` (12) and `any_holdable` (4), where the
  engine shows a stated stand-in and the second program a witness it calls exact; they
  agree by output and in whole load and differ in the role shown (erratum 11). Nothing
  checks by a second route that the second program's witness there is the least.
- **Under the full limit: output for output** on every list, and every counter compared.
  Of the 2,745 lines either program marks not exact, 354 differ in role (34 of rank 26,
  320 of rank 27) and 2,391 have the same role; of the lines both programs mark exact,
  none differs in role.
- **One difference was found, and the second program was wrong.** The first run had 4
  records not equal: at one state (`unbiased_007`'s stall), rank 27, under the two
  least-load keys of D6c at capacity 8, the engine counted 69 hidden and 0 unknown, the
  second program 68 and 1. The second program had not reduced its families of node sets to
  those least under inclusion, as the definitions state, and so ran past a cap of
  1,000,000 partial unions where the defined search does not. Ten lines of the second
  program were changed; the engine was not. After it: 0 not equal. The listed outputs were
  equal throughout.
- **The negative control.** Seven cells given things that must differ (24 links against 8;
  capacity 729 against 8; five kinds against three; `walk` against `no_candidate_twice`;
  `as_written` against `single_output`; `root_no` against `root_yes`; a tampered listing)
  each report differences: 187, 82, 98, 677, 56, 156 and 2 records not equal.
- **Budget hits, none silent.** The engine hit a budget at 32 (cell, state) looks, all
  under a least-load key at capacity 729 or under `no_candidate_twice` with `root_yes`;
  the second program reported none. Those looks are outside the claim of agreement by the
  definitions; at them 160 of 160 lists have equal outputs all the same (4,374 entry
  lines).
- **Each program against itself and the hand examples.** The engine: 82 cases and 2 graphs
  checked, 0 disagree. The second program: passed 81, failed 0, skipped 4 (rank 19 under
  D8, which it does not build). No hand figure was found wrong.
- **The closure S06 recorded** (look 28 of `study_advanced`, placing 4 at r2c8), by a
  dedicated query with no budget, in both programs with equal listings line for line:
  1,244 admitted paths to 15 links under `no_literal_twice`, 540 under
  `no_candidate_twice`, 368 under `no_cell_twice`. The fewest links are 9. A proof of 9
  links over 9 candidates exists under every key; one of 11 links over 8 exists under a
  walk and under neither node key. **The least load is 8, and it is reached at 9 links
  already** (4 witnesses), which S06's record does not mention (erratum 9). So the one
  recorded case is not a case where the fewest links miss the least load.
- **Not covered by the agreement:** D8 and D9; a no-repeat key crossed with a least-load
  key of D6c at the full limit; the evidence questions, the least load itself, the
  outputs past a limit and the two reference counters, which the second program does not
  report; and any run of a grid from its givens. Past 7 links the two are compared by
  output, which does not compare the witnesses neither lists.

#### The SAT checks

One encoding, S06's: 729 variables, the rules as 11,988 clauses, and for a state a unit
clause against every digit that is not a cell's candidate. One solver for each state
(`cadical153`), each query a call under assumptions. The oracle reads no technique,
witness, role, load or order and shares no code with either program. Its self-test, before
any check, against a backtracking solver written for it: 42 cases, 0 disagree. A negative
control of the checker on planted entries: 9 of 9 unsound entries reported, and the 3
states with a planted contradiction reported as without a model.

**Soundness.** Every entry gathered is sound against the digits and candidates standing
at its look: **138,077 distinct (state, output) pairs checked, 138,077 sound, 0 unsound,
0 malformed**, by 188,142 queries. They stand for 3,183,751 entry lines: 734,725 of ranks
23 to 27 in the runs, 289,124 in the 312 static lists, 437,000 in the comparison made
again (both programs), 147,233 distinct outputs of the engine's witness listings, 337,905
in the lists made on the ceiling states, and 1,237,764 of ranks 18, 19, 20 and 22 under D8,
D9 and the baseline. By rank, pairs checked and sound (a pair listed under two ranks is
counted under each, so these sum to more than 138,077): 18, 3,461; 19, 3,890; 20, 6,121;
22, 83,080; 23, 12; 24, 1,276; 25, 1,663; 26, 29,957; 27, 42,205. No defect to list.

**States with no model, reported as failures and not as soundness.** Of 10,656 distinct
states, 10,646 have a model and each of those exactly one. Ten have none: the ten hand
states (the seven examples of the definitions and three near misses), which are synthetic
states with inert filler and which the definitions give to no check. The 303 pairs and 423
implications on them were not checked and are not counted as sound. No other state is
without a model.

**The ceiling**: every candidate of an open cell that no completion of the state uses, by
one query each; an upper bound on what any sound deduction can strike at that look.

| Sample | States | Candidates of open cells | The ceiling |
| --- | --- | --- | --- |
| the 53 stalled states | 53 | 9,091 | 6,394 |
| the 54 sampled states | 54 | 9,159 | 6,432 |
| every ceiling state (the 54 and every chain look of the baseline run at 729) | 225 | 35,866 | 24,603 |

Every struck candidate of every list lies within the ceiling (0 outside, over the 312
static cells and the 32 cells made on the ceiling states). Each technique's share, under
the baseline keys at capacity 729 (candidates struck, united by state):

| Sample | Link limit | Rank 23 | Rank 24 | Rank 25 | Rank 26 | Rank 27 | Union of the five | Cut off from the union by the limit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 225 states | none | 0 | 10 | 310 (1.3%) | 2,505 (10.2%) | 5,477 (22.3%) | 5,477 (22.3%) | 0 |
| 225 states | 24 | 0 | 10 | 298 (1.2%) | 2,481 (10.1%) | 5,477 (22.3%) | 5,477 (22.3%) | 0 |
| 225 states | 16 | 0 | 10 | 254 (1.0%) | 2,165 (8.8%) | 5,450 (22.2%) | 5,450 (22.2%) | 27 |
| 225 states | 12 | 0 | 10 | 146 (0.6%) | 1,665 (6.8%) | 5,147 (20.9%) | 5,147 (20.9%) | 330 |
| 225 states | 8 | 0 | 10 | 34 (0.1%) | 885 (3.6%) | 3,778 (15.4%) | 3,778 (15.4%) | 1,699 |
| the 53 stalled states | none | 0 | 3 | 103 (1.6%) | 552 (8.6%) | 923 (14.4%) | 923 (14.4%) | 0 |
| the 53 stalled states | 24 | 0 | 3 | 100 (1.6%) | 552 (8.6%) | 923 (14.4%) | 923 (14.4%) | 0 |
| the 53 stalled states | 16 | 0 | 3 | 87 (1.4%) | 490 (7.7%) | 920 (14.4%) | 920 (14.4%) | 3 |
| the 53 stalled states | 12 | 0 | 3 | 69 (1.1%) | 362 (5.7%) | 896 (14.0%) | 896 (14.0%) | 27 |
| the 53 stalled states | 8 | 0 | 3 | 11 (0.2%) | 206 (3.2%) | 755 (11.8%) | 755 (11.8%) | 168 |

- **What a chain of unbounded length reaches.** With no limit on links the union is the
  same 5,477 as at 24: the limit of 24 cuts 12 struck candidates from rank 25 and 24 from
  rank 26, all of which rank 27 still strikes within 24. 16 links cut 27 of the union, 12
  cut 330, 8 cut 1,699. The no-limit lists equal a fixpoint written apart (not a SAT
  check) on 225 of 225 states at each of ranks 24 to 27.
- **19,126 of the 24,603 candidates of the ceiling are struck by no chain of ranks 23 to
  27 of any length** at that look (5,471 of 6,394 on the 53 stalled states). Inference: a
  longer limit cannot help there.
- **At capacity 729 the union is rank 27's own set**: the other four strike nothing rank
  27 does not, except under `rank27_residual`.
- **Each option's share**, capacity 729, 24 links, 225 states: the union is 5,477 under
  every key set but `D6b=per_literal` with `max_forcing_paths` 2, which reaches 2,505
  (10.2%). Within ranks, `no_candidate_twice` and `no_cell_twice` bring rank 26 to 2,017
  and 1,986 and rank 25 to 281; `rank27_residual` leaves rank 27 with 2,996.
- **Capacity cuts more than any limit.** With loads held whole the baseline's union on the
  225 states is 1,013 (4.1%) at capacity 8 and 30 (0.1%) at capacity 4; on the 54 states,
  276 and 30 of 6,432. On the 54 states `D5=no_load` reaches at capacity 8 and 4 what the
  baseline reaches at 729 (961); `D5=link_by_link` reaches 961 at capacity 8 and 590 at
  capacity 4.

**The static graph.** 215,852 distinct implications checked, 0 fail: the stated sample
(every tenth implication of each of the 54 states and every one on a kept witness of the
baseline, 13,775), every implication of the 54 states' graphs (92,112), and every
implication on a kept witness of every entry of ranks 24 to 27 in the runs (127,674); the
three sets overlap, so their counts sum to more than the 215,852 distinct. The
query as the ticket states it can fail only where the premise is true in the state's one
completion (61,055 of them); a local form (one cell's or one unit's clauses and the
candidates standing there) was asked beside it and can fail on any; none does (erratum
14).

**What the checks do not show.** That a pattern named is the technique claimed; that a
witness is the least in links, load or order; that a list or the graph is complete;
anything of load, capacity, order, counters or budgets. Those are the second program's.
Not given to the check at all: ranks 1 to 17, 21, 28 and 29; ranks 18, 19, 20 and 22
outside the D8, D9 and baseline runs; the hand states. (The decided run's check, below,
added rank 21.) No list with no limit on links was made under a no-repeat key or under
any key set but the baseline. The fixpoint behind "no limit" has no second reader.

#### Every pair of limits D7 names

`L` is `max_chain_links`, `m` is `max_forcing_paths`. Each row is a run over all 226 grids
by the method above; five figures are ranks 23 / 24 / 25 / 26 / 27. In brackets, the same
run made without the evidence questions (erratum 12).

| Keys | Capacity | `L` | `m` | Solved / stalled / undecided | Chain looks | Steps taken | Entries at chain looks | Grids whose end differs from `L` 24, `m` 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| baseline (`per_root`) | 729 | 24 | 9 | 204 / 21 / 1 (205 / 21 / 0) | 224 | 0 / 6 / 36 / 98 / 62 | 0 / 6 / 307 / 4,477 / 7,285 | 0 |
| baseline | 729 | 16 | 9 | 204 / 21 / 1 (205 / 21 / 0) | 221 | 0 / 5 / 35 / 97 / 62 | 0 / 5 / 251 / 3,431 / 6,956 | 0 |
| baseline | 729 | 12 | 9 | 205 / 21 / 0 | 201 | 0 / 4 / 23 / 90 / 62 | 0 / 4 / 93 / 1,858 / 5,369 | 0 |
| baseline | 729 | 8 | 9 | 205 / 21 / 0 | 220 | 0 / 4 / 15 / 113 / 66 | 0 / 4 / 24 / 774 / 5,066 | 0 |
| baseline | 8 | 24 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 519 / 254 | 0 |
| baseline | 8 | 16 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 519 / 254 | 0 |
| baseline | 8 | 12 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 519 / 254 | 0 |
| baseline | 8 | 8 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 508 / 254 | 0 |
| baseline | 4 | 24 | 9 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| baseline | 4 | 16 | 9 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| baseline | 4 | 12 | 9 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| baseline | 4 | 8 | 9 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| baseline, the check of the duality lemma | 729 | 24 | 2 | 204 / 21 / 1 (205 / 21 / 0) | 224 | 0 / 6 / 36 / 98 / 62 | 0 / 6 / 307 / 4,477 / 7,285 | 0 |
| `D6b=per_literal` | 729 | 24 | 2 | 191 / 35 / 0 | 132 | 0 / 5 / 23 / 65 / 0 | 0 / 5 / 214 / 3,118 / 2,474 | 14 |
| `D6b=per_literal` | 729 | 16 | 2 | 191 / 35 / 0 | 129 | 0 / 4 / 23 / 63 / 0 | 0 / 4 / 174 / 2,485 / 2,343 | 14 |
| `D6b=per_literal` | 729 | 12 | 2 | 191 / 35 / 0 | 117 | 0 / 3 / 18 / 56 / 1 | 0 / 3 / 79 / 1,393 / 1,795 | 14 |
| `D6b=per_literal` | 729 | 8 | 2 | 191 / 35 / 0 | 130 | 0 / 3 / 12 / 69 / 7 | 0 / 3 / 20 / 574 / 1,908 | 14 |
| `D6b=per_literal` | 8 | 24 | 2 | 190 / 36 / 0 | 120 | 0 / 4 / 11 / 65 / 0 | 0 / 4 / 19 / 506 / 162 | 0 |
| `D6b=per_literal` | 4 | 24 | 2 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| `D6b=per_literal` | 729 | 24 | 3 | 204 / 21 / 1 (205 / 21 / 0) | 219 | 0 / 6 / 36 / 98 / 57 | 0 / 6 / 307 / 4,477 / 7,115 | 0 |
| `D6b=per_literal` | 729 | 24 | 4 | 204 / 21 / 1 (205 / 21 / 0) | 224 | 0 / 6 / 36 / 98 / 62 | 0 / 6 / 307 / 4,477 / 7,271 | 0 |
| `D6b=per_literal` | 729 | 24 | 9 | 204 / 21 / 1 (205 / 21 / 0) | 224 | 0 / 6 / 36 / 98 / 62 | 0 / 6 / 307 / 4,477 / 7,285 | 0 |
| `D6b=per_literal` | 8 | 24 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 519 / 254 | 0 |
| `D6b=per_literal` | 4 | 24 | 9 | 172 / 54 / 0 | 63 | 0 / 3 / 0 / 0 / 0 | 0 / 3 / 0 / 3 / 0 | 0 |
| `D5=first_holdable` | 8 | 24 | 9 | 190 / 36 / 0 | 146 | 0 / 4 / 11 / 72 / 19 | 0 / 4 / 19 / 521 / 251 | 0 |
| `D5=first_holdable` | 8 | 12 | 9 | 190 / 36 / 0 | 146 | 0 / 4 / 11 / 72 / 19 | 0 / 4 / 19 / 521 / 251 | 0 |
| `D5=first_holdable` | 8 | 8 | 9 | 190 / 36 / 0 | 147 | 0 / 4 / 11 / 73 / 19 | 0 / 4 / 19 / 508 / 254 | 0 |

- **`max_chain_links`.** At each capacity 16, 12 and 8 links end with the same grids solved
  as 24. A lower limit shifts the work up the ladder: at capacity 729 and 8 links rank 26
  takes 113 steps against 98 and rank 27 66 against 62. Where the limit binds, at the
  baseline's own chain looks at capacity 729, of the outputs with no limit on links: rank
  25, 8 / 64 / 194 / 289 of 315 are past 24 / 16 / 12 / 8 links; rank 26, 65 / 857 / 1,978
  / 3,617 of 4,542; rank 27, 0 / 41 / 434 / 2,347 of 7,285; ranks 23 and 24, none. 24
  links is never met at ranks 24 to 26 (their chains have an odd number of links); 23 is.
- **`max_forcing_paths` under `per_root` never binds**, at any figure from 1: the least
  count is 1 on every entry of rank 27, and the run at 2 lists what the run at 9 lists,
  12,075 entry lines equal line for line. That is the duality lemma, checked.
- **Under `per_literal`, 2, 3 and 4 bind and 5 to 8 never do.** The least count over the
  baseline's 7,285 entries of rank 27 at 24 links: 2 on 3,515, 3 on 3,605, 4 on 151, 5 on
  14. So a limit of 2 would cut 3,770, 3 would cut 165, 4 would cut 14, and 5 or more
  none; the greatest count kept is 5. At 2, 14 grids that solve at capacity 729 stall.
- **Pairs not run:** `(16, 3)`, `(12, 3)`, `(8, 3)`, `(16, 4)`, `(12, 4)` and `(8, 4)`
  under `per_literal`, and any `m` other than 9 and 2 at capacity 8 or 4. The definitions
  (definition choice 20) run 2 at every `L` and the other binding figures at 24 links only; the
  entries each would cut at 16, 12 and 8 links are counted (173, 159 and 158 for 3; 10,
  9 and 2 for 4) and no run shows how the grids then go on.

#### The completed table of decisions (step 7)

Completed on 2026-10-03, before any decision was put to the maintainer. How to read it:

- Every figure is copied from the results of steps 5 and 6, which the sections above
  summarise; a figure not repeated above was copied from the same scratch tables of
  results those sections were made from. None was computed for the table, so two
  figures stand side by side ("191 against 172") and the difference is left unsaid.
- **Fact** is what a file says or a command printed. **Inference** is reasoning no
  command checked, and is marked where it occurs.
- The letters with a number (H = HoDoKu, E = Sudoku Explainer, W = SudokuWiki,
  P = Sudopedia, B = Berthier, I, A, L = the papers on implications, paths and least
  proofs, M = the memory literature, T = rating, C = the sources added for D8 and D9)
  are the keys of the sources of step 3, listed earlier in these notes. "Definition
  choice N" is the numbered choice of the definitions of step 5, listed earlier in
  these notes. `XC2`, `XC3`, `NP` and `RP` are hand states of those definitions,
  synthetic and with no solution: `RP` is the remote-pair state of five pair cells
  (`{1,2}` at r1c1, r1c4, r4c4, r4c2 and r7c2), the "hand example" of the D3 rows; `XC2` has the digit 1 at r1c1, r1c2 and r2c3 only; `XC3` has it
  at r1c4, r1c9, r2c4 and r2c6 only; `NP` has `{1,2}` at r1c1 and r1c2 and
  `{1,2,7,8,9}` in every other cell ("Handed back", part 3, gives what each lists).
- "Runs" are each run's own looks: a run solves a grid step by step, so two runs part
  at the first step they take differently. "54 states" is like for like: the 53 states
  where the grids stall without chains, and one more, the same looks under every
  option. Grid counts are solved / stalled / undecided of 226.
- A chain look is a look where ranks 1 to 22 list nothing, so only a chain can go on.
  The baseline is the key set every option was held against. `R0` is a run of ranks 1
  to 22, 28 and 29 alone. The union is the candidates struck by any of ranks 23 to 27
  at a look. The `fits` questions ("is some witness within 4, within 8 candidates")
  were asked of every entry to gather D5's evidence; one budget hit of theirs leaves
  one grid (`unbiased_044`) undecided though it ends solved, so grid counts read
  204 / 21 / 1 with them and 205 / 21 / 0 without.
- Capacities: 4 is the expert preset's; the runs also use 8, and 729, which hides
  nothing.
- The D7 rows keep the letters A to G; no other row uses a letter for an option. A
  budget named in a cell (`B_dfs`, `B_rp`, `B_set`, `B_set_look`) is one of the eight
  of step 5, and a figure beside it is a count of hits. A cell that says "nothing
  measured" is a stated gap, not a figure.
- The column "Recommended" is the recommendation made before the maintainer answered.
  The answers are under "Open points settled"; D7's differs from its recommendation.

| Id | Option | Literature | Cost | Lists | Hidden at 4 and at 8 | Sentences changed | Recommended |
| --- | --- | --- | --- | --- | --- | --- | --- |
| D1 | `walk`: a chain may return to a candidate | SAT and graph papers put no condition on the path (I1, I5, I7, I8, I9); player-facing definitions exclude repeats (B1, B3, H1, H2, W2) | a stated polynomial search at every rank, one pass at ranks 26 and 27; rank 26 779,935 implications on the 54 states, no directed search; but the dearest option under which to ask D5's least-load question | 54 states: 8 / 85 / 1,121 / 1,286 entries at ranks 24 to 27; 204 / 21 / 1 of 226 (205 / 21 / 0 without the `fits` questions); 47 of the 53 stalled states have an entry, the same 47 under every option | runs, at 8: 0 / 125 / 1,888 / 4,106; at 4: 2 / 122 / 1,344 / 1,569. 54 states, rank 26: 826 at 8, 1,113 at 4 | `technique.allium` 981-999, 1002-1004 added to; 78-84 if "walk" becomes a word; `human-solving.md:247` | **yes**, as the ticket began; at ranks 26 and 27 the kept witness holds no literal twice, at no cost; the measurements do not tell it from `no_literal_twice` |
| D1 | `no_literal_twice`: no literal twice, a candidate once in each sign (added by step 2(a)) | Sudoku Explainer, by inference (E1); shortest 2-SAT proofs (I7 = L1) | as `walk` at ranks 26, 27; exhaustive only (no argued bound) at ranks 24, 25, where the directed search measured 59 and 1,245 on the 54 states and hit no budget | as `walk` but rank 24 on the 54 states: 7 entries, same candidates struck (the entry dropped is a conjugate pair shown as an `x_chain` round a loop); same grids | as `walk` (rank 24 on the 54 states: 7 / 0 at 8, 4 / 3 at 4, entries / hidden) | `technique.allium` 981-999, 1002-1004 added to | no, narrowly: same lists but one entry; no argued bound at ranks 24, 25 |
| D1 | `no_candidate_twice`: no candidate twice, a closure's two ends excepted | "2n different candidates" (B1), W2; polynomial by inference only, key paper unopened (A1 to A3) | exhaustive only; directed search 5,670,459 (rank 26) and 8,628,342 (rank 27) on the 54 states at 24 links (108,565 and 595,983 at 16, the static table); no budget hit in a run or static list; the budget `B_dfs` was hit with `D6a=root_yes` in the comparison | 54 states: 7 / 79 / 930 / 1,286; the same grids solve as under `walk`, all 29 ranks held (205 / 21 / 0: `set_states` is 0 here, so the `fits` question that marks one grid under `walk` hits no budget); the same 47 of 53; union struck unchanged | runs, at 8: 0 / 116 / 1,525 / 4,106; at 4: 2 / 114 / 1,095 / 1,569. 54 states, rank 26: 655 at 8, 923 at 4 (fewer exist to hide) | as `walk`, and 992-993 given its exception; `human-solving.allium:965-970` by extension | no |
| D1 | `no_cell_twice`: as above and no cell entered twice (HoDoKu's rule) | H1, H2, B3; the general hard problem, 2^O(k)·E (A8) | exhaustive only; directed search 4,061,178 and 12,077,181 on the 54 states | 54 states: 7 / 79 / 915 / 1,286; the same grids solve as under `walk`, all 29 ranks held (205 / 21 / 0: `set_states` is 0 here, so the `fits` question that marks one grid under `walk` hits no budget); the same 47 of 53; union struck unchanged | runs, at 8: 0 / 116 / 1,482 / 4,100; at 4: 2 / 114 / 1,079 / 1,569. 54 states, rank 26: 645 at 8, 908 at 4 (fewer exist to hide) | as above, and 978's rule carried to ranks 24 to 27 with what "a cell twice" means | no |
| D2 | `D2=distinct`: `x_chain` at least four distinct candidates (definition choice 7), `xy_chain` at least three distinct cells | supports it: every stated minimum is over things that cannot recur (P1, P2, W3, B3, B2, B6; H1 with H7); polynomial for a small fixed minimum (by counting, in the record of step 3; A8 by its abstract, by inference) | `implications` the same under both keys, the counter being the same under every option: 108,982 and 108,917 at ranks 24 and 25 over the 224 chain looks of the run at 729; `set_states` on the 54 states at 729: rank 24 370, rank 25 7,185, the `fits` questions' search; no budget hit | 54 states at 729: rank 24 8 entries, rank 25 85; 204 solved, 21 stalled, 1 undecided of 226; of the 53 stalled states 3 gain a rank 24 entry, 9 a rank 25; `XC2` lists nothing at rank 24 | runs: at 8 rank 24 0, rank 25 125; at 4 rank 24 2, rank 25 122. 54 states: at 8 0 and 74; at 4 3 and 85 | `technique.allium:985` text; `:988` none; `human-solving.md:247` clarify; reach, effort, lapse extension; `catalogue_version` | **yes** (the ticket's starting recommendation, with definition choice 7 stated) |
| D2 | `D2=links_walked`: `x_chain` at least 3 links walked, `xy_chain` 5 | no source that states a minimum for either counts a repeated link; links as the unit where nothing repeats (H1, H3, H7); implications walked for a cycle (E1, by inference no literal twice) | `implications` as `distinct`, figure for figure (108,982 and 108,917); `set_states` on the 54 states at 729: rank 24 789, rank 25 9,780, the `fits` questions' search; no budget hit | the same as `distinct` on the 54 states and the 226 grids; on the hand states, `XC2` lists a conjugate pair as an `x_chain` of load 2 and `NP` a naked pair as two `xy_chain` entries of 7 links and load 4 (kept lists of the comparison) | the same as `distinct`, runs and 54 states; the `NP` entries are listed at capacity 4 | `technique.allium:985` clarify; `:988` text, contradicted as written; `human-solving.md:247` clarify | no |
| D2 | as written, each in its own unit (added by the reading; no key) | no source mixes units (inference from P1, P2, W3, B3, H1, H7) | nothing measured as a set; by inference `links_walked`'s rank 24, `distinct`'s rank 25 | nothing measured as a set; by inference as `distinct` and `links_walked` on the grids, `links_walked`'s on `XC2`, `distinct`'s on `NP` | nothing measured as a set; by inference as `distinct` and `links_walked` | `technique.allium:985` clarify (without it the silence stays); `human-solving.md:247` clarify | no (the nearest rival) |
| D2 | no minimum (added by the literature; no key, not built) | Berthier sets none (B2, B3); HoDoKu's code would take two cells (H1, by inference); against: P1, P2, W3 | nothing measured for ranks 24 and 25 | nothing measured; at rank 26, which has none, `one_link` (the entries whose kept witness is one link) is 0 over every chain look | nothing measured | `technique.allium:985` and `:988` text, minimums removed | no |
| D3 | `per_chain`: one for each chain (as written; `technique:973-978`) | H1, P3 for it, both with the neighbours left out (H1 by inference); H7 keeps the shorter | `patterns` 3,516 over 224 chain looks at 729, most at a look 268; needs budget `B_rp` (not hit); S06 paper bound about 10 million witnesses | 0 entries at any chain look of a true grid (no census under this key); 204 / 21 / 1 of 226 as baseline; 53 stalled states gain 0; hand state `RP`: 1 entry from 3 witnesses | runs 0 and 0; 54 states 0 and 0; `RP` hidden at 4 (load 8), at 8 nothing measured | none contradicted; new text for load, "odd", "each once" | no |
| D3 | `component`: one for the two-coloured group (baseline) | no source lists it under this name; W1's definition by parity, and A5, A7, lean to it by inference; W1, H5, B3, E1 (E4) say the technique is subsumed or absent, which is the last row's case | `patterns` 1,442 over the same looks, most 28; no budget; S06 paper bound 20 entries | 0 entries at any chain look of a true grid; 204 / 21 / 1; 53 stalled states gain 0; census 19 entries, none at a chain look; SAT 12 pairs of rank 23, 12 sound; hand state `RP`: 1 entry from 1 witness | runs 0 and 0; 54 states 0 and 0; `RP` load 10 (5 with one a cell, hidden at 4); under `D5=link_by_link` the load read is 3, not measured | `technique:973-978`, 1094-1095; `effort:314`; `human-solving.md:284`; extensions in reach, effort, lapse, human-solving | **yes**, as the ticket's start; confidence low on lists, moderate on cost |
| D3 | `per_strike`: one for each struck candidate | no source offers it | as `per_chain`: 3,516, most 268 | 0 entries at any chain look of a true grid (no census under this key); 204 / 21 / 1; hand state `RP`: 3 entries from 9 deductions | runs 0 and 0; 54 states 0 and 0; `RP` nothing measured | `technique:973-978`, exception to 905-906; `reach:238`, 383-390 | no |
| D3 | switch D3a: `adjacent_yes` (as written) or `adjacent_no` | `adjacent_no`: P3, and H1 by inference; `adjacent_yes`: no source states it (H5 and W1 do not leave the neighbours out, by inference) | unchanged: 1,442, or 3,516 with `per_chain` | 0 entries either way; `RP` under `per_chain`+`adjacent_no`: 3 entries against 1 | runs 0 and 0; 54 states 0 and 0 | `adjacent_no`: `technique:975-977` | `adjacent_yes` |
| D3 | switch D3l: load counts `both_digits` or `one_per_cell` | none | unchanged: 874 at 8, 448 at 4 | 0 entries either way; 190 / 36 at 8, 172 / 54 at 4 | runs 0 and 0; 54 states 0 and 0; `RP` at 4: `per_chain` listed at load 4 under `one_per_cell`, hidden at load 8 under `both_digits` | a new sentence either way (`technique:912-913` or `RemotePairs`) | decide with D5 |
| D3 | not built: no `remote_pair` (raised by W1, H5, B3, E1) | W1, H5, B3, E1, E4 | nothing measured | nothing measured; rank 23 took 0 steps in every run | nothing measured | `technique:642-644`, 692-699, 973-978, 1094-1095 | no; outside the ticket's options |
| D4 | `as_written`, `closures_all`: an endpoint strike holds every target, a closure its one candidate | the practice: H5, P1, P2, W3; H2, P4, W2, E1; I5, I7, I8 | baseline: rank 26 implications 779,935 on the 54 states | 54 states at 729, ranks 24 / 25 / 26: 8 / 85 / 1,121 entries; 204 solved, 21 stalled, 1 undecided of 226; 53 stalled states: 47 with an entry | 54 states, rank 26 hidden: 1,113 at 4, 826 at 8; runs: 1,344 and 1,888 | `technique.allium` 906-907 and 992-999 clarified; models none | **yes** (the ticket's starting recommendation) |
| D4 | `single_output`: one struck candidate or one placement to an entry | no solver read does it; nearest B2, B6 | the same implications; run 230 chain looks against 224 | 7 / 101 / 854 entries; same candidates struck (590 at rank 26); same grids | rank 26 hidden: 847 at 4, 588 at 8; runs: 1,028 and 1,513; same candidates left (6 and 238) | `technique.allium` 905-907, 983-984 rewritten; reach 238, 383-390, effort 110-112, human-solving 916-917 by extension | no |
| D4 | `D4c=closures_residual`: no closure where a listed endpoint strike covers its candidate | no source states it; HoDoKu shows a one-strike chain only as a closure (H5, H2); the overlap is a record's inference | the same implications; run 221 chain looks | rank 26: 1,057 entries; same candidates struck; same grids | rank 26 hidden: 1,549 at 4, 1,039 at 8; runs: 1,867 and 2,427; list depends on capacity as built (inference; the other form, definition choice 26, not measured) | a sentence added at `technique.allium` 992-999; 314-315 by inference | no; nearest alternative, to be weighed with D5 |
| D4 | `single_output` with `closures_residual` | none | nothing measured (not in the matrix) | hand fixture only: 26 entries, as `single_output` alone | nothing measured | as the two together | no |
| D4 | a continuous loop as its own output (no key) | H5, W4 | nothing measured (excluded by 998-999, not built) | nothing measured | nothing measured | 998-999 rewritten | no |
| D4 | a placing closure recorded as strikes (no key) | H2, H5 | nothing measured (not built) | nothing measured | nothing measured | 992-993 rewritten | no |
| D5 | `fewest_links`: keep fewest links, read its whole load | what every solver keeps (H1, H2, H7, W2; E2 = T5 nearest); none hides by load | implications only; no set search (inference) | runs: 190 / 36 solved / stalled at 8, 172 / 54 at 4; entries at 8: 0 / 4 / 19 / 519 / 254 | run: at 8, 125 / 1,888 / 4,106 of ranks 25 to 27; at 4, 2 / 122 / 1,344 / 1,569 of ranks 24 to 27. 54 states: lists 485 of 2,500 at 8, 57 at 4 | `technique.allium` new sentence and order; `:1162` answered yes; `effort.allium:314` "by its length" | fallback |
| D5 | `first_holdable`: hide only if no chain fits, show the first that fits | no precedent; the search unpublished for candidates (I7 = L1, L2, gap 1) | set search, not polynomially bounded: 9,041,366 `set_states` at rank 26 in its run at 8, no budget hit at 8 or 4 | same grids as `fewest_links`; 54 states at 8: 314 against 295 at rank 26 | 54 states: at 8, 74 / 807 / 1,115 of ranks 25 to 27; at 4 as `fewest_links`. Run at 8: 118 / 1,840 / 4,056 | `reach.allium:369-372` conflict; `:238`; `effort.allium:266-270`; `:314` unanswered | no |
| D5 | `every_witness`: each chain its own deduction (as written) | no solver lists every chain (H1, H7) | list as `first_holdable`; counting witnesses exhaustive only | as `first_holdable` | as `first_holdable` | one new sentence for ranks 13 to 29; models extension; effort needs least load (inference) | no |
| D5 | `least_load`: keep the chain over fewest candidates | polynomial for links (I7 = L1), unstated for candidates (gap 1), NP-hard unrestricted (L2) | 11,594,757 `set_states` at rank 26 at 8; the exact least load at capacity 729 on the 54 states hit budgets at rank 26 (`B_set` 253 times, `B_set_look` 914 times), 0 outputs unknown | as `first_holdable`; least below kept load on 28 and 13 entries | as `first_holdable` | nothing recorded; as `fewest_links` by inference | no |
| D5 | `no_load`: chains never hidden by capacity | called unsupported on one second-hand report (M7; the studies unread); two raters price length (E2 = T5, T7), HoDoKu's score does not (H4, T6) | implications only (inference); longer runs | 205 / 21 solved / stalled at 8 and at 4 (without evidence questions) | nothing at 4 or 8, run and 54 states | `technique.allium:598-599`, `:204`; `reach.allium:7-9, 100-103, 383-390`; `effort.allium:29-30, 100-108, 314`; `puzzle-design.md:918, 920` | no |
| D5 | `link_by_link`: load is 3 for a chain, paths + 2 for a forcing chain (as built `min(whole, paths + 2)`, definition choice 9) | fits by transfer, no chain measured (M1, M3, M4, M5, M2; M9 abstract only) | implications only (inference) | 205 / 21 at 8 (without evidence questions); 191 / 35 at 4; ranks 23 to 26 as `no_load` | at 8 nothing; at 4 rank 27 only: 2,378 in the run, 491 on the 54 states | `technique.allium:598-599, 912, 1162`; `reach.allium:7-9, 100-103, 383-390`; `effort.allium:29-30, 100-108, 314`; `human-solving.allium:772-775` | **yes** |
| D6 | `D6a=root_no`: a root is never its own target | HoDoKu's stated intent; a root reaching its negation is a closure (H2, H5, E1) | baseline: 4,055,316 implications, 110,359,566 `combinations`, run at 729 | 7,285 entries; 204 / 21 / 1 of 226; 47 of 53 stalled states | runs 1,569 and 4,106; 54 states 1,242 and 1,115 | `technique.allium:1005-1012` one sentence | yes |
| D6 | `D6a=root_yes`: a root reaches itself by no links | Sudoku Explainer admits, SudokuWiki shows one (E1, W5) | as baseline | 7,290 against 7,285; 1,287 against 1,286 on the 54; same struck, same grids | runs 1,571 and 4,112; 54 states 1,243 and 1,116 | `technique.allium:1005-1012`; a sentence on the path of no links | no |
| D6 | `D6b=per_root`: a path counted for each root | verity: one for each root (H2, E1, W5, P6); memory, by inference (M7) | as baseline | as baseline; the limit cannot bind from 1 to 9 | as baseline | `technique.allium:1002-1003`; `human-solving.md:248`; `puzzle-design.md:925` | no |
| D6 | `D6b=per_literal`: a path counted for each path held | both counts in use, each for its kind (H2); I4 | as baseline at 9 | at 9 as baseline, line for line; limits 2, 3, 4 bind; at 2: 191 / 35 | at 9 as baseline; at 2: runs 1,031 and 1,607, 54 states 810 and 713 | `technique.allium:1002-1003`; `human-solving.md:248`; `puzzle-design.md:925` (9 binds under neither count) | yes, at 9 |
| D6 | `D6c=shortest_each`: fewest links in all | what HoDoKu and Sudoku Explainer do (H4, E2) | `set_states` 18,992, run at 8 (the spike's `fits` questions; the option needs none) | as baseline; no verity ever shown (definition choice 28) | runs 1,569 and 4,106; 54 states 1,242 and 1,115; another witness within 8 for 13 of 1,115, within 4 for none | `technique.allium` new witness sentence; `effort.allium:100-108` extension | yes, load as D5 decides |
| D6 | `D6c=least_load_together`: least load over all paths | a directed Steiner problem (L3, L4, L5, by inference); B2 by inference | `set_states` 15,820,170, run at 8; budget hit at 729; stand-in witness | at 8: 184 against 171 on the 54, 300 against 254 in the run; same grids | runs 1,569 and 4,453; 54 states 1,242 and 1,102 | as above, least load named | no |
| D6 | `D6c=any_holdable`: the first witness the player can hold | none: no solver asks what a player holds (the sources of step 3, under D5) | `set_states` 15,571,679, run at 8; 40 stand-ins | as `least_load_together` | runs 1,569 and 4,453; 54 states 1,242 and 1,102 | witness sentence names the profile; `reach.allium:369-372` conflict (inference) | no |
| D6 | `D6k=kinds_five`: five kinds of contradiction | HoDoKu recognises them (H2) | `combinations` 110,359,566 | as baseline | as baseline | none | yes |
| D6 | `D6k=kinds_three`: three kinds | none reduces to three (H2, H6) | `combinations` 54,262,068 | same entries at 729; 1,016 against 1,063 at 8 links; same grids | runs 1,569 and 4,114; 54 states 1,242 and 1,121 | `technique.allium:1005-1009` | no |
| D6 | `D6x=rank27_all`: rank 27 lists all it proves | both families kept in HoDoKu and Sudoku Explainer (H2, H5, E1) | as baseline | as baseline; up to 112 entries at a look | as baseline | none | yes |
| D6 | `D6x=rank27_residual`: only what rank 26 does not list | one narrow precedent (H2) | implications and `combinations` as baseline; no search saved (inference) | 3,822 against 7,285; 432 against 1,286 on the 54; union and grids unchanged | runs 1,566 and 3,758; 54 states 1,240 and 965 | `technique.allium:915-916`; `reach.allium:369-372` (inference) | no |
| D7 | A: `(24, 9)`, stay | no source uses 24; HoDoKu 20, SudokuWiki twelve cells, Sudoku Explainer none (H1, H4, H7, W3, E1, E2); no human evidence; a search budget by inference (M4) | 54 states, ranks 26 / 27: 779,935 / 780,087 implications; run at 729: 4,052,736 / 4,055,316 | 729: 204 solved / 21 stalled / 1 undecided of 226; entries 0 / 6 / 307 / 4,477 / 7,285; 53 stalled states: union 923 (14.4%); cuts 0 from the union (baseline keys) | runs: at 8 hidden 0 / 0 / 125 / 1,888 / 4,106 (190 / 36), at 4 0 / 2 / 122 / 1,344 / 1,569 (172 / 54); 54 states: at 8 0 / 0 / 74 / 826 / 1,115, at 4 0 / 3 / 85 / 1,113 / 1,242 | none for the figure; add the unit at `technique.allium:1003`, `:982`; record what binds | **yes** |
| D7 | B: `(16, 9)` | none for 16; Berthier's largest whip 16 is another unit and a ceiling (B5 = T4, B2) | 54 states: 761,858 / 768,328; under `no_candidate_twice` rank 26 `dfs_inspections` 108,565 against 5,670,459 | 729: 204 / 21 / 1, same grids; entries 0 / 5 / 251 / 3,431 / 6,956; cuts 27 of 5,477 (225 states), 3 of 923 (53 states) | lists as A at 8 and 4; runs: hidden at 8 0 / 0 / 105 / 1,608 / 4,034, at 4 0 / 2 / 100 / 1,150 / 1,563; 54 states: at 8 0 / 0 / 60 / 679 / 1,109, at 4 0 / 3 / 71 / 966 / 1,236 | `technique.allium:529`, `:536`, `:527-528` "smaller"; `human-solving.md:247`, `:130`, `:257-258`; `human-solving.allium:108` | no; weigh again if D1 is a no-repeat key |
| D7 | C: `(12, 9)` | none; W3's twelve is cells; whips within 7 or 9 are ceilings (B5 = T4, B2) | 54 states: 699,832 / 719,507 | 729: 205 / 21 / 0, same grids; entries 0 / 4 / 93 / 1,858 / 5,369; cuts 330 (225 states), 27 (53 states) | lists as A at 8 and 4; runs: hidden at 8 0 / 0 / 62 / 967 / 3,735, at 4 0 / 2 / 59 / 767 / 1,522; 54 states: at 8 0 / 0 / 40 / 384 / 1,075, at 4 0 / 3 / 51 / 671 / 1,202 | as B | no |
| D7 | D: `(8, 9)` | none; equals the standard player's `reasoning_depth` of 8 (`human-solving.allium:485`) | 54 states: 541,830 / 584,377 | 729: 205 / 21 / 0, same grids, hardest rank of the solved grids shifts (5 / 13 / 15 against 10 / 9 / 14); entries 0 / 4 / 24 / 774 / 5,066; cuts 1,699 (225 states), 168 (53 states) | at 8 rank 26 lists 508 against 519 (run), 279 against 295 (54 states); 28 outputs cut that capacity 8 could hold; runs: hidden at 8 0 / 0 / 0 / 0 / 2,792, at 4 0 / 2 / 12 / 275 / 1,296; 54 states: at 8 0 / 0 / 0 / 0 / 892, at 4 0 / 3 / 11 / 271 / 1,019 | as B | no |
| D7 | E: `m` below 9, `D6b=per_root` | no solver has the limit (H2, E1); verity and contradiction are duals (H6) | `(24, 2)`: as A to the figure | as A: 12,075 entry lines equal at `m` = 2; no figure binds | nothing measured (729 only); as A by the lemma (inference) | `technique.allium:530`, `:536`; `human-solving.md:248` | no: changes nothing |
| D7 | F: `m` below 9, `D6b=per_literal` | HoDoKu counts a contradiction this way (H2); two paths are a rank 26 chain by inference (B2, I1, I6) | `(24, 2)` rank 27: 2,348,480 implications over 132 chain looks | `m` = 2: 191 / 35, 14 grids lost, rank 27 takes 0 steps, 53 stalled states 552 against 923; `m` = 3, 4: same grids, rank 27 entries 7,115, 7,271 against 7,285; 5 to 8 cut none of the entries measured | `m` = 2 runs: at 8 hidden 0 / 0 / 125 / 1,751 / 1,607 (190 / 36), at 4 0 / 2 / 122 / 1,344 / 1,031 (172 / 54); 54 states rank 27: at 8 141 listed, 713 hidden, at 4 44 and 810; `m` = 3, 4: nothing measured at capacity 8 or 4 (run at 729 and 24 links only), and `m` = 2 below 24 links was run at 729 only: a stated gap | as E; the count is D6b's sentence (`technique.allium:1002`) | no |
| D7 | G: no limit on links (added) | Sudoku Explainer sets none (E1, E2); HoDoKu none for AICs and forcing chains (H1, H4, H7) | nothing measured | union 5,477 on 225 states and 923 on the 53, as A; grids: nothing measured | nothing measured; no output past 24 is holdable at 8 or 4 | `technique.allium:527-530` and every guarantee that reads the limit; `effort.allium:69-71`, `:312` | no |
| D8 | `witness_cells`: a target is outside every cell of the witness (S06's reading) | no source states it (C5 to C9, docs and source, no paper); HoDoKu's kite strikes one cell outside the pattern and its W-Wing comment says "forbidden" of the case (C7; the second read otherwise by inference) | `patterns` equal under both: 378,044,604 / 8,155,080 / 9,174,465 at ranks 18 / 19 / 20, `R0`, 729 | entries 2,638 / 1,687 / 4,075 (`R0`, 729); `R0` 173 solved, 53 stalled of 226; the 53 gain nothing | runs: 0 and 0; 54 states: nothing measured (not in the static table) | `technique.allium:929, 934, 941, 944` gain an antecedent ("outside the pattern"); models: extension only | no |
| D8 | `named_cells`: a target is other than the two named cells only | no page read states an exclusion (C5, C6, C8, C9: silence); HoDoKu's code does this for the W-Wing and the skyscraper, by inference, and not for the kite (C7, read in part); no strike of a pattern cell seen on a real grid | the same, figure for figure | entries 2,767 / 3,422 / 4,131 (`R0`, 729); same 173 / 53; same steps 25 / 17 / 18; ranks 23 to 27 line for line; 0 unsound; the 53 gain nothing | runs: 0 and 0; 54 states: nothing measured (not in the static table) | the same four lines, in the form of `:984`; a fixture to add; `catalogue_version` by inference | **yes**, low to moderate confidence; rests on `:984` as written, so decide D4 first or with it |
| D9 | `separate`: one trap for each component, one wrap for each wrapped label; witness the component | C1 to C4 keep the two apart; `separate` by inference; critic only, no paper | `patterns` 455,916 (`R0`, 729); same under all keys on the same looks | 30,290 entries (`R0`, 729); `R0` 173/53 at 729, 171/55 at 8 and 4; chains 204/21/1, 190/36, 172/54; the 53: no gain | runs, loads by definition choice 18: `R0` 1,413 at 4, 137 at 8; with chains 1,447 and 180; 54 states: not measured | `technique.allium:965-971` one sentence added (and which is listed first); 905-906 a note; 912-913 if "supporting" is named | **yes** (the ticket's start; the text leans the other way) |
| D9 | `component`: one deduction for each component, wrap and trap strikes together | none lists them as one (C1 to C4) | same `patterns` | 29,704 entries; grids as `separate` at every capacity, 0 ends differ; the 53: no gain | runs, definition choice 18: `R0` 1,366 at 4, 109 at 8; with chains 1,393 and 145; 54 states: not measured | `technique.allium:965-971` one sentence; `human-solving.md:283`; `reach.allium:238` by extension | no; the specification's lean, a tie on the measurements |
| D9 | `per_target`: one for each trap target and each wrap pair, the conjugate path as proof and load (added by step 2(a)) | no precedent in what was read (the sources of step 3) | same `patterns` on the same looks; path search not counted | 60,358 entries; `R0` 173/53 at 729, 172/54 at 8 and 4 (one grid more); chains as baseline; the 53: no gain | runs, load the path's (definition choice 18): `R0` 327 at 4, 3 at 8; with chains 327 and 3; not like for like; 54 states: not measured | `technique.allium:965-971` rewritten, 968 narrowed (definition choice 32), 912-913, 905-906, 1091-1093; `effort.allium:314` touched | no; revisit after D5 |

#### Where two decisions cannot be made apart

What no option changes, said once (fact): soundness (138,077 pairs of state and output
checked with the SAT solver, 138,077 sound, under every option built), and which grids
solve with all twenty-nine ranks held: 204 / 21 / 1 at capacity 729 (205 / 21 / 0
without the `fits` questions), 190 / 36 / 0 at 8, 172 / 54 / 0 at 4, under every option
of D1, D2, D3, D4, D8 and D9, under every option of D6 while the path limit is 9, and
under every link limit of D7. Without the chain techniques: 173 / 53 at 729, 171 / 55
at 8 and at 4. Only three answers change a grid's end: `D5=no_load` (205 / 21 at
capacity 8 and at 4, without the `fits` questions), `D5=link_by_link` (205 / 21 at
capacity 8, without them, and 191 / 35 / 0 at capacity 4), and `D6b=per_literal` with
the path limit at 2 (191 / 35 at capacity 729). So most of the nine decide what a step
looks like, what it is called and what the specification must say, not how hard a grid
is; the one that decides what a preset player can do is D5.

The couplings:

1. **D1 with D2.** D2 has no force apart from D1. Under a walk or "no literal twice"
   the minimums must say what they count; under "no candidate twice" or "no cell twice"
   the two keys of D2 agree and only wording is left. D1 is also where D2's leftover is
   mended: under a walk, "distinct" still lets a conjugate pair through as an `x_chain`
   by a detour, and any no-repeat answer to D1 takes that entry away (fact on the 54
   states: 7 entries of rank 24 against 8, the same 7 candidates struck).
2. **D1 with D5.** Under a walk or "no literal twice" the chain of fewest links is not
   always the one of least load (the recorded closure: 9 links over 9 candidates kept,
   9 links over 8 exists). The least-load search is dear only there (`set_states` at
   rank 26 over the run 19,370,994 against 0 under the two strict keys), and its
   budgets were hit. So a walk goes with a D5 that keeps the fewest links
   (`fewest_links`, `link_by_link`, `no_load`) or with a budgeted question.
3. **D5 with D6c: one answer, not two.** D6c is D5 for rank 27. `fewest_links`,
   `link_by_link` and `no_load` go with `shortest_each`; `first_holdable` with
   `any_holdable`; `least_load` with `least_load_together` (inference).
4. **D5 with D6b, D6a and the tie of rank 27.** Under `link_by_link` a forcing chain's
   load is its paths plus two, the paths being those of at least one link in the proof
   kept. Under `root_no` that is `per_literal`'s count, so `per_literal` gives "path"
   one meaning for the limit and for the load. The load also turns on which proof is
   kept (D6c) and on the rule that a contradiction is kept before a verity (definition
   choice 28, the other order not built). D5's reading is answered first, then D6; the
   sentence for rank 27's load is written after both.
5. **D6b with D7: one decision in two rows.** At 9 the path limit binds under neither
   count. Counted for each root, no figure from 1 to 9 binds (the least count is 1 on
   all 7,285 entries). Counted for each path held, 2 cuts 3,770 entries, 3 cuts 165,
   4 cuts 14.
6. **D5 with D7.** Under the load as written, capacity hides almost every long chain
   and a link limit of 12 or more is invisible to capacity 4 and 8 (the lists there
   are the baseline's at 24, 16 and 12). Under `link_by_link` or `no_load` it is not:
   the link limit is then the only bound on the length of a chain a player is shown
   (inference).
7. **D5 with D3, D4c and D9.** Under `link_by_link` the load read for a remote pair is
   3 under either key of D3 (the definition's figure; not measured, since no remote
   pair stood at a chain look), so the switch D3l decides nothing and `component`'s
   cost to a player goes. The measured objection to `D4c=closures_residual` (its list
   depends on capacity) goes too (inference; no run crosses D4 with D5). And if the
   answer reaches rank 22, `D9=per_target` loses its one measured advantage
   (inference). A D5 answer for ranks 23 to 27 alone leaves ranks 21, 22 and 29 on the
   whole count (definition choice 18): that must be said, or decided with it.
8. **D4 with D8,** through `technique.allium:984`. With 984 as written (a chain may
   strike its own interior), `named_cells` is the consistent partner; `witness_cells`
   has rank 19 withhold a strike that rank 24 makes on the same four cells (inference;
   shown on the hand state `XC3`).
9. **D3 with D9, and both with D4.** `D3=component` copies a colouring listed by
   component, which fits `D9=separate` or `D9=component` and not `per_target`. Under
   `D4=single_output` the partners are `D3=per_strike` and `D9=per_target`.
10. **D1 with D7 and D6.** A link limit saves little search under a walk and much under
    a no-repeat key. The link limit also sets where rank 26 ends and rank 27 begins:
    two forcing paths of up to 24 links join into a closure of up to 48 (inference).

Pairs that are incoherent, each from the evidence above:

- a walk with `D2=links_walked` for `xy_chain` (contradicts `technique.allium:988`);
- a walk with a least load required exact at every capacity (budgets hit);
- a strict D1 key chosen in order to show easy witnesses (it takes the 8-candidate
  witness of the recorded closure away);
- `D3=component` "as `simple_colouring` has it" with `D9=per_target`;
- `D3=per_strike` or `D9=per_target` with `D4=as_written`; `D4=single_output` with
  `D3=component` unless rank 23 is named as an exception;
- `D9=per_target` with a D5 that reads a chain's whole load, unsaid why;
- D3l decided one way and D5's "what a proposition is" the other;
- `D8=witness_cells` with `technique.allium:984` as written;
- `D4c=closures_residual` as built with a D5 that hides by capacity;
- "some witness fits" for ranks 24 to 26 and shortest-only for rank 27, or the reverse;
  either least-load option of D6c under `no_load` or `link_by_link`;
- `D6b=per_root` beside a load that counts every path; `per_root` with a lower path
  limit meant to bind; `per_literal` with a path limit of 2;
- `D5=first_holdable` at capacity 8 with a link limit of 8;
- any link limit below 24 with `reasoning_depth` ranging `0..24` and the word "smaller"
  at `technique.allium:527-528` left as they are;
- `no_load` or `link_by_link` with `effort.allium:100-108` left as it is.

The order the couplings ask for, which step 8 followed: D1, D2, D5, D4, D8, D9, D3, D6,
D7. D1 first (D2, D5, D6 and D7 read it); D5 before D4's third option, D3's load, D6
and D7; D4 before or with D8; D9 with D3; D6b with D7.

#### The recommendation made for each

Each is the recommendation of step 7, made by this ticket's agent before the maintainer
was asked. It decides nothing; the answers are under "Open points settled".

| Decision | Recommended | Reason |
| --- | --- | --- |
| D1 | a walk, for ranks 24 to 27 | It is the only option with a stated polynomial search at every rank and it lists the most; no measurement tells it from "no literal twice". |
| D2 | distinct: four distinct candidates for `x_chain` (definition choice 7), three distinct cells for `xy_chain` | Every published minimum is over things that cannot recur, and `technique.allium:988` already says "distinct". |
| D3 | the component, neighbours counted; its load decided by D5 | It has the paper bound and one witness; no grid separates the options, since rank 23 has no entry at any chain look. |
| D4 | as written, closures listed beside endpoint strikes | It is what the solvers read show and changes the fewest sentences; nothing struck or solved differs under the other options built. |
| D5 | held link by link, the chain of fewest links kept | It gives an expert preset that can use chains (191 grids at capacity 4 against 172 under the load as written); the formula is the spike's own, with no source measuring a chain on people. |
| D6 | root never its own target; paths counted for each path held; fewest links kept; five kinds; rank 27 lists all it proves | It is the shorter definition and gives "path" one meaning with D5; at 9 no option changes which grids solve. |
| D7 | stay at 24 and 9, and record what binds | No lower figure gains anything measured under a walk; under link by link the confidence between 24 and 16 is less. The maintainer chose 16. |
| D8 | `named_cells` | It is consistent with `technique.allium:984` as D4 keeps it; the measurements do not choose, and confidence is low to moderate. |
| D9 | separate, the component as witness, the trap listed first as built | Every source read keeps the two apart (by inference) and it is S06's reading; against `component` it is a tie on the measurements. |

What the recommended set commits to, knowingly: capacity then hides no single chain
from any preset (the load read at ranks 24 to 26 has a greatest of 3; the lowest preset
capacity is 3), only forcing chains of too many paths; the kept chains of rank 26 run
to a median of 11 links and a greatest of 23 at 24 links; so `max_chain_links`, one
figure for every player, is the only bound on the length of a chain a player is shown.

#### Recorded, not settled

**The ladder's order past rank 12** (a non-goal). Nothing here orders the ranks; the
order of ranks 23 to 27 in every run is the specification's as it stands. What the
measurements show, each a fact:

- Rank 23 does no work where ranks 1 to 22 have stopped: no entry at any chain look of
  any run made before the decisions; in the census (fifteen grids, every look) 19
  entries, all at looks that are not chain looks. (The decided configuration, run
  afterwards, lists 9 entries of rank 23 at capacity 3 and none at 729, 8 or 4.)
- Which rank is the lowest of the five with an entry, at the baseline's 224 chain looks
  at capacity 729: rank 24 at 6, rank 25 at 36, rank 26 at 98, rank 27 at 62, none
  at 22.
- Rank 27 covers the four below it: at capacity 729 it strikes everything ranks 23 to
  26 strike at the same look, so by the list's order it is the step only where they
  list nothing.
- Cost against rank, median implications a look at 729: rank 24 439, rank 25 132.5,
  rank 26 14,200, rank 27 14,200 (rank 23: 6 patterns).
- Load against rank: the kept `xy_chain` (rank 25) is heavier than the kept
  `alternating_inference_chain` (rank 26), median load 14 against 11. At capacity 8
  rank 25 lists 19 entries in the run against rank 26's 519.
- The link limit shifts the hardest rank of the grids that solve (5 / 13 / 15 grids at
  ranks 25 / 26 / 27 at 8 links against 10 / 9 / 14 at 24), so an order should not be
  settled on figures made at one limit and applied at another (inference).
- On the 53 stalled states at 729: `remote_pair` applies on 0, `x_chain` on 3,
  `xy_chain` on 9, `alternating_inference_chain` on 31, `forcing_chain` on 47; none of
  the five has any output on 6.

**Calibration** (a non-goal).

- `R0` loses two grids to capacity (173 at 729, 171 at 8 and at 4). The loads behind
  that (ranks 21, 22 and 29) are the rebuild's readings, not decisions (definition
  choice 18).
- With chains under the baseline: 205 at 729 (without the `fits` questions), 190 at 8,
  172 at 4. At capacity 4 the five take 3 steps in all, every one of rank 24.
- A ceiling on any calibration of these five: 19,126 of 24,603 candidates no completion
  uses are struck by none of the five at any length; 21 of 226 grids stall with all
  twenty-nine ranks at capacity 729.
- D5's recommendation is a reading of load, not a calibration. By inference no capacity
  in range does the same work: the kept chains read a whole load of median 11 and
  greatest 24 at rank 26, against a capacity ranging 1 to 12.

**The ticket's three open points**, with what the evidence says; how each was settled
is under "Open points settled".

- The ladder's order, with these decisions or after: the evidence says after
  (inference): the order turns on the load reading (D5) and the link limit (D7), and
  rank 25 measures heavier than rank 26.
- D8 and D9, here or in the specification change S06 drafted: both rest on the first
  program alone, with soundness, and no outcome measured waits on either. But D8 is
  coupled to D4 through `technique.allium:984` and D9 to D3, which by inference speaks
  for answering them here, so that one change states every reading.
- A run at capacity 4: it was made (fact). The baseline and the keys of D1 to D6, D8
  and D9 were run at capacity 4 as well as at 8, and the static table has 729, 8 and 4.
  The exceptions are pairs of D7: a path limit of 3 or 4, and of 2 below 24 links, were
  run at 729 only. What the decisions hide from the expert preset is measured (under
  the load as written, 57 of 2,500 chain outputs listed on the 54 states and 172 grids
  solved), not only argued.

#### Limits a reader of the decisions should know

- **One program behind every run.** The second program lists states and runs no grid;
  the two agree on 64 states (854,075 records, 0 not equal). The check of the results
  made 10,381 table cells again, 0 differ, and made ranks 26 and 27 again under the
  baseline keys; it did not make again ranks 23 to 25, the no-repeat keys at the full
  limit, D4c, D5 and D6 beyond the baseline, D8 or D9.
- **D8 and D9** are not in the second program and not in the static table; no
  like-for-like figure on the 54 states exists for either.
- **Rank 23**: its lists rest on the hand states alone; no true grid separates D3's
  options.
- **Chain looks only** (definition choice 19): nothing says what any key but the
  baseline at 729 lists at looks where a lower rank still lists something.
- **Budgets.** In the runs one budget was hit, by one `fits` question at rank 27, on
  one grid, with 0 outputs unknown. In the static lists the hits are all under
  `D5=least_load` and `D6c=least_load_together` at capacity 729: the exact least load
  past 8 candidates is not known for 735 entries of rank 26 and 896 of rank 27, and
  the least load rests on the first program alone.
- **Not run**: `(16, 3)`, `(12, 3)`, `(8, 3)`, `(16, 4)`, `(12, 4)`, `(8, 4)` under
  `per_literal`; no list with no link limit under a no-repeat key (what 24 cuts there
  is an upper bound); no run below 24 links under a no-repeat key, `link_by_link` or
  `no_load` before the decisions (the decided configuration was run at 16 after them,
  see "Deviations, and why"); no profile without `forcing_chain`; no two keys of D6
  together before the decisions; D1 with `rank27_residual`; D4 or D4c with any key
  of D5.
- **Choices fixed so that two programs agree, alternatives not built**: definition
  choices 5 and 6 (D1 across a forcing chain's paths; "a cell twice"), 7 (four distinct
  candidates), 8 and 9 (the loads read as 1 and as paths plus 2), 12 and 13 (the
  component's size, what the limit bounds there), 18 (loads of ranks 21, 22, 29), 25
  (targets inside a chain), 26 (the other form of `closures_residual`), 28
  (contradiction before verity), 29 (a struck target not in the load), 30 (a root
  walking out and back), 32 (a wrap's strikes on the path only).
- **Hand states** were given to no SAT check (they have no model); the hand figures
  were derived by hand and both programs' self-tests agree with them. The hand state
  `NP` (a naked pair listed as two `xy_chain` entries of 7 links and load 4 under
  `links_walked`) is in no table of results: it was read from the kept lists of the
  comparison of the two programs.
- **Stated gaps in the table**: cells that read "nothing measured" (D2's two unbuilt
  options, D3 without `remote_pair`, D4's cross and two unbuilt shapes, D7's option G
  and parts of E and F). The criterion "no option without its cost, its list, what it
  hides" is met for every option the ticket names but two, both of them the Goal's "a
  lower figure if step 5 finds one that binds": the hidden cell of D7's option E (a
  lower path limit counted by root) rests on a lemma, by inference; and for D7's option
  F (counted for each path held, where 2, 3 and 4 bind) what is hidden at capacity 4
  and 8 was not measured for a path limit of 3 or 4, nor for 2 below 24 links. The
  other "nothing measured" cells are options the reading added.
- **"The definitions were fixed before any figure"**: the hash and the file times are
  fact, the order they show is inference.
- **The errata a decision reads** (fifteen were recorded; the definitions were never
  edited after they were fixed): under the strict D1 keys fewest links is not always
  least load, and no list changes at capacity 4, 8 or 729 (fact on the 54 states,
  inference in general); the undecided grid, and the 24 companion runs without the
  `fits` questions, equal line for line; under `component` a link limit shrinks an
  output and takes no witness away; no no-limit list under a no-repeat key; S06's
  recorded closure has a witness of 9 links over 8 candidates, which S06's record does
  not mention; under `least_load_together` at the full limit the witness shown is a
  stand-in whose load can be above the least.
- **Published work unopened, deciding nothing**: Yinnone 1997, on which rests the
  report that "no candidate twice" has a polynomial search (by inference); Gabow,
  Maheshwari and Osterweil 1976; Van Gelder 2005; Rips 1994 and Braine and O'Brien
  1998, the leads for a human study.
- **No source for**: the least load over distinct candidates; a person measured against
  a chain of any length, so D5's reading fits the memory literature by transfer only
  and no figure for D7 has human evidence; the cost of a walk as one pass (textbook
  facts no record opened, though step 5 built it).
- **Thin**: D8 and D9 have documentation and source only, with no second pass; HoDoKu
  was read from an unofficial clone and in part; SudokuWiki's chain code is on its
  server; several sources are abstracts only.
- **Inferences the records made and no source states**: an endpoint strike of k links
  is a closure of k + 2; a two-root forcing chain is a closure; a remote pair's
  component has no odd cycle.

#### The decided configuration, run as a set

No run of the matrix held the nine answers together, so the decided set was run after
the maintainer answered: `walk`, `distinct`, `component` with adjacent pairs, `as_written`
with every closure, `link_by_link`, `root_no`, `per_literal`, `shortest_each`, five kinds,
rank 27 listing all, `named_cells`, `separate`, `max_forcing_paths` 9, at
`max_chain_links` **16** and at 24 for contrast, at capacity 729, 8, 4 and 3, over all 226
grids by the same method (the carry-over checked on every grid at 16 links). Twelve runs
and one of `R0` alone; every one ended with exit status 0; no budget hit, no unknown
output, no undecided grid. The evidence questions were not asked. The run at capacity 3 is
not the `novice` preset: it is this profile with the capacity alone lowered.

| Capacity | 16 links: solved / stalled | 24 links: solved / stalled | `R0` alone: solved / stalled | `V` | Chain looks at 16 / 24 |
| --- | --- | --- | --- | --- | --- |
| 729 | 205 / 21 | 205 / 21 | 173 / 53 | 54 | 221 / 224 |
| 8 | 205 / 21 | 205 / 21 | 171 / 55 | 55 | 222 / 225 |
| 4 | 191 / 35 | 191 / 35 | 171 / 55 | 55 | 132 / 135 |
| 3 | 191 / 35 | 191 / 35 | 155 / 71 | 71 | 191 / 194 |

- **16 links against 24.** At no capacity does any grid's end or hardest rank differ;
  steps differ on four grids at capacity 729 and 8 and on one at 4 and 3. So what the
  maintainer was told at D7 holds under the decided set. At 16 links the longest single
  chain a player is shown in a step is 15 links, against 23 at 24 links. A lower limit can
  lengthen a run: `norvig_hardest_11` takes 86 steps at 16 links against 82 at 24.
- **Steps at ranks 23 / 24 / 25 / 26 / 27, at 16 links:** capacity 729, 0 / 5 / 35 / 97 /
  62; capacity 8, 0 / 6 / 35 / 97 / 62; capacity 4, 0 / 7 / 23 / 63 / 0; capacity 3, 9 /
  54 / 30 / 63 / 0. Entries at chain looks: 0 / 5 / 251 / 3,431 / 6,956; 0 / 6 / 251 /
  3,434 / 6,986; 0 / 7 / 181 / 2,584 / 2,262; 9 / 248 / 536 / 4,979 / 0.
- **What the load reads.** Every entry of ranks 23 to 26 listed at a chain look of these
  runs reads a load of 3, whatever its links (none there is one strong link alone, which
  would read 2). Rank 27 reads its paths plus 2: 4 to 7 in the runs (2 to 5 paths). The count the
  load reads and the count held against `max_forcing_paths` are equal on every entry; the
  greatest is 5, so 9 binds nowhere.
- **What each capacity hides.** Capacity 8 hides nothing of ranks 23 to 27 and solves the
  same 205 grids as 729. Capacity 4 hides, in these runs, rank 27 with three paths or
  more and nothing of ranks 23 to 26: 2,271 outputs of rank 27 hidden and 2,262 listed at 16 links, every one
  of two paths; rank 27 takes no step at capacity 4, because an entry of a lower rank
  comes first at each look; fourteen grids that solve at 729 stall at 4 and none goes the
  other way. Capacity 3 hides all of rank 27 in the runs and solves the same 191 grids as
  capacity 4.
- **Against the earlier runs.** At capacity 729 the decided set equals the baseline run
  without the evidence questions in end, steps and hardest rank on all 226 grids, at 24
  and at 16 links. Against the baseline at the same capacity it solves 15 more grids at 8
  (205 against 190) and 19 more at 4 (191 against 172); no grid that solves under the
  baseline stalls here.
- **Checked.** SAT: 775,699 entry lines of ranks 18 to 27, 51,220 distinct (state, output)
  pairs, 51,220 sound, 0 unsound, on 10,351 states each with exactly one model; the
  checker's control reported 40 of 40 tampered outputs. Ends: 2,938 grid runs against each
  grid's one solution, 0 bad. The second program, on the 54 states under
  `link_by_link` with `per_literal` at 16 links and each capacity: 1,080 of 1,080 lists
  equal by output, 7,624 entry lines equal with the role too (at 24 links 1,080 of 1,080
  and 8,280). D8 and D9 have no second route; their entries have the SAT check alone.
- **Left open, and not decided by this run** (fact, then what it bears on).
  `remote_pair` lists and takes steps for the first time at a chain look, at capacity 3: 9
  entries on 7 grids, all sound. A forcing chain of one path reads a load of 3 or 2 by
  `min(whole, paths + 2)`, not "paths plus 2": 57 such entries stand on one state (look 28
  of `study_advanced`) and none at any chain look of any run; whether the text carries the
  `min` or says "at least two paths" is for the proposed text to state as an open
  question. The capacity-3 and capacity-4 figures rest on the loads of ranks 21, 22 and
  29 held whole, the rebuild's readings (definition choice 18): whether D5's reading reaches ranks
  21, 22 and 29 was not asked.

#### Each acceptance criterion

- **Answers (a) to (c) of step 2 with lines and dates:** met; the three sub-sections on
  step 2 above.
- **Every source with citation, where and when read, tag and decision; nothing unread
  decides:** met; "Step 3" above. Every row says where it was read: an address, a
  repository with its commit, or a PubMed record number (M9 to M11); a row read from
  S06's copy gives the repository and commit, or the address, that S06's notes record.
  Three papers stayed closed (Gabow, Maheshwari and Osterweil; Yinnone; Van Gelder) and
  are listed as deciding nothing.
- **The rebuild reproduced S06's fifteen rows:** met, on the first run; then all 226
  grids, with one difference of notation explained; "Step 4" above.
- **Definitions fixed before any figure; counts by technique and option; the stalled
  states:** met in part, with two deviations recorded under "Deviations, and why". The
  counts are at chain looks, not at each look (definition choice 19), with a census on
  fifteen grids. And the table by technique and option carries entries and hidden
  outputs for every option at each capacity, but the links and load of the kept witness
  and `implications` by option at capacity 729 only; the rest stayed in scratch. Only
  the engine ran grids; the second program lists states.
- **The two programs agreed, or each difference is listed and explained:** met; 16
  small-limit entry lines agree by output only and one real difference was found and
  fixed in the second program; "the agreement of the two programs" above.
- **Soundness, the ceiling and shares, unbounded length, the implications:** met for
  soundness (0 defects) and the implications (0 fail); met for the ceiling and for
  unbounded length under the baseline keys only, since no no-limit list was made under a
  no-repeat key; "The SAT checks" above.
- **Every pair of limits D7 names was run:** not met for six pairs, by definition choice
  20, and disclosed: `(16, 3)`, `(12, 3)`, `(8, 3)`, `(16, 4)`, `(12, 4)` and `(8, 4)`
  under `per_literal`, the key decided, where step 5 found that 2, 3 and 4 bind. Every
  pair at 9 paths and every pair at 2 was run; "every pair of limits" above. The
  maintainer kept 9 and chose no lower path limit, and the pair decided, (16, 9), was
  run at four capacities; whether the six matter is the maintainer's to judge.
- **The table of decisions is complete:** not met for two cells of options the ticket
  names, both under D7: the hidden cell of option E rests on a lemma, by inference, and
  the hidden cell of option F has no figure at capacity 4 or 8 for a path limit of 3 or
  4, nor for 2 below 24 links. Met for every other option the ticket names; the other
  cells that read "nothing measured" are options the reading added. "The completed
  table of decisions" and "Limits a reader of the decisions should know" above.
- **Each of D1 to D9 answered by the maintainer with a date, or deferred:** met; nine
  answers on 2026-10-03 and none deferred; "Open points settled".
- **The follow-ups cite the decisions by id and name every specification change as a
  change to make first; the notes carry what a rebuild needs:** met; "Handed back",
  parts 1 to 4. One counter is left unpinned and part 4 says so: rank 23's `patterns`
  on its own under the decided keys.
- **`git status --porcelain` before the commit:** run when these notes were handed back,
  before any commit: two lines, this file and `tickets/README.md`. The commit is step
  10's and was not made by the author of these notes.

The Verification, run after the last edit of this file and before any commit, with its
output as printed. It meets what "Verification" expects: the four guarantees; the two
limits in the config and in the three guarantees that read them; 93, which is at least
18; 9; and two lines naming this file and `tickets/README.md`:

```text
$ rg -n 'guarantee (LinkMeaning|RemotePairs|AlternatingChains|BoundedForcingChains)' docs/specs/technique.allium
952:    @guarantee LinkMeaning
973:    @guarantee RemotePairs
980:    @guarantee AlternatingChains
1001:    @guarantee BoundedForcingChains
$ rg -n 'max_chain_links|max_forcing_paths' docs/specs/technique.allium
529:    max_chain_links: Integer = 24
530:    max_forcing_paths: Integer = 9
978:        -- At most config.max_chain_links peer links; no repeating cells.
982:        -- starting and ending strong, with at most config.max_chain_links.
1002:        -- forcing_chain admits up to config.max_forcing_paths static-link
1003:        -- paths, each of at most config.max_chain_links. No nested guesses
$ rg -c '^\| D[1-9] \|' tickets/S11-chain-techniques.md
93
$ rg -c '^- \*\*D[1-9]\.\*\* .*(20[0-9]{2}-[0-9]{2}-[0-9]{2}|deferred)' tickets/S11-chain-techniques.md
9
$ git status --porcelain
 M tickets/README.md
 M tickets/S11-chain-techniques.md
```

#### The adversarial review by Codex, 2026-10-04

The maintainer asked for an adversarial review by Codex once the notes were complete
("Open points settled"), and it was run on 2026-10-04 over this file and the scratch
under `ai_tmp/s11/`. It was read-only: it ran the programs and wrote
nothing, its own report included, so what follows is its report as returned, each
finding then checked again by the author of these notes with programs kept under
`ai_tmp/s11/codex/` (scratch). Line numbers in its report are of this file before the
fixes.

**What it checked and found nothing in.** The polarity of the links; the reachability
of walks; the conversion of an endpoint strike to a closure, on 2,760 cases; the
mirroring of verity proofs; the ceiling of nine paths; the two-colouring of components;
parity over the 54 states; all 452 per-grid rows of part 4, at capacities 729 and 4,
against the stored outputs of the runs; and that ranks 1 to 22 change nowhere a
decision did not mean them to.

**What it did not do.** It did not report on the amendments to T35 (part 2) as such,
and it could not run `just check`.

**Its seven findings.** None changes a figure of a run, a count in a table or any of D1
to D9. All seven were confirmed, five by running and two by reading; each entry gives
the verdict, the evidence and where the fix is.

1. *The tie order of a forcing chain was incomplete in the proposed guarantee (P2).*
   Confirmed by running. On the stalled state of `unbiased_005`, placing 3 at r2c6 has
   29 contradictions of the fewest links, 14: 27 of two paths (load 4) and 2 of three
   (load 5). The text said "fewest links, then a contradiction before a verity" and
   stopped. Fixed in part 1, `BoundedForcingChains`: the order part 4 already gave a
   rebuild (the kind, then the anchor) is now in the guarantee, marked as built, with
   the count of paths said not to be compared; Q5 asks whether fewer paths should come
   first.
2. *"Capacity 4 sees only forcing chains of two paths" is not a rule (P2).* Confirmed by
   running. The load as built is `min(whole, paths + 2)`. Look 28 of `study_advanced`
   holds 57 proofs of one path that read 2 or 3; a constructed state gives three
   one-link paths over four candidates, load 4, listed at capacity 4 and hidden at 3.
   Fixed in part 1 (the load sentences and their note), part 3 (S07's third correction
   and S08's contract paragraph) and "The decided configuration": the formula is given
   as built wherever a forcing chain's load is, and "two paths" is said of the measured
   runs only. Q7 asks whether the text carries the `min` or "paths plus two" outright.
3. *"Nothing to merge at these ranks" was false of rank 23 (P2).* Confirmed by running,
   and found true of rank 22 as well. Two components of `{1,2}` that are not joined each
   strike exactly 1 at r4c6 on a state with a model; the engine finds two witnesses and
   lists one entry. At rank 22 the decided run over all 226 grids at capacity 729 has
   38,067 witnesses for 31,250 entries. Fixed in part 1 (the paragraph on what is
   listed at ranks 22 to 27 and the note under it): the output is listed once,
   `Entry` and `listed` are needed at ranks 22 and 23, ranks 24 to 27 keep one witness
   to an output inside the technique; part 4 says which component is kept as built, and
   Q4 asks.
4. *The load text gave a remote pair "one path", and its witness has none (P2).*
   Confirmed by reading: `proto/chains_rp.py` (scratch) sets the count of paths to 1
   for a component and reads `min(whole, 3)`, while `Proof.paths` (`technique:179`)
   holds inference paths. Fixed in part 1: the load sentences give `remote_pair` 3
   outright and count paths for ranks 24 to 27 only; `RemotePairs` says the `Proof` of a
   component has links and no path, proposed and not run; part 4's load line follows;
   Q4 asks.
5. *The bound given for rank 23 in the Rust table was that of the counter (P3).*
   Confirmed by running: a path of four cells reads 6 to colour and 24 for the
   distances, as the review counted, and 27 common peers. Fixed in part 3: the table
   names `2e` as `patterns`, and a note bounds the whole search.
6. *"Not an ordered breadth-first search" was wrong (P3).* Confirmed by running, with a
   limit to what it shows. A first in, first out search with ascending successors and
   marking at first reach returns the kept path on 418,190 of 418,190 pairs over the 54
   states, and by induction always. The sentence was true only of a search that sorts
   each layer by literal number, which differs on 38,200 of those pairs; and a search
   over literals alone is still wrong for ranks 24 and 25, where the minimum needs the
   capped sets. Fixed in part 3: the two notes under the Rust table and two cells of
   it. The same sentence in the scratch definitions (section 3.1) is left as it was and
   is corrected here.
7. *Q1 said remote pairs list nothing, without the capacities (P3).* Confirmed by
   reading: "The decided configuration" records nine entries on seven grids at
   capacity 3 and none at 729, 8 or 4. Fixed in part 1, Q1.

Three things the fixes state as built, because no decision reaches them, each with its
question: the order among contradictions of equal links (Q5); which of two components
with one output is shown, and what a component's `Proof` holds (Q4); and the `min` in a
forcing chain's load (Q7). "Open points settled" lists them among what the maintainer
was not asked. The five commands of the Verification were run again after the fixes and
print what the block above shows: the same lines of `technique.allium`, 93, 9, and the
two lines of `git status --porcelain`.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `ticket/S11-chain-techniques`, with
  capitals, not the `ticket/s11-chain-techniques` the frontmatter names. The `branch:`
  field is left as written, as S06 and S09 left theirs.
- **S06's scratch still existed, and was used.** The Context says S06's prototype,
  grids and oracle stayed in S06's worktree, and step 4 asks for a rebuild from the
  notes. The rebuild was made blind, from S06's hand-back notes with nothing of S06's
  scratch opened, and reproduced the fifteen rows on its first run; then, by the
  maintainer's answer, it was checked against S06's scratch on all 226 grids under
  both profiles (differences in all: 0, one difference of notation in S06's own files
  explained). Nothing in the rebuilt program changed after the blind phase.
- **S06's grids were copied, not fetched.** The 211 fetched grids were copied byte for
  byte from S06's worktree, which the maintainer authorised; fetching them again was
  not authorised and not done. The raw sources' hashes equal the five S06's ticket
  records; the grids were not rebuilt from the raw sources.
- **A scratch install against the index's rule for spikes.** `tickets/README.md` says
  of a spike that "nothing is installed or changed outside the ticket file". Step 4
  authorises a SAT solver in a throwaway environment under scratch, on asking; the
  maintainer authorised it and `python-sat` 1.9.dev15 (with `six` 1.17.0) was
  installed with `uv` into a virtual environment under the gitignored `ai_tmp/`. pixi,
  `pyproject.toml` and every manifest are untouched.
- **Step 8 was asked in the order the couplings need, not in numeric order**: D1, D2,
  D5, D4, D8, D9, D3, D6, D7, one at a time, each decision apart with
  its evidence and its recommendation. D6 was put as three questions (the root, the
  count of paths, the witness kept). The nine lines under "Open points settled" are in
  numeric order.
- **Options the reading added beyond the ticket's.** Step 2 allows it; the completed
  table carries them marked "added". D1: "no literal twice". D2: each sentence in its
  own unit; no minimum. D3: the switches for adjacent cells and for what the load
  counts; no `remote_pair` at all. D4: `closures_residual`; a continuous loop as its
  own output; a placing closure recorded as strikes. D5: every witness its own
  deduction; least load. D6: three kinds of contradiction against five; rank 27
  listing only what rank 26 does not. D7: no limit on links. D9: one deduction for
  each target. Those not built say "nothing measured".
- **The run matrix changes one decision at a time, not every combination.** Step 5
  asks for each option of D1 to D6 that can be built. Each was run against one
  baseline with the other keys held, and a few pairs besides; no run crosses D4 or D4c
  with D5, two keys of D6, D1 with `rank27_residual`, or a no-repeat key, `no_load` or
  `link_by_link` with a link limit below 24. The full cross was not run because its
  size is the product of the options.
- **Chain ranks are searched only at chain looks.** Step 5 says "at each look". Ranks
  23 to 29 were searched only where ranks 1 to 22 list nothing after hiding
  (definition choice 19), with one census at every look over fifteen grids for the
  baseline at capacity 729. So nothing says what any other key lists at looks where a
  lower rank still lists something.
- **Chain runs are over the grids that need them.** A run with all twenty-nine ranks
  was made over the grids that stall in `R0` or take a step of rank 28 or 29 in it,
  and `R0`'s rows stand for the rest. The carry-over was checked on every grid at
  capacity 729, 8 and 4: outside those grids 0 differ from `R0`.
- **Not every pair of limits was run.** Under `per_root` no path limit from 1 to 9
  binds, so only `(24, 2)` was run as a check. Under `per_literal` 2, 3 and 4 bind:
  a path limit of 2 was run at every link limit at capacity 729 and at 24 links at
  capacity 8 and 4; 3 and 4 at 24 links at capacity 729. Not run: `(16, 3)`,
  `(12, 3)`, `(8, 3)`, `(16, 4)`, `(12, 4)`, `(8, 4)`, nor any path limit other than 9
  and 2 at capacity 8 or 4. The criterion "every pair of limits D7 names was run" is
  not met for those six pairs; the choice not to run them is definition choice 20, made
  by this ticket's agent. It leaves a gap in the criterion on the table too: for D7's
  option F, a lower path limit counted for each path held, what is hidden at capacity 4
  and 8 was not measured for a limit of 3 or 4, nor for 2 below 24 links.
- **The links and load of the kept witness are carried by option at one capacity.**
  Step 5 asks, for each technique and option, the entries, the patterns examined, the
  links of the witness kept and its load. Entries and hidden outputs are in these notes
  for every option at capacity 729, 8 and 4. The implications, and the median and
  greatest links and load of the kept witness, are carried by option at capacity 729
  only; at capacity 8 and 4, for the keys of D5 and D6c and for the pairs of D7 they
  were recorded in the scratch results and are not in these notes.
- **Searches with no argued bound ran under budgets.** Eight budgets were stated
  before any program existed; an output whose search runs out is unknown and treated
  as hidden. In the runs one budget was hit, by one `fits` question at rank 27, at two
  looks of one grid (`unbiased_044`), with 0 outputs unknown; the grid counts as
  undecided though it ends solved, and the 24 runs it marks were made again without
  the `fits` questions, equal line for line. In the static lists every hit is under
  `D5=least_load` or `D6c=least_load_together` at capacity 729, and touches the
  witness shown and its load, never a list.
- **One program behind every run.** Step 5 says "run both over S06's grids". The
  second program lists a state and does not run a grid; the two were compared on 64
  states, not on runs. D8 and D9 are in no comparison.
- **The second program was mended once after the first comparison.** The first run of
  the comparison found one real difference (4 records); it is explained and was fixed
  in the second program, and the comparison made again. Under the small limit 16 entry
  lines of rank 27 agree by output and not by witness; they are listed and explained.
- **The definitions were not edited after they were fixed; fifteen errata stand
  beside them.** Each erratum says what was done where the definitions were silent or
  wrong. One changes a check of step 6: the implication query as defined cannot fail
  on a premise the state's one completion denies, so a local form was asked beside it.
- **The SAT checks cover more and less than step 6 names.** More: the ceiling was
  taken on 225 states, not 53, and ranks 18 to 22 were added to the soundness check
  under D8 and D9. Less: the hand states have no model and were given to no SAT check
  (303 pairs of state and output and 423 implications on them are not counted as
  sound); what a chain of unbounded length reaches was listed under the baseline keys
  only, not under a no-repeat key; S06's soundness check of ranks 1 to 22, 28 and 29
  was not repeated and carries over by inference, the lists being equal entry for
  entry.
- **Sources left unopened.** Of the ticket's leads, Gabow, Maheshwari and Osterweil
  1976 was not opened; nor were Yinnone 1997, Van Gelder 2005, Rips 1994 and Braine
  and O'Brien 1998, which the reading turned up. Each is marked unread and decides
  nothing. Several sources were read as abstracts only, and HoDoKu from an unofficial
  clone and in part.
- **One citation leaves out its author's surname.** Source L11 of step 3 is cited by
  title and arXiv identifier alone. The spelling check of the gate rewrote the surname
  to the word "Liberate" in this file; its allow-list, `_typos.toml`, is not one of the
  two files this ticket may touch, so the surname was taken out and the rewrite undone.
  L11 bears on no decision. A later change may add the name to the allow-list and
  restore it.
- **The decided configuration was run after the decisions.** The ticket has no such
  step. No run of the matrix held the nine answers as a set, so after step 8 the set
  was run over all 226 grids at 16 links, and at 24 for contrast, at capacity 729, 8,
  4 and 3 (twelve runs and one of `R0` alone; no budget hit; the `fits` questions not
  asked). Capacity 3 is the novice preset's capacity with the rest of the profile
  unchanged, not the novice preset. Solved / stalled of 226 at 16 links: 205 / 21 at
  729 and at 8, 191 / 35 at 4 and at 3; at 24 links the same. What the maintainer was
  told at D7, that the same grids solve at 16 as at 24, holds under the decided set:
  no grid's end or hardest rank differs at any capacity, and the steps taken differ on
  four grids at 729 and at 8 and on one at 4 and at 3. `D3l=both_digits` is carried
  there as the baseline's provisional reading; no decision covers it.
- **"Fixed before any figure" is shown by file times.** The definitions' hash and the
  times of the first program files are fact; that the one preceded the other in the
  work is the inference they support.
- **The Goal's table is left as written**, as the record of the starting
  recommendations. The completed table is in "What was verified, and how".

### Handed back

Every part below takes the nine decisions of "Open points settled" as given and cites
them by id. **Every specification change named here is a change to make first**: no
Rust, no fixture and no generator text is written against `technique.allium` as it
stands at `0682b06`. Fact is what a file or a run said; inference is marked. Figures
tagged "run table n" are from the run of the decided configuration (2026-10-03, all 226
grids, sixteen links, part 4). The tag is a label of that run's scratch report, kept
only so that two figures from one table can be told together; it is not a source a
reader needs, since every figure quoted is carried here. The seventeen tables are: 1,
the runs; 2, chain looks for a grid, and grids by hardest rank; 3, steps by rank; 4,
entries listed and outputs hidden by rank; 5, implications a look; 6, links of the kept
witness of the entries listed; 7, links of the kept witness of the steps taken; 8, rank
27 by paths; 9, the 54 sampled states; 10, 16 links against 24; 11, across capacities;
12, against the earlier runs; 13, the carry-over; 14, the table for a rebuild; 15, rank
23 and the paths counted; 16, the run without chains; 17, the SAT check's gather.
Figures tagged "baseline" were made at 24 links under the keys the options were held
against, and are never mixed with the first kind.

#### 1. Proposed specification text

Line numbers are `docs/specs/technique.allium` at `0682b06` unless a file is named.
Each block gives the lines as they stand and the text proposed. A clause marked
**(as built)** states a reading the prototype ran and the maintainer was not asked; each
has an open question in 1.11, and none is decided by this ticket.

**Config, `technique:527-536` (D7; and D1 to D6, D8, D9 for the version). Text change.**

```text
before (527-530, 536)
    -- Finite catalogue proof limits, independent of a player's smaller
    -- reasoning/memory limits. Changing either changes catalogue_version.
    max_chain_links: Integer = 24
    max_forcing_paths: Integer = 9
    catalogue_version: String = "technique-2"

after
    -- Finite catalogue proof limits, the same for every player and apart
    -- from any player's own reasoning and memory limits. Changing either
    -- changes catalogue_version.
    -- max_chain_links counts links of LinkMeaning walked on ONE path,
    -- strong and weak alike, a link walked twice counted twice. For
    -- x_chain, xy_chain and alternating_inference_chain the path is the
    -- chain, whose links are odd in number, so fifteen is the most. For
    -- forcing_chain it is each path apart, of either parity. For
    -- remote_pair it is the peer links between the two cells a target sees.
    -- max_forcing_paths counts the paths one forcing_chain proof holds.
    -- On a grid of side nine a proof holds at most nine, so it never binds.
    max_chain_links: Integer = 16
    max_forcing_paths: Integer = 9
    catalogue_version: String = "technique-3"
```

Why the version is raised: `technique:532-535` says it names "which techniques are
specified, their exact variants and the two limits above", and that it changes when any
of them does. D7 changes a limit; D1, D2, D3, D5, D6, D8 and D9 each change an exact
variant (what is listed, or the load an entry carries). One raise covers them all, and the comment at
532-535 needs no change. "smaller" at 527 goes because it no longer holds of every
player once the limit is 16: a player's `reasoning_depth` can equal the limit if its
range becomes `0..16`, and exceed it if the range stays `0..24`. Which of the two it is
is asked in the specification change (1.12 (a)).
What binds, to record beside the figures (fact): no entry of any run held more than 5
paths (run table 15); at 16 links the longest single chain a step shows is 15 links, against 23
at 24 links (run table 7).

**Vocabulary, `technique:78-84` (D1, D4). Text change: two words added.**

```text
before (80)
--   path, literals each following from the last by one link. Base and

after
--   path, literals each following from the last by one link; walk, a
--   path that may pass through a candidate it has already met; closure,
--   a path from a literal to its negation. Base and
```

**The load sentences, `technique:597-600` and `912-913` (D5). Text change.**

```text
before (597-600)
-- These loads are provisional costs, not measurements of human working
-- memory. A variable load is the number of distinct candidate propositions
-- in the proof (at least one). human-solving.allium uses explicit retained
-- facts and its own learned chunks instead of this shortcut.

after
-- These loads are provisional costs, not measurements of human working
-- memory. A variable load is the number of distinct candidate propositions
-- in the proof (at least one).
-- A chain is held link by link and not whole: for x_chain, xy_chain,
-- alternating_inference_chain and forcing_chain the load is the lesser
-- of that number and the proof's paths plus two, the two candidates of
-- the link in hand and one thing remembered for each path. A path
-- counted here is an inference path of the proof with at least one
-- link. The first three have one path, so a chain of any length loads
-- three and one strong link alone two; a forcing chain loads its paths
-- plus two, or the number of its distinct candidate propositions where
-- that is less. remote_pair loads three: its witness is a component
-- and no path, read as a chain is. human-solving.allium uses explicit
-- retained facts and its own learned chunks instead of this shortcut.

before (912-913)
        -- Variable load counts its distinct candidate propositions; colour
        -- and BUG proofs include all their supporting candidate propositions.

after
        -- Variable load counts its distinct candidate propositions; colour
        -- and BUG proofs include all their supporting candidate propositions.
        -- Ranks 24 to 27 read the lesser of that count and paths plus two,
        -- and rank 23 reads three, as the note above the defaults says.
```

This is the formula that was run for ranks 24 to 27, `min(whole, paths + 2)`, and not a
bare "3" or a bare "paths plus 2": the `min` matters for an
`alternating_inference_chain` of one strong link (load 2), for a forcing chain of one
path (load 2 or 3), and for a forcing chain of three paths or more that holds few
candidates. A constructed state shows the last (fact; the Codex review's finding 2, run
again here): with 1 at r1c1, r2c1 and r3c1 alone in box 1 and no strong link anywhere, 1
asserted at r4c1 denies all three by one link each, a contradiction of three paths over
four candidates, and it reads a load of 4, not 5, and is listed at capacity 4. The
maintainer was told "paths plus 2" at D5; the formula with its `min` was in the option
as put (definition choice 9), and Q7 asks which the text should carry. Fact (run tables
6 and 8): every entry of ranks 23 to 26 in every run read 3; rank 27 read 4 to 7, on 2
to 5 paths, and on every entry of the runs the load was the paths plus 2.

Rank 23 is given its figure outright (the review's finding 4). Its witness is a
component, a set of cells, and no inference path runs through it, so "paths plus two"
has nothing to count there. **(As built)** the engine read `min(whole, 3)`, the
definition's figure for a chain held link by link, by setting the count of paths to 1
for a component (`proto/chains_rp.py`, scratch); the whole of a component of four cells
or more is at least 8, so the figure is 3 on every component. Under D5, "a chain reads
load 3", this is the reading decided; that a component is read as a chain is, and not
by its size, is the part no one was asked apart from D5, and Q4 names it. The paths the
formula counts for ranks 24 to 27 are `Proof.paths`, the inference paths
(`technique:179`): one for a chain or a closure, and for a forcing chain one to each
incompatible literal or one from each root. For a `remote_pair` the prototype held the
cells and the two digits and no `Proof`; the text proposed under `RemotePairs` has
`Proof.paths` empty and `Proof.links` the links of the component, and Q4 asks it.

The loads of ranks 21, 22 and 29 are untouched: they still count the whole proof (1.11,
Q1). The general sentence on a variable load is left word for word. What one proposition is governs ranks 21, 22
and 29 as well and no decision covers it: as built a candidate asserted and denied is
one, which under the lesser-of formula changes nothing at ranks 23 to 26, and Q1 asks
it.

**What is listed "each once" at ranks 22 to 27, new text after `technique:337` (D3, D5,
D6, D9). Text change: no such sentence exists past rank 12 today.**

```text
after (a new paragraph of the comment on Grid.deductions)
    -- Past the first twelve the techniques are pinned in Catalogue. Of
    -- ranks 22 to 27, licensed holds: for simple_colouring one deduction
    -- for each trap and one for each wrap of each component; for
    -- remote_pair one for each component that strikes; for x_chain,
    -- xy_chain, alternating_inference_chain and forcing_chain one for
    -- each output the technique proves within the limits, each once,
    -- carrying the witness kept. Other proofs of that output are not
    -- further deductions: a walk can be lengthened without end, so they
    -- are not listed. The witness kept is a function of the grid and the
    -- limits and of no profile. Two components can strike the same
    -- candidates; they are two deductions with one output, and the list
    -- holds that output once, as for every rank.
```

The last sentence is the Codex review's finding 3, and it corrects an inference these
notes made: that with one deduction to a technique and output S06's drafted `Entry` and
`listed` (part 2) had "nothing to merge at these ranks". That is true of ranks 24 to 27
alone, and only because the technique itself keeps one witness for each output: two
endpoint pairs, or a closure and an endpoint strike, or two cases of a forcing chain,
often prove one output, and the fewest-links witness is kept before anything is listed.
It is false of ranks 22 and 23, where a deduction is a component (D3, D9) and two
components can have one output:

- Rank 23 (fact, run for this review; `ai_tmp/s11/codex/f3_merge.py`, scratch). A state
  with a model, built from a completed grid, holds two components of `{1,2}` that are
  not joined, r1c4, r1c5, r6c4, r6c5 and r4c1, r5c2, r8c2, r8c6; each strikes exactly 1
  at r4c6, and a SAT check finds the strike sound. The engine finds two witnesses and
  lists one entry. No run met the case: the run of all 29 ranks at capacity 3 has nine
  witnesses of rank 23 and nine entries, and the runs at 729, 8 and 4 have none.
- Rank 22 (fact, the decided run of all 29 ranks over the 226 grids at capacity 729 and
  16 links). Witnesses the profile sees number 38,067 and entries 31,250 over 14,345
  looks; at 4,428 looks at least two witnesses of rank 22 share an output. So merging
  by output is common at rank 22 and was in every run.

**(As built)**, at ranks 22 and 23 as at every rank below them: the list holds one entry
for each technique and output, and of several witnesses the least is kept, hiding done
first. At rank 22 the order of witnesses is S06's (cells, then digits, then the role);
at rank 23 it is the role alone (definition choice 15), which for a component is the
fewer cells and then the lesser list of cells (`ai_tmp/s11/proto/core.py` and
`chains_rp.py`, scratch). The maintainer decided the deduction (D3: the component) and
was not asked which of two components with one output the list shows, so the paragraph
above says only that the output is listed once and leaves the choice of witness to
`Entry` and `listed`, where S06's draft puts it; Q4 names it. So S06's `Entry` and
`listed` are needed at ranks 22 and 23 and must stay in the specification change.
`reach.allium`'s `LimitsOnlyHide` stands for ranks 24 to 27, since no profile chooses a
witness there; at ranks 22 and 23, as at the ranks below, a profile hides witnesses and
the least of those left is kept, which is S06's rule and nothing new.

The paragraph speaks for ranks 22 to 27 alone, the ranks a decision here touches. Ranks
13 to 21, 28 and 29 still have no "one deduction for each ..., each once" sentence. It
is not supplied here: it falls to S06's drafted `Entry` and `listed`, or stays unsaid,
and part 2 names it as an open point of the specification change.

**"Other cells", `technique:929, 934, 941, 944` (D8). Text change, in the form of `:984`.**

```text
before
929     -- and {b,c}. Strike c from other cells seeing both wings.
934     -- Strike b in other cells seeing P and Q. Swapping a and b is a
941     -- Strike d from other cells seeing both unaligned endpoints.
944     -- Strike d from other cells seeing both remaining endpoints.

after
929     -- and {b,c}. Strike c from cells seeing both wings, the wings excepted.
934     -- Strike b in cells seeing P and Q, P and Q excepted. Swapping a and b is a
941     -- Strike d from cells seeing both unaligned endpoints, those two
        -- excepted: an aligned endpoint that sees both is struck too.
944     -- Strike d from cells seeing both remaining endpoints, those two
        -- excepted: a joined endpoint that sees both is struck too.
```

Fact: the lists of `w_wing`, `skyscraper` and `two_string_kite` change (2,767 / 3,422 /
4,131 entries against 2,638 / 1,687 / 4,075 in runs of ranks 1 to 22, 28 and 29 at
capacity 729, baseline); `xy_wing` and `xyz_wing` list the same under either reading; no
grid's end and no count of steps changed; every entry checked was sound.

**`SimpleColouring`, `technique:965-971` (D9). Text change: one sentence added after 969.**

```text
after (inserted after "An outside node seeing both labels loses d (colour trap).")
        -- Wrap and trap are separate deductions: one wrap for each label a
        -- same-label peer pair makes false, striking every node of that
        -- label, and one trap for each component, striking every outside
        -- node that sees both labels. Each has the whole component as its
        -- witness and counts every node of it in its load.
```

**`RemotePairs`, `technique:973-978` (D3, D7). Text change: the guarantee is rewritten.**

```text
before
        -- At least four distinct cells each have exactly {a,b}. Consecutive
        -- cells are peers, so their assignments alternate. Two cells an odd
        -- number of links apart have opposite assignments. Strike both a
        -- and b, where present, from outside cells seeing such a pair.
        -- At most config.max_chain_links peer links; no repeating cells.

after
        -- For one pair of digits {a,b}, join the open cells that hold
        -- exactly {a,b} wherever two are peers. A connected component of
        -- at least four such cells that can be two-coloured, each peer link
        -- joining opposite labels, is ONE deduction, as a component is in
        -- SimpleColouring: its cells take a and b alternately. A component
        -- that cannot be two-coloured is nothing. Strike both a and b,
        -- where present, from every open cell outside the component that
        -- sees two of its cells of opposite labels; two cells that are
        -- themselves peers, one link apart, count. The two cells seen are
        -- joined within the component by at most config.max_chain_links
        -- peer links. The witness is the component, a set of cells: no
        -- chain through it is chosen and no cell repeats. Its Proof has no
        -- paths; Proof.links holds the strong link of each of its cells
        -- and the weak links on a and on b of each two joined cells. Its
        -- load is three.
```

"At least four cells" for the component, "the limit bounds the distance between the two
cells seen" and "a component that cannot be two-coloured is nothing" are **(as built)**
(1.11, Q4). So is "its Proof has no paths", with what `Proof.links` holds: the prototype
kept the cells and the pair of digits and built no `Proof`, so those two sentences are
proposed and were not run, and Q4 asks them. "Its load is three" is the figure the
engine read on every component (the load sentences above). Whether a pair cell counts
as one proposition or two, which the definitions left provisional, no longer matters:
the load read is 3 either way. Fact: no remote pair stood at any look
where ranks 1 to 22 list nothing at capacity 729, 8 or 4; nine stood at capacity 3 (run table 15).

**`AlternatingChains`, `technique:980-999` (D1, D2, D4, D5). Text change.**

```text
before (981-988, 992-993)
        -- For endpoint strikes, candidate nodes alternate strong and weak links,
        -- starting and ending strong, with at most config.max_chain_links.
        -- It proves that at least one endpoint is true. Strike any candidate
        -- weakly linked to BOTH endpoints, excluding the endpoints.
        -- X-Chain restricts all nodes to one digit, with at least three links.
        -- XY-Chain uses only bivalue cells: strong links within cells,
        -- weak links between peers on a common digit, equal endpoint digits,
        -- at least three distinct cells.
        -- Also permit a path from a literal to its negation: strike its
        -- candidate if the start is positive, place it if negative.

after
        -- A chain of these three techniques is a walk: it may pass through
        -- a candidate it has already met.
        -- For endpoint strikes, candidate nodes alternate strong and weak links,
        -- starting and ending strong, with at most config.max_chain_links
        -- links walked, between two distinct endpoints.
        -- It proves that at least one endpoint is true. Strike every candidate
        -- weakly linked to BOTH endpoints, excluding the endpoints and nothing
        -- else: a node of the chain so linked is struck too. One deduction
        -- holds every such strike.
        -- X-Chain restricts all nodes to one digit and holds at least four
        -- distinct candidates.
        -- XY-Chain uses only bivalue cells: strong links within cells,
        -- weak links between peers on a common digit, equal endpoint digits,
        -- at least three distinct cells.
        -- Also permit a closure, a path from a literal to its negation: strike
        -- its candidate if the start is positive, place it if negative. One
        -- deduction holds that one candidate. Every closure within the limit
        -- is listed, whether or not an endpoint strike holds its candidate.

after (added at the end of the guarantee, after 999)
        -- Of the chains that prove one output under one technique the
        -- witness kept has the fewest links. Among those of fewest links it
        -- is the least when the chains are written as lists of literals and
        -- compared element by element, a literal ordered by cell in
        -- row-major order, then digit, asserted before denied. A chain and
        -- its mirror, reversed with every literal negated, are one witness,
        -- written from the endpoint of the lesser candidate. An
        -- alternating_inference_chain so kept holds no literal twice,
        -- though it may hold one candidate asserted and denied. An x_chain
        -- or xy_chain kept to meet its minimum may hold a literal twice.
```

"Four" for `x_chain` is definition choice 7. It was stated in the D2 option as put ("at
least four distinct candidates") and accepted with it; the alternative, three distinct
candidates, was never weighed, so Q3 asks it. "Excluding the endpoints and nothing
else: a node of the chain so linked is struck too" is **(as built)** (definition choice
25, Q9): `technique:984` as it stands excludes the endpoints only and D4 kept it as
written, but the maintainer was not asked whether a chain may strike a node of its own.
The order among chains of equal links is **(as built)** (Q5). That a fewest-links
witness holds no literal twice is a lemma for a technique with no minimum (inference,
proved in the definitions): were a literal held twice, the stretch between could be cut
out. It does not hold where a minimum must be met: the walk of the fixture below
(not-B, A, not-B, C, not-D, C) holds two literals twice, and on the 54 states
`no_literal_twice` lists 7 entries of rank 24 against 8. Lines
989-991 and 994-999 stand. `alternating_inference_chain` has no minimum, in the text
today and as built: Q6.

**`BoundedForcingChains`, `technique:1001-1016` (D1, D5, D6, D7). Text change.**

```text
before (1002-1004)
        -- forcing_chain admits up to config.max_forcing_paths static-link
        -- paths, each of at most config.max_chain_links. No nested guesses
        -- and no path step that needs results of earlier nonadjacent steps.

after
        -- forcing_chain admits up to config.max_forcing_paths static-link
        -- paths, each a walk of at least one and at most
        -- config.max_chain_links links. A path is counted for each path the
        -- proof holds: in a contradiction one to each incompatible literal,
        -- in a verity one from each root. No nested guesses
        -- and no path step that needs results of earlier nonadjacent steps.

after (inserted after 1012, "Every root must reach the same literal; assert that literal.")
        -- A root is not its own target: no incompatible literal is the root
        -- itself and a verity's conclusion is none of its roots, whether by
        -- a path of no links or by a walk that comes back.
        -- Of the proofs of one conclusion the witness kept has the fewest
        -- links in all, summed over its paths, and each path so kept holds
        -- no literal twice. On a tie a contradiction is
        -- kept before a verity. Every verity has a contradiction with the
        -- same conclusion and the same paths mirrored, so the witness kept
        -- is always a contradiction; a verity stays a proof the catalogue
        -- admits and a fixture checks.
        -- Among contradictions of equal links the kinds are taken in the
        -- order written above: p and not-p; two digits in one cell; equal
        -- digits in peer cells; a cell's candidates all false; a unit
        -- digit's places all false. Within a kind the least is kept: by
        -- the candidate p; by the lesser and then the greater of the two
        -- asserted candidates; by the cell; by the unit and then the
        -- digit. Candidates are ordered by cell, row by row, then digit;
        -- units are rows, then columns, then boxes. That fixes the proof:
        -- the root is the conclusion's negation, and each path the least
        -- of fewest links from it, as AlternatingChains orders them. The
        -- count of paths is not compared: of two proofs of equal links
        -- the one of more paths may be kept.
        -- forcing_chain lists every conclusion it proves, also where
        -- alternating_inference_chain lists the same placement or strike
        -- by a closure: the two names remain separate deductions, as
        -- ProofsMatchTheirOutputs says.
```

Lines 1005-1011 (the five kinds of contradiction, the three of verity) and 1013-1016
stand: D6 keeps five kinds and has rank 27 list all it proves. "Or by a walk that comes
back" is **(as built)**: the maintainer was asked about the path of no links
only (Q7). "Fewest links in all" against "the shortest longest path" is **(as built)**
(Q5). That each path of the kept witness holds no literal twice is the lemma of the
guarantee above, a path of rank 27 having no minimum (inference). "A contradiction
before a verity" was stated in the D6 question and accepted. The order among
contradictions of equal links is **(as built)** and was never put to the maintainer; it
is the order part 4 gives a rebuild (the total, the kind, the anchor), written here in
the guarantee's words because the text above it left two implementers free to differ
(the Codex review's finding 1). Fact, run again for the review
(`ai_tmp/s11/codex/f1_ties.py`, scratch): on the stalled state of `unbiased_005`,
placing 3 at r2c6 has 29 contradictions of 14 links, the fewest, 27 of two paths and 2
of three; the engine keeps two digits asserted in r1c6, of two paths and load 4, at
capacity 729 and at 4. Over the 54 states, 917 of 1,280 outputs have more than one
contradiction of fewest links; on 22 of them the tied proofs differ in their count of
paths, and on 5 the proof kept holds more paths than another tied with it and reads the
greater load: 7 against 5 once, and 4 against 3 four times, all four on look 28 of
`study_advanced`, where a profile of capacity 3 is not shown the output at rank 27 and
a tied proof of one path would fit. None of the five changes what capacity 4 or 8
lists. An output and a kind and anchor name one proof, so the order above is total (0
of 1,280 outputs had two cases with one kind and anchor). Whether fewer paths should come before the
kind on a tie is Q5's. The duality
(every verity has a mirror contradiction) is a lemma of the definitions (inference, by
hand): the conclusion's negation is the contradiction's root and each path is reversed
and negated.

**`ProofsMatchTheirOutputs`, `technique:914-916`. By extension only.** "Different valid
technique names remain separate deductions" already says what D4 (closures listed beside
endpoint strikes) and D6 (rank 27 lists all it proves) keep; no word changes. "Duplicate
witnesses from reversed paths ... have one canonical representation" is given its
content by the two guarantees above.

**`AdvancedExamples`, `technique:1091-1103` (D1, D3, D4, D6, D8, D9). Text change.**

```text
before (1094-1095, 1102-1103)
        -- Remote Pair: {1,2} at r1c1,r1c4,r4c4,r4c2; strike 1,2 at
        -- r2c2. Add 3 to a chain cell and reject that chain.
        -- Forcing chain: complete paths for p and not-p both reach q;
        -- assert q. Omitting either path or exceeding the bound rejects.

after (1094-1095)
        -- Remote Pair: {1,2} at r1c1,r1c4,r4c4,r4c2,r7c2 is one component
        -- and one deduction: strike 1,2 at r2c2, which sees r1c1 and r4c2,
        -- and 1 at r7c4, which holds 1 and not 2 and sees the neighbours
        -- r1c4 and r4c4. Add 3 to r1c4: three pair cells are left joined
        -- and nothing is licensed.

after (added after 1101, the AIC line)
        -- A walk may repeat: d conjugate at A=B, B=C and C=D with B, D
        -- peers. B=C is an X-Chain by not-B, A, not-B, C, not-D, C: five
        -- links over four candidates; strike D. The pair A=B alone, walked
        -- there, back and there, holds two candidates and is no X-Chain.
        -- In the same state the chain A=B-C=D strikes B, a node of its own.
        -- Closure: r1c1,r1c2={1,2}; the path 1 r1c3, not-1 r1c1, 2 r1c1,
        -- not-2 r1c2, 1 r1c2, not-1 r1c3 has five links and strikes 1 at
        -- r1c3, beside the endpoint strike that holds every such target.
        -- A path that stops short of its own negation rejects.
        -- Skyscraper strikes its own cell: d only c4,c9 in r1 and c4,c6
        -- in r2; r1c4 sees both unaligned endpoints and loses d.

after (1102-1103)
        -- Forcing chain: complete paths for p and not-p both reach q;
        -- assert q. Omitting either path or exceeding the bound rejects.
        -- That verity is admitted and never the witness kept. Contradiction:
        -- r1c1,r1c2={1,2}; from 1 at r1c3 asserted, two links reach 2 at
        -- r1c1 and two reach 2 at r1c2, equal digits in peer cells; strike
        -- 1 at r1c3 on two paths, load four. A proof whose incompatible
        -- literal is its own root rejects.
```

Two of these lines pin readings marked **(as built)** and change if their question is
answered otherwise: the remote-pair near miss rests on "at least four cells" (Q4), and
"the chain A=B-C=D strikes B, a node of its own" on definition choice 25 (Q9). The
skyscraper line pins D8, which was decided.
The simple-colouring line (1091-1093) gains "wrap and trap are listed apart" when D9's
sentence lands. Every shape above is taken from the spike's hand states (part 3); the
closure example is a striking one because the hand state with a placing closure has no
solution, which `ElementaryExamples` forbids of a fixture (1043-1045): a placing closure
fixture comes from a true grid (part 3).

**Resolved and open questions, `technique:1131-1166`. Text change.**

A new entry under "Resolved Questions" (the opening sentence at 1131-1132, which today
says nine questions were asked and the catalogue answers eight, is recounted):

```text
--   A long chain: held link by link. A chain of ranks 23 to 26 loads
--     at most three whatever its length and a forcing chain at most its
--     paths plus two, so no capacity of three or more hides a single
--     chain, and a small capacity hides a forcing chain of many paths.
--     Its length is bounded by max_chain_links alone.
```

1.11. The open question at `technique:1162`, before and after, and the new ones:

```text
before (1162)
open question "Loads and ladder order past subsets are provisional. A variable load counts the candidate propositions of the proof, which puts a long chain beyond every capacity Cowan suggests and so hides it from an expert in reach.allium and lapse.allium. Is that right, should a chain be held link by link, and is rank 13 to 29 the order a careful solver tries them in?"

after (Q1)
open question "Loads and ladder order past subsets are provisional. A chain of ranks 23 to 27 is held link by link, and empty_rectangle, simple_colouring and bug_plus_one still count every candidate proposition of the proof, which can put a large component beyond every capacity Cowan suggests. Should those three be read as a chain is, and is a candidate asserted and denied one proposition, as built, or two? And is rank 13 to 29 the order a careful solver tries them in: over 226 grids remote_pair lists nothing where the ranks below it have stopped at a capacity of 4 or more and nine times on seven grids at a capacity of 3, a kept xy_chain is longer than a kept alternating_inference_chain, and forcing_chain strikes whatever the four below it strike."

new (Q2)
open question "A colouring's wrap and trap are separate deductions on one component. Which is listed first where both stand: the order by output, which the list has for every rank, or the wrap, which makes a whole label false?"

new (Q3)
open question "x_chain needs four distinct candidates and xy_chain three distinct cells. A walk still passes a conjugate pair as an x_chain by a detour of five links wherever its neighbours allow one. Is that an x_chain, or should the minimum count distinct links, or the pair be refused? And is four the right minimum, or three distinct candidates, which would list more?"

new (Q4)
open question "remote_pair is one deduction for a two-coloured component of at least four pair cells, and max_chain_links bounds the peer links between the two cells a target sees. Should a component of two or three cells count, must it hold a chain of four cells, which a star does not, should the limit bound the component instead, and is a component that cannot be two-coloured rightly nothing? Its load is three whatever its size, as a chain's is, and its Proof holds links and no path: is each right? And where two components strike the same candidates, which is shown? As built, the one of fewer cells."

new (Q5)
open question "The witness kept is the one of fewest links; for a forcing chain, fewest in all over its paths and not the shortest longest path. Among proofs of equal links the least in a fixed order of literals is kept, and for a forcing chain the kind of contradiction and its place are compared first and the count of paths never. Is that the order a player should be shown, should the proof of fewer paths be kept on a tie since it loads less, and is the order the catalogue's to fix or a hint's?"

new (Q6)
open question "alternating_inference_chain has no least length: one strong link is a chain and strikes what pointing or claiming strikes, and three links through two bivalue cells strike what a naked pair strikes. Should it have a minimum, as x_chain and xy_chain do?"

new (Q7)
open question "A forcing chain's root is never its own target, by a path of no links or by a walk that comes back. Is the second right? And a forcing chain of one path, which arises where a cell has one candidate or a digit one place, loads two or three: should the text ask for two paths at least? A forcing chain's load is the lesser of its distinct candidates and its paths plus two, so three one-link paths over four candidates load four and not five: should it be its paths plus two outright?"

new (Q8)
open question "A struck candidate counts toward a load only where it is a node of the proof. Should every candidate a deduction strikes count, so that a chain with many targets is heavier than one with few?"

new (Q9)
open question "An endpoint strike takes every candidate weakly linked to both endpoints, a node of its own chain included. Should a chain strike only candidates outside it?"
```

What 1162 had asked and is now answered (D5): "Is that right" (no, for ranks 23 to 27)
and "should a chain be held link by link" (yes). What stays, never decided here, and
where each is asked: whether that reading reaches ranks 21, 22 and 29, what one
proposition is, and the ladder's order (Q1); which of trap and wrap is first (Q2);
definition choice 7's alternative, and the detour (Q3); definition choices 12 and 13
(Q4), with a component's load and `Proof` and which of two components with one output
is shown; 16, and the order among proofs of equal links, the count of paths on a tie
included (Q5); rank 26's missing minimum (Q6); 30, a forcing chain of one path, and the
`min` of definition choice 9 against "paths plus 2" (Q7); 29 (Q8); 25 (Q9). Definition choice 18,
the loads of ranks 21, 22 and 29, is Q1's. The choices that "Open points settled" names
and that get no question are moot under the answers: 5 and 6 define the no-repeat keys,
which D1's walk does not use; 26 defines `closures_residual`, which D4 did not take; 32
defines `D9=per_target`, which D9 did not take; and the switch `D3l` decides nothing
once D5 reads a remote pair's load as 3.
Two more readings ran as built and change no list on a kept grid, so they are named for
the specification change and given no question: a placed cell contributes no node to a
chain, and two candidates joined through two units are one link. Calibration gets no new
question. The standing one is `docs/specs/human-solving.allium` 1272 ("Every numerical
default here is a model assumption and none is calibrated"), which no decision here
answers or changes, and Q1 keeps the loads "provisional"; part 3 has what was measured.

1.12. **The other models and pages, each a change to make with the one above.**

| Where | Lines at `0682b06` | Decision | Kind | Proposed |
| --- | --- | --- | --- | --- |
| `effort.allium` | 100-108 | D5 | text (comment); formula by extension | below |
| `effort.allium` | 314 | D5 | text | below |
| `effort.allium` | 29-30, 69-71, 91-98, 110-112, 189-204, 266-270, 312 | D5, D7 | by extension only | "a chain past its limits" names no figure; a price reads the new load |
| `reach.allium` | 7-9, 23-24, 100-113, 238, 296-299, 324-327, 359-372, 383-390, 405-407; questions at 423 to 429 | D1 to D9 | by extension only | "cannot hold", "small enough to hold" and "of a size they can hold" still read `load <= capacity`; what a chain's load is has changed, in `technique.allium`. `LimitsOnlyHide` (369-372) stands: no profile chooses a witness |
| `lapse.allium` | 49-52, 128-137, 154-159, 195-200, 288, 414-417, 433-454; question at 531 | D1 to D9 | by extension only | no word changes. More chains seen means a wrong guess is found out later (154-159), by extension |
| `human-solving.allium` | 108 | D7 | text | below |
| `human-solving.allium` | 787-791 | D5, D7 | text: one sentence | below |
| `human-solving.allium` | 54-55, 93, 195, 208, 474-475, 567-568, 772-775, 804-807, 839-852, 911-925, 944-970, 1094-1097, 1135-1148 | D1 to D9 | by extension only | 475 reads `technique/config.catalogue_version` by reference; 567-568 and 804-807 enforce "every declared range", so 17 to 24 are then refused; `standard_ability`'s 8 (485) stays valid |
| `docs/explanation/human-solving.md` | 96, 130, 247, 248, 257-258 | D5, D6, D7 | text | below |
| `docs/explanation/human-solving.md` | 146-149, 283-285, 288-292 | D3, D5, D9 | by extension only | "Simple colouring, with colour wrap and colour trap" and "Remote Pair" under Chains still hold |
| `docs/explanation/puzzle-design.md` | 918, 920, 924, 925, 949 | D3, D5, D6, D7 | text | below |
| `docs/explanation/puzzle-design.md` | 905, 929, 950 | D5 | by extension only | 950's Expert row is now true of the `expert` preset for single chains (part 3) |

```text
effort.allium, before (103-105)
    -- before the deduction is found. A deduction that fills the player's
    -- capacity costs its base figure, a smaller one a share of it, and one
    -- past capacity more than it.

after
    -- before the deduction is found. A deduction that fills the player's
    -- capacity costs its base figure, a smaller one a share of it, and one
    -- past capacity more than it. x.load is technique.allium's: a chain
    -- is held link by link and loads at most three whatever its length,
    -- a forcing chain at most its paths plus two, so nothing here prices
    -- a chain's length.

effort.allium, before (314)
open question "Pricing a variable load. For colouring, chains, forcing chains and BUG+1 a deduction's load is the count of candidate propositions in its proof, which may be many times any capacity; base_figure * load / capacity was written for loads of one to four. Is a long chain priced by its length, by what is held at once, or by its base figure alone?"

after
open question "Pricing a variable load. A chain is priced by what is held at once: technique.allium gives it a load of at most three, and a forcing chain at most its paths plus two, so base_figure * load / capacity reads small figures for them. For empty_rectangle, simple_colouring and BUG+1 the load is still the count of candidate propositions in the proof, which may be many times any capacity. Are those three priced the same way, and should a chain's length be priced at all, in its base figure or in search_effort?"

human-solving.allium, before (108)
    reasoning_depth: Integer      -- 0..24 implications per reasoning episode

after (a proposal, pending the question of (a) below; the other answer leaves 108 as it is)
    reasoning_depth: Integer      -- 0..16 implications per reasoning episode

human-solving.allium, after (added to 787-791, before "It loses what the Profile cannot say")
        -- reasoning_depth and chunk_facts are not projected: the coarse
        -- models bound a chain's length by technique.allium's
        -- max_chain_links alone, the same for every player.

human-solving.md, before
96   ... | A universal four-item limit, a mapping from one cell to one chunk, or capacities for named player levels. |
130  | `reasoning_depth` | 8 | 0–24 chained implications per episode, ...
247  | Catalogue chain limit | 24 | Links per catalogue proof path; distinct from the profile's smaller reasoning bound. |
248  | Catalogue forcing-path limit | 9 | Paths in a bounded forcing proof. |
257  The initial version identifiers are `human-solving-1`, `technique-2` — which

after
96   ... | A universal four-item limit, a mapping from one cell to one chunk, capacities for named player levels, or that a chain is held link by link: that is the catalogue's reading, measured on no person. |
130  | `reasoning_depth` | 8 | 0–16 chained implications per episode, ...   (pending (a) below)
247  | Catalogue chain limit | 16 | Links walked on one catalogue proof path, a link walked twice counted twice; the same for every player and apart from the profile's reasoning bound. |
248  | Catalogue forcing-path limit | 9 | Paths one bounded forcing proof holds, one to each incompatible literal or from each root; at most nine on a 9 by 9 grid, so it never binds. |
257  The version identifiers are `human-solving-1`, `technique-3` — which

puzzle-design.md
918  "(`technique.allium:597-600` load counts propositions)" gains: "for a chain, what is held at once"
920  "compression is proof load at `technique.allium:597`" gains the same clause
924  the three citations (`543-668`, `50-52`, `529-530`) are refreshed to the new lines
925  before: `forcing_chain` bounded by `max_forcing_paths` and `max_chain_links` (`technique.allium:1001-1003`)
     after:  `forcing_chain` bounded by `max_chain_links`, sixteen links a path; `max_forcing_paths` is nine and never binds on a 9 by 9 grid (citation refreshed)
949  before: `fish`, `wings`, `colouring`, the single-digit `chains` (rank 13 to 23)
     after:  `fish`, `wings`, the single-digit `chains`, `colouring` and `remote_pair` (rank 13 to 23)
```

Two things in that table are proposals within a decision and not the decision itself.
(a) `0..16` at `human-solving.allium:108` and at `human-solving.md` 130 is these notes'
proposal and no part of D7's answer. When D7 was put the maintainer was told that the
range `0..24` "must then change", and chose 16. That overstated it. What cannot stand
together is a link limit below 24, the range `0..24` and the word "smaller" at
`technique:527-528`; there are two ways to mend it: narrow the range to `0..16`, or
leave `0..24` and rely on the reworded 527-528, under which a player may chain further
than the catalogue names. The specification change puts that choice to the maintainer
(part 2, step 3), and until it is answered the two `0..16` lines above are conditional.
(b) Line 949's correction is
a fact about the row (a remote pair runs on two digits, `technique:974`), independent of
any decision. Not read for this spike and to be checked by the change:
`generation.allium`, `board.allium` and `solver.allium`, for any sentence that reads a
limit or a chain's load; `docs/manifest.yml`, for which page owns the limit figures.

#### 2. The specification-change ticket: amendments to S06's drafted T35

**T35 has not been created (fact, 2026-10-03).** Searched: the tree of every ref
(`git for-each-ref`, 73 refs; `git ls-tree -r --name-only <ref> -- tickets/`) and the
ticket and documentation text of every ref (`git grep -w -E 'T3[4-9]|T[4-9][0-9]|S1[1-9]|S[2-9][0-9]'
<every ref> -- tickets/ docs/ AGENTS.md`). The highest ids with a file on any ref are
**T33** and **S11**; no file is named `T34`, `T35`, `T36` or `S12` anywhere. In text:
`T34` is claimed by S10's notes (a branch `ticket/t34-guide-page-and-diagrams` exists and
holds no ticket file); `T35` is S06's draft, in the hand-back notes of
`tickets/S06-licensed-deductions-in-order.md`; `T36` and `S12` appear once each, on the
unmerged branch `ticket/S07-undoing-a-deduction` (commit `a7bd369`), named there as the
next free ids and claimed by nothing. So, as step 9 says, what follows amends the draft.
The next free ids on this evidence are T36 and S12.

**The draft can carry the decisions, on four conditions.** Its Non-goal "Answering the
three open questions on chains: they are added, not settled" is reversed; its statement
"*Not changed:* `solver.allium`, `human-solving.allium`, `generation.allium`" is no
longer true of `human-solving.allium`; its `estimated_size` goes from M to L; and its
step "Say whether `catalogue_version` is raised, and why" becomes "raise it". None of
these breaks its goal, which is one order and the first taken. If the maintainer would
rather land the order alone, the amendments below lift out whole as a ticket of their
own under T36, depending on T35; they are written so that either works.

**Which of the draft's open questions the decisions answer.** S06's step 7 has the
change add six open questions to `technique.allium`. Five are now answered and are
written as text, not as questions:

| The draft's open question | Answered by | Now |
| --- | --- | --- |
| whether a chain may pass through a candidate twice, and what the minimums of `x_chain` and `xy_chain` count | D1, D2 | a walk; four distinct candidates, three distinct cells (`AlternatingChains`) |
| what `remote_pair`'s output is | D3 | the component, adjacent pairs counted (`RemotePairs`) |
| the order on paths and which witness `holds` reads | D5, D6 | fewest links kept, the load read link by link (the load sentences and the two chain guarantees) |
| what "other cells" excludes | D8 | only the two named cells (`Wings`, `SingleDigitPatterns`) |
| whether a colouring's wrap and trap are one deduction | D9 | separate, the component the witness (`SimpleColouring`) |

The sixth, what `full_unit` and the hidden subsets license on a grid that contradicts
itself, is untouched and stays the draft's. In their place the change adds Q1 to Q9 of
1.11. D4 and D7 answer no question of the draft; they add text to the config and to
the two chain guarantees.

**Replacement text, section by section.**

> **Context** gains, after its last sentence: "S11 (hand-back notes) then put nine
> readings of ranks 13 to 27 to the maintainer, who answered all nine on 2026-10-03
> (D1 to D9, "Open points settled" of `tickets/S11-chain-techniques.md`). They change
> what ranks 18 to 20 and 22 to 27 list, what load a chain carries, and one catalogue
> limit; S11's notes carry the proposed text for each sentence, with its line. S11's
> runs took the first entry of a list ordered as S06 proposes, at every rank."
>
> **Goal** gains: "`technique.allium` states the chain techniques as decided: a walk
> (D1); the minimums in distinct candidates and cells (D2); `remote_pair` as a component
> (D3); what an entry holds (D4); the load read link by link and the witness of fewest
> links (D5); a forcing chain's root, paths and witness (D6); `max_chain_links` 16 and
> `catalogue_version` `technique-3` (D7); "other cells" as the two named cells (D8); wrap
> and trap apart (D9). `effort.allium`, `human-solving.allium` and the two explanation
> pages follow, as S11's notes list them."
>
> **Non-goals**: strike "Answering the three open questions on chains: they are added,
> not settled." Add: "Deciding anything S11's maintainer was not asked: whether the
> link-by-link load reaches ranks 21, 22 and 29, which of trap and wrap is listed first,
> the definition choices S11's notes mark as built, the ladder's order past rank 12, and
> calibration. Each is written as an open question. Making the order normative past
> rank 12 without the maintainer saying so."
>
> **Files touched** gains `docs/specs/human-solving.allium` (line 108 and one sentence
> of the projection), `docs/explanation/human-solving.md` (lines 96, 130, 247, 248,
> 257-258) and, in `docs/explanation/puzzle-design.md`, lines 918, 920, 924, 925 and 949
> beside L914.
>
> **Steps**, step 2 is replaced: "Change `technique.allium` first, in two commits. The
> first is S06's: the Excludes sentence, the order, `Entry`, the two guarantees, and
> the open question on a grid that contradicts itself. The second is S11's: the
> config, the vocabulary, the load sentences, the sentence on what is listed each once
> at ranks 22 to 27, `Wings` and `SingleDigitPatterns` (D8), `SimpleColouring` (D9),
> `RemotePairs`, `AlternatingChains`, `BoundedForcingChains`, `AdvancedExamples`, the
> resolved question and the open questions Q1 to Q9, each as S11's notes give it with
> its line. Raise `catalogue_version` to `technique-3`: D7 changes a limit and D1, D2, D3,
> D5, D6, D8 and D9 each change an exact variant." Step 3 gains: "then
> `human-solving.allium`, after asking the maintainer whether `reasoning_depth` becomes
> `0..16` or keeps `0..24`." Step 4 gains the two explanation pages' lines. A new step
> 6: "Check by hand, on the states S11's notes give: the remote-pair component of five
> cells is one deduction striking r2c2 and r7c4; the naked pair `{1,2}` at r1c1, r1c2
> gives rank 26 two endpoint strikes and 26 closures and rank 27 26 entries, each kept
> as a contradiction of two paths and load 4; an `xy_chain` of seven links reads a load
> of 3."
>
> **Acceptance criteria** gains: "No sentence of `technique.allium` leaves unsaid any of
> D1 to D9; each clause S11 marks as built has an open question; nothing the maintainer
> was not asked is stated without one. `max_chain_links` is 16 and every page that
> prints the figure, the forcing-path count or `technique-2` agrees. The witness of each
> of ranks 22 to 27 is ordered, so the draft's sentence 'by an order on proofs for ranks
> 23 to 27' is met; whether the guarantee `OneOrder` is then made normative for those
> ranks is asked of the maintainer and recorded."
>
> **Verification** gains:
> `rg -n 'max_chain_links: Integer = 16|technique-3' docs/specs/technique.allium` (two
> lines); `rg -n 'technique-2|0\.\.24|0–24' docs/` (nothing, or only what the maintainer
> kept at step 3); `rg -c '^open question' docs/specs/technique.allium` (13: the
> four that stand, 1162 among them rewritten in place, the draft's one, and Q2 to Q9).
>
> **Open points** gains: whether `reasoning_depth`'s range follows the limit; whether
> ranks 13 to 21, 28 and 29 get their "each once" sentence from `Entry` and `listed` or
> one of their own, S11's paragraph speaking for ranks 22 to 27 only; whether
> `OneOrder` becomes normative for ranks 13 to 29 now that every witness is ordered
> (ranks 13 to 21, 28 and 29 by the role of S06's "Step 5, extended", ranks 22 to 27 by
> this change), or waits on the ladder's order.

**What remains the draft's own:** the order itself and its normative reach to rank 12;
`Entry` and `listed`; the two guarantees; the three lines of `reach`, `effort` and
`lapse` that take the first in order; `puzzle-design.md` L914; the question on a grid
that contradicts itself; its three open points. One coupling to know of (fact, read on
the unmerged S07 branch; nothing here depends on it): S07's notes propose two further
guarantees that "could ride with that change".

#### 3. What the tickets downstream need

**S07, the five rows of its table (ranks 23 to 27).** S07's own table, on its unmerged
branch, holds these rows as provisional on D1 to D7. As decided:

| Rank | The deduction | Its witness | What an undoing must restore on the earlier grid |
| --- | --- | --- | --- |
| 23 `remote_pair` | one for each two-coloured component of at least four cells holding exactly `{a,b}`; strikes a and b at every outside cell seeing two cells of unlike label within 16 peer links, neighbours counted (D3, D7) | the component, a set of cells | a, b or both given back at targets, all of one deduction together; every cell of the component still exactly `{a,b}`; no pair cell added to or cut from it, since the component is the earlier grid's and a cell given `{a,b}` beside it joins it |
| 24 `x_chain` | one for each output: every candidate of the digit weakly linked to both endpoints, a node of the chain included (D4 as written; as built, Q9); a walk holding at least four distinct candidates (D1, D2) | the walk of fewest links (D5) | the digit given back at a nonempty set of targets, every target that is a node of the witness among them, since the chain stands on it; every conjugate link the witness walks exactly two places on the earlier grid, so no target is given back on the line of a link it is not part of |
| 25 `xy_chain` | as rank 24, on bivalue cells, equal endpoint digits, at least three distinct cells | as rank 24 | the endpoint digit given back at targets; every chain cell still bivalue. A foreign digit is never in an output |
| 26 `alternating_inference_chain` | endpoint strikes as rank 24 with mixed digits and no minimum; and every closure, one candidate each, striking or placing (D4) | the walk of fewest links; a closure and an endpoint strike with one output are one entry, the fewer links kept | a strike: as above. A placement: the digit out of the cell, with the candidates upkeep took; on the earlier grid the closure from the denied literal must stand, so its first link is strong (the cell bivalue, or the digit with two places in a unit) |
| 27 `forcing_chain` | one for each conclusion: a placement or one struck candidate; the root never its own target; at most 9 paths, each a walk of 1 to 16 links (D6, D7) | the proof of fewest links in all; always a contradiction (D6) | every path of the kept contradiction valid link by link on the earlier grid, and the terminals still incompatible there (the cell's candidates, or the digit's places, exactly those the proof denies) |

Three corrections to what S07's rows assumed. (1) Row 23 read "one deduction for each
chain, as written"; it is the component. (2) Row 27 read "no candidate twice and no
empty path"; the decision is a walk with the root never its own target. S07 found that
under a walk a forcing chain licenses a single wherever the placed candidate has a
strong link. One such proof is the root coming back to itself in two links, and as
built that proof is refused (Q7); whether any other is left under a walk was not
checked here (S07 confirms on paper before its row 27 stops being provisional). S07's
other finding stands under every option: a path of two links through another cell's single can place what a single
places, so `forcing_chain` overlaps the singles. (3) Capacity: under D5 the load of rows
23 to 26 is 3 (2 for one strong link), so seeing them no longer waits on the witness;
row 27's load is the lesser of its distinct candidates and its paths plus two (as
built, Q7), read from the kept contradiction. In the decided runs every entry of rank
27 a profile of capacity 4 was shown had two paths, but that is a fact of those runs and
not a rule: a proof of one path loads 2 or 3 (57 stand on look 28 of `study_advanced`,
one of them `-4r1c3`, `+4r1c9`, `-4r5c9`, load 3), and a proof of three paths over four
candidates loads 4 (the constructed state under the load sentences, part 1). An undoing
at capacity 4 must therefore read the load and not count the paths. Rank 26 also undoes what `pointing`,
`claiming` and a naked pair would (Q6), and `x_chain` what a conjugate pair with a
detour would (Q3). The strict reading stays unmeasured for these rows.

**S08 and the designed generator (T20, drafted in T19's hand-back notes).** A chain in a
run, under the decided configuration, all twenty-nine ranks, systematic, full marks:

- **How often a chain is the step** (what is listed: D1 to D4, D6, D8, D9; what is
  hidden: D5; the limits: D7). At capacity 729, of 14,324 steps over the 226
  grids, ranks 23 / 24 / 25 / 26 / 27 take 0 / 5 / 35 / 97 / 62 (run table 3). Chains are
  searched at 221 looks on 54 grids: 1 look on a grid at the least, 2.5 at the median,
  16 at most (run table 2). At capacity 4: 0 / 7 / 23 / 63 / 0 of 13,418 steps, 132 such looks.
- **Cost a look** (a walk, D1; sixteen links, D7), counted in implications inspected, at capacity 729 (run table 5): 28,869 at
  the median and 115,477 at most for the five ranks together; and for rank 27 a further
  452,304 cases tested at the median, 1,460,976 at most. Every other look costs what
  S06 measured. Fact (baseline, 24 links, fifteen grids): searched at every look and
  not only where the lower ranks have stopped, rank 26 costs 10,428 a look at the
  median and lists 28,373 entries over 785 looks.
- **What a profile without chains loses.** Ranks 1 to 22, 28 and 29 alone solve 173 of
  226 at capacity 729 and 171 at capacity 8 and at 4; with the five, 205, 205 and 191
  (run tables 1 and 16). The hardest step is of rank 25, 26 or 27 on 10, 10 and 27 grids at
  capacity 729, of which 10, 9 and 14 solve and 0, 1 and 13 stall (run table 2, and the per-grid
  table of part 4). No run held a repertoire with ranks 23 to 26 and without
  `forcing_chain`; the nearest evidence is capacity 4, where, the load read as D5 has
  it and the paths counted as D6b has them, rank 27 lists 2,262
  entries and takes no step, and 14 grids that solve at 729 stall (run tables 3, 4 and 11).
- **The limits (D7; paths counted as D6b has them).** Sixteen links against 24 changes
  no grid's end and no hardest rank at any capacity; steps differ on four grids at
  capacity 729 and 8 and on one at 4 and 3 (run table 10). Nine paths never binds (greatest
  held: 5, run table 15). Baseline, 24-link keys: sixteen links cuts 27 of the 5,477 candidates
  the five reach on 225 states; and 19,126 of the 24,603 candidates no completion uses
  are struck by none of the five at any length. 21 grids stall with every rank (run table 1).
- **For a technique contract (D5).** `reach.allium`'s `Rate` for the `expert` preset
  (capacity 4) now sees every single chain and solves 191 of these grids where the
  load as written solved 172 (baseline, 24 links); a contract whose ceiling is rank 24
  to 26 can now be met by that preset. A contract that names `forcing_chain` is met at
  capacity 4 by any proof whose load, the lesser of its distinct candidates and its
  paths plus two (as built, Q7), is at most 4: in the decided runs those were proofs of
  two paths and no other, and by the formula a proof of one path, or of three paths
  over four candidates, is seen too. A lower limit can lengthen a run (one grid: 86
  steps at 16 links, 82 at 24, run table 10), so more removed, or a tighter limit, is not shown
  to be harder. S08 measures ranks 1 to 12 only; everything here is a condition on its
  carrying over, not a measurement of removal. Calibration of capacity is open: at
  capacity 3 and 4 the same 191 grids solve by other steps (run table 11).

**The ticket that builds the catalogue in Rust** (no id is taken for it here). The
static graph first: a node is a standing candidate of an open cell, at most 729; a
literal a node asserted or denied, at most 1,458; a weak link gives two implications
(`+u -> -v`, `+v -> -u`) and a strong link two (`-u -> +v`, `-v -> +u`), at most
20,412 weak and 2,916 strong, 23,328 in all. A positive literal is left only by weak
links and a negative one only by strong ones, so "alternate, starting and ending
strong" is "from a denied literal to an asserted one".

| Rank | The search to take | Its bound | Kept witness | Measured a look (run table 5; median, most) |
| --- | --- | --- | --- | --- |
| 23 (D3, D7) | components of each pair graph by breadth-first search in cell order, two-coloured from the least cell; distances between cells by breadth-first search | the counter `patterns` is twice the joined pairs of pair cells, the reads of the colouring alone; the whole search is bounded apart, in the note below the table; no budget needed | the component (D3); of two with one output the fewer cells, then the lesser list of cells (as built, Q4) | `patterns`, baseline, 24 links: 6 and 28 |
| 24, 25 (D1, D2, D4) | layered reachability over (literal, the set of distinct candidates or cells met, capped at four candidates or three cells) on the graph of one digit, or of the bivalue cells; the pair is listed when the goal is within 15 links | polynomial: literals times small sets, each set joined to the start; no budget needed | fewest links (D5), then the least list of literals: a first in, first out search over those states gives it (notes below) | 435 and 1,631; 133 and 5,446 |
| 26 (D1, D4) | one breadth-first pass from every literal, both signs, to 15 links; an endpoint strike on `{s, t}` exists when `+t` is within 15 of `-s`, a closure when a literal's negation is | at most 1,458 passes of 23,328 implications, 34,012,224 a look | as above (D5); a first in, first out search from the start gives it (notes below) | 14,190 and 55,788 |
| 27 (D1, D6, D7) | the same passes to 16 links; then each case once: a root literal with a kind and its anchor (contradiction), or a kind and anchor with a conclusion (verity) | the passes as rank 26; cases: literals times (nodes + weak links + open cells + unit-digit pairs), twice over | fewest links in all (D5, D6c), then kind (contradictions first), anchor, paths; the kinds and anchors are tabled in part 4 | 14,190 and 56,916; cases 452,304 and 1,460,976 |

- **Rank 23's whole search, beside `patterns`** (the Codex review's finding 5). Where a
  component of `m` cells and `e` joined pairs is kept, the search as built reads `2e`
  joined cells to colour it, `2e` again from each of its `m` cells for the distances,
  and then, for each two cells of unlike colour, their common peers, at most 20. A path
  of four cells reads 6, then 24, then 27 common peers over 4 unlike pairs; the hand
  state of five cells 8, 40 and 36 over 6 (`ai_tmp/s11/codex/f5_rp_work.py`, scratch).
  Only the first figure is in `patterns`. With at most 81 pair cells and 20 peers to a
  cell the bounds are 1,620 for the colouring, 131,220 for the distances and 32,800 for
  the common peers (at most 1,640 unlike pairs), so no budget is needed; over the 54
  states the most at one look was 28, 84 and 59.
- **Which breadth-first search gives the witness** (the Codex review's finding 6;
  these notes said none does, which was wrong). The least list of literals among walks
  of fewest links, from a fixed start to a fixed goal, is what a breadth-first search
  from the start returns if it is first in, first out, takes the implications leaving
  each literal in ascending literal number, marks a literal when it is first reached
  and reads the path back through the literal that first reached it. Inference, by
  induction on the layer: literals leave the queue in the order of their least lists,
  so the first to reach a literal has the least list of all that do. Fact
  (`ai_tmp/s11/codex/f6_bfs.py` and `f6_every_pair.py`, scratch): on the 54 states it
  equals the engine's kept path on 418,190 of 418,190 pairs of start and goal of the
  whole graph with no limit on links, and on every pair the techniques use: 1,169
  endpoint pairs with a target and 854 closures of rank 26 (1,098 and 726 of them
  within 15 links), and 3,031 paths of the kept cases of rank 27 at 16 links. The engine's own way, forward by exact distance to the goal (one
  pass backward from each goal), gives the same list and is not needed for it.
- **Where it does not.** Three cases, each measured on the same states. (1) A search
  that puts each layer in ascending literal number before expanding it, which is what
  "in literal order" most readily suggests: it differs on 38,200 of the 418,190 pairs,
  on 408 of the 1,169 endpoint pairs, on all 854 closures (on each of the 854 it returns
  the mirror, the greater list) and on 275 of the 3,031 paths. The definitions' sentence "an ordered
  breadth-first search does not return it" (section 3.1) is true of this search and was
  written too widely. (2) Ranks 24 and 25: a search over literals alone stops at a walk
  that is under the minimum. Of the 10 endpoint pairs of rank 24 with a target, one is
  reached by a shorter walk of fewer than four candidates; none of the 88 of rank 25
  is. The search must run over the capped sets, (literal, set), as the table says;
  there the same first in, first out search gives the kept walk on 10 of 10 and 88 of
  88, and the layer-sorted one differs on 13 of the 88. (3) What no single search from
  a start gives: the least over several endpoint pairs or a closure with one output,
  and for rank 27 the least over kinds and anchors; those are comparisons of kept
  witnesses, by the order of part 4.
- **Entries (D4, D6).** Several endpoint pairs with one output are one entry keeping the least
  witness; a closure with the same single output joins it. Rank 27 keeps the least over
  all cases with one conclusion. The list is by rank, a placement before strikes, then
  the output; that order is total whatever the witnesses.
- **Counters to keep**, each a function of the grid and the limits, not of a program,
  so that a rebuild can be held against part 4: `implications` (one for each implication
  leaving a literal reached with a link to spare, once for each start; starts are the
  denied literals of the digit at rank 24, of bivalue cells at rank 25, every literal at
  ranks 26 and 27), `patterns` at rank 23, `combinations` at rank 27; and for each entry
  its links, its longest path, its paths, the candidates held whole and the load read.
- **What `implications` leaves out.** At ranks 24 and 25 the search over capped sets
  reads more than the counter shows; it has a polynomial bound and its own work was
  not recorded under the decided keys.
- **No budget, no timeout.** Under the decided keys no search above needed a budget and
  none was hit in any run (run table 1). The searches that did need one (no candidate twice,
  least load) are not the decision.
- **Laziness is a choice to state.** The runs searched ranks 23 to 29 only at looks
  where ranks 1 to 22 list nothing the profile sees. What was checked (run table 13) is the
  carry-over, still lazy: with all 29 ranks held over all 226 grids at sixteen links,
  every grid outside those that need a chain is the same run as without chains (172,
  171, 171 and 155 grids at capacity 729, 8, 4 and 3; 0 differ). That searching every
  rank at every look gives the same run is inference from the list's order (the step
  is the first entry, and a lower rank comes first), not a measurement: no run of the
  decided keys searched the chain ranks at every look. A list for a caller, as against
  a run, needs every rank.
- **Soundness to hold the build to.** 51,220 distinct (state, output) pairs of ranks 18
  to 27 from the decided runs were checked with a SAT solver: all sound, on 10,351
  states each with one solution. A second program written apart agreed on 1,080 of
  1,080 lists at sixteen links on 54 states. D8's and D9's entries have the SAT check
  and no second program.

**The fixtures `AdvancedExamples` asks for.** `CatalogueValidation` wants each positive
and near-miss schema on a full 81-cell state with a solution. The spike's ten hand
states are **synthetic**: every cell open, inert filler (`{7,8,9}`, or `{1,2,7,8,9}` for
the naked-pair state), no solution; they fix
shapes and expected entries and are not fixtures. What each gives, and where a true
state stands:

| Schema | Hand state (synthetic) | Expected, as decided | A true state from the sampled runs |
| --- | --- | --- | --- |
| remote-pair component | `{1,2}` at r1c1, r1c4, r4c4, r4c2, r7c2; r2c2 `{1,2,7,8,9}`; r7c4 `{1,7,8,9}` | one entry striking 1, 2 at r2c2 and 1 at r7c4; load 3, a component's figure, not run on this state. Near miss: 3 added to r1c4 | only in the run at capacity 3: `seventeen_024` look 39, `seventeen_055` look 45, `seventeen_059` looks 42 and 46, `unbiased_002` looks 17 and 37, among the nine (run table 15); none at capacity 729, 8 or 4 |
| `x_chain`; a weak first link | 1 at r1c2, r1c8, r5c2, r5c7, r2c7, r3c7 only | one entry striking 1 at r2c7 and r3c7, 3 links. Near miss: 1 added to r1c5 | `top1465_041` look 20, a step striking 1 at r5c9 (run table 7) |
| the minimum, a walk that repeats, a node of the chain struck, a skyscraper striking its own cell | 1 at r1c4, r1c9, r2c4, r2c6 only; and 1 at r1c1, r1c2, r2c3 only | first state: 1 struck at r1c4 by a chain of 3 links that holds it, and at r2c6 by a walk of 5 links over 4 candidates; rank 19 strikes r1c4. Second state: no `x_chain` | a walk that repeats: `study_advanced` look 28, an `x_chain` of 5 links striking 2 at r8c7 and r8c9 beside two of 3 links (baseline, 24 links). A pattern striking its own cell: none was looked for on a true grid |
| `xy_chain`; a trivalue member | r1c1 `{1,2}`, r1c5 `{2,3}`, r5c5 `{3,4}`, r5c3 `{1,4}`, 1 at r2c3, r3c3 | one entry striking 1 at r2c3 and r3c3; 7 links, 8 candidates, load 3, seen at capacity 4. Near miss: r1c5 `{2,3,5}` | `derived_advanced_min` look 42, a step of 15 links striking 4 at r1c9 (run table 7) |
| AIC, closure, contradiction | r1c1, r1c2 `{1,2}`, every other cell `{1,2,7,8,9}` | rank 26: 2 endpoint strikes and 26 closures of 5 links; rank 27: 26 entries, each a contradiction of two paths of 2 links, load 4 by the formula | a placing closure: `derived_advanced_min` look 20, 1 placed at r1c6 (run table 7). A contradiction: `derived_advanced_minrev` look 24, 1 placed at r1c6 on paths of 12 links at most (run table 7) |
| a closure whose last link lies inside its own cell | r5c5 `{1,2}`, r5c1 `{1,3}`, r1c1 `{2,3}`, r1c5 `{2,7,8,9}` | 1 placed at r5c5 by 7 links; contradictory, so a test of the search and never a fixture | not looked for |

The true states are named by grid and look of the decided run at capacity 729 and
sixteen links; a rebuild reaches each by running the grid (part 4). The study puzzles
are in `docs/explanation/puzzle-design.md`, and T19's notes already propose them as
`AdvancedExamples` candidates once well-posedness is verified in Rust. No hand figure
was confirmed by a SAT check (the states have no solution); the first program agreed
with all 82 expected cases and the second found no hand figure wrong.

#### 4. What a rebuild needs

**The definitions as decided.** The grid, links and implications are part 3's. Then:

- *Rank 23 (D3).* For each pair of digits, the open cells holding exactly that pair,
  joined where peers. A component of at least four cells that two-colours is one
  deduction; a target is an open cell outside it that is a peer of two cells of unlike
  colour at distance at most 16 in the component, distance 1 included; strike both
  digits where they stand. Witness the set of cells; links recorded as cells less one.
  Two components with one output are one entry, keeping the one of fewer cells, then
  the lesser list of cells (as built, Q4); no run met the case.
- *Ranks 24 to 26, endpoint strikes (D1, D2, D4).* A walk from a denied candidate `s` to
  an asserted candidate `t`, `s` and `t` distinct, of at most 15 links. Rank 24: nodes
  of one digit, unit strong links, at least four distinct candidates. Rank 25: nodes of
  bivalue cells, cell strong links, weak links between peers on one digit and never
  inside a cell, equal endpoint digits, at least three distinct cells. Rank 26: every
  node and link, no minimum. Targets: every node other than `s` and `t` weakly linked
  to both in the whole graph, nodes of the walk included (as built, Q9). One deduction
  for all targets.
- *Rank 26, closures (D4).* A walk from a literal to its negation, at most 15 links:
  strike if the start is asserted, place if denied. Every closure listed.
- *Rank 27 (D6).* Contradiction: from a root literal, one walk of 1 to 16 links to each
  terminal of one of five kinds (a candidate asserted and denied; two digits asserted in
  one cell; one digit asserted in two peer cells; every candidate of an open cell
  denied; every place of an unplaced digit in a unit denied); conclude the root's
  negation. Verity: roots that are both values of one candidate, every candidate of a
  cell, or every place of an unplaced digit in a unit, each with a walk to one
  conclusion. No terminal is the root and no conclusion is a root. Paths are counted one
  to each terminal, or one from each root, and at most 9.
- *The witness kept (D5, D6).* Fewest links (rank 27: fewest in all). Then, compared as
  lists of integers element by element: for ranks 24 to 26 the links then the literals
  of the canonical form (the walk or its mirror, whichever starts at the lesser node;
  for a closure the lesser list); for rank 27 the total, the kind (contradictions 1 to
  5, verities 6 to 8), the anchor, the count of paths, then each path's links and
  literals, paths in order of terminal or root. A literal's number is twice its node's
  for the asserted and one more for the denied; a node's is nine times its cell's
  row-major index plus its digit less one. For rank 27 the list is `[total, code, a, b,
  P, then for each path its links and its literals]`, `P` the count of paths listed.
  For one output a kind and its anchor name one proof, so the comparison ends at the
  anchor and `P` and the paths decide nothing: fewer paths are not preferred (Q5).
  The codes and the anchor `(a, b)` of each kind are these. Units are numbered 0 to 26:
  rows 1 to 9 are 0 to 8, columns 9 to 17, boxes 18 to 26; the same places in two units
  are two witnesses, one for each unit. A cell is its row-major index.

  | Code | Kind | Terminals (contradiction) or roots (verity) | Anchor `(a, b)` |
  | --- | --- | --- | --- |
  | 1 | contradiction: a candidate asserted and denied | `+x` and `-x`, `x` a node | (node of `x`, 0) |
  | 2 | contradiction: two digits asserted in one cell | `+x` and `+y`, distinct nodes of one cell | (lesser node, greater node) |
  | 3 | contradiction: one digit asserted in two peer cells | `+x` and `+y`, nodes of one digit in peer cells | (lesser node, greater node) |
  | 4 | contradiction: every candidate of an open cell denied | `-x` for every node `x` of the cell | (cell, 0) |
  | 5 | contradiction: every place of a unit's unplaced digit denied | `-x` for every place `x` of digit `d` in unit `U` | (unit number, `d`) |
  | 6 | verity: both values of one candidate | `+p` and `-p` | (node of `p`, 0) |
  | 7 | verity: every candidate of one open cell | `+x` for every node `x` of the cell | (cell, 0) |
  | 8 | verity: every place of a unit's unplaced digit | `+x` for every place `x` of digit `d` in unit `U` | (unit number, `d`) |

- *The load (D5).* Ranks 24 to 27: the lesser of the distinct candidates of the witness
  and its paths plus two; a path of no links is not counted; targets count only as
  nodes of the witness. Rank 23: 3. A component has no path; the engine counted one for
  it and took the lesser of that sum and its candidates, at least 8, which is 3 always.
- *Ranks 16 to 20 (D8).* A target is any cell but the two the sentence names; the
  lists of ranks 18 to 20 alone change.
  *Rank 22 (D9).* One trap for each component and one wrap for each label with two
  nodes that are peers, each with the component as witness.
- *The list and the run.* By rank, a placement before strikes, then the output. A
  profile hides by `sees`; the step is the first entry. The run's profile: all
  twenty-nine ranks, full marks, every extent, systematic, unfixed,
  `uniqueness_promised` true, capacity as stated; upkeep in every unit after a
  placement. The loads of ranks 21, 22 and 29 are the rebuild's readings of S06's
  notes, held whole.

**The counters** are those of part 3; S06's `patterns` for ranks 1 to 22, 28 and 29 are
as S06's notes define them. Counts, never seconds.

Totals a rebuild can be held to beyond the per-grid table below, each over the chain
looks of the decided run at sixteen links (run tables 3 to 6 and 15). Five figures are ranks 23 /
24 / 25 / 26 / 27, four are ranks 24 to 27:

| Counter | Capacity 729 | Capacity 4 |
| --- | --- | --- |
| Chain looks | 221 | 132 |
| Steps taken | 0 / 5 / 35 / 97 / 62 | 0 / 7 / 23 / 63 / 0 |
| Entries listed | 0 / 5 / 251 / 3,431 / 6,956 | 0 / 7 / 181 / 2,584 / 2,262 |
| Outputs hidden by capacity | 0 / 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 / 2,271 |
| `implications`, ranks 24 to 27 | 107,414 / 97,910 / 3,835,215 / 3,867,666 | 61,084 / 54,493 / 2,274,764 / 2,298,386 |
| `combinations`, rank 27 | 109,029,778 | 66,005,552 |
| Links of the kept witness of the entries listed, median and greatest, ranks 24 to 27 | 5, 7 / 13, 15 / 9, 15 / 14, 42 | 5, 7 / 11, 15 / 9, 15 / 10, 32 |
| Load read, least and greatest: ranks 24 to 26; rank 27 | 3, 3; 4, 7 | 3, 3; 4, 4 |
| Paths of an entry of rank 27, least and greatest | 2, 5 | 2, 2 |

The per-grid column of implications sums ranks 23 to 27 together (7,909,625 and
4,689,553), so it holds rank 23's `patterns` beside the four ranks' `implications`.
Not pinned by these notes: rank 23's `patterns` on its own under the decided keys, which
the report of the decided run does not tabulate; and each entry's links, longest path
and whole candidates beyond the figures here and under "The decided configuration, run
as a set".

**The grids.** 226 names, each grid with exactly one solution, in the order of the table
below: each of the 4 study puzzles followed by its 2 derived grids (12), then 100
`seventeen`, 50 `unbiased`, 50 `top1465`, 11 `norvig_hardest`, 3 named. The 226 names
are 225 distinct grids: `derived_expert_min` and `derived_expert_minrev` are one grid,
both orders of removal reaching it, so their two rows are equal and are not a slip. Fifteen need
no network: the four study puzzles are in `docs/explanation/puzzle-design.md`; each
derived grid is made from its study puzzle by visiting the 81 positions once, row-major
for `_min` and reversed for `_minrev`, and taking a given out wherever exactly one
solution is left; the three named grids are printed in the table. The other 211 are a
file, a line range and a SHA-256, as fetched on 2026-10-03, and fetching them again is a
separately authorised action:

```text
https://raw.githubusercontent.com/t-dillon/tdoku/master/data.zip
  9be0601c721ac4e702e3fe097576f025fcb99b216aabfe9dbea37cac43e6bc4f
  data/puzzles2_17_clue   lines 5-104 (seventeen_001 is line 5)
  2cf268735dfd40d32269b5f20ebbea067037e5976b4d016ce0b4d3c0a50deb30
  data/puzzles1_unbiased  lines 1-50
  07a35f8a9cb394315c0b85bc73fea52ce552f4181acdcd0d550aace171498f40
http://magictour.free.fr/top1465  lines 1-50
  32837f38ece94e75678deadbe256aeafda704c8d4c2f8b5095a630ce2d0114d3
https://norvig.com/hardest.txt  all 11 lines
  398e1e5df4f50723078e3b510e00ea7eb9e104c521a9a186b4c20cd066c0459f
grids.txt, 226 lines of name, a tab and 81 characters, as built from the above
  49d96f93ea1da57559492887481ab5837f3464c76e18f7753028e6fb076990a3
```

(The path `data/puzzles2_17_clue` is the file's name in its archive, quoted as it is.)

**Each grid's counts under the decided configuration**, at sixteen links, at capacity
729 and at capacity 4. Columns: name; givens (81 characters, row by row, `.` an open
cell); the count of givens; then for each capacity the end, the steps taken, the hardest
rank of a step (0: none), the looks at which ranks 23 to 27 were searched, the entries
of those ranks summed over those looks, and their `implications` summed likewise. A
grid with no such look has ranks 23 to 27 unsearched: 0, not "none found". The givens
of the 211 fetched grids are written `-`: S06 left other people's collections out of
the repository as not a spike's to decide, and the source, line and hash above give
each one. Sums, as a check: capacity 729, steps 14,324, such looks 221, entries 10,643,
implications 7,909,625; capacity 4, 13,418, 132, 5,034 and 4,689,553.

```text
name givens givens_count c729_end c729_steps c729_hardest c729_chain_looks c729_entries_23_27 c729_implications c4_end c4_steps c4_hardest c4_chain_looks c4_entries_23_27 c4_implications
study_beginner ..792..5..2..16.376..3..4.9......29....6.1....78......9.2..5..356.87..4..8..496.. 32 solved 49 2 0 0 0 solved 49 2 0 0 0
derived_beginner_min ...9...5..2..1..376..3...........2.....6......78......9.2.....356.87..4..8..496.. 24 solved 57 2 0 0 0 solved 57 2 0 0 0
derived_beginner_minrev ..79...5..2..1..376..3..4.9......2.....6......78......9.......356..7........4.6.. 22 solved 59 3 0 0 0 solved 59 3 0 0 0
study_intermediate .3.47..5...2.8......51.3..7..9...2.42.79.83.11.3...5..3..7.49......9.6...5..31.4. 32 solved 54 6 0 0 0 solved 54 6 0 0 0
derived_intermediate_min ...4...5...2.8.........3..7..9...2..2.79.8..11.3......3..7.49......9.6...5..31.4. 25 solved 71 26 5 364 172008 solved 71 26 5 247 172008
derived_intermediate_minrev .3.47..5...2.8.......1.3..7..9...2.42.79.83.11...........7..9........6...5..3.... 24 solved 63 6 0 0 0 solved 63 6 0 0 0
study_advanced 8..92.35.......8.7....84.....3..8.2..1.562.3..5.4..9.....81....5.9.......87.59..3 29 solved 58 16 0 0 0 solved 58 16 0 0 0
derived_advanced_min ...92..5.......8.7.....4.....3....2..1.56..3..5.4..9.....81....5.9.......87.59..3 24 solved 75 26 7 693 345515 solved 75 26 7 571 345515
derived_advanced_minrev ...92..5.......8.7.....4.....3....2..1.562....5.4..9.....81....5.9.......87.59..3 24 solved 82 27 11 705 456573 stalled 24 26 5 16 156908
study_expert ..94.......8..9...7.4.38.9..6.....7...1.4.8...7.....1..9.75.3.6...1..5.......69.. 25 solved 70 19 0 0 0 solved 70 19 0 0 0
derived_expert_min ..9........8..9...7.4.38....6.....7...1.4.8...7.....1..9.75.3.6...1..5.......69.. 23 solved 71 19 0 0 0 solved 71 19 0 0 0
derived_expert_minrev ..9........8..9...7.4.38....6.....7...1.4.8...7.....1..9.75.3.6...1..5.......69.. 23 solved 71 19 0 0 0 solved 71 19 0 0 0
seventeen_001 - 17 solved 76 7 0 0 0 solved 76 7 0 0 0
seventeen_002 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_003 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_004 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_005 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_006 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_007 - 17 solved 70 7 0 0 0 solved 70 7 0 0 0
seventeen_008 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_009 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_010 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_011 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_012 - 17 solved 72 5 0 0 0 solved 72 5 0 0 0
seventeen_013 - 17 solved 71 5 0 0 0 solved 71 5 0 0 0
seventeen_014 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_015 - 17 solved 70 7 0 0 0 solved 70 7 0 0 0
seventeen_016 - 17 solved 70 7 0 0 0 solved 70 7 0 0 0
seventeen_017 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_018 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_019 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_020 - 17 solved 68 5 0 0 0 solved 68 5 0 0 0
seventeen_021 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_022 - 17 solved 71 26 1 106 27594 solved 71 26 1 96 27594
seventeen_023 - 17 solved 68 5 0 0 0 solved 68 5 0 0 0
seventeen_024 - 17 solved 76 18 0 0 0 solved 76 18 0 0 0
seventeen_025 - 17 solved 70 7 0 0 0 solved 70 7 0 0 0
seventeen_026 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_027 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_028 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_029 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_030 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_031 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_032 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_033 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_034 - 17 solved 66 5 0 0 0 solved 66 5 0 0 0
seventeen_035 - 17 solved 66 5 0 0 0 solved 66 5 0 0 0
seventeen_036 - 17 solved 69 5 0 0 0 solved 69 5 0 0 0
seventeen_037 - 17 solved 71 5 0 0 0 solved 71 5 0 0 0
seventeen_038 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_039 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_040 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_041 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_042 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_043 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_044 - 17 solved 71 5 0 0 0 solved 71 5 0 0 0
seventeen_045 - 17 solved 66 6 0 0 0 solved 66 6 0 0 0
seventeen_046 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_047 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_048 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_049 - 17 solved 70 6 0 0 0 solved 70 6 0 0 0
seventeen_050 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_051 - 17 solved 71 6 0 0 0 solved 71 6 0 0 0
seventeen_052 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_053 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_054 - 17 solved 76 8 0 0 0 solved 76 8 0 0 0
seventeen_055 - 17 solved 82 18 0 0 0 solved 82 18 0 0 0
seventeen_056 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_057 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_058 - 17 solved 69 5 0 0 0 solved 69 5 0 0 0
seventeen_059 - 17 solved 81 18 0 0 0 solved 81 18 0 0 0
seventeen_060 - 17 solved 67 5 0 0 0 solved 67 5 0 0 0
seventeen_061 - 17 solved 79 18 0 0 0 solved 79 18 0 0 0
seventeen_062 - 17 solved 70 5 0 0 0 solved 70 5 0 0 0
seventeen_063 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_064 - 17 solved 74 7 0 0 0 solved 74 7 0 0 0
seventeen_065 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_066 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_067 - 17 solved 69 5 0 0 0 solved 69 5 0 0 0
seventeen_068 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_069 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_070 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_071 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_072 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_073 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_074 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_075 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_076 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_077 - 17 solved 66 5 0 0 0 solved 66 5 0 0 0
seventeen_078 - 17 solved 73 8 0 0 0 solved 73 8 0 0 0
seventeen_079 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_080 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_081 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_082 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_083 - 17 solved 66 5 0 0 0 solved 66 5 0 0 0
seventeen_084 - 17 solved 68 5 0 0 0 solved 68 5 0 0 0
seventeen_085 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_086 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_087 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_088 - 17 solved 70 5 0 0 0 solved 70 5 0 0 0
seventeen_089 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_090 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_091 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_092 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_093 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_094 - 17 solved 65 5 0 0 0 solved 65 5 0 0 0
seventeen_095 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_096 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_097 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_098 - 17 solved 64 2 0 0 0 solved 64 2 0 0 0
seventeen_099 - 17 solved 64 3 0 0 0 solved 64 3 0 0 0
seventeen_100 - 17 solved 67 5 0 0 0 solved 67 5 0 0 0
unbiased_001 - 25 solved 58 5 0 0 0 solved 58 5 0 0 0
unbiased_002 - 25 solved 69 22 0 0 0 solved 69 24 1 7 23645
unbiased_003 - 24 solved 57 3 0 0 0 solved 57 3 0 0 0
unbiased_004 - 27 solved 54 3 0 0 0 solved 54 3 0 0 0
unbiased_005 - 26 solved 67 25 2 287 158208 solved 67 25 3 390 235104
unbiased_006 - 25 solved 56 3 0 0 0 solved 56 3 0 0 0
unbiased_007 - 28 solved 65 25 2 276 125642 solved 65 25 2 211 125642
unbiased_008 - 27 solved 54 2 0 0 0 solved 54 2 0 0 0
unbiased_009 - 25 solved 56 3 0 0 0 solved 56 3 0 0 0
unbiased_010 - 24 solved 74 27 8 292 282072 stalled 15 26 5 16 143053
unbiased_011 - 26 solved 68 25 1 104 51687 solved 68 25 1 91 51687
unbiased_012 - 27 solved 60 8 0 0 0 solved 60 8 0 0 0
unbiased_013 - 25 solved 79 26 7 941 534532 solved 79 26 7 810 534532
unbiased_014 - 25 solved 62 7 0 0 0 solved 62 7 0 0 0
unbiased_015 - 25 solved 64 17 0 0 0 solved 64 17 0 0 0
unbiased_016 - 25 solved 58 25 1 105 24409 solved 58 25 1 104 24409
unbiased_017 - 25 solved 58 7 0 0 0 solved 58 7 0 0 0
unbiased_018 - 25 solved 67 25 1 198 115477 solved 67 25 1 194 115477
unbiased_019 - 25 solved 65 7 0 0 0 solved 65 7 0 0 0
unbiased_020 - 26 solved 58 8 0 0 0 solved 58 8 0 0 0
unbiased_021 - 26 solved 55 2 0 0 0 solved 55 2 0 0 0
unbiased_022 - 26 solved 69 25 2 151 83466 solved 69 25 2 136 83466
unbiased_023 - 26 solved 66 26 7 770 399793 solved 66 26 8 453 465127
unbiased_024 - 25 solved 56 3 0 0 0 solved 56 3 0 0 0
unbiased_025 - 25 solved 56 2 0 0 0 solved 56 2 0 0 0
unbiased_026 - 26 solved 62 20 0 0 0 solved 62 20 0 0 0
unbiased_027 - 25 solved 56 3 0 0 0 solved 56 3 0 0 0
unbiased_028 - 26 solved 55 3 0 0 0 solved 55 3 0 0 0
unbiased_029 - 27 solved 71 27 10 263 357479 stalled 12 26 3 7 86617
unbiased_030 - 24 solved 57 3 0 0 0 solved 57 3 0 0 0
unbiased_031 - 26 solved 58 19 0 0 0 solved 58 19 0 0 0
unbiased_032 - 25 solved 64 14 0 0 0 solved 64 14 0 0 0
unbiased_033 - 27 stalled 18 27 5 13 90753 stalled 13 8 1 0 16837
unbiased_034 - 26 solved 72 25 1 187 82662 solved 72 25 1 170 82662
unbiased_035 - 25 solved 60 16 0 0 0 solved 60 16 0 0 0
unbiased_036 - 25 solved 62 25 1 149 58742 solved 62 25 1 149 58742
unbiased_037 - 27 solved 62 19 0 0 0 solved 62 19 0 0 0
unbiased_038 - 25 solved 58 16 0 0 0 solved 58 16 0 0 0
unbiased_039 - 24 solved 66 19 0 0 0 solved 66 19 0 0 0
unbiased_040 - 27 solved 54 2 0 0 0 solved 54 2 0 0 0
unbiased_041 - 26 solved 61 25 2 250 77065 solved 61 25 2 249 77065
unbiased_042 - 25 solved 59 5 0 0 0 solved 59 5 0 0 0
unbiased_043 - 26 solved 59 10 0 0 0 solved 59 10 0 0 0
unbiased_044 - 25 solved 91 27 16 1271 1122929 stalled 14 26 3 7 83079
unbiased_045 - 25 solved 56 3 0 0 0 solved 56 3 0 0 0
unbiased_046 - 26 solved 58 18 0 0 0 solved 58 18 0 0 0
unbiased_047 - 24 solved 69 27 1 14 21665 stalled 20 19 1 0 21665
unbiased_048 - 26 solved 55 2 0 0 0 solved 55 2 0 0 0
unbiased_049 - 26 solved 55 2 0 0 0 solved 55 2 0 0 0
unbiased_050 - 26 solved 64 19 0 0 0 solved 64 19 0 0 0
top1465_001 - 18 stalled 31 28 6 9 111697 stalled 27 28 2 0 35129
top1465_002 - 18 stalled 8 27 2 1 21129 stalled 7 8 1 0 10654
top1465_003 - 18 stalled 8 27 2 1 21327 stalled 7 8 1 0 10753
top1465_004 - 18 solved 84 8 0 0 0 solved 84 8 0 0 0
top1465_005 - 18 solved 75 8 0 0 0 solved 75 8 0 0 0
top1465_006 - 18 solved 75 8 0 0 0 solved 75 8 0 0 0
top1465_007 - 18 stalled 8 27 2 1 24353 stalled 7 8 1 0 12264
top1465_008 - 18 solved 87 8 0 0 0 solved 87 8 0 0 0
top1465_009 - 18 solved 85 8 0 0 0 solved 85 8 0 0 0
top1465_010 - 18 solved 87 8 0 0 0 solved 87 8 0 0 0
top1465_011 - 18 solved 83 10 0 0 0 solved 83 10 0 0 0
top1465_012 - 18 solved 82 27 3 270 123257 stalled 21 19 1 0 16955
top1465_013 - 18 solved 87 27 2 29 38423 stalled 29 28 3 2 57074
top1465_014 - 18 solved 85 9 0 0 0 solved 85 9 0 0 0
top1465_015 - 18 solved 73 7 0 0 0 solved 73 7 0 0 0
top1465_016 - 18 solved 82 27 1 2 16670 stalled 24 19 1 0 16670
top1465_017 - 18 solved 86 8 0 0 0 solved 86 8 0 0 0
top1465_018 - 18 solved 82 8 0 0 0 solved 82 8 0 0 0
top1465_019 - 18 solved 86 8 0 0 0 solved 86 8 0 0 0
top1465_020 - 18 solved 83 8 0 0 0 solved 83 8 0 0 0
top1465_021 - 18 solved 94 27 15 437 398473 stalled 16 20 1 0 13237
top1465_022 - 18 solved 83 8 0 0 0 solved 83 8 0 0 0
top1465_023 - 18 stalled 21 27 4 6 60950 stalled 18 8 1 0 15576
top1465_024 - 18 stalled 11 27 4 4 55246 stalled 8 20 1 0 13840
top1465_025 - 18 solved 88 27 3 41 59201 stalled 30 28 4 4 77836
top1465_026 - 18 solved 85 8 0 0 0 solved 85 8 0 0 0
top1465_027 - 18 solved 85 8 0 0 0 solved 85 8 0 0 0
top1465_028 - 18 solved 85 8 0 0 0 solved 85 8 0 0 0
top1465_029 - 18 stalled 20 27 3 3 46802 stalled 18 8 1 0 15887
top1465_030 - 18 stalled 12 7 1 0 17332 stalled 12 7 1 0 17332
top1465_031 - 18 solved 83 8 0 0 0 solved 83 8 0 0 0
top1465_032 - 18 solved 74 10 0 0 0 solved 74 10 0 0 0
top1465_033 - 18 solved 77 8 0 0 0 solved 77 8 0 0 0
top1465_034 - 18 solved 90 27 5 44 96939 stalled 29 28 4 4 76576
top1465_035 - 18 stalled 24 27 3 3 50618 stalled 21 20 1 0 16816
top1465_036 - 18 stalled 22 27 4 14 66927 stalled 20 26 2 11 31941
top1465_037 - 18 solved 84 9 0 0 0 solved 84 9 0 0 0
top1465_038 - 18 stalled 32 27 7 30 123304 stalled 23 19 1 0 17473
top1465_039 - 18 stalled 31 27 4 6 62710 stalled 26 26 2 2 30322
top1465_040 - 18 solved 84 8 0 0 0 solved 84 8 0 0 0
top1465_041 - 18 solved 75 26 2 203 63698 solved 75 26 2 139 63698
top1465_042 - 18 solved 91 27 12 611 395197 stalled 18 26 8 32 222479
top1465_043 - 18 stalled 26 26 2 2 28962 stalled 26 26 2 2 28962
top1465_044 - 18 solved 84 18 0 0 0 solved 84 18 0 0 0
top1465_045 - 18 solved 86 25 1 172 90887 solved 86 25 1 138 90887
top1465_046 - 18 stalled 19 27 3 3 41943 stalled 17 8 1 0 14174
top1465_047 - 18 solved 84 18 0 0 0 solved 84 18 0 0 0
top1465_048 - 18 stalled 24 27 2 1 34065 stalled 23 8 1 0 17137
top1465_049 - 18 solved 87 26 3 80 72029 solved 87 26 3 76 72029
top1465_050 - 18 solved 85 8 0 0 0 solved 85 8 0 0 0
norvig_hardest_01 - 22 solved 66 5 0 0 0 solved 66 5 0 0 0
norvig_hardest_02 - 23 stalled 4 20 1 0 15395 stalled 4 20 1 0 15395
norvig_hardest_03 - 26 solved 70 27 6 174 161057 stalled 24 26 5 70 128172
norvig_hardest_04 - 24 solved 62 5 0 0 0 solved 62 5 0 0 0
norvig_hardest_05 - 22 solved 59 3 0 0 0 solved 59 3 0 0 0
norvig_hardest_06 - 23 stalled 1 2 1 0 9306 stalled 1 2 1 0 9306
norvig_hardest_07 - 28 solved 69 18 0 0 0 solved 69 18 0 0 0
norvig_hardest_08 - 28 solved 65 26 2 174 78533 solved 65 26 2 128 78533
norvig_hardest_09 - 24 solved 72 14 0 0 0 solved 72 14 0 0 0
norvig_hardest_10 - 28 solved 70 26 5 503 247246 solved 69 26 5 362 246556
norvig_hardest_11 - 22 solved 86 27 16 680 658443 stalled 18 26 7 140 255821
inkala_2012 8..........36......7..9.2...5...7.......457.....1...3...1....68..85...1..9....4.. 21 stalled 0 0 1 0 9598 stalled 0 0 1 0 9598
sudokuwiki_unsolvable_28 6....894.9....61...7..4....2..61..........2...89..2.......6...5.......3.8....16.. 22 stalled 2 5 1 0 11347 stalled 2 5 1 0 11347
sudokuwiki_unsolvable_49 ..28......3..6...71......4.6...9.....5.6....9....57.6....3..1...7...6..84......2. 22 stalled 0 0 1 0 8290 stalled 0 0 1 0 8290
```

**Where the scratch stayed.** Everything was made under the gitignored `ai_tmp/s11/` of
the worktree `ticket/S11-chain-techniques` and is part of no commit: the two programs
(`proto/`, `indep/`), the SAT oracle (`sat/`, with `python-sat` in a throwaway
environment), the runs (`runs/`, `decided/runs/`), the full per-grid table with every
givens string (`decided/per_grid.tsv`), the hand states (`inputs/`), and the evidence
pages (the definitions with their SHA-256
`dcd25dfdd8578f53649e830f8915bc29dcab0b0f7dffc43bb7a9e9819cd0b328`, their errata, the
results, and the report of the decided run whose seventeen tables "Handed back" cites
as run tables 1 to 17). **Nothing in these notes depends on it**: a rebuild is made
from this part, part 3 and S06's notes, and is right when it reproduces the table above
and the totals under "The counters".

### Open points settled

Step 8, 2026-10-03. Each decision was put to the maintainer with its evidence and its
recommendation; none was deferred and none was taken by this ticket's agent.

- **D1.** A chain of ranks 24 to 27 may pass through a candidate twice: it is a walk (`D1=walk`), which is the recommendation. Answered by the maintainer, 2026-10-03.
- **D2.** The minimums count distinct things, an `x_chain` at least four distinct candidates and an `xy_chain` at least three distinct cells (`D2=distinct`), which is the recommendation; "four" was in the option as put, and its alternative, three distinct candidates, was not weighed (Q3 of the proposed text). Answered by the maintainer, 2026-10-03.
- **D3.** A `remote_pair` is one deduction for the whole two-coloured component, as `simple_colouring` has it, with adjacent pair cells counted (`D3=component`, `adjacent_yes`), which is the recommendation. Answered by the maintainer, 2026-10-03.
- **D4.** An entry holds what the specification writes, an endpoint strike every target of its endpoints and a closure or a forcing chain its one candidate, with closures listed beside endpoint strikes (`D4=as_written`, `closures_all`), which is the recommendation. Answered by the maintainer, 2026-10-03.
- **D5.** A chain is held link by link and the chain of fewest links is kept, so that a single chain reads a load of 3 whatever its length and a forcing chain its paths plus 2 (`D5=link_by_link`), which is the recommendation made at step 7, the ticket having started undecided. Answered by the maintainer, 2026-10-03.
- **D6.** For `forcing_chain`, a root does not reach itself by an empty path (`D6a=root_no`), a path is counted for each path the proof holds, which is each incompatible literal's path and not one for each root (`D6b=per_literal`), and the witness kept is the one of fewest links in all (`D6c=shortest_each`), with five kinds of contradiction, rank 27 listing all it proves and a contradiction kept before a verity on a tie, as built: all three parts are the recommendation. Answered by the maintainer, 2026-10-03.
- **D7.** `max_chain_links` becomes 16 and `max_forcing_paths` stays 9, which departs from the recommendation (stay at 24 and 9): the maintainer chose 16 after being told that with link by link the link limit is the only bound on how long a chain a player is shown, that 16 cuts 27 of 5,477 reachable candidates on 225 states with the same grids solving under the baseline, and that the range `0..24` of `reasoning_depth` in `human-solving.allium` must then change; that last overstated it, since narrowing the range is one of two ways to mend the text, the other being to reword `technique.allium` 527-528 alone, and which is taken is asked in the specification change ("Handed back", 1.12 (a)). Answered by the maintainer, 2026-10-03.
- **D8.** For ranks 13 to 22, "other cells" exclude only the two named cells (`D8=named_cells`), which is the recommendation. Answered by the maintainer, 2026-10-03.
- **D9.** A colouring's wrap and trap are separate deductions, each with the whole component as its witness (`D9=separate`), which is the recommendation. Answered by the maintainer, 2026-10-03.

The ticket's three open points:

The ladder's order past rank 12 is left, and is not settled with these decisions. It
is a non-goal of this ticket and the maintainer was not asked. What the evidence says
is under "Recorded, not settled": the order turns on the load reading (D5) and the
link limit (D7), both now answered. The figures recorded for it were made under the
baseline at 24 links, so by inference they are not figures for the decided set.

D8 and D9 were kept here and answered with the rest, so that one specification change
states every reading. They were not taken out to the specification change S06 drafted.

The run at capacity 4 was made, by the maintainer's answer of 2026-10-03: step 5 ran
the baseline and the keys of D1 to D6, D8 and D9 at capacity 4 as well as at 8 and
at 729.

Earlier answers of the maintainer, all on 2026-10-03, as the session recorded them (the
scratch evidence pages name no authoriser, so these rest on the session's record alone):
the rebuild of step 4 was to be made blind, from S06's hand-back notes alone, and
then checked against S06's scratch; fetching published work from the web for step 3
was authorised; copying S06's grids from S06's worktree was authorised; installing
`python-sat` with `uv` into a throwaway environment under `ai_tmp/` was authorised;
fetching S06's grids again from their sources was not authorised and was not done;
the nine decisions were to be put in one sitting; and an adversarial review by Codex
was to be asked for at the end.

What the maintainer was not asked, and which therefore stays open. None is decided by
anyone here. Each has an open question in the proposed specification text ("Handed
back", 1.11, Q1 to Q9), or is moot under the answers, as said:

- whether D5's reading (link by link) reaches ranks 21, 22 and 29, whose loads at
  capacity 8 and 4 are still the rebuild's readings of the whole count (definition
  choice 18): Q1; it also bears on `effort.allium:314`, whose proposed text asks it;
- which of a colouring's trap and wrap is listed first (as built the trap; HoDoKu
  looks for the wrap first): Q2;
- definition choice 7: "four" distinct candidates for `x_chain` was in D2's option as
  put and accepted; three distinct candidates, the alternative, was never weighed: Q3;
- definition choices 12 and 13 (the component's size, what the limit bounds there):
  Q4; 16 (fewest links in all against the shortest longest path): Q5; 30 (a root
  walking out and back): Q7; 29 (a struck target not in the load): Q8; 25 (a target
  that is a node of its own chain): Q9;
- definition choices 5, 6, 26 and 32 have no question because the answers leave them
  nothing to decide: 5 and 6 define the no-repeat keys of D1, and D1 is a walk; 26
  defines `closures_residual`, and D4 lists every closure; 32 defines
  `D9=per_target`, and D9 is `separate`;
- two definition choices were stated inside an option and accepted with it: 9's
  formula (in D5's option) and 28's tie order (in D6's option). The Codex review of
  2026-10-04 reopened a part of each. The answer on D5 reads "paths plus 2" and the
  formula has a `min` that differs from it on a proof of one path and on one of three
  paths over four candidates: Q7 asks which the text carries. And 28 orders a
  contradiction before a verity only; the order among contradictions of equal links
  (the kind, then the anchor, the count of paths never) was not in the option: Q5;
- which of two components with one output a `remote_pair` entry shows (as built the
  fewer cells), and what its `Proof` holds, neither of which D3 asked: Q4;
- `D3l`, whether a remote pair's load counts both digits of a pair cell or one: no
  question, because under D5's answer the load read is 3 either way and nothing
  turns on it; no decision covers it;
- that `alternating_inference_chain` has no minimum in the text, so rank 26 lists a
  chain of one strong link; D2 as the ticket put it does not ask: Q6;
- the ladder's order past rank 12: Q1;
- calibration of any profile: no new question; the standing one is
  `docs/specs/human-solving.allium` 1272, which these decisions neither answer nor
  change, and Q1 keeps the loads provisional.

## Open points

- Whether the ladder's order past rank 12 should be settled with these decisions or
  after them; `technique.allium` holds both in one open question.
- Whether D8 and D9 belong here or in the specification change S06 drafted; they are
  here so that one change states every reading, and may be taken out.
- Whether the profile of step 5 should also be run at a capacity of 4, the expert
  preset's, so that what the decisions hide from the preset is measured and not
  only argued.
