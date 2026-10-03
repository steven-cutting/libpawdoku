---
title: "Decision 0015: A basic generator from a published method, as the yardstick"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_basic_generator]
requires: []
---

# Decision 0015: A basic generator from a published method, as the yardstick

## Context

The engine can judge givens, hold a puzzle in play and draw through the randomness
boundary. It cannot make a puzzle. `docs/specs/generation.allium` was written for the
designed generator [decision 0012](0012-generation-and-dev-time-judges.md) made the
engine's: givens removed one orbit of a symmetry scheme at a time, a candidate kept only
when the solver's verdict is one and a `reach.allium` run ends solved within a technique
contract. That generator waits on two modules in Rust, `technique` and `reach`, and on
three open spikes, S06 to S08.

On 2026-10-02 the maintainer asked for something smaller and sooner: "a minimal but
sufficient classic random Sudoku puzzle generator. It doesn't need to be perfect in terms
of the quality of puzzles it produces. It just needs to be incredibly average [...]
compared to other basic random puzzle generators. The purpose of this is to have
something quick that just works well enough for testing and also something that can
provide a baseline that we can compare against for our later more complex generators."

The method chosen is a published one: "Sudoku Puzzles Generating: from Easy to Evil", a
paper headed "Team # 3485" that names no author. It makes a full valid grid, then takes
digits away one cell at a time, checking after each that the puzzle still has one
solution. Five settings each fix a range for the count of givens, a lower bound for the
givens in any row or column and an order in which cells are tried.

## Decision

Four decisions, taken by the maintainer on 2026-10-02:

1. **The basic generator's home is `generation.allium`.** It is a second way of finding
   givens beside the designed way, not a layer under it. It removes one position at a
   time and accepts on the solver's verdict alone: no orbit, no technique contract and
   no `Rate` run.
2. **Its settings are the paper's five, with the paper's figures**, "because this
   generator is supposed to be different and very standard. So that it will make for a
   better baseline to compare against."
3. **It is public API**, specified before it is built.
4. **Two tickets:** T31 for the specification, T32 for the Rust.

And what the paper left open, settled by the maintainer on 2026-10-03:

- The paper's five are **tiers**, numbered 1 to 5. "Level" stays the research report's
  word and "band" a row of three boxes.
- "More than" in the paper's two restrictions reads **at least**, as the paper's own
  example does.
- Tier 1, "more than 50" in the paper, is **51 to 60**; no tier's range holds 50.
- The every-other-cell order's second pass, which the paper's figure does not show, takes
  **the skipped positions along the same path**, and the module says the reading is its
  own.
- Each of the eleven starting givens takes two draws: **a position among the empty
  cells in row order, then a digit among those its row, column and box allow, lowest
  first**. A position with no digit left ends the attempt.
- When the solver finds two solutions of the eleven givens, the grid is **the one that
  comes first reading row by row**: the lower digit at the first position where they
  differ. `solver.allium` gives its solved branches no order and is not changed.
- A draw chooses among `n` by **the binary64 product of the draw and `n`, truncated
  toward zero**. This is not the whole part of the exact product, and the module gives a
  worked draw for which the two differ.
- **Replay is stated over the draws taken**, so it holds for a stream already drawn
  from; a seed stands for the draws exactly when the stream begins at index zero.
- Removal that ends above the tier's range **returns what it reached** and retries
  nothing, as the paper's flow does.
- The basic way reads **one limit, a count of grid attempts, 100 by default** in the
  module's config. No measurement stands behind the figure. `step_budget` and
  `candidate_limit` stay the designed way's.
- Spent grid attempts are **a refusal with nothing to show**.

## Consequences

One module now states two ways of finding givens. Every passage of `generation.allium`
has to say which way it speaks of, and a reader who wants one must read past the other.
The designed way's seven open questions all stand; four carry a clause giving the basic
way's own answer.

The specifications now carry settings built on a count of givens beside the sentence
that a count of givens is never a rating. That is not a contradiction, because a tier is
a construction setting: it says how a puzzle is made and nothing of how it solves. The
module excludes any claim that one tier's puzzles are harder than another's, and whether
they differ is exactly what a later comparison through `reach.allium` measures. But the
paper's names for its five invite the reading, which is why the module quotes them once
and never uses them.

The same seed gives the same puzzle only for one implementation of the solver. Eleven
givens nearly always have many solutions, and which two the search reaches depends on
the order it takes tied cells in and on what its propagation strikes before it guesses,
both of which `solver.allium` leaves to the implementation. A solver that propagates
more or breaks ties otherwise is a new `generation_version`, and a seed carried across
it promises nothing.

Grids come from eleven givens and the solver's first solutions, so they are not spread
evenly over all valid grids, as the paper's are not. Tiers 3 to 5 leave their givens
where a fixed order puts them: their only draws are the grid and the bound, and the
shape of what is left shows the order.

A result can hold more givens than its tier's range, and for the drawn order at the
sparse tiers the paper found it usually does. A caller that wants a count in range reads
the count and asks again; the generator does not.

The draw-to-index arithmetic differs in stance from `human-solving.allium`, whose
`ExactReplay` asks for exact arithmetic in its comparisons. Two modules that will share
one stream now read it by two rules, until that module's builder chooses.

The limit of 100 grid attempts is a guess. T32 measures how many attempts seeds need,
and the figure may move, which is a new `generation_version` only if it changes what a
seed gives.

## What would reopen this

A comparison that shows the tiers do not differ as rated by `reach.allium`, which would
leave five settings where one would do. A consumer that needs symmetry from the basic
way. A second published method worth keeping beside this one, which would ask whether
`generation.allium` is still one module. Or `human-solving.allium`'s Rust settling on
exact arithmetic for a draw's index, which would ask whether this module should follow.

## Related pages

- [Puzzle design objectives](../explanation/puzzle-design.md)
- [Layering](../explanation/layering.md)
- [Specifications](../explanation/specifications.md)
- [Decision 0012: Generation is the engine's; judges are development-time only](0012-generation-and-dev-time-judges.md)
