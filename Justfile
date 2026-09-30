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
# the pin; an upstream release asset, which nothing would sign, is refused
# (decision 0013). That pass runs without a GitHub token, so no build script it
# compiles can read one; its one quickinstall lookup needs no rate-limit lift.
install-tools:
    grep -v '^#' tools.txt | xargs cargo binstall --root .tools --no-confirm --locked --disable-strategies compile
    grep -v '^#' tools-source.txt | xargs env -u GITHUB_TOKEN -u GH_TOKEN cargo binstall --root .tools --no-confirm --locked --disable-strategies crate-meta-data --no-discover-github-token

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

# Gate 7: complexity, cohesion, coupling and the module boundaries of
# docs/explanation/layering.md, through rustqual against rustqual.toml. The
# probe follows because a misconfigured architecture section is silent: the
# fixture breaks each boundary rule once, so rustqual must exit 1 (0 is no
# findings, 2 a configuration it could not read) and name every rule. `want`
# reads every `name =` line in rustqual.toml, which is right while the pattern
# rules are the only tables there with a `name` key.
metrics:
    #!/bin/sh
    set -eu
    rustqual crates/pawdoku --config rustqual.toml --fail-on-warnings --format github
    status=0
    out=$(rustqual tests/fixtures/metrics-violation --config rustqual.toml --format github 2>&1) || status=$?
    want=$(sed -n 's/^name = "\(.*\)"$/\1/p' rustqual.toml | sort)
    got=$(printf '%s\n' "$out" | sed -n 's|.*architecture/pattern/\([a-z_]*\) .*|\1|p' | sort -u)
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
