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
`src/sudoku/` is the rules, `src/solver.rs` with the files under `src/solver/` is the
solver, and `tests/` holds the integration tests. Behaviour arrives
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
The solver needs none: `solver.allium` shows that every search concludes, so solving
takes the givens alone.

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
anywhere but the solver has found a defect. Outside test code there is one call, in
`solver::solve` in `src/solver.rs`. The constructor's own checks bound the damage: it
can be handed a wrong claim of uniqueness and nothing else that is wrong. Outside the
crate the rule is mechanical, because nothing there can make a proof: `WellPosed` is
public and its constructor is not, and a doc example on the type that names the
constructor is there to fail to compile.

A `Puzzle` keeps the proof's solution and never hands it out. It answers one question,
whether one digit is the solution's at one position, yes or no, and that answer is
`pub(crate)`, for the board's check. Its `Debug` leaves the solution out. Solved is the
specification's definition, a full grid with no conflict, and never a comparison with
the stored solution. Whoever holds the proof can read its givens and its solution; what
is hidden is the solution once it is inside a puzzle.

`WellPosed`, `Puzzle` and the errors they return are public, because the solver makes
the proof and outside code can therefore make a puzzle: solve, then set. The tests
inside `sudoku` still build the proof by hand, since `sudoku` imports nothing.

## The solver

`solver` is `solver.allium` in Rust: the search that gives a set of givens its verdict
of none, one or many solutions. There is one solver. It is not a trait and not a
parameter, it takes no budget because every search concludes, and it draws on no
randomness because nothing in a search is left to chance
([decision 0014](../decisions/0014-board-imports-solver.md)). It has two entries.

| Entry | Gives | A caller wants it to |
| --- | --- | --- |
| `solver::search(givens)` | A `SearchResult`: the verdict, the solutions found (none, one or two) and the count of guesses. It never fails; the worst givens get `none`. | Learn about givens: whether they can be solved, whether the answer is open, and how much guessing it took. A generator asks this of every candidate. |
| `solver::solve(givens)` | The proof, `WellPosed`, or a refusal. | Set a puzzle: `Puzzle::set(solve(givens)?)`. |

`solve` is built on `search`. It refuses three ways: the givens have no solution, they
have more than one, or they have one and leave nothing to play. The third is the proof
constructor's own guard and reaches the caller as the constructor's error, wrapped:
eighty-one givens that agree with a valid grid get the verdict `one` from `search`, and
`SetPuzzle` still refuses them.

Both take the givens as anything that yields them, in any order, and read them as a
set, so a given handed over twice is one given.

**A result has no status.** The `SearchResult` surface exposes `search.status`, and the
functions here return only once the search has concluded, so being handed a result is
how the status is read; `solver.allium` says so in the sentence above that surface.

**The solver runs once for each puzzle**, as it is set. A `Puzzle` keeps the solution
the proof carried, so nothing asks the solver again while the puzzle is played.

**The search's shape is the specification's and its storage is not.** The count of
guesses is exposed and must be the same every time, so the code makes exactly the splits
`solver.allium` describes: propagation by the three rules until nothing is left, the
three kinds of contradiction, a split on a cell with the fewest candidates, depth first,
and a stop at the second solution. How a branch is kept is the implementation's: a grid
of candidate sets, each a bit to a digit, and the waiting branches as a stack rather
than recursion ([decision 0013](../decisions/0013-metrics-gate.md)). A branch has no
status field, no parent and no depth; where it is says what it is. The module also fixes
the one order the specification leaves open, and documents it: among tied cells the
first in grid order, and among a split's children the lowest digit first. Counts of
guesses depend on that rule, so changing it changes what consumers may have recorded.

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
