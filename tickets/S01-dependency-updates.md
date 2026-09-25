---
id: S01
title: "Spike: dependency updates, Dependabot, Renovate or none"
status: open
depends_on: [T11]
parallel_with: []
branch: ticket/s01-dependency-updates
estimated_size: S
---

# S01: Spike: dependency updates, Dependabot, Renovate or none

## Context

Nothing in this repository moves a pin. `docs/how-to/maintain-dependencies.md` (T07,
CONVENTIONS.md §6) says "nothing updates them" until this spike decides otherwise, as
G's page does at `78d03cdf` lines 11-13 ("Nothing updates them for you"). The house
precedent is T's `tickets/C04-dependency-updates.md` at `2283589c`: its Context (lines
15-39) counts four ecosystems pinned four ways, its Goal recommends Renovate with a
shared preset and Dependabot as the fallback, and lines 47-52 name the one thing an
updater changes that is not a pin: it becomes the first author of a branch in the
repository who is not a person with write access. Here the pins are more varied:

| Pin | File | Owner | Moved by |
| --- | --- | --- | --- |
| Crate versions | `Cargo.toml` caret ranges, `Cargo.lock` the pin (decision 0007) | T02 | `cargo update` |
| GitHub Actions | `ci.yml`, `audit.yml`, `.github/actions/setup/action.yml`, SHA plus version comment | T04 | by hand |
| Every tool, Python, prek and the B tag | `pyproject.toml` and `pixi.lock` (decision 0011) | T00 | `pixi update <name>` |
| The toolchain | `rust-toolchain.toml` `channel = "1.98.1"` (frozen, CONVENTIONS.md §2) | T00 follow-up | by hand, with `rust-version` and a clippy re-run (§13) |
| cargo-hack | `tools.txt` (frozen, one line) | T00 follow-up | by hand |
| Hook revisions | both prek configs, `rev:` SHA plus version comment (frozen) | T00 follow-up | by hand |
| The Allium binary | B's `install_allium.py` version and four checksums | B | a B release, then the tag in `pyproject.toml` |
| Bootstrap pin | pixi 0.81.0: `requires-pixi` in `pyproject.toml` and `pixi-version` in the setup action | T00, T04 | by hand |

Facts to start from and verify at execution (reasoned from the tools' documentation on
2026-09-23, none run): Dependabot has `cargo` and `github-actions` ecosystems and can
group updates weekly, but reads no `rust-toolchain.toml`, `tools.txt`, nextest-version
or prek `rev`, and no `uv` ecosystem applies here; whether Dependabot or Renovate
understands `pixi.lock` is a fact to verify (Renovate documents a pixi manager; verify
on the day). Renovate reaches the frozen pins through regex custom managers (the one
`tools.txt` line as `name@version` against the crates.io datasource, the toolchain
`channel` against Rust releases, a prek `rev` with its version comment against GitHub
tags) and pins action digests with `helpers:pinGitHubActionDigests`, but needs the Mend
app installed on the account (T's C04 step 2 records the same authorisation). Neither
recomputes the Allium checksums. Three constraints are this repository's own: the
frozen files change only through a `main` pull request, which an updater's is; every
gate runs `--locked`, so a range bump without a relock fails `lock-check` rather than
drifting; and a toolchain bump must also bump `rust-version` and re-run clippy, which
no updater does.

The trust question is smaller than C04's: `ci.yml` runs with `contents: read` and no
secret, and nothing runs installed code beside a token. What remains is
`just install-tools` binstalling whatever a bump writes into `tools.txt` (cargo-hack
alone), and `just sync` and `setup-pixi` installing whatever `Cargo.lock` and
`pixi.lock` name, all on the bot's branch in CI.

Read first: CONVENTIONS.md §1 fact 8, §2, §11, §13 (the last risk);
`docs/decisions/0007-dependency-policy.md`; `docs/how-to/maintain-dependencies.md`;
T's C04 in full; G's `docs/how-to/maintain-dependencies.md` lines 1-40.

## Goal

A recommendation with evidence in the hand-back notes and, where one is taken, a
drafted build ticket. Nothing is installed, no app is authorised, no file outside this
ticket changes. The starting recommendation: **Renovate if the maintainer accepts the
Mend app**, because it is the only option that reaches the frozen pins; otherwise
**Dependabot for `cargo` and `github-actions`, plus a written manual routine** in
`docs/operations/maintenance.md` for the rest.

| Option | Reaches | Misses | Cost to adopt | Trust surface | Fits decision 0007 |
| --- | --- | --- | --- | --- | --- |
| Dependabot | crates, actions with SHA; `pixi.lock` only if verified | toolchain, `tools.txt`, hook revs, allium, the pixi pin | one `.github/dependabot.yml` (a new path: T00 follow-up) | bot branches in the base repository; no app | verify: relock only, or rewrite the caret range |
| Renovate | the above plus `pixi.lock` through its pixi manager (verify) and the frozen-file pins via regex managers and `pinDigests` | allium checksums; the `rust-version` half of a toolchain bump | the Mend app (authorisation), `renovate.json`, three regex managers written and tested | the same, plus an app with repository access | verify: `rangeStrategy: update-lockfile` keeps ranges |
| None | nothing | everything | a routine in `docs/operations/maintenance.md` | none | yes, by hand |

## Non-goals

- Installing either app, adding a configuration file or moving any pin: the follow-up
  build ticket does that after the maintainer chooses.
- A shared house preset (T's C04 owns that; if it ships first, this repository's file
  is one `extends` line and the rest of this spike is moot).
- Changing decision 0007 or the frozen files.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S01-dependency-updates.md` | ticket | the evidence, the recommendation, the drafted follow-up; `status: done` |

## Steps

1. Create the worktree on `ticket/s01-dependency-updates` from `main` after T11 has
   merged (README.md "How to pick up a ticket").

2. Inventory the pins on `main` today, so the table above is checked, not trusted:

   ```sh
   grep -v '^#' tools.txt
   grep -n 'channel' rust-toolchain.toml
   grep -n -E 'rev:|uses:' .pre-commit-config.yaml .pre-commit-fix.yaml .github/workflows/*.yml .github/actions/setup/action.yml | wc -l
   grep -n -E '==|tag = |requires-pixi' pyproject.toml
   cargo metadata --locked --format-version 1 | jq '.packages | length'
   ```

3. Verify, read-only, against each tool's current documentation, recording each answer
   with its source: (a) Dependabot `cargo` on caret ranges with a committed lockfile:
   does a bump rewrite the range, refresh only `Cargo.lock`, or both, and can that be
   chosen; (b) whether Dependabot or Renovate understands `pixi.lock` (Renovate
   documents a pixi manager), moves an exact conda pin in `pyproject.toml`, and moves a
   PyPI git dependency pinned to a tag; (c) whether Dependabot `github-actions` keeps
   the `# vX.Y.Z` comment beside a SHA it moves; (d) Renovate `cargo` with
   `rangeStrategy: update-lockfile`; (e) a Renovate regex manager for the `tools.txt`
   line (datasource `crate`), for `channel` (a Rust release datasource) and for `rev:`
   with the comment; (f) whether Renovate's `pre-commit` manager is still disabled by
   default (T's C04 line 107); (g) whether either bot can move `rust-toolchain.toml` and
   `rust-version` together (expected: no).

4. Settle the trust surface for a bot branch (the five jobs with `contents: read`;
   `setup-pixi` installing a bumped `pixi.lock` by hash; `just install-tools`
   binstalling a bumped `tools.txt` line under cargo-binstall's checksum verification,
   T03's §12 outcome). Write the sentence `docs/how-to/maintain-dependencies.md` will
   carry.

5. Weigh the options and decide. If Renovate: draft `renovate.json` in the hand-back
   notes (schema, `config:recommended`, `helpers:pinGitHubActionDigests`,
   `rangeStrategy`, a grouped weekly schedule, the three regex managers, and a
   `packageRules` entry holding the B tag for dashboard approval as C04 line 131 does
   for the hub package). If Dependabot: draft `.github/dependabot.yml` with two
   ecosystems, weekly, grouped, and the manual routine for what it misses. If none:
   the routine alone.

6. Draft the follow-up build ticket: id `T12`, files (the configuration file,
   `docs/how-to/maintain-dependencies.md`, `docs/operations/maintenance.md`), the
   authorisation step, the proof (one bot pull request per ecosystem through the five
   checks), and the note that a new root path is a T00 follow-up (CONVENTIONS.md §11).

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (g) are recorded with the documentation they came from and the date.
- The options table is completed against the tree inventoried in step 2.
- Exactly one option is recommended, with its condition stated, and the follow-up
  draft names its files and authorisation steps.
- The sentence for `docs/how-to/maintain-dependencies.md` is written.
- `git status --porcelain` on the ticket branch lists only this file.

## Verification

```sh
git -C /Users/scutting/projects/biscuit_games_template show 2283589c:tickets/C04-dependency-updates.md | sed -n 47,52p
git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/how-to/maintain-dependencies.md | sed -n 11,13p
git status --porcelain
```

Expected: C04's trust paragraph; G's "Nothing updates them for you" lines; one line
naming this file.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether T's C04 ships a house preset first; if so the recommendation collapses to
  "consume it" and the Rust regex managers become a C04 hand-back.
- Whether a toolchain bump is left out of any updater, because it carries a
  `rust-version` edit and a clippy re-run only a person does, and a bot's attempt
  would fail `clippy` in a way that reads as a lint regression.
