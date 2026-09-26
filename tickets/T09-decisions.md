---
id: T09
title: "Decision records"
status: done
depends_on: [T00, D01, D02]
parallel_with: [T01, T02, T03, T04, T05, T06, T07, T08]
branch: ticket/t09-decisions
estimated_size: M
---

# T09: Decision records

## Context

This repository starts a decision series of its own under `docs/decisions/`: an index
and eleven numbered records (CONVENTIONS.md §8). Four are carried from Pawdoku (G) at
`78d03cdf`, two are decision texts (D01's 0004 and D02's 0011), and five are new to a
Rust library. A carried record opens "Carried from Pawdoku's decision NNNN at
`78d03cdf`" and keeps G's `canonical_for` slug, following the hub's precedent: "Ported
entries say so and keep the topic slug they had, because the slug is what a
cross-repository reference names" (H `AGENTS.md` lines 242-243 at `575e3dd1`).

T00 has merged: `docs/manifest.yml` carries the twelve entries (T00 step 9), and every
file this ticket owns exists as a stub with valid frontmatter and 40 words of prose (T00
step 10). This lane replaces the stubs. T01 to T08 run beside it and touch none of these
files; T11 waits for all nine lanes. Read `CONVENTIONS.md` §0, §1 (every decision and fact
below cites it), §2, §7, §8, §9, §11 and §13 first, then the sources.

Sources, at the commits CONVENTIONS.md §0 pins, read with `git -C <clone> show <commit>:<path>`:

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`. `docs/decisions/README.md` (55
  lines; "The record" at line 20, "The numbering" at 35, "Writing a new one" at 46,
  "Related pages" at 52), `0002-ports-and-fakes.md` (58 lines), `0003-specs-are-the-source-of-truth.md`
  (59), `0004-python-toolchain.md` (61), `0007-project-managed-allium-cli.md` (98). Each
  record has the opening line at 11 and the sections Context, Decision, Consequences,
  What would reopen this, Related pages: at lines 13, 26, 35, 50, 55 in 0002; 13, 24, 35,
  49, 56 in 0003; 13, 20, 32, 51, 58 in 0004; 13, 28, 47, 86, 94 in 0007. `AGENTS.md`
  lines 43-47 (invariant 4, exact pins) and line 73 ("Tests live in `tests/`, never
  colocated with `src/`"). `.markdownlint-cli2.jsonc` line 24 (the reference directory
  ignored). The commit message of `189348e`
  (`git -C /Users/scutting/projects/pawdoku show -s --format=%B 189348e`), whose third
  paragraph says the relocation "is a template departure worth a decision record; noted
  for the PR rather than written here".
- B = `/Users/scutting/projects/biscuit_games_tooling` at `6c5c07f6`,
  `src/biscuit_games_tooling/validate_agents.py`: `MANAGED_DIRECTORIES` at line 27,
  `_expected_files` at 64-75 (only `SKILL.md` per skill and per provider), `_managed_files`
  at 106-114 (every versioned path under a managed directory), and the errors at 146-148
  (`missing managed file`, `unexpected managed file`).
- D01's hand-back notes in this repository: the section "The decision, and decision record
  0004" holds the full text of record 0004.
- D02's hand-back notes in this repository: the section "The decision, and decision record
  0011" holds the full text of record 0011.
- T = `/Users/scutting/projects/biscuit_games_template` at `2283589c`,
  `tickets/T09-decisions.md`, for the shape of this ticket's hand-back notes only.

Verified on 2026-09-23 with `cargo search allium --limit 5` and `cargo info allium-cli`:
`allium-cli` 3.6.1, "CLI for checking Allium specification files", MIT, is on crates.io,
beside `allium-parser` 3.6.1. G's record 0007 lines 42-45 rejected `cargo install allium-cli`
for a frontend repository; record 0005 below reasons from the fact that it exists.

## Goal

Twelve files under `docs/decisions/` in final form: an index of eleven rows and eleven
records in G's shape, each passing `bg-validate-docs`, markdownlint, typos and offline
lychee, so that `just check-docs` and `just lint` are green and a reader of any record
knows what was chosen, what it costs, and what would reopen it, without reading
`tickets/`.

## Non-goals

- `docs/manifest.yml` and `docs/README.md`: frozen at T00 (CONVENTIONS.md §11). A title,
  audience or slug that has to change is handed back as a T00 follow-up on `main`; this
  lane's file keeps the manifest's value meanwhile.
- The pages the records link to: `project/`, `tutorials/` and `how-to/` are T07;
  `explanation/`, `reference/` and `operations/` are T08. A stub is a valid link target;
  do not wait for a lane.
- A twelfth record, a different number, or a different file name for any of the eleven.
- D01's and D02's arguments: records 0004 and 0011 are copied, not rewritten. If either
  ticket's `status:` is not `done`, or its hand-back notes carry no record text, stop and
  report; T00 could not have merged in that state, so this is a defect to raise, not a
  gap to fill.
- G's, H's and T's own records: never modified.

## Files touched

| Path | Source | Change |
| --- | --- | --- |
| `docs/decisions/README.md` | G's index, reshaped by T00 | The table filled; "The numbering" final |
| `docs/decisions/0001-engine-as-a-library.md` | new | Why a separate repository and crate |
| `docs/decisions/0002-specs-are-the-source-of-truth.md` | carried, G 0003 | Nearly verbatim |
| `docs/decisions/0003-effects-behind-traits.md` | adapted, G 0002 | One effect; a trait, a shipped generator, a fake; no clock |
| `docs/decisions/0004-hook-runner-and-checkers.md` | D01's text | Verbatim from D01's hand-back notes |
| `docs/decisions/0005-project-managed-allium-cli.md` | adapted, G 0007 | The toolchain objection lapses; the checksum pin stays |
| `docs/decisions/0006-apache-2-0.md` | new | The licence, no headers, no NOTICE |
| `docs/decisions/0007-dependency-policy.md` | new | Caret ranges, the lockfile as the pin, `--locked`, cargo-deny |
| `docs/decisions/0008-no-std-core.md` | new | The core's constraints and their costs |
| `docs/decisions/0009-rust-quality-gate.md` | new | The Rust gates and the test-placement deviation |
| `docs/decisions/0010-skill-reference-material.md` | new | The record G's `189348e` deferred |
| `docs/decisions/0011-tool-manager.md` | D02's text | Verbatim from D02's hand-back notes |

Every record: frontmatter equal to its manifest entry (five keys in G's order, `title`
double-quoted, lists inline); H1 `# Decision NNNN: <title>`; a carried record's line 11
is the opening sentence; then `## Context`, `## Decision`, `## Consequences`,
`## What would reopen this`, `## Related pages`, the shape G's records carry at the lines
cited above. Related pages are relative Markdown links, as in G; the ticket text below
names targets as paths.

## Steps

1. Create the worktree on `ticket/t09-decisions` from `main` (README.md "How to pick up
   a ticket"), run `just initialize` if `.pixi/envs/default` or `.tools/bin` is missing,
   then `just check-docs` on the untouched stubs and keep the output: it is the baseline
   every later run is compared with.

2. Read `docs/manifest.yml` in the worktree. Its twelve `decisions/` entries are the
   authority for every file's frontmatter. Two entries differ from CONVENTIONS.md §8's
   titles and the manifest wins: 0003 is "Decision 0003: Effects behind traits" and 0008
   is "Decision 0008: A pure no_std core" (T00 step 9). Record 0004's title is
   "Decision 0004: Hook runner and checkers", its slug `decision_python_toolchain` and
   its file name `0004-hook-runner-and-checkers.md` (D01). Record 0011's title is
   "Decision 0011: Tool manager", its slug `decision_tool_manager` and its file name
   `0011-tool-manager.md` (D02). The manifest was frozen at T00, so run
   `grep -n -E '0004|0011' docs/manifest.yml` before writing anything: each row must
   carry that title and slug, because the record cannot pass `bg-validate-docs` with a
   different slug and this lane cannot edit the manifest. A mismatch is a T00 follow-up
   on `main`, handed back first.

3. Rules every file obeys (B's `validate_docs.py` and the hooks enforce them; run step 17
   rather than trusting a reading): 40 or more words, no `TODO`, `TBD`, `FIXME` or lorem
   ipsum; every relative link resolves with exact case to a registered page; no link to
   a G-only page or record; no `/Users/` path; G's name appears only where a record
   reports what Pawdoku decided; the voice is G's records', which describe a choice and
   its cost in plain declarative sentences.

4. The opening line of each carried record (0002, 0003, 0004, 0005), on
   line 11, modelled on G's line 11:

   ```markdown
   *Carried from Pawdoku's decision 0003 at `78d03cdf`, and restated for the library the game's engine moved to. Pawdoku's own record stands where it is.*
   ```

   An adapted record (0003, 0005) adds one sentence after the full stop saying what
   changed: "The boundaries shrank to one." and "The objection to `cargo install` lapses
   in a Rust repository; the checksum pin does not."

5. `0001-engine-as-a-library.md`, new. Context: CONVENTIONS.md §1 decisions 1 and 2 and
   facts 6 and 7; the seven engine modules were written in the game, carry almost no
   game-only wording, and the game's root module is the only one that names a surface.
   Consumers: the game through WebAssembly, a CLI, Python bindings, each a sibling crate
   later (S04). Decision: the engine is `crates/pawdoku` in this repository, a Cargo
   workspace from day one, and the specs move with it. Give the two alternatives and why
   each lost: a TypeScript engine inside the game (one implementation for one surface;
   the solver and the human-solving model are compute the CLI and analysis also need,
   and a second implementation drifts); a Rust crate inside the game's repository (the
   game is rendered from the template with a Node toolchain and a managed file set,
   and a crate there would be gated by neither). Consequences, the ones that hurt first:
   two repositories, where the game restates the clauses it needs and holds them equal
   by test until S04 ships the specs inside the wasm package, so the text is shared truth
   that can drift silently (§13); a second house toolchain with its own CI, hooks and
   pins; a game release now couples to an engine release. What would reopen this: the
   game remaining the only consumer after the bindings spikes close, or a consumer that
   needs the engine's source rather than a package. Related pages:
   `docs/project/purpose-and-scope.md`, `docs/explanation/architecture.md`,
   `docs/explanation/specifications.md`.

6. `0002-specs-are-the-source-of-truth.md`, G's 0003 nearly verbatim with these edits.
   Line 11 is step 4's sentence with 0003. Lines 15-18: "This game's behaviour ... a root
   module under `docs/specs/`, and beside it the modules the game adds" becomes "This
   library's behaviour is written in Allium before the code that implements it: the seven
   engine modules under `docs/specs/`, which moved here from the game with it". Lines
   20-22: "A game is a good argument for the former. The rule that decides a round"
   becomes "A solver is a good argument for the former. The rule that decides whether a
   technique applies". Lines 26-28 keep the `AGENTS.md` split as written. Lines 44-47:
   the link to G's 0007 becomes a link to this repository's 0005 with the same words.
   After that paragraph add one: the modules are read in two repositories, and the
   game's restated clauses are held equal by test on the game's side (§1 fact 6), so a
   clause changed here without a matching change there is a drift no gate in this
   repository sees. Lines 51-54 (reopen) verbatim. Related pages: G's two,
   `docs/explanation/specifications.md` and `docs/how-to/work-with-the-specs.md`.

7. `0003-effects-behind-traits.md`, adapted from G's 0002. Context: G isolated storage,
   randomness and the clock behind ports with fakes and paid for a rule "enforced by
   review because no tool checks it" (G 0002 lines 46-48). Here the core is `#![no_std]`
   (§1 decision 5, §7 invariant 2): a clock, threads, the filesystem and the environment
   are unnameable, and limits are step budgets, never timeouts. Randomness is the one
   effect, and `human-solving.allium` lines 39-40, 476-478 and 1096-1099 already state
   its contract. Decision: the randomness boundary of §9, in the words used there: a
   trait in `crates/pawdoku/src/random.rs` for a stream of draws in `[0, 1)` begun from a
   seed, indexed from zero, the same for the same seed and `random_version`; the library
   ships one generator named by `random_version` so `ExactReplay` holds across callers;
   a test supplies draws through the fake; a caller may implement the trait. `rand` and
   `getrandom` are banned from the core by cargo-deny, with the CLI as the only wrapper.
   Consequences: the review-enforced rule G paid for becomes a compiler error here; a
   replay is reproducible across the game, the CLI and Python from one seed; the costs
   are a generator that must stay bit-stable under its name forever, a stream every
   caller threads through the API, and a fake that is part of the public test story
   (`crates/pawdoku/tests/random.rs`). What would reopen this: an effect that cannot be
   passed as a trait argument, such as cancellation from outside a long solve; a second
   effect that can be is a second trait, not a reopening. Related pages:
   `docs/explanation/architecture.md`, `docs/explanation/layering.md`,
   `docs/reference/testing.md`.

8. `0004-hook-runner-and-checkers.md`: the full record text from D01's hand-back notes,
   copied without edits except that the frontmatter equals the manifest entry. The text
   opens with "Carried from Pawdoku's decision 0004 at `78d03cdf`" and the slug is
   `decision_python_toolchain`. If D01's text lacks a section of the
   shape, or links to a page the manifest does not register, stop and hand it back;
   do not repair it here.

9. `0005-project-managed-allium-cli.md`, adapted from G's 0007. Keep G's lines 15-17
   (why something must check the modules), 23-26 (the two unrelated packages of the same
   name), 30-40 (the decision and where the pin sits: `bg-install-allium`, per D01),
   49-52 (the lockfile cannot speak for it), 54-59 (checksums are hand-computed because
   upstream's `SHA256SUMS.txt` covers only the editor extension and language server),
   61-62 (`.tools/` stays ignored), 64-66 with the gate numbers of §4 (16 and 17), 68-81
   (why the wrapper reads the JSON; a missing binary fails the gate) and 83-84. Replace
   lines 42-45 with the argument for this repository: `allium-cli` 3.6.1 is on crates.io
   (state the date verified), so `cargo install allium-cli --locked` and a
   `tools.txt` line (or a pixi dependency, if conda-forge packages allium) are both real
   alternatives, and G's objection to a third toolchain lapses because cargo is this
   repository's toolchain. Neither is taken: a source build costs minutes on every cold
   runner and verifies nothing, and a binstall of the release artefact verifies only
   checksums upstream publishes, which for the platform binaries is none (G lines
   54-59), so the house installer with its four recorded values is still the only path
   that proves what was downloaded. Also: one
   owner per pin (the tooling package, moved by one line, shared with every house
   repository) and the JSON-reading runner travel together with the version they were
   verified against. What would reopen this: G's two clauses (lines 88-90), restated as
   upstream publishing checksums that cover the platform binaries, at which point a
   `tools.txt` line or a pixi dependency replaces the installer and the record is
   superseded. Related pages:
   G's three, `docs/how-to/work-with-the-specs.md`, `docs/how-to/maintain-dependencies.md`,
   `docs/reference/quality-gates.md`.

10. `0006-apache-2-0.md`, new. Context: §1 decision 3; the first licensed repository in
    the house; the crates are `publish = false` until S02, but a licence is decided at
    the first commit because every later contribution is made under it, and the wasm and
    Python packages S04 designs each carry a licence field. Decision: Apache-2.0;
    `LICENSE` at the root is the verbatim text; `license = "Apache-2.0"` in
    `[workspace.package]` and inherited by every crate; no dual MIT/Apache; no per-file
    headers; no `NOTICE` file. Say why Apache over MIT (the explicit patent grant and its
    termination clause, and section 5's contribution term, which does the work a
    contributor agreement would); why not the Rust-ecosystem dual licence (it exists so
    that MIT-only and GPLv2 consumers can take a crate; the consumers are house games,
    and one licence is one thing to explain); why no headers (the licence's appendix
    makes them optional, and a header is one more thing every new file gets wrong); why
    no `NOTICE` (section 4(d) obliges a redistributor only where one exists, and nothing
    here needs attribution beyond the licence). Consequences: Apache-2.0 is incompatible
    with GPLv2-only programs, which is accepted; every dependency must be compatible,
    which `deny.toml`'s licence allow list enforces at gate 12; a contributor's work is
    licensed by section 5 with no separate agreement. What would reopen this: a consumer
    that can only take MIT (adding it is additive and needs no relicensing of past
    contributions under section 5), or the house choosing one licence for every
    repository. Related pages: `docs/explanation/security-model.md`,
    `docs/how-to/maintain-dependencies.md`.

11. `0007-dependency-policy.md`, new. Context: §1 fact 8; G's `AGENTS.md` lines 43-47
    (invariant 4: "No `^`, no `~`"), right for an application; §1 fact 3 (the snapshot
    between recipes); §11 ("No dependency is added outside T02"). Decision: manifests
    write full `x.y.z` caret ranges in one `[workspace.dependencies]` table;
    `Cargo.lock` is committed and is the pin; every cargo invocation in a gate carries
    `--locked` and `just lock-check` proves the lockfile matches; cargo-deny checks
    licences, bans and sources offline at gate 12 and advisories in `just audit`, which
    is CI-only because the RustSec fetch can fail for reasons unrelated to the diff;
    `cargo-shear` fails the gate on an unused dependency; `rust-version` follows the
    toolchain pin until first publication; no `.cargo/config.toml`. State the deviation
    plainly: this is the one place `AGENTS.md` departs from the games' invariants, and
    why: an `=1.2.3` in a library manifest is a constraint every consumer inherits, so
    two such libraries in one tree cannot resolve, and the game's wasm package would
    carry the poison downstream. Consequences that hurt: the lockfile tests one
    resolution and consumers get another, so a bug in a dependency version this
    repository never resolves is invisible until a consumer hits it; nothing updates any
    pin until S01 decides how; every addition goes through one lane (T02) and later
    through one reviewer, with the reason recorded; caret ranges make `cargo update`
    a real change every time. What would reopen this: a binary crate in the workspace
    is not a reopening, because one lockfile still resolves once; publishing (S02) may
    add a minimal-versions check; a native dependency in a bindings crate that must be
    pinned exactly would be a per-crate exception recorded here. Related pages:
    `docs/how-to/maintain-dependencies.md`, `docs/reference/configuration.md`.

12. `0008-no-std-core.md`, new. Context: §1 decision 5; §7 invariants 2, 3, 4 and 5; the
    bindings the workspace is shaped for (pyo3, wasm-bindgen) and what they need of a
    type. Decision, as a list a reader can check against `Cargo.toml` and `lib.rs`:
    `#![no_std]` with `extern crate alloc`; `cargo check --target wasm32v1-none` under
    the feature powerset as the mechanical proof (a target with no std at all, beside
    `wasm32-unknown-unknown`, which wasm-bindgen targets); `rand` and `getrandom` banned
    by cargo-deny, the seed comes from the caller (decision 0003); every public type
    `Send + Sync + 'static`, `Clone` and `Debug`, asserted in
    `crates/pawdoku/tests/api_bounds.rs`; public enums and structs `#[non_exhaustive]`;
    no panics in the public surface, with clippy's `unwrap_used`, `expect_used`, `panic`,
    `unreachable`, `todo` and `unimplemented` lints as errors at gate 6; errors are
    `thiserror` enums with stable `Display` text, because pyo3 and wasm-bindgen build
    their exceptions from that text, so it is the cross-language contract; features
    `default = []` with `serde` optional and additive, and nothing else; `usize` is
    32 bits on wasm, so counts that must agree across targets are `u32` or `u64`;
    `libm` for `ln`, `exp` and `sqrt` if a rating model needs them, never `std` maths.
    Consequences: `alloc::vec::Vec`, `alloc::string::String` and `BTreeMap` in place of
    `HashMap` (there is no hasher in `alloc`); `#[cfg(test)] extern crate std;` so
    proptest runs; `core::error::Error` rather than `std::error::Error`; the
    `std_instead_of_core` family of lints on every file; float maths through `libm` or
    avoided; a step-budget limit where a timeout would have been simpler. What would
    reopen this: a need for a std-only API in the core (a clock, files, threads, a
    hashed map for speed), and the answer would be an additive `std` feature or a
    sibling crate, not `std` in the core; or rustup dropping `wasm32v1-none`, which would
    change the proof, not the decision. Related pages: `docs/explanation/architecture.md`,
    `docs/reference/api.md`, `docs/reference/configuration.md`.

13. `0009-rust-quality-gate.md`, new. Context: the seventeen gates of §4 and which are
    Rust's (4 to 13); the `[workspace.lints]` table of T02's `Cargo.toml`; §5's reason
    clippy is not a hook; §13's doctest caveat; G's `AGENTS.md` line 73. Decision:
    clippy `pedantic` and `cargo` groups at `warn` in the manifests and `-D warnings` on
    the `clippy` recipe alone, so a local build never breaks mid-edit and the gate still
    fails; `#[expect(lint, reason = "...")]` only, never a blanket `allow`; a line
    coverage floor of 90 over `crates/pawdoku/src/**` measured by `cargo llvm-cov nextest`,
    lowered by lowering complexity, never the number; `cargo hack --feature-powerset` on
    the host and both wasm targets; rustdoc with `-D warnings --cfg docsrs`; taplo for
    every TOML; `cargo-shear` for unused dependencies; nextest for unit and integration
    tests and `cargo test --doc` for doctests, which nextest cannot run and llvm-cov
    cannot measure on stable, so doctests run uncovered; every tool pinned in
    `pyproject.toml` and installed by pixi into the environment, cargo-hack through
    `tools.txt`, one owner per pin (decision 0011); clippy runs at gate 6 and in CI but
    never as a commit hook, because a cold run blocks every commit for tens of seconds
    to minutes, the same call G makes by keeping `svelte-check` out of its hook. The
    test-placement deviation, stated as such: unit tests live in
    `#[cfg(test)] mod tests` inside the module they test, integration tests in
    `crates/pawdoku/tests/`, and doc examples are tests; this departs from the games'
    "never colocated" rule because Rust's privacy model means a private item can only be
    tested from its own module, `tests/` sees only the public API, and clippy's
    `tests_outside_test_module` lint holds the module shape. Consequences that hurt:
    pedantic lints produce false positives that each need an `expect` with a reason; the
    lint set moves with the toolchain pin, so every bump is a clippy pass; the powerset
    doubles with each feature; the floor is a statement about unit and integration
    tests only; a cold `just check` is minutes. What would reopen this: llvm-cov
    measuring doctests on stable; clippy's cold time falling enough to hook it; a
    mutation or fuzzing gate (S03) that changes what "tested" means. Related pages:
    `docs/reference/quality-gates.md`, `docs/reference/testing.md`,
    `docs/explanation/quality-philosophy.md`.

14. `0010-skill-reference-material.md`, new; the record G's `189348e` deferred. Context:
    §1 fact 5; B's validator lists exactly one `SKILL.md` per skill and per provider as
    expected (lines 64-75), counts every versioned path under `.agents/`, `.claude/` and
    `.codex/` as managed (27, 106-114) and fails on any other (146-148), so the seven
    vendored skills' reference material (language reference, patterns, worked examples,
    JSON schemas) cannot sit beside the `SKILL.md` that cites it; G's skills link to it
    by relative path (`.agents/skills/allium/SKILL.md` line 35 and elsewhere). Decision:
    `allium-skill-reference/` at the repository root, vendored byte-for-byte with the
    skills and the hashes in `skills-lock.json`; excluded from markdownlint, lychee and
    typos and marked `linguist-vendored` in `.gitattributes` (§5), because it is
    upstream's text and not this repository's to lint; the skills keep their
    `../../../allium-skill-reference/` links. Consequences: a root directory that is not
    in the house shape and has to be explained on the repository map; an upstream typo or
    dead link lands here unflagged; moving a skill pin means moving both the skill and
    its reference in one change; the agent contract page (`docs/reference/agent-contract.md`)
    carries the rule. What would reopen this: B's validator tolerating a `references/`
    subdirectory beside a `SKILL.md` (a C02 change), or the Allium plugin shipping the
    reference itself, at which point the directory is deleted and the links move.
    Related pages: `docs/reference/agent-contract.md`, `docs/project/repository-map.md`.

15. `0011-tool-manager.md`: the full record text from D02's hand-back notes ("The
    decision, and decision record 0011"), copied without edits except that the
    frontmatter equals the manifest entry. The slug is `decision_tool_manager`; the
    record is new, not carried, so it has no line 11 opening sentence, and it names
    which consequences of 0004 it supersedes while 0004 itself stays verbatim. If D02's
    text lacks a section of the shape, or links to a page the manifest does not
    register, stop and hand it back; do not repair it here.

16. `docs/decisions/README.md`: T00's shape kept (G's first two paragraphs with "how
    Pawdoku is built" made "how this library is built"; "Writing a new one" and "Related
    pages" verbatim from G lines 46-55). The table: eleven rows, each linking its file
    and carrying the short title (the manifest `title` after "Decision NNNN: "), and one
    sentence beneath it: 0004 is superseded in part by 0011, which says which of its
    consequences stand. "The numbering", final, in place of G's lines 35-44: this series
    is the repository's own and starts at 0001; four entries were carried from Pawdoku
    at `78d03cdf` (0002, 0003, 0004, 0005), each says so under its heading and keeps the
    slug it had, because the slug is what a cross-repository reference names; the game
    restates the engine's clauses and cites these records by slug; nothing here is
    rendered from a template, so no frozen inventory applies, and the next decision
    this repository takes is 0012.

17. After each file, `just check-docs`; the output must match step 1's baseline apart
    from the word counts. When all twelve are done, `just lint`, then `just check`, then
    the Verification commands. Set `status: done` in this file, commit on the ticket
    branch in one or a few commits with short imperative subjects, and stop: pushing and
    opening the pull request are separately authorised (CONVENTIONS.md §11).

## Acceptance criteria

- Twelve files under `docs/decisions/`, and no other file changed except this ticket's
  `status:` line.
- Each record has the five sections in G's order and 40 or more words in the body; a
  realistic total is 45 to 100 lines, as G's carried records are 58 to 98.
- Frontmatter of all twelve files equals the manifest entry key for key; H1 equals
  `title`; 0003 and 0008 carry the manifest's shorter titles.
- Line 11 of 0002, 0003, 0004 and 0005 is step 4's sentence with
  the right Pawdoku number (0003, 0002, 0007, 0004), and each keeps G's slug.
- 0004 is D01's text unchanged and 0011 is D02's; 0005 states that `allium-cli` is on
  crates.io and why it is still not the install path; 0007 and 0009 each name their
  deviation from the games' `AGENTS.md` and say why; 0010 cites what the validator
  forbids.
- Every "Related pages" link resolves to a registered page; no link to a G-only page.
- The index has eleven rows, the sentence on 0004 and 0011 beneath them, and a final
  "The numbering"; "Writing a new one" is G's verbatim.
- `just check-docs`, `just lint` and `just check` green in the worktree.

## Verification

```sh
just check-docs
just lint
just check
ls docs/decisions | wc -l
for f in docs/decisions/000[2-5]-*.md; do sed -n 11p "$f"; done
for f in docs/decisions/*.md; do printf '%s %s\n' "$f" "$(grep -c '^## ' "$f")"; done
grep -rn -E 'TODO|TBD|FIXME|/Users/' docs/decisions/ || echo "no placeholders"
grep -L 'What would reopen this' docs/decisions/00*.md || echo "every record has the section"
git status --porcelain
```

Expected: the three `just` runs exit 0 and `check-docs` reports every page valid; `ls`
prints `12`; the loop prints four lines each beginning `*Carried from Pawdoku's decision`;
every record
counts `5` sections and the index `4`; the two greps print their messages; `git status`
lists only the twelve files and this ticket. Quote each in the hand-back notes.

## Hand-back notes

### What was verified, and how

Executed on 2026-09-25 in the Supacode worktree for this ticket (branch `T09-decisions`,
Supacode's name for `ticket/t09-decisions`), with `PATH="$HOME/.cargo/bin:$PATH"` on every
`just` call so that `check-toolchain` saw rustup's proxy. `.pixi/envs/default` and
`.tools/bin` already existed, so `just initialize` was not run.

Step 1, the baseline `just check-docs` on the untouched stubs:

```text
markdownlint.............................................................Passed
typos....................................................................Passed
lychee...................................................................Passed
bg-validate-docs
Validated 37 pages and 38 canonical topics.
```

Step 2: `grep -n -E '0004|0011' docs/manifest.yml` printed the two rows with
"Decision 0004: Hook runner and checkers" / `decision_python_toolchain` and
"Decision 0011: Tool manager" / `decision_tool_manager`, so no T00 follow-up was needed.
D01 and D02 both carry `status: done` and their hand-back notes hold the record texts.

Steps 8 and 15: `diff` of D01's fenced block (lines 352-436) against `0004` and of D02's
(lines 294-381) against `0011` printed nothing; both copies are byte-identical.

Step 9, the crates.io fact re-verified on 2026-09-25 with `cargo info allium-cli`:

```text
allium-cli
CLI for checking Allium specification files
version: 3.6.1
license: MIT
rust-version: unknown
documentation: https://docs.rs/allium-cli/3.6.1
crates.io: https://crates.io/crates/allium-cli/3.6.1
```

`cargo info allium-parser` printed version 3.6.1, MIT, "Parser and structural validator
for the Allium specification language", so 0005's "beside `allium-parser` 3.6.1" is also
verified on 2026-09-25. `cargo search allium --limit 5` also showed that the crate named
plain `allium` (0.1.3) is an onion-routing library; 0005's same-name paragraph gained one
clause saying so.

Step 17: `just check-docs` after every file matched the baseline exactly, including the
final count `Validated 37 pages and 38 canonical topics.` `just lint` printed twenty-three
hooks, every one `Passed`. `just check` ran the seventeen recipes in order and ended:

```text
==> just check-clean
bg-project-check clean "$1"
The worktree matches the check baseline.

All checks passed and the worktree is unchanged.
```

Within it, `check-docs` reported the baseline count, `check-agents` reported
`Validated AGENTS.md, 2 adapters, and 14 skills.`, and `check-specs` and `analyse-specs`
reported eight specifications with no diagnostics and no findings.

The Verification block:

```text
$ ls docs/decisions | wc -l
12
$ for f in docs/decisions/000[2-5]-*.md; do sed -n 11p "$f"; done
*Carried from Pawdoku's decision 0003 at `78d03cdf`, and restated for ...
*Carried from Pawdoku's decision 0002 at `78d03cdf`, and restated for ...
*Carried from Pawdoku's decision 0004 at `78d03cdf`, and restated for ...
*Carried from Pawdoku's decision 0007 at `78d03cdf`, and restated for ...
$ for f in docs/decisions/*.md; do printf '%s %s\n' "$f" "$(grep -c '^## ' "$f")"; done
docs/decisions/0001-engine-as-a-library.md 5
docs/decisions/0002-specs-are-the-source-of-truth.md 5
docs/decisions/0003-effects-behind-traits.md 5
docs/decisions/0004-hook-runner-and-checkers.md 5
docs/decisions/0005-project-managed-allium-cli.md 5
docs/decisions/0006-apache-2-0.md 5
docs/decisions/0007-dependency-policy.md 5
docs/decisions/0008-no-std-core.md 5
docs/decisions/0009-rust-quality-gate.md 5
docs/decisions/0010-skill-reference-material.md 5
docs/decisions/0011-tool-manager.md 5
docs/decisions/README.md 4
$ grep -rn -E 'TODO|TBD|FIXME|/Users/' docs/decisions/ || echo "no placeholders"
no placeholders
$ grep -L 'What would reopen this' docs/decisions/00*.md || echo "every record has the section"
every record has the section
$ git status --porcelain
 M docs/decisions/0001-engine-as-a-library.md
 ... (the twelve files under docs/decisions/)
 M tickets/T09-decisions.md
```

A small Python loop over `docs/manifest.yml` compared lines 1-7 and the H1 of each of the
twelve files with its manifest entry key for key: twelve `ok`, zero mismatches. The
records run 58 to 113 lines; the index 57.

### Deviations, and why

- 0004's line 11 carries a third sentence beyond step 4's two ("This repository adds the
  Rust facts: ..."). It is D01's text, and step 8 and the Non-goals say the record is
  copied without edits, so the sentence stays; the Verification loop only requires the
  line to begin `*Carried from Pawdoku's decision`, which it does.
- 0002's Decision quotes the `AGENTS.md` split as "what the *engine* should do", not G's
  "the game". Step 6 says to keep the split "as written", and this repository's
  `AGENTS.md` line 17 writes "engine"; the record quotes the file it names.
- 0005 departs from step 9's keep-list in three places. G's lines 19-21 are adapted rather
  than dropped ("Neither `Cargo.lock` nor `pixi.lock` can name it ... every other pin in
  this repository is proved by `just lock-check`"), because they hold the only use of the
  `[upstream]` reference and the sentence that frames the whole record, while G's "pins
  every dependency exactly" would contradict 0007. G's line 50 compared the binary to the
  Chromium build the story tests render in, a frontend artefact this repository lacks;
  with the maintainer's agreement on 2026-09-25 it names cargo-hack in `.tools/bin` via
  `tools.txt` instead, the same shape here. And the verification date is 2026-09-25, the
  maintainer having chosen a fresh `cargo info` over citing the ticket's 2026-09-23.
- 0005 is 113 lines, above the acceptance criteria's "realistic 45 to 100". G's 0007 was
  98 and the crates.io argument adds a paragraph; nothing was cut to meet a guide figure.
- Commits are on `T09-decisions`, the branch Supacode created for this worktree, rather
  than the `branch:` field's `ticket/t09-decisions`; the same convention every earlier
  lane in this repository followed.

### Handed back

Nothing. No manifest title or slug needed changing, typos flagged no word in any record,
and lychee resolved every relative link, so `_typos.toml` and the frozen files are
untouched. The one cross-repository note is already on record in T06's hand-back: the
game owes its own record for the `allium-skill-reference/` relocation; 0010 here is this
repository's, not the game's.

### Open points settled

- 0008 and 0009 stay two records, for the reason the Open points give: 0008 is a property
  every consumer inherits, 0009 a property of this repository's gate, each with its own
  reopening condition.
- No separate rustup record. 0011 records the pixi/rustup split and 0009's Context states
  the exact toolchain pin as the reason the lint set is stable.
- typos passed over every record in `just lint` and `just check`, so no `extend-words`
  entry is handed back to T03.

## Open points

- Whether 0008 and 0009 should be one record. No: 0008 is a property of the crate that
  every consumer inherits, 0009 is a property of this repository's gate, and each has
  its own reopening condition.
- Whether "rustup and the exact toolchain pin" (§1 decision 4, §2) needs a record of its
  own. Answered by D02: 0011 records the split, pixi for the tools and rustup for the
  compiler, with the argument against a pixi-owned cargo, so a separate rustup record
  is not needed; 0009's Context still states the pin as the reason the lint set is
  stable.
- Whether typos flags a word in a record (a slug like `no_std` or a crate name). Check:
  the `just lint` run; the remedy is an `extend-words` entry in `_typos.toml`, which is
  T03's file, so hand it back.
