---
id: T10
title: "Maintainer docs: README, CHANGELOG, SECURITY"
status: open
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
0006); the prerequisites are rustup (no default toolchain: T03's §12 outcome),
cargo-binstall, just, gh and, under D01 option A, uv; `just initialize` is the one
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
   - `## Prerequisites`: rustup (no default toolchain: T03's §12 outcome),
     cargo-binstall, just, gh; uv under D01 option A. One line on the pixi hazard, then
     a link to `docs/how-to/develop-locally.md` for the detail.
   - `## Quick start`: a console block with `just initialize` then `just check`, and
     G's sentence adapted: what `initialize` installs (the pinned toolchain, the tool
     binaries into `.tools/bin`, the Allium checker, the hook) and that it never
     stages, commits, tags or pushes.
   - `## Check your work`: G's lines 27-35 in shape (`just fix`, `just check`,
     `just --list`, the link to `docs/reference/commands.md`).
   - `## Layout`: a `text` block of CONVENTIONS.md §3 reduced to the top level:
     `crates/pawdoku/` (the engine; `src/random.rs` is the one effect boundary),
     `docs/`, `docs/specs/` (the seven Allium modules, the source of truth for
     behaviour), `tickets/`, `tools.txt` and `rust-toolchain.toml` (the pins),
     `.tools/` and `target/` (ignored).
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
   dotfiles and pinned tool installs, the agent contract with fourteen skills, the
   seven engine modules migrated from Pawdoku with the two explanation pages, the
   handbook of twenty-five pages, the ten decision records. Close with
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
   the core; tool binaries pinned in `tools.txt` and installed from release checksums;
   every GitHub Action and hook pinned to a commit SHA; CI with `contents: read` and no
   stored secret; ripsecrets on every commit with output suppressed; the Allium binary
   pinned by SHA-256. Close with the link to `docs/explanation/security-model.md`.

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

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether the README should state the `just check` timings T11 measured; the
  recommendation is no (they belong in `docs/operations/troubleshooting.md` and go
  stale), but a maintainer may want one sentence on the cold-start cost.
- Whether `crates/pawdoku/README.md` should be pulled into the crate-level docs with
  `include_str!`. T00 wrote the doc comment in `lib.rs` by hand and T02 kept it so;
  this ticket follows them, and S02 may revisit at publication.
