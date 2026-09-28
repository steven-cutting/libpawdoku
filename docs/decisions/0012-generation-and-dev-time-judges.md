---
title: "Decision 0012: Generation is the engine's; judges are development-time only"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_generation_and_judges]
requires: []
---

# Decision 0012: Generation is the engine's; judges are development-time only

## Context

On 2026-09-27 the maintainer received a research report on what makes a Sudoku puzzle
worth solving, carried cleaned to the house rules in
[Puzzle design objectives](../explanation/puzzle-design.md). Its claim is that a good
puzzle is a designed sequence of deductions: the givens are the interface and the solve
path is the authored object. Some of what it asks for the engine can measure, because
the four player models already expose every step of a run. Some of it, whether a puzzle
is elegant, novel or looks deliberate, is a judgement that no rule here can make.

Two questions followed. Does this engine generate puzzles at all? Until then
[Purpose and scope](../project/purpose-and-scope.md) said generation was a later module
and [Architecture](../explanation/architecture.md) said it was out of scope, and the two
disagreed. And may a judgement model help? Hosted classifiers that return typed answers
with calibrated probabilities exist, and one, Jev, was the case that prompted the
question. The maintainer's instruction, on the same day: "Jev and other such things that
require requests to third party servers/api's can not be part of the app/runtime. Only
dev time, so if we were to use Jev to help generate puzzles we would need to pre generate
them or use Jev to help test / judge their output such so that we can tune their
parameters."

The core is `no_std` ([decision 0008](0008-no-std-core.md)), so a network is not
avoided but unnameable, and `AGENTS.md` invariant 2 makes randomness the engine's only
effect, reached through the boundary [decision 0003](0003-effects-behind-traits.md)
describes. A judge that calls out would be a second effect and a second boundary, and
one that cannot run under `wasm32v1-none`.

## Decision

Generation is the engine's. `docs/specs/generation.allium` is its module, specified
before it is built as [decision 0002](0002-specs-are-the-source-of-truth.md) requires,
and today a skeleton: scope, config and open questions, no triggers. It imports the
rules, the solver, the technique catalogue and the reach model, draws through the
randomness boundary and nothing else, and runs under a step budget so that it can run
anywhere. Where puzzles are made, ahead of time into a set the game carries, on the
device that poses them, or both, is not decided here; the module records it as an open
question and the maintainer owns the answer.

Judgement models, classifiers and any tool that needs a network are development-time
only. They consume the engine's serialised outputs, the runs and assessments the player
models expose and the givens themselves, and their verdicts tune what the generator is
asked for or select a curated set. None enters the runtime, the crate or a
specification: no module names a judge, no crate depends on one, and no rule waits on
one. Jev is named here as the case that prompted the rule and nowhere else in the
repository.

## Consequences

What the engine owes a judge is stable, serialisable outputs to judge. `RunResult`,
`PricedRun`, `TackledRun` and `AssessmentSummary` are already values a consumer reads;
a judge reads them off the command line or the Python bindings, so their shape becomes
API that a pipeline depends on, and a change to it is a change the pipeline must take.

A curated set needs a pipeline outside this crate: something that generates, rates,
serialises, asks a judge and keeps what passes. The Python crate that ticket S04
opens is the likely home. A generator on the device cannot be helped by a judge at all,
so whatever a puzzle made there is held to must be measurable by the engine alone: the
technique contract, the pacing figures once `reach.allium` can report them, and
symmetry. The judgement dimensions the report names are enforced only on a curated set.

The two scope pages now say the same thing: generation is a planned module, and where
puzzles are made is open. The skeleton adds a ninth module to every count.

## What would reopen this

A judge that runs offline, on the device, under the same invariants: no network, no
clock, no entropy of its own, compiling for `wasm32v1-none`. That would be a library
decision about a second kind of computation, not a reversal of this one. A decision
that puzzles are made only ahead of time, which would let the generator itself lean on a
judge and make the step budget a pipeline concern rather than a runtime one. Or the
specification of `generation.allium` finding that the technique contract cannot be
checked by `Rate` alone, which would reopen how much of acceptance is the engine's.

## Related pages

- [Puzzle design objectives](../explanation/puzzle-design.md)
- [Decision 0003: Effects behind traits](0003-effects-behind-traits.md)
- [Decision 0008: A pure no_std core](0008-no-std-core.md)
- [Purpose and scope](../project/purpose-and-scope.md)
