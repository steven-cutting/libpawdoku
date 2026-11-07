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
  and the workflows need nothing more: the Pages deployment proves itself with the run's
  own identity token, so it stores no credential either. Publishing to crates.io uses
  trusted publishing, which exchanges the run's identity token for a short-lived registry
  token, so no registry token is stored here either. The first version of a crate is the
  exception: it is published with a token the maintainer holds off the repository and
  revokes once trusted publishing is on. Publishing the WebAssembly package to GitHub
  Packages uses the run's own `GITHUB_TOKEN`, which GitHub mints and discards with the
  run.

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
  commit SHA, not to a mutable tag. Three tools are the exceptions worth naming.
  cargo-hack and cargo-release, the two lines in `tools.txt`, are downloaded over TLS
  from their GitHub releases, and neither publishes a checksum or signature, so nothing
  verifies them beyond the transport. rustqual, the one line in `tools-source.txt`, publishes no binary at all, so
  it arrives as its crates.io source, checked against the registry index's checksum, and
  is built on the machine with `--locked`, taking the dependency versions its own
  lockfile names. Were cargo-quickinstall to publish a build of the pin, cargo-binstall
  would take that instead, and only if it is signed (`--only-signed`). A release asset rustqual itself
  published would not be taken: that pass disables cargo-binstall's `crate-meta-data`
  strategy, so nothing unsigned arrives by it. It also runs with CI's GitHub token
  unset, so the build scripts it compiles cannot read the token.
- **Memory safety.** `#![forbid(unsafe_code)]` holds in every crate through the
  workspace lint table. `forbid`, not `deny`: no member and no item can opt back in.
- **No network at runtime.** The core is `no_std`, so it cannot name a socket, and no
  dependency it ships with can open one on its behalf.
- **Workflow permissions.** `ci.yml` and `audit.yml` hold `contents: read` and nothing
  else. `pages.yml` holds more, `pages: write` and `id-token: write`, on its deploy job
  alone, which checks nothing out and runs no code from the repository
  ([decision 0016](../decisions/0016-api-reference-on-github-pages.md)). `release.yml`
  is the second, in a file of its own rather than widening `ci.yml`. Its one job,
  `publish`, holds `id-token: write` and runs in the `release` environment, whose
  required reviewer approves each run and whose deployment policies admit the four
  release tag patterns alone; a tag ruleset restricts who may create those tags. Unlike
  Pages' deploy job, it checks out the tagged commit and runs its setup action and
  `Justfile`, and GitHub grants the identity token to every step of the job, not to the
  auth step alone. So the ruleset and the reviewer are what stop a malicious tag; the
  guards, which refuse a tag off `main` or one that disagrees with the manifest, stop a
  mistaken one before the token is minted
  ([decision 0019](../decisions/0019-releases.md)). `release-npm.yml` is the third. Its
  one job, `publish`, holds `packages: write` and no `id-token`, runs in the same
  `release` environment and runs the same guards before it builds. The run's own
  `GITHUB_TOKEN` reaches npm only on the two steps that need it, the existence check and
  the publish, as an environment variable that `pkg/.npmrc` names and never holds; but
  `packages: write` is granted to the whole job, so, as with `release.yml`, the ruleset
  and the reviewer are what stop a malicious tag
  ([decision 0020](../decisions/0020-webassembly-package.md)).
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
