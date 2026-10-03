---
id: T30
title: "The first-change tutorial: an exercise that can be followed again"
status: done
depends_on: [T25]
parallel_with: [T26, T27, T28]
branch: ticket/t30-first-change-tutorial
estimated_size: S
---

# T30: The first-change tutorial: an exercise that can be followed again

## Context

`docs/tutorials/first-change.md` takes a new contributor from a fresh clone to a green
gate through one small change that passes through every layer: a clause read in a
specification, a failing test, an item, and `just check`. T07 wrote it when the crate
held one documented item, the constant `SIDE` at the root of `crates/pawdoku/src/lib.rs`.
Its exercise was to add `BOX_SIDE` beside it, test first.

T25 built the `sudoku` module and that exercise can no longer be followed:

- **Step 2** says that beside the `random` module there is one documented item, `SIDE`
  in `crates/pawdoku/src/lib.rs`. `SIDE` is now `pawdoku::sudoku::SIDE` in
  `crates/pawdoku/src/sudoku.rs`, beside `BOX_SIDE`, `Position` and `Given`.
- **Step 3** ends "The crate already restates `side`; your change restates the box."
  The crate restates both.
- **Step 4** has the reader add a test of `BOX_SIDE * BOX_SIDE` to the `tests` module at
  the foot of `lib.rs` and watch it fail to compile "because `BOX_SIDE` does not exist
  yet". `lib.rs` has no such module, `BOX_SIDE` exists, and the same assertion is
  already a test, `box_side_is_three_and_side_is_its_square`. The step would start
  green, which the page itself calls "either already covered or vacuous".
- **Step 5** adds `pub const BOX_SIDE` "above `SIDE`" with a doctest of
  `pawdoku::BOX_SIDE`. Adding it is now a duplicate definition.

Steps 1, 6 and 7 and "What you just touched" still hold.

T25 handed the page back and did not rewrite it, because the page needs a new exercise
and choosing one is a decision and not a sentence to correct. Both reviewers of pull
request 26 asked for the fix; the maintainer decided on 2026-10-01 that it has its own
ticket, which is this one.

What an exercise has to be, from the page as T07 built it:

- **Small, and through every layer.** A clause the reader finds in `docs/specs/`, a test
  written first, an item, and the gate.
- **Red first.** The test fails before the item exists. Failing to compile counts.
- **Green under the whole gate.** Step 6 runs `just check`, so the item the reader adds
  must pass clippy pedantic with the workspace lint table (a private item nothing uses
  is `dead_code`), carry a doctest if it is public, and keep the metrics gate quiet.
- **Never landed.** The reader makes the change in their own clone. The repository does
  not gain the item, or the next reader would start green. T07 ran its exercise for real
  and then reverted it.
- **Still true after T26, T27 and T28.** Each of them changes the public surface: T26
  makes `sudoku`'s crate-only types public and adds the solver, T27 and T28 add the
  board. An exercise that says "the one documented item" or that adds something one of
  those tickets adds breaks again within weeks. Read the three tickets' "Files touched"
  and "Names later tickets need" in `tickets/T25-sudoku-rules.md` before choosing.

## Goal

A contributor can follow `docs/tutorials/first-change.md` from a fresh clone of `main`,
step by step, and see what each step says they will see: a test that fails first, then
passes, then a green `just check`.

## Non-goals

- No change to the crate. The exercise is something a reader does in their clone; this
  ticket lands a page.
- No change to a specification. If the exercise needs a clause the modules do not state,
  it is the wrong exercise.
- No new page, no rename, and no change to `docs/manifest.yml` or `docs/README.md`.
- Not the two READMEs: T25's review round corrected `README.md` and
  `crates/pawdoku/README.md`.

## Files touched

| Path | Change |
| --- | --- |
| `docs/tutorials/first-change.md` | Steps 2 to 5 rewritten around the new exercise; the rest kept |
| `tickets/T30-first-change-tutorial.md` | `status: done` and the hand-back notes |
| `tickets/README.md` | T30's row set `done` |

## Steps

1. Read `docs/tutorials/first-change.md`, `tickets/T07-handbook-a.md` section 4 ("The
   tutorial"), which says what each step was built to show, and
   `docs/reference/documentation-contract.md`.
2. Settle the exercise with the maintainer: see "Open points". Do not choose alone.
3. Run the exercise for real in the worktree, before writing a word: write the test,
   run `just test` and keep the failure's text, add the item, run `just test` and then
   `just check`. Then revert the crate with `git checkout` of the files the exercise
   touched, so that only the page changes.
4. Rewrite steps 2 to 5 of the page to match what step 3 showed. Keep the seven-step
   shape, every heading that still applies, the frontmatter, and the sentences of steps
   1, 6 and 7. Step 2 describes what `just doc` shows in terms that later tickets do not
   falsify: name the module the exercise works in, not a count of items.
5. Run `just check-docs`, then `just check`.
6. Write the hand-back notes, set `status: done` here and in `tickets/README.md`, commit
   on the ticket branch, and stop before pushing.

## Acceptance criteria

- Every path, item name and command the page names exists on `main` as the page says.
- Followed from a clean checkout, the test of step 4 fails before step 5 and passes
  after it, and step 6's `just check` is green with the exercise applied.
- The commit changes no file under `crates/`.
- The page keeps its frontmatter and its seven steps, and holds 500 words or more, the
  floor T07 set for it.
- `just check` is green.

## Verification

```sh
just check-docs
just check
git diff --stat main -- crates/        # expect: no output
```

Quote, in the hand-back notes, the failing output of step 4 and the passing output of
step 5 from the run of step 3 above.

## Hand-back notes

Done on 2026-10-02 on `ticket/t30-first-change-tutorial`, starting from `cd27d19`
(the merge of T27), with a clean worktree. The maintainer chose the position helper
in this session before the exercise was run.

### The exercise chosen

`Position::is_on_grid()` reads the bounds already stated by `CellsSitOnTheGrid`:
both coordinates from 1 to `config.side`. It adds no rule or configuration figure,
and leaves positions outside the grid constructible. The unit test checks the four
corners, an interior position, and each coordinate independently at 0, `SIDE + 1`
and `u8::MAX`. The public method has `#[must_use]` and a running doc example.

T26 and T27 are already merged here. Their file lists and T25's "Names later tickets
need" preserve `sudoku::Position`, its constructor, getters and `SIDE`. T28 changes
the board's recording surface and does not add this method or move those items.
None of T26 to T28 needs a tutorial row added to its "Files touched" for this exercise.

The exercise lasts only while the method stays unbuilt. The crate already checks these
bounds privately in three places: `at` and `at_mut` in `sudoku.rs`, and the `on_grid`
closure in `solver/branch.rs`. A later ticket that lands a public
`Position::is_on_grid` makes step 4 start green; that ticket must choose a new exercise
and list the tutorial in its "Files touched".

### The exercise run

Before editing the page, the exact test now printed in step 4 was added to the existing
unit-test module in `crates/pawdoku/src/sudoku.rs`. `just test` exited 101, with this
compiler output:

```text
error[E0599]: no method named `is_on_grid` found for struct `Position` in the current scope
   --> crates/pawdoku/src/sudoku.rs:319:51
    |
 97 | pub struct Position {
    | ------------------- method `is_on_grid` not found for this struct
...
319 |             assert_eq!(Position::new(row, column).is_on_grid(), expected);
    |                                                   ^^^^^^^^^^ method not found in `Position`

```

The method and doc example now printed in step 5 were then added to the first
`impl Position` block. Selected output from the successful `just test` run:

```text
        PASS [   0.008s] ( 81/207) pawdoku sudoku::tests::positions_on_the_grid_have_rows_and_columns_from_one_to_side
     Summary [   0.859s] 207 tests run: 207 passed, 0 skipped
test crates/pawdoku/src/sudoku.rs - sudoku::Position::is_on_grid (line 144) ... ok
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

`just check` also passed with the exercise applied, including clippy, metrics,
feature combinations, both wasm targets, coverage and the documentation build.
It ended with:

```text
All checks passed and the worktree is unchanged.
```

Only the exercised crate file was restored with `git checkout --
crates/pawdoku/src/sudoku.rs`. The method and test are absent from the final crate;
`git diff --stat main...HEAD -- crates/` has no output. The source used in the rehearsal and
its logs remain only in gitignored `ai_tmp/`.

### The page and final verification

Steps 2 to 5 use the rehearsed snippets and existing paths and imports. Step 2 sends
the reader into `sudoku::Position` without counting the public items. Frontmatter,
all seven step headings, steps 1, 6 and 7, and the sections after step 7 are unchanged.
The page body contains 869 whitespace-delimited words, above the 500-word floor.
Both Rust snippets match the exercised source apart from indentation: the page prints
them flush-left and tells the reader to indent each to match its neighbours. Review
reworded one line of step 5's doc comment; both snippets were then pasted and run again,
with the same 207 passing tests and the doctest still at line 144, and the crate file
was restored again.

`just check-docs` passes: Markdown, spelling, links and the documentation contract
(41 pages and 42 canonical topics). The final `just check` passes and reports that
its run changed nothing. All validation used rustup's cargo first on `PATH` and
`CARGO_NET_OFFLINE=true`; the installed prek cache required access outside the
worktree sandbox.

The final change contains only the tutorial, this ticket and its index row. T30 is
done here and in `tickets/README.md`; the local commit is on the ticket branch.

## Open points

One condition, stated under "The exercise chosen": a ticket that lands a public
`Position::is_on_grid` must re-do the exercise. Its compatibility with T26 to T28 is
settled above.
