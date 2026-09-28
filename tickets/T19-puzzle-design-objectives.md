---
id: T19
title: "Puzzle design objectives: the research report in the docs, the gaps in the specs, and the generation module's skeleton"
status: open
depends_on: [T06, T08, T12]
parallel_with: []
branch: ticket/t19-puzzle-design-objectives
estimated_size: L
---

# T19: Puzzle design objectives: the research report in the docs, the gaps in the specs, and the generation module's skeleton

## Context

The maintainer has a research report, "What Makes a Sudoku Puzzle Fun? A Rigorous Guide
to Hand-Designing Elegant Sudoku", received on 2026-09-27 and carried in full at the end
of this Context. Its central claim: a good Sudoku is a designed sequence of deductions,
so the givens are the interface and the solve path is the authored object. From that it
argues that difficulty is a vector (technique depth, spotting difficulty, dependency
length, bottleneck narrowness, candidate volume, and the penalty for looking in the wrong
place), never a scalar; that the count of givens is not difficulty; that the hardest
named technique is not the whole difficulty; and that pacing, elegance, novelty, the
layout of the givens and a "technique contract" with the intended solver are what
separate a puzzle that is merely solvable from one that feels composed. Two of its
sources are already this repository's: Pelánek 2014 sits in the evidence table of
`docs/explanation/human-solving.md`, and HoDoKu is linked from that page's technique
coverage table. A third, Nishikawa and Toda on generating givens solvable by a stated
strategy set, is the formal analogue of the generation this engine will do.

The engine will generate puzzles, and the maintainer wants those puzzles to meet both the
design objectives the specifications already hold and the ones the report adds. This
ticket is the first step: the report lands in the handbook cleaned to the house rules, the
gap between the report and the specifications is recorded objective by objective, the
gaps that are product decisions become `open question` blocks in the module that owns
each, the generation module gets its skeleton, and the follow-up build tickets are
drafted. No Rust is written.

### What the maintainer decided on 2026-09-27

These are settled; the executing agent does not reopen them.

- **A build ticket, not a spike.** Docs and spec changes land in this ticket. Follow-ups
  are drafted in the hand-back notes.
- **The engine generates.** A `generation.allium` module is planned and, per
  `AGENTS.md`, is specified before it is built. This ticket lands its skeleton.
- **Where puzzles are made is open.** Whether the game ships a curated pre-generated
  set, generates on device, or both, is recorded as an open question, not decided. The
  module must therefore be written so it can run anywhere, under a step budget.
- **Classifier models are development-time tools only.** In the maintainer's words:
  "Jev and other such things that require requests to third party servers/api's can not
  be part of the app/runtime. Only dev time, so if we were to use Jev to help generate
  puzzles we would need to pre generate them or use Jev to help test / judge their output
  such so that we can tune their parameters." Jev is TypeSafe's "System One" model
  (<https://typesafe.ai/blog/introducing-system-one-models-and-jev>): a hosted API that
  returns typed, structured values with calibrated probabilities, meant for
  classification, routing and scoring. The core is `no_std` with no network, and
  `AGENTS.md` invariant 2 makes randomness the engine's only effect, so nothing in the
  engine names, calls or depends on such a model. What the engine owes a dev-time judge is
  stable, serialisable outputs to judge.
- **The report lives under `docs/`**, as a cleaned page with a manifest entry, not as a
  raw file under `tickets/` and not only as this ticket's appendix.

### What the specifications already hold

Verified against `main` at `ffcdca0` on 2026-09-27; re-check each line number on the day.

- No solver, generator, rating or catalogue exists in code. `crates/pawdoku/src/` holds
  `lib.rs` (`SIDE`) and `random.rs` (the randomness boundary: `RandomStream`,
  `SeededStream`, `ReplayStream`). Everything else is specification.
- Eight modules under `docs/specs/`: `sudoku` (the rules), `board` (a puzzle in play),
  `solver` (the verdict: none, one or many solutions, stopping at the second),
  `technique` (29 techniques in ladder order, rank 1 to 29, with provisional loads;
  `novice` and `expert` profile presets), and four player models that never import each
  other: `reach` (a limit hides a deduction; `Rate` and `NextStep`), `effort` (a limit
  makes a deduction dear; `Price`), `lapse` (a limit leads to error; `Tackle`) and
  `human-solving` (a seeded, fallible simulation; `Assess` returns distributions).
- Rating refuses a universal scalar. `human-solving.allium` line 33 excludes "a single
  universal difficulty number"; its `Interpretation` guarantee (1184-1191) forbids pooling
  differing conditions as one rating; `docs/project/terminology.md` line 83 defines an
  assessment "rather than a universal scalar difficulty". Nothing in this ticket may
  propose a fifth model or a combined "fun score".
- No module says anything about where the givens sit (symmetry), about pacing, or about
  how a generator chooses. `technique.allium` line 1113 is the only "symmetry" in the
  specs, and it is fixture validation.
- The docs disagree on generation. `docs/project/purpose-and-scope.md` lines 40-41:
  "**No puzzle generation yet.** How a setter finds givens is excluded from
  `sudoku.allium`; generation is a later module, specified before it is built."
  `docs/explanation/architecture.md` lines 82-84: "no puzzle generation for now. Those
  are not deferred; they are out of scope". The first is right; the second is corrected
  here.
- `sudoku.allium` already anticipates a generator: its `PuzzleSetting` surface (315-323)
  says "Where a puzzle comes from: a person, a book or a generator" and guarantees
  `OnlyWellPosedPuzzlesArePosed`. `solver.allium` line 606 ("a setter checking its work,
  or a generator") and `reach.allium` lines 342-343 ("a generator grading what it has
  made") say the same at their surfaces.
- No page in `docs/manifest.yml` owns puzzle design, difficulty or generation. The
  nearest are `project_non_goals` (holds the "no generation yet" line),
  `sudoku_solving_strategies` (`docs/explanation/solving-sudoku.md`, which already says at
  lines 440 and 450 that few givens need not mean hard and that the count is a poor proxy)
  and `human_solving_model`.
- Research sources live in one place: the table "Research and the claims it supports"
  in `docs/explanation/human-solving.md` (lines 88-107), three columns: Evidence, What it
  motivates here, What it does not establish. Every row has a checked link.

### Rules the report is held to

- **Given, never clue** (`sudoku.allium` line 45; `docs/project/terminology.md` line
  33). The report says "clue" throughout. Every sentence this repository writes says
  "given"; "clue" survives only inside a paper's title ("There is no 16-Clue Sudoku") and
  inside the received text below.
- **Steps, never time.** The report's playtest measures are in minutes. The engine
  expresses every limit as a step budget (`AGENTS.md` invariant 2), and
  `human-solving.allium` lines 32-33 exclude "predictions in minutes". The page may
  describe minutes as external practice, with one sentence saying the engine's analogue
  is steps.
- **Unresolved product decisions become `open question` blocks**, never drafted rules
  (`AGENTS.md`, "Safety and authority"). The spec output of this ticket is open questions
  plus a skeleton.
- **The report's citations are unverified.** Its markers were tool residue and are
  removed below. A source enters the handbook only after the executing agent has opened
  it. What cannot be verified is dropped, not kept.
- **The four study puzzles are unverified.** The report says they were "generated and
  analyzed for this report" and "independently checked" by its author. No solver exists
  here. They may be kept as candidate fixtures, labelled unverified, and nothing may
  assert they are unique or that their stated paths hold.
- **The frozen-file rule does not stop this ticket.** CONVENTIONS.md §11 and §3 (line
  261) froze `docs/manifest.yml` and `docs/README.md` for the parallel bootstrap lanes
  T01 to T09, and T12, running after T11, still handed its `docs/README.md` edit back as
  a `main` follow-up. T19 departs from that: the maintainer, approving this ticket's
  plan on 2026-09-27, authorised it to edit both files itself, so both are in Files
  touched and no follow-up is raised for them.

### Read first

`AGENTS.md`; `docs/reference/documentation-contract.md`; `docs/explanation/specifications.md`;
`docs/how-to/work-with-the-specs.md`; `docs/decisions/README.md` ("Writing a new one");
`.agents/skills/spec-change/SKILL.md`; `.agents/skills/review-docs/SKILL.md`;
`docs/specs/sudoku.allium` header (1-60) and `PuzzleSetting` (315-323);
`docs/specs/technique.allium` lines 27-28 (the ladder is not a cognitive law), 50-52
(deferred families), 543-600 (the catalogue and loads), 676-710 (presets), 1160-1162
(open questions); `docs/specs/reach.allium` `RunResult` (396-415) and open questions
(421-425); `docs/specs/effort.allium` open questions (304-316);
`docs/specs/human-solving.allium` `AssessmentSummary` and `AssessmentDetail` (418-438),
presets (483-560), `Interpretation` (1184-1191) and open questions (1272-1282);
`docs/explanation/human-solving.md` "Research and the claims it supports" (88-107) and
"Named technique coverage" (263 on); `docs/explanation/solving-sudoku.md` lines 436-454;
`docs/README.md` "This library" (69-77).

### The objective map

One row per objective: the report's, and the ones the specifications already care about
that the report does not name. Classes: **satisfied now** (the specs already do it);
**adjustment** (an addition to what is already planned, derivable from existing outputs);
**algorithmic** (a new mechanism the specs must state); **classifier** (a development-time
judge can help, over serialised engine outputs); **product** (a decision the maintainer
owns, recorded as an open question). "New page" is `docs/explanation/puzzle-design.md`.
Line numbers are at `ffcdca0`.

| Objective | What the specs already do | Class | Where it lands | Note |
| --- | --- | --- | --- | --- |
| Well-posed givens (exactly one solution) | `sudoku.allium:169` requires `solution_count = 1`; `sudoku.allium:316-323` `PuzzleSetting` guarantees only well-posed puzzles are posed; `solver.allium:292,617,623` stops at the second solution and ties the verdict to it. | satisfied now | none | The report's "combinatorial validity" is already the floor. A generator's output goes through `Solving` (`solver.allium:607`) before it is posed. |
| Partially completed grid; no maximum on givens | `sudoku.allium:157` requires `givens.count < side*side`; 17 is named as the fewest that can pass (`sudoku.allium:166`). | satisfied now | none | No target count exists and none should become a rule; see the next row. |
| Given count is not difficulty | `docs/explanation/solving-sudoku.md:440` and `:450` already say few givens need not mean hard and that count is a poor proxy. No spec reads the count as a rating. | satisfied now | new page restates the claim with the report's 32-given singles-only example marked unverified | The generator must never rate by count; a generation open question records the report's per-level count ranges as construction starting points only. |
| Fairness / technique contract (required techniques, forbidden techniques, no guessing) | `technique.allium:676-710` profiles carry a repertoire; `reach.allium:153` names the hardest step; `reach.allium:359,369` stalled means this profile sees nothing more and limits only hide; `effort.allium:84,290` counts escalations past the profile. | adjustment; algorithmic | `generation.allium` (new): a contract of required-at-most and forbidden techniques as a generation constraint, in the sense of Nishikawa and Toda's strategy-solvable generation; `reach.allium:425` open question already asks what a stalled rating names | Rating a candidate against a profile exists. Generating *to* a contract is new: accept a candidate only if a `Rate` run for the target profile ends solved and its hardest step is within the contract. |
| Solvability by a human route (strategic validity) | `reach.allium:344` `Rating`, `effort.allium:261` `Pricing`, `lapse.allium:428` `Tackling`, `human-solving.allium:1152` `AssessmentService` each produce a route or a distribution of attempts. | satisfied now | none | Four models, none of them a scalar. The report's "experiential validity" (people can find it) stays a human playtest; see the playtest row. |
| Challenge (hardest required step) | `reach.allium:153` `is_hardest`; `effort.allium:129` `is_dearest`; ladder order at `technique.allium:543-547`. | satisfied now | none | This is the report's T (technique depth). |
| Difficulty vector T, S, L, B, V, P | T: `reach.allium:153`. S (spotting): `effort.allium:106-108` `search_effort`, a black box, open at `effort.allium:306`; `human-solving.allium:829` notice draws. L (dependency length): `human-solving.allium:437` `dependency_depth`. B (bottleneck narrowness), V (candidate volume), P (wrong-area penalty): none. | adjustment (T, S, L); algorithmic (B, V, P) | new page maps the vector onto the four models; `reach.allium` gains an open question on exposing per-step alternatives (B) and candidate counts (V); `human-solving.allium:1054` `EffortAccounting` already pays fruitless searches (P) | No fifth model and no combined scalar: `human-solving.allium:33` excludes "a single universal difficulty number". Each component stays reported separately. |
| Pacing: routine-run length between hook steps | `reach.allium:398-415` `RunResult` exposes every step in order with its technique; `effort.allium:281-298` `PricedRun` likewise with costs. | adjustment | new page defines "hook" as a step whose technique rank exceeds the run's median or the profile's floor; a derived measure over `RunResult.steps`, not a new run | Derivable from existing outputs by a consumer. The ticket decides whether the derivation is specified (a `Pacing` value in `reach.allium`) or left to a dev-time tool. |
| Pacing: cascade size after a step | `reach.allium:398-415` exposes steps but not which earlier step licensed which later one. | algorithmic | `reach.allium` open question: should a step record the step(s) whose eliminations it depends on, so consequence counts can be read | Pelánek's dependency structure. Without dependency edges a cascade is only "steps until the next hook", which conflates causation with order. |
| Pacing: dependency length (prerequisites before the centrepiece) | `human-solving.allium:437` `dependency_depth` distribution. | adjustment | same open question as the cascade row, so `reach.allium` can report it deterministically | Today only the seeded model reports it; the deterministic models cannot. |
| Satisfaction: the key deduction has consequences | None; a step's eliminations are in its proof but no count of what they unlock. | algorithmic | `reach.allium` dependency edges (above); new page states the objective | Measured as cascade size once dependency edges exist. |
| Flow: no long flat stretches, no unsignalled cliffs | `effort.allium:84` escalations mark cliffs relative to a profile; long runs of rank 1 to 4 steps are visible in `RunResult`. | adjustment | new page; generation acceptance uses run-length thresholds from `RunResult`, figures as an open question | The report's story arc (orientation, compression, resistance, hook, release, echo, closure) is a target shape over the step sequence. |
| Shortcut suppression (a simpler route bypasses the intended hook) | `reach.allium:41-43` takes the first licensed deduction in ladder order, so a shortcut is what `Rate` finds; `reach.allium:423` open question notes order can change what is reachable. | algorithmic | `generation.allium`: a candidate is kept only if the intended technique is *necessary*, checked by rating with that technique removed from the repertoire and confirming the run stalls | Uses `LimitsOnlyHide` (`reach.allium:369`) in reverse. Cheap to specify, dear to run; a step budget applies. |
| Path-dependence of difficulty across alternate routes | `reach.allium:423` open question on monotonicity; `technique.allium:891` `SameGridSameDeductions`; `human-solving.allium:1184-1191` reports distributions, never one route. | adjustment | new page: rate with several profiles and orders, report the spread; `human-solving.allium` already does this by seed | The report's "one tester stalled on an unintended X-Wing" is a profile with a different scanning order. No new mechanism. |
| Aesthetics: given symmetry (180-degree rotation, orbits, centre cell) | None. The only "symmetry" in the specs is fixture validation at `technique.allium:1113`. | algorithmic; product | `generation.allium`: a symmetry scheme as a generation config with removal by orbit; which schemes and whether symmetry is default is an open question | Purely geometric, deterministic, easy to specify and test. Symmetry ranks below uniqueness and the technique contract in the report's hierarchy; the spec must say so. |
| Aesthetics: layout looks deliberate, no awkward isolated givens | None. | classifier; product | new page names it; a dev-time judge over a rendered or serialised grid | A human or a classifier judges; nothing in the engine can. |
| Elegance (one relationship does much work; theme recurs) | None beyond proof size (`technique.allium:597-600` load counts propositions). | algorithmic (proxy); classifier | new page: proxy is cascade size divided by proof load; a dev-time judge over the solve log for the rest | The proxy needs the dependency edges above. Whether a puzzle "is about" something is a judgement, not a measurement. |
| Novelty (a memorable idea, not a random assortment) | None. | classifier; product | new page; dev-time judge over `RunResult` or `PricedRun` serialised; a curated set is where novelty is enforced | A generator can bound the count of distinct advanced families per puzzle (algorithmic); "memorable" is a judgement. |
| Aha quality (preparation, compression, consequence) | Preparation and consequence follow from dependency edges; compression is proof load at `technique.allium:597`. | adjustment; classifier | new page; measured from the same derived pacing values | No new run kind. |
| Finish (closing cascade, not bookkeeping) | Visible in `RunResult` as the rank profile of the last steps. | adjustment | new page; a generation acceptance threshold, figure open | Derived measure. |
| Player-level targets (beginner, intermediate, advanced, expert) | `technique.allium:676-710` novice and expert presets; `human-solving.allium:483-560` four presets; the presets do not meet (`human-solving.allium:1278` open question). | adjustment; product | `technique.allium` or `human-solving.allium`: whether two more deterministic presets (intermediate, advanced) are added, and how the report's four levels map onto them | The report's per-level technique lists match the ladder families; the mapping is a table on the new page, and the preset question stays open. |
| Difficulty labels are audience contracts, not properties | `human-solving.allium:33` excludes a universal number; `:1184-1191` `Interpretation` forbids pooling conditions. | satisfied now | new page cites it | Labels belong to the game, which picks a profile per label. |
| Technique vocabulary (singles to forcing) | 29 techniques at `technique.allium:543-668`; deferred families at `technique.allium:50-52`; chain limits at `technique.allium:529-530`. | satisfied now | none | The report's taxonomy is a subset of the catalogue. ALSO and forcing nets are deferred, which caps the expert level the engine can rate. |
| Contradiction reasoning is opt-in, short, and declared | `forcing_chain` bounded by `max_forcing_paths` and `max_chain_links` (`technique.allium:1001-1003`); a profile omits it from its repertoire. | satisfied now | none | The report's expert-audience taste question is a profile choice. |
| Guessing is not a deduction | `lapse.allium:118-119` counts guesses and withdrawn guesses; `reach.allium:38` stalls rather than guesses; `solver.allium` guesses but explains nothing (`:42-45`). | satisfied now | none | "Guessing branching" versus "logical branching" is already the reach/lapse split. |
| Uniqueness techniques need a promise | `technique.allium:878-881`; `reach.allium:205` promises only when rating. | satisfied now | none | A generator that has verified uniqueness may rate with the promise; the report does not raise this. |
| Hints | `reach.allium:376-380` `Hinting`, `NextStep`. | satisfied now | none | Out of the report's scope; kept as an existing objective. |
| Deterministic runs, exact replay | `reach.allium:365`, `lapse.allium:456` `SameInputSameRun`; `human-solving.allium:1093` `ExactReplay`; `technique.allium:53-54`. | satisfied now | `generation.allium` must inherit it: same seed, same profile, same givens | Generation draws from the randomness boundary (`crates/pawdoku/src/random.rs`) and nothing else. |
| Step budgets, never time | `human-solving.allium:20,87,93` limits; `technique.allium:238-239` budget and fatigue; the report's minutes are excluded at `human-solving.allium:32-33`. | satisfied now | `generation.allium` states a step budget for the search and a candidate limit | The report's "total solving time" and "longest no-progress interval" become steps and longest run of looks without a step. |
| Playtest protocol (blind solve, stall location, technique at stall, recall) | `human-solving.allium:418-438` reports stops, technique counts, obstacles, guesses; `docs/explanation/human-solving.md` names calibration by consented human traces. | adjustment; product | new page carries the protocol and the stall measures in steps; whether the game records anything is `board.allium:785`'s open question and the game's | The simulated assessment is the in-engine analogue; human playtesting is a game and process concern, recorded here so the measures line up. |
| The 0-4 rubric (challenge fit, fairness, pacing, elegance, novelty, visual aesthetics, aha, finish) | None. | classifier; product | new page, as a dev-time judging schema over serialised `RunResult`, `PricedRun`, `TackledRun` and `AssessmentSummary` plus the givens; not a spec value | Eight independent dimensions, no sum. A calibrated-probability classifier such as Jev is a candidate judge at dev time only; nothing calls out at runtime. |
| Curated set versus on-device generation | None; `docs/project/purpose-and-scope.md:40-41` and `docs/explanation/architecture.md:82-84` disagree on whether generation is planned. | product | `docs/project/purpose-and-scope.md`, `docs/explanation/architecture.md`: both say generation is a planned module; `generation.allium` open question on where puzzles are made | Decided by the maintainer on 2026-09-27: the engine specifies generation; where puzzles are made stays open. |
| Sources (Pelánek, Nishikawa and Toda, McGuire et al., GameFlow, Ryan et al., Snyder, Nikoli, HoDoKu, WPF, CtC) | `docs/explanation/human-solving.md:88-107` holds the evidence table; Pelánek 2014 is already in it. | adjustment | the evidence table on the new page, one row per source, each verified before it is kept | The report's citation markers are dangling; a source enters only with a checked link. |
| Study puzzles (four grids, 32/32/29/25 givens) | No solver exists in code; `technique.allium:1042-1069` name example fixtures. | adjustment | new page lists them as candidate fixtures marked unverified; they become `AdvancedExamples` fixtures only after the solver verifies uniqueness and the stated path | Never asserted unique or solvable as described until checked. |

### The report, as received

The text below is the report as the maintainer received it, kept here so this ticket
stands alone. Six mechanical edits, made by script, and no other: the dangling citation
markers (`cite` followed by `turn`, a source id and a view id, tool residue that named
nothing) are removed; the two flowchart blocks are fenced as `text` instead of their
diagram language, and the three formula blocks are fenced as `text` with their markup
kept, so markdownlint and typos read them as verbatim; every heading is demoted three
levels so this ticket keeps one level-one heading; the eight bold-only checklist labels
carry a trailing colon so markdownlint does not read them as headings; the table
delimiter rows are written in the house shape (`| --- |`) so markdownlint accepts them;
and a run of separate quotations is one quotation with a bare `>` between its lines.
The vocabulary is the report's own, so "clue" here means what this repository calls a
given, and its quotation marks and hard line breaks are as received. Nothing in this
subsection is a claim this repository makes; the page the ticket writes is where the
claims are weighed.

#### What Makes a Sudoku Puzzle “Fun”? A Rigorous Guide to Hand-Designing Elegant Sudoku

##### Executive summary

A good Sudoku is not simply a valid grid with one solution, nor is a hard Sudoku necessarily a good one. The most useful way to think about a “fun” Sudoku is as a **designed sequence of deductions**. The givens are the interface, but the real authored object is the solve path.

That distinction is strongly supported by both research and constructor practice. Radek Pelánek’s empirical work on Sudoku difficulty found that human difficulty is better modeled through human-style solving than through crude structural measures. In particular, difficulty depends on both the complexity of individual deductions and the dependency structure connecting those deductions. Nishikawa and Toda go further and formalize the notion of a puzzle being solvable by a specified collection of strategies, which is very close to the practical constructor’s question: “Will my intended audience be able to get through this using the tools I expect them to know?”

This leads to the central conclusion of this report:

> **A fun Sudoku is a fair sequence of comprehensible discoveries whose difficulty is well matched to the intended solver, with enough resistance to make progress meaningful and enough structure to make the eventual breakthroughs feel earned.**

“Fun” therefore has several interacting dimensions. **Challenge** supplies resistance. **Flow** keeps that resistance near the player’s current capability. **Novelty** prevents the solve from becoming mechanical. **Elegance** makes a small number of relationships do a surprising amount of work. **Aesthetics** operate both visually, through clue layout, and logically, through patterns in the solution path. **Fairness** means that the advertised puzzle actually rewards the reasoning vocabulary expected of its audience. **Solvability** and uniqueness establish trust. **Pacing** controls the alternation of progress, tension, bottlenecks, and cascades. **Satisfaction** is the emotional payoff when the puzzle’s previously hidden structure becomes visible.

Research on games offers a useful parallel. Sweetser and Wyeth’s GameFlow model connects enjoyment with appropriately matched challenge, concentration, clear goals, feedback, control, and related factors. Ryan, Rigby, and Przybylski found that perceived competence and autonomy were associated with game enjoyment and motivation. Sudoku naturally supplies a clear goal and frequent feedback. A constructor’s main job is therefore to manage **competence, uncertainty, and discovery**.

The hand constructor has a major advantage over a generator: deliberate authorship. Nikoli, the publisher that coined the name Sudoku, explicitly emphasizes the special status of human-crafted puzzles. World champion and constructor Thomas Snyder describes classic Sudokus created specifically for “artistic qualities,” and records a hand-crafted classic deliberately built to make repeated use of a particular logical step. He also praises another for its “interesting narrow solving path.” Those examples identify the essence of craft. **Hand construction lets the author manipulate not only whether deductions exist, but when they become visible and what they teach the solver to notice.**

The most important practical recommendations are:

| Design objective | What to optimize |
| --- | --- |
| Fairness | Every serious bottleneck should be resolvable with techniques appropriate to the advertised level. |
| Flow | Maintain a usable frontier of deductions. Avoid both long stretches of trivial filling and un-signaled cliffs in difficulty. |
| Elegance | Prefer deductions that arise naturally from the clue geometry and interact with later deductions. |
| Novelty | Give the puzzle one or two memorable ideas rather than a random assortment of unrelated advanced techniques. |
| Pacing | Aim for opening traction, increasing organization, a recognizable central hook, then a satisfying cascade. |
| Difficulty | Control the hardest required technique, how hard it is to spot, how many prerequisites it has, and how narrow the solve path becomes. |
| Aesthetics | Treat clue symmetry as composition rather than proof of quality. Logical symmetry and repeated motifs can matter even more. |
| Satisfaction | Make the key deduction materially change the grid. A difficult observation that eliminates one irrelevant candidate is usually less satisfying than one that unlocks several consequences. |

Two warnings are especially important.

First, **clue count is not a difficulty rating**. McGuire, Tugemann, and Civario proved that a uniquely solvable standard 9×9 Sudoku cannot have fewer than 17 givens, but that theorem concerns uniqueness, not human difficulty. A 25-given puzzle can be gentle while a 32-given puzzle can be extremely awkward. The study grids later in this report deliberately demonstrate that phenomenon.

Second, **the hardest named technique is not the whole difficulty**. A conspicuous X-Wing can be easier than an obscure hidden single. A puzzle can also be tiring despite using only modest techniques if each deduction depends on a long chain of earlier discoveries. Pelánek’s findings strongly support treating deduction complexity and dependency structure as separate difficulty variables.

A useful conceptual model is therefore:

```text
\[
\text{Enjoyment} \approx
\text{challenge-skill fit}
+\text{discovery}
+\text{elegance}
+\text{progress}
+\text{closure}
-\text{friction}
-\text{unfairness}.
\]
```

This is a **design heuristic, not a validated psychological equation**. Its value is diagnostic. When a puzzle is technically correct but not enjoyable, ask which term is failing.

##### What “fun” means in Sudoku

Sudoku is unusually revealing as a game-design medium because its external rules are extremely simple while its internal reasoning space is rich. The basic classic form asks the solver to place digits so every row, column, and 3×3 region contains the required digits without repetition. Competitive organizations such as the World Puzzle Federation use classic Sudoku alongside variants in formal timed competition, while mature solving references distinguish a large hierarchy of human techniques, from singles and intersections through fish, wings, coloring, chains, Almost Locked Sets, and forcing methods.

The consequence is that there are really **two games** inside one grid:

1. the combinatorial problem, “What is the unique completed grid?”;
2. the experiential problem, “What sequence of observations will a human make while discovering it?”

A constructor concerned with fun designs primarily for the second.

###### The dimensions of enjoyment

The following table gives an operational definition of each requested dimension.

| Dimension | In Sudoku, it means | Failure mode | Constructor’s test |
| --- | --- | --- | --- |
| **Challenge** | The solver must notice relationships that are not immediately obvious. | Too little becomes clerical filling. Too much becomes opaque search. | “Does the solver have to think, but still know what kind of thinking might pay off?” |
| **Flow** | Difficulty stays near the solver’s competence often enough to sustain concentration. | Long dead periods, sudden technique cliffs, or excessive triviality. | Plot the solve chronologically. Are there avoidable flat spots or spikes? |
| **Novelty** | The puzzle asks the solver to notice a pattern, interaction, or presentation that differs from routine scanning. | Every step feels interchangeable with thousands of generated puzzles. | “What will the solver remember tomorrow?” |
| **Elegance** | A small structural fact produces several consequences, or the same theme reappears in different forms. | Advanced steps exist merely because candidates happen to permit them. | “Does the key deduction explain the puzzle rather than merely advance it?” |
| **Aesthetics** | The clue pattern is visually coherent, and the logical structure feels composed. | Random-looking clues or decorative symmetry that damages the solve. | View the unsolved grid from a distance, then review the deduction map. |
| **Fairness** | Required reasoning matches the stated rules, level, and reasonable solver expectations. | Hidden need for guessing or an accidental technique far above the label. | Give it blind to players from the target group. Record where they stop. |
| **Solvability** | There is one intended completion and a human route to it under the chosen technique contract. | Multiple solutions, contradiction-prone construction, or a solver-only brute-force route. | Verify uniqueness separately from human solvability. |
| **Pacing** | Progress has rhythm: traction, development, resistance, breakthrough, release. | Fifty routine placements followed by one arbitrary wall. | Record placements per deduction and time between meaningful advances. |
| **Satisfaction** | Progress feels caused by understanding. The finish resolves earlier tension. | The key step does almost nothing or the ending dribbles into bookkeeping. | “What changed because of the breakthrough?” |

The GameFlow literature is useful here because enjoyment is not equated with maximum challenge. Challenge is valuable when it remains related to skill and when players retain feedback and a sense of control. Ryan and colleagues similarly found links between enjoyment and the satisfaction of competence and autonomy. For Sudoku, that suggests a crucial distinction between **productive difficulty** and **friction**. Productive difficulty makes the player feel, “There must be something here I have not seen yet.” Friction makes the player feel, “I have no reason to believe looking longer will help.”

###### Elegance is not the same as difficulty

An elegant deduction often compresses information.

Suppose candidate 4 can appear in only columns 3 and 9 of both row 1 and row 5. Those four candidate positions create an X-Wing. Every other 4 in columns 3 and 9 can be removed. HoDoKu classifies X-Wing among the basic fish techniques.

The elegance does not come from the name “X-Wing.” It comes from the compression:

> four candidate positions  
> become one global relationship  
> which produces eliminations somewhere else.

A badly composed advanced puzzle can contain ten X-Wings that feel like bookkeeping. A beautifully composed intermediate puzzle can make one pointing pair feel revelatory because the givens direct attention toward it and the elimination causes a cascade.

Snyder’s construction archive illustrates this design mentality unusually clearly. He calls one classic a puzzle with an “interesting narrow solving path,” and describes another as repeatedly exploiting one logical step in a way that is characteristic of hand-crafting. His broader collection explicitly includes classic puzzles “specially made for artistic qualities.”

That suggests a useful constructor’s principle:

> **Do not ask, “How many advanced techniques can I put in?” Ask, “What is this puzzle about?”**

###### Novelty without obscurity

Novelty can be structural rather than technical.

A classic Sudoku can feel novel because:

- several pointing pairs are arranged around the center and echo each other;
- an early pair seems incidental but becomes one wing of the later breakthrough;
- the same digit repeatedly carries the solve;
- the givens suggest rotational visual symmetry while the logical path moves asymmetrically;
- a technique appears in an unusually clean geometric form;
- the puzzle repeatedly alternates local box logic with global row-column logic.

This is exactly where hand construction distinguishes itself. Nikoli’s public emphasis on human-crafted puzzles and Snyder’s explicitly themed construction practice both point toward Sudoku as an authored logical experience rather than merely a generated constraint-satisfaction instance.

###### Fairness, guessing, and trust

“Fair” does **not** mean easy. It means that the puzzle keeps the contract it implicitly made with its player.

For a beginner puzzle, requiring an XY-Chain without warning is unfair even if the chain is logically impeccable. For a puzzle advertised to experts, requiring an Alternating Inference Chain may be entirely appropriate. HoDoKu’s taxonomy itself reflects this spectrum. Singles and intersections appear near the foundational end, while chains, ALSO structures, and forcing methods occupy progressively more specialized territory.

Forcing chains are logically legitimate. HoDoKu defines them broadly as chains establishing a contradiction or necessary truth, but places forcing methods near its “last resort” category, with forcing nets described as realistically manual only for very experienced players. That distinction matters. A constructor should decide in advance whether contradiction reasoning is part of the intended vocabulary, rather than accidentally discovering during testing that the puzzle needs it.

A publishable classic should therefore satisfy three different tests:

**Combinatorial validity:** exactly one solution.

**Strategic validity:** a route exists within the intended technique repertoire.

**Experiential validity:** ordinary target solvers can plausibly *find* that route.

Nishikawa and Toda’s work formalizes the second problem. Given clue positions and a chosen strategy set, they study whether a clue assignment can be generated that is solvable using those strategies. The third problem still requires humans.

##### Cognition, emotion, and differences among players

A Sudoku solve repeatedly cycles among **search, recognition, inference, action, and reorganization**.

At the start, the solver sees a large field of apparently unrelated information. Each placement reduces uncertainty. Candidate distributions gradually acquire recognizable form. Eventually a pattern that had been latent becomes salient: two places for a digit in a box, a pair occupying two cells, an X-Wing across distant rows, or an implication chain linking apparently unrelated candidates. HoDoKu’s large hierarchy of techniques can be understood as a vocabulary for these recurring structures.

###### Pattern recognition changes what “hard” means

A novice and an expert are not merely doing the same computation at different speeds.

A novice may inspect one empty cell and ask which digits fit. An intermediate solver scans houses for conjugate positions, pairs, and locked candidates. An advanced player may look at the distribution of one candidate across the entire grid and immediately notice fish or link structure. An expert may perceive a network of strong and weak links rather than isolated candidates.

This is why difficulty labels are audience-relative. A solver who has recently learned XY-Wings can experience a clean XY-Wing as an exciting breakthrough. An expert may regard precisely the same configuration as routine visual syntax. HoDoKu’s taxonomy gives concrete evidence of how large this learned technique vocabulary can become.

Difficulty ratings also lack a universal scale. Contemporary attempts to compare ratings across Sudoku sites have found substantial inconsistency among published difficulty labels, while earlier research likewise shows that good human-difficulty models require more than superficial puzzle statistics. Thus “beginner,” “hard,” and “expert” should be treated as **audience contracts**, not mathematical properties.

###### Aha moments

The characteristic Sudoku “aha” occurs when a previously noisy state can suddenly be represented much more simply:

> “Those two 7s are locked in this box.”
>
> “These three bivalue cells form an XY-Wing.”
>
> “Whichever way this pair resolves, that cell cannot be 6.”

General insight research characterizes the Aha experience in terms of sudden understanding, fluency, positive affect, and increased confidence in the discovered solution. Sudoku is especially fertile ground for such moments because a successful abstraction often converts dozens of pencil marks into one comprehensible relationship.

A good constructor creates the **conditions** for insight rather than hiding an arbitrary needle in a haystack.

The best breakthroughs generally have three properties:

**Preparation.** Earlier deductions simplify the relevant region.

**Compression.** The key observation can be mentally stated in one coherent sentence.

**Consequence.** The observation materially changes the puzzle.

That final property is easy to underestimate. Consider two equally difficult eliminations. One removes a candidate from a five-candidate cell and produces nothing else. The other makes a naked single, which makes a hidden single, which exposes a pair. The second is usually more satisfying because the player immediately receives evidence that the insight mattered.

###### Frustration thresholds

There is no single universal number of minutes after which a Sudoku becomes frustrating. The same ten-minute impasse can be delightful to an expert who expects a difficult hunt and intolerable to a casual solver expecting a coffee-break puzzle. Challenge-skill matching and perceived competence therefore offer a better model than absolute time.

A constructor can nevertheless monitor warning signs:

| Productive struggle | Unproductive frustration |
| --- | --- |
| Player has several plausible areas to inspect. | Player repeatedly rescans the entire board without a hypothesis. |
| Candidate structure is becoming cleaner. | Pencil marks remain uniformly noisy. |
| The solver can describe what they are trying. | The solver says, “I have no idea what kind of thing I am looking for.” |
| A failed line teaches something. | Repeated inspection yields no new information. |
| Breakthrough causes visible progress. | Breakthrough gives an obscure elimination followed by another equally obscure search. |

The constructor should not eliminate frustration completely. Tension is part of the reward system. The aim is **credible resistance**.

###### Skill level changes the desired emotional curve

| Player | Desired experience | Good bottleneck | Typical emotional payoff | Bad design for this audience |
| --- | --- | --- | --- | --- |
| **Beginner** | “I can do this.” | Finding the next single through systematic scanning. | Growing competence and momentum. | Long candidate bookkeeping or an unexplained advanced pattern. |
| **Intermediate** | “I saw something clever.” | Locked candidates, pairs, triples, modest interactions. | First real Aha moments beyond direct placements. | A puzzle consisting entirely of singles despite being labeled medium. |
| **Advanced** | “The grid had hidden structure.” | Fish, wings, coloring, multi-stage setups. | Recognizing a global pattern and watching it unlock the grid. | Random accumulation of named techniques with no thematic coherence. |
| **Expert** | “I understood the architecture.” | Chains, unusual interactions, ALSO structures, tightly controlled forcing. | Compressing a complicated candidate network into a decisive proof. | Blind exhaustive search masquerading as logical depth. |

The crucial design shift is that beginners need **frequent confirmation**, while experts can tolerate longer delayed rewards. Ryan and colleagues’ competence findings help explain why both can be enjoyable when matched to the player.

##### The craft of designing Sudoku by hand

Human construction works best as **backward design plus iterative subtraction**. Instead of starting from a nearly empty grid and hoping a nice solve appears, decide what sort of solve you want and manipulate clues until the target deductions become necessary.

Nikoli explicitly champions human-crafted puzzles, while Snyder’s archive demonstrates deliberate construction around artistic qualities, themes, repeated logical devices, and narrow solve paths.

A practical workflow is:

```text
flowchart TD
    A[Choose target solver] --> B[Define allowed technique vocabulary]
    B --> C[Choose or construct a complete solution grid]
    C --> D[Choose visual clue geometry]
    D --> E[Place a generous symmetric clue set]
    E --> F[Remove a clue or symmetry orbit]
    F --> G{Still unique?}
    G -- No --> H[Restore or replace clue]
    G -- Yes --> I[Human-solve and log every deduction]
    I --> J{Desired technique and pacing?}
    J -- Too easy --> K[Remove or relocate a clue]
    J -- Too hard --> L[Add a clue that repairs the bottleneck]
    J -- Wrong logic --> M[Alter clues around target pattern]
    K --> F
    L --> I
    M --> I
    J -- Good --> N[Blind playtest]
    N --> O[Retune difficulty and presentation]
    O --> P[Final uniqueness and solve-path audit]
```

###### Start with a technique contract

Before touching clues, write something like:

> **Audience:** confident intermediate  
> **Expected knowledge:** singles, locked candidates, naked and hidden pairs  
> **Forbidden requirements:** fish, wings, chains, guessing  
> **Intended hook:** two pointing interactions, second one releases center  
> **Desired rhythm:** easy opening, one moderate stall, fast finish.

This one page of intent prevents a common failure: designing the grid first and inventing its target audience afterward.

Nishikawa and Toda’s strategy-solvability framework gives this informal practice a formal analogue. Puzzle generation can be constrained by the set of solving strategies that must suffice.

###### Choose the completed grid before sculpting the clues

For hand construction, working backward from a complete valid solution is usually easier than trying to invent givens directly.

You can obtain the completed grid by:

- constructing one manually;
- transforming a known completed grid by permitted Sudoku symmetries;
- building from a patterned base and then permuting digits, rows within bands, columns within stacks, bands, stacks, or transposition.

At this stage, the completed grid is **raw material**. Its quality as a puzzle will come from the clue selection.

###### Treat clue symmetry as composition

The most useful default for classic Sudoku is often **180-degree rotational symmetry of clue positions**. It is visually stable, easy to maintain by removing clues in opposite pairs, and does not require the values themselves to be symmetric.

Symmetry is an aesthetic convention, not a Sudoku rule and not evidence of logical quality. A perfectly symmetric ugly solve is still an ugly solve. If preserving the visual pattern forces you to leave a clue that destroys your key deduction, either find a different symmetric partner structure or consciously relax the symmetry.

A practical hierarchy is:

**First:** uniqueness and intended logical path.  
**Second:** pacing.  
**Third:** visual symmetry.

Do not reverse that order.

Center-cell decisions deserve special attention. Under rotational symmetry the center is its own orbit, so removing it changes the clue count by one while all ordinary paired removals change it by two. That makes it useful for fine control of both appearance and clue density.

###### Remove clues in orbits, but solve after every meaningful change

Begin with more clues than you expect to publish. Remove one symmetry orbit. Then ask four questions:

1. Is the solution still unique?
2. What new technique has become necessary?
3. What old deduction disappeared?
4. Did the ordering of the solve improve?

The fourth question is the most “artistic.”

Suppose removing a clue creates a beautiful X-Wing, but the X-Wing is irrelevant because a hidden single elsewhere solves the same cell first. Combinatorially, you succeeded. Experientially, you did not. The desired pattern exists but is not **necessary at the moment you want it noticed**.

This is why construction must be iterative.

###### Design target deductions backward

For a **pointing pair**, decide which digit you want confined to a row or column inside one box. Add or remove clues so the other candidate positions for that digit inside the box disappear, while leaving at least one meaningful elimination outside.

For a **naked pair**, create two cells whose candidate sets become exactly the same pair. Then ensure at least one other unsolved cell in the house contains one or both pair digits so the pattern has work to do.

For an **X-Wing**, you need one digit to occupy exactly the same two columns in two rows, or the same two rows in two columns. HoDoKu’s basic fish taxonomy formalizes this pattern. The constructor’s problem is not merely to make an X-Wing exist. It is to control when earlier deductions strip away the extra candidates that reveal it.

For an **XY-Wing**, create a pivot `{x,y}` seeing two wings `{x,z}` and `{y,z}`. The wings need not see each other. A cell seeing both wings can lose `z`. HoDoKu includes XY-Wing, XYZ-Wing, and W-Wing as a distinct technique family. Again, the craft lies in making those three bivalue cells emerge at the appropriate stage rather than being visible trivially from the start or destroyed prematurely.

For an **expert chain**, work backward from the conclusion. Decide the candidate you want eliminated, identify the implication chain that proves it, and then protect the links by ensuring that no simpler deduction collapses them early. HoDoKu defines forcing-chain logic in terms of chains producing necessary truths or contradictions.

###### Construct dependencies, not merely techniques

Imagine two puzzles both contain an X-Wing.

In Puzzle A, the X-Wing is visible from the initial grid.

In Puzzle B:

1. a hidden single removes a candidate;
2. that creates a pointing pair;
3. the pointing elimination turns one row into a two-position configuration;
4. a second row already has matching positions;
5. the X-Wing becomes visible;
6. its elimination produces a naked single;
7. five placements cascade.

Puzzle B will usually feel more authored because the X-Wing is part of a **causal arc**.

This is exactly the kind of distinction Pelánek’s difficulty model highlights. The complexity of individual steps and the dependency structure among steps are separate contributors to difficulty. For constructors, they are also separate contributors to pacing.

###### Use a solve log

For every test solve, write something like:

```text
r6c7 = 4        hidden single
r2c3 = 8        naked single
candidate 6     pointing pair in box 4 -> remove 6 from r2c2, r3c2
r7c2 = 1        naked single caused by pointing pair
digit 4         X-Wing rows 1/5, columns 3/9
r4c9 != 4       X-Wing elimination
...
```

Then mark each line:

`R` = routine  
`H` = intended hook  
`C` = cascade consequence  
`F` = friction or ugly step  
`A` = accidental higher-level technique

A good puzzle rarely needs every line to be clever. In fact, that would be exhausting. The routine deductions are the connective tissue that makes the hooks comprehensible.

###### Use computation as quality control without surrendering authorship

“Designed by hand” need not mean “never checked by a computer.”

A sensible division of labor is:

**Human constructor:** chooses aesthetics, target techniques, clue placement, theme, pacing, and revisions.

**Software:** verifies uniqueness, catches accidental multiple solutions, inventories possible techniques, and detects unintended logical cliffs.

This preserves exactly the element Nikoli values in human-crafted puzzles while using computational methods for the combinatorial checks humans are worst at.

The minimum-clue theorem is a good illustration of why this distinction matters. Proving that no 16-clue standard Sudoku has a unique solution required exhaustive computational work over the enormous solution space. You should not make heroic manual uniqueness checking the centerpiece of your construction process.

##### Engineering difficulty, pacing, and skill-level targets

Difficulty should be treated as a vector rather than a scalar.

A useful constructor model is:

```text
\[
D =
(T,\ S,\ L,\ B,\ V,\ P)
\]
```

where:

- \(T\) = technique depth;
- \(S\) = spotting difficulty;
- \(L\) = dependency length;
- \(B\) = bottleneck narrowness;
- \(V\) = candidate volume or visual noise;
- \(P\) = penalty for choosing the wrong area to investigate.

This model is a design framework rather than a standardized rating formula. It reflects the research result that step complexity alone is insufficient. Dependency structure matters too.

###### A useful difficulty taxonomy

```text
flowchart LR
    A[Beginner] --> B[Intermediate]
    B --> C[Advanced]
    C --> D[Expert]

    A1[Full houses<br/>Naked singles<br/>Hidden singles] --> A
    B1[Locked candidates<br/>Pairs<br/>Triples] --> B
    C1[X-Wing<br/>Swordfish<br/>Wings<br/>Coloring] --> C
    D1[XY-Chains<br/>AICs<br/>ALSO patterns<br/>Forcing chains] --> D
```

This ordering is deliberately approximate. HoDoKu catalogues these families but real difficulty depends heavily on presentation and interaction. Difficulty labels also vary substantially across publishers and websites, so the categories should be calibrated against your own audience.

###### Skill-level design targets

The clue counts below are **starting heuristics for construction, not definitions of difficulty**. They are intentionally broad.

| Target | Useful starting clue range | Core required techniques | Bottleneck design | Desired pacing |
| --- | --- | --- | --- | --- |
| **Beginner** | Roughly 36 to 45 | Full houses, naked singles, obvious hidden singles | None longer than a few scans | Continuous progress |
| **Intermediate** | Roughly 30 to 37 | Singles, locked candidates, pairs, occasional triple | One or two clearly discoverable reductions | Progress, pause, insight, cascade |
| **Advanced** | Roughly 26 to 33 | Subsets plus fish, wings, coloring or equivalent | Several prerequisites before a centerpiece deduction | Multiple waves and one memorable peak |
| **Expert** | Roughly 22 to 30 | Chains, difficult fish, ALSO interactions, controlled forcing | Narrow logical frontier with substantial candidate structure | Long preparation, major compression, strong release |

These ranges should never override the solve path. The 17-clue lower bound concerns uniquely solvable standard 9×9 Sudoku, not difficulty. Nishikawa and Toda’s work likewise separates clue selection from solvability by a particular strategy set.

Indeed, the beginner sample later in this report has **only 32 givens and resolves entirely through naked singles**. That is exactly why counting clues is a poor substitute for solving the puzzle.

###### Tuning a puzzle downward

When a puzzle is too hard, do not simply add arbitrary givens.

First locate the exact point where the target solver stalls. Then ask why.

If the obstacle is an X-Wing you do not want, add a clue that eliminates one of the candidates the X-Wing was needed to remove.

If the problem is spotting difficulty, add a clue that reduces visual noise around the intended pattern without solving the pattern directly.

If the problem is a dependency chain that is too long, add a clue midway through that chain. This preserves the later hook while removing one prerequisite.

If there are too many simultaneous search fronts, add a clue that closes an irrelevant region and directs attention.

For beginners, the strongest rule is:

> **Every placement should make at least one reasonably visible next placement more likely.**

You are effectively building a chain of invitations.

###### Tuning a puzzle upward

Removing clues randomly is a poor difficulty-control mechanism.

Instead, remove a clue because you can predict what its absence will do:

- turn a single into a pair;
- preserve an extra candidate until a locked-candidate step removes it;
- hide the second half of a fish until later;
- prevent a shortcut;
- make two solving fronts depend on each other;
- force the player to shift from cell scanning to candidate scanning.

That is controlled difficulty.

A particularly effective advanced technique is **shortcut suppression**. When a beautiful intended deduction exists but a simpler route bypasses it, identify the clue that enables the shortcut. Remove or relocate that clue while preserving uniqueness.

###### Branching has two meanings

Constructors should distinguish **logical branching** from **guessing branching**.

Logical branching means several valid deductions are simultaneously available. Some of this is good. It gives players agency and accommodates different scanning styles, which is consistent with the broader importance of perceived autonomy in game enjoyment.

Too much logical branching, however, can dilute a theme because every solver follows a different path and the intended centerpiece becomes optional.

Guessing branching means the solver chooses a candidate without proof and explores consequences. For a classic puzzle advertised as logically solvable, that should usually be unnecessary. If contradiction chains are part of your expert-level vocabulary, make that explicit and keep them short enough to constitute reasoning rather than blind tree search. HoDoKu makes a similar conceptual separation between recognizable chain logic, forcing nets, and brute-force style last resorts.

###### Pacing should look like a story arc

A particularly reliable pacing model is:

**Orientation.** Several accessible deductions teach the solver where the action is.

**Compression.** Candidate structure becomes cleaner.

**Resistance.** Easy progress slows.

**Hook.** One nontrivial pattern explains the resistance.

**Release.** The deduction produces several consequences.

**Echo.** A related but easier version of the same idea may reappear.

**Closure.** The last phase accelerates so the player finishes feeling mastery rather than exhaustion.

Do not make every section equally difficult. Contrast is part of the experience.

A constructor can quantify pacing informally by recording the number of routine deductions between conceptual deductions. A rough pattern such as

`6 routine → hook → 8 routine → hook → 20-cell cascade`

often feels more satisfying than

`45 routine → one brutal chain → 35 routine`.

###### Symmetry and difficulty sometimes conflict

Visual symmetry can force you to remove or restore clues in pairs even when only one of the two positions matters logically.

There are three reasonable responses:

1. preserve symmetry and rework both sides of the logical structure;
2. permit a slight asymmetry because the solve improves substantially;
3. change to another coherent symmetry scheme.

The wrong response is to sacrifice the solving experience merely to obtain an attractive silhouette.

The best handcrafted puzzles can make visual and logical symmetry reinforce each other. For example, symmetric clue removal can create matching candidate structures on opposite sides of the grid, only for the solve to reveal that one side resolves first and unlocks its counterpart.

##### Annotated study puzzles from beginner to expert

The following four grids are **original study puzzles generated and analyzed for this report**. I used them because their behavior can be reported reproducibly rather than attributing someone else’s construction. Each was independently checked to have exactly one solution. I then analyzed it with a deterministic human-style strategy hierarchy. Solver order matters, so the annotations show **a valid representative path, not a claim that every human must follow the same path**.

A dot is an empty cell. Coordinates use `rNcM`, meaning row N, column M.

###### Beginner study puzzle

**32 givens. Unique solution. Singles only in the tested path.**

```text
..7 92. .5.
.2. .16 .37
6.. 3.. 4.9

... ... 29.
... 6.1 ...
.78 ... ...

9.2 ..5 ..3
56. 87. .4.
.8. .496 ..
```

The striking feature is that this sparse-looking puzzle does **not** need advanced logic.

The opening begins:

`r2c7 = 8`  
`r1c7 = 1`  
`r1c9 = 6`  
`r2c1 = 4`  
`r1c2 = 3`  
`r1c1 = 8`  
`r1c6 = 4`  
`r2c4 = 5`

and the chain continues entirely through naked singles in the deterministic solve.

**Intended hook:** the puzzle teaches the player that a crowded-looking grid can simplify through repeated local constraint propagation. The satisfaction comes from **momentum**, not a named advanced technique.

**Design lesson:** a low clue count need not imply a difficult puzzle. This is a concrete counterexample to the common constructor instinct to classify by givens. The research literature independently supports using human solving behavior rather than clue count alone for difficulty assessment.

For a true novice publication, I would probably add several clues despite their being logically unnecessary. That would make the opening less visually intimidating. This distinction is important: **mathematical redundancy can be experiential value**.

###### Intermediate study puzzle

**32 givens. Unique solution. Singles plus locked candidates.**

```text
.3. 47. .5.
..2 .8. ...
..5 1.3 ..7

..9 ... 2.4
2.7 9.8 3.1
1.3 ... 5..

3.. 7.4 9..
... .9. 6..
.5. .31 .4.
```

After an opening run of singles, the lower-left box reaches a useful position: candidate `6` is restricted to **r4c2 or r6c2** within that box.

Therefore the 6 for that box must occur in column 2. No other cell in column 2 outside that box can be 6.

This eliminates candidate 6 from positions including:

`r2c2`  
`r3c2`  
`r7c2`

That is a classic **pointing locked-candidate** deduction, one of the foundational intersection techniques in HoDoKu’s taxonomy. The solve later uses several related locked-candidate eliminations before returning to singles.

**Intended hook:** the player graduates from “Which number belongs in this cell?” to “Where can this digit exist inside this box, and what does that imply outside the box?”

That cognitive change is more important than the formal technique name. It teaches **cross-house reasoning**.

**How to hand-design this effect:** first choose two cells in one box and one shared column. Arrange the givens so digit 6 remains possible in precisely those two box cells. Then leave one or more useful 6 candidates elsewhere in the column. During tuning, protect the pattern from premature singles.

###### Advanced study puzzle

**29 givens. Unique solution. Includes locked candidates, a pair, an X-Wing, and an XY-Wing in the tested path.**

```text
8.. 92. 35.
... ... 8.7
... .84 ...

..3 ..8 .2.
.1. 562 .3.
.5. 4.. 9..

... 81. ...
5.9 ... ...
.87 .59 ..3
```

The first major global hook is an **X-Wing on digit 4**.

At the relevant stage:

- in row 1, candidate 4 appears only in columns 3 and 9;
- in row 5, candidate 4 also appears only in columns 3 and 9.

Therefore rows 1 and 5 must occupy the 4s in those two columns in some order. Any other candidate 4 in columns 3 and 9 can be deleted. In the tested path that removes 4 from `r4c9` and `r7c3`. This is the standard X-Wing relationship classified under basic fish.

Shortly afterward the puzzle produces an especially clean **XY-Wing**:

- pivot `r2c1 = {2,4}`;
- wing `r1c3 = {4,6}`;
- wing `r6c1 = {2,6}`.

If the pivot is 2, the first wing is forced toward 6 through the alternative structure. If the pivot is 4, the other wing provides the corresponding 6. In either case, any cell seeing both wings cannot itself be 6.

The tested path therefore removes 6 from `r3c1` and `r6c3`.

HoDoKu groups XY-Wing with other wing techniques precisely because the deduction is built from interacting bivalue cells rather than one candidate’s global distribution.

**Intended hook:** the puzzle changes representational mode twice. First the solver sees a **single-digit global pattern** in the X-Wing. Later the solver has to see a **multi-digit relational pattern** in the XY-Wing.

This is a useful advanced-puzzle pattern. Do not merely increase the technical difficulty. Change what kind of structure the solver must recognize.

**Construction lesson:** an advanced centerpiece should preferably have aftermath. When hand-tuning, remove or move clues until the X-Wing elimination produces a candidate structure that helps form the later wing. Then the two techniques feel like chapters of the same story rather than unrelated tricks.

###### Expert study puzzle

**25 givens. Unique solution. The tested singles, intersection, and subset repertoire stalls. A short contradiction chain provides a representative expert continuation.**

```text
..9 4.. ...
..8 ..9 ...
7.4 .38 .9.

.6. ... .7.
..1 .4. 8..
.7. ... .1.

.9. 75. 3.6
... 1.. 5..
... ..6 9..
```

The initial solve makes substantial progress using singles, six locked-candidate eliminations, a naked pair, and a hidden pair. Then the ordinary advanced repertoire used for the analysis stalls.

At that stage, `r3c9 = {2,5}`.

Assume temporarily that:

`r3c9 = 5`.

Straightforward consequences then give:

`r3c2 = 1`  
`r5c9 = 3`  
`r5c8 = 5`  
`r5c4 = 6`  
`r3c4 = 2`  
`r2c4 = 5`  
`r1c6 = 1`  
`r2c2 = 3`  
`r1c2 = 5`  
`r2c8 = 6`

At that point `r3c7` has **no legal candidate**.

The assumption is impossible. Therefore:

```text
\[
\boxed{r3c9=2}
\]
```

This is a short contradiction-style forcing argument. HoDoKu defines forcing chains as implications that establish a necessary truth or contradiction, while treating more elaborate forcing structures as high-end or last-resort techniques.

**Intended hook:** the player stops searching for a static shape and begins reasoning about the **consequences of a proposition**.

That is an important threshold between advanced and expert Sudoku.

It is also a taste issue. Some expert audiences enjoy concise contradiction reasoning. Others prefer every difficult deduction expressed as an AIC, XY-Chain, ALSO relationship, or another non-bifurcating representation. For the latter audience, this puzzle should be retuned until the same dependency becomes visible as a cleaner chain.

The construction principle is broader than the sample:

> Expert difficulty should come from **compressed logical architecture**, not from asking the player to perform arbitrary brute-force search.

HoDoKu’s separation of chain reasoning from forcing nets and brute-force-style methods is helpful when deciding where to set that boundary.

##### Playtesting, manual-design checklist, and source priorities

A constructor is too familiar with their own puzzle to be its final judge. Once you know the intended hook, you cannot accurately simulate the experience of not knowing it.

That is why **blind playtesting is not optional for serious hand construction**.

###### A rigorous playtesting protocol

Use at least three types of test.

**Constructor solve.** Solve from scratch after enough time has passed that you no longer remember every placement. Record every deduction.

**Technique audit.** Use a trusted solver to inventory possible techniques at each stage. The purpose is not to replace human judgment, but to detect accidental shortcuts, multiple solutions, or a harder-than-intended bottleneck. Scientific work on both difficulty modeling and strategy-constrained Sudoku generation supports treating such computational analysis as a separate layer from human solving.

**Blind human solve.** Give only the puzzle and its advertised difficulty to a player who was not involved in construction.

For each playtester, record:

| Measure | Why it matters |
| --- | --- |
| Total solving time | Crude but useful calibration. |
| Longest no-progress interval | Better indicator of frustration risk than total time. |
| Location of first serious stall | Reveals whether opening guidance works. |
| Technique used at the stall | Tests your level contract. |
| Wrong turns or guesses | Detects misleading candidate structure. |
| Moment described as “nice” or “aha” | Identifies the actual highlight, which may differ from your intended one. |
| Whether hints were requested | Useful measure of practical accessibility. |
| Post-solve recall | “What do you remember about it?” is an excellent test of whether the puzzle had an identity. |

Do not merely ask, “Was it fun?” That question produces noisy politeness.

Ask:

> “Where did you first feel stuck?”
>
> “At what point did you know what the puzzle was doing?”
>
> “Was any deduction irritating rather than satisfying?”
>
> “Which step felt best?”
>
> “Which step felt least fair?”
>
> “Did you ever believe guessing was necessary?”

Those answers are actionable.

###### How to interpret playtest disagreement

Suppose four intermediate testers behave as follows:

- one solves in 8 minutes;
- two solve in 13 to 16 minutes;
- one stalls for 12 minutes on an X-Wing you did not intend.

Do not simply average their times.

Inspect the stalled player’s state. If the X-Wing was genuinely necessary because they took a legitimate alternate route, your puzzle may be **path-dependent in difficulty**. That matters even if your own intended solve never needs the technique.

This is one reason narrowly authored paths are attractive. Snyder explicitly describes the interest of a “narrow solving path,” though absolute linearity is not necessary or always desirable. A little branching gives agency. Too much can make difficulty unstable.

###### Manual construction checklist

Use this before publication.

**Audience and contract:**

- [ ] I can name the intended player level in terms of techniques, not merely “easy/medium/hard.”
- [ ] I have listed techniques that may be required.
- [ ] I have listed techniques that must **not** be required.
- [ ] The puzzle’s hardest spotting task is appropriate even if its formal technique is simple.

**Validity:**

- [ ] The givens do not violate the classic rules.
- [ ] The puzzle has exactly one solution.
- [ ] Uniqueness has been verified independently.
- [ ] The puzzle is solvable without uncontrolled guessing under the intended technique contract.
- [ ] I have checked legitimate alternate solve paths.

**Aesthetics:**

- [ ] The clue pattern looks deliberate.
- [ ] Symmetry supports rather than damages the solve.
- [ ] There are no visibly awkward isolated clues unless they serve a purpose.
- [ ] The puzzle has at least one identifiable logical motif or character.

**Opening:**

- [ ] The target audience can find meaningful progress reasonably soon.
- [ ] The first deductions orient attention rather than merely fill random cells.
- [ ] The puzzle does not initially resemble a much harder puzzle than it really is without good reason.

**Middle game:**

- [ ] Difficulty rises through structure, not candidate clutter alone.
- [ ] At least one deduction feels more significant than routine progress.
- [ ] Important advanced patterns are necessary or meaningfully useful, not decorative.
- [ ] No simple shortcut destroys the intended centerpiece.
- [ ] No unintended advanced step creates a difficulty cliff.

**Pacing:**

- [ ] There are no excessively long sequences of mechanical placements.
- [ ] There are no unjustified dead zones.
- [ ] Major deductions produce meaningful consequences.
- [ ] The puzzle alternates tension and release.
- [ ] The final phase accelerates or resolves earlier structure cleanly.

**Playtesting:**

- [ ] At least one target-level solver has solved it blind.
- [ ] I recorded stalls rather than only completion times.
- [ ] I asked what the solver thought the puzzle’s highlight was.
- [ ] The actual highlight resembles the intended highlight.
- [ ] Difficulty labels were calibrated from real solvers, not clue count.

**Final polish:**

- [ ] Every given earns its place logically, aesthetically, or pedagogically.
- [ ] I have considered whether removing each apparently redundant clue actually improves the experience.
- [ ] I have considered whether adding a logically redundant clue improves approachability.
- [ ] The puzzle has a reason to exist beyond being another valid Sudoku.

###### A practical final rubric

For publication, score each category from 0 to 4.

| Category | 0 | 2 | 4 |
| --- | --- | --- | --- |
| Challenge fit | Clearly mismatched | Mostly appropriate | Precisely calibrated |
| Fairness | Guessing or surprise cliff | Minor irregularity | Technique contract completely trustworthy |
| Pacing | Flat or erratic | Mostly smooth | Deliberate tension and release |
| Elegance | Accidental | Some clean deductions | Strongly interconnected logic |
| Novelty | Generic | One interesting feature | Distinct identity |
| Visual aesthetics | Careless | Coherent | Striking without harming logic |
| Aha quality | None | One modest insight | Memorable, well-prepared breakthrough |
| Finish | Tedious | Acceptable | Satisfying cascade or resolution |

A score is not a scientific measure of fun. Its purpose is to force the constructor to evaluate **different failure modes independently**.

A puzzle scoring 4 for difficulty and 1 for pacing should not be called finished.

###### Sources worth prioritizing

The strongest source base combines mathematical research, human-difficulty research, constructor practice, and community technique references. No single source covers the full craft.

| Priority | Source | Why it matters |
| --- | --- | --- |
| **Essential research** | Radek Pelánek, *Difficulty Rating of Sudoku Puzzles: An Overview and Evaluation* | One of the most useful research treatments of human Sudoku difficulty. It separates individual-step complexity from dependency structure and evaluates measures against real solving data. |
| **Essential research** | Kohei Nishikawa and Takahisa Toda, *Exact Method for Generating Strategy-Solvable Sudoku Clues* | Formalizes the idea that puzzle generation can target a specified repertoire of human strategies. Extremely relevant to difficulty-controlled construction. |
| **Mathematical foundation** | Gary McGuire, Bastian Tugemann, and Gilles Civario, *There is no 16-Clue Sudoku* | Establishes the 17-given minimum for uniquely solvable standard 9×9 Sudoku. Useful mainly as a warning not to confuse clue count with human difficulty. |
| **Enjoyment theory** | Sweetser and Wyeth, *GameFlow: A Model for Evaluating Player Enjoyment in Games* | Useful framework for challenge-skill balance, concentration, goals, feedback, and player control. |
| **Motivation theory** | Ryan, Rigby, and Przybylski, *The Motivational Pull of Video Games* | Provides empirical support for competence and autonomy as important components of game enjoyment and motivation. Sudoku is not the paper’s subject, so application to Sudoku should be treated as principled transfer rather than direct evidence. |
| **Constructor practice** | Thomas Snyder, Grandmaster Puzzles and Friday Puzzle Series | Especially valuable because Snyder explicitly discusses puzzles made for artistic qualities, narrow solve paths, and logical behavior achievable through hand-crafting. |
| **Handcrafted tradition** | Nikoli | Important primary source for the publishing tradition that popularized Sudoku under that name and continues to emphasize human-crafted puzzle design. |
| **Technique reference** | HoDoKu | Exceptionally useful operational taxonomy of singles, intersections, subsets, fish, wings, coloring, chains, ALSO methods, and forcing techniques. It is a community/technical reference, not peer-reviewed difficulty research. |
| **Competition reference** | World Puzzle Federation Sudoku Grand Prix | Useful for seeing how Sudoku is presented, timed, and scored for high-level human solvers and for studying competition puzzle archives. |
| **Contemporary handcrafted ecosystem** | Cracking the Cryptic puzzle collections | Useful for studying modern curated constructor culture. Its classic Sudoku collection explicitly emphasizes handcrafted and curated work from recognized constructors. |

The hierarchy among these sources matters. For questions such as uniqueness and clue minima, prefer mathematical research. For human difficulty, prefer empirical difficulty research. For the vocabulary and mechanics of a technique, references such as HoDoKu are excellent. For the question “What does beautiful hand construction look like?”, the archives and commentary of elite constructors such as Snyder, together with Nikoli’s long-running human-construction tradition, are more informative than an automated rating formula.

The resulting philosophy can be stated compactly:

**Validity is the floor. Uniqueness establishes trust. Technique selection controls accessibility. Dependency design controls depth. Pacing controls emotion. Theming creates identity. Playtesting turns an intended experience into a reliable one.**

A machine can establish that a Sudoku has one solution. A sophisticated solver can estimate what techniques might solve it. What distinguishes excellent hand construction is the additional act of arranging those deductions so that a human encounters them in a meaningful order. That is the difference between a Sudoku that is merely solvable and one that feels *composed*.

## Goal

Five observable outcomes, all green under `just check`:

1. **The page.** `docs/explanation/puzzle-design.md`, kind `explanation`, audience
   `[contributor, maintainer, agent]`, `canonical_for: [puzzle_design_objectives]`,
   registered in `docs/manifest.yml` and linked from the "This library" list in
   `docs/README.md`. It carries the report cleaned to the rules above, then a closing
   section "What the engine does about it" that is the objective map, every file:line
   re-checked against the tree on the day. It names the four study puzzles as candidate
   fixtures and says in the same breath that nothing here has verified them.
2. **The docs agree on generation.** `docs/project/purpose-and-scope.md` says a
   `generation.allium` module is planned and specified before it is built, and that
   whether puzzles are made on device, ahead of time, or both is open.
   `docs/explanation/architecture.md` says the same and no longer says "not deferred".
3. **A decision record.** `docs/decisions/0012-generation-and-dev-time-judges.md`:
   generation is the engine's and is specified first; judgement models, classifiers and
   any tool that needs a network are development-time only, consume the engine's
   serialised outputs, and never enter the runtime, the crate or a spec. Context,
   decision, consequences including the ones that hurt, what would reopen it; manifest
   entry; row in `docs/decisions/README.md`.
4. **The gaps in the specs.** One `open question` per gap, in the module that owns it,
   in that module's voice (see Files touched for the list), and
   `docs/specs/generation.allium` as a skeleton: header, `config`, open questions, and
   the least the checker needs, passing `just check-specs` and `just analyse-specs` with
   no waiver. If the pinned checker refuses a module with no triggers and no honest fix
   exists, the skeleton's header text lands on the page instead and the module becomes
   T20 (recorded under Deviations).
5. **The sources.** Every report source the executing agent can open is a row in the
   evidence table of `docs/explanation/human-solving.md`, in that table's three columns
   and with the same restraint about what a source does not establish. Nikoli, Snyder's
   archive, the WPF Grand Prix and Cracking the Cryptic are constructor practice, not
   evidence; they belong on the page, not in the table. What cannot be opened is dropped.

## Non-goals

- Any Rust. No solver, generator, rating, pacing measure or serialisation code. Nothing
  under `crates/` changes.
- Deciding any product question: where puzzles are made, the base figures, a pacing
  target per level, which symmetry schemes, how many presets. Each becomes an `open
  question`.
- A "fun score", a fifth player model, or any scalar that combines the report's
  dimensions. The report's own formula is kept on the page as the report's heuristic,
  never as a specification.
- Verifying the four study puzzles. No solver exists; they are candidate fixtures marked
  unverified until one does.
- Naming Jev or any vendor in a specification. Decision 0012 may name it as the case that
  prompted the rule.
- Changing `Justfile`, `tools.txt`, `pyproject.toml`, `pixi.lock`, any workflow, any
  skill, `AGENTS.md`, or `docs/reference/documentation-contract.md`.
- Rewriting `docs/explanation/solving-sudoku.md` or `human-solving.md` beyond the table
  rows named. Their generation and difficulty remarks stay where they are; the page links
  to them.
- Touching `docs/specs/sudoku.allium`, `solver.allium`, `board.allium` or
  `lapse.allium`. The gaps land in the four modules named below; if the review finds one
  that belongs elsewhere, it is handed back, not made.

## Files touched

| Path | Change |
| --- | --- |
| `docs/explanation/puzzle-design.md` | New: the cleaned report, the objective map, the study puzzles marked unverified |
| `docs/manifest.yml` | Two entries: the page and decision 0012 |
| `docs/README.md` | "This library" gains the page; the Decisions bullet is unchanged (the decisions index owns its rows) |
| `docs/project/purpose-and-scope.md` | The "No puzzle generation yet" bullet: planned, `generation.allium`, both modes open |
| `docs/explanation/architecture.md` | The "not deferred" sentence corrected to match |
| `docs/decisions/0012-generation-and-dev-time-judges.md` | New decision |
| `docs/decisions/README.md` | Row for 0012 |
| `docs/explanation/human-solving.md` | Evidence table rows for the verified report sources |
| `docs/specs/generation.allium` | New skeleton (or T20 if the checker refuses it) |
| `docs/specs/technique.allium` | Open questions: spotting difficulty (the report's S) as a property of a pattern's visibility on a grid; whether "hook" and "cascade" are properties of a deduction or of a run |
| `docs/specs/reach.allium` | Open questions: pacing measures over `run.steps` (routine-run length between hooks, cascade size after a step, dependency length), which need a step to record the steps whose eliminations it depends on; path-dependence of the hardest step across scanning orders |
| `docs/specs/effort.allium` | Open question: whether the report's vector (T, S, L, B, V, P) maps onto the price terms, with V as candidate volume and B as the count of alternatives at a step |
| `docs/specs/human-solving.allium` | Open question: which `AssessmentDetail` counts a generator may target, and which a development-time judge reads |
| `docs/how-to/work-with-the-specs.md` | Module table gains the `generation.allium` row, only if the module lands |
| `docs/explanation/layering.md` | Import graph gains `generation`, only if the module lands |
| `docs/explanation/specifications.md` | "eight" becomes nine and a `generation.allium` paragraph, only if the module lands |
| `docs/decisions/0001-engine-as-a-library.md` | The module count, only if the module lands |
| `README.md` | The module count at line 55, only if the module lands |
| `docs/project/terminology.md` | Entries for hook, cascade and technique contract, if the page uses them as terms of art |
| `tickets/README.md` | The T19 index row's status |
| `tickets/T19-puzzle-design-objectives.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t19-puzzle-design-objectives` from `main`
   (README.md "How to pick up a ticket"); `just install-allium`; `just check` green before
   any edit. Read the "Read first" list and the received report above.

2. **The page.** Write `docs/explanation/puzzle-design.md` from the received report:
   frontmatter with the five keys the contract requires, in manifest order; H1 equal to
   the title; at least 40 words; no placeholder. Cleaning rules: every "clue" becomes
   "given" except inside a paper's title; the flowcharts become numbered lists; the two
   formula blocks become prose ("the report sums challenge-skill fit, discovery, elegance,
   progress and closure and subtracts friction and unfairness, and says itself that this
   is a heuristic"); minutes stay as the report's external practice with one sentence that
   the engine's analogue is steps; the study grids stay in fenced `text` under a heading
   that says they are unverified; outbound links point at whole pages, never fragments
   (`documentation-contract.md` 73-83); a link enters only after step 7 verified it. Keep
   the report's structure and voice where the rules allow; this is a cleaned copy, not a
   rewrite.

3. **The objective map** becomes the page's closing section. Copy the table from this
   ticket, re-read every cited line against the tree, correct what moved, and add a row
   for anything the executing agent finds that the map missed. The five classes and their
   meaning are stated above the table on the page as they are here.

4. **The scope pages.** Reword the "No puzzle generation yet" bullet in
   `purpose-and-scope.md` and the "What is not here" paragraph in `architecture.md` so
   both say: generation is a planned module, `generation.allium`, specified before it is
   built; where puzzles are made is open (decision 0012).

5. **Decision 0012.** Copy the shape of `docs/decisions/0011-tool-manager.md`. Context:
   the report, the maintainer's instruction quoted above, the `no_std` core and invariant
   2. Decision: generation is the engine's; judgement models are development-time only.
   Consequences that hurt: a curated set needs a pipeline outside this crate (S04's
   Python or CLI crate is the likely home); a generator on device cannot be helped by a
   judge at all; every output a judge reads must be stable and serialisable, which
   constrains the API. What reopens it: a judge that runs offline on device under the
   same invariants. Manifest entry with a unique `canonical_for`
   (`decision_generation_and_judges`); row in `docs/decisions/README.md`.

6. **The specs.** Follow `.agents/skills/spec-change/SKILL.md`. Add the open questions
   listed in Files touched to the Open Questions section of each module, one block per
   gap, in the module's voice and vocabulary, with no rule drafted. Then write
   `docs/specs/generation.allium`: `-- allium: 3`; header with Scope ("how a setter that
   is a program finds givens: a solution grid drawn through the randomness boundary,
   givens removed under a symmetry until the puzzle is well-posed and solvable within a
   stated technique contract, under a step budget"), Includes, Excludes (rating and hints,
   which are `reach.allium`'s; where puzzles are made; any judge), Dependencies (`sudoku`,
   `solver`, `technique`, `reach`), Vocabulary (given, never clue; symmetry orbit; technique
   contract; hook; cascade); a `config` block with a step budget and a candidate limit;
   and Open Questions for the symmetry schemes and their default, the technique contract's
   shape (a profile plus a ceiling, or a required set and a forbidden set), shortcut
   suppression (rate with the intended technique removed and require a stall), the
   pacing thresholds a candidate must meet, and where puzzles are made. Run `just
   check-specs` and `just analyse-specs`; both must be empty with no waiver. If the
   checker refuses a module that declares no trigger, try the smallest honest trigger
   (a `Generate` that requires a seed and a profile and guarantees only well-posedness,
   with its outcome left to an open question); if that is not honest either, move the
   header text to the page, leave the four modules' open questions in place, and draft
   T20 under Deviations.

7. **The sources.** Open each report source. Where it can be opened, add a row to the
   evidence table in `human-solving.md` (Evidence / What it motivates here / What it does
   not establish) for Nishikawa and Toda; McGuire, Tugemann and Civario; Sweetser and
   Wyeth; and Ryan, Rigby and Przybylski. Pelánek 2014 is already there. Constructor
   practice (Nikoli, Snyder, WPF, Cracking the Cryptic) is cited on the page only. Run
   `just check-links-online` (the manual lychee stage) and record its output. Drop what
   cannot be opened, and say so in the hand-back.

8. **The counts**, only if step 6 landed the module: the module table in
   `work-with-the-specs.md`, the import graph in `layering.md`, the paragraph and the
   "eight" in `specifications.md`, the count in decision 0001 and in the root `README.md`.
   Grep for `eight` across `docs/` and `README.md` before declaring this done.

9. `just check-docs`, `just check-specs`, `just analyse-specs`, then `just check`. Quote
   the tail of each in the hand-back notes.

10. **Hand-back notes.** What was verified and how; deviations; and the follow-ups,
    drafted with the next free ids on the day (T20, T21 and S05 if no other ticket has
    taken them; `rg -n 'T2[0-9]|S0[5-9]' tickets/` says):
    - **T20, the generation module.** Its triggers, guarantees and fixtures (the full
      specification if only a skeleton landed here): drawing a solution grid; removal by
      orbit; the acceptance run through `Rate`; shortcut suppression; the step budget and
      what a budget exhausted reports. Depends on T19 and on the solver's Rust ticket.
    - **T21, pacing measures.** In `reach.allium` (deterministic) and read by
      `human-solving.allium`'s `AssessmentDetail`: dependency edges between steps, then
      routine-run length, cascade size and dependency length as values of a run. Depends
      on T19 and on a solver in Rust.
    - **S05, a development-time judge.** A spike: what a judge is asked (the report's
      eight rubric dimensions as independent typed answers, never a sum), what it is shown
      (serialised `RunResult`, `PricedRun`, `TackledRun`, `AssessmentSummary` and the
      givens), how its verdicts tune generation `config` or select a curated set, where
      the pipeline lives (S04's Python crate is the candidate), and Jev as one candidate
      judge among others. Nothing at runtime. Depends on T19 and S04.
    - **Fixtures.** The four study puzzles are named as candidates for
      `technique.allium`'s example fixtures once `solver.rs` exists and has verified each
      one's uniqueness and stated path; a note for that Rust ticket, not a ticket of its
      own.

11. Set `status: done` here and in the index row, commit on the ticket branch, stop
    before pushing.

## Acceptance criteria

- `docs/explanation/puzzle-design.md` exists, is in the manifest, is linked from
  `docs/README.md`, has the five frontmatter keys in manifest order, and contains no
  citation marker, no "clue" outside a paper's title, no flowchart source and no formula
  markup. Its closing section is the objective map with one row per objective, each with
  a class and a landing place, and every cited line true on the day.
- `purpose-and-scope.md` and `architecture.md` say the same thing about generation, and
  neither says "not deferred" or "out of scope" of it.
- Decision 0012 exists, is in the manifest, has its row in the decisions index, and says
  development-time only in its own words.
- Each of `technique.allium`, `reach.allium`, `effort.allium` and `human-solving.allium`
  gains the open questions named in Files touched, and no rule.
- `docs/specs/generation.allium` exists and both spec gates report no diagnostics and no
  findings with no waiver, or Deviations records why it did not land and T20 is drafted.
- The evidence table in `human-solving.md` has a row for every report source that could
  be opened and none for one that could not; `just check-links-online` output is quoted.
- No page under `docs/` says "eight" modules where there are nine, if the module landed.
- `just check` is green; `git status --porcelain` on the ticket branch lists only paths in
  Files touched.

## Verification

```sh
just install-allium
just check-docs
just check-specs
just analyse-specs
just check-links-online
just check
rg -n 'citeturn|\bclue' docs/explanation/puzzle-design.md
rg -n 'generation' docs/project/purpose-and-scope.md docs/explanation/architecture.md
rg -n 'open question' docs/specs/generation.allium docs/specs/reach.allium docs/specs/effort.allium docs/specs/technique.allium docs/specs/human-solving.allium
rg -n '\beight\b' docs README.md
git status --porcelain
```

Expected: the four gates green; the two spec gates naming nine specifications (or eight,
with T20 drafted) and no diagnostics or findings; the online link check green; the first
`rg` empty apart from a paper's title; the second showing the same wording in both pages;
the third listing at least one block per module; the fourth empty of module counts;
`git status` listing only paths in Files touched.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- **Where puzzles are made.** A curated pre-generated set, generation on device, or
  both. Decision 0012 records it as open; the generation module's step budget and the
  weight of shortcut suppression (dear to run) depend on it. The maintainer's, not the
  executing agent's.
- **Pacing targets.** Whether the routine-run lengths, cascade sizes and closing-cascade
  thresholds a candidate must meet are specification obligations per profile or
  generation `config` a consumer sets. The open question in `reach.allium` poses it; the
  answer is T20's or T21's.
- **Symmetry as input or property.** Whether a symmetry scheme is something a generator
  is asked for, something it reports of what it made, or both; and whether asymmetric
  puzzles are ever accepted (the report ranks symmetry third, below uniqueness and the
  intended path).
- **The study puzzles.** Whether four unverified grids from a report are worth carrying
  once real fixtures exist, or whether the page should drop them then. Carried to the
  solver's Rust ticket.
- **Presets.** The report's four levels against `technique.allium`'s two deterministic
  presets and `human-solving.allium`'s four; `human-solving.allium` line 1278 already asks
  whether the presets meet. This ticket maps the levels on the page and adds nothing to
  the presets.
