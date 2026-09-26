---
id: T10
title: "Maintainer docs: README, CHANGELOG, SECURITY"
status: done
depends_on: [T11]
parallel_with: []
branch: ticket/t10-maintainer-docs
estimated_size: S
---

# T10: Maintainer docs: README, CHANGELOG, SECURITY

## Context

T00 left four root-level documents as stubs of forty words each (T00 step 12):
`README.md`, `CHANGELOG.md`, `SECURITY.md` and `crates/pawdoku/README.md`. Every lane
since has replaced the stubs it owned; these four wait for T11 because a README should
describe a repository known to work from a fresh clone, a CHANGELOG entry should
summarise what actually merged, and the SECURITY policy's reporting route depends on
the visibility T01 chose and on whether GitHub accepted private vulnerability
reporting, which T11 recorded.

The model is G at `78d03cdf`. G's `README.md` (74 lines) opens with one sentence, the
place to play, a paragraph on what the game is, then Quick start, Check your work,
Layout, Documentation, Boundaries. G's `CHANGELOG.md` (17 lines) is Keep a Changelog
1.1.0 with SemVer (lines 5-6), one `[Unreleased]` section, prose bullets, and a link
definition of the form `https://github.com/steven-cutting/pawdoku/commits/main/` (line
17) because no tag exists. G's `SECURITY.md` (50 lines) carries both reporting routes
at lines 5-11 (private vulnerability reporting on a public repository; the owner
directly on a private one), supported versions, scope, out of scope, and "What this
project already does". The docs contract reads `README.md` and `SECURITY.md` (G's
`.pre-commit-config.yaml` line 41), so both keep 40 words and resolving links;
relative links inside these root files are allowed and lychee checks them offline. The
crate README is different: crates.io and docs.rs render it outside the repository,
where a relative link points nowhere, so it carries none.

Facts that shape the content (CONVENTIONS.md §0, §1, §2, §4, §10): the crate is
`no_std` with `alloc`, `publish = false`, Apache-2.0 with no per-file headers (decision
0006); the prerequisites are rustup (no default toolchain: T03's §12 outcome), pixi,
just and gh, not uv and not cargo-binstall (decision 0011); `just initialize` is the one
first-run command and never stages, commits, tags or pushes; `just fix` is the only
mutating recipe; CI pins every
action to a SHA and runs with `contents: read`; cargo-deny bans `rand` and `getrandom`
from the core and checks licences, bans and sources offline; `#![forbid(unsafe_code)]`
is a workspace lint; ripsecrets scans every commit with output suppressed; the Allium
binary is pinned by checksum. How the game will consume the engine is S04's spike, so
the README says "through a WebAssembly package, planned" and no more.

Read: G's three files at `78d03cdf`; `docs/README.md` and
`docs/project/purpose-and-scope.md` (T07: the README's first paragraph must agree with
it and not restate it); `docs/how-to/develop-locally.md` (T07) for the prerequisite
list the README points to; T11's hand-back notes for the visibility, the
vulnerability-reporting outcome and the timings; the "Handed back" notes of T00 to T09
for what each lane shipped, which is the CHANGELOG's material.

## Goal

Four final documents, each in the voice of G's, each true of this repository on the
day it is written, with the docs contract, markdownlint, typos and lychee green.

## Non-goals

- No handbook page; the README links into `docs/` rather than repeating it.
- No release, tag, version bump or `[0.1.0]` heading; S02 decides how a release entry
  is cut. No crates.io or docs.rs badge: a badge to a page that does not exist fails
  lychee's online check.
- No `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` or issue templates; not in CONVENTIONS.md
  §3, and a new path is a T00 follow-up.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `README.md` | stub -> final | the repository front page, step 2 |
| `CHANGELOG.md` | stub -> final | Keep a Changelog `[Unreleased]` prose, step 3 |
| `SECURITY.md` | stub -> final | the policy for a `publish = false` crate, step 4 |
| `crates/pawdoku/README.md` | stub -> final | the crate page crates.io and docs.rs show, step 5 |
| `tickets/T10-maintainer-docs.md` | ticket | `status:` and hand-back notes |

## Steps

1. Create the worktree on `ticket/t10-maintainer-docs` from `main` after T11 has merged
   (README.md "How to pick up a ticket"); `just sync && just check-docs` is green
   before any edit.

2. `README.md`, in G's order: the title and opening, then seven `##` sections:
   - Title `# libpawdoku` and one sentence: the classic-sudoku engine behind the
     Biscuit Games game Pawdoku, a Rust library. One paragraph on what it is (solver,
     technique catalogue, difficulty rating, hints, a model of a human solver), that
     every rule comes from `docs/specs/`, and that the core is `no_std` so it runs the
     same in a browser, in Python and on the command line.
   - `## Prerequisites`: D02's bootstrap story: rustup (no default toolchain: T03's
     §12 outcome; `~/.cargo/bin` ahead of `~/.pixi/bin` on `PATH`), pixi 0.81.0 or
     newer, just (`pixi global install just`; any just launches the recipes, because
     the pinned one in the environment runs every nested call and CI) and gh; not uv,
     not cargo-binstall. One line on the pixi hazard, then a link to
     `docs/how-to/develop-locally.md` for the detail.
   - `## Quick start`: a console block with `just initialize` then `just check`, and
     G's sentence adapted: what `initialize` installs (the pixi environment, the pinned
     toolchain, cargo-hack and the Allium checker into `.tools/bin`, the hook) and that
     it never stages, commits, tags or pushes.
   - `## Check your work`: G's lines 27-35 in shape (`just fix`, `just check`,
     `just --list`, the link to `docs/reference/commands.md`).
   - `## Layout`: a `text` block of CONVENTIONS.md §3 reduced to the top level:
     `crates/pawdoku/` (the engine; `src/random.rs` is the one effect boundary),
     `docs/`, `docs/specs/` (the seven Allium modules, the source of truth for
     behaviour), `tickets/`, `pyproject.toml` and `pixi.lock` (the tool manifest and its
     pin), `rust-toolchain.toml` (the compiler pin), `tools.txt` (the one exception);
     `.pixi/`, `.tools/` and `target/` (ignored).
   - `## Documentation`: G's shape; the map, then Purpose and scope, Make your first
     change, Architecture, Specifications, API reference; the `AGENTS.md` sentence.
   - `## How the game will use it`: two sentences: the game will consume the engine
     through a WebAssembly package that also ships the specification text, so the game
     can hold its restated clauses equal by test (CONVENTIONS.md §1 fact 6); that
     package, a CLI and Python bindings are planned as sibling crates and are S04's.
   - `## Boundaries`: the specification is right and the code is a defect (G's lines
     63-65); the crate is `publish = false` with no release yet; the licence is
     Apache-2.0, `LICENSE` at the root, no per-file headers.

3. `CHANGELOG.md`: G's header (lines 1-6) verbatim, then `## [Unreleased]` with
   `### Added` and ten prose bullets covering T00 to T09 in ticket order, each naming
   the thing rather than the ticket: the workspace and toolchain pin, the licence, the Justfile
   gate and hooks, the GitHub repository and CI (five checks, weekly audit), the
   `no_std` crate skeleton with the randomness boundary and the lint tables, the
   dotfiles and the pixi environment, the agent contract with fourteen skills, the
   seven engine modules migrated from Pawdoku with the two explanation pages, the
   handbook of twenty-five pages, the eleven decision records. Close with
   `[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/` (G's line
   17 form; T01 created the remote, so it resolves online; no compare link until a tag
   exists, which S02 decides).

4. `SECURITY.md`: G's five sections. Reporting: if T11 recorded private vulnerability
   reporting as enabled, G's lines 5-7; if `404`, G's lines 9-11 adapted. Supported
   versions: `main` only; no released version, no backports, not on crates.io. Scope:
   the crate's code (a puzzle input that makes the solver run past its step budget or
   allocate without bound counts), credential material in the repository, a dependency,
   tool binary or GitHub Action tampered with, a lockfile that disagrees with its
   manifest. Out of scope: anything a caller does with its own seed or entropy source.
   What it already does: `#![forbid(unsafe_code)]` in every crate; no network, clock,
   filesystem or environment access in the core, made unnameable by `no_std`;
   `Cargo.lock` committed and every gate `--locked`; `cargo deny` over licences, bans
   and sources on every check and advisories weekly; `rand` and `getrandom` banned from
   the core; tool binaries pinned by hash in `pixi.lock` and installed from conda-forge,
   the compiler by rustup from `rust-toolchain.toml`; every GitHub Action and hook
   pinned to a commit SHA; CI with `contents: read` and no stored secret; ripsecrets on
   every commit with output suppressed; the Allium binary pinned by SHA-256. Close with
   the link to `docs/explanation/security-model.md`.

5. `crates/pawdoku/README.md`: what crates.io and docs.rs show. Eight to fifteen lines:
   the crate name, one paragraph on what it does and that it is `no_std` with `alloc`,
   one on the randomness boundary (the caller supplies the seeded stream), one line
   naming the specifications directory in a code span, one line on the licence. No
   relative link, no badge, no example code (the crate-level doc comment in `lib.rs`
   owns the example; a README copy is one no test checks).

6. Run `just check-docs` then `just check` and quote the output.

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- `README.md` has the seven `##` sections of step 2 in that order, links only to pages
  that exist, and repeats no paragraph of `docs/project/purpose-and-scope.md`.
- `CHANGELOG.md` has exactly one section heading, `[Unreleased]`, ten bullets covering
  T00 to T09 under `### Added`, and one link definition.
- `SECURITY.md` names the reporting route T11 recorded and lists every item of step 4's
  "what it already does" that is true on the day (drop one only with a reason).
- `crates/pawdoku/README.md` contains no `](` and no `http`.
- `just check-docs` and `just check` are green; nothing outside the four files and this
  ticket changed.

## Verification

```sh
grep -n '^## ' README.md
grep -c '^- ' CHANGELOG.md
grep -n 'commits/main' CHANGELOG.md
grep -n -E '\]\(|http' crates/pawdoku/README.md || echo "crate README has no links"
just check-docs
git status --porcelain
```

Expected: seven `##` lines in step 2's order; `10`; one line; `crate README has no
links`; every page valid; the four documents and this file.

## Hand-back notes

Done on 2026-09-26 in the worktree `../T10-maintainer-docs` on branch
`ticket/t10-maintainer-docs`, stacked on `T11-integration` at `1e083e8` (see Deviations).

### What was verified, and how

**Step 1.** The worktree was created with `git worktree add`. The maintainer authorised
the network, so `pixi install --locked`, `just install-tools` (cargo-hack 0.6.45) and
`just install-allium` (allium 3.6.1) ran there. `just sync && just check-docs` was then
green on the untouched stubs: markdownlint, typos and lychee `Passed`, and
`Validated 37 pages and 38 canonical topics.`

**Steps 2 to 5, the facts the documents state.** Each claim was read from the tree on
the day, not carried from the ticket:

- Code: `lib.rs` exports `random` and `SIDE`, and nothing else. There is no solver,
  rating or hint code yet, so both READMEs say the engine is built one specified module
  at a time and the crate so far holds the randomness boundary. `docs/specs/` holds
  eight modules.
- Counts: there are fourteen skills under `.agents/skills/`. CONVENTIONS.md §6 counts
  twenty-five handbook pages, and there are eleven records under `docs/decisions/`.
  `ci.yml` has five gate jobs (`rust`, `coverage`, `wasm`, `deny`, `documents`) behind
  the aggregate `check`. `audit.yml` runs on `pull_request` and on a Monday 06:00 UTC
  cron. The `Justfile`'s `check` runs the seventeen gates `pyproject.toml` lists.
- `forbid(unsafe_code)`: `unsafe_code = "forbid"` is in `[workspace.lints.rust]`.
- `deny.toml`: it bans `getrandom` and `rand` (wrappers `pawdoku-cli` only) and
  `openssl-sys`; `[sources]` allows only the crates.io index.
- SHA pins: every `uses:` in both workflows and in `.github/actions/setup/action.yml`
  (setup-pixi, rust-cache, actions/cache twice) is pinned to a commit SHA. So is every
  remote `rev:` in `.pre-commit-config.yaml`.
- Tokens and permissions: both workflows declare `contents: read`. The only token is
  `${{ github.token }}` in the setup action.
- Tool pins: `pixi.lock` pins `biscuit-games-tooling` as
  `?tag=v0.3.0#6c5c07f6…`. Decision 0005 records the Allium binary pinned by version
  and SHA-256.

**Step 6, the Verification block** (run before the ticket edit):

```text
$ grep -n '^## ' README.md
11:## Prerequisites
26:## Quick start
37:## Check your work
47:## Layout
61:## Documentation
73:## How the game will use it
80:## Boundaries
$ grep -c '^- ' CHANGELOG.md
10
$ grep -n 'commits/main' CHANGELOG.md
33:[Unreleased]: https://github.com/steven-cutting/libpawdoku/commits/main/
$ grep -n -E '\]\(|http' crates/pawdoku/README.md || echo "crate README has no links"
crate README has no links
$ just check-docs
markdownlint.............................................................Passed
typos....................................................................Passed
lychee...................................................................Passed
bg-validate-docs
Validated 37 pages and 38 canonical topics.
$ git status --porcelain
 M CHANGELOG.md
 M README.md
 M SECURITY.md
 M crates/pawdoku/README.md
```

`CHANGELOG.md` has exactly one `##` heading. `crates/pawdoku/README.md` is 15 lines.

`time just check`: exit 0 in 26.6 s (`93.19s user 14.09s system 403% cpu 26.597
total`), 18 recipes, 22 tests passed. Line coverage was 97.84% (139 lines, 3 missed). The
run ended with `The worktree matches the check baseline.` and `All checks passed and the
worktree is unchanged.` That run preceded this ticket edit. `just check` was run again on the
committed tree: exit 0 in 6.6 s with a warm `target/`, and the same two closing lines.

`README.md` repeats no paragraph of `docs/project/purpose-and-scope.md`. Four phrases
from its opening paragraph were searched for with `grep -F` in that page, and none
matched.

### Deviations, and why

- **Stacked on T11, not branched from `main`.** Pull request #12 (T11) had not merged
  when this ticket was picked up, and the maintainer chose to stack this branch on
  `T11-integration` at `1e083e8`. The two touch no common file. The maintainer then chose
  to push this commit onto `T11-integration` itself, so pull request #12 carries both
  tickets and no separate T10 pull request exists.
- **The branch is `ticket/t10-maintainer-docs`, as the ticket names it.** The worktree
  was made with `git worktree add` rather than by Supacode, so no rename arose.
- **Eight modules, not seven.** T12 added `board.allium` after this ticket was written.
  The README's Layout says eight. The CHANGELOG folds it into the migration bullet, which
  keeps the count at ten. Leaving it out would have made the entry untrue of what
  merged.
- **Written true of the day.** Step 2 asks for a paragraph on what the engine is (solver,
  techniques, rating, hints). The crate holds only the randomness boundary, so both
  READMEs describe the engine as specified and being built, not as shipped. The SECURITY
  scope bullet says "the solver, once it exists".
- **The conda-forge item is qualified, not dropped.** Step 4 has "tool binaries pinned by
  hash in `pixi.lock` and installed from conda-forge". Three tools come from elsewhere,
  and SECURITY.md names all three. `biscuit-games-tooling` comes from its Git tag, pinned
  to a commit in `pixi.lock`. The Allium checker comes from its GitHub release, pinned by
  version and SHA-256 in that package (decision 0005). cargo-hack comes from its GitHub
  release over TLS with no checksum, as `docs/explanation/security-model.md` already says.
  Step 4's separate Allium item is folded into this one.
- **One out-of-scope line added.** "What a consumer does with the engine's output"
  follows the security model's "What is out of scope". G's private-repository paragraph
  (lines 9-11) is dropped, because the repository is public.
- **CI's "five checks"** are written as five gate jobs behind one required `check`,
  because branch protection requires only the aggregate.

### Review follow-up

On the maintainer's instruction, `6829050` was pushed onto `T11-integration` (pull
request #12), and Codex and Copilot were asked to review again. Every finding was taken:

- **Codex, `SECURITY.md` line 44.** The exception list left out the Allium checker,
  which `just initialize` downloads from its GitHub release and no lockfile names. It
  is now the third item, pinned by version and SHA-256.
- **Codex, `SECURITY.md` line 27.** "Anything a caller does with its own seed" was
  broader than the security model's "how it sources the seed it passes in". Read that
  way, it would have excluded a crash that a particular valid seed triggers. The line
  now excludes only how the seed or entropy is sourced.
- **Copilot, `tickets/T11-integration.md` line 414.** T11 said its branch changes only
  its ticket file, which the stack made false. T11's Deviations and this ticket's now
  record the stack, and the pull request's title and description cover both tickets.

### Handed back

- **`tickets/README.md`** still shows T10 and T11 as `open`, because this ticket changes
  only its four documents and its own file. Both rows go to `done` with the next index
  edit.
- **The CHANGELOG's counts** (seventeen gates, five jobs, fourteen skills, twenty-five
  pages, eleven records) go stale when any of them changes. They describe what merged
  under `[Unreleased]` and need no edit until S02 cuts the first release entry.

### Open points settled

- **Timings in the README: no.** The README states none. The figures stay in T11's
  hand-back, and `docs/operations/troubleshooting.md` is their home if they are wanted.
- **`include_str!` for the crate README: no.** The crate-level doc comment in `lib.rs`
  stays hand-written, as T00 and T02 left it. S02 may revisit this at publication.

## Open points

- Whether the README should state the `just check` timings T11 measured; the
  recommendation is no (they belong in `docs/operations/troubleshooting.md` and go
  stale), but a maintainer may want one sentence on the cold-start cost.
- Whether `crates/pawdoku/README.md` should be pulled into the crate-level docs with
  `include_str!`. T00 wrote the doc comment in `lib.rs` by hand and T02 kept it so;
  this ticket follows them, and S02 may revisit at publication.
