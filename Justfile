set positional-arguments := true
set shell := ["sh", "-eu", "-c"]

# Pinned binaries no lockfile can name live in the gitignored .tools/bin (see
# tools.txt). Every recipe, and every hook that runs through a recipe, sees them
# first; cargo finds cargo-nextest and friends on PATH by name.
export PATH := justfile_directory() / ".tools" / "bin" + ":" + env("PATH")

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

# Pinned binaries into .tools/bin (network). cargo-binstall itself is a
# per-machine prerequisite that scripts/initialize.sh installs when absent.
install-tools:
    grep -v '^#' tools.txt | xargs cargo binstall --root .tools --no-confirm --locked --disable-strategies compile

# The Allium checker for docs/specs/, pinned and checksummed in the tooling
# package. Downloads over the network into .tools/bin, which Git ignores.
install-allium:
    uv run --frozen bg-install-allium

sync:
    cargo fetch --locked
    uv sync --frozen

lock:
    cargo update --workspace
    uv lock

lock-upgrade:
    cargo update
    uv lock --upgrade

lock-check:
    cargo update --workspace --locked
    uv lock --check

install-hooks:
    git rev-parse --is-inside-work-tree >/dev/null
    test -f uv.lock || { printf '%s\n' 'uv.lock is missing; run just initialize first' >&2; exit 2; }
    uv run --frozen prek install --overwrite --hook-type=pre-commit

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
    -uv run --frozen prek run --all-files --config .pre-commit-fix.yaml
    uv run --frozen prek run --all-files --config .pre-commit-fix.yaml
    cargo clippy --workspace --all-targets --all-features --locked --fix --allow-dirty --allow-staged
    just lint

# ------------------------------------------------------------------ check ---

lint:
    uv run --frozen prek run --all-files

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
coverage:
    cargo llvm-cov nextest --workspace --all-features --locked --no-report
    cargo llvm-cov report --fail-under-lines {{coverage_floor}}
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
    uv run --frozen prek run --all-files markdownlint-cli2 typos lychee
    uv run --frozen bg-validate-docs

check-agents:
    uv run --frozen bg-validate-agents

# The specifications, checked mechanically: every module must report an empty
# `diagnostics` array. The wrapper asserts that, because neither subcommand's
# exit code does. Waiver terms: docs/how-to/work-with-the-specs.md.
check-specs:
    uv run --frozen bg-run-allium check

analyse-specs:
    uv run --frozen bg-run-allium analyse

check-links-online:
    uv run --frozen prek run --all-files --hook-stage manual lychee-online

# --------------------------------------------------------------- aggregate ---

check-clean baseline="":
    uv run --frozen bg-project-check clean "$1"

# The complete gate: the recipes pyproject.toml lists, in order, with the
# worktree snapshotted between each, then check-clean.
check:
    uv run --frozen bg-project-check run
