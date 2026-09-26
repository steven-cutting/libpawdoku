---
id: T03
title: "Hooks and the language-agnostic gate: dotfiles, first-run script, the pixi environment"
status: done
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
there, every shell script is shellcheck-clean, and the seven CONVENTIONS.md §12 claims
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
  `just lint` would download three toolchains, not only hook repositories; that is why
  `install-hooks` prepares every environment (`prek install --prepare-hooks`) and runs
  lychee once, whose `language: script` hook fetches its binary at first run rather
  than at prepare (`scripts/lychee_pre_commit.sh` in the hook's cached checkout,
  installing into `<checkout>/.cargo/bin` under prek's home).
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
   `.pixi/envs/default/bin/prek`, and the comment adds that the recipe also provisions
   every hook environment, the last line of the first-run path that reaches the network
   (D02). Nothing compiles at bootstrap and there is no `uv lock` line.

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

   d. **prek's Rust hooks and the default toolchain.** The `just install-hooks` of
      step 7 answers this. After it, `ls "$PREK_HOME/tools"` and
      `cat "$PREK_HOME"/hooks/rust-*/.prek-hook.json`.
      Expected, from the 0.4.12 evidence in Context: `tools/rustup/` exists and the
      manifest's `toolchain` points inside it, which means conda-forge's prek 0.5.3
      provisions its own rustup and stable toolchain under `$PREK_HOME` for the
      `language: rust` ripsecrets hook, and CONVENTIONS.md §2's "stable default
      toolchain" prerequisite is unnecessary. Either outcome is recorded; the
      prerequisite change is a `main` follow-up because CONVENTIONS.md is not this
      ticket's file. Also record the wall-clock time of the ripsecrets build, the one
      compile in the first-run path, from step 7's `install-hooks` timing. Delete
      `ai_tmp/prek-cold` only after this step has read it.

   e. **`lock-check` offline and without writes.** After `just sync`, record the
      lockfile's hash, then `just lock-check`, whose pixi line is
      `pixi lock --check --offline`: expected exit 0, no output naming a change, and
      the hash unchanged; this proves the check needs no network even with the git
      PyPI source in the lockfile. If it exits non-zero for want of the network, the
      one fallback is recorded as a `main` follow-up for the frozen `Justfile`: the
      pixi half of `lock-check` moves to CI only, where `setup-pixi`'s `locked: true`
      already checks it (T04). Then prove the
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

   g. **`lint` offline after `install-hooks`.** Step 7 answers this: after
      `just install-hooks` from an empty `PREK_HOME`, `just lint` with the network
      blocked is green, the lychee download included. Record the outcome line here.

7. Cold-cache proof that `install-hooks` pays for the network and `lint` does not.
   Never clear `~/.cache/prek` (outside the worktree). Instead point prek at an empty
   home inside the scratch directory:

   ```sh
   export PREK_HOME="$PWD/ai_tmp/prek-cold"
   .pixi/envs/default/bin/prek cache dir     # must print $PREK_HOME
   time just install-hooks
   ls "$PREK_HOME/tools" "$PREK_HOME/hooks"
   HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just lint
   unset PREK_HOME
   ```

   The first command needs the network and takes minutes: seven hook repositories are
   cloned and Go, Node and rustup toolchains are downloaded, ripsecrets compiles, and the
   lychee warm line fetches its release into the hook's cached checkout. Quote the clone
   and install lines and the time. The `just lint` after it runs with the network
   blocked and must print a hook table with every hook `Passed` and no clone, download
   or build line; this is the §12 claim that `check` never fetches, lychee included. If
   a hook fetches anyway, name it and hand back a warm line for it as a `main`
   follow-up for the frozen `Justfile`. If `prek cache dir` does not print the exported
   path, `PREK_HOME` is not the variable on 0.5.3: stop, find the documented one with
   `prek cache --help` and prek's README, and record the correction (the CI setup action
   exports the same variable, so T04 is told). Step 6d reads the directory before it is
   deleted.

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
  matches step 4; G's lines 45-61 are inside `scripts/initialize.sh` with the comment
  changes step 4 names and no other.
- `just lint` is green in this worktree and, with the network blocked, after
  `just install-hooks` from an empty `PREK_HOME`; `just fix` repairs
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
PREK_HOME="$PWD/ai_tmp/prek-cold" sh -c '.pixi/envs/default/bin/prek cache dir && time just install-hooks && HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just lint'
typos_bin=$(ls "$(.pixi/envs/default/bin/prek cache dir)"/hooks/*/bin/typos | head -1); "$typos_bin" --dump-config - | sed -n 1,10p
grep -c 'tool.typos' pyproject.toml || echo "no [tool.typos]"
HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check
just lock-check && echo "lock current"
just check-docs
just check
git status --porcelain
```

Expected: the toolchain line names `1.98.1`; the second `pixi install --frozen` reports
nothing to install; the environment listing shows prek, just, cargo-binstall,
cargo-nextest, cargo-llvm-cov, cargo-deny, cargo-shear, taplo and the six `bg-*`
scripts, and `.tools/bin` lists `allium` and `cargo-hack`; every script `executable`;
the cold run prints the exported path, the clone and install lines, then an offline
hook table with every hook `Passed`; the
dump opens with `[files]` and the seven-entry `extend-exclude`; the grep prints
`no [tool.typos]`; `toml-check` exits 0 within seconds; the lock check prints
`lock current`; `check-docs` and `check` exit 0; `git status --porcelain` prints
nothing. Quote each in the hand-back notes.

## Hand-back notes

**Done on 2026-09-25**, in one session, on the Supacode branch `T03-hooks-and-dotfiles`
(see Deviations). Commits: `a9c30ef` (the dotfiles and checker configurations),
`35b3bcb` (the first-run script's comments and the manifest note), and the closing
commit with these notes. Nothing was pushed. Transcripts are in `ai_tmp/t03/`.

### Actions with effects outside this worktree

- **pixi's package cache** (`~/Library/Caches/rattler/cache`): read by
  `pixi install --frozen`; it gained `ripgrep` 15.2.0 repodata and records from the
  step 6e scratch copy, which used the network as the step allows.
- **prek's default cache** (`~/.cache/prek`): read only (the typos and shellcheck
  binaries of steps 6c and 9, and the hook runs of `just lint`); nothing was prepared
  into it.
- **A `stable` Rust toolchain was installed into `~/.rustup`**, 1.3 GB, at 16:53:20 local
  time, by step 7's cold `install-hooks`. The ticket did not expect it; it is the
  finding under 6d below. It was left in place: it is the maintainer's to keep or remove
  (`rustup toolchain uninstall stable`). No default toolchain was set
  (`rustup default` still prints "no default toolchain is configured").
- **The session scratch directory** held the cold prek homes and a sandbox
  `RUSTUP_HOME` (an APFS clone of the 1.98.1 toolchain) for the 6d attribution; removed
  at the end.
- **The shared hook shim** `/Users/scutting/projects/libpawdoku/.git/hooks/pre-commit`
  was not touched: the cold-cache proof ran in a clone with its own `.git`
  (maintainer's choice). SHA-256 `0c6253b3…` before and after; it still names the
  primary checkout's prek.
- `~/.cargo/bin` was not written to. No clone (G, T, B) was modified.

Inside the worktree but gitignored: `.pixi/envs/default`, `.tools/bin` (`allium` and
`cargo-hack`), `target/`, and `ai_tmp/` (the transcripts, the clone `ai_tmp/clone`).

### What was verified, and how

Every command ran with `PATH="$HOME/.cargo/bin:$PATH"`, so `cargo` was rustup's proxy.

**Step 1.** `pixi install --frozen` (1.5 s), then `just check-toolchain`
(`1.98.1-aarch64-apple-darwin (overridden by '…/T03-hooks-and-dotfiles/rust-toolchain.toml')`).
The first `just lint` failed only `check-specs` and `analyse-specs` ("allium is not
installed; this project pins 3.6.1"): a fresh worktree has no `.tools/bin/allium`, which
step 1 does not install. After `just install-allium`, `just lint` was green. Not a T00
fault; recorded under Deviations.

**Steps 2 and 3.** Each dotfile was diffed against the ticket's fence. The only
differences are the four under Deviations: `[LICENSE]` in `.editorconfig`, `-diff` on
the `pixi.lock` line, and taplo's layout of the two TOML arrays. `.gitignore` and
`.markdownlint-cli2.jsonc` match byte for byte.

**Step 4.** `scripts/initialize.sh`: lines 1-7 and the Allium comment (G's 32-36) are
identical to G's (`cmp` of the extracted ranges). The `install-hooks` block against G's
lines 45-61 differs only in the comment, reflowed for the two edits step 4 names:

```text
< # from a secondary worktree runs that worktree's virtual environment for commits
---
> # from a secondary worktree runs that worktree's .pixi/envs/default/bin/prek for
…
> # the git directory differ. The recipe also provisions every hook environment,
> # so this is the last line of the first-run path that reaches the network.
```

**Step 5.** The five D02 comments are present. The one added line sits where G's
`[tool.typos]` sat, above `[tool.biscuit-games-tooling]`. `pixi lock --check` prints
"Lock-file was already up-to-date", and `taplo fmt --check pyproject.toml` passes.

**CONVENTIONS.md §12 claims assigned to T03:**

- **6a. binstall root, checksum, idempotence: root and idempotence hold; no checksum is
  verified for cargo-hack.** After `rm -rf .tools && just install-tools`:

  ```text
  cargo-binstall:  WARN The package cargo-hack v0.6.45 (aarch64-apple-darwin) has been downloaded from github.com
  cargo-binstall:  INFO   - cargo-hack => .tools/bin/cargo-hack
  cargo-binstall:  INFO Done in 2.391679834s
  ```

  `.tools/bin` held `cargo-hack` alone. binstall's own metadata is at
  `.tools/.crates.toml` and `.tools/binstall/crates-v1.json`, and nothing else is under
  `.tools/`. The second run downloaded nothing and exited 0:

  ```text
  cargo-binstall:  INFO cargo-hack v0.6.45 is already installed, use --force to override
  cargo-binstall:  INFO Done in 1.094292ms
  ```

  Then `just install-allium`; `.tools/bin` is `allium` and `cargo-hack`. The verbose
  flag is `-v` (`--log-level debug`). A `-v` install into a scratch root logs the
  crate's metadata as
  `PkgMeta { pkg_url: Some("{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"), …, signing: None, … }`,
  then `Downloading package url=https://github.com/taiki-e/cargo-hack/releases/download/v0.6.45/cargo-hack-aarch64-apple-darwin.tar.gz`,
  `Download OK`, and the install. No checksum or signature line appears. cargo-hack
  publishes no signing key, so the claim is true only vacuously: the binary is trusted
  on TLS to GitHub and nothing else. Which binstall runs: `cargo-binstall --version`
  is refused (`a value is required for '--version <VERSION>'`); `-V` prints `1.23.0`
  for both the environment's and the per-machine `~/.cargo/bin` build. They are told
  apart by `-V -v`: the environment's says `build-date: 2026-09-05`,
  `rustc-version: 1.97.1` (conda-forge), and the per-machine one says
  `rustc-version: 1.98.1`. `cargo binstall -V -v` with the recipe's `PATH` prints the
  conda-forge build, so the environment's binstall is the one that runs. The recipe
  `PATH` (`just --evaluate`) is `.pixi/envs/default/bin`, `.tools/bin`, then the
  caller's, with `~/.cargo/bin` after both. Beware `cargo --list -v`: it prints
  `binstall  /Users/scutting/.cargo/bin/cargo-binstall` even when a probe script
  placed first on `PATH` is the one that runs. It reports the last match, not the one
  cargo executes.
- **6b. taplo lint offline: holds.**
  `HTTP_PROXY=http://127.0.0.1:9 HTTPS_PROXY=http://127.0.0.1:9 just toml-check` exits 0
  in 0.15 s with no schema line. Nothing for T02.
- **6c. typos precedence: holds.** Two typos environments are in `~/.cache/prek`: G's
  1.48.0 (rev `bee27e3a`) and this repository's 1.50.2 (rev `512fc24f`); the one used
  is the 1.50.2 one. From the root, `typos --dump-config -` opens with `[files]` and the
  seven-entry `extend-exclude`. In `ai_tmp/typos-precedence/`, the dump shows
  `alpha = "alpha"` and no `bravo`. Control: with `_typos.toml` moved aside, the same
  directory dumps `bravo = "bravo"`. So typos reads `pyproject.toml`'s table only when
  no `_typos.toml` is found, and never merges the two.
- **6d. prek's Rust hooks and the default toolchain: the expectation fails on 0.5.3,
  and the lychee warm installs `stable`.** After the cold `install-hooks`,
  `$PREK_HOME/tools` holds `go` (271 MB), `node` (empty) and `rustup` (empty). The
  ripsecrets manifest:

  ```text
  "language": "rust", "language_version": "1.98.1",
  "toolchain": "/Users/scutting/.rustup/toolchains/1.98.1-aarch64-apple-darwin",
  "extra": { "channel": "stable" }
  ```

  prek's trace says why: `Using system installed rustup at /Users/scutting/.cargo/bin/rustup`,
  `Found matching system rust name=1.98.1-aarch64-apple-darwin`, then
  `cargo install --bins --root $PREK_HOME/hooks/rust-… --path . --locked` with that
  toolchain's cargo. So prek 0.5.3 does not provision its own rustup when the machine
  has one: it uses an installed toolchain, here the pinned 1.98.1, and needs no default.
  Node and Python likewise come from the machine: Homebrew's node 26.5.1, and the
  `pixi global` uv 0.11.18 (`~/.pixi/bin/uv`) with an already-installed Python.
  Only Go was downloaded (`https://go.dev/dl/go1.27.1.darwin-arm64.tar.gz`). The
  ripsecrets build, the one compile in the first-run path, took **12.7 s**
  (23:54:22.71 to 23:54:35.46 in the trace).

  The side effect: `just install-hooks`'s last line, the lychee warm, installed a
  `stable` toolchain into `~/.rustup`. Reproduced with a sandbox `RUSTUP_HOME` holding
  only 1.98.1: the toolchain list was unchanged after the recipe's first and second
  lines and gained `stable-aarch64-apple-darwin` after the third. The cause is lychee's
  own checkout: it carries `rust-toolchain.toml` with `channel = "stable"`, its hook
  script `pushd`s into that checkout and runs cargo-binstall, and binstall calls `rustc`
  to detect the target; rustup honours lychee's pin and auto-installs it. A bare `rustc`
  outside any pin refuses instead ("no default is configured"). With
  `RUSTUP_AUTO_INSTALL=0` on that line, lychee installs and passes and no toolchain
  appears (binstall falls back to its built-in target). CI runners ship `stable`, so CI
  is unaffected. The follow-ups are under Handed back.
- **6e. `lock-check` offline and without writes: holds for a current lock; it rewrites
  a stale one.** After `just sync`, `pixi.lock`'s SHA-256 was `86e30aae…` before and
  after `HTTP_PROXY=… HTTPS_PROXY=… just lock-check`, which printed
  `Locking 0 packages` and `✔ Lock-file was already up-to-date`, exit 0. The check
  bites: in a scratch copy with `ripgrep = "*"` added, `pixi lock --check` exits 1 with
  `× lock file not up-to-date with the workspace`. But it also printed
  `✔ Updated lock file` and **rewrote the copy's `pixi.lock`**. The recipe's own form
  does the same whenever the stale manifest can be solved offline. With `cargo-shear`
  removed from a copy:

  ```text
  pixi lock --check --offline            exit=1  REWRITTEN
       WARN `pixi.lock` was written from a solve restricted to locally available packages, so it may pin older versions than the channels offer.
  pixi lock --check --offline --dry-run  exit=1  unchanged
       i Dry-run: lock file would be updated (not written to disk)
  ```

  With `ripgrep` added, the offline form fails its solve ("not available locally") and
  writes nothing. `--dry-run` on the current lock exits 0 with "Dry-run: lock file
  would not change". So the recipe is read-only on a current lock, as §12 claims, and
  not on a stale one: `check-clean` would still go red, but the working tree is left
  with a lock solved from local packages. Follow-up under Handed back.
- **6f. `pixi run` and hook filenames: holds.** `pixi run -x` exists on 0.81.0
  ("Execute the command as an executable without resolving Pixi tasks"). The argv the
  child sees:

  ```text
  $ pixi run -x --frozen python -c '…print(sys.argv[1:])…' "ai_tmp/quoting/it's a file.txt" 'a"b c'
  ["ai_tmp/quoting/it's a file.txt", 'a"b c']
  [True, False]
  ```

  `pixi run -x --frozen bg-ripsecrets "ai_tmp/quoting/it's a file.txt"` needs
  `ripsecrets` on `PATH`, which prek supplies inside a hook. Outside one it exits 1
  with "ripsecrets is unavailable; run just install-hooks". With the cached ripsecrets
  0.1.11 on `PATH`, the clean file exits 0. With two credential-shaped lines appended,
  it prints "ripsecrets found credential material; the matched values are suppressed"
  and exits 1, and ripsecrets itself names `ai_tmp/quoting/it's a file.txt`. A
  caution for whoever switches the entry: ripsecrets exits 0 for a path that does not
  exist, so a clean exit alone never proves a filename arrived intact.
- **6g. `lint` offline after `install-hooks`: holds.** Step 7 below: every hook
  `Passed` with the network blocked, and prek's trace of that run has no clone,
  download, install or build line.

Context facts re-checked at the pinned versions: prek 0.5.3's help names
`$PREK_HOME/prek.log`, and `prek cache dir` printed the exported `PREK_HOME` (so the
variable T04 exports is right). `prek install --overwrite` ran in every `install-hooks`.
`--hook-stage manual` parses, although `--help` shows `--stage`:
`prek run --hook-stage manual lychee-online --files does-not-exist.md` prints
`Lychee online (manual)…(no files to check)Skipped`, and a misspelt flag is refused.
typos 1.50.2's help lists `--dump-config`, `--config` and `--isolated`.

**Step 7, cold cache.** Run in `ai_tmp/clone`, a clone of the branch at `35b3bcb` with
its own `.git`, after `pixi install --frozen` and `just install-allium` there. The
ticket's `PREK_HOME="$PWD/ai_tmp/prek-cold"` cannot work: prek clones ripsecrets under
it, and `cargo metadata` on the clone's `Cargo.toml` walks up into this repository's
workspace:

```text
error: Failed to install hook `ripsecrets`
  caused by: Failed to find package directory using cargo metadata
error: current package believes it's in a workspace when it's not:
workspace: /Users/scutting/.supacode/repos/libpawdoku/T03-hooks-and-dotfiles/Cargo.toml
```

So `PREK_HOME` went outside any Cargo workspace: the session scratch directory, as CI's
`runner.temp` is. Then:

```text
prek cache dir: /private/tmp/claude-501/…/scratchpad/prek-cold
prek install --overwrite --hook-type=pre-commit --prepare-hooks
Installed Git hook at `.git/hooks/pre-commit`
prek prepare-hooks --config .pre-commit-fix.yaml
prek run lychee --files README.md
lychee...................................................................Passed
install-hooks exit=0 wall=35s
```

prek 0.5.3 prints no clone or install line at its default verbosity, so the lines come
from `$PREK_HOME/prek.log`. A second empty home with the recipe's three lines run
separately (19 s, 0 s, 3 s) gave:

```text
DEBUG Cloning repo … repo=https://github.com/editorconfig-checker/editorconfig-checker@675b1261…
DEBUG Cloning repo … repo=https://github.com/DavidAnson/markdownlint-cli2@b82a6c88…
DEBUG Cloning repo … repo=https://github.com/crate-ci/typos@512fc24f…
DEBUG Cloning repo … repo=https://github.com/lycheeverse/lychee@2bba2716…
DEBUG Cloning repo … repo=https://github.com/shellcheck-py/shellcheck-py@745eface…
DEBUG Cloning repo … repo=https://github.com/rhysd/actionlint@914e7df2…
DEBUG Cloning repo … repo=https://github.com/sirwart/ripsecrets@7d946209…
DEBUG Downloading url=https://go.dev/dl/go1.27.1.darwin-arm64.tar.gz
DEBUG Installed hook `typos` / `markdownlint-cli2` / `shellcheck` / `ripsecrets` / `editorconfig-checker` / `actionlint`
```

and the lychee warm: `Installing lychee@0.24.2 by cargo-binstall...`, `The package
lychee v0.24.2 (aarch64-apple-darwin) has been downloaded from github.com`. The home
reached 972 MB. It took 35 s, not minutes, because Node, Python and Rust came from the
machine (6d). Then, with `HTTP_PROXY`, `HTTPS_PROXY`, `http_proxy` and `https_proxy`
all set to `http://127.0.0.1:9`, `just lint` in the clone printed all 23 hooks
`Passed`, exit 0 in 8 s. prek's trace of that run has only `Executing` lines, no
clone, download or install. `lint` never fetches, lychee included.

**Step 8, `just fix`.** A tracked `scratch.toml` holding `[a]` and `key=1` plus a trailing space, with
no final newline: `taplo format … Failed - files were modified by this hook`, and
`git diff scratch.toml` shows `-key=1` (with its trailing space), then
`\ No newline at end of file`, then `+key = 1`.
taplo runs first and repairs all three defects, so trailing-whitespace and
end-of-file-fixer found nothing in it. A second tracked `scratch.txt` (a trailing space,
no final newline) showed both: `fix end of files … Failed`,
`trim trailing whitespace … Failed`, and the diff removes the space and adds the
newline. `git rm --cached -q` refused both files ("staged content different from both
the file and the HEAD"), so they were unstaged with `git restore --staged` and deleted.
`git status --porcelain` then showed no scratch line and only the ticket's files, and
`just lint` was green. `just fix` changed no other file.

**Step 9, shellcheck.** `prek run --all-files shellcheck --verbose` prints the hook's
id, description and duration but no file list on 0.5.3. Visibility was proved instead
with a tracked `scripts/scratch.sh` (`echo $1`): the hook failed with
`In scripts/scratch.sh line 2: … SC2086 (info): Double quote to prevent globbing and word splitting.`
The file was then unstaged and deleted. The hook's own shellcheck 0.11.0, run directly
on `scripts/*.sh`, exits 0 with no output. `scripts/initialize.sh` is the only
`scripts/*.sh` and is executable.

**The Verification block**, from this worktree (`ai_tmp/t03/verification-1.txt`). The
`install-hooks` line is step 7's clone run above. The two listings and the dump print one
name per line and are joined here for length.

```text
$ just check-toolchain
1.98.1-aarch64-apple-darwin (overridden by '…/T03-hooks-and-dotfiles/rust-toolchain.toml')
$ pixi install --frozen && pixi install --frozen
✔ The default environment has been installed.
✔ The default environment has been installed.
bin listing unchanged
$ ls -1 .pixi/envs/default/bin | grep -E '^(prek|just|cargo-|taplo|bg-)' ; ls -1 .tools/bin
bg-install-allium bg-project-check bg-ripsecrets bg-run-allium bg-validate-agents
bg-validate-docs cargo-binstall cargo-deny cargo-llvm-cov cargo-nextest cargo-shear
just prek taplo
allium cargo-hack
scripts/initialize.sh executable
$ "$typos_bin" --dump-config - | sed -n 1,10p     # the 1.50.2 environment
[files]
extend-exclude = [
    "Cargo.lock", "pixi.lock", ".pixi/", ".tools/", "target/", "ai_tmp/", "allium-skill-reference/",
]
$ grep -c '^\[tool\.typos' pyproject.toml || echo "no [tool.typos]"
0
no [tool.typos]
$ HTTP_PROXY=… HTTPS_PROXY=… just toml-check
toml-check exit=0
$ just lock-check && echo "lock current"
✔ Lock-file was already up-to-date
lock current
$ just check-docs
Validated 37 pages and 38 canonical topics.
$ just check
The worktree matches the check baseline.

All checks passed and the worktree is unchanged.
$ git status --porcelain
```

`just check` exited 0 in 12 s, and `git status --porcelain` printed nothing. The
second `pixi install --frozen` printed the same line as the first and the `bin`
listing was identical before and after. pixi 0.81.0 prints no "nothing to install"
wording.

### Deviations, and why

- **Branch.** Committed on the Supacode branch `T03-hooks-and-dotfiles`, not
  `ticket/t03-hooks-and-dotfiles` (the maintainer's choice, 2026-09-25). The `branch:`
  field is left as the ticket wrote it.
- **Step 1 also ran `just install-allium`.** A fresh worktree has no `.tools/bin/allium`,
  and `lint` runs the two spec hooks.
- **`.editorconfig` keeps T00's `[LICENSE] indent_size = unset` block**, after
  `[Makefile]`. Without it editorconfig-checker reports 17 errors in the Apache text
  (T00's hand-back). The hook's `exclude` is frozen, so the exemption belongs here.
- **`.gitattributes` adds `-diff` to the `pixi.lock` line**:
  `pixi.lock merge=binary linguist-language=YAML linguist-generated=true -diff`. The
  lockfile page (`https://pixi.prefix.dev/latest/workspace/lock_file/`, fetched
  2026-09-25) says nothing about attributes. pixi's recommendation is the file
  `pixi init` writes: `pixi init` 0.81.0 writes exactly this line under
  `# SCM syntax highlighting & preventing 3-way merges`, from
  `crates/pixi_api/src/workspace/init/options.rs` on `prefix-dev/pixi` `main`, where
  `-diff` arrived in PR 4913 (merged 2025-11-12, fixing issue 4892 "Don't show
  `pixi.lock` in regular `git diff`"). Effect: `git diff` shows "Binary files … differ"
  for the lock, and `git diff -a` shows the text.
- **`_typos.toml` and `lychee.toml` keep one array entry per line.** The ticket's one-line
  arrays are 114 and 88 columns. `taplo fmt` reflows both at its default width, so
  `toml-check` fails on them (checked in scratch against a copy of the config). Same
  values, taplo's layout, which is also G's layout for `lychee.toml`. T02's pull request
  (#3) sets `column_width = 100` and folds `lychee.toml` onto one line, which is step 3's
  text. This branch leaves `lychee.toml` as T00 wrote it, so the two merge without
  conflict (`git merge-tree` against `origin/ticket/t02-rust-gate`) and T02's form wins.
  `_typos.toml`'s array is 114 columns, so it stays one entry per line under T02's
  width too; both files pass `taplo fmt --check` with T02's `taplo.toml`.
- **The typos grep in the Verification block is anchored.** `grep -c 'tool.typos'`
  counts the comment step 5 asks for ("a [tool.typos] here would be ignored") and
  prints 1. `grep -c '^\[tool\.typos'` asks the question the block means.
- **The typos binary is chosen by rev, not `ls … | head -1`.** The default cache holds
  G's 1.48.0 environment beside this repository's 1.50.2 one, and `head -1` can pick
  either.
- **Step 7's `PREK_HOME` is outside the worktree**, in the session scratch directory.
  Any home under a Cargo workspace breaks the `language: rust` hook (above). It was
  removed afterwards.
- **Step 8 used a second scratch file**, `scratch.txt`, because taplo repairs everything
  in a TOML file before the two builtin fixers run; and `git restore --staged` in place
  of `git rm --cached`, which refuses a staged file the fixers changed.
- **Step 9 proved shellcheck's view with a failing scratch script**, because prek 0.5.3's
  `--verbose` prints no file list.

### Handed back

- **`main` follow-up, `Justfile` (frozen): `lock-check` must not write.** Make its pixi
  line `pixi lock --check --offline --dry-run`. Proven above: exit 1 and nothing
  written on a stale lock, exit 0 and "lock file would not change" on a current one.
  The recipe's comment and CONVENTIONS.md §2, §4 and §13 ("every pixi call in a gate is
  `--frozen` or `lock --check` so none rewrites `pixi.lock`") follow.
- **`main` follow-up, `Justfile` (frozen): the lychee warm must not install a
  toolchain.** Make the line `-RUSTUP_AUTO_INSTALL=0 prek run lychee --files README.md`,
  with a comment that lychee's checkout pins `stable` and its script's cargo-binstall
  would otherwise make rustup install it. Proven in a sandbox `RUSTUP_HOME`. The same
  auto-install would presumably fire on a commit's first lychee run if `install-hooks`
  never ran (not tested).
- **`main` follow-up, CONVENTIONS.md §2 and §12.** The prerequisite sentence is right
  that no `stable` default is needed, but for a different reason than it gives. prek
  0.5.3 uses the machine's rustup and an installed toolchain (the pinned 1.98.1), not a
  rustup of its own under `$PREK_HOME/tools/rustup`. And with the current recipe,
  `install-hooks` installs `stable` anyway (6d). §12's binstall claim should say that
  cargo-hack publishes no checksum or signature, so nothing is verified. Whether prek
  provisions its own rustup, node and uv on a machine that has none was not tested
  (every one was present here).
- **`main` follow-up, `.pre-commit-config.yaml` (frozen), optional:** the ripsecrets
  entry may become `pixi run -x --frozen bg-ripsecrets` for uniformity with the four
  script hooks (6f). Keep the path form if a clean exit on a mangled name matters,
  since ripsecrets exits 0 for a missing path.
- **T04.** `PREK_HOME` is the right variable on 0.5.3. It must never sit under the
  checkout: `${{ runner.temp }}/prek` is right, and anything under the workspace breaks
  the ripsecrets install. The cold `install-hooks` downloads Go on a runner, and Node
  and Python only if the image lacks them. The `.tools` cache key is unaffected.
- **T07.** `develop-locally.md`: `just initialize` installs a `stable` toolchain today
  (the lychee warm) until the `main` follow-up lands. binstall reads the per-user
  `~/.cargo/binstall.toml`. prek builds its Python hooks with any `uv` on `PATH`. A
  cold `install-hooks` took 35 s on Apple silicon with node, uv and rustup present, and
  the ripsecrets build 12.7 s.
- **T08.** `troubleshooting.md`: `cargo --list -v` names the last `cargo-binstall` on
  `PATH`, not the one cargo runs (use `cargo binstall -V -v`). A `PREK_HOME` inside the
  checkout fails the ripsecrets install with "current package believes it's in a
  workspace". An unexpected `stable` toolchain comes from the lychee warm.
  `security-model.md`: cargo-hack is downloaded unsigned, over TLS from its GitHub
  release. `configuration.md`: `_typos.toml` is the only typos configuration.
- **T02.** Nothing from 6b. Its one-line `lychee.toml` stands; this branch does not touch
  that file.
- **Maintainer.** The `stable` toolchain in `~/.rustup` (1.3 GB) is yours to keep or
  remove with `rustup toolchain uninstall stable`.

### Open points settled

Put to the maintainer on 2026-09-25 and answered:

- **Branch name:** keep the Supacode name `T03-hooks-and-dotfiles`.
- **The cold proof and the shared shim:** run it in a scratch clone with its own
  `.git`, so the primary's shim is never rewritten.
- **`[*.toml]` in `.editorconfig`:** implicit, as step 2 prints it. The `[*.rs]`
  comment names taplo as the owner of the two-space default.
- **`.ruff_cache/`:** kept.
- **`allium-skill-reference/` and editorconfig-checker:** needs no change. The frozen
  top-level `exclude` of `.pre-commit-config.yaml` already lists
  `allium-skill-reference/`, and it applies to every hook, editorconfig-checker
  included.

## Open points

- Whether prek 0.5.3 provisions its own rustup, Node and uv under `$PREK_HOME/tools` on
  a machine that has none (every one was present here, so each came from the machine;
  only Go was downloaded). T04's runner answers it for Linux.
- The three `main` follow-ups above (the `lock-check` dry run, the lychee warm's
  `RUSTUP_AUTO_INSTALL=0`, the CONVENTIONS.md wording), which T11 reconciles if they
  have not landed.
