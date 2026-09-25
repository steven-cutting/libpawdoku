---
id: T03
title: "Hooks and the language-agnostic gate: dotfiles, first-run script, the pixi environment"
status: open
depends_on: [T00, D01, D02]
parallel_with: [T01, T02, T04, T05, T06, T07, T08, T09]
branch: ticket/t03-hooks-and-dotfiles
estimated_size: M
---

# T03: Hooks and the language-agnostic gate: dotfiles, first-run script, the pixi environment

## Context

The hook gate (`.pre-commit-config.yaml`, run by `just lint`) and its mutating twin
(`.pre-commit-fix.yaml`, run by `just fix`) are frozen at T00 (CONVENTIONS.md §5, §11).
They read seven configuration files that T00 created in working form so the first
`just check` could be green: `.editorconfig`, `.gitattributes`, `.gitignore`,
`.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` and `pyproject.toml`. T00 also
left `scripts/initialize.sh` in D02's sequence with its comments still to be written
(T00 step 7). This ticket finalises those files: every gitignored path a gate writes is
explained in G's voice, the first-run script says in G's voice why each of its lines is
there, every shell script is shellcheck-clean, and the six CONVENTIONS.md §12 claims
assigned to T03 are executed and recorded. There is no separate install script: the pixi
environment is installed by `pixi install --locked` on first run and re-synced by `just
sync`, and `just install-tools` is the re-runnable install for the one binary conda-forge
lacks (cargo-hack, from the one-line `tools.txt`).

Read first: `CONVENTIONS.md` in full (§2, §5, §11, §12, §13 matter most), then
`README.md` in this directory, then T00's hand-back notes (what its working dotfiles
look like) and D02's hand-back notes ("Files T00, T03 and T04 create" and "Handed back",
T03).

Sources, read-only, at the commits CONVENTIONS.md §0 pins:

- G = `/Users/scutting/projects/pawdoku` at `78d03cdf`, via
  `git -C /Users/scutting/projects/pawdoku show 78d03cdf:<path>`: `.editorconfig`,
  `.gitattributes`, `.gitignore` (lines 14-17 and 24-27 are the two comments this ticket
  keeps and extends), `.markdownlint-cli2.jsonc`, `lychee.toml`, `scripts/initialize.sh`
  (lines 4-5 the root-finding prologue, 45-58 the worktree-aware hook install, 60-61 the
  closing lines), `.pre-commit-config.yaml` (which hook reads which file),
  `pyproject.toml` lines 56-66 (the `[tool.typos]` tables that move to `_typos.toml`).

Facts checked on the maintainer's machine, each to be re-checked at the pinned versions
during this ticket:

- prek's help names `$PREK_HOME` as its home (the default log path is
  `$PREK_HOME/prek.log`) and `prek cache dir` prints the cache location; checked on
  2026-09-23 on prek 0.4.12 from G's virtual environment, not on the 0.5.3 this
  repository pins.
- prek 0.4.12 did **not** use the maintainer's rustup for its `language: rust` hook. Its
  cache holds `tools/rustup/` with its own `rustup` binary and a
  `toolchains/stable-aarch64-apple-darwin`, and the ripsecrets hook manifest records
  `"toolchain": "<cache>/tools/rustup/toolchains/stable-aarch64-apple-darwin"` with
  `"channel": "stable"`. ripsecrets is the only `language: rust` hook: lychee is
  `language: script` (its script downloads a release) and typos and shellcheck are
  Python wheels. The cache also holds `tools/go` and `tools/node`: actionlint and
  editorconfig-checker are Go hooks and markdownlint-cli2 is a Node hook, so a cold
  `just lint` downloads three toolchains, not only hook repositories.
- prek 0.5.3 from conda-forge (D02, 2026-09-24) is a native binary with no Python
  wrapper, and accepts `prek install --overwrite` and `prek run --hook-stage manual`, the
  forms the frozen `Justfile` uses, although its `--help` shows `--force` and `--stage`.
- typos-cli 1.48.0 accepts `--dump-config -` (stdout), `--config`, `--isolated`; the
  repository pins 1.50.2.
- cargo-binstall 1.23.0 is in `~/.cargo/bin` (T00 installed it), but the one that
  matters under D02 is the pixi environment's, at `.pixi/envs/default/bin`; its flags
  for the one `tools.txt` line are unverified until step 6a.

## Goal

The seven dotfiles and the first-run path in their final form, with `just lint`,
`just fix`, `just check-docs` and `just check` green in this worktree and from an empty
prek cache; `pixi install --frozen` proven idempotent; every `scripts/*.sh`
shellcheck-clean; and an outcome line in the hand-back notes for each §12 claim.

## Non-goals

- No change to `.pre-commit-config.yaml`, `.pre-commit-fix.yaml`, `Justfile` or
  `tools.txt`: frozen (CONVENTIONS.md §11). A change any proof here demands is handed
  back as a `main` follow-up, not made.
- No Rust configuration: `taplo.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml` and
  `.config/nextest.toml` are T02's. A taplo finding goes to T02 in the hand-back notes.
- No CI: the composite action's `install-tools` step is T04's.
- No install outside the worktree. `pixi install` writes to pixi's per-user package cache
  (the environment is hardlinks from it, about 150 MB; CONVENTIONS.md §13), and the cold
  prek run of step 7 writes only under the scratch `PREK_HOME`; nothing else outside the
  worktree is touched, and `~/.cargo/bin` is not written to.
- No handbook prose: first-run and troubleshooting facts go to T07 and T08 as hand-backs.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `.editorconfig`, `.gitattributes`, `.gitignore` | working -> final | step 2 |
| `.markdownlint-cli2.jsonc`, `lychee.toml`, `_typos.toml` | working -> final | step 3 |
| `scripts/initialize.sh` | working -> final | step 4; comments only, the sequence is D02's |
| `pyproject.toml` | comments only | step 5; content is D02's |
| `tickets/T03-hooks-and-dotfiles.md` | ticket | `status:` and hand-back notes |

## Steps

1. Create the worktree on `ticket/t03-hooks-and-dotfiles` from `main` (README.md "How to
   pick up a ticket"). A fresh worktree has no environment: run `pixi install --frozen`
   first, then `just check-toolchain` and `just lint` once; both must be green before any
   edit, or the fault is T00's and this ticket stops.

2. The three Git-facing dotfiles. Final content:

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
   pixi.lock merge=binary linguist-language=YAML linguist-generated=true
   allium-skill-reference/** linguist-vendored=true
   ```

   The `pixi.lock` line is the shape pixi recommends for its lockfile (D02); confirm the
   wording against pixi's documentation on the day and record the page consulted. If the
   documentation recommends a different attribute set, use the documented one and note
   the change.

   `.gitignore`:

   ```text
   .DS_Store
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

   # The pixi environment `just initialize` installs from pixi.lock: every tool
   # binary, Python, prek and the tooling package's scripts. Ignored for the same
   # reason as target/: `just check` snapshots the worktree between recipes, and
   # an environment Git can see aborts the snapshot.
   .pixi/

   # The pinned allium binary, installed by `just install-allium`, and the
   # binaries tools.txt pins because conda-forge lacks them, installed by
   # `just install-tools`. Ignored for the same reason as target/: a binary Git
   # can see aborts the snapshot.
   .tools/

   # Raw profiles cargo-llvm-cov leaves behind when a run is interrupted before
   # it merges them, and an lcov report written anywhere but target/llvm-cov/.
   *.profraw
   lcov.info

   # What cargo-mutants writes if S03 adopts it; ignored now so a first run can
   # never trip the snapshot.
   mutants.out*/
   ```

3. The three checker configurations.
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
     "ignores": ["target", ".tools", ".pixi", "ai_tmp", "allium-skill-reference"]
   }
   ```

   `lychee.toml`, G's settings verbatim:

   ```toml
   cache = true
   max_retries = 2
   max_concurrency = 8
   timeout = 20
   accept = [200, 204, 206, 429]
   exclude_path = [".git", ".pixi", "ai_tmp", "target", ".tools", "allium-skill-reference"]
   ```

   `_typos.toml`, the only typos configuration:

   ```toml
   # The only typos configuration. pyproject.toml carries no [tool.typos]: typos
   # stops at the first configuration file it finds, and finds this one.

   [files]
   # Lockfiles and vendored reference carry hashes and names that look like
   # typos; the environment and tool directories hold binaries.
   extend-exclude = ["Cargo.lock", "pixi.lock", ".pixi/", ".tools/", "target/", "ai_tmp/", "allium-skill-reference/"]

   [default.extend-words]
   # "mis-" as a hyphenated prefix (mis-wired, mis-asserting) reads as the
   # fragment "mis", which the checker's dictionary mistakes for "miss"/"mist".
   mis = "mis"
   ```

4. `scripts/initialize.sh`: G's script reshaped as D02 ordered it and T00 step 7 wrote
   it; this step adds the comments. Keep G's lines 1-7 (the shebang, `set -eu`, the
   root-finding prologue, the worktree check), then these steps with a comment each in
   G's voice: `pixi install --locked` ("every later line needs prek, cargo-binstall or a
   `bg-*` script from the environment; `--locked` refuses a lockfile that disagrees with
   `pyproject.toml`, so a stale lock is fixed by `just lock` and committed, never
   rewritten silently by a first run"); `just install-toolchain` (the toolchain,
   components and targets the file names); `just check-toolchain` ("the pin in
   `rust-toolchain.toml` is honoured only by rustup's cargo proxy, so this refuses any
   other cargo, the pixi environment holding none, before anything is built with it");
   `just install-tools` ("what conda-forge lacks, from `tools.txt` into `.tools/bin`
   through the environment's cargo-binstall: one of the two network downloads in the
   first-run path that no lockfile accounts for"); `just install-allium` (G's lines
   32-37 comment verbatim, the other one); `test -f Cargo.lock || cargo generate-lockfile`
   ("`Cargo.lock` is committed and every gate passes `--locked`; this line exists for a
   clone whose lockfile was deleted by hand"); `just sync`; `just format` (G's lines
   39-40 comment verbatim); then G's lines 45-61: the worktree-aware `install-hooks`
   block and the two closing `printf` lines, byte for byte except that the comment's
   "that worktree's virtual environment" becomes the absolute path the shim records,
   `.pixi/envs/default/bin/prek` (D02). Nothing compiles at bootstrap and there is no
   `uv lock` line.

5. `pyproject.toml`: confirm the comments D02 prints are present (above
   `[tool.pixi.workspace]`, on `platforms`, on the `python` pin, on `cargo-binstall` and
   on `cargo-shear`), and add one comment where G's `[tool.typos]` sat: "typos is configured
   in `_typos.toml`; a `[tool.typos]` here would be ignored". No key changes;
   `pixi lock --check` must still pass. There is no `.python-version`: the manifest pins
   `python = "3.14.*"` and `pixi.lock` records the exact interpreter, so two machines and
   CI resolve the same one (D02). T07 records that in `develop-locally.md`.

6. The §12 claims, each with a fenced proof and an outcome line in the hand-back notes.

   a. **binstall root, checksum, idempotence, for the one tool.** `rm -rf .tools && just
      install-tools`, then `ls .tools/bin` (expected: `cargo-hack` alone, and nothing
      under `.tools/` but `bin/` plus binstall's own metadata, whose paths are recorded).
      Run `just install-tools` a second time and quote the line that says the version is
      already installed; a second run must download nothing and exit 0. Then
      `just install-allium`, because the delete took `.tools/bin/allium` with it. For
      the checksum: re-run with binstall's verbose flag (`cargo binstall --help` names
      it; unverified) and quote the lines that show a release checksum or signature
      being checked, or record that binstall's log does not show it. `cargo` in the
      recipe is rustup's proxy and the `binstall` subcommand it finds is the
      environment's: `.pixi/envs/default/bin/cargo-binstall --version` must print
      `1.23.0` (the spelling of that output is unverified; if 1.23.0 differs, use what
      `--help` prints and note it), and `~/.cargo/bin` must not be on the recipe's
      `PATH` ahead of the environment.

   b. **taplo lint offline.** With the environment installed
      (`.pixi/envs/default/bin/taplo`):
      `HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check`.
      Expected: green within seconds. If it stalls or reports a schema fetch (taplo's
      catalog associates `Cargo.toml` with a remote schema unless told otherwise), the
      outcome is "needs a `[schema]` setting in `taplo.toml`", handed to T02.

   c. **typos precedence.** Find the hook's binary:
      `$(.pixi/envs/default/bin/prek cache dir)/hooks/<id>/bin/typos` where `<id>` is the
      environment whose manifest names `crate-ci/typos`. From the worktree root,
      `<typos> --dump-config - | head -20` must
      print the `_typos.toml` exclude list. Then, in `ai_tmp/typos-precedence/`, write a
      `_typos.toml` with `[default.extend-words] alpha = "alpha"` and a `pyproject.toml`
      with `[tool.typos.default.extend-words] bravo = "bravo"`; `<typos> --dump-config -`
      run in that directory must print `alpha` and not `bravo`. If it prints both, typos
      merges and the claim fails: record it; the design (one file) still holds because
      `pyproject.toml` carries no table.

   d. **prek's Rust hooks and the default toolchain.** The cold run of step 7 answers
      this. After it, `ls "$PREK_HOME/tools"` and `cat "$PREK_HOME"/hooks/rust-*/.prek-hook.json`.
      Expected, from the 0.4.12 evidence in Context: `tools/rustup/` exists and the
      manifest's `toolchain` points inside it, which means conda-forge's prek 0.5.3
      provisions its own rustup and stable toolchain under `$PREK_HOME` for the
      `language: rust` ripsecrets hook, and CONVENTIONS.md §2's "stable default
      toolchain" prerequisite is unnecessary. Either outcome is recorded; the
      prerequisite change is a `main` follow-up because CONVENTIONS.md is not this
      ticket's file. Also record the wall-clock time of the ripsecrets build, the one
      compile in the gate. Delete `ai_tmp/prek-cold` only after this step has read it.

   e. **`pixi lock --check` offline and without writes.** After `just sync`, record the
      lockfile's hash, then `pixi lock --check --offline`: expected exit 0, no output
      naming a change, and the hash unchanged; this proves the check needs no network
      even with the git PyPI source in the lockfile. If it exits non-zero for want of
      the network, the fallback is recorded as a `main` follow-up for the frozen
      `Justfile`: `lock-check` gains `--offline` (an offline solve on a stale lock still
      fails, which is the right answer), or the pixi half of `lock-check` moves to CI
      only, where `setup-pixi`'s `locked: true` already checks it (T04). Then prove the
      check bites: in a scratch copy of the worktree, add `ripgrep = "*"` under
      `[tool.pixi.dependencies]` and run `pixi lock --check` there (the copy may use the
      network); expected non-zero exit and a message saying the lockfile is out of date.

   f. **`pixi run` and hook filenames.** The ripsecrets hook entry is the binary's path
      (`.pixi/envs/default/bin/bg-ripsecrets`) because whether `pixi run` passes a
      filename with a space or a quote through unchanged is unverified. Create
      `ai_tmp/quoting/it's a file.txt` with harmless content and run
      `pixi run -x --frozen bg-ripsecrets "ai_tmp/quoting/it's a file.txt"`; expected: the
      file is scanned by that exact name (quote the output, or a `--verbose` line if the
      wrapper is quiet). If it passes unchanged, record that the entry may switch to
      `pixi run -x --frozen bg-ripsecrets` for uniformity with the four script hooks, as
      a `main` follow-up for the frozen config; if it does not, record why the path form
      stays.

7. Cold-cache proof of `just lint`. Never clear `~/.cache/prek` (outside the worktree).
   Instead point prek at an empty home inside the scratch directory:

   ```sh
   export PREK_HOME="$PWD/ai_tmp/prek-cold"
   .pixi/envs/default/bin/prek cache dir     # must print $PREK_HOME
   time just lint
   ls "$PREK_HOME/tools" "$PREK_HOME/hooks"
   unset PREK_HOME
   ```

   This needs the network and takes minutes: seven hook repositories are cloned and Go,
   Node and rustup toolchains are downloaded, lychee's script fetches a release, and
   ripsecrets compiles. Quote the clone and install lines and the final hook table
   (every hook `Passed`). If `prek cache dir` does not print the exported path,
   `PREK_HOME` is not the variable on 0.5.3: stop, find the documented one with
   `prek cache --help` and prek's README, and record the correction. Step 6d reads the
   directory before it is deleted.

8. `just fix` proof. `--all-files` means tracked files, so an ignored scratch file proves
   nothing. Create `scratch.toml` at the root with `[a]`, a key with no spaces around
   `=`, a trailing space and no final newline; `git add scratch.toml`; `just fix`;
   `git diff scratch.toml` must show taplo-fmt, trailing-whitespace and end-of-file-fixer
   each repaired something; then `git rm --cached -q scratch.toml && rm scratch.toml`
   and `git status --porcelain` shows no `scratch.toml` line. Then `just lint` green.

9. shellcheck over every script runs inside `just lint` (the shellcheck-py hook, no
   `files` filter). Prove it saw the scripts: `.pixi/envs/default/bin/prek run --all-files
   shellcheck --verbose` and quote the file list; then
   `just check-docs` and `just check` green.

10. Record every outcome, set `status: done`, commit on the ticket branch with short
    imperative subjects. Stop before pushing.

## Acceptance criteria

- The seven dotfiles match step 2 and step 3 byte for byte; `scripts/initialize.sh`
  matches step 4; G's lines 45-61 are inside `scripts/initialize.sh` with the one
  comment change step 4 names and no other.
- `just lint` is green in this worktree and from an empty `PREK_HOME`; `just fix` repairs
  the tracked scratch file and leaves the tree clean; `just check-docs` and `just check`
  are green.
- `pixi install --frozen` run twice leaves `.pixi/envs/default/bin` listing the same
  names and the second run installs nothing; `ls .tools/bin` shows `allium` and
  `cargo-hack` and nothing else.
- Every `scripts/*.sh` is executable and shellcheck reports nothing.
- `_typos.toml` is the configuration typos dumps from the root, and `pyproject.toml`
  carries no `[tool.typos]`.
- Each §12 claim of step 6 has an outcome line; a failed claim names the follow-up.
- `git status --porcelain` is empty after `just check`; nothing was pushed; nothing was
  installed outside the worktree but pixi's package cache.

## Verification

```sh
just check-toolchain
pixi install --frozen && pixi install --frozen
ls -1 .pixi/envs/default/bin | grep -E '^(prek|just|cargo-|taplo|bg-)' ; ls -1 .tools/bin
for s in scripts/*.sh; do test -x "$s" && echo "$s executable"; done
PREK_HOME="$PWD/ai_tmp/prek-cold" sh -c '.pixi/envs/default/bin/prek cache dir && time just lint'
typos_bin=$(ls "$(.pixi/envs/default/bin/prek cache dir)"/hooks/*/bin/typos | head -1); "$typos_bin" --dump-config - | sed -n 1,10p
grep -c 'tool.typos' pyproject.toml || echo "no [tool.typos]"
HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check
pixi lock --check && echo "lock current"
just check-docs
just check
git status --porcelain
```

Expected: the toolchain line names `1.98.1`; the second `pixi install --frozen` reports
nothing to install; the environment listing shows prek, just, cargo-binstall,
cargo-nextest, cargo-llvm-cov, cargo-deny, cargo-shear, taplo and the six `bg-*`
scripts, and `.tools/bin` lists `allium` and `cargo-hack`; every script `executable`;
the cold run prints the exported path then a hook table with every hook `Passed`; the
dump opens with `[files]` and the seven-entry `extend-exclude`; the grep prints
`no [tool.typos]`; `toml-check` exits 0 within seconds; the lock check prints
`lock current`; `check-docs` and `check` exit 0; `git status --porcelain` prints
nothing. Quote each in the hand-back notes.

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
- Whether `.ruff_cache/` stays in `.gitignore` (D02 left it to this ticket). Nothing in
  this repository runs ruff, so the line is inert; keeping it costs nothing and covers a
  future bindings crate's Python tests. Recommendation: keep it, and drop it if T08's
  repository map has to explain it.
