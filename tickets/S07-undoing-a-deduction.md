---
id: S07
title: "Spike: undoing a deduction, technique by technique"
status: done
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
  (`ADeductionPlacesOrStrikes`), and it is the deduction's output that decides, not
  its technique. The four singles (ranks 1 to 4) and `bug_plus_one` always place. A
  forcing chain places on a positive conclusion and strikes on a negative one. An
  alternating inference chain places only on a closure from a negative start, and
  strikes on a closure from a positive start and on every other path. Every other
  technique strikes. To undo a placement is to take a digit out of a cell. To undo a
  strike is to give a cell a candidate back, which changes no digit.
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
  that have no verdict yet can promise it only on condition: the verdict of the
  givens where removal ends must be one, which acceptance asks for anyway. With the
  premise false, neither technique is licensed, so neither has anything to undo.
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
| 1 | `full_unit` | places | Yes under the broad reading wherever a unit of the cell is otherwise full; strict only if it is also the least rank-1 placement on the earlier grid (the same answer on every member). | Takes the digit out; the cell keeps it as its one candidate, since nothing else is allowed. | Which open peers get the digit back: any subset of P(c). | Holds on every member of the family; the same members license ranks 2, 3 and 4 as well. |
| 2 | `cross_hatch` | places | Yes, where some unit of the cell has no other cell open to the digit by sight. | Takes the digit out. Licensed on every member: it reads digits alone. | The cell's candidates (any set with d within A(c)) and the peers given d back (any subset of P(c)). | Also licensed where the cell's marks lack d; those grids are not true for S and are outside the family. |
| 3 | `naked_single` | places | Yes; always under the broad reading. | Takes the digit out and leaves it the cell's only candidate. | Which peers get the digit back: any subset of P(c). | Canonical only when A(c) = {d}; otherwise every other allowed digit of the cell becomes a strike left standing. |
| 4 | `hidden_single` | places | Yes; always under the broad reading. | Takes the digit out; no peer in one chosen unit of the cell gets it back. | The cell's candidates, and the peers outside that unit given d back. | On a canonical member it coincides with `cross_hatch`, so for a systematic profile whose repertoire holds `cross_hatch` (profiles A and B) it is never strict there and never a step of a run before its first strike; such a run's hardest rank is never exactly 4. |
| 5 | `pointing` | strikes | Yes, where L has standing strikes: box's places of d on one line, no d on the line outside the box. | Gives d back to a nonempty set of the line's cells outside the box. | Which subset: each nonempty subset of M's members there is its own earlier grid. | Giving back cannot break the pattern. Nothing to undo on a born-kept grid. The rest of M stays standing. |
| 6 | `claiming` | strikes | Yes, where the line's places of d lie in one box and no box cell off the line allows d in L. | Gives d back to a nonempty set of the box's cells off the line. | Which subset of M's members there. | Giving back cannot break the pattern. Nothing to undo on a born-kept grid. |
| 7 | `naked_pair` | strikes | Yes, where two cells of a unit hold two digits and no other cell of the unit allows either in L. | Gives the pair's digits back to other open cells of the unit. | Which subset of M's members among those cells and digits. | Giving back cannot break the pattern: it never touches the pair's cells. |
| 8 | `hidden_pair` | strikes | Yes, where two digits have all their places in two cells of a unit that allow nothing else in L. | Gives other digits back to those two cells. | Which subset of M's members at those cells. | Giving back cannot break the pattern: it reads only where the two digits stand. |
| 9 | `naked_triple` | strikes | Yes, as `naked_pair` with three cells and three digits. | Gives the triple's digits back to other open cells of the unit. | Which subset of M's members there. | A triple holding a pair stays licensed; under the strict reading the pair comes first where it yields anything. |
| 10 | `hidden_triple` | strikes | Yes, as `hidden_pair` with three digits and three cells. | Gives other digits back to those three cells. | Which subset of M's members at those cells. | Cannot be broken by giving back. Strict needs every lower rank to yield nothing on the earlier grid. |
| 11 | `naked_quad` | strikes | Yes, as `naked_pair` with four cells and four digits. | Gives the quad's digits back to other open cells of the unit. | Which subset of M's members there. | Cannot be broken by giving back. Often the complement of a smaller hidden subset, which strict takes first. |
| 12 | `hidden_quad` | strikes | Yes, as `hidden_pair` with four digits and four cells. | Gives other digits back to those four cells. | Which subset of M's members at those cells. | Cannot be broken by giving back. Last rank of profile A and of S06's normative order. |
| 13 | `x_wing` | strikes | Yes, where the two bases hold d only in the two covers and no cover cell outside the bases allows d in L. | Gives d back to cover cells outside the bases. | Which subset of M's members there. | Cannot be broken: targets lie outside every base. Not in profile A; measured under profile B. |
| 14 | `swordfish` | strikes | Yes, as `x_wing` with three bases and three covers. | Gives d back to cover cells outside the bases. | Which subset of M's members there. | Cannot be broken by giving back. Reducible fish stay licensed; ladder order is separate. |
| 15 | `jellyfish` | strikes | Yes, as `x_wing` with four bases and four covers. | Gives d back to cover cells outside the bases. | Which subset of M's members there. | Cannot be broken by giving back. Measured under profile B. |
| 16 | `xy_wing` | strikes | Yes, where pivot and wings hold exactly their pairs and no other cell seeing both wings allows c in L. | Gives c back to cells outside the three that see both wings. | Which subset of M's members there. | "Other cells" is S06's reading (D8); for this technique the other reading undoes the same things. |
| 17 | `xyz_wing` | strikes | Yes, as `xy_wing` with a pivot of three; targets see all three cells. | Gives c back to outside cells seeing pivot and both wings. | Which subset of M's members there. | Cannot be broken under S06's reading of "other cells" (D8). |
| 18 | `w_wing` | strikes | Yes, where P and Q hold exactly {a, b}, joined by a conjugate pair for a. | Gives b back to other cells seeing both P and Q. | Which subset of M's members there. | Cannot be broken: b's return changes no place of a. D8 decides whether the conjugate pair's cells can be targets. |
| 19 | `skyscraper` | strikes | Yes, but (S06's reading of "other cells") only at targets off the two base lines. | Gives d back to cells seeing both unaligned endpoints. | Which subset of M's members among those targets. | Giving back can break it: d on a base line is a third place there. On S06's reading of "other cells" (D8) such cells are never in its output; on the other, an aligned endpoint seeing both unaligned ones is a target, is in every output of that witness and is given back as part of the pattern. |
| 20 | `two_string_kite` | strikes | Yes, but (S06's reading of "other cells") only at targets off the kite's row and column. | Gives d back to cells seeing both free endpoints. | Which subset of M's members among those targets. | Giving back can break it: d on the row or column ends its conjugate pair. S06's reading of "other cells" (D8); on the other, a joined endpoint seeing both free ones is a target and is in every output of that witness. |
| 21 | `empty_rectangle` | strikes | Yes, where the one target is in M and the box and conjugate line hold on L. | Gives d back to the one target cell. | Nothing: one candidate, so one earlier grid for each target, whichever witness accounts for it. | Cannot be broken: the target is outside the box and off the conjugate line. Two-place variant is deferred. |
| 22 | `simple_colouring` | strikes | Yes. Trap: outside cells seeing both labels. Wrap: the whole false label. | Trap gives d back to outside cells; wrap gives d back to every node of one label. | Trap: which subset, among those admissible jointly on the earlier grid (not closed under taking subsets). Wrap: nothing, the label is undone whole. | Giving back can dissolve or make links, so the component is the earlier grid's. Wrap and trap separate is S06's reading (D9). |
| 23 | `remote_pair` | strikes | Provisional: yes, where the chain's cells hold exactly {a, b} and no target stands in L. | Gives a, b or both back to outside cells seeing two chain cells an odd number of links apart. | Which subset of M's members among the targets. | Provisional. Reading: one deduction for each chain, as written. Waits on D3 (output and witness), D5 (its load is variable, and seeing reads it) and D7. |
| 24 | `x_chain` | strikes | Provisional: yes, at targets whose return leaves every conjugate link of the chain standing. | Gives d back to cells seeing both endpoints. | Which subset of the targets, every target that is an interior node of the chain included; which chain is the witness. | Provisional. Reading: a chain may repeat a candidate, entry as written. Waits on D1, D2, D4, D5, D7. Giving back can break links. |
| 25 | `xy_chain` | strikes | Provisional: yes; the targets are the endpoint digit at cells seeing both endpoint cells, chain cells among them. | Gives the endpoint digit back to cells seeing both endpoint cells. | Which subset of the targets, every target that is a node of the chain included; which chain is the witness. | Provisional. Reading as rank 24. Waits on D1, D2, D4, D5, D7. A target that is a node of the chain must be given back: it supplies the bivalue cell. A digit foreign to a chain cell is never in an output. |
| 26 | `alternating_inference_chain` | strikes; places on a closure from a negative start | Provisional: yes in both forms, wherever the path is valid on the earlier grid. | Strike: gives targets or the closure's one candidate back. Placement: takes the digit out. | Placement: C and G, within "c bivalue or d two-placed in a unit of c". Strike: the subset, every interior-node target included. | Provisional. Reading: repeats allowed, entry as written. Waits on D1, D4, D5, D7. Strict not measured: S06's order by rank and output reaches this rank, but the catalogue it orders waits on S11. |
| 27 | `forcing_chain` | places on a positive conclusion; strikes on a negative one | Provisional: yes, wherever every path is valid on the earlier grid. | Placement: takes the digit out. Strike: gives the one denied candidate back. | Placement: C and G, which also set the roots. Strike: nothing, one candidate. | Provisional. Reading, a pair: no candidate twice (D1) and no empty path (D6), in the verity and the contradiction form alike; with an empty path it would license every single, and under a walk it licenses a single wherever (c, d) has a strong link. That pair shuts out only those trivial proofs: a two-link path through another cell's single can still place what a single places. Waits on D1, D4, D5, D6, D7. Strict not measured. |
| 28 | `unique_rectangle_type_1` | strikes | Yes, on condition: only with `uniqueness_promised`. Where the later grid admits S alone, it keeps every solution exactly when the earlier grid passes check 2 (its digits having verdict one is enough); it is promised only if the verdict where the walk ends is one. For a born-kept end the two agree. | Gives both a and b back to the fourth corner, together. | Nothing: both digits, one cell. | With verdict many the step is still correct for S but the givens are not a puzzle and a solved run proves nothing. |
| 29 | `bug_plus_one` | places | Yes, on the same condition, and on at most one member of the family. | Takes the digit out of T, leaving three candidates, and gives it back to exactly two peers in each unit of T. | Nothing: C and G are forced by L's counts. | Not seen by profile A, nor by any profile whose capacity is below the BUG proof's load (the expert preset's 4): then not even broad. No single is licensed on such a grid. The condition is on the earlier grid, which has one digit fewer. Worth exactly the verdict, as rank 28. |

The letters in the table (L, S, C, G, A(c), P(c), M) are defined under "The frame" in
the hand-back notes, and the D-numbers are the decisions of
`tickets/S11-chain-techniques.md`. The verdict reached is in step 7 of the notes.

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

2. Verify, read-only, and record with sources and dates: (a) what has been published on
   making a puzzle by undoing deductions or by aiming at a named technique; (b) for each
   placing technique, which candidates the emptied cell and its peers may hold so that
   the earlier grid still licenses the placement; (c) for each striking technique, which
   candidates may be given back so that the earlier grid still licenses the strike, and
   whether the several strikes of one deduction are undone together or one by one; (d)
   whether either uniqueness technique can be undone under a `uniqueness_promised` given
   on condition that the verdict where removal ends is one, and what an undoing of it is
   worth when that verdict turns out many; (e) what undoing a placement does to the
   candidates upkeep struck, with upkeep pinned as `reach.allium`'s `KeepMarksTrue`
   states it. Fetching a source is separately authorised: stop and ask.

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

5. Check every undoing found, by the prototype's own search or by the SAT or SMT solver
   S06 weighed: the earlier grid meets `DeductionsAreSound`'s premise for the solution
   grid (every placed digit is the solution's, every open cell allows the solution's
   digit), its digits and candidates together admit that solution alone, the deduction
   undone is on its list, and taking that deduction, with upkeep after a placement as
   `KeepMarksTrue` states it (every candidate a placed peer rules out struck in every
   unit), gives exactly the later grid. The second condition is a cross-check on the
   prototype, not a filter: `DeductionsAreSound` holds for any solution meeting its
   premise, so a licensed deduction outside the two uniqueness techniques keeps every
   solution of the earlier grid, and an earlier grid of a later grid with one solution
   has that one alone. A failure there is a defect in the prototype, except for
   `unique_rectangle_type_1` and `bug_plus_one`, where it is question (d). Whether the
   digits alone are well-posed is a verdict on givens and S08's to measure.

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

Run on 2026-10-03 (PDT) in the worktree on `ticket/S07-undoing-a-deduction`, from `main`
at `0682b06`, after S06 had merged (#35). No tracked file changed except this one and
`tickets/README.md`. Every program, download and output went under the gitignored
`ai_tmp/s07/` of this worktree and goes no further; what a rebuild needs is in these
notes.

The maintainer authorised on the day, for this ticket: reading S06's already-fetched
grid file and running S06's scratch SAT venv (`python-sat` 1.9.dev15 on the pixi Python
3.14.7), both left in S06's worktree, with no fresh fetch of the grids and no install;
read-only web searches and fetches for 2(a); measuring ranks 13 to 22, 28 and 29 beside
ranks 1 to 12; and keeping this ticket's words while flagging that `board.allium` owns
"undo". Nothing was installed.

```text
$ git rev-parse --short HEAD
0682b06
$ git status --porcelain
$
```

The definitions below were fixed in one scratch file before any program existed.
Agents then worked apart: two rebuilt S06's prototypes from S06's text, forbidden to
read S06's scratch; one rebuilt the SAT oracle; three searched the literature and a
fourth fetched every cited page again; one drafted the paper analysis, three tried to
refute it (21 findings, all upheld and fixed); one built the undoing prototype; one,
forbidden to read it, wrote a brute-force search; a referee compared the two; an
auditor computed the figures again from the raw outputs with its own code, except
those taken from the programs' own counters: the counts of earlier grids tried and
found, the member and family counts of (b), and the strike undone deductions seen in
walk M.

**The frame, used throughout and by the table in the Goal.** L is the later grid, E an
earlier grid, S the solution a walk starts from.

- A grid is *kept* when no open cell holds a candidate a placed peer holds. Only kept
  grids are later or earlier grids: a run looks only when `grid.is_kept` (`reach:138`),
  and `KeepMarksTrue` strikes every stale candidate (`reach:224`).
- allowed(x) is the digits no placed peer of the open cell x holds. A grid is *born
  kept* when every open cell holds exactly allowed(x) (`technique:739-741`).
- The *missing set* M(L) is every (x, d) with x open, d in allowed(x) and d not a
  candidate of x. Each member is a strike left standing. M is empty exactly when L is
  born kept.
- An *undoing* at L is a kept grid E and a deduction on E's catalogue that the profile
  sees, such that taking it on E, with upkeep after a placement, gives exactly L. It is
  named by the deduction's technique and output; two with the same technique and
  output are one *undone deduction*, whatever E and whatever witness. *Broad*: as
  stated. *Strict*: the deduction's entry is also the first of S06's list on E.
- *Earlier grids of a placement.* c is a placed cell of L holding d. A(c) is the digits
  no placed peer of c holds once c is emptied. P(c) is the open peers of c that see no
  other placed d. The *family* F(L, c) is every E(C, G) with d in C, C within A(c) and
  G within P(c): L with c open and holding C, and d added to each cell of G. It has
  2^(|A(c)|-1) x 2^|P(c)| members, and among grids true for S these are exactly the
  kept grids from which placing d at c gives L. The *canonical* member has C = A(c) and
  G = P(c); if L is born kept, so is it. The *minimal* member has C = {d} and G empty.
- *Earlier grids of a strike.* For a nonempty O within M(L), E = L + O is L with O
  given back; O is undone by a technique when E's catalogue holds a deduction of it
  whose output is exactly O.
- Profiles are S06's: A (ranks 1 to 12, capacity 4) and B (ranks 1 to 22, 28, 29,
  capacity 729), both `full_marks`, every extent, `systematic`, `unfixed`,
  `uniqueness_promised` true.

**Step 2. The five questions.**

**(a) Nothing found makes a puzzle by undoing deductions; every published way of aiming
at a named technique is generate-and-test, and one paper constructs givens for a set of
techniques being sufficient.**

| Source | Kind of method | What it shows |
| --- | --- | --- |
| Nishikawa and Toda 2020 | construction | Fixes the positions of the givens and a set of techniques (naked singles, hidden singles, locked candidates), encodes the forward run as a constraint problem and solves for the digits; says earlier work is all generate-and-test; runs pass its 600-second limit as the givens near 20 |
| Simonis 2005, section 8 | generate-and-test | Removes values from a full grid while the puzzle stays well posed or search-free for a chosen propagation scheme |
| Berthier 2011 | generate-and-test | Names the two classical generators, bottom-up and top-down; both add or delete givens, and techniques only rate the output |
| Stuart, Sudoku Creation and Grading | generate-and-test | Fill, subtract with a uniqueness test, grade afterwards, reject misfits |
| HoDoKu manual, chapter 4, and `BackgroundGenerator.java` | generate-and-test | Promises a puzzle holding "at least one instance" of a chosen technique; the source generates, solves and retries up to 20,000 times until a step of that technique is in the path |
| QQWing `qqwing.cpp` | generate-and-test | Solves an empty grid, rolls back every cell its solver filled by logic and keeps the guesses, then removes givens while the solution stays unique; wholesale, and no technique is named |
| l2sg `Generator.cpp`, SukakuExplainer `Generator.java`, Isanaki | generate-and-test | Remove or restore givens, or regenerate, until the wanted technique is in the solving path or the rating fits |
| `sudoku-variants` crate | generate-and-test | Removes givens while a supplied solver still solves: a technique set sufficient, none necessary |
| Snyder, GM Puzzles 2025 | construction by hand | Setters add givens toward a solve; no procedure |
| Sudopedia, Sukaku | none | Candidates as the unit of a puzzle, which is the state an undone strike produces; no way of generating |

"Backward" and "reverse" in the sources mean starting from the full grid and removing
givens. Looked for and not found: any paper, tool or setter's account that takes a
solved or part-solved grid and inverts a named deduction; any method that guarantees a
named technique is required, as against present in one path or sufficient as a set;
any constraint encoding past singles and locked candidates; how the forum's lists of
puzzles needing one named pattern were made. Absence here is absence in what was
searched, mostly English and open access; Zama and Sasano 2016 is known only through
Nishikawa and Toda.

Sources, all fetched 2026-10-03, each fetched a second time by the verifying agent:

- `https://arxiv.org/html/2005.14098` (paper): Nishikawa and Toda, Exact Method for
  Generating Strategy-Solvable Sudoku Clues.
- `https://ai.dmi.unibas.ch/_files/teaching/fs13/ki/material/ki10-sudoku-inference.pdf`
  (paper): Simonis, Sudoku as a Constraint Problem. "We start with a full grid and
  remove values until the problem is no longer well posed or no longer search free for
  some propagation scheme."
- `https://arxiv.org/pdf/1111.4083` (paper): Berthier, Unbiased Statistics of a CSP.
- `https://www.sudokuwiki.org/Sudoku_Creation_and_Grading.pdf` (paper): Stuart. "Given
  a filled board I then start subtracting numbers to make the puzzle."
- `https://hodoku.sourceforge.net/en/docs_cre.php` (docs; refused a direct download and
  was read through the fetch tool's extractor only) and
  `https://raw.githubusercontent.com/PseudoFish/Hodoku/master/src/generator/BackgroundGenerator.java`
  (source; `MAX_TRIES = 20000`).
- `https://raw.githubusercontent.com/stephenostermiller/qqwing/main/src/cpp/qqwing.cpp`
  (source), lines 356 to 359.
- `https://raw.githubusercontent.com/rafaelfassi/l2sg/HEAD/lib/src/Generator.cpp`
  (source), `https://raw.githubusercontent.com/SudokuMonster/SukakuExplainer/HEAD/diuf/sudoku/generator/Generator.java`
  (source) and `https://tgmdev.be/applications/isanaki/isanaki_generator.php` (docs).
- `https://docs.rs/sudoku-variants` (docs).
- `https://www.gmpuzzles.com/blog/?p=33500` (docs): Snyder, 25 July 2025. "Humans
  construct by adding givens, like in a house of cards".
- `https://sudopedia.sudocue.net/index.php?title=Sukaku` (docs).
- `https://www.sudokuwiki.org/Tridagons` (docs) and, from Wayback Machine snapshots,
  two threads of the New Sudoku Players' Forum (forum): "Reverse sudoku generator?"
  (2006), which asks for removal from a filled grid, and page 1 of 9 of "The tridagon
  rule" (2022); neither says how a pattern-specific list was made.

The branches named `master`, `main` and `HEAD` were read as they stood that day.

**(b) Which earlier grids license each placement.** Each line was argued from the
predicates and then counted against the listing, in every family the round trip
searched whole (its per-step searches and its enumeration together): over 7,209,694
family members under profile A and 7,430,637 under B, in 60,073 and 67,042 families,
there was no exception.

- `full_unit` and `cross_hatch` read digits alone (`technique:360`, `366-369`), so each
  holds on every member of a family or on none. `full_unit` holds where a unit of c is
  otherwise full in L; `cross_hatch` where some unit u of c has no cell of P(c). The
  emptied cell may hold any C; any peers of P(c) may get d back.
- `naked_single` holds exactly on the members with C = {d} (`technique:508`); any G.
  The minimal member always licenses it, so it is always undone under the broad
  reading. It is the canonical member's technique only when A(c) = {d}.
- `hidden_single` in unit u holds exactly on the members with no cell of G in u
  (`technique:373-376`); any C. The minimal member always licenses it. On a born-kept
  grid `hides_single` and `cross_hatches` are one predicate, since a candidate there
  is exactly what is open by sight, so on the canonical member it holds exactly where
  `cross_hatch` does, and rank 2 comes first on the list.
- `bug_plus_one` holds on at most one member: G is forced to be L's open cells with one
  candidate, and C to three digits fixed by the counts (`technique:1036-1040`). A
  profile sees it only if its capacity holds the whole proof's load
  (`technique:912-913`); the expert preset's 4 does not, and then nothing is undone.
- The placing forms of `alternating_inference_chain` and `forcing_chain`: a necessary
  condition only, that c is bivalue on E or d has exactly two places in a unit of c on
  E (`technique:955-961`); past that the path must be valid on E, member by member.
  Provisional on S11's decisions.

**(c) Which candidates may be given back, and together or one by one.** A deduction's
strikes are given back together: one undoing is one deduction's whole output
(`technique:904-906`). But what the whole output is depends on the earlier grid,
because a strike "names only candidates that stand" (`technique:335-336`). So where a
witness's targets hold n members of M and none stands in L, every nonempty subset of
them that leaves the pattern standing is a whole output on its own earlier grid, and
the rest stay in M: the same witness can never account for them afterwards. Measured:
the subsets proposed this way equal the listing of every nonempty O wherever every O
was listed (160,404 subsets where M has 1 to 12 members, and 2,990,040 more where it
has 13 to 18), none missed and none extra.

- Giving back cannot break the pattern for `pointing`, `claiming`, the six subsets, the
  three fish, `xy_wing`, `xyz_wing`, `w_wing`, `empty_rectangle` and `remote_pair`: the
  targets lie outside what the pattern reads.
- It can for `skyscraper` and `two_string_kite` (a third place on a conjugate line),
  for `simple_colouring`'s trap (a link dissolved or made, so the component is the
  earlier grid's, and the admissible outputs are not closed under taking subsets) and
  for the endpoint strikes of ranks 24 to 26.
- The output is whole or nothing for `simple_colouring`'s wrap (the whole label),
  `unique_rectangle_type_1` (both digits at the fourth corner), `empty_rectangle` (one
  candidate) and the one-candidate strikes of ranks 26 and 27.
- A strike undoing changes no digit and only shrinks M. On a born-kept grid there is
  nothing to give back, so no strike can be undone there: 0 at every grid with M empty
  in every measurement.

**(d) Both uniqueness techniques can be undone on condition, and an undoing of either
is worth exactly the verdict.** With the premise supplied true each is undone as (b)
and (c) say; with it false neither is licensed. If the givens where the walk ends have
one solution the premise is true of every grid of the walk, since their digits contain
those givens. The set of solutions a walk's grid admits grows only at an undoing of
rank 28 or 29, so for a walk that ends born kept the end verdict is one exactly when
every such undoing's earlier grid admits S alone: decidable at the undoing by one
solver call, and a failure there is permanent. When the verdict is many, the step
undone is still correct for S, but the givens are not a puzzle, `DeductionsAreSound`
promises nothing for these two (`technique:878-880`), `reach` begins no rating
(`reach:177`), and a solved run is no evidence. Who keeps a walk's grid, and whether a
keeper may promise on condition, the specifications do not say ("supplied by the
keeper, never inferred", `technique:287-288`).

Measured: 30 distinct earlier grids of the two techniques were found (29 at the grids
of S06's runs under profile B, 1 in a walk). The 29 whose digits have one solution
admit S alone. The one whose digits have two or more does not: `seventeen_021` under
profile B, a `unique_rectangle_type_1` undone at 43 digits left on a grid with 13
strikes standing. From that earlier grid a run with the premise true takes the
uniqueness step and ends solved at S after 39 steps; with it false the run stalls with
14 cells open; from the born-kept grid of the same digits it stalls after 2 steps
either way. No run left S. That is one grid, not a sample. `bug_plus_one` undid nothing
at any grid of any walk from a solution.

**(e) Undoing a placement has to choose what the step discarded.** `TakeStep` leaves
the placed cell its digit as its one candidate, whatever it held (`reach:278-279`), and
`KeepMarksTrue` strikes every stale candidate (`reach:224`); on a kept grid that is d
at the open peers of c that held it. So an undoing chooses C and G, and

```text
M(E(C, G)) = M(L) + {(c, e): e in A(c), not in C} + {(x, d): x in P(c), not in G}
```

The canonical member leaves M as it was; every other member adds strikes left
standing; no placement undoing shrinks M. Measured over the 37,525 placement undoings
of walk M: M
unchanged in 25,557 (the canonical ones), larger in 11,968, smaller in none.
`Grid.start` is set when a grid is created and no rule writes it (`reach:204-206`,
`technique:478`), and a run "never undoes anything" (`reach:293-294`). So the grids of
a walk are not states of one `Grid`, and "the givens left" is reached only by opening
afresh, which gives all of M back at once with no deduction licensing any of it.

**Step 3. The two readings and what each guarantees.**

| Guarantee of the run from the grid the walk ends on | Broad | Strict |
| --- | --- | --- |
| It can start there | only if that grid is born kept: `OpenGrid` opens no other | the same |
| It ends solved | yes if every undoing was a single to the canonical member; otherwise not guaranteed | yes, from a born-kept end; where rank 28 or 29 was undone, only if the end's digits have one solution |
| It retraces the walk | no | yes, on the same conditions |
| Its hardest step is the hardest technique undone | no: at most the hardest undone for singles to canonical members, no bound otherwise | yes, on the same conditions |

- *Strict* retraces by induction: the first entry on each earlier grid is the deduction
  undone, and taking it gives the next grid. It leans on S06's order being what the run
  takes, which is normative for ranks 1 to 12 and measured to 22, 28 and 29. For ranks
  23 to 27 the order by rank and output is stated, but the catalogue it orders waits
  on S11, so the strict reading there is provisional and unmeasured.
- *Broad* guarantees only that some sequence of licensed deductions leads back. The run
  takes its own first entry and may leave the walk's path.
- *Singles survive more digits.* If a single of ranks 1 to 4 places d at c on a
  born-kept grid true for S, the same technique places it on any kept grid true for S
  with more digits where c is still open. So a walk that undoes only singles, each to
  the canonical member, stays born kept, and the run from any of its grids never
  stalls and ends solved, under either reading.
- *A run's hardest step is never rank 4* for a systematic profile that knows
  `cross_hatch`: a run stays born kept until its first strike, and there every
  `hidden_single` is also a `cross_hatch` in the same unit. A `hidden_single` is a step
  only after a strike.
- So singles to canonical members arrive only at givens whose run's hardest step is
  rank 1, 2 or 3. To aim at rank 5 or above a walk must leave strikes standing, and
  then either undo them back to a born-kept grid or open afresh over them, and neither
  is guaranteed to give a run that needs what was undone.

Both readings are counted in step 4.

**Step 4. The rebuild, the prototype and the counts.**

*The rebuild.* Both of S06's prototypes were written again from S06's step 3, step 4,
"Step 5, extended", "What S07 needs" and `technique:284-515`, `896-971`, `1018-1108`
alone. Every figure of S06's that was compared was reproduced, and no difference was
found. Not repeated: S06's own completeness checks (4,205 and 5,219 states), its
counts of sound steps and entries, and the figures of its other runs for (d).

- Ranks 1 to 12: all 226 per-grid rows, string for string; the worked look; 11,576
  looks, 11,499 steps, 149 solved, 77 stalled; the group table; the hardest-rank tally;
  every cell of the by-rank table; the per-look table. 309 checks agree and 0 differ.
- Ranks 13 to 22, 28 and 29: the 15 recorded rows; 12,585 looks, 12,532 steps, 173
  solved, 53 stalled; the group, by-rank and per-look tables; each grid's run begins
  with its run under ranks 1 to 12. 74 checks agree and 0 differ. All 474,257
  catalogue deductions agree with the grids' solutions.
- The SAT oracle, written again from S06's (e): all 226 grids have one solution, and
  the solutions equal a separate backtracking solver's on all 226.

Readings the rebuild had to take, none of which touched a figure: `full_unit` lists
nothing in a unit with one open cell and a repeated digit; a hidden subset's predicate
is taken literally; the 99th percentile is nearest-rank. S06's ticket gives no literal
strings for the study and derived grids, only their sources; the rebuild read all 226
from S06's scratch file, as authorised.

*The undoing prototype.* For each placed cell of L it builds members of the family and
lists each with the rebuilt lister: a technique undoes the placement, broad, when the
listing holds it, and strict when it is the first entry. A family of at most 4,096
members is searched whole. A larger one is tried on its *structured* members only: C
either A(c) or {d}, with G one of P(c), nothing, or P(c) outside c's row, column or
box; under profile B also the one `bug_plus_one` member. Strike outputs are proposed
from shapes (each member of M alone; every nonempty subset of what one witness strikes;
both digits at a rectangle's fourth corner; subsets of one digit's missing cells for
`simple_colouring`) and each is confirmed by listing L + O. At most 4,096 subsets are
kept for each witness, the whole set first and then by size; the rest are counted.

*Round trip.* Every step of the forward runs, 11,499 under A and 12,532 under B, with
L the grid after the step and E the grid before: E is among the earlier grids of L and
the step is found, broad and strict, at every one, with no failed check. So every
technique the runs use, through rank 22 and both uniqueness techniques, has strict
undoings at the grids where runs take it. Where the step is a placement, the run's own
earlier grid is the canonical member for 0 of the 256 `hidden_single` steps and for
505 of the 1,178 `naked_single` steps.

Undone deductions were enumerated in full at the later grid of every tenth look (1,254
grids under A, 1,359 under B; 6,328,881 and 6,607,729 earlier grids tried, 3,169,791
and 3,282,533 found):

| Technique | Later grids where undone, A | Undone broad, A | Of them strict, A | Later grids where undone, B | Undone broad, B | Of them strict, B |
| --- | --- | --- | --- | --- | --- | --- |
| `full_unit` | 732 | 30,597 | 22,831 | 825 | 34,881 | 26,376 |
| `cross_hatch` | 1,244 | 43,552 | 6,725 | 1,349 | 49,120 | 7,324 |
| `naked_single` | 1,254 | 55,676 | 3,563 | 1,359 | 61,851 | 3,872 |
| `hidden_single` | 1,254 | 55,676 | 2,839 | 1,359 | 61,851 | 3,097 |
| `pointing` | 273 | 5,698 | 2,361 | 316 | 6,246 | 2,713 |
| `claiming` | 279 | 5,230 | 259 | 327 | 5,843 | 332 |
| `naked_pair` | 208 | 15,505 | 408 | 271 | 17,916 | 567 |
| `hidden_pair` | 205 | 9,284 | 512 | 262 | 10,626 | 632 |
| `naked_triple` | 159 | 10,575 | 0 | 198 | 10,948 | 0 |
| `hidden_triple` | 124 | 8,589 | 100 | 156 | 9,439 | 115 |
| `naked_quad` | 116 | 10,293 | 31 | 151 | 11,824 | 31 |
| `hidden_quad` | 125 | 11,414 | 0 | 159 | 12,509 | 0 |
| `x_wing` | not seen | not seen | not seen | 181 | 2,612 | 7 |
| `swordfish` | not seen | not seen | not seen | 93 | 3,192 | 2 |
| `jellyfish` | not seen | not seen | not seen | 89 | 4,012 | 0 |
| `xy_wing` | not seen | not seen | not seen | 68 | 394 | 6 |
| `xyz_wing` | not seen | not seen | not seen | 74 | 447 | 3 |
| `w_wing` | not seen | not seen | not seen | 28 | 79 | 1 |
| `skyscraper` | not seen | not seen | not seen | 70 | 181 | 4 |
| `two_string_kite` | not seen | not seen | not seen | 77 | 108 | 4 |
| `empty_rectangle` | not seen | not seen | not seen | 11 | 12 | 0 |
| `simple_colouring` | not seen | not seen | not seen | 346 | 8,444 | 1 |
| `unique_rectangle_type_1` | not seen | not seen | not seen | 8 | 8 | 0 |
| all | 1,254 | 262,089 | 39,629 | 1,359 | 312,543 | 45,087 |

`naked_single` and `hidden_single` are undone, broad, at every placed cell of every
enumerated grid. The most undone at one grid is 4,543 broad (A) and 4,587 (B), and 215
strict. No strike is undone at any of the 946 and 976 enumerated grids with M empty.
Caps: 5,776 of 55,676 families under A and 5,867 of 61,851 under B were over 4,096
members and tried on structured members only, so their strict answers are found, not
complete; the strike cap dropped 4,095 `simple_colouring` outputs at one grid under B.
At the strike searches run at every strike step it also dropped 4,095 `naked_triple`
outputs at one grid under A, and 28,669 `simple_colouring` and 4,095 `naked_triple`
outputs at four grids under B; the steps there were found all the same. Checks 1, 2
and 4 ran on every earlier grid found.
The enumeration covers a tenth of the later grids, and nothing is claimed of the rest
beyond the step itself being found.

*Walk P: placements only, each to the canonical member, from each of the 226 solutions
(218 distinct) towards 30 digits.* Both profiles, both readings, and four choices among
the eligible undone deductions: the least in list order (`first`), the greatest
(`last`), and two seeded draws. 3,616 walks, 103,712 grids, 6,212,388 earlier grids
tried (one canonical member for each placed cell), no failed check. Profile B walks
the same grids as A wherever the choice does not draw.

| Reading, choice (profile A) | Digits left where the walk stops: fewest, median, most | Walks reaching 30 | Techniques the walks undid | Run from the end: solved | Retraces | Hardest step of the run |
| --- | --- | --- | --- | --- | --- | --- |
| broad, `first` | 30, 34, 37 | 2 | `full_unit`, `cross_hatch`, `naked_single` | 226 | 0 | `cross_hatch` on 226 |
| broad, seeded | 30, 34, 39 | 4 of 452 | all four singles | 452 | 0 | `cross_hatch` on 452 |
| broad, `last` | 31, 34, 36 | 0 | `hidden_single`, `naked_single` | 226 | 0 | `cross_hatch` on 226 |
| strict, `first` | 80, 80, 80 | 0 | `full_unit` only | 226 | 226 | `full_unit` on 226 |
| strict, seeded | 69, 76, 80 | 0 | `full_unit` only | 452 | 452 | `full_unit` on 452 |
| strict, `last` | 60, 60, 60 | 0 | `full_unit` only | 226 | 226 | `full_unit` on 226 |

How both readings change as the digits fall, profile A, mean undone deductions at a
grid of the broad walks (the most at one grid in brackets):

| Digits left | Broad, choice `first` | Strict, choice `first` | Broad, seed 1 | Strict, seed 1 | Broad, choice `last` | Strict, choice `last` |
| --- | --- | --- | --- | --- | --- | --- |
| 81 | 324.0 (324) | 81.0 (81) | 324.0 (324) | 81.0 (81) | 324.0 (324) | 81.0 (81) |
| 75 | 300.0 (300) | 0.0 (0) | 288.3 (298) | 10.6 (45) | 300.0 (300) | 75.0 (75) |
| 70 | 254.0 (254) | 0.0 (0) | 244.6 (263) | 5.5 (32) | 267.0 (267) | 63.0 (63) |
| 60 | 123.1 (136) | 0.0 (0) | 155.8 (185) | 1.8 (15) | 195.1 (199) | 47.8 (48) |
| 50 | 67.2 (80) | 0.0 (0) | 78.7 (96) | 0.6 (9) | 121.0 (146) | 30.1 (37) |
| 40 | 16.0 (27) | 0.0 (0) | 19.6 (32) | 0.4 (3) | 41.3 (54) | 10.3 (15) |
| 35 | 1.8 (9), 208 walks | 0.0 (0) | 2.7 (11), 214 walks | 0.1 (2) | 2.7 (14), 220 walks | 0.5 (6) |

- Every one of the 103,712 grids is born kept. All 11,772 forward runs (from each end
  grid and every tenth grid) end solved at the walk's solution, and every end grid's
  digits have one solution.
- No run is harder than its walk. The hardest step of a run is `full_unit` or
  `cross_hatch` in all 11,772; `naked_single` is the hardest of none and
  `hidden_single` is a step of none.
- `naked_single` and `hidden_single` are strict on a canonical member at no grid of
  any walk. `hidden_single` is undone on a canonical member exactly as often as
  `cross_hatch`, at every grid.
- Strict walks take only `full_unit`. They stop because, on every canonical earlier
  grid at the stopping grid, the first entry is a `full_unit` at another cell (a
  `cross_hatch` at another cell in a handful). The strict reading asks that the cell
  just emptied hold the first entry of the list, and the list's order, not the puzzle,
  decides that.
- Broad walks stop with nothing left to undo between 31 and 39 digits left, median 34;
  10 of the 1,808 reached the floor of 30. Yet 42 of the 226 source puzzles are solved
  by ranks 1 and 2 alone, 34 of them from 17 givens. To be put back by a single in one
  step is a stronger condition than for the run to solve.

*Walk M: non-canonical members and strike undoings allowed.* 2,260 walks, 41,111
grids, 38,851 undoings taken; structured members only. Policy `debt-first`: undo a
strike if one can be undone (the least in list order), otherwise a placement by the
highest-ranked technique available, preferring a member where no lower rank places the
same digit, then the fewest strikes left standing.

- All 452 debt-first walks reach 30 digits, and every one ends with 28 to 37 strikes
  standing (median 33) that no striking technique of the profile undoes.
- Mean size of M, profile A (B within 0.1): 0.9 at 65 digits left, 2.4 at 60, 10.2 at 50, 16.9 at 40, 32.9 at
  30. No grid below 64 digits left has M empty.
- Placements undone: `hidden_single` only (11,526 for each profile). Strike undone
  deductions found, summed over grids, profile A: `naked_pair` 403, `hidden_pair` 234,
  `naked_triple` 210, `hidden_triple` 107, `naked_quad` 145, `hidden_quad` 84; profile
  B adds `xy_wing` 6, `xyz_wing` 13 and `unique_rectangle_type_1` 1. None is strict.
  No `pointing` or `claiming` undoing arose.
- A born-kept grid is reached again, after M was non-empty, in 192 of 226 walks for
  each profile, at 778 grids, only between 64 and 69 digits left. At all 778 the digits have one solution
  and the run ends solved, never retraces, and needs nothing above `cross_hatch`,
  although a `naked_pair` or harder had been undone on the way to every one.
- Policy `strict-only` (only strict undoings, the highest rank first, four tie-breaks,
  1,808 walks): M is never non-empty and no strict strike undoing is ever seen. The
  walks stop at 60 to 80 digits; the hardest technique undone strictly is `cross_hatch`
  in 3 walks and `full_unit` in the rest.
- Both policies are greedy, and no search with look-ahead was built. The standing
  strikes of a debt-first walk are what its own choices left: d withheld from peers
  by a `hidden_single` undone to a non-canonical member, which is no pattern's output
  unless one happens to fit. The round trip shows that walks whose every strike is
  accounted for exist, one for each run; these policies did not find them.

*Completeness.* The independent search builds every member of every family of at most
4,096 members, and every nonempty O within M when M has at most 12 members (otherwise
every O of at most 3; the referee also ran every O at the 40 sample grids with 13 to
18 missing), and lists each; it shares only the rebuilt lister with the
prototype. Sample: 680 later grids drawn with seed 20261003, 340 for each profile (200
grids after a run's step, 60 of walk P, 80 of walk M), 18 to 81 digits placed, M empty
at 200, of 1 to 12 at 334 and larger at 146. On all 680 the two programs found the
same undone deductions with the same strict flags: 148,934 broad and 21,122 strict,
none found by one alone. By technique, broad and strict: `full_unit` 18,683 and
13,862; `cross_hatch` 25,913 and 2,867; `naked_single` 30,456 and 943; `hidden_single`
30,456 and 647; `pointing` 4,366 and 2,171; `claiming` 3,806 and 256; `naked_pair`
8,034 and 169; `hidden_pair` 4,250 and 185; `naked_triple` 5,769 and 0;
`hidden_triple` 1,979 and 11; `naked_quad` 2,744 and 0; `hidden_quad` 7,460 and 0;
`x_wing` 665 and 0; `swordfish` 385 and 2; `jellyfish` 659 and 0; `xy_wing` 169 and 1;
`xyz_wing` 197 and 1; `w_wing` 40 and 1; `skyscraper` 79 and 6; `two_string_kite` 33
and 0; `empty_rectangle` 1 and 0; `simple_colouring` 2,789 and 0;
`unique_rectangle_type_1` 1 and 0. On four further files of 1,177 later grids from the
runs the two also agree, on 220,427 undone deductions.

The acceptance criterion asks for the same deductions on every sampled grid, and that
is shown in full on 416 of the 680: the grids (205 under A, 211 under B; M empty at
164 and of 1 to 12 at 252) where no family is over 4,096 members and every nonempty O
was listed, so that the independent search is exhaustive over every earlier grid.
There the two programs give the same 92,500 undone deductions, 13,414 of them strict.
On the other 264 grids the agreement is over part of the output only, as the next
paragraph says, and the criterion is met to that extent and no further.

What the comparison does not cover, each stated and none silent: the 1,737 placed
cells of the sample whose family is over 4,096 members (the prototype's 3,474 undone
placements there stand on its own checks); 18,134 strike outputs of more than 3
candidates at the 106 sample grids with more than 18 missing; and a defect in the
lister itself, which both programs and the audit share. The lister is the one that
reproduced S06's figures. The count of distinct earlier grids is reported as found,
never as complete.

**Step 5. Every undoing checked.** The four checks: E is true for S; E's digits and
candidates admit S alone (the SAT oracle); the deduction is on E's catalogue, or first
on its list for the strict reading; taking it, with upkeep after a placement, gives L
exactly.

- In the measurements themselves, over every earlier grid found: 0 failures in the
  round trip and its enumeration, walk P and the sample; 1,409 in walk M, all of the
  second check and all in one walk. The 1,409 are earlier grids that walk's searches
  built (1,408 members of placement families and 1 strike), not undoings it took.
- The audit checked again, with its own code: every step of the runs (24,031), every
  earlier grid of walk P (100,096), every undoing walk M took (38,851), every strike
  of the sample output (66,153), the 30 uniqueness records, and a draw of 600 sample
  placements. The only failures are of the second check, at 14 earlier grids of the
  one walk.
- Every failure descends from the one `unique_rectangle_type_1` undoing of (d), on
  `seventeen_021` under profile B: it and the grids below it in that walk admit a
  second solution. That is question (d) and not a defect. No undoing outside the two
  uniqueness techniques failed the first, third or fourth check. The 13 undoings that
walk took below the rectangle are placements by `hidden_single`; they fail the second
check only because their later grid already admits a second solution, which is the
premise the cross-check rests on and not the prototype's doing. So no defect is
listed.

The audit found two cells and one sentence wrong in the prototype's own summary of
walk M and corrected them here; no program and no other figure changed.

**Step 6. The table.** The 29 rows in the Goal were drafted from the catalogue
guarantees, attacked by three reviewers and revised. Rows 1 to 22, 28 and 29 are also
measured, as above. Rows 23 to 27 are on paper alone and provisional: no profile
measured sees those ranks, each row names the reading it assumes, and the D-numbers
are the decisions of `tickets/S11-chain-techniques.md`. Rows 16 to 20 are on S06's
reading of "other cells" (S11's D8) and row 22 on wrap and trap being separate
deductions (D9).

**Step 7. The verdict and the follow-up.**

Verdict: **adopt it for a named part only, narrower than the starting recommendation,
and not as a list: placements by `full_unit`, `cross_hatch` and `naked_single`, each
to the canonical earlier grid, under the broad reading.** Nothing else of the list of
undoings is adopted, and what is adopted is one condition on a removal; no list that
runs the other way is proposed for any specification. The starting recommendation
was every placing technique through rank 4; `hidden_single` drops out.

- *What the part is.* A digit may be taken out when the born-kept grid of the digits
  left licenses a single of ranks 1 to 3 that puts it back. Every grid on the way is
  born kept, so there is no choice of earlier grid, no strike left standing and
  nothing to give back; the givens have one solution; the run from them ends solved.
  All of that is argued and held in all 11,772 measured runs.
- *What it cannot do.* For a systematic profile it arrives only at givens whose run's
  hardest step is rank 1 to 3 (measured: rank 1 or 2); an unsystematic profile that
  knows more may take a strike on the same givens, and only "ends solved" is
  guaranteed for it. It ran out at 31 to 39 digits in all but 10 of 1,808 walks. It
  does not aim at a technique.
- *Beside removal followed by a test.* The basic way keeps a removal while the
  solver's verdict stays one, and the published generators of 2(a) keep one while a
  chosen solver still finishes. This condition is stronger than "a run of ranks 1 to
  3 still solves", so it removes less: it stopped 13 or more digits above the 17
  givens from which 34 of the source puzzles are solved by ranks 1 and 2. What it
  saves is the solver call and the run at each removal: one look at one grid. Whether
  that is worth having beside the basic way's removal is S08's to weigh; nothing here
  measures it against one.
- *Why the canonical member alone.* Under the broad reading every placed cell can be
  undone by `naked_single` or `hidden_single` to some member, so undoing to any
  member is no condition at all.
- *Not `hidden_single`.* On a canonical earlier grid it is `cross_hatch` again and adds
  no cell; anywhere else it leaves strikes standing.
- *Not strikes.* A strike can be undone only where a strike already stands, and one
  stands only after a placement was undone to a non-canonical member. Under the one
  greedy policy measured the debt is not paid back: every such walk ended with 28 to
  37 strikes that nothing accounts for, and wherever a born-kept grid was regained the run needed nothing
  above `cross_hatch`. Strikes are given back all at once, by opening afresh on the
  givens left, which `OpenGrid` already does; the run from there says what the puzzle
  asks.
- *Not the strict reading.* It retraces, and that is all it offers. Walked greedily it
  undoes `full_unit` and little else and stops at 60 to 80 digits, because the list's
  order decides which cell holds the first entry. Strict undoings of every technique
  exist at the grids of real runs, so a search could find strict walks; finding them
  is finding a puzzle and its run, which is what removal followed by a rating does.
- *Not the two uniqueness techniques.* An undoing of either is worth the verdict of
  the givens, which the solver must give anyway.
- *Ranks 23 to 27.* Not adopted and not refuted: on paper they can be undone, the
  strict reading is unmeasured for them, and the rows wait on S11.

Proposed text. No specification is edited here. The part adopted needs no word for
undoing and no list that runs the other way: it is a condition on a removal, read from
a grid laid out as `LayOutGrid` lays one out. It belongs to
`docs/specs/generation.allium`, in whatever ticket takes the designed way's removal,
with S08's verdict. Three limits on the text. It is for one position at a time (the
scheme `none`); what it asks of an orbit of two or four positions is S08's question.
It needs generation to open a grid of its own on the givens left: today only a run
opens one (`reach:200-206`) and generation reaches a grid only through a rating, so a
keeper that is not a run has to be stated first. And "born kept" below means every
open cell holding exactly the digits no placed peer holds, which is more than
`AGridIsBornKept` says; a build would name that as a derived value of `Grid`.

```text
-- A removal kept only where a single puts the digit back. The candidate
-- without the position is laid out, every open cell holding exactly the
-- digits no placed peer holds, and the removal is
-- kept only if that grid licenses a deduction of full_unit, cross_hatch or
-- naked_single that the profile sees and that places the removed digit at
-- the removed position. Every such removal keeps the verdict at one and the
-- run solved, and a systematic profile's hardest step is then at most
-- naked_single.
-- Nothing is given back and no strike is left standing: the grid is opened
-- afresh at every removal.
```

Two properties the argument leans on are true of `docs/specs/technique.allium` as it
stands and would be worth stating there as guarantees, because a build would test
them:

```text
@guarantee BornKeptHiddenSingleIsCrossHatch
    -- On a grid whose every open cell holds exactly the digits no placed
    -- peer holds, hides_single and cross_hatches hold of the same units and
    -- digits. So a systematic run whose profile knows
    -- cross_hatch takes no hidden_single before its first strike.

@guarantee SinglesSurviveMoreDigits
    -- If a full_unit, cross_hatch, naked_single or hidden_single places a
    -- digit in a cell of such a grid that is true for a solution, the
    -- same technique places it on every kept grid true for that solution
    -- that holds those digits and more and leaves the cell open.
```

No build ticket is drafted. The text above is input to S08, which weighs this removal
against removal followed by a rating, and the designed way's removal is T20's, still a
draft in T19's notes. If the maintainer wants the two guarantees stated before then,
the ticket takes the next free id. On 2026-10-03 a search of ticket and documentation
text on every ref (`git grep -E 'T3[6-9]|S1[2-9]'` over `git for-each-ref`) found
T34 and T35 claimed by the notes of S10 and S06 and nothing above, so the next free
ids are **T36** and **S12**.

**What S08 needs for a rebuild.** `ai_tmp/s07/` stays in this worktree. A rebuild is
checked against the following.

- *S06's record:* S06's steps 3 and 4, "Step 5, extended" and "What S07 needs". Rebuilt
  here from that text alone with no difference, so it suffices.
- *The frame and the two readings:* "The frame" and step 3 above.
- *How an earlier grid is proposed and tested:* "The undoing prototype" above, and the
  four checks of step 5. For the part adopted only the canonical member is needed: L
  with c emptied, c holding A(c), and d added to every cell of P(c); equivalently the
  born-kept grid of L's digits without c, when L is born kept.
- *The grids:* S06's 226. Fifteen rebuild offline as S06's notes say (the four study
  puzzles, the eight derived and the three named), and their 15 rows below are the
  check that needs no network; the other 211 are S06's file, line range and hash, a
  separately authorised fetch. A walk starts from a grid's one solution, which any
  solver gives; 218 are distinct, the derived grids sharing their study puzzle's.
- *Walk P, choice `first`:* from the solution, at each grid list, for every placed
  cell, the deductions of ranks 1 to 4 (and `bug_plus_one` under profile B, which
  never arose) that its canonical earlier grid licenses and that place its digit.
  Those are the undone deductions under the broad reading; the strict ones are those
  that are also the first entry of that earlier grid's list. The eligible ones are
  the broad ones in a broad walk and the strict ones in a strict walk. Take the least
  eligible in list order (rank, row, column, digit); stop at 30 digits or when none is
  eligible. The forward run is S06's run, started from the end grid's digits as
  givens. Choice `last` takes the greatest. The seeded draws use one Python
  `random.Random("<choice>|<grid name>|<profile>|<reading>")` for a walk, with
  `seed1` or `seed2`, `A` or `B`, `broad` or `strict`, and its `choice` over the
  eligible ones in list order.
- *What is offered as a check.* Only walk P with choice `first`, by the rows below.
  The enumeration, the 680-grid sample and walk M are reported as totals; their
  sampling and tie-breaks are described in step 4 but not to the point where a rebuild
  must give the same totals, and a rebuild that differs there is not thereby wrong.
- *Each grid's counts.* Under the strict reading with choice `first` every grid gives
  the same row, for both profiles: 80 digits left, 644 undone deductions broad and 81
  strict summed over the walk's two grids, 161 earlier grids tried, the run solved with
  hardest rank 1. Under the broad reading with choice `first` the rows follow, the
  same for both profiles. Columns: name; digits left at the end; undone deductions
  under the broad reading, summed over every grid of the walk with the solution and
  the end grid included; the same under the strict reading; earlier grids tried; how
  the run from the end grid ends; its hardest rank. The column sums are 1,540,516,
  18,306 and 625,683, with 226 solved.

```text
study_beginner 33 6837 81 2793 solved 2
derived_beginner_min 33 6837 81 2793 solved 2
derived_beginner_minrev 33 6837 81 2793 solved 2
study_intermediate 32 6780 81 2825 solved 2
derived_intermediate_min 32 6780 81 2825 solved 2
derived_intermediate_minrev 32 6780 81 2825 solved 2
study_advanced 33 6839 81 2793 solved 2
derived_advanced_min 33 6839 81 2793 solved 2
derived_advanced_minrev 33 6839 81 2793 solved 2
study_expert 33 6733 81 2793 solved 2
derived_expert_min 33 6733 81 2793 solved 2
derived_expert_minrev 33 6733 81 2793 solved 2
seventeen_001 32 6734 81 2825 solved 2
seventeen_002 35 6990 81 2726 solved 2
seventeen_003 32 6881 81 2825 solved 2
seventeen_004 35 6852 81 2726 solved 2
seventeen_005 36 6744 81 2691 solved 2
seventeen_006 35 6777 81 2726 solved 2
seventeen_007 37 6628 81 2655 solved 2
seventeen_008 34 6652 81 2760 solved 2
seventeen_009 34 6998 81 2760 solved 2
seventeen_010 35 6823 81 2726 solved 2
seventeen_011 35 6742 81 2726 solved 2
seventeen_012 33 6899 81 2793 solved 2
seventeen_013 34 6813 81 2760 solved 2
seventeen_014 36 6834 81 2691 solved 2
seventeen_015 36 6792 81 2691 solved 2
seventeen_016 35 6786 81 2726 solved 2
seventeen_017 35 6994 81 2726 solved 2
seventeen_018 33 6713 81 2793 solved 2
seventeen_019 34 6693 81 2760 solved 2
seventeen_020 33 6870 81 2793 solved 2
seventeen_021 34 6943 81 2760 solved 2
seventeen_022 35 6760 81 2726 solved 2
seventeen_023 34 6912 81 2760 solved 2
seventeen_024 34 6898 81 2760 solved 2
seventeen_025 32 6813 81 2825 solved 2
seventeen_026 34 6839 81 2760 solved 2
seventeen_027 33 6909 81 2793 solved 2
seventeen_028 34 6774 81 2760 solved 2
seventeen_029 32 7019 81 2825 solved 2
seventeen_030 31 6973 81 2856 solved 2
seventeen_031 32 6984 81 2825 solved 2
seventeen_032 32 6934 81 2825 solved 2
seventeen_033 34 6857 81 2760 solved 2
seventeen_034 32 6693 81 2825 solved 2
seventeen_035 33 6756 81 2793 solved 2
seventeen_036 33 6826 81 2793 solved 2
seventeen_037 34 6758 81 2760 solved 2
seventeen_038 34 6739 81 2760 solved 2
seventeen_039 34 6748 81 2760 solved 2
seventeen_040 33 6822 81 2793 solved 2
seventeen_041 30 6892 81 2886 solved 2
seventeen_042 33 6780 81 2793 solved 2
seventeen_043 36 6752 81 2691 solved 2
seventeen_044 34 6889 81 2760 solved 2
seventeen_045 35 6862 81 2726 solved 2
seventeen_046 33 6987 81 2793 solved 2
seventeen_047 36 6926 81 2691 solved 2
seventeen_048 33 6864 81 2793 solved 2
seventeen_049 33 6907 81 2793 solved 2
seventeen_050 35 6700 81 2726 solved 2
seventeen_051 33 7027 81 2793 solved 2
seventeen_052 33 6804 81 2793 solved 2
seventeen_053 35 6888 81 2726 solved 2
seventeen_054 35 6939 81 2726 solved 2
seventeen_055 30 6892 81 2886 solved 2
seventeen_056 32 6874 81 2825 solved 2
seventeen_057 32 6913 81 2825 solved 2
seventeen_058 32 6809 81 2825 solved 2
seventeen_059 34 6900 81 2760 solved 2
seventeen_060 34 6812 81 2760 solved 2
seventeen_061 33 7001 81 2793 solved 2
seventeen_062 32 6889 81 2825 solved 2
seventeen_063 33 6828 81 2793 solved 2
seventeen_064 34 6744 81 2760 solved 2
seventeen_065 35 6601 81 2726 solved 2
seventeen_066 35 6663 81 2726 solved 2
seventeen_067 35 6757 81 2726 solved 2
seventeen_068 34 6658 81 2760 solved 2
seventeen_069 34 6726 81 2760 solved 2
seventeen_070 36 6638 81 2691 solved 2
seventeen_071 34 6724 81 2760 solved 2
seventeen_072 35 6640 81 2726 solved 2
seventeen_073 33 6829 81 2793 solved 2
seventeen_074 34 6735 81 2760 solved 2
seventeen_075 32 6748 81 2825 solved 2
seventeen_076 35 6889 81 2726 solved 2
seventeen_077 33 6834 81 2793 solved 2
seventeen_078 34 6877 81 2760 solved 2
seventeen_079 36 6626 81 2691 solved 2
seventeen_080 33 6742 81 2793 solved 2
seventeen_081 33 6715 81 2793 solved 2
seventeen_082 34 6768 81 2760 solved 2
seventeen_083 33 6674 81 2793 solved 2
seventeen_084 35 6778 81 2726 solved 2
seventeen_085 34 6777 81 2760 solved 2
seventeen_086 35 6693 81 2726 solved 2
seventeen_087 33 6874 81 2793 solved 2
seventeen_088 33 6899 81 2793 solved 2
seventeen_089 36 6695 81 2691 solved 2
seventeen_090 35 6633 81 2726 solved 2
seventeen_091 34 6725 81 2760 solved 2
seventeen_092 33 6758 81 2793 solved 2
seventeen_093 33 6782 81 2793 solved 2
seventeen_094 34 6792 81 2760 solved 2
seventeen_095 33 6811 81 2793 solved 2
seventeen_096 33 6873 81 2793 solved 2
seventeen_097 36 6704 81 2691 solved 2
seventeen_098 33 6877 81 2793 solved 2
seventeen_099 35 6939 81 2726 solved 2
seventeen_100 34 6755 81 2760 solved 2
unbiased_001 35 6779 81 2726 solved 2
unbiased_002 33 6847 81 2793 solved 2
unbiased_003 33 6768 81 2793 solved 2
unbiased_004 32 6898 81 2825 solved 2
unbiased_005 33 6965 81 2793 solved 2
unbiased_006 33 6886 81 2793 solved 2
unbiased_007 34 6616 81 2760 solved 2
unbiased_008 32 6995 81 2825 solved 2
unbiased_009 36 6806 81 2691 solved 2
unbiased_010 34 6758 81 2760 solved 2
unbiased_011 34 6816 81 2760 solved 2
unbiased_012 33 6804 81 2793 solved 2
unbiased_013 35 6722 81 2726 solved 2
unbiased_014 35 6767 81 2726 solved 2
unbiased_015 34 6689 81 2760 solved 2
unbiased_016 33 6927 81 2793 solved 2
unbiased_017 34 6814 81 2760 solved 2
unbiased_018 35 6860 81 2726 solved 2
unbiased_019 34 6745 81 2760 solved 2
unbiased_020 32 6829 81 2825 solved 2
unbiased_021 35 6801 81 2726 solved 2
unbiased_022 33 6783 81 2793 solved 2
unbiased_023 35 6836 81 2726 solved 2
unbiased_024 34 6944 81 2760 solved 2
unbiased_025 34 6796 81 2760 solved 2
unbiased_026 32 6969 81 2825 solved 2
unbiased_027 35 6976 81 2726 solved 2
unbiased_028 33 6908 81 2793 solved 2
unbiased_029 33 6833 81 2793 solved 2
unbiased_030 32 6812 81 2825 solved 2
unbiased_031 32 6831 81 2825 solved 2
unbiased_032 33 6860 81 2793 solved 2
unbiased_033 33 6936 81 2793 solved 2
unbiased_034 35 6628 81 2726 solved 2
unbiased_035 34 6693 81 2760 solved 2
unbiased_036 34 6790 81 2760 solved 2
unbiased_037 33 6721 81 2793 solved 2
unbiased_038 34 7033 81 2760 solved 2
unbiased_039 33 6974 81 2793 solved 2
unbiased_040 33 6854 81 2793 solved 2
unbiased_041 35 6796 81 2726 solved 2
unbiased_042 32 6864 81 2825 solved 2
unbiased_043 35 6687 81 2726 solved 2
unbiased_044 34 6792 81 2760 solved 2
unbiased_045 32 6762 81 2825 solved 2
unbiased_046 35 6671 81 2726 solved 2
unbiased_047 31 6839 81 2856 solved 2
unbiased_048 35 6930 81 2726 solved 2
unbiased_049 36 6716 81 2691 solved 2
unbiased_050 35 6854 81 2726 solved 2
top1465_001 33 6802 81 2793 solved 2
top1465_002 34 6790 81 2760 solved 2
top1465_003 34 6787 81 2760 solved 2
top1465_004 32 6851 81 2825 solved 2
top1465_005 33 6957 81 2793 solved 2
top1465_006 32 6994 81 2825 solved 2
top1465_007 34 6707 81 2760 solved 2
top1465_008 33 6903 81 2793 solved 2
top1465_009 35 6711 81 2726 solved 2
top1465_010 33 6856 81 2793 solved 2
top1465_011 35 6950 81 2726 solved 2
top1465_012 32 6798 81 2825 solved 2
top1465_013 34 6666 81 2760 solved 2
top1465_014 35 6718 81 2726 solved 2
top1465_015 32 6840 81 2825 solved 2
top1465_016 35 6754 81 2726 solved 2
top1465_017 32 6869 81 2825 solved 2
top1465_018 34 6905 81 2760 solved 2
top1465_019 34 6931 81 2760 solved 2
top1465_020 33 6890 81 2793 solved 2
top1465_021 34 6903 81 2760 solved 2
top1465_022 33 6959 81 2793 solved 2
top1465_023 37 6728 81 2655 solved 2
top1465_024 32 6925 81 2825 solved 2
top1465_025 34 6723 81 2760 solved 2
top1465_026 34 6771 81 2760 solved 2
top1465_027 34 6896 81 2760 solved 2
top1465_028 33 6883 81 2793 solved 2
top1465_029 34 6831 81 2760 solved 2
top1465_030 36 6663 81 2691 solved 2
top1465_031 33 6905 81 2793 solved 2
top1465_032 31 6930 81 2856 solved 2
top1465_033 32 6972 81 2825 solved 2
top1465_034 33 6799 81 2793 solved 2
top1465_035 36 6715 81 2691 solved 2
top1465_036 34 6967 81 2760 solved 2
top1465_037 32 6705 81 2825 solved 2
top1465_038 36 6873 81 2691 solved 2
top1465_039 35 6639 81 2726 solved 2
top1465_040 33 6770 81 2793 solved 2
top1465_041 34 6909 81 2760 solved 2
top1465_042 34 6738 81 2760 solved 2
top1465_043 34 6829 81 2760 solved 2
top1465_044 33 6834 81 2793 solved 2
top1465_045 35 6676 81 2726 solved 2
top1465_046 36 6886 81 2691 solved 2
top1465_047 34 6709 81 2760 solved 2
top1465_048 35 6923 81 2726 solved 2
top1465_049 32 6866 81 2825 solved 2
top1465_050 34 6922 81 2760 solved 2
norvig_hardest_01 34 6830 81 2760 solved 2
norvig_hardest_02 33 6949 81 2793 solved 2
norvig_hardest_03 34 6913 81 2760 solved 2
norvig_hardest_04 34 6820 81 2760 solved 2
norvig_hardest_05 36 6678 81 2691 solved 2
norvig_hardest_06 35 6817 81 2726 solved 2
norvig_hardest_07 33 6732 81 2793 solved 2
norvig_hardest_08 34 6823 81 2760 solved 2
norvig_hardest_09 34 6717 81 2760 solved 2
norvig_hardest_10 34 6583 81 2760 solved 2
norvig_hardest_11 34 6864 81 2760 solved 2
inkala_2012 34 6744 81 2760 solved 2
sudokuwiki_unsolvable_28 35 6744 81 2726 solved 2
sudokuwiki_unsolvable_49 34 6767 81 2760 solved 2
```

**Step 8. Verification.**

```text
$ rg -n 'ADeductionPlacesOrStrikes|uniqueness_promised' docs/specs/technique.allium
290:    uniqueness_promised: Boolean
773:invariant ADeductionPlacesOrStrikes {
851:        grid.uniqueness_promised
879:        -- deduction additionally requires a truthful uniqueness_promised:
1019:        -- Both require uniqueness_promised and, as DeductionsAreSound does
1148:--     keeper states uniqueness_promised, true of givens sudoku.allium has
$ rg -n 'born kept' docs/specs/technique.allium
726:    -- what the start does not rule out. So a grid is born kept: no candidate
$ rg -c '^\| [0-9]+ \| `' tickets/S07-undoing-a-deduction.md
29
$ git status --porcelain
 M tickets/README.md
 M tickets/S07-undoing-a-deduction.md
```

### Deviations, and why

- **The branch.** The worktree was created by Supacode on `ticket/S07-undoing-a-deduction`;
  the `branch:` field says `ticket/s07-undoing-a-deduction`. The same difference S06
  recorded; the field is left as written.
- **Context facts that had drifted.** "Every candidate the start allows and no other"
  is the module's Excludes (`technique:45-46`), not the sentence that a grid is born
  kept (`technique:724-728`); `AGridIsBornKept` says only that no candidate is stale.
  There is no ordered list in the specifications yet: S06's is drafted as T35 and not
  taken. `OpenGrid` is `reach.allium`'s rule, and `technique.allium`'s is
  `LayOutGrid`. The ticket's facts are left as written.
- **The rebuild read S06's grids from S06's scratch,** not from a fresh fetch, and ran
  S06's venv, both with the maintainer's leave. The programs were rebuilt from the
  ticket text alone.
- **More measured than asked.** Ranks 13 to 22, 28 and 29 were measured as well as
  ranks 1 to 12, at the maintainer's request. A round trip over S06's runs, a walk with
  strike undoings and a fourth choice (`last`) were added to the walk the ticket
  describes, because under the strict reading the least choice stops after one undoing
  and a walk of canonical placements never meets a strike.
- **The walk undoes to the canonical member; other members are counted at the runs'
  grids and in walk M.** Step 4 asks for the earlier grids tried at each grid; the
  families are too large to walk through, as the ticket expects.
- **The independent search shares the lister.** It was written apart and by brute
  force, but both programs list an earlier grid's deductions with the rebuilt
  prototype, which is the one checked against S06's figures.
- **The caps:** families over 4,096 members and strike subsets past 4,096 for a
  witness, each counted in step 4.
- **Fetched sources** were read as they stood on the day; the HoDoKu manual through an
  extractor and the forum threads from archive snapshots.
- **No build ticket is drafted,** for the reason step 7 gives.
- **Reviews answered.** Before the commit, three reviewers read the written ticket
  against the data, the specifications and the acceptance criteria; their findings
  were checked and the text corrected. After it, an adversarial review by Codex made
  four findings, each checked and upheld: a `forcing_chain` can place what a single
  places even with no empty path and no candidate twice (row 27 and the hand-back to
  S11 now say so); "keeps every solution only if" in row 28 needed the premise that
  the later grid admits S alone; the completeness comparison is exhaustive on 416 of
  the 680 sampled grids and partial on the rest (step 4 now says which); and S06's
  order by rank and output does reach ranks 23 to 27, so the strict reading there is
  unmeasured, not undefined. Codex found nothing in the frame, the singles
  guarantees, the walk's uniqueness argument, the proposed text or the arithmetic.

### Handed back

- **To S08:** the verdict, the proposed text and the rebuild material of step 7; the
  211 fetched grids need their fetch authorised again. The
  removal S08's proposal draws from "the list of undoings" is, on this evidence, the
  one-line condition above, good only for givens whose run stays within rank 3; for a
  contract above that, undoing does not aim, and removal followed by a rating is what
  remains to weigh. The SAT oracle needs `python-sat` in a scratch venv, a separately
  authorised install.
- **To S11:** the rows for ranks 23 to 27, each with the decisions it waits on; and two
  findings the paper analysis met on the way. Under a walk (D1's starting
  recommendation) a `forcing_chain` licenses a single wherever the placed candidate has
  a strong link, whatever D6 says; "no candidate twice" with "no empty path" shuts
  out those trivial proofs and no more. A nonempty path with no repeat can still place
  what a single places: with r1c1 holding 1 alone, r2c1 holding 1 and 4 and r2c7
  holding 1 alone, denying 1 at r1c1 forces it at r2c1 (two places in the column) and
  so out of r2c7, which is left with nothing. So `forcing_chain` overlaps the singles
  under every option, and what S11 decides is how far. And D2 decides how often `x_chain` undoes what `pointing` and
  `claiming` would.
- **To T35 (drafted in S06's notes):** the strict reading rests on the first entry of
  the list being what a run takes; nothing here contradicts the order, and the two
  guarantees proposed above could ride with that change.
- **To the maintainer, as open questions the specifications do not answer:** whether a
  by-sight single is meant to be licensed where the cell's marks lack the digit
  (`technique:900-901`); whether "every eligible target" (`technique:905-906`) means
  every target that stands, on which the answer to 2(c) rests; and who keeps a grid
  that is not a run's, and whether a keeper may promise uniqueness on condition.
- **To the index:** S07's row set to `done`.

### Open points settled

- **Whether the strict reading is too narrow.** Measured: walked greedily it never
  undoes a strike and never anything above `cross_hatch`, and stops at 60 to 80 digits.
  It is not empty: every step of every run is a strict undoing at its own grid. But
  reaching givens whose run needs rank 5 or above by strict undoings means finding the
  run itself. Recommendation: do not build on the strict reading. The maintainer
  decides what follows.
- **Whether strikes need undoing at all.** Recommendation: no. Undo placements of
  ranks 1 to 3 only, as the verdict states, to the canonical earlier grid, and let a run from the givens left say what the puzzle
  asks. Under the greedy policy measured, undoing strikes neither returns to a set of givens
  (28 to 37 strikes left standing at 30 digits in every walk) nor makes the run need what was undone.
  The maintainer decides.
- **The words.** Kept here as the ticket set them. `docs/specs/board.allium` already
  owns "undo" for taking back a player's entry, where the board remembers which notes
  upkeep struck; an undoing in this ticket's sense remembers nothing and must choose.
  Recommendation: the specifications take no word for it. The part adopted is a
  condition on a removal and is stated without one. If a later ticket does need the
  general notion, "withdrawing a deduction" and "the grid before" keep clear of the
  board's word. The maintainer decides.

## Open points

The three below are answered under "Open points settled", each with a recommendation
that waits on the maintainer; they are left as written.

- Whether the strict reading is too narrow to arrive at givens whose run needs rank 5
  or above; step 4 measures it, the maintainer decides what follows.
- Whether strikes need undoing at all, or only placements, with a run from the givens
  left to say what the puzzle asks.
- The words. Undoing and earlier grid are this ticket's; the specification that takes
  them, if one does, names them in its Vocabulary.
