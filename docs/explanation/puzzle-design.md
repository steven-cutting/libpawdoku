---
title: "Puzzle design objectives"
kind: "explanation"
audience: [contributor, maintainer, agent]
canonical_for: [puzzle_design_objectives]
requires: []
---

# Puzzle design objectives

This page carries a research report on what makes a Sudoku puzzle worth solving, and
closes with what the engine does about each objective it names: which the
specifications already meet, which are recorded as open questions in the module that
owns them, and which no rule here can decide. The maintainer received the report on
2026-09-27 under the title "What Makes a Sudoku Puzzle “Fun”? A Rigorous Guide to
Hand-Designing Elegant Sudoku". It is written for someone designing by hand. The
engine's setter is a program, [`generation.allium`](../specs/generation.allium), and
the objectives are the same; the report is the ground that module stands on, and
[decision 0012](../decisions/0012-generation-and-dev-time-judges.md) says why
generation is the engine's and why any judge of these objectives is a development-time
tool and never part of the runtime.

The report is carried cleaned to the house rules and not rewritten. The cleaning:

- Given, never the report's word for one, which survives only inside two paper titles;
  every sentence here says "given". Its other words stay its own: "house" where the
  specifications say unit, "eliminate" where they say strike, "pencil marks" where they
  say marks. The engine's words are in [Terminology](../project/terminology.md). The
  report abbreviated almost locked sets as "ALSO", which names no technique family
  anywhere; every occurrence is "ALS", the conventional abbreviation, and one that
  `technique.allium` defers under its full name. The spell checker in the docs gate
  rewrites "ALS" to "also", which is the likely origin of the report's word; the
  abbreviation is now on its allow list.
- Its citation markers were tool residue and are gone. A source is linked only where it
  was opened on 2026-09-28, and always to a whole page; the research sources are in the
  evidence table of [Modelling a human Sudoku solver](human-solving.md) with what each
  does not establish, and constructor practice is cited here only. The report's
  quotations of constructors are the report's: the pages linked from its source table
  support their gist and were not checked word for word.
- Its two flowcharts are numbered lists and its three formulas are prose. Its enjoyment
  heuristic and its difficulty vector are the report's own and no specification's: no
  fun score and no fifth player model follow from this page, because
  `human-solving.allium` excludes a single universal difficulty number and its
  `Interpretation` guarantee forbids pooling differing conditions as one rating.
- Its playtest measures are in minutes. They stay as the report's external practice,
  with one sentence where they appear saying that the engine's analogue is steps.
- Its four study puzzles are unverified. The report says they were generated and
  checked by its author; no solver exists here, so nothing on this page asserts that
  any of them is well-posed or that the path described beneath it holds. One sentence
  in the advanced puzzle's XY-Wing explanation stated the two implications the wrong
  way round; it is corrected in place and says so.

Nothing in the report is a claim this repository makes until the closing section weighs
it.

## Executive summary

A good Sudoku is not simply a valid grid with one solution, nor is a hard Sudoku necessarily a good one. The most useful way to think about a “fun” Sudoku is as a **designed sequence of deductions**. The givens are the interface, but the real authored object is the solve path.

That distinction is strongly supported by both research and constructor practice. Radek Pelánek’s empirical work on Sudoku difficulty found that human difficulty is better modeled through human-style solving than through crude structural measures. In particular, difficulty depends on both the complexity of individual deductions and the dependency structure connecting those deductions. Nishikawa and Toda go further and formalize the notion of a puzzle being solvable by a specified collection of strategies, which is very close to the practical constructor’s question: “Will my intended audience be able to get through this using the tools I expect them to know?”

This leads to the central conclusion of this report:

> **A fun Sudoku is a fair sequence of comprehensible discoveries whose difficulty is well matched to the intended solver, with enough resistance to make progress meaningful and enough structure to make the eventual breakthroughs feel earned.**

“Fun” therefore has several interacting dimensions. **Challenge** supplies resistance. **Flow** keeps that resistance near the player’s current capability. **Novelty** prevents the solve from becoming mechanical. **Elegance** makes a small number of relationships do a surprising amount of work. **Aesthetics** operate both visually, through given layout, and logically, through patterns in the solution path. **Fairness** means that the advertised puzzle actually rewards the reasoning vocabulary expected of its audience. **Solvability** and uniqueness establish trust. **Pacing** controls the alternation of progress, tension, bottlenecks, and cascades. **Satisfaction** is the emotional payoff when the puzzle’s previously hidden structure becomes visible.

Research on games offers a useful parallel. Sweetser and Wyeth’s GameFlow model connects enjoyment with appropriately matched challenge, concentration, clear goals, feedback, control, and related factors. Ryan, Rigby, and Przybylski found that perceived competence and autonomy were associated with game enjoyment and motivation. Sudoku naturally supplies a clear goal and frequent feedback. A constructor’s main job is therefore to manage **competence, uncertainty, and discovery**.

The hand constructor has a major advantage over a generator: deliberate authorship. Nikoli, the publisher that coined the name Sudoku, explicitly emphasizes the special status of human-crafted puzzles. World champion and constructor Thomas Snyder describes classic Sudokus created specifically for “artistic qualities,” and records a hand-crafted classic deliberately built to make repeated use of a particular logical step. He also praises another for its “interesting narrow solving path.” Those examples identify the essence of craft. **Hand construction lets the author manipulate not only whether deductions exist, but when they become visible and what they teach the solver to notice.**

The most important practical recommendations are:

| Design objective | What to optimize |
| --- | --- |
| Fairness | Every serious bottleneck should be resolvable with techniques appropriate to the advertised level. |
| Flow | Maintain a usable frontier of deductions. Avoid both long stretches of trivial filling and un-signaled cliffs in difficulty. |
| Elegance | Prefer deductions that arise naturally from the given geometry and interact with later deductions. |
| Novelty | Give the puzzle one or two memorable ideas rather than a random assortment of unrelated advanced techniques. |
| Pacing | Aim for opening traction, increasing organization, a recognizable central hook, then a satisfying cascade. |
| Difficulty | Control the hardest required technique, how hard it is to spot, how many prerequisites it has, and how narrow the solve path becomes. |
| Aesthetics | Treat given symmetry as composition rather than proof of quality. Logical symmetry and repeated motifs can matter even more. |
| Satisfaction | Make the key deduction materially change the grid. A difficult observation that eliminates one irrelevant candidate is usually less satisfying than one that unlocks several consequences. |

Two warnings are especially important.

First, **given count is not a difficulty rating**. McGuire, Tugemann, and Civario proved that a uniquely solvable standard 9×9 Sudoku cannot have fewer than 17 givens, but that theorem concerns uniqueness, not human difficulty. A 25-given puzzle can be gentle while a 32-given puzzle can be extremely awkward. The study grids later in this report deliberately demonstrate that phenomenon.

Second, **the hardest named technique is not the whole difficulty**. A conspicuous X-Wing can be easier than an obscure hidden single. A puzzle can also be tiring despite using only modest techniques if each deduction depends on a long chain of earlier discoveries. Pelánek’s findings strongly support treating deduction complexity and dependency structure as separate difficulty variables.

A useful conceptual model is therefore:

The report sums challenge-skill fit, discovery, elegance, progress and closure and
subtracts friction and unfairness, and says itself that this is a heuristic.

This is a **design heuristic, not a validated psychological equation**. Its value is diagnostic. When a puzzle is technically correct but not enjoyable, ask which term is failing.

## What “fun” means in Sudoku

Sudoku is unusually revealing as a game-design medium because its external rules are extremely simple while its internal reasoning space is rich. The basic classic form asks the solver to place digits so every row, column, and 3×3 region contains the required digits without repetition. Competitive organizations such as the World Puzzle Federation use classic Sudoku alongside variants in formal timed competition, while mature solving references distinguish a large hierarchy of human techniques, from singles and intersections through fish, wings, coloring, chains, Almost Locked Sets, and forcing methods.

The consequence is that there are really **two games** inside one grid:

1. the combinatorial problem, “What is the unique completed grid?”;
2. the experiential problem, “What sequence of observations will a human make while discovering it?”

A constructor concerned with fun designs primarily for the second.

### The dimensions of enjoyment

The following table gives an operational definition of each requested dimension.

| Dimension | In Sudoku, it means | Failure mode | Constructor’s test |
| --- | --- | --- | --- |
| **Challenge** | The solver must notice relationships that are not immediately obvious. | Too little becomes clerical filling. Too much becomes opaque search. | “Does the solver have to think, but still know what kind of thinking might pay off?” |
| **Flow** | Difficulty stays near the solver’s competence often enough to sustain concentration. | Long dead periods, sudden technique cliffs, or excessive triviality. | Plot the solve chronologically. Are there avoidable flat spots or spikes? |
| **Novelty** | The puzzle asks the solver to notice a pattern, interaction, or presentation that differs from routine scanning. | Every step feels interchangeable with thousands of generated puzzles. | “What will the solver remember tomorrow?” |
| **Elegance** | A small structural fact produces several consequences, or the same theme reappears in different forms. | Advanced steps exist merely because candidates happen to permit them. | “Does the key deduction explain the puzzle rather than merely advance it?” |
| **Aesthetics** | The given pattern is visually coherent, and the logical structure feels composed. | Random-looking givens or decorative symmetry that damages the solve. | View the unsolved grid from a distance, then review the deduction map. |
| **Fairness** | Required reasoning matches the stated rules, level, and reasonable solver expectations. | Hidden need for guessing or an accidental technique far above the label. | Give it blind to players from the target group. Record where they stop. |
| **Solvability** | There is one intended completion and a human route to it under the chosen technique contract. | Multiple solutions, contradiction-prone construction, or a solver-only brute-force route. | Verify uniqueness separately from human solvability. |
| **Pacing** | Progress has rhythm: traction, development, resistance, breakthrough, release. | Fifty routine placements followed by one arbitrary wall. | Record placements per deduction and time between meaningful advances. |
| **Satisfaction** | Progress feels caused by understanding. The finish resolves earlier tension. | The key step does almost nothing or the ending dribbles into bookkeeping. | “What changed because of the breakthrough?” |

The GameFlow literature is useful here because enjoyment is not equated with maximum challenge. Challenge is valuable when it remains related to skill and when players retain feedback and a sense of control. Ryan and colleagues similarly found links between enjoyment and the satisfaction of competence and autonomy. For Sudoku, that suggests a crucial distinction between **productive difficulty** and **friction**. Productive difficulty makes the player feel, “There must be something here I have not seen yet.” Friction makes the player feel, “I have no reason to believe looking longer will help.”

### Elegance is not the same as difficulty

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

### Novelty without obscurity

Novelty can be structural rather than technical.

A classic Sudoku can feel novel because:

- several pointing pairs are arranged around the center and echo each other;
- an early pair seems incidental but becomes one wing of the later breakthrough;
- the same digit repeatedly carries the solve;
- the givens suggest rotational visual symmetry while the logical path moves asymmetrically;
- a technique appears in an unusually clean geometric form;
- the puzzle repeatedly alternates local box logic with global row-column logic.

This is exactly where hand construction distinguishes itself. Nikoli’s public emphasis on human-crafted puzzles and Snyder’s explicitly themed construction practice both point toward Sudoku as an authored logical experience rather than merely a generated constraint-satisfaction instance.

### Fairness, guessing, and trust

“Fair” does **not** mean easy. It means that the puzzle keeps the contract it implicitly made with its player.

For a beginner puzzle, requiring an XY-Chain without warning is unfair even if the chain is logically impeccable. For a puzzle advertised to experts, requiring an Alternating Inference Chain may be entirely appropriate. HoDoKu’s taxonomy itself reflects this spectrum. Singles and intersections appear near the foundational end, while chains, almost locked set (ALS) structures, and forcing methods occupy progressively more specialized territory.

Forcing chains are logically legitimate. HoDoKu defines them broadly as chains establishing a contradiction or necessary truth, but places forcing methods near its “last resort” category, with forcing nets described as realistically manual only for very experienced players. That distinction matters. A constructor should decide in advance whether contradiction reasoning is part of the intended vocabulary, rather than accidentally discovering during testing that the puzzle needs it.

A publishable classic should therefore satisfy three different tests:

**Combinatorial validity:** exactly one solution.

**Strategic validity:** a route exists within the intended technique repertoire.

**Experiential validity:** ordinary target solvers can plausibly *find* that route.

Nishikawa and Toda’s work formalizes the second problem. For a set of positions and a chosen strategy set, they study whether an assignment of givens to those positions can be generated that is solvable using those strategies. The third problem still requires humans.

## Cognition, emotion, and differences among players

A Sudoku solve repeatedly cycles among **search, recognition, inference, action, and reorganization**.

At the start, the solver sees a large field of apparently unrelated information. Each placement reduces uncertainty. Candidate distributions gradually acquire recognizable form. Eventually a pattern that had been latent becomes salient: two places for a digit in a box, a pair occupying two cells, an X-Wing across distant rows, or an implication chain linking apparently unrelated candidates. HoDoKu’s large hierarchy of techniques can be understood as a vocabulary for these recurring structures.

### Pattern recognition changes what “hard” means

A novice and an expert are not merely doing the same computation at different speeds.

A novice may inspect one empty cell and ask which digits fit. An intermediate solver scans houses for conjugate positions, pairs, and locked candidates. An advanced player may look at the distribution of one candidate across the entire grid and immediately notice fish or link structure. An expert may perceive a network of strong and weak links rather than isolated candidates.

This is why difficulty labels are audience-relative. A solver who has recently learned XY-Wings can experience a clean XY-Wing as an exciting breakthrough. An expert may regard precisely the same configuration as routine visual syntax. HoDoKu’s taxonomy gives concrete evidence of how large this learned technique vocabulary can become.

Difficulty ratings also lack a universal scale. Contemporary attempts to compare ratings across Sudoku sites have found substantial inconsistency among published difficulty labels, while earlier research likewise shows that good human-difficulty models require more than superficial puzzle statistics. Thus “beginner,” “hard,” and “expert” should be treated as **audience contracts**, not mathematical properties.

### Aha moments

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

### Frustration thresholds

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

### Skill level changes the desired emotional curve

| Player | Desired experience | Good bottleneck | Typical emotional payoff | Bad design for this audience |
| --- | --- | --- | --- | --- |
| **Beginner** | “I can do this.” | Finding the next single through systematic scanning. | Growing competence and momentum. | Long candidate bookkeeping or an unexplained advanced pattern. |
| **Intermediate** | “I saw something clever.” | Locked candidates, pairs, triples, modest interactions. | First real Aha moments beyond direct placements. | A puzzle consisting entirely of singles despite being labeled medium. |
| **Advanced** | “The grid had hidden structure.” | Fish, wings, coloring, multi-stage setups. | Recognizing a global pattern and watching it unlock the grid. | Random accumulation of named techniques with no thematic coherence. |
| **Expert** | “I understood the architecture.” | Chains, unusual interactions, ALS structures, tightly controlled forcing. | Compressing a complicated candidate network into a decisive proof. | Blind exhaustive search masquerading as logical depth. |

The crucial design shift is that beginners need **frequent confirmation**, while experts can tolerate longer delayed rewards. Ryan and colleagues’ competence findings help explain why both can be enjoyable when matched to the player.

## The craft of designing Sudoku by hand

Human construction works best as **backward design plus iterative subtraction**. Instead of starting from a nearly empty grid and hoping a nice solve appears, decide what sort of solve you want and manipulate givens until the target deductions become necessary.

Nikoli explicitly champions human-crafted puzzles, while Snyder’s archive demonstrates deliberate construction around artistic qualities, themes, repeated logical devices, and narrow solve paths.

A practical workflow is:

1. Choose the target solver.
2. Define the allowed technique vocabulary.
3. Choose or construct a complete solution grid.
4. Choose the visual geometry of the givens.
5. Place a generous symmetric set of givens.
6. Remove a given or a symmetry orbit.
7. Is the solution still unique? If not, restore or replace the given and go back to
   step 6.
8. Human-solve and log every deduction.
9. Is the technique and pacing what was wanted? Too easy: remove or relocate a given and
   go back to step 6. Too hard: add a given that repairs the bottleneck and go back to
   step 8. Wrong logic: alter the givens around the target pattern and go back to step 8.
10. When it is good, blind playtest.
11. Retune difficulty and presentation.
12. Audit uniqueness and the solve path one final time.

### Start with a technique contract

Before touching givens, write something like:

> **Audience:** confident intermediate  
> **Expected knowledge:** singles, locked candidates, naked and hidden pairs  
> **Forbidden requirements:** fish, wings, chains, guessing  
> **Intended hook:** two pointing interactions, second one releases center  
> **Desired rhythm:** easy opening, one moderate stall, fast finish.

This one page of intent prevents a common failure: designing the grid first and inventing its target audience afterward.

Nishikawa and Toda’s strategy-solvability framework gives this informal practice a formal analogue. Puzzle generation can be constrained by the set of solving strategies that must suffice.

### Choose the completed grid before sculpting the givens

For hand construction, working backward from a complete valid solution is usually easier than trying to invent givens directly.

You can obtain the completed grid by:

- constructing one manually;
- transforming a known completed grid by permitted Sudoku symmetries;
- building from a patterned base and then permuting digits, rows within bands, columns within stacks, bands, stacks, or transposition.

At this stage, the completed grid is **raw material**. Its quality as a puzzle will come from the given selection.

### Treat given symmetry as composition

The most useful default for classic Sudoku is often **180-degree rotational symmetry of given positions**. It is visually stable, easy to maintain by removing givens in opposite pairs, and does not require the values themselves to be symmetric.

Symmetry is an aesthetic convention, not a Sudoku rule and not evidence of logical quality. A perfectly symmetric ugly solve is still an ugly solve. If preserving the visual pattern forces you to leave a given that destroys your key deduction, either find a different symmetric partner structure or consciously relax the symmetry.

A practical hierarchy is:

**First:** uniqueness and intended logical path.  
**Second:** pacing.  
**Third:** visual symmetry.

Do not reverse that order.

Center-cell decisions deserve special attention. Under rotational symmetry the center is its own orbit, so removing it changes the given count by one while all ordinary paired removals change it by two. That makes it useful for fine control of both appearance and given density.

### Remove givens in orbits, but solve after every meaningful change

Begin with more givens than you expect to publish. Remove one symmetry orbit. Then ask four questions:

1. Is the solution still unique?
2. What new technique has become necessary?
3. What old deduction disappeared?
4. Did the ordering of the solve improve?

The fourth question is the most “artistic.”

Suppose removing a given creates a beautiful X-Wing, but the X-Wing is irrelevant because a hidden single elsewhere solves the same cell first. Combinatorially, you succeeded. Experientially, you did not. The desired pattern exists but is not **necessary at the moment you want it noticed**.

This is why construction must be iterative.

### Design target deductions backward

For a **pointing pair**, decide which digit you want confined to a row or column inside one box. Add or remove givens so the other candidate positions for that digit inside the box disappear, while leaving at least one meaningful elimination outside.

For a **naked pair**, create two cells whose candidate sets become exactly the same pair. Then ensure at least one other unsolved cell in the house contains one or both pair digits so the pattern has work to do.

For an **X-Wing**, you need one digit to occupy exactly the same two columns in two rows, or the same two rows in two columns. HoDoKu’s basic fish taxonomy formalizes this pattern. The constructor’s problem is not merely to make an X-Wing exist. It is to control when earlier deductions strip away the extra candidates that reveal it.

For an **XY-Wing**, create a pivot `{x,y}` seeing two wings `{x,z}` and `{y,z}`. The wings need not see each other. A cell seeing both wings can lose `z`. HoDoKu includes XY-Wing, XYZ-Wing, and W-Wing as a distinct technique family. Again, the craft lies in making those three bivalue cells emerge at the appropriate stage rather than being visible trivially from the start or destroyed prematurely.

For an **expert chain**, work backward from the conclusion. Decide the candidate you want eliminated, identify the implication chain that proves it, and then protect the links by ensuring that no simpler deduction collapses them early. HoDoKu defines forcing-chain logic in terms of chains producing necessary truths or contradictions.

### Construct dependencies, not merely techniques

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

### Use a solve log

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

### Use computation as quality control without surrendering authorship

“Designed by hand” need not mean “never checked by a computer.”

A sensible division of labor is:

**Human constructor:** chooses aesthetics, target techniques, given placement, theme, pacing, and revisions.

**Software:** verifies uniqueness, catches accidental multiple solutions, inventories possible techniques, and detects unintended logical cliffs.

This preserves exactly the element Nikoli values in human-crafted puzzles while using computational methods for the combinatorial checks humans are worst at.

The theorem on the fewest givens is a good illustration of why this distinction matters. Proving that no 16-given standard Sudoku has a unique solution required exhaustive computational work over the enormous solution space. You should not make heroic manual uniqueness checking the centerpiece of your construction process.

## Engineering difficulty, pacing, and skill-level targets

Difficulty should be treated as a vector rather than a scalar.

A useful constructor model is:

The report writes difficulty as six components: technique depth, spotting difficulty,
dependency length, bottleneck narrowness, candidate volume (the visual noise of the
marks) and the wrong-area penalty, the cost of choosing the wrong place to look.

This model is a design framework rather than a standardized rating formula. It reflects the research result that step complexity alone is insufficient. Dependency structure matters too.

### A useful difficulty taxonomy

1. Beginner: full houses, naked singles, hidden singles.
2. Intermediate: locked candidates, pairs, triples.
3. Advanced: X-Wing, Swordfish, wings, colouring.
4. Expert: XY-Chains, AICs, ALS patterns, forcing chains.

This ordering is deliberately approximate. HoDoKu catalogues these families but real difficulty depends heavily on presentation and interaction. Difficulty labels also vary substantially across publishers and websites, so the categories should be calibrated against your own audience.

### Skill-level design targets

The given counts below are **starting heuristics for construction, not definitions of difficulty**. They are intentionally broad.

| Target | Useful starting given range | Core required techniques | Bottleneck design | Desired pacing |
| --- | --- | --- | --- | --- |
| **Beginner** | Roughly 36 to 45 | Full houses, naked singles, obvious hidden singles | None longer than a few scans | Continuous progress |
| **Intermediate** | Roughly 30 to 37 | Singles, locked candidates, pairs, occasional triple | One or two clearly discoverable reductions | Progress, pause, insight, cascade |
| **Advanced** | Roughly 26 to 33 | Subsets plus fish, wings, coloring or equivalent | Several prerequisites before a centerpiece deduction | Multiple waves and one memorable peak |
| **Expert** | Roughly 22 to 30 | Chains, difficult fish, ALS interactions, controlled forcing | Narrow logical frontier with substantial candidate structure | Long preparation, major compression, strong release |

These ranges should never override the solve path. The 17-given lower bound concerns uniquely solvable standard 9×9 Sudoku, not difficulty. Nishikawa and Toda’s work likewise separates given selection from solvability by a particular strategy set.

Indeed, the beginner sample later in this report has **only 32 givens and resolves entirely through naked singles**. That is exactly why counting givens is a poor substitute for solving the puzzle.

### Tuning a puzzle downward

When a puzzle is too hard, do not simply add arbitrary givens.

First locate the exact point where the target solver stalls. Then ask why.

If the obstacle is an X-Wing you do not want, add a given that eliminates one of the candidates the X-Wing was needed to remove.

If the problem is spotting difficulty, add a given that reduces visual noise around the intended pattern without solving the pattern directly.

If the problem is a dependency chain that is too long, add a given midway through that chain. This preserves the later hook while removing one prerequisite.

If there are too many simultaneous search fronts, add a given that closes an irrelevant region and directs attention.

For beginners, the strongest rule is:

> **Every placement should make at least one reasonably visible next placement more likely.**

You are effectively building a chain of invitations.

### Tuning a puzzle upward

Removing givens randomly is a poor difficulty-control mechanism.

Instead, remove a given because you can predict what its absence will do:

- turn a single into a pair;
- preserve an extra candidate until a locked-candidate step removes it;
- hide the second half of a fish until later;
- prevent a shortcut;
- make two solving fronts depend on each other;
- force the player to shift from cell scanning to candidate scanning.

That is controlled difficulty.

A particularly effective advanced technique is **shortcut suppression**. When a beautiful intended deduction exists but a simpler route bypasses it, identify the given that enables the shortcut. Remove or relocate that given while preserving uniqueness.

### Branching has two meanings

Constructors should distinguish **logical branching** from **guessing branching**.

Logical branching means several valid deductions are simultaneously available. Some of this is good. It gives players agency and accommodates different scanning styles, which is consistent with the broader importance of perceived autonomy in game enjoyment.

Too much logical branching, however, can dilute a theme because every solver follows a different path and the intended centerpiece becomes optional.

Guessing branching means the solver chooses a candidate without proof and explores consequences. For a classic puzzle advertised as logically solvable, that should usually be unnecessary. If contradiction chains are part of your expert-level vocabulary, make that explicit and keep them short enough to constitute reasoning rather than blind tree search. HoDoKu makes a similar conceptual separation between recognizable chain logic, forcing nets, and brute-force style last resorts.

### Pacing should look like a story arc

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

### Symmetry and difficulty sometimes conflict

Visual symmetry can force you to remove or restore givens in pairs even when only one of the two positions matters logically.

There are three reasonable responses:

1. preserve symmetry and rework both sides of the logical structure;
2. permit a slight asymmetry because the solve improves substantially;
3. change to another coherent symmetry scheme.

The wrong response is to sacrifice the solving experience merely to obtain an attractive silhouette.

The best handcrafted puzzles can make visual and logical symmetry reinforce each other. For example, symmetric given removal can create matching candidate structures on opposite sides of the grid, only for the solve to reveal that one side resolves first and unlocks its counterpart.

## Annotated study puzzles from beginner to expert, unverified

The following four grids are **original study puzzles generated and analyzed for this report**. I used them because their behavior can be reported reproducibly rather than attributing someone else’s construction. Each was independently checked to have exactly one solution. I then analyzed it with a deterministic human-style strategy hierarchy. Solver order matters, so the annotations show **a valid representative path, not a claim that every human must follow the same path**.

Those are the report's claims about its own grids. Nothing in this repository has verified any of them: no solver exists here yet, so neither the uniqueness of any grid nor the path described beneath it has been checked, and the four are carried as candidate fixtures only, as the closing section says.

A dot is an empty cell. Coordinates use `rNcM`, meaning row N, column M.

### Beginner study puzzle

The report states 32 givens, a unique solution and singles only in its tested path; none of that has been verified here.

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

**Design lesson:** a low given count need not imply a difficult puzzle. This is a concrete counterexample to the common constructor instinct to classify by givens. The research literature independently supports using human solving behavior rather than given count alone for difficulty assessment.

For a true novice publication, I would probably add several givens despite their being logically unnecessary. That would make the opening less visually intimidating. This distinction is important: **mathematical redundancy can be experiential value**.

### Intermediate study puzzle

The report states 32 givens, a unique solution, and singles plus locked candidates; none of that has been verified here.

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

### Advanced study puzzle

The report states 29 givens, a unique solution, and locked candidates, a pair, an X-Wing and an XY-Wing in its tested path; none of that has been verified here.

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

If the pivot is 2, the wing at r6c1 loses its 2 and is forced to 6. If the pivot is 4, the wing at r1c3 loses its 4 and is forced to 6. In either case, any cell seeing both wings cannot itself be 6. (The report's sentence here had the two implications the other way round; this copy corrects it, and the deduction it reaches is unchanged.)

The tested path therefore removes 6 from `r3c1` and `r6c3`.

HoDoKu groups XY-Wing with other wing techniques precisely because the deduction is built from interacting bivalue cells rather than one candidate’s global distribution.

**Intended hook:** the puzzle changes representational mode twice. First the solver sees a **single-digit global pattern** in the X-Wing. Later the solver has to see a **multi-digit relational pattern** in the XY-Wing.

This is a useful advanced-puzzle pattern. Do not merely increase the technical difficulty. Change what kind of structure the solver must recognize.

**Construction lesson:** an advanced centerpiece should preferably have aftermath. When hand-tuning, remove or move givens until the X-Wing elimination produces a candidate structure that helps form the later wing. Then the two techniques feel like chapters of the same story rather than unrelated tricks.

### Expert study puzzle

The report states 25 givens, a unique solution, that its tested singles, intersection and subset repertoire stalls, and that a short contradiction chain provides a representative expert continuation; none of that has been verified here.

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

So the report concludes that r3c9 is 2.

This is a short contradiction-style forcing argument. HoDoKu defines forcing chains as implications that establish a necessary truth or contradiction, while treating more elaborate forcing structures as high-end or last-resort techniques.

**Intended hook:** the player stops searching for a static shape and begins reasoning about the **consequences of a proposition**.

That is an important threshold between advanced and expert Sudoku.

It is also a taste issue. Some expert audiences enjoy concise contradiction reasoning. Others prefer every difficult deduction expressed as an AIC, XY-Chain, ALS relationship, or another non-bifurcating representation. For the latter audience, this puzzle should be retuned until the same dependency becomes visible as a cleaner chain.

The construction principle is broader than the sample:

> Expert difficulty should come from **compressed logical architecture**, not from asking the player to perform arbitrary brute-force search.

HoDoKu’s separation of chain reasoning from forcing nets and brute-force-style methods is helpful when deciding where to set that boundary.

## Playtesting, manual-design checklist, and source priorities

A constructor is too familiar with their own puzzle to be its final judge. Once you know the intended hook, you cannot accurately simulate the experience of not knowing it.

That is why **blind playtesting is not optional for serious hand construction**.

### A rigorous playtesting protocol

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

Those are the report's measures of a human playtest, taken in minutes, and they stay
external practice here: the engine's analogue is steps, since `human-solving.allium`
excludes predictions in minutes, so the two time rows become the total steps of an
attempt and the longest run of looks without a step.

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

### How to interpret playtest disagreement

Suppose four intermediate testers behave as follows:

- one solves in 8 minutes;
- two solve in 13 to 16 minutes;
- one stalls for 12 minutes on an X-Wing you did not intend.

Do not simply average their times.

Inspect the stalled player’s state. If the X-Wing was genuinely necessary because they took a legitimate alternate route, your puzzle may be **path-dependent in difficulty**. That matters even if your own intended solve never needs the technique.

This is one reason narrowly authored paths are attractive. Snyder explicitly describes the interest of a “narrow solving path,” though absolute linearity is not necessary or always desirable. A little branching gives agency. Too much can make difficulty unstable.

### Manual construction checklist

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

- [ ] The given pattern looks deliberate.
- [ ] Symmetry supports rather than damages the solve.
- [ ] There are no visibly awkward isolated givens unless they serve a purpose.
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
- [ ] Difficulty labels were calibrated from real solvers, not given count.

**Final polish:**

- [ ] Every given earns its place logically, aesthetically, or pedagogically.
- [ ] I have considered whether removing each apparently redundant given actually improves the experience.
- [ ] I have considered whether adding a logically redundant given improves approachability.
- [ ] The puzzle has a reason to exist beyond being another valid Sudoku.

### A practical final rubric

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

### Sources worth prioritizing

The strongest source base combines mathematical research, human-difficulty research, constructor practice, and community technique references. No single source covers the full craft.

The report's own citation markers were tool residue and are gone. A name below is linked only where the page behind it was opened on 2026-09-28, and each link is to a whole page. The four research sources also sit in the evidence table of [Modelling a human Sudoku solver](human-solving.md), beside Pelánek, with what each does not establish; the constructor practice is cited here and not there. Snyder's Friday Puzzle series could not be opened and is dropped; his archive at Grandmaster Puzzles could, and describes a classic Sudoku designed to have a narrow solving path.

| Priority | Source | Why it matters |
| --- | --- | --- |
| **Essential research** | [Radek Pelánek](https://arxiv.org/abs/1403.7373), *Difficulty Rating of Sudoku Puzzles: An Overview and Evaluation* | One of the most useful research treatments of human Sudoku difficulty. It separates individual-step complexity from dependency structure and evaluates measures against real solving data. |
| **Essential research** | [Kohei Nishikawa and Takahisa Toda](https://arxiv.org/abs/2005.14098), *Exact Method for Generating Strategy-Solvable Sudoku Clues* | Formalizes the idea that puzzle generation can target a specified repertoire of human strategies. Extremely relevant to difficulty-controlled construction. |
| **Mathematical foundation** | [Gary McGuire, Bastian Tugemann, and Gilles Civario](https://arxiv.org/abs/1201.0749), *There is no 16-Clue Sudoku* | Establishes the 17-given minimum for uniquely solvable standard 9×9 Sudoku. Useful mainly as a warning not to confuse given count with human difficulty. |
| **Enjoyment theory** | [Sweetser and Wyeth](https://eprints.qut.edu.au/44776/), *GameFlow: A Model for Evaluating Player Enjoyment in Games* | Useful framework for challenge-skill balance, concentration, goals, feedback, and player control. |
| **Motivation theory** | [Ryan, Rigby, and Przybylski](https://doi.org/10.1007/s11031-006-9051-8), *The Motivational Pull of Video Games* | Provides empirical support for competence and autonomy as important components of game enjoyment and motivation. Sudoku is not the paper’s subject, so application to Sudoku should be treated as principled transfer rather than direct evidence. |
| **Constructor practice** | [Thomas Snyder](https://www.gmpuzzles.com/blog/about-thomas-snyder/), Grandmaster Puzzles | Especially valuable because Snyder explicitly discusses puzzles made for artistic qualities, narrow solve paths, and logical behavior achievable through hand-crafting. |
| **Handcrafted tradition** | [Nikoli](https://www.nikoli.co.jp/en/) | Important primary source for the publishing tradition that popularized Sudoku under that name and continues to emphasize human-crafted puzzle design. |
| **Technique reference** | HoDoKu, linked technique by technique from [Modelling a human Sudoku solver](human-solving.md) | Exceptionally useful operational taxonomy of singles, intersections, subsets, fish, wings, coloring, chains, ALS methods, and forcing techniques. It is a community/technical reference, not peer-reviewed difficulty research. |
| **Competition reference** | [World Puzzle Federation Sudoku Grand Prix](https://gp.worldpuzzle.org/) | Useful for seeing how Sudoku is presented, timed, and scored for high-level human solvers and for studying competition puzzle archives. |
| **Contemporary handcrafted ecosystem** | [Cracking the Cryptic puzzle collections](https://www.studiogoya.co/cracking-the-cryptic/index.html) | Useful for studying modern curated constructor culture. Its classic Sudoku collection explicitly emphasizes handcrafted and curated work from recognized constructors. |

The hierarchy among these sources matters. For questions such as uniqueness and given minima, prefer mathematical research. For human difficulty, prefer empirical difficulty research. For the vocabulary and mechanics of a technique, references such as HoDoKu are excellent. For the question “What does beautiful hand construction look like?”, the archives and commentary of elite constructors such as Snyder, together with Nikoli’s long-running human-construction tradition, are more informative than an automated rating formula.

The resulting philosophy can be stated compactly:

**Validity is the floor. Uniqueness establishes trust. Technique selection controls accessibility. Dependency design controls depth. Pacing controls emotion. Theming creates identity. Playtesting turns an intended experience into a reliable one.**

A machine can establish that a Sudoku has one solution. A sophisticated solver can estimate what techniques might solve it. What distinguishes excellent hand construction is the additional act of arranging those deductions so that a human encounters them in a meaningful order. That is the difference between a Sudoku that is merely solvable and one that feels *composed*.

## What the engine does about it

One row per objective: the report's, and the ones the specifications already care about
that the report does not name. Every file and line was re-read against the tree on
2026-09-28. The classes:

- **Satisfied now**: the specifications already do it.
- **Adjustment**: an addition to what is already planned, derivable from outputs the
  specifications already expose.
- **Algorithmic**: a new mechanism a specification must state before it is built.
- **Classifier**: a development-time judge can help, over the engine's serialised
  outputs; nothing at runtime ([decision 0012](../decisions/0012-generation-and-dev-time-judges.md)).
- **Product**: a decision the maintainer owns, recorded as an `open question` in the
  module that owns it and answered by nobody else.

| Objective | What the specifications already do | Class | Where it lands | Note |
| --- | --- | --- | --- | --- |
| Well-posed givens (exactly one solution) | `sudoku.allium:169` requires `solution_count = 1`; `sudoku.allium:316-323` `PuzzleSetting` guarantees only well-posed puzzles are posed; `solver.allium:292,617,623` stops at the second solution and ties the verdict to it. | satisfied now | none | The report's "combinatorial validity" is already the floor. A generator's output goes through `Solving` (`solver.allium:607`) before it is posed. |
| Partially completed grid; no maximum on givens | `sudoku.allium:157` requires `givens.count < side*side`; 17 is named as the fewest that can pass (`sudoku.allium:166`). | satisfied now | none | No target count exists and none should become a rule; see the next row. |
| Given count is not difficulty | `docs/explanation/solving-sudoku.md:440` and `:450` already say few givens need not mean hard and that count is a poor proxy. No specification reads the count as a rating. | satisfied now | this page restates the claim; the report's 32-given singles-only example is above, unverified; `generation.allium:130` records the report's per-level count ranges as construction starting points only | The generator must never rate by count. |
| Fairness / technique contract (required techniques, forbidden techniques, no guessing) | `technique.allium:676-710` profiles carry a repertoire; `reach.allium:153` names the hardest step; `reach.allium:359,369` stalled means this profile sees nothing more and limits only hide; `effort.allium:84,290` counts escalations past the profile. | adjustment; algorithmic | `generation.allium:126` asks the contract's shape (a profile plus a ceiling, or a required set and a forbidden set), in the sense of Nishikawa and Toda's strategy-solvable generation; `reach.allium:425` already asks what a stalled rating names | Rating a candidate against a profile exists. Generating *to* a contract is new. What one `Rate` run for the target profile can check: it ends solved, which excludes guessing because `reach.allium:38` stalls rather than guesses; and the hardest step is within the ceiling. A forbidden set is not the repertoire's complement: a forbidden technique the profile holds can still be taken, and `LimitsOnlyHide` (`reach.allium:369`) proves absence only of what the profile does not hold. Enforcing it needs an explicit check, either by hiding the technique from the repertoire before the run or by checking each step in `RunResult` against the set, and the open question asks which. A solved run does not show that a *required* technique was needed, only that it was used; whether the contract demands that, and how it is checked, is that open question, read with the shortcut row below. |
| Solvability by a human route (strategic validity) | `reach.allium:344` `Rating`, `effort.allium:261` `Pricing`, `lapse.allium:428` `Tackling`, `human-solving.allium:1152` `AssessmentService` each produce a route or a distribution of attempts. | satisfied now | none | Four models, none of them a scalar. The report's "experiential validity" (people can find it) stays a human playtest; see the playtest row. |
| Challenge (hardest required step) | `reach.allium:153` `is_hardest`; `effort.allium:129` `is_dearest`; ladder order at `technique.allium:543-547`. | satisfied now, for one run; adjustment across routes | none for the hardest step a profile took; `reach.allium:429` for whether the puzzle *requires* it | This is the report's technique depth. `is_hardest` names the hardest step of the run it belongs to, under one profile and order; that another route avoids it is the shortcut and path-dependence rows below, and a generator that reads `is_hardest` as "required" accepts a centrepiece a shortcut bypasses. |
| Difficulty vector (technique depth, spotting difficulty, dependency length, bottleneck narrowness, candidate volume, wrong-area penalty) | Depth: `reach.allium:153`. Spotting: `effort.allium:106-108` `search_effort`, a black box, open at `effort.allium:306`; `human-solving.allium:829` notice draws. Dependency length: `human-solving.allium:437` `dependency_depth`. Narrowness, volume, wrong-area penalty: none. | adjustment (depth, spotting, length); algorithmic (narrowness, volume, penalty) | this page maps the vector onto the four models; `technique.allium:1164` asks whether spotting is a property of a pattern on a grid; `reach.allium:427` asks for the dependency edges that make length deterministic; `effort.allium:318` asks whether all six map onto the price terms, with volume as candidate count and narrowness as the count of alternatives at a step; `human-solving.allium:1054` `EffortAccounting` already pays fruitless searches, which is the penalty | No fifth model and no combined scalar: `human-solving.allium:33` excludes "a single universal difficulty number". Each component stays reported separately. |
| Pacing: routine-run length between hook steps | `reach.allium:398-415` `RunResult` exposes every step in order with its technique; `effort.allium:281-298` `PricedRun` likewise with costs. | adjustment | this page names "hook" as the report's term for a step that stands above a run's routine; `reach.allium:427` poses the cutoff (a rank above the run's median, above the profile's floor, or one the caller names) and whether the measures are derived values of a run or left to whatever reads `RunResult`; `technique.allium:1166` asks whether a hook is a property of a deduction or of a run | Derivable from existing outputs by a consumer. Whichever cutoff is chosen is a derived measure over `RunResult.steps`, not a new run. |
| Pacing: cascade size after a step | `reach.allium:398-415` exposes steps but not which earlier step licensed which later one. | algorithmic | `reach.allium:427`: whether a `Step` records the steps whose eliminations its proof rests on, so consequence counts can be read | Pelánek's dependency structure. Without dependency edges a cascade is only "steps until the next hook", which conflates causation with order. |
| Pacing: dependency length (prerequisites before the centrepiece) | `human-solving.allium:437` `dependency_depth` distribution. | adjustment | `reach.allium:427`, the same question, so `reach.allium` can report it deterministically | Today only the seeded model reports it; the deterministic models cannot. |
| Satisfaction: the key deduction has consequences | None; a step's eliminations are in its proof but no count of what they unlock. | algorithmic | `reach.allium:427`, the dependency edges; this page states the objective | Measured as cascade size once dependency edges exist. |
| Flow: no long flat stretches, no unsignalled cliffs | `effort.allium:84` escalations mark cliffs relative to a profile; long runs of rank 1 to 4 steps are visible in `RunResult`. | adjustment | this page; `generation.allium:130` asks whether the run-length thresholds a candidate must meet are obligations per profile or a consumer's config | The report's story arc (orientation, compression, resistance, hook, release, echo, closure) is a target shape over the step sequence. |
| Shortcut suppression (a simpler route bypasses the intended hook) | `reach.allium:25` takes the lowest technique on the ladder that is licensed, so a shortcut is what `Rate` finds; which of several tied deductions is taken is the implementation's (`reach.allium:41-43`); `reach.allium:423` notes order can change what is reachable. | algorithmic | `generation.allium:128`: what rating with the intended technique removed from the repertoire can show, whether other profiles and orders are searched, and whether a spent budget is reported as inconclusive | `LimitsOnlyHide` (`reach.allium:369`) makes the removal clean: hiding the technique changes nothing else. But a stall under that profile and order is evidence that *this route* needs the technique, not proof that every route does: `StalledIsNotUnsolvable` (`reach.allium:359`) says a stall speaks only for the profile, and `reach.allium:423` says a strike one order takes can close off a pattern another needed. |
| Path-dependence of difficulty across alternate routes | `reach.allium:423` open question on monotonicity; `technique.allium:891` `SameGridSameDeductions`; `human-solving.allium:1184-1191` reports distributions, never one route. | adjustment | this page: rate with several profiles and orders, report the spread; `reach.allium:429` asks whether a rating reports the hardest step over several orders; `human-solving.allium` already does this by seed | The report's "one tester stalled on an unintended X-Wing" is a profile with a different scanning order. No new mechanism. |
| Aesthetics: given symmetry (half-turn rotation, orbits, centre cell) | None. The only "symmetry" in the specifications is fixture validation at `technique.allium:1113`. | algorithmic; product | `generation.allium:124`: which schemes, whether a scheme is asked for or reported, whether asymmetric givens are ever kept, and that symmetry ranks below well-posedness and the contract | Purely geometric, deterministic, easy to specify and test. Symmetry ranks third in the report's hierarchy, below uniqueness and the intended path; the module says so. |
| Aesthetics: layout looks deliberate, no awkward isolated givens | None. | classifier; product | this page names it; a development-time judge over a rendered or serialised grid; whether any judge runs, and over which set, follows from `generation.allium:122` (where puzzles are made) and [decision 0012](../decisions/0012-generation-and-dev-time-judges.md) | A human or a classifier judges; nothing in the engine can. |
| Elegance (one relationship does much work; theme recurs) | None beyond proof size (`technique.allium:597-600` load counts propositions). | algorithmic (proxy); classifier | this page: the proxy is cascade size divided by proof load, which needs `reach.allium:427`'s dependency edges; a development-time judge over the solve log for the rest | Whether a puzzle "is about" something is a judgement, not a measurement. |
| Novelty (a memorable idea, not a random assortment) | None. | classifier; product | this page; a development-time judge over `RunResult` or `PricedRun` serialised; a curated set is where novelty is enforced, and whether there is one is `generation.allium:122` (where puzzles are made) and [decision 0012](../decisions/0012-generation-and-dev-time-judges.md) | A generator can bound the count of distinct advanced families per puzzle (algorithmic); "memorable" is a judgement. |
| Aha quality (preparation, compression, consequence) | Preparation and consequence follow from dependency edges; compression is proof load at `technique.allium:597`. | adjustment; classifier | this page; measured from the same derived pacing values | No new run kind. |
| Finish (closing cascade, not bookkeeping) | Visible in `RunResult` as the rank profile of the last steps. | adjustment | this page; `generation.allium:130` for the threshold | Derived measure. |
| Player-level targets (beginner, intermediate, advanced, expert) | `technique.allium:676-710` novice and expert presets; `human-solving.allium:483-560` four presets; the presets do not meet (`human-solving.allium:1278`). | adjustment; product | this page maps the report's four levels onto the ladder families and the presets, in the table below; whether two more deterministic presets are added stays with `human-solving.allium:1278`, and nothing here changes a preset | The report's per-level technique lists match the ladder families. |
| Difficulty labels are audience contracts, not properties | `human-solving.allium:33` excludes a universal number; `:1184-1191` `Interpretation` forbids pooling conditions. | satisfied now | this page cites it | Labels belong to the game, which picks a profile per label. |
| Technique vocabulary (singles to forcing) | 29 techniques at `technique.allium:543-668`; deferred families at `technique.allium:50-52`; chain limits at `technique.allium:529-530`. | satisfied now | none | The report's taxonomy is a subset of the catalogue. ALS and forcing nets are deferred, which caps the expert level the engine can rate. |
| Contradiction reasoning is opt-in, short, and declared | `forcing_chain` bounded by `max_forcing_paths` and `max_chain_links` (`technique.allium:1001-1003`); a profile omits it from its repertoire. | satisfied now | none | The report's expert-audience taste question is a profile choice. |
| Guessing is not a deduction | `lapse.allium:118-119` counts guesses and withdrawn guesses; `reach.allium:38` stalls rather than guesses; `solver.allium` guesses but explains nothing (`:42-45`). | satisfied now | none | "Guessing branching" versus "logical branching" is already the reach/lapse split. |
| Uniqueness techniques need a promise | `technique.allium:878-881`; `reach.allium:205` promises only when rating. | satisfied now | none | A generator that has verified uniqueness may rate with the promise; the report does not raise this. |
| Hints | `reach.allium:376-380` `Hinting`, `NextStep`. | satisfied now | none | Out of the report's scope; kept as an existing objective. |
| Deterministic runs, exact replay | `reach.allium:365`, `lapse.allium:456` `SameInputSameRun`; `human-solving.allium:1093` `ExactReplay`; `technique.allium:53-54`. | satisfied now | `generation.allium:34` inherits it: same seed, same profile, same config and the same three versions (`generation_version`, `catalogue_version`, `random_version` at `generation.allium:109-115`, as `human-solving.allium`'s `ExactReplay` names them), same givens | Generation draws from the randomness boundary (`crates/pawdoku/src/random.rs`) and nothing else. |
| Step budgets, never time | `human-solving.allium:20,87,93` limits; `technique.allium:238-239` budget and fatigue; the report's minutes are excluded at `human-solving.allium:32-33`. | satisfied now | `generation.allium:85-116` states a step budget over the module's own operations and a candidate limit, both the caller's figures; the work inside an acceptance call ends of itself (`reach.allium:348` `AlwaysEnds`, the solver's stop at the second solution) and is bounded in count by the candidate limit, not in size by the step budget; `:134` asks what a spent budget reports and whether that inner work is charged | The report's "total solving time" and "longest no-progress interval" become steps and the longest run of looks without a step. |
| Playtest protocol (blind solve, stall location, technique at stall, recall) | `human-solving.allium:418-438` reports stops, technique counts, obstacles, guesses; `docs/explanation/human-solving.md` names calibration by consented human traces. | adjustment; product | this page carries the protocol and the stall measures in steps; whether the game records anything is `board.allium:785`'s open question and the game's to answer | The simulated assessment is the in-engine analogue; human playtesting is a game and process concern, recorded here so the measures line up. |
| The 0 to 4 rubric (challenge fit, fairness, pacing, elegance, novelty, visual aesthetics, aha, finish) | None. | classifier; product | this page, as a development-time judging schema over serialised `RunResult`, `PricedRun`, `TackledRun` and `AssessmentSummary` plus the givens; not a specification value; `human-solving.allium:1284` asks which `AssessmentSummary` figures and `AssessmentDetail` counts a generator may target and which only a judge reads | Eight independent dimensions, no sum. A calibrated-probability classifier is a candidate judge at development time only; nothing calls out at runtime. |
| Curated set versus on-device generation | None until this page: `docs/project/purpose-and-scope.md` and `docs/explanation/architecture.md` disagreed on whether generation was planned. | product | `docs/project/purpose-and-scope.md:40-46` and `docs/explanation/architecture.md:84-88` now both say generation is a planned module and where puzzles are made is open; `generation.allium:122` records it; decision 0012 | Decided by the maintainer on 2026-09-27: the engine specifies generation; where puzzles are made stays open. |
| Redundant givens (a given that is logically unnecessary but makes the opening less forbidding) | None; `sudoku.allium:157` only caps the count, and no module asks a generator to keep a given it does not need. | product | `generation.allium:132`: whether removal stops at a floor per level, whether a redundant given is asked for or reported, or whether every removable given goes | The report says redundancy can be experiential value. A generator that removes every given it can works against that, so whether a scheme keeps a floor of givens per level is a product decision. The map in the ticket missed this row. |
| Sources (Pelánek, Nishikawa and Toda, McGuire et al., GameFlow, Ryan et al., Snyder, Nikoli, HoDoKu, WPF, Cracking the Cryptic) | `docs/explanation/human-solving.md:88-106` holds the evidence table; Pelánek 2014 was already in it. | adjustment | `docs/explanation/human-solving.md:103-106`, one row per research source that could be opened; constructor practice is linked from the source table above | The report's citation markers were dangling; a source entered only with a checked link, and every link passed `just check-links-online` on 2026-09-28. |
| Study puzzles (four grids, 32/32/29/25 givens) | No solver exists in code; `technique.allium:1042-1069` name example fixtures. | adjustment | this page lists them as candidate fixtures marked unverified; they become `AdvancedExamples` fixtures only after the solver verifies uniqueness and the stated path | Never asserted well-posed or solvable as described until checked. |

### The report's levels against the presets

The report's four levels and the techniques it expects of each, against the ladder
families of `technique.allium` and the presets that exist. The mapping is this page's;
no preset changes, and whether the deterministic presets should meet the seeded ones is
`human-solving.allium`'s open question.

| Report level | Report's core techniques | Ladder families | Deterministic preset (`technique.allium`) | Seeded preset (`human-solving.allium`) |
| --- | --- | --- | --- | --- |
| Beginner | Full houses, naked singles, hidden singles | `singles` (rank 1 to 4) | `novice` knows the four singles but keeps no marks, so only `full_unit` and `cross_hatch` are within reach | `beginner` (practised beginner): the four singles familiar, notes on paper |
| Intermediate | Singles, locked candidates, pairs, occasional triple | `locked_candidates`, `naked_subsets`, `hidden_subsets` (rank 5 to 12) | none; made by moving `novice`'s repertoire, marking and spans | `intermediate_player`: singles, pointing, claiming, pairs, triples and `x_wing` familiar |
| Advanced | Subsets plus fish, wings, colouring | `fish`, `wings`, `colouring`, the single-digit `chains` (rank 13 to 23) | none; between the two presets | none; between `intermediate_player` and `expert_player` |
| Expert | Chains, difficult fish, ALS interactions, controlled forcing | `chains`, `contradiction`, `uniqueness` (rank 24 to 29); ALS patterns, finned fish and forcing nets are deferred | `expert`: the whole catalogue, systematic, full marks | `expert_player`: all 29 familiar |

## Related pages

- [Modelling a human Sudoku solver](human-solving.md)
- [Strategies for solving Sudoku](solving-sudoku.md)
- [Specifications](specifications.md)
- [Decision 0012](../decisions/0012-generation-and-dev-time-judges.md)
- [Terminology](../project/terminology.md)
