---
id: T11
title: "Integration: first green `just check` from a fresh clone and in CI, branch protection verified"
status: open
depends_on: [T01, T02, T03, T04, T05, T06, T07, T08, T09]
parallel_with: []
branch: ticket/t11-integration
estimated_size: M
---

# T11: Integration: first green `just check` from a fresh clone and in CI, branch protection verified

## Context

Every build ticket before this one changed the repository. This one changes nothing in
it: it proves that the merged `main`, T00's foundation with the nine lanes T01 to T09
on top, passes its own gate from nothing, first from a fresh clone on the maintainer's
machine and then in a CI run dispatched on `main`; it proves that branch protection
still reads back exactly as T01 applied it, with `check` the one required context now
reported by T04's aggregate job (CONVENTIONS.md §10); and it reconciles what the lanes
accumulated as T00 follow-ups. T10 waits
for this ticket so that the README describes a repository known to work; S01 to S04
and C01 wait for it too.

Why a fresh clone rather than the worktree the lanes used (CONVENTIONS.md §13, "The
network at first run, none in `check`"): every lane ran `just check` on a tree already
initialised, with `.tools/bin`, the uv environment, prek's cache and `target/` in
place. Nothing has yet proved that `scripts/initialize.sh` alone, on a machine with
only the prerequisites of CONVENTIONS.md §2, reaches a green gate, which is where a new
contributor and every CI runner start.

Why the protection script runs again at all (CONVENTIONS.md §10, last paragraph): T01
protected `main` behind the one context `check` while the only workflow was the
one-job stub, and T04's workflow keeps reporting `check` through an aggregate job that
needs `rust`, `coverage`, `wasm`, `deny` and `documents`, so nothing about protection
should have changed and no pull request in the lanes needed a bypass. This ticket is
where that is proved rather than assumed: T's `scripts/bootstrap_repo.sh` reads,
compares and mutates (lines 3-6 at `2283589c`), and run with T01's arguments it must
print `already` for step 2 and `changed: 0`. Its default check list is the games'
`ci / frontend,ci / documents,ci / stories` (lines 36-39), so `--checks check` is passed
explicitly. Its step 2 (lines 228-275) also writes `strict: false`, no required
reviews, administrators not bound and force pushes refused; its step 4 (lines 328-343)
turns on private vulnerability reporting, which answers `404` on a private repository,
and T10's `SECURITY.md` wording depends on what this ticket records.

Read first: CONVENTIONS.md §2, §4, §10, §11, §12 and §13; the hand-back notes of T00
and of every lane T01 to T09; T's `tickets/T11-integration.md` lines 13-46 at
`2283589c` for the shape of an integration ticket, and its steps 2 and 3 for how a
failure is diagnosed to its owning lane without being fixed on this branch.

## Goal

- A fresh clone of `main` from the GitHub remote, in the session's scratch directory,
  in which `just initialize` then `just check` are green, both timed.
- A `workflow_dispatch` run of `ci.yml` on `main` green in all six jobs, `check`
  included, and an `audit.yml` run that is green or whose failure is shown to be the
  RustSec fetch.
- Branch protection on `main` still requiring `check` alone, shown unchanged by a dry
  run of T's script that prints `changed: 0`.
- Every T00 follow-up the lanes handed back either merged before the clone step or
  listed as deliberately carried, with the reason.
- One table in the hand-back notes with every CONVENTIONS.md §12 claim, its owning
  ticket and the outcome that ticket recorded, so §12 can be closed out in one edit.
- `git status --porcelain` empty on `main` after the gate, and no `TODO`, `TBD` or
  `FIXME` anywhere outside `tickets/` and `docs/specs/`.

## Non-goals

- Editing any file the lanes own. A failure found here is diagnosed to a path and its
  owning lane (CONVENTIONS.md §3) and written up; the fix lands on `main` through that
  lane's follow-up pull request, separately authorised. This branch carries only its
  own ticket file.
- The four root documents and `docs/` (T10, T07, T08); installing anything on the
  machine; Pages (`--no-pages` always); a tag, release or version bump (S02's, and
  `publish = false` stays).

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/T11-integration.md` | ticket | `status: done`, the hand-back notes, the §12 table |
| repository settings on `steven-cutting/libpawdoku` | settings | none expected: the dry run must print `changed: 0`; an `--apply` is a separately authorised deviation |

A scratch clone lives in the session's scratch directory and is deleted at the end.

## Steps

1. Create the worktree on `ticket/t11-integration` from `main` after T01 to T09 have
   merged (README.md "How to pick up a ticket"). Confirm the machine and the remote:

   ```sh
   command -v rustup cargo cargo-binstall just gh
   just check-toolchain
   git fetch origin main && git log --oneline -1 origin/main
   gh repo view steven-cutting/libpawdoku --json visibility,defaultBranchRef --jq '[.visibility, .defaultBranchRef.name] | join(" ")'
   ```

   `cargo` resolves under `~/.cargo/bin`; the toolchain line names `1.98.1`. Record the
   visibility: it decides step 5's vulnerability-reporting outcome and T10's wording.

2. Reconcile the follow-ups. For each item the "Handed back" sections of T00 to T09
   address to `main`, find the pull request that carried it or record that none did.
   **Authorisation required** for any still open: stop and list them. Do not proceed
   while one that changes `Justfile`, `tools.txt`, `rust-toolchain.toml` or either
   prek config is open, because the clone would prove the wrong tree.

3. Fresh clone, from the GitHub remote and not from the worktree, so nothing the lanes
   left behind is inherited:

   ```sh
   S=<scratch directory>/t11-clone
   git clone git@github.com:steven-cutting/libpawdoku.git "$S"
   cd "$S" && git rev-parse HEAD
   time just initialize
   time just check
   git status --porcelain
   ```

   Both recipes exit 0; `git status --porcelain` prints nothing (CONVENTIONS.md §1
   fact 3). Record the `HEAD` SHA and both times (the troubleshooting page and T00's
   open point on hook cost want them). If `check` touched the network, that is a §13
   finding. On failure, diagnose to a path and an owning lane, write the recipe, the
   output and the proposed change into the hand-back notes, stop, and repeat from step
   2 once the fix has merged.

4. CI on `main`. Dispatch both workflows and wait:

   ```sh
   gh workflow run ci.yml --ref main
   gh workflow run audit.yml --ref main
   gh run list --workflow ci.yml --branch main --limit 1 --json databaseId,status
   gh run watch <id> --exit-status
   gh run view <id> --json jobs --jq '.jobs[] | [.name, .conclusion] | join(" ")'
   ```

   Six lines, `rust`, `coverage`, `wasm`, `deny`, `documents`, `check`, each `success`.
   Record the run URL and each job's duration. An `audit` failure is acceptable only if
   its log shows the RustSec fetch failing; a real advisory is a T02 hand-back.

5. Branch protection, verified. Dry run only, by the script's absolute path, with the
   arguments T01 used:

   ```sh
   /Users/scutting/projects/biscuit_games_template/scripts/bootstrap_repo.sh steven-cutting/libpawdoku --no-pages --checks check --hygiene
   ```

   Drop `--hygiene` only if T01's hand-back says T01 did not pass it. Expected: step 2
   prints `already`, every other step prints `already` or its not-applicable note, and
   the closing line is `changed: 0`. Anything else is a deviation to record: name the
   step and what differs, and stop. **Authorisation required** before any `--apply`,
   which is then run twice to confirm `changed: 0` on the second. Record what step 4
   reports for private vulnerability reporting (enabled, or `404` on a private
   repository): T10 writes `SECURITY.md` from this line. Then confirm the protection
   reads back:

   ```sh
   gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.checks[].context] | sort | join(",")'
   ```

   Prints `check`, and the run in step 4 shows that context reported by T04's aggregate
   job on `main`.

6. The §12 table. One row per claim in CONVENTIONS.md §12: claim, ticket, outcome,
   source (the hand-back section that records it). S02's crates.io row and any claim a
   lane could not reach are marked `pending` with the ticket that will settle them.

7. Placeholders and cleanliness, on `main` at the SHA step 3 cloned:

   ```sh
   grep -rn -E 'TODO|TBD|FIXME' --exclude-dir=.git --exclude-dir=target --exclude-dir=.tools --exclude-dir=.venv --exclude-dir=tickets --exclude-dir=docs/specs --exclude-dir=allium-skill-reference . || echo "no placeholders"
   ```

8. Delete the scratch clone. Set `status: done` and commit on the ticket branch. Stop
   before pushing.

## Acceptance criteria

- The hand-back notes name the `main` SHA the fresh clone came from and quote exit 0
  from `just initialize` and `just check` there, with both wall-clock times.
- One `ci.yml` run on `main` at that SHA is green in all six jobs, with the run URL
  quoted; the `audit.yml` outcome is recorded with its reason if not green.
- `main` requires exactly `check`; the script's dry run with T01's arguments reports
  `changed: 0`, or the deviation is recorded and any `--apply` was authorised.
- Every T00 follow-up is merged before the clone or listed as carried; the §12 table
  covers every claim with an outcome or `pending`; the visibility and the
  vulnerability-reporting outcome are recorded for T10.
- Nothing outside `tickets/T11-integration.md` changed on this branch; no tag exists.

## Verification

```sh
git -C <scratch>/t11-clone rev-parse HEAD
gh run list --workflow ci.yml --branch main --limit 1 --json conclusion --jq '.[0].conclusion'
gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.checks[].context] | sort | join(",")'
git ls-remote --tags origin | wc -l
git status --porcelain
```

Expected: the SHA of `main`; `success`; `check`; `0`; one line naming this file. Quote
each in the hand-back notes.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the first `just initialize` on a clean machine hits GitHub's anonymous API
  rate limit through cargo-binstall's release lookups (CI sets a `GITHUB_TOKEN` on that
  step; the local script does not). If so, the answer is a note in
  `docs/how-to/develop-locally.md`, handed back to T07.
- Whether `coverage` on the merged tree clears the 90% floor with T02's `random.rs` but
  no engine code; if not, the fix is a T02 test, never a lower floor (CONVENTIONS.md §4).
