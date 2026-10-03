---
id: T31
title: "Spec change: the basic generator in generation, a published method of removal checked by the verdict"
status: open
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
- **Replay stands on the solver's order.** Eleven givens nearly always have many
  solutions. The search stops at two, and which solutions it reaches depends on the
  order "in which tied cells and tied branches are taken", which `solver.allium` leaves
  to the implementation and requires to be "the same each time". So the same seed gives
  the same solution grid for one implementation of the solver, and a change to that
  order gives other givens. The module's `generation_version` comment already says it
  is raised "whenever removal or acceptance changes so that a seed gives other givens";
  it must now name this cause too.
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
   the figure for grid attempts. Raise `generation_version` and extend its comment: a
   change to the order in which the solver takes tied cells and branches is a change
   that makes a seed give other givens. Run `just check-specs`.

5. **The basic way's constructs.** Write them with the `tend` skill, at the level
   `solver.allium` is written: what happens and what is guaranteed, not how it is
   stored. Run `just check-specs` after each block. The module must say, in whatever
   constructs fit:

   - **What is asked.** A tier and a seed. Nothing else chooses the result.
   - **A grid attempt.** Eleven positions and their digits are drawn as step 2 settled;
     the givens go to `solver.allium`; a solution it finds is the solution grid; a
     verdict of none ends the attempt and another begins.
   - **Spent attempts.** What is reported when the figure is reached with no grid.
   - **The bound.** One draw inside the tier's range, after the grid.
   - **The order.** The four orders, each stated so that the position visited at every
     step is fixed: the three fixed orders as Context gives them, with the second pass
     of the every-other-cell order as step 2 settled, and the drawn order as a choice
     among the positions not yet visited.
   - **One visit.** Refused when removing would leave fewer givens than the bound;
     refused when it would leave the position's row or column below the floor;
     otherwise the givens without it are put to the solver, and the position is emptied
     when the verdict is one and keeps its given when it is many. Either way the
     position is not visited again.
   - **The end.** When every position has been visited. Say whether removal may stop
     sooner once the count equals the bound, since nothing more can then be removed.
   - **What comes back.** The givens left, and that their verdict is one.
   - **The draws.** In the order they are taken, and how many each step takes. A choice
     among `n` is `floor(u * n)` over a list in a stated order, the wording
     `human-solving.allium`'s `ExactReplay` uses, restated here because this module does
     not import that one.

   And it must guarantee, each as a named invariant or guarantee:

   - every given left sits in the solution grid at its position;
   - the givens left are well-posed, and the one solution is the solution grid;
   - the count of givens left is at least the bound, and every row and column holds at
     least the floor;
   - no position is visited twice and none is refilled;
   - removal asks for at most 81 verdicts and always ends;
   - the same tier, seed, config, `generation_version` and `random_version` give the
     same givens, for one order of the solver's tied cells and branches;
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
     contradiction; the same seed gives the same puzzle only while the solver's order
     of tied cells holds; grids come from eleven givens and the solver's first
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
- `generation_version` is no longer `"generation-0"`, and its comment names the solver's
  order as a cause.
- No page under `docs/` says the whole module is a skeleton or that the engine has no
  generator specified; `purpose-and-scope.md` and `architecture.md` say the same thing.
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

### Deviations, and why

### Names T32 needs

The words the module settled on, each construct's name, the config figures, and the
obligation count from `just plan-spec generation`.

### Handed back

To be filled in. At the least:

- **S08.** Its Context calls `generation.allium` a skeleton and its Verification expects
  "the seven open questions". Say what it finds instead. Its control row, removal "at
  random, then the verdict and `Rate`", is not the basic way: that row draws its order,
  has no bound and no floor, and is put to `Rate`. Tiers 1 and 2 are its nearest kin.
- **T20.** Its draft writes "triggers, guarantees and fixtures on the skeleton". It now
  writes a second way beside the basic one, and inherits the vocabulary and the replay
  clause this ticket wrote.

### Open points settled

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
