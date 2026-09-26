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
dependency only.

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

Two integration tests exist. `tests/api_bounds.rs` holds, at compile time, that every
public type is `Send + Sync + 'static`, `Clone` and `Debug`, and that the boundary trait
is usable as a trait object. `tests/random.rs` holds the boundary's contract from outside
the crate: the same seed gives the same stream, indexed from zero; the first draws of seed
zero are pinned bit for bit, so a changed generator must change `RANDOM_VERSION`; the fake
replays its script and then reports exhaustion.

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

## Coverage

cargo-llvm-cov runs every nextest test under instrumentation, and `just coverage` fails
when line coverage over `crates/pawdoku/src/**` falls below 90 per cent. The floor is the
`coverage_floor` variable in the `Justfile`. Doctests are not measured, because
llvm-cov's doctest support needs a nightly toolchain, so `test-doc` runs them uncovered
and the floor is a statement about unit and integration tests. While the crate is small
the denominator is too: one untested branch in `random.rs` can breach the floor alone.
The recipe also writes `target/llvm-cov/lcov.info`, which CI keeps as an artifact.

Distinguish an untested branch from an unreachable one. Defensive code no input can reach
should be deleted rather than covered; see
[Quality philosophy](../explanation/quality-philosophy.md).

## What the current suite proves

| Suite | Covers |
| --- | --- |
| In-module tests | `lib.rs`: the grid's side squares to 81 cells. `random.rs`: the stable `Display` text of each error; a scripted draw outside `[0, 1)`, NaN included, rejected; the seeded index wrapping with its state; clones replaying the same draws; a deserialised script held to the same range check as a constructed one; and, by property, every seed drawing in range. |
| Doctests | Every public item's example, among them `SIDE`'s value, `RANDOM_VERSION`'s name, and the trait used as a trait object. |
| `tests/api_bounds.rs` | Every public type is `Send + Sync + 'static`, `Clone` and `Debug`; the boundary trait is usable as a trait object; under the `serde` feature, both streams serialise. |
| `tests/random.rs` | The same seed draws the same stream bit for bit; seed zero draws the golden stream; the fake replays its script and is then exhausted, directly and behind a trait object; every draw is in the unit interval. |

## Related pages

- [Test and debug](../how-to/test-and-debug.md)
- [Quality gates](quality-gates.md)
- [Quality philosophy](../explanation/quality-philosophy.md)
- [Decision 0009](../decisions/0009-rust-quality-gate.md)
