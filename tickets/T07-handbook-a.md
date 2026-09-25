---
id: T07
title: "Handbook A: project, tutorial and how-to pages"
status: open
depends_on: [T00]
parallel_with: [T01, T02, T03, T04, T05, T06, T08, T09]
branch: ticket/t07-handbook-a
estimated_size: L
---

# T07: Handbook A: project, tutorial and how-to pages

## Context

T00 has merged to `main`. For the handbook that means `docs/manifest.yml` and
`docs/README.md` are final and frozen (CONVENTIONS.md §11), and every registered page
exists as a stub: frontmatter equal to its manifest entry, an H1 equal to the title, forty
or more words saying what the page will contain, and no links (T00 step 10). This ticket
replaces the stubs for the eight pages CONVENTIONS.md §6 assigns to T07: the three
project pages, the tutorial and the four how-to pages. T08 replaces the explanation,
reference and operations stubs, T06 finalises the two migrated explanation pages, and
T09 writes the decisions. All of them run in parallel with the other lanes and wait for
nothing but T00; T11 re-reads the whole handbook against the merged repository.

Every page here is a rewrite of the G page at the same path (CONVENTIONS.md §6, tier A
or B). G is `/Users/scutting/projects/pawdoku` at `78d03cdf`; every line number below
was verified against that commit with `git -C /Users/scutting/projects/pawdoku show
78d03cdf:docs/<path> | cat -n`, which is also how to read a page if the clone has moved.
Never modify G.

Read, in this order: `tickets/README.md`; CONVENTIONS.md §0, §2 (the toolchain facts
develop-locally states), §3 (the tree repository-map draws), §4 (every recipe a page may
name), §5 (the hook pins), §6 (the page table and the dropped pages), §8 (the decision
numbers), §9 (the boundary wording), §11 and §13; `tickets/T00-foundation.md` steps 9
and 10 (the manifest rows, which are the authority for every page's frontmatter, and the
stub rules); D01's hand-back notes (the `pyproject.toml` content, and the reason
`.python-version` is kept, under "Handed back", T03);
`G docs/reference/documentation-contract.md` (strict-JSON manifest 20-29; five
order-sensitive frontmatter keys 33-48; the rules at 61-68: H1
equals title, forty words, no placeholder prose, exact-case links, reachable from the
map); then the eight G pages and the module Vocabulary sections named in the briefs.
`/Users/scutting/projects/biscuit_games_template/tickets/T07-handbook-a.md` is the shape
reference for the hand-back notes; nothing in it is content here.

## Goal

Eight pages under `docs/` in final form, each a rewrite of G's page for a Rust library
rather than a browser game, such that `just check-docs` and `just lint` are green, every
link resolves to a page in this repository's manifest, every `just` recipe named exists
in CONVENTIONS.md §4, and a contributor who has never seen G can go from a fresh clone to
a green gate using nothing but these pages. `just check` is green in the worktree.

## Non-goals

- `docs/manifest.yml` and `docs/README.md` (frozen at T00). A retitle, a rename, a new
  page or a missing map link is handed back as a T00 follow-up on `main`.
- Any page owned by T08, T06 or T09, including the decision records this ticket links
  to. Link to the paths T00's manifest declares; never touch the files.
- The hook configs, `Justfile`, `tools.txt`, `rust-toolchain.toml` (frozen), the
  dotfiles (T03) and the root documents (T10). A page describes them; it does not edit
  them. The tutorial names one item in `crates/pawdoku/src/lib.rs`; it does not design
  the crate (T02).

## Files touched

Exactly the eight rows CONVENTIONS.md §6 assigns to T07, plus this ticket's `status:`
line. Every page keeps the frontmatter T00 wrote, byte for byte.

| Path | Tier | G source (lines at `78d03cdf`) | Change |
| --- | --- | --- | --- |
| `docs/project/purpose-and-scope.md` | A | `docs/project/purpose-and-scope.md` (40) | Rewritten: the engine, its non-goals, its consumers |
| `docs/project/repository-map.md` | A | `docs/project/repository-map.md` (60) | The §3 tree in G's presentation; new responsibility table |
| `docs/project/terminology.md` | B | `docs/project/terminology.md` (97) | "The repository" kept and edited; "The game" becomes "The engine" |
| `docs/tutorials/first-change.md` | A | `docs/tutorials/first-change.md` (91) | G's seven steps, around a clause, a test and `just check` |
| `docs/how-to/develop-locally.md` | A | `docs/how-to/develop-locally.md` (112) | rustup, the pixi hazard, cargo-binstall, the offline first-run caveats |
| `docs/how-to/test-and-debug.md` | B | `docs/how-to/test-and-debug.md` (75) | nextest filters, doctests, llvm-cov HTML, backtraces |
| `docs/how-to/work-with-the-specs.md` | A | `docs/how-to/work-with-the-specs.md` (141) | Seven-module table; waiver terms verbatim; `just test` |
| `docs/how-to/maintain-dependencies.md` | A | `docs/how-to/maintain-dependencies.md` (184) | Caret ranges and `Cargo.lock`; `tools.txt`; hook, action and toolchain pins |

## Steps

### 1. Rules every page satisfies

1. **Frontmatter** is the stub's, unchanged: `title` and `kind` quoted, the three lists
   inline, the five values equal to the manifest row (T00 step 9). The H1 equals `title`.
2. **Links** are relative Markdown links, exact case, to a page in this repository's
   manifest. G links to five pages that do not exist here (CONVENTIONS.md §6:
   `project/platform.md`, `explanation/accessibility.md`,
   `how-to/work-in-the-component-workshop.md`, `how-to/deploy-to-github-pages.md`,
   `how-to/update-from-template.md`); every such link is rewritten or dropped, and each
   brief below names the lines. G's decision 0007 is this repository's
   `docs/decisions/0005-project-managed-allium-cli.md` (CONVENTIONS.md §8); every
   `0007-project-managed-allium-cli.md` link moves there with link text `decision 0005`.
   The seven modules are valid link targets (`../specs/sudoku.allium`), as in G's map.
3. **Recipes.** A page names a `just` recipe only if CONVENTIONS.md §4 defines it. G's
   `just dev`, `just storybook`, `just frontend-*`, `just preview` and
   `just storybook-browsers` have no counterpart; each brief says what replaces them.
4. **Versions.** A page states no version that `rust-toolchain.toml`, `tools.txt`,
   `Cargo.toml`, `pyproject.toml` or a hook `rev` already states; it names the file, or
   links `../reference/configuration.md`, instead. Two exceptions, both places where G's
   page states a figure too: the one-time cargo-binstall command in develop-locally
   (G lines 15-16 state Node 26 and npm 11; the pin lives in `scripts/install_tools.sh`, not
   in `tools.txt`), and allium `3.6.1` in the waiver terms (G work-with-the-specs line
   108 and 128; maintain-dependencies line 144).
5. **Voice and shape** as G's pages: prose wrapped by hand near 90 columns, `console`
   fences for commands, `text` fences for file content, one `## Related pages` list at
   the end, "this repository" or "the engine" rather than "this game", and never a
   `/Users/` path. Paths are code spans.
6. **The Python toolchain stays** (decision 0004): `pyproject.toml` pins prek and B's
   `bg-*` console scripts, and every sentence below is written for that. Nothing here
   describes a Python-free form; the reader has one repository.

### 2. Worktree and the baseline

Create the worktree on `ticket/t07-handbook-a` from `main` (README.md "How to pick up a
ticket"), run `just initialize` if `.tools/bin` is empty, then `just check-docs` and
`just lint` on the untouched stubs and keep the output: it is the baseline every later
run is compared with. Both must be green before any page is written.

### 3. The project pages

**`docs/project/purpose-and-scope.md`** (G 40 lines: intro 11-15, "What it does" 17-22,
"What it deliberately does not do" 24-29, "Who it is for" 31-34, Related 36-40). Keep
G's four headings and its shape; every paragraph is new. State:

- Intro: `pawdoku` is the classic-sudoku engine behind the Biscuit Games game Pawdoku,
  a Rust library crate at `crates/pawdoku`, `no_std` with `alloc`, Apache-2.0, in a
  Cargo workspace that will hold its bindings as sibling crates. What this repository
  decides is the rules and the reasoning; what a player sees is a consumer's.
- What it does: the rules of classic Sudoku (`sudoku.allium`); the search that says
  whether givens are well-posed (`solver.allium`); the catalogue of 29 named techniques
  (`technique.allium`, G work-with-the-specs line 22); four models of a player, three
  without chance and one with (`reach`, `effort`, `lapse`, `human-solving`); and what
  those models are for, a difficulty rating and a hint pitched at a player. Behaviour is
  stated first in `docs/specs/`, one rule at a time, and built second; link
  `../explanation/specifications.md` as G line 21 does.
- What it deliberately does not do, one bold lead per bullet: **No surface.** Nothing
  here draws, and no module says how a puzzle looks (`sudoku.allium` line 35-37).
  **No persistence.** Nothing is remembered between calls; a consumer keeps what it
  wants kept. **No puzzle generation yet.** How a setter finds givens is excluded by
  `sudoku.allium` line 30-32; generation is a later module. **No variants.** Other
  sizes, irregular boxes, diagonals, cages (line 28-29). **No clock, threads, filesystem
  or network.** The core is `no_std`, so they are unnameable rather than avoided;
  randomness is the one effect, reached through the randomness boundary (CONVENTIONS.md
  §9); link `../explanation/architecture.md` and `../explanation/security-model.md`.
- Who it is for: consumers, not players. The game, through a WebAssembly crate; a
  command line; Python, through a bindings crate. Today only the library exists; the
  three are sibling crates to come (decision 0001, `../decisions/0001-engine-as-a-library.md`).
- Related pages: G's three (`repository-map.md`, `terminology.md`,
  `../explanation/architecture.md`) plus `../explanation/specifications.md`.

Target 300-450 words (G has 236).

**`docs/project/repository-map.md`** (G 60 lines: intro 11-12, tree 14-37, table 39-52,
layering sentence 54-55, Related 57-60). Keep the shape exactly. The intro becomes two
sentences: a Cargo workspace with one member today, and where the bindings crates will
sit beside it. The tree is CONVENTIONS.md §3 drawn in G's style (lines 14-37: `├──`
branches, one annotation per line, aligned), without the lane annotations, in this
order: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `tools.txt`, the five tool
configs (`rustfmt.toml`, `clippy.toml`, `taplo.toml`, `deny.toml`,
`.config/nextest.toml`), `crates/pawdoku/` with `src/lib.rs`, `src/random.rs` and
`tests/`, `docs/` with `specs/`, `scripts/`, `.agents/skills/`, `allium-skill-reference/`,
`.github/`, `AGENTS.md`, `Justfile`, then `pyproject.toml`, `uv.lock` and
`.python-version`. Gitignored directories (`target/`, `.tools/`, `ai_tmp/`) close the tree with
"never committed" in the annotation, as §3's last line says. The responsibility table
(G 41-52) has these rows: `crates/pawdoku/src/` (pure behaviour; the one effect behind
the trait `random.rs` defines; every public item documented and its example a test),
`crates/pawdoku/tests/` (integration tests: the API bounds and the boundary; unit tests
sit in-module, a stated deviation recorded in decision 0009), `docs/specs/` (G row 51
verbatim), `scripts/` (the first-run script and the tool installer; the validators, the
allium installer and runner, the gate runner and the ripsecrets wrapper are console
scripts of `biscuit-games-tooling` pinned in `pyproject.toml`, as G row 50 says),
`.agents/skills/` (canonical agent
procedures; the two bridge trees point at them), `allium-skill-reference/` (vendored
reference material for the Allium skills; ignored by every linter, decision 0010),
`.tools/bin` (the pinned binaries `tools.txt` names; on `PATH` inside every recipe and
nowhere else), `target/` (cargo's; `just coverage` writes `target/llvm-cov/`). Keep G
54-55 (the layering link). Related pages: G's two.

Target 400-550 words (G has 471).

**`docs/project/terminology.md`** (tier B; G 97 lines: intro 11-12, "The game" 14-77
with its table 21-77, "The repository" 79-91 with its table 81-91, Related 93-97). Keep
11-12 verbatim. Rename "The game" to "The engine" and rewrite its lead (16-19) to say the
rows are the modules' own Vocabulary sections, which pick one word where the literature
has several: `sudoku.allium` 43-46, `solver.allium` 55-59, `technique.allium` 65-84,
`reach.allium` 51-55, `effort.allium` 50-55, `lapse.allium` 66-76, `human-solving.allium`
52-70. Keep G's table rows 25-77 as they are, because each is drawn from those sections
and none names the game: they cover box (never block, region or subgrid), given (never
clue), well-posed (never proper), unit, peer, conflict, candidate, search, branch,
propagate, guess (never trial), contradiction, verdict, solution, technique, ladder,
deduction, placement, strike (never eliminate), mark, subset, cross-hatch, pointing and
claiming (never box-line reduction), bivalue, conjugate pair, link, literal, path, base,
cover, proof, witness, uniqueness promise, profile, projection, load, capacity, extent,
span, run, step, see, stall, price, escalation, check, repair, attempt, microstep, fact,
belief, chunk, note, entry, sheet, coverage, observer, assessment, familiar recognition,
elementary derivation, fatigue, frustration. Three edits: row 23 (Player) becomes "The
person the four models describe. A profile is a parameterised one; nothing here knows
who is at a device."; row 24 (Play surface) is dropped; row 27 (Cell) ends "a
consumer's surface is what draws one" in place of "the platform's play cell is what
draws one". Compare each kept row against the module's Vocabulary section once; a row
that contradicts the module is corrected to the module and named in the hand-back.

"The repository" (81-91): keep Specification, Surface, Guarantee, Gate and Recipe. Row 86
(Port) becomes "Randomness boundary | The trait `crates/pawdoku/src/random.rs` defines:
a stream of draws in `[0, 1)` begun from a seed, the same for the same seed and
`random_version`. The only effect the engine has; a consumer supplies the implementation
and a test supplies draws through the fake." Rows 87 (Platform) and 88 (Exact) are
dropped: the platform page does not exist here and the word `Exact` is the platform's.
Row 89 (Fake) becomes "The in-memory implementation of the randomness boundary, used by
tests. Not a mock: it behaves, rather than recording calls." Add two rows: "Consumer |
Whatever calls the engine: the game through WebAssembly, a command line, Python. What a
player is shown is a consumer's decision, never a rule here." and "Workspace | The Cargo
workspace at the root. One member today, `crates/pawdoku`; the bindings crates join it
as siblings." Related pages: G's three, unchanged (all exist here).

Target 1500-1700 words (G has 1736; the table carries most of them).

### 4. The tutorial

**`docs/tutorials/first-change.md`** (G 91 lines: intro 11-15; steps 17-27, 29-37,
39-45, 47-57, 59-65, 67-75, 77-79; "What you just touched" 81-85; Related 87-91). Keep
G's seven-step shape and every heading that still applies. The change is small on
purpose and passes through every layer: a clause, a test, an item, the gate.

- Intro (11-15): about half an hour from a fresh clone to a green gate; the first run
  downloads a toolchain, the pinned tools, the Allium checker and the hook environments,
  so it needs the network and rustup first (link `../how-to/develop-locally.md`).
- Step 1 (17-27): `just initialize` then `just check`. What `initialize` does, in T00
  step 7's order: refuses a cargo that is not rustup's, installs the pinned toolchain,
  the tools into `.tools/bin` and the Allium checker, syncs, normalises formatting, and
  installs the pre-commit hook from the primary worktree only. Keep G 26 ("It never
  stages, commits, tags or pushes.").
- Step 2, "See the crate" for "See the app" (29-37): `just doc`, then open
  `target/doc/pawdoku/index.html`. The one documented item is the constant `SIDE` in
  `crates/pawdoku/src/lib.rs`, restating `sudoku.allium`'s `config.side`; its example
  is a test, which `just test-doc` runs.
- Step 3 (39-45): open `docs/specs/sudoku.allium` and read the `config` block (module
  lines 133-143): `box_side` is 3, named so no rule carries a bare number, and `side`
  follows from it (142); then `Puzzle.is_full` (86-88). The crate restates `side`; the
  tutorial's change restates the box.
- Step 4 (47-57): the test first. In the `tests` module at the foot of `lib.rs`, add a
  test asserting `BOX_SIDE * BOX_SIDE` equals `SIDE`, read from the `config` comment;
  run `just test` and watch it fail to compile, which is the point. Keep G 57.
- Step 5 (59-65): add `pub const BOX_SIDE: u8 = 3;` above `SIDE` with a doc comment and
  an example (a doc example is a test), run `just test` again, and land the item, its
  example and the assertion in one commit. The rule for every later change: a clause,
  its test and the code land together, and the test is named for the clause.
- Step 6 (67-75): G's paragraph kept, `../operations/troubleshooting.md` linked as G does.
- Step 7 (77-79): kept, plus one sentence: the hook runs the read-only gate but not
  clippy, which is why `just check` came first.
- "What you just touched" (81-85): a clause, a test, an item and the gate; the first real
  change starts by adding a rule to a module rather than reading one. Related: G's three.

If T02 has merged and `SIDE` has moved or `BOX_SIDE` exists, pick another figure the
crate does not yet restate and record the substitution in the hand-back; if T02 has not
merged, write against T00's `lib.rs` and say so, so that T11 re-reads the page.

Target 500-650 words (G has 518).

### 5. The how-to pages

**`docs/how-to/develop-locally.md`** (G 112 lines: Prerequisites 11-23, First run 25-52,
Every day 54-87, Before handing work back 89-97, Keeping the workspace current 99-105,
Related 107-112). Keep the six headings. Content:

- Prerequisites table (13-19): `rustup` (installs the exact toolchain
  `rust-toolchain.toml` pins, and the only cargo the gate accepts); `cargo-binstall`
  (installs the pinned binaries `tools.txt` names, without compiling them); `just` (the
  task runner, and the only supported interface to the checks); `uv` (runs the pinned
  Python tooling the hook gate needs; `.python-version` pins the interpreter series so
  two machines and CI resolve the same one, as D01 settled); `gh` (only for the pin-bump
  commands in maintain-dependencies). Replace 21-23 with: `rust-toolchain.toml` names
  the exact toolchain, its components and its two wasm targets, and rustup installs them
  the first time cargo runs here; `tools.txt` names every other binary and its version;
  link `../reference/configuration.md` rather than restating a number.
- First run (25-52), rewritten around three facts. One: rustup, installed once per
  machine; no default toolchain is required, because prek provisions its own rustup for
  its one `language: rust` hook (CONVENTIONS.md §2 "Maintainer prerequisites"; T03's
  §12 outcome is the record, and a contrary outcome is a hand-back to this page); the
  one-time command is upstream's, with the flags §2 implies:

  ```console
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --profile minimal
  ```

  Say in one sentence that this is the only curl-pipe in the handbook, and that any
  route ending with `rustup` on `PATH` and `command -v cargo` printing
  `~/.cargo/bin/cargo` is equivalent. Two: the pixi hazard, in the tone of G's `~/.npmrc`
  paragraph (37-44): a cargo from `pixi global`, Homebrew's `rust` formula or a
  distribution ignores `rust-toolchain.toml` and builds with whatever it is, so
  `just check-toolchain` refuses it before anything else runs; the remedy is
  `~/.cargo/bin` ahead of `~/.pixi/bin` on `PATH`, or `pixi global uninstall rust`.
  Quote the recipe's two error lines (`Justfile`, `check-toolchain`) so a reader can
  search for them. Three: cargo-binstall, once per machine, which `scripts/install_tools.sh`
  does when it is absent and which is the one tool compiled from source:

  ```console
  cargo install cargo-binstall@1.23.0 --locked
  ```

  Then `just initialize` (46-52 rewritten): what it does, in order, and that it needs the
  network for the toolchain, the tools, the Allium checker, `cargo fetch` and the Python
  environment. Then the offline caveat: the first `just lint` clones and
  builds the hook environments and needs the network too; `just check` runs `lint` as
  its third gate and adds no network need of its own, so run `just lint` once after
  `just initialize` and the gate is offline from then on. Name `.tools/bin`: gitignored,
  prepended to `PATH` by the `Justfile` for every recipe and not on the shell's `PATH`,
  so a bare `cargo nextest` from a shell needs `PATH="$PWD/.tools/bin:$PATH"` or a recipe.
- Every day (54-87): `just build`, `just test`, `just doc` in place of `dev`,
  `storybook`, `preview`; the `BASE_PATH` paragraphs (73-87) go. One sentence each.
- Before handing work back (89-97): G's block verbatim, with 96 corrected as the
  template's T07 hand-back did: `fix` is the one of the pair that modifies files;
  `lock` and `format` write too.
- Keeping the workspace current (99-105): `just sync` after pulling, and
  `just install-tools` after `tools.txt` moved, which skips a version already there.
- Related pages: G's four minus the workshop page; add `../reference/configuration.md`.

Target 550-750 words (G has 558). Ubuntu and macOS only; Windows is an open point.

**`docs/how-to/test-and-debug.md`** (tier B; G 75 lines: Run the suites 11-28, Narrow
down 30-40, Debug a coverage failure 42-52, Debug a browser problem 54-68, Related
70-75). Keep the first three headings; the fourth becomes "Debug a `no_std` or `--locked`
failure".

- Run the suites (13-18 becomes): `just test` (nextest, then doctests), `just test-doc`
  (doctests alone: nextest cannot run them), `just coverage` (nextest under
  instrumentation, with the floor enforced), `just clippy`. To iterate on one test (20-25
  becomes), call nextest directly with its filter language:

  ```console
  cargo nextest run -p pawdoku -E 'test(name)'
  RUST_BACKTRACE=1 cargo nextest run -p pawdoku -E 'test(name)'
  ```

  with the `.tools/bin` sentence from develop-locally. Keep 27-28 (the link to
  `../reference/testing.md`).
- Narrow down a failing test (32-40): item 1 becomes one test by filter; item 2 becomes
  "read the assertion's left and right and, for a panic, the backtrace `RUST_BACKTRACE=1`
  prints; CI sets it for every job"; items 3 and 4 verbatim. Add item 5: a proptest
  failure writes a regression file beside the test, under a `proptest-regressions/`
  directory; commit it, because it is the one input that failed and the suite replays it
  first from then on.
- Debug a coverage failure (44-52): `just coverage` prints the summary; for the lines,
  render the HTML report, which lands at `target/llvm-cov/html/index.html`:

  ```console
  cargo llvm-cov report --html
  ```

  (after a `just coverage` run, whose profile data it reuses; or
  `cargo llvm-cov nextest --workspace --all-features --locked --html` from cold). G's
  two bullets kept, with the Svelte example replaced by a `match` arm no input reaches.
  Line 52 becomes "Never lower `coverage_floor` in the `Justfile`." Doctests are not
  measured (CONVENTIONS.md §13), so a doc example does not count toward the floor.
- The fourth section: `just wasm-check` failing names the `std` item that crept in; the
  fix is the trait boundary, never a `cfg`. A `--locked` failure means `Cargo.lock`
  disagrees with a manifest; link `maintain-dependencies.md` and never run `cargo update`
  from inside a fix.
- Related pages: G's four minus the workshop page.

Target 400-550 words (G has 370).

**`docs/how-to/work-with-the-specs.md`** (G 141 lines: intro 11-13, module table 15-29,
Change behaviour 31-45, Handle an open question 47-56, Tooling 58-75, Diagnostics and
waivers 77-136, Related 138-141). Keep every heading.

- Intro (11-13): the seven Allium modules under `docs/specs/` decide what the engine
  does; the game's root module stays in the game and restates what it needs, held equal
  by test on the game's side (CONVENTIONS.md §1 fact 6). Keep the link to
  `../explanation/specifications.md`.
- Module table (17-26): drop row 19 (the root module). Keep rows 20-26 with two edits:
  row 20 ends "It imports nothing; a module that draws the rules imports it." (the words
  "and the root" go); row 26's "drawn through the randomness port" becomes "drawn
  through the randomness boundary". Keep 28-29.
- Change behaviour (33-45): step 2 (35-37) loses the `tests/platformSpecs.test.ts`
  clause and gains "the game restates clauses of these modules and holds them equal by
  test on its side, so a reworded clause here is a change the game must take"; step 6
  (45) becomes "Run `just test`, then `just check`." Steps 1, 3, 4 and 5 verbatim.
- Handle an open question (49-56): 50-51 ("A fresh game has none; ...") becomes "These
  modules carry a few (`effort.allium` line 312, `human-solving.allium` line 1280 at
  `78d03cdf`; T06 may have moved them); treat a low count as the settled state, not as a
  reason to stop adding them." Bullets verbatim.
- Tooling (60-75): 63-64's "either lockfile" becomes "a lockfile" and the decision link
  becomes `../decisions/0005-project-managed-allium-cli.md`; the `console` block and
  71-75 verbatim.
- Diagnostics and waivers: G lines 77-136 verbatim (CONVENTIONS.md §6 writes 78-139; the
  heading is line 77 and the section ends at 136, with Related pages at 138), with
  exactly one edit: the decision link at 81 moves to 0005. Everything else, including "No
  waiver is currently in the modules." (109-110) and the 3.6.1 sentences, stays word for
  word; the `Justfile`'s `check-specs` comment points here for the terms.
- Related pages: `../explanation/specifications.md` kept; line 141 (Accessibility, a
  dropped page) becomes `../reference/quality-gates.md`.

Target 1450-1650 words (G has 1558).

**`docs/how-to/maintain-dependencies.md`** (G 184 lines: intro 11-18, lock-check 20-27,
Update deliberately 29-38, Upgrading a package 40-58, the design system package 60-89,
the tooling package 91-114, the Allium binary 116-149, template update 151-156, Actions
158-178, Related 180-184). The page is restructured; G's headings that survive are the
first three, the tooling package, the Allium binary and Actions.

- Intro (11-18), the inverse of G's: manifests write full `x.y.z` caret ranges and
  `Cargo.lock` is the pin, committed and marked `linguist-generated`; every gate passes
  `--locked`, so no recipe can rewrite it; this is decision 0007
  (`../decisions/0007-dependency-policy.md`), a stated deviation from the games'
  exact-pin invariant, because a library that writes `=1.2.3` poisons every downstream
  resolution. Keep G's "Nothing updates them for you." and say why it stays true:
  no Dependabot or Renovate until S01 decides.
- Check that the lockfile still matches (20-27): `just lock-check`, which runs
  `cargo update --workspace --locked` and `uv lock --check`; gate 2 of
  `just check`, so a manifest edited without relocking fails rather than drifting.
- Update deliberately (29-38): `just lock` relocks the workspace members at what the
  manifests state; `just lock-upgrade` moves every dependency to the newest version its
  range admits, which here, unlike under exact pins, changes plenty; read the diff.
- Upgrading a crate (40-58, rewritten): the version lives in `[workspace.dependencies]`
  in the root `Cargo.toml` and nowhere else; edit the range, `just lock`, read the
  lockfile diff, `just sync`, `just deny` (licences, bans and sources, offline),
  `just check`; `just audit` for advisories, which needs the network and is why it sits
  outside `check`. A new crate is a decision-0007 note in the pull request.
- The tools in `tools.txt` (new, in place of 60-89): one `name@version` per line; bump a
  pin by editing the line and running `just install-tools`, which skips a matching
  version and refuses a source build (`--disable-strategies compile`); the two bootstrap
  pins outside the file, cargo-binstall in `scripts/install_tools.sh` and, in CI, `just` and
  cargo-binstall in `.github/actions/setup/action.yml`, move together with it.
- The toolchain (new): `channel` in `rust-toolchain.toml` and `rust-version` in the root
  `Cargo.toml` move together; after a bump run `just install-toolchain`, then
  `just clippy`, because the lint set moves with the release (CONVENTIONS.md §13).
- Hook pins (new, from G 158-178's method): every remote hook in
  `.pre-commit-config.yaml` and `.pre-commit-fix.yaml` is a full commit SHA with the
  release tag as a comment; lychee also carries `LYCHEE_VERSION` at the head of both
  argument lists, which must match the comment (G's hook comment says why). Quote G
  166-172 for the method:

  > To move the pin ahead of the template, resolve the release's commit and replace both
  > the SHA and the comment in all three files. Its tags are annotated, so ask for the
  > commit: the SHA a tag reference answers with names the tag object, which no `uses:`
  > line accepts.
  >
  > `gh api repos/steven-cutting/biscuit_games_tooling/commits/v0.1.0 --jq .sha`

  and generalise: `gh api repos/<owner>/<repo>/commits/<tag> --jq .sha` for a hook or
  an action, with typos' `crate-ci/typos` as the worked example. Drop "ahead of the
  template". The `builtin` hygiene hooks carry no pin.
- Actions in the workflows (158-178 rewritten): `ci.yml`, `audit.yml` and the composite
  `setup` action pin every `uses:` to a full SHA with a version comment; the same `gh
  api` line resolves them; `actionlint` runs inside `just lint` (178, kept).
- Moving the tooling package (91-114): kept with two edits: 104-106's
  `uv lock --upgrade-package biscuit-games-tooling` stays; 110-111 names the hooks that
  read `pyproject.toml` (`check-specs` and `analyse-specs`, CONVENTIONS.md §5).
- Moving the Allium binary (116-149): kept with the decision link at 121 moved to 0005,
  the "package pin" sentences (121-122, 147-149) kept (the pin is B's package version,
  which `pyproject.toml` names), and the
  reference to `work-with-the-specs.md` at 146 kept.
- Drop 151-156 (the template update) entirely.
- Related pages: G's three (all exist here) plus `../decisions/0007-dependency-policy.md`.

Target 1000-1250 words (G has 1329).

### 6. Check after each page

After every page, `just check-docs`; a page that fails is fixed before the next is
started, because the validator reports every page and a later failure hides behind an
earlier one. When all eight are written, `just lint` (markdownlint, typos and lychee
offline over the whole tree) and then the word counts below. A typos finding in a
rewritten page is fixed in the page; one in a module's vocabulary word is handed back
(the remedy is T03's `_typos.toml`).

### 7. Check, commit, hand over

Run the Verification commands, quote their output in the hand-back notes, set `status:`
to `done`, and commit on the ticket branch with short imperative subjects.
**Authorisation required:** pushing the branch and opening the pull request
(CONVENTIONS.md §11). Stop and ask before either.

## Acceptance criteria

- [ ] Exactly the eight pages in Files touched changed, plus this ticket's `status:`
      line; `git diff --stat main -- . ':!tickets'` lists eight files under `docs/`.
- [ ] Every page's frontmatter is byte-identical to the stub's; every H1 equals its
      `title`.
- [ ] Word counts, after the frontmatter: purpose-and-scope 300 or more, repository-map
      400 or more, terminology 1500 or more, first-change 500 or more, develop-locally
      550 or more, test-and-debug 400 or more, work-with-the-specs 1450 or more,
      maintain-dependencies 1000 or more. None is near the forty-word floor.
- [ ] `just check-docs` is green: markdownlint, typos and lychee pass, and the validator
      reports every page valid (the same line count it printed on the stubs).
- [ ] No link to a dropped page or to a G-numbered decision: the grep in Verification
      prints nothing.
- [ ] Every `just` recipe a page names is a recipe in the `Justfile`: the loop in
      Verification prints nothing.
- [ ] No page states a version a pin file states, except the two exceptions rule 4
      names; `../reference/configuration.md` is linked where a figure would otherwise be.
- [ ] G lines 77-136 of work-with-the-specs appear word for word (two edits) under
      "Diagnostics and waivers".
- [ ] `just check` is green in the worktree and `git status --porcelain` is empty
      afterwards.

## Verification

From the worktree root, after the last edit.

```sh
git diff --stat main -- . ':!tickets'
just check-docs
just lint
for p in project/purpose-and-scope project/repository-map project/terminology tutorials/first-change how-to/develop-locally how-to/test-and-debug how-to/work-with-the-specs how-to/maintain-dependencies; do printf '%-36s %s\n' "$p" "$(awk 'f{print} /^---$/{c++; if(c==2)f=1}' "docs/$p.md" | wc -w)"; done
grep -rnE 'platform\.md|accessibility\.md|component-workshop\.md|deploy-to-github-pages\.md|update-from-template\.md|0007-project-managed-allium-cli|frontend-|just dev|just preview|just storybook|npm |npx ' docs/project docs/tutorials docs/how-to || echo "no dropped targets"
for r in $(grep -ohE 'just [a-z][a-z-]*' docs/project docs/tutorials docs/how-to -r | sort -u | cut -d' ' -f2); do just --summary | tr ' ' '\n' | grep -qx "$r" || echo "unknown recipe: $r"; done
for p in project/purpose-and-scope project/repository-map project/terminology tutorials/first-change how-to/develop-locally how-to/test-and-debug how-to/work-with-the-specs how-to/maintain-dependencies; do diff <(git show "main:docs/$p.md" | sed -n '1,7p') <(sed -n '1,7p' "docs/$p.md") >/dev/null && echo "$p frontmatter unchanged"; done
diff <(git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/how-to/work-with-the-specs.md | sed -n '77,137p' | sed 's/0007-project-managed-allium-cli/0005-project-managed-allium-cli/') <(awk '/^### Diagnostics and waivers/{f=1} /^## Related pages/{f=0} f' docs/how-to/work-with-the-specs.md) && echo "waiver terms verbatim"
just check
git status --porcelain
```

Expected: the diff lists eight files, all under `docs/`; `check-docs` and `lint` end
with every hook `Passed` and the validator's one summary line; every word count is at or
above its target; the grep prints `no dropped targets`; the recipe loop prints nothing;
eight `frontmatter unchanged` lines; `waiver terms verbatim`; `just check` exits 0 and the
porcelain output is empty. Quote each in the hand-back notes.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

CONVENTIONS.md §12 assigns this ticket no claim. Settle these while executing and record
the answer in the hand-back notes:

- **Terminology: verbatim or paraphrase.** The brief above keeps G's table rows, which
  are already a paraphrase of the modules' Vocabulary sections, and adds nothing from the
  sections themselves. The alternative is a shorter page holding only `sudoku.allium`'s
  six words (43-46) and pointing at each module's section for the rest. Recommendation:
  keep the rows; the page is tier B because little changes, not because it is short, and
  a reviewer arguing vocabulary needs the table in one place. Whichever is chosen, the
  hand-back says so and T08's `explanation/specifications.md` need not change.
- **Windows in develop-locally.** No. The page covers Ubuntu and macOS, where the
  pixi hazard and `~/.cargo/bin` ordering are the real problems; `windows-latest` joins
  CI the day `crates/pawdoku-cli` exists (CONVENTIONS.md §10) and the page gains a section
  then. Say so in one sentence under Prerequisites.
- **The tutorial's item.** Whether T02 has merged when this page is written, and whether
  `SIDE` still sits in `crates/pawdoku/src/lib.rs` with a `tests` module at its foot.
  Check: `git log --oneline main -- crates/pawdoku/src` and read the file. The
  substitution rule in step 4 applies.
- **The first `lint` and the network.** Whether `just initialize` (which runs
  `prek install`) leaves the hook environments built, or the first `just lint` is still
  the run that clones them. Check: `just lint` with the network off, after
  `just initialize` in a fresh clone. The develop-locally caveat is worded for whichever
  is true.
- **The HTML report path.** Whether `cargo llvm-cov report --html` writes
  `target/llvm-cov/html/index.html` under the pinned cargo-llvm-cov. Check: run it once
  after `just coverage` and `ls target/llvm-cov/html`.
- **proptest regression files and `.gitignore`.** T03 owns `.gitignore`; if it ignores
  `proptest-regressions/`, the test-and-debug sentence that says to commit them is wrong
  for one lane or the other. Check: `grep -n proptest .gitignore` on `main`; hand back
  to T03 if they disagree.
- **CONVENTIONS.md §6 line range.** The waiver-terms row now cites G 77-136 (the
  "Diagnostics and waivers" section), verified; nothing to hand back unless the range
  differs on the day.
