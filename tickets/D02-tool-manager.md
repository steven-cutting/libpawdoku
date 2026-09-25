---
id: D02
title: "Tool manager: pixi owns the tools, rustup keeps the compiler"
status: done
depends_on: [D01]
parallel_with: []
branch: ticket/d02-tool-manager
estimated_size: M
---

# D02: Tool manager: pixi owns the tools, rustup keeps the compiler

## Context

CONVENTIONS.md §2 gave every tool binary one of three owners. rustup owns the compiler and
its components through `rust-toolchain.toml`. cargo-binstall owns six cargo-shaped tools
through `tools.txt`, fetching each from its GitHub release into the gitignored
`.tools/bin`. uv owns the Python side through `pyproject.toml`, `uv.lock` and
`.python-version`: `prek` and the six `bg-*` console scripts of `biscuit-games-tooling`
(B), each reached through `uv run --frozen`. A fresh machine therefore needs four
bootstrap installs before `just initialize` can run (rustup, cargo-binstall, just, uv),
and CI restates two of them in YAML (`taiki-e/install-action` for just and cargo-binstall,
`astral-sh/setup-uv` for uv).

T00 stopped on that design. Its hand-back notes record that `just install-tools` fails on
`taplo-cli@0.10.0`: the crate publishes no `[package.metadata.binstall]` table, its GitHub
release ships raw gzip binaries binstall cannot unpack, cargo-quickinstall has no 0.10.0
build, and `--disable-strategies compile` then refuses the source build on purpose. The
five other tools resolved only because binstall's default asset template happened to match
their release names; nothing pins that. The maintainer asked whether pixi
(`pixi.prefix.dev`, a conda-forge package manager with a lockfile) would make the tooling
easier, and asked for the answer as a decision with a record.

Facts the decision rests on, each verified on 2026-09-24:

- pixi 0.81.0 is the newest release (2026-09-15) and is installed on the maintainer's
  machine at `~/.pixi/bin/pixi`. `prefix-dev/setup-pixi` v0.10.2 (2026-08-28) installs it
  on a runner, caches the environment keyed on the lockfile's hash, and can put the
  environment's `bin` on every later step's `PATH`.
- `pixi search` on conda-forge: cargo-nextest 0.9.146, cargo-llvm-cov 0.9.1, cargo-deny
  0.20.2, taplo 0.10.0, prek 0.5.3 and cargo-binstall 1.23.0, each equal to the pin
  CONVENTIONS.md §2 already states; cargo-shear 1.13.4 (§2 pinned 1.14.0, which
  conda-forge does not carry); just 1.58.0 (CI pinned 1.51.0); python 3.14. None of these
  packages depends on conda-forge's `rust` package, so an environment holding them puts no
  `cargo` on `PATH`.
- Not on conda-forge: cargo-hack, ripsecrets, editorconfig-checker. cargo-hack is the
  `features` and `wasm-check` recipes' tool; the other two are prek-pinned hooks.
- conda-forge's `rust` 1.98.1 exists but ships neither the `wasm32v1-none` standard
  library nor `llvm-tools`, and only rustup's proxy honours `rust-toolchain.toml`. So pixi
  cannot own the compiler without giving up the `no_std` proof, the coverage floor and the
  single owner of the Rust pin.
- pixi accepts `pyproject.toml` as its manifest under `[tool.pixi.*]` tables, takes the
  workspace name from `[project] name`, supports a PyPI git dependency with a `tag`, and
  runs PyPI resolution through uv's library. B reads its configuration from
  `pyproject.toml` only (`_project.py` lines 44-50), so the file survives under every
  option and the `[tool.biscuit-games-tooling]` table is untouched.
- pixi's CLI: `pixi install --locked` aborts on a lockfile that disagrees with the
  manifest; `pixi install --frozen` installs the lockfile as it is; `pixi lock --check`
  exits non-zero when the lockfile would change; `pixi run --frozen <binary>` runs any
  binary of the environment; `pixi update <name>` moves one dependency. The environment
  lives in `.pixi/envs/default`, whose `bin` holds every tool and console script.
  `pixi search python --platform linux-64` lists 3.15.0rc2 first, so Python must be pinned
  `3.14.*`, not `>=3.14`.
- prek 0.5.3 from conda-forge accepts `prek install --overwrite` and
  `prek run --hook-stage manual`, the forms the frozen `Justfile` uses (hidden aliases of
  `--force` and `--stage`).
- B's `bg-install-allium` hard-codes `<root>/.tools/bin/allium` and `bg-project-check`
  snapshots the worktree between recipes, so `.tools/` stays gitignored and so must
  `.pixi/`.

Sources, read-only, at the commits CONVENTIONS.md §0 pins: `G/Justfile` and
`G/scripts/initialize.sh` (the shape T00's copies keep), `B/src/biscuit_games_tooling/_project.py`
lines 44-50 and `install_allium.py` (the two hard-coded paths above), D01's hand-back
notes (the file list this ticket replaces), and T00's hand-back notes on
`ticket/t00-foundation` (the taplo failure, verbatim).

## Goal

A decision, recorded in this ticket's hand-back notes as the full text of decision record
0011 and as the exact list of files T00, T03 and T04 create under it, so that T00 can be
rewritten and resumed. The decision is one of:

- **Option A: keep the design.** cargo-binstall and uv as today; taplo built from source
  by a second `install-tools` line (T00's own recommendation), or pinned at 0.9.3 through
  uv.
- **Option B: pixi owns the tools, uv keeps the Python side.** A pixi manifest lists every
  cargo-shaped tool, just and uv; `.tools/bin` keeps only what conda-forge lacks;
  `pyproject.toml`, `uv.lock`, `.python-version` and every `uv run --frozen` site stay.
- **Option C: pixi owns the tools and the Python environment, rustup keeps the
  compiler.** The manifest also pins Python, prek and B; `uv.lock`, `.venv/` and
  `.python-version` go; `prek` and the `bg-*` scripts land on the recipe `PATH` like
  every other tool, and `uv run --frozen` disappears. `rust-toolchain.toml`,
  `check-toolchain` and the components rustup installs are unchanged.
- **Option D: pixi owns everything, including rustc.** Rejected on the facts above: no
  `wasm32v1-none`, no `llvm-tools`, and a second owner of the Rust pin.

The recommendation this ticket starts from is **option C**. The maintainer set the house
style of the other Biscuit Games repositories aside for this question and asked which
option is better on its own terms.

## Non-goals

- No file outside `tickets/` is created or changed by the decision commit. The files are
  created by T00, T03 and T04; record 0011 is written by T09 from this ticket's text.
- No tool is installed and no lockfile is generated here; every version above comes from
  `pixi search`, a feedstock or a release page, and T00 re-verifies each with
  `pixi search <name> --platform linux-64` on the day it writes the manifest.
- No change to rustup's ownership of the compiler. Option D is recorded and closed.
- No change to which tools prek pins (typos, lychee, shellcheck, actionlint,
  markdownlint-cli2, editorconfig-checker, ripsecrets). One owner per pin holds, and two
  of them are not on conda-forge anyway.
- No change to B and no change to D01's record 0004 text; 0011 says what of it stands.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/D02-tool-manager.md` | ticket | the decision, the record text and the file list, in Hand-back notes; `status: done` |

## Steps

1. Create the worktree on `ticket/d02-tool-manager` from `main` (README.md "How to pick
   up a ticket"). There is nothing to run yet; `just` does not exist on `main`.

2. Read T00's hand-back notes on `ticket/t00-foundation` for the taplo failure and the
   state of the worktree, and D01's hand-back notes for the file list this ticket
   replaces.

3. Verify the facts in Context: `pixi search <name>` for each tool, the conda-forge
   `rust` feedstock's outputs, taplo's release assets, pixi's manifest and CLI reference,
   and `prek 0.5.3 --help` for the two flag aliases.

4. Weigh the four options against D01's criteria plus two of this ticket's, in a table in
   the hand-back notes: fidelity to the house gate; onboarding (what a fresh machine
   installs before `just initialize`); CI cost; what is lost; drift (how a pin proves
   itself); **one owner per pin** (§2's rule, which option D breaks and options A and B
   keep only by exception); **what a Rust contributor expects** (`cargo build` with plain
   rustup must keep working, and `rust-toolchain.toml` must stay the signal rust-analyzer
   and plain cargo read).

5. Decide. Write decision record 0011 in the house shape (CONVENTIONS.md §8), slug
   `decision_tool_manager`, file `docs/decisions/0011-tool-manager.md`, and say which
   consequences of record 0004 it supersedes and which stand.

6. Write the file list for T00, T03 and T04: every path whose content changes, with the
   exact text T00 writes where this ticket decides it (the `[tool.pixi.*]` tables, the
   one-line `tools.txt`, the `Justfile` changes, `scripts/initialize.sh`, the hook entries,
   the exclusion lists, the composite action) and the §12 claims and §13 risks that
   replace the binstall and uv ones.

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- The hand-back notes name exactly one option and give the reason against every
  criterion in step 4.
- The full text of decision record 0011 is in the hand-back notes, follows CONVENTIONS.md
  §8's shape, and names what it supersedes in 0004.
- The file list is exact: every path whose content changes under the chosen option is
  listed with its owner, and every file whose content this ticket decides has that
  content written out.
- The T00 taplo blocker is resolved by the decision, and the notes say how.
- Nothing outside this file changed in the decision commit: `git status --porcelain` on
  the ticket branch lists only `tickets/D02-tool-manager.md`.

## Verification

```sh
pixi --version
for p in cargo-nextest cargo-llvm-cov cargo-deny cargo-shear taplo prek cargo-binstall just python cargo-hack; do
  printf '%s: ' "$p"; pixi search "$p" --platform linux-64 2>&1 | grep -E '^(Version|Name)|No packages' | tr '\n' ' '; echo
done
git -C /Users/scutting/.supacode/repos/libpawdoku/T00-foundation show ticket/t00-foundation:tickets/T00-foundation.md | grep -n 'Fallback to cargo-install is disabled'
git status --porcelain
```

Expected: `pixi 0.81.0`; each tool line prints the version in Context, and cargo-hack
prints `No packages found`; the grep prints the line of T00's failure transcript; `git
status --porcelain` prints one line naming this file.

## Hand-back notes

Executed on 2026-09-24 on the branch `ticket/d02-tool-manager`, in the Supacode worktree
`D02`. Nothing was installed; no clone was modified; the T00 worktree was read and not
changed.

### What was verified, and how

- `pixi --version` prints `pixi 0.81.0`. `prefix-dev/pixi` releases list v0.81.0
  (2026-09-15) as the newest; `prefix-dev/setup-pixi` releases list v0.10.2 (2026-08-28).
- `pixi search` on the maintainer's machine (osx-arm64 by default; T00 repeats it with
  `--platform linux-64`), 2026-09-24:

  ```text
  cargo-hack: No packages found matching 'cargo-hack'
  cargo-shear: 1.13.4
  cargo-llvm-cov: 0.9.1
  cargo-nextest: 0.9.146
  cargo-deny: 0.20.2
  taplo: 0.10.0
  prek: 0.5.3
  just: 1.58.0
  rust: 1.98.1
  cargo-binstall: 1.23.0
  ripsecrets: No packages found matching 'ripsecrets'
  editorconfig-checker: No packages found matching 'editorconfig-checker'
  ```

  Also present, not used: uv 0.12.18, lychee 0.24.2, typos 1.50.2, shellcheck 0.11.0,
  actionlint 1.7.12, markdownlint-cli2 0.23.3.
- The Verification block's loop, `--platform linux-64`, run after the propagation commit
  (2026-09-25 UTC, still 2026-09-24 locally):

  ```text
  cargo-nextest: 0.9.146
  cargo-llvm-cov: 0.9.1
  cargo-deny: 0.20.2
  cargo-shear: 1.13.4
  taplo: 0.10.0
  prek: 0.5.3
  cargo-binstall: 1.23.0
  just: 1.58.0
  python: 3.15.0rc2 (first hit; the manifest pins 3.14.*, which the search also lists)
  cargo-hack: No packages found matching 'cargo-hack'
  ```

  Every pin the manifest names has a linux-64 build at the pinned version, so T00's
  re-check is a formality.
- The conda-forge `rust` feedstock's `recipe/meta.yaml` at `main`: version 1.98.1;
  outputs `rust`, `rust-src`, `rust-docs` and `rust-std-<target>` for a list that includes
  `wasm32-unknown-unknown` and `wasm32-unknown-emscripten` and does not include
  `wasm32v1-none`; no `llvm-tools` output. The `rustup` feedstock does not exist (404).
- Taplo: `tamasfe/taplo` release `0.10.0` (2025-05-23) ships `taplo-darwin-aarch64.gz`
  and siblings; `crates/taplo-cli/Cargo.toml` at `master` has no
  `[package.metadata.binstall]` table; `cargo-bins/cargo-quickinstall` has no
  `taplo-cli-0.10.0-*` release. crates.io's newest `taplo-cli` is 0.10.0. So the pin was
  never stale; binstall simply cannot reach it.
- pixi's manifest reference: `pyproject.toml` carries the same tables as `pixi.toml`
  under `[tool.pixi.*]`; `[project] name` is the workspace name; `requires-python` is
  mapped to a Python dependency; `[pypi-dependencies]` accept
  `{ git = "...", tag = "..." }`; `requires-pixi` in the workspace table is a version floor
  for pixi itself. Its CLI reference: the `--locked`, `--frozen` and `--check` semantics
  in Context. `pixi init` on an existing `pyproject.toml` adds the project as an editable
  PyPI dependency, which this virtual project (no `[build-system]`) cannot satisfy, so
  T00 writes the tables by hand and never runs `pixi init`.
- `pixi search <tool>` dependency lists: taplo pulls `openssl`, cargo-binstall pulls
  `libcxx`; the rest depend on virtual packages and `libgcc` only. None pulls `rust`.
- prek 0.5.3 (`.venv/bin/prek` in the T00 worktree): `prek install --overwrite` and
  `prek run --hook-stage manual` are accepted although `--help` shows `--force` and
  `--stage`.
- B at `v0.3.0`: `install_allium.py` resolves `<root>/.tools/bin/allium`;
  `run_project_check.py` runs `subprocess.run(["just", recipe])` per recipe and hashes the
  worktree between them; `_project.py` reads `[tool.biscuit-games-tooling]` from
  `pyproject.toml` only.
- The maintainer's machine, from the agent's shell: `command -v cargo` printed
  `/Users/scutting/.pixi/bin/cargo` (conda-forge rust 1.96.0 exposed by `pixi global`);
  `~/.cargo/bin` held rustup and cargo-binstall (installed by T00) but was not on the
  shell's `PATH`. `~/.pixi/manifests/pixi-global.toml` exposes `cargo`, `rustc`, `rustfmt`,
  `cargo-clippy`, `just`, `uv` and others. The §13 risk "pixi shadows rustup" is real and
  stays.

### The comparison

| Criterion | A: keep binstall and uv | B: pixi owns tools, uv stays | C: pixi owns tools and Python, rustup keeps the compiler | D: pixi owns everything |
| --- | --- | --- | --- | --- |
| Fidelity to the house gate | Identical recipes; taplo needs a source build or a downgrade. | Recipes identical; `uv run --frozen` everywhere. | The same seventeen recipes, the same six `bg-*` scripts at the same tag, the same runner; only the prefix `uv run --frozen` goes, because the scripts are on the recipe `PATH`. | Recipes identical; `check-toolchain` must be removed or inverted. |
| Onboarding | rustup, cargo-binstall, just, gh, uv (five), and cargo-binstall compiles from source on first run. | rustup, pixi, just, gh (four); uv comes from pixi. | rustup, pixi, just, gh (four); just also comes from pixi for CI and nested calls, so a global just of any version launches it. | pixi, just, gh (three); the compiler is conda-forge's. |
| CI cost | `taiki-e/install-action`, rust-cache, `actions/cache` on `.tools`, `setup-uv`, `uv python install`: five steps, three caches. | `setup-pixi` (cached on `pixi.lock`), rust-cache, `.tools` cache, `setup-uv`: four steps, four caches. | `setup-pixi`, rust-cache, `.tools` cache (cargo-hack and allium): three steps, three caches; one action pin fewer. | As C minus rust-cache's toolchain half. |
| What is lost | Nothing, at the price of one compiled tool or one downgraded one. | Nothing. | `uv.lock` and `.python-version`; both are replaced by `pixi.lock` and a `python = "3.14.*"` pin in the same manifest. | `rust-toolchain.toml` as the pin, the `no_std` proof against `wasm32v1-none`, and llvm-tools for the coverage floor. |
| Drift | `tools.txt` names versions, not hashes; binstall verifies checksums only where a crate publishes them; `uv lock --check` proves the Python side. | `pixi.lock` records every tool by hash and `pixi lock --check` proves it; `uv lock --check` still needed. | One lockfile proves every tool, Python, prek and B; `pixi lock --check` is the whole check. cargo-shear follows conda-forge (1.13.4 today), not crates.io. | As C, plus the compiler lagging conda-forge's build. |
| One owner per pin | Kept, by the `tools.txt` header. | Broken: the same pin list is split by "which installer can reach it", and uv still pins prek. | Kept: pixi owns every tool binary and the Python side; `tools.txt` is the documented exception for what conda-forge lacks (cargo-hack), and prek's hooks keep theirs. | Broken: `rust-toolchain.toml` and the pixi manifest would both name a Rust version. |
| What a Rust contributor expects | Standard. | Standard; pixi is unusual for a Rust library but only the gate needs it. | Standard for the crate: `cargo build` with rustup alone works, rust-analyzer reads the toolchain file; pixi is needed for the gate only. | `cargo` must come from the pixi environment; contributors with rustup are refused or silently use the wrong build. |

Absent the house style, C is better than B: it removes a second lockfile, a second
install step, a second CI action and twenty `uv run --frozen` prefixes, and it does not
touch the checkers themselves. A keeps a fragile installer for the sake of nothing. D
loses two invariants.

### The decision, and decision record 0011

**Option C: pixi owns the tools and the Python environment; rustup keeps the compiler.**
It resolves the T00 blocker (taplo 0.10.0 is on conda-forge), cuts the per-machine
bootstrap from four installs to two plus just, replaces a version list with a hashed
lockfile, keeps every recipe and every checker as it was, and keeps `rust-toolchain.toml`
as the only owner of the Rust pin. cargo-hack is not on conda-forge, so cargo-binstall
stays as a pixi dependency and `tools.txt` keeps one line; the day cargo-hack lands on
conda-forge, the line moves and `tools.txt` empties.

Decision record 0011, the full text T09 writes to `docs/decisions/0011-tool-manager.md`.
It is in a fenced block so the relative links in Related pages are verbatim text here,
not links the offline lychee run over `tickets/` would follow.

```markdown
---
title: "Decision 0011: Tool manager"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_tool_manager]
requires: []
---

# Decision 0011: Tool manager

## Context

Decision 0004 keeps the Python hook runner and the shared checkers, and its consequences
named the installers of the day: uv for the Python side, cargo-binstall for the cargo
tools, each with its own lockfile or pin list, beside rustup for the compiler. A fresh
machine installed four tools before the first-run script could start, and the tool pin
list named versions rather than hashes, trusting each tool's GitHub release naming.

The foundation ticket stopped on that trust. taplo 0.10.0 publishes no binstall metadata
and its release archives are in a shape binstall cannot unpack, so the pinned, current
version could not be installed without compiling it, which the install command refuses by
design. The other five tools installed only because binstall's default naming guess
matched their releases.

pixi resolves the same tools from conda-forge, where every one of them except cargo-hack
is packaged at the pinned version, and records the whole environment, Python and the
checkers included, in one hashed lockfile. conda-forge also packages the Rust compiler,
but without the `wasm32v1-none` standard library that proves the core is `no_std`, and
without the llvm tools the coverage floor needs; and only rustup's cargo proxy honours
`rust-toolchain.toml`.

## Decision

pixi owns every tool binary and the Python environment. `pyproject.toml` is the manifest:
its `[tool.pixi.*]` tables pin Python, just, prek, cargo-binstall, cargo-nextest,
cargo-llvm-cov, cargo-deny, cargo-shear and taplo exactly, and pin `biscuit-games-tooling`
as a PyPI git dependency at a release tag. `pixi.lock` is the pin; `pixi install --locked`
is the first thing the first-run script does; `pixi lock --check` is the gate's lockfile
check; the environment lives in the gitignored `.pixi/`. The `Justfile` puts the
environment's `bin` first on every recipe's `PATH`, then `.tools/bin`, and calls every
tool by name; nothing runs through `pixi run` inside a recipe.

rustup keeps the compiler. `rust-toolchain.toml` remains the single owner of the Rust
version, its components and its targets; the pixi environment holds no `rust` package,
and `check-toolchain` keeps refusing any cargo that is not rustup's proxy.

`tools.txt` remains as the escape hatch for a tool conda-forge does not carry. It lists
`cargo-hack` alone; cargo-binstall, itself a pixi dependency, installs it into
`.tools/bin`, beside the Allium checker the tooling package puts there. The hooks that
prek pins (typos, lychee, shellcheck, actionlint, markdownlint-cli2, editorconfig-checker,
ripsecrets) keep their pins in the hook configuration.

## Consequences

Of decision 0004's consequences, these are superseded: uv is not a prerequisite and not a
resolver here; `uv.lock`, `.venv/` and `.python-version` do not exist; CI carries no
`setup-uv` step; no recipe begins with `uv run --frozen`. These stand: the hook runner is
prek, the checkers are the six console scripts of `biscuit-games-tooling` at a pinned
tag, `pyproject.toml` is where the package reads its configuration, and the interim
`runes` sentence in `AGENTS.md` waits for the same follow-up.

Contributors need rustup, pixi and just (any just launches the recipes; the pinned one in
the environment runs every nested call and CI). CI installs pixi with one action, restores
the environment from a cache keyed on the lockfile's hash, and needs one more cache only
for what pixi cannot own. Every pin except the compiler's is a hash in `pixi.lock`, and a
moved pin is one `pixi update` and a committed lockfile.

The cost is a package manager that Rust contributors do not usually carry, needed for
the gate and not for `cargo build`; one environment per worktree, not relocatable; a
version that conda-forge lags on occasionally (cargo-shear is one minor version behind
crates.io today); and cargo-hack still arriving by a second mechanism until conda-forge
packages it.

## What would reopen this

cargo-hack appearing on conda-forge, which empties `tools.txt` and removes cargo-binstall;
that is a pin move, not a new decision. conda-forge shipping the `wasm32v1-none` standard
library and llvm-tools for its Rust package, which would make a pixi-owned compiler
possible and would reopen the question of who owns the Rust pin. pixi's PyPI git
resolution failing for the tooling package, which would return the Python side to uv
without moving the tools. A house-wide move of every Biscuit Games repository to pixi,
which would make the manifest shape a template concern.

## Related pages

- [Decision 0004: Hook runner and checkers](0004-hook-runner-and-checkers.md)
- [Develop locally](../how-to/develop-locally.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
```

### Files T00, T03 and T04 create

Every path whose content this decision changes. Unchanged paths are not repeated.
Everything below is written by T00 unless the Owner column says otherwise; the exact
texts here are what CONVENTIONS.md §2, §4, §5 and §10 now print, so a ticket that cites
those sections gets the same text.

| Path | Owner | Change |
| --- | --- | --- |
| `pyproject.toml` | T00 -> T03 | the `[tool.pixi.*]` tables below; `[dependency-groups]` and `[tool.uv]` gone |
| `pixi.lock` | T00 | **create** with `pixi lock` after the manifest exists (network); committed; `linguist-generated` |
| `uv.lock`, `.python-version` | T00 | **omit** (removed from the T00 worktree) |
| `tools.txt` | T00 (frozen) | one line, text below |
| `Justfile` | T00 (frozen) | the `PATH` export and the recipes below |
| `scripts/initialize.sh` | T00 -> T03 | the sequence below |
| `scripts/install_tools.sh` | T03 | **omit**: nothing compiles at bootstrap any more, and `just install-tools` is the re-runnable install; T03's step for it goes |
| `.pre-commit-config.yaml`, `.pre-commit-fix.yaml` | T00 (frozen) | the `exclude` lists and the five script-hook entries below |
| `.gitignore`, `.gitattributes`, `_typos.toml`, `lychee.toml`, `taplo.toml`, `.markdownlint-cli2.jsonc` | T00 -> T03 (`taplo.toml` -> T02) | `.venv` and `uv.lock` out, `.pixi` and `pixi.lock` in, texts below |
| `.github/actions/setup/action.yml` | T00 stub -> T04 | the steps below |
| `docs/manifest.yml`, `docs/decisions/README.md`, `docs/decisions/0011-tool-manager.md` | T00 stub -> T09 | an eleventh record: manifest row `Decision 0011: Tool manager`, kind `decision`, audience `[maintainer, agent]`, slug `decision_tool_manager`; the index table gets an eleventh row and its numbering sentence says the next decision is 0012 |

**`pyproject.toml`**, the content T00 writes. `[project]` and `[tool.biscuit-games-tooling]`
are D01's, unchanged. No `[tool.pixi.tasks]`: the `Justfile` is the only task runner, so
`pixi run <name>` always names a binary of the environment and never a task.

```toml
[project]
name = "libpawdoku-tooling"
version = "0.1.0"
description = "Repository tooling for libpawdoku. Not the library; see Cargo.toml for that."
requires-python = ">=3.14"
dependencies = []

# pixi owns every tool binary and the Python environment (.pixi/envs/default,
# gitignored; pixi.lock is the pin). rustup owns the compiler: rust is deliberately
# absent here, so the cargo first on a recipe's PATH is still rustup's proxy and
# rust-toolchain.toml is honoured (check-toolchain proves it).
[tool.pixi.workspace]
channels = ["conda-forge"]
# CI is linux-64; the maintainer is osx-arm64. osx-64 joins when a claim verifies it;
# win-64 never: bg-install-allium and the Justfile are POSIX-only.
platforms = ["linux-64", "osx-arm64"]
requires-pixi = ">=0.81.0"

[tool.pixi.dependencies]
# Exact pins. An unqualified "3.14" would let the linux-64 solve pick 3.15.0rc2.
python = "3.14.*"
just = "==1.58.0"
prek = "==0.5.3"
# For tools.txt only: cargo-hack is not on conda-forge.
cargo-binstall = "==1.23.0"
cargo-nextest = "==0.9.146"
cargo-llvm-cov = "==0.9.1"
cargo-deny = "==0.20.2"
# conda-forge's newest on 2026-09-24; the pin follows conda-forge, not crates.io.
cargo-shear = "==1.13.4"
taplo = "==0.10.0"

[tool.pixi.pypi-dependencies]
biscuit-games-tooling = { git = "https://github.com/steven-cutting/biscuit_games_tooling", tag = "v0.3.0" }

[tool.biscuit-games-tooling]
recipes = [
  "check-toolchain", "lock-check", "lint", "fmt-check", "toml-check", "clippy",
  "features", "wasm-check", "test-doc", "coverage", "doc", "deny", "deps-unused",
  "check-docs", "check-agents", "check-specs", "analyse-specs",
]
predicates = {}
```

Why `[dependency-groups]` is deleted rather than kept: pixi maps a dependency group to a
feature that belongs to no environment, so prek and B would silently drop out of the
default environment. Why not `pixi init`: on an existing `pyproject.toml` it adds the
project itself as an editable PyPI dependency, and a virtual project with no
`[build-system]` cannot be installed. `prek` comes from conda-forge, not PyPI: the same
0.5.3, as a native binary with no Python wrapper.

**`tools.txt`**, exact:

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

**`Justfile`**: the `PATH` export and its comment become

```just
# pixi owns every tool binary and the Python environment (.pixi/envs/default;
# pyproject.toml is the manifest, pixi.lock the pin). Tools conda-forge lacks
# (tools.txt) live in .tools/bin, and so does the Allium checker. Every recipe,
# and every hook that runs through a recipe, sees both first; cargo finds
# cargo-nextest and friends on PATH by name. Neither directory holds a cargo, so
# rustup's proxy stays first for the compiler (check-toolchain proves it).
export PATH := justfile_directory() / ".pixi" / "envs" / "default" / "bin" + ":" + justfile_directory() / ".tools" / "bin" + ":" + env("PATH")
```

and these recipes change (every other recipe, `check-toolchain` included, keeps §4's
text). Recipes call binaries directly, never through `pixi run`: a bare `pixi run` may
rewrite `pixi.lock`, which `check-clean` would report; it exports `CONDA_PREFIX` and
`PIXI_*` around every cargo build; and it runs the command line through its own shell
instead of `sh -eu`. The `PATH` export is the one activation.

```just
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

# Offline, like every gate recipe: an offline solve on a stale lock still
# fails, which is the right answer.
lock-check:
    cargo update --workspace --locked
    pixi lock --check --offline

# The shim, then every hook environment (seven clones; Go, Node and a rustup
# toolchain under prek's cache; the ripsecrets build), so that `just check`
# never fetches. lychee is `language: script` and downloads its binary at first
# run, not at prepare, so it is run once here on one Markdown file; its exit
# status is lint's business, not this recipe's.
install-hooks:
    git rev-parse --is-inside-work-tree >/dev/null
    test -x .pixi/envs/default/bin/prek || { printf '%s\n' 'the pixi environment is not installed; run just initialize first' >&2; exit 2; }
    prek install --overwrite --hook-type=pre-commit --prepare-hooks
    prek prepare-hooks --config .pre-commit-fix.yaml
    -prek run lychee --files README.md

# The only recipe that modifies files. The fix config is run twice because a
# fixer's first pass may itself fail on what another fixer then repairs.
fix:
    -prek run --all-files --config .pre-commit-fix.yaml
    prek run --all-files --config .pre-commit-fix.yaml
    cargo clippy --workspace --all-targets --all-features --locked --fix --allow-dirty --allow-staged
    just lint

lint:
    prek run --all-files

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

check-clean baseline="":
    bg-project-check clean "$1"

# The complete gate: the recipes pyproject.toml lists, in order, with the
# worktree snapshotted between each, then check-clean.
check:
    bg-project-check run
```

**`scripts/initialize.sh`**, the sequence under `set -eu`, G's shape: `pixi install
--locked` first (every later recipe needs prek, cargo-binstall or a `bg-*` script;
`--locked` refuses a lockfile that disagrees with the manifest, so a stale lock is fixed
by `just lock` and committed, never rewritten silently by a first run); `just
install-toolchain`; `just check-toolchain`; `just install-tools`; `just install-allium`;
`test -f Cargo.lock || cargo generate-lockfile`; `just sync`; `just format`; then G's
worktree-aware `install-hooks` block, whose comment now names
`.pixi/envs/default/bin/prek` as the absolute path the shim records and says that the
recipe also provisions every hook environment, so that this is the last line of the
first-run path that reaches the network and `just check` never does; and the closing
lines. prek's cache is per user (`prek cache dir`) and shared by every worktree, so a
secondary worktree that skips the block is warm as long as the primary checkout ran
`just initialize` once; the other case is a troubleshooting entry (T08). The
cargo-binstall compile line and G's `uv lock` line are gone.

**Hooks.** In both prek configs the `exclude` list becomes `\.git/`, `\.pixi/`,
`ai_tmp/`, `target/`, `\.tools/`, `allium-skill-reference/`, `Cargo\.lock$`,
`pixi\.lock$`, with a comment that `pixi.lock` is YAML and the builtin `check-yaml` would
otherwise parse it on every commit. The three `just <recipe>` hooks are unchanged. The
four script hooks that receive no filenames use pixi as their activator, because a git
hook does not inherit the `Justfile`'s `PATH` and the recipe that could provide it
(`check-docs`) runs prek inside prek:

```yaml
      - id: validate-docs
        name: Documentation contract
        entry: pixi run --frozen bg-validate-docs
        language: system
        pass_filenames: false
        files: ^(README\.md|SECURITY\.md|docs/|pyproject\.toml$)
```

and likewise `validate-agents` (`pixi run --frozen bg-validate-agents`), `check-specs`
(`pixi run --frozen bg-run-allium check`) and `analyse-specs`
(`pixi run --frozen bg-run-allium analyse`), each keeping its `files` trigger. The
comment above `check-specs` says `--frozen` because a bare `pixi run` may rewrite
`pixi.lock` from inside a commit; that pixi resolves the manifest upward from the hook's
working directory, so the environment used is the committing worktree's own; and that
`pyproject.toml` is in the trigger because the Allium pin is the tooling package version,
which the manifest pins, so any pin edit in the file now fires these hooks, which is
accepted. The ripsecrets hook receives filenames, and whether `pixi run` passes a filename
with a space or a quote through unchanged is unverified (§12), so its entry is the
binary's path, which git resolves from the worktree root:

```yaml
      - id: ripsecrets
        entry: .pixi/envs/default/bin/bg-ripsecrets
```

In `.pre-commit-fix.yaml` the entries are unchanged and the `taplo-fmt` comment reads
"`just fix` is the only caller, so `.pixi/envs/default/bin` is already on PATH here".

**Dotfiles.** `.gitignore`: `.venv/` goes; `.pixi/` is added with a comment in the voice
of the `.tools/` one (the environment `just initialize` installs; ignored because
`just check` snapshots the worktree between recipes); `__pycache__/` and `*.py[cod]` stay,
`.ruff_cache/` is T03's call. `.gitattributes`: `uv.lock linguist-generated=true` becomes
`pixi.lock merge=binary linguist-language=YAML linguist-generated=true` (the shape pixi
recommends for its lockfile; T03 confirms the wording against pixi's documentation).
`_typos.toml`: `extend-exclude = ["Cargo.lock", "pixi.lock", ".pixi/", ".tools/",
"target/", "ai_tmp/", "allium-skill-reference/"]`. `lychee.toml`: `exclude_path` is
`.git`, `.pixi`, `ai_tmp`, `target`, `.tools`, `allium-skill-reference`. `taplo.toml`:
`exclude = ["target/**", ".tools/**", ".pixi/**", "ai_tmp/**"]`; this one is
load-bearing, because `toml-check` runs taplo over `**/*.toml` and the environment's
`site-packages` contain TOML. `.markdownlint-cli2.jsonc`: `ignores` is `target`, `.tools`,
`.pixi`, `ai_tmp`, `allium-skill-reference`. `.editorconfig`: the `[*.py]` block stays
(B's scripts are Python; nothing here is).

**Composite action** `.github/actions/setup/action.yml`, input `cache-key`, in this order,
every `uses:` pinned to a full SHA looked up on the day:

1. `prefix-dev/setup-pixi` (v0.10.2) with `pixi-version: v0.81.0`,
   `manifest-path: pyproject.toml`, `locked: true`, `cache: true`,
   `cache-write: ${{ github.ref == 'refs/heads/main' }}`, `activate-environment: true`.
   The comment says: the one version pin in YAML is pixi itself, which nothing else can
   install, and it must satisfy `requires-pixi`; `locked` makes this step the CI lockfile
   check; activation puts `.pixi/envs/default/bin`, and so `just`, on every later step's
   `PATH`; the environment holds no rust, so cargo stays the runner's rustup proxy.
2. `just install-toolchain`.
3. `Swatinem/rust-cache` with `shared-key: ${{ inputs.cache-key }}`, `save-if` on `main`
   only, `cache-bin: false`.
4. `actions/cache` on `.tools` keyed
   `${{ runner.os }}-${{ runner.arch }}-tools-${{ hashFiles('tools.txt') }}`, with a
   comment that `.tools` holds what pixi cannot: cargo-hack from `tools.txt`, and allium as
   a side effect (`bg-install-allium` version-checks, so a stale one is replaced).
5. `just install-tools` with `GITHUB_TOKEN: ${{ github.token }}` on that step only.
6. `just sync`.
7. A step that appends `PREK_HOME=${{ runner.temp }}/prek` to `$GITHUB_ENV`, with a
   comment: prek's cache outside the workspace, so `check-clean`'s snapshot never sees
   it, under the variable T03 confirms on 0.5.3.
8. `actions/cache` on `${{ runner.temp }}/prek` keyed
   `${{ runner.os }}-${{ runner.arch }}-prek-${{ hashFiles('.pre-commit-config.yaml', '.pre-commit-fix.yaml', 'pixi.lock') }}`
   (`pixi.lock` because it pins prek), saving on any branch like the `.tools` cache, with
   a comment that it holds the seven hook clones, the Go, Node and rustup toolchains and
   the ripsecrets build, which every gate job would otherwise fetch again.
9. `just install-hooks`: the prepare and the lychee warm are the point; the shim it
   writes into the runner's `.git` is harmless. After this step no job step reaches the
   network.
10. For the `documents` job, `just install-allium`.

Gone: `taiki-e/install-action`, `astral-sh/setup-uv`, `uv python install 3.14`.
`ci.yml` is unchanged. The pixi version appears twice, exact in YAML and as a floor in the
manifest; the comment says so rather than pretend one owner.

**Bootstrap story**, the sentence every page and ticket reuses: per-machine prerequisites
are rustup (with `~/.cargo/bin` ahead of `~/.pixi/bin` on `PATH`, or
`pixi global uninstall rust`), pixi 0.81.0 or newer, just (`pixi global install just`;
any just launches the recipes, because the pinned one in the environment runs every
nested call and CI; `pixi run --frozen just <recipe>` is the escape hatch when no global
just exists) and gh. Not uv, not cargo-binstall. First run: `just initialize`, then
`just check`.

**§12 claims that replace the binstall and uv ones**, each with its ticket:

- `pixi install --locked` on this `pyproject.toml` installs the nine conda packages and
  B, and does not try to install `libpawdoku-tooling` itself (no `[build-system]`,
  `dependencies = []`). **T00.**
- pixi builds `biscuit-games-tooling` from the git tag with `uv_build` fetched at
  solve time, under the environment's Python 3.14; the six `bg-*` scripts land in
  `.pixi/envs/default/bin`. **T00.**
- `pixi lock --check --offline`, the `lock-check` recipe's own line, writes nothing and
  exits 0 when the lockfile is current, the git PyPI source included. Harness: the
  recipe after `just sync`, with the lockfile's hash recorded before and after. The one
  fallback: the pixi half of `lock-check` moves to CI only, where `setup-pixi`'s
  `locked: true` already checks it. **T03.**
- After `just install-hooks` from an empty `PREK_HOME`, `just lint` is green with the
  network blocked (`HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9`), the
  lychee download included: `install-hooks` is where the gate's hook side pays for the
  network, and `check` never does. Fallback: a hook that still fetches at run is named
  and gets its own warm line in the recipe. **T03.**
- conda-forge's prek 0.5.3 provisions its own rustup and toolchain under `$PREK_HOME` for
  the `language: rust` ripsecrets hook (0.4.12 did), and `prek install --prepare-hooks`
  does so at `install-hooks` time rather than at the first `lint`. **T03.**
- `pixi run -x --frozen bg-ripsecrets` passes a filename containing a space and a single
  quote unchanged; if so, the ripsecrets entry may switch to it for uniformity. **T03.**
- `cargo binstall --root .tools` puts binaries in `.tools/bin`, verifies release checksums
  where the crate publishes them, and skips a matching installed version (unchanged claim,
  now for one tool). **T03.**
- The activation variables `setup-pixi` exports (`CONDA_PREFIX`, `PIXI_*`) are inert for
  cargo; if a build script ever reacts to them, CI appends the environment's `bin` to
  `$GITHUB_PATH` instead of activating. **T04.**
- `setup-pixi` restores `.pixi/envs` and the package cache keyed on the `pixi.lock` hash,
  and `cache-write` gated to `main` behaves like rust-cache's `save-if`. **T04.**
- The prek cache under `${{ runner.temp }}/prek` restores on the second dispatch, and
  `just lint`'s log then shows no clone, download or build line. **T04.**
- Every pin above resolves with `pixi search <name> --platform linux-64` (the osx-arm64
  search does not prove a linux-64 build). **T00.**

**§13 risks.** Kept: "rustup is absent and pixi shadows it", now with the sentence that
an agent's tool shell may not source `~/.cargo/env` (in this session `command -v cargo`
printed `~/.pixi/bin/cargo`), so every agent command prefixes `PATH="$HOME/.cargo/bin:$PATH"`
or the maintainer runs `pixi global uninstall rust`. Added: one environment per worktree,
about 150 MB of hardlinks from pixi's cache, not relocatable because the `bg-*` shebangs
name the worktree's absolute Python, so a renamed worktree needs `pixi install --frozen`
again; the pixi environment must never gain `rust` or a tool that depends on it, and
`check-toolchain`'s `case` line is the alarm if it does; the hook shim's primary-checkout
rule stays. Deleted: "Bootstrapping cargo-binstall".

### Handed back

- **T00.** Rewritten in place: `depends_on: [D01, D02]`; preconditions rustup, pixi, just,
  gh; the manifest tables above, `pixi lock`, the one-line `tools.txt`, the `Justfile`
  changes, the hook entries, the exclusion lists, the composite action; `uv.lock`,
  `.venv/` and `.python-version` removed from the worktree; eleven records in the
  manifest; the taplo blocker marked resolved; the fresh-clone `just check` of its
  acceptance and verification runs with the network blocked, so the "none in `check`"
  invariant is proven rather than assumed.
- **T02.** The tool listing in its step 1 names `.pixi/envs/default/bin` and
  `.tools/bin/cargo-hack`; `lock` recipe lines; `taplo.toml` excludes `.pixi/**`;
  `nextest-version` in `.config/nextest.toml` equals the `cargo-nextest` pin in
  `pyproject.toml`.
- **T03.** No `scripts/install_tools.sh`; `initialize.sh` finalised as above; the four
  §12 claims above (the offline `lock-check` line, the offline `lint` after
  `install-hooks`, prek's toolchain, the hook filename quoting); its cold-cache proof is
  now `just install-hooks` with the network, then `just lint` without it; exclusion
  lists; the prek-from-conda-forge facts.
- **T04.** The action above, now with the `PREK_HOME` step, the prek cache and
  `just install-hooks` after `just sync`; `setup-pixi`'s SHA looked up on the day; the
  three §12 claims above; `.tools` cache kept for cargo-hack and allium.
- **T05.** `AGENTS.md`'s Stack sentence names pixi and rustup, not uv; the Provenance
  bullet on the Python toolchain names pixi as its resolver; the `project-check` skill's
  step 2 lists `.pixi/envs/default/bin`, `.tools/bin/cargo-hack` and `.tools/bin/allium`.
- **T07.** `develop-locally.md`: the bootstrap story above, the pixi hazard kept;
  `repository-map.md`: `pixi.lock` and `.pixi/`; `maintain-dependencies.md`: `pixi update`
  and `pixi lock`, no `uv lock`; `first-change.md`: `just initialize` installs the
  environment; `develop-locally.md`'s first-run caveat is that `just install-hooks` is
  the last network step, after which `just check` is offline.
- **T08.** `commands.md`: the recipe rows as above; `configuration.md`: `pyproject.toml`
  as the tool manifest, `tools.txt` as the exception; `maintenance.md`: pin rotation is
  `pixi update`; `troubleshooting.md`: the pixi environment missing, `pixi.lock` stale,
  the shadowing cargo, and a `lint` that clones or downloads (a cleared prek cache, or a
  secondary worktree whose primary never ran `just initialize`: run `just install-hooks`
  again), never "the first `lint` needs the network".
- **T09.** Record 0011 from the text above, an eleventh row in the index, the numbering
  sentence; 0009's Decision sentence names pixi; the open point on a record for rustup is
  answered by 0011.
- **T10.** README prerequisites and quick start; the layout paragraph; SECURITY's sentence
  on how tools are installed.
- **T11.** The prerequisites line; the fresh-clone rationale names the pixi environment;
  the rate-limit open point now concerns one binstall lookup.
- **S01.** `pixi.lock` in the pin table; whether Renovate or Dependabot understands it is
  a fact to verify.
- **S02, S03.** A new tool is a pixi dependency when conda-forge has it and a `tools.txt`
  line otherwise.
- **S04.** maturin is a pixi dependency when the bindings crate lands; the second
  `pyproject.toml` question is asked of pixi, not uv.
- **C01.** The reusable action's inputs are the pixi version and the manifest path; no
  Python input.
- **C02.** The pin bump is `pixi update biscuit-games-tooling` and a committed `pixi.lock`.
- **D01.** Not edited. Record 0004 is copied verbatim by T09; 0011 names what it
  supersedes.

### Open points settled

Put to the maintainer on 2026-09-24 and answered:

- **Whether pixi replaces uv or only installs it.** Replaces it, with house style set
  aside: one lockfile, one install step, no `uv run --frozen` prefix.
- **cargo-hack.** Kept, through cargo-binstall from the pixi environment and a one-line
  `tools.txt`; not replaced by hand-listed feature combinations, not compiled.
- **The prek-pinned hook tools.** Stay with prek.
- **How the decision is recorded.** This ticket, before the T00 rewrite, like D01.
- **How T00 is reworked.** In place, same id and branch, gaining `depends_on: [D01, D02]`.
- **Where this lands.** A branch from `main`, like D01, with the propagation as a second
  commit.

### Follow-up on this branch

After the decision commit, the maintainer's request to bring every other ticket in line
with this direction is a second commit on this branch: T00 is rewritten in place from the
version on `ticket/t00-foundation` (so its hand-back notes survive), CONVENTIONS.md and
README.md are updated, and every ticket named under Handed back is edited. That commit
supersedes the first non-goal and the last acceptance criterion, both of which were true
of the decision commit itself. The T00 branch must merge or rebase onto `main` after this
branch lands; its `tickets/T00-foundation.md` will conflict and the `main` version wins.

## Open points

- cargo-shear stays at 1.13.4 until conda-forge carries 1.14.0. Nothing this repository
  uses changed between them as far as the release notes say; T02 confirms `cargo shear`
  behaves as its ticket expects at 1.13.4.
- `just` moves from 1.51.0 to 1.58.0 for CI and nested calls. The `Justfile` uses only
  syntax that predates 1.51.0, so any global just launches it.
- Platforms beyond `linux-64` and `osx-arm64` (osx-64 for an Intel Mac) are added when a
  `pixi lock` on that platform succeeds; the manifest comment says how.
- Whether Renovate or Dependabot can move pins in `pixi.lock` (S01).
- Whether `pixi lock --check --offline` passes on a current lock with a git PyPI source
  (§12, T03); if not, the pixi half of `lock-check` moves to CI.
