---
id: T00
title: "Foundation: workspace, toolchain, licence, Justfile, hooks, manifest, stubs for every path"
status: open
depends_on: [D01, D02]
parallel_with: []
branch: ticket/t00-foundation
estimated_size: XL
---

# T00: Foundation: workspace, toolchain, licence, Justfile, hooks, manifest, stubs for every path

## Context

This repository (`steven-cutting/libpawdoku`, branch `main`, one commit holding a stub
`README.md` and this `tickets/` directory) becomes the Cargo workspace CONVENTIONS.md §0
describes. `CONVENTIONS.md` is the design; read it in full before anything else, then
`README.md` in this directory for the worktree rules, then D01's and D02's hand-back
notes: D01 decided the hook runner and checkers (record 0004), and D02 decided who
installs them and every other tool (record 0011); D02 holds the `pyproject.toml`
content this ticket writes, the one-line `tools.txt`, the `Justfile` changes, the hook
entries and the composite action. This is the first build ticket: nothing else
can start until it has merged to `main`, because the nine lane tickets T01 to T09 each
replace stubs this ticket creates, and every lane's definition of done is `just check`
green in this repository, which only this ticket can make true.

Why one large ticket rather than several: the documentation contract and the agent
contract (B's `bg-validate-docs` and `bg-validate-agents`) walk every
registered page and every skill, and the ordered gate snapshots the worktree between
recipes. They can only be green if every path in CONVENTIONS.md §3 exists from the first
commit in a valid shape. So this ticket ships the whole shape: the workspace and a crate
with one documented, tested item; the toolchain pin, the pixi manifest with its lockfile
and the one-line tool list; the complete
`Justfile` and both prek configs (which no lane may touch, CONVENTIONS.md §11); the
complete `docs/manifest.yml` and `docs/README.md` (frozen too); the licence; and a stub
for everything else. Lanes replace stubs; they never add, rename or reclassify a path.

Sources, at the commits CONVENTIONS.md §0 pins (read-only, never modified):

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`. Read first: `Justfile`,
  `.pre-commit-config.yaml`, `.pre-commit-fix.yaml`, `.editorconfig`, `.gitattributes`,
  `.gitignore`, `.markdownlint-cli2.jsonc`, `lychee.toml`, `pyproject.toml`,
  `docs/manifest.yml`, `docs/README.md`, `docs/decisions/README.md`, `AGENTS.md`,
  `CLAUDE.md`, `.github/copilot-instructions.md`, `.agents/skills/*/SKILL.md`,
  `.claude/skills/fix-quality/SKILL.md`, `.claude/settings.json`, `scripts/initialize.sh`,
  `.github/workflows/ci.yml`, `docs/specs/*.allium`.
- B = `/Users/scutting/projects/biscuit_games_tooling` at `6c5c07f6`. Read
  `src/biscuit_games_tooling/validate_docs.py` and `validate_agents.py` for what a stub
  must satisfy, and `_project.py` for how the recipe list is read.
- T = `/Users/scutting/projects/biscuit_games_template` at `2283589c`. Read
  `tickets/T00-foundation.md` for the shape of a foundation ticket's hand-back notes.

Check `git -C /Users/scutting/projects/pawdoku rev-parse --short=8 HEAD` prints
`78d03cdf`; if a clone has moved, read the pinned content with
`git -C <clone> show <commit>:<path>` instead of the working file.

Preconditions, stated and not performed (CONVENTIONS.md §2): rustup installed (a default
toolchain is probably not required: §2, and T03 confirms); `command -v cargo` printing
`$HOME/.cargo/bin/cargo`; pixi 0.81.0 or newer; any just; gh. Not uv and not
cargo-binstall: pixi installs both just and cargo-binstall into the environment. If any
is missing, stop and report it; do not install anything outside the worktree.

## Goal

A first commit on `ticket/t00-foundation` after which, in a fresh clone,
`just initialize` then `just check` are green, with every path in CONVENTIONS.md §3
present: final content for the frozen files, the licence, the manifest, the map, the
adapters and the bridges; verbatim copies of the seven specification modules; and a stub
for every page, decision, skill, root document and configuration file that a lane will
replace. `cargo nextest run` runs one test, `cargo test --doc` runs one doctest,
`just wasm-check` passes on both targets, and `just coverage` reports 100% of the lines
of a crate that has almost none. The pixi environment, `pixi.lock` and the one-line
`tools.txt` are the tool pins from this commit on (decision 0011).

## Non-goals

- No engine code. `crates/pawdoku/src/lib.rs` holds one item (below) so that the gate
  has something to format, lint, test, document and measure; `random.rs` is T02's.
- No dependency anywhere, not even in `[workspace.dependencies]`: gate 13
  (`deps-unused`, cargo-shear) flags a workspace entry nothing uses, and T02 is the one
  lane that adds dependencies (CONVENTIONS.md §11). `Cargo.lock` lists the one member.
- No final prose. Pages, decisions, skills, `AGENTS.md`, `README.md`, `CHANGELOG.md`,
  `SECURITY.md` and `crates/pawdoku/README.md` are stubs of 40 or more real words that
  say what the file will contain and name the lane that writes it.
- No final `rustfmt.toml`, `clippy.toml`, `taplo.toml`, `deny.toml`,
  `.config/nextest.toml` beyond what the gate needs to pass (T02); no final dotfiles
  beyond what the hooks need (T03); no CI beyond the one-job stub (T04); no repository
  creation, remote or push (T01).
- No edit to any spec module: the seven are copied byte-for-byte from G (T06 adapts).
- No `skills-lock.json` and no `allium-skill-reference/` (T05); the seven Allium skills
  are stubbed like the other seven, with G's frontmatter and a stub body.

## Files touched

Every path in CONVENTIONS.md §3 except those marked T02, T04 (`audit.yml`), T05
(`skills-lock.json`, `allium-skill-reference/`). Class: `final` means
no lane touches it again; `stub` means the named lane replaces it.

| Path | Class | Content |
| --- | --- | --- |
| `Cargo.toml` | skeleton -> T02 | step 4 |
| `Cargo.lock` | generated | step 4 |
| `rust-toolchain.toml` | final | CONVENTIONS.md §2, exact |
| `tools.txt` | final | CONVENTIONS.md §2, exact (one line) |
| `rustfmt.toml`, `clippy.toml`, `taplo.toml`, `deny.toml`, `.config/nextest.toml` | stub -> T02 | step 5 |
| `crates/pawdoku/Cargo.toml` | skeleton -> T02 | step 4 |
| `crates/pawdoku/src/lib.rs` | skeleton -> T02 | step 4 |
| `crates/pawdoku/README.md` | stub -> T10 | step 12 |
| `LICENSE` | final | step 3 |
| `README.md`, `CHANGELOG.md`, `SECURITY.md` | stub -> T10 | step 12 |
| `Justfile` | final | CONVENTIONS.md §4, exact |
| `.pre-commit-config.yaml`, `.pre-commit-fix.yaml` | final | CONVENTIONS.md §5 applied to G's files |
| `.editorconfig`, `.gitattributes`, `.gitignore`, `.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` | working -> T03 | CONVENTIONS.md §5's edits applied to G's files, so the hooks pass; T03 finalises the comments |
| `pyproject.toml` | working -> T03 | D02's content (the pixi manifest and B's table) |
| `pixi.lock` | generated | `pixi lock`; committed |
| `scripts/initialize.sh` | stub -> T03 | step 7 |
| `.github/actions/setup/action.yml` | stub -> T04 | step 8 |
| `.github/workflows/ci.yml` | stub -> T04 | step 8 |
| `docs/manifest.yml`, `docs/README.md` | final | step 9 |
| `docs/project/*`, `docs/tutorials/*`, `docs/how-to/*`, `docs/explanation/*` (except the two migrated pages), `docs/reference/*`, `docs/operations/*` | stub -> T07/T08 | step 10 |
| `docs/explanation/solving-sudoku.md`, `docs/explanation/human-solving.md` | verbatim from G -> T06 | step 10 |
| `docs/decisions/README.md`, `docs/decisions/0001-*.md` .. `0011-*.md` | stub -> T09 | step 10 |
| `docs/specs/{sudoku,solver,technique,reach,effort,lapse,human-solving}.allium` | verbatim from G -> T06 | step 11 |
| `AGENTS.md` | stub -> T05 | step 13 |
| `CLAUDE.md`, `.github/copilot-instructions.md`, `.claude/settings.json` | final | byte-identical to G's |
| `.agents/skills/<14>/SKILL.md` | stub -> T05 | step 13 |
| `.claude/skills/<14>/SKILL.md`, `.codex/skills/<14>/SKILL.md` | final | step 13 |
| `tickets/T00-foundation.md` | ticket | `status:` and hand-back notes |

## Steps

1. Create the worktree on `ticket/t00-foundation` from `main` (README.md "How to pick up
   a ticket"). Verify the preconditions:

   ```sh
   command -v rustup && command -v cargo && command -v pixi && command -v just && command -v gh
   pixi --version
   rustup toolchain list
   ```

   `cargo` must resolve under `~/.cargo/bin` (a `pixi global` rust at `~/.pixi/bin/cargo`
   shadows rustup in any shell that does not source `~/.cargo/env`; prefix
   `PATH="$HOME/.cargo/bin:$PATH"` on every command if so). pixi must print 0.81.0 or
   newer. Stop otherwise.

2. `rust-toolchain.toml` and the one-line `tools.txt`, exactly as CONVENTIONS.md §2 prints
   them. Then
   `rustup toolchain install` from the worktree and confirm `rustup show active-toolchain`
   prints `1.98.1-<host>`; this is CONVENTIONS.md §12's first claim (the no-argument form
   reads the file). If 1.98.1 is no longer the newest stable, keep the pin: bumping it
   is a `main` follow-up, not this ticket's.

3. `LICENSE`: the Apache License 2.0 text, verbatim from
   `https://www.apache.org/licenses/LICENSE-2.0.txt`, with the appendix's boilerplate
   left as it is (no per-file headers anywhere; decision 0006). No `NOTICE` file.

4. The workspace. Root `Cargo.toml`:

   ```toml
   [workspace]
   resolver = "3"
   members = ["crates/*"]

   [workspace.package]
   edition = "2024"
   rust-version = "1.98"
   license = "Apache-2.0"
   repository = "https://github.com/steven-cutting/libpawdoku"
   homepage = "https://github.com/steven-cutting/libpawdoku"
   authors = ["Steven Cutting"]
   publish = false

   [workspace.lints.rust]
   unsafe_code = "forbid"
   missing_docs = "warn"

   [workspace.lints.clippy]
   pedantic = { level = "warn", priority = -1 }
   ```

   T02 adds `[workspace.dependencies]`, grows the lint tables and adds the profiles; this
   ticket keeps them to what a one-item crate with no dependencies can satisfy. `crates/pawdoku/Cargo.toml`:

   ```toml
   [package]
   name = "pawdoku"
   version = "0.1.0"
   description = "Classic-sudoku engine: solver, human-technique catalogue, difficulty rating, hints."
   keywords = ["sudoku", "puzzle", "solver"]
   categories = ["games", "algorithms", "no-std"]
   readme = "README.md"
   edition.workspace = true
   rust-version.workspace = true
   license.workspace = true
   repository.workspace = true
   homepage.workspace = true
   authors.workspace = true
   publish.workspace = true

   [lints]
   workspace = true

   [features]
   default = []

   [dependencies]

   [dev-dependencies]

   [package.metadata.docs.rs]
   all-features = true
   rustdoc-args = ["--cfg", "docsrs"]
   targets = ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
   ```

   `crates/pawdoku/src/lib.rs`:

   ```rust
   //! The Pawdoku engine: classic sudoku, specified in `docs/specs/` and built here.
   //!
   //! This crate is `no_std` with `alloc`: it has no clock, no threads, no filesystem and
   //! no source of randomness of its own. What it needs from the outside world it takes
   //! through traits, so that it runs the same in a browser, in Python and on the command
   //! line. See `docs/explanation/architecture.md`.

   #![no_std]

   extern crate alloc;

   #[cfg(test)]
   extern crate std;

   /// The side of a classic sudoku grid: nine cells to a row, a column and a box.
   ///
   /// ```
   /// assert_eq!(pawdoku::SIDE, 9);
   /// ```
   pub const SIDE: u8 = 9;

   #[cfg(test)]
   mod tests {
       use super::SIDE;

       #[test]
       fn a_grid_is_nine_by_nine() {
           assert_eq!(usize::from(SIDE) * usize::from(SIDE), 81);
       }
   }
   ```

   The constant restates `sudoku.allium`'s `config.side`, which is why it is the one
   item: a doctest, a unit test, a public documented item, and nothing T02 must undo
   (T02 may move it). `cargo generate-lockfile` produces `Cargo.lock`; commit it. With no
   dependencies it lists one package.

5. The tool configuration stubs, minimal but valid, so that `toml-check`, `clippy`,
   `deny` and `coverage` run: `rustfmt.toml` with `style_edition = "2024"` and
   `max_width = 100`; `clippy.toml` with `allow-unwrap-in-tests = true`; `taplo.toml`
   with `include = ["**/*.toml"]` and `exclude = ["target/**", ".tools/**", ".pixi/**",
   "ai_tmp/**"]` (the pixi environment's `site-packages` contain TOML, so the exclude is
   load-bearing);
   `deny.toml` with a `[licenses] allow = ["Apache-2.0"]` list, `[bans] wildcards =
   "deny"`, `[sources] unknown-registry = "deny"` and `allow-registry` naming crates.io
   (CONVENTIONS.md §12 asks T02 to confirm the 0.20 schema); `.config/nextest.toml` with
   `[profile.default] fail-fast = true`. Each carries a one-line comment naming T02 as
   the lane that finalises it.

6. `Justfile`, exactly as CONVENTIONS.md §4 prints it; and `.pre-commit-config.yaml` and
   `.pre-commit-fix.yaml`, G's files with every change CONVENTIONS.md §5 lists applied
   and nothing else (copy the SHAs and comments, do not retype them). `pyproject.toml`
   from D02's hand-back notes ("Files T00, T03 and T04 create"), written by hand and
   never by `pixi init`; then `pixi lock` (GitHub must be reachable for the git
   dependency, conda-forge for the tools) and `pixi install --frozen`. Before the lock,
   re-verify every pin with `pixi search <name> --platform linux-64` and record the
   output; a version conda-forge does not carry for linux-64 is a design change (§12).
   `pixi.lock` is committed. No `uv.lock`, no `.venv/`, no `.python-version`.

7. The dotfiles, G's with CONVENTIONS.md §5's edits: `.editorconfig` (add `[*.rs]`),
   `.gitattributes`, `.gitignore` (drop the Node, Svelte, Storybook and Chromatic blocks;
   add `target/`, `*.profraw`, `lcov.info`, `mutants.out*/`; keep `.tools/` with G's
   comment; `.pixi/` in place of `.venv/`, with a comment in the same voice; keep
   `__pycache__/` and `*.py[cod]`), `.markdownlint-cli2.jsonc` (`ignores`), `lychee.toml`
   (`exclude_path`), and `_typos.toml` as §5 gives it.
   `scripts/initialize.sh`: G's script with the npm, Storybook and ruff sections removed
   and these steps in their place, in order, under `set -eu`: `pixi install --locked`;
   `just install-toolchain`; `just check-toolchain`; `just install-tools`;
   `just install-allium`; `test -f Cargo.lock || cargo generate-lockfile`; `just sync`;
   `just format`; then G's worktree-aware `install-hooks` block (its comment naming
   `.pixi/envs/default/bin/prek`) and closing lines verbatim. Nothing compiles at
   bootstrap; T03 finalises the comments.

8. CI stub. `.github/actions/setup/action.yml`: a composite action with the `cache-key`
   input and the steps CONVENTIONS.md §10 lists, with every `uses:` pinned to the SHA
   looked up on the day (`gh api repos/<owner>/<repo>/git/ref/tags/<tag>`, dereferencing
   an annotated tag through `git/tags/<sha>`) and a version comment; `setup-pixi`'s
   `pixi-version` is the exact pixi version the manifest's `requires-pixi` floors.
   `.github/workflows/ci.yml`:
   G's header (triggers, permissions `contents: read`, concurrency) and one job named
   `check`, `timeout-minutes: 45`, checkout with `persist-credentials: false`, the setup
   action, `just install-allium`, `just check`. `actionlint` must pass on it (`just lint`
   runs it). The name `check` is permanent: T01 makes it the one required status, and
   T04 replaces the single job with five gate jobs plus a `check` aggregate that needs
   them, so protection never changes (CONVENTIONS.md §10).

9. `docs/manifest.yml` and `docs/README.md`, final and frozen. The manifest is strict
   JSON in G's layout; every page below must exist by the end of this ticket. Carried
   pages keep G's `title`, `kind`, `audience` and `canonical_for` exactly; the rows below
   are the complete list:

   ```text
   README.md                                  Documentation map                 project    [user, contributor, maintainer, operator, agent]  documentation_navigation
   project/purpose-and-scope.md               Purpose and scope                 project    [user, contributor, maintainer, agent]            project_purpose, project_non_goals
   project/repository-map.md                  Repository map                    project    [contributor, maintainer, agent]                  repository_layout
   project/terminology.md                     Terminology                       project    [contributor, maintainer, operator, agent]        project_terminology
   tutorials/first-change.md                  Make your first change            tutorial   [contributor, agent]                              first_change_tutorial
   how-to/develop-locally.md                  Develop locally                   how-to     [contributor, maintainer, agent]                  local_development
   how-to/test-and-debug.md                   Test and debug                    how-to     [contributor, maintainer, agent]                  test_workflow
   how-to/work-with-the-specs.md              Work with the specifications      how-to     [contributor, maintainer, agent]                  specification_workflow
   how-to/maintain-dependencies.md            Maintain dependencies             how-to     [maintainer, agent]                               dependency_maintenance
   explanation/architecture.md                Architecture                      explanation [contributor, maintainer, operator, agent]       system_architecture
   explanation/layering.md                    Layering and dependency direction explanation [contributor, maintainer, agent]                 dependency_boundaries
   explanation/specifications.md              Specifications                    explanation [contributor, maintainer, agent]                 specification_model
   explanation/security-model.md              Security model                    explanation [user, contributor, maintainer, operator, agent] security_model
   explanation/quality-philosophy.md          Quality philosophy                explanation [contributor, maintainer, agent]                 quality_philosophy
   reference/commands.md                      Commands                          reference  [contributor, maintainer, operator, agent]        command_reference
   reference/configuration.md                 Configuration                     reference  [contributor, maintainer, operator, agent]        configuration_reference
   reference/testing.md                       Testing                           reference  [contributor, maintainer, agent]                  testing_reference
   reference/quality-gates.md                 Quality gates                     reference  [contributor, maintainer, agent]                  quality_gate_reference
   reference/documentation-contract.md        Documentation contract            reference  [contributor, maintainer, agent]                  documentation_contract
   reference/agent-contract.md                Agent contract                    reference  [contributor, maintainer, agent]                  agent_contract
   reference/api.md                           API reference                     reference  [user, contributor, maintainer, agent]            api_reference
   operations/maintenance.md                  Maintenance                       operations [maintainer, operator, agent]                     maintenance_routine
   operations/troubleshooting.md              Troubleshooting                   operations [contributor, maintainer, operator, agent]        troubleshooting
   decisions/README.md                        Architecture decisions            decision   [contributor, maintainer, agent]                  decision_index
   decisions/0001-engine-as-a-library.md      Decision 0001: The engine is a library of its own          decision [maintainer, agent]               decision_engine_as_a_library
   decisions/0002-specs-are-the-source-of-truth.md  Decision 0002: Specifications decide behaviour        decision [contributor, maintainer, agent]  decision_spec_first
   decisions/0003-effects-behind-traits.md    Decision 0003: Effects behind traits                       decision [contributor, maintainer, agent]  decision_ports_and_fakes
   decisions/0004-hook-runner-and-checkers.md Decision 0004: Hook runner and checkers                    decision [maintainer, agent]               decision_python_toolchain
   decisions/0005-project-managed-allium-cli.md  Decision 0005: A project-managed Allium binary          decision [maintainer, agent]               decision_allium_cli
   decisions/0006-apache-2-0.md               Decision 0006: Apache-2.0                                  decision [maintainer, agent]               decision_license
   decisions/0007-dependency-policy.md        Decision 0007: Dependency policy for a library             decision [contributor, maintainer, agent]  decision_dependency_policy
   decisions/0008-no-std-core.md              Decision 0008: A pure no_std core                          decision [contributor, maintainer, agent]  decision_no_std_core
   decisions/0009-rust-quality-gate.md        Decision 0009: The Rust quality gate                       decision [contributor, maintainer, agent]  decision_rust_gate
   decisions/0010-skill-reference-material.md Decision 0010: Skill reference material beside the skills  decision [maintainer, agent]               decision_skill_reference
   decisions/0011-tool-manager.md             Decision 0011: Tool manager                                decision [maintainer, agent]               decision_tool_manager
   explanation/solving-sudoku.md              (G's title, verbatim)             explanation [contributor, maintainer, agent]                 sudoku_solving_strategies
   explanation/human-solving.md               Modelling a human Sudoku solver   explanation [contributor, maintainer, agent]                 human_solving_model
   ```

   Every `requires` is `[]`. `docs/README.md` is G's map rewritten for the library: the
   same frontmatter; the opening paragraph; a specifications paragraph naming the seven
   modules in G's order with `sudoku.allium` first and no platform sentence, ending "The
   game's root module, `pawdoku.allium`, stays in the game and restates what it needs";
   the sections Start here, How to (four pages), Understand (five pages, then the two
   migrated pages under a "This library" heading at the end, in G's shape), Look up
   (seven pages, `api.md` last), Run it, Decisions. Links are relative Markdown links as
   in G; every registered page is linked exactly once.

10. Page stubs. For every page in the manifest except the two migrated ones, a file with
    the manifest's frontmatter (inline lists, G's key order), an H1 equal to the title,
    and 40 or more words of real prose stating what the page will contain, which G page
    it is rewritten from (or that it is new), and the owning lane (T07, T08 or T09), with
    no `TODO`, `TBD`, `FIXME` or lorem ipsum (the validator rejects them) and no link.
    The two migrated pages are copied byte-for-byte from G. `docs/decisions/README.md`
    is G's index page reshaped: the same first two paragraphs, a table of the eleven
    numbers and titles above linking to the files, a "The numbering" paragraph saying
    the series is this repository's own, that carried records say which Pawdoku
    record they came from, and that the next decision is 0012, and G's "Writing a new
    one" and "Related pages" sections.

11. The seven modules: `git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/specs/<name>.allium > docs/specs/<name>.allium`
    for `sudoku`, `solver`, `technique`, `reach`, `effort`, `lapse`, `human-solving`.
    Byte-identical; `cmp` each. Then `just install-allium`, `just check-specs` and
    `just analyse-specs` must pass on the unedited copies (they do in G, whose gate is
    green at `78d03cdf`).

12. Root document stubs: `README.md` (the repository name, one sentence on what it is,
    one on where the tickets are, and that T10 writes the rest), `CHANGELOG.md` (Keep a
    Changelog 1.1.0 header, `## [Unreleased]`, `### Added` with one line: "The
    repository skeleton: workspace, toolchain pin, quality gate, handbook and agent
    contract, as `tickets/` describes."), `SECURITY.md` (reporting route to be decided
    with visibility in T01; supported versions `main`; T10 writes the rest),
    `crates/pawdoku/README.md` (the crate name, one paragraph, T10). The docs contract
    reads `README.md` and `SECURITY.md` (G's `validate-docs` hook lists them), so both
    must have 40 words and valid links.

13. The agent surface. `CLAUDE.md`, `.github/copilot-instructions.md` and
    `.claude/settings.json` copied byte-for-byte from G (`cmp` each). `AGENTS.md`: G's
    section headings, at least 300 words, carrying the six required phrases of
    CONVENTIONS.md §1 fact 1 in sentences that are true of this repository ("this
    repository has no Svelte runes; ..."), stating in one paragraph each
    what the section will say and that T05 writes it. Fourteen skill directories under
    `.agents/skills/`: `code-review`, `fix-quality`, `plan-change`, `project-check`,
    `review-docs`, `spec-change`, `rust-change`, `allium`, `distill`, `elicit`,
    `propagate`, `tend`, `weed`, `witness`. Each `SKILL.md` carries frontmatter of
    exactly `name` (equal to the directory) and `description` (G's description for the
    thirteen carried skills; for `rust-change`: "Implement or review a change to the
    engine with the trait boundary, no_std, tests and documentation evidence the
    invariants require."), and a stub body of 40 or more words that cites `AGENTS.md`
    and one `just` recipe and names T05. Twenty-eight bridges, each with the canonical
    frontmatter repeated verbatim and the body exactly:
    ``Follow `../../../.agents/skills/<name>/SKILL.md`. That file is canonical and this bridge adds nothing to it.``

14. Run the gate from the top and quote the results in the hand-back notes:

    ```sh
    just initialize
    just check
    ```

    Then, from a fresh clone of the worktree into `ai_tmp/clone` (or the session
    scratch directory), the same two commands, to prove nothing depends on state the
    worktree accumulated. Record the wall-clock time of each `just check`.

15. Record every CONVENTIONS.md §12 claim assigned to T00 with its outcome (the
    no-argument `rustup toolchain install`; `wasm32v1-none` on the stub crate; B's
    checkers with no `package.json`; `bg-project-check` tolerating `target/`, `.tools/`
    and `.pixi/`; `pixi install --locked` installing the nine conda packages and B and
    not the virtual project itself; B built from its git tag with `uv_build` under the
    environment's Python, its six scripts landing in `.pixi/envs/default/bin`; every pin
    resolving for `linux-64`). A failed claim is a design change: stop, write it up in
    the hand-back notes, and ask.

16. Set `status: done` and commit on the ticket branch, in one or a few commits with
    short imperative subjects. Stop before pushing.

## Acceptance criteria

- Every path in CONVENTIONS.md §3 not owned by T02, T04 (`audit.yml`) or T05
  (`skills-lock.json`, `allium-skill-reference/`) exists.
- `rust-toolchain.toml`, `tools.txt`, `Justfile`, `.pre-commit-config.yaml`,
  `.pre-commit-fix.yaml`, `docs/manifest.yml`, `docs/README.md`, `CLAUDE.md`,
  `.github/copilot-instructions.md`, `.claude/settings.json` and every bridge are in
  their final form; `CLAUDE.md`, `.github/copilot-instructions.md`, `.claude/settings.json`,
  the twenty-six bridges of carried skills and the seven modules are byte-identical to G's.
- `just check` is green in the worktree and in a fresh clone after `just initialize`.
- `just check-agents` prints `Validated AGENTS.md, 2 adapters, and 14 skills.`;
  `just check-docs` reports every page valid; `just check-specs` and
  `just analyse-specs` report empty arrays for seven modules.
- `cargo nextest run --workspace` runs one test; `cargo test --doc --workspace` runs
  one doctest; `just wasm-check` passes on both targets; `just coverage` reports 100%.
- `Cargo.lock` and `pixi.lock` are committed and `just lock-check` passes.
- `git status --porcelain` is empty after `just check` (the snapshot proved it, but say
  so).
- No file outside `tickets/` and `docs/specs/` contains `TODO`, `TBD` or `FIXME`, and no
  file the two contracts read (`README.md`, `SECURITY.md`, `CHANGELOG.md`, `docs/`,
  `AGENTS.md`, `.agents/`, `.claude/`, `.codex/`) contains `{{`, `{%` or `{#` (the
  `Justfile` and the workflow files legitimately do).
- Nothing was pushed, and no remote exists.

## Verification

```sh
pixi --version
rustup show active-toolchain
just check-toolchain
just initialize
ls .pixi/envs/default/bin | grep -E '^(cargo-nextest|cargo-llvm-cov|cargo-deny|cargo-shear|taplo|prek|just|cargo-binstall|bg-project-check)$' | wc -l
ls .tools/bin
time just check
git status --porcelain
for f in CLAUDE.md .github/copilot-instructions.md .claude/settings.json; do cmp "$f" <(git -C /Users/scutting/projects/pawdoku show "78d03cdf:$f") && echo "$f identical"; done
for m in sudoku solver technique reach effort lapse human-solving; do cmp "docs/specs/$m.allium" <(git -C /Users/scutting/projects/pawdoku show "78d03cdf:docs/specs/$m.allium") && echo "$m identical"; done
ls .agents/skills | wc -l; ls .claude/skills | wc -l; ls .codex/skills | wc -l
cargo nextest run --workspace --locked 2>&1 | tail -3
cargo test --doc --workspace --locked 2>&1 | tail -3
grep -rn -E 'TODO|TBD|FIXME' --exclude-dir=.git --exclude-dir=target --exclude-dir=.tools --exclude-dir=.pixi --exclude-dir=ai_tmp --exclude-dir=tickets --exclude-dir=specs . || echo "no placeholders"
grep -rn -E '\{\{|\{%|\{#' README.md SECURITY.md CHANGELOG.md AGENTS.md docs .agents .claude .codex || echo "no template syntax"
```

Expected: pixi prints 0.81.0 or newer; the toolchain line names `1.98.1`; the count is
`9` and `.tools/bin` lists `allium` and `cargo-hack`; `just check` ends with the clean-worktree
line and exit 0; `git status --porcelain` prints nothing; every `cmp` line says
`identical`; the three counts are `14`; nextest reports `1 test run: 1 passed`; the
doctest run reports `1 passed`; the two greps print `no placeholders` and `no template
syntax`. Quote each in the hand-back notes.

## Hand-back notes

**In progress, stopped on 2026-09-24 at the maintainer's request, then redirected by
D02.** The skeleton described below is the work-in-progress commit `8ddb2c2` on
`ticket/t00-foundation`; `status:` stays `open`. The taplo question that stopped the
work was settled by D02 (decision 0011: pixi owns the tools), and this ticket was
rewritten in place to that design. The section "Pivot to D02" below says what the
commit already has and what changes.

### Actions with effects outside this worktree

Each was authorised by the maintainer on 2026-09-24 ("I install them" for the
prerequisites; "Rename" for the branch), except where marked.

- **rustup 1.29.1 installed** with the official installer, downloaded to the session
  scratch directory from `https://sh.rustup.rs` and run as
  `sh rustup-init.sh -y --default-toolchain none --no-modify-path`. It created
  `~/.rustup/` (which held only `settings.toml` before) and `~/.cargo/bin/` (absent
  before), with the proxies and `~/.cargo/env`. There is no default toolchain.
- **`~/.zshrc` edited.** Right after line 33 (`export PATH="/Users/scutting/.pixi/bin:$PATH"`)
  these lines were inserted, so a new shell resolves `cargo` to `~/.cargo/bin/cargo` ahead
  of pixi's:

  ```sh

  # rustup's proxies ahead of pixi's rust, so rust-toolchain.toml pins are honoured.
  . "$HOME/.cargo/env"
  ```

  pixi's rust is **not** uninstalled; it is shadowed. Verified: `zsh -ic 'command -v cargo'`
  prints `/Users/scutting/.cargo/bin/cargo`. (The agent's own tool shell does not re-read
  `~/.zshrc`, so every command it ran prefixed `PATH="$HOME/.cargo/bin:$PATH"`.)
- **Toolchain 1.98.1 installed** into `~/.rustup/toolchains/1.98.1-aarch64-apple-darwin`
  (components `cargo`, `clippy`, `llvm-tools`, `rust-src`, `rust-std`, `rustc`, `rustfmt`;
  targets `aarch64-apple-darwin`, `wasm32-unknown-unknown`, `wasm32v1-none`). This came
  from rustup's auto-install, the first time `rustup` ran in this worktree, and not from
  the explicit `rustup toolchain install` that followed (see the §12 claim below).
- **cargo-binstall 1.23.0 installed** per machine, into `~/.cargo/bin/cargo-binstall`, by
  `cargo install cargo-binstall@1.23.0 --locked` (compiled from source).
- **Cargo's registry cache** under `~/.cargo/registry/` was populated by that install and
  by compiling `taplo-cli` (below).
- **uv's cache** (`~/.cache/uv`) gained the git checkout of `biscuit_games_tooling` at
  `v0.3.0` (`6c5c07f6bec86e86b3930dfa41392e4b440e8c85`) and the `prek` 0.5.3 wheel.
  `uv sync` used an existing CPython 3.14.3; no interpreter was downloaded.
- **Branch renamed** from the Supacode worktree's `T00-foundation` to
  `ticket/t00-foundation` (`git branch -m`), as D01 did, so the `branch:` field is true.
  The rename is in the shared `.git`, so the primary checkout sees the new name.
- Nothing was pushed; no remote exists; no clone (G, T, B) was modified. prek has not run
  yet, so nothing is in `~/.cache/prek`, and no git hooks were installed.

Inside the worktree but gitignored: `.tools/bin/` (allium 3.6.1 from
`just install-allium`; cargo-nextest, cargo-llvm-cov, cargo-deny, cargo-hack and
cargo-shear from binstall; taplo from a source build, see below), `.venv/`, `target/`,
and `ai_tmp/rustup-claim/` (823 MB, a throwaway `RUSTUP_HOME`/`CARGO_HOME` from the §12
check; safe to delete). Under D02, `.venv/` and every binary in `.tools/bin` except
`allium` and `cargo-hack` are dead weight: delete `.venv/` and re-run `just initialize`.

### What was done (commit `8ddb2c2`, before D02)

Steps 1 to 13 as the ticket then read are written; step 14 (the gate) has not run.
Items below that D02 changes are listed again under "Pivot to D02".

- Step 2: `rust-toolchain.toml` and `tools.txt`, exactly as CONVENTIONS.md §2 prints
  them.
- Step 3: `LICENSE` from `https://www.apache.org/licenses/LICENSE-2.0.txt`, 202 lines,
  SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`. No `NOTICE`.
- Step 4: `Cargo.toml`, `crates/pawdoku/Cargo.toml`, `crates/pawdoku/src/lib.rs`, exactly
  as the step prints them. `cargo generate-lockfile` gives a `Cargo.lock` with the one
  package; `cargo build --locked` succeeds on 1.98.1.
- Step 5: `rustfmt.toml`, `clippy.toml`, `taplo.toml`, `deny.toml` (`allow-registry`
  names `https://github.com/rust-lang/crates.io-index`), `.config/nextest.toml`, each with
  a comment naming T02.
- Step 6: `Justfile` extracted by script from CONVENTIONS.md §4's fence (not retyped).
  `pyproject.toml` extracted from D01's fence. `.python-version` is `3.14`. `uv lock`
  resolved 3 packages; `uv sync --frozen` installed `biscuit-games-tooling==0.3.0` and
  `prek==0.5.3`. `.pre-commit-config.yaml` and `.pre-commit-fix.yaml` are G's files with
  §5's edits applied by a script that asserts each replaced string occurs once:
  the `exclude` list, the ruff and eslint hooks replaced by `fmt-check`, `toml-check` and
  `deps-unused` (fix config: `cargo-fmt`, `taplo-fmt`, `cargo-shear-fix`),
  `skills-lock\.json$` added to `validate-agents`, and typos moved to
  `512fc24f… # v1.50.2`. Every other line matches G's.
- Step 7: `.editorconfig` (`[*.rs]` added), `.gitattributes`, `.gitignore`,
  `.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` per §5.
  `scripts/initialize.sh` (executable) is G's with the npm, Storybook and ruff sections
  replaced by the step-7 sequence. D01's hand-back asks to keep G's `uv lock` line, so it
  sits before `just sync`. G's allium comment is kept, minus its opening "The other one.",
  whose referent (the browser download) is gone.
- Step 8: `.github/actions/setup/action.yml` and `.github/workflows/ci.yml`. SHAs looked
  up on 2026-09-24 with `gh api` (annotated tags dereferenced): actions/checkout
  `3d3c42e5aac5ba805825da76410c181273ba90b1` v7.0.1; taiki-e/install-action
  `9983c65e42da123ff25d1f78505eb6de315aa172` v2.87.20; Swatinem/rust-cache
  `63fed3e2fecf6f7b51dc6f043341b79ef82a9ae7` v2.9.2 (annotated); actions/cache
  `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` v6.1.0; astral-sh/setup-uv
  `c18668ad3cf93ea998bef934396af7bb5c839dc7` v10.2.0. actionlint has not run on them yet.
- Steps 9 to 11: `docs/manifest.yml` (36 entries in the step-9 order, G's one-line-per-page
  layout) and every stub page, generated from one table so that frontmatter and manifest
  cannot disagree. `docs/README.md` and `docs/decisions/README.md` written per steps 9
  and 10. The two migrated pages and the seven modules were copied with `git show`;
  `cmp` says each module is identical.
- Steps 12 and 13: `README.md` (replacing the `# LibPawdoku` stub), `CHANGELOG.md`,
  `SECURITY.md`, `crates/pawdoku/README.md`. `AGENTS.md` has 524 words, G's eight
  headings and the six phrases; `runes` occurs once, in D01's Provenance sentence.
  `CLAUDE.md`, `.github/copilot-instructions.md` and `.claude/settings.json` were copied
  from G with `git show`. There are fourteen canonical skill stubs. The twenty-six
  carried bridges were copied from G; the two `rust-change` bridges were written with
  the canonical frontmatter and B's `BRIDGE_BODY`.

None of the checkers (`bg-validate-docs`, `bg-validate-agents`, prek, allium) has run yet.

### What was verified, and how

- Preconditions (step 1), as found: no rustup; `command -v cargo` gave
  `/Users/scutting/.pixi/bin/cargo`; no cargo-binstall; just 1.51.0 and uv 0.11.18 were
  present. Resolved by the installs above.
- `rustup show active-toolchain` prints
  `1.98.1-aarch64-apple-darwin (overridden by '…/T00-foundation/rust-toolchain.toml')`.
- **§12 claim: no-argument `rustup toolchain install`.** It holds. Run in a fresh
  `RUSTUP_HOME`/`CARGO_HOME` under `ai_tmp/rustup-claim/` with `RUSTUP_AUTO_INSTALL=0`, so
  auto-install could not do the work, it printed "downloading 9 components" and "the
  active toolchain `1.98.1-aarch64-apple-darwin` has been installed". Afterwards all four
  components (`llvm-tools-preview` lands as `llvm-tools`) and both wasm targets were
  listed. The command exited 1 only because its closing self-update check found no rustup
  binary in that throwaway `CARGO_HOME`. That is an artefact of the test, not of the
  install. Note: rustup 1.29 auto-installs the pinned toolchain on any `rustup` call,
  so on a real machine `just install-toolchain` usually finds the toolchain present already.
- The other three §12 claims assigned to T00 (`wasm32v1-none` on the stub crate; B's
  checkers with no `package.json`; `bg-project-check` tolerating `target/`) are **not yet
  checked**: they need the gate to run.

### Deviations, and why

- `docs/README.md`: G's opening paragraph links to the documentation contract and to
  Specifications, which the lists below link again. Step 9 says every registered page is
  linked exactly once, so the opening paragraph names both pages in plain text and
  points to the section that links them.
- The branch was renamed (above).

### Resolved by D02: taplo could not be installed as the design specified

`just install-tools` fails. binstall resolves all six tools before installing any, so the
single failure aborts the whole batch:

```text
cargo-binstall:  WARN resolve: … could not GET https://github.com/cargo-bins/cargo-quickinstall/releases/download/taplo-cli-0.10.0/taplo-cli-0.10.0-aarch64-apple-darwin.tar.gz: … 404 Not Found
cargo-binstall: ERROR Fatal error:
  × For crate taplo-cli: Fallback to cargo-install is disabled
error: recipe `install-tools` failed on line 37 with exit code 1
```

Why: `taplo-cli` 0.10.0 has no `[package.metadata.binstall]` table, and cargo-quickinstall
has no build of it. Its GitHub release (`tamasfe/taplo` tag `0.10.0`, the newest) ships
raw gzip binaries such as `taplo-darwin-aarch64.gz`, a format binstall cannot unpack.
`--disable-strategies compile` then refuses a source build, as designed. This fails on
every platform, CI included. The newest taplo on PyPI is 0.9.3 (wheels for macOS, Linux
and Windows). `tools.txt` and the `Justfile` are frozen, and CONVENTIONS.md §2 and §4
print the failing text, so any fix is a CONVENTIONS.md change.

To keep finding other failures, the agent installed the five other tools with the
unchanged binstall command minus the taplo line, and built taplo into the gitignored
`.tools/` with `cargo install taplo-cli@0.10.0 --locked --root .tools`. Both succeeded.
That is a workaround in the worktree, not a fix in any committed file.

Options for the maintainer, put to them when the session stopped:

1. Build taplo alone from source: `install-tools` gains a second line,
   `cargo install taplo-cli@0.10.0 --locked --root .tools` (a no-op once installed; CI's
   `.tools` cache holds the result), and binstall stays strict for the other five. The
   agent's recommendation.
2. Drop `--disable-strategies compile`, so taplo compiles through binstall. This loses the
   loud failure for any other tool.
3. Pin `taplo==0.9.3` in `pyproject.toml`'s dev group and call it with
   `uv run --frozen taplo` (the `Justfile`'s `format` and `toml-check`, and the fix
   config's `taplo-fmt` hook). This downgrades a minor version.
4. Settle it on `main` first, and resume this ticket afterwards.

The maintainer took none of the four: D02 (2026-09-24) moved every tool conda-forge
carries to pixi, where taplo 0.10.0 is packaged, and the CONVENTIONS.md change went
through `main` with D02, as §11 says.

### Pivot to D02

D02 landed on `main` after commit `8ddb2c2`. This branch merges (or rebases onto)
`main` first; `tickets/T00-foundation.md` conflicts and the `main` version wins. Then,
against the rewritten steps above:

- **Already right, keep:** everything from steps 3, 4, 5 (except `taplo.toml`'s
  exclude list), 9 to 13 as `8ddb2c2` has it; `rust-toolchain.toml`; the SHAs looked up
  for actions/checkout, Swatinem/rust-cache and actions/cache.
- **Replace:** `tools.txt` (one line); `Justfile` (the `PATH` export and the recipes
  D02 prints; extract from CONVENTIONS.md §4's fence again, as before); `pyproject.toml`
  (D02's text); both prek configs (the `exclude` lists and the five script-hook
  entries); `scripts/initialize.sh` (the step-7 sequence); `.github/actions/setup/action.yml`
  (D02's steps; look up `prefix-dev/setup-pixi` v0.10.2's SHA; drop the taiki-e and
  setup-uv SHAs); `.gitignore`, `.gitattributes`, `_typos.toml`, `lychee.toml`,
  `.markdownlint-cli2.jsonc`, `taplo.toml` (`.pixi` and `pixi.lock` in, `.venv` and
  `uv.lock` out).
- **Remove from the tree:** `uv.lock`, `.python-version`. Delete the gitignored
  `.venv/`.
- **Add:** `pixi.lock` from `pixi lock`; `docs/decisions/0011-tool-manager.md` as an
  eleventh stub, its manifest row, and the eleventh row in `docs/decisions/README.md`
  with the numbering sentence.
- **Then** step 14 and 15 as written, with the three new §12 claims.

### Remaining steps

1. Merge `main` (D02) into this branch and apply the pivot above.
2. Run `just lint`, `just check-docs`, `just check-agents`, `just check-specs`,
   `just analyse-specs`, `just wasm-check`, `just coverage` and `actionlint`
   individually; fix what they find. Likely candidates:
   `LICENSE` indentation under editorconfig-checker, and whether `just coverage` finds a
   denominator in a crate whose only item is a `const` (the ticket's open point).
3. `just initialize` then `time just check`. Then the same from a fresh clone in
   `ai_tmp/clone`, once everything is committed.
4. The Verification block, with its output quoted here, and the remaining §12 claims.
5. `status: done`, then commit on `ticket/t00-foundation`. Do not push.

### Handed back

The taplo design question, answered by D02.

### Open points settled

- Branch name: renamed to `ticket/t00-foundation` (maintainer, 2026-09-24).
- Prerequisites: the agent installed rustup and cargo-binstall and edited `~/.zshrc`
  rather than uninstalling pixi's rust (maintainer, 2026-09-24). Under D02 the
  per-machine cargo-binstall is unused; the pixi environment's own is what
  `just install-tools` runs.
- The taplo blocker: settled by D02 (maintainer, 2026-09-24).

## Open points

- Whether `rustup toolchain install` with no arguments installs the targets as well as
  the components on rustup's current release (CONVENTIONS.md §12). If not, the recipe
  becomes `rustup show`, which is a `main` follow-up because the `Justfile` is frozen.
- Whether `cargo llvm-cov` on a crate whose only code is a `const` reports a
  denominator at all, or reports "no coverage data" and exits non-zero. If the latter,
  the stub gains one trivially covered function (say, `pub const fn cells() -> u8`) in
  this ticket, and T02 inherits it.
- The wall-clock cost of a cold `just check` on the maintainer's machine, for the
  troubleshooting page and for deciding whether `features` belongs in the commit hook
  later (it does not now).
