---
title: "Layering and dependency direction"
kind: "explanation"
audience: [contributor, maintainer, agent]
canonical_for: [dependency_boundaries]
requires: []
---

# Layering and dependency direction

The module graph is the specification import graph. Each specification module under
`docs/specs/` names what it imports with a `use` line, and the Rust module that
implements it may use what its specification imports and nothing above it.

| Module | Imports | Imported by |
| --- | --- | --- |
| `sudoku` | nothing | every other specification module |
| `solver` | `sudoku` | `board`, `generation` |
| `technique` | `sudoku` | `reach`, `effort`, `lapse`, `human-solving`, `generation` |
| `reach` | `sudoku`, `technique` | `generation` |
| `effort` | `sudoku`, `technique` | nothing |
| `lapse` | `sudoku`, `technique` | nothing |
| `human-solving` | `sudoku`, `technique` | nothing |
| `board` | `sudoku`, `solver` | nothing |
| `generation` | `sudoku`, `solver`, `technique`, `reach` | nothing |
| `random` | nothing | whichever module draws |

`sudoku` is the rules and sits beneath everything. `solver` and `technique` stand on it
alone. The four models of a player — `reach`, `effort`, `lapse` and `human-solving` —
each stand on `sudoku` and `technique` and never on each other. `board`, the puzzle in
play, imports the rules and the solver, and nothing imports it: a board answers a check
against the puzzle's one solution, and only the solver finds one. The edge is safe
because the board is a leaf, so no cycle can arise through it, and because the claim
under "Why the direction matters" is about the four models, none of which gains an
import ([decision 0014](../decisions/0014-board-imports-solver.md)). `generation`, the
setter that is a program, is the one module that stands on a model: it imports the
rules, the solver, the catalogue and `reach`, because a candidate puzzle is accepted on a
`reach` run, and nothing imports it. It is a skeleton today, and its Rust module arrives
with its triggers. `random` is not a specification
module: it is the randomness boundary in `crates/pawdoku/src/random.rs`, beside the
others rather than beneath them, and it is used by whichever module draws.

## Why the direction matters

The rule is not tidiness. It is what makes the claim below true, and that claim is
load-bearing:

**Every model of a player is testable without the others and without a generator.** A
model that imported another model could only be tested with that one's behaviour in the
way, and a module that reached for a source of randomness of its own could only be
tested against whatever that source produced. Because the four models share only the
rules and the technique catalogue, and because the only thing that reaches outside the
engine is the boundary, a test of any one of them supplies the grid, the profile and the
draws it chose, and the real code path still runs.

## Where a side effect goes

The engine has one effect, randomness, and it has its boundary: the `RandomStream` trait,
the generator the library ships and the fake a test drives. A second effect is not
expected; the core is `no_std`, so a clock, the filesystem and the environment cannot be
named in it at all. If one were ever needed, it would get a boundary of the same shape —
a trait in the engine's vocabulary, the implementation the library ships, and a fake —
and the decision would be recorded first.

The rule that follows: **tests inject fakes through the boundary, they never stub
globals.** A stubbed global leaks between tests and hides the fact that the code reached
outside its layer.

## Enforcement

rustc will not enforce the table: it allows a cycle between modules of the same crate.
Gate 7, `just metrics`, does. `rustqual.toml` at the repository root holds one
`[[architecture.pattern]]` rule per row of the table above, ten in all. Each lists every
`crate::` path its module may not name, and its `reason` is the row. `random`'s rule
forbids all nine others, and `random` is in no other rule's list, because anything may
import it. A rule covers the file named after its module and everything under the
directory of the same name, so `src/technique.rs` and `src/technique/` are one module and
a child is held to its parent's row. A rule for a module the crate does not have yet
matches nothing, so the guardrail is in place before the code is.

Each rule catches both ways of naming a module: a `use crate::effort::Price;` line, and an
inline `crate::effort::price()` in a function body. rustqual reports nothing, and the gate
would pass, when its architecture section is switched off or misconfigured, so the recipe
also runs a probe: a fixture that breaks every rule once, and the gate fails unless
rustqual names each of the ten exactly once. [Quality gates](../reference/quality-gates.md) describes it.

Three paths are out of the rules' sight. A path written relative to the module, such as
`super::super::solver` from `src/technique/catalogue.rs`, names no `crate::` prefix. An
alias of the crate, `use crate as root;` and then `root::solver::item()`, replaces the
prefix the rules match; rustqual reports nothing for it. And a re-export in `lib.rs` lets
any module write `crate::Price` for `crate::effort::Price`. The third is guarded from the
side: `unreachable_pub` and `unnameable_types` in the workspace table keep what is `pub`
deliberate, so a module's surface is the one it meant to export, and `lib.rs` is the one
file to read for re-exports in review.
`mod_module_files` bans `mod.rs`, so every module is the file named after it and the
graph above can be read from the file tree. What the rules cannot see is left to review,
to the `rust-change` and `code-review` skills, and to the shape of the tests: code in the
wrong layer is usually code that is hard to test.

## Related pages

- [Architecture](architecture.md)
- [Testing](../reference/testing.md)
- [Decision 0003](../decisions/0003-effects-behind-traits.md)
- [Decision 0013](../decisions/0013-metrics-gate.md)
- [Decision 0014](../decisions/0014-board-imports-solver.md)
