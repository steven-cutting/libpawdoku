---
id: T11
title: "Integration: first green `just check` from a fresh clone and in CI, branch protection verified"
status: done
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
initialised, with the pixi environment, `.tools/bin`, prek's cache and `target/` in
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
  `FIXME` anywhere outside `tickets/`, `docs/specs/` and the vendored Allium material
  (`allium-skill-reference/` and the seven Allium skills under `.agents/skills/`, which
  T05 keeps byte-identical to G). Amended after review; see Deviations.

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
   command -v rustup cargo pixi just gh
   just check-toolchain
   git fetch origin main && git log --oneline -1 origin/main
   gh repo view steven-cutting/libpawdoku --json visibility,defaultBranchRef --jq '[.visibility, .defaultBranchRef.name] | join(" ")'
   ```

   `cargo` resolves under `~/.cargo/bin`, not `~/.pixi/bin`; the toolchain line names
   `1.98.1`. Record the
   visibility: it decides step 5's vulnerability-reporting outcome and T10's wording.

2. Reconcile the follow-ups. For each item the "Handed back" sections of T00 to T09
   address to `main`, find the pull request that carried it or record that none did.
   **Authorisation required** for any still open: stop and list them. Do not proceed
   while one that changes `Justfile`, `tools.txt`, `rust-toolchain.toml`,
   `pyproject.toml`'s pixi tables or either prek config is open, because the clone would
   prove the wrong tree.

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

5. Branch protection, verified. Dry run only, from the pinned copy T01 took (not T's
   working file, which may have moved on), with the arguments T01 used:

   ```sh
   mkdir -p ai_tmp
   git -C /Users/scutting/projects/biscuit_games_template show 2283589c:scripts/bootstrap_repo.sh > ai_tmp/bootstrap_repo.sh
   sh ai_tmp/bootstrap_repo.sh steven-cutting/libpawdoku --no-pages --checks check --hygiene
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

6. The §12 table. One row per claim in CONVENTIONS.md §12, whichever ticket added it:
   claim, ticket, outcome, source (the hand-back section that records it). S02's
   crates.io row and any claim a lane could not reach are marked `pending` with the
   ticket that will settle them.

7. Placeholders and cleanliness, on `main` at the SHA step 3 cloned:

   ```sh
   grep -rn -E 'TODO|TBD|FIXME' --exclude-dir=.git --exclude-dir=target --exclude-dir=.tools --exclude-dir=.pixi --exclude-dir=ai_tmp --exclude-dir=tickets --exclude-dir=specs --exclude-dir=allium-skill-reference . | grep -v -E '^(\./)?\.agents/skills/(allium|distill|elicit|propagate|tend|weed|witness)/' || echo "no placeholders"
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

Done on 2026-09-26 in the Supacode worktree `T11-integration`, against `main` at
`2066be727c7ed728ad9c1a2a0e12e671177babdd` (the merge of pull request #11).

### What was verified, and how

**Step 1, the machine and the remote.** `command -v` printed
`~/.cargo/bin/rustup`, `~/.cargo/bin/cargo`, `~/.pixi/bin/pixi`, `~/.pixi/bin/just` and
`/opt/homebrew/bin/gh`. `just check-toolchain` printed
`1.98.1-aarch64-apple-darwin (overridden by '…/rust-toolchain.toml')`. The `gh repo view`
line printed `PUBLIC main`. `rustup toolchain list` showed `stable` already installed
beside the active `1.98.1`.

**Step 2, the follow-ups.** When this ticket was picked up, all nine lanes and T12 had
merged (pull requests #1 to #10, `main` at `3852aa7`), but no follow-up pull request
had. Three open follow-ups changed the `Justfile`, so the maintainer chose to land them
first, together with the text-only ones, in pull request #11 (branch `t00-followups`,
commits `348babe` and `c8709a7`). Before the push, T03's proofs were re-run against the
new recipe. `just lock-check` on the current locks exits 0 with both hashes unchanged
(`Dry-run: lock file would not change`). With `cargo-shear` removed from
`pyproject.toml`, it exits 1 (`lock file not up-to-date with the workspace`) and again
leaves both hashes unchanged. With the crate's version bumped, the cargo line exits 101
(`cannot update the lock file … because --locked was passed`). With an empty
`CARGO_HOME`, it exits 101 (`no matching package named serde found`) and fetches
nothing. `just check` was green on the branch. All seven checks were green on the PR,
and it merged as `2066be7`. The table lists every item the Handed back sections of T00 to T09 address
to `main` or to T11:

| Item | From | Outcome |
| --- | --- | --- |
| `lock-check`'s pixi line gains `--dry-run` | T03, T08 | Merged in #11 |
| `lock-check`'s cargo line gains `--offline` | T08 (PR #10 review) | Merged in #11 |
| The lychee warm runs under `RUSTUP_AUTO_INSTALL=0` | T03, T08 | Merged in #11 |
| The troubleshooting sections and the quality-philosophy paragraph that go with those three | T08 | Merged in #11 |
| CONVENTIONS.md §2 and §12 wording on prek's rustup and cargo-hack's missing checksum | T03 | Merged in #11 |
| §12 on cargo-shear and `[workspace.dependencies]` | T02 | The fact merged in #11 (the §12 outcome sentence); the decision it asks for is carried |
| §7's `spec-change` row: steps 1, 4 and 8 | T05 | Merged in #11 |
| `docs/README.md` names `board.allium` | T08, T12 | Merged in #11 |
| `tickets/README.md` shows T07, T08 and T09 done | T08 | Merged in #11 |
| §6's quality-philosophy note | T08 | No change needed: the row already reads "every frontend example (browser, story, bundle) replaced", not the "verbatim minus the browser example" T08 quoted |
| The stale 0005 sentence in `work-with-the-specs.md` | T07 | Already closed in PR #9 |
| `audit.yml`'s first run | T04 | Already closed: green on PR #5 |
| The T00 coverage `mkdir`; the `[LICENSE]` EditorConfig section; the 0011 stub | T00 | Already closed: landed with T00, T03 and T09 |
| The hook-shim repair from the primary checkout | T00 | Already closed: the shim names the primary's `.pixi/envs/default/bin/prek` |
| Everything else | several | Carried; see Handed back |

**Step 3, the fresh clone.** `git clone git@github.com:steven-cutting/libpawdoku.git`
went into the session's scratch directory, and `git rev-parse HEAD` printed
`2066be727c7ed728ad9c1a2a0e12e671177babdd`.

- `time just initialize`: exit 0, **10.5 s** wall-clock (zsh `time`: `0.84s user 0.89s
  system 16% cpu 10.500 total`). It ran `pixi install --locked`, the no-argument
  `rustup toolchain install`, the cargo-hack binstall (`Done in 2.257479083s`), the
  allium 3.6.1 download, `cargo fetch --locked`, `pixi install --frozen` and
  `cargo fmt --all`. Last came `install-hooks`, whose new line
  `RUSTUP_AUTO_INSTALL=0 prek run lychee --files README.md` reported `Passed`. The shim
  went into the clone's own `.git`. `git status --porcelain` afterwards: empty.
- `time just check`: exit 0, **28.1 s** wall-clock (`98.92s user 15.34s system 406% cpu
  28.078 total`), 18 recipes, ending `The worktree matches the check baseline.` and
  `All checks passed and the worktree is unchanged.` Line coverage was **97.84%** (139
  lines, 3 missed); regions 97.72%, functions 100%. `git status --porcelain`: empty.
- The network, CONVENTIONS.md §13: `just check` again, with `HTTP_PROXY`, `HTTPS_PROXY`,
  `http_proxy`, `https_proxy` and `ALL_PROXY` set to `http://127.0.0.1:9`. Exit 0 in
  8.0 s (warm `target/`), with the same closing lines. Nothing in the gate reached the
  network.

These are a fresh clone's times on a warm machine (Apple silicon), not a new
contributor's: pixi's package cache, Cargo's registry, prek's cache at `~/.cache/prek`
and the `stable` toolchain were all present.

**Step 4, CI on `main`.** Both workflows were dispatched with `gh workflow run … --ref
main`, and both ran at `2066be7`.

- `ci.yml`, run 36270382102
  (`https://github.com/steven-cutting/libpawdoku/actions/runs/36270382102`): `success`.

  ```text
  rust       success  34 s
  coverage   success  41 s
  wasm       success  43 s
  deny       success  30 s
  documents  success  46 s
  check      success   3 s
  ```

- `audit.yml`, run 36270383635: `success` in 34 s. The log shows `cargo deny --locked
  check advisories` and `advisories ok`.
- The checks API lists `check` as `success` twice for `2066be7`: once for the merge's
  push run 36270145719 and once for this dispatch.
- Caches in the `rust` job (the `documents` job's lines are the same with `documents`
  in the rust-cache key). This is **the first rust-cache hit**, which T04 left to T11:

  ```text
  Cache hit for: pixi-linux-64-946edba9…
  Restored from cache key "v0-rust-rust-Linux-x64-492f33ac-…" full match: true.
  Cache restored from key: Linux-X64-tools-c06ff119…
  Cache restored from key: Linux-X64-prek-204cc165…
  Cache hit occurred on the primary key Linux-X64-prek-204cc165…, not saving cache.
  ```

  From `just lint` onward, the `documents` log has no clone, download, fetch, install or
  build line.

**Step 5, branch protection.** The script was extracted with
`git -C /Users/scutting/projects/biscuit_games_template show
2283589c:scripts/bootstrap_repo.sh` to `ai_tmp/bootstrap_repo.sh`. Its SHA-256 was
`62cec7df09935f2350936c36f0891f29c9d0243e21859a83d37e47050bdb6ace`, T01's value. It was
run with T01's arguments, `--hygiene` included, and without `--apply`:

```text
2. Protection on main requiring check
   state: false check false false false false false
   already
4. Private vulnerability reporting
   state: true
   already
6. Hygiene
   state: true false false
   already
changed: 0 (dry run; 0 would change)
```

The script's step 3 printed its not-applicable note ("no token supplied"), and its step
5 its note that the package is public, so no grant is needed. The protection read-back
printed `check`. **For T10:** the repository is **public** and private vulnerability reporting
is **enabled**. `SECURITY.md` can name GitHub's private reporting form as the route.

**Step 6, the CONVENTIONS.md §12 table.** There is one row per §12 bullet, 30 in all.
Each outcome is the one the named ticket recorded, with T11's own evidence added where
this ticket re-observed a claim. Pull request #11 wrote "Outcome:" sentences into four
of the bullets (rows 7, 10, 14 and 19). The rest can be closed out from this table in
one edit.

| # | Claim | Ticket | Outcome | Source |
| --- | --- | --- | --- | --- |
| 1 | No-argument `rustup toolchain install` installs the toolchain, components and targets | T00 | **holds**: nine components and both wasm targets in a fresh `RUSTUP_HOME` with `RUSTUP_AUTO_INSTALL=0`; the exit 1 came from the self-update check in a throwaway `CARGO_HOME` | T00 "What was verified", §12 list |
| 2 | `wasm32v1-none` ships `rust-std` for 1.98.1; `cargo hack check` passes on an `alloc` crate | T00, T02 | **holds**: `just wasm-check` green on both targets for T00's stub and under every configuration for T02 | T00 and T02 "What was verified" |
| 3 | `cargo update --workspace --locked` fails on drift and exits 0 otherwise | T02 | **holds**: exit 101 on a drifted copy, exit 0 on the tree; T11 re-proved it with `--offline` added (#11) | T02 "What was verified" |
| 4 | `--fail-under-lines 90` enforces the floor; `--doctests` is still nightly-only | T02 | **holds**: exit 1 at 101 and at a real 83.33% breach; `--doctests` fails on 1.98.1 | T02 "What was verified" |
| 5 | `cargo deny check licenses bans sources` runs offline; only `advisories` needs the network | T02 | **holds**: offline exit 0; offline `advisories` exit 1 without the database. T11's network-blocked `check` includes `deny` | T02 "What was verified" |
| 6 | `cargo hack check --feature-powerset` gives two configurations and exits 0 | T02 | **holds, qualified**: four configurations while `default = []` was present; two once PR #3 dropped it | T02 "What was verified", "Review follow-up" |
| 7 | `cargo shear` reads source without a build and understands `[workspace.dependencies]` | T02 | **holds, qualified**: no build and `workspace = true` understood, but 1.13.4 does not flag an unused `[workspace.dependencies]` entry | T02 "What was verified", "Handed back" |
| 8 | `thiserror` 2 derives `core::error::Error` under `no_std`; `proptest` runs under `cfg(test)` std | T02 | **holds**: 2.0.21 compiles on `wasm32v1-none`; in-module `proptest!` passes under nextest | T02 "What was verified" |
| 9 | The two `clippy.toml` `-in-tests` keys exist in clippy 1.98 | T02 | **holds**: no config error; a misspelt key gives "unknown field" | T02 "What was verified" |
| 10 | `cargo binstall --root .tools` lands in `.tools/bin`, verifies checksums, skips an installed version | T03 | **holds, qualified**: location and idempotence hold; nothing is verified, because cargo-hack publishes no checksum or signature | T03 "What was verified" 6a |
| 11 | `pixi install --locked` installs the nine conda packages and B, not `libpawdoku-tooling` | T00 | **holds** | T00 "What was verified" |
| 12 | pixi builds B from its tag with `uv_build`; six `bg-*` scripts land in the environment's `bin` | T00 | **holds**: all six present, with the environment's Python in the shebang | T00 "What was verified" |
| 13 | Every manifest pin resolves with `pixi search --platform linux-64` | T00 | **holds** on 2026-09-24 | T00 "What was verified" |
| 14 | `pixi lock --check --offline` writes nothing and exits 0 on a current lock | T03 | **holds, qualified**: true of a current lock, but a stale one that solves offline was rewritten; fixed by `--dry-run` in #11, which T11 re-proved (exit 1, both hashes unchanged) | T03 "What was verified" 6e; this ticket, step 2 |
| 15 | After `install-hooks` from an empty `PREK_HOME`, `just lint` is green with the network blocked | T03 | **holds**: all hooks passed with the proxies pointed at a closed port | T03 "What was verified" 6g |
| 16 | `pixi run -x --frozen bg-ripsecrets` passes an awkward filename unchanged | T03 | **holds**: the argv arrives intact; the switch stays optional (carried) | T03 "What was verified" 6f |
| 17 | `taplo lint` stays offline with no schema | T03 | **holds**: `toml-check` exit 0 with the network blocked | T03 "What was verified" 6b |
| 18 | typos stops at the first configuration (`_typos.toml`) | T03 | **holds**: never merged with `pyproject.toml`'s table | T03 "What was verified" 6c |
| 19 | prek 0.5.3 provisions its own rustup under `$PREK_HOME`; prek resolves through binstall | T03, D01 | **fails** (T03): prek uses the machine's rustup and the pinned 1.98.1, so no `stable` default is needed for that reason instead; the lychee warm's `stable` install is fixed in #11. D01's half **holds** | T03 "What was verified" 6d; D01 "What was verified" |
| 20 | B's checkers run unchanged with no `package.json` | T00 | **holds** | T00 "What was verified" |
| 21 | `bg-project-check` tolerates `target/`, `.tools/` and `.pixi/` growth | T00 | **holds**; again in T11's clone, where `check` ended "the worktree is unchanged" | T00 "What was verified"; this ticket, step 3 |
| 22 | No-argument `rustup toolchain install` works on the runner's rustup | T04 | **holds** on image `20260920.314.1` (not the `20260907.300.1` the claim names); the runner's rustup version was not quoted | T04 "What was verified", step 8 |
| 23 | `setup-pixi`'s activation variables are inert for cargo | T04 | **holds**: a cold build with build-script crates, no warning | T04 "What was verified", step 8 |
| 24 | `setup-pixi` restores keyed on `pixi.lock`; `cache-write` on `main` acts like `save-if` | T04 | **holds, qualified**: the key hit on T04's two dispatches and on T11's; no run observed a save gated off on a branch, because every run hit the primary key | T04 "What was verified"; this ticket, step 4 |
| 25 | The prek cache under `runner.temp` restores, and `lint` then shows no fetch | T04 | **holds**: hit on T04's dispatches and T11's; T11's `documents` log has no clone, download or build line from `lint` onward | T04 "What was verified"; this ticket, step 4 |
| 26 | `Swatinem/rust-cache` with `cache-bin: false` leaves `.tools/` to `actions/cache` | T04 | **holds**: rust-cache caches the registry, git and `target`; `.tools` restores from its own key. First rust-cache hit: T11's dispatch, full match | T04 "What was verified", step 8; this ticket, step 4 |
| 27 | The `check` aggregate reports `success` or `failure` and is listed as `check` | T04 | **holds, qualified**: the green half and the name were proved (again on `2066be7`); the red half was **not exercised**, because no gate job has failed | T04 "What was verified", step 8; this ticket, step 4 |
| 28 | allium 3.6.1 reports no diagnostics and no findings for the migrated modules | T06 | **holds**: seven modules clean in T06, eight with `board.allium` since T12; green again in T11's clone | T06 "What was verified", step 7 |
| 29 | `bootstrap_repo.sh` applies cleanly with no Pages and one required check | T01 | **holds**: the second apply changed nothing; T11's dry run printed `changed: 0` | T01 "What was verified"; this ticket, step 5 |
| 30 | The `pawdoku` name is free on crates.io | S02 | `pending`: S02 has not run | none |

**Step 7, placeholders.** The grep as first written ran at `2066be7` in the clone and
found three hits, all in one vendored skill:

```text
.agents/skills/propagate/SKILL.md:165:5. If the wiring is too complex or opaque to generate confidently, generate a test
.agents/skills/propagate/SKILL.md:180:// TODO: deferred spec — InterviewerMatching.suggest
.agents/skills/propagate/SKILL.md:280:- Cross-module tests require understanding component wiring across service boundar
```

All three are upstream prose about the placeholders the `propagate` skill generates, not
placeholders in this repository. The amended grep in step 7 prints `no placeholders`,
under both ugrep and `/usr/bin/grep`. It ran on this worktree, whose tree outside
`tickets/` is identical to `2066be7` (`git diff --quiet 2066be7 HEAD -- . ':!tickets'`).

**Step 8 and the Verification block.** The scratch clone was deleted after the commands
below had run in it.

```text
$ git -C <scratch>/t11-clone rev-parse HEAD
2066be727c7ed728ad9c1a2a0e12e671177babdd
$ gh run list --workflow ci.yml --branch main --limit 1 --json conclusion --jq '.[0].conclusion'
success
$ gh api repos/steven-cutting/libpawdoku/branches/main/protection --jq '[.required_status_checks.checks[].context] | sort | join(",")'
check
$ git ls-remote --tags origin | wc -l
       0
$ git status --porcelain            # in the clone, after check
```

In this worktree, before the commit, `git status --porcelain` printed one line, the
modified `tickets/T11-integration.md`.

### Deviations, and why

- **The branch is `T11-integration`, not `ticket/t11-integration`.** Supacode names the
  branch this way. It was kept, as T01, T04 and T06 kept theirs.
- **The follow-up pull request was written in this ticket's session.** It went on its
  own branch (`t00-followups`, pull request #11), not on this one, and the maintainer
  authorised the push and the PR separately and merged it. This branch still changes only
  its own ticket file. The maintainer chose to fold the text-only follow-ups into the
  same pull request.
- **The clone's times are on a warm machine**, as step 3 says. A cold `PREK_HOME` was
  offered and not chosen.
- **An extra offline proof.** Beyond the ticket's timed `just check`, a second run with
  every proxy variable pointed at a closed port proved §13's "none in `check`".
- **The no-`stable` half of the lychee change was not re-proved.** This machine already
  has `stable`, so the proof would need a sandbox `RUSTUP_HOME`. It rests on T03's 6d.
  The clone ran the new line, and it passed.
- **The placeholder criterion was narrowed after review.** The first grep found three
  `TODO` hits in `.agents/skills/propagate/SKILL.md`, one of the seven Allium skills T05
  keeps byte-identical to G for provenance, so they cannot be edited here. The maintainer
  first chose to carry them as a deviation. Copilot's review of pull request #12 pointed
  out that this still broke the Goal line as worded, so the maintainer chose to amend
  the criterion. The grep already excluded `allium-skill-reference/`, the other half of
  the same vendored material. The Goal line and step 7 now exclude the seven vendored
  skills as well: the `.agents/skills/` copies only, not their bridges. The grep then
  prints `no placeholders`.
- **Scratch.** `ai_tmp/bootstrap_repo.sh` sits in this worktree (gitignored). The logs
  and the clone were in the session's scratch directory, and the clone is deleted.

### Handed back

These are carried, not landed. Each has the reason it stayed out of pull request #11.

- **T10.** The repository is public, and private vulnerability reporting is enabled (step
  5). `tickets/README.md` still shows T11 `open`, because this ticket changes only its
  own file. The row goes to `done` with the next index edit.
- **`main` follow-up, `Justfile` (frozen): `test-one` and `coverage-html`** (T07, from
  PR #9's Codex review). An enhancement that needs recipe names and arguments decided.
- **`main` follow-up, `.pre-commit-config.yaml` (frozen), optional: ripsecrets through
  `pixi run -x --frozen bg-ripsecrets`** (T03). Left alone: the path form exits cleanly
  on a mangled name, and nothing breaks without the change.
- **Decision records.** Record 0008 line 43 still reads "`default = []`" (T02 asked T09
  for "no default features"). Record 0011's "pixi owns every tool binary" is overstated
  (T09 to D02). Records are final, so each needs an amendment with its reason, not a
  silent edit.
- **T03, the hook shim's structural fix** (from T00). A shim that resolves prek per
  worktree was never taken, so the primary-checkout rule in `scripts/initialize.sh`
  stands.
- **T04, `runs-on`** (from T01). `ubuntu-latest` moves to Ubuntu 26 from
  **2026-10-19**, and every job still says `ubuntu-latest`. Decide before then whether
  to pin an image.
- **Maintainer, the pickup procedure** (from T01). Supacode's `<ID>-<slug>` branch names
  differ from §11's `ticket/<id>-<slug>`, and a secondary worktree needs a
  network-authorised `pixi install --locked`, `just install-tools` and
  `just install-allium` before `just check` can run. Both call for a convention
  decision.
- **Maintainer, external.** The defects T05 found in the vendored juxt/allium material
  are for upstream. The C02 `runes` sentence goes when B's pin moves. The G hand-back
  (T06's thirteen items, confirmed by T12) belongs to a G ticket.
- **Maintainer, `deps-unused`** (from T02). cargo-shear 1.13.4 does not flag an unused
  `[workspace.dependencies]` entry. #11 recorded that fact in §12. Still undecided:
  whether an inert entry is acceptable, or whether the gate needs a second check. That
  decision belongs to whichever ticket next touches the gate.
- **Keep in step** (T08): a lint added to `[workspace.lints]` needs a mention in
  `docs/reference/configuration.md`.

### Open points settled

- **cargo-binstall and GitHub's anonymous rate limit.** The clone's `just initialize`
  ran with no `GITHUB_TOKEN`. Its one binstall lookup succeeded ("cargo-hack v0.6.45
  … has been downloaded from github.com", `Done in 2.257479083s`), so no note goes to
  `develop-locally.md`. A burst of fresh clones from one address could still hit the
  limit. That was not tested.
- **Coverage on the merged tree.** It clears the floor: 97.84% of 139 lines in the clone,
  and `coverage` green in CI. No T02 test is needed.

## Open points

- Whether the first `just initialize` on a clean machine hits GitHub's anonymous API
  rate limit through cargo-binstall's release lookup (CI sets a `GITHUB_TOKEN` on that
  step; the local script does not). Only cargo-hack is binstalled (decision 0011), one
  lookup rather than six, so this is much less likely; if it happens, the answer is a
  note in `docs/how-to/develop-locally.md`, handed back to T07.
- Whether `coverage` on the merged tree clears the 90% floor with T02's `random.rs` but
  no engine code; if not, the fix is a T02 test, never a lower floor (CONVENTIONS.md §4).
