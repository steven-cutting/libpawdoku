---
id: T27
title: "The board in play: notes, moves, undo and redo, reading back, and the check"
status: open
depends_on: [T26]
parallel_with: []
branch: ticket/t27-board-in-play
estimated_size: L
---

# T27: The board in play: notes, moves, undo and redo, reading back, and the check

## Context

T25 built the rules (`pawdoku::sudoku`) and T26 the solver (`pawdoku::solver`). This
ticket builds `pawdoku::board` from `docs/specs/board.allium`: one puzzle as it is being
played. It keeps a note in every cell, every move the player has made so that the latest
can be taken back and re-taken, the board as it stood after any move, and one question
about one cell, answered yes or no. T28 adds the record and reopening; this ticket
builds everything else in the module.

The design was decided by the maintainer on 2026-09-30 and is recorded in the decision
T23 wrote (`docs/decisions/0014-board-imports-solver.md`, unless renumbered). What this
ticket needs from it:

- **The board imports the solver.** T23 changed `board.allium`, the layering table and
  the boundary rule, now `board_imports_sudoku_and_solver`. `src/board` may name
  `crate::sudoku`, `crate::solver` and `crate::random`, and no other engine module.
- **A board opens from givens in one call.** It runs `solver::solve` itself and sets the
  puzzle from the proof. The caller hands over givens and gets a board or a refusal.
- **The board is the playable abstraction, and it hides its puzzle.** A `Board` owns its
  `Puzzle` and hands out values of its own. No method returns the puzzle, a reference to
  it, a `&mut` to anything, or one of its internal collections. Everything the `Playing`
  surface exposes is read through the board.
- **The check reads the solution through the puzzle's crate-only answer.** `Puzzle`
  keeps the solution privately and answers yes or no for one digit at one position; that
  answer is `pub(crate)` and this module is its one caller. The digits never leave.
- **Solved is the rules' definition**, full and consistent, read from the puzzle.

Facts that shape the work:

- **No board from a puzzle.** `Board` has no constructor that takes a `Puzzle`. A puzzle
  already played on would arrive with digits no move recorded, against
  `TheMovesReplayToTheBoard` and `EveryMoveIsTheBoards`.
- **A puzzle with no board is not a defect.** `OpenBoard` gives every puzzle that is set
  a board, and the two-step path (solve, then set) yields a `Puzzle` with none. The
  reading: `OpenBoard` is this module's view, of puzzles set through it. A `Puzzle` on
  its own is `sudoku.allium`'s `PuzzleSolving` surface, which stands as the rules' own
  boundary. `OneBoardToAPuzzle` holds by ownership: a board owns its puzzle and no
  second board can reach it. Record this reading in the module's documentation.
- **Undo and redo go through the puzzle's own two moves.** `board.allium` says where a
  cell's digit ends up on undo and redo, and after T23 it does not say how it is put
  there: "Whether it is put back through the rules' own placing and erasing or
  written directly is not said here." If that sentence is not in the `Undo` comment
  when this ticket starts, stop and report: the text then asks for a direct write,
  and whether `Puzzle` gains one is the maintainer's. `Puzzle` offers only placing
  and erasing, and those are enough. Undoing a placement erases or places what stood
  before; undoing an erasure places the digit again; redo does the move again. None of
  these can solve
  the puzzle or be refused by the rules. Undo restores a state that stood before a move,
  and every move requires an unsolved puzzle, so that state was not complete. Redo
  restores the state after a move that was then undone, and undo is refused once the
  puzzle is solved, so that state was not complete either. If an implementation finds a
  case this argument misses, stop and report; a second way to write a cell in `sudoku`
  is the maintainer's to allow and not this ticket's to add.
- **Reading back never takes a move back.** `digit_after` and `note_after` for any move,
  standing or undone, are computed from the moves forward and leave the board as it was.
- **The method cap is real here.** `rustqual.toml` allows a struct 20 methods and 12
  fields, LCOM4 at most 2, a file 500 code lines before its first `#[cfg(test)]` and
  1000 for test code, a function 60 lines, cognitive complexity 15, cyclomatic 10,
  nesting 4, five parameters, and no recursion. `Playing` exposes five facts about the
  board, six about each cell, every move with its readings and every check, and offers
  seven operations; T28 adds two more. One flat `Board` would be a god object and would
  fail the gate. Use the specification's own four entities: the board hands back small
  values (a cell as the board shows it, a move, a check), each with its own few
  accessors, and the module is `src/board.rs` plus `src/board/<part>.rs`. Never a
  suppression and never a moved number. If structure cannot meet a limit, stop and
  report.
- **Snapshots show and detect change; they prove nothing.** T29 added insta and the rule,
  which `docs/reference/testing.md` states: a snapshot shows a reviewer what a value
  looks like and makes a change to it visible in a diff. It is never the test of a
  clause, so no line of "What the tests must cover" and no row of the hand-back table
  names one. Snapshots are `.snap` files, taken from fixed inputs after the module's
  own tests are green, by tests named `snapshot_...`, from a text picture rendered in
  test code. No `Display`, method or field is added to the library to feed one.
  Here they are taken through `Board` alone, so no snapshot of a board can hold the
  solution. An earlier ticket's snapshot that changes has its diff read and its reason
  given in the hand-back notes.
- **T25 left one thing for this ticket.** The puzzle's crate-only answer carries a
  `dead_code` expectation, under `cfg_attr(not(test), ...)`, naming T27. Once the check
  calls it, the expectation is unfulfilled and `just clippy` fails until the line is
  removed.
- **The module's one open question is not this ticket's to answer.** "What, if anything,
  is counted from the checks - wrong answers, checks made - and is it shown to the
  player?" The board exposes the checks, as `Playing` does. It adds no count, no tally
  of wrong answers and no method that returns one.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11; `docs/specs/board.allium` and
`docs/specs/sudoku.allium` in full; the decision T23 wrote; the hand-back notes of
`tickets/T25-sudoku-rules.md` and `tickets/T26-solver.md` ("Names later tickets need");
`crates/pawdoku/src/sudoku.rs` and `crates/pawdoku/src/solver.rs`;
`docs/explanation/layering.md` and `docs/explanation/architecture.md`;
`docs/reference/testing.md`, its Snapshots section included;
`docs/project/terminology.md` (the board's words: move,
standing, undone, upkeep, note, mark, check); `.agents/skills/rust-change/SKILL.md` and
`.agents/skills/propagate/SKILL.md`; `rustqual.toml` and `clippy.toml`. Background only,
never modified: `/Users/scutting/projects/pawdoku/prototypes/board/board.py`, which
models this module in Python with the puzzle exposed; it is a guide to behaviour and not
to this design.

## Goal

`pawdoku::board` exists and does what `board.allium` says of a board in play: opening,
the four moves with upkeep, undo and redo, reading back, and the check. A consumer plays
a whole puzzle through `Board` alone. Every rule, invariant and guarantee of the module
outside the record has a named test or a stated reason, listed in the hand-back notes.
`just check` is green with the coverage floor held by this module's own tests and no
suppression.

The name later tickets are written against, fixed here: `board::Board`. Every other name
is this ticket's to choose and to report.

## Non-goals

- No record and no reopening: `contract Recording`, `surface Reopening`,
  `ReopeningIsExact`, `WritingChangesNothing`, `AReopenedBoardIsTheSameBoard` and
  `ARecordIsEnoughOnItsOwn` are T28's. `Board` does not serialise.
- No count read off the checks, and no answer to the open question.
- No hint, no timer, nothing drawn, and no words for a surface.
- No way to reach the puzzle or the solution through a board, other than the yes or no
  of a check the player asked for.
- No change to `board.allium` or any module, and no change to `sudoku` or `solver`
  beyond the one removed attribute. If no clause says what the code needs, stop: that is
  a `spec-change` first.
- No new dependency and no new feature flag.
- No snapshot standing in for a test that asserts, and nothing added to `Board` so
  that a snapshot can read it: the method cap is not spent on pictures.

## Files touched

| Path | Change |
| --- | --- |
| `crates/pawdoku/src/board.rs` | New: the module's surface |
| `crates/pawdoku/src/board/` | New: one file per part, as the limits ask |
| `crates/pawdoku/src/sudoku.rs` | The answer's `dead_code` expectation removed, the whole `cfg_attr` line, and nothing else |
| `crates/pawdoku/src/lib.rs` | `pub mod board;` and the crate overview |
| `crates/pawdoku/tests/api_bounds.rs` | Every new public type |
| `crates/pawdoku/tests/board.rs` | New: a puzzle played through `Board` alone |
| `crates/pawdoku/tests/snapshots.rs` | The board's picture added to T26's render helper, and the snapshot tests of step 6 |
| `crates/pawdoku/tests/snapshots/` | The `.snap` files those tests write |
| `docs/explanation/architecture.md` | The board as the playable abstraction: what it owns, what it hands out |
| `docs/reference/testing.md` | The suite table; how the state-machine property is built |
| `docs/project/repository-map.md` | The `src/` and `tests/` lines |
| `CHANGELOG.md` | Under Unreleased, Added |
| `tickets/README.md` | T27's row set `done` |
| `tickets/T27-board-in-play.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t27-board-in-play` from `main` after T26 has merged.
   Run `just initialize` if `.pixi/` is absent; it uses the network and this ticket
   authorises it. Run `just check` and confirm it is green before any edit.

2. **Write the test list before any code.** One line per expected test: a name taken
   from the clause it proves and what it expects. Seed it from "What the tests must
   cover" below. Then run `just plan-spec board` and add any obligation the list lacks.
   Strike the ones that belong to the record and mark them T28's. Put the list in the
   hand-back notes under "The test list" before the first test is written.

3. **Work the list one line at a time.**

   - Take one line. Write the test, run `just test`, and see it fail for the reason the
     line expects.
   - Write the least code that makes it pass. Then tidy with the tests green.
   - Before taking the next line, re-read the whole list against what the loop taught.
     Add the lines it revealed, strike the ones that proved wrong or redundant, and
     reorder so that the next line is the smallest step. Note each change to the list in
     the hand-back notes in one line, with its reason.

   Repeat until the list is empty. Run `just clippy` and `just metrics` every few loops:
   the method cap is easier to meet at ten methods than at twenty-five. A good order is
   opening, then one placement, then erasure, then marks, then upkeep, then undo, then
   redo, then reading back, then the check.

4. **Prefer tests through the public API.** A board is meant to be driven from outside,
   so most of the list belongs in `crates/pawdoku/tests/board.rs`. Tests that need the
   solution take it from `solver::solve`, whose proof exposes it. Write two test
   helpers, each gathering a single comparable value, because the guarantees compare
   two different things:

   - **The picture:** every cell's digit and its note, shown or waiting. It is what
     `UndoAndRedoAreExact` and `EveryPastBoardIsReadable` speak of. `Playing` shows a
     note only while its cell is empty, so a note waiting beneath a digit is read
     through `note_after` on the latest standing move, which `TheMovesReplayToTheBoard`
     pins to the board. When no move stands, no digit of the player's stands and no
     note waits. A picture that leaves the waiting notes out proves half the guarantee.
   - **The full view:** everything `Playing` exposes. The picture, the puzzle's three
     facts, `can_undo` and `can_redo`, each cell's given and conflict flags, every move
     with its index, kind, target, digit, undone flag and readings, and every check.
     It is what a refused operation leaves alone and what T28 compares across
     reopening.

   Both are values that tests compare with assertions. Neither is an insta snapshot,
   which is step 6's and proves nothing.

   Undo and redo do not restore the full view, and no test may ask them to. A move taken
   back stays on the record as undone, `can_redo` becomes true, and a check asked
   meanwhile is kept.

5. **The state-machine property.** A property drives arbitrary sequences of the seven
   operations, refused ones included, against a board opened on the fixture below. It
   keeps the picture taken before and after each move for as long as the move is on the
   record. After every step it checks:

   - the seventeen invariants as far as the surface shows them;
   - a refused operation: the full view is unchanged;
   - an undo: the picture equals the one taken before the move taken back. That move is
     still on the record with its index, now undone; no other move changed; the checks
     are unchanged; `can_redo` is true;
   - a redo: the picture equals the one taken after that move. The move stands again;
     no other move changed; the checks are unchanged;
   - a move: it joins the record standing, at one past the count that stood; every move
     that was undone is gone; the checks are unchanged;
   - a check: the picture and the moves are unchanged, and the checks grew by one;
   - reading back: for every move on the record, standing or undone, `digit_after` and
     `note_after` over every cell equal the picture taken after that move was made.

6. **Snapshots**, once the list is empty. In `crates/pawdoku/tests/snapshots.rs`,
   extend T26's render helper with the board as
   text, read through `Board` alone: the grid; each cell's note, shown or waiting;
   the givens and the conflicts; `can_undo` and `can_redo`; every move with its index,
   kind, target, digit and whether it is undone; and every check with its answer. No
   line ends in a space. Then take these in the same file, each by a test named
   `snapshot_...`:

   - the board as opened on the fixture;
   - the board after one fixed script that uses every kind of move, upkeep striking a
     peer's mark, an undo, a redo, a move that discards an undone one, and a check
     answered each way. Write the script once and name it: T28 writes this same board
     to a record;
   - the fixture played to the end, solved;
   - the `Display` text of every refusal the module defines, in one snapshot.

   Accept with `just snapshots-accept` and read each file before committing it. List
   them in the hand-back notes under "Snapshots taken", with the script's name.

7. Remove the `dead_code` expectation on the puzzle's answer once the check calls it.

8. **Doc examples and the pages.** Every public item this ticket adds has a doc
   example that runs and asserts (`AGENTS.md`: "a doctest on every public item"):
   `Board`, its constructor, each operation and each reading, every value the board
   hands out with its accessors, and each error. `missing_docs` and `just doc` do not
   notice an absent example, so the hand-back notes list each public item beside the
   doctest that covers it; `just test` runs them.

   `docs/explanation/architecture.md`: the board owns its puzzle and
   hands out values; why it has no constructor from a puzzle; the reading of a puzzle
   with no board. `docs/reference/testing.md`: the suite rows, the property and the
   snapshots taken.
   `docs/project/repository-map.md` and `CHANGELOG.md` follow.

9. Run `just fmt-check`, `just clippy`, `just metrics`, `just test`,
   `just snapshots-check`, `just wasm-check`, `just features`, `just coverage`,
   `just doc` and `just check-docs`, then `just check`.
   Quote the closing lines of each and the coverage line for the module. Fill in the
   hand-back notes: the test list as it ended, as a table from clause to test name or
   reason; and "Names later tickets need", with every public item's signature and what
   T28 needs of the board's inside to write a record. Set `status: done` here and in
   `tickets/README.md`, commit on the ticket branch, and stop before pushing.

The fixture: thirty givens, one solution, rows top to bottom.

```text
puzzle       solution
53..7....    534678912
6..195...    672195348
.98....6.    198342567
8...6...3    859761423
4..8.3..1    426853791
7...2...6    713924856
.6....28.    961537284
...419..5    287419635
....8..79    345286179
```

### What the tests must cover

From `docs/specs/board.allium`, by name.

- **`OpenBoard` and `LayOutBoard`.** Opening on well-posed givens gives a board with
  eighty-one cells, every note empty, no move and no check. Opening is refused for
  givens with no solution, with several, and with nothing left to play.
- **`Place`.** Refused on a solved puzzle, on a given cell, for a digit outside 1 to 9,
  and when the cell already holds that digit: the board's own guard, so nothing is
  recorded and nothing undone is discarded. A placement over the player's other digit is
  a move and remembers what it displaced. A conflicting placement is a move.
- **Upkeep.** A placed digit is struck from the note of every peer that holds it, notes
  hidden beneath a digit included, and from no other cell; the target's own note is
  kept. Erasing gives nothing back to the peers.
- **`Erase`.** Refused on a solved puzzle and on a cell that holds no digit of the
  player's. The note beneath the digit shows again.
- **`WriteMark` and `StrikeMark`.** Each refusal: a solved puzzle, a cell that does not
  accept marks, a digit out of range, a mark already written, a mark not there. A mark a
  placed peer rules out may still be written.
- **`Undo` and `Redo`.** Each kind of move taken back exactly and re-taken exactly,
  where exact is of the picture: every digit and every note, shown or waiting. What an
  undo leaves on the record is asserted beside it: the move, now undone, the offer to
  redo it, and every check. Both refused with nothing to act on; both refused once the
  puzzle is solved.
- **`CheckCell`.** Yes for the solution's digit and no for another; refused on a solved
  puzzle, on an empty cell and on a given. A check records the digit as it stood and how
  many moves stood; it is not a move, and undo does not remove it.
- **Derived values.** `can_undo`, `can_redo`, `holds_players_digit`, `accepts_marks`,
  `shows_note`, the latest standing move and the next to redo.
- **The seventeen invariants:** `OneBoardToAPuzzle` (by ownership; say so),
  `TheBoardIsWhole`, `OneBoardCellToAPosition`, `MarksAreDigits`, `NoMarkInAGiven`,
  `MovesNeverTouchAGiven`, `MoveIndicesRunFromOne`, `MoveIndicesAreDistinct`,
  `MovesCarryWhatTheirKindNeeds`, `UndoneMovesAreTheLatest`, `WhatIsReTakenComesNext`,
  `ASolvedBoardHasNothingUndone`, `UpkeepStrikesPeersOfAPlacement`,
  `TheMovesReplayToTheBoard`, `ChecksRunFromOne`, `CheckIndicesAreDistinct`,
  `ACheckIsOnAPlayersCell`.
- **The eleven guarantees of `Playing`:** `EveryMoveIsTheBoards`, `UndoAndRedoAreExact`,
  `ANewMoveDiscardsWhatWasUndone`, `AMarkIsAMove`, `UndoStopsWithThePuzzle`,
  `EveryPastBoardIsReadable`, `GivensNeverMove`, `ANoteWaitsBeneathADigit`,
  `EveryCheckIsKept`, `ACheckNeverTellsTheDigit`, `TheSolutionIsNeverShown`. The last
  two are held by the shape of the API as well as by tests: no public item of the module
  returns a digit read from the solution. State that in the hand-back notes after
  reading the module's public surface item by item.
- **What `Playing` exposes and provides.** Everything it lists can be read through the
  board; a note is shown only while its cell is empty. What it provides is two tests and
  not one. A `when` clause says whether an operation is offered, and it speaks of the
  board and the cell. The rule's `requires` clauses say whether it is accepted, and they
  also judge the digit. So the oracle for acceptance is the rule: an operation is
  accepted exactly when every `requires` clause of its rule holds and its position
  names a cell. Wherever the `when` clause is false the operation is refused, since
  every `when` condition is also a `requires`; the converse does not hold. Three
  operations are offered and still refused: `Place` for a digit out of range or the
  digit already standing, `WriteMark` for a digit out of range or a mark already
  written, and `StrikeMark` for a mark not in the note. For `Erase`, `CheckCell`, `Undo`
  and `Redo` the two agree. Test the `when` clauses against what the board lets a
  caller read (`can_undo`, `can_redo` and each cell's facts); this ticket asks for no
  separate "is it offered" method.
- **A whole game.** The fixture played to the end through `Board`: solved, and then
  every move, undo, redo and check refused.

## Acceptance criteria

- `pawdoku::board` exports `Board`. Nothing in `src/board` names an engine module other
  than `sudoku` and `solver`.
- No public item of `board` returns a `Puzzle`, a reference to one, a `&mut` to
  anything, or a digit read from the solution. `Board` has no constructor that takes a
  `Puzzle`.
- No public item returns a count of checks or of wrong answers.
- Every line under "What the tests must cover" maps to a named test in the hand-back
  table, and every `just plan-spec board` obligation is in the table, struck with a
  reason, or marked T28's.
- No recursion. No `#[allow]`, no `#[expect]` added, no `qual:allow` line; the answer's
  expectation in `src/sudoku.rs` is gone and nothing else in that file changed.
- No struct in `src/board` exceeds 20 methods, by `just metrics`.
- Every public item of `board` has a doc example that runs and asserts, and the
  hand-back notes list each beside its doctest.
- Line coverage of `src/board` alone is at or above 90 per cent.
- The four snapshots of step 6 exist as `.snap` files, each taken by a test named
  `snapshot_...` through `Board` alone, and none shows anything read from the solution
  but a check's yes or no. None appears in the hand-back table as the test of a
  clause. `just snapshots-check` is green, and any earlier snapshot that changed has
  its reason in the hand-back notes.
- `just check` is green.

## Verification

```sh
just plan-spec board
just fmt-check
just clippy
just metrics
just test
just snapshots-check
just wasm-check
just features
just coverage
just doc
just check-docs
just check
```

Expected: the obligation list the table accounts for; each recipe green with no warning;
`metrics` reporting no finding; both wasm targets under both feature sets; the coverage
total at or above the floor; `All checks passed and the worktree is unchanged.`

## Hand-back notes

### The test list

The list as first written on 2026-10-01, before any test or code, seeded from "What the
tests must cover" and then checked against the 78 obligations of `just plan-spec board`.
One line is one expected test: its name, then what it expects. B is
`crates/pawdoku/tests/board.rs`, through the public API; U is a unit test in the part of
`src/board/` it names.

Opening, `OpenBoard` and `LayOutBoard` (B):

1. `opening_gives_eighty_one_cells_no_note_no_move_and_no_check`: one cell to a position
   in grid order, every note empty, nothing to undo or redo, unsolved.
2. `a_board_shows_its_givens_and_leaves_every_other_cell_empty`: thirty given cells
   holding the fixture's digits, fifty-one empty.
3. `opening_is_refused_for_givens_with_no_solution`.
4. `opening_is_refused_for_givens_with_several_solutions`.
5. `opening_is_refused_for_givens_that_leave_nothing_to_play`.

`Place` (B):

1. `a_placement_puts_the_digit_in_the_cell_and_joins_the_record`: move 1, kind place,
   its target and digit, standing.
2. `placing_is_refused_on_a_given_cell`.
3. `placing_is_refused_for_a_digit_out_of_range`: 0, 10 and 255.
4. `placing_the_standing_digit_again_is_refused_and_discards_nothing`: nothing joins the
   record and an undone move can still be re-taken.
5. `a_placement_over_the_players_other_digit_is_a_move_that_remembers_what_it_displaced`.
6. `a_conflicting_placement_is_a_move`: both cells conflict and the board is not
   consistent.
7. `a_position_off_the_grid_is_refused_by_every_operation`.
8. `refusals_come_in_a_fixed_order`: off the grid, then solved, then the cell's own
   clauses as written (the third open point).

Upkeep (B):

1. `a_placed_digit_is_struck_from_every_peer_that_holds_it_and_from_no_other_cell`: a
   mark in the row, the column and the box goes, one outside stays, another digit's mark
   in a peer stays.
2. `upkeep_reaches_a_note_hidden_beneath_a_digit`.
3. `the_targets_own_note_is_kept_beneath_its_digit`.
4. `erasing_gives_nothing_back_to_the_peers`.

`Erase` (B):

1. `an_erasure_empties_the_cell_and_joins_the_record`: kind erase, no digit.
2. `erasing_is_refused_on_a_cell_with_no_digit_of_the_players`: an empty cell and a
   given.
3. `the_note_beneath_a_digit_shows_again_when_it_is_erased`: `ANoteWaitsBeneathADigit`.

`WriteMark` and `StrikeMark` (B):

1. `a_written_mark_joins_the_note_and_the_record`.
2. `a_struck_mark_leaves_the_note_and_joins_the_record`.
3. `writing_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`: a given and a cell
   holding the player's digit.
4. `writing_a_mark_is_refused_for_a_digit_out_of_range`.
5. `writing_a_mark_already_written_is_refused`.
6. `striking_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`.
7. `striking_a_mark_that_is_not_there_is_refused`: a digit out of range included.
8. `a_mark_a_placed_peer_rules_out_may_still_be_written`.

`Undo` and `Redo` (B). Exact is of the picture: every digit and every note, shown or
waiting.

1. `undoing_a_placement_leaves_the_board_as_it_stood`: the struck marks go back to the
   notes they were struck from and no others; beside it, the move is still on the
   record, undone, redo is offered and a check asked meanwhile is kept.
2. `undoing_a_placement_over_a_digit_puts_the_former_digit_back`.
3. `undoing_an_erasure_puts_the_digit_back`.
4. `undoing_a_written_mark_strikes_it_and_undoing_a_struck_mark_writes_it`:
   `AMarkIsAMove`.
5. `redo_retakes_each_kind_of_move_exactly`.
6. `undo_takes_back_the_latest_standing_move_and_redo_retakes_the_earliest_undone`: the
   two derived moves, and `WhatIsReTakenComesNext`.
7. `undo_is_refused_with_nothing_standing_and_redo_with_nothing_undone`.
8. `a_new_move_discards_every_undone_move`: `ANewMoveDiscardsWhatWasUndone`.
9. `can_undo_and_can_redo_follow_the_moves`.

Reading back (B):

1. `every_move_reads_back_the_board_as_it_stood_after_it`: a fixed script, each picture
   kept as it is made, then compared for every move, standing and undone.
2. `reading_back_takes_nothing_back`: the full view is the same before and after.
3. `the_reading_at_the_latest_standing_move_is_the_board`: `TheMovesReplayToTheBoard`.
4. `a_reading_off_the_grid_is_empty`.

`CheckCell` (B):

1. `a_check_answers_yes_for_the_solutions_digit_and_no_for_another`.
2. `a_check_is_refused_on_an_empty_cell_and_on_a_given`.
3. `a_check_records_the_digit_as_it_stood_and_how_many_moves_stood`.
4. `a_check_is_not_a_move_and_undo_does_not_remove_it`: `EveryCheckIsKept`.
5. `after_move_is_a_count_and_not_a_reference`: after an undo and a new move the check
   says what it said.
6. `a_check_never_tells_the_digit`: a wrong digit's check holds the player's digit and a
   no, and nothing else.
7. `a_board_does_not_print_its_solution`: `TheSolutionIsNeverShown`, for `Debug`.

Derived values and the surface (B):

1. `a_cells_facts_follow_its_digit_and_whether_it_is_given`: `holds_players_digit`,
   `accepts_marks` and `shows_note` for a given, an empty cell and a player's digit.
2. `a_note_is_shown_only_while_its_cell_is_empty`.
3. `everything_playing_exposes_can_be_read_through_the_board`.
4. `an_offered_operation_may_still_be_refused`: the three the `when` clauses offer and
   the rules refuse.
5. `two_boards_never_share_a_puzzle`: `OneBoardToAPuzzle`, which holds by ownership; a
   clone played on leaves the first as it was.

A whole game (B):

1. `a_whole_game_is_played_through_the_board_and_then_everything_is_refused`: the
   fixture to its end, solved; then every move, undo, redo and check refused, and every
   move still read back.

The state machine (B):

1. `any_script_keeps_the_board_to_its_rules`: the property of step 5. Each of the
   seventeen invariants is asserted under its own name as far as the surface shows it,
   an operation is accepted exactly when the model of its rule's `requires` clauses says
   so, and wherever its `when` clause is false it is refused.

Inside the module (U):

1. U `error` `refusal_texts_are_stable`.
2. U `error` `the_rules_refusals_convert_to_the_boards`: the conversion, called directly.
3. U `note` `a_note_holds_only_digits_from_one_to_nine`: `MarksAreDigits`, whatever is
   offered.
4. U `note` `marks_are_written_and_struck`.

From outside the crate:

1. Every new public type in the two bounds tests of `tests/api_bounds.rs`.
2. A doc example on every public item.

Expected to be struck or marked T28's: `contract-signature.Recording.write`,
`contract-signature.Recording.reopen` and `surface-actor.Reopening` are T28's;
`surface-actor.Playing` has no actor to test, since the surface names a context and no
actor; `config-default.side` is `sudoku`'s figure, tested there.

### Names later tickets need

### Snapshots taken

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **Opening from a proof.** A caller that has already solved the givens pays for a
  second search when it opens a board. A constructor that takes the proof would save
  it, and would be safe, since a proof has not been played on. The decision says a board
  opens from givens in one call, so the ticket builds that alone and leaves the second
  constructor to the maintainer.
- **How a cell is addressed.** The specification's stimuli take a cell. The ticket
  expects operations to take a `Position` and to refuse one that names no cell, as
  `Puzzle` does. If a typed cell handle reads better, it is the agent's choice and is
  reported under "Names later tickets need".
- **What a refused operation returns.** One error per `requires` clause is the rule. Two
  clauses can fail at once (a given cell on a solved puzzle); which is reported first is
  the agent's choice, fixed by a test and stated in the documentation.
- **The check and the open question.** The checks are exposed in order, so a consumer
  can count them itself. Nothing here counts for it. If the maintainer answers the open
  question, that is a `spec-change` and a later ticket.
