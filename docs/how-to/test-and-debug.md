---
title: "Test and debug"
kind: "how-to"
audience: [contributor, maintainer, agent]
canonical_for: [test_workflow]
requires: []
---

# Test and debug

## Run the suites

```console
just test        # nextest over unit and integration tests, then the doctests
just test-doc    # the doctests alone: nextest cannot run them
just coverage    # nextest under instrumentation, with the coverage floor enforced
just clippy      # every lint the manifests declare, as errors
```

To iterate on one test, call nextest directly with its filter language:

```console
cargo nextest run -p pawdoku -E 'test(name)'
RUST_BACKTRACE=1 cargo nextest run -p pawdoku -E 'test(name)'
```

The tools live in `.pixi/envs/default/bin` and `.tools/bin`, which the `Justfile` puts on
`PATH` for every recipe and your shell does not. From a shell, prefix
`PATH="$PWD/.pixi/envs/default/bin:$PWD/.tools/bin:$PATH"`, or use
`pixi run --frozen cargo nextest ...`.

Framework configuration and conventions are described in
[Testing](../reference/testing.md); this page is about narrowing down a failure.

## Narrow down a failing test

1. Run the single test first, by filter. A failure that only appears in the whole suite
   is usually shared state, and there is very little of it here: the engine has no
   globals, and every stream is constructed per test.
2. Read the assertion's left and right and, for a panic, the backtrace
   `RUST_BACKTRACE=1` prints. CI sets it for every job, so a failure in CI already
   carries one.
3. If the expectation is about a rule, work it through by hand against the module under
   `docs/specs/` that states it. The specification is the arbiter, not the current code.
4. Do not weaken an assertion to make it pass. If the specification is wrong, change the
   specification — see [Work with the specifications](work-with-the-specs.md).
5. A proptest failure writes a regression file beside the test, under a
   `proptest-regressions/` directory. Commit it: it is the one input that failed, and
   the suite replays it first from then on.

## Debug a coverage failure

`just coverage` prints the summary per file and fails below the floor. For the lines
themselves, render the HTML report, which lands at `target/llvm-cov/html/index.html`:

```console
cargo llvm-cov report --html
```

That reuses the profile data a `just coverage` run left behind. From cold,
`cargo llvm-cov nextest --workspace --all-features --locked --html` does both at once.
Two cases look alike and are not:

- **Untested behaviour.** Add the test. This is the common case.
- **Unreachable code.** A defensive branch no input can reach, such as a `match` arm for
  a value the types already rule out. Remove the branch, or make the types say so, rather
  than inventing a test that reaches it artificially.

Doctests are not measured, so a doc example does not count toward the floor; the lines
it exercises need a unit or integration test as well. Never lower `coverage_floor` in the
`Justfile`.

## Debug a `no_std` or `--locked` failure

`just wasm-check` checks the crate for `wasm32v1-none`, a target with no standard library
at all, and for `wasm32-unknown-unknown`. When it fails, the error names the `std` item
that crept in: a clock, a thread, a file, an environment variable, or a dependency that
enables its own `std` feature. The fix is to take the effect through the trait boundary,
as randomness is taken, or to find the `alloc` or `core` equivalent; never a `cfg` that
hides the item on one target.

A `--locked` failure means `Cargo.lock` disagrees with a manifest: a dependency was edited
without relocking. Read [Maintain dependencies](maintain-dependencies.md) and relock
deliberately; never run `cargo update` from inside a fix for something else.

## Related pages

- [Testing](../reference/testing.md)
- [Quality gates](../reference/quality-gates.md)
- [Troubleshooting](../operations/troubleshooting.md)
