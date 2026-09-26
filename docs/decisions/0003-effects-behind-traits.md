---
title: "Decision 0003: Effects behind traits"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_ports_and_fakes]
requires: []
---

# Decision 0003: Effects behind traits

*Carried from Pawdoku's decision 0002 at `78d03cdf`, and restated for the library the game's engine moved to. Pawdoku's own record stands where it is. The boundaries shrank to one.*

## Context

Pawdoku isolates its side effects, device storage, randomness and the clock, with two more
arriving from the platform package, behind ports in the application's vocabulary, each with
a real adapter and an in-memory fake, so that time and randomness become ordinary values a
test controls. It pays for that with a rule enforced by review because no tool checks it:
nothing may reach for a global.

The engine has less to isolate. Its core is `#![no_std]`, so a clock, threads, the
filesystem and the environment are not merely avoided but unnameable: there is no
`std::time`, `std::thread`, `std::fs` or `std::env` to reach for, and a limit on a long
solve is a step budget, never a timeout. Randomness is the one effect left, and the
specification already states its contract. `human-solving.allium` says that draws come from
the library's randomness boundary and a caller supplies the stream; that the generator is
the library's, named by `random_version` so that a different generator is a different
name; and that an exact replay records the seed and every consumed draw and asks for a
stream of `u` in `[0, 1)`, begun from the seed, indexed from zero, the same for the same
seed and `random_version`.

## Decision

Every effect the engine has sits behind a trait the library defines, and it has one. The
randomness boundary is the `RandomStream` trait in `crates/pawdoku/src/random.rs`: a
stream of draws in `[0, 1)` begun from a seed, indexed from zero, the same for the same
seed and `random_version`. The library ships one generator, `SeededStream`, named by
`RANDOM_VERSION`, so that `ExactReplay` holds across callers: the game, the command line
and Python replay the same solve from the same seed. A test supplies its draws through
the fake beside the trait, `ReplayStream`, which plays a scripted sequence and reports
exhaustion as an error rather than a panic. A caller may implement the trait with a
generator of its own, at the price of a different name.

`rand` and `getrandom` are banned from the core by cargo-deny. The command-line crate is
the only wrapper permitted to source entropy, because it is the one consumer with no host
to take a seed from.

## Consequences

The rule Pawdoku enforces by review is a compiler error here. A module that named a clock
or the filesystem would not build for `wasm32v1-none`, and `just wasm-check` proves that
under every feature combination; a dependency that sourced entropy fails `just deny`.

A replay is reproducible across the game, the command line and Python from one seed,
because the generator is the library's and not the host's.

The costs are three. The shipped generator must stay bit-stable under its name for as long
as any recorded replay cites it; a golden test pins the first draws of seed zero, and a
change to those bits is a new `RANDOM_VERSION` in the same commit, never a quiet edit.
Every caller threads a stream through the API, including the ones that would rather not
think about randomness. And the fake is part of the public test story:
`crates/pawdoku/tests/random.rs` exercises the trait, the generator and the fake from
outside the crate, so the fake's behaviour is a contract, not a convenience.

## What would reopen this

An effect that cannot be passed as a trait argument. Cancellation from outside a long solve
is the likely candidate: a step budget bounds the work, but a host that wants to stop a
solve early from another thread has nothing to hand in. A second effect that can be passed
is a second trait beside the first, not a reopening of this decision.

## Related pages

- [Architecture](../explanation/architecture.md)
- [Layering and dependency direction](../explanation/layering.md)
- [Testing](../reference/testing.md)
