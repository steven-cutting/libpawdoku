---
title: "Maintenance"
kind: "operations"
audience: [maintainer, operator, agent]
canonical_for: [maintenance_routine]
requires: []
---

# Maintenance

There is no service to operate. The library runs inside whoever calls it, nothing
accumulates between releases, and there is no on-call. Two workflows publish. `pages.yml`
deploys the API reference on every push to `main`; see
[Deploy to GitHub Pages](../how-to/deploy-to-github-pages.md). `release.yml` publishes
`pawdoku` to crates.io on a `pawdoku-v*` tag; see [Cut a release](#cut-a-release) below
and [decision 0019](../decisions/0019-releases.md). What follows is upkeep of the pins
that build and check it, and the release procedure.

## Routine

**Weekly.** Read the week's run of the `audit` workflow. It runs `just audit` against the
RustSec advisory database every Monday, as well as on every pull request, and it is never
a required check, so a failure there blocks nothing and is easy to miss. An advisory
against a crate in `Cargo.lock` is fixed by moving that crate, not by ignoring it.

**Monthly.** Move the dependency pins, one kind at a time, as
[Maintain dependencies](../how-to/maintain-dependencies.md) describes. `just lock-upgrade`
moves both lockfiles within the manifests' constraints — `Cargo.lock` within the caret
ranges, `pixi.lock` beneath the exact tool pins; run `just check` and commit both. A tool
pin itself moves in `pyproject.toml`, then `pixi update <name>`, then `pixi.lock` is
committed. A line in `tools.txt` or `tools-source.txt` moves by hand and takes effect at
the next `just install-tools`; a `tools-source.txt` line moves one release at a time,
with `just check` run on each, because rustqual is a gate. A hook `rev` or a GitHub
Action SHA moves by hand, with its version comment moved beside it.

**Monthly.** Run `just check-links-online`. It is not part of the gate because it needs
the network, so external links rot silently until someone looks.

**Per toolchain release.** Bump `rust-toolchain.toml`, then run `just clippy` before
anything else: clippy's lint set moves with the release, and a lint that is new under
`-D warnings` fails the gate with no change to the code. `just check` follows.
`rust-version` in the root `Cargo.toml` does not move with it: it is the oldest compiler
a consumer may build the crate with, and it rises only on purpose, in a minor release
([decision 0019](../decisions/0019-releases.md)).

**Per Allium release.** The checker's version and its checksums live in the tooling
package, not here; moving that package's pin in `pyproject.toml` moves the checker. A
moved checker re-verifies every module under `docs/specs/`, and any waiver is valid only
against the version it was verified on, so each is checked again.

Nothing here moves on its own. No bot opens a pull request for any of these pins until
ticket S01 decides whether one should.

## Cut a release

A release of `pawdoku` is cut in five steps, each authorised on its own: the push, the
pull request, the tag and the deployment approval are separate acts. Versions are
independent per crate and tagged `<crate>-v<version>`.

1. **Prepare.** From an up-to-date `main`, switch to `release/pawdoku-v<version>`. Run
   `cargo release -p pawdoku <level>`, where the level is `patch`, `minor` or `major`,
   or nothing for 0.1.0, which the manifest already names. With no `--execute` it is a
   dry run, and shows the CHANGELOG rewrite and the commit it would make. Then run it
   again with
   `--no-tag --execute`. It bumps the version, moves the entries under `[Unreleased]`
   in the root `CHANGELOG.md` beneath a heading for the version, points the links at the
   tag, and commits. It never publishes and never pushes. The heading's date is
   cargo-release's `date` template variable, the UTC date, so a release cut late in the evening may
   carry tomorrow's date.
2. **Merge.** Push the branch and open a pull request; merge it once `check` is green.
3. **Tag.** `git switch main && git pull --ff-only`, so the tag lands on the merge
   commit and not on a stale local `main`, then `cargo release tag -p pawdoku
   --execute`, which writes the annotated tag `pawdoku-v<version>`. The first version
   only: publish it from this commit with `cargo publish -p pawdoku --locked` and a
   crates.io token scoped `publish-new` and `publish-update`, held off the repository;
   then configure the trusted publisher on crates.io (`steven-cutting`, `libpawdoku`,
   `release.yml`, environment `release`), require trusted publishing for new versions,
   and revoke the token.
4. **Publish.** `git push origin pawdoku-v<version>`. `release.yml` waits for the
   `release` environment's reviewer. Once approved, the guards refuse a tag off `main`
   or one that disagrees with the manifest, the auth step exchanges the run's identity
   token for a crates.io token, and `cargo publish` runs unless the version is already
   on crates.io.
5. **Watch.** docs.rs builds the version for the targets in
   `[package.metadata.docs.rs]`; check that the build is green.

**The 1.0 release.** Before 1.0 a breaking change is allowed with a minor bump, so CI's
`semver` job reports an API break against the last `pawdoku-v*` tag and blocks nothing.
The pull request that prepares 1.0 adds `semver` to the `needs` of `ci.yml`'s `check`
job, so from then on a break fails the required check.

## Secrets

None are stored, and there is nothing to rotate. Continuous integration uses the token
GitHub mints for each run and discards with it. `release.yml` publishes through trusted
publishing: the run's identity token is exchanged for a crates.io token that lives thirty
minutes at most and is revoked when the job ends. The one exception is the token the
first version of a crate needs, because crates.io cannot name a trusted publisher for a
crate with no version; it lives on the maintainer's machine and is revoked once trusted
publishing is on.

## Related pages

- [Maintain dependencies](../how-to/maintain-dependencies.md)
- [Troubleshooting](troubleshooting.md)
- [Security model](../explanation/security-model.md)
