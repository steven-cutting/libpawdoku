# Security policy

## Reporting a vulnerability

Report suspected vulnerabilities privately, through GitHub's private vulnerability
reporting on this repository. Please do not open a public issue for anything you believe
is exploitable.

Include what you did, what happened, and what you expected. A puzzle, a seed or a short
program that reproduces the behaviour is worth more than a description of it.

## Supported versions

Only `main` is supported. No version has been released and the crate is not on
crates.io, so there are no backports: a fix lands on `main`.

## What is in scope

- The crate's code, including a puzzle input that would make the solver, once it exists,
  run past its step budget or allocate without bound.
- Credential material committed to the repository.
- A dependency, tool binary or GitHub Action that has been tampered with, or a lockfile
  that does not match its manifest.

## Out of scope

- How a caller sources the seed it passes in, or its entropy.
- What a consumer does with the engine's output.

## What this project already does

- `#![forbid(unsafe_code)]` in every crate, through the workspace lint table, so nothing
  can opt back in.
- No network, clock, filesystem or environment access in the core. It is `no_std`, so
  none of them can even be named.
- `Cargo.lock` committed and every gate run with `--locked`; `just lock-check` fails if a
  manifest and its lockfile disagree.
- `cargo deny` checks licences, bans and sources on every `just check`, and the RustSec
  advisories weekly and on every pull request.
- `rand` and `getrandom` banned from the core, so it can never source entropy of its own.
- Tool binaries pinned by hash in `pixi.lock` and installed from conda-forge, and the
  compiler installed by rustup from `rust-toolchain.toml`. Three tools come from
  elsewhere, and no lockfile names the last two:
  - the `biscuit-games-tooling` package, from its Git tag, pinned to a commit in
    `pixi.lock`;
  - the Allium checker, from its GitHub release, pinned by version and SHA-256 in that
    package;
  - cargo-hack, from its GitHub release over TLS, which publishes no checksum to verify.
- Every GitHub Action and every remote hook pinned to a commit SHA rather than a mutable
  tag.
- Continuous integration runs with `contents: read` and holds no stored secret, only the
  token GitHub mints for each run.
- `ripsecrets` scans every commit, with its output suppressed so a match never copies the
  matched value into a log.

The reasoning behind all of this is in
[Security model](docs/explanation/security-model.md).
