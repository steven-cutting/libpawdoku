---
id: T03
title: "Hooks and the language-agnostic gate: dotfiles, first-run script, pinned tool installs"
status: open
depends_on: [T00, D01]
parallel_with: [T01, T02, T04, T05, T06, T07, T08, T09]
branch: ticket/t03-hooks-and-dotfiles
estimated_size: M
---

# T03: Hooks and the language-agnostic gate: dotfiles, first-run script, pinned tool installs

## Context

The hook gate (`.pre-commit-config.yaml`, run by `just lint`) and its mutating twin
(`.pre-commit-fix.yaml`, run by `just fix`) are frozen at T00 (CONVENTIONS.md §5, §11).
They read seven configuration files that T00 created in working form so the first
`just check` could be green: `.editorconfig`, `.gitattributes`, `.gitignore`,
`.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` and, under D01 option A,
`pyproject.toml`. T00 also left `scripts/initialize.sh` with the cargo-binstall bootstrap
inline (T00 step 7). This ticket finalises those files: every gitignored path a gate
writes is explained in G's voice, the first-run script is split so the tool install can be
re-run on its own after a `tools.txt` bump, every shell script is shellcheck-clean, and
the five CONVENTIONS.md §12 claims assigned to T03 are executed and recorded.

Read first: `CONVENTIONS.md` in full (§2, §5, §11, §12, §13 matter most), then
`README.md` in this directory, then T00's hand-back notes (which option D01 chose, and
what its working dotfiles look like) and D01's hand-back notes.

Sources, read-only, at the commits CONVENTIONS.md §0 pins:

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`, via
  `git -C /Users/scutting/projects/pawdoku show 78d03cdf:<path>`: `.editorconfig`,
  `.gitattributes`, `.gitignore` (lines 14-17 and 24-27 are the two comments this ticket
  keeps and extends), `.markdownlint-cli2.jsonc`, `lychee.toml`, `scripts/initialize.sh`
  (lines 4-5 the root-finding prologue, 45-58 the worktree-aware hook install, 60-61 the
  closing lines), `.pre-commit-config.yaml` (which hook reads which file),
  `pyproject.toml` lines 56-66 (the `[tool.typos]` tables that move to `_typos.toml`).
- B = `/Users/scutting/projects/biscuit_games_tooling` at `6c5c07f6`:
  `src/biscuit_games_tooling/install_allium.py` line 34 (`VERSION = "3.6.1"`), lines
  51-56 (the four checksums) and 61-69 (the target table), and
  `run_ripsecrets_redacted.py` (29 lines). Needed only under option B.

Facts checked on 2026-09-23 on the maintainer's machine, each to be re-checked at the
pinned versions during this ticket:

- prek's help names `$PREK_HOME` as its home (the default log path is
  `$PREK_HOME/prek.log`) and `prek cache dir` prints the cache location; checked on prek
  0.4.12 from G's virtual environment, not on the 0.5.3 this repository pins.
- prek 0.4.12 did **not** use the maintainer's rustup for its `language: rust` hook. Its
  cache holds `tools/rustup/` with its own `rustup` binary and a
  `toolchains/stable-aarch64-apple-darwin`, and the ripsecrets hook manifest records
  `"toolchain": "<cache>/tools/rustup/toolchains/stable-aarch64-apple-darwin"` with
  `"channel": "stable"`. ripsecrets is the only `language: rust` hook: lychee is
  `language: script` (its script downloads a release) and typos and shellcheck are
  Python wheels. The cache also holds `tools/go` and `tools/node`: actionlint and
  editorconfig-checker are Go hooks and markdownlint-cli2 is a Node hook, so a cold
  `just lint` downloads three toolchains, not only hook repositories.
- typos-cli 1.48.0 accepts `--dump-config -` (stdout), `--config`, `--isolated`; the
  repository pins 1.50.2.
- cargo-binstall is not installed on the machine, so none of its flags are verified.

## Goal

The seven dotfiles and the first-run path in their final form, with `just lint`,
`just fix`, `just check-docs` and `just check` green in this worktree and from an empty
prek cache; `scripts/install_tools.sh` added and idempotent; every `scripts/*.sh`
shellcheck-clean; and an outcome line in the hand-back notes for each §12 claim.

## Non-goals

- No change to `.pre-commit-config.yaml`, `.pre-commit-fix.yaml`, `Justfile` or
  `tools.txt`: frozen (CONVENTIONS.md §11). A change any proof here demands is handed
  back as a `main` follow-up, not made.
- No Rust configuration: `taplo.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml` and
  `.config/nextest.toml` are T02's. A taplo finding goes to T02 in the hand-back notes.
- No CI: the composite action's `install-tools` step is T04's.
- No install outside the worktree. The cargo-binstall bootstrap branch of
  `scripts/install_tools.sh` is not exercised here: cargo-binstall is a stated
  prerequisite, and `cargo install` into `~/.cargo/bin` is separately authorised
  (CONVENTIONS.md §11). Record it as untested with that reason.
- No handbook prose: first-run and troubleshooting facts go to T07 and T08 as hand-backs.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `.editorconfig`, `.gitattributes`, `.gitignore` | working -> final | step 2 |
| `.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` | working -> final | step 3 |
| `scripts/install_tools.sh` | new | step 4; `chmod +x` before `git add` |
| `scripts/initialize.sh` | stub -> final | step 5 |
| `pyproject.toml` | comments only (option A) | step 6; content is D01's |
| `.python-version` | unchanged (option A) | see Open points |
| `scripts/project-check.sh`, `scripts/install_allium.sh`, `scripts/run_allium.sh`, `scripts/ripsecrets_redacted.sh` | working -> final (option B) | step 7 |
| `tickets/T03-hooks-and-dotfiles.md` | ticket | `status:` and hand-back notes |

## Steps

1. Create the worktree on `ticket/t03-hooks-and-dotfiles` from `main` (README.md "How to
   pick up a ticket"). Run `just check-toolchain` and `just lint` once; both must be
   green before any edit, or the fault is T00's and this ticket stops.

2. The three Git-facing dotfiles. Final content, option A form; under option B delete
   the `[*.py]` block, the `uv.lock` attribute and the four Python lines of `.gitignore`.

   `.editorconfig`:

   ```ini
   root = true

   [*]
   charset = utf-8
   end_of_line = lf
   indent_style = space
   indent_size = 2
   insert_final_newline = true
   trim_trailing_whitespace = true

   # rustfmt owns Rust layout and writes four spaces; this entry stops
   # editorconfig-checker from reporting what rustfmt has just written. TOML is
   # left at the two-space default above, which is what taplo writes.
   [*.rs]
   indent_size = 4

   [*.py]
   indent_size = 4

   [Justfile]
   indent_size = 4

   # The specifications are written to a fixed column width with their own
   # comment-led layout; treating them as ordinary source reflows them.
   [*.allium]
   indent_size = 4

   # List continuations and fenced blocks legitimately indent by other amounts.
   [*.md]
   indent_size = unset
   trim_trailing_whitespace = false

   [Makefile]
   indent_style = tab
   indent_size = unset
   ```

   `.gitattributes`:

   ```text
   * text=auto eol=lf

   Cargo.lock linguist-generated=true
   uv.lock linguist-generated=true
   allium-skill-reference/** linguist-vendored=true
   ```

   `.gitignore`:

   ```text
   .DS_Store
   .venv/
   __pycache__/
   *.py[cod]
   .ruff_cache/
   .cache/
   .env
   .env.*
   !.env.example

   # lychee's link cache: `cache = true` in lychee.toml, so the offline hook
   # writes it beside the config and reads it on the next run.
   .lycheecache

   # Scratch for agents and maintainers alike: anything that is not a
   # deliverable goes here, never in a commit.
   ai_tmp/

   # Assistant state written on first use; the agent contract covers only the
   # committed surface, so these must never enter the inventory.
   .claude/settings.local.json
   .codex/settings.local.json

   # Everything cargo builds, and the coverage report the coverage recipe writes
   # under it. `just check` snapshots the worktree between recipes, so a build
   # product Git can see would abort the run before any recipe's exit code is
   # read.
   target/

   # The pinned allium binary, installed by `just install-allium`, and the
   # binaries tools.txt pins, installed by `just install-tools`. Ignored for the
   # same reason as target/: a binary Git can see aborts the snapshot.
   .tools/

   # Raw profiles cargo-llvm-cov leaves behind when a run is interrupted before
   # it merges them, and an lcov report written anywhere but target/llvm-cov/.
   *.profraw
   lcov.info

   # What cargo-mutants writes if S03 adopts it; ignored now so a first run can
   # never trip the snapshot.
   mutants.out*/
   ```

3. The three checker configurations (option B drops `.venv` from the first two).
   `.markdownlint-cli2.jsonc`, G's rules and comments verbatim:

   ```jsonc
   {
     "config": {
       "default": true,
       // Prose is wrapped by the author, not by a column limit.
       "MD013": false,
       // Duplicate headings are legitimate under different parents.
       "MD024": { "siblings_only": true },
       // Frontmatter carries a title field, but the level-one heading in the body
       // is the document's real title and the contract requires it.
       "MD025": { "front_matter_title": "" },
       "MD033": { "allowed_elements": ["br"] },
       // The documentation contract owns the first heading, not markdownlint.
       "MD041": false
     },
     "globs": ["**/*.md"],
     "ignores": ["target", ".tools", "ai_tmp", ".venv", "allium-skill-reference"]
   }
   ```

   `lychee.toml`, G's settings verbatim:

   ```toml
   cache = true
   max_retries = 2
   max_concurrency = 8
   timeout = 20
   accept = [200, 204, 206, 429]
   exclude_path = [".git", ".venv", "ai_tmp", "target", ".tools", "allium-skill-reference"]
   ```

   `_typos.toml`, the only typos configuration:

   ```toml
   # The only typos configuration. pyproject.toml carries no [tool.typos]: typos
   # stops at the first configuration file it finds, and finds this one (step 8).

   [files]
   # Lockfiles and vendored reference carry hashes and names that look like
   # typos; the tool directories hold binaries.
   extend-exclude = ["Cargo.lock", ".tools/", "target/", "ai_tmp/", "uv.lock", "allium-skill-reference/"]

   [default.extend-words]
   # "mis-" as a hyphenated prefix (mis-wired, mis-asserting) reads as the
   # fragment "mis", which the checker's dictionary mistakes for "miss"/"mist".
   mis = "mis"
   ```

4. `scripts/install_tools.sh`, new, POSIX sh, `chmod +x` before `git add` (the builtin
   `check-shebang-scripts-are-executable` hook fails otherwise):

   ```sh
   #!/bin/sh
   # Installs the pinned binaries no lockfile can name: cargo-binstall once per
   # machine, then everything tools.txt lists into the gitignored .tools/bin.
   # Run by scripts/initialize.sh, and by hand after a tools.txt bump; `just sync`
   # never runs it, because a binary is not a dependency a lockfile resolves.
   set -eu

   project_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd -P)
   cd "$project_root"

   # cargo-binstall cannot binstall itself, so it is the one tool installed with
   # `cargo install`, into ~/.cargo/bin beside rustup, once per machine. That is
   # a source build and takes minutes. The curl-pipe installer upstream offers
   # is faster and is refused on purpose: a script fetched and run unread is
   # exactly the dependency this gate exists to keep out. CI takes the same pin
   # from taiki-e/install-action (.github/actions/setup/action.yml).
   if command -v cargo-binstall >/dev/null 2>&1; then
       printf '%s\n' "cargo-binstall present: $(cargo binstall --version)"
   else
       printf '%s\n' 'cargo-binstall is absent; building 1.23.0 with cargo install (minutes)' >&2
       cargo install cargo-binstall@1.23.0 --locked
   fi

   # The loop over tools.txt lives in the Justfile's install-tools recipe and
   # nowhere else, so the flags binstall runs with have one owner. A version
   # already in .tools/bin is skipped, which is what makes a re-run safe.
   just install-tools

   printf '\n%s\n' 'In .tools/bin:'
   ls -1 .tools/bin
   ```

   `cargo binstall --version` is unverified; if 1.23.0 spells it differently, use what
   `cargo binstall --help` prints and note it.

5. `scripts/initialize.sh`: G's script reshaped as T00 step 7 ordered it, with the
   inline bootstrap replaced by `sh scripts/install_tools.sh`. Keep G's lines 1-7 (the
   shebang, `set -eu`, the root-finding prologue, the worktree check), then these steps
   with a comment each in G's voice: `just check-toolchain` ("the pin in
   `rust-toolchain.toml` is honoured only by rustup's cargo proxy, so this refuses any
   other cargo before anything is installed with it"); `just install-toolchain` (the
   toolchain, components and targets the file names); `sh scripts/install_tools.sh` ("one
   of the network downloads in the first-run path that no lockfile accounts for");
   `just install-allium` (G's lines 32-37 comment verbatim under option A; under option B
   "pinned and checksummed in `scripts/install_allium.sh`"); `test -f Cargo.lock || cargo
   generate-lockfile` ("`Cargo.lock` is committed and every gate passes `--locked`; this
   line exists for a clone whose lockfile was deleted by hand"); `just sync`;
   `just format` (G's lines 39-40 comment verbatim); then G's lines 45-61 byte for byte:
   the worktree-aware `install-hooks` block and the two closing `printf` lines.

6. Option A only. `pyproject.toml`: add, above `[tool.uv]`, G's comment reshaped ("this
   library ships no Python; this project exists so `uv run --frozen` can provide a pinned
   prek and biscuit-games-tooling to the hooks and recipes; see
   `docs/decisions/0004-hook-runner-and-checkers.md`"), and one comment where G's
   `[tool.typos]` sat: "typos is configured in `_typos.toml`; a `[tool.typos]` here would
   be ignored". No key changes; `uv lock --check` must still pass. `.python-version`
   stays `3.14`; see Open points.

7. Option B only. The four scripts T00 wrote in minimal form become final: a header
   comment naming the B script each ports and its line numbers, shellcheck-clean, and
   each tested by hand. `scripts/ripsecrets_redacted.sh` ports B's
   `run_ripsecrets_redacted.py` whole: `command -v ripsecrets` or exit 1 with "ripsecrets
   is unavailable; run just install-hooks", run with both streams to `/dev/null`, B's
   two messages on status 1 and on any other failure, exit with ripsecrets's status.
   `scripts/install_allium.sh` carries `3.6.1` and the four checksums from B lines 51-56
   verbatim, resolves the target from `uname -s` and `uname -m` per B lines 61-69,
   downloads with `curl --fail --location --max-time 300`, verifies with `shasum -a 256`
   before extracting, and is a no-op when `.tools/bin/allium --version` reports 3.6.1.
   `scripts/run_allium.sh` and `scripts/project-check.sh` keep D01's specification; add
   comments only. Hand tests: `install_allium.sh` twice (the second downloads nothing);
   `run_allium.sh check` (seven modules, empty arrays); `project-check.sh clean` on a
   clean tree (0), after `touch ai_tmp/x` (still 0: ignored) and after `touch x`
   (non-zero; remove `x`); `ripsecrets_redacted.sh ai_tmp/fake.txt` on a made-up
   AWS-shaped key (exit 1, and the key text absent from the output).

8. The §12 claims, each with a fenced proof and an outcome line in the hand-back notes.

   a. **binstall root, checksum, idempotence.** `rm -rf .tools && just install-tools`,
      then `ls .tools/bin` (expected: the six `tools.txt` binaries, seven under option B,
      and nothing under `.tools/` but `bin/` plus binstall's own metadata, whose paths are
      recorded). Run `just install-tools` a second time and quote the lines that say each
      version is already installed; a second run must download nothing and exit 0. Then
      `just install-allium`, because the delete took `.tools/bin/allium` with it. For
      the checksum: re-run with binstall's verbose flag (`cargo binstall --help` names
      it; unverified) and quote the lines that show a release checksum or signature
      being checked, or record that binstall's log does not show it.

   b. **taplo lint offline.** With `.tools/bin/taplo` installed:
      `HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check`.
      Expected: green within seconds. If it stalls or reports a schema fetch (taplo's
      catalog associates `Cargo.toml` with a remote schema unless told otherwise), the
      outcome is "needs a `[schema]` setting in `taplo.toml`", handed to T02.

   c. **typos precedence.** Find the hook's binary: under option A
      `$(uv run --frozen prek cache dir)/hooks/<id>/bin/typos` where `<id>` is the
      environment whose manifest names `crate-ci/typos`; under option B the same path
      with bare `prek`. From the worktree root, `<typos> --dump-config - | head -20` must
      print the `_typos.toml` exclude list. Then, in `ai_tmp/typos-precedence/`, write a
      `_typos.toml` with `[default.extend-words] alpha = "alpha"` and a `pyproject.toml`
      with `[tool.typos.default.extend-words] bravo = "bravo"`; `<typos> --dump-config -`
      run in that directory must print `alpha` and not `bravo`. If it prints both, typos
      merges and the claim fails: record it; the design (one file) still holds because
      `pyproject.toml` carries no table.

   d. **prek's Rust hooks and the default toolchain.** The cold run of step 9 answers
      this. After it, `ls "$PREK_HOME/tools"` and `cat "$PREK_HOME"/hooks/rust-*/.prek-hook.json`.
      Expected, from the 0.4.12 evidence in Context: `tools/rustup/` exists and the
      manifest's `toolchain` points inside it, which means prek 0.5.3 provisions its own
      rustup and stable toolchain and CONVENTIONS.md §2's "stable default toolchain"
      prerequisite is unnecessary. Either outcome is recorded; the prerequisite change is
      a `main` follow-up because CONVENTIONS.md is not this ticket's file. Also record the
      wall-clock time of the ripsecrets build, the one compile in the gate. Delete
      `ai_tmp/prek-cold` only after this step has read it.

   e. **prek binstall assets (option B only).** `just install-tools` in step 8a installs
      `prek@0.5.3`; quote binstall's resolution line for it (the asset name and source).
      Under option A, record "not applicable: prek is a uv dependency".

9. Cold-cache proof of `just lint`. Never clear `~/.cache/prek` (outside the worktree).
   Instead point prek at an empty home inside the scratch directory:

   ```sh
   export PREK_HOME="$PWD/ai_tmp/prek-cold"
   uv run --frozen prek cache dir            # bare `prek` under option B; must print $PREK_HOME
   time just lint
   ls "$PREK_HOME/tools" "$PREK_HOME/hooks"
   unset PREK_HOME
   ```

   This needs the network and takes minutes: seven hook repositories are cloned and Go,
   Node and rustup toolchains are downloaded, lychee's script fetches a release, and
   ripsecrets compiles. Quote the clone and install lines and the final hook table
   (every hook `Passed`). If `prek cache dir` does not print the exported path,
   `PREK_HOME` is not the variable on 0.5.3: stop, find the documented one with
   `prek cache --help` and prek's README, and record the correction. Step 8d reads the
   directory before it is deleted.

10. `just fix` proof. `--all-files` means tracked files, so an ignored scratch file proves
    nothing. Create `scratch.toml` at the root with `[a]`, a key with no spaces around
    `=`, a trailing space and no final newline; `git add scratch.toml`; `just fix`;
    `git diff scratch.toml` must show taplo-fmt, trailing-whitespace and end-of-file-fixer
    each repaired something; then `git rm --cached -q scratch.toml && rm scratch.toml`
    and `git status --porcelain` shows no `scratch.toml` line. Then `just lint` green.

11. shellcheck over every script runs inside `just lint` (the shellcheck-py hook, no
    `files` filter). Prove it saw the scripts: `uv run --frozen prek run --all-files
    shellcheck --verbose` (bare `prek` under B) and quote the file list; then
    `just check-docs` and `just check` green.

12. Record every outcome, set `status: done`, commit on the ticket branch with short
    imperative subjects. Stop before pushing.

## Acceptance criteria

- The seven dotfiles match step 2 and step 3 byte for byte (option A) or with the named
  option-B deletions; `scripts/install_tools.sh` and `scripts/initialize.sh` match steps
  4 and 5; G's lines 45-61 are byte-identical inside `scripts/initialize.sh`.
- `just lint` is green in this worktree and from an empty `PREK_HOME`; `just fix` repairs
  the tracked scratch file and leaves the tree clean; `just check-docs` and `just check`
  are green.
- `sh scripts/install_tools.sh` run twice ends with the same `ls -1 .tools/bin` listing
  and the second run downloads nothing.
- Every `scripts/*.sh` is executable and shellcheck reports nothing.
- `_typos.toml` is the configuration typos dumps from the root, and `pyproject.toml`
  (option A) carries no `[tool.typos]`.
- Each §12 claim of step 8 has an outcome line; a failed claim names the follow-up.
- `git status --porcelain` is empty after `just check`; nothing was pushed; nothing was
  installed outside the worktree.

## Verification

```sh
just check-toolchain
sh scripts/install_tools.sh && sh scripts/install_tools.sh
ls -1 .tools/bin
for s in scripts/*.sh; do test -x "$s" && echo "$s executable"; done
PREK_HOME="$PWD/ai_tmp/prek-cold" sh -c 'uv run --frozen prek cache dir && time just lint'
typos_bin=$(ls "$(uv run --frozen prek cache dir)"/hooks/*/bin/typos | head -1); "$typos_bin" --dump-config - | sed -n 1,10p
grep -c 'tool.typos' pyproject.toml || echo "no [tool.typos]"
HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check
just check-docs
just check
git status --porcelain
```

Expected: the toolchain line names `1.98.1`; the second `install_tools.sh` run prints
`cargo-binstall present` and no download; six (option B: seven) binaries listed; every
script `executable`; the cold run prints the exported path then a hook table with every
hook `Passed`; the dump opens with `[files]` and the six-entry `extend-exclude`; the grep
prints `no [tool.typos]`; `toml-check` exits 0 within seconds; `check-docs` and `check`
exit 0; `git status --porcelain` prints nothing. Quote each in the hand-back notes.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether `.editorconfig` should carry an explicit `[*.toml]` entry naming taplo's
  two-space indent, or rely on the `[*]` default as written. Explicit costs a line and
  states the owner; implicit breaks silently if `[*]` ever changes.
- Whether `allium-skill-reference/` should also be excluded from editorconfig-checker
  (its own `.ecrc` or an `exclude` in the hook's `args`, which is a frozen-file change),
  since it is excluded from markdownlint, lychee and typos alike; vendored text that
  fails `trim_trailing_whitespace` would otherwise block T05.
- Whether `.python-version` is needed under option A when `pyproject.toml` states
  `requires-python = ">=3.14"`. uv resolves the newest matching interpreter without it;
  the file pins the series so two machines resolve the same one. If kept, the reason is
  recorded in T07's `develop-locally.md`; if dropped, it is a T00 follow-up because
  CONVENTIONS.md §3 lists it.
