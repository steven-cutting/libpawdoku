set positional-arguments := true
set shell := ["sh", "-eu", "-c"]

# pixi owns every tool binary and the Python environment (.pixi/envs/default;
# pyproject.toml is the manifest, pixi.lock the pin). Tools conda-forge lacks
# (tools.txt, and tools-source.txt for those with no release binary) live in
# .tools/bin, and so does the Allium checker. Every recipe, and every hook that
# runs through a recipe, sees both first; cargo finds cargo-nextest and friends
# on PATH by name. Neither directory holds a cargo, so rustup's proxy stays
# first for the compiler (check-toolchain proves it).
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

# Tools conda-forge lacks into .tools/bin (network). cargo-binstall comes from
# the pixi environment. tools.txt is binary-only: a release archive or nothing.
# tools-source.txt holds tools with no release binary, so its loop may compile,
# and cargo-binstall passes --locked through to `cargo install`. It tries a
# signed cargo-quickinstall build first, and would take one that appeared for
# the pin; --only-signed refuses an unsigned one, and an upstream release
# asset, which nothing would sign, is refused too (decision 0013). That pass runs without a GitHub token, so no build script it
# compiles can read one; its one quickinstall lookup needs no rate-limit lift.
# A pass whose list is empty is skipped, since cargo binstall with no crate
# fails; the guard is plain sh, so it holds for GNU and BSD alike. sed, not
# grep, strips the comments: it exits 0 on a list with no pins but still fails,
# and so fails the recipe, when the file is missing or unreadable.
install-tools:
    pins=$(sed -e '/^#/d' -e '/^$/d' tools.txt); [ -z "$pins" ] || cargo binstall --root .tools --no-confirm --locked --disable-strategies compile $pins
    pins=$(sed -e '/^#/d' -e '/^$/d' tools-source.txt); [ -z "$pins" ] || env -u GITHUB_TOKEN -u GH_TOKEN cargo binstall --root .tools --no-confirm --locked --disable-strategies crate-meta-data --only-signed --no-discover-github-token $pins

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
# follow through cargo test. INSTA_UPDATE=no makes a mismatch fail without a
# pending file, which would trip the gate's worktree check.
test:
    INSTA_UPDATE=no cargo nextest run --workspace --all-features --locked
    cargo test --doc --workspace --all-features --locked

test-doc:
    cargo test --doc --workspace --all-features --locked

# Gate 19: snapshots match, and every .snap file is referenced by a test.
# Run the whole suite so the orphan check sees every reference. --check and
# INSTA_UPDATE=no fail mismatches without writing pending files.
# cargo-insta 1.48.0 forwards trailing options as test arguments, so nextest
# rejects --locked. Check the locks first, offline, before cargo-insta runs.
# Doctests already have their own gate; disabling its extra run avoids a warning.
snapshots-check: lock-check
    INSTA_UPDATE=no CARGO_NET_OFFLINE=true cargo insta test --check --unreferenced reject --test-runner nextest --disable-nextest-doctest --workspace --all-features

# Interactive review for a person at a terminal.
snapshots-review:
    cargo insta review --workspace

# The snapshot recipe that writes: rerun and accept without a terminal.
# Read the diff before committing, and explain each intended change. A .snap
# file no test refers to, left by a renamed or removed test, is deleted, so the
# deletion shows in the diff instead of blocking the accept.
snapshots-accept: lock-check
    INSTA_UPDATE=new CARGO_NET_OFFLINE=true cargo insta test --accept --unreferenced delete --test-runner nextest --disable-nextest-doctest --workspace --all-features

# ----------------------------------------------------------------- format ---

format:
    cargo fmt --all
    taplo fmt

# Automatic repairs. The fix config is run twice because a
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

# Gate 7: complexity, cohesion, coupling and the module boundaries of
# docs/explanation/layering.md, through rustqual against rustqual.toml. The
# probe follows because a misconfigured architecture section is silent: the
# fixture breaks each boundary rule once, so rustqual must exit 1 (0 is no
# findings, 2 a configuration it could not read) and name every rule exactly
# once; a rule named twice matches more than the fixture breaks. `want`
# reads every `name =` line in rustqual.toml, which is right while the pattern
# rules are the only tables there with a `name` key. The version check comes
# first: rustqual's version decides what the gate reports, so a missing
# .tools/bin/rustqual must not fall through to another one on PATH.
metrics:
    #!/bin/sh
    set -eu
    pin=$(sed -n 's/^rustqual@//p' tools-source.txt)
    have=$(rustqual --version 2>/dev/null | sed -n 's/^rustqual //p')
    if [ "$have" != "$pin" ]; then
        printf 'metrics: rustqual %s on PATH, tools-source.txt pins %s; run just install-tools\n' "${have:-missing}" "$pin" >&2
        exit 1
    fi
    rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github
    status=0
    out=$(rustqual tests/fixtures/metrics-violation --config rustqual.toml --format github 2>&1) || status=$?
    want=$(sed -n 's/^name = "\(.*\)"$/\1/p' rustqual.toml | sort)
    got=$(printf '%s\n' "$out" | sed -n 's|.*architecture/pattern/\([a-z_]*\) .*|\1|p' | sort)
    if [ "$status" -ne 1 ] || [ "$want" != "$got" ]; then
        printf '%s\n' "$out" >&2
        printf 'metrics: probe exited %s, want 1\nrules named:\n%s\nrules fired:\n%s\n' "$status" "$want" "$got" >&2
        exit 1
    fi

# Every feature combination compiles. Not --no-dev-deps: that rewrites Cargo.toml
# while it runs and would trip the worktree snapshot on a crash.
features:
    cargo hack check --workspace --feature-powerset --locked

# The wasm-bindgen target, and a target with no std at all, which is what proves
# the core is no_std rather than merely happening to compile. pawdoku-wasm is
# checked on the wasm-bindgen target only: wasm-bindgen needs std, so the second
# target stays the core's proof alone.
wasm-check:
    cargo hack check -p pawdoku --feature-powerset --target wasm32-unknown-unknown --locked
    cargo hack check -p pawdoku --feature-powerset --target wasm32v1-none --locked
    cargo hack check -p pawdoku-wasm --feature-powerset --target wasm32-unknown-unknown --locked

# wasm-build's and wasm-test's first step, a recipe both depend on because a
# shell function cannot outlive the recipe that defines it. The wasm-bindgen
# CLI must be the version Cargo.lock resolves for the wasm-bindgen crate, or
# the glue it writes disagrees with the code the macro expanded; pyproject.toml
# pins the CLI, and this stops on a mismatch, naming both, before anything
# builds. cargo pkgid reads the lockfile offline and never rewrites it.
wasm-guard:
    #!/bin/sh
    set -eu
    lock=$(cargo pkgid --locked --offline -p wasm-bindgen | sed 's/.*@//')
    cli=$(wasm-bindgen --version 2>/dev/null | sed -n 's/^wasm-bindgen //p')
    if [ "$cli" != "$lock" ]; then
        printf 'wasm-guard: wasm-bindgen %s on PATH, Cargo.lock resolves wasm-bindgen %s; pin wasm-bindgen-cli to it in pyproject.toml\n' "${cli:-missing}" "$lock" >&2
        exit 1
    fi

# @steven-cutting/pawdoku-wasm into crates/pawdoku-wasm/pkg/, which Git ignores.
# Not a gate and outside `just check`: a release build with fat LTO, run by CI's
# wasm job and by release-npm.yml. --mode no-install forbids wasm-pack to fetch
# a wasm-bindgen of its own, so it uses the guarded one on PATH (decision 0020).
# lock-check follows the guard because wasm-pack's own `cargo metadata`, run
# before the build and without --locked, would rewrite a stale Cargo.lock, and
# `-- --locked` would then reach cargo with a lock already made to agree. The
# crate's metadata turns wasm-opt on, which --mode no-install skips with success
# when wasm-opt is missing, so the pinned binaryen is required first. Then the
# specification modules and the crate's changelog go into the package, and
# package.json gains files and exports.
wasm-build: wasm-guard lock-check
    #!/bin/sh
    set -eu
    have=$(wasm-opt --version 2>/dev/null || true)
    if [ "$have" != "wasm-opt version 121" ]; then
        printf 'wasm-build: wasm-opt says "%s"; want "wasm-opt version 121", the binaryen pyproject.toml pins\n' "${have:-missing}" >&2
        exit 1
    fi
    wasm-pack build crates/pawdoku-wasm --mode no-install --target web --scope steven-cutting --release -- --locked
    python -I scripts/wasm_pkg.py

# The package wasm-build wrote, as a consumer meets it; not a gate and outside
# `just check`, because it needs that release build: CI's wasm job runs it after
# wasm-build. It does not depend on wasm-build, so CI builds once; locally, run
# wasm-build first. snapshots.mjs writes each JSON snapshot of the core as a
# TypeScript literal into target/, which Git ignores; tsc holds the declarations to
# them, to a strict consumer and to a declaration module; then play.mjs loads
# pkg/ through initSync and plays. typescript's tsc script finds itself through
# CONDA_PREFIX, which pixi sets only inside `pixi run`.
wasm-package-test:
    test -f crates/pawdoku-wasm/pkg/pawdoku_wasm.d.ts || { printf '%s\n' 'wasm-package-test: no crates/pawdoku-wasm/pkg/; run just wasm-build first' >&2; exit 1; }
    node crates/pawdoku-wasm/tests/package/snapshots.mjs
    CONDA_PREFIX="{{justfile_directory()}}/.pixi/envs/default" tsc -p crates/pawdoku-wasm/tests/package/tsconfig.json --noEmit --strict
    node crates/pawdoku-wasm/tests/package/play.mjs

# Gate 20: crates/pawdoku-wasm/tests/web.rs, compiled for the wasm-bindgen target
# and run in Node by wasm-bindgen-test-runner, which cargo calls as the target's
# runner. Node and the runner are the pixi environment's.
wasm-test: wasm-guard
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p pawdoku-wasm --target wasm32-unknown-unknown --locked

# Runs pawdoku's nextest tests under instrumentation and enforces the floor,
# which is over crates/pawdoku/src/** alone; the WebAssembly crate is glue, proved
# by wasm-test. Doctests are not measured (nightly-only in llvm-cov); test-doc
# runs them uncovered.
# llvm-cov does not create the lcov file's directory, so the recipe does.
# INSTA_UPDATE=no fails a mismatch without a pending file that would trip the
# gate's worktree check.
coverage:
    INSTA_UPDATE=no cargo llvm-cov nextest -p pawdoku --all-features --locked --no-report
    cargo llvm-cov report --fail-under-lines {{coverage_floor}}
    mkdir -p target/llvm-cov
    cargo llvm-cov report --lcov --output-path target/llvm-cov/lcov.info

doc:
    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --workspace --no-deps --all-features --locked

# What GitHub Pages serves: the API reference `doc` builds, from an empty
# target/doc so that nothing stale is published, and a root index.html that
# sends a reader to the crate, because stable rustdoc writes none. Not a
# gate and outside `just check`: `doc` is the gate, and this adds one file
# to what it built. The flags are `doc`'s own, through the recipe, so the
# hosted reference is the one the gate proved.
site:
    rm -rf target/doc
    just doc
    printf '%s\n' '<!doctype html>' '<html lang="en">' '<meta charset="utf-8">' '<title>pawdoku: API reference</title>' '<meta http-equiv="refresh" content="0; url=pawdoku/index.html">' '<p><a href="pawdoku/index.html">The pawdoku API reference</a></p>' '</html>' > target/doc/index.html

# Licences, bans and sources are answerable offline once `just sync` has run.
deny:
    cargo deny --locked check licenses bans sources

# Advisories need the RustSec database over the network, so this sits outside
# `just check` for the same reason check-links-online does.
audit:
    cargo deny --locked check advisories

deps-unused:
    cargo shear

# API breaks in pawdoku against the last pawdoku-v* tag, through the pinned
# cargo-semver-checks. Not a gate: before 1.0 a break is allowed with a minor
# bump, so CI reports it and never blocks; at 1.0 the job joins check's needs.
# With no tag yet there is nothing to compare, and the recipe says so.
# cargo-semver-checks 0.51.0 has no --locked and passes none through to cargo.
semver:
    #!/bin/sh
    set -eu
    base=$(git describe --tags --match 'pawdoku-v*' --abbrev=0 2>/dev/null) || { printf '%s\n' 'semver: no pawdoku-v* tag yet, nothing to compare'; exit 0; }
    cargo semver-checks -p pawdoku --baseline-rev "$base"

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

# The test obligations allium derives from one module, as JSON: what each
# rule, invariant, transition and surface clause would have a test for. A
# suggestion list for a test list, not a gate: nothing asserts on it and it
# is outside `just check`. `allium plan` takes one file and plans that module
# alone, reading no import. The binary is named by path so that a missing
# .tools/bin/allium fails here and does not fall through to another allium
# on PATH.
plan-spec module:
    .tools/bin/allium plan "docs/specs/$1.allium"

check-links-online:
    prek run --all-files --hook-stage manual lychee-online

# --------------------------------------------------------------- aggregate ---

check-clean baseline="":
    bg-project-check clean "$1"

# The complete gate: the recipes pyproject.toml lists, in order, with the
# worktree snapshotted between each, then check-clean.
check:
    bg-project-check run
