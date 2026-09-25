---
id: C01
title: "Reusable rust-ci.yml and a setup-rust-toolchain action in biscuit_games_tooling"
status: open
depends_on: [T04, T11]
parallel_with: []
branch: ticket/c01-reusable-rust-ci
estimated_size: M
---

# C01: Reusable rust-ci.yml and a setup-rust-toolchain action in biscuit_games_tooling

## Context

The games do not carry their CI jobs. G's `.github/workflows/ci.yml` at `78d03cdf` is
28 lines: triggers, `permissions`, `concurrency`, and one job `ci` whose `uses:` names
B's reusable workflow at a commit with the tag as a comment (line 28). The jobs live in
B, `.github/workflows/game-ci.yml` at `6c5c07f6` (`on: workflow_call`, line 7), and
each begins with `actions/checkout` and B's composite action `actions/setup-toolchain`
named by commit (lines 17, 39, 71). B's `README.md` states the pattern (lines 3-9: a
fix to a job is a release of B and a moved pin, not the same edit in every game), the
check naming (lines 129-131: a check from a called workflow is named after the calling
job then the called job, `ci / frontend`), the constraint that a reusable workflow runs
only from `.github/workflows/` at the root of the repository hosting it (lines 21-22),
and the two-commit rule for the action (lines 133-144): a `uses:` must name a commit
that already exists, so every change to the action is the action alone, then the
workflows moving their pin. B's own history is the worked example: `73df4e2` "Add the
composite action that sets up the game toolchain", then `be41556` "Add the reusable
game workflows, this repository's gate and its docs", which is `v0.1.0`. B is public
(line 24), so any repository calls it whatever T01 chose for this one.

This repository carries its five jobs and its composite action locally (CONVENTIONS.md
§10) because the jobs did not exist until T04 wrote them and nothing proved them until
T11 ran them from a fresh clone and on `main`. The house rule that a shared thing is
hosted once applies as soon as there is a second Rust repository to share with; a
second engine, or B itself gaining Rust, would be that. Until then, moving five jobs
into B buys a second repository to release for every CI fix and nothing else.

What moves if it moves: the five jobs become `B/.github/workflows/rust-ci.yml` under
`on: workflow_call` (the allium install for `documents` behind a `with:` boolean; the
`cache-key` per job internal; the coverage floor is the caller's `Justfile`'s); the
composite action becomes `B/actions/setup-rust-toolchain/action.yml` with inputs
`pixi-version` and `manifest-path` defaulting to CONVENTIONS.md §2's pins (decision
0011: pixi owns the tools and the Python side alike, so there is no separate `python`
input; a consumer supplies its own manifest). Triggers, `permissions` and `concurrency`
stay in the caller (B README lines 31-34). The caller becomes G's 28-line shape plus the
`check` aggregate job, which now needs the one call job instead of five local ones
(CONVENTIONS.md §10): the called jobs report as `ci / rust`, `ci / coverage`,
`ci / wasm`, `ci / deny`, `ci / documents`, but `check` stays the only required context,
so branch protection is not touched. B's versioning (README lines
250-291): a new workflow and action no existing caller must change is MINOR; the next
tag after `v0.3.0` is `v0.4.0` unless C02 ships first. B's `check` job runs the action
from its checkout as a smoke test (B `ci.yml` line 30); B has no Rust code, so a Rust
action needs a fixture or a job that only proves the toolchain installs.

Read first: CONVENTIONS.md §10, §11 (every edit to B is separately authorised), §12
(the two action claims T04 settled); B's `README.md` lines 1-35, 129-144 and 250-291;
B's `game-ci.yml` and `actions/setup-toolchain/action.yml`; this repository's
`ci.yml`, `audit.yml` and `.github/actions/setup/action.yml`; T04's and T11's
hand-back notes.

## Goal

A recommendation, and the ticket filed so it can be taken up the day the trigger
arrives. The starting recommendation: **file, do not apply**. The jobs are proven and
portable and this ticket records how they move, but the move is made only once a
second Rust repository exists to call `rust-ci.yml`. If the maintainer applies it now
anyway, the Steps are the work.

| Option | Cost now | Cost of a CI fix later | Coupling | Verdict |
| --- | --- | --- | --- | --- |
| Keep local | none | one pull request here | none | now |
| Move to B now | a B MINOR release, two commits, a caller rewrite, a protection round | a B release plus a pin bump here, for one consumer | this repository's CI depends on B's tags | not yet |
| Move when a second Rust repository exists | the same, amortised over two callers | a B release plus a pin bump in each | the same, justified | the trigger |

## Non-goals

- Editing B, this repository's workflows or branch protection before the trigger and
  before authorisation.
- Moving the checkers, the Justfile, `tools.txt` or the hooks into B (C02 owns the
  package; the rest is this repository's).
- A reusable `release.yml` (S02) or workflows for the bindings crates (S04).

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/C01-reusable-rust-ci.md` | ticket | the recommendation and the trigger; `status: done` when filed, or when applied |
| `B/.github/workflows/rust-ci.yml` | other repository, new | the five jobs under `workflow_call`; authorisation required |
| `B/actions/setup-rust-toolchain/action.yml` | other repository, new | the composite action with version inputs; authorisation required |
| `B/README.md`, `B/CHANGELOG.md` | other repository | a "What it hosts" row, a Rust "Calling it" block, the MINOR entry; authorisation required |
| `.github/workflows/ci.yml`, `.github/actions/setup/action.yml` | T04's files | the thin caller; the action deleted; one pull request here once B has tagged |
| repository settings | settings | none: `check` stays the one required context; the caller's aggregate job keeps reporting it |
| `docs/reference/quality-gates.md`, `docs/explanation/security-model.md` | T08's pages | the check names and where the jobs live; the same pull request |

## Steps

1. Create the worktree on `ticket/c01-reusable-rust-ci` from `main` after T04 and T11
   have merged (README.md "How to pick up a ticket"). Answer the trigger: is there a
   second repository that would call `rust-ci.yml`? If not, do steps 2 and 3, record
   "filed, not applied", and go to step 7.

2. Diff this repository's action against B's for what a generalised action needs:

   ```sh
   git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:actions/setup-toolchain/action.yml
   cat .github/actions/setup/action.yml
   ```

   Record the inputs (`cache-key`, and the pixi version and manifest path that become
   inputs with defaults), the `setup-pixi` step (this repository's one version pin in
   YAML) and what B's action has where it stands, the step that stays with the caller's
   job or behind a boolean (`just install-allium`), and every `uses:` SHA with its
   comment.

3. Write in the hand-back notes the exact `rust-ci.yml` (the five jobs from `ci.yml`
   with the action's `uses:` changed to
   `steven-cutting/biscuit_games_tooling/actions/setup-rust-toolchain@<sha> # vX.Y.Z`),
   the caller (`ci.yml` reduced to G's shape naming
   `steven-cutting/biscuit_games_tooling/.github/workflows/rust-ci.yml@<sha> # vX.Y.Z`),
   and the `bootstrap_repo.sh` command with the five `ci / <job>` names.

4. **Authorisation required.** In B, on a branch: commit 1 adds the action alone;
   commit 2 adds `rust-ci.yml` pinning the action at commit 1's SHA, with the README
   and CHANGELOG changes. B's `check` green; pull request, merge, annotated MINOR tag.

5. **Authorisation required.** Here, on this branch: `ci.yml` becomes the thin caller
   at the tagged SHA (`gh api repos/steven-cutting/biscuit_games_tooling/commits/vX.Y.Z --jq .sha`,
   B README line 261); `.github/actions/setup/` is deleted; the two T08 pages updated.
   The caller keeps T04's `check` job with `needs: [ci]` (the call job's id), the same
   `if: always()` and the same result test, so the pull request's run shows five checks
   named `ci / rust` and so on plus `check`, and `check` is green only when the five are.

6. Protection, read-only. Confirm before merging that `check` is still the one required
   context and that the pull request's run reports it:

   ```sh
   gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.checks[].context] | sort | join(",")'
   gh api repos/steven-cutting/libpawdoku/commits/<head sha>/check-runs --jq '.check_runs[].name'
   ```

   `check` on the first line; `check` and the five `ci / <job>` names on the second. No
   protection change: the aggregate job is what makes the rename invisible to `main`.

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- The trigger is answered and the verdict follows it.
- The hand-back notes hold the full `rust-ci.yml`, the action with its inputs and the
  caller with its `check` aggregate, applied or not.
- If applied: B has a MINOR tag whose commit holds both files; this repository's
  `ci.yml` is the thin caller at that SHA; `.github/actions/setup/` is gone; `main`
  still requires `check` alone and the pull request's run reported it green alongside
  the five `ci / <job>` checks; `just check` is green here and B's `check` is green
  there.
- Every action on B and on repository settings was authorised before it was taken.
- `git status --porcelain` on this branch lists only the files in Files touched.

## Verification

```sh
git -C /Users/scutting/projects/biscuit_games_tooling log --oneline -2 be41556
git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:README.md | sed -n 133,144p
grep -n 'uses:' .github/workflows/ci.yml
gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.checks[].context] | sort | join(",")'
```

Expected: the two commits of the pin dance (`73df4e2` then `be41556`); B's "Changing
the action" section; the local action and five jobs (filed) or one `uses:` naming B
(applied); `check` on the protection line either way, and the check-runs list naming
`check` with the five plain job names (filed) or with the five `ci / <job>` names
(applied).

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- How B smoke-tests a Rust action with no Rust code: a fixture crate under B's
  `tests/`, a job that runs the action and `cargo --version`, or the first caller's
  run as the test. The recommendation is the second.
- Whether the composite action stays local even when the workflow moves, because a
  called workflow cannot use a relative action path (B README lines 135-137) and the
  action holds the pixi version pin S01 may want an updater to move.
