---
id: T13
title: "Release: cargo-release, release.yml with trusted publishing, the release guards, pawdoku 0.1.0"
status: open
depends_on: [S02]
parallel_with: [S12, T40, T43]
branch: ticket/t13-release
estimated_size: M
---

# T13: Release: cargo-release, release.yml with trusted publishing, the release guards, pawdoku 0.1.0

## Context

S02 (`tickets/S02-release-and-publishing.md`, `status: done`) recommended how a release
is cut, and drafted this ticket in its hand-back notes, Step 7, on 2026-09-27. The draft
is the starting point; this file is that draft brought up to date with `main` on
2026-10-31 and with what S04 (`tickets/S04-bindings-and-cli.md`, `status: done`) handed
to it. Read S02's Steps 3 to 6 for the evidence; it is not repeated here.

**The recommendation, as S02 left it.** cargo-release, run on the maintainer's machine,
bumps the version and rewrites the CHANGELOG on a `release/*` branch; the release commit
reaches `main` through a pull request, because `main` requires `check`; the maintainer
tags the merge commit and pushes the tag; `release.yml` publishes on the tag through
crates.io trusted publishing. Versions are independent per crate, tagged
`<crate>-v<version>` (`pawdoku-v0.1.0`). `rust-version` is decoupled from the toolchain
pin at the first publish and rises only on purpose, in a minor release. The first
publish needs a token, because crates.io cannot configure a trusted publisher for a
crate that has no version yet; that token lives on the maintainer's machine and is
revoked once trusted publishing is on.

**What changed since S02's draft, checked on 2026-10-31.** Each line names its source.

| Fact | 2026-09-27 (S02) | 2026-10-31 | Source |
| --- | --- | --- | --- |
| The four crate names | free | still free: the API answers "does not exist" for `pawdoku`, `pawdoku-wasm`, `pawdoku-cli` and `pawdoku-py` | `https://crates.io/api/v1/crates/<name>` with a User-Agent |
| cargo-release | 1.1.6, not on conda-forge | 1.1.7 (2026-10-21, `rust_version` 1.97, under the 1.98.1 pin), still not on conda-forge (`pixi search cargo-release` finds nothing for `linux-64` or `osx-arm64`) | crates.io API; `pixi search` 0.83.0 |
| cargo-semver-checks | 0.50.0 on conda-forge | 0.51.0 (2026-10-03, `rust_version` 1.93) on crates.io and on conda-forge for both platforms | crates.io API; `pixi search` |
| `rust-lang/crates-io-auth-action` | `c6f97d42243bad5fab37ca0427f495c86d5b1a18 # v1.0.5` | `3268eb413133e2960fa4ae0e963ca76380c54cd9 # v1.0.6` (released 2026-10-21; `git/ref/tags/v1.0.6` is a commit, so the tag is lightweight and there is nothing to dereference) | `gh api repos/rust-lang/crates-io-auth-action/releases/latest` and `.../git/ref/tags/v1.0.6` |
| `actions/checkout` | the SHA `ci.yml` pins | unchanged: `3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1` is still the latest release | `gh api repos/actions/checkout/releases/latest` |
| The record number | `0012-releases.md` | 0012 to 0016 are on `main`; 0017 is taken by `docs/decisions/0017-guide-pages-in-the-api-reference.md` on the open branch `ticket/t34-guide-page-and-diagrams` (pull request #41); 0018 is claimed by T37. The next free number on 2026-10-31 is **0019**. | `ls docs/decisions`; `git ls-tree` over every remote branch; `tickets/T37-layout-spec.md` |
| What publishes | nothing | one workflow, `pages.yml`, which T33 added (decision 0016); `AGENTS.md` lines 129 to 136 say so and name S02 for crates.io | `AGENTS.md` |
| GitHub environments | `copilot` only | `copilot`, and `github-pages` (T33), whose deployments are limited by a custom branch policy; no rulesets | `gh api repos/steven-cutting/libpawdoku/environments` and `.../rulesets` |

**What S04 handed to this ticket** (S04's "Handed back", "To T13"), each now part of
the design below:

1. **The `release` environment admits four tag patterns**, not `pawdoku-v*` alone,
   because S04 designed one workflow per registry (`release.yml` for crates.io,
   `release-npm.yml` for GitHub Packages, `release-pypi.yml`, `release-cli.yml`), all
   sharing the one environment. GitHub's "Create a deployment branch policy" endpoint
   takes `name` and `type`, whose allowed values are `branch` and `tag`, and needs the
   environment's `deployment_branch_policy.custom_branch_policies` to be `true`
   (<https://docs.github.com/en/rest/deployments/branch-policies>, read 2026-10-31). So
   the rule is four policies of `type: "tag"`: `pawdoku-v*`, `pawdoku-wasm-v*`,
   `pawdoku-py-v*` and `pawdoku-cli-v*`. `pawdoku-v*` does not match
   `pawdoku-wasm-v0.1.0`, because the pattern needs the `v` straight after `pawdoku-`.
2. **The two guards are a composite action**, `.github/actions/release-guards/action.yml`,
   because the three sibling workflows reuse them verbatim. Inputs: `package`, the cargo
   package name; the tag prefix is `<package>-v`.
3. **Each binding crate carries its own `[package.metadata.release]`.** This ticket
   writes the workspace table and `pawdoku`'s; T15 writes `pawdoku-wasm`'s.
4. **One CHANGELOG per binding crate, not one root file with a heading per crate.**
   S04 recommended the single root file. The replacements S02 proved cannot serve it:
   each rewrites the file's one `## [Unreleased]` heading and its one `[Unreleased]:`
   link, so cutting `pawdoku-wasm` would move every unreleased entry, the core's
   included, under the wasm release, and point the one compare link at the wasm tag.
   So the root `CHANGELOG.md` stays the core's and the repository's, and its
   replacements are `pawdoku`'s (step 3, unchanged). Each binding crate keeps its own
   `crates/<crate>/CHANGELOG.md`, which its own `[package.metadata.release]` rewrites
   with the same two replacements and its own tag prefix, and which its package can
   ship. The maintainer settled it (below); `pawdoku-wasm`'s file is T15's, and
   `pawdoku-py`'s and `pawdoku-cli`'s are T17's and T16's.

**Settled by the maintainer, 2026-10-31.** Three questions this ticket raised were put
to the maintainer, each with the ticket writer's proposal, and each proposal was taken:

- **One changelog per binding crate**, as item 4 above says.
- **When to publish 0.1.0: soon after the first pull request merges.** The crate holds
  the rules, the solver, the board with its record, and the basic generator, which is
  something a consumer can use, and publishing secures the name. 0.1.0 does not wait
  for T43, which makes a record name its format version: that changes the record's
  serialised shape, and if it lands after 0.1.0 it ships as 0.2.0, which semver allows
  before 1.0.
- **The full setup in the privileged job.** The `id-token: write` job runs the
  composite setup action (the pixi environment, the tools, the hooks) before
  `cargo publish`, as step 4 writes it, and not crates.io's leaner checkout, auth and
  publish. It is the one tested path to `just` and the pinned toolchain, and the guards
  need `cargo metadata --locked`; the token is minted after every setup step has run
  and lives thirty minutes at most.

**Why T15 depends on this ticket, and on what part of it.** T15 publishes
`@steven-cutting/pawdoku-wasm` to GitHub Packages, not to crates.io, so it does not need
`pawdoku` on crates.io. It needs the machinery: `[workspace.metadata.release]`, the
cargo-release pin, the `release` environment and the tag ruleset, and the guards'
composite action. So this ticket lands in two pull requests, as T33 did: the first
lands the machinery and the record, and T15 may start once it has merged; the second
carries the 0.1.0 release and its proof, and follows soon after the first merges
(settled above).

Four files this ticket edits are frozen under CONVENTIONS.md §11: the `Justfile`,
`tools.txt`, `rust-toolchain.toml`, and, for the record's entry, `docs/manifest.yml`.
T22, T24, T29 and T33 reopened the `Justfile` as follow-ups carried in their own tickets;
this ticket is the same kind of carrier, and step 1 asks before any edit.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §1 decision 3, §10, §11, §12, §13;
S02's Steps 3 to 7 and its Deviations; S04's Step 5 "Release" paragraph and its
"Handed back"; `Cargo.toml` and `crates/pawdoku/Cargo.toml`; `CHANGELOG.md`;
`.github/workflows/ci.yml`, `.github/workflows/pages.yml` and
`.github/actions/setup/action.yml`; `docs/decisions/0007-dependency-policy.md` and
`docs/decisions/0016-api-reference-on-github-pages.md`;
`docs/operations/maintenance.md`; `docs/explanation/security-model.md`.

## Goal

`pawdoku 0.1.0` is on crates.io with a green docs.rs build. Every later version is
published by `release.yml` with `trustpub_data` set. The token the first publish needs is
revoked. The `release` environment has a required reviewer and admits the four tag
patterns; a tag ruleset restricts who may create those tags; the guards that refuse a
tag off `main` or a tag that disagrees with the manifest are one composite action that
every release workflow calls. A `semver` job reports API breaks against the last
`pawdoku-v*` tag. The handbook says how a release is cut, and no page still says nothing
is published to crates.io.

## Non-goals

- The binding crates' release workflows: `release-npm.yml` is T15's, `release-pypi.yml`
  T17's and `release-cli.yml` T16's. This ticket writes only what they share.
- A 1.0 date or an API stability promise.
- A GitHub Release object: the CHANGELOG links to the tag page, which GitHub serves
  without one, so `release.yml` needs no `contents: write`.
- An MSRV CI job: the maintainer declined one (S02, "Open points settled"), and clippy's
  `incompatible_msrv` lint reads `rust-version` as a partial guard.
- The binding crates' changelogs: one per crate (settled above), and each is written by
  the ticket that makes the crate, `crates/pawdoku-wasm/CHANGELOG.md` by T15 and the
  others by T16 and T17.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `Cargo.toml` | workspace manifest | A `[workspace.metadata.release]` table (step 3) |
| `crates/pawdoku/Cargo.toml` | crate manifest | `publish.workspace = true` deleted; `[package.metadata.release]` (step 3) |
| `crates/pawdoku/LICENSE` | crate | New: a symlink to `../../LICENSE` (step 3) |
| `.github/actions/release-guards/action.yml` | workflow | New: the two guards as a composite action (step 4) |
| `.github/workflows/release.yml` | workflow | New: publish `pawdoku` on a `pawdoku-v*` tag (step 4) |
| `.github/workflows/ci.yml` | workflow | A `semver` job, not in `check`'s `needs` (step 5) |
| `Justfile` | T00 follow-up | The `semver` recipe (step 5). Frozen; step 1 |
| `tools.txt` | T00 follow-up | `cargo-release@1.1.7` (step 2). Frozen; step 1 |
| `pyproject.toml` | T00 follow-up | `cargo-semver-checks = "==0.51.0"` with a comment (step 2) |
| `pixi.lock` | generated | `pixi lock` after the pin (step 2) |
| `rust-toolchain.toml` | T00 follow-up | The comment at lines 3 and 4: `rust-version` no longer follows the pin. Frozen; step 1 |
| `docs/decisions/0019-releases.md` | decision | New record (step 7); the number is the next free one on the day |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | T00 follow-up | The record's entry. Frozen; step 1 |
| `docs/decisions/0007-dependency-policy.md` | decision | Lines 41 and 42: "`rust-version` follows the toolchain pin until the crate is first published" becomes past tense, pointing at the new record |
| `docs/operations/maintenance.md` | operations | A "Cut a release" section; "Per toolchain release" at 37 to 40 no longer moves `rust-version`; "Secrets" at 50 to 54 in the present tense; the sentence at 11 to 15 names both publishing workflows |
| `docs/explanation/security-model.md` | explanation | Lines 26 and 27 in the present tense; the publishing bullets name `release.yml` and its environment |
| `docs/reference/quality-gates.md` | reference | "In continuous integration": the `semver` job, and `release.yml` beside `pages.yml` |
| `docs/reference/commands.md` | reference | A row for `just semver`, outside `just check` |
| `docs/reference/configuration.md` | reference | The `Cargo.toml` row at 42: a member is publishable unless it writes `publish.workspace = true` or `publish = false` |
| `docs/reference/api.md` | reference | Lines 55 to 58: docs.rs is no longer prospective once 0.1.0 is out (second pull request) |
| `docs/project/repository-map.md` | project | The tree and the responsibility table: `crates/pawdoku/LICENSE`, `release.yml`, the guards action |
| `AGENTS.md` | agent contract | The paragraph at 129 to 136: two workflows publish, and how (step 8) |
| `SECURITY.md` | maintainer docs | Lines 14 and 15: the supported versions once 0.1.0 is out (second pull request) |
| `README.md` | maintainer docs | Lines 97 to 99: how a release is cut, and where it is published (second pull request) |
| `CHANGELOG.md` | maintainer docs | This ticket's entries under `[Unreleased]`; the `[0.1.0]` heading comes from the release run, never a hand edit |
| `tickets/README.md` | ticket index | T13's row set `done`, in the closing pull request |
| `tickets/T13-release.md` | ticket | Hand-back notes in both pull requests; `status:` in the closing one |

## Steps

1. Create the worktree on `ticket/t13-release` from `main` (`tickets/README.md`, "How to
   pick up a ticket"). Run `just initialize` if `.pixi/` is absent, and `just
   install-tools` if `.tools/bin` lacks cargo-hack or rustqual; each uses the network
   and this ticket authorises it. Run `just check` before any edit and quote its last
   line.

   Before editing anything, confirm with the maintainer that the four frozen files in
   Files touched may be edited here: the `Justfile`, `tools.txt`, `rust-toolchain.toml`
   and `docs/manifest.yml`. A record, its manifest entry and its index row are one commit
   or the documentation contract fails, and the `semver` job runs the recipe, so there is
   no landing part of this without them: without leave for all four, stop and say so.

   Then re-check every row of the Context's table on the day, and quote each: the four
   names, cargo-release on crates.io and its absence from conda-forge,
   cargo-semver-checks on conda-forge, the auth action's latest release and its SHA, the
   environments and rulesets. A row that moved is updated here and in the record, and
   said so under Deviations.

2. **The pins.** Add `cargo-release@1.1.7` to `tools.txt`, under the existing line, with
   the date in the file's header comment updated. cargo-binstall verifies no checksum for
   it (S02 Step 4), the same standing as cargo-hack; say so in the comment. The
   alternative S02 named, a maintainer prerequisite instead of a pin, is not taken: one
   pin, one owner. Add `cargo-semver-checks = "==0.51.0"` to `[tool.pixi.dependencies]`
   in `pyproject.toml` with a one-line comment, then `pixi lock`, and quote that both
   platforms solved. Run `just install-tools` and quote `cargo release --version`.

3. **The configuration, on the first branch.** In the root `Cargo.toml`:

   ```toml
   [workspace.metadata.release]
   allow-branch = ["main", "release/*"]
   pre-release-commit-message = "Release {{crate_name}} {{version}}"
   tag-message = "Release {{crate_name}} {{version}}"
   publish = false
   push = false
   ```

   The local step never publishes and never pushes: the release commit reaches `main`
   through a pull request, and the tag is pushed by hand. In
   `crates/pawdoku/Cargo.toml`, delete `publish.workspace = true`. The workspace's
   `publish = false` stays, but it is not a default: Cargo applies a
   `[workspace.package]` field to a member only when the member writes
   `<field>.workspace = true`, so deleting the line makes `pawdoku` publishable, and
   every crate that must stay off crates.io writes `publish.workspace = true` or
   `publish = false` itself. Add the crate's table, exactly as S02 proved it:

   ```toml
   [package.metadata.release]
   pre-release-replacements = [
     { file = "../../CHANGELOG.md", search = "## \\[Unreleased\\]\n", replace = "## [Unreleased]\n\n## [{{version}}] - {{date}}\n", exactly = 1 },
     { file = "../../CHANGELOG.md", search = "\\[Unreleased\\]: .*", replace = "[Unreleased]: {{repository}}/compare/pawdoku-v{{version}}...HEAD\n[{{version}}]: {{repository}}/releases/tag/pawdoku-v{{version}}", exactly = 1 },
   ]
   ```

   The tag is named outright because `{{tag_name}}` is left unrendered under
   `--no-tag` (S02 Step 4(b)). Create `crates/pawdoku/LICENSE` as a symlink to
   `../../LICENSE`, and quote `cargo package -p pawdoku --list --locked` listing it. Run
   `cargo release -p pawdoku` (a dry run, the default) and quote it; then quote
   `git status --porcelain` showing the tree unchanged.

4. **The guards and the workflow.** `.github/actions/release-guards/action.yml`, a
   composite action with one input, `package`, and two `bash` steps, run after the
   checkout and the setup action:

   - **On `main`.** `git fetch --no-tags origin main`, then fail unless
     `git merge-base --is-ancestor "$GITHUB_SHA" FETCH_HEAD` succeeds, so only a commit
     that reached `main` through a checked pull request is published. The checkout
     needs `fetch-depth: 0` or the fetch must deepen; say which in a comment.
   - **The tag is the manifest's version.** Fail unless `$GITHUB_REF_NAME` equals
     `<package>-v` followed by that package's version from
     `cargo metadata --locked --no-deps --format-version 1`.

   Inputs reach the shell through `env:`, never through `${{ }}` inside a `run:` line,
   as `ci.yml`'s `check` job comment says. Each step's message names what it refused.

   `.github/workflows/release.yml`: `on: push: tags: ['pawdoku-v*']`; top-level
   `permissions: contents: read`; one job, `publish`, with `environment: release` and
   `permissions: id-token: write, contents: read`; concurrency grouped on the tag, never
   cancelled. Steps: `actions/checkout` at the SHA `ci.yml` pins, with
   `persist-credentials: false`; the composite setup action; `just check-toolchain`; the
   guards with `package: pawdoku`, before any token exists;
   `rust-lang/crates-io-auth-action@3268eb413133e2960fa4ae0e963ca76380c54cd9 # v1.0.6`
   with `id: auth`, which runs on every tag so every tag proves the OIDC exchange; a step
   that asks `https://crates.io/api/v1/crates/pawdoku/<version>` with a User-Agent
   whether the version exists and skips the publish if it does, because the first
   version is published by token and a re-run must not fail; and `cargo publish -p
   pawdoku --locked` with `CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}`. Plain
   `cargo publish`, not `cargo release publish`: the workspace table says
   `publish = false`, and a tag checkout is a detached `HEAD`, which `allow-branch`
   refuses. Re-read the auth action's `action.yml` at the pinned SHA and quote its
   inputs and outputs; S02 read v1.0.5.

   Run `actionlint` over both files through `just lint` and quote it green.

5. **The `semver` job.** A recipe in the `Justfile`, after `deps-unused`, outside the
   gate list:

   ```text
   # API breaks in pawdoku against the last pawdoku-v* tag, through the pinned
   # cargo-semver-checks. Not a gate: before 1.0 a break is allowed with a minor
   # bump, so CI reports it and never blocks; at 1.0 the job joins check's needs.
   # With no tag yet there is nothing to compare, and the recipe says so.
   semver:
       #!/bin/sh
       set -eu
       base=$(git describe --tags --match 'pawdoku-v*' --abbrev=0 2>/dev/null) || { printf '%s\n' 'semver: no pawdoku-v* tag yet, nothing to compare'; exit 0; }
       cargo semver-checks -p pawdoku --baseline-rev "$base"
   ```

   Find out whether `cargo semver-checks` 0.51.0 accepts `--locked` or passes it
   through, and add it if it does; quote the help text either way. In `ci.yml`, a job
   `semver` (`name: semver`, `timeout-minutes: 20`), checkout with `fetch-depth: 0` and
   `persist-credentials: false`, the setup action, then `just semver`. It is **not** in
   `check`'s `needs`, and the comment above it says why and that it joins at 1.0. Write
   that trigger into "Cut a release" too, so the 1.0 release carries it.

6. **The platform controls, before any publisher exists.** Each is a separately
   authorised action; ask for each.

   - Create the environment `release`: the maintainer as a required reviewer;
     `deployment_branch_policy` `{protected_branches: false, custom_branch_policies:
     true}`; then four deployment branch policies, each `type: "tag"`, named
     `pawdoku-v*`, `pawdoku-wasm-v*`, `pawdoku-py-v*` and `pawdoku-cli-v*`.
   - Create a tag ruleset, enforcement `active`, targeting `refs/tags/pawdoku-v*`,
     `refs/tags/pawdoku-wasm-v*`, `refs/tags/pawdoku-py-v*` and
     `refs/tags/pawdoku-cli-v*`, restricting creation, update and deletion, with the
     maintainer alone on the bypass list. Without both, anyone who can push a matching
     tag makes the auth action mint a crates.io token.

   Read both back with `gh api` and quote them in the notes.

7. **The decision record**, `docs/decisions/0019-releases.md` in the shape of 0016, with
   `canonical_for: [decision_releases]`; use the next free number on the day and rename
   the file, the title, the index row and the manifest entry if 0019 is taken. Context:
   S02's survey and the maintainer's choice of shape A; the token the first publish
   needs. Decision: cargo-release locally, the commit through a pull request, the tag by
   hand, CI publishing through trusted publishing; independent versions,
   `<crate>-v<version>` tags; one workflow per registry sharing the `release`
   environment, its four tag patterns and the guards; `rust-version` decoupled at the
   first publish; one changelog per crate, the root file the core's and the
   repository's; the full setup in the privileged job. Consequences, including the ones
   that hurt: the first token, a `tools.txt` binary with no checksum, a changelog per
   binding crate because the two replacements rewrite one `[Unreleased]`, and more code
   beside the token than a lean job would run. What would reopen it: crates.io offering
   pending publishers; a second maintainer. Register it in `docs/decisions/README.md`
   and `docs/manifest.yml`, and change "The next decision this repository takes is" to
   the number after it.

8. **The prose.** `AGENTS.md`'s paragraph at 129 to 136 names two publishing workflows:
   `pages.yml` (decision 0016) and `release.yml`, which publishes `pawdoku` to crates.io
   through trusted publishing on a `pawdoku-v*` tag, behind the `release` environment's
   required reviewer, and only its `publish` job holds `id-token: write`; a tag push and
   each deployment approval are separately authorised. Then `just check-agents`. Write
   the rest of the Files touched rows as listed. The "Cut a release" section in
   `docs/operations/maintenance.md` carries the sequence of steps 10 to 14, that
   `{{date}}` is the UTC date, and the 1.0 trigger of step 5.

9. **The first pull request.** Run `just check-docs`, then `just check`, and quote the
   last line. Commit on the ticket branch. Pushing and opening the pull request are
   separately authorised: ask. The hand-back notes of this pull request record steps 1
   to 8. After it merges, T15 may start; say so in the notes.

10. **The second pull request: prepare 0.1.0**, soon after the first has merged (settled
    above). On `release/pawdoku-v0.1.0` from `main`, run `cargo release -p pawdoku`
    (a dry run, quoted), then `cargo release -p pawdoku --no-tag --execute`. Push and
    open the pull request (each authorised separately) and merge it once `check` is
    green.

11. **Tag and publish 0.1.0 by token.** `git switch main && git pull --ff-only`, so the
    tag lands on the merge commit and not on a stale local `main`; `cargo release tag -p
    pawdoku --execute`; then `cargo publish -p pawdoku --locked` from that commit with a
    crates.io token scoped `publish-new` and `publish-update`, which the maintainer
    creates and holds. Never write the token to a file in the repository.

12. **Configure trusted publishing on crates.io**: the publisher (`steven-cutting`,
    `libpawdoku`, `release.yml`, environment `release`), then "Require trusted
    publishing for all new versions". Revoke the token.

13. **Push the tag**: `git push origin pawdoku-v0.1.0`. `release.yml` waits for the
    environment's approval; then the guards pass, the auth step proves the exchange, the
    existence check skips the publish, and the job is green. Quote the run's log lines.

14. **Watch docs.rs** build 0.1.0 for both targets in `[package.metadata.docs.rs]`.
    Optionally, prove the path that matters: cut 0.1.1 the same way with step 11's token
    publish removed, and read `trustpub_data` from the API. Otherwise the next real
    release is the proof, and the notes say so. Update `docs/reference/api.md`,
    `SECURITY.md` and `README.md` in this pull request, set `status: done` and T13's row
    in `tickets/README.md` to `done`.

**Authorisation**, each asked for on its own (CONVENTIONS.md §11): each push and each
pull request; creating the `release` environment, its policies and the tag ruleset;
creating the maintainer's crates.io token; the token publish; the trusted-publisher
configuration and the trusted-only setting; revoking the token; pushing the tag; and
approving each `release.yml` deployment in the `release` environment, which is the
publish itself and a separate decision from the tag push (`AGENTS.md`, "Safety and
authority").

## Acceptance criteria

- First pull request: `just check` green; `tools.txt`, `pyproject.toml` and `pixi.lock`
  carry the two pins; the dry run of step 3 quoted with the tree unchanged after it;
  `cargo package -p pawdoku --list --locked` lists `LICENSE`; `actionlint` green on both
  new files; the environment and the ruleset read back with four tag patterns each,
  before any publisher is configured; the record is in the manifest and the index; no
  page says nothing is published to crates.io or names S02 as a decision still to come.
- Second pull request: `https://crates.io/api/v1/crates/pawdoku` lists 0.1.0; docs.rs
  shows a green build; the first `release.yml` run is green with the token exchange in
  its log; `trustpub_only` is `true` on the crate; the token is gone from the
  maintainer's tokens page; either 0.1.1 carries `trustpub_data.repository:
  steven-cutting/libpawdoku` or the notes record that the first trusted publish is
  still to come; `just check` green.
- `git status --porcelain` lists only paths in Files touched.

## Verification

```sh
grep -n 'cargo-release' tools.txt
grep -n 'cargo-semver-checks' pyproject.toml
grep -n -A6 'workspace.metadata.release' Cargo.toml
grep -n 'publish' Cargo.toml crates/pawdoku/Cargo.toml
cargo package -p pawdoku --list --locked | grep -x LICENSE
cargo release -p pawdoku
just semver
gh api repos/steven-cutting/libpawdoku/environments/release --jq '{rules: [.protection_rules[].type], policy: .deployment_branch_policy}'
gh api repos/steven-cutting/libpawdoku/environments/release/deployment-branch-policies --jq '.branch_policies[] | {name, type}'
gh api repos/steven-cutting/libpawdoku/rulesets --jq '.[] | {name, target, enforcement}'
rg -n 'S02|later decision' AGENTS.md docs README.md SECURITY.md
just check-docs
just check
git status --porcelain
```

Expected: the two pins; the table of step 3; `publish = false` in the workspace table
and no `publish` line in the crate; `LICENSE`; a dry run ending "aborting release due to
dry run"; `semver: no pawdoku-v* tag yet, nothing to compare` before 0.1.0 and a summary
after; `required_reviewers` and `custom_branch_policies: true`; four policies of type
`tag`; one active tag ruleset; no line saying crates.io publishing is a decision still
to come (decision records of their day excepted); `All checks passed and the worktree is
unchanged.`; only paths in Files touched.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- **Paths later tickets also edit.** T15 edits `.github/workflows/ci.yml`, `AGENTS.md`,
  `docs/operations/maintenance.md`, `docs/project/repository-map.md`,
  `docs/reference/quality-gates.md`, `docs/reference/commands.md`, `CHANGELOG.md` and
  `docs/explanation/security-model.md` after this ticket's first pull request. T15
  starts once that pull request has merged, so the two never overlap it; but the second
  pull request follows soon after (settled above), so T15 may be open beside it, and
  then both edit the root `CHANGELOG.md` (whose `[Unreleased]` the 0.1.0 release run
  rewrites) and `README.md`. T37 to T39, T34, T40 and T43 may be open at the same time
  as either pull request and share `CHANGELOG.md`, `docs/manifest.yml` and
  `docs/decisions/README.md`. Each shared edit is an entry or a sentence; whichever
  merges second merges `main` first and takes the next decision number, the deviation
  from CONVENTIONS.md §11 that T31 and T33 recorded.
