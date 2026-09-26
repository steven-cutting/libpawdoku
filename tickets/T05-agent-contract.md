---
id: T05
title: "Agent contract: AGENTS.md, adapters, fourteen skills, their bridges"
status: done
depends_on: [T00]
parallel_with: [T01, T02, T03, T04, T06, T07, T08, T09]
branch: ticket/t05-agent-contract
estimated_size: M
---

# T05: Agent contract: AGENTS.md, adapters, fourteen skills, their bridges

## Context

Every Biscuit Games repository carries the same agent surface: `AGENTS.md` as the single
source of truth, two byte-pinned adapters (`CLAUDE.md` and
`.github/copilot-instructions.md`), canonical skills under `.agents/skills/<name>/SKILL.md`
and a fixed-body bridge for each skill under `.claude/skills/` and `.codex/skills/`. B's
`validate_agents.py` gates the whole surface on every `just check` (gate 15,
CONVENTIONS.md §4) and on every commit through the `validate-agents` hook. CONVENTIONS.md
§7 is the design for this repository's surface; read it in full, then §1 fact 1 and fact
5, §8 (decisions 0007, 0009 and 0010), §11 and §13 before anything below.

T00 merged first and placed a stub at every path this ticket owns: an `AGENTS.md` of 300
words carrying the six required phrases and one paragraph per section naming this ticket;
fourteen skill stubs with G's frontmatter (and the `rust-change` description T00 step 13
gives) and a 40-word body; and, in final form, the two adapters, `.claude/settings.json`
and all twenty-eight bridges. This ticket replaces the stubs with the final content and
adds the two paths T00 left out: `skills-lock.json` and `allium-skill-reference/`.

**C02 is a dependency of this ticket's final state.** D01 kept the Python checkers
(decision 0004), so `AGENTS.md` carries the honest `runes` sentence CONVENTIONS.md §7
quotes until C02 ships a `biscuit-games-tooling` release whose `REQUIRED_GUIDANCE` is
configurable. The follow-up pull request that bumps the pin in `pyproject.toml`, relocks
`pixi.lock` and adds `agent_guidance_drop = ["runes"]` removes the sentence and changes
this ticket's third verification block with it, after this ticket has merged.

**The trap this ticket must not walk into.** A bridge repeats its skill's frontmatter
verbatim (`validate_agents.py` line 223 compares the two dictionaries whole), and every
bridge is frozen at T00 (CONVENTIONS.md §11). So a change to any skill's `description`
is a change to three files, two of which this ticket may not touch. T00's fourteen
descriptions are therefore final, and this ticket writes skill **bodies** only. If a
description must change, stop, leave the frontmatter as T00 wrote it, and hand the change
back as a T00 follow-up pull request on `main` in the hand-back notes.

Sources, at the commits CONVENTIONS.md §0 pins (read-only; `git -C <clone> show
<commit>:<path>` if a clone has moved on):

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`. `AGENTS.md`, all 173 lines:
  1-6 the preamble; 8-24 "What this project is" (22-24 the untrusted-data paragraph);
  26-56 the seven invariants (30-32 invariant 1, 33-36 invariant 2, 37-42 invariant 3,
  43-47 invariant 4, 48-50 invariant 5, 51-53 invariant 6, 54-56 invariant 7); 58-88
  "Stack and conventions"; 90-105 "Change workflow" (92-100 the seven steps, 102-105 the
  never-invent-a-command paragraph); 107-125 "Safety and authority" (123-125 the
  licence-and-publishers paragraph); 127-138 "Documentation and durable context";
  140-144 "External automation policy"; 146-173 "Provenance" (148-155 the paragraph,
  157-168 the six-item list, 170-173 the deviations paragraph). The seven house skills
  `.agents/skills/{code-review,fix-quality,plan-change,project-check,review-docs,spec-change,svelte-change}/SKILL.md`;
  the seven Allium skills `.agents/skills/{allium,distill,elicit,propagate,tend,weed,witness}/SKILL.md`;
  `skills-lock.json`; the 23 files under `allium-skill-reference/`
  (`git -C /Users/scutting/projects/pawdoku ls-tree -r --name-only 78d03cdf allium-skill-reference`);
  `.claude/settings.json`; `docs/reference/agent-contract.md`; and commit `189348e`,
  whose message explains why the reference material sits at the root.
- B = `/Users/scutting/projects/biscuit_games_tooling` at `6c5c07f6`,
  `src/biscuit_games_tooling/validate_agents.py`, all 260 lines: `BRIDGE_BODY` 23-26,
  `MANAGED_DIRECTORIES` 27, `TOLERATED` 32, `ADAPTERS` 34-42, `REQUIRED_GUIDANCE` 47-54,
  the Git-based inventory 78-114, the literal frontmatter parser 117-140, the 300-word
  floor 170, the `CODEX.md` ban 172-173, the description floor of eight words 207, the
  body rule (`just` followed by a space, and `AGENTS.md`) 209, the bridge comparison 214-230 and the success
  line 255.
- T = `/Users/scutting/projects/biscuit_games_template` at `2283589c`,
  `tickets/T04-agent-contract.md`, the shape of an agent-contract ticket: exact
  replacement lines per step, `cmp` for the verbatim files, a diff-count loop for the
  edited ones.

Three facts found in these sources, now recorded in CONVENTIONS.md §1 fact 5 and §7,
that the ticket was first drafted against differently. The sources win:

1. The line in `allium/SKILL.md` that names `just` recipes is **line 10**, not line 30
   (line 30 is the `distill` row of the routing table).
2. `propagate/SKILL.md` line 12 also names `just frontend-unit`, and says tests live in
   `tests/`, "never colocated with `src/`", which contradicts decision 0009. So **five**
   Allium skills are byte-for-byte and **two** carry a one-line body edit, not seven and
   one. Neither edit touches frontmatter, so the bridges stay frozen.
3. `skills-lock.json` records a `computedHash` per skill that matches nothing in G's
   tree: the SHA-256 of G's `allium/SKILL.md` is `97798540…`, the lock says `52446c61…`,
   and neither the body alone nor the file without its final newline matches either. The
   lock and the already-edited skills landed in the same commit (`189348e`), so G's own
   edits (frontmatter trimmed, line 10 inserted, formatting fixed) had already broken the
   correspondence, and `validate_agents.py` never opens the lock. The hashes are
   provenance, not a check; see the Allium section of Steps.

## Goal

- `AGENTS.md` is the library's working agreement in G's shape: G's eight headings in
  order, the eight invariants of CONVENTIONS.md §7, a Rust stack, a change workflow whose
  recipe names are this repository's, the safety and automation policy carried, a
  Provenance section that says where the conventions came from and lists the three
  deliberate deviations, 300 or more words and the six phrases B requires.
- Fourteen canonical skills under `.agents/skills/`: six house skills rewritten for Rust
  at the lines named below, one new `rust-change` skill, five Allium skills byte-identical
  to G's and two Allium skills differing from G's at one line each; every frontmatter
  exactly what T00 wrote.
- `skills-lock.json` byte-identical to G's and `allium-skill-reference/` byte-identical to
  G's 23 files, so every relative link the seven Allium skills carry resolves.
- `just check-agents` prints `Validated AGENTS.md, 2 adapters, and 14 skills.`, and
  `just check` is green.

## Non-goals

- No change to `CLAUDE.md`, `.github/copilot-instructions.md`, `.claude/settings.json` or
  any of the twenty-eight bridges: final at T00, frozen (CONVENTIONS.md §11).
- No change to any skill's `name` or `description` (Context, the trap).
- No `accessibility-review` and no `svelte-change` skill, under any of the three roots.
- No handbook page. `docs/reference/agent-contract.md` is T08's (it describes fourteen
  skills, uses `rust-change` as its example frontmatter and states the reference-directory
  rule, CONVENTIONS.md §6); decision 0010 is T09's. Facts they need from this ticket go
  in the hand-back notes.
- No edit to `_typos.toml`, `lychee.toml`, `.markdownlint-cli2.jsonc` or `.gitattributes`
  (T03's) even though the vendored reference depends on their exclusions; a missing
  exclusion is handed back (Steps, "The reference material").
- No change to `docs/specs/` and no run of the Allium loop. The skills are installed, not
  exercised.

## Files touched

| Path | Class | Source | Change |
| --- | --- | --- | --- |
| `AGENTS.md` | stub -> final | G `AGENTS.md` | Rewritten section by section (Steps) |
| `.agents/skills/code-review/SKILL.md` | stub -> final | G, same path | Body: steps 2, 4, 5 (G lines 9, 11, 12) |
| `.agents/skills/fix-quality/SKILL.md` | stub -> final | G, same path | Body: the dispatch list (G lines 10-16) |
| `.agents/skills/plan-change/SKILL.md` | stub -> final | G, same path | Body: step 5 (G line 12), one noun phrase |
| `.agents/skills/project-check/SKILL.md` | stub -> final | G, same path | Body: step 2 (G line 9) |
| `.agents/skills/review-docs/SKILL.md` | stub -> final | G, same path | Body: step 3 (G line 10) dropped, steps renumbered |
| `.agents/skills/spec-change/SKILL.md` | stub -> final | G, same path | Body: steps 1, 4, 8 (G lines 10, 13, 17) |
| `.agents/skills/rust-change/SKILL.md` | stub -> final | new | Body written out in Steps |
| `.agents/skills/allium/SKILL.md` | stub -> final | G, same path | Byte-for-byte except line 10 |
| `.agents/skills/propagate/SKILL.md` | stub -> final | G, same path | Byte-for-byte except line 12 |
| `.agents/skills/{distill,elicit,tend,weed,witness}/SKILL.md` | stub -> final | G, same paths | Byte-for-byte |
| `skills-lock.json` | new | G, same path | Byte-for-byte |
| `allium-skill-reference/**` (23 files) | new | G, same paths | Byte-for-byte |
| `tickets/T05-agent-contract.md` | ticket | | `status:` and hand-back notes |

Not touched, because frozen: the twenty-eight bridges, `CLAUDE.md`,
`.github/copilot-instructions.md`, `.claude/settings.json`. Every frontmatter under
`.agents/skills/` stays byte-identical to T00's.

## Steps

Fenced blocks under a numbered step show file content starting at column 0. Run
everything from the worktree root. `G=/Users/scutting/projects/pawdoku` throughout.

### Prepare

1. Create the worktree on `ticket/t05-agent-contract` from `main` (README.md "How to pick
   up a ticket"), run `just initialize`, and confirm `just check` is green before
   changing anything. Read D01's hand-back notes ("Handed back", T05): the interim
   `runes` sentence is quoted there, and C02's follow-up removes it after this ticket
   merges.
2. Read `validate_agents.py` at the lines Context names and keep them open. In
   particular: the frontmatter parser splits every line between the `---` markers on its
   first colon and rejects a blank line (`invalid frontmatter line`), the two keys must
   be adjacent, and the inventory is `git ls-files --cached --others
   --exclude-from=.gitignore` restricted to `AGENTS.md`, the two adapters and the three
   skill roots. `allium-skill-reference/` and `skills-lock.json` are outside every managed
   directory, which is the whole reason they live where they do.
3. Record each T00 frontmatter before editing, so that the verification loop can prove
   nothing moved:

   ```sh
   mkdir -p ai_tmp/t05
   for n in $(ls .agents/skills); do sed -n 1,4p ".agents/skills/$n/SKILL.md" > "ai_tmp/t05/$n.frontmatter"; done
   ```

### `AGENTS.md`

Start from G's file (`git -C $G show 78d03cdf:AGENTS.md > AGENTS.md`), keep lines 1-6
verbatim, keep every heading in G's order, and rewrite the sections as follows. Two of
G's links target pages this repository dropped (line 72 `docs/project/platform.md`, line
151 `docs/how-to/update-from-template.md`); neither may survive, because `just check-docs`
runs lychee offline over this file. Every other link G carries (lines 20, 86-88, 171)
targets a page T00 stubbed, and stays.

1. **What this project is** (G lines 8-24). Replace lines 10-13 with one paragraph: the
   `pawdoku` crate in `crates/pawdoku` is the classic-sudoku engine behind Pawdoku, a
   Biscuit Games game; a `no_std` Rust library, Apache-2.0, in a Cargo workspace that
   will also hold a command-line, a Python and a WebAssembly crate; no UI, no persistence,
   no server. Keep lines 15-20 with "the game" changed to "the engine" and the skill
   name kept (`spec-change`). Keep lines 22-24, the untrusted-data paragraph, verbatim.
2. **Invariants** (G lines 26-56). Keep line 28 and replace the seven items with these
   eight, worded in G's register. G's invariant 1 (lines 30-32) is kept word for word;
   G's invariant 4 (lines 43-47, "Every dependency is pinned to an exact version. No `^`,
   no `~`, in `package.json` or `pyproject.toml`. Lockfiles are committed and
   `just lock-check` proves they match.") is replaced by item 8, which states the
   opposite policy and names decision 0007 as the reason:

   ```markdown
   1. **The specifications are the source of truth for behaviour.** No rule,
      threshold or wording that `docs/specs/` states is re-decided in code. When
      the code needs to differ, change the spec first and say why.
   2. **Effects sit behind traits.** Randomness is the only effect the engine
      has, reached through the boundary `crates/pawdoku/src/random.rs` defines,
      with a fake beside it that tests supply draws through. A clock, threads,
      the filesystem and the environment are not merely avoided but unnameable,
      because the core is `#![no_std]`. Limits are step budgets, never timeouts.
   3. **Every public type is `Send + Sync + 'static`, `Clone` and `Debug`.**
      Public enums and structs are `#[non_exhaustive]`; errors are `thiserror`
      enums with stable `Display` text, because pyo3 and wasm-bindgen build
      their exceptions from it.
   4. **`#![forbid(unsafe_code)]` in every crate.** Not `deny`: nothing may opt
      back in, and the workspace lint table says so.
   5. **The core compiles for `wasm32-unknown-unknown` and `wasm32v1-none`**
      under every feature combination. `just wasm-check` and `just features`
      prove it; the second target has no std at all, which is what makes the
      proof mechanical rather than conventional.
   6. **Clippy pedantic with the workspace lint table.** A lint is silenced
      with `#[expect(lint, reason = "...")]` on the item it names, never a
      blanket `allow`, so a suppression that stops being needed reports itself.
   7. **Coverage does not fall below the floor.** 90% of lines over
      `crates/pawdoku/src/**`. Lower the code's complexity, not the
      `coverage_floor` in the `Justfile`.
   8. **`Cargo.lock` is the pin.** Manifests write full `x.y.z` caret ranges;
      every gate passes `--locked`; `just lock-check` proves the lockfile
      matches. This is a stated deviation from the games' exact-pin rule ("no
      `^`, no `~`"), recorded in decision 0007: a library that writes `=1.2.3`
      poisons every downstream resolution.
   ```

3. **Stack and conventions** (G lines 58-88). Replace the bullets: Rust 2024 edition,
   pinned by `rust-toolchain.toml` to an exact stable release and honoured only by
   rustup's cargo; a Cargo workspace with `crates/pawdoku` as its only member today; the
   core is `#![no_std]` with `alloc`; rustfmt for Rust, taplo for TOML, clippy pedantic
   through the workspace lint table (the manifests say `warn`, `just clippy` turns
   warnings into errors); EditorConfig for whitespace and `markdownlint-cli2` for
   Markdown; tests in three places, each stated (unit tests in the module they test,
   integration tests under `crates/pawdoku/tests/`, doctests on every public item, which
   is decision 0009's departure from the games' "never colocated" rule at G lines
   73-75); fakes come through the randomness boundary, never a global; **Just** is the
   task runner and the only supported interface to the checks, with G's sentence on the
   read-only and fix prek configs (lines 81-84) carried and its "under `uv`" replaced:
   pixi owns the tools and the Python environment (`pyproject.toml` is the manifest,
   `pixi.lock` the pin) and rustup owns the compiler (decision 0011). Drop G's
   platform bullet (lines 69-72) and its stories
   bullet (76-80). Keep lines 86-88 (the four owning-page links).
4. **Change workflow** (G lines 90-105). Keep the seven steps; step 6 becomes "Run the
   narrowest recipe that covers the change (`just fmt-check`, `just clippy`, `just test`
   for code; `just check-docs` for a page; `just check-specs` for a module), then
   `just check` before handing back." Keep lines 102-105 verbatim.
5. **Safety and authority** (G lines 107-125). Keep lines 109-121 verbatim, with
   "enabling GitHub Pages" replaced by "publishing a crate" in the list at line 112. Replace
   lines 123-125 (no licence; two publishing workflows) with: this repository is
   Apache-2.0 (decision 0006); no workflow publishes anything, `audit.yml` only reads the
   advisory database, and crates.io publishing is a later decision (S02) that will use
   trusted publishing and never a token in this repository.
6. **Documentation and durable context** (G lines 127-138). Keep the four bullets, and
   add to the third: the seven Allium skills' reference material lives at
   `allium-skill-reference/` at the repository root, because the agent contract permits
   one file per skill under `.agents/` (decision 0010); a skill links to it relatively
   and nothing else does. Keep the `just check-agents` and `just check-docs` sentence.
7. **External automation policy** (G lines 140-144). Verbatim, with "deploying" dropped
   from the list.
8. **Provenance** (G lines 146-173). Replace the whole section body with the block below.
   The last sentence is CONVENTIONS.md §7's honest `runes` sentence, required until C02
   ships (decision 0004).

   ```markdown
   This repository is not rendered by Copier. Its conventions were taken by
   hand from Pawdoku, the game this engine serves, at commit `78d03cdf`, and
   through it from the `biscuit_games_template` template at `v2.0.0`
   (<https://github.com/steven-cutting/biscuit_games_template>) and from Poodl,
   the first Biscuit Games game. The tickets under `tickets/` record what was
   carried and what was changed. What the house decides is recorded in the
   decision records under `docs/decisions/`:

   - Specifications in Allium as the source of truth for behaviour, checked by a
     project-managed binary the `biscuit-games-tooling` package pins and installs.
   - Effects behind a boundary the library owns, with a fake beside it.
   - A small Python toolchain (`prek` and the `biscuit-games-tooling` package,
     installed by pixi) for the hook gate and the two validators; Markdown
     formatted by `markdownlint-cli2` alone.

   Deliberate deviations from the games' conventions, each recorded in
   the decision records (in the real file, a relative Markdown link to
   `docs/decisions/README.md`; written as a code span here because the ticket gate
   runs lychee offline over `tickets/`):

   - Dependencies are caret ranges pinned by `Cargo.lock`, not exact pins in the
     manifests (decision 0007).
   - Unit tests live in the module they test, beside integration tests under
     `tests/` and doctests on the public API (decision 0009).
   - The Allium skills' reference material lives at `allium-skill-reference/`
     at the repository root, outside `.agents/` (decision 0010).

   This repository has no Svelte runes; the games' reactivity rule does not
   apply here.
   ```

9. Check the result before moving on: `wc -w AGENTS.md` is 300 or more (expect about
   1000); each of `untrusted`, `just check`, `explicit authorization`, `ai_tmp/`,
   `docs/specs/` and `runes` occurs, case-insensitively; `svelte`, `runes`, `npm`,
   `storybook`, `chromatic`, `copier update` and `src/lib` occur nowhere except the
   one Provenance sentence.

### The six house skills

Start each from G's file (`git -C $G show 78d03cdf:.agents/skills/<name>/SKILL.md`),
then paste T00's frontmatter back over lines 1-4 (`ai_tmp/t05/<name>.frontmatter`) so
that the frontmatter is provably T00's, and change only the lines named. Every other
line, including the single long line per step, stays G's.

1. `code-review`: steps 2, 4 and 5 (G lines 9, 11, 12) become:

   ```markdown
   2. Check the invariants in `AGENTS.md` one at a time. Specs decide behaviour, randomness behind the boundary and no other effect, `Send + Sync` and `#[non_exhaustive]` on every public type, no `unsafe`, both wasm targets, pedantic clippy with `expect` only, the coverage floor intact, `Cargo.lock` the pin.
   4. Check the test evidence. A test that only asserts a function was called is not evidence; a test that supplies draws through the fake and asserts what the engine did with them is. A public item without a doctest that asserts is a finding.
   5. Check the boundaries. Nothing outside `crates/pawdoku/src/random.rs` may name a source of draws, nothing in the core may reach `std`, and no test may replace the boundary with a global or a thread-local.
   ```

2. `fix-quality`: the dispatch list (G lines 10-16) becomes, keeping G's three
   unchanged entries where shown:

   ```markdown
      - `fmt-check` or `toml-check` — run `just fix`; never hand-format to match.
      - `clippy` — fix the code. A lint that is wrong for one item gets `#[expect(lint, reason = "...")]` on that item; a lint that is wrong for the crate is changed in the workspace lint table in `Cargo.toml` with a comment saying why, never disabled at the call site with `allow`.
      - `features` or `wasm-check` — a `std` path, or a feature that does not compile alone. The fix is `cfg`, `core` and `alloc`; a target is never dropped.
      - `coverage` below the floor — add the missing test. Never lower `coverage_floor` in the `Justfile`.
      - `doc` — a missing doc comment or a broken intra-doc link; `missing_docs` is on and warnings are errors there.
      - `deny` or `deps-unused` — a licence, a ban or an unused dependency; a new dependency is a decision recorded per decision 0007, and `just fix` runs `cargo shear --fix`.
      - `check-docs` — a frontmatter list that disagrees with `docs/manifest.yml` is usually list order; the comparison is order-sensitive.
      - `check-agents` — a bridge under `.claude/` or `.codex/` has grown content, or a managed file is missing from the inventory.
      - `check-specs` or `analyse-specs` — hand to the `spec-change` skill; a diagnostic is a regression and a finding cannot be waived.
      - `lock-check` — run `just lock`, and read the lockfile diff before accepting it.
   ```

3. `plan-change`: step 5 (G line 12) changes one noun phrase, "a new side effect needs a
   port and a fake" becomes "a new effect needs a trait at the randomness boundary and a
   fake, and almost certainly a decision first"; the rest of the line stays.
4. `project-check`: step 2 (G line 9) becomes:

   ```markdown
   2. Check the prerequisites exist: rustup's cargo first on `PATH` (`just check-toolchain` says so), the pixi environment at `.pixi/envs/default/bin` (nextest, llvm-cov, deny, shear, taplo, prek, just, the `bg-*` scripts), `.tools/bin/cargo-hack`, the pinned checker at `.tools/bin/allium`, `Cargo.lock` and `pixi.lock`. If any is missing, run `just initialize` — it creates them, normalises formatting and installs the hook, and it never stages, commits, tags or pushes. The checker alone is `just install-allium`; it is gitignored and per-worktree, so a fresh worktree needs it before gates 3, 16 and 17 can pass.
   ```

   Gates 3, 16 and 17 are `lint`, `check-specs` and `analyse-specs` in
   CONVENTIONS.md §4's order; `lint` needs the binary because the two spec hooks run
   inside it.
5. `review-docs`: delete step 3 (G line 10, the managed/seed step; nothing here arrives
   by `copier update`) and renumber G's steps 4-8 to 3-7.
6. `spec-change`: steps 1, 4 and 8 (G lines 10, 13, 17) become:

   ```markdown
   1. Read `AGENTS.md` and `docs/explanation/specifications.md`. Identify which module under `docs/specs/` owns the behaviour; each module's header states its Scope, Includes and Excludes. The game's root module stays in Pawdoku and restates what it needs from these seven; nothing here imports it.
   4. Check that dependent modules still hold. A module others import is depended on by each of them, so a change to a trigger or an entity ripples; and a clause the game restates is held to this repository's text by a test on the game's side, so a wording change here is a change the game must take.
   8. Run `just test`, then `just check` before handoff.
   ```

### The new `rust-change` skill

Keep T00's frontmatter (`name: rust-change` and the description T00 step 13 gives) and
replace the stub body with exactly this:

```markdown
# Change the engine

1. Read `AGENTS.md` and `docs/explanation/architecture.md`. Identify which module of `crates/pawdoku` the change lands in and whether it touches the public API; a public item is a contract the Python and WebAssembly crates will build on, so its shape is part of the change.
2. Find the clause. Every behaviour has an owning module under `docs/specs/`; read the rule, its guards and its outcomes before writing code. If no clause says what you need, this is a `spec-change` first, not a judgement call in code.
3. Put any effect behind the trait boundary. The only effect the engine has is randomness, reached through the trait `crates/pawdoku/src/random.rs` defines and its fake. A change that wants a clock, a thread, the filesystem or the environment has found a design problem, not a missing import: the core is `#![no_std]` and cannot name them. Limits are step budgets, never timeouts.
4. Derive the test from the clause before implementing, and confirm it fails first. A unit test lives in the module beside the code it tests; an integration test under `crates/pawdoku/tests/` drives the public API through the fake; a public item carries a doctest that asserts, because doc examples are tests. A test already green proves nothing about the new behaviour.
5. Import from `core` and `alloc`, never `std`; `#[cfg(test)] extern crate std;` is the one exception and it stays under `cfg(test)`. Every public type is `Send + Sync + 'static`, `Clone` and `Debug`; public enums and structs are `#[non_exhaustive]`; errors are `thiserror` enums whose `Display` text is stable.
6. Silence a lint only with `#[expect(lint, reason = "...")]` on the item it names, never a blanket `allow`, and never write `unsafe`; the crate forbids it.
7. Run `just fmt-check`, `just clippy`, `just test` and `just wasm-check`, then `just check` before handoff. A bare `std::` path already fails `clippy` and `test` on the host; `wasm-check` is what catches a dependency that needs std, or a `cfg`-gated path that only ever compiled there.
8. Report by `file:line`: what changed, which clause it implements, and which test proves it.
```

### The seven Allium skills and the lock

1. Copy G's seven files over the stubs, then confirm that the frontmatter of each is
   unchanged from T00's (T00 copied G's descriptions, so this is a check, not an edit):

   ```sh
   for n in allium distill elicit propagate tend weed witness; do
     git -C $G show "78d03cdf:.agents/skills/$n/SKILL.md" > ".agents/skills/$n/SKILL.md"
     cmp <(sed -n 1,4p ".agents/skills/$n/SKILL.md") "ai_tmp/t05/$n.frontmatter" && echo "$n frontmatter unchanged"
   done
   ```

   A `cmp` that reports a difference means T00 wrote a description that is not G's;
   stop and hand it back rather than editing either side.
2. `allium/SKILL.md` line 10 names `just frontend-unit`, a recipe this repository does
   not have. G's line reads:

   ```markdown
   In this repository, `AGENTS.md` decides how the loop below fits the project: the specification is the source of truth for behaviour, and every phase's output is proven by a `just` recipe — `just check-specs` and `just analyse-specs` for the spec itself, `just frontend-unit` for the tests it produces, and `just check` as the full gate before handoff.
   ```

   It becomes, changing only the recipe name:

   ```markdown
   In this repository, `AGENTS.md` decides how the loop below fits the project: the specification is the source of truth for behaviour, and every phase's output is proven by a `just` recipe — `just check-specs` and `just analyse-specs` for the spec itself, `just test` for the tests it produces, and `just check` as the full gate before handoff.
   ```

3. `propagate/SKILL.md` line 12 names the same recipe and the games' test-location rule.
   G's line reads:

   ```markdown
   This repository's `AGENTS.md` sets those conventions: tests live in `tests/`, never colocated with `src/`, and `just frontend-unit` is the recipe that proves the generated tests before handoff.
   ```

   It becomes the decision-0009 wording:

   ```markdown
   This repository's `AGENTS.md` sets those conventions: unit tests live in the module they test, integration tests under `crates/pawdoku/tests/`, doc examples are tests, and `just test` is the recipe that proves the generated tests before handoff.
   ```

   No other line of any Allium skill changes. `elicit/SKILL.md` line 286 contains the
   word "frontend" in upstream example text; it stays, which is why the verification
   grep below matches `frontend-unit` and not `frontend`.
4. `skills-lock.json`: `git -C $G show 78d03cdf:skills-lock.json > skills-lock.json`,
   byte-for-byte, hashes untouched. The lock is provenance: it records that the seven
   skills came from `juxt/allium` at `skills/<name>/SKILL.md` and what the installer
   hashed on the day. As Context fact 3 shows, the recorded hash already matches nothing
   in G, because G edited the skills in the same commit that added the lock; the two
   line edits above therefore change nothing any check reads, and nothing is
   recomputed. What the installer hashed (the upstream file, or the upstream skill
   directory with its reference material) is not verifiable offline and is an open
   point. The `validate-agents` hook fires on a change to this file (CONVENTIONS.md §5)
   so that a future re-install is reviewed against the contract, even though the
   validator does not read the file.

### The reference material

1. Copy G's 23 files and prove the bytes with the tree hashes, which needs no network:

   ```sh
   git -C $G archive 78d03cdf allium-skill-reference | tar -x
   git add allium-skill-reference
   diff <(git -C $G ls-tree -r 78d03cdf allium-skill-reference | awk '{print $3, $4}') <(git ls-files -s allium-skill-reference | awk '{print $2, $4}') && echo "reference identical"
   ```

   Equal blob ids are equal bytes, so this proves the copy without a network, and it
   reads the index rather than `HEAD` so that it works before the commit exists.
   Verification runs the same comparison.
2. The directory is 23 files and about 360 KB; the two largest are
   `allium/language-reference.md` (107 KB) and `allium/patterns.md` (94 KB), both under
   the 768 KB hook limit and both excluded from every hook anyway. The exclusions this
   ticket relies on and does not own: `allium-skill-reference/` in the frozen
   `.pre-commit-config.yaml` `exclude` (CONVENTIONS.md §5), in `.markdownlint-cli2.jsonc`
   `ignores`, in `lychee.toml` `exclude_path`, in `_typos.toml` `extend-exclude` and as
   `linguist-vendored` in `.gitattributes` (all T03's working files from T00). If
   `just lint` reports anything under the directory, the exclusion is missing: hand it
   back to T03 (a dotfile) or as a T00 follow-up on `main` (the hook config), and do not
   edit the vendored file.
3. The seven skills link into the directory with `../../../allium-skill-reference/...`
   paths and, in three places, heading anchors (`language-reference.md#contracts`,
   `#invariants`, `assessing-specs.md#communicating-with-stakeholders`). The skills are
   not lychee-excluded, so those links are checked offline by `just check-docs`; they
   resolve because the directory and the headings are G's, where the same check is
   green.

### Check, commit, hand back

1. Run the Verification blocks and quote their output in the hand-back notes.
2. Run `just check`; every gate green, ending with the clean-worktree line.
3. Write the hand-back notes: for T08, the facts `docs/reference/agent-contract.md` must
   state (fourteen skills; the `rust-change` frontmatter as the example; the rule that a
   skill's reference material lives at `allium-skill-reference/` and is linked relatively;
   the literal-parser caveat G's page already carries); for T09, what decision 0010 must
   record (G's `189348e` message; the hash mismatch of Context fact 3; that the
   validator, not taste, forced the location); for the maintainer, the CONVENTIONS.md §7
   corrections of Context.
4. Set `status: done` and commit on the ticket branch, in one or a few commits with
   short imperative subjects. **Authorisation required:** pushing and opening the pull
   request are separately authorised (CONVENTIONS.md §11); stop and ask.

## Acceptance criteria

- `just check-agents` prints `Validated AGENTS.md, 2 adapters, and 14 skills.`;
  `just check-docs` and `just lint` are green with the vendored reference present;
  `just check` is green.
- `AGENTS.md` has G's eight headings in order, 300 or more words, the six phrases, the
  eight invariants of CONVENTIONS.md §7 with decision 0007 named in the eighth, and a
  Provenance section naming decisions 0007, 0009 and 0010; it carries the `runes`
  sentence exactly as CONVENTIONS.md §7 quotes it.
- `.agents/skills/` holds exactly the fourteen names; every frontmatter is byte-identical
  to T00's; the five Allium skills `distill`, `elicit`, `tend`, `weed`, `witness` are
  `cmp`-equal to G's; `allium` and `propagate` differ from G's at one line each; the six
  house skills differ from G's only at the lines Steps name; `rust-change` has the body
  Steps give.
- No file under `.agents/` contains `frontend-unit`, `src/lib`, `platformSpecs`,
  `colocated`, `runes` or `svelte`.
- `skills-lock.json` is `cmp`-equal to G's; `allium-skill-reference/` has the same 23
  blob ids as G's tree.
- `git ls-files .agents .claude .codex` lists 43 paths: 14 canonical skills, 28 bridges,
  `.claude/settings.json`. The bridges, the adapters and `.claude/settings.json` are
  byte-identical to T00's (`git diff main -- .claude .codex CLAUDE.md .github/copilot-instructions.md` is empty).
- No file outside the Files touched table changed, other than this ticket's `status:`
  line and hand-back notes.

## Verification

```sh
G=/Users/scutting/projects/pawdoku
just check-agents
just check-docs
just lint
just check
git status --porcelain
```

Expected: `Validated AGENTS.md, 2 adapters, and 14 skills.`; the docs and lint runs
green; `just check` ending with the clean-worktree line and exit 0; nothing from
`git status`.

```sh
wc -w AGENTS.md
for p in untrusted 'just check' 'explicit authorization' 'ai_tmp/' 'docs/specs/' runes; do printf '%s: ' "$p"; grep -c -i -F -- "$p" AGENTS.md; done
grep -n -i -E 'svelte|runes|npm|storybook|chromatic|copier update|src/lib|platform\.md|update-from-template' AGENTS.md
grep -n '^## ' AGENTS.md
```

Expected: 300 or more (about 1000); six counts of 1 or more (once C02's follow-up
lands, `runes` is 0 and the phrase list no longer includes it); the third command
prints exactly one line (the Provenance sentence); the headings are G's eight in G's
order.

```sh
ls .agents/skills .claude/skills .codex/skills
git ls-files .agents .claude .codex | wc -l
for n in $(ls .agents/skills); do cmp <(sed -n 1,4p ".agents/skills/$n/SKILL.md") "ai_tmp/t05/$n.frontmatter" || echo "FRONTMATTER MOVED: $n"; done
git diff --stat main -- .claude .codex CLAUDE.md .github/copilot-instructions.md
grep -rn -i -E 'frontend-unit|src/lib|platformSpecs|colocated|runes|svelte' .agents/; echo "exit $?"
```

Expected: three listings of the same fourteen names; `43`; no `FRONTMATTER MOVED` line;
an empty diff; nothing printed and `exit 1`.

```sh
for n in distill elicit tend weed witness; do cmp ".agents/skills/$n/SKILL.md" <(git -C $G show "78d03cdf:.agents/skills/$n/SKILL.md") && echo "$n identical"; done
for n in allium propagate; do printf '%s: ' "$n"; diff <(git -C $G show "78d03cdf:.agents/skills/$n/SKILL.md") ".agents/skills/$n/SKILL.md" | grep -c '^[<>]'; done
for n in code-review fix-quality plan-change project-check review-docs spec-change; do printf '%s: ' "$n"; diff <(git -C $G show "78d03cdf:.agents/skills/$n/SKILL.md") ".agents/skills/$n/SKILL.md" | grep -c '^[<>]'; done
cmp skills-lock.json <(git -C $G show 78d03cdf:skills-lock.json) && echo "lock identical"
diff <(git -C $G ls-tree -r 78d03cdf allium-skill-reference | awk '{print $3, $4}') <(git ls-files -s allium-skill-reference | awk '{print $2, $4}') && echo "reference identical"
git ls-files allium-skill-reference | wc -l
```

Expected: five `identical` lines; `allium: 2` and `propagate: 2`; for the six house
skills, `code-review: 6`, `fix-quality: 17` (seven lines out, ten in), `plan-change: 2`,
`project-check: 2`, `review-docs: 11` (G's steps 3-8 out, five renumbered steps in),
`spec-change: 6`, with any other count
explained line by line in the hand-back notes;
`lock identical`; `reference identical`; `23`.

## Hand-back notes

**Done on 2026-09-25**, in one session, on the Supacode worktree's branch
`T05-agent-contract` (see Deviations). The maintainer authorised pushing that branch
and opening its pull request on 2026-09-25; nothing else was published. No clone (G, B, T) was modified: every read of G used `git -C $G show 78d03cdf:`
or `git -C $G archive 78d03cdf`.

Actions with effects outside the tracked tree: `just initialize` in this worktree (the
gitignored `.pixi/envs/default`, `.tools/bin/cargo-hack` 0.6.45 and `.tools/bin/allium`
3.6.1, downloaded; pixi's and cargo's caches). It is a secondary worktree, so the script
skipped `install-hooks`; the shared shim at the primary's `.git/hooks/pre-commit`
already named the primary checkout's `.pixi/envs/default/bin/prek`, which exists, and
was not rewritten.

### What was verified, and how

Before any change, `just check` was green on the untouched branch (exit 0, "All checks
passed and the worktree is unchanged."). The T00 frontmatter of all fourteen skills was
recorded to `ai_tmp/t05/<name>.frontmatter` first (Prepare step 3).

First block, on the final tree (these notes included) before the commit:

```text
$ just check-agents
Validated AGENTS.md, 2 adapters, and 14 skills.
$ just check-docs
lychee...Passed
Validated 37 pages and 38 canonical topics.
$ just lint
prek run --all-files          (every hook Passed or Skipped; exit 0)
$ just check
==> just check-toolchain, lock-check, lint, fmt-check, toml-check, clippy, features,
    wasm-check, test-doc, coverage, doc, deny, deps-unused, check-docs, check-agents,
    check-specs, analyse-specs, check-clean
The worktree matches the check baseline.
All checks passed and the worktree is unchanged.     (exit 0)
```

`git status --porcelain` is empty after the commit.

Second block:

```text
$ wc -w AGENTS.md
1445
untrusted: 1
just check: 5
explicit authorization: 1
ai_tmp/: 1
docs/specs/: 2
runes: 1
$ grep -n -i -E 'svelte|runes|npm|storybook|chromatic|copier update|src/lib|platform\.md|update-from-template' AGENTS.md
179:This repository has no Svelte runes; the games' reactivity rule does not
$ grep -n '^## ' AGENTS.md
8:## What this project is
26:## Invariants
60:## Stack and conventions
90:## Change workflow
108:## Safety and authority
129:## Documentation and durable context
146:## External automation policy
152:## Provenance
```

Third block: the three listings are the same fourteen names; `git ls-files .agents
.claude .codex | wc -l` is `43`; the frontmatter loop printed nothing (no `FRONTMATTER
MOVED`); `git diff --stat main -- .claude .codex CLAUDE.md
.github/copilot-instructions.md` is empty; the forbidden-word grep over `.agents/`
printed nothing and `exit 1`.

Fourth block:

```text
distill identical
elicit identical
tend identical
weed identical
witness identical
allium: 2
propagate: 2
code-review: 6
fix-quality: 11
plan-change: 2
project-check: 2
review-docs: 11
spec-change: 6
lock identical
reference identical
      23
```

The Allium skills' frontmatter check of Steps (the `cmp` loop after copying G's seven
files) printed `<name> frontmatter unchanged` for all seven, so T00's descriptions are
G's and no bridge needed to move. The five exclusions the vendored reference relies on
are all present (`.pre-commit-config.yaml` line 10, `.markdownlint-cli2.jsonc` line 21,
`lychee.toml` line 12, `_typos.toml` line 11, `.gitattributes` line 5), and `_typos.toml`
already carries G's `mis` allowlist, which the non-excluded Allium skills need.

### Deviations, and why

- **`fix-quality: 11`, not 17.** The expected count assumed all seven of G's dispatch
  lines are replaced. Three of them (`check-docs`, `check-agents`, `lock-check`) are kept
  byte-for-byte, as the step says, and `diff` pairs them with themselves: four lines out
  (G's Prettier/Ruff, ESLint, `svelte-check` and coverage entries) and seven in
  (`fmt-check`/`toml-check`, `clippy`, `features`/`wasm-check`, `coverage`, `doc`,
  `deny`/`deps-unused`, `check-specs`/`analyse-specs`). The file is exactly what the
  step prescribes.
- **`AGENTS.md` is 1445 words**, not about 1000. Every section follows the step that
  governs it; the estimate was low, and the floor is 300.
- **One clause beyond the Stack step's list.** The `Just` bullet ends: "the one tool
  conda-forge lacks is pinned in `tools.txt` and installed with the Allium checker into
  the gitignored `.tools/bin`." T00's stub promised the section would name `tools.txt`,
  and the statement is true of the `Justfile` and `scripts/initialize.sh`. Drop it if the
  maintainer prefers the step's list exactly.
- **Wording choices inside the Stack step.** The tests bullet says unit tests sit
  "under `#[cfg(test)]`" and renders G's rule as "the games' rule that tests are never
  colocated with the code"; the fakes bullet adds "a test supplies its draws to the fake
  that sits beside the trait". The `prek` sentence drops "under `uv`" and puts the pixi
  and rustup ownership in the sentence after it.
- **Safety bullet rewrapped.** G lines 111-114 with "enabling GitHub Pages" replaced by
  "publishing a crate" no longer fit G's line breaks; the words are otherwise G's.
  "deploying" stays in that list, because the step names only the one substitution
  there; it is dropped from External automation policy, as that step says.
- **`project-check` step 2 differs from the text Steps prescribe.** Copilot's review of
  pull request 1 found that the prescribed line told an agent to run `just initialize`
  whenever a prerequisite is missing, although the command downloads and `AGENTS.md`
  requires explicit authorization for each network operation (G carries the same
  contradiction). At the maintainer's choice on 2026-09-25 the line now says the
  command is a network operation that needs authorization first, unless the task
  already grants it (a ticket's Prepare step does), and that `just install-allium`
  falls under the same rule. It is still one line, so `project-check: 2` holds.
- **Branch name.** Supacode created the worktree on `T05-agent-contract`, not the
  `branch:` field's `ticket/t05-agent-contract`. Asked on 2026-09-25, the maintainer
  chose to keep the Supacode name, so the `branch:` field above is not the branch this
  work was pushed from.

### Handed back

- **For T08 (`docs/reference/agent-contract.md`).** Fourteen canonical skills: six
  carried house skills (`code-review`, `fix-quality`, `plan-change`, `project-check`,
  `review-docs`, `spec-change`), one new (`rust-change`), seven vendored Allium skills
  (`allium`, `distill`, `elicit`, `propagate`, `tend`, `weed`, `witness`); twenty-eight
  bridges; `.claude/settings.json` tolerated, not expected. The example frontmatter,
  exactly as committed:

  ```markdown
  ---
  name: rust-change
  description: Implement or review a change to the engine with the trait boundary, no_std, tests and documentation evidence the invariants require.
  ---
  ```

  The reference rule: the seven Allium skills' reference material lives at
  `allium-skill-reference/` at the repository root and is linked from the skills with
  relative `../../../allium-skill-reference/...` paths; nothing else links to it. The
  literal-parser caveat is G's page lines 55-56 ("The frontmatter parser is deliberately
  literal. It splits every line between the `---` markers on the first colon and treats
  a line without one as an error, so a blank line ..."); B at `6c5c07f6` still behaves
  so. `skills-lock.json` is read by no validator.
- **For T09 (decision 0010).** G's `189348e` message: "The skills' reference material
  (language reference, patterns, worked examples, JSON schemas) moved to
  allium-skill-reference/ at the repo root, since the Agent instruction contract only
  permits a single SKILL.md per managed skill. That relocation is a template departure
  worth a decision record; noted for the PR rather than written here." The validator
  forced the location, not taste: `_check_inventory` (B lines 141-148) reports every
  Git-visible file under `.agents/`, `.claude/` or `.codex/` that is neither an expected
  skill or bridge nor `.claude/settings.json` as an "unexpected managed file". The hash
  mismatch, re-verified: `git -C $G show 78d03cdf:.agents/skills/allium/SKILL.md |
  shasum -a 256` is `97798540d089e3a583eaa34eeb3f355332553d869bd6067aaead23b1b79e10f2`;
  the lock's `computedHash` for `allium` is
  `52446c6173badac1804a635b3d8d39f8b2ed137222b0d7dbcb6591ebb928c0a0`.
- **For the maintainer (CONVENTIONS.md §7).** Re-verified: §7 already records line 10,
  line 12 and five/two. One difference remains: §7's `spec-change` row says "Steps 4, 7
  and 8", but the edited steps are 1, 4 and 8. Step 1 (G line 10) names
  `@steven-cutting/biscuit-games`; step 7 is kept verbatim, because everything it names
  exists here (`just check-specs`, `just analyse-specs`, `just install-allium`,
  `docs/how-to/work-with-the-specs.md`). Also, this ticket's fourth verification block
  should expect `fix-quality: 11`.
- **For T02 and T04.** `AGENTS.md`, `code-review` and `rust-change` name
  `crates/pawdoku/src/random.rs` (T02 step 7 creates it) and `AGENTS.md` names
  `audit.yml` (T04). Both are code spans, so no gate checks them today; they must land
  at those paths.
- **Upstream defects in the vendored Allium material (for `juxt/allium`).** Copilot's
  review of pull request 1 raised twelve findings against files this ticket keeps
  byte-identical to G. At the maintainer's choice they are recorded, not patched, so
  the `cmp` and blob-id checks and the lock's provenance stay true. Verified defects:
  `distill/worked-examples.md` line 811 calls `userRepository`, which the example never
  declares; lines 818 and 843 pass a document id to `findAllIncludingDeleted`, whose
  parameter (line 768) is a workspace id; `witness/SKILL.md` check 6 names four
  convergence conditions and omits the empty `uncovered_obligations` that
  `driving-the-loop.md` line 137 requires, while check 3 tolerates explained uncovered
  obligations; and `migration-v2-to-v3.md` lines 110-112 declare `deleted -> active`
  beside `terminal: deleted`. The other findings are design critiques rather than
  defects: optional `phase`, `open_questions` and routing fields in five result schemas
  and `phases` in `trace-entry.schema.json`, although the skills say those fields are
  always emitted; `tests.failed` and "a fresh `distill` finds nothing new" in the
  convergence formula (line 137), which no typed record carries; and a witness verdict
  of PASS while some checks are INCONCLUSIVE. Filing them upstream is an external
  action for the maintainer. One further finding, that `allium/SKILL.md` line 18
  ("integration and end-to-end tests (not unit tests)") contradicts decision 0009, was
  judged not a defect: it says what a specification generates, not where unit tests
  may live.
- **C02 follow-up, unchanged.** The `runes` sentence closes Provenance, as CONVENTIONS.md
  §7 quotes it; it goes when the B pin moves (Context).

### Open points settled

- **Where the reference material lives.** At the root, as G has it; the maintainer did
  not ask otherwise.
- **What the lock's hash covers.** Not settled: it needs the installer's source or a
  fetch of `juxt/allium`, and this ticket used no network for it. The lock is committed
  byte-for-byte as provenance.
- **CONVENTIONS.md §7.** Re-verified on the day; the one difference is handed back above.

## Open points

- **Where the reference material lives.** `allium-skill-reference/` at the root is G's
  choice and decision 0010 records it. The alternative, a `docs/` subtree, would put the
  23 files inside the documentation contract's walk, where every page must be registered
  in `docs/manifest.yml` with 40 words and an owning topic, which vendored upstream
  material cannot satisfy; and `docs/manifest.yml` is frozen. The root it is, unless the
  maintainer wants the record to say otherwise.
- **What the lock's hash covers.** The `computedHash` values match neither G's files nor
  any obvious transform of them (Context fact 3). Whether the installer hashed the
  upstream `SKILL.md` before G's edits, or the whole upstream skill directory including
  the material now under `allium-skill-reference/`, needs the installer's source or a
  network fetch of `juxt/allium`, which this ticket does not perform. Until that is
  known, the lock is a record of origin and a re-install would rewrite it; the
  maintainer may prefer to drop the file, in which case CONVENTIONS.md §3, §5 and §7 and
  decision 0010 change together.
- **CONVENTIONS.md §7 already records** that the recipe line in `allium/SKILL.md` is
  line 10, that `propagate/SKILL.md` line 12 is also edited, and that the count is five
  byte-for-byte and two edited; re-verify on the day and hand back only a difference.
