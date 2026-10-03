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
| T00 | Foundation: workspace, toolchain, licence, Justfile, hooks, manifest, stubs for every path | `T00-foundation.md` | D01, D02 | none | done |
| T01 | GitHub repository: create, first push, settings, branch protection | `T01-github-repository.md` | T00 | T02 to T09 | done |
| T02 | Rust quality gate: tool configs, the no_std crate skeleton, every Rust recipe green | `T02-rust-gate.md` | T00 | T01, T03 to T09 | done |
| T03 | Hooks and the language-agnostic gate: dotfiles, first-run script, the pixi environment | `T03-hooks-and-dotfiles.md` | T00, D01, D02 | T01, T02, T04 to T09 | done |
| T04 | CI workflows: the composite setup action, five gate jobs and the audit job | `T04-ci.md` | T00, T01 | T02, T03, T05 to T09 | done |
| T05 | Agent contract: AGENTS.md, adapters, fourteen skills, their bridges | `T05-agent-contract.md` | T00 | T01 to T04, T06 to T09 | done |
| T06 | Specification migration: the seven engine modules, adapted, and the Allium gate | `T06-spec-migration.md` | T00 | T01 to T05, T07 to T09 | done |
| T07 | Handbook A: project, tutorial and how-to pages | `T07-handbook-a.md` | T00 | T01 to T06, T08, T09 | done |
| T08 | Handbook B: explanation, reference and operations pages | `T08-handbook-b.md` | T00 | T01 to T07, T09 | done |
| T09 | Decision records | `T09-decisions.md` | T00, D01, D02 | T01 to T08 | done |
| T11 | Integration: first green `just check` from a fresh clone and in CI, branch protection verified | `T11-integration.md` | T01 to T09 | none | done |
| T10 | Maintainer docs: README, CHANGELOG, SECURITY | `T10-maintainer-docs.md` | T11 | none | done |
| T12 | Board migration: board.allium adapted, and the alignment with the game at add73be7 | `T12-board-migration.md` | T06, T07, T08, T11 | none | done |
| T19 | Puzzle design objectives: the research report in the docs, the gaps in the specs, and the generation module's skeleton | `T19-puzzle-design-objectives.md` | T06, T08, T12 | none | done |
| T22 | Metrics gate: clippy thresholds and rustqual as gate 7, with the boundary rules from the layering table | `T22-metrics-gate.md` | S09 | none | done |
| T23 | Spec change: board imports solver, the proof sentence in sudoku, and decision 0014 | `T23-board-imports-solver.md` | T12, T22 | none | done |
| T24 | A plan-spec recipe: the test obligations allium derives from one module | `T24-plan-spec-recipe.md` | T23 | none | done |
| T29 | The snapshot harness: insta, its recipes and the rule for what a snapshot is for | `T29-snapshot-harness.md` | T24 | none | done |
| T25 | The rules in Rust: the sudoku module, the proof value and Puzzle | `T25-sudoku-rules.md` | T23, T24, T29 | none | done |
| T26 | The solver in Rust: the search, its result and the proof | `T26-solver.md` | T25 | none | done |
| T27 | The board in play: notes, moves, undo and redo, reading back, and the check | `T27-board-in-play.md` | T26 | none | done |
| T28 | The record and reopening: a board written down whole and had again | `T28-record-and-reopening.md` | T27 | none | done |
| T30 | The first-change tutorial: an exercise that can be followed again | `T30-first-change-tutorial.md` | T25 | T26 to T28 | done |
| T31 | Spec change: the basic generator in generation, a published method of removal checked by the verdict | `T31-basic-generator-spec.md` | T19 | none | done |
| T32 | The basic generator in Rust: the solution grid, removal in order and the five tiers | `T32-basic-generator.md` | T31, T28 | none | done |
| T33 | The API reference on GitHub Pages: a deploy workflow, a site recipe and the first workflow that publishes | `T33-api-reference-on-github-pages.md` | T11 | S10 | open |

Spike tickets. Each weighs options and ends in a recommendation; none blocks the build and
none is picked up before T11.

| Id | Title | File | Depends on | Status |
| --- | --- | --- | --- | --- |
| S01 | Spike: dependency updates, Dependabot, Renovate or none | `S01-dependency-updates.md` | T11 | done |
| S02 | Spike: release and publishing, cargo-release or release-plz, crates.io, lifting publish = false | `S02-release-and-publishing.md` | T11 | done |
| S03 | Spike: benchmarks, fuzzing and mutation testing | `S03-bench-fuzz-mutants.md` | T11 | done |
| S04 | Spike: bindings and CLI groundwork, pawdoku-cli, pawdoku-py, pawdoku-wasm | `S04-bindings-and-cli.md` | T11, T06 | done |
| S06 | Spike: one ordered list of licensed deductions, and a run that takes the first | `S06-licensed-deductions-in-order.md` | T19 | open |
| S07 | Spike: undoing a deduction, technique by technique | `S07-undoing-a-deduction.md` | S06 | open |
| S08 | Spike: removal by undoing deductions, against removal and a rating | `S08-removal-by-undoing.md` | S06, S07 | open |
| S09 | Spike: code-quality metrics as a gate, module boundaries, complexity, coupling and cohesion | `S09-code-quality-metrics.md` | T11 | done |
| S10 | Spike: prose pages, diagrams and raw HTML inside rustdoc | `S10-pages-and-diagrams-in-rustdoc.md` | T11 | open |

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
                    ├── T06 ──────────────────┤    ├── S01, S02, S03, S04, S09 ── T22
                    ├── T07 ──────────────────┤    ├── T12 (also after T06, T07, T08) ── T19 ── S06 ── S07 ── S08
                    ├── T08 ──────────────────┤    └── C01 (also after T04)
                    └── T09 ──────────────────┘
C02 hangs off D01 alone (D01 kept the Python checkers; decision 0004)

T12, T22 ── T23 ── T24 ── T29 ── T25 ──┬── T26 ── T27 ── T28   (the first engine modules)
                                       └── T30   (the tutorial T25 left unfollowable)

T19 ── T31 ── T32 (also after T28)   (the basic generator: its specification, then its Rust)
T11 ──┬── T33   (the API reference on GitHub Pages)
      └── S10   (pages and diagrams inside rustdoc; its follow-up needs T33)
```

The graph is acyclic: D01, then D02, then T00, then nine parallel lanes, then T11 and T10
in sequence. T12 follows T11 because it edits pages T07 and T08 own; T19 follows T12
because it counts the modules T12 made eight. S06, S07 and S08 follow T19 in sequence,
each reading the hand-back notes of the one before. T22 follows S09, the spike that
chose its tool and thresholds. T04 needs the remote T01 creates before its proof run, but its files can be
written in parallel with T01.

T23 to T29 are the work on the first engine modules, strictly in sequence: the spec change that lets
`board` import `solver`, the recipe that prints a module's test obligations, the
snapshot harness (T29, numbered last and run third), then `sudoku`, `solver`, the board
in play, and the record. T29 adds insta and the rule that a snapshot shows a value to a
reviewer and detects a change to it, and never proves a clause; T25 to T28 each take
snapshots at the interface they build. T23 follows T12, whose module it
edits, and T22, whose boundary rule it renames. T25 to T28 each edit files the one
before it also edited (`crates/pawdoku/src/lib.rs`, `crates/pawdoku/tests/api_bounds.rs`,
`docs/reference/testing.md`, `docs/explanation/architecture.md`, `CHANGELOG.md`, the
snapshot directories, and `crates/pawdoku/src/sudoku.rs`, whose crate-only types T26 makes public and from which
T27 removes one attribute). That is not two
owners of one path in the sense of CONVENTIONS.md §11, which is about lanes that run at
once: here no two are ever open together. The solver T20 and T21 wait for, in T19's
hand-back notes, is T26.

T30 follows T25, which moved the items the first-change tutorial's exercise is built on.
It edits one page, `docs/tutorials/first-change.md`, that none of T26 to T28 lists, so it
may run beside them.

T31 and T32 are the basic generator: a published method that draws a full grid and
removes givens in a stated order, each removal checked by the solver's verdict. T31
writes it into `generation.allium`, the skeleton T19 landed, beside the designed
generator that T20 is drafted to specify; T32 builds it, after T31 and after T28, the
last ticket to edit the files the engine modules share. Neither waits on T20 or on the
spikes S06 to S08.

T33 and S10 are the hosted API reference. T33 publishes what `just doc` builds to GitHub
Pages on every push to `main`, in a workflow of its own, and is the first ticket whose
workflow publishes. S10 is the spike on how prose pages, diagrams and raw HTML go inside
rustdoc. It shares one path with T33, this index, where each sets its own row and
nothing else; its experiments run in a scratch copy outside the repository. So it may
run beside T33, and the build ticket it drafts depends on T33. The ids between T30 and T33 are the
basic-generator tickets'.

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
