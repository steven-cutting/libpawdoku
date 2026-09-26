---
id: T01
title: "GitHub repository: create, first push, settings, branch protection"
status: done
depends_on: [T00]
parallel_with: [T02, T03, T04, T05, T06, T07, T08, T09]
branch: ticket/t01-github-repository
estimated_size: M
---

# T01: GitHub repository: create, first push, settings, branch protection

## Context

This repository exists only on the maintainer's machine: one `init` commit, then T00's
foundation merged to `main` (CONVENTIONS.md §0, §3). It has no remote, so nothing in it
has ever run in CI, no pull request can be opened, and T04 cannot make its proof run
(README.md, dependency graph). This ticket creates `steven-cutting/libpawdoku` on GitHub,
pushes `main`, and applies the settings no file can carry, in the shape the house already
uses. The GitHub identity is `steven-cutting`; the sibling repositories `pawdoku`,
`biscuit_games_template` and `biscuit_games_tooling` are public.

Every mutation here is a separately authorised action (CONVENTIONS.md §11): creating the
repository, pushing, and every settings change. The steps below list the exact command
for each, mark it **Authorisation required**, and the executing agent runs it only after
the maintainer says so, then records in the hand-back notes what was run and what it
printed. Read-only `gh api` GETs, `gh repo view` and the script's dry run need no
authorisation (the precedent is T's `tickets/C03-repository-bootstrap.md` lines 89-92).

The settings come from T's `scripts/bootstrap_repo.sh` (T = `biscuit_games_template` at
`2283589c`; CONVENTIONS.md §1 fact 9 and §12), read with
`git -C /Users/scutting/projects/biscuit_games_template show 2283589c:scripts/bootstrap_repo.sh`.
What it does, by line, at that commit:

- Usage (lines 14-16): `<owner>/<repo> [--apply] [--checks a,b,c] [--no-pages]
  [--chromatic-token-stdin] [--hygiene]`. Without `--apply` it reads, prints each
  step's state and the command it would run, and changes nothing (lines 4-6, 151-158).
- Step numbers are fixed; `--no-pages` omits step 1 rather than renumbering (lines 8-10).
- Step 2 (lines 224-283) is classic branch protection on `main` through
  `PUT repos/<owner>/<repo>/branches/main/protection`: `strict: false`, one `checks`
  entry per `--checks` name, `enforce_admins: false`, no review requirement, no
  restrictions, force pushes and deletions refused. That is Poodl's protection
  (P `docs/reference/quality-gates.md` lines 147-153, "On `main`"), which pawdoku
  carries today with no rulesets (read on 2026-09-23).
- Step 3 (lines 285-323) lists secrets and, without `--chromatic-token-stdin`, prints
  `skipped: no token supplied`. Step 5 (lines 346-354) prints the npm package grant.
  Neither applies to a Rust library; both print and do nothing.
- Step 4 (lines 325-344) turns on private vulnerability reporting with
  `gh api -X PUT repos/<owner>/<repo>/private-vulnerability-reporting`; on a private
  repository the endpoint answers 404 and the script only notes it (lines 337-339).
- Step 6 (lines 356-376), only with `--hygiene`, runs
  `gh repo edit <owner>/<repo> --delete-branch-on-merge --enable-wiki=false --enable-projects=false`.
- The script sets no merge method. All four sibling repositories allow squash, merge
  commit and rebase with GitHub's default title and message settings (read with
  `gh api repos/steven-cutting/<name>` on 2026-09-23). This ticket leaves that alone.

The required check is `check`, and it is the only one this repository ever requires
(CONVENTIONS.md §10 last paragraph). Today it is the one job of T00's `ci.yml` stub (T00
step 8); once T04 merges it is the aggregate job that needs the five gate jobs, so the
name every pull request must satisfy never changes and no lane's merge waits on a
protection edit. T11 reruns the same script with the same `--checks check` only to prove
it reads back `already` and `changed: 0`. A required context may be set before any
workflow has reported it (C03's hand-back, lines 760-762).

One trap, verified in gh's source (`cli/cli`, `pkg/cmd/repo/create/create.go`, the
`Push(... "HEAD")` call): `gh repo create --source . --push` pushes `HEAD`, which in this
ticket's worktree is `ticket/t01-github-repository`, so the one-shot form would push the
ticket branch and GitHub would make it the default. The steps use the two-step form.

Tooling: gh 2.100.0 as `steven-cutting`, scopes `repo`, `workflow`, `read:org`, `gist`,
`admin:public_key`, git protocol ssh (CONVENTIONS.md §0); `repo` is the scope every call
here needs.

## Goal

`steven-cutting/libpawdoku` exists on GitHub with `main` as its default branch holding
exactly the local `main`; `origin` points at it; the description is set and the homepage
empty; issues on, wiki and projects off, branches deleted on merge; `main` protected
behind the single check `check` with no review requirement, force pushes and deletions
refused, administrators not bound; private vulnerability reporting on if the repository
is public; and the visibility decision recorded with its reasons.

## Non-goals

- No file in this repository changes except this ticket. `Cargo.toml`'s `repository`
  URL (T00) already names the address this ticket creates; `SECURITY.md` is T10's.
- No workflow change and no CI proof run: T04 owns `.github/`. This ticket only observes
  the stub run the first push triggers.
- No merge-method change, topics, social preview, Pages, secrets, rulesets, Dependabot
  (S01) or release settings (S02). No tag, no GitHub issue, no pull request beyond this
  ticket's own. There is no second round of protection: T11 only reruns the script to
  show nothing changed.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/T01-github-repository.md` | ticket | `status:`, hand-back notes |

Repository settings, none of them a file:

| Setting | Value | How set |
| --- | --- | --- |
| Existence, visibility | `steven-cutting/libpawdoku`, public (open point) | `gh repo create`, step 3 |
| Description | `Classic-sudoku engine for Biscuit Games: solver, technique catalogue, difficulty rating, hints.` | `gh repo create -d`, step 3 |
| Homepage | empty | nothing passed; verified in step 9 |
| Default branch | `main` | first push of `main` to an empty repository, step 4; `gh repo edit --default-branch main` only if the read says otherwise |
| Issues | on | `gh repo create`'s default (no `--disable-issues`) |
| Wiki, projects | off | script step 6 (`--hygiene`), step 7 |
| Delete branch on merge | on | script step 6 (`--hygiene`), step 7 |
| Merge methods | GitHub's defaults, untouched | nothing; open point for S02 |
| Protection on `main` | required check `check`, `strict: false`, no review, no restrictions, no force push, no deletion, administrators not bound | script step 2, step 7 |
| Private vulnerability reporting | on (public only) | script step 4, step 7 |

## Steps

1. Create the worktree on `ticket/t01-github-repository` from `main` (README.md "How to
   pick up a ticket"). Confirm the starting state and quote it:

   ```sh
   git remote -v
   git log --oneline main | head -5
   gh auth status
   gh repo view steven-cutting/libpawdoku 2>&1 | head -1
   mkdir -p ai_tmp
   git -C /Users/scutting/projects/biscuit_games_template show 2283589c:scripts/bootstrap_repo.sh > ai_tmp/bootstrap_repo.sh
   ```

   Expected: no remote; `main`'s tip is T00's merge; `gh` is logged in as
   `steven-cutting`; the last `gh repo view` reports the repository does not exist. The
   copy in `ai_tmp/` is the pinned script; run that, not T's working file, which may
   have moved on.

2. Settle visibility with the maintainer (Open points). Nothing below runs until it is
   answered, because the create command carries it and flipping later is a second
   authorised action.

3. **Authorisation required: create the repository.** No `--source`, no `--push`, no
   `--license`, no `--add-readme`, no `--gitignore`: each of the last three seeds a
   commit, after which the push of `main` is refused as non-fast-forward, and `LICENSE`
   already exists from T00.

   ```sh
   gh repo create steven-cutting/libpawdoku --public \
     --description "Classic-sudoku engine for Biscuit Games: solver, technique catalogue, difficulty rating, hints."
   ```

   With `--private` in place of `--public` if the open point goes that way. Quote the
   printed URL.

4. **Authorisation required: the first push.** From the worktree; `main` is a shared
   ref, so it pushes without being checked out:

   ```sh
   git remote add origin git@github.com:steven-cutting/libpawdoku.git
   git push -u origin main
   ```

   Then read the default branch back. An empty repository takes the first branch
   pushed as its default, so this should print `main`; if it does not, `gh repo edit
   steven-cutting/libpawdoku --default-branch main` is a further authorised action.

   ```sh
   gh repo view steven-cutting/libpawdoku --json defaultBranchRef --jq .defaultBranchRef.name
   git ls-remote origin main
   ```

5. Observe the stub run. The push triggers T00's `ci.yml` on `main` (`on: push` to
   `main`, CONVENTIONS.md §10). Read-only:

   ```sh
   gh run list -R steven-cutting/libpawdoku --branch main --limit 3
   gh run watch -R steven-cutting/libpawdoku --exit-status "$(gh run list -R steven-cutting/libpawdoku --branch main --limit 1 --json databaseId --jq '.[0].databaseId')"
   ```

   Record green or red with the job name. Protection goes on either way in step 7: with
   `enforce_admins: false` the owner's merges are not blocked, which is the reasoning
   C03 step 10 gives for protecting before the first push. A red `check` is handed back
   to T04 (and to T00 if the cause is in a frozen file), because every lane's pull
   request will need it.

6. Dry run, read-only. Quote the whole output in the hand-back notes:

   ```sh
   sh ai_tmp/bootstrap_repo.sh steven-cutting/libpawdoku --no-pages --checks check --hygiene
   ```

   Expected: `mode: dry run (nothing is changed)`; no step 1; step 2 with state
   `not protected (HTTP 404)`, `wanted: false check false false false false false` and
   the JSON body naming one context, `check`; step 3 with state `not set` and the
   `skipped: no token supplied` note; step 4 with state `false` and the PUT it would run,
   or `true` and `already` if the account turns the feature on for new public
   repositories (on a private one: `no such endpoint (HTTP 404)` and `not available`);
   step 5 printed and irrelevant; step 6 with state `false true true` and
   `wanted: true false false`; the closing line `changed: 0 (dry run; 3 would change)`,
   or 2 where step 4 had nothing to do. Any other output is a CONVENTIONS.md §12 outcome
   to record before going on.

7. **Authorisation required: apply.** One run covers protection, vulnerability
   reporting and hygiene, exactly as the quoted plan showed; the maintainer authorises
   that plan. Then run it a second time to prove idempotency:

   ```sh
   sh ai_tmp/bootstrap_repo.sh steven-cutting/libpawdoku --no-pages --checks check --hygiene --apply
   sh ai_tmp/bootstrap_repo.sh steven-cutting/libpawdoku --no-pages --checks check --hygiene --apply
   ```

   Expected: the first prints one `+` line per changed step and the count the dry run
   promised; the second prints `already` for each and `changed: 0` with no `+` line.

8. Record the CONVENTIONS.md §12 claim assigned to this ticket ("`bootstrap_repo.sh`
   applies cleanly to a repository with no Pages and one required check") with its
   outcome, and the rulesets and hygiene open points as settled. Under "Handed back",
   give T10 the visibility and the vulnerability-reporting state as read back, so that
   `SECURITY.md` names the form or the fallback (T00 step 12 left that to this ticket).

9. Run the verification commands, quote their output, set `status: done`, and commit on
   the ticket branch. Pushing the branch and opening the pull request are separately
   authorised: stop and ask. That pull request is the first one `check` gates.

## Acceptance criteria

- `gh repo view steven-cutting/libpawdoku` succeeds; visibility matches the recorded
  decision; the description is the exact sentence above; `homepageUrl` is empty.
- The default branch is `main`, and `git ls-remote origin main` prints the same commit
  as `git rev-parse main`.
- `origin` is `git@github.com:steven-cutting/libpawdoku.git` for fetch and push.
- Issues on; wiki off; projects off; delete branch on merge on.
- The protection read (the script's own `--jq`, lines 229-237) prints
  `false check false false false false false`.
- Private vulnerability reporting reads `{"enabled":true}` on a public repository; on a
  private one the endpoint answers 404 and the hand-back notes say so for T10.
- The second `--apply` printed `changed: 0` and no line beginning `+`.
- Every command marked **Authorisation required** was authorised before it ran, and
  each is listed in the hand-back notes with what it printed.
- The stub run's result on `main` is recorded, green or red.
- `git status --porcelain` on the ticket branch lists only this file.

## Verification

```sh
gh repo view steven-cutting/libpawdoku --json visibility,defaultBranchRef,hasIssuesEnabled,hasWikiEnabled,hasProjectsEnabled,deleteBranchOnMerge,description,homepageUrl
gh api repos/steven-cutting/libpawdoku --jq '[.allow_squash_merge, .allow_merge_commit, .allow_rebase_merge] | map(tostring) | join(" ")'
gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.strict, ([.required_status_checks.checks[]?.context] | sort | join(",")), .enforce_admins.enabled, .allow_force_pushes.enabled, .allow_deletions.enabled, (.required_pull_request_reviews != null), (.restrictions != null)] | map(tostring) | join(" ")'
gh api repos/steven-cutting/libpawdoku/rulesets
gh api repos/steven-cutting/libpawdoku/private-vulnerability-reporting
git remote -v
git ls-remote origin main
git rev-parse main
gh run list -R steven-cutting/libpawdoku --branch main --limit 3
git status --porcelain
```

Expected: the view prints `PUBLIC` (or `PRIVATE`), `main`, `true`, `false`, `false`,
`true`, the description, and an empty `homepageUrl`; the merge line is
`true true true`; the protection line is `false check false false false false false`;
rulesets is `[]`; vulnerability reporting is `{"enabled":true}` (public); `origin` is the
ssh URL twice; the two commit hashes agree; the run list shows the stub run with its
conclusion; `git status --porcelain` names only this file.

## Hand-back notes

**Done on 2026-09-25.** One session with gh 2.100.0 as `steven-cutting`. The repository
is public at <https://github.com/steven-cutting/libpawdoku>. `main` is pushed at
`9eb968a` and protected behind `check`, and the stub run of `check` on `main` is green.
The ticket branch is pull request #2, and its `check` is green.

### Authorised actions, and what each printed

The maintainer answered in this session, before each action ran. Visibility: "Public
(Recommended)". Steps 3 and 4: "Authorise both", given against the three commands
quoted verbatim in the question. Step 7: "Authorise apply", given after the step 6 dry
run below had been shown in full. Step 9's push and pull request: "Push and open PR",
given after the closing commit. The network fetches under "What was verified" were not
asked for beforehand; the maintainer approved them afterwards (see Deviations).

- Step 3, run once:

  ```sh
  gh repo create steven-cutting/libpawdoku --public \
    --description "Classic-sudoku engine for Biscuit Games: solver, technique catalogue, difficulty rating, hints."
  ```

  It printed `https://github.com/steven-cutting/libpawdoku` and exited 0.
- Step 4: `git remote add origin git@github.com:steven-cutting/libpawdoku.git` exited 0
  and printed nothing. `git push -u origin main` printed:

  ```text
  To github.com:steven-cutting/libpawdoku.git
   * [new branch]      main -> main
  branch 'main' set up to track 'origin/main'.
  ```

  Read back:
  `gh repo view steven-cutting/libpawdoku --json defaultBranchRef --jq .defaultBranchRef.name`
  printed `main`, and
  `git ls-remote origin main` printed
  `9eb968a5a9ca22696257e2999c2e19e41fc1e79e  refs/heads/main`. So `gh repo edit
  --default-branch` was not needed and was not run.
- Step 7, first `--apply`: the dry-run output below, with `+` in place of each `would:`,
  `mode: apply` and `changed: 3`. The three `+` lines:

  ```text
     + gh api -X PUT repos/steven-cutting/libpawdoku/branches/main/protection --input -
     + gh api -X PUT repos/steven-cutting/libpawdoku/private-vulnerability-reporting
     + gh repo edit steven-cutting/libpawdoku --delete-branch-on-merge --enable-wiki=false --enable-projects=false
  ```

- Step 7, second `--apply`: no line beginning `+` (`grep -c '^ *+'` printed `0`):

  ```text
  repository: steven-cutting/libpawdoku
  mode: apply

  2. Protection on main requiring check
     state: false check false false false false false
     already

  3. CHROMATIC_PROJECT_TOKEN
     state: not set
     skipped: no token supplied; chromatic.yml notes the absence and skips the publish

  4. Private vulnerability reporting
     state: true
     already

  5. Package read access for @steven-cutting/biscuit-games
     state: cannot be read: no REST endpoint, and the package needs a read:packages scope
     The package is public, so any repository installs it with the run's own
     token and no grant is needed. Were it made private, the grant is by hand:
     open the package page, Package settings, Manage Actions access, Add
     repository, search for steven-cutting/libpawdoku, and set its Role to Read.

  6. Hygiene
     state: true false false
     already

  changed: 0
  ```

- Step 9, after the closing commit: `git push -u origin T01-github-repository` printed

  ```text
  remote:
  remote: Create a pull request for 'T01-github-repository' on GitHub by visiting:
  remote:      https://github.com/steven-cutting/libpawdoku/pull/new/T01-github-repository
  remote:
  To github.com:steven-cutting/libpawdoku.git
   * [new branch]      T01-github-repository -> T01-github-repository
  branch 'T01-github-repository' set up to track 'origin/T01-github-repository'.
  ```

  and `gh pr create --base main --head T01-github-repository --title "Finish T01:
  repository created, main pushed and protected" --body-file -` printed
  `https://github.com/steven-cutting/libpawdoku/pull/2`. The pull request's `check`
  (run `36202500034`, job `108291958246`) passed in 50s, and
  `gh pr view 2 --json mergeStateStatus,mergeable` read `CLEAN MERGEABLE`. It is the
  first pull request `check` gated. Pull request #1 is T05's, opened by another session.

Git's own configuration also changed: `origin` and `main`'s upstream
(`branch.main.remote origin`, `branch.main.merge refs/heads/main`) are in the shared
`.git/config`, so every worktree of this repository, and the primary checkout, now sees
`origin`. The setup under "What was verified" also wrote outside the worktree; it is
listed there.

### What was verified, and how

Step 1, before any mutation: `git remote -v` printed nothing. `git log --oneline main`
began `9eb968a Correct the hook hand-off order in the T00 hand-back`, which is T00's
closing commit; T00 reached `main` by fast-forward, so its tip is not a merge commit.
`gh auth status` reported `steven-cutting`, protocol ssh, and scopes `admin:public_key`,
`gist`, `read:org`, `repo`, `workflow`. `gh repo view` reported `Could not resolve to a
Repository with the name 'steven-cutting/libpawdoku'`. The pinned script was copied with
the step's `git show 2283589c:scripts/bootstrap_repo.sh`. The copy is 382 lines with
SHA-256 `62cec7df09935f2350936c36f0891f29c9d0243e21859a83d37e47050bdb6ace`, and every
line reference in Context matched it.

Before the push, a read-only check found that every commit on `main` has
`tech@scutting.com` as author and committer. The latest three commits of `pawdoku`,
`biscuit_games_template` and `biscuit_games_tooling` already show that address
publicly, so making this repository public exposed nothing new. The shared
`pre-commit` shim names `/Users/scutting/projects/libpawdoku/.pixi/envs/default/bin/prek`,
which exists, and there is no `pre-push` hook, so the push ran no hook.

Step 5: run `36201752678` (`CI`, event `push`, on `main` at `9eb968a`) was **green**.
Job `check` (ID `108289700703`) passed in 1m48s, including every step:
checkout, `./.github/actions/setup`, `just install-allium` and `just check`. The run
carried one annotation: "The ubuntu-latest label will migrate to Ubuntu 26 beginning
October 19, 2026."

Step 6, the dry run, matched the expected output exactly. It printed no step 1; step 2
`not protected (HTTP 404)` with the one-context body; step 3 `not set` and skipped; step
4 `false` and the PUT (the account does not turn reporting on by default for new public
repositories); step 5 printed only; step 6 `false true true` with
`wanted: true false false`; and `changed: 0 (dry run; 3 would change)`. The full output:

```text
repository: steven-cutting/libpawdoku
mode: dry run (nothing is changed)

2. Protection on main requiring check
   state: not protected (HTTP 404)
   wanted: false check false false false false false
   would: gh api -X PUT repos/steven-cutting/libpawdoku/branches/main/protection --input -
     {
       "required_status_checks": {
         "strict": false,
         "checks": [
           { "context": "check" }
         ]
       },
       "enforce_admins": false,
       "required_pull_request_reviews": null,
       "restrictions": null,
       "allow_force_pushes": false,
       "allow_deletions": false
     }

3. CHROMATIC_PROJECT_TOKEN
   state: not set
   skipped: no token supplied; chromatic.yml notes the absence and skips the publish

4. Private vulnerability reporting
   state: false
   would: gh api -X PUT repos/steven-cutting/libpawdoku/private-vulnerability-reporting

5. Package read access for @steven-cutting/biscuit-games
   state: cannot be read: no REST endpoint, and the package needs a read:packages scope
   The package is public, so any repository installs it with the run's own
   token and no grant is needed. Were it made private, the grant is by hand:
   open the package page, Package settings, Manage Actions access, Add
   repository, search for steven-cutting/libpawdoku, and set its Role to Read.

6. Hygiene
   state: false true true
   wanted: true false false
   would: gh repo edit steven-cutting/libpawdoku --delete-branch-on-merge --enable-wiki=false --enable-projects=false

changed: 0 (dry run; 3 would change)
```

CONVENTIONS.md §12 claim: "`T/scripts/bootstrap_repo.sh` applies cleanly to a
repository with no Pages and one required check". **Holds.** `--no-pages` skipped step
1 without renumbering. `--checks check` produced one context. The first apply changed
exactly the three steps the dry run named, and the second changed nothing. The script
never exited non-zero.

Step 9, the Verification block, run after step 7 (`git status --porcelain` was run
again after this file was edited). The tabs in gh's and git's output are shown as two
spaces here and in step 4 above, because markdownlint refuses hard tabs:

```text
$ gh repo view steven-cutting/libpawdoku --json visibility,defaultBranchRef,hasIssuesEnabled,hasWikiEnabled,hasProjectsEnabled,deleteBranchOnMerge,description,homepageUrl
{"defaultBranchRef":{"name":"main"},"deleteBranchOnMerge":true,"description":"Classic-sudoku engine for Biscuit Games: solver, technique catalogue, difficulty rating, hints.","hasIssuesEnabled":true,"hasProjectsEnabled":false,"hasWikiEnabled":false,"homepageUrl":"","visibility":"PUBLIC"}
$ gh api repos/steven-cutting/libpawdoku --jq '[.allow_squash_merge, .allow_merge_commit, .allow_rebase_merge] | map(tostring) | join(" ")'
true true true
$ gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.strict, ([.required_status_checks.checks[]?.context] | sort | join(",")), .enforce_admins.enabled, .allow_force_pushes.enabled, .allow_deletions.enabled, (.required_pull_request_reviews != null), (.restrictions != null)] | map(tostring) | join(" ")'
false check false false false false false
$ gh api repos/steven-cutting/libpawdoku/rulesets
[]
$ gh api repos/steven-cutting/libpawdoku/private-vulnerability-reporting
{"enabled":true}
$ git remote -v
origin  git@github.com:steven-cutting/libpawdoku.git (fetch)
origin  git@github.com:steven-cutting/libpawdoku.git (push)
$ git ls-remote origin main
9eb968a5a9ca22696257e2999c2e19e41fc1e79e  refs/heads/main
$ git rev-parse main
9eb968a5a9ca22696257e2999c2e19e41fc1e79e
$ gh run list -R steven-cutting/libpawdoku --branch main --limit 3
completed  success  Correct the hook hand-off order in the T00 hand-back  CI  main  push  36201752678  1m53s  2026-09-25T23:37:45Z
$ git status --porcelain
 M tickets/T01-github-repository.md
```

Every acceptance criterion holds as read back above.

The local gate also ran on this branch before the commit. `pixi install --locked`,
`just install-tools` and `just install-allium` ran first, all inside the worktree,
because Supacode's secondary worktree had no environment. They used the network, as
`just initialize` does. cargo-binstall downloaded cargo-hack 0.6.45 from github.com, and
`bg-install-allium` downloaded the allium 3.6.1 tarball from juxt/allium-tools'
releases. `pixi install --locked` refreshed pixi's shared cache
(`~/Library/Caches/rattler/cache`: the `repodata`, `conda-pypi-mapping` and `uv-cache`
entries are newer than the run). Its log does not show which packages, if any, it
downloaded rather than linking from the cache, and a concurrent session may account for
some of those writes. Inside the worktree the setup wrote only gitignored paths
(`.pixi/`, `.tools/`, `target/`). Outside it, prek also updated
`~/.cache/prek/config-tracking.json` to register this worktree's configuration. None of
this was asked for beforehand (see Deviations). `prek run --files` on this
file passed every hook that applies (markdownlint, typos, lychee, EditorConfig,
ripsecrets and the builtins). `just check` exited 0, ending `All checks passed and the
worktree is unchanged.`

### Deviations, and why

- **Branch name.** The Supacode worktree's branch is `T01-github-repository`, not the
  `ticket/t01-github-repository` in this file's `branch:` field, which CONVENTIONS.md
  §11 and `tickets/README.md` ("How to pick up a ticket") require. The maintainer chose
  to keep that name and record the difference rather than rename, as T00 and D01 did.
  The question noted that Supacode's bookkeeping might not follow a rename. The choice
  was asked again after review of pull request #2 and kept. The `branch:` field is left
  as written; the branch pushed and gated as pull request #2 is `T01-github-repository`.
  Step 1's worktree already existed, created by Supacode from `main` at `9eb968a`.
- **Network setup without prior authorisation.** `AGENTS.md` ("Safety and authority")
  requires the maintainer's explicit authorisation, at the time, for network operations.
  `pixi install --locked`, `just install-tools` and `just install-allium` ran without it,
  because the commit hook and `just check` needed an environment this worktree lacked.
  Review of pull request #2 raised it. On 2026-09-25 the maintainer approved those
  operations after the fact. They were not approved in advance, and nothing else ran
  without authorisation.
- **One aborted invocation.** The first attempt at step 7 was a compound shell command
  that printed the watch log, then a separator line (`echo ======`), then ran both
  `--apply` runs. zsh read the `=` as filename expansion and stopped with `(eval):1:
  ===== not found` before either `--apply` ran. The first real apply then read the same
  states as the dry run (404, `false`, `false true true`), which confirms that nothing
  had changed. Both runs were then made as separate commands under the same
  authorisation.

### Handed back

- **T10 and T11 (`SECURITY.md`).** The repository is **public** and private
  vulnerability reporting reads `{"enabled":true}`. `SECURITY.md` can therefore name
  GitHub's private vulnerability reporting form as the route; it does not need the
  fallback.
- **T11 (the protection rerun).** The arguments were
  `steven-cutting/libpawdoku --no-pages --checks check --hygiene`, and `--hygiene` was
  passed. T01's pinned copy was a scratch file, not committed, so T11 extracts its own
  with
  `git -C /Users/scutting/projects/biscuit_games_template show 2283589c:scripts/bootstrap_repo.sh`
  and confirm SHA-256
  `62cec7df09935f2350936c36f0891f29c9d0243e21859a83d37e47050bdb6ace` before the dry run.
- **T04.** The stub `check` is green on `main` (run `36201752678`), so every lane's pull
  request starts from a passing required check. The runner reported that `ubuntu-latest`
  moves to Ubuntu 26 from 2026-10-19. T04 owns the `runs-on` lines and decides whether
  to pin an image before then.
- **Maintainer.** Protection is live, so landing a lane on `main` now goes through a
  pull request that `check` gates. Administrators are not bound, so the owner can still
  push to `main` directly, but that bypasses the check. `origin` is configured for every
  worktree through the shared `.git/config`.
- **Maintainer (ticket pickup).** Two gaps in the pickup procedure showed up here, and
  T05's pull request #1 shares the first. First, Supacode names a worktree's branch
  `<ID>-<slug>`, not the `ticket/<id>-<slug>` that CONVENTIONS.md §11 and
  `tickets/README.md` require; either the procedure renames every time, or the
  convention changes. Second, a Supacode secondary worktree has no pixi environment, so
  the commit hook and `just check` (the definition of done) cannot run until
  `pixi install --locked`, `just install-tools` and `just install-allium` fetch from the
  network. The pickup steps should say to ask for that authorisation, or say that it is
  given. Neither file is T01's to change.

### Open points settled

- **Visibility: public.** The maintainer decided on 2026-09-25, before step 3, for the
  reasons recorded under Open points.
- **`--hygiene`: applied cleanly.** It read `false true true`, applied
  `gh repo edit steven-cutting/libpawdoku --delete-branch-on-merge --enable-wiki=false --enable-projects=false`,
  and read back `true false false` (`already`) on the second run. Nothing differed from
  the ticket's description.
- **Rulesets or classic protection: classic.** The protection line reads
  `false check false false false false false`, and `rulesets` is `[]`.
- **Merge methods: GitHub's defaults, untouched.** They read `true true true`. S02 owns
  any restriction.
- **Administrators not bound.** `enforce_admins.enabled` reads `false`, as decided on
  2026-09-24.

## Open points

- **Visibility, decided before step 3.** Public: Apache-2.0 is already chosen
  (CONVENTIONS.md §1 decision 3); crates.io publication (S02) expects public source;
  private vulnerability reporting exists only on public repositories, so T10's
  `SECURITY.md` can name the form rather than a fallback; C01's reusable workflow in
  `biscuit_games_tooling` is callable without a grant; Actions minutes are unmetered.
  Private: nothing is gained for a library with no secrets and no unreleased product,
  and the engine's specifications already live in a public repository (G). Recommended:
  **public**. Reversible with `gh repo edit steven-cutting/libpawdoku --visibility
  private --accept-visibility-change-consequences`, itself an authorised action.
- **`--hygiene`.** The flag exists at the pinned commit (line 54; step 6, lines 356-376)
  and does exactly the three things this ticket wants, so the steps pass it. Record
  that it applied cleanly, or what differed.
- **Rulesets or classic protection.** The script writes classic protection; Poodl and
  pawdoku use classic with no rulesets; C03 kept classic (T `tickets/C03-repository-bootstrap.md`
  lines 158-162 and 751-752). Recommended: classic, so T11's rerun is the same
  command. Reopen only if GitHub deprecates the branch-protection endpoint.
- **Merge methods.** Left at GitHub's defaults (all three allowed), matching every
  sibling. Whether to restrict to squash with the pull-request title is S02's question,
  when release tooling starts reading history.
- **Administrators not bound.** `enforce_admins: false` is the house setting (Poodl,
  pawdoku, C03) and the script hardcodes it: its step 2 compares the whole protection
  line against a `false` there and re-PUTs the body on any mismatch, so binding
  administrators by a separate call would be undone by T11's rerun and would break the
  `changed: 0` idempotency check. Binding them is a reasonable hardening for a
  single-maintainer repository, since the aggregate `check` means no planned merge needs
  a bypass, but it starts with an `--enforce-admins` flag in T's script, a change to
  another repository. Decided 2026-09-24: leave unbound; reopen as a T hand-back if a
  bypass is ever used.
