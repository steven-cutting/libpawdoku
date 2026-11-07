---
title: "Decision 0019: Releases, cut locally and published by trusted publishing"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_releases]
requires: []
---

# Decision 0019: Releases, cut locally and published by trusted publishing

## Context

Until now every crate was `publish = false` and nothing reached crates.io. Spike S02
surveyed how Rust libraries cut a release: cargo-release run by hand, release-plz's bot
pull request, git-cliff alone, and a tag and `cargo publish` from a laptop. It proved
cargo-release's two CHANGELOG replacements against the house CHANGELOG, and the
maintainer chose shape A during the spike: cargo-release on the maintainer's machine, a
pull request for the release commit, a tag pushed by hand, and CI publishing on the tag.
release-plz was turned down because its pull request needs a stored credential to be
checked, and its publisher sits on a workflow that runs on every push to `main`.

crates.io's trusted publishing exchanges a workflow's OIDC identity token for a
short-lived publish token, so no registry token is stored in the repository. It cannot
be configured for a crate that has no version yet: the first version is published with
an API token, and only then can a trusted publisher be named and the token revoked.

Spike S04 then designed one release workflow per registry for the binding crates. They
share the `release` environment and the guards, so this record decides what they share.
The maintainer settled three further questions on 2026-10-31: one changelog per binding
crate, 0.1.0 soon after the machinery merges, and the full setup in the privileged job.

## Decision

**cargo-release prepares a release locally.** It runs on the maintainer's machine, on
`main` or a `release/*` branch, bumps the version, moves the CHANGELOG's unreleased
entries under it, and commits. The workspace's `[workspace.metadata.release]` says
`publish = false` and `push = false`, so the local step never publishes and never
pushes. cargo-release is pinned in `tools.txt`.

**The release commit reaches `main` through a pull request**, because `main` requires
`check`. The maintainer tags the merge commit and pushes the tag by hand.

**CI publishes through trusted publishing.** `.github/workflows/release.yml` runs on a
`pawdoku-v*` tag. Its one job, `publish`, holds `contents: read` and `id-token: write`,
runs in the `release` environment, and runs the guards, then
`rust-lang/crates-io-auth-action` for the token, then `cargo publish -p pawdoku
--locked`, skipped when the version is already on crates.io. The first version is
published by the maintainer's token, and every later one by the workflow.

**Versions are independent per crate**, and each is tagged `<crate>-v<version>`:
`pawdoku-v0.1.0`, later `pawdoku-wasm-v0.1.0`.

**One workflow per registry, sharing what they can.** `release.yml` publishes to
crates.io; each binding crate's workflow is its own ticket's. All of them use the one
`release` environment, whose required reviewer approves each run and whose deployment
policies admit four tag patterns, `pawdoku-v*`, `pawdoku-wasm-v*`, `pawdoku-py-v*` and
`pawdoku-cli-v*`. A tag ruleset restricts who may create, move or delete those tags.
Every workflow calls the guards, the composite action
`.github/actions/release-guards`: one refuses a tag whose commit is not on `main`, the
other a tag that is not `<package>-v` and the manifest's version.

**`rust-version` is decoupled from the toolchain pin at the first publish.** It no
longer follows `rust-toolchain.toml`, and rises only on purpose, in a minor release.

**One changelog per crate.** The root `CHANGELOG.md` is the core's and the
repository's, and `pawdoku`'s replacements rewrite it. Each binding crate keeps its own
`crates/<crate>/CHANGELOG.md`, rewritten by its own table.

**The full setup in the privileged job.** The `publish` job runs the composite setup
action before the guards and the publish, rather than crates.io's leaner checkout,
auth and publish: it is the one tested path to `just` and the pinned toolchain, and the
guards need `cargo metadata --locked`.

**API breaks are reported, not gated.** A `semver` job in `ci.yml` runs
cargo-semver-checks against the last `pawdoku-v*` tag. Before 1.0 a break is allowed
with a minor bump, so the job is not among `check`'s `needs`; at 1.0 it joins them.

Turned down:

- **release-plz.** Its release pull request needs a stored credential to run `check`.
- **Publishing from the maintainer's machine every time.** A token on a laptop for
  every release, and a publish with no tag to show for it.
- **One root changelog with a heading per crate.** The two replacements each rewrite
  the file's one `[Unreleased]`, so cutting one crate would carry every crate's entries.

## Consequences

The ones that hurt.

- **The first publish needs a token.** It is created by the maintainer, scoped
  `publish-new` and `publish-update`, held off the repository, and revoked once the
  trusted publisher is configured.
- **cargo-release is a `tools.txt` binary with no checksum.** cargo-binstall takes the
  upstream release archive as published, the same standing as cargo-hack.
- **A changelog per binding crate.** Four files to keep where one would read better,
  because the two replacements rewrite one `[Unreleased]`.
- **More code beside the token than a lean job would run.** GitHub grants
  `id-token: write` to the whole job, so every step of `publish` could request an OIDC
  token from its first: the setup action, and the `Justfile` and composite actions of
  the tagged commit. Ordering the steps orders the mint, not the capability. What stops
  a malicious tag is the tag ruleset and the environment's reviewer; the guards stop a
  mistaken tag by the maintainer, before the auth step mints the token.
- **Two settings live outside the repository.** The `release` environment and the tag
  ruleset are not in any file, so no gate reads them. A workflow that names a missing
  environment does not wait: GitHub creates it with no protection rules. So a
  `preflight` job, holding no `id-token` and naming no environment, reads both and
  refuses the tag until the environment has a required reviewer and an active tag
  ruleset restricts creating, updating and deleting `pawdoku-v*` tags; `publish` needs
  it. It stops a skipped or drifted setting, not a malicious tag, since a tagged commit
  can rewrite the workflow.
- **Each release is three separately authorised acts**: the release pull request's
  merge, the tag push, and the deployment approval.
- **A red `semver` job blocks nothing before 1.0**, so a break is easy to miss.

## What would reopen this

crates.io offering pending publishers, so the first version needs no token. A second
maintainer, who changes who may tag and who approves.

## Related pages

- [Maintenance](../operations/maintenance.md)
- [Security model](../explanation/security-model.md)
- [Quality gates](../reference/quality-gates.md)
- [Decision 0007](0007-dependency-policy.md)
- [Decision 0016](0016-api-reference-on-github-pages.md)
