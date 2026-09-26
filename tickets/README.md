# Tickets for bootstrapping `libpawdoku`

This directory is the work breakdown for setting up `steven-cutting/libpawdoku`, the Rust
sudoku engine behind the Biscuit Games game Pawdoku. Each ticket is written for an AI agent
with no other context, working in its own git worktree. `CONVENTIONS.md` is the shared
design every ticket obeys and cites by section; read it first, then the ticket.

Creating the GitHub repository, pushing, tagging, filing these as GitHub issues, opening
pull requests and editing any other repository are separately authorised actions. Nothing
here has been filed, and nothing outside `tickets/` exists yet.

## Index

Build tickets, in dependency order. The `status:` field in each ticket's frontmatter is
authoritative; this table is a snapshot.

| Id | Title | File | Depends on | Parallel with | Status |
| --- | --- | --- | --- | --- | --- |
| D01 | Hook runner and checkers: keep the Python toolchain or go Python-free | `D01-hook-runner.md` | none | none | done |
| D02 | Tool manager: pixi owns the tools, rustup keeps the compiler | `D02-tool-manager.md` | D01 | none | done |
| T00 | Foundation: workspace, toolchain, licence, Justfile, hooks, manifest, stubs for every path | `T00-foundation.md` | D01, D02 | none | open |
| T01 | GitHub repository: create, first push, settings, branch protection | `T01-github-repository.md` | T00 | T02 to T09 | open |
| T02 | Rust quality gate: tool configs, the no_std crate skeleton, every Rust recipe green | `T02-rust-gate.md` | T00 | T01, T03 to T09 | open |
| T03 | Hooks and the language-agnostic gate: dotfiles, first-run script, the pixi environment | `T03-hooks-and-dotfiles.md` | T00, D01, D02 | T01, T02, T04 to T09 | open |
| T04 | CI workflows: the composite setup action, five gate jobs and the audit job | `T04-ci.md` | T00, T01 | T02, T03, T05 to T09 | open |
| T05 | Agent contract: AGENTS.md, adapters, fourteen skills, their bridges | `T05-agent-contract.md` | T00 | T01 to T04, T06 to T09 | open |
| T06 | Specification migration: the seven engine modules, adapted, and the Allium gate | `T06-spec-migration.md` | T00 | T01 to T05, T07 to T09 | open |
| T07 | Handbook A: project, tutorial and how-to pages | `T07-handbook-a.md` | T00 | T01 to T06, T08, T09 | open |
| T08 | Handbook B: explanation, reference and operations pages | `T08-handbook-b.md` | T00 | T01 to T07, T09 | open |
| T09 | Decision records | `T09-decisions.md` | T00, D01, D02 | T01 to T08 | open |
| T11 | Integration: first green `just check` from a fresh clone and in CI, branch protection verified | `T11-integration.md` | T01 to T09 | none | open |
| T10 | Maintainer docs: README, CHANGELOG, SECURITY | `T10-maintainer-docs.md` | T11 | none | open |
| T12 | Board migration: board.allium adapted, and the alignment with the game at add73be7 | `T12-board-migration.md` | T06, T07, T08, T11 | none | open |

Spike tickets. Each weighs options and ends in a recommendation; none blocks the build and
none is picked up before T11.

| Id | Title | File | Depends on | Status |
| --- | --- | --- | --- | --- |
| S01 | Spike: dependency updates, Dependabot, Renovate or none | `S01-dependency-updates.md` | T11 | open |
| S02 | Spike: release and publishing, cargo-release or release-plz, crates.io, lifting publish = false | `S02-release-and-publishing.md` | T11 | open |
| S03 | Spike: benchmarks, fuzzing and mutation testing | `S03-bench-fuzz-mutants.md` | T11 | open |
| S04 | Spike: bindings and CLI groundwork, pawdoku-cli, pawdoku-py, pawdoku-wasm | `S04-bindings-and-cli.md` | T11, T06 | open |

Centralisation recommendations. Each is written to be picked up on its own and touches
another repository, so every step in it is separately authorised.

| Id | Title | File | Depends on | Status |
| --- | --- | --- | --- | --- |
| C01 | Reusable rust-ci.yml and a setup-rust-toolchain action in biscuit_games_tooling | `C01-reusable-rust-ci.md` | T04, T11 | open |
| C02 | biscuit_games_tooling: a configurable agent contract | `C02-tooling-agent-contract.md` | D01 | open |

## Dependency graph

```text
D01 ── D02 ── T00 ──┬── T01 ──────────────────┐
                    ├── T02 ──────────────────┤
                    ├── T03 ──────────────────┤
                    ├── T04 (needs T01's remote) ┤
                    ├── T05 ──────────────────┼── T11 ── T10
                    ├── T06 ──────────────────┤    ├── S01, S02, S03, S04
                    ├── T07 ──────────────────┤    ├── T12 (also after T06, T07, T08)
                    ├── T08 ──────────────────┤    └── C01 (also after T04)
                    └── T09 ──────────────────┘
C02 hangs off D01 alone (D01 kept the Python checkers; decision 0004)
```

The graph is acyclic: D01, then D02, then T00, then nine parallel lanes, then T11 and T10
in sequence. T12 follows T11 because it edits pages T07 and T08 own. T04 needs the remote T01 creates before its proof run, but its files can be
written in parallel with T01.

## How to pick up a ticket

1. Create a worktree on the branch the ticket's `branch:` field names, from `main`,
   after every ticket it depends on has merged. Below, `<branch>` is that field, which
   is lowercase (`ticket/t03-hooks-and-dotfiles`), and `<id>` is the ticket id (`T03`):

   ```sh
   supacode repo worktree-new --branch <branch> --base main --name <id>
   ```

   Outside a Supacode terminal:

   ```sh
   git worktree add ../<id> -b <branch> main
   ```

2. Read `CONVENTIONS.md`, then the ticket. Read the source files the ticket names in the
   clones at the commits `CONVENTIONS.md` §0 pins, with `git -C <clone> show <commit>:<path>`
   if a clone has moved on. Never modify a clone.
3. Edit only the files the ticket lists, plus the `status:` line of the ticket itself.
   A change needed elsewhere is handed back in the ticket's hand-back notes, not made.
4. Run the ticket's verification commands and quote their output in the hand-back notes.
5. Commit on the ticket branch. Pushing and opening the pull request are separately
   authorised: stop and ask.

## Definition of done

For a build ticket: `just check` is green in this repository, the ticket's acceptance
criteria are met, its verification commands ran with the output quoted in the hand-back
notes, every open point is answered or explicitly carried forward, and the ticket's
`status:` is `done` in the same pull request. D01 and D02 are done when their decision is
recorded in the ticket and the files T00 must create are listed there.

For a spike ticket: the options are weighed against the criteria the ticket states, a
recommendation is written, and the follow-up build ticket (if any) is drafted in the
hand-back notes; nothing is installed or changed outside the ticket file.

For a centralisation ticket: the recommendation is implemented where the ticket says, its
acceptance criteria are met, and every action that touches another repository or a
repository setting was authorised before it was taken.

## Ticket format

Every ticket carries frontmatter (`id`, `title`, `status`, `depends_on`, `parallel_with`,
`branch`, `estimated_size`) and these sections in this order: Context, Goal, Non-goals,
Files touched, Steps, Acceptance criteria, Verification, Hand-back notes, Open points.
Repository paths are written as code spans, never as links, because the hook gate T00
installs runs lychee offline over this directory.
