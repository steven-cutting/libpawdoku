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

Open `target/doc/pawdoku/index.html` and follow the `sudoku` module, then `Position`.
Its source is `crates/pawdoku/src/sudoku.rs`. Read `Position::new`, `row` and `column`:
rows count from the top, columns from the left, and a position outside the grid can
still be constructed. Each public item's example is a test, which `just test-doc`
runs. Your exercise adds a method to this type.

## 3. Read what decides the behaviour

Open `docs/specs/sudoku.allium` and find the `config` block.
`box_side` is 3, and the comment says why it has a name at all: so that no rule carries a
bare number, not so that it can be tuned. `side` follows from it, as `box_side * box_side`.
The crate already restates both figures as `BOX_SIDE` and `SIDE`.

Now find `CellsSitOnTheGrid`: a cell's row and column are each at least 1 and at most
`config.side`. Your change makes those bounds readable on a `Position` through
`is_on_grid()`. It reports whether both coordinates fit; it does not prevent making
an off-grid position or change how a move is refused. The bounds come from the clause,
so there is no new rule or setting to decide.

## 4. Change something

The test comes first. In the existing `tests` module at the foot of
`crates/pawdoku/src/sudoku.rs`, add this test. Its `use super::` line already imports
`Position` and `SIDE`. The cases include the four corners, an interior position, and
each coordinate independently below and above the bounds:

```rust
#[test]
fn positions_on_the_grid_have_rows_and_columns_from_one_to_side() {
    let cases = [
        (1, 1, true),
        (1, SIDE, true),
        (SIDE, 1, true),
        (SIDE, SIDE, true),
        (4, 7, true),
        (0, 1, false),
        (1, 0, false),
        (SIDE + 1, 1, false),
        (1, SIDE + 1, false),
        (u8::MAX, 1, false),
        (1, u8::MAX, false),
    ];
    for (row, column, expected) in cases {
        assert_eq!(Position::new(row, column).is_on_grid(), expected);
    }
}
```

Run the suite and watch it fail to compile, because the method does not exist yet.
That is the point. The diagnostic names what is missing:

```text
error[E0599]: no method named `is_on_grid` found for struct `Position` in the current scope
```

```console
just test
```

A test that is green before you have changed anything is either already covered or vacuous.

## 5. Add the behaviour and its test together

In the first `impl Position` block, after `column` and before the block's closing brace,
add this method. Indent it to match the other methods. Its doc example is a test too,
and `#[must_use]` follows the convention for public methods returning a value:

```rust
/// Whether this position lies on the grid: both coordinates are from 1 to [`SIDE`].
///
/// This tests the bounds in `CellsSitOnTheGrid` in `sudoku.allium`.
/// A position outside those bounds can still be constructed.
///
/// ```
/// use pawdoku::sudoku::{Position, SIDE};
///
/// assert!(Position::new(1, SIDE).is_on_grid());
/// assert!(!Position::new(0, 1).is_on_grid());
/// assert!(!Position::new(1, SIDE + 1).is_on_grid());
/// ```
#[must_use]
pub const fn is_on_grid(self) -> bool {
    1 <= self.row && self.row <= SIDE && 1 <= self.column && self.column <= SIDE
}
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
