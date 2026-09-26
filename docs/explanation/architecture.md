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
root, `src/random.rs` is the randomness boundary, and `tests/` holds the integration
tests. Behaviour arrives as one Rust module per specification module, and a module may
use only what its specification imports, in the direction
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

No I/O, no clock, no threads, no `rand`, no network, no persistence, no user interface,
and no puzzle generation for now. Those are not deferred; they are out of scope, as
[Purpose and scope](../project/purpose-and-scope.md) records.

## Related pages

- [Layering and dependency direction](layering.md)
- [Security model](security-model.md)
- [Repository map](../project/repository-map.md)
- [API reference](../reference/api.md)
