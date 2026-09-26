---
id: T04
title: "CI workflows: the composite setup action, five gate jobs and the audit job"
status: done
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
for the design this ticket embeds, §11 for the lane rules, §12 for the claims assigned
here, §13 for the action-pin risk), then `README.md` in this directory, then T00's hand-back
notes for the stub as it landed and D02's hand-back notes ("Files T00, T03 and T04
create", the composite action).

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
  as its own step. The action's lines 46-50 are B's setup-uv step, which this action
  does not carry (D02), and lines 51-56 the rule that an input reaches a shell line
  through `env`.
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
- `prefix-dev/setup-pixi` v0.10.2 (2026-08-28) installs the pixi named by
  `pixi-version`, installs the environment from `manifest-path` (`locked: true` refuses
  a lockfile that disagrees with the manifest), caches `.pixi/envs` and the package cache
  keyed on the `pixi.lock` hash with `cache-write` gating the save, and with
  `activate-environment: true` puts the environment's `bin` on every later step's `PATH`
  (D02, 2026-09-24). Because `just` is in that environment, no other action installs it.
- `Swatinem/rust-cache` v2.9.2 declares `cache-bin` and `save-if`, both defaulting to
  `true`. `cache-bin` governs `~/.cargo/bin` only; the action never touches `.tools/` or
  `.pixi/`.

Action pins, resolved read-only with `gh api repos/<owner>/<repo>/releases/latest` and
`git/ref/tags/<tag>` on 2026-09-23 (`setup-pixi`'s commit is resolved the same way in
step 2); re-verified on the day, a moved tag recorded:

| Action | Tag | Commit | Note |
| --- | --- | --- | --- |
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` | as B and S |
| `actions/cache` | v6.1.0 | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` | B carries v4.3.0; same inputs |
| `actions/upload-artifact` | v7.0.1 | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` | S carries v5.0.0 |
| `prefix-dev/setup-pixi` | v0.10.2 | `d3f436a425481402e6a95a1d1fc10331c708cd9e` | D02; `pixi-version` must satisfy the manifest's `requires-pixi` |
| `Swatinem/rust-cache` | v2.9.2 | `6323deb102c322ba6fcbdcafc7e3dddab59af2b6` | annotated tag `63fed3e2` dereferenced; pin the commit, never the tag object |
| `dtolnay/rust-toolchain` | `v1` = `master` | `02cb101ec7c40f2c49e1d9714d64511d8e1b74de` | fallback only, not used; `stable` head `6bed0761d98439e5a578e2877258200ad565ba87` |

## Goal

Three files on `ticket/t04-ci`: the finished composite action and `ci.yml` (replacing
T00's stubs) and a new `audit.yml`; `just lint` green (actionlint over the workflows,
`check-yaml` over all three); one `workflow_dispatch` run of `ci.yml` on the ticket branch
with all six jobs green and their durations quoted, `check` among the names the checks
API reports for the commit; and the CONVENTIONS.md §12 claims assigned to this ticket
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
   for r in actions/checkout actions/cache actions/upload-artifact prefix-dev/setup-pixi Swatinem/rust-cache; do
     tag=$(gh api "repos/$r/releases/latest" --jq .tag_name)
     gh api "repos/$r/git/ref/tags/$tag" --jq "\"$r $tag \" + .object.type + \" \" + .object.sha"
   done
   ```

   An object of type `tag` is annotated: dereference it with
   `gh api repos/<owner>/<repo>/git/tags/<sha> --jq .object.sha` and pin the commit it
   names. A tag that has moved past the table is used at its new commit with the new
   version in the comment, and the change is noted in the hand-back notes. Quote the
   `?ref=` paths in double quotes: zsh treats `?` as a glob. `prefix-dev/setup-pixi` has
   no commit in the table: write the one this step resolves into the action in place of
   `<setup-pixi-sha>` below, and into the table with the hand-back notes.

3. Write `.github/actions/setup/action.yml`, the steps D02 orders:

   ```yaml
   name: Set up the libpawdoku toolchain
   description: >-
     pixi and the environment pixi.lock pins (every tool binary, Python, prek and
     the tooling package's checkers), the toolchain rust-toolchain.toml pins
     through the runner's rustup, the cargo caches, the binaries in .tools/bin
     that conda-forge lacks, and the lockfile sync. Runs after the checkout. The
     allium binary is the documents job's own step.

   inputs:
     cache-key:
       description: >-
         Shared key for the cargo cache, one per job, so that an instrumented
         coverage build never restores over a plain one.
       required: true

   runs:
     using: composite
     steps:
       # The one version pin in YAML is pixi itself, which nothing else can
       # install. It must satisfy the `requires-pixi` floor in pyproject.toml,
       # so the version appears twice, exact here and as a floor there, rather
       # than pretend one owner. `locked` refuses a lockfile that disagrees with
       # the manifest, which makes this step the CI lockfile check. Activation
       # puts .pixi/envs/default/bin, and so `just`, on every later step's PATH.
       # The environment holds no rust, so cargo stays the runner's rustup
       # proxy. The cache is keyed on pixi.lock's hash and written from main
       # only, for the reason rust-cache's save-if gives below.
       - uses: prefix-dev/setup-pixi@<setup-pixi-sha> # v0.10.2
         with:
           pixi-version: v0.81.0
           manifest-path: pyproject.toml
           locked: true
           cache: true
           cache-write: ${{ github.ref == 'refs/heads/main' }}
           activate-environment: true
       # The runner image ships rustup, and rustup 1.28 or later reads
       # rust-toolchain.toml when given no toolchain, so the pin stays in one
       # file. dtolnay/rust-toolchain is not used: at a SHA it requires a
       # `toolchain` input, which would be a second copy of the pin.
       - run: just install-toolchain
         shell: bash
       # After the toolchain, because the key hashes the rustc version. Saved
       # from main only: a pull request restores main's cache and never writes
       # its own, so one branch's target/ cannot evict another's. cache-bin
       # governs ~/.cargo/bin, which holds nothing this gate installs: every
       # tool is in the pixi environment setup-pixi restored above, or in
       # .tools/bin, which actions/cache restores below.
       - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2.9.2
         with:
           shared-key: ${{ inputs.cache-key }}
           save-if: ${{ github.ref == 'refs/heads/main' }}
           cache-bin: false
       # .tools holds what pixi cannot: cargo-hack from tools.txt, and allium as
       # a side effect of the documents job (bg-install-allium version-checks,
       # so a stale one is replaced). Keyed on the pin list: a bumped line is a
       # new key, and cargo-binstall skips a matching version already present
       # on a hit.
       - uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
         with:
           path: .tools
           key: ${{ runner.os }}-${{ runner.arch }}-tools-${{ hashFiles('tools.txt') }}
       # On the step that installs, never on the job: cargo-binstall (the
       # environment's) reads GITHUB_TOKEN to raise the GitHub API rate limit
       # while it resolves the release archive, and no other step has a use for
       # it. It is the run's own token, minted and discarded with the run, so
       # nothing is stored here.
       - run: just install-tools
         shell: bash
         env:
           GITHUB_TOKEN: ${{ github.token }}
       # Exactly what the lockfiles say: Cargo.lock fetched, pixi.lock
       # installed, neither rewritten.
       - run: just sync
         shell: bash
       # prek's cache outside the workspace, so check-clean's snapshot never
       # sees it. PREK_HOME is the variable T03 confirms on prek 0.5.3.
       - run: echo "PREK_HOME=${{ runner.temp }}/prek" >> "$GITHUB_ENV"
         shell: bash
       # The seven hook clones, the Go, Node and rustup toolchains and the
       # ripsecrets build, which every gate job would otherwise fetch again.
       # Keyed on both hook configs and on pixi.lock, which pins prek; saved on
       # any branch, like .tools.
       - uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
         with:
           path: ${{ runner.temp }}/prek
           key: ${{ runner.os }}-${{ runner.arch }}-prek-${{ hashFiles('.pre-commit-config.yaml', '.pre-commit-fix.yaml', 'pixi.lock') }}
       # The prepare and the lychee warm are the point; the shim it writes into
       # the runner's .git is harmless. After this step no step reaches the
       # network.
       - run: just install-hooks
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
         # Over the network, into the gitignored .tools/bin. The checker is
         # pinned by the tooling package, not by pixi.lock, so `just sync`
         # deliberately does not install it and this does. Before `lint`,
         # because the hook gate runs the specification checks. It lands in the
         # .tools cache as a side effect; bg-install-allium version-checks and
         # downloads only when the binary is missing or stale.
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
   `setup-pixi` cache line, the rust-cache restore line and the `actions/cache` lines
   for `.tools` and for prek's cache. Both `actions/cache` entries hit, because
   `actions/cache` saves on any branch, and the `install-hooks` step then prints no
   clone, download or build line. The
   `setup-pixi` cache is keyed on the `pixi.lock` hash and written from `main` only, so
   it hits only if `main`'s stub run saved an entry for the same lockfile; quote the line
   as observed and say which it was. rust-cache does not hit: `save-if` is `main` only,
   and the one entry `main` holds was saved by T00's stub job `check`, whose id
   rust-cache put in the key (`add-job-id-key` defaults to `true`), so `rust` on the
   branch cannot match it. Expect "no cache found" and say so; the first rust-cache hit
   is T11's to quote after merge.

8. CONVENTIONS.md §12, the claims assigned to T04, plus the runner facts. Record each
   outcome in the hand-back notes:

   - The activation variables `setup-pixi` exports (`CONDA_PREFIX`, `PIXI_*`) are inert
     for cargo: the `rust` job's build log shows no build script reacting to them and
     `check-toolchain`'s `active-toolchain` line is rustup's. Quote `env | grep -E
     '^(CONDA_PREFIX|PIXI_)'` from a temporary step after the setup action on the
     branch, removed before the final commit. If a build script ever
     reacts to them, the fallback is `activate-environment: false` and a step that
     appends `.pixi/envs/default/bin` to `$GITHUB_PATH`; record which was needed.
   - `setup-pixi` restores `.pixi/envs` and the package cache keyed on the `pixi.lock`
     hash, and `cache-write` gated to `main` behaves like rust-cache's `save-if`: from
     the first dispatch's log, quote the cache key it computed and whether it saved (it
     must not, on the branch); from the second, the restore line of step 7.
   - `dtolnay/rust-toolchain` at a SHA with no `toolchain` input reads
     `rust-toolchain.toml`: false by source reading, as CONVENTIONS.md §10 and §12 now
     say. Re-read `action.yml` at `02cb101e` on the day and quote lines 9-11 and 36-39
     to confirm nothing changed; if the no-argument `rustup toolchain install` fails on
     the runner, hand back `rustup show` for the frozen `install-toolchain` recipe.
   - `Swatinem/rust-cache` with `cache-bin: false` leaves `.tools/` to `actions/cache`:
     confirm from rust-cache's log, which prints the paths it caches; none of `.tools`,
     `.pixi` and `~/.cargo/bin` may be among them.
   - The no-argument `rustup toolchain install` on the runner: quote `check-toolchain`'s
     `active-toolchain` line (`1.98.1-x86_64-unknown-linux-gnu`) and the `wasm` job's two
     green steps.
   - The prek cache under `${{ runner.temp }}/prek` restores on the second dispatch, and
     `just lint`'s log then shows no clone, download or build line: quote the cache
     restore line and the `install-hooks` step's output from the second dispatch, and
     the first dispatch's `install-hooks` duration against the second's.

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
- On the second dispatch, the `.tools` and prek cache lines report a hit; the
  `setup-pixi` and rust-cache lines are quoted as observed and explained.
- The `lcov` artifact exists on the run with a 7-day retention.
- The six §12 outcomes and the runner facts of step 8 are recorded in "What was
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

Executed on 2026-09-25 (runs dated 2026-09-26 UTC) on the maintainer's machine (Apple
silicon) in the Supacode worktree for this ticket, on branch `T04-ci` (see Deviations).

**Step 1.** The worktree was fresh at `main`'s tip `52b8215`, with no `.pixi/`. With the
maintainer's authorisation, `pixi install --locked`, `just install-tools` and
`just install-allium` ran (cargo-hack 0.6.45 and allium 3.6.1 were already in `.tools/bin`).
`just install-hooks` was not run, because in a secondary worktree it would repoint the
shared pre-commit shim. The baseline `just check` on the untouched tree exited 0: "All
checks passed and the worktree is unchanged."

**Step 2, the pins.** All five `releases/latest` lookups matched the table; no tag has
moved:

```text
actions/checkout v7.0.1 commit 3d3c42e5aac5ba805825da76410c181273ba90b1
actions/cache v6.1.0 commit 55cc8345863c7cc4c66a329aec7e433d2d1c52a9
actions/upload-artifact v7.0.1 commit 043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
prefix-dev/setup-pixi v0.10.2 commit d3f436a425481402e6a95a1d1fc10331c708cd9e
Swatinem/rust-cache v2.9.2 tag 63fed3e2fecf6f7b51dc6f043341b79ef82a9ae7
```

`git/tags/63fed3e2…` dereferences to `commit 6323deb102c322ba6fcbdcafc7e3dddab59af2b6`.
T00's stub had pinned rust-cache to the tag object `63fed3e2`, so this ticket moves the pin
to the commit, as the table requires. `setup-pixi`'s commit `d3f436a4…` is written into
the action and the table. It is the same commit T00 found on 2026-09-24.

**Step 6.** The three files are the texts of steps 3 to 5, extracted from this ticket with
only the setup-pixi SHA substituted. `just lint` exited 0, including "Lint GitHub Actions
workflow files…Passed" (actionlint) and "check yaml…Passed". `just check` then exited 0
with "All checks passed and the worktree is unchanged." The verification greps printed:

```text
every remote action pinned
.github/actions/setup/action.yml:70:        GITHUB_TOKEN: ${{ github.token }}
36:    name: rust
60:    name: coverage
83:    name: wasm
96:    name: deny
112:    name: documents
147:    name: check
```

`audit.yml`'s one job is `30:    name: audit`.

**Step 7, the proof runs.** The branch was pushed and dispatched twice. The first
dispatch ran on `606f809`, which carried the temporary probe. The second ran on
`0e0661e`, which removes the probe. Its tree is identical to `04e3195`, the commit that
wrote the three files (`git diff 04e3195 0e0661e` is empty).

- Dispatch 1, <https://github.com/steven-cutting/libpawdoku/actions/runs/36213713627>,
  `success`, 65 s wall-clock:

  ```text
  rust success 03:05:26Z 03:06:21Z       55 s
  coverage success 03:05:26Z 03:06:13Z   47 s
  deny success 03:05:26Z 03:05:59Z       33 s
  wasm success 03:05:31Z 03:06:13Z       42 s
  documents success 03:05:26Z 03:06:07Z  41 s
  check success 03:06:24Z 03:06:27Z       3 s
  ```

- Dispatch 2, <https://github.com/steven-cutting/libpawdoku/actions/runs/36213824706>,
  `success`, 66 s wall-clock:

  ```text
  rust success 03:07:34Z 03:08:25Z       51 s
  documents success 03:07:34Z 03:08:16Z  42 s
  coverage success 03:07:34Z 03:08:29Z   55 s
  deny success 03:07:34Z 03:08:05Z       31 s
  wasm success 03:07:34Z 03:08:08Z       34 s
  check success 03:08:32Z 03:08:36Z       4 s
  ```

The jobs added up to 221 s of runner time on dispatch 1 and 217 s on dispatch 2, so about
3.7 runner-minutes each. The timing API reports `billable … total_ms: 0` because the
repository is public.

`check` is last in both runs and started only after the five gate jobs finished.

`gh workflow run audit.yml --ref T04-ci` was refused with "HTTP 404: workflow audit.yml not
found on the default branch". A raw `POST …/actions/workflows/audit.yml/dispatches` with
`ref=T04-ci` was refused the same way, and `actions/workflows` lists only `ci.yml`. So the
audit proof is the `pull_request` trigger when the pull request is opened. That is a
separately authorised action and has not been taken.

Caches in the `rust` job on both dispatches. The lines were the same on each, because
`main`'s stub runs had already saved the `.tools`, prek and pixi entries:

```text
Cache hit for: pixi-linux-64-946edba90c2ef653aa40a82f44643bd0ab60248b8a93e82211bce42138be3065
Restored cache with key `pixi-linux-64-946edba90c2ef653aa40a82f44643bd0ab60248b8a93e82211bce42138be3065`
Cache Key:
    v0-rust-rust-Linux-x64-492f33ac-… (hash suffix elided: typos reads it as a word)
No cache found.
Cache hit for: Linux-X64-tools-c06ff119a141184425fb9cf4832314239aac4fdb75885f18f320e468892259d0
Cache restored from key: Linux-X64-tools-c06ff119a141184425fb9cf4832314239aac4fdb75885f18f320e468892259d0
Cache hit for: Linux-X64-prek-204cc165e714bd039d9316669a99bb2b337da0be6df7b11fa52859bb1e8c9d20
Cache restored from key: Linux-X64-prek-204cc165e714bd039d9316669a99bb2b337da0be6df7b11fa52859bb1e8c9d20
```

- **setup-pixi** hit on both dispatches. The ticket anticipated this: `main`'s stub run
  had saved an entry for the same `pixi.lock`.
- **rust-cache** found no cache, as the ticket predicted, because the key prefix
  `v0-rust-rust-…` is new and `main` holds only the stub job's `check` entry. The first
  rust-cache hit is T11's to quote.
- **`.tools` and prek** hit on both dispatches, not only the second, for the same reason.
  Both post steps printed "Cache hit occurred on the primary key …, not saving cache."

**Step 8, the CONVENTIONS.md §12 claims.**

- **Activation variables are inert for cargo: holds.** The probe
  `env | grep -E '^(CONDA_PREFIX|PIXI_)' || true` ran after the setup action in dispatch
  1's `rust` job and printed:

  ```text
  PIXI_PROMPT=(libpawdoku-tooling)
  CONDA_PREFIX=/home/runner/work/libpawdoku/libpawdoku/.pixi/envs/default
  PIXI_PROJECT_MANIFEST=/home/runner/work/libpawdoku/libpawdoku/pyproject.toml
  PIXI_PROJECT_NAME=libpawdoku-tooling
  PIXI_ENVIRONMENT_NAME=default
  PIXI_IN_SHELL=1
  PIXI_EXE=/home/runner/.pixi/bin/pixi
  PIXI_PROJECT_VERSION=0.1.0
  PIXI_PROJECT_ROOT=/home/runner/work/libpawdoku/libpawdoku
  PIXI_ENVIRONMENT_PLATFORMS=linux-64,osx-arm64
  ```

  setup-pixi also exports `CONDA_SHLVL=1` and `CONDA_DEFAULT_ENV=libpawdoku-tooling`,
  which the probe's pattern does not match. The build compiled every dependency from
  scratch, because rust-cache missed. That included the crates with build scripts
  (among them `proc-macro2`, `libc`, `num-traits` and `getrandom`). It produced no warning, and
  `clippy`, `test` and `doc` passed. `check-toolchain` passed its `case` line, so cargo
  was rustup's proxy, and printed
  `1.98.1-x86_64-unknown-linux-gnu (overridden by '/home/runner/work/libpawdoku/libpawdoku/rust-toolchain.toml')`.
  `activate-environment: true` is kept, and the `$GITHUB_PATH` fallback was not needed.
  The probe was removed by commit `0e0661e`, not by rewriting history.
- **setup-pixi's cache is keyed on `pixi.lock` and not written off `main`: holds.** The
  key is `pixi-linux-64-946edba9…`, and it was restored on both dispatches. The
  `prefix-dev/setup-pixi` post step printed nothing and saved nothing on the branch, the
  same silence rust-cache's post step shows under `save-if: false`.
- **dtolnay/rust-toolchain reads `rust-toolchain.toml`: false, unchanged.** `master` is
  still `02cb101ec7c40f2c49e1d9714d64511d8e1b74de`. Its `action.yml` lines 9-11 read
  `toolchain:` / `description: Rust toolchain specification -- …` / `required: true`, and
  lines 36-39 read `if [[ -z $toolchain ]]; then` / ``# GitHub does not enforce `required: true` inputs
  itself. …`` / `echo "'toolchain' is a required input" >&2` / `exit 1`.
- **rust-cache with `cache-bin: false` leaves `.tools/` alone: holds.** Its "Cache
  Configuration" lists exactly `/home/runner/.cargo/registry`, `/home/runner/.cargo/git`
  and `/home/runner/work/libpawdoku/libpawdoku/target`. It does not list `.tools`, `.pixi`
  or `~/.cargo/bin`.
- **The no-argument `rustup toolchain install` works on the runner: holds.** It printed
  `info: syncing channel updates for 1.98.1-x86_64-unknown-linux-gnu`, then `info:
  downloading 9 components`, then ``info: the active toolchain
  `1.98.1-x86_64-unknown-linux-gnu` has been installed``. The `wasm` job's two steps were
  green on both dispatches, `cargo hack check … --target wasm32-unknown-unknown` and
  `… --target wasm32v1-none`, each finishing both powerset configurations. No
  `rustup show` follow-up is needed. The runner image has moved since 2026-09-23: the
  jobs ran on `ubuntu-24.04` version `20260920.314.1`, not `20260907.300.1`.
- **The prek cache restores and `lint` fetches nothing: holds.** On dispatch 2 the cache
  hit on `Linux-X64-prek-204cc165…`. The `install-hooks` step printed only its commands,
  ``Installed Git hook at `.git/hooks/pre-commit` ``, and
  `lychee…Passed`, with no clone, download or build line. The same is true on dispatch 1,
  because the cache already hit there. The `just lint` step in `documents` shows no clone,
  download, install, build or fetch line on either dispatch. Durations of `install-hooks`
  per job:

  | Job | Dispatch 1 | Dispatch 2 |
  | --- | --- | --- |
  | `rust` | 2634 ms | 664 ms |
  | `coverage` | 341 ms | 427 ms |
  | `deny` | 2001 ms | 339 ms |
  | `wasm` | 428 ms | 665 ms |
  | `documents` | 518 ms | 405 ms |

  All ten are cache hits. The two slow first-dispatch values are noise, not a miss.
- **The `check` aggregate: green half proved, red half not exercised.** No gate job failed
  in either dispatch. The check-runs API for the dispatched commit `0e0661e2…` lists:

  ```text
  check success
  wasm success
  deny success
  coverage success
  documents success
  rust success
  ```

  These are the six names, with no slash or prefix.

**Artifacts.** `lcov` was uploaded by both dispatches with `retention-days: 7`:
`lcov expires 2026-10-03T03:06:09Z` (run 36213713627) and `lcov expires
2026-10-03T03:08:24Z` (run 36213824706). Each expires seven days after its run.

**Other checks.** In the `documents` job, `just install-allium` ran before `just lint`. It
printed "allium 3.6.1 is already installed", from the `.tools` cache. `check-specs` and
`analyse-specs` each reported "7 specifications, no diagnostics and no findings".

### Deviations, and why

- **The branch is `T04-ci`, not `ticket/t04-ci`.** It keeps the Supacode name because the
  maintainer chose "keep and record", as for T01. Every `--ref` and `--branch` above uses
  `T04-ci`.
- **Two extra commits on the branch.** `606f809` adds the step 8 probe and `0e0661e`
  removes it. Both were pushed so the probe could run. They cancel out, and
  `git diff 04e3195 0e0661e` is empty.
- **The rust-cache SHA differs from the stub, not from this ticket.** It moves from the
  tag object `63fed3e2` to the commit `6323deb1`.

### Handed back

- **`audit.yml`'s first run.** It waits for the pull request's `pull_request` trigger,
  because GitHub will not dispatch a workflow that is absent from `main`. Opening the pull
  request is the maintainer's call.
- **No `main` follow-ups.** `install-toolchain` works on the runner as written.

### Open points settled

- The first open point, whether the no-argument `rustup toolchain install` installs the
  toolchain, the components and both targets on the runner's rustup: **yes.** Nine
  components were downloaded, `check-toolchain` reported 1.98.1 from
  `rust-toolchain.toml`, and both wasm targets compiled. No fallback is needed.
- The other two open points, Codecov and an issue on a failed audit, stay open as the
  ticket words them.

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
