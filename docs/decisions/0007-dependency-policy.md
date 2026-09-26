---
title: "Decision 0007: Dependency policy for a library"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_dependency_policy]
requires: []
---

# Decision 0007: Dependency policy for a library

## Context

The games pin every dependency exactly. Their `AGENTS.md` invariant reads "No `^`, no
`~`", in `package.json` or `pyproject.toml`, with lockfiles committed and proved by
`just lock-check`. That is right for an application, whose manifest is the last word on
what it runs. A library's manifest is a constraint every consumer inherits. A crate that
writes `=1.2.3` for a dependency forbids every other crate in a consumer's tree from
resolving a different `1.2.x`, so two such libraries in one tree cannot resolve at all,
and the game's WebAssembly package would carry the poison to everyone downstream of it.

Two further facts shape the mechanism. The gate runner snapshots the worktree between
recipes and aborts on any change, so no recipe may rewrite `Cargo.lock`. And the founding
tickets give the lockfile one owner: no dependency is added outside the ticket that builds
the Rust gate.

## Decision

Manifests write full `x.y.z` caret ranges, in one `[workspace.dependencies]` table that
every crate inherits from; a crate manifest names a dependency with `workspace = true`
and never a version of its own. `Cargo.lock` is committed and is the pin. Every cargo
invocation in a gate carries `--locked`, so a recipe that would need to change the
lockfile fails instead, and `just lock-check` proves the lockfile matches the manifests.
cargo-deny checks licences, bans and sources offline at gate 12 of `just check`, and
checks advisories in `just audit`, which runs in CI only, because the RustSec fetch can
fail for reasons unrelated to the diff. `cargo-shear` fails the gate on a dependency
nothing uses. `rust-version` follows the toolchain pin until the crate is first
published. There is no `.cargo/config.toml`: `RUSTFLAGS` set there changes fingerprints
and thrashes the build directory against rust-analyzer.

This is the one place `AGENTS.md` departs from the games' invariants, and it says so: its
eighth invariant states the lockfile as the pin and cites this record. The reason is the
one above. An exact pin is a promise about this repository's resolution; in a library it
becomes a demand on everyone else's.

## Consequences

The consequences that hurt. The lockfile tests one resolution and consumers get another:
a bug in a dependency version this repository never resolves is invisible here until a
consumer hits it. Nothing updates any pin until the dependency-updates spike decides how,
so the lockfile ages deliberately rather than by neglect. Every addition goes through one
lane while the foundation is built and through one reviewer afterwards, with the reason
recorded in the manifest beside the range. And caret ranges make `cargo update` a real
change every time it runs: a moved lockfile is a diff to read, not noise to wave through.

## What would reopen this

A binary crate in the workspace is not a reopening: one lockfile still resolves once, and
the binary ships that resolution. Publishing may add a minimal-versions check, because a
caret range is a claim that the crate builds against its floor, and nothing here tests
that today. A native dependency in a bindings crate that must be pinned exactly would be a
per-crate exception recorded in this file, not a return to exact pins everywhere.

## Related pages

- [Maintain dependencies](../how-to/maintain-dependencies.md)
- [Configuration](../reference/configuration.md)
