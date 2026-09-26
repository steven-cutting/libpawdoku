---
title: "Security model"
kind: "explanation"
audience: [user, contributor, maintainer, operator, agent]
canonical_for: [security_model]
requires: []
---

# Security model

This library has no process, no network and no data about anyone. That removes most of
the attack surface software usually has, and it is worth being precise about what
remains rather than claiming the problem away: the supply chain that builds the crate,
and the code handed to whoever consumes it.

## What there is to protect

Very little, and that is the point.

- **Nothing is collected or stored.** The engine takes values from its caller and hands
  values back. It keeps nothing between calls that the caller did not give it, and it
  has nowhere to send anything.
- **There are no credentials in the repository.** No file here carries a token.
  Continuous integration uses the token GitHub mints for each run and discards with it,
  and the workflows need nothing more. Publishing to crates.io, when ticket S02 decides
  it, will use trusted publishing, so no registry token will be stored here either.

## What is deliberately not secure

**Nothing in the engine is a secret.** A puzzle's solution, a rating or a hint is
computable by anyone holding the crate and the same inputs, and the seeded generator is
not a cryptographic one: nothing in the specifications asks for one, and it must not be
used where unpredictability matters.

## What the build defends

- **Supply chain.** `Cargo.lock` is the pin, and every gate passes `--locked`, so no
  recipe can resolve a version the lockfile does not name; `just lock-check` fails on
  drift between a manifest and its lockfile. `just deny` checks licences, bans and
  sources offline on every run of `just check`, and `just audit` checks the RustSec
  advisories weekly and on every pull request in its own workflow. `deny.toml` bans
  `rand` and `getrandom` from everything the workspace ships, so the core can never
  source entropy of its own, and `openssl-sys` from the whole workspace, so no native TLS
  stack arrives by accident. The ban is on what ships: development dependencies are
  outside the graph, because proptest depends on both randomness crates and the ban is
  not about the test harness. Every remote hook and every GitHub Action is pinned to a
  commit SHA, not to a mutable tag. One tool is the exception worth naming: cargo-hack,
  the one line in `tools.txt`, is downloaded over TLS from its GitHub release, and it
  publishes no checksum or signature, so nothing verifies it beyond the transport.
- **Memory safety.** `#![forbid(unsafe_code)]` holds in every crate through the
  workspace lint table. `forbid`, not `deny`: no member and no item can opt back in.
- **No network at runtime.** The core is `no_std`, so it cannot name a socket, and no
  dependency it ships with can open one on its behalf.
- **Workflow permissions.** Every workflow runs with `contents: read` and nothing else.
  The release workflow S02 designs will be the only one to hold more, and it will live in
  a file of its own rather than widen `ci.yml`.
- **Credential leakage.** `ripsecrets` scans every commit, and its output is suppressed so
  a match never copies the matched value into a log.

## What is out of scope

What a consumer does with the engine's output, and how it sources the seed it passes in.
There are no timeouts in the engine: where a computation needs a limit, the limit is a
step budget, and choosing one small enough for its own setting is the consumer's
responsibility.

## Reporting

See `SECURITY.md` at the repository root.

## Related pages

- [Architecture](architecture.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
- [Quality gates](../reference/quality-gates.md)
- [Decision 0007](../decisions/0007-dependency-policy.md)
