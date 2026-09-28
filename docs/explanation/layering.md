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
| `solver` | `sudoku` | `generation` |
| `technique` | `sudoku` | `reach`, `effort`, `lapse`, `human-solving`, `generation` |
| `reach` | `sudoku`, `technique` | `generation` |
| `effort` | `sudoku`, `technique` | nothing |
| `lapse` | `sudoku`, `technique` | nothing |
| `human-solving` | `sudoku`, `technique` | nothing |
| `board` | `sudoku` | nothing |
| `generation` | `sudoku`, `solver`, `technique`, `reach` | nothing |
| `random` | nothing | whichever module draws |

`sudoku` is the rules and sits beneath everything. `solver` and `technique` stand on it
alone. The four models of a player — `reach`, `effort`, `lapse` and `human-solving` —
each stand on `sudoku` and `technique` and never on each other. `board`, the puzzle in
play, imports the rules alone, and nothing imports it. `generation`, the setter that is
a program, is the one module that stands on a model: it imports the rules, the solver,
the catalogue and `reach`, because a candidate puzzle is accepted on a `reach` run, and
nothing imports it. It is a skeleton today, and its Rust module arrives with its
triggers. `random` is not a specification
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

There is no import-boundary checker here, and rustc will not supply one: it allows a
cycle between modules of the same crate. The direction is enforced by review, by the
`rust-change` and `code-review` skills, and by the shape of the tests: code in the wrong
layer is usually code that is hard to test. Two lints in the workspace table help from
the side. `unreachable_pub` and `unnameable_types` keep what is `pub` deliberate, so a
module's surface is the one it meant to export, and `mod_module_files` bans `mod.rs`, so
every module is the file named after it and the graph above can be read from the file
tree.

## Related pages

- [Architecture](architecture.md)
- [Testing](../reference/testing.md)
- [Decision 0003](../decisions/0003-effects-behind-traits.md)
