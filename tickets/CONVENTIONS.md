# Conventions for bootstrapping `libpawdoku`

This document is the design every ticket under `tickets/` obeys. A ticket cites it by
section (`CONVENTIONS.md §4`) instead of restating it, and embeds exact content only where
the agent executing the ticket would otherwise have to guess. Where a ticket and this
document disagree, this document wins and the ticket is corrected. Changes to this
document go through a pull request on `main`, never through a lane branch, because every
lane reads it.

## 0. What is being built, and where the sources are

`steven-cutting/libpawdoku` (this repository; today one `init` commit holding a stub
`README.md`, no remote) becomes a Cargo workspace whose core crate `crates/pawdoku` (crate
name `pawdoku`, Apache-2.0, `publish = false` until S02 lifts it) is the classic-sudoku
engine the game Pawdoku consumes later through WebAssembly, and a CLI and Python bindings
consume later as sibling crates. The engine implements the seven engine-level Allium
modules that live in the game today (`sudoku`, `solver`, `technique`, `reach`, `effort`,
`lapse`, `human-solving`); T06 moves them here, adapted. The repository carries the house
toolchain, handbook, agent contract and quality gate in the shape Pawdoku has, with the
frontend tools replaced by Rust ones.

Five repositories are the sources. This document cites them with a letter and line
numbers that refer to these exact commits. **P** and **H** keep the meaning they have in the
template's own tickets; the game is **G**, not P.

| Letter | Repository | Local clone | Commit |
| --- | --- | --- | --- |
| **G** | `steven-cutting/pawdoku`, the game this engine serves; the newest rendered toolchain and the specs | `/Users/scutting/projects/pawdoku` | `78d03cdf` (HEAD on 2026-09-23; rendered from T at `v2.0.0`; pins B at `v0.2.0`) |
| **T** | `steven-cutting/biscuit_games_template`, the Copier template; its `tickets/` is the style reference for this directory | `/Users/scutting/projects/biscuit_games_template` | `2283589c` (HEAD; `v2.0.0` sits two commits earlier and differs only in C08's workflow pins and CHANGELOG) |
| **B** | `steven-cutting/biscuit_games_tooling`, the Python checkers and reusable workflows | `/Users/scutting/projects/biscuit_games_tooling` | `6c5c07f6` (= `v0.3.0`; `v0.2.0`, which G pins, differs only in the Chromatic single-commit skip) |
| **P** | `steven-cutting/poodl`, the first game | `/Users/scutting/projects/poodl` | `a2860fc1` |
| **H** | `steven-cutting/biscuit_games`, the hub | `/Users/scutting/projects/biscuit_games` | `575e3dd1` (cited only for `AGENTS.md` lines 242-243, the carried-slug precedent) |

Two older repositories, `/Users/scutting/projects/workato/street_meats` (one `init`
commit) and `/Users/scutting/projects/foo/www` (`42ed403`), are the earlier copier
template Poodl was distilled from; a ticket cites them only where G and T lack something.

Read a pinned file with `git -C <clone> show <commit>:<path>` when a clone has moved on;
never modify a clone.

Tooling verified on 2026-09-23 on the maintainer's machine: just 1.51.0, uv 0.11.18, gh
2.100.0 (authenticated as `steven-cutting`), node 26.5.1, `rustc`/`cargo` 1.96.0 installed
by `pixi global` at `~/.pixi/bin` (a conda-forge build, **not** a rustup proxy), **no
rustup**, and none of cargo-binstall, cargo-nextest, cargo-llvm-cov, cargo-deny,
cargo-hack, cargo-shear, taplo, prek, typos or lychee on `PATH`. Rust stable on that day
was 1.98.1 (`channel-rust-stable.toml`, released 2026-09-01). Re-verified on 2026-09-24
for D02: rustup 1.29.1 and cargo-binstall 1.23.0 now in `~/.cargo/bin` (T00 installed
them); pixi 0.81.0 at `~/.pixi/bin/pixi`; `~/.pixi/bin/cargo` still shadows rustup in
any shell that does not source `~/.cargo/env`. conda-forge that day: cargo-nextest
0.9.146, cargo-llvm-cov 0.9.1, cargo-deny 0.20.2, taplo 0.10.0, prek 0.5.3,
cargo-binstall 1.23.0, cargo-shear 1.13.4, just 1.58.0, python 3.14; no cargo-hack, no
ripsecrets, no editorconfig-checker.

## 1. Decisions taken with the maintainer, and the facts everything rests on

Decisions, taken on 2026-09-23:

1. **The engine is a library of its own**, in a Cargo workspace from day one:
   `crates/pawdoku` now; `crates/pawdoku-cli`, `crates/pawdoku-py` (pyo3 + maturin) and
   `crates/pawdoku-wasm` (wasm-bindgen) later, as sibling crates. Only the library is
   built by these tickets; every decision keeps the other three cheap.
2. **The engine specs move here.** The seven engine modules and the two explanation pages
   they cite leave G for this repository, adapted so that nothing game-only remains, after
   a clause-by-clause relevance review (§9). `pawdoku.allium`, the game's root module,
   stays in G.
3. **Apache-2.0**, the first licensed repository in the house. `publish = false` on every
   crate until S02 decides the release process and the crates.io name.
4. **rustup and `rust-toolchain.toml`**, pinned to an exact stable release (§2). The
   maintainer's pixi toolchain is removed or shadowed; a gate refuses to run on any cargo
   that is not rustup's proxy.
5. **A pure `no_std` core.** `crates/pawdoku` is `#![no_std]` with `alloc`, so a clock,
   threads, the filesystem, the environment and `HashMap` are unnameable rather than
   forbidden by convention; `cargo check --target wasm32v1-none` (a target with no std)
   proves it. Randomness is the one effect, reached through a library-owned trait for a
   seeded stream of draws; `rand` and `getrandom` are banned from the core by cargo-deny.
   No timeouts: limits are step budgets.
6. **The Python toolchain stays**, decided in D01 (done 2026-09-24; decision 0004). G's
   decision 0004 keeps prek and B's checkers in a repository that ships no Python; the
   same argument holds here. The maintainer asked for the evaluation to be written
   down, and D01's hand-back notes hold it. Every Rust-side decision in this document
   was the same under either outcome. Who installs the toolchain is decision 9.
7. **Tickets in this repository**, in T's format, executed by AI agents in separate
   worktrees (§11). D01 first, then T00 as one shape-complete foundation, then nine
   parallel lanes.
8. **Dependency updates, release automation, benchmarks, fuzzing and the bindings
   crates are spikes** (S01 to S04), picked up after T11.
9. **pixi owns the tools, rustup keeps the compiler**, decided in D02 (done 2026-09-24;
   decision 0011). `pyproject.toml` is a pixi manifest pinning Python, just, prek,
   cargo-binstall and every cargo tool conda-forge carries, plus B as a PyPI git
   dependency; `pixi.lock` is the pin; the environment is the gitignored `.pixi/`. uv,
   `uv.lock`, `.python-version` and the per-machine cargo-binstall are gone. cargo-hack
   is not on conda-forge, so a one-line `tools.txt` and the environment's cargo-binstall
   put it in `.tools/bin` beside the Allium checker. Decision 4 is untouched: the pixi
   environment holds no `rust`, and `check-toolchain` still refuses any cargo that is
   not rustup's proxy. D02's hand-back notes hold the comparison and the record.

Facts, each verified in source, that shape the mechanism:

1. **B's agent checker is game-shaped.** `validate_agents.py` lines 47-54 require the
   literal phrases `untrusted`, `just check`, `explicit authorization`, `ai_tmp/`,
   `docs/specs/` and **`runes`** in `AGENTS.md`, and `_project.py` lines 44-50 read the
   package's configuration only from `pyproject.toml`. `AGENTS.md` carries an honest
   sentence with the word until C02 ships a release that makes the list configurable
   (decision 0004).
2. **B's docs checker is language-neutral.** `validate_docs.py` accepts kinds `project`,
   `tutorial`, `how-to`, `explanation`, `reference`, `operations`, `decision`, audiences
   `user`, `contributor`, `maintainer`, `operator`, `agent`, a 40-word minimum and
   predicates from `[tool.biscuit-games-tooling]`. Nothing in it names a frontend.
3. **B's gate runner snapshots the worktree.** `run_project_check.py` lines 41-62 hash
   every tracked and untracked-unignored path after each recipe and abort on any change.
   So `target/`, `.tools/`, `.pixi/` and everything a Rust recipe writes are gitignored,
   and every cargo invocation in a gate carries `--locked` so no recipe can rewrite
   `Cargo.lock`; the pixi calls in a gate are `--frozen` or `lock --check` for the same
   reason.
4. **The Allium binary is pinned by checksum.** B's `install_allium.py` line 34 pins
   allium-tools `3.6.1` with SHA-256 per target; `run_allium.py` reads the JSON
   diagnostics because the binary's exit code is not trustworthy. G's modules are
   `-- allium: 3`.
5. **G vendors the Allium skills.** `skills-lock.json` records seven skills from
   `juxt/allium` with hashes that match no file as committed (the hash's input is
   unknown; no gate reads the lock); their reference material sits at `allium-skill-reference/`
   because the validator forbids extra files under `.agents/`; G's commit `189348e`
   says the relocation "is a template departure worth a decision record" and did not
   write one. This repository writes that record (§8, 0010).
6. **Allium has no cross-repository import.** G will restate the clauses it needs and
   hold them equal to this repository's text by test, exactly as it holds the platform's
   figures today (T's CONVENTIONS §7). Until S04 ships the specs inside the wasm package,
   the text is shared truth across two repositories and can drift silently.
7. **Game-only wording in the engine modules is small.** `sudoku.allium` lines 36-37 and
   40-41, `lapse.allium` lines 45-46, `human-solving.allium` lines 39-40, 476-478 and
   1096-1099, and `docs/explanation/human-solving.md` lines 16 and 255 name the game, its
   Play surface or its randomness port; `solver`, `technique`, `reach`, `effort` and
   `solving-sudoku.md` carry none. Both explanation pages are cited as Sources by the
   modules, so they move too.
8. **Exact pins in a library manifest are wrong.** The games' invariant 4 ("no `^`, no
   `~`") is right for an application. A library that writes `=1.2.3` poisons every
   downstream resolution. Here `Cargo.lock` is the pin, manifests write full `x.y.z`
   caret ranges, and `--locked` everywhere makes the lockfile authoritative (§8, 0007).
9. **T's repository bootstrap script exists.** `T/scripts/bootstrap_repo.sh` takes
   `--no-pages --checks <a,b>` and applies branch protection and hygiene settings through
   `gh api`; T01 and T11 reuse it rather than restating the calls.

## 2. Toolchain and pins

### `rust-toolchain.toml` (exact text; T00 creates it, T02 verifies it)

```toml
[toolchain]
# An exact stable release, bumped deliberately. `channel = "stable"` would move the
# clippy lint set every six weeks and break `-D warnings` with no diff. rust-version
# in Cargo.toml follows this pin until the crate is first published.
channel = "1.98.1"
profile = "minimal"
components = ["rustfmt", "clippy", "llvm-tools-preview", "rust-src"]
# wasm32-unknown-unknown is what wasm-bindgen targets. wasm32v1-none has no std at
# all, so a check against it is the mechanical proof that the core is no_std.
targets = ["wasm32-unknown-unknown", "wasm32v1-none"]
```

`rust-src` serves rust-analyzer; `llvm-tools-preview` serves cargo-llvm-cov (§12 checks
whether the component is now named `llvm-tools`; rustup accepts both).

### `pyproject.toml` (the pixi manifest; T00 writes it, T03 finalises comments)

The complete text is in D02's hand-back notes ("Files T00, T03 and T04 create") and is
not repeated here. Its pins: `python = "3.14.*"`, `just ==1.58.0`, `prek ==0.5.3`,
`cargo-binstall ==1.23.0`, `cargo-nextest ==0.9.146`, `cargo-llvm-cov ==0.9.1`,
`cargo-deny ==0.20.2`, `cargo-shear ==1.13.4` (the newest conda-forge carried on
2026-09-24; the pin follows conda-forge, not crates.io), `taplo ==0.10.0`;
`biscuit-games-tooling` at git tag `v0.3.0` under `[tool.pixi.pypi-dependencies]`;
`channels = ["conda-forge"]`, `platforms = ["linux-64", "osx-arm64"]`,
`requires-pixi = ">=0.81.0"`. No `rust`: rustup owns the compiler (decision 9). No
`[tool.pixi.tasks]`: the `Justfile` is the task runner. `pixi.lock` is generated by
`pixi lock`, committed, and never hand-edited; `pixi install --frozen` (in `just sync`)
installs it, `pixi lock --check` (in `just lock-check`) proves it, and `pixi install
--locked` (the first line of `scripts/initialize.sh`, and `setup-pixi` in CI) refuses a
lockfile that disagrees with the manifest. The environment is `.pixi/envs/default`,
gitignored; its `bin` is first on every recipe's `PATH`.

### `tools.txt` (exact text; the exception list, frozen at T00)

```text
# Binaries pixi cannot own, installed by `just install-tools` into the gitignored
# .tools/bin through cargo-binstall (itself a pixi dependency in pyproject.toml), one
# name@version per line, verified 2026-09-24. Everything conda-forge carries is pinned
# in pyproject.toml and locked in pixi.lock; this file is the escape hatch for what it
# lacks, and empties the day cargo-hack is packaged there. Tools a prek hook pins
# (typos, ripsecrets, lychee, shellcheck, actionlint, editorconfig-checker,
# markdownlint-cli2) are not listed: one owner per pin.
cargo-hack@0.6.45
```

The install command is
`grep -v '^#' tools.txt | xargs cargo binstall --root .tools --no-confirm --locked --disable-strategies compile`
(§12: `--root .tools` lands binaries in `.tools/bin`; a matching version already there is
skipped; `--disable-strategies compile` refuses a source build so an unpinnable tool fails
loudly). cargo-binstall comes from the pixi environment; nothing is compiled at bootstrap
and nothing is installed by a curl-pipe script.

Note, 2026-09-29: T22 left `tools.txt` as written and added `tools-source.txt` beside it,
a second pass that may compile, for rustqual alone (decision 0013).

Other pins this document fixes: pixi 0.81.0 (`requires-pixi` floors it in the manifest;
`setup-pixi`'s `pixi-version` names it exactly in CI), allium-tools 3.6.1 with B's
checksums, proptest 1.11.0, thiserror 2.0.17, serde 1.0.228 (§12 re-verifies each on the
day a ticket writes it). Every remote hook and every GitHub Action is pinned to a full
commit SHA with a version comment (§5, §10).

### Maintainer prerequisites (not ticket steps)

rustup installed (no `stable` default toolchain is needed: prek 0.5.3 builds its one
`language: rust` hook, ripsecrets, with the machine's rustup and the pinned toolchain,
as T03 found, and `install-hooks` runs lychee's warm under `RUSTUP_AUTO_INSTALL=0`);
`~/.cargo/bin` ahead of `~/.pixi/bin` on `PATH`, or
`pixi global uninstall rust`; `command -v cargo` printing `~/.cargo/bin/cargo`; pixi
0.81.0 or newer; just (`pixi global install just`; any just launches the recipes, because
the pinned one in the environment runs every nested call and CI; `pixi run --frozen just
<recipe>` is the escape hatch when no global just exists); gh. Not uv, not
cargo-binstall. T00's first verification is `rustup show active-toolchain` from this
repository printing `1.98.1`; an executing agent that finds otherwise runs
`rustup toolchain install` from the worktree, the one install T00 performs (§11), and
stops if the pin still does not print. An agent's tool shell may not source
`~/.cargo/env`; if `command -v cargo` prints `~/.pixi/bin/cargo`, every command is
prefixed `PATH="$HOME/.cargo/bin:$PATH"`.

## 3. Target repository tree (exact paths and owners)

Owner is the lane that writes the final content; T00 creates every path as a stub or in
final form so that `just check` is green on the first commit. No path has two owners.
`(frozen)` marks files no lane touches after T00 (§11).

```text
Cargo.toml                       T00 skeleton -> T02 final (workspace, lints, profiles)
Cargo.lock                       T00 (generated; only T02 adds a dependency)
rust-toolchain.toml              T00 (frozen)
rustfmt.toml  clippy.toml        T00 stub -> T02
taplo.toml  deny.toml            T00 stub -> T02
.config/nextest.toml             T00 stub -> T02
tools.txt                        T00 (frozen; one line)
crates/pawdoku/Cargo.toml        T00 skeleton -> T02
crates/pawdoku/README.md         T00 stub -> T10
crates/pawdoku/src/lib.rs        T00 (no_std, one documented, tested item) -> T02
crates/pawdoku/src/random.rs     T02 (the randomness boundary)
crates/pawdoku/tests/api_bounds.rs   T02
crates/pawdoku/tests/random.rs   T02
LICENSE                          T00 (Apache-2.0 text, final)
README.md  CHANGELOG.md  SECURITY.md   T00 stub -> T10
AGENTS.md                        T00 stub (six phrases, 300 words) -> T05
CLAUDE.md  .github/copilot-instructions.md   T00 (byte-identical to G; frozen)
.agents/skills/<14>/SKILL.md     T00 stub -> T05
.claude/skills/<14>/SKILL.md  .codex/skills/<14>/SKILL.md   T00 (fixed bridge body; frozen)
.claude/settings.json            T00 (frozen)
skills-lock.json  allium-skill-reference/   T05
Justfile                         T00 (frozen)
.pre-commit-config.yaml  .pre-commit-fix.yaml   T00 (frozen)
.editorconfig  .gitattributes  .gitignore   T00 -> T03
.markdownlint-cli2.jsonc  lychee.toml  _typos.toml   T00 -> T03
pyproject.toml                   T00 (the pixi manifest and B's table) -> T03 comments
pixi.lock                        T00 (generated by `pixi lock`; committed)
scripts/initialize.sh            T00 stub -> T03
.github/actions/setup/action.yml T00 stub -> T04
.github/workflows/ci.yml         T00 one-job stub -> T04
.github/workflows/audit.yml      T04
docs/manifest.yml  docs/README.md   T00 (frozen)
docs/project/*.md                T00 stub -> T07
docs/tutorials/*.md  docs/how-to/*.md   T00 stub -> T07
docs/explanation/*.md            T00 stub -> T08 (two pages T06)
docs/reference/*.md  docs/operations/*.md   T00 stub -> T08
docs/decisions/README.md  docs/decisions/0001..0011-*.md   T00 stub -> T09
docs/specs/{sudoku,solver,technique,reach,effort,lapse,human-solving}.allium   T00 verbatim from G -> T06
docs/specs/board.allium          T12 (from G at add73be7)
tickets/                         this directory
ai_tmp/  .tools/  .pixi/  target/   gitignored; never committed
```

## 4. `Justfile` (complete; frozen at T00)

Note, 2026-09-29: T22 reopened the gate list for `metrics`, gate 7, and gave
`install-tools` a second line over `tools-source.txt` (decision 0013); the text below is
T00's, and the live `Justfile` is the authority.

The text below is frozen at T00. D01 kept the Python toolchain (decision 0004) and D02
made pixi its installer (decision 0011), so `prek` and B's console scripts are binaries
on the recipe `PATH`, called by name, and the recipe list `check` runs lives in
`pyproject.toml`. No recipe runs through `pixi run`: a bare `pixi run` may rewrite
`pixi.lock`, it exports `CONDA_PREFIX` and `PIXI_*` around every cargo build, and it
runs the line through its own shell instead of `sh -eu`; the `PATH` export is the one
activation.

```just
set positional-arguments := true
set shell := ["sh", "-eu", "-c"]

# pixi owns every tool binary and the Python environment (.pixi/envs/default;
# pyproject.toml is the manifest, pixi.lock the pin). Tools conda-forge lacks
# (tools.txt) live in .tools/bin, and so does the Allium checker. Every recipe,
# and every hook that runs through a recipe, sees both first; cargo finds
# cargo-nextest and friends on PATH by name. Neither directory holds a cargo, so
# rustup's proxy stays first for the compiler (check-toolchain proves it).
export PATH := justfile_directory() / ".pixi" / "envs" / "default" / "bin" + ":" + justfile_directory() / ".tools" / "bin" + ":" + env("PATH")

# Line-coverage floor over crates/pawdoku/src/**. Lower the complexity, not the
# number.
coverage_floor := "90"

default:
    @just --list

# ------------------------------------------------------------------ setup ---

# One explicit first-run command. Never stages, commits, tags, or pushes.
initialize:
    sh scripts/initialize.sh

# The pin in rust-toolchain.toml is honoured only by rustup's cargo proxy. A
# cargo from pixi, Homebrew or a distribution ignores the file and builds with
# whatever it is, so the gate refuses it before anything else runs.
check-toolchain:
    command -v rustup >/dev/null || { printf '%s\n' 'rustup is not installed; see docs/how-to/develop-locally.md' >&2; exit 2; }
    case "$(command -v cargo)" in "${CARGO_HOME:-$HOME/.cargo}"/bin/cargo) ;; *) printf '%s\n' "cargo resolves to $(command -v cargo), not rustup's proxy" >&2; exit 2;; esac
    rustup show active-toolchain

# Installs the toolchain, components and targets rust-toolchain.toml names.
install-toolchain:
    rustup toolchain install

# Tools conda-forge lacks (tools.txt) into .tools/bin (network). cargo-binstall
# comes from the pixi environment.
install-tools:
    grep -v '^#' tools.txt | xargs cargo binstall --root .tools --no-confirm --locked --disable-strategies compile

# The Allium checker for docs/specs/, pinned and checksummed in the tooling
# package. Downloads over the network into .tools/bin, which Git ignores.
install-allium:
    bg-install-allium

# Both lockfiles, exactly as committed. Never rewrites either.
sync:
    cargo fetch --locked
    pixi install --frozen

lock:
    cargo update --workspace
    pixi lock

# Within the manifest's constraints only; every direct pin is exact, so this
# moves transitives (openssl, libcxx, the Python patch level). `pixi upgrade`
# rewrites the pins themselves and is not this recipe.
lock-upgrade:
    cargo update
    pixi update

# Offline, like every gate recipe: a cold Cargo cache fails rather than
# fetching the registry index (`just sync` fills it), and an offline solve on a
# stale lock still fails, which is the right answer. --dry-run because pixi
# would otherwise rewrite a stale lock from that restricted solve.
lock-check:
    cargo update --workspace --locked --offline
    pixi lock --check --offline --dry-run

# The shim, then every hook environment (seven clones; Go under prek's cache;
# the ripsecrets build with the machine's rustup and the pinned toolchain), so
# that `just check` never fetches. lychee is `language: script` and downloads
# its binary at first run, not at prepare, so it is run once here on one
# Markdown file; its exit status is lint's business, not this recipe's. Its
# checkout pins `stable`, and its script's cargo-binstall calls rustc, so
# without RUSTUP_AUTO_INSTALL=0 rustup would install that toolchain.
install-hooks:
    git rev-parse --is-inside-work-tree >/dev/null
    test -x .pixi/envs/default/bin/prek || { printf '%s\n' 'the pixi environment is not installed; run just initialize first' >&2; exit 2; }
    prek install --overwrite --hook-type=pre-commit --prepare-hooks
    prek prepare-hooks --config .pre-commit-fix.yaml
    -RUSTUP_AUTO_INSTALL=0 prek run lychee --files README.md

# ---------------------------------------------------------------- develop ---

build:
    cargo build --workspace --all-features --locked

# nextest runs the unit and integration tests; it cannot run doctests, so those
# follow through cargo test.
test:
    cargo nextest run --workspace --all-features --locked
    cargo test --doc --workspace --all-features --locked

test-doc:
    cargo test --doc --workspace --all-features --locked

# ----------------------------------------------------------------- format ---

format:
    cargo fmt --all
    taplo fmt

# The only recipe that modifies files. The fix config is run twice because a
# fixer's first pass may itself fail on what another fixer then repairs.
fix:
    -prek run --all-files --config .pre-commit-fix.yaml
    prek run --all-files --config .pre-commit-fix.yaml
    cargo clippy --workspace --all-targets --all-features --locked --fix --allow-dirty --allow-staged
    just lint

# ------------------------------------------------------------------ check ---

lint:
    prek run --all-files

fmt-check:
    cargo fmt --all --check

toml-check:
    taplo fmt --check
    taplo lint

# Every lint the manifests declare, as errors. The manifests say `warn` so a
# local build never breaks mid-edit; this is where warnings become failures.
clippy:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Every feature combination compiles. Not --no-dev-deps: that rewrites Cargo.toml
# while it runs and would trip the worktree snapshot on a crash.
features:
    cargo hack check --workspace --feature-powerset --locked

# The wasm-bindgen target, and a target with no std at all, which is what proves
# the core is no_std rather than merely happening to compile.
wasm-check:
    cargo hack check -p pawdoku --feature-powerset --target wasm32-unknown-unknown --locked
    cargo hack check -p pawdoku --feature-powerset --target wasm32v1-none --locked

# Runs every nextest test under instrumentation and enforces the floor. Doctests
# are not measured (nightly-only in llvm-cov); test-doc runs them uncovered.
# llvm-cov does not create the lcov file's directory, so the recipe does.
coverage:
    cargo llvm-cov nextest --workspace --all-features --locked --no-report
    cargo llvm-cov report --fail-under-lines {{coverage_floor}}
    mkdir -p target/llvm-cov
    cargo llvm-cov report --lcov --output-path target/llvm-cov/lcov.info

doc:
    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --workspace --no-deps --all-features --locked

# Licences, bans and sources are answerable offline once `just sync` has run.
deny:
    cargo deny --locked check licenses bans sources

# Advisories need the RustSec database over the network, so this sits outside
# `just check` for the same reason check-links-online does.
audit:
    cargo deny --locked check advisories

deps-unused:
    cargo shear

# --------------------------------------------------------------- documents ---

check-docs:
    prek run --all-files markdownlint-cli2 typos lychee
    bg-validate-docs

check-agents:
    bg-validate-agents

# The specifications, checked mechanically: every module must report an empty
# `diagnostics` array. The wrapper asserts that, because neither subcommand's
# exit code does. Waiver terms: docs/how-to/work-with-the-specs.md.
check-specs:
    bg-run-allium check

analyse-specs:
    bg-run-allium analyse

check-links-online:
    prek run --all-files --hook-stage manual lychee-online

# --------------------------------------------------------------- aggregate ---

check-clean baseline="":
    bg-project-check clean "$1"

# The complete gate: the recipes pyproject.toml lists, in order, with the
# worktree snapshotted between each, then check-clean.
check:
    bg-project-check run
```

**Gate order** (the `recipes` list in `pyproject.toml`), then `check-clean`:

1. `check-toolchain`
2. `lock-check`
3. `lint`
4. `fmt-check`
5. `toml-check`
6. `clippy`
7. `features`
8. `wasm-check`
9. `test-doc`
10. `coverage`
11. `doc`
12. `deny`
13. `deps-unused`
14. `check-docs`
15. `check-agents`
16. `check-specs`
17. `analyse-specs`

Items 4 and 5 also run inside `lint`; the duplication is the house pattern, so a
regression names itself in the step list rather than inside `lint`. `test`, `build`,
`audit` and `check-links-online` are outside `check`. No `.cargo/config.toml`: `RUSTFLAGS`
there changes fingerprints and thrashes `target/` against rust-analyzer; `RUSTDOCFLAGS` is
set only on the `doc` recipe.

## 5. Hooks

`.pre-commit-config.yaml` is the read-only gate and the one `just install-hooks`
installs; `.pre-commit-fix.yaml` is the mutating counterpart, never installed, run only by
`just fix`. Both are G's files at `78d03cdf` with the changes below, and both are frozen at
T00 (§11).

Changes to G's `.pre-commit-config.yaml`:

- `exclude` becomes `\.git/`, `\.pixi/`, `ai_tmp/`, `target/`, `\.tools/`,
  `allium-skill-reference/`, `Cargo\.lock$`, `pixi\.lock$`, with a comment that
  `pixi.lock` is YAML and the builtin `check-yaml` would otherwise parse it on every
  commit.
- The `ruff-check`, `ruff-format-check` and `eslint` local hooks are dropped.
- Three local hooks are added, each `language: system`, `pass_filenames: false`, with
  `entry: just <recipe>`: a git hook does not inherit the Justfile's `PATH`, so routing
  through `just` is what lets a hook find the environment's `taplo`, and it keeps each
  tool's arguments in exactly one place. `fmt-check` with
  `files: ^(crates/.*\.rs|rustfmt\.toml)$`; `toml-check` with `types: [toml]`;
  `deps-unused` with `files: ^(Cargo\.toml|crates/.*/Cargo\.toml|crates/.*\.rs)$`.
  Clippy is deliberately not a hook: a cold `cargo clippy --all-targets --all-features`
  blocks every commit for tens of seconds to minutes; it is gate 6 and a CI step, the
  same call G makes by keeping `svelte-check` out of the hook.
- `validate-docs`, `validate-agents`, `check-specs` and `analyse-specs` run through
  pixi as their activator: `entry: pixi run --frozen bg-validate-docs`,
  `pixi run --frozen bg-validate-agents`, `pixi run --frozen bg-run-allium check`,
  `pixi run --frozen bg-run-allium analyse` (`--frozen` because a bare `pixi run` may
  rewrite `pixi.lock` from inside a commit; pixi resolves the manifest upward from the
  hook's working directory, so the environment is the committing worktree's own; the
  recipe that could provide the `PATH` instead, `check-docs`, runs prek inside prek).
  `validate-docs` keeps its trigger; `validate-agents` adds `skills-lock\.json$` to its
  `files`; `check-specs` and `analyse-specs` keep `^(docs/specs/|pyproject\.toml$)`
  (the allium pin is the tooling package version, which `pyproject.toml` pins; since
  the pixi manifest lives in the same file, any pin edit now fires them, accepted).
- The `builtin` hygiene set is unchanged (`check-added-large-files --maxkb=768`,
  `check-case-conflict`, `check-executables-have-shebangs`, `check-json`,
  `check-merge-conflict`, `check-shebang-scripts-are-executable`, `check-toml`,
  `check-yaml`, `detect-private-key`).
- Remote hooks keep G's revs and comments verbatim except typos, which moves to v1.50.2:
  editorconfig-checker `675b1261a4d9668c357fcd67deb87aed8b881b94 # v3.11.1` with
  `types: [text]`; markdownlint-cli2 `b82a6c8896e491b9cb377a99ff3412131920681b # v0.23.2`;
  typos `512fc24f32f44ab01972217aaaf3dc86ec234d53 # v1.50.2` (resolved from the annotated
  tag on 2026-09-23); lychee `2bba271688c1abb1503097a064e6c3bc1d1b6a9b # lychee-v0.24.2`
  with `LYCHEE_VERSION=0.24.2` leading both argument lists and G's comment explaining why;
  shellcheck-py `745eface02aef23e168a8afb6b5737818efbea95 # v0.11.0.1`; actionlint
  `914e7df21a07ef503a81201c76d2b11c789d3fca # v1.7.12`; ripsecrets
  `7d94620933e79b8acaa0cd9e60e9864b07673d86 # v0.1.11` with entry
  `.pixi/envs/default/bin/bg-ripsecrets` (the one hook that receives filenames; git runs
  hooks from the worktree root, so the relative path resolves, and argv passes through
  untouched, which `pixi run` is not proven to do: §12).

Changes to G's `.pre-commit-fix.yaml`: the same `exclude`; `ruff-fix` and `ruff-format`
dropped; local `cargo-fmt` (`entry: cargo fmt --all`, `pass_filenames: false`,
`types: [rust]`), `taplo-fmt` (`entry: taplo fmt`, `types: [toml]`; the fix config runs
only through `just fix`, so `.pixi/envs/default/bin` is on its `PATH` and the file list
passes through) and `cargo-shear-fix` (`entry: cargo shear --fix`,
`pass_filenames: false`) added; builtin `end-of-file-fixer` and `trailing-whitespace` and
`markdownlint-cli2 --fix` unchanged. `cargo clippy --fix` stays in the `fix` recipe body
because it needs `--allow-dirty --allow-staged` and a full build.

Config files the hooks read, each G's with the named edits (T03 owns them):

- `.editorconfig`: G's, plus `[*.rs]` with `indent_size = 4`; `[*.py]` stays. TOML stays
  at the 2-space default, which is taplo's.
- `.gitattributes`: `* text=auto eol=lf`, `Cargo.lock linguist-generated=true`,
  `pixi.lock merge=binary linguist-language=YAML linguist-generated=true` (the shape
  pixi recommends for its lockfile; T03 confirms the wording),
  `allium-skill-reference/** linguist-vendored=true`.
- `.gitignore`: G's minus the Node, Svelte, Storybook and Chromatic blocks, plus
  `target/`, `*.profraw`, `lcov.info`, `mutants.out*/`, with G's `.tools/` comment kept
  and a comment on `target/` in the same voice; `.pixi/` in place of `.venv/`, with a
  comment in the same voice (the environment `just initialize` installs). `__pycache__/`
  and `*.py[cod]` stay; `.ruff_cache/` is T03's call.
- `.markdownlint-cli2.jsonc`: G's rules verbatim; `ignores` becomes `target`, `.tools`,
  `.pixi`, `ai_tmp`, `allium-skill-reference`.
- `lychee.toml`: G's settings; `exclude_path` becomes `.git`, `.pixi`, `ai_tmp`,
  `target`, `.tools`, `allium-skill-reference`.
- `taplo.toml` (T02's): `exclude = ["target/**", ".tools/**", ".pixi/**", "ai_tmp/**"]`,
  load-bearing because `toml-check` runs taplo over `**/*.toml` and the environment's
  `site-packages` contain TOML.
- `_typos.toml` at the root is the only typos configuration; `pyproject.toml` carries no
  `[tool.typos]` (§12 checks precedence). Contents: `[files] extend-exclude =
  ["Cargo.lock", "pixi.lock", ".pixi/", ".tools/", "target/", "ai_tmp/", "allium-skill-reference/"]`
  and `[default.extend-words] mis = "mis"` (G's allowlist for the hyphenated prefix).

## 6. Handbook

The Diátaxis handbook under `docs/`, governed by the documentation contract G ships
(`docs/reference/documentation-contract.md`): `docs/manifest.yml` is strict JSON with
`schema_version: 1`; every page's frontmatter equals its manifest entry; each topic has
one owning page; every page is reachable from `docs/README.md`; 40 words minimum; links
resolve with exact case. `docs/manifest.yml` and `docs/README.md` are final at T00 and
frozen. Tier A pages are rewritten from G's page of the same path; tier B pages are
shorter adaptations. Twenty-five pages plus the decision index and eleven records.

| Path | Tier | Owner | From G | Rewrite note |
| --- | --- | --- | --- | --- |
| `docs/README.md` | A | T00 | same | A map for a library: the seven modules; the game's root module stays in G; no Platform section; a closing "This library" section |
| `docs/project/purpose-and-scope.md` | A | T07 | same | What the engine is; what it is not (no UI, no persistence, no generation yet); who consumes it |
| `docs/project/repository-map.md` | A | T07 | same | The workspace tree of §3 |
| `docs/project/terminology.md` | B | T07 | same | Keep "The repository"; engine words from `sudoku.allium`'s vocabulary |
| `docs/tutorials/first-change.md` | A | T07 | same | Clone, `just initialize`, change a clause, derive a test, `just check` |
| `docs/how-to/develop-locally.md` | A | T07 | same | rustup and pixi prerequisites, the pixi-rust/`PATH` note, the environment's `bin`, offline first-run caveats |
| `docs/how-to/test-and-debug.md` | B | T07 | same | nextest filters, doctests, llvm-cov HTML, `RUST_BACKTRACE` |
| `docs/how-to/work-with-the-specs.md` | A | T07 | same | Module table = the seven engine modules; waiver terms verbatim (G lines 77-136, the "Diagnostics and waivers" section); `just frontend-unit` becomes `just test` |
| `docs/how-to/maintain-dependencies.md` | A | T07 | same | Caret ranges + `Cargo.lock`, the pixi pins + `pixi.lock`, `tools.txt`, hook SHAs, action SHAs, the allium pin; "nothing updates them" until S01 |
| `docs/explanation/architecture.md` | A | T08 | same | Crate layout, the randomness boundary, the future bindings crates, what stays out of the core |
| `docs/explanation/layering.md` | B | T08 | same | Module dependency direction mirrors the spec import graph |
| `docs/explanation/specifications.md` | A | T08 | same | "Seven are the library's"; the game's root module and platform figures are G's, held equal by test on G's side |
| `docs/explanation/quality-philosophy.md` | A | T08 | same | The argument verbatim; every frontend example (browser, story, bundle) replaced by the Rust gate's equivalent |
| `docs/explanation/security-model.md` | B | T08 | same | A library: supply chain, `forbid(unsafe_code)`, cargo-deny, no network at runtime |
| `docs/explanation/solving-sudoku.md` | A | T06 | same | Verbatim |
| `docs/explanation/human-solving.md` | A | T06 | same | Lines 16 and 255 reworded (§9) |
| `docs/reference/commands.md` | A | T08 | same | The recipe table of §4 |
| `docs/reference/configuration.md` | B | T08 | same | `rust-toolchain.toml`, `[workspace.lints]`, `deny.toml`, the floor, `pyproject.toml` as the pixi manifest, `tools.txt` |
| `docs/reference/testing.md` | A | T08 | same | nextest, doctests, coverage scope, fakes through the trait, in-module unit tests |
| `docs/reference/quality-gates.md` | A | T08 | same | The seventeen gates; allium paragraphs verbatim; no network gate |
| `docs/reference/documentation-contract.md` | A | T08 | same | Verbatim minus the managed/seed paragraph |
| `docs/reference/agent-contract.md` | A | T08 | same | Fourteen skills; `rust-change` as the example frontmatter; the reference-directory rule |
| `docs/reference/api.md` | A | T08 | new | rustdoc is the API reference: `just doc`, where it lands, the doctest policy |
| `docs/operations/maintenance.md` | B | T08 | same | No secrets; tool-pin rotation (`pixi update`); allium checksums |
| `docs/operations/troubleshooting.md` | A | T08 | same | pixi's rust shadowing rustup, a missing pixi environment or `.tools/bin/allium`, a stale `pixi.lock`, a `lint` that clones or downloads (a cleared prek cache, or a secondary worktree whose primary never ran `just initialize`: `just install-hooks` again), `--locked` failures |
| `docs/decisions/README.md` and eleven records | A | T09 | see §8 | |

Dropped from G: `project/platform.md`, `explanation/accessibility.md`,
`how-to/work-in-the-component-workshop.md`, `how-to/deploy-to-github-pages.md`,
`how-to/update-from-template.md`. The manifest keeps G's grouping, decisions last, then
the two migrated explanation pages. Every `canonical_for` slug is unique; carried pages
keep G's slugs.

## 7. Agent contract

`AGENTS.md` is canonical; `CLAUDE.md` is exactly `@AGENTS.md` and
`.github/copilot-instructions.md` is B's exact adapter text (`validate_agents.py`
`ADAPTERS`), both byte-identical to G's. `AGENTS.md` keeps G's section order (What this
project is, Invariants, Stack and conventions, Change workflow, Safety and authority,
Documentation and durable context, External automation policy, Provenance) with these
invariants in place of G's:

1. The specifications are the source of truth for behaviour (as G).
2. Effects sit behind traits. Randomness is the only effect the engine has, reached
   through the boundary `crates/pawdoku/src/random.rs` defines; a clock, threads, the
   filesystem and the environment are not merely avoided but unnameable, because the
   core is `#![no_std]`. Limits are step budgets, never timeouts.
3. Every public type is `Send + Sync + 'static`, `Clone` and `Debug`; public enums and
   structs are `#[non_exhaustive]`; errors are `thiserror` enums with stable `Display`
   text, because pyo3 and wasm-bindgen build their exceptions from it.
4. `#![forbid(unsafe_code)]` in every crate.
5. The core compiles for `wasm32-unknown-unknown` and `wasm32v1-none` under every
   feature combination; `just wasm-check` and `just features` prove it.
6. Clippy pedantic with the workspace lint table; `#[expect(lint, reason = "...")]` only,
   never a blanket `allow`.
7. Coverage does not fall below the floor: 90% of lines over `crates/pawdoku/src/**`.
8. `Cargo.lock` is the pin. Manifests write full `x.y.z` caret ranges; every gate passes
   `--locked`; `just lock-check` proves the lockfile matches. This is a stated deviation
   from the games' exact-pin invariant, recorded in decision 0007.

The file also carries, in Provenance, one honest sentence containing the word `runes`
("this repository has no Svelte runes; the games' reactivity rule does not apply here")
until C02 ships (decision 0004).

Skills, canonical under `.agents/skills/<name>/SKILL.md` with frontmatter of exactly
`name` and `description` (eight or more words stating a trigger), a body that cites
`AGENTS.md` and at least one `just` recipe, and a bridge in `.claude/skills/<name>/` and
`.codex/skills/<name>/` whose body is exactly B's `BRIDGE_BODY`:

| Skill | Source | Change |
| --- | --- | --- |
| `code-review` | G | Rust invariants in the checklist; `file:line` findings by severity |
| `fix-quality` | G | Gate names of §4 |
| `plan-change` | G | Unchanged in shape |
| `project-check` | G | Step 2 prerequisites (rustup, `just initialize`); gate numbers |
| `review-docs` | G | The managed/seed step dropped |
| `spec-change` | G | Steps 1, 4 and 8: `tests/platformSpecs.test.ts` and `just frontend-unit` become the boundary and `just test` |
| `rust-change` | new | Read `AGENTS.md` and `docs/explanation/architecture.md`; the trait boundary first; derive tests from the clause; doc examples are tests; `just fmt-check`, `just clippy`, `just test`, then `just check` |
| `allium`, `distill`, `elicit`, `propagate`, `tend`, `weed`, `witness` | G, vendored from `juxt/allium` | Five byte-for-byte; `allium/SKILL.md` line 10 (recipe names) and `propagate/SKILL.md` line 12 (`just frontend-unit`, "never colocated") edited; `skills-lock.json` copied verbatim as provenance (its hashes match no file in G and no validator reads it) |

`accessibility-review` and `svelte-change` are not carried. Fourteen skills, twenty-eight
bridges. `.claude/settings.json` is `{"enabledPlugins": {"allium@juxt-plugins": true}}`.
`allium-skill-reference/` lives at the root, markdownlint- and lychee-ignored, linguist-
vendored, with decision 0010 recording why. `just check-agents` prints
`Validated AGENTS.md, 2 adapters, and 14 skills.`

## 8. Decision records

A fresh series. A carried record opens "Carried from Pawdoku's decision NNNN at
`78d03cdf`" and keeps G's `canonical_for` slug (H `AGENTS.md` lines 242-243 is the
precedent). Format: frontmatter, `# Decision NNNN: title`, Context, Decision,
Consequences, What would reopen this, Related pages.

| Number | Title | Source | Slug |
| --- | --- | --- | --- |
| 0001 | The engine is a library of its own | new | `decision_engine_as_a_library` |
| 0002 | Specifications decide behaviour | G 0003 | `decision_spec_first` |
| 0003 | Effects behind traits | G 0002, adapted: randomness is the one effect | `decision_ports_and_fakes` |
| 0004 | Hook runner and checkers | D01's text (carried from G 0004) | `decision_python_toolchain` |
| 0005 | A project-managed Allium binary | G 0007, adapted: `allium-cli` 3.6.1 is on crates.io (verified 2026-09-23), so the third-toolchain objection G lines 42-45 raise lapses in a Rust repository; but a source build verifies nothing and upstream publishes no checksums for the platform binaries (G lines 54-59), so B's hand-computed checksums stay the pin | `decision_allium_cli` |
| 0006 | Apache-2.0 | new; no per-file headers | `decision_license` |
| 0007 | Dependency policy for a library | new: caret ranges, `Cargo.lock` committed, `--locked`, cargo-deny | `decision_dependency_policy` |
| 0008 | A pure `no_std` core | new: the constraints that keep bindings possible: no clock/threads/fs, the randomness trait, `Send + Sync`, `#[non_exhaustive]`, no panics, feature policy, the two wasm targets | `decision_no_std_core` |
| 0009 | The Rust quality gate | new: pedantic clippy, the floor, feature powerset, rustdoc warnings, in-module unit tests as a stated deviation from the games' "never colocated" rule | `decision_rust_gate` |
| 0010 | Skill reference material beside the skills | new: the record G's `189348e` deferred | `decision_skill_reference` |
| 0011 | Tool manager | D02's text: pixi owns the tools and the Python environment, rustup keeps the compiler; supersedes 0004's uv-specific consequences | `decision_tool_manager` |

## 9. Specification migration

The library boundary wording used everywhere: **the randomness boundary**, a trait the
library defines (`random.rs`): a stream of draws in `[0, 1)` begun from a seed, indexed
from zero, the same for the same seed and `random_version`. The caller chooses the
implementation; the library ships one named by `random_version` so that
`ExactReplay` holds across callers; a test supplies draws through the fake. This is the
contract `human-solving.allium` lines 1096-1099 already state; only "port" and "this
game" change.

T06 is a relevance review, not a grep. Before any edit it tabulates, per module, every
Includes and Excludes clause with a verdict **keep / adapt / drop** and a reason; the
known edits below are the adaptations already found, not the whole job. Borderline items
go to T06's Open points for the maintainer: human-solving's notes, highlighting and
feedback *environment* (a surface's concern or a solver input?), technique's marking
profile, hint wording in reach, effort's open figures, and anything that presumes a
single player on a single device.

| Module | Lines at `78d03cdf` | Known edit |
| --- | --- | --- |
| `sudoku.allium` | 36-37 | "pawdoku.allium's Play surface and the modules beneath it decide what a player is shown" becomes "a consumer's surfaces decide what a player is shown; nothing here draws" |
| | 40-41 | Dependencies: "None. No figure a consumer's root module states is read here; a consumer that draws the rules imports this module beside its own" |
| `solver.allium` | none | Verbatim; the Source path `docs/explanation/solving-sudoku.md` stays valid because the page moves |
| `technique.allium` | none | Verbatim |
| `reach.allium` | none | Verbatim ("a hint pitched at them" is engine output, not UI) |
| `effort.allium` | none | Verbatim; the open question at line 312 is carried |
| `lapse.allium` | 45-46 | "which this game reaches through its randomness port" becomes "which this library reaches through its randomness boundary" |
| `human-solving.allium` | 39-40 | "Draws come from this game's randomness port" becomes "Draws come from the library's randomness boundary: a caller supplies the stream" |
| | 476-478 | "Names the randomness port's seeded stream. The generator is the port's" becomes "Names the seeded stream the randomness boundary supplies. The generator is the library's, named here so a different one is a different name" |
| | 1096-1099 | "Draws are the randomness port's ... through the port's fake" becomes the boundary wording |
| | 1280 | Open question carried as-is |
| `docs/explanation/human-solving.md` | 16 | "no executable simulator or game surface exists yet" becomes "no executable simulator exists yet; a surface is the game's" |
| | 255 | "the game's randomness port, as every side effect does: the port" becomes the boundary wording |
| `docs/explanation/solving-sudoku.md` | none | Verbatim |
| `pawdoku.allium` | — | Stays in G; it is the only module that names `tests/platformSpecs.test.ts` |

Acceptance:
`grep -n -i "pawdoku\.allium\|this game\|randomness port\|src/lib\|platformSpecs" docs/specs/*.allium docs/explanation/*.md`
prints nothing; `just check-specs` and `just analyse-specs` print seven blocks with empty
`diagnostics` and `findings`; every header stays `-- allium: 3`; the binary is 3.6.1 with
B's checksums.

Hand-back to G, recorded in T06's notes and performed by a later G ticket, separately
authorised: delete the seven modules and the two explanation pages with their
`docs/manifest.yml` entries and `docs/README.md` lines; rewrite
`docs/how-to/work-with-the-specs.md`'s module table (one row), `explanation/specifications.md`
("Eight are the game's today" becomes one, and where the engine's live), `AGENTS.md`'s
deviations list and `pawdoku.allium`'s Scope and Dependencies (the rules are the engine's,
held equal by test, not imported); two decisions G owes ("The engine lives in libpawdoku"
and the `allium-skill-reference/` relocation). G's `[tool.typos] mis` allowlist serves two
files under `allium-skill-reference/`, not the modules, so it stays. How G consumes the
engine (the wasm package shipping `specs/`) is S04's.

## 10. CI

`.github/workflows/ci.yml`: `on: pull_request`, `push` to `['main']` (quoted, G's
comment), `workflow_dispatch`; `permissions: contents: read`; concurrency group
`ci-${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true`; `env:`
`CARGO_TERM_COLOR: always`, `CARGO_INCREMENTAL: '0'`, `CARGO_NET_RETRY: '10'`,
`RUST_BACKTRACE: '1'`. Five plain gate jobs on `ubuntu-latest`, each with an explicit
`name:`, each beginning with `actions/checkout` (`persist-credentials: false`) and the
composite setup action, then one `just` recipe per `run:` line; and a sixth job, `check`,
the one required check, which needs the five and runs nothing of its own:

| Job | Timeout | Recipes |
| --- | --- | --- |
| `rust` | 20 | `check-toolchain`, `lock-check`, `fmt-check`, `toml-check`, `clippy`, `features`, `test`, `doc`, `deps-unused` |
| `coverage` | 20 | `coverage`; upload `target/llvm-cov/lcov.info` with `actions/upload-artifact` (7-day retention) |
| `wasm` | 10 | `wasm-check` |
| `deny` | 10 | `deny` |
| `documents` | 15 | `lint`, `check-docs`, `check-agents`, `check-specs`, `analyse-specs` (needs `install-allium` in setup) |
| `check` | 5 | none; `needs` the five above, `if: always()`, one step that exits 1 when any `needs.*.result` is `failure`, `cancelled` or `skipped` |

`.github/workflows/audit.yml`: `pull_request`, a weekly `schedule`, `workflow_dispatch`;
one job `audit` running `just audit`; never a required check, because the RustSec fetch
can fail for reasons unrelated to the diff.

`.github/actions/setup/action.yml`, composite, input `cache-key`, in this order:
`prefix-dev/setup-pixi` with `pixi-version: v0.81.0` (the one version pin in YAML; it
must satisfy the manifest's `requires-pixi`), `manifest-path: pyproject.toml`,
`locked: true` (the CI lockfile check), `cache: true` (keyed on the `pixi.lock` hash),
`cache-write` on `main` only, `activate-environment: true` (the environment's `bin`, and
so `just`, on every later step's `PATH`; the environment holds no rust, so cargo stays
the runner's rustup proxy); `just install-toolchain` (rustup 1.29 is preinstalled on GitHub runners; there is no
fallback that reads `rust-toolchain.toml` for free: `dtolnay/rust-toolchain` makes its
`toolchain` input required (its `action.yml` lines 9-11 and 36-39 at `master`
`02cb101e`), so the fallback is the no-argument `rustup show`, a `main` follow-up because
the `Justfile` is frozen);
`Swatinem/rust-cache` with `shared-key: ${{ inputs.cache-key }}`, `save-if` on `main`
only, `cache-bin: false`; `actions/cache` on `.tools` keyed
`${{ runner.os }}-${{ runner.arch }}-tools-${{ hashFiles('tools.txt') }}` (what pixi
cannot own: cargo-hack, and allium as a side effect, which `bg-install-allium`
version-checks); `just install-tools` with `GITHUB_TOKEN: ${{ github.token }}` on that
step only; `just sync`; a step appending `PREK_HOME=${{ runner.temp }}/prek` to
`$GITHUB_ENV` (prek's cache outside the workspace, so `check-clean`'s snapshot never
sees it); `actions/cache` on that directory keyed
`${{ runner.os }}-${{ runner.arch }}-prek-${{ hashFiles('.pre-commit-config.yaml', '.pre-commit-fix.yaml', 'pixi.lock') }}`
(the hook clones, the Go, Node and rustup toolchains and the ripsecrets build, which
every gate job would otherwise fetch again); `just install-hooks` (the prepare and the
lychee warm are the point; the shim in the runner's `.git` is harmless), after which no
step reaches the network; for the `documents` job, `just install-allium`. No `setup-uv`,
no `install-action`.
Every `uses:` is pinned to a full SHA with a version comment looked up on the day T04 is
written (`gh api repos/<owner>/<repo>/git/ref/tags/<tag>`, dereferencing annotated tags).

Ubuntu only: the crate is pure computation with `eol=lf` forced by `.gitattributes` and
rustfmt; `windows-latest` joins `rust` the day `crates/pawdoku-cli` exists, and wheels get
maturin's own matrix in the release workflow S02 designs (which needs `contents: write`
and `id-token: write` for crates.io trusted publishing, so it never shares `ci.yml`).
The required check is `check`, from T01 onward and never anything else. T00's stub reports
it as the one job that runs `just check`; T04's workflow reports it as the aggregate job
above, which needs every gate job and fails when any of them did not succeed. Protection
therefore never changes when a job is added, renamed or moved into a reusable workflow
(C01): a new gate job joins `check`'s `needs` and nothing else moves. `audit` is never in
`needs`. Without this, T04's pull request and every lane pull request pushed after it
would report five checks and not the one `main` requires, and could merge only by an
administrator bypassing the gate.

## 11. Rules for tickets and lanes

- **Worktrees and branches.** Each ticket is executed on the branch its `branch:` field
  names (`ticket/<id>-<slug>`, lowercase, for every prefix: `ticket/d01-hook-runner`,
  `ticket/s02-release-and-publishing`) in its own worktree, created from `main` after
  every ticket it depends on has merged. From a Supacode terminal:
  `supacode repo worktree-new --branch <branch> --base main --name <id>`; otherwise
  `git worktree add ../<id> -b <branch> main`. A ticket touches only its listed files plus
  the `status:` line of its own `tickets/<id>-*.md`.
- **D01 and D02 before T00.** Both are done: T00 creates `pyproject.toml` (the content
  in D02's hand-back notes: the pixi manifest and B's table) and `pixi.lock`, and
  writes the `Justfile` as §4 prints it.
- **T00 ships a shape-complete skeleton.** `bg-validate-docs` and `bg-validate-agents`
  (or their ports) can only be green if every path in §3 exists from the first commit,
  so T00 ships a stub for every file: each registered page with valid frontmatter, a
  matching H1 and 40 or more words of real prose summarising what the page will say;
  `AGENTS.md` carrying the six required phrases and 300 words; all fourteen skills with
  descriptions that state a trigger and bodies that cite `AGENTS.md` and a `just`
  recipe, plus the twenty-eight fixed-body bridges; the seven modules copied verbatim
  from G; a `lib.rs` with one documented, tested item so coverage has a denominator.
  Lanes **replace** stubs; they never add, rename or reclassify a path.
- **Files no lane touches:** `Justfile`, `.pre-commit-config.yaml`,
  `.pre-commit-fix.yaml`, `rust-toolchain.toml`, `tools.txt`, `docs/manifest.yml`,
  `docs/README.md`, `CLAUDE.md`, `.github/copilot-instructions.md`, `.claude/settings.json`
  and every bridge. A lane that needs a change there (a recipe flag T02 finds wrong, a
  tool version, a page rename) stops and hands it back as a T00 follow-up pull request
  on `main`, because every other lane reads those files.
- **No dependency is added outside T02**, so `Cargo.lock` has one owner; a lane that
  needs a crate hands it back to T02 with the decision-0007 note the ADR asks for.
- **No path appears in two lane tickets' Files-touched lists.** An agent that finds a
  need to edit another lane's file hands it back instead.
- **Self-contained.** A ticket is written for an agent with no context: it embeds exact
  content or cites this document by section, and names the G, T, B, P or H source paths
  to read. It never cites a chat transcript, a scratch directory or a tool result.
- **Paths as code spans.** Ticket text writes repository paths as code spans, never as
  relative Markdown links: the hook gate runs lychee offline over `tickets/`, and a link
  to a file that does not exist yet fails it.
- **Definition of done**, for every build ticket: `just check` green in this repository,
  the ticket's own acceptance criteria met, the verification commands run with their
  output quoted in the hand-back notes, and the ticket's `status:` set to `done` in the
  same pull request.
- **Separately authorised actions.** Commits on the ticket branch are the ticket's work.
  Pushing, opening a pull request, tagging, filing GitHub issues, creating the repository,
  changing repository settings, publishing anywhere, installing anything outside the
  worktree (rustup, pixi itself, `pixi global` changes) and editing another repository (G, T,
  B, P, H) are each a separately authorised action: the ticket says where one occurs, and
  the agent stops and asks the maintainer rather than proceeding.
- **Scratch.** Anything an agent writes that is not a deliverable goes in `ai_tmp/`
  (gitignored from T00) or the session's scratch directory, never in a commit.
- **Credentials.** No file in this repository carries a token. CI uses the run's own
  `github.token`; crates.io publishing, when it comes, uses trusted publishing (OIDC).

## 12. Unverified claims, and the ticket that checks each

Each claim below was reasoned from source or documentation but not executed. The named
ticket runs the check and records the outcome in its hand-back notes; a claim that fails
is a design change that goes back through this document.

- `rustup toolchain install` with no arguments installs the toolchain, components and
  targets `rust-toolchain.toml` names (rustup 1.28 or later). Check: a fresh
  `RUSTUP_HOME` in `ai_tmp/`. Fallback: `rustup show`. **T00.**
- `wasm32v1-none` ships `rust-std` for 1.98.1 and `cargo hack check --target wasm32v1-none`
  passes on an `alloc`-only crate. **T00** (the stub crate), **T02.**
- `cargo update --workspace --locked` exits non-zero when `Cargo.toml` and `Cargo.lock`
  disagree, and zero otherwise. Fallback: `cargo metadata --locked --format-version 1`.
  **T02.**
- `cargo llvm-cov report --fail-under-lines 90` after `cargo llvm-cov nextest --no-report`
  enforces the floor, and `--doctests` is still nightly-only. **T02.**
- `cargo deny check licenses bans sources` runs offline once `cargo fetch` has populated
  the registry index, and `advisories` is the only network subcommand. **T02.**
- `cargo hack check --feature-powerset` on a crate with one optional feature (`serde`)
  produces two configurations and exits 0. **T02.**
- `cargo shear` reads source without a build and understands `[workspace.dependencies]`.
  **T02.** Outcome: cargo-shear 1.13.4 reports unused crate-level dependencies only, not
  an unused `[workspace.dependencies]` entry, which stays inert until a crate names it.
- `thiserror` 2 with `default-features = false` derives `core::error::Error` under
  `no_std`; `proptest` runs under `#[cfg(test)] extern crate std;`. **T02.**
- The `clippy.toml` keys `allow-panic-in-tests` and `allow-indexing-slicing-in-tests`
  exist under those names in clippy 1.98. **T02.**
- `cargo binstall --root .tools` puts binaries in `.tools/bin`, verifies release
  checksums where the crate publishes them, and skips a matching installed version (one
  tool, cargo-hack). **T03.** Outcome: cargo-hack publishes no checksum or signature, so
  nothing is verified; the binary comes unsigned over TLS from its GitHub release.
- `pixi install --locked` on this `pyproject.toml` installs the nine conda packages and
  B, and does not try to install `libpawdoku-tooling` itself (no `[build-system]`,
  `dependencies = []`). **T00.**
- pixi builds `biscuit-games-tooling` from its git tag with `uv_build` fetched at solve
  time, under the environment's Python 3.14; the six `bg-*` scripts land in
  `.pixi/envs/default/bin`. **T00.**
- Every pin in the manifest resolves with `pixi search <name> --platform linux-64`; the
  osx-arm64 search does not prove a linux-64 build. **T00.**
- `pixi lock --check --offline`, the `lock-check` recipe's own line, writes nothing and
  exits 0 when the lockfile is current, the git PyPI source included. Harness: the
  recipe after `just sync`, with the lockfile's hash recorded before and after. The one
  fallback: the pixi half of `lock-check` moves to CI only, where `setup-pixi`'s
  `locked: true` already checks it. **T03.** Outcome: true of a current lock, but a stale
  one that solves offline is rewritten, so the recipe's line gained `--dry-run`.
- After `just install-hooks` from an empty `PREK_HOME`, `just lint` is green with the
  network blocked (`HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9`), the
  lychee download included. Fallback: a hook that still fetches at run is named and gets
  its own warm line in the recipe. **T03.**
- `pixi run -x --frozen bg-ripsecrets` passes a filename containing a space and a single
  quote unchanged; if so, the ripsecrets entry may switch to it for uniformity. **T03.**
- `taplo lint` stays offline when no `[schema]` section and no `#:schema` directive exists.
  **T03.**
- typos stops at the first configuration it finds (`_typos.toml` before
  `pyproject.toml`), so a root `_typos.toml` is the only source. **T03.**
- prek 0.5.3 from conda-forge provisions its own rustup and toolchain under `$PREK_HOME`
  for `language: rust` hooks (0.4.12 does), so no `stable` default toolchain is required
  of the maintainer, and `prek install --prepare-hooks` does so at `install-hooks` time
  rather than at the first `lint`. **T03.** Outcome: prek 0.5.3 provisions no rustup
  when the machine has one; it builds with an installed toolchain, the pinned 1.98.1, so
  no `stable` default is needed for that reason instead. The second half of the original claim, that
  prek publishes release binaries cargo-binstall can resolve, was verified by **D01** on
  2026-09-24 and is recorded there as evidence only: prek is a pixi dependency here.
- B's checkers run unchanged in a repository with no `package.json`. **T00.**
- `bg-project-check` tolerates `target/`, `.tools/` and `.pixi/` growth because all
  three are ignored. **T00.**
- `rustup toolchain install` with no arguments works on the runner's rustup (1.29.1 on
  image 20260907.300.1) as it does locally. Not a fallback: `dtolnay/rust-toolchain`
  requires its `toolchain` input (verified 2026-09-23 by reading its `action.yml`), so it
  would restate the pin. **T04.**
- The activation variables `setup-pixi` exports (`CONDA_PREFIX`, `PIXI_*`) are inert for
  cargo; if a build script ever reacts to them, CI appends the environment's `bin` to
  `$GITHUB_PATH` instead of activating. **T04.**
- `setup-pixi` restores `.pixi/envs` and the package cache keyed on the `pixi.lock` hash,
  and `cache-write` gated to `main` behaves like rust-cache's `save-if`. **T04.**
- The prek cache under `${{ runner.temp }}/prek` restores on the second dispatch, and
  `just lint`'s log then shows no clone, download or build line. **T04.**
- `Swatinem/rust-cache` with `cache-bin: false` leaves `.tools/` to `actions/cache`. **T04.**
- The `check` aggregate job under `if: always()` reports `success` when its five `needs`
  succeed and `failure` when any of them failed, was cancelled or was skipped, and the
  checks API lists it under the name `check` (§10). The green half is proved by T04's
  dispatch; the red half only if a gate job fails during T04, otherwise recorded as not
  exercised. **T04.**
- allium 3.6.1 reports empty `diagnostics` and `findings` for the seven migrated modules
  after the §9 edits. **T06.**
- `T/scripts/bootstrap_repo.sh` applies cleanly to a repository with no Pages and one
  required check. **T01.**
- The `pawdoku` name is free on crates.io. **S02.**

## 13. Risks every ticket states where it applies

- **pixi's rust shadows rustup.** T00 cannot honour the pin until rustup is installed
  and ahead on `PATH`; a maintainer prerequisite, and T00's first verification. An
  agent's tool shell may not source `~/.cargo/env` (in the D02 session `command -v cargo`
  printed `~/.pixi/bin/cargo`), so every agent command prefixes
  `PATH="$HOME/.cargo/bin:$PATH"`, or the maintainer runs `pixi global uninstall rust`.
- **The pixi environment must never gain `rust`.** Adding it, or a tool that depends on
  it, puts a cargo first on every recipe's `PATH`; `check-toolchain`'s `case` line is
  the alarm, and the manifest comment says so.
- **One environment per worktree.** `pixi install --frozen` in each worktree (about
  150 MB of hardlinks from pixi's cache); not relocatable, because the `bg-*` shebangs
  name the worktree's absolute Python, so a renamed worktree needs `pixi install
  --frozen` again. The hook shim's primary-checkout rule in `scripts/initialize.sh`
  stays for the same reason.
- **`runes`.** `AGENTS.md` carries the honest sentence until C02 ships; `check-agents`
  must be green at T00.
- **Doctests and the small denominator.** nextest skips doctests and llvm-cov cannot
  measure them on stable, so the floor is a statement about unit and integration tests;
  on a tiny crate one untested branch in `random.rs` breaches 90%.
- **`--locked` and the snapshot.** Any recipe that rewrites `Cargo.lock` fails
  `check-clean`; T02 proves every recipe is read-only on a clean tree. `target/`,
  `.tools/` and `.pixi/` must be ignored before the first `just check`, and every pixi
  call in a gate is `--frozen` or `lock --check --dry-run` so none rewrites `pixi.lock`.
- **The network at first run, none in `check`.** Hook clones and their toolchains, the
  pixi environment, the cargo-hack binstall, the allium download, `cargo fetch` and
  cargo-deny's advisory database all need the network; `initialize` has it, `check` must
  not. The hook side is paid in `install-hooks` (`--prepare-hooks` and the lychee warm),
  the last line of the first-run path and a step of the CI setup action, so the first
  `lint` fetches nothing; T03 proves it with the network blocked, T00 does the same for
  the fresh-clone `check`. If `cargo deny check licenses bans sources` proves to need it,
  `deny` moves to CI only.
- **Exact pins versus a library.** `AGENTS.md` states decision 0007 instead of copying
  G's invariant 4.
- **Visibility.** Public versus private decides private vulnerability reporting, the
  SECURITY wording and whether C01's reusable workflow can be called without a grant.
  Open point in T01, decided before the first push.
- **The crates.io name.** `pawdoku` may be taken; S02 checks; not load-bearing until
  `publish = false` lifts, but a rename later touches every page.
- **Frozen files and follow-ups.** T02 and T06 are the lanes most likely to need a
  recipe or pin changed; each such change is a small `main` pull request, and T11
  reconciles what accumulated.
- **Allium version drift.** Waivers (none today) are valid only against 3.6.1; a moved
  pin re-verifies every module.
- **Shared truth across two repositories.** Until S04 ships the specs in the wasm
  package, G's restated clauses and this repository's modules can drift silently; T06's
  hand-back and `explanation/specifications.md` say so.
- **Action pins.** The toolchain is installed by `rustup` from the file, not by an
  action, so no action restates the pin; a runner image whose preinstalled cargo
  already equals the pin would mask a failed install, which is why `check-toolchain`'s
  active-toolchain line and the `wasm` job are the evidence, not a green `rust` job.
- **lychee and typos over `tickets/` and the vendored reference.** `allium-skill-reference/`
  is excluded from markdownlint, lychee and typos alike; `tickets/` is not, so ticket
  prose must pass typos and carry no relative links.
- **The 1.98.1 pin will be stale by the time a ticket runs.** A ticket that bumps it
  bumps `rust-version` with it and re-runs `just clippy`, because the lint set moves.
