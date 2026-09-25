---
id: T02
title: "Rust quality gate: tool configs, the no_std crate skeleton, every Rust recipe green"
status: open
depends_on: [T00]
parallel_with: [T01, T03, T04, T05, T06, T07, T08, T09]
branch: ticket/t02-rust-gate
estimated_size: L
---

# T02: Rust quality gate: tool configs, the no_std crate skeleton, every Rust recipe green

## Context

T00 has merged: the workspace exists, `crates/pawdoku/src/lib.rs` holds one documented,
tested constant, the tool configuration files are one-line stubs, `Cargo.lock` lists one
package, and `just check` is green (T00, step 4 and step 5). This lane turns that skeleton
into the Rust quality gate CONVENTIONS.md §8 decision 0009 describes, and adds the one
piece of engine code every later module depends on: the randomness boundary
(CONVENTIONS.md §1 decision 5, §9). Read `CONVENTIONS.md` in full first; §2, §4, §8, §12
and §13 matter most here. Then `tickets/README.md` for the worktree rules, then
`T00-foundation.md` for what the skeleton looks like.

Three things make this lane different from the other eight:

- It is the only lane that may add a dependency (CONVENTIONS.md §11): `thiserror`
  (errors), `serde` (optional, behind a feature) and `proptest` (dev). `Cargo.lock` has one
  owner, and this is it. T00 declared none, not even in `[workspace.dependencies]`
  (cargo-shear flags an unused entry); step 3 adds the three.
- It exercises every `cargo` recipe in the `Justfile` for the first time on real code,
  and so it carries seven of CONVENTIONS.md §12's unverified claims. The `Justfile` and
  `tools.txt` are frozen; a recipe flag this lane finds wrong is handed back as a T00
  follow-up pull request on `main`, never edited here (§11, §13).
- It defines the contract other lanes and later tickets code against. T06 rewrites the
  specs' "randomness port" into "the randomness boundary" (§9); T08's architecture page
  describes it; S04 decides how it crosses each binding. What this lane ships must be no
  more than the specification states, so nothing here has to be undone.

The specification, at G `78d03cdf` (`/Users/scutting/projects/pawdoku`; read with
`git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/specs/<name>`, never the
working file):

- `human-solving.allium` lines 39-40: "The random generator. Draws come from this game's
  randomness port; what this module states is how a draw becomes a decision."
- Line 192: `seed: Integer -- 0..2147483647` on `AttemptRequest`.
- Lines 308-312: `value Draw { index: Integer, purpose: String, value: Decimal }`.
- Lines 476-478, in `config`: "Names the randomness port's seeded stream. The generator is
  the port's; a different generator is a different name." followed by
  `random_version: String = "seeded-stream-1"`.
- Lines 1091-1099, `@guarantee ExactReplay`: identical inputs, seed, `model_version`,
  `catalogue_version` and `random_version` yield identical draws and outcome; "Draws are
  the randomness port's: a stream of u in [0,1), begun from the seed, indexed from zero,
  the same for the same seed and random_version. Which generator is the port's and not
  stated here; a test supplies the draws through the port's fake."
- Lines 1100-1102: how a draw becomes a Bernoulli outcome, a uniform index or a weighted
  choice. That is the consumer's arithmetic, not the boundary's.
- `lapse.allium` lines 45-47: slips "need chance, which this game reaches through its
  randomness port, and they are human-solving.allium's, which draws them by seed."

So the boundary is: a stream of `f64` draws in `[0, 1)`, begun from a seed, indexed from
zero, the same for the same seed and `random_version`; the library ships one generator
and names it; a test supplies draws through a fake. Everything else in those lines
(the `0..2147483647` seed range, `purpose`, the Bernoulli and weighted rules) belongs to
the ticket that implements `human-solving`.

## Goal

On `ticket/t02-rust-gate`: the final `Cargo.toml`, `crates/pawdoku/Cargo.toml`,
`rustfmt.toml`, `clippy.toml`, `taplo.toml`, `deny.toml` and `.config/nextest.toml`; a
`random.rs` module holding the `RandomStream` trait, the library's `SeededStream`
generator named by `RANDOM_VERSION`, the `ReplayStream` fake and a `RandomError`; two
integration test files; `Cargo.lock` grown by the three dependencies' trees; every Rust
recipe green when run on its own, `just check` green, line coverage at or above 90 with
the number recorded, and every CONVENTIONS.md §12 claim assigned to T02 answered in the
hand-back notes.

## Non-goals

- No engine module beyond `random.rs`: no grid, no solver, no technique. `SIDE` stays
  where T00 put it (Open points).
- No `rand`, `getrandom`, `libm` or any crate other than the three named. A later ticket
  that needs one hands it back here with the decision-0007 note.
- No edit to `Justfile`, `tools.txt`, `rust-toolchain.toml`, `_typos.toml` (T03),
  `.gitignore` (T03), CI (T04), any handbook page (T07, T08) or decision record (T09).
  Decision 0008 and 0009 are T09's to write from this ticket's hand-back notes.
- No `.cargo/config.toml` (CONVENTIONS.md §4): `RUSTFLAGS` there changes fingerprints and
  thrashes `target/` against rust-analyzer; `RUSTDOCFLAGS` is set on the `doc` recipe only.
- No consumer-side arithmetic (Bernoulli, uniform index, weighted choice), no `Draw`
  record with a `purpose`, no seed-range validation, no trace of consumed draws.

## Files touched

This lane owns exactly these (CONVENTIONS.md §3). Nothing else is edited; a need
elsewhere is a hand-back.

| Path | Class | Content |
| --- | --- | --- |
| `Cargo.toml` | T00 skeleton -> final | step 3: lints, profiles, workspace dependencies |
| `Cargo.lock` | generated | step 4: `thiserror`, `serde`, `proptest` and their trees |
| `rustfmt.toml` | T00 stub -> final | step 5 |
| `clippy.toml` | T00 stub -> final | step 5 |
| `taplo.toml` | T00 stub -> final | step 5 |
| `deny.toml` | T00 stub -> final | step 6 |
| `.config/nextest.toml` | T00 stub -> final | step 5 |
| `crates/pawdoku/Cargo.toml` | T00 skeleton -> final | step 4: the feature and the dependencies |
| `crates/pawdoku/src/lib.rs` | T00 -> final for this lane | step 7: `pub mod random;` and the crate doc |
| `crates/pawdoku/src/random.rs` | new | step 7 |
| `crates/pawdoku/tests/api_bounds.rs` | new | step 8 |
| `crates/pawdoku/tests/random.rs` | new | step 8 |
| `tickets/T02-rust-gate.md` | ticket | `status:` and hand-back notes |

## Steps

1. Create the worktree on `ticket/t02-rust-gate` from `main` (README.md "How to pick up a
   ticket"). Confirm the preconditions T00 relied on: `just check-toolchain` prints
   `1.98.1-<host>`, and `ls .tools/bin` shows cargo-nextest, cargo-llvm-cov, cargo-deny,
   cargo-hack, cargo-shear and taplo (run `just install-tools` if the worktree is fresh;
   it is idempotent). Then, before touching anything, run `just check` on T00's stubs and
   quote its last lines in the hand-back notes: this is the baseline every later step is
   measured against.

2. Read the specification lines listed in Context at the pinned commit and confirm the
   line numbers still hold (`git -C /Users/scutting/projects/pawdoku rev-parse --short=8
   HEAD` prints `78d03cdf`; if not, the `show` form still reads the pinned text).

3. The workspace `Cargo.toml`. Replace T00's file with this content, then run `taplo fmt`
   (step 5 writes the rule that sorts dependency tables); the formatted result is the
   final file, so the key order below is not load-bearing.

   ```toml
   [workspace]
   resolver = "3"
   members = ["crates/*"]

   [workspace.package]
   edition = "2024"
   rust-version = "1.98"
   license = "Apache-2.0"
   repository = "https://github.com/steven-cutting/libpawdoku"
   homepage = "https://github.com/steven-cutting/libpawdoku"
   authors = ["Steven Cutting"]
   publish = false

   [workspace.dependencies]
   # The only three crates the core may use (decision 0007). thiserror without its std
   # feature derives core::error::Error; serde with alloc, never std.
   thiserror = { version = "2.0.17", default-features = false }
   serde = { version = "1.0.228", default-features = false, features = ["derive", "alloc"] }
   proptest = "1.11.0"

   # Every level here is `warn` so a local build never breaks mid-edit; `just clippy`
   # passes `-D warnings`, which is where a warning becomes a failure.
   [workspace.lints.rust]
   unsafe_code = "forbid"
   non_ascii_idents = "forbid"
   missing_docs = "warn"
   missing_debug_implementations = "warn"
   rust_2018_idioms = { level = "warn", priority = -1 }
   unreachable_pub = "warn"
   unnameable_types = "warn"
   unused_qualifications = "warn"
   trivial_casts = "warn"
   trivial_numeric_casts = "warn"

   [workspace.lints.rustdoc]
   broken_intra_doc_links = "deny"
   private_intra_doc_links = "deny"
   missing_crate_level_docs = "warn"
   unescaped_backticks = "warn"

   [workspace.lints.clippy]
   pedantic = { level = "warn", priority = -1 }
   cargo = { level = "warn", priority = -1 }
   missing_const_for_fn = "warn"
   use_self = "warn"
   unwrap_used = "warn"
   expect_used = "warn"
   panic = "warn"
   unreachable = "warn"
   todo = "warn"
   unimplemented = "warn"
   dbg_macro = "warn"
   print_stdout = "warn"
   print_stderr = "warn"
   std_instead_of_core = "warn"
   std_instead_of_alloc = "warn"
   alloc_instead_of_core = "warn"
   exhaustive_enums = "warn"
   exhaustive_structs = "warn"
   allow_attributes = "warn"
   allow_attributes_without_reason = "warn"
   tests_outside_test_module = "warn"
   mod_module_files = "warn"
   iter_over_hash_type = "warn"
   str_to_string = "warn"
   string_to_string = "warn"
   lossy_float_literal = "warn"
   float_cmp_const = "warn"
   same_name_method = "warn"
   # technique::TechniqueStep is the readable name; the lint would rename it.
   module_name_repetitions = "allow"
   # cargo-deny's bans.multiple-versions owns this check.
   multiple_crate_versions = "allow"

   [profile.release]
   lto = "fat"
   codegen-units = 1
   # panic stays "unwind": pyo3 turns panics into exceptions and needs it.

   [profile.test]
   opt-level = 1
   ```

   What the table deliberately does not contain, and why:

   - `clippy::indexing_slicing` is off. Grid code indexes by typed positions the sudoku
     module will define; a lint against every `[i]` buys nothing a typed index does not,
     and the two `allow(` rules below still forbid a silent slice by an untyped integer
     being waved through.
   - `unsafe_code = "forbid"` cannot be lowered by a member crate, which is the point for
     the core. A future bindings crate whose macro output trips it declares its own
     `[lints]` table instead of `workspace = true`.
   - No `libm`: `f64` multiplication and comparison are in `core`; a generator needs no
     transcendental function. A rating model that needs `ln` or `exp` adds `libm` through
     this lane's successor with the decision-0007 note.
   - `module_name_repetitions = "allow"` and `multiple_crate_versions = "allow"` are lint
     levels in a manifest, not `#[allow]` attributes; the acceptance grep is for the
     attribute.
   - `rustdoc::unescaped_backticks` is the one entry likely to be nightly-only. If `just
     doc` reports it as an unknown lint (which `-D warnings` turns into a failure), delete
     that line and record the deviation; do not weaken `RUSTDOCFLAGS`.

4. The crate manifest and the lockfile. `crates/pawdoku/Cargo.toml`:

   ```toml
   [package]
   name = "pawdoku"
   version = "0.1.0"
   description = "Classic-sudoku engine: solver, human-technique catalogue, difficulty rating, hints."
   keywords = ["sudoku", "puzzle", "solver"]
   categories = ["games", "algorithms", "no-std"]
   readme = "README.md"
   edition.workspace = true
   rust-version.workspace = true
   license.workspace = true
   repository.workspace = true
   homepage.workspace = true
   authors.workspace = true
   publish.workspace = true

   [lints]
   workspace = true

   [features]
   default = []
   serde = ["dep:serde"]

   [dependencies]
   thiserror = { workspace = true }
   serde = { workspace = true, optional = true }

   [dev-dependencies]
   proptest = { workspace = true }

   [package.metadata.docs.rs]
   all-features = true
   rustdoc-args = ["--cfg", "docsrs"]
   targets = ["x86_64-unknown-linux-gnu", "wasm32-unknown-unknown"]
   ```

   Then `cargo update --workspace` (the `lock` recipe's first line; do not run `uv lock`)
   to grow `Cargo.lock`, and `cargo fetch --locked` so every later recipe is offline.
   `cargo tree -p pawdoku --all-features -e normal --depth 1` must list `thiserror` and
   `serde` as the only direct dependencies (the full tree also shows their proc-macro
   crates and, for serde 1.0.220 or later, `serde_core`); `cargo tree -p pawdoku -e dev
   --depth 1` adds `proptest`, whose tree contains `rand` and `getrandom`. That last fact
   shapes step 6.

5. The formatter, linter and runner configurations. Each replaces T00's stub whole.

   `rustfmt.toml`, stable options only, so `cargo fmt --all --check` never needs nightly:

   ```toml
   style_edition = "2024"
   max_width = 100
   newline_style = "Unix"
   use_field_init_shorthand = true
   use_try_shorthand = true
   ```

   `clippy.toml`. The `-in-tests` keys lift the restriction lints inside `#[cfg(test)]`
   so a test may `unwrap`, `panic!` and index freely; `disallowed-types` names the two
   collections whose iteration order is nondeterministic, which no engine that promises
   `ExactReplay` may use (the core cannot name `std` anyway; this protects tests and the
   bindings crates):

   ```toml
   allow-unwrap-in-tests = true
   allow-expect-in-tests = true
   allow-panic-in-tests = true
   allow-indexing-slicing-in-tests = true
   allow-dbg-in-tests = true
   allow-print-in-tests = true
   disallowed-types = [
     { path = "std::collections::HashMap", reason = "iteration order is not deterministic; use BTreeMap" },
     { path = "std::collections::HashSet", reason = "iteration order is not deterministic; use BTreeSet" },
   ]
   ```

   A mistyped key prints a configuration error at the top of `cargo clippy`'s output; a
   quiet run is the pass (§12).

   `taplo.toml`. The `[[rule]]` sorts only dependency tables; everything else keeps the
   author's order:

   ```toml
   include = ["**/*.toml"]
   exclude = ["target/**", ".tools/**", "ai_tmp/**", ".venv/**"]

   [formatting]
   column_width = 100
   indent_string = "  "
   align_entries = false
   reorder_keys = false
   reorder_arrays = false

   [[rule]]
   include = ["Cargo.toml", "crates/*/Cargo.toml"]
   keys = ["dependencies", "dev-dependencies", "build-dependencies", "workspace.dependencies"]

   [rule.formatting]
   reorder_keys = true
   ```

   Confirm that `.config/nextest.toml` is in `taplo fmt --check`'s file list (a deliberate
   misformat there must fail the check); if `**` does not enter dot-directories, add
   `.config/*.toml` to `include`.

   `.config/nextest.toml`. `nextest-version` must equal the `cargo-nextest` pin in
   `tools.txt`; if the two disagree, `tools.txt` wins and the mismatch is a hand-back. The
   `ci` profile is for T04, which selects it with `--profile ci`:

   ```toml
   nextest-version = "0.9.146"

   [profile.default]
   fail-fast = true

   [profile.ci]
   fail-fast = false
   retries = 0

   [profile.ci.junit]
   path = "junit.xml"
   ```

   Run `just format` once all four exist, then `just fmt-check` and `just toml-check`.

6. `deny.toml`, cargo-deny 0.20 schema. Licences, bans and sources run inside `just
   check`; advisories need the RustSec database over the network and run only from
   `just audit` (CONVENTIONS.md §4). The ban on `getrandom` and `rand` is decision 5 made
   mechanical: the core takes a seed and never sources entropy; the wrappers entry
   pre-authorises the CLI crate S04 will add, and only it.

   ```toml
   [graph]
   all-features = true
   # Dev-dependencies are outside the graph: proptest depends on rand and getrandom,
   # and the ban below is on what ships, not on the test harness.
   exclude-dev = true
   targets = [
     "x86_64-unknown-linux-gnu",
     "aarch64-apple-darwin",
     "x86_64-pc-windows-msvc",
     "wasm32-unknown-unknown",
   ]

   # Network: `just audit`, never `just check`.
   [advisories]
   yanked = "deny"
   unmaintained = "workspace"
   ignore = []

   [licenses]
   allow = [
     "Apache-2.0",
     "Apache-2.0 WITH LLVM-exception",
     "MIT",
     "BSD-2-Clause",
     "BSD-3-Clause",
     "ISC",
     "Zlib",
     "0BSD",
     "Unicode-3.0",
     "Unlicense",
     "CC0-1.0",
   ]
   confidence-threshold = 0.9
   private = { ignore = true }

   [bans]
   multiple-versions = "warn"
   wildcards = "deny"
   highlight = "all"
   deny = [
     { crate = "getrandom", reason = "core takes a seed; only the CLI may source entropy", wrappers = ["pawdoku-cli"] },
     { crate = "rand", reason = "same", wrappers = ["pawdoku-cli"] },
     { crate = "openssl-sys", reason = "no native TLS anywhere in this workspace" },
   ]

   [sources]
   unknown-registry = "deny"
   unknown-git = "deny"
   allow-registry = ["https://github.com/rust-lang/crates.io-index"]
   allow-git = []
   ```

   `exclude-dev = true` is the one line this ticket adds to the design's table, because
   without it the `rand` ban fires on `proptest`'s tree and gate 12 fails; it also means
   dev-dependency licences are not checked, which is acceptable for code that never
   ships. If cargo-deny 0.20.2 rejects the key, the fallback is `proptest`, `rand` and
   `rand_core` added to both `wrappers` lists; record whichever was needed. A warning that
   an allowed licence was not encountered is expected on a tree this small and does not
   fail the recipe; do not trim the list to silence it.

7. The randomness boundary: `crates/pawdoku/src/random.rs`, reached through
   `pub mod random;` in `lib.rs`. `lib.rs` keeps T00's `#![no_std]`, `extern crate alloc;`,
   `#[cfg(test)] extern crate std;`, `SIDE` and its tests, and its crate-level doc gains
   one sentence pointing at `random` as the one effect. The module, in this order, every
   public item with a doc comment and a runnable example (they are the doctests):

   - `pub const RANDOM_VERSION: &str = "seeded-stream-1";` The name of the generator
     the library ships, equal to the specification's default at line 478 so that G's
     restated `config.random_version` and this constant agree. The doc says: a different
     generator is a different name, and the golden test below is what enforces it.
   - `#[derive(Debug, Clone, PartialEq, thiserror::Error)] #[non_exhaustive] pub enum RandomError`
     with two variants: `Exhausted { index: u64 }`, `Display` text exactly
     `the replay stream is exhausted at draw {index}`; and `OutOfRange { index: u64, value: f64 }`,
     `Display` text exactly `draw {index} of the replay script is {value}, outside [0, 1)`.
     No `Eq`, because of the `f64`. The texts are stable API: pyo3 and wasm-bindgen build
     their exceptions from them (CONVENTIONS.md §7, invariant 3).
   - `pub trait RandomStream` with two methods and nothing else, dyn-compatible so a
     binding can hold `Box<dyn RandomStream>`:
     `fn next_draw(&mut self) -> Result<f64, RandomError>` returns the draw at the current
     index, in `[0, 1)`, and advances the index by one; `fn index(&self) -> u64` returns
     the index the next draw will carry, zero before the first draw. The doc quotes the
     `ExactReplay` clause and says the library's stream never returns `Err`; the fake does
     when its script ends, which is what a test wants to hear rather than a panic. Each
     method carries the `# Errors` section pedantic clippy asks for.
   - `#[derive(Debug, Clone, PartialEq, Eq)] #[non_exhaustive] pub struct SeededStream`,
     with `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`,
     fields `state: u64` and `index: u64`, both private. `pub const fn new(seed: u64) -> Self`
     sets `state = seed`, `index = 0`. The generator is SplitMix64: one `u64` of state, a
     Weyl increment and two multiply-xorshift rounds per output, twenty lines of integer
     arithmetic with no seeding procedure of its own. It is chosen over xoshiro256++
     because xoshiro needs SplitMix64 anyway to expand a seed into its 256-bit state, and
     because a boundary whose whole job is `ExactReplay` wants the generator a reviewer
     can check against the published reference by hand. It is not a cryptographic
     generator and the doc says so; nothing in the specs asks for one.

     ```rust
     const INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
     const MIX_1: u64 = 0xBF58_476D_1CE4_E5B9;
     const MIX_2: u64 = 0x94D0_49BB_1331_11EB;
     /// 2^-53, exact, without a cast: `f64::EPSILON` is 2^-52.
     const UNIT: f64 = f64::EPSILON / 2.0;

     // inside `impl RandomStream for SeededStream`
     fn next_draw(&mut self) -> Result<f64, RandomError> {
         self.state = self.state.wrapping_add(INCREMENT);
         let mut z = self.state;
         z = (z ^ (z >> 30)).wrapping_mul(MIX_1);
         z = (z ^ (z >> 27)).wrapping_mul(MIX_2);
         let bits = z ^ (z >> 31);
         self.index += 1;
         #[expect(clippy::cast_precision_loss, reason = "53 bits fit f64's mantissa exactly")]
         let mantissa = (bits >> 11) as f64;
         Ok(mantissa * UNIT)
     }
     ```

     The top 53 bits scaled by 2^-53 give every draw an exact `f64` in `[0, 1)`, and a
     multiply by a power of two is exact on every IEEE 754 target, so the golden below
     holds on wasm as it does on the host. This `expect` is the one lint attribute the
     module should need; `#[allow]` is forbidden by the lint table and by invariant 6.
   - `#[derive(Debug, Clone, PartialEq)] #[non_exhaustive] pub struct ReplayStream`, the
     fake, also with the `serde` `cfg_attr`, fields `draws: alloc::vec::Vec<f64>` and
     `index: u64` (a `u64`, not a `usize` cursor: pedantic clippy treats `usize as u64`
     as a possible truncation, and the position into the `Vec` is derived from the
     index with `usize::try_from`, which fails only past the script's end and so folds
     into `Exhausted`). `pub fn new(draws: Vec<f64>) -> Result<Self, RandomError>` rejects
     the first value outside `[0, 1)` (a NaN is outside, because `(0.0..1.0).contains`
     is false for it) with `OutOfRange`, so a test cannot script a draw the contract
     forbids. Its `next_draw` returns the scripted values in order and `Exhausted` after
     the last; `index` returns the field.
   - Constructors and getters carry `#[must_use]` (pedantic `must_use_candidate`), and
     what can be `const fn` is (`missing_const_for_fn`).
   - `#[cfg(test)] mod tests` at the bottom with the unit tests that keep the floor:
     both `Display` texts verbatim; `OutOfRange` for `1.0`, `-0.0` accepted, NaN rejected;
     `Exhausted` reports the index of the missing draw; and one `proptest!` block over
     `any::<u64>()` seeds asserting the range, which is the §12 check that proptest runs
     under `extern crate std` inside a `no_std` crate.

   Every type is `Send + Sync + 'static` by construction (plain data, no references, no
   interior mutability); step 8 proves it.

8. The integration tests. `crates/pawdoku/tests/api_bounds.rs`, in full:

   ```rust
   //! Every public type crosses a binding boundary one day, so each is
   //! `Send + Sync + 'static`, `Clone` and `Debug` (`AGENTS.md`, invariant 3), and the
   //! boundary trait is usable as a trait object.

   use pawdoku::random::{RandomError, RandomStream, ReplayStream, SeededStream};

   fn assert_send_sync<T: Send + Sync + 'static>() {}
   fn assert_clone_debug<T: Clone + core::fmt::Debug>() {}
   fn assert_dyn_compatible(_: &mut dyn RandomStream) {}

   #[test]
   fn public_types_are_send_sync_and_static() {
       assert_send_sync::<SeededStream>();
       assert_send_sync::<ReplayStream>();
       assert_send_sync::<RandomError>();
   }

   #[test]
   fn public_types_are_clone_and_debug() {
       assert_clone_debug::<SeededStream>();
       assert_clone_debug::<ReplayStream>();
       assert_clone_debug::<RandomError>();
   }

   #[test]
   fn the_boundary_is_a_trait_object() {
       let mut stream = SeededStream::new(1);
       assert_dyn_compatible(&mut stream);
   }

   #[cfg(feature = "serde")]
   #[test]
   fn streams_serialise_under_the_serde_feature() {
       fn assert_serde<T: serde::Serialize + serde::de::DeserializeOwned>() {}
       assert_serde::<SeededStream>();
       assert_serde::<ReplayStream>();
   }
   ```

   No `serde_json`: the bound check is the test, because a fourth dependency is not
   this ticket's to add. Every new public type gets a line in each of the first two
   functions; that is the rule T08's testing page will state.

   `crates/pawdoku/tests/random.rs` holds, as plain `#[test]` functions and one
   `proptest!` block:

   - Determinism: two `SeededStream::new(2024)` produce identical bit patterns for the
     first 1000 draws, and `index()` counts from 0 to 1000.
   - The golden: the first eight draws of `SeededStream::new(0)`, compared with
     `to_bits()` against these constants, computed on 2026-09-23 from the SplitMix64
     reference algorithm (the raw first output for seed 0 is `0xE220_A839_7B1D_CDAF`, the
     value the published reference prints). The executing agent recomputes them with an
     independent ten-line script in the scratch directory before pinning; a mismatch is
     a bug in the module, not in the table.

     | Index | Draw | `to_bits()` |
     | --- | --- | --- |
     | 0 | 0.8833108082136426 | `0x3FEC_4415_072F_63B9` |
     | 1 | 0.43152799704850997 | `0x3FDB_9E27_9AA8_6E58` |
     | 2 | 0.026433771592597743 | `0x3F9B_1174_6200_2500` |
     | 3 | 0.9708819781538285 | `0x3FEF_1177_150E_4990` |
     | 4 | 0.10634669156721244 | `0x3FBB_3989_6A51_A870` |
     | 5 | 0.32732576421812576 | `0x3FD4_F2E7_C31D_1FA8` |
     | 6 | 0.17386786595968284 | `0x3FC6_414D_5F0F_A298` |
     | 7 | 0.771546556331567 | `0x3FE8_B082_6759_22D5` |

     The test's doc comment says why it exists: a changed generator changes these bits,
     and whoever changes them must change `RANDOM_VERSION` in the same commit, because
     `ExactReplay` promises the same draws for the same seed and name.
   - Replay: `ReplayStream::new(vec![0.25, 0.5, 0.75])` yields those three at indices
     0, 1, 2 and then `Err(RandomError::Exhausted { index: 3 })`; a stream passed as
     `&mut dyn RandomStream` behaves the same.
   - Range: the first 10 000 draws of seeds 0, 1 and `u64::MAX` satisfy
     `(0.0..1.0).contains(&u)`.
   - The proptest: for `seed in any::<u64>()` and `n in 1usize..256`, two fresh streams
     agree bit for bit on `n` draws and every draw is in range.

   Comparisons of draws use `to_bits()` throughout, so no float equality lint is ever
   in question. If `clippy::tests_outside_test_module` fires on the top-level `#[test]`
   functions of these two files, add `#![expect(clippy::tests_outside_test_module,
   reason = "an integration test file is its own test module")]` at the top of each;
   add it only if it fires, because an unfulfilled `expect` is itself a warning.

9. Run each recipe on its own and quote the last lines of every output in the hand-back
   notes, in this order: `just fmt-check`, `just toml-check`, `just clippy`,
   `just features`, `just wasm-check`, `just test`, `just coverage`, `just doc`,
   `just deny`, `just deps-unused`, `just lock-check`. Fix what fails in this lane's
   files. A failure that can only be fixed by a `Justfile` or `tools.txt` change is not
   fixed: write it up under "Handed back" with the exact flag and the evidence, and
   continue with the remaining recipes. Then `just check`, and record its wall-clock time
   beside T00's.

10. Verify the CONVENTIONS.md §12 claims assigned to T02, each with the command run and
    the outcome, under "What was verified, and how":

    - **Lock drift.** Copy the worktree to the scratch directory, change `version = "0.1.0"`
      to `"0.1.1"` in the copy's `crates/pawdoku/Cargo.toml`, run
      `cargo update --workspace --locked` there: non-zero exit with a message naming the
      lockfile. In the real worktree the same command exits 0 and prints nothing about
      updates. If the copy exits 0, the fallback `cargo metadata --locked --format-version 1`
      is tested the same way and the `lock-check` recipe is handed back.
    - **Coverage floor and doctests.** `cargo llvm-cov report --fail-under-lines 90` after
      `cargo llvm-cov nextest --no-report` enforces the floor: run it once with the floor (exit 0) and once with a value one
      above the percentage it reported (non-zero exit), and quote both. `cargo llvm-cov --doctests` on the pinned stable toolchain must
      refuse with a message naming nightly; quote it.
    - **deny offline.** `CARGO_NET_OFFLINE=true cargo deny --offline --locked check
      licenses bans sources` after `just sync`: exit 0. Then `cargo deny --offline --locked
      check advisories`: it fails for want of the database, which is the proof that
      advisories is the only network subcommand.
    - **Feature powerset.** `cargo hack check -p pawdoku --feature-powerset --locked`
      prints two configurations (`--no-default-features` and `--features serde`).
    - **cargo-shear.** In the scratch copy, add `libm = "0.2.15"` to
      `[workspace.dependencies]` and run `cargo shear` there: it must name the unused
      workspace entry. On the real tree it prints nothing to fix. Confirm
      it made no build (`target/` timestamps unchanged, or the run is sub-second).
    - **`wasm32v1-none` on an `alloc`-using crate.** T00 checked the target on a crate
      holding one constant; `ReplayStream`'s `Vec` makes this the first crate here that
      uses `alloc`, so `just wasm-check` green on `wasm32v1-none` under both feature
      configurations is the real test of §12's second claim. Quote the two result lines.
    - **thiserror and proptest under `no_std`.** `just wasm-check` green on both targets
      proves the first; the in-module `proptest!` running under `cargo nextest run` proves
      the second.
    - **clippy.toml keys.** `cargo clippy --workspace --all-targets --all-features --locked
      -- -D warnings` prints no configuration error for any of the six `-in-tests` keys
      or `disallowed-types`. To prove the check bites, misspell one key in the scratch copy
      and quote the error clippy prints there.

    A claim that fails is a design change: record it, hand it back to `CONVENTIONS.md`
    through a `main` pull request, and do not paper over it in this lane's files.

11. Confirm the acceptance criteria one by one, set `status: done`, and commit on the
    ticket branch in a few commits with short imperative subjects (the configs, the
    module and tests, the lockfile, the ticket). Stop before pushing.

## Acceptance criteria

- `just fmt-check`, `just toml-check`, `just clippy`, `just features`, `just wasm-check`,
  `just test`, `just coverage`, `just doc`, `just deny`, `just deps-unused` and
  `just lock-check` each exit 0 on their own; `just check` exits 0 and ends with the
  clean-worktree line.
- `just coverage` reports line coverage at or above 90 over `crates/pawdoku/src/**`; the
  actual number is written in the hand-back notes.
- `just wasm-check` passes for `wasm32-unknown-unknown` and `wasm32v1-none` under both
  feature configurations.
- `Cargo.lock` lists `thiserror`, `serde` and `proptest` with their trees, and nothing
  else new; `cargo tree -p pawdoku -e normal --all-features --depth 1` lists exactly
  `thiserror` and `serde` as direct dependencies.
- `rg -n 'allow\(' crates/` prints nothing; every lint attribute in `crates/` is
  `#[expect(..., reason = "...")]`.
- `just doc` completes with no warning under `RUSTDOCFLAGS="-D warnings --cfg docsrs"`, and
  every public item in `random.rs` has a doc example that `cargo test --doc` runs.
- `just deny` is green with the licence allow list as written; any not-encountered
  warnings are quoted, not silenced.
- `just test` reports every test in `tests/api_bounds.rs`, `tests/random.rs` and the
  in-module test modules passing, and the doctest count is at least the number of public
  items in `random.rs` plus T00's one.
- The seven §12 claims are recorded with commands and outcomes; each `Justfile` or
  `tools.txt` change they force is under "Handed back", and none was made here.
- `git status --porcelain` is empty after `just check`, and nothing outside this lane's
  Files touched changed (`git diff --stat main` lists only those paths).
- Nothing was pushed.

## Verification

```sh
just check-toolchain
mkdir -p ai_tmp/t02
for r in fmt-check toml-check clippy features wasm-check test coverage doc deny deps-unused lock-check; do echo "== $r"; if just "$r" >"ai_tmp/t02/$r.log" 2>&1; then tail -4 "ai_tmp/t02/$r.log"; else tail -20 "ai_tmp/t02/$r.log"; echo "FAILED $r"; break; fi; done
time just check
git status --porcelain
git diff --stat main -- . ':!tickets'
cargo tree -p pawdoku --all-features -e normal --depth 1 --locked
cargo nextest run --workspace --all-features --locked 2>&1 | tail -3
cargo test --doc --workspace --all-features --locked 2>&1 | grep -E 'test result'
cargo llvm-cov report 2>&1 | tail -3
rg -n 'allow\(' crates/ || echo "no allow attributes"
rg -n 'pub const RANDOM_VERSION' crates/pawdoku/src/random.rs
CARGO_NET_OFFLINE=true cargo deny --offline --locked check licenses bans sources 2>&1 | tail -3
```

Expected: the toolchain line names `1.98.1`; every `== <recipe>` block ends with a
success line and no `FAILED` line prints; `just check` ends with the clean-worktree line
and exit 0; `git status --porcelain` prints nothing; the diff stat lists only the twelve
paths in Files touched; the depth-1 tree shows `thiserror` and `serde` and no other direct dependency; nextest
reports every test passed with a count of at least twelve; the doctest line reports at least
the public-item count passed and 0 failed; the report's `TOTAL` line shows a lines percentage
of 90.00 or more; the `rg` for `allow\(` prints `no allow attributes`; the constant's
definition is found once; the offline deny run ends with `licenses ok, bans ok,
sources ok`. Quote each in the hand-back notes.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether `SIDE` stays in `lib.rs` or moves now to a `grid` module. Recommendation: keep
  it where T00 put it until the ticket that implements `sudoku.allium` creates the module
  that owns `config.side`; moving a one-line constant twice serves nothing, and this lane
  owns no `grid.rs`.
- Whether `wasm32v1-none` is installed for 1.98.1 on the executing machine. T00 verified
  it on the stub crate; if `rustup target list --installed` no longer shows it, the fix
  is `just install-toolchain`, and a target that has vanished from the channel is a
  design change for `CONVENTIONS.md` §2.
- Whether `clippy::cargo_common_metadata` wants a field T00 did not set. T00 set
  `description`, `license`, `repository`, `readme`, `keywords` and `categories`; if the
  lint asks for `documentation` or anything else, add it in `crates/pawdoku/Cargo.toml`
  and record it, since that file is this lane's.
- Whether `rustdoc::unescaped_backticks` is accepted by stable rustdoc 1.98.1 or is
  nightly-only, and so whether the line survives step 3.
- Whether `[profile.test] opt-level = 1` disturbs line attribution under llvm-cov. If the
  reported number is implausible against a reading of the tests, rerun with the profile
  line removed in the scratch copy and compare; keep the profile if the two agree.
- Nightly-only rustfmt options deliberately not used: `wrap_comments`,
  `format_code_in_doc_comments`, `imports_granularity`, `group_imports`,
  `normalize_doc_attributes`. Each would be a `rustfmt +nightly` dependency for every
  commit hook and CI step; the trade is recorded here so nobody adds one by habit.
- Whether `wrappers = ["pawdoku-cli"]` naming a crate not yet in the workspace makes
  cargo-deny 0.20.2 warn. A warning is quoted and kept; an error means the two wrappers
  entries are dropped until S04 adds the crate, and that is recorded.
