---
title: "Make your first change"
kind: "tutorial"
audience: [contributor, agent]
canonical_for: [first_change_tutorial]
requires: []
---

# Make your first change

About half an hour, from a fresh clone to a green gate. The first run downloads the pixi
environment, a Rust toolchain, cargo-hack, the Allium checker and the hook environments,
so it needs the network, and it needs rustup and pixi installed first;
[Develop locally](../how-to/develop-locally.md) describes both. The change is small on
purpose; what matters is that it passes through every layer the repository has: a
clause, a test, an item and the gate.

## 1. Get the workspace running

```console
just initialize
just check
```

`just initialize` installs the pixi environment from `pixi.lock`, installs the toolchain
`rust-toolchain.toml` pins, refuses a cargo that is not rustup's, installs cargo-hack
into `.tools/bin` and then the Allium checker, syncs both lockfiles, normalises
formatting and installs the pre-commit hook, from the primary worktree only. It never
stages, commits, tags or pushes. If it complains about a missing tool, read
[Develop locally](../how-to/develop-locally.md).

## 2. See the crate

```console
just doc
```

Open `target/doc/pawdoku/index.html`. Beside the `random` module there is one documented
item, the constant `SIDE` in `crates/pawdoku/src/lib.rs`, which restates `sudoku.allium`'s
`config.side`: nine cells to a row, a column and a box. Its example is a test, which
`just test-doc` runs.

## 3. Read what decides the behaviour

Open `docs/specs/sudoku.allium` and find the `config` block near the foot of the module.
`box_side` is 3, and the comment says why it has a name at all: so that no rule carries a
bare number, not so that it can be tuned. `side` follows from it, as `box_side * box_side`.
Then find `is_full` on `Puzzle`, which counts filled cells against `config.side *
config.side` rather than against 81. The crate already restates `side`; your change
restates the box.

## 4. Change something

The test comes first. In the `tests` module at the foot of `lib.rs`, add a test that
asserts what the `config` comment says, that a grid is `box_side` boxes of `box_side`
cells:

```rust
#[test]
fn a_side_is_a_box_side_of_boxes() {
    assert_eq!(BOX_SIDE * BOX_SIDE, SIDE);
}
```

Add `BOX_SIDE` to the module's `use super::` line, then run the suite and watch it fail to
compile, because `BOX_SIDE` does not exist yet. That is the point.

```console
just test
```

A test that is green before you have changed anything is either already covered or vacuous.

## 5. Add the behaviour and its test together

Above `SIDE`, add the item with a doc comment and an example, because a doc example is a
test too:

```rust
/// The side of a box: three cells across and three down, and three boxes to a band.
///
/// ```
/// assert_eq!(pawdoku::BOX_SIDE, 3);
/// ```
pub const BOX_SIDE: u8 = 3;
```

Run `just test` again; it runs the new test and the new example. Land the item, its
example and the assertion in the same commit. The rule holds for every later change: a
clause, its test and the code land together, and the test is named for the clause it
proves.

## 6. Run the whole gate

```console
just check
```

It runs every recipe in order and proves the run did not modify the worktree. Read only
the first failure; the later ones are often consequences. If a gate fails, do not work
around it — [Troubleshooting](../operations/troubleshooting.md) covers the common causes.

## 7. Commit

The pre-commit hook runs the read-only gate again. It does not run clippy, the tests or
the wasm checks, which is why `just check` came first. Nothing is pushed until you ask for
it.

## What you just touched

A clause you read, a test, an item, and the gate. That is the whole loop; every change
after this one is the same shape, and the first real one starts by adding a rule to a
module rather than reading one.

## Related pages

- [Develop locally](../how-to/develop-locally.md)
- [Test and debug](../how-to/test-and-debug.md)
- [Work with the specifications](../how-to/work-with-the-specs.md)
