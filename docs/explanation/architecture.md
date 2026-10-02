---
title: "Architecture"
kind: "explanation"
audience: [contributor, maintainer, operator, agent]
canonical_for: [system_architecture]
requires: []
---

# Architecture

This library is a library and nothing else. It has no process of its own: no `main`, no
I/O, no clock, and nothing it can do until a consumer calls it. The game consumes it
through WebAssembly later; a command-line tool and Python bindings consume it later
still, as sibling crates in the same workspace.

That constraint is not a limitation waiting for a runtime to be added — it is the
architecture. See [Decision 0001](../decisions/0001-engine-as-a-library.md) for why the
engine is a library of its own, and
[Decision 0008](../decisions/0008-no-std-core.md) for why its core is `no_std`.

## Workspace

The root `Cargo.toml` is a virtual manifest: it names the members, the package fields
every crate inherits, the dependency table every crate draws from, and the
`[workspace.lints]` table every crate opts into. `Cargo.lock` beside it is the pin.

`crates/pawdoku` is the engine and today the only member. `src/lib.rs` is the crate
root, `src/random.rs` is the randomness boundary, `src/sudoku.rs` with the files under
`src/sudoku/` is the rules, and `tests/` holds the integration tests. Behaviour arrives
as one Rust module per specification module, and a module may use only what its
specification imports, in the direction
[Layering and dependency direction](layering.md) describes.

Three siblings join when ticket S04 opens them: `crates/pawdoku-cli`,
`crates/pawdoku-py` (pyo3, built with maturin) and `crates/pawdoku-wasm`
(wasm-bindgen). None exists yet. Every decision below is taken so that each stays a thin
wrapper when it arrives.

## The core is `no_std`

`crates/pawdoku` is `#![no_std]` with `extern crate alloc`. A clock, threads, the
filesystem, the environment and `HashMap` are therefore not avoided by convention but
unnameable: the standard library that would name them is not in scope. The proof is
mechanical rather than a promise. `just wasm-check` checks the crate against
`wasm32v1-none`, a target that has no standard library at all, so a single `std` path
anywhere in the core fails the gate.

With no clock there can be no timeout. Where a computation needs a limit, the limit is a
step budget the caller sets, and the same budget gives the same answer on every machine.

## The randomness boundary

Randomness is the one effect the engine has, and `random.rs` is its boundary. The trait
`RandomStream` is a stream of draws in `[0, 1)`, begun from a seed, indexed from zero,
the same for the same seed and `random_version`. The caller chooses the implementation.
The library ships one generator, `SeededStream`, named by the constant `RANDOM_VERSION`,
so that exact replay holds across callers: a consumer that records the name beside the
seed can replay the same draws in a browser, in Python or on the command line. A test
supplies its draws through the fake, `ReplayStream`, which replays a script and reports
exhaustion rather than panicking.

The core never sources entropy. `deny.toml` bans `rand` and `getrandom` from everything
the workspace ships, and names `pawdoku-cli` as the only crate permitted to wrap them;
the command line is the one place a seed will come from the operating system.

## How well-posedness reaches the rules

`SetPuzzle` in `sudoku.allium` admits only givens that leave something to play and have
exactly one solution. The `sudoku` module cannot decide the second: counting solutions
is the solver's work, and `sudoku` imports nothing. So the answer is carried to the
rules as a value ([decision 0014](../decisions/0014-board-imports-solver.md)).

`sudoku` defines the proof, `WellPosed`: a set of givens and their one solution. A
`Puzzle` is set from a proof and from nothing else, and setting cannot fail, so no
puzzle exists whose givens `SetPuzzle` would refuse.

The proof has one constructor, `WellPosed::vouch`. It returns a `Result` and never
panics, and it checks everything it can see: that there are fewer givens than cells;
that the solution is full, every digit from 1 to 9, and no two peers hold one digit;
and that every given is on the grid and holds the solution's digit at its position.
That last check is also what refuses two givens that disagree about one position.
Uniqueness is the one thing it takes on its caller's word.

**"Only the solver calls the constructor" is a rule review holds, not the compiler.**
The constructor is `pub(crate)`, and Rust cannot restrict visibility to one sibling
module, so any module in the crate could name it. A reviewer who sees `vouch` called
anywhere but the solver has found a defect. The constructor's own checks bound the
damage: it can be handed a wrong claim of uniqueness and nothing else that is wrong.
Outside the crate the rule is mechanical, because nothing there can make a proof.

A `Puzzle` keeps the proof's solution and never hands it out. It answers one question,
whether one digit is the solution's at one position, yes or no, and that answer is
`pub(crate)`, for the board's check. Its `Debug` leaves the solution out. Solved is the
specification's definition, a full grid with no conflict, and never a comparison with
the stored solution. Whoever holds the proof can read its givens and its solution; what
is hidden is the solution once it is inside a puzzle.

Until the solver exists nothing outside the crate could make a proof, so `WellPosed`,
`Puzzle` and the errors only they return are crate-only today, and the module's own
tests build the proof by hand. The solver makes them public.

## What crosses each boundary

| Consumer | Crosses | Through |
| --- | --- | --- |
| WebAssembly | Values and the seed | wasm-bindgen |
| Python | Values and the seed; errors as exceptions | pyo3 |
| Command line | Values, and the one source of entropy | `pawdoku-cli` |

Python and WebAssembly build their exceptions from each error's `Display` text, which is
why that text is stable API. It is also why every public type is `Send + Sync +
'static`, `Clone`, `Debug` and `#[non_exhaustive]`: a binding may move a value between
threads, copy it across the boundary or print it, and a new field or variant must not
break a consumer that matched on the old ones. `AGENTS.md` states this as invariant 3,
and `tests/api_bounds.rs` holds it at compile time.

## What is not here

No I/O, no clock, no threads, no `rand`, no network, no persistence and no user
interface. Those are excluded by design, as
[Purpose and scope](../project/purpose-and-scope.md) records. Puzzle generation is not
among them: generation is a planned module, `generation.allium`, specified before it is
built and today a skeleton of scope, config and open questions; whether puzzles are made
on the device, ahead of time, or both is open
([decision 0012](../decisions/0012-generation-and-dev-time-judges.md)).

## Related pages

- [Layering and dependency direction](layering.md)
- [Security model](security-model.md)
- [Repository map](../project/repository-map.md)
- [API reference](../reference/api.md)
