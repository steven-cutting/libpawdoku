---
id: T04
title: "CI workflows: the composite setup action, five gate jobs and the audit job"
status: open
depends_on: [T00, T01]
parallel_with: [T02, T03, T05, T06, T07, T08, T09]
branch: ticket/t04-ci
estimated_size: M
---

# T04: CI workflows: the composite setup action, five gate jobs and the audit job

## Context

T00 step 8 left two stubs: `.github/actions/setup/action.yml`, a composite action with the
`cache-key` input and the steps CONVENTIONS.md §10 lists, and `.github/workflows/ci.yml`,
G's header over one job named `check` that runs `just install-allium` then `just check`
under a 45-minute timeout. T01 pushed that stub to the remote and made `check` the one
required status. This ticket replaces the stub with the final shape: the composite action
finished, five plain gate jobs (`rust`, `coverage`, `wasm`, `deny`, `documents`), a
`check` job that needs the five and stays the one required check (CONVENTIONS.md §10),
and a separate `audit.yml` for the one recipe that needs the network. Keeping `check`
reporting is what lets this ticket's pull request, and every lane pull request pushed
after it merges, pass T01's protection without an administrator bypass.

Read first: `CONVENTIONS.md` in full (§2 for the pins, §4 for what each recipe does, §10
for the design this ticket embeds, §11 for the lane rules, §12 for the two claims assigned
here, §13 for the action-pin risk), then `README.md` in this directory, then T00's hand-back
notes for the stub as it landed.

Sources, read-only, at the commits CONVENTIONS.md §0 pins:

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`: `.github/workflows/ci.yml`. Lines
  6-8 are the quoted-branch comment this ticket copies verbatim; lines 11-12 the
  `contents: read` permission; lines 18-20 the concurrency group. G's jobs come from B's
  reusable workflow and are not the shape here.
- B = `/Users/scutting/projects/biscuit_games_tooling` at `6c5c07f6`:
  `.github/workflows/game-ci.yml` and `actions/setup-toolchain/action.yml`. The workflow's
  lines 14-16 are the checkout with `persist-credentials: false`; lines 19-23 the comment on
  why a token sits on one step and never on the job; lines 49-52 why `install-allium` is a
  workflow step and is not cached; lines 56-60 why a gate already inside `lint` is repeated
  as its own step. The action's lines 46-50 are the setup-uv step (version, cache on
  `uv.lock`) and lines 51-56 the rule that an input reaches a shell line through `env`.
- S = `/Users/scutting/projects/workato/street_meats` at its one `init` commit:
  `.github/workflows/ci.yml`, a plain multi-job workflow with SHA pins; the closest shape
  to what this ticket writes. Lines 40-42 are the checkout; lines 176-181 the
  `upload-artifact` step with `retention-days: 7`.

Read a pinned file with `git -C <clone> show <commit>:<path>`; never modify a clone.

Facts verified on 2026-09-23 that shape this ticket; each is re-verified on the day:

- The `ubuntu-latest` image (Ubuntu 24.04, version 20260907.300.1) ships Rustup 1.29.1
  and Cargo 1.98.1 (`images/ubuntu/Ubuntu2404-Readme.md` lines 139 and 142 in
  `actions/runner-images`), so `just install-toolchain` has a rustup new enough to read
  `rust-toolchain.toml`. Because the runner's own cargo equals the pin, a silently failed
  install would still let `rust` pass; the evidence it worked is `check-toolchain`'s
  `active-toolchain` line and the `wasm` job compiling for two targets the runner's
  default toolchain does not carry.
- `dtolnay/rust-toolchain` on `master` at `02cb101e` (also tag `v1`) declares `toolchain`
  as `required: true` (action.yml lines 9-11) and exits 1 when it is empty (lines 36-39);
  the `stable` branch head `6bed0761` differs only by defaulting `toolchain: stable`. So
  the action cannot read `rust-toolchain.toml` for free, which CONVENTIONS.md §10 and
  §12 already record: the only fallback restates the pin in YAML (`toolchain: "1.98.1"`),
  a second copy of a frozen value, which is why it is not used. Step 8 confirms.
- `taiki-e/install-action` takes `tool:` as a whitespace or comma separated list, and its
  manifests at v2.87.19 contain `just` 1.51.0 and `cargo-binstall` 1.23.0. It passes the
  run's token to itself as `DEFAULT_GITHUB_TOKEN` whenever `fallback` is `cargo-binstall`,
  its default.
- `Swatinem/rust-cache` v2.9.2 declares `cache-bin` and `save-if`, both defaulting to
  `true`. `cache-bin` governs `~/.cargo/bin` only; the action never touches `.tools/`.

Action pins, resolved read-only with `gh api repos/<owner>/<repo>/releases/latest` and
`git/ref/tags/<tag>` on 2026-09-23; re-verified on the day, a moved tag recorded:

| Action | Tag | Commit | Note |
| --- | --- | --- | --- |
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` | as B and S |
| `actions/cache` | v6.1.0 | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` | B carries v4.3.0; same inputs |
| `actions/upload-artifact` | v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | S carries v5.0.0 |
| `taiki-e/install-action` | v2.87.19 | `7623a79cdfecb99d681017af368ca353d9f49bb5` | |
| `Swatinem/rust-cache` | v2.9.2 | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` | annotated tag `63fed3e2` dereferenced; pin the commit, never the tag object |
| `astral-sh/setup-uv` | v10.2.0 | `c18668ad3cf93ea998bef934396af7bb5c839dc7` | B and S carry v9.0.0 |
| `dtolnay/rust-toolchain` | `v1` = `master` | `02cb101ec7c40f2c49e1d9714d64511d8e1b74de` | fallback only, not used; `stable` head `6bed0761d98439e5a578e2877258200ad565ba87` |

## Goal

Three files on `ticket/t04-ci`: the finished composite action and `ci.yml` (replacing
T00's stubs) and a new `audit.yml`; `just lint` green (actionlint over the workflows,
`check-yaml` over all three); one `workflow_dispatch` run of `ci.yml` on the ticket branch
with all six jobs green and their durations quoted, `check` among the names the checks
API reports for the commit; and the two CONVENTIONS.md §12 claims assigned to this ticket
checked and their outcomes recorded.

## Non-goals

- No change to `Justfile`, `tools.txt`, `rust-toolchain.toml` or any other frozen file. A
  recipe this ticket finds wrong in CI (say, `install-toolchain` needing `rustup show`) is
  handed back as a `main` follow-up, not fixed here (CONVENTIONS.md §11).
- No branch-protection change, now or later. T01's single required check is `check`,
  and this workflow keeps reporting it through the aggregate job, so a pull request from
  this branch merges under T01's protection like any other. T11 only reruns the
  protection script to prove it reads back unchanged.
- No `release.yml`, no Windows or macOS runner, no matrix (CONVENTIONS.md §10, S02).
- No Codecov, no coverage badge, no comment bot: nothing external.
- No reusable workflow: C01 lifts this shape into B later, after T11.
- No `dtolnay/rust-toolchain` step in the shipped action; it is documented as the
  fallback and its claim is recorded.

## Files touched

- `.github/actions/setup/action.yml` (replaces T00's stub)
- `.github/workflows/ci.yml` (replaces T00's one-job stub)
- `.github/workflows/audit.yml` (new)
- `tickets/T04-ci.md` (`status:` and hand-back notes)

Nothing else. Every other change is handed back.

## Steps

1. Create the worktree on `ticket/t04-ci` from `main` after T00 and T01 have merged
   (README.md "How to pick up a ticket"). Confirm `just check` is green on `main` before
   touching anything.

2. Re-verify every pin in the table above:

   ```sh
   for r in actions/checkout actions/cache actions/upload-artifact taiki-e/install-action Swatinem/rust-cache astral-sh/setup-uv; do
     tag=$(gh api "repos/$r/releases/latest" --jq .tag_name)
     gh api "repos/$r/git/ref/tags/$tag" --jq "\"$r $tag \" + .object.type + \" \" + .object.sha"
   done
   ```

   An object of type `tag` is annotated: dereference it with
   `gh api repos/<owner>/<repo>/git/tags/<sha> --jq .object.sha` and pin the commit it
   names. A tag that has moved past the table is used at its new commit with the new
   version in the comment, and the change is noted in the hand-back notes. Quote the
   `?ref=` paths in double quotes: zsh treats `?` as a glob.

3. Write `.github/actions/setup/action.yml`:

   ```yaml
   name: Set up the libpawdoku toolchain
   description: >-
     just and cargo-binstall from their release archives, the toolchain
     rust-toolchain.toml pins through the runner's rustup, the cargo caches, the
     pinned binaries in .tools/bin, uv with its cache, and the lockfile sync. Runs
     after the checkout. The allium binary is the documents job's own step.

   inputs:
     cache-key:
       description: >-
         Shared key for the cargo cache, one per job, so that an instrumented
         coverage build never restores over a plain one.
       required: true

   runs:
     using: composite
     steps:
       # The only two version pins in YAML, and both are bootstrap: just runs
       # every recipe, and cargo-binstall installs everything tools.txt names.
       # Neither can install itself. Restated from tools.txt's header comment.
       - uses: taiki-e/install-action@7623a79cdfecb99d681017af368ca353d9f49bb5 # v2.87.19
         with:
           tool: just@1.51.0,cargo-binstall@1.23.0
       # The runner image ships rustup, and rustup 1.28 or later reads
       # rust-toolchain.toml when given no toolchain, so the pin stays in one
       # file. dtolnay/rust-toolchain is not used: at a SHA it requires a
       # `toolchain` input, which would be a second copy of the pin.
       - run: just install-toolchain
         shell: bash
       # After the toolchain, because the key hashes the rustc version. Saved
       # from main only: a pull request restores main's cache and never writes
       # its own, so one branch's target/ cannot evict another's. cache-bin
       # governs ~/.cargo/bin, which the install step above refills in seconds
       # every run; caching it would only resurrect a stale bootstrap binary
       # after a pin bump. The tools in .tools/bin are actions/cache's below.
       - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2.9.2
         with:
           shared-key: ${{ inputs.cache-key }}
           save-if: ${{ github.ref == 'refs/heads/main' }}
           cache-bin: false
       # The pinned binaries, keyed on the pin list: a bumped line is a new key,
       # and cargo-binstall skips a matching version already present on a hit.
       - uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
         with:
           path: .tools
           key: ${{ runner.os }}-${{ runner.arch }}-tools-${{ hashFiles('tools.txt') }}
       # On the step that installs, never on the job: cargo-binstall reads
       # GITHUB_TOKEN to raise the GitHub API rate limit while it resolves
       # release archives, and no other step has a use for it. It is the run's
       # own token, minted and discarded with the run, so nothing is stored here.
       - run: just install-tools
         shell: bash
         env:
           GITHUB_TOKEN: ${{ github.token }}
       # The hook runner and the tooling package's checkers are Python
       # programs (decision 0004); uv runs them from uv.lock.
       - uses: astral-sh/setup-uv@c18668ad3cf93ea998bef934396af7bb5c839dc7 # v10.2.0
         with:
           version: 0.11.18
           enable-cache: true
           cache-dependency-glob: uv.lock
       - run: uv python install 3.14
         shell: bash
       # Exactly what the lockfiles say, nothing a lockfile cannot name.
       - run: just sync
         shell: bash
   ```

4. Write `.github/workflows/ci.yml`:

   ```yaml
   name: CI

   on:
     pull_request:
     push:
       # Quoted because a branch named `true`, `false`, `null`, `on`, or `1.0` is
       # valid to Git but is not a string to a YAML 1.1 parser.
       branches: ['main']
     workflow_dispatch:

   # Read-only: nothing here publishes, comments or writes a check of its own.
   # The token each step is minted with can do no more than clone this
   # repository, which is what a gate needs and all it needs.
   permissions:
     contents: read

   concurrency:
     group: ci-${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: true

   env:
     CARGO_TERM_COLOR: always
     # Incremental artefacts are large and never reused on a fresh runner.
     CARGO_INCREMENTAL: '0'
     CARGO_NET_RETRY: '10'
     RUST_BACKTRACE: '1'

   # Ubuntu only: the crate is pure computation with eol=lf forced by
   # .gitattributes and rustfmt, so a second operating system would prove
   # nothing the first does not. windows-latest joins `rust` the day
   # crates/pawdoku-cli exists; wheels get maturin's own matrix in the release
   # workflow S02 designs. `check`, last, is the one required check; the five
   # gate jobs are its `needs`, so no name carries a slash or a space.
   jobs:
     rust:
       name: rust
       runs-on: ubuntu-latest
       timeout-minutes: 20
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: rust
         # One recipe per step, in the gate's order, so a failure names itself
         # in the step list. `check-toolchain` first: it proves the runner's
         # cargo is rustup's proxy on the pinned release before anything builds.
         - run: just check-toolchain
         - run: just lock-check
         - run: just fmt-check
         - run: just toml-check
         - run: just clippy
         - run: just features
         - run: just test
         - run: just doc
         - run: just deps-unused

     coverage:
       name: coverage
       runs-on: ubuntu-latest
       timeout-minutes: 20
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: coverage
         - run: just coverage
         # Only after the floor passed: the recipe writes lcov.info in its last
         # line, so an upload on failure would fail a second time for the
         # missing file. Seven days is long enough to read a number into a
         # review; nothing downstream consumes it yet.
         - uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
           with:
             name: lcov
             path: target/llvm-cov/lcov.info
             if-no-files-found: error
             retention-days: 7

     wasm:
       name: wasm
       runs-on: ubuntu-latest
       timeout-minutes: 10
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: wasm
         - run: just wasm-check

     deny:
       name: deny
       runs-on: ubuntu-latest
       timeout-minutes: 10
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: deny
         # Licences, bans and sources are answerable from the registry index
         # `just sync` fetched. Advisories are audit.yml's, because they need the
         # RustSec database over the network.
         - run: just deny

     documents:
       name: documents
       runs-on: ubuntu-latest
       timeout-minutes: 15
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: documents
         # Over the network, into the gitignored .tools/bin. No lockfile can name
         # a binary, so `just sync` deliberately does not install it and this
         # does. Before `lint`, because the hook gate runs the specification
         # checks. It is not cached: the download is faster than a cache round
         # trip.
         - run: just install-allium
         - run: just lint
         - run: just check-docs
         - run: just check-agents
         # Already run by the hook gate above; repeated here so a specification
         # regression names itself in the step list rather than inside `lint`.
         # The same duplication as check-docs and check-agents, for the same
         # reason.
         - run: just check-specs
         - run: just analyse-specs

     # The one required check (CONVENTIONS.md §10). It needs every gate job and
     # fails if any of them failed, was cancelled or was skipped, so protection
     # names `check` alone and never changes when a job is added, renamed or
     # moved into a reusable workflow: a new gate job joins `needs` here.
     # `if: always()` is what makes it report on a failure instead of being
     # skipped, which is why the step reads `needs.*.result` itself. No checkout,
     # no setup action, and no `${{ }}` inside a shell line.
     check:
       name: check
       needs: [rust, coverage, wasm, deny, documents]
       if: always()
       runs-on: ubuntu-latest
       timeout-minutes: 5
       steps:
         - if: ${{ contains(needs.*.result, 'failure') || contains(needs.*.result, 'cancelled') || contains(needs.*.result, 'skipped') }}
           run: exit 1
   ```

5. Write `.github/workflows/audit.yml`:

   ```yaml
   name: Audit

   # Separate from CI because `cargo deny check advisories` fetches the RustSec
   # database, and a fetch can fail for reasons that have nothing to do with the
   # diff. So this is never a required check, and it also runs on a schedule:
   # an advisory published against an unchanged Cargo.lock is found by the
   # week's run, not by the next pull request.

   on:
     pull_request:
     schedule:
       # Mondays, 06:00 UTC.
       - cron: '0 6 * * 1'
     workflow_dispatch:

   permissions:
     contents: read

   concurrency:
     group: audit-${{ github.workflow }}-${{ github.ref }}
     cancel-in-progress: true

   env:
     CARGO_TERM_COLOR: always
     CARGO_INCREMENTAL: '0'
     CARGO_NET_RETRY: '10'

   jobs:
     audit:
       name: audit
       runs-on: ubuntu-latest
       timeout-minutes: 10
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: audit
         - run: just audit
   ```

6. `just lint`, then `just check`. actionlint runs over `.github/workflows/*.yml` only
   (G's `.pre-commit-config.yaml` line 160, carried by T00), and validates the `with:`
   keys each workflow passes to `./.github/actions/setup` against the inputs the action
   declares; `check-yaml` covers all three files' syntax. Both must be clean. Commit.

7. The proof run. Pushing is a separately authorised action (CONVENTIONS.md §11): stop
   and ask the maintainer to authorise `git push -u origin ticket/t04-ci`. Once pushed,
   `ci.yml` does not run on the push itself (its `push` trigger is `main` only), so the
   proof is a dispatch:

   ```sh
   gh workflow run ci.yml --ref ticket/t04-ci
   gh run list --workflow ci.yml --branch ticket/t04-ci --limit 1
   gh run watch <run-id> --exit-status
   gh run view <run-id> --json jobs --jq '.jobs[] | .name + " " + .conclusion + " " + .startedAt + " " + .completedAt'
   ```

   `ci.yml` exists on `main` from T00 and T01, so the dispatch is accepted and runs the
   branch's copy. The job list has six lines, `check` last and `success` only once the
   five before it are. `audit.yml` does not exist on `main` yet: try
   `gh workflow run audit.yml --ref ticket/t04-ci`; if GitHub refuses it, the proof is
   the `pull_request` trigger when the pull request is opened (also separately
   authorised). Record which happened. Quote the job list, each job's duration and the
   run's total minutes in the hand-back notes.

   Then dispatch `ci.yml` a second time and quote, from the `rust` job's setup log, the
   rust-cache restore line and the `actions/cache` line for `.tools`. The `.tools` cache
   hits, because `actions/cache` saves on any branch. rust-cache does not: `save-if` is
   `main` only, and the one entry `main` holds was saved by T00's stub job `check`, whose
   id rust-cache put in the key (`add-job-id-key` defaults to `true`), so `rust` on the
   branch cannot match it. Expect "no cache found" and say so; the first rust-cache hit
   is T11's to quote after merge.

8. CONVENTIONS.md §12, the two claims assigned to T04, plus the runner facts. Record each
   outcome in the hand-back notes:

   - `taiki-e/install-action` accepts `tool: just@1.51.0,cargo-binstall@1.23.0`: verified
     by manifest on 2026-09-23; confirm from the install step's log, which names each tool
     and version it installed.
   - `dtolnay/rust-toolchain` at a SHA with no `toolchain` input reads
     `rust-toolchain.toml`: false by source reading, as CONVENTIONS.md §10 and §12 now
     say. Re-read `action.yml` at `02cb101e` on the day and quote lines 9-11 and 36-39
     to confirm nothing changed; if the no-argument `rustup toolchain install` fails on
     the runner, hand back `rustup show` for the frozen `install-toolchain` recipe.
   - `Swatinem/rust-cache` with `cache-bin: false` leaves `.tools/` to `actions/cache`:
     confirm from rust-cache's log, which prints the paths it caches; neither `.tools` nor
     `~/.cargo/bin` may be among them.
   - The no-argument `rustup toolchain install` on the runner: quote `check-toolchain`'s
     `active-toolchain` line (`1.98.1-x86_64-unknown-linux-gnu`) and the `wasm` job's two
     green steps.

9. Confirm from
   `gh api repos/steven-cutting/libpawdoku/commits/<sha>/check-runs --jq '.check_runs[].name'`
   on the dispatched commit that the checks API reports `check` alongside `rust`,
   `coverage`, `wasm`, `deny` and `documents`, spelled exactly so, with no slash or
   prefix: `check` is the context T01's protection names, and this is the proof the new
   workflow still satisfies it. If any dispatch in this ticket had a gate job fail,
   quote `check`'s conclusion on that run (`failure`); otherwise record that the failing
   path was not exercised.

10. Set `status: done`, commit on the ticket branch with short imperative subjects, and
    stop before opening the pull request.

## Acceptance criteria

- `just lint` and `just check` are green in the worktree; actionlint reports nothing on
  either workflow and `check-yaml` nothing on the action.
- The three files match the texts in steps 3 to 5, and no file outside Files touched
  changed.
- Every `uses:` is `owner/repo@<40-hex-sha> # vX.Y.Z`, with the commit for annotated
  tags; `grep -rn 'uses:' .github | grep -v -E '@[0-9a-f]{40} # ' | grep -v './.github/actions/setup'`
  prints nothing.
- No job name contains a slash, a space or a prefix; the six names are `rust`,
  `coverage`, `wasm`, `deny`, `documents`, `check`, and `audit.yml`'s is `audit`.
- `check` has `needs: [rust, coverage, wasm, deny, documents]`, `if: always()`, no
  checkout and no setup action, and its one step exits 1 when any `needs.*.result` is
  `failure`, `cancelled` or `skipped`.
- The `documents` job runs `just install-allium` before `just lint` and therefore before
  `check-specs` and `analyse-specs`.
- The `GITHUB_TOKEN` environment variable appears on the `install-tools` step and nowhere
  else; `permissions` is `contents: read` in both workflows.
- All six jobs of `ci.yml` are green on the dispatch run on `ticket/t04-ci`; the job
  list, per-job durations and the total minutes are quoted in the hand-back notes; the
  check-runs API lists `check` for the dispatched commit.
- On the second dispatch, the `.tools` cache line reports a hit; the rust-cache line is
  quoted as observed and explained.
- The `lcov` artifact exists on the run with a 7-day retention.
- The three §12 outcomes and the runner facts of step 8 are recorded in "What was
  verified, and how".

## Verification

```sh
just lint
just check
git status --porcelain
grep -rn 'uses:' .github | grep -v -E '@[0-9a-f]{40} # ' | grep -v './.github/actions/setup' || echo "every remote action pinned"
grep -n 'GITHUB_TOKEN:' .github/actions/setup/action.yml .github/workflows/*.yml
grep -n -E '^    name: ' .github/workflows/ci.yml
gh run view <run-id> --json jobs --jq '.jobs[] | .name + " " + .conclusion'
gh api repos/steven-cutting/libpawdoku/actions/runs/<run-id>/artifacts --jq '.artifacts[] | .name + " expires " + .expires_at'
```

Expected: `just lint` and `just check` exit 0 and the worktree is clean; the pin grep
prints `every remote action pinned`; `GITHUB_TOKEN:` matches one line, the `env` key of
the action's `install-tools` step; the `name:` grep, anchored at job depth, prints the
six job names, `check` last, and nothing else (the artifact's `name: lcov` sits deeper);
every job's conclusion is `success`; the artifact is `lcov` with an expiry seven days
after the run. Quote each in the hand-back notes, with the run's URL.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether `rustup toolchain install` with no arguments installs the toolchain, components
  and both targets on the runner's rustup (1.29.1 on the image verified 2026-09-23), so
  that the `Justfile`'s `install-toolchain` recipe holds in CI as it did for T00 locally.
  If it does not, the fix is `rustup show` or the dtolnay fallback with the pin restated;
  either is a `main` follow-up because the `Justfile` is frozen and the pin must stay in
  one file.
- Whether `coverage` should upload `lcov.info` to Codecov or a similar service later. Not
  now: nothing external, no token, no comment bot; the artifact is enough to read a number
  into a review. Reopened when a badge or a per-pull-request delta is wanted.
- Whether the weekly `audit` run should open an issue on failure. Not now: it would need
  `issues: write` and a deduplication step, and a failed scheduled run already emails the
  maintainer. Reopened if a failure goes unnoticed for a week.
