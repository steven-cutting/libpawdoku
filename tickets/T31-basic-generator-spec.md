---
id: T31
title: "Spec change: the basic generator in generation, a published method of removal checked by the verdict"
status: done
depends_on: [T19]
parallel_with: []
branch: ticket/t31-basic-generator-spec
estimated_size: M
---

# T31: Spec change: the basic generator in generation, a published method of removal checked by the verdict

## Context

The engine can judge givens (`solver`), hold a puzzle in play (`board`) and draw through
the randomness boundary (`random`). It cannot make a puzzle. `docs/specs/generation.allium`
is a skeleton: scope, config and seven open questions, no entity, rule or surface. The
generator it is written for is the *designed* one: givens removed one orbit of a symmetry
scheme at a time, a candidate kept only when the solver's verdict is one and a
`reach.allium` `Rate` run ends solved within a technique contract. T20, drafted in
`tickets/T19-puzzle-design-objectives.md` under "Handed back", is to write its triggers;
it waits on `technique` and `reach` in Rust and on the spikes S06 to S08.

On 2026-10-02 the maintainer asked for something smaller and sooner: "a minimal but
sufficient classic random Sudoku puzzle generator. It doesn't need to be perfect in terms
of the quality of puzzles it produces. It just needs to be incredibly average [...]
compared to other basic random puzzle generators. The purpose of this is to have
something quick that just works well enough for testing and also something that can
provide a baseline that we can compare against for our later more complex generators."
This ticket calls it **the basic generator**. This ticket specifies it; T32 builds it.

**Decided by the maintainer on 2026-10-02.**

1. **Its home is `generation.allium`.** The module says "how a setter that is a program
   finds givens", and the basic generator is one way of doing that. The designed
   generator is a second way beside it, not built on top of it. The basic way removes
   one position at a time and accepts on the solver's verdict alone: no orbit, no
   technique contract and no `Rate` run.
2. **Its settings are the source paper's five**, with the paper's figures, "because this
   generator is supposed to be different and very standard. So that it will make for a
   better baseline to compare against."
3. **It is public API**, specified here before T32 builds it.
4. **Two tickets:** this one for the specification, T32 for the Rust.

**The source.** "Sudoku Puzzles Generating: from Easy to Evil", a twenty-page paper
headed "Team # 3485" that names no author, at
<https://zhangroup.aporc.org/images/files/Paper_3485.pdf>, read on 2026-10-02. Everything
this ticket needs from it is written out below, figures included, so the paper need not
be fetched; fetching it is a network action and separately authorised. Its method, which
it calls digging holes: make a full valid grid, then take digits away one cell at a time,
checking after each that the puzzle still has one solution.

The paper's two tables of figures and its assignment of orders, joined. "Level", the
names of the levels and the names of the orders are the paper's words:

| Paper's level | Paper's name | Givens left | Fewest givens in any row or column | Order of removal |
| --- | --- | --- | --- | --- |
| 1 | Extremely easy | "more than 50" | 5 | Randomizing globally |
| 2 | Easy | 36 to 49 | 4 | Randomizing globally |
| 3 | Medium | 32 to 35 | 3 | Jumping one cell |
| 4 | Difficult | 28 to 31 | 2 | Wandering along "S" |
| 5 | Evil | 22 to 27 | 0 | Left to Right then Top to Bottom |

The paper's five operators, in its own order, and what each becomes here:

| Paper | Here |
| --- | --- |
| The full grid, by a Las Vegas algorithm: "randomly locate n cells in an empty grid and then fill these empty cells with random 1-to-9 digits while satisfy the game rules", solve, and try again if no solution arrives within a time limit (0.1 s). It settles on `n` of 11. | Kept, without the clock: eleven givens that do not conflict are drawn through the randomness boundary and put to `solver.allium`. A solution it finds is the solution grid. A verdict of none is a failed attempt; attempts are counted against a figure. |
| Operator 1, the order of removal: the four orders in the table above. | Kept as stated below. |
| Operator 2, two restrictions. "Restriction 1: randomize a bound value within the range of the total givens, and the remained cells must be more than that bound value. Restriction 2: the remained cells in each row and column must be more than the lower bound of givens in rows and columns." | Kept: one draw picks a bound inside the range, and a removal that would break either restriction is refused. |
| Operator 3, uniqueness "by reduction to absurdity": put each of the other eight digits in the cell and ask a depth-first solver whether any gives a solution. | Replaced by `solver.allium`'s verdict on the givens without the cell. The answer is the same; the paper's check exists because its solver lists every solution, and this one stops at the second. |
| Operator 4, pruning: "a cell can only be tried to dig once [...] and any empty cells should not be refilled again". | Kept: every position is visited once and never again, so removal asks for at most 81 verdicts. |
| Operator 5, "equivalent propagation": swap two digits, two columns of a stack, two stacks, or roll the grid, to vary what comes out. | Not carried. The seed already varies the solution grid. |

The paper's flow, from its Figure 5.2: fix the order and the restrictions; while a
position is left to try, take the next in order; if removing it breaks a restriction, it
is never removed; otherwise, if the puzzle without it has more than one solution, it is
never removed; otherwise it is removed; then output. Nothing is retried and nothing is
put back. The paper has no step for a puzzle that ends above its range: it outputs what
it reached.

The three fixed orders, from its Figure 5.3, with positions as (row, column):

- **Left to right then top to bottom.** Row 1 from column 1 to 9, then row 2 the same
  way, to row 9. It begins (1,1), (1,2), (1,3).
- **Wandering along S.** Row 1 from column 1 to 9, row 2 from column 9 to 1, row 3 from
  1 to 9, alternating to row 9. It runs (1,8), (1,9), (2,9), (2,8) at the first turn.
- **Jumping one cell.** The figure shades the grid as a chequerboard and hops along the
  S path taking one cell and skipping the next: (1,1), (1,3), (1,5), (1,7), (1,9),
  (2,8), (2,6), (2,4), (2,2), (3,1), (3,3). Those are the 41 positions whose row and
  column sum to an even number. The figure shows that pass and nothing after it. A
  second pass must exist: level 3 leaves 32 to 35 givens, so it removes at least 46.

What the paper measured, for whoever compares against it: making a level 5 puzzle
succeeded 13.33 per cent of the time under the random order, 89.66 per cent jumping one
cell and 100 per cent under the other two; its sparsest puzzle had 22 givens, not the 17
that are possible. In levels 3 to 5 the order of removal is fixed, so the only draws are
the solution grid and the bound.

Not carried, beside Operator 5: the paper's grading (its Section 4, a weighted score
from the count of givens, the row and column floor, the techniques that apply and a
count of search steps). Rating is `reach.allium`'s, and `generation.allium` says of the
count of givens that it is "never a rating".

**Facts that shape the work**, each verified on 2026-10-02 against `main` at `460b93c`:

- **The skeleton says acceptance is two things.** Its Scope and its Includes say a
  candidate is kept only when the verdict is one *and* the run ends solved within the
  technique contract. The basic way keeps on the verdict alone. Both passages need a
  sentence that says so, or the module contradicts itself.
- **The skeleton excludes "how a full grid is drawn quickly".** The basic way states how
  its grid is drawn, because the method is the point of it. The exclusion is reworded so
  that it still covers speed and storage.
- **Words already taken.** `generation.allium` uses "level" for the research report's
  four (its pacing question), and S08 records that "the specifications say technique
  contract and rating". `sudoku.allium` uses "band" for a row of three boxes. So neither
  "level" nor "band" names the paper's five here, and "difficulty" is not claimed of
  them. The proposed word is **tier**; the first Open point asks the maintainer.
- **A tier is a construction setting.** A range for the count of givens, a floor per row
  and column and an order of removal. Whether a tier's puzzles are harder than
  another's is exactly what a later comparison through `reach.allium` measures, and
  nothing here asserts it.
- **Removal is bounded by construction.** At most 81 positions are visited and at most
  81 verdicts asked. The one step that can repeat without end is drawing the solution
  grid, so that is the one limit the basic way needs. `step_budget` and
  `candidate_limit` are declared for the designed way; whether the basic way reads
  them is an Open point.
- **Replay stands on what the solver does.** Eleven givens nearly always have many
  solutions. The search stops at two, and which it reaches depends on two things
  `solver.allium` leaves open. One is the order "in which tied cells and tied branches
  are taken", required only to be "the same each time". The other is what propagation
  strikes before a guess: of deductions past singles the module says "A solver may make
  them", and a guess is on a cell with the fewest candidates, so a solver that strikes
  more can guess on another cell and reach other solutions with its tie-break
  unchanged. So the same draws give the same solution grid for one implementation of
  the solver, and no further. The module's `generation_version` comment already says it
  is raised "whenever removal or acceptance changes so that a seed gives other givens";
  it must now cover any change, here or in the solver, that makes the same draws give
  other givens or take a different number of draws.
- **Two solutions, one grid.** `solver.allium`'s `SearchResult` exposes the solved
  branches and gives them no order. The basic way needs one of them, so the module
  must say which.
- **A seed is not a stream.** The boundary is a stream of draws "begun from a seed,
  indexed from zero" (`crates/pawdoku/src/random.rs`), and T32's entry takes a stream,
  which a caller may already have drawn from. What fixes the result is the draws taken,
  from wherever the stream stands. A seed stands for them only when the stream is at
  index zero. The module states replay in terms that hold for both.
- **`floor(u * n)` has two readings.** `human-solving.allium`'s `ExactReplay` words a
  choice among `n` as `floor(u*n)`. Read as exact arithmetic, and read as the binary64
  product truncated, the two differ for a few draws beside a boundary. For the draw
  `6004799503160661 / 2^53` and `n` of 3 the exact product is `2 - 2^-53`, whose floor
  is 1, and the binary64 product rounds to 2.0. Some such draw exists for 74 of the
  values of `n` from 1 to 81 (both computed on 2026-10-02). Each reading gives the same
  answer on every IEEE 754 target; they do not give the same answer as each other. The
  module must say which it means. Under either, the largest draw below one never
  reaches `n`.
- **Replay does not read the catalogue.** The module's Replay clause lists three
  versions. The basic way uses no technique, so its givens depend on
  `generation_version` and `random_version` and not on `catalogue_version`.
- **A floor can hold the count above the bound.** Tier 2's floor of 4 allows no fewer
  than 36 givens, which is the low end of its range, and then only with exactly four in
  every row and column. A result with more givens than its bound is the ordinary case,
  not a failure.
- **An unreferenced `use` is not a diagnostic** (T23 verified it), so the module's four
  `use` lines stay as they are. The basic way reads `sudoku` and `solver` alone.
- **`generation_version` is `"generation-0"`**, and its comment says it is "raised when
  its triggers land". They land here.

**Every page that calls the module a skeleton**, found by search on 2026-10-02:

| Where | What it says today |
| --- | --- |
| `docs/specs/generation.allium` 14-18 | "This module is a skeleton [...] nothing should be built on it until they are" |
| `docs/explanation/layering.md` 38-39 | "It is a skeleton today, and its Rust module arrives with its triggers" |
| `docs/explanation/specifications.md` 71-78 | "It is a skeleton, scope, config and open questions with no trigger" |
| `docs/explanation/architecture.md` 307-311 | "today a skeleton of scope, config and open questions" |
| `docs/project/purpose-and-scope.md` 40-46 | "No puzzle generation yet [...] today a skeleton" |
| `docs/how-to/work-with-the-specs.md` 28 | "A skeleton today: scope, config and open questions, no triggers" |
| `docs/README.md` 24-25 | "today a skeleton of scope, config and open questions" |

`docs/explanation/puzzle-design.md` cites `generation.allium` by line number in its
closing table (`generation.allium:130` for one). Every edit above an open question moves
those lines.

The done tickets and the open spikes S06 to S08 call the module a skeleton and S08
expects seven open questions. They are records of their day and are not edited; what
this ticket changes under them is handed back.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `.agents/skills/spec-change/SKILL.md`
and `.agents/skills/tend/SKILL.md`; `docs/specs/generation.allium`, `docs/specs/solver.allium`
and `docs/specs/sudoku.allium` in full; `docs/specs/human-solving.allium`, the `ExactReplay`
guarantee; `docs/explanation/puzzle-design.md`, its closing table;
`docs/decisions/0012-generation-and-dev-time-judges.md`; `docs/decisions/README.md`
("Writing a new one"); `tickets/S08-removal-by-undoing.md`, Context; the T20 draft in
`tickets/T19-puzzle-design-objectives.md`.

## Goal

`generation.allium` states the basic generator in full: what a caller asks for, how the
solution grid is drawn, how removal proceeds and when it refuses, what comes back, and
what is guaranteed of it. A reader can tell the basic way from the designed way in every
passage, and each of the designed way's open questions still stands. Every page that
calls the module a skeleton says what is true instead. Both specification gates end
`9 specifications, no diagnostics and no findings.` with no waiver, and `just check` is
green.

## Non-goals

- No Rust under `crates/`. T32 builds the module.
- Nothing of the designed way: no orbit, no technique contract, no `Rate` run, no
  shortcut suppression, no pacing. T20 writes those.
- No open question of the designed way answered. Where the basic way has its own
  answer, the question gains a clause saying so and keeps asking for the designed way.
- No grading and no claim that a tier is harder than another.
- No symmetry: the basic way removes one position at a time.
- No swap of digits, rows, columns or stacks after removal.
- No new module and no new `use` line. `generation.allium` stays the ninth.
- No edit to a done ticket, to S06, S07 or S08, or to the T20 draft.

## Files touched

| Path | Change |
| --- | --- |
| `docs/specs/generation.allium` | The header, the config, the basic way's constructs, clauses on four open questions (steps 3 to 6) |
| `docs/decisions/0015-basic-generator.md` | New decision record (step 7); the number is the next free one on the day |
| `docs/decisions/README.md` | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | Entry for the record. Frozen under CONVENTIONS.md §11; see Open points |
| `docs/explanation/layering.md` | The prose at 35-39: what the basic way stands on, and that the module is no longer a skeleton |
| `docs/explanation/specifications.md` | The paragraph at 71-78 |
| `docs/explanation/architecture.md` | The sentence at 307-311 |
| `docs/project/purpose-and-scope.md` | The bullet at 40-46: specified, not yet built |
| `docs/how-to/work-with-the-specs.md` | The `generation.allium` row at 28 |
| `docs/README.md` | The clause at 24-25. Frozen under CONVENTIONS.md §11; see Open points |
| `docs/project/terminology.md` | Rows for the new words |
| `docs/explanation/puzzle-design.md` | Line numbers in the closing table that cite `generation.allium`, and nothing else |
| `CHANGELOG.md` | Under Unreleased: the basic generator specified; the count of decision records |
| `tickets/README.md` | T31's index row set `done` |
| `tickets/T31-basic-generator-spec.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t31-basic-generator-spec` from `main` (README.md "How
   to pick up a ticket"). Run `just initialize` if `.pixi/` is absent; it uses the
   network and this ticket authorises it. Run `just check-specs` and `just analyse-specs`
   before any edit and quote their closing lines.

2. **Put the Open points to the maintainer before writing a clause.** Each has a
   proposed answer. They are product decisions and the specification is where they are
   recorded, so do not pick one silently. Record each answer, with its date, under "Open
   points settled".

3. **The header.** Follow the `spec-change` skill.

   - **Scope.** Say that the module now states two ways of finding givens and which is
     which. The basic way: a solution grid drawn through the randomness boundary, givens
     removed one position at a time in a stated order, each removal kept only while the
     solver's verdict stays one. The designed way stays as the Scope words it today,
     and stays a skeleton. Replace the paragraph "This module is a skeleton [...]" with
     one that says which part is stated and which is not, and that nothing is built on
     the part that is not.
   - **Source.** Add the paper beside `docs/explanation/puzzle-design.md`: its title,
     "Team # 3485", the address and the date read. Say which of its operators are
     carried and which are not, in a line each.
   - **Includes.** Removal, Acceptance, Limits and Replay each gain the basic way's
     sentence beside the designed way's: one position at a time; the verdict alone; a
     count of grid attempts; `generation_version` and `random_version`, and not the
     catalogue's.
   - **Excludes.** Reword "How a full grid is drawn quickly" so the basic way's drawing
     is stated and speed and storage stay excluded. Add: grading, and any claim that a
     tier is harder than another; the swaps of the paper's Operator 5.
   - **Vocabulary.** Add the words step 2 settled: the tier, the bound, the floor, the
     order of removal, a grid attempt. Say that "level" stays the research report's.

4. **The config.** The five tiers' figures as the table in Context gives them, with the
   answers to the Open points for tier 1's upper end; the count of seeded givens, 11;
   the figure for grid attempts. Raise `generation_version` and extend its comment: it
   is raised for any change, in this module or in the solver, that makes the same draws
   give other givens or take a different number of draws. Name the two the solver
   allows itself: the order in which it takes tied cells and branches, and what its
   propagation strikes before it guesses. Run `just check-specs`.

5. **The basic way's constructs.** Write them with the `tend` skill, at the level
   `solver.allium` is written: what happens and what is guaranteed, not how it is
   stored. Run `just check-specs` after each block. The module must say, in whatever
   constructs fit:

   - **What is asked.** A tier and a stream of draws. A seed names a stream at index
     zero, and then the tier and the seed choose the result between them. Say what
     holds for a stream that has already been drawn from, as step 2 settled.
   - **A grid attempt.** Eleven positions and their digits are drawn as step 2 settled;
     the givens go to `solver.allium`; a solution it finds is the solution grid, and
     when it finds two, the one step 2 settled; a verdict of none ends the attempt and
     another begins.
   - **Spent attempts.** What is reported when the figure is reached with no grid.
   - **The bound.** One draw inside the tier's range, after the grid.
   - **The order.** The four orders, each stated so that the position visited at every
     step is fixed: the three fixed orders as Context gives them, with the second pass
     of the every-other-cell order as step 2 settled, and the drawn order as a choice
     among the positions not yet visited. A drawn order takes one draw for every visit,
     the last included, where one position is left and the draw decides nothing: 81
     draws, whatever removal does. `human-solving.allium`'s `ExactReplay` has the same
     rule for a forced outcome, "Draw even for p=0 or p=1".
   - **One visit.** Refused when removing would leave fewer givens than the bound;
     refused when it would leave the position's row or column below the floor;
     otherwise the givens without it are put to the solver, and the position is emptied
     when the verdict is one and keeps its given when it is many. Either way the
     position is not visited again.
   - **The end.** When every position has been visited, and not before. Removal does
     not stop when the count reaches the bound. Every later visit is then refused by
     the bound, which changes nothing in the result, and the draws taken do not depend
     on how removal went.
   - **What comes back.** The givens left, and that their verdict is one.
   - **The draws.** In the order they are taken, and how many each step takes. A choice
     among `n` is an index into a list in a stated order, computed from the draw as
     step 2 settled. State the arithmetic in words that leave one answer for the draw
     in Context's example, and give that draw as a worked case. `human-solving.allium`'s
     `ExactReplay` words it `floor(u*n)`; this module does not import that one and
     restates what it means.

   And it must guarantee, each as a named invariant or guarantee:

   - every given left sits in the solution grid at its position;
   - the givens left are well-posed, and the one solution is the solution grid;
   - the count of givens left is at least the bound, and every row and column holds at
     least the floor;
   - no position is visited twice and none is refilled;
   - removal asks for at most 81 verdicts and always ends;
   - the same tier, config, `generation_version` and `random_version` and the same
     draws give the same givens, for one implementation of the solver; and a seed
     stands for the draws exactly when the stream begins at index zero;
   - no draw is taken from anywhere but the boundary.

   Then one surface for the asking and one for what comes back, in the shape of
   `solver.allium`'s `Solving` and `SearchResult`.

6. **The open questions.** Seven stand today. For each of the four the basic way
   touches, add a clause that gives the basic way's answer and leaves the question
   asking for the designed way. Do not delete one and do not answer one whole.

   - Symmetry: the basic way removes one position at a time, the scheme `none`.
   - Redundant givens: the basic way keeps whatever its two restrictions stop it
     removing, and removes every other given it can.
   - The pacing question's ranges for the report's four levels: the figures stay as
     they are, and the question gains one line saying the tiers' ranges are the basic
     way's config and no answer to it.
   - What a spent budget reports: for the basic way, what step 2 settled for spent grid
     attempts.

   The technique contract, shortcut suppression and where puzzles are made are not
   touched.

7. **The decision record**, in the shape of 0012: frontmatter with
   `canonical_for: [decision_basic_generator]`; the title "Decision 0015: A basic
   generator from a published method, as the yardstick". Use the next free number on the
   day and rename the file, the title and the index row if 0015 is taken.

   - **Context:** what the maintainer asked for and why; that the designed generator
     waits on two modules and three spikes; the paper.
   - **Decision:** the four decisions under Context above, and each answer to the Open
     points.
   - **Consequences, the ones that hurt included:** one module now states two ways of
     finding givens; the specifications carry settings built on a count of givens beside
     the sentence that a count is never a rating, and say why that is not a
     contradiction; the same seed gives the same puzzle only for one implementation of
     the solver, so a solver that propagates more or breaks ties otherwise is a new
     `generation_version`; grids come from eleven givens and the solver's first
     solutions, so they are not spread evenly over all valid grids, as the paper's are
     not; tiers 3 to 5 leave their givens where a fixed order puts them.
   - **What would reopen this:** a comparison that shows the tiers do not differ as
     rated by `reach.allium`; a consumer that needs symmetry from the basic way; a
     second published method worth keeping beside this one.
   - **Related pages:** Puzzle design objectives, Layering, Specifications, decision
     0012.

   The docs validator refuses a page without its manifest entry, so the record, its
   entry in `docs/manifest.yml` (audience `contributor`, `maintainer` and `agent`) and
   its row in `docs/decisions/README.md` go in one commit. Move the index's "next
   decision" sentence on by one.

8. **The pages.** Each stays within what `docs/manifest.yml` says it owns. The three
   that describe the specifications (`specifications.md`, `work-with-the-specs.md` and
   `docs/README.md`) say what the module now states, the basic way in full and the
   designed way as a skeleton, and make no claim about what is built. The three that
   describe the engine (`purpose-and-scope.md`, `architecture.md` and `layering.md`) say
   the same and add that the basic generator's Rust module is not yet built; T32 edits
   those three again when it is.
   `docs/explanation/layering.md` also says the basic way stands on the rules and the
   solver alone; its table and its count of ten rules do not change.
   `docs/project/terminology.md` gains a row for each word step 3 added. Search `docs/`,
   `README.md`, `AGENTS.md` and `crates/pawdoku/README.md` once more for "skeleton" and
   for any sentence that says the engine has no generator, and record each further hit
   under Deviations.

9. **`docs/explanation/puzzle-design.md`.** Print the text at every `generation.allium`
   line its closing table cites and correct each number that moved. Change nothing else
   on the page.

10. `CHANGELOG.md`, under Unreleased: `generation.allium` states the basic generator,
    in two or three lines; the count of decision records.

11. Run `just check-specs`, `just analyse-specs`, `just plan-spec generation`,
    `just check-docs` and `just check`. Quote the closing lines of each and the count of
    obligations `plan-spec` prints, which T32 starts from. Fill in the hand-back notes,
    set `status: done` here and in `tickets/README.md`, commit on the ticket branch with
    short imperative subjects, and stop before pushing.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` each end
  `9 specifications, no diagnostics and no findings.`, and no `allium-ignore` directive
  exists under `docs/specs/`.
- `generation.allium` states everything step 5 lists, and each guarantee there is a
  named invariant or guarantee.
- The words "level" and "difficulty" are used of the paper's five nowhere in the module
  outside the Source note, and "band" nowhere of a range of givens.
- Seven `open question` blocks remain. Four carry a clause for the basic way; the text
  that asks for the designed way is still there in each.
- The module has four `use` lines, unchanged, and nothing the basic way states names
  `technique` or `reach`.
- `generation_version` is no longer `"generation-0"`, and its comment covers any change,
  here or in the solver, that makes the same draws give other givens, naming the
  solver's order of tied cells and what its propagation strikes.
- The module says how a draw becomes an index in words that leave one answer for the
  draw in Context's example, says which solution is the grid when the search finds two,
  and states replay for a stream that has already been drawn from.
- No page under `docs/` says the whole module is a skeleton or that the engine has no
  generator specified, decision records of their day excepted: 0012 records that the
  module was a skeleton on its date and is not edited. `purpose-and-scope.md` and
  `architecture.md` say the same thing.
- Every `generation.allium` line cited in `puzzle-design.md` holds the text its row
  describes.
- The decision record exists, is in the manifest and the index, and states the four
  decisions and each settled Open point.
- `just plan-spec generation` prints obligations, and their count is in the hand-back
  notes.
- `just check` is green; `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
just check-specs
just analyse-specs
just plan-spec generation
rg -n '^open question' docs/specs/generation.allium | cut -c1-80
rg -n -i 'skeleton' docs README.md crates/pawdoku/README.md
rg -n 'generation\.allium:[0-9]+' docs/explanation/puzzle-design.md
just check-docs
just check
git status --porcelain
```

Expected: two runs ending `9 specifications, no diagnostics and no findings.`; a list of
obligations with an empty `diagnostics` array; seven open questions; "skeleton" only
where a page says the designed generator is one, and in decision records of their day;
each cited line checked by hand; the documents gate green with one more page than
before; `All checks passed and the worktree is unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

Done on 2026-10-03 in the supplied Supacode worktree, on `b4eecc2`, the merge of the
pull request that added this ticket. The worktree's branch is
`ticket/T31-basic-generator-spec`, as Supacode named it. `.pixi/` and `.tools/bin/allium`
were present, so `just initialize` was not run.

- **Before any edit.** `just check-specs` ended
  `allium check: 9 specifications, no diagnostics and no findings.` and
  `just analyse-specs` ended
  `allium analyse: 9 specifications, no diagnostics and no findings.`
- **After the last edit.** Both end with the same two lines. No `allium-ignore`
  directive exists under `docs/specs/`.
- **`just plan-spec generation`** prints 87 obligations and an empty `diagnostics`
  array.
- **Open questions.** `rg -c '^open question' docs/specs/generation.allium` prints 7.
  Symmetry, pacing, redundant givens and the spent budget each end with the basic way's
  clause and "The question stands for the designed way"; the text that was there is
  unchanged before it. The other three are untouched.
- **Words.** `rg -n -w -i 'level|levels|difficulty|band|bands'` over the module hits the
  Source note (the paper's "levels", quoted as its word), the Vocabulary's sentence that
  tier is never level, the Excludes line on swapping bands or stacks (`sudoku.allium`'s
  band), and the two open questions that already spoke of the report's levels.
  "Difficulty" appears nowhere.
- **The worked draw.** Recomputed in Python: `6004799503160661 / 2**53 * 3` is `2.0` in
  binary64 and the floor of the exact product is 1. The largest draw below one gives
  `n - 1` for every `n` from 1 to 81, and a draw on which the two readings differ exists
  for 74 of those `n`.
- **The every-other-cell order.** Along the S path each step changes the row or the
  column by one, so the sum of row and column alternates. The paper's first pass, the 41
  positions with an even sum, is therefore the odd-numbered steps of the S path, and the
  second pass settled below is its even-numbered steps. The module states it that way.
- **`puzzle-design.md`.** Fourteen citations of `generation.allium` moved, one of them
  written `:134` with no file name, which the first pass missed and Codex's review
  found. Each new line was printed and holds what its row describes: 825, 827, 829, 831,
  833, 835 and 837 are the open questions that were 122 to 134; 71 is the Replay clause
  that was 34; 346-353 is the three versions, 109-115 before; 278-354 is the config
  block, 85-116 before.
- **"Skeleton".** `rg -n -i 'skeleton' docs README.md crates/pawdoku/README.md` hits
  only sentences that call the designed way one, and decisions 0004 and 0012, records
  of their day.
- **`just check-docs`** ended `Validated 42 pages and 43 canonical topics.`, one page
  more than before.
- **`just check`** ended `All checks passed and the worktree is unchanged.`
- **Decision 0015** was free on `origin/main` when the record was committed: the
  newest there was 0014.

### Deviations, and why

- **The module adds the word "visit"** to the five the ticket named. The one look a
  position gets needed a name of its own, because it is an entity (`Visit`) and four
  invariants speak of it. `docs/project/terminology.md` has it in the order-of-removal
  row.
- **Two rules that leave `drawing_grid` became one.** A rule for a failed attempt and a
  rule for a found grid both wrote `Generation.status`, and `allium analyse` reported a
  conflict between them, which cannot be waived. `FollowGridAttempt` is one rule with
  three ends, as `solver.allium`'s `AttendWaitingBranch` is one with two.
- **The solver is reached through black boxes**, `found_grid` and `verdict_is_one`, each
  pinned by prose to `solver.allium`'s verdict and solved branches, and not through a
  `solver/Solve` emission. `board.allium`'s `solution_digit_at` is the precedent.
- **No further page called the module a skeleton or said the engine has no generator.**
  `purpose-and-scope.md` keeps its bullet's heading, "No puzzle generation yet", which
  is still true of the crate; the bullet says the basic generator is specified.
- **`git fetch origin main`** was run once, to check that 0015 was still free. It reads
  and writes nothing outside `.git`.
- **A Codex adversarial review was run on 2026-10-03**, at the maintainer's request,
  over the first four commits, and answered in a fifth. Fixed: the refusal's guard and
  the drawn visit's position were read from the state the rule itself produces, and are
  now bound before `ensures`; the next draw's number likewise; `GenerationResult`
  showed a refused generation's givens; `found_grid` had no stated answer for a short
  seeding; `AlwaysEnds` and `DrawsInOrder` overstated the solver calls and the draws of
  a refused generation; a stream that runs out had no stated outcome, and
  `AStreamThatRunsOut` now gives T32's; the config says its guarantees are stated of
  its figures as they stand. The review also found three wrong line citations in
  `puzzle-design.md` that predate this ticket, of `sudoku.allium:169`,
  `board.allium:785` and `architecture.md:84-88`. Step 9 allows no other change to
  that page, so they are handed back below.
- **Nothing was pushed.**

### Names T32 needs

**Words.** Tier, bound, floor, order of removal, visit, grid attempt.

**Enumerations.** `Tier { tier_1 | tier_2 | tier_3 | tier_4 | tier_5 }`.
`RemovalOrder { drawn | every_other | s_path | row_by_row }`: the paper's "Randomizing
globally", "Jumping one cell", 'Wandering along "S"' and "Left to Right then Top to
Bottom", for tiers 1 and 2, 3, 4 and 5.

**Entities.** `Generation` (status `drawing_grid | removing | finished | refused`;
`tier`, `order`, `draws_taken`, `solution`, `bound`, `givens`), `GridAttempt` (status
`seeding | failed | found`; `number`, `grid`), `Visit` (status
`pending | refused_by_bound | refused_by_floor | kept | emptied`; `step`, `position`).

**Rules.** `BeginGeneration`, `OpenFirstGridAttempt`, `SettleGridAttempt`,
`FollowGridAttempt`, `BeginVisit`, `DecideVisit`, `FinishRemoval`.

**Black boxes**, each pinned in a comment where it is first used: `draw_at`,
`index_among`, `order_of`, `drawn_seeding`, `found_grid`, `nth_unvisited`,
`order_position`, `givens_in_row`, `givens_in_column`, `without_position`,
`verdict_is_one`.

**Invariants.** `TheSolutionGridIsFull`, `GivensSitInTheSolutionGrid`,
`GivensLeftAreWellPosed`, `TheBoundIsInTheTiersRange`, `GivensStayAtOrAboveTheBound`,
`RowsAndColumnsKeepTheFloor`, `NoPositionIsVisitedTwice`, `VisitsAreNumberedOnce`,
`EmptiedPositionsStayEmpty`, `UnemptiedPositionsKeepTheirGiven`, `RemovalIsBounded`,
`GridAttemptsStayWithinTheLimit`, `EndsAreEarned`.

**Surfaces.** `Generating`, which provides `Generate(tier)` and carries the guarantees
`AlwaysEnds`, `OneSolutionAndItIsTheGrid`, `TheRestrictionsHold`, `EveryPositionOnce`,
`DrawsInOrder`, `AStreamThatRunsOut`, `SameDrawsSameGivens`, `OnlyTheBoundary` and
`ATierClaimsNothing`; and
`GenerationResult`, which exposes the tier, the status, `draws_taken` and, of a finished
generation, the count of givens and each given.

**Config.**

| Figure | Value |
| --- | --- |
| `tier_1_fewest_givens`, `tier_1_most_givens`, `tier_1_floor` | 51, 60, 5 |
| `tier_2_fewest_givens`, `tier_2_most_givens`, `tier_2_floor` | 36, 49, 4 |
| `tier_3_fewest_givens`, `tier_3_most_givens`, `tier_3_floor` | 32, 35, 3 |
| `tier_4_fewest_givens`, `tier_4_most_givens`, `tier_4_floor` | 28, 31, 2 |
| `tier_5_fewest_givens`, `tier_5_most_givens`, `tier_5_floor` | 22, 27, 0 |
| `seeded_givens` | 11 |
| `grid_attempt_limit` | 100 |
| `generation_version` | `"generation-1"` |

**What follows for `generate`.**

- The figure for grid attempts is config's, with a default, so `generate(tier, stream)`
  keeps two arguments.
- **The draws, in order.** For each grid attempt, for each seeded given, one draw for
  its position among the empty positions in row order and one for its digit among those
  its row, column and box allow, lowest first: 22 for a full seeding. A position with no
  digit left takes its position draw and no digit draw, and ends the attempt: `2k + 1`
  draws after `k` givens. A refused generation takes those and no more. One that
  finds a grid then takes one draw for the bound, `fewest + index`, and, for tiers 1 and
  2 alone, one draw a visit among the unvisited positions in row order, the 81st
  included. A first attempt that succeeds takes 23 draws in tiers 3 to 5 and 104 in
  tiers 1 and 2.
- **A stream that runs out** ends `generate` with the boundary's own error at that
  draw. It is not the module's refusal and not a failed attempt.
- **A draw's index** is `(u * n) as usize` on `f64`: the binary64 product, truncated.
  Not exact arithmetic.
- **Two solutions.** The grid is the one with the lower digit at the first position, row
  by row, where the two differ. It is not the first the search found.
- **A full seeding is put to the solver** whatever it holds; a short one is not, so an
  attempt asks the solver at most once.
- **Removal runs all 81 visits**, and what it reached comes back, in the tier's range or
  above it. There is no retry.
- **Spent attempts** are the one refusal of the module's own, with nothing to show. The
  draws the failed attempts took stay taken.
- **The first visit always empties its position**, so a finished generation's givens
  number at most 80 and `SetPuzzle` accepts them.
- **87 obligations** from `just plan-spec generation`, with an empty `diagnostics`
  array. Two of them are not the basic way's: `config-default.step_budget` and
  `config-default.candidate_limit`, the designed way's two figures, which were declared
  before this ticket and have no default to verify. T32's step 2 says to stop if an
  obligation of the designed way appears; these two are what it will see, and they ask
  nothing of T32.

### Handed back

- **S08.** Its Context calls `generation.allium` a skeleton and its Verification expects
  "the seven open questions". It now finds a module that states the basic way in full
  and the designed way as a skeleton; the seven open questions are all there, four with
  a closing clause for the basic way, at lines 825 to 837. Its control row, removal "at
  random, then the verdict and `Rate`", is not the basic way: that row draws its order,
  has no bound and no floor, and is put to `Rate`. Tiers 1 and 2 are its nearest kin.
- **T20.** Its draft writes "triggers, guarantees and fixtures on the skeleton". It now
  writes a second way beside the basic one. It inherits the vocabulary, the black boxes
  `draw_at` and `index_among`, and the replay clause `SameDrawsSameGivens`, and must say
  of each rule it adds which way it is: the Rules section opens "Every rule here is the
  basic way's", and `Generate(tier)` is the basic way's stimulus. `step_budget` and
  `candidate_limit` are still declared and still its own.
- **`human-solving.allium`.** Its `ExactReplay` words a choice among `n` as
  `floor(u*n)`, which has two readings, and says elsewhere that "no platform rounding
  changes a choice". This ticket chose the binary64 product truncated toward zero for
  `generation.allium`, on the maintainer's answer. Whoever builds that module chooses
  for it; if it chooses exact arithmetic, two modules read one stream by two rules, and
  decision 0015 names that as a reason to reopen.
- **T32.** Its Context table and its "One entry" paragraph were written before the Open
  points were settled. Each agrees with what was settled, bar the word "visit", which is
  new. Its measurement step may move `grid_attempt_limit`.
- **`puzzle-design.md`'s other citations.** Three in its closing table were wrong
  before this ticket and are not this ticket's to change: `sudoku.allium:169` should be
  172, `board.allium:785` should be 797, and `architecture.md:84-88` should be the
  generation paragraph, 307-313 today. A `main` follow-up.
- **T33.** This ticket took decision 0015. If T33's pull request merges second, it
  merges `main` first and takes 0016, and moves the index's "next decision" sentence on.

### Open points settled

Each by the maintainer, on 2026-10-03.

- **The word for the paper's five.** Tier, numbered 1 to 5, as proposed.
- **"More than" in the two restrictions.** Both mean at least.
- **Tier 1's range.** 51 to 60; 50 is in no range.
- **The second pass of the every-other-cell order.** The skipped positions along the
  same path, and the module says the reading is its own.
- **How the eleven givens are drawn.** A position among the empty cells in row order,
  then a digit among those its row, column and box allow, lowest first. A position with
  no digit left ends the attempt, which is counted.
- **Which solution is the grid when the search finds two.** The one that comes first
  reading the grid row by row. `solver.allium` is not edited.
- **How a draw becomes an index.** The binary64 product of the draw and `n`, truncated
  toward zero, with the worked draw. The maintainer was shown `human-solving.allium`'s
  sentence on exact arithmetic beside the choice.
- **A stream already drawn from.** Replay is stated over draws; a seed stands for them
  when the stream begins at index zero.
- **Removal that ends above the tier's range.** What was reached comes back; nothing is
  retried.
- **Which limits the basic way reads.** The figure for grid attempts alone, in config
  with a default of 100.
- **What spent grid attempts report.** A refusal with nothing to show.
- **The two frozen files.** Leave given for both `docs/manifest.yml` and
  `docs/README.md`.
- **The decision record.** Kept.
- **Paths T33 also edits.** T33's deviation is followed, and this ticket took 0015.

## Open points

Each is the maintainer's. The proposal beside it is the ticket writer's, made on
2026-10-02.

- **The word for the paper's five.** Proposed: tier, numbered 1 to 5, with the paper's
  names quoted once in the Source note as the paper's own.
- **"More than" in the two restrictions.** The paper says the givens left "must be more
  than" the bound and each row and column "more than" the lower bound. Its own example
  reads otherwise: a puzzle with "total 22 givens, within the range in level 5", and
  "0 givens in Row 1, reach the lower bound in Level 5". Proposed: both mean at least.
- **Tier 1's range.** The paper says only "more than 50", so the range has no upper end
  to draw a bound within, and 50 itself is in no range. Proposed: 51 to 60, with the
  gap at 50 kept as the paper has it.
- **The second pass of the every-other-cell order.** The figure shows one pass.
  Proposed: the skipped positions follow along the same path, (1,2), (1,4), (1,6),
  (1,8), (2,9), (2,7) and on, and the module says this pass is its own reading.
- **How the eleven givens are drawn.** The paper does not say, and replay needs it.
  Proposed: a position among the empty cells, in row order; then a digit among those no
  earlier given holds in its row, column or box, lowest first. A position with no digit
  left ends the attempt, which is counted and begun again.
- **Which solution is the grid when the search finds two.** `solver.allium` exposes the
  solved branches with no order. Proposed: the one that comes first reading the grid row
  by row, a rule this module can state without reaching into the solver. The other
  choice is the first the search found, which needs a sentence in `solver.allium` to
  give the branches an order, and that file is not in this ticket.
- **How a draw becomes an index.** Proposed: the binary64 product of the draw and `n`,
  truncated toward zero, stated as such with the worked draw from Context, so that an
  implementation in exact arithmetic knows it differs. The other choice is the floor of
  the exact product, which T32 would compute from the draw's bits.
- **A stream already drawn from.** Proposed: replay is stated over draws, which holds
  wherever the stream stands, and the module adds that a seed stands for them when the
  stream begins at index zero. A caller that makes several puzzles from one stream
  records the index each began at beside the seed.
- **Removal that ends above the tier's range.** The floor or the verdict can stop
  removal early. Proposed: return what was reached and retry nothing, as the paper's
  flow does; the count of givens says whether the range was met. This is what makes the
  paper's "rate of success" something a comparison can measure.
- **Which limits the basic way reads.** Proposed: the figure for grid attempts alone;
  `step_budget` and `candidate_limit` stay the designed way's. Whether the figure is the
  caller's with no default, as those two are, or carries one (100 is suggested, and no
  measurement stands behind it) is part of this point.
- **What spent grid attempts report.** Proposed: a refusal with nothing to show, since
  no grid exists yet. That answers the module's spent-budget question for the basic way
  only.
- **The two frozen files.** `docs/manifest.yml` and `docs/README.md` are frozen under
  CONVENTIONS.md §11. T19 edited both with the maintainer's leave, and T22 and T23 each
  added a decision record's manifest entry in their own commit. This ticket lists both;
  if leave is not given for `docs/README.md`, hand that edit back as a `main` follow-up
  and say so under Deviations.
- **The decision record.** It is in scope because the tiers sit beside "never a rating"
  and that wants a reason on record. The maintainer may strike it as more than a minimal
  ask; the Source note in the module then carries the reason alone.
- **Paths T33 also edits.** T33, the API reference on GitHub Pages, is open beside this
  ticket. It edits `CHANGELOG.md`, `docs/manifest.yml`, `docs/decisions/README.md` and
  `docs/README.md`, and its draft takes decision 0015 as this one does. CONVENTIONS.md
  §11 forbids two open lanes sharing a path. T33's Open points record the deviation and
  this ticket follows it: each shared edit is a row, an entry or a sentence, and
  whichever pull request merges second merges `main` first and takes the next decision
  number, in the file's name, its title, its index row and its manifest entry.
