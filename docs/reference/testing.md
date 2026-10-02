---
title: "Testing"
kind: "reference"
audience: [contributor, maintainer, agent]
canonical_for: [testing_reference]
requires: []
---

# Testing

## Framework

cargo-nextest runs the unit and integration tests. It cannot run doctests, so `just test`
runs nextest and then `cargo test --doc`, and `just test-doc` runs the doctests alone.
`.config/nextest.toml` holds the profiles: the default one stops at the first failure, and
a `ci` profile, which runs everything and writes a JUnit report, is there for a job that
wants one; no recipe selects it today. Property tests use proptest, a development
dependency only. File snapshots use insta, also a development dependency only, with
default features disabled and `json` enabled for record snapshots.

## Layout

Tests live in three places, and each is deliberate.

| Where | What |
| --- | --- |
| `#[cfg(test)] mod tests` at the foot of a module | Unit tests of that module, with access to its private items. |
| `crates/pawdoku/tests/*.rs` | Integration tests: the crate as a consumer sees it, through its public API only. |
| `///` examples on public items | Doctests: each example compiles and runs as a test. |

Unit tests live in the module they test. The crate root carries
`#[cfg(test)] extern crate std;`, so a test may use the standard library while the crate
itself stays `no_std`. This departs from the games' rule that tests are never colocated
with the code: in Rust a unit test can reach a private item only from inside its module,
and [Decision 0009](../decisions/0009-rust-quality-gate.md) records the deviation.

Five integration test files exist. `tests/api_bounds.rs` holds, at compile time, that every
public type is `Send + Sync + 'static`, `Clone` and `Debug`, and that the boundary trait
is usable as a trait object. `tests/random.rs` holds the boundary's contract from outside
the crate: the same seed gives the same stream, indexed from zero; the first draws of seed
zero are pinned bit for bit, so a changed generator must change `RANDOM_VERSION`; the fake
replays its script and then reports exhaustion. `tests/sudoku.rs` holds the rules from
outside the crate: the two figures, the value types, and a puzzle made the way outside
code makes one, solved and then set, and played to its end; each rule is proved clause
by clause by the unit tests inside `src/sudoku/`. `tests/solver.rs` holds the solver's
two entries, and is where most of the solver's clauses are proved, because what
`solver.allium` promises is what a result shows. `tests/snapshots.rs` holds every
snapshot test taken through the public API, and no test that asserts.

A helper shared between two files under `tests/` would be compiled into each, and each
would report the functions it does not call as dead code. So each file carries the few
helpers and fixtures it uses, and the snapshot tests share one file.

When a property test fails, proptest writes the failing case to a `proptest-regressions/`
file beside the test that produced it. That file is committed, so the case is replayed
first on every later run.

## Conventions

**Derive a test from the clause it proves, and name it after the clause.** A test named
`the_same_seed_draws_the_same_stream` says which guarantee broke when it fails; a test
named after the function it calls does not.

**A doc example is a test.** It runs under `just test-doc`, so an example that stops
being true stops the gate. Every public item with a contract worth stating carries one.

**Supply draws through the fake; never through a generator.** A test that needs
randomness scripts its draws with `ReplayStream`, the fake beside the `RandomStream`
trait, and gets exactly the draws it chose. A test of the generator itself is the one
place `SeededStream` is driven directly.

**Restriction lints relax inside tests only.** `clippy.toml` allows `unwrap`, `expect`,
`panic`, indexing, `dbg!` and printing inside `#[cfg(test)]` code, and nowhere else, so a
test can fail loudly while the library cannot.

## The rules' fixture

The tests of `sudoku` share one puzzle, in `src/sudoku/fixture.rs`: the example of the
article `sudoku.allium` cites as its source. It has 30 givens and one solution, and
naked and hidden singles alone solve it. Rows run top to bottom, a dot for an empty
cell:

```text
puzzle       solution
53..7....    534678912
6..195...    672195348
.98....6.    198342567
8...6...3    859761423
4..8.3..1    426853791
7...2...6    713924856
.6....28.    961537284
...419..5    287419635
....8..79    345286179
```

**These tests build the proof by hand.** A `Puzzle` is set from a `WellPosed`, and only
the solver makes one; `sudoku` imports nothing, so its tests cannot call the solver. They
pass the givens and the solution above, checked by hand, to the crate-only constructor
([Architecture](../explanation/architecture.md) says why the proof exists). The same
constructor lets a test state a proof no solver would make, such as no givens with one
solution grid, which is how "solved is not a comparison with the stored solution" is
shown: the puzzle is filled with a different valid grid and is solved.

The invariants and the surface are held by property: a script of placements and
erasures, refused ones included, runs on the fixture from some way towards its solution,
so that some scripts solve it and go on. The oracle for a refusal is a model written
from the rules' `requires` clauses in test code, never the implementation.

## The solver's tests

`tests/solver.rs` drives `search` and `solve` through the public API. The unit tests in
`src/solver/branch.rs` look inside one branch, where a result cannot: each propagation
rule alone, each kind of contradiction alone, and the cell a split is made on.

**The fixtures**, each with where it came from stated beside it in the file:

| Givens | What they are for |
| --- | --- |
| The rules' fixture | Singles alone solve it: verdict `one`, no guess. |
| Norvig's `grid2`, from the essay `solver.allium` cites, with the solution the essay prints | Seventeen givens and one solution that singles cannot reach: a puzzle that needs a guess. |
| The rules' fixture without its given at row 3, column 8 | Exactly two solutions. |
| The fixture's solution with four cells emptied | Exactly two solutions, found in one split. |
| Thirty-four cells of the fixture's solution | No empty cell has one candidate to begin with, so the first placement is a hidden single's. |
| Three small sets, one for each kind of contradiction | No two of their givens conflict, and each leaves a cell with no candidate, a unit with no place for a digit, or a cell that is the only place for two digits. |

**The oracle.** `VerdictIsTrue` ties the verdict to `sudoku.allium`'s `solution_count`,
so the test needs a count the solver did not produce. `count_solutions` in
`tests/solver.rs` is one: it tries digits cell by cell in grid order with no candidates
and no propagation, stops at two, and keeps its place in a list where another would
recurse. A property compares it with `search`. Its inputs are chosen to keep it quick:
cells of the fixture's solution with at most some four in ten removed, which have one
solution or many; such a set, denser still, with one digit changed, which mostly has
none; and arbitrary givens with one added that is certainly malformed, which the oracle
counts as none without a search. A sparse set that is well-formed and has no solution
can take a plain counter a very long time: five scattered givens that leave a row no
place for a digit are enough. No strategy the oracle is handed can produce one. The
properties that call the solver alone take arbitrary givens with no such limit.

**The pinned guess counts follow from the tie-break, not from the specification.**
`solver.allium` leaves the order of tied cells and tied branches to the implementation.
The module documents its rule: the first cell in grid order, and the lowest digit first.
`the_guess_counts_follow_from_the_tie_break` pins what that rule gives: 0 for the
fixture, 50 for Norvig's puzzle, 1 for the two-solution givens and 47 for the empty
grid. Another rule would be the same solver with other numbers, so a change to these is
a change to the tie-break and is to be read as one. The same holds for which two
solutions are found for givens with many, and in which order.

**Properties.** Over givens of every kind, malformed ones included: every search
returns; the same givens get the same result on a second call, reversed and shuffled;
each solution is full, free of conflict and holds every given; two solutions differ;
there are never more than two; the verdict matches their number; and `solve` gives a
proof exactly when the verdict is `one`, eighty-one givens apart. Inside the crate, a
property walks whole searches a branch at a time and holds the three invariants the
implementation has the structure for.

## Coverage

cargo-llvm-cov runs every nextest test under instrumentation, and `just coverage` fails
when line coverage over `crates/pawdoku/src/**` falls below 90 per cent. The floor is the
`coverage_floor` variable in the `Justfile`. Doctests are not measured, because
llvm-cov's doctest support needs a nightly toolchain, so `test-doc` runs them uncovered
and the floor is a statement about unit and integration tests. While the crate is small
the denominator is too: one untested branch in `random.rs` can breach the floor alone.
The recipe also writes `target/llvm-cov/lcov.info`, which CI keeps as an artifact.

Coverage counts a snapshot test like any other test. That count proves no specification
clause: every clause needs its own asserting test, even when a snapshot runs the same code.

Distinguish an untested branch from an unreachable one. Defensive code no input can reach
should be deleted rather than covered; see
[Quality philosophy](../explanation/quality-philosophy.md).

## Snapshots

A snapshot has two purposes and no other:

1. **Show a reviewer what a value looks like.** It is print-based debugging kept in the
   repository: a reviewer sees the grid, moves or result without running anything.
2. **Cheap change detection.** A change to what an interface hands out appears as a diff
   in a committed file, as a component snapshot shows a changed front end.

**A snapshot never verifies correctness.** The first snapshot records what the code
printed that day; it was not checked against the specification. No clause is proved by
a snapshot. Each clause keeps its own test that asserts, and a ticket's table from
clause to test never names a snapshot test.

Write snapshot tests only after the behaviour's own tests are green, never as the failing
test of a red-green loop. Their names begin `snapshot_`, so readers and filters can
distinguish them. Use a fixed input and a fixed script, never a property-test input.

Render a text picture in test code from what the public surface hands out. Add no
`Display`, method or field to the library to feed a snapshot. A small value, such as an
error or record, may use its `Debug` or serialised form; records use
`insta::assert_json_snapshot!`. Every picture has no trailing whitespace and ends in a
newline.

Snapshots are `.snap` files, never inline in Rust source. They appear as their own diffs,
and acceptance rewrites no source. A test in `src/foo.rs` stores files in
`src/snapshots/`; a test in `src/foo/bar.rs` stores them in `src/foo/snapshots/`; a test
in `tests/foo.rs` stores them in `tests/snapshots/`, beside the source file's directory.
These files are committed. Pending `*.snap.new` and `*.pending-snap` files from runs
outside the recipes are ignored.

`just test`, `just coverage` and `just snapshots-check` fail on a mismatch without
writing a snapshot or pending file. The snapshot gate also rejects a `.snap` file no
test refers to. It runs the entire suite through cargo-insta with nextest and every
feature, so every reference is seen. cargo-insta cannot pass `--locked` to nextest;
the snapshot recipes validate both locks first and run cargo-insta offline.

A changed snapshot asks whether the change was intended. The person who changed it reads
the diff, reviews pending snapshots through `just snapshots-review` at a terminal, and
accepts intended changes through `just snapshots-accept`, which reruns the tests,
accepts without prompting and deletes any `.snap` file no test refers to. Give the reason
in the pull request or hand-back notes.
The [snapshot workflow](../how-to/test-and-debug.md#read-and-accept-a-changed-snapshot)
explains the commands in order.

## What the current suite proves

| Suite | Covers |
| --- | --- |
| In-module tests, `random.rs` | The stable `Display` text of each error; a scripted draw outside `[0, 1)`, NaN included, rejected; the seeded index wrapping with its state; clones replaying the same draws; a deserialised script held to the same range check as a constructed one; and, by property, every seed drawing in range. |
| In-module tests, `sudoku.rs` | `box_side` is 3 and `side` its square; the value types equal when their fields are; two givens that agree are one given; every cell has twenty peers and is not among them. |
| In-module tests, `sudoku/proof.rs` | `SetPuzzle` through the proof: refused for eighty-one givens, for a solution that is not full, has a digit out of range or has two peers holding one digit, for a given off the grid, and for a given that is not the solution's, two disagreeing givens included; an accepted proof gives back its givens and its solution; the stable `Display` text of each error. |
| In-module tests, `sudoku/puzzle.rs` | `LayOutGrid`, `PlaceDigit`, `EraseDigit` and `PuzzleSolved`, each `requires` clause refused by name; solved at once, final, and not a comparison with the stored solution; `is_full`, `is_consistent` and conflicts; the yes-or-no answer, total over any position; a `Debug` that leaves the solution out; the stable `Display` text of each error; and, by property over scripts of moves, the eight invariants, the givens never changing, and a move accepted exactly when its rule allows it. |
| `sudoku/puzzle.rs`: four `snapshot_` tests | Show the fixture as set, the fixture after each of four scripted moves, `Puzzle`'s `Debug`, and the `Display` text of every error the module defines, for review and change detection; they prove no clause. Their files are under `src/sudoku/snapshots/`. |
| In-module tests, `solver.rs` | Two solutions are sought; the refusal's one conversion from the proof's error, called directly with every variant; the stable `Display` text of each refusal. |
| In-module tests, `solver/candidates.rs` | A set of digits holds only digits from 1 to 9, whatever is offered; sets join, part and overlap; a single is a set of exactly one. |
| In-module tests, `solver/branch.rs` | The units and each cell's twenty peers; `LayOutRootBranch`, and no root for malformed givens; `EliminateFromPeers`, `PlaceNakedSingle` and `PlaceHiddenSingle`, each alone, the last in a row, a column and a box; each kind of contradiction read from the cells; an overdemanded cell not placed; the split on the first cell with the fewest candidates, one child for each candidate, lowest digit first; and, by property over whole searches, `PlacedCellsKeepOnlyTheirDigit`, `CandidatesAreDigits` and `GuessesAreOnFewestCandidates`. |
| Doctests | Every public item's example, among them `SIDE`'s and `BOX_SIDE`'s values, the value types' equality, `RANDOM_VERSION`'s name, the trait used as a trait object, a puzzle solved, set and played, and the one `compile_fail` example, which shows outside code cannot name the proof's constructor. |
| `tests/api_bounds.rs` | Every public type is `Send + Sync + 'static`, `Clone` and `Debug`, the proof, the puzzle and the solver's result among them; the boundary trait is usable as a trait object; under the `serde` feature, both streams and both value types serialise. |
| `tests/sudoku.rs` | From outside the crate: the side is the square of the box side; the value types compare by their fields; a position off the grid and a given out of range can be made; two givens that agree are one given; a puzzle solved and set from outside is played to its end and is then final; each move is offered and refused; everything `PuzzleSolving` exposes can be read; a puzzle does not print its solution. |
| `tests/solver.rs` | `RefuseMalformedGivens` for each way givens are malformed; anything may be handed over; singles alone solve the fixture; hidden singles solve a puzzle with no naked single; conflicts and each kind of contradiction end the search with no guess; a published puzzle that needs a guess; the pinned guess counts; `none`, `one` and `many`, the last with both solutions; the empty grid stopping at two; `solve`'s proof and its three refusals; the oracle; and the properties above. |
| `tests/snapshots.rs`: six `snapshot_` tests | Show the search result for the fixture, for the puzzle that needs a guess, for the givens with two solutions and for malformed givens; the proof `solve` gives for the fixture, givens and solution; and the `Display` text of the three refusals. For review and change detection; they prove no clause. Their files are under `tests/snapshots/`. |
| `tests/random.rs` | The same seed draws the same stream bit for bit; seed zero draws the golden stream; the fake replays its script and is then exhausted, directly and behind a trait object; every draw is in the unit interval. |
| `tests/random.rs`: `snapshot_the_randomness_boundary` | Shows seed zero's first eight draws and each error's `Display` text for review and change detection; proves no clause. Its file is under `tests/snapshots/`. |

## Related pages

- [Test and debug](../how-to/test-and-debug.md)
- [Quality gates](quality-gates.md)
- [Quality philosophy](../explanation/quality-philosophy.md)
- [Decision 0009](../decisions/0009-rust-quality-gate.md)
