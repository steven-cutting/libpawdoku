---
title: "Terminology"
kind: "project"
audience: [contributor, maintainer, operator, agent]
canonical_for: [project_terminology]
requires: []
---

# Terminology

These words mean one thing here. Most of them come from the specifications, and using
them loosely is how a review ends up arguing about vocabulary instead of behaviour.

## The engine

The rows below are drawn from the modules' own Vocabulary sections, each of which picks
one word where the literature of Sudoku has several. The Sudoku terms are
`sudoku.allium`'s, the solving terms are `solver.allium`'s, the terms of a puzzle in play
are `board.allium`'s, the terms of a person solving are `technique.allium`'s and its
four models': `reach.allium`, `effort.allium`, `lapse.allium` and `human-solving.allium`,
and the terms of a setter that is a program are `generation.allium`'s.
When a row and a module disagree, the module is right and the row is corrected.

| Term | Meaning |
| --- | --- |
| Player | The person the four models describe. A profile is a parameterized one; nothing here knows who is at a device. |
| Puzzle | One game of Sudoku: a grid, the givens its setter supplied, and whether it is solved. |
| Setter | Whoever or whatever supplies a puzzle's givens. The rules say what it may pose, never how it finds one. |
| Cell | One position of the grid, at a row and a column, holding a digit or empty. A rules concept: a consumer's surface is what draws one. |
| Digit | 1 to 9. The only thing a cell holds. |
| Box | One of the nine 3 by 3 squares that tile the grid. Never block, region or subgrid. Its band counts boxes from the top and its stack from the left. |
| Unit | A row, a column or a box: nine cells that must end up holding nine different digits. |
| Peer | Another cell sharing a unit with this one. |
| Given | A digit the setter wrote in before play. The player can never change it. Never clue. |
| Well-posed | Having exactly one solution. Every puzzle a setter may pose is. Never proper. |
| Conflict | Two peers holding the same digit. The rules permit it and both cells are in it. |
| Solved | Every cell holds a digit and none conflicts. Final: a solved puzzle takes no more moves. |
| Board | The puzzle in play: a puzzle, a note in every cell, the moves made and the checks asked. `board.allium`'s, as are the six terms below. A board draws nothing; what draws one is a consumer's. |
| Move | One of the four things a player does to a board: place a digit, erase one, write a mark or strike one. Recorded with what it displaced, so it can be taken back exactly. Standing until undone; undone until re-taken, or discarded by a new move. |
| Undo, redo | Taking back the latest standing move, and re-taking the latest move undone. Both stop once the puzzle is solved. |
| Reading back | The board as it stood once any move and no later one had been made, read from the moves and never by taking any back. |
| Check | On the board: the player's question whether one cell's digit is the solution's, answered yes or no and never with a digit. Every check is kept and none is a move. In `lapse.allium` a different thing under the same word, below. |
| Record, reopen | A record is a board written down whole; to reopen one is to have that board again from it: the same puzzle, digits, notes, moves, undone moves and checks. The specification leaves what a record is made of open; the engine's `Record` holds the givens, the moves, how many are undone and the checks, and nothing worked out from them or read from the solution. Where a record is kept, and in what format, is the caller's. |
| Upkeep | Striking a placed digit from its peers' marks. On the board it is always whole, hidden notes included; in `technique.allium` a profile's upkeep is the unit kinds it reaches. |
| Candidate | A digit a cell may still hold while givens are being solved. `solver.allium`'s, as are the seven terms below. |
| Search | The solving of one set of givens, from the moment they are handed over to its verdict. |
| Branch | One grid of candidates within a search. One works at a time, and a guess splits it into a child for each candidate of one cell. |
| Propagate | To strike a placed digit from its peers' candidates and place the singles that leaves, until nothing more follows. |
| Guess | A digit tried in a cell when propagation has run out, and the branch opened to try it. Never trial or assumption. |
| Contradiction | A branch that can hold no solution: a cell with no candidate, a unit with no place for a digit, or a cell that is the only place for two digits. |
| Verdict | What a search concludes of its givens: none, one or many solutions. One is well-posed. |
| Solution | A full grid with no conflict that holds every given. What a solved branch holds. |
| Technique | A named way a person deduces something from a grid: a full unit, a cross-hatch, a naked or hidden single, pointing, claiming, a naked or hidden subset, and the rest of `technique.allium`'s catalogue. |
| Ladder | The order techniques are tried in, cheap and local before dear and global. A technique's rank is its place on it. |
| Deduction | One place a technique holds on a grid: where, with which digits, and what it places or strikes. |
| Placement | A digit a deduction, or a guess, puts in a cell. |
| Strike | Candidates a deduction removes from a cell. Never eliminate. |
| Mark | A candidate as the player has it written. Stale when a placed peer rules it out; lapsed marks are marks left stale. On the board, one digit of a cell's note. |
| Subset | A pair, a triple or a quad, naked or hidden. |
| Cross-hatch | A hidden single read from the placed digits alone, with no marks, in a box or a line. |
| Pointing, claiming | A box whose places for a digit lie on one line, and a line whose places lie in one box. Never box-line reduction. |
| Bivalue, conjugate pair | An open cell with two candidates, and a digit's only two places in a unit. |
| Link, literal, path | Two candidates joined weakly (at most one is true) or strongly (at least one is); a candidate asserted or denied; literals each following from the last by one link. |
| Base, cover | The two sets of lines a fish is read across: the digit's places in the bases lie in the covers, and it is struck from the rest of the covers. |
| Proof, witness | The units, links and paths a deduction rests on, and one particular place a technique holds, with its proof, as against the technique holding somewhere. |
| Uniqueness promise | The premise that the givens have one solution, stated by whoever keeps a grid and never inferred. A rating makes it; a hint over a player's unverified entries does not. |
| Profile | A parameterized player. `technique/Profile` has repertoire, capacity, spans, marking, order, fixation, upkeep, budget, fatigue and patience, and `reach`, `effort` and `lapse` read it. `human-solving/PlayerProfile` separates ability, experience, biases and coping. |
| Projection | `human-solving.allium`'s statement of the `technique/Profile` a `PlayerProfile` in an environment amounts to, so that one player can be put through all four models. |
| Load, capacity | In `technique.allium` load is a coarse per-proof figure and capacity its ceiling. In `human-solving.allium` capacity bounds simultaneously retained chunks, including pending reasoning and unwritten branch data. Neither is a count of Sudoku cells or an intelligence score. |
| Extent, span | How much of the grid a deduction is read across, and an extent a player takes in. |
| Run | One player model put to one set of digits. It looks, steps and looks again. Each model has its own; `human-solving.allium` calls its run an attempt and its step a microstep. `Rate`, `Price`, `Tackle` and `Simulate` ask for one of each. |
| Step | A deduction taken; in `lapse.allium` also a check, a guess or a repair. |
| See, stall | In `reach.allium`: a deduction the profile does not hide is seen, and a run that sees nothing in an unfilled grid stalls. |
| Hook, cascade | In a run: a step that stands above the routine steps around it, and the steps it lets loose. `generation.allium`'s words; what makes a step a hook, and how a cascade is counted, are `reach.allium`'s open questions, and no cutoff is settled. |
| Technique contract | What a generated candidate is held to before it is kept, stated against a profile: a profile plus a ceiling, or a required set and a forbidden set. Which of the two is `generation.allium`'s open question. |
| Tier | In `generation.allium`: one of the basic generator's five construction settings, numbered 1 to 5, each a range for the count of givens, a floor and an order of removal. Never level, which is the research report's word, and never a rating: nothing says a tier's puzzles are harder than another's. |
| Bound, floor | In `generation.allium`: the count of givens a generation will not go below, drawn once from its tier's range; and the fewest givens any row or column may be left with. |
| Order of removal, visit | In `generation.allium`: the order in which the positions of the grid are visited, drawn or one of three fixed orders; and the one look a position gets, which empties it or leaves its given. |
| Grid attempt | In `generation.allium`: one try at drawing a solution grid from eleven drawn givens, which finds one or fails. Attempts are counted against a figure, never timed. |
| Price, escalation | In `effort.allium`: what a step costs this player, and a step taken from beyond their profile. |
| Check, repair | In `lapse.allium`: bringing every mark up to date, taken only while some mark is stale, and returning the grid to how it stood before a wrong guess. |
| Attempt | In `human-solving.allium`: one fresh puzzle, resolved profile, environment, budgets, versions and seed, carried through to a stopping reason and independent judgement. |
| Microstep | One paid action such as attending, inspecting a fact, rehearsing, inferring, writing a note or committing a digit. An unsuccessful action is still recorded. |
| Fact, belief | One proposition acquired or derived by the simulated person. It has provenance and confidence and can be false; a coverage assertion does not contain all the facts it summarizes. |
| Chunk | A bounded group of learned, recognized facts occupying one working-memory slot. Arbitrary bundles do not qualify. |
| Note, entry | In `human-solving.allium`: one cell's written marks together with a claim of coverage, and a digit standing on the sheet, given or written by the player. A mark is still one written candidate. On the board a note is one cell's marks alone, shown while the cell is empty and waiting beneath a digit otherwise. |
| Sheet | Actual entries, external candidate notes and deliberately written branch records, separate from mental beliefs and observer knowledge. |
| Coverage | An assertion that all alternatives of one cell, or all positions for one digit in a unit, were checked. Missing partial notes do not establish coverage. |
| Observer | The independent evaluator that can inspect full traces and solution truth; its judgements do not steer the simulated person. |
| Assessment | Results from a declared seed list under one fixed condition, retaining counts, distributions and every attempt rather than a universal scalar difficulty. |
| Familiar recognition, elementary derivation | A learned pattern shortcut, and explicit reasoning from acquired Sudoku premises. Deriving an unfamiliar pattern does not make it familiar. |
| Fatigue, frustration | Bounded transient states altered by spent effort, fruitless search, perceived progress and rest; independent of fixed experience. `technique/Profile`'s fatigue and patience are coarser figures under the same words: the effort that takes one from capacity, and the fruitless looks before a guess. |

## The repository

| Term | Meaning |
| --- | --- |
| Specification | An `.allium` file under `docs/specs/`. Decides behaviour. |
| Surface | A boundary in a specification: what is exposed, what operations are provided, and what is guaranteed. |
| Guarantee | A named prose assertion on a surface. Acceptance criteria, not aspiration. |
| Randomness boundary | The trait `crates/pawdoku/src/random.rs` defines: a stream of draws in `[0, 1)` begun from a seed, the same for the same seed and `random_version`. The only effect the engine has; a consumer supplies the implementation and a test supplies draws through the fake. |
| Fake | The in-memory implementation of the randomness boundary, used by tests. Not a mock: it behaves, rather than recording calls. Here it is `ReplayStream`, which plays back the draws a test scripts. |
| Gate | A check that can fail the build. Listed in [Quality gates](../reference/quality-gates.md). |
| Recipe | A `Justfile` target. The only supported interface to the checks. |
| Consumer | Whatever calls the engine: the game through WebAssembly, a command line, Python. What a player is shown is a consumer's decision, never a rule here. |
| Workspace | The Cargo workspace at the root. One member today, `crates/pawdoku`; the bindings crates join it as siblings. |

## Related pages

- [Specifications](../explanation/specifications.md)
- [Modelling a human Sudoku solver](../explanation/human-solving.md)
- [Repository map](repository-map.md)
