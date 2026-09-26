---
id: T08
title: "Handbook B: explanation, reference and operations pages"
status: open
depends_on: [T00]
parallel_with: [T01, T02, T03, T04, T05, T06, T07, T09]
branch: ticket/t08-handbook-b
estimated_size: L
---

# T08: Handbook B: explanation, reference and operations pages

## Context

T00 has merged and left a stub at every page CONVENTIONS.md §6 registers: frontmatter
equal to the `docs/manifest.yml` entry, an H1 equal to the title and forty or more words
saying what the page will hold, so `just check-docs` is green before any lane starts.
This ticket replaces the fourteen stubs under `docs/explanation/`, `docs/reference/` and
`docs/operations/` that §6 assigns to T08 with the real pages. T06 owns the two migrated
explanation pages, T07 the project, tutorial and how-to pages, T09 the decisions; all run
in parallel, and a link from this lane into theirs resolves today because a stub exists
at every path.

Thirteen pages are rewritten from G's page of the same path (G =
`/Users/scutting/projects/pawdoku` at `78d03cdf`, CONVENTIONS.md §0); `reference/api.md`
is new. G describes a static Svelte site with a component workshop, a Pages deployment
and a platform package; none of that exists here. A fact a page states comes from
CONVENTIONS.md, from another ticket this one names, or from the G lines it cites;
nothing is invented.

Read first: CONVENTIONS.md in full, with §2, §4, §5, §6 (this ticket's page table), §7,
§10, §11 and §13 a second time; `tickets/T00-foundation.md` steps 4, 9, 10 and 13 (the
workspace manifest and `lib.rs`, the manifest rows that are the frontmatter authority,
what a stub is, the skill descriptions); `tickets/T02-rust-gate.md` (the tool configs
the configuration page describes, which land in parallel); D01's and D02's hand-back
notes (decision 0004, then the `pyproject.toml`, `tools.txt` and recipe texts D02
holds); and G's thirteen source pages, each with
`git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/<path>`. Line numbers below
are of those files at `78d03cdf`, verified on 2026-09-23; re-verify every cited range
with `sed -n 'a,bp'` before splicing it.

Several ranges kept verbatim name B's console scripts (`bg-project-check`,
`bg-validate-docs`, `bg-validate-agents`, `bg-run-allium`) and `pyproject.toml`; that is
what this repository runs (decision 0004), so "verbatim" below means verbatim.

## Goal

Fourteen pages that read as this library's handbook and pass every gate: `just check-docs`
green, `just check` green, every relative link resolving to a page in `docs/manifest.yml`,
no sentence that is true of the game and false of the library. `reference/commands.md`
lists every recipe in CONVENTIONS.md §4 and no other; `reference/quality-gates.md` lists
the seventeen gates in §4's order; the allium paragraphs, the documentation contract and
the bridge rule are G's words where G's words are still true.

## Non-goals

- `docs/manifest.yml` and `docs/README.md` (frozen, §11). A page whose title, audience or
  topic slug would have to change is handed back, never renamed here.
- The two migrated explanation pages (T06), the project, tutorial and how-to pages (T07),
  the decision records (T09), `README.md`, `SECURITY.md`, `CHANGELOG.md` (T10).
- Any file outside `docs/`. A recipe, lint or pin a page finds wrong is handed back.
- Pushing, opening a pull request, tagging (§11).

## Files touched

The fourteen T08 rows of CONVENTIONS.md §6. Frontmatter is frozen at the manifest entry
(T00 step 9); the H1 equals the title; relative links go only to pages in
`docs/manifest.yml`, so a specification module, `SECURITY.md` or `Cargo.toml` is written
as a code span. Tier is §6's: a tier B page is the first to cut if the ticket runs long.

| Path | Tier | `audience` | `canonical_for` | From G |
| --- | --- | --- | --- | --- |
| `docs/explanation/architecture.md` | A | contributor, maintainer, operator, agent | system_architecture | 86 lines |
| `docs/explanation/layering.md` | B | contributor, maintainer, agent | dependency_boundaries | 75 lines |
| `docs/explanation/specifications.md` | A | contributor, maintainer, agent | specification_model | 96 lines |
| `docs/explanation/quality-philosophy.md` | A | contributor, maintainer, agent | quality_philosophy | 72 lines |
| `docs/explanation/security-model.md` | B | user, contributor, maintainer, operator, agent | security_model | 98 lines |
| `docs/reference/commands.md` | A | contributor, maintainer, operator, agent | command_reference | 93 lines |
| `docs/reference/configuration.md` | B | contributor, maintainer, operator, agent | configuration_reference | 134 lines |
| `docs/reference/testing.md` | A | contributor, maintainer, agent | testing_reference | 120 lines |
| `docs/reference/quality-gates.md` | A | contributor, maintainer, agent | quality_gate_reference | 177 lines |
| `docs/reference/documentation-contract.md` | A | contributor, maintainer, agent | documentation_contract | 109 lines |
| `docs/reference/agent-contract.md` | A | contributor, maintainer, agent | agent_contract | 99 lines |
| `docs/reference/api.md` | A | user, contributor, maintainer, agent | api_reference | new |
| `docs/operations/maintenance.md` | B | maintainer, operator, agent | maintenance_routine | 79 lines |
| `docs/operations/troubleshooting.md` | A | contributor, maintainer, operator, agent | troubleshooting | 138 lines |

`kind` is the directory name; `requires` is `[]`. Plus `tickets/T08-handbook-b.md`
(`status:` and hand-back notes).

## Steps

1. Create the worktree on `ticket/t08-handbook-b` from `main` (`tickets/README.md`). Run
   `just check-docs` once before editing and keep the output: it is green on the stubs,
   and every failure after this point is yours.
2. Confirm the frontmatter authority: `grep -n '<path>' docs/manifest.yml` for each row
   above must agree with the table. Hand back any difference.
3. Rules every page obeys; the docs validator, markdownlint, typos and lychee enforce them:
   - Frontmatter of exactly five keys, inline lists in the manifest's order; the first
     heading level one and equal to `title`; forty or more words; none of `TODO`, `TBD`,
     `FIXME`, `{{`, `{%`, `{#`.
   - Relative links only to `docs/manifest.yml` pages, exact case. G's links to the pages
     §6 drops are dead here and go: architecture 86, specifications 94, testing 78 and
     120, quality-gates 74, documentation-contract 75 and 96, configuration 133,
     maintenance 25, 31, 48 and 77, troubleshooting 124.
   - Decision links are renumbered to T00 step 9's files. G's 0001 at architecture 17
     becomes `../decisions/0001-engine-as-a-library.md` (same number, a different record,
     so the sentence changes too); G's 0002 at architecture 74 and layering 75 becomes
     `0003-effects-behind-traits.md`; G's 0003 at specifications 96 becomes
     `0002-specs-are-the-source-of-truth.md`; G's 0007 at quality-gates 36 becomes
     `0005-project-managed-allium-cli.md`. G's 0005, 0006 and 0008 (maintenance 25,
     quality-gates 140 and 69) have no counterpart.
   - No version number copied from a pin file. A page names `rust-toolchain.toml`,
     `pyproject.toml`, `pixi.lock`, `tools.txt`, `Cargo.lock`, the hook revs and the
     action SHAs as where a pin lives, never the number, so a bump never touches the
     handbook (§13, last risk).
   - No word that is true of the game only: "this game", Svelte, Vitest, Storybook,
     Chromatic, npm, Playwright, jsdom, ESLint, `platformSpecs`, "randomness port". Say
     "this library" or "the engine", and "the randomness boundary" (§9).
   - Every page ends with `## Related pages`, pruned to targets the rule above allows.
4. Rewrite each page as its subsection below says, overwriting the whole stub. "Verbatim"
   means byte for byte from G at `78d03cdf`.
5. Tier B (`layering.md`, `security-model.md`, `configuration.md`, `maintenance.md`) is
   cut whole or not at all: leave T00's stub, list the page under hand-back notes as
   carried forward with the reason, and leave its row above as the follow-up's scope.
6. Run Verification and fix until clean. `just check` must be green.
7. Commit on the ticket branch with `status: done` and the hand-back notes filled in.
   **Authorisation required:** pushing and opening the pull request. Stop and ask.

### `explanation/architecture.md`

G: opening 11-17; Build 19-34; Runtime shape 36-52; State 54-64; Side effects 66-74;
What is not here 76-79; Related 81-86. Nothing is verbatim; every section describes the
site. Keep the shape of 15-17 (the constraint is the architecture, then the decision
link) pointing at decisions 0001 and 0008. Sections, in order:

- **Opening.** A library with no process: no main, no I/O, no clock, nothing it can do
  until a consumer calls it. The game consumes it through WebAssembly later; a CLI and
  Python bindings consume it as sibling crates later still (§1 decision 1).
- **Workspace.** §3's tree in words: the root `Cargo.toml` with `[workspace.lints]` and
  the dependency table; `crates/pawdoku` with `src/lib.rs`, `src/random.rs` and `tests/`,
  one Rust module per specification module in `layering.md`'s direction;
  `crates/pawdoku-cli`, `crates/pawdoku-py` (pyo3 and maturin) and `crates/pawdoku-wasm`
  (wasm-bindgen) as siblings when S04 opens them. Every decision below keeps them cheap.
- **The core is `no_std`.** `#![no_std]` with `extern crate alloc`, so a clock, threads,
  the filesystem, the environment and `HashMap` are unnameable rather than forbidden by
  convention; `cargo check --target wasm32v1-none`, a target with no std, is the proof
  (`just wasm-check`). Limits are step budgets, never timeouts (§1 decision 5).
- **The randomness boundary.** The one port: a trait in `random.rs` for a stream of draws
  in `[0, 1)` begun from a seed, indexed from zero, the same for the same seed and
  `random_version`. The caller chooses the implementation; the library ships one
  generator named by `random_version` so that exact replay holds across callers; a test
  supplies draws through the fake (§9's wording). `rand` and `getrandom` are banned from
  the core by `deny.toml`, with `pawdoku-cli` the only permitted wrapper.
- **What crosses each boundary.** wasm: values and the seed, through wasm-bindgen.
  Python: the same, with exceptions built from the error enums' `Display` text, which is
  why that text is stable and every public type is `Send + Sync + 'static`, `Clone`,
  `Debug` and `#[non_exhaustive]` (§7 invariant 3). CLI: the one place entropy is sourced.
- **What is not here.** I/O, a clock, threads, `rand`, the network, persistence, a UI,
  and puzzle generation for now: out of scope, as `project/purpose-and-scope.md` records.
- Related: `layering.md`, `security-model.md`, `../project/repository-map.md`,
  `../reference/api.md`.

### `explanation/layering.md`

Tier B. G: table 11-18; rules 20-28; Why 30-39; Where a side effect goes 41-62;
Enforcement 64-69; Related 71-75. Nothing verbatim; the rule at 61-62 is kept, reworded
to the boundary.

- The table becomes the specification import graph, which is the module graph, read from
  the `use` lines at `78d03cdf`: `sudoku` imports nothing; `solver` and `technique`
  import `sudoku` (solver 61, technique 86); `reach`, `effort`, `lapse` and
  `human-solving` each import `sudoku` and `technique` (57-58, 57-58, 78-79, 72-73) and
  never each other; `board` (G `add73be7`, migrated by T12) imports `sudoku` alone
  (board 104) and nothing imports it; `random` sits beside them, used by whichever
  module draws. A Rust module may use what its specification module imports and nothing
  above it.
- Why it matters (30-39): every model of a player is testable without the others and
  without a generator, because the only thing reaching outside is the boundary.
- Enforcement (64-69): rustc allows a cycle between modules of one crate, so the
  direction is held by review and by the `rust-change` and `code-review` skills, as G says
  of its own; `unreachable_pub` and `unnameable_types` keep what is `pub` deliberate, and
  `mod_module_files` keeps one file per module.
- Related: `architecture.md`, `../reference/testing.md`,
  `../decisions/0003-effects-behind-traits.md`.

### `explanation/specifications.md`

G: opening 11-17; What a specification is for 19-29; What the modules are 31-72; Open
questions 74-83; What a specification is not 85-89; Related 91-96. Verbatim: 16-17,
from "`sudoku.allium` states" at 37 through 68 except the end of 39 (below; the link at
66 resolves here), 74-89. Since T12 (2026-09-25) the base is G at `add73be7`, where
What the modules are runs 31-79, the verbatim run from 37 extends through 75 and takes
in the `board.allium` paragraph at 40-46 (its "as `human-solving.allium` does" stays:
the page describes that contract in the paragraphs that follow), Open questions is
81-90, What a specification is not 92-96 and Related 98-102; every number below is the
pin's and shifts by seven from line 40 on.

- 11-14: this library's behaviour; the eight modules under `docs/specs/` that
  `docs/README.md` names are the source of truth; handbook, code and tests answer to them.
- 21-29, the "stated once" example: the side of the grid is stated once, in
  `sudoku.allium`'s `config` block; `pawdoku::SIDE` mirrors it (T00 step 4) and a doctest
  and a unit test hold the mirror, so a grid of the wrong size fails on the number rather
  than on a reviewer's eye. Same closing sentence as 28-29.
- 33 to "present." at 37: "Eight are the library's." and nothing about a root module.
  Then 39's "a module that draws the rules imports both" loses its referent, so it takes
  §9's wording for `sudoku.allium` 40-41: a consumer that draws the rules imports this
  module beside its own.
- 69-72: the game's root module, `pawdoku.allium`, stays in G. Allium has no
  cross-repository import, so the game restates the clauses it needs and holds them equal
  to this repository's text by test on G's side. Until S04 ships the modules inside the
  wasm package that text is shared truth across two repositories and can drift silently
  (§13); a change to a module here is a change the game has to see.
- After 83, one sentence: the open questions at `effort.allium` 312 and
  `human-solving.allium` 1280 were carried as-is (§9); the modules are where they live.
- Related: `../how-to/work-with-the-specs.md`, `human-solving.md`,
  `../decisions/0002-specs-are-the-source-of-truth.md`.

### `explanation/quality-philosophy.md`

G: opening 11-13; Checks are read-only 15-26; Fix the cause 28-39; Unreachable is not
untested 41-51; Tests inject 53-61; The specification is the arbiter 63-66; Related
68-72. §6 says "verbatim minus the browser example": that is 59-61, and it goes. Three
more ranges name the frontend and would be false here, so they are adapted too (an Open
point asks for §6's note to say so). Everything else verbatim.

- 34-39: the config file is the `[workspace.lints]` table and `clippy.toml`; a rule
  wrong for this project is lowered there once, with a comment; at a call site only
  `#[expect(lint, reason = "...")]`, never a blanket `allow`, so a suppression that stops
  suppressing fails (§7 invariant 6).
- 48-51: the floor is 90 lines over `crates/pawdoku/src/**`, the denominator is small
  while the crate is, so one untested branch in `random.rs` can breach it alone (§13);
  doctests are not counted, so an example is not evidence for the floor.
- 55-56: the fake is the boundary's; it walks the draws the test chose.

### `explanation/security-model.md`

Tier B. G: opening 11-13; What there is to protect 15-36; What is deliberately not
secure 38-42; What the build defends 44-82; Out of scope 84-88; Reporting 90-92; Related
94-98. Verbatim: 79-80 (ripsecrets), 90-92. Rewrite for a library:

- Opening: no process, no network, no data about anyone; what remains is the supply
  chain and the code handed to a consumer.
- What there is to protect: nothing collected or stored; no credential in any file (CI
  uses the run's own token; publishing, when it comes, uses trusted publishing, §11).
- What the build defends: supply chain (`Cargo.lock` is the pin, every gate passes
  `--locked`, `just lock-check` fails on drift; `just deny` checks licences, bans and
  sources offline on every run and `just audit` the advisories weekly in CI; `rand`,
  `getrandom` and `openssl-sys` are banned, `tickets/T02-rust-gate.md` step 6; hooks and
  actions are pinned to commit SHAs);
  memory safety (`#![forbid(unsafe_code)]` in every crate, which a member cannot lower);
  no network at runtime (a `no_std` core cannot name a socket); workflow permissions
  (`contents: read` everywhere; the release workflow S02 designs is the only one that
  will hold more and never shares `ci.yml`); credential leakage (79-80).
- What is deliberately not secure: nothing in the engine is a secret; a puzzle's
  solution is computable by anyone holding the crate.
- Out of scope: what a consumer does with the output; a step budget is the consumer's
  limit to set, since there are no timeouts.
- Reporting: 90-92. The route is decided with visibility in T01; name the file only.
- Related: `architecture.md`, `../how-to/maintain-dependencies.md`,
  `../reference/quality-gates.md`, `../decisions/0007-dependency-policy.md`.

### `reference/commands.md`

G: opening 11-13; Setup 15-24; Dependencies 26-32; Develop 34-40; Format and repair
42-47; Check 49-59; Documents and agents 61-74; Publish 76-80; Aggregate 82-87; Related
89-93. Verbatim: 11-13 (`just --list` prints the live set, which is what the `default`
recipe does, so `default` gets no row), 65-69, 71-74, 86-87,
89-93. One row per recipe in G's `| just <recipe> | Purpose |` shape, grouped as the
Justfile's banners group them: **Setup** (`initialize`, `check-toolchain`,
`install-toolchain`, `install-tools`, `install-allium`, `sync`, `lock`, `lock-upgrade`,
`lock-check`, `install-hooks`); **Develop** (`build`, `test`, `test-doc`); **Format**
(`format`, `fix`); **Check** (`lint`, `fmt-check`, `toml-check`, `clippy`, `features`,
`wasm-check`, `coverage`, `doc`, `deny`, `audit`, `deps-unused`); **Documents**
(`check-docs`, `check-agents`, `check-specs`, `analyse-specs`, `check-links-online`);
**Aggregate** (`check-clean`, `check`). Thirty-three rows. Each purpose is the recipe's
§4 comment made a sentence (`sync` installs `Cargo.lock` and `pixi.lock` exactly as
committed, `lock` relocks both, `lock-upgrade` runs `cargo update` and `pixi update`,
`lock-check` proves both, `install-tools` installs `tools.txt` through the
environment's cargo-binstall; decision 0011), and says which recipes reach the network
(`install-tools`, `install-allium`, `install-hooks`, `sync`, `lock`, `lock-upgrade`,
`audit`, `check-links-online`; `lock-check` and `lint` never, because `install-hooks`
prepares every hook environment and `lock-check` is offline) and that `build`, `test`, `audit` and `check-links-online` are outside
`check`. `fix` is the recipe that repairs, not the only one that writes (G 47, kept).

### `reference/configuration.md`

Tier B. G: opening 11-12; Build-time environment 14-42; Tooling environment 44-56;
Configuration files 58-91; Values the specifications decide 93-118; Version pins
120-128; Related 130-134. Nothing verbatim; the argument of 20-22 (a value baked into a
build is a constant and belongs in source) is kept, reworded. T02 finalises the tool
configs in parallel: describe what `tickets/T02-rust-gate.md` steps 3, 5 and 6 specify
(the lint table, `rustfmt.toml`, `clippy.toml`, `taplo.toml`, `.config/nextest.toml`,
`deny.toml`), and T11 reconciles wording against the merged files.

- Opening: a library has no runtime configuration; the one knob is Cargo features, and
  none is on by default (`default = []`, T00 step 4).
- Environment: the crate reads none. Tooling reads `RUSTDOCFLAGS` on the `doc` recipe
  only, `GITHUB_TOKEN` on CI's `install-tools` step only, and the `CARGO_*` and
  `RUST_BACKTRACE` values `ci.yml` exports (§10). No `.cargo/config.toml`, for §4's
  reason: `RUSTFLAGS` there changes fingerprints and thrashes `target/` against
  rust-analyzer.
- Configuration files, one row each: `rust-toolchain.toml` (an exact channel, minimal
  profile, four components, two targets, and why not `stable`: the clippy lint set would
  move every six weeks under `-D warnings`); `Cargo.toml` (`[workspace.lints]`: `warn` in
  the manifest and `-D warnings` on the `clippy` recipe so a local build never breaks
  mid-edit; the notable lints with their reasons, at least `unsafe_code = "forbid"`,
  `missing_docs`, `unreachable_pub`, `pedantic` at priority -1, the `unwrap_used` family,
  `std_instead_of_core`, the `exhaustive_*` pair, `allow_attributes`, and the allows T02
  makes, with why); `clippy.toml` (the `allow-*-in-tests` allowances and the disallowed
  `HashMap` and `HashSet`, whose iteration order is not deterministic); `rustfmt.toml`;
  `taplo.toml`; `deny.toml` by section (`graph`, `advisories` for `audit` only,
  `licenses`, `bans` with the three denies and their reasons, `sources`);
  `.config/nextest.toml`; `pyproject.toml` (the pixi manifest: every tool, Python, prek
  and the tooling package under `[tool.pixi.*]`, and B's own table; `pixi.lock` is the
  pin) and `tools.txt` (the one-line exception for what conda-forge lacks); the
  `coverage_floor` variable in the `Justfile`; the two prek configs;
  `.markdownlint-cli2.jsonc`; `lychee.toml`; `_typos.toml` (the only typos
  configuration); `.editorconfig`.
- Values the specifications decide: `pawdoku::SIDE` mirrors `sudoku.allium`'s
  `config.side`; a constant with no `config` entry to name is drift the other way.
- Version pins: where each lives, never the number (step 3).
- Related: `commands.md`, `quality-gates.md`, `../how-to/maintain-dependencies.md`.

### `reference/testing.md`

G: Framework 11-22; Layout 24-43; Conventions 45-65; Story tests 67-87; Coverage 89-104;
What the current suite proves 106-114; Related 116-120. Verbatim: 102-104.

- Framework: cargo-nextest runs the unit and integration tests and cannot run doctests,
  so `just test` runs it and then `cargo test --doc`; `.config/nextest.toml` holds the
  profiles.
- Layout: unit tests live in the module they test under `#[cfg(test)] mod tests`, with
  `#[cfg(test)] extern crate std;` at the crate root, a stated deviation from the games'
  "never colocated" rule recorded in decision 0009. Integration tests live in
  `crates/pawdoku/tests/`: `api_bounds.rs` (every public type is `Send + Sync + 'static`,
  `Clone` and `Debug`, held at compile time) and `random.rs` (the boundary's contract:
  the same seed gives the same stream, indexed from zero, named by `random_version`);
  `tickets/T02-rust-gate.md` steps 7 and 8 give both. Property tests use proptest; a
  regression file is committed beside the test that produced it (Open points).
- Conventions: derive a test from the clause it proves and name it after the clause; a
  doc example is a test; tests supply draws through the fake, never a generator;
  `clippy.toml` lifts the restriction lints inside `#[cfg(test)]` only (T02 step 5).
- Coverage: cargo-llvm-cov over nextest, a floor of 90 lines over
  `crates/pawdoku/src/**`, enforced by `just coverage`. Doctests are not measured because
  llvm-cov's doctest support is nightly-only, so `test-doc` runs them uncovered and the
  floor is a statement about unit and integration tests (§13). Then 102-104.
- What the suite proves: a table with one row each for the in-module tests, the
  doctests, `tests/api_bounds.rs` and `tests/random.rs`.
- Related: `../how-to/test-and-debug.md`, `quality-gates.md`,
  `../explanation/quality-philosophy.md`, `../decisions/0009-rust-quality-gate.md`.

### `reference/quality-gates.md`

G: opening 11-14; table 16-29; allium 31-47; the missing network check 49-51; the
workshop's network gate 53-74; What the hook gate contains 76-107; The mutating
counterpart 109-113; In continuous integration 115-146; On `main` 148-170; Related
172-177. Verbatim: 11-14, 31-35 (link renumbered at 36), 38-47,
49-51, 78-79, 96, 157-161, 172-177. Drop 53-74, 129-146, 163-170.

- The table: seventeen numbered rows in §4's order, each naming the recipe and what it
  proves in G's voice (`check-toolchain`: cargo is rustup's proxy and the pin is active;
  `lock-check`: `Cargo.toml` and `Cargo.lock` agree; `wasm-check`: both targets, the
  second being the `no_std` proof; `coverage`: every test passes at or above the floor;
  `doc`: rustdoc is warning-free under `--cfg docsrs`; and so on), then `check-clean` as
  row 18, "The run changed nothing", as G's row 12 has it. One sentence that `fmt-check`
  and `toml-check` also run inside `lint` and why (§4).
- The hook table (81-94) from §5: the local `fmt-check`, `toml-check` and `deps-unused`
  and why they route through `just`; the four contract and spec hooks, which run
  through `pixi run --frozen` because a git hook does not inherit the `Justfile`'s
  `PATH` and a bare `pixi run` could rewrite `pixi.lock` (§5); the seven remote hooks;
  the builtin set. One sentence that clippy is deliberately not a hook and why.
- 98-107, the actionlint gap: kept as a gap, ending: every `run:` here is one `just`
  line, so there is no embedded shell to miss today.
- 111-113: `cargo fmt`, `taplo fmt`, `cargo shear --fix`, the whitespace fixers and
  `markdownlint --fix`; `cargo clippy --fix` stays in the recipe body (§5).
- CI (117-127): the five gate jobs of §10 in a table with their recipes, the `check`
  aggregate that needs them and runs nothing (why protection names a job with no
  recipe), the `audit` job, and that the mapping is not one-to-one: `rust` runs `test`
  where the gate runs `test-doc`
  and `coverage`, `lint` runs in `documents`, and past the composite setup action nothing
  in CI runs a command the `Justfile` lacks.
- On `main` (150-155): the one required check is `check`, the job that needs `rust`,
  `coverage`, `wasm`, `deny` and `documents` and fails when any of them did not succeed,
  so a gate job is added by joining its `needs` and protection never changes; `audit` is
  never required (§10). Then 157-161. There is no deployment.
- Related: 172-177 plus `../decisions/0009-rust-quality-gate.md`.

### `reference/documentation-contract.md`

G: opening 11-16; The manifest 18-29; Frontmatter 31-48; The rules 50-71; Links that
leave the repository 73-81; Managed and seed pages 83-96; Outside the contract 98-103;
Related 105-109. Verbatim: 11-71, 80-81, 98-109. Drop 83-96, the managed
and seed paragraph §6 names. 75-78 links `project/platform.md`, which does not exist
here, so the section keeps its heading and says: no page points outward today; when one
does, the link is to a whole page, never to a heading, for G's reason at 77-78; then
80-81.

### `reference/agent-contract.md`

G: opening 11-14; The four surfaces 16-26; What `AGENTS.md` must contain 28-35; What a
skill must be 37-63; What a bridge must be 65-79; The inventory 81-94; Related 96-99.
Verbatim: 11-35 (the phrase list at 31-32 has six entries, with the honest `runes`
sentence §7 gives, until C02's follow-up makes it five), 50-63, 65-79 (the heading
is an anchor `troubleshooting.md` links to), 83-94, 96-99.

- 39: fourteen canonical skills in two groups: seven the house writes (`code-review`,
  `fix-quality`, `plan-change`, `project-check`, `review-docs`, `spec-change`,
  `rust-change`) and seven vendored byte for byte from `juxt/allium`, pinned by hash in
  `skills-lock.json` (`allium`, `distill`, `elicit`, `propagate`, `tend`, `weed`,
  `witness`). Twenty-eight bridges.
- 43-48, the example frontmatter: `rust-change`, with T00 step 13's description.
- After 94, one paragraph: the inventory forbids any other file under `.agents/`, which
  is why the vendored skills' reference material lives at `allium-skill-reference/` at
  the root, linguist-vendored and ignored by markdownlint, lychee and typos; decision
  0010 is the record. `just check-agents` prints
  `Validated AGENTS.md, 2 adapters, and 14 skills.`
- Related: 96-99 plus `../decisions/0010-skill-reference-material.md`.

### `reference/api.md`

New. Rustdoc is the API reference; this page says how to build it, where it lands, what
it promises and what a doc example is.

- **Building it.** `just doc` runs `cargo doc --workspace --no-deps --all-features` with
  `RUSTDOCFLAGS="-D warnings --cfg docsrs"`; the result opens at
  `target/doc/pawdoku/index.html`. It is gate 11, so a broken intra-doc link or a missing
  doc comment fails `just check`.
- **What is documented.** `missing_docs` warns in `[workspace.lints.rust]` and fails
  under `just doc` and `just clippy`, so every public item and the crate root carry docs.
- **Doc examples are tests.** Every public function with a contract worth stating
  carries an example; `just test-doc` (gate 9) compiles and runs them. They are not
  counted toward the coverage floor, so an example proves the contract and a unit test
  earns the coverage.
- **docs.rs.** `crates/pawdoku/Cargo.toml` carries `[package.metadata.docs.rs]` with all
  features, `--cfg docsrs` and two targets (T00 step 4). Prospective: the crate is
  `publish = false` until S02 decides the release process.
- **What the API promises.** §7 invariant 3 in prose, and that `tests/api_bounds.rs`
  holds the bounds.
- Related: `../explanation/architecture.md`, `testing.md`,
  `../project/purpose-and-scope.md`.

### `operations/maintenance.md`

Tier B. G: opening 11-13; Routine 15-32; Deploying 34-49; Rolling back 51-60; Secrets
62-73; Related 75-79. Verbatim: 27-28. Drop 34-60.

- Opening: no service, no deployment; what follows is upkeep of pins.
- Routine. **Weekly**: read the `audit` workflow's run; it runs `just audit` on a
  schedule and is never a required check (replacing 17-18, which says nothing is
  scheduled). **Monthly**: `just lock-upgrade` within the manifests' ranges, `just check`,
  commit `Cargo.lock`; move a pin in `pyproject.toml`, `pixi update <name>`, commit
  `pixi.lock`; a `tools.txt` line still re-runs `just install-tools`; move a hook rev or
  an action SHA by hand with its version comment; 27-28. **Per toolchain
  release**: bump `rust-toolchain.toml` and `rust-version` together and re-run
  `just clippy`, because the lint set moves (§13). **Per allium release**: the pin and
  checksums live in B's package; a
  moved pin re-verifies every module and invalidates every waiver (§13). Nothing updates
  any of these on its own until S01 decides.
- Secrets: none stored, nothing to rotate; CI uses the run's own token; crates.io, when
  it comes, uses trusted publishing.
- Related: `../how-to/maintain-dependencies.md`, `troubleshooting.md`,
  `../explanation/security-model.md`.

### `operations/troubleshooting.md`

G: 11; check-clean 13-17; docs validation 19-31; agent validation 33-46; coverage 48-56;
site-only sections 58-132; Related 134-138. Verbatim: 11, 13-17 (plus one sentence
naming `target/`, `.tools/`, `.pixi/` and a rewritten `Cargo.lock` or `pixi.lock` as the
usual paths), 19-46,
48-52. Drop 54-56, 58-132. New sections, symptoms as headings, in the voice of 13-17:

- `cargo resolves to <path>, not rustup's proxy` and `rustup is not installed`: the
  `check-toolchain` recipe's own messages (§4). A pixi, Homebrew or distribution cargo
  ignores `rust-toolchain.toml`; put `~/.cargo/bin` ahead on `PATH` or
  `pixi global uninstall rust`, until `command -v cargo` prints the proxy (§2).
- `just check-specs` cannot find `allium`, or a recipe cannot find `prek` or a `bg-*`
  script: the pixi environment and `.tools/bin` are per-worktree installs;
  `pixi install --frozen`, `just install-tools` or `just install-allium` restores the
  missing one, and `just initialize` does all three on a fresh worktree. A renamed or
  moved worktree needs `pixi install --frozen` again, because the `bg-*` shebangs name
  the worktree's absolute Python (§13).
- `just lock-check` or `pixi install --locked` fails on `pixi.lock`: the lockfile is
  stale against `pyproject.toml`; run `just lock`, read the diff and commit it. A first
  run never rewrites it silently.
- `just lint` clones or downloads: prek's cache was cleared, or this is a secondary
  worktree whose primary checkout never ran `just initialize`. Run `just install-hooks`,
  which prepares every hook environment; `just initialize` has the network,
  `just check` must not (§13).
- `cargo` reports that `Cargo.lock` needs to be updated: every gate passes `--locked`, so
  an edited `Cargo.toml` fails until `just lock` rewrites the lockfile and it is
  committed. A dependency added by any lane but T02 is handed back (§11).
- Coverage below the floor on a small crate: one untested branch in `random.rs` is enough
  while the denominator is tiny; test it or delete it, never lower the number (48-52).
- The `wasm32v1-none` target is missing: `just install-toolchain` installs what
  `rust-toolchain.toml` names; a cargo that is not rustup's cannot.
- `just install-tools` refuses to build from source (cargo-hack, the one `tools.txt`
  line): `--disable-strategies compile` is deliberate; choose a pin with a release
  binary for this host (§2).
- prek behaves as if a pin had not moved: clear its cache and re-run (Open points).
- Related: `../how-to/test-and-debug.md`, `../reference/quality-gates.md`,
  `maintenance.md`.

## Acceptance criteria

- All fourteen paths in Files touched are rewritten, or a tier B page is left as T00's
  stub and listed under hand-back notes as carried forward; nothing outside the table
  changed except this ticket's `status:`.
- `just check-docs` and `just check` are green.
- Every page's frontmatter equals its manifest entry, every H1 equals the title, every
  relative link targets a page in `docs/manifest.yml`.
- `reference/commands.md` has one row for every recipe in CONVENTIONS.md §4 except
  `default` and no other row: thirty-three rows.
- `reference/quality-gates.md` lists the seventeen gates in §4's order as rows 1 to 17
  and `check-clean` as row 18.
- No page carries a version number of the form `x.y.z`, and none carries a game-only
  word from step 3's list.
- The verbatim ranges named above are byte-identical to G's, allowing only for the
  renumbered decision links.

## Verification

```sh
just check-docs
ls docs/explanation docs/reference docs/operations | grep -c '\.md$'
diff <(just --summary | tr ' ' '\n' | grep -v '^default$' | sort) \
     <(grep -oE '`just [a-z][a-z-]*`' docs/reference/commands.md | sed 's/`just //; s/`//' | sort -u)
grep -cE '^\| [0-9]+ \|' docs/reference/quality-gates.md
grep -rnE '(^|[^0-9.])[0-9]+\.[0-9]+\.[0-9]+([^0-9.]|$)' docs/explanation docs/reference docs/operations \
  || echo "no versions"
grep -rniE 'this game|svelte|vitest|storybook|chromatic|npm|playwright|jsdom|eslint|/Users/|platformSpecs|randomness port' \
  docs/explanation docs/reference docs/operations \
  --exclude=solving-sudoku.md --exclude=human-solving.md || echo "no game words"
mkdir -p ai_tmp
while read -r p a b; do
  git -C /Users/scutting/projects/pawdoku show "78d03cdf:docs/$p" | sed -n "${a},${b}p" > ai_tmp/g.txt
  n=$(wc -l < ai_tmp/g.txt)
  s=$(grep -nF -- "$(head -n 1 ai_tmp/g.txt)" "docs/$p" | head -n 1 | cut -d: -f1)
  [ -n "$s" ] && sed -n "${s},$((s + n - 1))p" "docs/$p" | cmp -s - ai_tmp/g.txt && echo "$p $a-$b present"
done <<'EOF'
reference/quality-gates.md 38 47
reference/documentation-contract.md 11 71
reference/agent-contract.md 65 79
operations/troubleshooting.md 19 46
EOF
just check
```

Expected: `check-docs` ends with the validator's line naming every page valid; the count
is 16 (fourteen pages here plus the two T06 pages); the `diff` prints nothing (`--list`
at commands 11 is excluded by the pattern); the gate row count is 18; the two greps print
their `no ...` line; each verbatim range reports `present` (the loop anchors on the
range's first line and compares positionally, so a range that contains a renumbered
link is checked in two halves around it); `just check` exits 0. Quote each in the
hand-back notes.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

CONVENTIONS.md §12 assigns no claim to T08. Points raised while writing this ticket:

- Whether `reference/api.md` should embed a generated list of public items. No: rustdoc
  is the reference and a copied list rots on the first change; the page says where the
  built reference lands and what it promises.
- Whether `operations/troubleshooting.md` should cover Windows. No: CI is Ubuntu only
  until `crates/pawdoku-cli` exists (§10); a Windows section is written the day the
  `rust` job gains a Windows runner.
- §6 says `quality-philosophy.md` is "verbatim minus the browser example". Three further
  ranges (34-39, 48-51, 55-56) name the frontend and this ticket adapts them. Confirm, or
  correct §6's note to "verbatim minus the frontend ranges" in a `main` pull request.
- Whether proptest regression files are committed. T02 decides when it adds the
  dev-dependency; the testing page says "committed", and one sentence changes in a
  follow-up if T02 gitignores them.
- The exact prek subcommand that clears its hook cache. Check `prek --help` in the
  worktree and record it in the hand-back notes.
