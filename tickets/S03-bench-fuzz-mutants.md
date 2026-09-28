---
id: S03
title: "Spike: benchmarks, fuzzing and mutation testing"
status: done
depends_on: [T11]
parallel_with: []
branch: ticket/s03-bench-fuzz-mutants
estimated_size: S
---

# S03: Spike: benchmarks, fuzzing and mutation testing

## Context

The gate CONVENTIONS.md §4 fixes measures correctness (nextest, doctests, proptest
through T02's dev-dependency), coverage against a floor, lints, features and targets.
It measures nothing about speed, says nothing about inputs no test author thought of,
and cannot tell a test that asserts from one that merely runs. The tools for those are
benchmarks, fuzzing and mutation testing, and none belongs in a gate that must be
deterministic, offline and read-only on the worktree (CONVENTIONS.md §1 fact 3, §13
"`--locked` and the snapshot"). This spike decides what, if anything, is added and
where it runs: never in `check`, either in a scheduled CI job, or by hand through a
recipe.

Facts to start from, each verified at execution against the tool's documentation or
`pixi search` (as reasoned on 2026-09-23):

- **Benchmarks.** divan is attribute-style (`#[divan::bench]`), small, and prints a
  table; criterion keeps baselines on disk and compares runs, at the cost of a larger
  tree and files under `target/criterion`. Both are dev-dependencies, so adding either
  is a T02 hand-back with the decision 0007 note (§11, "No dependency is added outside
  T02"), and a `bench` recipe is a `Justfile` change, a T00 follow-up on `main`. A bench
  target under `crates/pawdoku/benches/` is its own crate root with `std`, so the
  `no_std` core is untouched.
- **Fuzzing.** cargo-fuzz drives libFuzzer on a nightly toolchain, invoked as
  `cargo +nightly fuzz run <target>`. Its `fuzz/` crate sits outside the workspace
  (`[workspace] exclude = ["fuzz"]`), so it enters neither `cargo hack`, `cargo shear`,
  coverage nor the lockfile; `taplo fmt --check` still walks `fuzz/Cargo.toml` unless
  `taplo.toml` excludes it. Nightly is a maintainer prerequisite beside the 1.98.1
  pin, never the pin (§2). First targets are the parsers: a grid-string import,
  anything that turns bytes into a `Sudoku`.
- **Mutation testing.** cargo-mutants (27 on 2026-09-23) runs on stable, writes
  `mutants.out/`, which T00's `.gitignore` already ignores (§5), and takes minutes to
  hours. It is a scheduled job and a report, never a gate: a surviving mutant is a
  finding for a person.
- What enters `check`: nothing. The gate list is frozen at T00 and the three are
  non-deterministic (fuzzing), slow (mutants) or write files (criterion baselines).

Read first: CONVENTIONS.md §4, §5, §10 (`audit.yml` is the precedent for a scheduled,
non-required job), §11, §13; `docs/reference/testing.md` and
`docs/reference/quality-gates.md` (T08); `crates/pawdoku/Cargo.toml` as T02 left it;
`.gitignore` for the `mutants.out*/` line; `taplo.toml` for its `exclude` list.

## Goal

A recommendation per tool (adopt now, adopt on a named trigger, or not at all), where
each runs, and a drafted follow-up for whatever is adopted. Nothing is installed or
changed outside this file. The starting recommendation: **divan** as a dev-dependency
with a `just bench` recipe once the solver exists (a benchmark of `SIDE` is noise);
**cargo-fuzz** with one target the day a parser lands, by hand on the maintainer's
nightly and as a short-budget scheduled job (`-max_total_time=300`) if the runner has
one; **cargo-mutants** as a weekly scheduled job that uploads `mutants.out/` as an
artefact and is never required.

| Tool | Deterministic | Writes to the tree | Toolchain | Where it runs | Adopt |
| --- | --- | --- | --- | --- | --- |
| divan | yes (timing varies) | `target/` only | stable | `just bench` by hand | when a solver exists |
| criterion | yes (timing varies) | `target/criterion` baselines | stable | `just bench` by hand | only if baselines are wanted |
| cargo-fuzz | no | `fuzz/corpus`, `fuzz/artifacts` (ignored) | nightly | by hand; scheduled with a time budget | when a parser exists |
| cargo-mutants | yes | `mutants.out/` (ignored) | stable | weekly scheduled job, artefact upload | once tests exist |

## Non-goals

- Adding a dependency, a recipe, a workflow or a `fuzz/` directory: the follow-up does
  that, with the recipe and the tool pin (a pixi dependency in `pyproject.toml` when
  conda-forge carries cargo-mutants, checked with `pixi search`, otherwise a `tools.txt`
  line) as T00 follow-ups, because the `Justfile` and `tools.txt` are frozen and the
  manifest is T00's.
- A performance target for the solver; a specification question for `effort.allium`
  and `solver.allium`, not a tooling one.
- Changing the coverage floor or the gate order.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S03-bench-fuzz-mutants.md` | ticket | the evidence, the recommendations, the drafted follow-up; `status: done` |
| `tickets/README.md` | ticket index | S03's row set to `done` (added in the first review round; see Deviations) |

## Steps

1. Create the worktree on `ticket/s03-bench-fuzz-mutants` from `main` after T11 has
   merged (README.md "How to pick up a ticket").

2. Verify, read-only, against each tool's current documentation and record with
   sources: (a) divan's and criterion's versions and MSRV against 1.98.1; (b) whether
   divan runs under `cargo bench` from a `no_std` crate's `benches/` target with
   `harness = false`; (c) cargo-fuzz's nightly requirement, the layout `cargo fuzz
   init` generates, and whether it writes into the workspace manifest;
   (d) cargo-mutants' version, whether it honours `--locked` and
   `.config/nextest.toml`, its `--in-place` and `--jobs` flags, and what `mutants.toml`
   can exclude (test modules, the fake in `random.rs`); (e) whether `ubuntu-latest`
   can install a nightly with `rustup toolchain install nightly --profile minimal`
   inside a job without touching `rust-toolchain.toml`.

3. Size the workload on the tree as it is: the functions cargo-mutants would mutate
   (`cargo mutants --list` from a scratch install under the session's scratch
   directory, by `pixi exec` if conda-forge carries it or by a binstall there, never
   `.tools/` or `.pixi/`) and the tests nextest lists. A crate with ten functions
   is not worth a weekly job yet; the recommendation says when it becomes worth it.

4. Write, per adopted tool, the exact changes the follow-up needs: the dev-dependency
   for `[workspace.dependencies]` and the crate (T02 hand-back), the `benches/<name>.rs`
   skeleton, the `bench` and `mutants` recipes and the cargo-mutants pin, a pixi
   dependency or a `tools.txt` line (T00 follow-up),
   the `taplo.toml` and `.gitignore` edits (T02 and T03 hand-backs), the `[workspace]
   exclude` line, and a `mutants.yml` workflow (weekly `schedule`, `workflow_dispatch`,
   `contents: read`, `actions/upload-artifact` of `mutants.out/`, never required)
   modelled on `audit.yml`.

5. Draft the follow-up in the hand-back notes: id `T14`, one ticket or one per tool
   (say which and why), each with the trigger that makes it worth picking up ("when
   `solver.rs` exists", "when the first parser lands").

6. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- Answers (a) to (e) are recorded with sources and dates.
- The table is completed with the versions and behaviours found.
- Each tool has a verdict of adopt now, adopt on a named trigger, or not at all, and
  "what enters `check`" is stated as nothing, with the reason.
- The follow-up draft lists every frozen-file change as a T00 follow-up and every
  dependency as a T02 hand-back.
- `git status --porcelain` on the ticket branch lists only this file.

## Verification

```sh
grep -n 'mutants.out' .gitignore
grep -n 'exclude' taplo.toml
rustup toolchain list
cargo nextest list --workspace --locked 2>/dev/null | tail -1
git status --porcelain
```

Expected: the `mutants.out*/` line; taplo's exclude list (`target/**`, `.tools/**`,
`.pixi/**`, `ai_tmp/**`; no `fuzz/**` today); the
installed toolchains (a nightly is not required for this spike); the test count; one
line naming this file.

## Hand-back notes

### What was verified, and how

Run on 2026-09-27 (US Pacific) in the Supacode worktree for this ticket, on branch
`S03-bench-fuzz-mutants` (see Deviations), from `main` at `b82da0a` after S02 had merged.
Commands stamped 2026-09-28 UTC belong to the same run. No tracked file changed except
this one. `just initialize` installed the ignored `.pixi/` and `.tools/` in the worktree.
Outside it, tools went only into the session's scratch directory and pixi's own package
cache (`pixi exec`). No nightly toolchain was installed. Every experiment ran
in a scratch copy made with `git archive HEAD`.

The maintainer authorised these network uses on the day: read-only documentation
(docs.rs, crates.io's API, GitHub, `actions/runner-images`, mutants.rs, the rust-fuzz
book, the rustup book), `pixi search`, `gh api` reads, and installs into the scratch
directory only, including cargo fetches for the scratch copies. The maintainer also
authorised one time-boxed cargo-mutants run on a scratch copy, and `just initialize` in
this worktree, which the agent ran.

The worktree before any step:

```text
$ git rev-parse --short HEAD origin/main
b82da0a
b82da0a
$ git status --porcelain
$ git rev-parse 'HEAD^{tree}'
2c0d624c5a3496042031f561a62376761ccf262c
```

**Step 1.** The worktree existed before the ticket ran (Deviations).

**Step 2. The five questions.** Sources were read on 2026-09-27. "docs" marks an official
page, and "source" marks code, a manifest or an API record.

**(a) Versions and MSRV: both are well under 1.98.1.**

```text
$ cargo search divan --limit 1
divan = "0.1.21"    # Statistically-comfy benchmarking library.
$ cargo search criterion --limit 1
criterion = "0.8.2"    # Statistics-driven micro-benchmarking library
```

- **divan 0.1.21** needs Rust 1.80.0 (`rust_version` on
  `https://crates.io/api/v1/crates/divan`, source). It was released on 2025-04-10.
  It has six normal dependencies, none optional: cfg-if, clap, condtype, divan-macros,
  libc and regex-lite. Its maintenance is thin. It has had no release since 2025-04-10.
  Its only commit since 2025-04-14 is a docs change on 2026-07-19. The repository is not
  archived and has 51 open issues (`gh api repos/nvzqz/divan`, source).
- **criterion 0.8.2** needs Rust 1.86 and was released on 2026-02-04. It has fifteen
  required and six optional normal dependencies, and its default features turn on
  `rayon` and `plotters`. It is active: the last commit was on 2026-09-08. Its book says
  a run "saves statistical information in the `target/criterion` directory. Subsequent
  executions of the benchmark will load this data and compare it with the current
  sample" (`criterion-rs.github.io/book/user_guide/command_line_output.html`, docs).
- **What divan adds to `Cargo.lock`.** Resolving it in a scratch copy locked nine
  packages: anstyle 1.0.14, clap 4.6.7, clap_builder 4.6.7, clap_lex 1.1.1, condtype
  1.3.0, divan 0.1.21, divan-macros 0.1.21, regex-lite 0.1.9 and terminal_size 0.4.4. Its
  other dependencies (cfg-if, libc, rustix, syn and their own) were already in the lock
  through proptest and thiserror.

**(b) divan runs from the `no_std` crate's `benches/` target, and the gate is happy with
it: proved, not only read.** The README declares `[[bench]] harness = false` with
`fn main() { divan::main(); }` (docs). divan is itself a std crate (its `src/divan.rs`
line 3 imports `std::`, source), which is fine because a bench file is compiled as a
crate of its own (the Cargo targets reference, docs). A scratch copy got the
dev-dependency, `[[bench]] name = "random"` with `harness = false`, and a
`benches/random.rs` benchmarking a thousand `SeededStream` draws.

```text
$ cargo bench --locked -p pawdoku --bench random
    Finished `bench` profile [optimized] target(s) in 10.42s
     Running benches/random.rs (target/release/deps/random-0061b203f095be12)
Timer precision: 41 ns
random                           fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ seeded_stream_thousand_draws  541.3 ns      │ 853.8 ns      │ 687.2 ns      │ 655 ns        │ 100     │ 800
```

The gate's own commands were then run on that copy, and every one exited 0:

- `cargo fmt --all --check`;
- `taplo fmt --check` and `taplo lint`;
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, which
  compiles and lints the bench under the whole workspace lint table;
- `cargo hack check --workspace --feature-powerset --locked`, and the same for
  `--target wasm32v1-none`;
- `cargo shear` ("no issues found": it sees the bench's use of divan);
- `cargo deny --locked check licenses bans sources`. It printed the same
  `license-not-encountered` warnings the tree on `main` prints.

A plain `cargo bench --workspace` also runs every unit test in bench mode, where each
reports "ignored". `--bench '*'` runs the bench targets only:

```text
$ cargo bench --workspace --all-features --locked --bench '*'
     Running benches/random.rs (target/release/deps/random-06a6604181663caf)
╰─ seeded_stream_thousand_draws  541.3 ns      │ 645.6 ns      │ 551.7 ns      │ 551.4 ns      │ 100     │ 800
```

Afterwards the copy's `target/` held `debug`, `release`, `tmp` and `wasm32v1-none`, and
no `criterion` directory. divan writes nothing outside `target/`.

**(c) cargo-fuzz: nightly is documented, but stable works with `--sanitizer none`.**

```text
$ pixi search cargo-fuzz --platform linux-64
Error:   × No packages found matching 'cargo-fuzz'
$ pixi search cargo-fuzz --platform osx-arm64
Error:   × No packages found matching 'cargo-fuzz'
$ cargo search cargo-fuzz --limit 1
cargo-fuzz = "0.13.2"    # A `cargo` subcommand for fuzzing with `libFuzzer`! Easy to use!
$ gh api repos/rust-fuzz/cargo-fuzz/releases/latest --jq '.tag_name, (.assets[].name)'
0.13.2
cargo-fuzz-0.13.2-x86_64-apple-darwin.tar.gz
cargo-fuzz-0.13.2-x86_64-pc-windows-msvc.zip
cargo-fuzz-0.13.2-x86_64-unknown-linux-musl.tar.gz
```

The release has no checksum files and no `aarch64-apple-darwin` archive. cargo-binstall
1.23.0 installed it into the scratch directory under `--disable-strategies compile`. It
logged "The package cargo-fuzz v0.13.2 (x86_64-apple-darwin) has been downloaded from
github.com", so on Apple silicon the binary runs under Rosetta.

- **The nightly requirement (docs).** The 0.13.2 README says it "needs a nightly compiler
  since it uses some unstable command-line flags", and the book's setup page says the
  same "since it uses the -Z compiler flag to provide address sanitization". Issue #411,
  "Support for stable Rust", was closed as not planned on 2025-05-23. PR #440, "Clarify
  stable support for --sanitizer none", is open and unmerged; its body says the "reduced
  `--sanitizer none` mode works on stable".
- **The flags (source).** In `src/project.rs` at 0.13.2, `Sanitizer::None => {} // needs
  no flags`. Every other sanitizer emits `-Zsanitizer=…`, because
  `has_sanitizers_on_stable()` compares against a `u32::MAX` placeholder. The remaining
  flags are `-Cpasses=sancov-module`, `-Cllvm-args=…`, `--cfg fuzzing` and
  `-Cdebug-assertions`, all stable.
- **Stable, proved on the pin.** In a scratch copy, with the layout fixed as described
  below:

  ```text
  $ cargo fuzz build --target aarch64-apple-darwin   # default sanitizer (address), 1.98.1
  error: failed to run `rustc` to learn about target-specific information
    error: the option `Z` is only accepted on the nightly compiler
  $ cargo fuzz build -s none --target aarch64-apple-darwin
      Finished `release` profile [optimized + debuginfo] target(s) in 40.15s
  $ cargo fuzz run -s none --target aarch64-apple-darwin fuzz_target_1 -- -max_total_time=20
  #55522539 DONE   cov: 20 ft: 34 corp: 10/1561b lim: 4096 exec/s: 2643930 rss: 26Mb
  Done 55522539 runs in 21 second(s)
  ```

  The target fed `ReplayStream::new` with `f64`s built from the input bytes and asserted
  every draw in `[0, 1)`. A second target panicked when the input began with `PAW`. The
  stable run found it in under a minute, printed `thread '<unnamed>' … panicked at
  fuzz_targets/panics.rs:5:9` and `SUMMARY: libFuzzer: deadly signal`, wrote
  `fuzz/artifacts/panics/crash-3d921a05…`, and `cargo fuzz` exited 1.
- **cargo-fuzz defaults `--target` to the triple it was built for.** The x86_64 macOS
  binary therefore builds for `x86_64-apple-darwin` unless told otherwise, so the recipe
  passes the host triple from `rustc -vV`.
- **What `cargo fuzz init` writes.** Run from the workspace root and, in a second copy,
  from `crates/pawdoku`, it wrote only `fuzz/.gitignore`, `fuzz/Cargo.toml` and
  `fuzz/fuzz_targets/fuzz_target_1.rs` beneath its directory (`git status --porcelain`
  in each copy). It never touched the workspace manifest, which matches the template in
  `src/templates.rs` (source). Both copies' `fuzz/Cargo.toml` were byte-identical. Each
  was `name = "pawdoku-fuzz"`, `publish = false`, `libfuzzer-sys = "0.4"`, with
  `[dependencies.pawdoku] path = ".."`. It had no `[workspace]` table, which only
  `--fuzzing-workspace` writes. From the root, `path = ".."` names the virtual workspace
  manifest, not the crate, and the build fails:

  ```text
  error: current package believes it's in a workspace when it's not:
  …
  Alternatively, to keep it out of the workspace, add the package to the `workspace.exclude` array, or add an empty `[workspace]` table to the package's manifest.
  ```

  With `exclude = ["fuzz"]` in the root `[workspace]` and the path changed to
  `"../crates/pawdoku"`, it builds. The generated `.gitignore` covers `target`, `corpus`,
  `artifacts` and `coverage`. The build writes `fuzz/Cargo.lock`, which is not ignored
  (152 lines, 18 packages). It is a second lockfile outside every `--locked` and
  `lock-check`. cargo-fuzz takes no `--locked` flag (`cargo fuzz run --help`), so the
  recipe checks the lockfile first with `cargo fetch --locked --manifest-path
  fuzz/Cargo.toml`: exit 0 on the current lock, and exit 101 ("cannot update the lock
  file … because --locked was passed") after a dependency was added.
- **The gate with `fuzz/` present.** With the exclude and the fixed path, each of these
  exited 0 on the scratch copy: `cargo fmt --all --check`; `taplo fmt --check` and
  `taplo lint`, which walk `fuzz/Cargo.toml` ("total=12 excluded=0"); `cargo shear`;
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; and
  `cargo update --workspace --locked --offline`. No TOML file appeared under
  `fuzz/target`, `fuzz/corpus` or `fuzz/artifacts`. So `taplo.toml` needs no exclude: the
  generated manifest is taplo-clean, and formatting it is wanted. None of the gate's
  commands reaches the fuzz crate's Rust: it is not a workspace member, so it has no
  workspace lints, clippy and `cargo fmt --all` skip it, and the `fmt-check` hook matches
  only `crates/`.
- **Bringing the fuzz crate into the gate (first review round, 2026-09-28).** Reviewers
  asked how invariant 4 would reach the fuzz crate. The maintainer chose a lint table in
  `fuzz/Cargo.toml` plus `--manifest-path` runs in the gate. It was proved offline on a
  fresh `git archive HEAD` copy with no cargo-fuzz binary. The copy got the root
  `exclude = ["fuzz"]`, a hand-written `fuzz/Cargo.toml` with
  `[lints.rust] unsafe_code = "forbid"` and `non_ascii_idents = "forbid"`, and one
  `fuzz_target!` feeding `ReplayStream::new`. libfuzzer-sys 0.4.13's macro expands to two
  `#[no_mangle]` functions (`src/lib.rs` lines 251 and 260, source), which `unsafe_code`
  covers when they are written in the crate itself. On the expansion it did not fire:

  ```text
  $ cargo generate-lockfile --offline --manifest-path fuzz/Cargo.toml
       Locking 17 packages to latest Rust 1.98.1 compatible versions
  $ cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked --offline -- -D warnings
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.84s
  $ cargo fmt --manifest-path fuzz/Cargo.toml --check                  # exit 0
  $ cargo update --workspace --locked --offline --manifest-path fuzz/Cargo.toml   # exit 0
  ```

  The 2.84 s was a cold build of the fuzz crate's target directory. It includes
  libfuzzer-sys's build script compiling libFuzzer's C++ through `cc` (26 objects and
  `libfuzzer.a`), so every gate run that lints the fuzz crate needs a C++ compiler.
  Both negative controls failed as they should:
  - An `unsafe {}` planted in the target: "error: usage of an `unsafe` block", exit 101.
  - A dependency added to `fuzz/Cargo.toml` without relocking: "cannot update the lock
    file … because --locked was passed", exit 101.

  The root `cargo update --workspace --locked --offline` still exited 0 beside it.
- **Components.** None is documented for building or running. Only `cargo fuzz coverage`
  needs `llvm-tools-preview`. libfuzzer-sys 0.4.13 compiles libFuzzer with the system C++
  compiler through `cc` (its `build.rs`, source).

**(d) cargo-mutants 27.1.0: on conda-forge for both platforms, and it runs on the pin.**

```text
$ pixi search cargo-mutants --platform linux-64
cargo-mutants-27.1.0-hb17b654_0    Timestamp 2026-06-24 16:02:15 UTC
Dependencies: libgcc >=14, __glibc >=2.17,<3.0.a0
$ pixi search cargo-mutants --platform osx-arm64
cargo-mutants-27.1.0-h6fdd925_0    Timestamp 2026-06-24 16:24:05 UTC
Dependencies: __osx >=11.0
```

The package depends on no `rust`, so pinning it cannot put a cargo ahead of rustup's
(CONVENTIONS.md §13). Its crates.io `rust_version` is 1.88. The GitHub release has
x86_64 Linux, x86_64 macOS and Windows archives and no checksum files. The pixi pin
makes the release assets irrelevant. It was run as
`pixi exec --spec cargo-mutants==27.1.0 -- cargo-mutants mutants …` in the scratch copy.

- **`--locked`.** It never adds `--locked` itself (`src/cargo.rs`, source).
  `--cargo-arg=--locked` (`-C`) reaches every build and test. The baseline log shows
  `cargo test --verbose --package=pawdoku@0.1.0 --all-features --locked`. Its one
  `cargo metadata` call runs `--no-deps` without it (`src/workspace.rs`, source), and
  `cmp` found the scratch `Cargo.lock` byte-identical to the worktree's after both runs.
- **`.config/nextest.toml`: honoured under `--test-tool nextest`.** It is proved, not
  read. The docs do not mention it. cargo-mutants copies hidden files (`src/copy_tree.rs`,
  source), and nextest reads the copy. In a third scratch copy with
  `nextest-version = "99.0.0"`, the baseline failed with "this repository requires
  nextest version 99.0.0, but the current version is 0.9.146". Nextest mode skips
  doctests. The docs say: "nextest currently does not run doctests, so behaviors that
  are only caught by doctests will show as missed". The baseline logs agree: the
  `cargo test` baseline ran "Doc-tests pawdoku … 11 passed", and the nextest baseline
  ran none.
- **`--in-place` and `--jobs`.** `--in-place` mutates the source tree itself instead of a
  copy, and it conflicts with `--jobs` (docs and `src/main.rs`). `-j`/`--jobs`
  (`CARGO_MUTANTS_JOBS`) runs parallel builds; the docs advise "very conservatively,
  starting at -j2 or -j3".
- **Configuration.** It is read from `.cargo/mutants.toml` only. The source joins
  `.cargo/mutants.toml` and has no root `mutants.toml` (`src/config.rs`, source). Its
  keys include `exclude_globs`, `examine_globs`, `exclude_re`, `examine_re`,
  `test_tool`, `additional_cargo_args`, `additional_cargo_test_args`, `all_features`,
  `timeout_multiplier` and `minimum_test_timeout`.
- **What is excluded without configuration.** `#[cfg(test)]` items and functions with a
  `#[test]`-like attribute are excluded (mutants.rs "mutants", docs), and `--list` shows
  no test function. So is every function named `new`: "Don't look inside constructors
  (called "new") because there's often no good alternative" (`src/visit.rs` lines
  455-466 at v27.1.0, source). That makes `ReplayStream::new`'s range check invisible to
  mutation today. `#[mutants::skip]` needs the `mutants` crate "as a regular dependency
  not a dev-dependency" (mutants.rs "attrs", docs). That is against decision 0007, so the
  house never uses the attribute and filters in `.cargo/mutants.toml` instead.
- **The copy includes ignored files unless told otherwise.** `--gitignore` is off by
  default (`gitignore_off_by_default` in `src/options.rs`, source). So a run from a
  worktree would copy `.pixi/`, `.tools/` and `ai_tmp/` into every build directory.
  cargo-mutants skips only the top-level `target` and VCS directories. In a git copy
  holding 200 MB of ignored `.pixi/` ballast, the debug log read
  `Copied source tree total_bytes=211779848` without the flag, and
  `total_bytes=2064648` with `--gitignore=true`. Both runs reported the same 27 caught
  and 1 unviable.
- **Output and exit codes.** A `mutants.out/` directory is created in the source root,
  and the previous one is renamed to `mutants.out.old`. `.gitignore`'s `mutants.out*/`
  covers both. Exit codes: 0 means every viable mutant was caught, 2 that some were
  missed, 3 a timeout, and 4 a failing baseline (mutants.rs "exit-codes", docs).

**(e) Runners: a nightly is available but no longer needed.**

- **The runner image.** `ubuntu-latest` is the Ubuntu 24.04 image 20260920.314.1: Rust
  1.98.1, rustup 1.29.1, no nightly, and GNU C++ 12 to 14
  (`actions/runner-images` `images/ubuntu/Ubuntu2404-Readme.md`, docs).
- **The move to 26.04.** It "will be rolled out over a period of several weeks beginning
  October 19, 2026" (`actions/runner-images` issue 14748), and that image lists the same
  Rust and rustup.
- **Override precedence.** The rustup book gives "A toolchain override shorthand used on
  the command-line, such as `cargo +beta`" before "The `rust-toolchain.toml` file" before
  "The default toolchain" (`rust-lang.github.io/rustup/overrides.html`, docs). So
  `cargo +nightly fuzz` would override the pin for that one command, while
  `rustup default nightly` (the rust-fuzz book's CI example) would not.
- **What was not verified.** The rustup book never states that `rustup toolchain install
  nightly --profile minimal` leaves `rust-toolchain.toml` alone. The command writes to
  `RUSTUP_HOME`, but that is inference, and the command was not run here: no nightly was
  installed. Open point 1 (below) removes nightly from every recipe and job, so no
  follow-up depends on it. Deviations records this as the one part of (e) left open.

**Step 3. The workload, on the tree as it is.**

```text
$ cargo nextest list --workspace --locked 2>/dev/null | wc -l
      19
$ cargo nextest list --workspace --all-features --locked 2>/dev/null | wc -l
      22
$ cargo test --doc --workspace --all-features --locked 2>&1 | grep -E '^test result'
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ cargo mutants --list-files
crates/pawdoku/src/lib.rs
crates/pawdoku/src/random.rs
$ cargo mutants --list | wc -l
      28
```

Here and below, `cargo mutants` stands for
`pixi exec --spec cargo-mutants==27.1.0 -- cargo-mutants mutants`, run in the scratch copy.

The 28 mutants fall in five functions of `random.rs`, plus the `UNIT` constant's division
(2):

| Function | Mutants |
| --- | --- |
| `SeededStream::next_draw` | 15 |
| `SeededStream::index` | 2 |
| `ReplayStream::next_draw` | 5 |
| `ReplayStream::index` | 2 |
| `TryFrom<ReplayScript> for ReplayStream` | 2 |

`lib.rs` has none, and neither `new` has any. With
`--workspace --all-features --gitignore=true --cargo-arg=--locked`, the T14 recipe's
flags, it lists the same 28, and a full run with them is recorded in (d). The authorised run, with the default `cargo test` tool:

```text
$ cargo mutants --all-features -C --locked -o <scratch>/mo-cargo --no-shuffle
Found 28 mutants to test
ok       Unmutated baseline in 4s build + 1s test
 INFO Auto-set test timeout to 20s
28 mutants tested in 24s: 27 caught, 1 unviable
```

It took 25 s of wall time, and `missed.txt` and `timeout.txt` were empty. The one
unviable mutant replaces `try_from` with `Ok(Default::default())`, and `ReplayStream` has
no `Default`. The same run with `--test-tool nextest` gave the same result in 35 s (36 s
wall). So the tests catch every mutant cargo-mutants can make today, doctests or no, and
the tool costs under a minute plus setup. The verdicts below say why it is adopted now
anyway.

**The completed table.**

| Tool | Version (MSRV) | Deterministic | Writes to the tree | Toolchain | Where it runs | Adopt |
| --- | --- | --- | --- | --- | --- | --- |
| divan | 0.1.21 (1.80.0) | yes (timing varies) | `target/` only, checked | stable 1.98.1 | `just bench` by hand | when `solver.rs` exists |
| criterion | 0.8.2 (1.86) | yes (timing varies) | `target/criterion` baselines | stable | nowhere | not at all, unless run-to-run baselines are wanted |
| cargo-fuzz | 0.13.2, libfuzzer-sys 0.4.13 | no | `fuzz/corpus`, `fuzz/artifacts`, `fuzz/target` (ignored by the generated `fuzz/.gitignore`); `fuzz/Cargo.lock` (committed) | stable 1.98.1 with `--sanitizer none`; nightly only for AddressSanitizer | `just fuzz` by hand; a weekly scheduled job, 300 s per target | when the first parser lands |
| cargo-mutants | 27.1.0 (1.88), on conda-forge | yes | `mutants.out/` (ignored) | stable 1.98.1 | `just mutants` by hand; a weekly scheduled job with an artefact | now: T14 |

**The verdicts.**

- **cargo-mutants: adopt now (T14).** The recommendation was a trigger ("when a module
  beyond `random.rs` lands"), because today's run misses nothing. The maintainer chose to
  adopt it now: the job costs little and is then watching when the first new module
  lands. It uses the default `cargo test` tool, not nextest, so that a mutant only a doc
  example catches is not reported as missed.
- **divan: adopt when `crates/pawdoku/src/solver.rs` exists.** A benchmark of `SIDE`, or
  of the generator, measures nothing anyone will act on. divan's thin maintenance
  (above) is re-checked at pickup, and criterion is the fallback.
- **criterion: not at all.** Its one advantage, a baseline compared across runs, answers a
  question nobody has asked: a performance target is a specification question for
  `solver.allium` and `effort.allium`, not a tooling one. Reconsider it if one is written.
- **cargo-fuzz: adopt when the first parser lands** (the first public function that turns
  a string or bytes into a grid). It runs on the pinned stable toolchain with
  `--sanitizer none`, both by hand and as a weekly job. The one byte-to-value path today is
  the `serde` deserialisation of the two streams. It needs a format crate to fuzz, and it
  guards the test fake, so it is not worth a target of its own.
- **What enters `check`: nothing.** Fuzzing is nondeterministic by design. Mutation
  testing takes 25 s today and grows with the crate towards hours, and a surviving mutant
  asks for a judgement, not a red gate. A benchmark measures the machine as much as the
  code. criterion writes baselines. Each would break one of the gate's properties:
  deterministic, bounded, or read-only on the worktree.

**Steps 4 and 5. The follow-ups: one ticket per tool.** They are one per tool because the
triggers are independent. Mutation testing is ready now, and the other two wait on code
that does not exist. A single ticket would sit two-thirds blocked, and it could not be
marked done. The files and lanes differ as well:

- **mutants:** no crate dependency;
- **bench:** a T02 hand-back;
- **fuzz:** a crate outside the workspace, with a lockfile of its own.

This spike owns the id `T14`. S02 claims `T13`, S04 claims `T15` to `T17` and S01 claims
`T18`. The unmerged `game-design-research` branch claims `T19` to `T21` and `S05`. S01's
reviews twice caught an id collision. So cargo-mutants, the one ready now, takes `T14`. The benchmark
and fuzzing drafts take the next free T number on the day their trigger fires. None of
the three is created here, because this spike's Files touched lists no ticket file for
them. The agent that opens one copies its block and adds the index row.

**T14, drafted for `tickets/T14-mutation-testing.md`.**

```yaml
---
id: T14
title: "Mutation testing: cargo-mutants as a weekly report"
status: open
depends_on: [S03]
parallel_with: []
branch: ticket/t14-mutation-testing
estimated_size: S
---
```

> **Context.** S03 (hand-back notes) measured cargo-mutants 27.1.0 on the pinned 1.98.1:
> 28 mutants in `random.rs`, 27 caught, 1 unviable, none missed, in 25 s. The maintainer
> chose to adopt it now, so that it is watching when the first module beyond `random.rs`
> lands. It is a report, never a gate: a surviving mutant is a finding for a person.
>
> **Goal.** `just mutants` runs cargo-mutants over the workspace on a copy of the tree. A
> scheduled workflow runs it weekly and by hand, and uploads `mutants.out/` as an
> artefact. The workflow is never a required check and never in `check`'s `needs`.
>
> **Non-goals.** A mutation score threshold. A gate. `--in-diff` on pull requests. A
> `.cargo/mutants.toml`, until a mutant needs excluding. Any crate dependency:
> `#[mutants::skip]` needs `mutants` as a regular dependency, which decision 0007
> forbids, so an exclusion goes in `.cargo/mutants.toml` as `exclude_re`.
>
> **Files touched.**
>
> - **`pyproject.toml` and `pixi.lock`** (T00 follow-up on `main`). Under
>   `[tool.pixi.dependencies]`, then `pixi lock`:
>
>   ```toml
>   # A weekly report, never a gate (mutants.yml). conda-forge's build depends on no
>   # rust, so rustup's cargo stays first on PATH.
>   cargo-mutants = "==27.1.0"
>   ```
>
> - **`Justfile`** (T00 follow-up on `main`). After `audit`:
>
>   ```just
>   # Mutation testing, a report and never a gate: it grows with the crate towards
>   # hours, and a mutant that survives asks for a judgement. It mutates a copy of the
>   # tree, so the worktree gains only mutants.out/, which Git ignores; the copy leaves
>   # out what Git ignores, or it would carry .pixi/ and .tools/ into every build.
>   # `cargo test`, not nextest, because nextest skips doctests and a mutant only a
>   # doc example catches would be reported missed. Exit 2 means a mutant survived.
>   mutants *args:
>       cargo mutants --workspace --all-features --gitignore=true --cargo-arg=--locked "$@"
>   ```
>
> - **`.github/workflows/mutants.yml`** (new), modelled on `audit.yml`:
>
>   ```yaml
>   name: Mutants
>
>   # Mutation testing is a report, not a gate: a mutant that survives is a
>   # finding for a person, and the run grows with the crate. So this is never a
>   # required check, never in ci.yml's `check` needs, and never on a pull
>   # request. A red run means a mutant survived; the artefact says which.
>
>   on:
>     schedule:
>       # Mondays, 07:00 UTC, an hour after audit.
>       - cron: '0 7 * * 1'
>     workflow_dispatch:
>
>   permissions:
>     contents: read
>
>   concurrency:
>     group: mutants-${{ github.workflow }}-${{ github.ref }}
>     cancel-in-progress: true
>
>   env:
>     CARGO_TERM_COLOR: always
>     CARGO_INCREMENTAL: '0'
>     CARGO_NET_RETRY: '10'
>
>   jobs:
>     mutants:
>       name: mutants
>       runs-on: ubuntu-latest
>       timeout-minutes: 30
>       steps:
>         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
>           with:
>             persist-credentials: false
>         - uses: ./.github/actions/setup
>           with:
>             cache-key: mutants
>         - run: just mutants
>         # Unless cancelled, so a run that found a surviving mutant (exit 2)
>         # still leaves its report. Four weeks, to compare a month of runs.
>         - uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
>           if: ${{ !cancelled() }}
>           with:
>             name: mutants
>             path: mutants.out
>             if-no-files-found: warn
>             retention-days: 28
>   ```
>
>   Re-check both action SHAs against `ci.yml` on the day, and raise `timeout-minutes`
>   when the mutants step takes more than half of it.
> - **`.gitignore`** (T03's file). Reword the comment above `mutants.out*/`, which today
>   says "if S03 adopts it", to say that `just mutants` writes it and renames the previous
>   run to `mutants.out.old`.
> - **`docs/reference/testing.md`.** A "Mutation testing" section. It says what a missed,
>   caught, unviable and timed-out mutant mean, and why the tool is `cargo test`. It names
>   the blind spot: cargo-mutants never mutates a function named `new`, so logic there,
>   such as `ReplayStream::new`'s range check, is covered only by tests a person wrote.
> - **`docs/reference/quality-gates.md`.** Add mutation testing to the checks missing
>   from the gate on purpose, with the reason, and add `mutants.yml` beside `audit.yml` in
>   "In continuous integration".
> - **`docs/reference/commands.md`.** A `just mutants` row, and the recipe added to the
>   list of recipes outside `just check`.
> - **`docs/explanation/quality-philosophy.md`.** The finding S03's open point 2 settled.
>   Coverage says a line ran, and a mutant says whether any test would notice the line
>   changing. A mutant that survives on a covered line is exactly what the floor cannot
>   see. The floor stays at 90. A survivor is either a missing assertion, which gets
>   added, or an equivalent mutant, which makes no observable difference and is excluded
>   in `.cargo/mutants.toml` with a comment.
> - **`docs/operations/maintenance.md`.** In **Weekly**, read the `mutants` run beside the
>   `audit` run.
> - **`CHANGELOG.md`.** An `[Unreleased]` line.
> - **`tickets/README.md`.** The T14 row.
>
> **Proof.** `just mutants` locally reports every mutant caught or unviable, and the
> worktree is unchanged apart from `mutants.out/`. `just check` is green. After the
> merge, and each separately authorised: a `workflow_dispatch` of `mutants.yml`, the
> `mutants` artefact downloaded and read, and branch protection still naming `check`
> alone.

**The benchmark ticket, drafted for `tickets/T<next>-benchmarks.md`.** It takes the next
free T number when its trigger fires: `crates/pawdoku/src/solver.rs` exists on `main`.

> **Goal.** `just bench` runs divan benchmarks of the solver by hand. It is never a gate
> and never in CI, because timings measure the machine and its load.
>
> **Files touched.**
>
> - **`Cargo.toml`** (T02 hand-back, with the decision 0007 note). Under
>   `[workspace.dependencies]`, add `divan = "0.1.21"`. The decision 0007 note: a
>   development dependency only, outside the graph a consumer resolves and outside
>   cargo-deny's. Its MSRV 1.80.0 is below the pin. It adds nine packages to
>   `Cargo.lock`: anstyle, clap, clap_builder, clap_lex, condtype, divan, divan-macros,
>   regex-lite and terminal_size. Reword the table's comment, "The only three crates the
>   core may use", so that the development-only entries are named as such.
> - **`crates/pawdoku/Cargo.toml`** (T02 hand-back). `divan = { workspace = true }` under
>   `[dev-dependencies]`, and:
>
>   ```toml
>   [[bench]]
>   name = "solver"
>   harness = false
>   ```
>
> - **`crates/pawdoku/benches/solver.rs`** (new). The shape has a crate doc comment, a
>   `main`, and a documented `#[divan::bench]` function. It passed `just clippy`'s
>   command on a scratch copy, with a body drawing from `SeededStream`. Here the body
>   benchmarks the solver's public entry point, whatever `solver.rs` names it:
>
>   ```rust
>   //! Benchmarks for the solver, run by `just bench`.
>
>   fn main() {
>       divan::main();
>   }
>
>   /// Solving one puzzle from a fixed, named set, the input passed through
>   /// `divan::black_box` so that the work cannot be hoisted.
>   #[divan::bench]
>   fn solve_one() {
>       // The solver's entry point on a puzzle from the set.
>   }
>   ```
>
> - **`Justfile`** (T00 follow-up on `main`). In the develop section:
>
>   ```just
>   # Benchmarks, by hand: timings measure the machine and its load as much as the
>   # code, so never a gate and never in CI. Bench targets only: without --bench '*'
>   # cargo also runs every unit test in bench mode, as ignored. just clippy already
>   # compiles the benches, so a bench that stops building fails the gate.
>   bench *args:
>       cargo bench --workspace --all-features --locked --bench '*' "$@"
>   ```
>
> - **Documentation.** A `just bench` row in `docs/reference/commands.md`. A
>   "Benchmarks" section in `docs/reference/testing.md`, which also names the puzzle set.
>   `crates/pawdoku/benches/` in `docs/project/repository-map.md`, if that page lists the
>   crate's directories. A `CHANGELOG.md` line. The index row.
>
> **Before starting.** Re-check divan's maintenance (no release since 2025-04-10 as of
> 2026-09-27). If it has been abandoned, criterion 0.8.2 is the fallback, with
> `default-features = false` to drop rayon and plotters. It writes `target/criterion`,
> which `target/` already ignores.

**The fuzzing ticket, drafted for `tickets/T<next>-fuzzing.md`.** It takes the next free T
number when its trigger fires: the first public function that turns a string or bytes into
a grid lands on `main`.

> **Goal.** A `fuzz/` crate at the root, with one target per parser, run by `just fuzz` on
> the pinned stable toolchain with `--sanitizer none`. A weekly workflow fuzzes each
> target for 300 s and uploads any crash. It is never a gate and never required, and
> nightly is never a prerequisite (S03's open point 1). `cargo +nightly fuzz run` with
> AddressSanitizer stays possible by hand but has no recipe: the core forbids unsafe
> code, so ASan has little to find.
>
> **Files touched.**
>
> - **`Cargo.toml`** (T02 hand-back). In `[workspace]`, add `exclude = ["fuzz"]`. Without
>   it, cargo refuses to build the fuzz crate ("current package believes it's in a
>   workspace when it's not"). With it, the crate stays out of `cargo hack`,
>   `cargo shear`, coverage and `Cargo.lock`.
> - **`fuzz/Cargo.toml`, `fuzz/.gitignore`, `fuzz/fuzz_targets/<parser>.rs`** (new), from
>   `cargo fuzz init --target <parser>` at the root. Then:
>   - change `path = ".."` to `path = "../crates/pawdoku"`, because `..` is the virtual
>     workspace manifest;
>   - write `libfuzzer-sys` as a full caret range (`"0.4.13"`), with the decision 0007
>     note. That note (T02 hand-back): outside the workspace, `publish = false`, never
>     shipped, and locked by its own lockfile;
>   - add the lint table below, because invariant 4 holds in every crate. An excluded
>     package is its own root, so `lints.workspace = true` cannot reach the root's
>     `[workspace.lints]`. The table repeats that table's two `forbid` levels, and S03
>     proved it builds with `fuzz_target!` (hand-back notes, (c)):
>
>     ```toml
>     # Invariant 4 in a crate outside the workspace, which cannot inherit
>     # [workspace.lints]: the root table's forbid levels, repeated.
>     [lints.rust]
>     unsafe_code = "forbid"
>     non_ascii_idents = "forbid"
>     ```
>
> - **`fuzz/Cargo.lock`** (new, committed). The generated `.gitignore` does not ignore
>   it. `sync` and `lock-check` cover it, as the `Justfile` bullet below describes.
> - **`tools.txt`** (T00 follow-up on `main`). Add `cargo-fuzz@0.13.2`, because
>   conda-forge does not carry it. The release publishes no checksum, which is
>   cargo-hack's standing in CONVENTIONS.md §12. The macOS archive is x86_64 only. On
>   Apple silicon, `just initialize` only downloads it, but `just fuzz` runs it under
>   Rosetta 2, so Rosetta becomes a macOS prerequisite for fuzzing alone (Documentation,
>   below). Every CI job's setup then downloads it.
> - **`Justfile`** (T00 follow-up on `main`). Existing recipes gain a line for the
>   fuzz crate, so that the gate proves it exactly as it proves the workspace, and no new
>   gate entry is added. `sync`'s comment, "Both lockfiles", names three:
>
>   ```just
>   sync:
>       cargo fetch --locked
>       cargo fetch --locked --manifest-path fuzz/Cargo.toml
>       pixi install --frozen
>
>   lock:
>       cargo update --workspace
>       cargo update --workspace --manifest-path fuzz/Cargo.toml
>       pixi lock
>
>   lock-check:
>       cargo update --workspace --locked --offline
>       cargo update --workspace --locked --offline --manifest-path fuzz/Cargo.toml
>       pixi lock --check --offline --dry-run
>
>   # cargo fmt --all skips the fuzz crate, which is outside the workspace.
>   fmt-check:
>       cargo fmt --all --check
>       cargo fmt --manifest-path fuzz/Cargo.toml --check
>
>   # The fuzz crate is outside the workspace, so --workspace skips it, and its own
>   # lint table (invariant 4) holds only if something compiles it. libfuzzer-sys's
>   # build script compiles libFuzzer's C++ through cc, so this line makes a C++
>   # compiler a prerequisite of the gate.
>   clippy:
>       cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
>       cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings
>   ```
>
>   `lock-upgrade` gains the same `--manifest-path` line after `cargo update`. `fix` and
>   `format` gain `cargo fmt --manifest-path fuzz/Cargo.toml` and the matching
>   `cargo clippy … --fix` line. In the develop section:
>
>   ```just
>   # Fuzzing, by hand: nondeterministic, so never a gate. On the pinned stable
>   # toolchain without a sanitizer: AddressSanitizer needs nightly, and the core
>   # forbids unsafe code. cargo-fuzz takes no --locked and would rewrite a stale
>   # fuzz/Cargo.lock, so the fetch refuses one first; after just sync it finds
>   # every crate in the cache. --target because cargo-fuzz defaults to the triple
>   # it was built for, and its macOS binary is x86_64.
>   fuzz target seconds="300":
>       cargo fetch --locked --manifest-path fuzz/Cargo.toml
>       cargo fuzz run --sanitizer none --target "$(rustc -vV | sed -n 's/^host: //p')" "$1" -- -max_total_time="$2"
>   ```
>
> - **`.pre-commit-config.yaml`** (T03 hand-back). The `fmt-check` hook's `files:` is
>   `^(crates/.*\.rs|rustfmt\.toml)$`. It becomes `^((crates|fuzz)/.*\.rs|rustfmt\.toml)$`,
>   so that a commit touching only a fuzz target still runs the recipe.
> - **`.github/workflows/fuzz.yml`** (new). It has the shape of T14's `mutants.yml`: a
>   weekly `schedule` and `workflow_dispatch`, `contents: read`, and the setup action. A
>   `matrix` of target names runs `just fuzz <target> 300`, then `actions/upload-artifact`
>   of `fuzz/artifacts/<target>` under `if: failure()`. The matrix sets
>   `strategy: fail-fast: false`. Without it, the first target that crashes cancels its
>   siblings before their 300 s and their uploads, and a run that finds one crash leaves
>   the other parsers unfuzzed. The corpus starts empty on each run. Caching
>   `fuzz/corpus` is the refinement, if runs stop finding coverage.
> - **Unchanged.** `taplo.toml`: taplo walks `fuzz/Cargo.toml`, which is taplo-clean.
>   `.gitignore`: the generated `fuzz/.gitignore` covers target, corpus, artifacts and
>   coverage.
> - **Documentation.**
>   - `docs/reference/testing.md`: a "Fuzzing" section. It says that the fuzz crate sits
>     outside the workspace and its lint table, and carries its own `forbid` table.
>     `clippy`, `fmt-check` and `lock-check` reach it by `--manifest-path`.
>   - `docs/reference/quality-gates.md`: the `lock-check`, `fmt-check` and `clippy` rows
>     name the fuzz crate.
>   - `docs/reference/commands.md`: the `just fuzz` row, and the `sync`, `lock`,
>     `lock-upgrade` and `lock-check` rows, which name the fuzz lockfile too.
>   - `docs/how-to/develop-locally.md`: two rows in Prerequisites. A C++ compiler, which
>     the gate needs for libfuzzer-sys's build script: Xcode's command-line tools on
>     macOS, and `g++` on Ubuntu, which the runner image carries. And Rosetta 2 on Apple
>     silicon, for `just fuzz` only (`softwareupdate --install-rosetta`), because the
>     cargo-fuzz binary is x86_64.
>   - `docs/reference/configuration.md`: the `tools.txt` row, today "one line, cargo-hack".
>   - `docs/project/repository-map.md`: the new top-level `fuzz/`.
>   - `docs/how-to/maintain-dependencies.md`: that `fuzz/Cargo.lock` and the `tools.txt`
>     line move too.
>   - `CHANGELOG.md` and the index row.
>
> **Before starting.**
>
> - Re-check that `--sanitizer none` still builds on the pin. It is the implementation's
>   behaviour, documented only in cargo-fuzz's unmerged PR #440.
> - Re-check whether a later cargo-fuzz release publishes an `aarch64-apple-darwin`
>   archive. If one does, drop the Rosetta row, and drop the macOS reason from the
>   recipe's `--target` comment.
> - Re-run the offline proof in S03's (c) with the real target: the lint table and the
>   `fuzz_target!` expansion under `just clippy`, and a planted `unsafe {}` failing it.

**Step 6.** `status: done`, and one commit on the ticket branch. Nothing is pushed.

**The Verification block.** Run after the notes were written:

```text
$ grep -n 'mutants.out' .gitignore
48:mutants.out*/
$ grep -n 'exclude' taplo.toml
2:exclude = ["target/**", ".tools/**", ".pixi/**", "ai_tmp/**"]
$ rustup toolchain list
stable-aarch64-apple-darwin
1.98.1-aarch64-apple-darwin (active)
$ cargo nextest list --workspace --locked 2>/dev/null | tail -1
pawdoku::random the_same_seed_draws_the_same_stream
$ git status --porcelain
 M tickets/S03-bench-fuzz-mutants.md
```

`cargo nextest list`'s last line is a test name, not a count, so step 3 records the
count: 19 tests, or 22 with `--all-features`.

### Deviations, and why

- **Branch name.** The Supacode worktree is on `S03-bench-fuzz-mutants`, not the
  `ticket/s03-bench-fuzz-mutants` the frontmatter names, as in earlier tickets. The
  `branch:` field is left as written.
- **Two of the Goal's premises changed.**
  - Fuzzing does not need nightly. The maintainer settled it on the evidence in (c).
  - cargo-mutants' configuration lives in `.cargo/mutants.toml`, and no root
    `mutants.toml` is read. Step 2 and step 4 name `mutants.toml`, and T14 needs neither
    file today.
- **The Goal's table is left as written**, as the record of what was believed on
  2026-09-23. Step 3 carries the completed table.
- **cargo-mutants is adopted now, not on a trigger.** The spike recommended a trigger,
  and the maintainer chose now (Verdicts).
- **No `taplo.toml` or `.gitignore` hand-back for fuzzing.** Step 4 expected both, and
  the scratch run showed that neither is needed. T14 rewords one `.gitignore` comment.
- **More than the ticket named was run.** The ticket asked for `--list` and documentation
  answers. Everything below ran in scratch copies, each with the maintainer's
  authorisation:
  - a full cargo-mutants run with both test tools, and the nextest-configuration probe;
  - a divan bench, and the gate's commands over it;
  - `cargo fuzz init` from two directories, a stable build, and two stable fuzz runs, one
    with a planted panic.
- **`just initialize` ran in this worktree** at the maintainer's direction, so that
  `cargo nextest list` and the commit hook have their tools. It changed no tracked file.
  Being a secondary worktree, it skipped `install-hooks`.
- **Step 2(e) is answered in part.** Recorded: the runner image's Rust, rustup and the
  missing nightly, and rustup's override precedence, with sources. Not run: whether
  `rustup toolchain install nightly --profile minimal` inside a job leaves
  `rust-toolchain.toml` alone. That it does is inference, because the command writes to
  `RUSTUP_HOME`. No nightly was installed. The maintainer's choice of the stable path
  (open point 1) removed nightly from every recipe and job, so nothing depends on the
  answer. The acceptance criterion "Answers (a) to (e) are recorded" holds with this one
  stated exception. A follow-up that brings nightly back runs the command first.
- **Two files, not one.** Files touched named this ticket alone. The first review round
  noted that S03's row in `tickets/README.md` still read `open`. S01 and S02 each set
  their row after review, so the row is set to `done`, and the Files touched table
  carries it. The acceptance criterion that `git status --porcelain` lists only this
  file held for the first commit.
- **The fuzzing draft gained gate lines in the first review round.** The draft first left
  the fuzz crate outside every gate command. Reviewers pointed out that invariant 4 then
  did not reach it, and that `fuzz/Cargo.lock` could go stale with `check` green. The
  maintainer chose to bring it in. It gets its own `forbid` table, and `--manifest-path`
  lines go in the recipes that Handed back lists under T00. That was proved in (c). It
  costs the gate a C++ compiler.

### Handed back

- **To T14** (drafted above, ready now): the recipe, the pin, the workflow, the
  `.gitignore` comment, and the pages, including the quality-philosophy paragraph
  from open point 2.
- **To the benchmark and fuzzing tickets** (drafted above, each on its trigger): the
  dependencies, the layout, the recipes and the workflow.
- **To T00 follow-ups on `main`:** the `mutants`, `bench` and `fuzz` recipes; the fuzz
  crate's `--manifest-path` lines in `sync`, `lock`, `lock-upgrade`, `lock-check`,
  `format`, `fix`, `fmt-check` and `clippy`; the cargo-mutants pin in `pyproject.toml`
  and `pixi.lock`; and the `cargo-fuzz@0.13.2` line in `tools.txt`.
- **To T02:** `divan` in `[workspace.dependencies]` and the crate's dev-dependencies and
  `[[bench]]`; `exclude = ["fuzz"]` in `[workspace]`; and `libfuzzer-sys` and the
  `[lints.rust]` table in the fuzz crate. Each dependency carries its decision 0007 note
  above.
- **To T03:** the `mutants.out*/` comment in `.gitignore`, and `fuzz/` in the
  `fmt-check` hook's `files:` pattern in `.pre-commit-config.yaml`.
- **To T18:** once `fuzz/` exists, the dependency updater needs `/fuzz` as a second cargo
  directory, and T14's pin joins the pixi manifest it already watches.
- **To the index:** nothing. S03's row in `tickets/README.md` was set to `done` in the
  first review round (Deviations).

### Open points settled

- **Nightly for fuzzing, or a stable path.** Stable. On 2026-09-27 the maintainer chose
  the pinned 1.98.1 with `--sanitizer none` for the recipe and the scheduled job. It is
  proved in (c) to build, run and catch a panic. Nightly is never a prerequisite, and
  `cargo +nightly fuzz run` with AddressSanitizer stays an optional extra with no recipe.
  The stable path is cargo-fuzz's behaviour rather than its documentation (PR #440 is
  unmerged), so the fuzzing ticket re-checks it before starting.
- **Whether mutation results feed the coverage-floor question.** Yes, as a finding and
  not as a change to the floor. The maintainer agreed on 2026-09-27. T14 writes the
  paragraph into `docs/explanation/quality-philosophy.md`: a mutant that survives on a
  covered line is what the floor cannot see. The floor stays at 90. It gets no mutation
  score beside it, because cargo-mutants is a report and decision 0009's "What would
  reopen this" names only a mutation-testing gate.

## Open points

Both are settled; the answers are under "Open points settled" above, and the questions
stay here as written.

- Whether a nightly on the maintainer's machine is acceptable as a prerequisite for
  fuzzing alone, or whether fuzzing waits for a stable path (verify whether cargo-fuzz
  now has one).
- Whether mutation results feed the coverage-floor question (a surviving mutant in a
  covered line is what the floor cannot see); a finding for
  `docs/explanation/quality-philosophy.md`, T08's page, if the maintainer agrees.
