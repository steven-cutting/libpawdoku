---
title: "Configuration"
kind: "reference"
audience: [contributor, maintainer, operator, agent]
canonical_for: [configuration_reference]
requires: []
---

# Configuration

A library has no runtime configuration: no file it reads, no variable it consults, no
flag it parses. What a consumer can choose is a Cargo feature, and the crate has one,
`serde`, which is optional and additive. There are no default features. Everything else on
this page configures the tools that build and check the crate.

## Environment

The crate reads no environment variable; it is `no_std` and cannot name the environment.
The tooling reads a few, each in one place:

| Variable | Read by | Effect |
| --- | --- | --- |
| `RUSTDOCFLAGS` | The `doc` recipe, which sets it | `-D warnings --cfg docsrs`: warnings fail, and the build is the one docs.rs would make. Set on that recipe and nowhere else. |
| `CARGO_HOME` | The `check-toolchain` recipe | Where rustup's proxy is expected; `~/.cargo` when unset. |
| `CARGO_TERM_COLOR`, `CARGO_INCREMENTAL`, `CARGO_NET_RETRY`, `RUST_BACKTRACE` | Every CI job | Coloured logs, no incremental artefacts on a fresh runner, retries for flaky fetches, and a backtrace on a panic. Exported by `ci.yml` and `pages.yml`; `audit.yml` exports the first three. |
| `GITHUB_TOKEN` | CI's `install-tools` step only | Lifts GitHub's anonymous rate limit for cargo-binstall's release lookup in the `tools.txt` pass. The run's own token. The recipe unsets it, and `GH_TOKEN`, for the `tools-source.txt` pass, so no build script compiled in that pass can read it. |
| `PREK_HOME` | CI's setup action | Moves prek's cache outside the checkout, so the worktree snapshot never sees it and the cache can be restored between runs. |
| `INSTA_UPDATE` | insta in tests | `test`, `coverage` and `snapshots-check` set `no`: a mismatch fails without writing files. `snapshots-accept` sets `new`, then cargo-insta accepts the pending files. Outside the recipes, `auto` defaults to `no` in CI and `new` elsewhere; `always` overwrites `.snap` files. |
| `CARGO_NET_OFFLINE` | Cargo in the snapshot recipes | Set to `true` after `lock-check`, because cargo-insta cannot forward `--locked` to nextest. Dependency resolution uses the local cache. |

There is no `.cargo/config.toml`. `RUSTFLAGS` set there would change every build's
fingerprint and make rust-analyzer and the recipes rebuild `target/` against each other;
a value that matters to one recipe is set on that recipe. The same argument applies to
the code: a value baked into a build is not configuration, it is a constant, and
constants belong in source where they can be reviewed.

## Configuration files

| File | What it configures |
| --- | --- |
| `rust-toolchain.toml` | The compiler: an exact stable release, the minimal profile, four components (rustfmt, clippy, `llvm-tools-preview` for coverage, `rust-src` for rust-analyzer) and two targets, `wasm32-unknown-unknown` and `wasm32v1-none`. Exact rather than `stable`, because clippy's lint set moves every six weeks and would break `-D warnings` with no diff here. Honoured only by rustup's proxy. |
| `Cargo.toml` (root) | The workspace: members, the package fields each crate inherits (edition, `rust-version`, licence, `publish = false`), the dependency table every crate draws from, the lint table, and the release and test profiles. |
| `crates/pawdoku/Cargo.toml` | The crate: its features, its dependencies drawn from the workspace table, `[lints] workspace = true`, and the docs.rs metadata. |
| `clippy.toml` | The restriction lints relaxed inside tests only (`allow-unwrap-in-tests` and its siblings for `expect`, `panic`, indexing, `dbg!` and printing), `HashMap` and `HashSet` disallowed, because their iteration order is not deterministic (the reason names `BTreeMap` and `BTreeSet`); and four thresholds, for cognitive complexity, nesting depth, function length and parameter count, whose numbers `rustqual.toml` repeats. [Decision 0013](../decisions/0013-metrics-gate.md) owns the numbers and says why each. |
| `rustfmt.toml` | The 2024 style edition, a width of 100, Unix newlines, and the field-init and `?` shorthands. Stable options only. |
| `taplo.toml` | TOML formatting at a width of 100 with two-space indentation, keys sorted in the dependency tables of every manifest, and `target/`, `.tools/`, `.pixi/` and `ai_tmp/` excluded. The exclusion is load-bearing: the pixi environment holds TOML files of its own. |
| `deny.toml` | cargo-deny, by section, below. |
| `.config/nextest.toml` | cargo-nextest's minimum version and its profiles: `default` stops at the first failure; `ci` runs everything and writes a JUnit report. |
| `pyproject.toml` | The pixi manifest: Python, just, prek, cargo-binstall and every cargo tool conda-forge carries, including cargo-insta pinned to the insta crate's release, under `[tool.pixi.*]`, with the tooling package as a git dependency; and the tooling package's own table, `[tool.biscuit-games-tooling]`, whose `recipes` list is what `just check` runs, in order. No `rust`: rustup owns the compiler. `pixi.lock` is the pin. |
| `tools.txt` | The exception list: binaries conda-forge lacks, one `name@version` per line, installed by `just install-tools` from a release archive and never compiled. Today one line, cargo-hack. |
| `tools-source.txt` | The second exception list: tools conda-forge lacks that also have no release binary, one `name@version` per line, which `just install-tools` builds from crates.io source with `--locked` unless cargo-quickinstall has a signed build of the pin. Today one line, rustqual. |
| `rustqual.toml` | rustqual, for the `metrics` gate: the dimensions it runs (complexity, SRP, coupling, architecture, and IOSP, which cannot be switched off), their thresholds, and ten `[[architecture.pattern]]` rules, one per module of [Layering](../explanation/layering.md). Its globs resolve against the crate directory, so the recipe runs it from the root with `--config`. |
| `Justfile` | Every recipe, and the `coverage_floor` variable, the line-coverage floor `just coverage` enforces. |
| `.pre-commit-config.yaml` | The read-only hook gate `just lint` runs and `just install-hooks` installs. |
| `.pre-commit-fix.yaml` | The mutating hooks, run only by `just fix`. |
| `.markdownlint-cli2.jsonc` | markdownlint's rules and the directories it ignores. |
| `lychee.toml` | The link checker's settings and excluded paths. |
| `_typos.toml` | The only typos configuration: the excluded lockfiles and directories, and the allowlisted `mis`. A `[tool.typos]` table in `pyproject.toml` would be ignored. |
| `.editorconfig` | Whitespace for every file type, four-space indentation for Rust. |

### The lint table

`[workspace.lints]` in the root `Cargo.toml`, which every crate opts into. Every level is
`warn`, so a local build never breaks mid-edit; `just clippy` passes `-D warnings`, which
is where a warning becomes a failure. The exceptions are `forbid` and `deny`, which fail
everywhere.

| Lint | Level | Why |
| --- | --- | --- |
| `unsafe_code` | forbid | No crate may contain `unsafe`, and `forbid` means no item can opt back in. |
| `non_ascii_idents` | forbid | Identifiers read the same to every reader and tool. |
| `missing_docs` | warn | Every public item is documented; see [API reference](api.md). |
| `unreachable_pub`, `unnameable_types` | warn | What is `pub` is exported on purpose. |
| `rustdoc::broken_intra_doc_links`, `rustdoc::private_intra_doc_links` | deny | A doc link that does not resolve, or resolves to something a reader cannot see, fails. |
| `clippy::pedantic`, `clippy::cargo` | warn, priority -1 | The whole group, so a single lint can be set above it. |
| `unwrap_used`, `expect_used`, `panic`, `unreachable`, `todo`, `unimplemented` | warn | The library does not panic; an error is returned. Relaxed in tests by `clippy.toml`. |
| `std_instead_of_core`, `std_instead_of_alloc`, `alloc_instead_of_core` | warn | Each path names the smallest crate that has it, so the core stays `no_std`. |
| `exhaustive_enums`, `exhaustive_structs` | warn | A public type is `#[non_exhaustive]`, so growing it breaks no consumer. |
| `allow_attributes`, `allow_attributes_without_reason` | warn | A suppression is `#[expect(lint, reason = "...")]`, which reports itself when it stops being needed. |
| `mod_module_files` | warn | No `mod.rs`: a module is the file named after it. |
| `cognitive_complexity` | warn | A restriction lint, so the pedantic group leaves it off; `clippy.toml` sets its threshold. |
| `module_name_repetitions` | allow | The readable name repeats the module's, and the lint would rename it. |
| `multiple_crate_versions` | allow | cargo-deny's `bans.multiple-versions` owns that check. |

The table holds more, each one line: `missing_debug_implementations`, `rust_2018_idioms`,
`unused_qualifications`, the trivial-cast pair, `missing_const_for_fn`, `use_self`,
`dbg_macro`, the print pair, `tests_outside_test_module`, `iter_over_hash_type`,
`str_to_string`, `lossy_float_literal`, `float_cmp_const` and `same_name_method`, and the
rustdoc pair `missing_crate_level_docs` and `unescaped_backticks`.

### `deny.toml`

| Section | Holds |
| --- | --- |
| `graph` | Every feature, on four targets (Linux, macOS on Apple silicon, Windows and `wasm32-unknown-unknown`), with development dependencies excluded: the bans are on what ships, and proptest depends on the randomness crates. |
| `advisories` | Yanked crates denied, unmaintained crates reported for the workspace's own dependencies. Read only by `just audit`, which needs the network. |
| `licenses` | The allowed licences, all permissive and compatible with Apache-2.0, at a confidence threshold of 0.9. |
| `bans` | Wildcard versions denied; duplicate versions warned. Three crates denied: `getrandom` and `rand`, because the core takes a seed and only `pawdoku-cli` may source entropy, and `openssl-sys`, because no native TLS belongs anywhere in the workspace. |
| `sources` | crates.io only: an unknown registry or git source is denied. |

## Values the specifications decide

A figure a specification states is mirrored in code once, by name.
`pawdoku::sudoku::SIDE` mirrors `config.side` in `sudoku.allium`, the side of the grid,
and `pawdoku::sudoku::BOX_SIDE` mirrors `config.box_side`; a doctest and a unit test
hold each mirror. A Rust constant that stands for a figure but has no `config` entry
to name is drift the other way: the figure belongs in the specification first.

## Version pins

Every pin lives in exactly one file, and this page names the file rather than the number,
so a bump never touches the handbook.

| Pin | Where |
| --- | --- |
| The Rust toolchain | `rust-toolchain.toml`, with `rust-version` in the root `Cargo.toml` following it |
| Rust dependencies | `Cargo.lock`, within the caret ranges the manifests write |
| pixi's tools and Python | `pyproject.toml`, locked in `pixi.lock` |
| Tools conda-forge lacks | `tools.txt`, or `tools-source.txt` for a tool with no release binary |
| The Allium checker | The tooling package, which pins its version and checksums; `pyproject.toml` pins the package |
| Hooks | The `rev` of each remote hook in the two prek configurations, a commit SHA with a version comment |
| GitHub Actions | Each `uses:` in the workflows and the setup action, a commit SHA with a version comment |

How to move each is in [Maintain dependencies](../how-to/maintain-dependencies.md).

## Related pages

- [Commands](commands.md)
- [Quality gates](quality-gates.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
