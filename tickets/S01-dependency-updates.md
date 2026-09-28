---
id: S01
title: "Spike: dependency updates, Dependabot, Renovate or none"
status: done
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
| Dependabot | crates: `Cargo.lock` alone in range, the range only when a release falls outside it (`increase-if-necessary`); action SHAs with their comments, the composite action through a second `directories` entry (source); the toolchain `channel`; the hook revs in `.pre-commit-config.yaml` with the existing comment style (source) | every pixi pin and `pixi.lock`, the B tag, `tools.txt`, `rust-version`, `pixi-version`, the rev in `.pre-commit-fix.yaml`, allium | one `.github/dependabot.yml` with four ecosystems (a new path: T00 follow-up); no app | bot branches in the base repository with a read-only token and no secrets; no app | yes: verified, `increase-if-necessary` refreshes the lockfile in range and rewrites the range only out of range (the bare `"1.0.228"` form: T18 proof) |
| Renovate | the above natively (cargo's default is `update-lockfile`; actions with comments and the composite; the toolchain), plus `pixi-version`; through regex managers `tools.txt`, `rust-version` grouped with the toolchain, the B tag and the hook revs in both prek files; the pixi tool pins as dashboard notices | allium checksums; the `pixi.lock` relock until the hosted app's `allowedUnsafeExecutions` is proven (undocumented, default off); the B tag through the pixi manager, which skips `tag` | the Mend app (authorisation), `renovate.json` (a new path: T00 follow-up), five regex managers written and proven | the same, plus an app holding write access to contents, workflows, issues and pull requests | yes: verified, `update-lockfile` is cargo's default and keeps the ranges |
| None | nothing | everything | the routine already in `docs/operations/maintenance.md` | none | yes, by hand |

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
| `tickets/README.md` | index | the S01 row set to `done` (added in the review follow-up; Deviations) |

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
   binstalling a bumped `tools.txt` line over TLS with no checksum to verify, T03's §12
   outcome; this step first said "under cargo-binstall's checksum verification", amended
   at hand-back). Write the sentence `docs/how-to/maintain-dependencies.md` will
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
- `git status --porcelain` on the ticket branch lists only this file (amended at hand-back:
  and `tickets/README.md` from the review follow-up on, as Files touched records).

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

Run on 2026-09-26 in the Supacode worktree for this ticket, on branch
`S01-dependency-updates` (see Deviations), from `main` at `93d11d3` after T11, T10 and T12
had merged. The session crossed 00:00 UTC, so commands stamped 2026-09-27 UTC belong to
the same run. No updater was installed, no app was authorised and no page changed. Network
use the maintainer authorised on the day: read-only fetches of the Renovate and GitHub
documentation, the two bots' source repositories on GitHub where a page did not answer
(every such answer says "source" below), and read-only `gh api` and `gh pr list` calls
against this repository. One network use went beyond that authorisation and is stated
here rather than left to be noticed: this worktree had no environment, so `just initialize`
was run to reach a green gate, which built the pixi environment and downloaded cargo-hack
and the allium binary into the gitignored `.tools/`, the documented first-run path and
nothing else.

**Step 1.** The worktree existed before the ticket ran (Deviations).

**Step 2.** The inventory, on `main` at `93d11d3`:

```text
$ grep -v '^#' tools.txt
cargo-hack@0.6.45
$ grep -n 'channel' rust-toolchain.toml
2:# An exact stable release, bumped deliberately. `channel = "stable"` would move the
5:channel = "1.98.1"
$ grep -n -E 'rev:|uses:' .pre-commit-config.yaml .pre-commit-fix.yaml .github/workflows/*.yml .github/actions/setup/action.yml | wc -l
      25
$ grep -n -E '==|tag = |requires-pixi' pyproject.toml
17:requires-pixi = ">=0.81.0"
22:just = "==1.58.0"
23:prek = "==0.5.3"
25:cargo-binstall = "==1.23.0"
26:cargo-nextest = "==0.9.146"
27:cargo-llvm-cov = "==0.9.1"
28:cargo-deny = "==0.20.2"
30:cargo-shear = "==1.13.4"
31:taplo = "==0.10.0"
34:biscuit-games-tooling = { git = "https://github.com/steven-cutting/biscuit_games_tooling", tag = "v0.3.0" }
$ cargo metadata --locked --format-version 1 | jq '.packages | length'
43
```

The 25 lines are three kinds. Eight hook `rev:` lines: seven remote hooks in
`.pre-commit-config.yaml` (editorconfig-checker `v3.11.1`, markdownlint-cli2 `v0.23.2`,
typos `v1.50.2`, lychee `lychee-v0.24.2`, shellcheck-py `v0.11.0.1`, actionlint `v1.7.12`,
ripsecrets `v0.1.11`) and markdownlint-cli2 again in `.pre-commit-fix.yaml`, so one pin
lives in two files and nothing in the gate checks that the two agree. Eleven remote `uses:`
lines: actions/checkout `v7.0.1` six times, upload-artifact `v7.0.1`, setup-pixi `v0.10.2`,
rust-cache `v2.9.2` and actions/cache `v6.1.0` twice. Six local
`uses: ./.github/actions/setup` lines, which are paths, not pins. The `==` grep undercounts
the movable tool pins by one, `python = "3.14.*"`, a range whose patch release
`just lock-upgrade` moves. Two pins sit outside every grep: `pixi-version: v0.81.0` on the
setup-pixi step and `rust-version = "1.98"` in the root `Cargo.toml`. The 43 packages are
three direct dependencies (proptest, serde, thiserror) and their closure. Every row of the
Context table matches the tree. What the table gets wrong is what moves the frozen pins:
two of them now have a Dependabot ecosystem, as (f) and (g) record.

**Step 3.** Every answer names the page it came from and was read on 2026-09-26. Where the
documentation was silent and the answer came from the bot's source on GitHub, the answer
says "source"; T18 proves each such answer with a real pull request before anything relies
on it.

(a) **Dependabot `cargo` on caret ranges with a committed lockfile.** The choice exists.
`versioning-strategy` is "Supported by: bundler, cargo, composer, helm, mix, npm, pip, pub,
and uv". Its values: `auto` (the default: "For libraries, widen the allowed version
requirements to include both the new and old versions, when possible"; for apps,
`increase`), `increase`, `increase-if-necessary` ("Leave the version requirement unchanged
if it already allows the new release (Dependabot still updates the resolved version).
Otherwise widen the requirement."), `lockfile-only` ("Only create pull requests to update
lockfiles. Ignore any new versions that would require package manifest changes.") and
`widen`. So `lockfile-only` never touches `Cargo.toml` and never proposes a release outside
the range, while `increase-if-necessary` refreshes `Cargo.lock` for an in-range release and
rewrites the range for one outside it, which is decision 0007's shape: the range stays the
floor consumers inherit, and a major is a diff to read. The worked example uses an explicit
`^1.0.0`; how the bare `"1.0.228"` form the manifests use is rewritten is not stated, a
T18 proof item. Transitive crates are reachable: `allow` with `dependency-type: indirect`
lists "bundler, pip, composer, cargo, gomod, uv", and `dependency-type: all` is supported
by every package manager: "All explicitly defined dependencies. For `bundler`, `pip`,
`composer`, `cargo`, `gomod`, `uv`, also the dependencies of direct dependencies." (re-read
2026-09-27 when a review doubted it). Source:
<https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference>.

(b) **pixi.** Dependabot: no. `pixi`, `pixi.lock` and `[tool.pixi]` appear nowhere in the
supported-ecosystems table or the options reference. A `conda` ecosystem exists, but
"Dependabot support for Conda does not include private registries, vendoring, or lock file
updates" and it reads Conda files, not a pixi manifest; `pip` reads `.txt` files and PEP
621 `pyproject.toml`, and this manifest's `[project]` table says `dependencies = []`.
Source:
<https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories>.
Renovate: a `pixi` manager exists (since 39.190.0) and matches `pyproject.toml` with a
`[tool.pixi]` section and `pixi.toml`. On the lockfile: "Running `pixi lock` can execute
arbitrary code from conda package hooks, so Renovate treats `pixi.lock` refreshes as an
unsafe execution. Self-hosted administrators must explicitly allow this path by including
`pixi` in the global `allowedUnsafeExecutions` setting. When `pixi` is not allowed, the
package file is still updated, but `pixi.lock` is left unchanged." The option is global,
defaults to `[]`, cannot be set in a repository's `renovate.json`, and whether the
Mend-hosted app sets it is not documented on the manager page, the self-hosted
configuration page, the security page or the hosted-app pages. Exact conda pins
(`==1.58.0`) are extracted with conda versioning, which treats `==X.Y.Z` as a single
version (source, `lib/modules/versioning/conda/index.ts`, not run). The B tag: the
manager's schema knows a PyPI git dependency only as `{ git, rev? }`, so an entry with
`tag` and no `rev` is skipped as `unspecified-version` (source,
`lib/modules/manager/pixi/schema.ts`). Sources:
<https://docs.renovatebot.com/modules/manager/pixi/>,
<https://docs.renovatebot.com/self-hosted-configuration/#allowedunsafeexecutions>,
<https://docs.renovatebot.com/security-and-permissions/>. Consequence: a Renovate pixi
bump without the relock leaves `pixi.lock` stale, which `just lock-check` and setup-pixi's
`locked: true` both refuse, so the pull request is red rather than wrong, but it is noise
until the relock is proven on the hosted app.

(c) **Dependabot `github-actions` and the comment.** Kept: "Dependabot updates the version
documentation of GitHub Actions when the comment is on the same line, such as
`actions/checkout@<commit> #<tag or link>`". Reach: "For GitHub Actions, use the value `/`.
Dependabot will search the `/.github/workflows` directory, as well as the
`action.yml`/`action.yaml` file from the root directory." A composite action under
`.github/actions/setup/` is outside that sentence and no page says how to reach it; the
file fetcher lists YAML files in any non-root directory it is given, so
`directories: ["/", "/.github/actions/*"]` (globbing is supported by `directories`, not
`directory`) should fetch it (source, T18 proof item). Local
`uses: ./.github/actions/setup` references are ignored, rightly: they are not pins.
Sources:
<https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories>,
<https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference>,
<https://github.com/dependabot/dependabot-core/blob/main/github_actions/lib/dependabot/github_actions/file_fetcher.rb>.

(d) **Renovate `cargo` with `rangeStrategy: update-lockfile`.** It is already the default:
"When using the default rangeStrategy=auto: If a "less than" instruction is found (e.g.
`<2`) then `rangeStrategy=widen` will be selected, Otherwise,
`rangeStrategy=update-lockfile` will be selected." And "`update-lockfile` = Update the
lock file when in-range updates are available, otherwise `replace` for updates out of
range." `lockFileMaintenance`, off by default, supports `Cargo.lock` and `pixi.lock` and
"will never be grouped with other dependency updates". Sources:
<https://docs.renovatebot.com/modules/manager/cargo/>,
<https://docs.renovatebot.com/configuration-options/#rangestrategy>,
<https://docs.renovatebot.com/configuration-options/#lockfilemaintenance>.

(e) **Renovate regex managers.** The fields today are `managerFilePatterns` (formerly
`fileMatch`, migrated automatically), `matchStrings`, `depNameTemplate`,
`packageNameTemplate`, `datasourceTemplate`, `versioningTemplate`,
`extractVersionTemplate`, `currentValueTemplate` and `autoReplaceStringTemplate`; the
engine is RE2, with no lookahead and no backreferences, and matching is per file, not per
line. Datasources `crate`, `github-tags` and `github-releases` exist, and so does a
purpose-built `rust-version` datasource reading `static.rust-lang.org/manifests.txt` with
`rust-release-channel` versioning, under which `1.98` is the range
`1.98.0 <= version < 1.99.0`. Two of the three managers the ticket asked for are native.
A `rust-toolchain` manager reads `rust-toolchain.toml` (depName `rust`). The
`github-actions` manager reads `uses: owner/repo@<sha> # vX.Y.Z` as digest plus version
("Renovate will update the commit SHA according to the GitHub tag you specified"; a bare
SHA without a comment is left alone), rewrites both together, matches `.github/actions/**`
and `action.yml`, and updates the `pixi-version` input of `prefix-dev/setup-pixi` from its
documented `with:` table. `helpers:pinGitHubActionDigests` is
`{"packageRules": [{"matchDepTypes": ["action", "workflow"], "pinDigests": true}]}`. What
still needs a regex manager: the `tools.txt` line (`crate`); `rust-version` in the root
`Cargo.toml`, which the cargo manager does not extract (source, `cargo/schema.ts`); the B
tag (`github-tags` on `steven-cutting/biscuit_games_tooling`); and the hook revs, because
Renovate's native `pre-commit` manager recognises a SHA rev only as
`rev: <sha> # frozen: <version>` (source, `pre-commit/extract.ts`), not this repository's
`# v3.11.1` form. A regex manager capturing `currentDigest` and `currentValue` with an
`autoReplaceStringTemplate` is the documented set of building blocks; the exact combination
is not shown as an example, and the comment shapes here are `v3.11.1`, `v0.11.0.1` (four
components), `v1.7.12` and `lychee-v0.24.2`, so the hook manager is a T18 proof item.
Sources: <https://docs.renovatebot.com/modules/manager/regex/>,
<https://docs.renovatebot.com/configuration-options/#custommanagers>,
<https://docs.renovatebot.com/modules/datasource/rust-version/>,
<https://docs.renovatebot.com/modules/manager/rust-toolchain/>,
<https://docs.renovatebot.com/modules/manager/github-actions/>,
<https://docs.renovatebot.com/presets-helpers/>.

(f) **Renovate `pre-commit`.** Still disabled: "The `pre-commit` manager is disabled by
default and must be opted into through config", and "we have chosen to disable the manager
by default indefinitely." Its default pattern is `.pre-commit-config.ya?ml` only;
`.pre-commit-fix.yaml` would need adding. Source:
<https://docs.renovatebot.com/modules/manager/pre-commit/>. And, against the ticket's
Context, Dependabot has a `pre-commit` ecosystem (version updates only): "When a hook pins
a specific commit SHA, Dependabot resolves the latest matching tag and updates the rev
value accordingly", with a `# frozen: <version>` comment convention. Its table says a SHA
"with no `# frozen:` comment" is "Updated to the HEAD SHA of the default branch", which
would be wrong here; the source resolves the tension. The comment pattern
`(?:frozen:\s*|#\s*)(v?\d+(?:\.\d+)*)` accepts a plain `# v3.11.1`, the SHA moves to the
new tag's commit, and the comment is rewritten in place to the new tag name with its
prefix kept (source, `pre_commit/lib/dependabot/pre_commit/` at `f3a79fa0`). Its file
fetcher reads one file matching `\.pre-commit(-config)?\.ya?ml$`, so `.pre-commit-fix.yaml`
is never read. Sources:
<https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories>,
<https://github.com/dependabot/dependabot-core/tree/f3a79fa0711aca171b2174b855d9f21b16237961/pre_commit/lib/dependabot/pre_commit>.

(g) **The toolchain and `rust-version` together.** Dependabot: no, but closer than the
ticket assumed. It has a `rust-toolchain` ecosystem ("Versioned toolchains such as
`channel = "1.xx.yy"` and `channel = "1.xx"`"), whose updater fetches only
`rust-toolchain.toml` and `rust-toolchain` and rewrites them with a whole-file substitution
of the old channel string (source, `rust_toolchain/.../file_updater.rb`); `Cargo.toml` is
untouched. The substitution is safe today because the comment in `rust-toolchain.toml`
says `channel = "stable"`, not the version; a comment that quoted the version would be
rewritten too. Renovate: yes, with two managers in one group: the native `rust-toolchain`
manager and a regex manager on `rust-version = "1.98"` with `datasourceTemplate:
rust-version`, `versioningTemplate: rust-release-channel` and `depNameTemplate: rust`,
grouped with `matchDepNames: ["rust"]` (`custom.regex` inside `matchManagers` is not
documented). Under that versioning `1.98` moves only on a minor release, so a `1.98.1` to
`1.98.2` pull request carries the toolchain alone, which is right: `rust-version` follows
the pin at minor precision. Neither bot re-runs clippy; CI's `rust` job does, on the bot's
branch, and a red `clippy` there is the lint set moving, not a regression. Sources:
<https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories>,
<https://github.com/dependabot/dependabot-core/tree/f3a79fa0711aca171b2174b855d9f21b16237961/rust_toolchain/lib/dependabot/rust_toolchain>,
<https://docs.renovatebot.com/modules/manager/rust-toolchain/>,
<https://docs.renovatebot.com/modules/versioning/rust-release-channel/>,
<https://docs.renovatebot.com/configuration-options/#groupname>.

Also verified, same date. Renovate `schedule` accepts cron and a deprecated subset of Later
syntax ("We recommend you always use Cron syntax"), so the ticket's `"before 6am on
monday"` becomes `"* 0-5 * * 1"` or the `schedule:earlyMondays` preset; `config:recommended`
is current and `config:base` migrates to it; `dependencyDashboardApproval` works inside
`packageRules` and turns the dashboard on by itself. Dependabot: `.github/dependabot.yml`
on the default branch with `version: 2` is the enabling act for a non-fork, with no app
and no setting; a three-day cooldown applies even when none is configured; `groups` and
`multi-ecosystem-groups` are documented without a preview label; runs its events trigger
"receive a read-only GITHUB_TOKEN and do not have access to any secrets that are normally
available". The Mend app installs from github.com/apps/renovate on all or selected
repositories and asks for read on Dependabot alerts, Administration and Metadata, and
read and write on Checks, Code, Commit statuses, Issues, Pull requests and Workflows;
Community Cloud runs one job per organisation every four hours. Self-hosted Renovate needs
a PAT with `repo` and `workflow` scopes, or an app installation token, stored as a secret,
which CONVENTIONS.md §11 forbids, so "Renovate" in this ticket means the Mend app only.
Sources: <https://docs.renovatebot.com/configuration-options/#schedule>,
<https://docs.renovatebot.com/presets-config/>,
<https://docs.renovatebot.com/security-and-permissions/>,
<https://docs.renovatebot.com/mend-hosted/overview/>,
<https://docs.renovatebot.com/modules/platform/github/>,
<https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/configure-version-updates>,
<https://docs.github.com/en/code-security/reference/supply-chain-security/troubleshoot-dependabot/dependabot-on-actions>.

Account and repository state, read with `gh` on the day. `gh api /user/installations`
answered 403 ("You must authenticate with an access token authorized to a GitHub App in
order to list installations"), so whether the Mend app is installed on the account cannot
be read from `gh`; T18's authorisation step reads it at github.com/settings/installations
instead. On the repository, `gh pr list --state all --author app/renovate` and
`--author app/dependabot` each count 0, Dependabot alerts answer 404,
`automated-security-fixes` reports `enabled: false`, and branch protection on `main`
requires the `check` context, so no bot can merge anything.

**Step 4.** The trust surface of a bot branch is narrower than C04's, and one premise in
the ticket was wrong. The five jobs run with `contents: read`, no secret, and the run's own
token; a Dependabot branch's token is read-only by GitHub's rule, and a Renovate branch is
an ordinary branch in the base repository. What a bump can make CI run is: `just
install-tools`, which binstalls the `tools.txt` line, and cargo-hack "publishes no checksum
or signature, so nothing verifies it beyond the transport" (T03's §12 outcome, already on
`docs/explanation/security-model.md`), not a checksum verification as this ticket's step 4
assumed; setup-pixi with `locked: true` and `just sync`, which install what `pixi.lock` and
`Cargo.lock` name by hash and refuse a lockfile that disagrees with its manifest; and
`just install-hooks`, which clones the hook revs. `main` takes nothing without a green
`check`, and that context is bound to the GitHub Actions app, so no other app can report
it. Protection requires no review, though, so "a person's merge" is policy, not something
protection enforces: an app holding pull-request and contents write could merge its own
green pull request, and Renovate's automerge is off by default. T18 either requires one
approving review on `main` before the app is installed or records that the guarantee rests
on automerge staying off. What Renovate adds that Dependabot does not is the app itself:
write access to contents, workflows, issues and pull requests on the repository, held by
Mend, which is the one trust a bot branch extends beyond a person's.

The sentence for `docs/how-to/maintain-dependencies.md`, replacing "Nothing updates either
lockfile for you. There is no Dependabot or Renovate here until ticket S01 decides how
updates should arrive." in its opening paragraph, once T18 lands:

> Renovate proposes the moves, on a Monday branch of its own: `Cargo.lock` within the
> caret ranges, and a range itself when a release falls outside it, which is the diff to
> read; action SHAs with their comments, the toolchain with `rust-version`, the
> hook revs, the `tools.txt` line and, held on its dashboard until you approve them, a tool
> pin in `pyproject.toml` or the tooling package's tag. Its branch runs the five jobs like
> anyone's, with `contents: read` and no secret, so what a bump can run in CI is
> `just install-tools` fetching the `tools.txt` line over TLS with nothing but the
> transport to verify it, and setup-pixi and `just sync` installing what the two lockfiles
> name by hash; `main` still takes nothing without a green `check` and your merge, so read
> the diff as you would anyone's. The app holds write access to this repository's
> contents, workflows, issues and pull requests, which is the one trust it is given that a
> contributor's branch is not.

If the fallback is taken, the same paragraph with "Dependabot" for "Renovate", "`Cargo.lock`
within the caret ranges, and a range itself when a release falls outside it, which is the
diff to read; action SHAs with their comments, the toolchain and the hook revs in
`.pre-commit-config.yaml`" for the list, and the last sentence dropped. For
`docs/operations/maintenance.md`, replacing "Nothing here moves on its own. No bot opens a
pull request for any of these pins until ticket S01 decides whether one should.":

> Renovate opens the pull requests for the pins above each Monday and holds the rest on
> its dashboard for approval; what no bot moves, `pixi.lock` until its relock is proven,
> the lychee `LYCHEE_VERSION` argument beside its rev, and the allium checksums, still
> moves by hand as the how-to describes.

**Step 5.** The decision, with the premise corrected first. The ticket's starting
recommendation rested on Renovate being "the only option that reaches the frozen pins".
That is no longer true: Dependabot now has `rust-toolchain` and `pre-commit` ecosystems,
and its pre-commit updater handles this repository's `rev: <sha> # vX.Y.Z` form correctly
while Renovate's native one does not. What Renovate alone reaches is five one-line pins,
four of them through regex managers still to be proven: the `tools.txt` line,
`rust-version` grouped with the toolchain, the B tag (as a dashboard notice), the
`pixi-version` input, and the markdownlint rev repeated in `.pre-commit-fix.yaml`. Neither
bot moves the largest bucket, the ten pixi pins, on its own: Dependabot cannot see them,
and Renovate edits the manifest but relocks `pixi.lock` only if the hosted app permits
`pixi lock`, which is undocumented. Renovate can at least list them.

**Recommended: Renovate, on the condition that the maintainer installs the Mend app on
this repository, accepting an app that holds write access to its contents, workflows,
issues and pull requests in exchange for those five pins and a dashboard that shows every
pending pixi tool release.** If that trade is refused, the fallback is Dependabot with
four ecosystems and no app, which is much stronger than the ticket assumed, plus the manual
routine below for what it misses. The maintainer said on 2026-09-26 that they are open to
the app; that settles which draft is primary, and the paragraph above is what the evidence
says about the size of the difference.

The `renovate.json` draft. Every regex manager and the pixi approval rule is a T18 proof
item; the pixi manager stays on only as a dashboard notice until the relock is proven.

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": [
    "config:recommended",
    "helpers:pinGitHubActionDigests",
    "schedule:earlyMondays",
    ":dependencyDashboardApproval"
  ],
  "rangeStrategy": "update-lockfile",
  "automerge": false,
  "lockFileMaintenance": { "enabled": true, "schedule": ["* 0-3 * * 1"] },
  "pre-commit": { "enabled": false },
  "packageRules": [
    { "matchManagers": ["cargo"], "groupName": "crates" },
    { "matchManagers": ["github-actions"], "groupName": "github actions" },
    {
      "matchDepNames": ["rust"],
      "groupName": "rust toolchain",
      "dependencyDashboardApproval": true
    },
    { "matchManagers": ["pixi"], "dependencyDashboardApproval": true },
    { "matchDepNames": ["biscuit-games-tooling"], "dependencyDashboardApproval": true },
    {
      "matchDepNames": ["prefix-dev/pixi"],
      "groupName": "pixi bootstrap",
      "dependencyDashboardApproval": true
    },
    { "matchUpdateTypes": ["lockFileMaintenance"], "dependencyDashboardApproval": true },
    { "matchDepNames": ["cargo-hack"], "dependencyDashboardApproval": true },
    {
      "matchFileNames": [".pre-commit-config.yaml", ".pre-commit-fix.yaml"],
      "groupName": "hook revs",
      "dependencyDashboardApproval": true
    },
    { "matchDepNames": ["lycheeverse/lychee"], "enabled": false }
  ],
  "customManagers": [
    {
      "customType": "regex",
      "description": "tools.txt: one name@version per line, resolved on crates.io",
      "managerFilePatterns": ["/^tools\\.txt$/"],
      "matchStrings": ["(?<depName>[a-z0-9_-]+)@(?<currentValue>\\d+\\.\\d+\\.\\d+)"],
      "datasourceTemplate": "crate"
    },
    {
      "customType": "regex",
      "description": "rust-version follows the toolchain pin at minor precision",
      "managerFilePatterns": ["/^Cargo\\.toml$/"],
      "matchStrings": ["rust-version = \"(?<currentValue>\\d+\\.\\d+)\""],
      "depNameTemplate": "rust",
      "datasourceTemplate": "rust-version",
      "versioningTemplate": "rust-release-channel"
    },
    {
      "customType": "regex",
      "description": "the tooling package's release tag; approved from the dashboard",
      "managerFilePatterns": ["/^pyproject\\.toml$/"],
      "matchStrings": [
        "biscuit-games-tooling = \\{ git = \"https://github\\.com/(?<packageName>[^\"]+)\", tag = \"(?<currentValue>v\\d+\\.\\d+\\.\\d+)\" \\}"
      ],
      "depNameTemplate": "biscuit-games-tooling",
      "datasourceTemplate": "github-tags"
    },
    {
      "customType": "regex",
      "description": "requires-pixi is the floor that moves with pixi-version in the setup action",
      "managerFilePatterns": ["/^pyproject\\.toml$/"],
      "matchStrings": ["requires-pixi = \">=(?<currentValue>\\d+\\.\\d+\\.\\d+)\""],
      "depNameTemplate": "prefix-dev/pixi",
      "datasourceTemplate": "github-releases",
      "extractVersionTemplate": "^v(?<version>.*)$"
    },
    {
      "customType": "regex",
      "description": "hook revs in both prek configs: a full SHA with the release tag as its comment",
      "managerFilePatterns": ["/^\\.pre-commit-config\\.yaml$/", "/^\\.pre-commit-fix\\.yaml$/"],
      "matchStrings": [
        "- repo: https://github\\.com/(?<depName>[^/\\s]+/[^/\\s]+)\\n    rev: (?<currentDigest>[a-f0-9]{40}) # (?<currentValue>v\\d+(?:\\.\\d+)+)"
      ],
      "datasourceTemplate": "github-tags",
      "versioningTemplate": "loose",
      "autoReplaceStringTemplate": "- repo: https://github.com/{{{depName}}}\n    rev: {{{newDigest}}} # {{{newValue}}}"
    }
  ]
}
```

Why each line. `:dependencyDashboardApproval` is the landing form: it holds every update
for a dashboard click, so the file can sit on `main` before the app is installed without
the app opening anything, and T18's activation pull request is the one that removes this
line, changing nothing else in the configuration, alongside the two page rewrites that
ride in it. `config:recommended` brings the dashboard and the monorepo groups;
`helpers:pinGitHubActionDigests` keeps every action a SHA; `schedule:earlyMondays` is
`* 0-3 * * 1`, one window a week, the cron form the docs recommend. `update-lockfile` is
cargo's default written down, so the caret ranges stay and only `Cargo.lock` moves within
them (decision 0007); `automerge: false` is also the default written down, because the
"a person's merge" guarantee in step 4 rests on it; `lockFileMaintenance` moves the transitive closure of both lockfiles
weekly, in a pull request of its own that Renovate never groups with anything, on the same
cron window written directly, because the nested object takes lock-file options, not
`extends`; it is held for dashboard approval, through a `packageRules` entry matching the
`lockFileMaintenance` update type rather than a key inside the object, because it rewrites
`pixi.lock` as well as `Cargo.lock`, and the pixi relock is the unproven step. The native `pre-commit` manager stays off because it
would read the revs as tags and misfire; the regex manager owns both files. Crates and
actions each group into one pull request a week. The toolchain group and the tooling package
wait for approval on the dashboard, because each is a deliberate release: the toolchain
moves the lint set and the package moves the checker and every module's verification. The
pixi rule turns the manager into a notice: no branch is created until a person approves
one from the dashboard, and T18 approves exactly one to learn whether the hosted app
relocks `pixi.lock`; if it does not, the approvals stop and the notice is what the
maintainer acts on with `pixi update <name>`. The `tools.txt` line is held for approval because cargo-hack is
the one download nothing verifies beyond the transport (step 4), and the hook revs are
held, as one group, because a hook is code the gate runs on every commit; both are
matched by the documented `matchDepNames` and `matchFileNames`, not by the undocumented
`custom.regex` in `matchManagers`. The pixi bootstrap pair moves together, as the how-to
requires: the github-actions manager reads `pixi-version` on the setup-pixi step under the
depName `prefix-dev/pixi`, the regex manager reads the `requires-pixi` floor under the
same name, and the rule groups them into one pull request held for approval. The input's handling
is the manager page's own `with:` table, which lists `prefix-dev/setup-pixi`,
`pixi-version`, `prefix-dev/pixi`, `conda` (read 2026-09-26); the shared depName is a T18
proof item, and if the day's dashboard shows the input unread, the fix under the step-4
rule is a regex manager on the `pixi-version:` line of `.github/actions/setup/action.yml`
with the same `depNameTemplate`. Lychee is off because `LYCHEE_VERSION` in both argument
lists must match its rev comment and its tag prefix is `lychee-v`, so it moves by hand. The hook regex excludes lychee's shape by its `v\d+` start, and its match
string carries the four-space indent the two files use.

The fallback `.github/dependabot.yml` draft, taken only if the app is refused:

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
      day: monday
    # In range: Cargo.lock alone. Out of range: the range moves, a diff to read.
    versioning-strategy: increase-if-necessary
    allow:
      - dependency-type: all
    groups:
      crates:
        patterns: ["*"]
  - package-ecosystem: github-actions
    # The second entry reaches the composite action; T18 proves it.
    directories: ["/", "/.github/actions/*"]
    schedule:
      interval: weekly
      day: monday
    groups:
      github-actions:
        patterns: ["*"]
  - package-ecosystem: rust-toolchain
    directory: /
    schedule:
      interval: weekly
      day: monday
  - package-ecosystem: pre-commit
    directory: /
    schedule:
      interval: weekly
      day: monday
    ignore:
      # LYCHEE_VERSION beside the rev must move with it, by hand.
      - dependency-name: lycheeverse/lychee
    groups:
      hooks:
        patterns: ["*"]
```

Its manual routine, appended to the Monthly entry in `docs/operations/maintenance.md`:
the pixi pins (`pixi update <name>` after reading conda-forge, and `just lock-upgrade` for
Python's patch), `tools.txt`, the clippy fixes on the toolchain pull request's branch and, only when the
release is a minor, `rust-version` beside them (a patch leaves `1.98` correct), the rev in `.pre-commit-fix.yaml` on the same branch as
the hook pull request that moves markdownlint-cli2, lychee with its `LYCHEE_VERSION`, the
`pixi-version` and `requires-pixi` pair, and the tooling package's tag when B releases.
The `ignore` dependency-name form for a pre-commit hook is a T18 proof item.

The second Open point, settled the same way under either bot: the toolchain stays in the
updater. With Renovate it waits for dashboard approval and arrives as one pull request
moving `channel` and, on a minor, `rust-version`; with Dependabot it arrives moving
`channel` alone. Either way the bot's branch is a prompt, not a finished change: a person
checks it out, bumps `rust-version` only when the release is a minor and the bot did not
(a patch leaves `1.98` correct), runs `just clippy`, fixes what the new lint set finds,
and pushes onto the bot's branch under their own name, with authorisation asked for that
push. A red `rust`
job on that branch before they do is expected, and the how-to will say so, so it does not
read as a regression.

**Step 6.** The follow-up build ticket, drafted for `tickets/T18-renovate.md`. It is not
created here (this spike's Files touched has no ticket file for it); the agent that opens
it copies this block and adds the index row.

```yaml
---
id: T18
title: "Renovate: the Mend app, renovate.json, and the pages that said nothing moves"
status: open
depends_on: [S01]
parallel_with: []
branch: ticket/t18-renovate
estimated_size: S
---
```

> **Context.** S01 (hand-back notes, step 5) recommended Renovate through the Mend app,
> conditional on the maintainer accepting the app's write access, with Dependabot as the
> fallback. This ticket installs the recommendation and proves each regex manager S01
> could only draft.
>
> **Files touched.** `renovate.json` (new; a new root path, so a T00 follow-up pull request
> on `main` under CONVENTIONS.md §11, and `.github/dependabot.yml` instead if the fallback
> is taken); `docs/how-to/maintain-dependencies.md` (the opening paragraph, with S01's
> sentence, plus a section "What the updater proposes and what stays manual", and a note
> that a red `rust` job on a toolchain pull request is the lint set moving);
> `docs/operations/maintenance.md` (the closing "Nothing here moves on its own" paragraph
> and the Monthly entry); on the Renovate rail, `docs/explanation/security-model.md`, which
> owns the trust model and gains the app as a named holder of the permissions it asked
> for at installation: read on Dependabot alerts, administration and metadata, and read and
> write on checks, code (contents), commit statuses, issues, pull requests and workflows,
> as recorded in step 3's hand-back and re-read from the installation screen on the day,
> in the activation pull request beside the other two pages; if the maintainer requires a review on `main`, `docs/reference/quality-gates.md`,
> whose branch-protection paragraph says no review is required and changes in the same
> pull request as the protection change is recorded; `tickets/README.md` (the index row);
> this ticket's `status:`.
>
> **Reads.** The dashboard issue, the pull requests, the Actions runs and, on the fallback
> rail, the Dependabot job logs are read with read-only `gh` calls, which are network
> operations under AGENTS.md; the maintainer authorises them once, for this ticket, at its
> start, as S01's were on 2026-09-26, and the notes record that grant. The Renovate job log
> on the Mend developer portal sits behind a login the agent does not hold, so the
> maintainer reads it and pastes the lines the proof needs, or reads them aloud into the
> notes; the agent never asks for those credentials.
>
> **Steps.** (1) **Authorisation required, and the order matters.** First the maintainer
> decides whether `main` gains a required approving review, because the `check` context is
> bound to GitHub Actions but nothing stops an app with pull-request write from merging
> its own green pull request. If so, the change is a `gh api` PUT to
> `repos/steven-cutting/libpawdoku/branches/main/protection` that repeats the current
> payload (T01's, with `check` bound to app 15368 and `enforce_admins: false`) and sets
> `required_pull_request_reviews` to `{ "required_approving_review_count": 1 }`; not T's
> pinned `bootstrap_repo.sh`, whose PUT sends `required_pull_request_reviews: null` and
> would remove the requirement, so T11's "already, changed: 0" re-run of that script
> would then report a change until the script gains review support, which T18 hands back
> to T. If not, the hand-back notes record that the "a person's merge" guarantee rests
> on Renovate's automerge staying off, which `renovate.json` writes as
> `"automerge": false`. Then `renovate.json` lands on `main` through a T00 follow-up pull
> request (a new root path, CONVENTIONS.md §11; its push, its opening and its merge are
> three separately authorised actions, each asked for on its own, as for the activation
> below) **before** the app is installed, as S01 drafted it, whose
> `extends` already carries `":dependencyDashboardApproval"`, which holds every update for
> approval from the dashboard ("To require manual approval for all updates, add the
> `:dependencyDashboardApproval` presets to the `extends` array", Renovate's dashboard
> page). The order and that line exist because a configuration committed to the
> default branch is how Renovate documents manual onboarding: installing the app then
> processes the repository at once, with no onboarding pull request to gate it, so the
> held-for-approval state is what stands in for that gate. Validation is Renovate's own:
> the dashboard carries a config-validation warning while any error remains. No ad hoc
> `npx` run: if a local validator is wanted, it is a `Justfile` recipe outside
> `just check`, beside `check-links-online` and `audit` which also need the network, at a
> pinned renovate version, with node supplied by a pixi pin (a frozen-file change, so the
> same T00 follow-up). (2) **Authorisation required:** the maintainer installs the Mend
> Renovate app from github.com/apps/renovate on `steven-cutting/libpawdoku` only, and
> records the permissions it asked for in the hand-back notes. With every update held,
> installation produces the dashboard issue and nothing else; the agent itself opens
> nothing, and each pull request in step 4 opens only on the maintainer's own approval
> click, which is its authorisation. If the app is refused, switch to the fallback and
> skip the dashboard steps. (3) Read the dashboard: it should name cargo, github-actions,
> rust-toolchain, pixi and the five regex managers among the detected dependencies (the
> dashboard's detected-dependencies section is the extraction proof for every manager
> that has no update pending; confirm on the day that the hosted app renders it), and
> list the pending updates. After the step 4 proofs comes the activation: a pull request
> that removes the `:dependencyDashboardApproval` line, so the crates and actions groups
> flow weekly, and those two alone, while the per-rule approvals for the toolchain, pixi,
> the pixi bootstrap pair, the tooling package, the `tools.txt` line, the hook revs and
> lock-file maintenance stay, each of those opening only on a maintainer's click. The page rewrites of step 5, the security-model page among
> them, ride in this same pull request, so no commit on `main` has the updater active while the pages still
> say nothing moves; on the fallback rail they ride with `.github/dependabot.yml` for the
> same reason. Its push, its opening and its merge are three separately authorised actions
> (CONVENTIONS.md §11), each asked for on its own; the merge is a person's.
> (4) Proof, one pull request per manager, each opened by a dashboard approval and taken
> through the five checks. One rule for the whole step, stated once so it need not be
> repeated at each mention: wherever a proof says "the manager is adjusted" or "the rule
> is adjusted", that adjustment is a change to `renovate.json` (or, on the fallback rail,
> `.github/dependabot.yml`) on `main`, made through a pull request whose push, opening and
> merge are each separately authorised under CONVENTIONS.md §11, exactly as the landing in
> step 1 and the activation in step 3 are; and every other write outside the worktree that
> this ticket names, each dashboard click included, is authorised on its own. A second
> rule, for `pixi.lock`: any bot branch that edits `pyproject.toml` (a pixi pin, the
> tooling tag, the `requires-pixi` floor) or runs lock-file maintenance may leave
> `pixi.lock` stale, because the hosted app's relock is the unproven step; wherever that
> happens, the passing path is the hand relock on the bot's branch, `pixi update <name>`
> for one pin or `pixi update` for maintenance (the upgrading command `just lock-upgrade`
> runs; `pixi lock` alone would only re-resolve against the manifest), pushed with
> authorisation asked for that push, and the notes record which branches needed it. The proofs: a `crates` group that moves `Cargo.lock` and no range, or, when
> a release has fallen outside a range, the range as well, which under `update-lockfile` is
> the `replace` fallback and a diff to read, not a failure; a `github actions` group
> that moves a SHA and its comment in `ci.yml` and in the composite action; the
> `tools.txt` line; a hook rev in `.pre-commit-config.yaml` and the markdownlint rev in
> both files; the pixi bootstrap pair, one approved pull request moving `pixi-version` in the
> setup action and the `requires-pixi` floor together (if the two arrive under different
> depNames, the rule is adjusted and a later bot pull request that carries both is the
> proof); the tooling-tag manager, proven for extraction by the dashboard's
> detected-dependencies section naming `biscuit-games-tooling v0.3.0` under it, and for
> the update by a dashboard-approved pull request at B's next release, which moves the
> `tag` in `pyproject.toml` and, because `pixi.lock` records the commit behind the tag,
> needs the relock rule above to pass `lock-check` (B has no tag
> newer than `v0.3.0` on 2026-09-27, so if none arrives during T18 the update half is
> recorded as pending B, not as done); one approved pixi pin, to learn whether `pixi.lock`
> moves with it (if it does not, relock by hand on the bot's branch with
> `pixi update <name>` and, **with authorisation asked for that push**, push under your own
> name so the pull request passes `check`; record that the hosted app does not relock, and
> leave the rule as the notice it then is); one approved toolchain group: on a patch release the bot's pull request carries
> `channel` alone, which is correct; on a minor release it must carry `rust-version` too,
> and if it does not, the regex manager is adjusted and a later bot pull request that does
> is the proof, never a hand edit standing in for the manager. What is finished by hand on
> the bot's branch is `just clippy` alone, **with authorisation asked for that push** as
> for the pixi relock; and the first
> approved `lockFileMaintenance` run, which either opens a pull request that shows
> `pixi.lock` regenerated or untouched, never deleted (that mode "deletes the lock file and
> runs the relevant package manager" and the unsafe-execution rule is documented for
> manifest edits, not for maintenance), or, when both lockfiles already hold their newest
> admissible resolutions, opens nothing, in which case the proof is the Renovate job log
> for that run on the Mend developer portal showing lock-file maintenance attempted with no
> diff, quoted in the notes. If the run leaves `pixi.lock` stale, the relock rule above
> applies; if it deletes `pixi.lock` without regenerating it, the branch is not pushed to
> by hand but closed, and lock-file maintenance is turned off for the pixi manager alone
> (`"pixi": { "lockFileMaintenance": { "enabled": false } }`) through a configuration pull
> request under the first rule, so `Cargo.lock` maintenance continues and `pixi.lock` moves
> by hand as the how-to already describes. (5) Rewrite the
> pages named under Files touched, in the activation pull request of step 3 (or the
> fallback's file pull request), and run `just check-docs` there. (6) Record each proof's outcome and the branch names Renovate
> used, then `just check`, `status: done`.
>
> **If the app is refused (the fallback).** The same ticket, on a different rail.
> Files: `.github/dependabot.yml` as S01 drafted it (a new path under `.github/`, a T00
> follow-up all the same) in place of `renovate.json`, and the same two pages, with the
> manual routine S01 wrote for what Dependabot misses. Activation is the file itself
> landing on `main`: its push, its opening and its merge are the three separately
> authorised actions, and no app is installed. That merge is, knowingly, the
> authorisation for Dependabot to open its weekly pull requests, one per ecosystem on the
> file's schedule, because GitHub opens them the moment the file is on the default branch
> and offers no held-for-approval state; the maintainer is told so before the merge, and
> the agent itself opens nothing. Proof, one Dependabot pull request per ecosystem, each
> through the five checks: `cargo` moving `Cargo.lock` and no range, or, when a release has fallen outside
> a range, the range as well, which is what `increase-if-necessary` does and a diff to read,
> not a failure (its behaviour on the bare `"1.0.228"` form is the open question);
> `github-actions` moving a SHA and its comment in `ci.yml` and, through the second
> `directories` entry, in the composite action; `rust-toolchain` moving `channel`, then
> finished by hand on the bot's branch with `just clippy` and, only when the release is a
> minor, `rust-version` (a patch release leaves `1.98` correct, as on the other rail),
> with authorisation asked for that push; `pre-commit` moving a rev and rewriting its
> `# vX.Y.Z` comment in place, with the markdownlint rev in `.pre-commit-fix.yaml` moved on
> the same branch by hand, again asking first, and lychee shown to be ignored. The
> no-update exception applies as above. Acceptance on this rail: each of the four passed
> `check` or, for any pin or file a proof names that has no update available (judged per
> pin and per file, as on the other rail, so a current composite action does not block the
> `github-actions` proof that `ci.yml` carries), is proven by that ecosystem's most recent
> Dependabot update job under the repository's Insights, Dependabot tab: the job succeeded
> and its log names that file and that pin as parsed, quoted in the notes (the dependency
> graph does not show these ecosystems' detections, so it is not the proof); the two pages name Dependabot and the routine; no app is installed and no
> secret exists.
>
> **Acceptance.** Every proof pull request in step 4 passed `check`; where one failed, the
> manager was adjusted and a later pull request from the adjusted manager passed, because
> these combinations were deferred to this ticket precisely for being unproven, and an
> explained failure is not a proof. Two named exceptions, each with its passing path in
> step 4: the pixi pull request passes either by the app's relock or by the hand relock on
> its branch, and the notes say which, because `allowedUnsafeExecutions` is not this
> repository's to set; and any single dependency or location a proof names that has no update available
> while T18 runs (the tooling tag until B releases; the composite action's pins while
> `ci.yml`'s move; markdownlint while another hook moves; any pin that happens to be
> current) is proven for extraction by its detected-dependencies entry, with its update
> half recorded as pending the next release rather than as done. The exception is judged
> per dependency and per file named in step 4, not per manager, so one current pin cannot
> block a proof that another pin in the same manager can carry. The
> two pages no longer say nothing moves; on the Renovate rail the app is installed on this
> repository alone, and on the fallback rail no app is installed at all; no token is stored
> anywhere on either.

**Step 7.** The verification block, in order:

```text
$ git -C /Users/scutting/projects/biscuit_games_template show 2283589c:tickets/C04-dependency-updates.md | sed -n 47,52p
One thing an updater changes that is not a pin: it becomes the first author of a branch
in these repositories who is not a person with write access. `chromatic.yml` decides
whose code may run beside the Chromatic token by asking whether the head is a fork
(Poodl's file, lines 129-135) — a question an updater's in-repo branch passes while
carrying code nobody wrote. Step 5 closes that before the first bump PR is opened, and
the Goal below counts it as part of shipping the updater.
$ git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/how-to/maintain-dependencies.md | sed -n 11,13p
Every dependency is pinned to an exact version, in `package.json` and in
`pyproject.toml`. No `^`, no `~`. Both lockfiles are committed and marked
`linguist-generated`. Nothing updates them for you.
$ git status --porcelain
 M tickets/S01-dependency-updates.md
```

`just check`, run in this worktree after `just initialize` had installed its environment
(the first run failed `lint` on a Markdown finding in this file, since fixed), closed:

```text
allium analyse: 8 specifications, no diagnostics and no findings.

==> just check-clean
bg-project-check clean "$1"
The worktree matches the check baseline.

All checks passed and the worktree is unchanged.
```

`git status --porcelain` was empty after the commit. The review follow-up commits also
changed `tickets/README.md` (one row), so from the second commit on the porcelain listing
before each commit named two files; `just check` was green before each.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `S01-dependency-updates`, not the
  `ticket/s01-dependency-updates` the frontmatter names; Supacode names the branch after
  the worktree (`--name`), as every earlier ticket's notes record. The `branch:` field is
  left as written.
- **T12 became T18.** Step 6 names the follow-up `T12`; that id was taken by the board
  migration after this ticket was written, S02 claims `T13`, S03 claims `T14` and S04
  claims `T15` to `T17`. `T18` is the first id no spike names, chosen on 2026-09-27 after
  the reviews caught collisions in drafts numbered `T14` and then `T15`.
- **Five regex managers, not three.** Step 5 asks for three (`tools.txt`, `channel`,
  `rev:`). The toolchain needs none, because Renovate has a native `rust-toolchain`
  manager, but `rust-version` in `Cargo.toml`, the B tag and the `requires-pixi` floor
  (which must move with `pixi-version`) each need one, and the hook revs need one because
  the native `pre-commit` manager misreads them; so the draft has five and the T18 proof
  list names all five.
- **Two files, not one.** Files touched named this ticket alone, and the first commit
  changed nothing else. The review follow-up also set S01's row in `tickets/README.md` to
  `done`, as T10 and T11 did in theirs; the Files touched table carries the row and the
  step 7 record says which commit added it.
- **T's C04 has not shipped.** At the template repository's HEAD, `2283589c`, C04 is still
  `status: open` and depends on C01; no sibling repository (poodl, the hub, the template,
  the tooling package, the game) carries a `renovate.json` or `dependabot.yml`. The first
  Open point's collapse to "consume the house preset" is not available, and this
  recommendation stands alone.
- **The Context table's Dependabot facts were wrong.** Dependabot now reads
  `rust-toolchain.toml` and `.pre-commit-config.yaml`; (f) and (g) record the sources. The
  table itself is left as the record of what was believed on 2026-09-23; the options table
  in the Goal was rewritten, as the acceptance criteria ask.
- **Step 4's checksum premise was wrong.** It called cargo-binstall's checksum verification
  "T03's §12 outcome"; that outcome was that cargo-hack publishes no checksum, and the
  security-model page already says so. The trust sentence says transport only.
- **The starting recommendation's reason is gone.** Renovate is still recommended, but not
  because it alone reaches the frozen pins; step 5 gives the smaller, true reason and the
  condition it rests on.
- **Source-labelled answers.** Where a documentation page was silent, the bot's source on
  GitHub was read and the answer is marked "source" and listed as a T18 proof item. The
  ticket asked for documentation; a documented "no answer" is recorded as such rather than
  filled from memory.
- **The Mend app's presence could not be read from `gh`.** The installations endpoint
  requires app authentication. What was read instead is recorded under step 3.
- **The schedule syntax the ticket named is deprecated.** `"before 6am on monday"` is
  Later syntax; the draft uses the cron preset the docs recommend.

### Handed back

- **To T18** (drafted above): the five regex managers, the pixi approval rule, the
  Dependabot `directories` entry and the `ignore` form, each a proof item because the
  documentation shows the building blocks and not the combination; the two page edits;
  the `tickets/README.md` row.
- **To C04** in the template repository: its step 1(d) assumes Renovate's native
  `pre-commit` manager updates `rev:` SHAs, and Poodl pins them as
  `rev: <sha> # v3.11.1` (C04's own Context, lines 35-36). On 2026-09-26 that manager reads a
  SHA only in the `# frozen: <version>` form, so the house preset needs either a regex
  manager like the one drafted here or Dependabot's `pre-commit` ecosystem, which does read
  the plain comment. The pixi finding does not apply to the games, which use uv.
- **To the how-to page, via T18:** the lychee coupling (`LYCHEE_VERSION` must move with
  the rev comment, so no bot moves lychee) and the markdownlint rev that lives in both prek
  files are facts the page states in passing; once a bot moves the other hooks they become
  the exceptions the page must name.
- **Nothing to the frozen files.** No pin moved, and none needs to for the recommendation.

### Open points settled

- **Whether C04 ships first:** it has not, and it depends on C01, which is also open. The
  recommendation is written to stand alone; if a house preset arrives later, `renovate.json`
  becomes an `extends` line and the regex managers move into the preset, with the
  pre-commit finding above as C04's input.
- **Whether the toolchain bump is left out of the updater:** kept in, held for dashboard
  approval under Renovate, and treated under either bot as a prompt a person finishes on
  the bot's branch (step 5). A red `rust` job on that branch is expected, and the how-to
  will say so.

## Open points settled

## Open points

Both are settled; the answers are under "Open points settled" above, and the questions
stay here as written, the shape every finished ticket keeps.

- Whether T's C04 ships a house preset first; if so the recommendation collapses to
  "consume it" and the Rust regex managers become a C04 hand-back.
- Whether a toolchain bump is left out of any updater, because it carries a
  `rust-version` edit and a clippy re-run only a person does, and a bot's attempt
  would fail `clippy` in a way that reads as a lint regression.
