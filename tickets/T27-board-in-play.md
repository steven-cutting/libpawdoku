---
id: T27
title: "The board in play: notes, moves, undo and redo, reading back, and the check"
status: done
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

Changes to the list, one line each, as the loops ran:

- The first run was against a crate with no `board` module, so the five opening tests
  failed to compile. The values the surface names (`BoardCell`, `Note`, `Move`,
  `MoveKind`, `Check`, `PlayError`), the journal and the reading were written with
  opening, because `moves()` and `checks()` need their item types. "Deviations" records
  this against step 3.
- The seven operations were then added as stubs that refuse everything, as T26 began
  with a search that answered `none`, so that the tests' one `Op` helper compiled.
  From there each group failed on its assertions before its code was written.
- `Place`: five tests failed, then passed.
- `Erase`, `WriteMark` and `StrikeMark` were taken in one run, not three: eleven tests
  written, six seen to fail before the run stopped, all green after the three methods.
- Upkeep's four lines were green on arrival: `Place` records which notes lost the
  digit, so it could not be written without upkeep. Each was later shown to bite.
- Moved: `placing_the_standing_digit_again_is_refused_and_discards_nothing` to after
  undo, since "discards nothing" needs an undone move to keep.
- Corrected: `redo_retakes_each_kind_of_move_exactly` first failed on its own script,
  which struck a mark that upkeep had already struck. The test was wrong and the board
  was not.
- `CheckCell`: seven tests written, six seen to fail, green with `Board::check`; the
  `dead_code` expectation on the puzzle's answer went in the same loop.
- Reading back's four lines were green on arrival: a move's reading was written with
  `moves()` in the first loop. Each was later shown to bite.
- Added: U `note` `a_note_off_the_grid_is_empty_and_takes_no_mark`, for the two arms of
  the notes' storage that no position on the grid reaches.
- Added to the property: `GivensNeverMove`, asserted after every step against the
  fixture's givens.
- Split: the list's B tests are in two files. `tests/board_rules.rs` holds the rules
  clause by clause; `tests/board.rs` holds the whole game, the surface and the property.
  One file passed the gate's thousand lines for a test file. "Deviations" says more.
- Added after the Codex review: R `undo_and_redo_reach_a_note_hidden_beneath_a_digit`,
  and two checks in the property, because a note beneath a digit was only ever read
  from the moves. "What was verified" says more.
- Struck, as expected: `surface-actor.Playing` and `config-default.side`. Marked T28's:
  the two `contract-signature.Recording` obligations and `surface-actor.Reopening`.
- What the gates taught, each a change of shape and none a suppression. The metrics
  gate holds that a function computes or delegates and not both, so the note's bit
  arithmetic is four small functions; it counts cohesion by the fields a method names,
  so the four moves name the board's fields themselves; and it reports one dispatch
  repeated in three files, so the scripted game is a list of steps and not a third
  enum. Clippy refuses `Option<Option<u8>>`, so what a move does to its cell's digit is
  a small value, `DigitChange`.

The list as it ended, from clause to test. R is `crates/pawdoku/tests/board_rules.rs`, B
is `crates/pawdoku/tests/board.rs`, and U is a unit test in the part of `src/board/` it
names. P is the property, `any_script_keeps_the_board_to_its_rules` in B, which asserts
each invariant under its own name. No row names a snapshot test.

| Clause | Test, or the reason there is none | `plan-spec` obligations |
| --- | --- | --- |
| `OpenBoard`, `LayOutBoard`: eighty-one cells, every note empty, no move, no check | R `opening_gives_eighty_one_cells_no_note_no_move_and_no_check`, R `a_board_shows_its_givens_and_leaves_every_other_cell_empty` | `rule-success.OpenBoard`, `rule-entity-creation.OpenBoard.1`, `rule-success.LayOutBoard`, `entity-fields.Board`, `entity-relationship.Board.cells`, `entity-fields.BoardCell`, `entity-relationship.BoardCell.puzzle_cell` |
| Opening refused: no solution, several, nothing left to play | R `opening_is_refused_for_givens_with_no_solution`, `..._with_several_solutions`, `..._that_leave_nothing_to_play` | none |
| `Place`: the digit, the move on the record | R `a_placement_puts_the_digit_in_the_cell_and_joins_the_record` | `rule-success.Place`, `rule-entity-creation.Place.1`, `entity-fields.Move`, `entity-relationship.Board.moves` |
| `Place` refused on a solved puzzle | B `a_whole_game_is_played_through_the_board_and_then_everything_is_refused`, R `refusals_come_in_a_fixed_order` | `rule-failure.Place.1` |
| `Place` refused on a given | R `placing_is_refused_on_a_given_cell` | `rule-failure.Place.2` |
| `Place` refused for a digit out of range | R `placing_is_refused_for_a_digit_out_of_range` | `rule-failure.Place.3` |
| `Place` refused for the standing digit: nothing recorded, nothing undone discarded | R `placing_the_standing_digit_again_is_refused_and_discards_nothing` | `rule-failure.Place.4` |
| A placement over the player's other digit is a move and remembers what it displaced | R `a_placement_over_the_players_other_digit_is_a_move_that_remembers_what_it_displaced`, R `undoing_a_placement_over_a_digit_puts_the_former_digit_back` | `entity-optional.Move.digit_before` |
| A conflicting placement is a move | R `a_conflicting_placement_is_a_move` | none |
| A position that names no cell | R `a_position_off_the_grid_is_refused_by_every_operation`, R `a_reading_off_the_grid_is_empty` | none |
| Which refusal is reported first | R `refusals_come_in_a_fixed_order` | none |
| Upkeep: struck from every peer that holds it and from no other cell | R `a_placed_digit_is_struck_from_every_peer_that_holds_it_and_from_no_other_cell`, P `UpkeepStrikesPeersOfAPlacement` | `invariant.UpkeepStrikesPeersOfAPlacement` |
| Upkeep reaches notes hidden beneath a digit | R `upkeep_reaches_a_note_hidden_beneath_a_digit` | none |
| The target's own note is kept | R `the_targets_own_note_is_kept_beneath_its_digit` | none |
| Erasing gives nothing back to the peers | R `erasing_gives_nothing_back_to_the_peers` | none |
| `Erase`: the cell emptied, the move on the record | R `an_erasure_empties_the_cell_and_joins_the_record` | `rule-success.Erase`, `rule-entity-creation.Erase.1`, `entity-optional.Move.digit` |
| `Erase` refused on a solved puzzle | B the whole game, R `refusals_come_in_a_fixed_order` | `rule-failure.Erase.1` |
| `Erase` refused on a cell with no digit of the player's | R `erasing_is_refused_on_a_cell_with_no_digit_of_the_players` | `rule-failure.Erase.2` |
| The note beneath the digit shows again | R `the_note_beneath_a_digit_shows_again_when_it_is_erased` | none |
| `WriteMark` | R `a_written_mark_joins_the_note_and_the_record` | `rule-success.WriteMark`, `rule-entity-creation.WriteMark.1` |
| `WriteMark` refused: solved; a cell that does not accept marks; out of range; already written | B the whole game; R `writing_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`, `writing_a_mark_is_refused_for_a_digit_out_of_range`, `writing_a_mark_already_written_is_refused` | `rule-failure.WriteMark.1` to `.4` |
| `StrikeMark` | R `a_struck_mark_leaves_the_note_and_joins_the_record` | `rule-success.StrikeMark`, `rule-entity-creation.StrikeMark.1` |
| `StrikeMark` refused: solved; a cell that does not accept marks; a mark not there | B the whole game; R `striking_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`, `striking_a_mark_that_is_not_there_is_refused` | `rule-failure.StrikeMark.1` to `.3` |
| A mark a placed peer rules out may still be written | R `a_mark_a_placed_peer_rules_out_may_still_be_written` | none |
| `Undo`, each kind taken back exactly, and what an undo leaves on the record | R `undoing_a_placement_leaves_the_board_as_it_stood`, `undoing_a_placement_over_a_digit_puts_the_former_digit_back`, `undoing_an_erasure_puts_the_digit_back`, `undoing_a_written_mark_strikes_it_and_undoing_a_struck_mark_writes_it`; P | `rule-success.Undo` |
| `Redo`, each kind re-taken exactly | R `redo_retakes_each_kind_of_move_exactly`; P | `rule-success.Redo` |
| Both refused with nothing to act on, and once solved | R `undo_is_refused_with_nothing_standing_and_redo_with_nothing_undone`, B the whole game | `rule-failure.Undo.1`, `rule-failure.Redo.1` |
| `can_undo`, `can_redo`, standing and undone moves | R `can_undo_and_can_redo_follow_the_moves`; P | `derived.Board.can_undo`, `derived.Board.can_redo`, `projection.Board.standing_moves`, `projection.Board.undone_moves` |
| The latest standing move and the next to redo | R `undo_takes_back_the_latest_standing_move_and_redo_retakes_the_earliest_undone` | `projection.Board.latest_standing_moves`, `projection.Board.next_to_redo`, `derived.Move.is_latest_standing`, `derived.Move.is_next_to_redo` |
| `CheckCell`: yes for the solution's digit, no for another | R `a_check_answers_yes_for_the_solutions_digit_and_no_for_another`, over every cell the setter left | `rule-success.CheckCell` |
| `CheckCell` refused: solved; an empty cell and a given | B the whole game; R `a_check_is_refused_on_an_empty_cell_and_on_a_given` | `rule-failure.CheckCell.1`, `.2` |
| A check records the digit as it stood and how many moves stood | R `a_check_records_the_digit_as_it_stood_and_how_many_moves_stood`, R `after_move_is_a_count_and_not_a_reference` | `rule-entity-creation.CheckCell.1`, `entity-fields.Check`, `entity-relationship.Board.checks` |
| `holds_players_digit`, `accepts_marks`, `shows_note` | R `a_cells_facts_follow_its_digit_and_whether_it_is_given`, R `a_note_is_shown_only_while_its_cell_is_empty`; P | `derived.BoardCell.holds_players_digit`, `derived.BoardCell.accepts_marks`, `derived.BoardCell.shows_note` |
| `OneBoardToAPuzzle` | Held by ownership: a board owns its puzzle and no second board can reach it. R `two_boards_never_share_a_puzzle` shows a clone played on leaves the first as it was | `invariant.OneBoardToAPuzzle` |
| `TheBoardIsWhole`, `OneBoardCellToAPosition` | R `opening_gives_eighty_one_cells_no_note_no_move_and_no_check`; P | the two `invariant.` obligations |
| `MarksAreDigits` | U `note` `a_note_holds_only_digits_from_one_to_nine`; P | `invariant.MarksAreDigits` |
| `NoMarkInAGiven` | R `writing_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`; P | `invariant.NoMarkInAGiven` |
| `MovesNeverTouchAGiven` | P; the refusals on a given above | `invariant.MovesNeverTouchAGiven` |
| `MoveIndicesRunFromOne`, `MoveIndicesAreDistinct` | P; held by the journal, where a move's index is its place in the list | the two `invariant.` obligations |
| `MovesCarryWhatTheirKindNeeds` | P, as far as the surface shows it: an erasure has no digit and displaced one, every other kind has a digit. Held by the type inside: each kind of `Made` carries its own fields | `invariant.MovesCarryWhatTheirKindNeeds`, `enum-comparable.MoveKind` |
| `UndoneMovesAreTheLatest`, `WhatIsReTakenComesNext` | P; R `undo_takes_back_the_latest_standing_move_and_redo_retakes_the_earliest_undone` | the two `invariant.` obligations |
| `ASolvedBoardHasNothingUndone` | P, on scripts that solve the puzzle; B the whole game | `invariant.ASolvedBoardHasNothingUndone` |
| `TheMovesReplayToTheBoard` | R `the_reading_at_the_latest_standing_move_is_the_board`; P, for the digits, the notes shown and, by erasing on a copy, the notes that wait | `invariant.TheMovesReplayToTheBoard` |
| `ChecksRunFromOne`, `CheckIndicesAreDistinct`, `ACheckIsOnAPlayersCell` | P; R `a_check_records_the_digit_as_it_stood_and_how_many_moves_stood` | the three `invariant.` obligations |
| `EveryMoveIsTheBoards` | P: an operation is accepted exactly when its rule accepts it, and every accepted move is on the record. By shape: no public item reaches the puzzle. U `error` `the_rules_refusals_convert_to_the_boards` | none |
| `UndoAndRedoAreExact` | The undo and redo rows above; P | none |
| `ANewMoveDiscardsWhatWasUndone` | R `a_new_move_discards_every_undone_move`; P | none |
| `AMarkIsAMove` | R `undoing_a_written_mark_strikes_it_and_undoing_a_struck_mark_writes_it` | none |
| `UndoStopsWithThePuzzle` | B the whole game; P | none |
| `EveryPastBoardIsReadable` | R `every_move_reads_back_the_board_as_it_stood_after_it`, R `reading_back_takes_nothing_back`; B the whole game, for a solved board; P | none |
| `GivensNeverMove` | P, against the fixture's givens after every step | none |
| `ANoteWaitsBeneathADigit` | R `the_note_beneath_a_digit_shows_again_when_it_is_erased`, R `upkeep_reaches_a_note_hidden_beneath_a_digit`, R `undo_and_redo_reach_a_note_hidden_beneath_a_digit`; P, which shows the waiting notes on a copy after every step | none |
| `EveryCheckIsKept` | R `a_check_is_not_a_move_and_undo_does_not_remove_it`, R `after_move_is_a_count_and_not_a_reference`; P | none |
| `ACheckNeverTellsTheDigit` | R `a_check_never_tells_the_digit`, and by shape, below | none |
| `TheSolutionIsNeverShown` | R `a_board_does_not_print_its_solution`, and by shape, below | none |
| What `Playing` exposes | B `everything_playing_exposes_can_be_read_through_the_board` | `surface-exposure.Playing` |
| What `Playing` provides: the `when` clauses, and the three offered and still refused | P, where `is_offered` is the `when` clause and `is_accepted_by_its_rule` the rule; R `an_offered_operation_may_still_be_refused` | `surface-provides.Playing` |
| A whole game | B `a_whole_game_is_played_through_the_board_and_then_everything_is_refused` | none |
| The stable text of each refusal | U `error` `refusal_texts_are_stable` | none |
| The notes' storage off the grid | U `note` `marks_are_written_and_struck`, `a_note_off_the_grid_is_empty_and_takes_no_mark` | none |
| Every public type meets invariant 3 | `tests/api_bounds.rs`, both tests | none |
| `surface-actor.Playing` | Struck: `Playing` names a context, a board, and no actor | `surface-actor.Playing` |
| `config-default.side` | Struck: the figure is `sudoku`'s, tested there; the board reads it as `sudoku::SIDE` through `in_range` | `config-default.side` |
| `contract Recording`, `surface Reopening` | T28's | `contract-signature.Recording.write`, `contract-signature.Recording.reopen`, `surface-actor.Reopening` |

**The last two guarantees, by the shape of the API.** The module's public surface was
read item by item. `Board`: `open`, `status`, `is_full`, `is_consistent`, `can_undo`,
`can_redo`, `cell`, `cells`, `moves`, `checks`, `place`, `erase`, `write_mark`,
`strike_mark`, `undo`, `redo`, `check`. `BoardCell`: `position`, `row`, `column`,
`digit`, `is_given`, `is_conflicting`, `note`, `holds_players_digit`, `accepts_marks`,
`shows_note`. `Note`: `contains`, `digits`, `len`, `is_empty`. `Move`: `index`, `kind`,
`target`, `digit`, `is_undone`, `digit_after`, `note_after`. `Check`: `index`,
`after_move`, `target`, `digit`, `is_right`. `MoveKind` and `PlayError` carry positions
and digits the caller supplied. Every digit any of these returns is a given's or one the
player placed or marked. The solution is read in one place, `Board::check`, through
`Puzzle::is_solution_digit`, and what is kept of it is one `bool`. No item returns a
`Puzzle`, a reference to one, a `&mut` to anything, or a count of checks or of wrong
answers. `Board`'s `Debug` prints its puzzle through `Puzzle`'s own, which leaves the
solution out.

### Names later tickets need

**Public** (`pawdoku::board`), each with a doc example that runs and asserts. The
doctest is named after the item: `board::Board::place` and so on, 51 in all with the
module's own.

| Item | Signature | Doctest |
| --- | --- | --- |
| `Board` | `Debug, Clone`; `#[non_exhaustive]`, fields private; no `PartialEq`, no serde | `board::Board` |
| `Board::open` | `pub fn open(givens: impl IntoIterator<Item = Given>) -> Result<Self, SolveError>` | `board::Board::open` |
| `Board::status` | `pub const fn status(&self) -> Status` | `board::Board::status` |
| `Board::is_full`, `is_consistent`, `can_undo`, `can_redo` | `pub fn ...(&self) -> bool` | one each |
| `Board::cell` | `pub fn cell(&self, position: Position) -> Option<BoardCell>` | `board::Board::cell` |
| `Board::cells` | `pub fn cells(&self) -> impl Iterator<Item = BoardCell> + '_`, in grid order | `board::Board::cells` |
| `Board::moves` | `pub fn moves(&self) -> impl Iterator<Item = Move> + '_`, in the order made | `board::Board::moves` |
| `Board::checks` | `pub fn checks(&self) -> impl Iterator<Item = Check> + '_`, in the order asked | `board::Board::checks` |
| `Board::place`, `write_mark`, `strike_mark` | `pub fn ...(&mut self, position: Position, digit: u8) -> Result<(), PlayError>` | one each |
| `Board::erase` | `pub fn erase(&mut self, position: Position) -> Result<(), PlayError>` | `board::Board::erase` |
| `Board::undo`, `redo` | `pub fn ...(&mut self) -> Result<(), PlayError>` | one each |
| `Board::check` | `pub fn check(&mut self, position: Position) -> Result<Check, PlayError>` | `board::Board::check` |
| `BoardCell` | `Debug, Clone, Copy, PartialEq, Eq`; getters `position` (`Position`), `row`, `column` (`u8`), `digit` (`Option<u8>`), `is_given`, `is_conflicting`, `holds_players_digit`, `accepts_marks`, `shows_note` (`bool`), `note` (`Option<Note>`, `Some` only while shown) | the type and one for each getter |
| `Note` | `Clone, Copy, PartialEq, Eq, Hash, Default`, a hand-written `Debug` that prints the set; `contains(u8) -> bool`, `digits() -> impl Iterator<Item = u8>`, `len() -> usize`, `is_empty() -> bool` | the type and one for each method |
| `Move` | `Debug, Clone, PartialEq, Eq`; `index() -> usize`, `kind() -> MoveKind`, `target() -> Position`, `digit() -> Option<u8>`, `is_undone() -> bool`, `digit_after(Position) -> Option<u8>`, `note_after(Position) -> Note` | the type and one for each method |
| `MoveKind` | `Debug, Clone, Copy, PartialEq, Eq, Hash`; `Place`, `Erase`, `WriteMark`, `StrikeMark` | `board::moves::MoveKind` |
| `Check` | `Debug, Clone, Copy, PartialEq, Eq`; `index() -> usize`, `after_move() -> usize`, `target() -> Position`, `digit() -> u8`, `is_right() -> bool` | the type and one for each getter |
| `PlayError` | `Debug, Clone, PartialEq, Eq, thiserror::Error`; `NoSuchCell { position }`, `Solved`, `GivenCell { position }`, `DigitOutOfRange { digit }`, `DigitAlreadyStands { position, digit }`, `NoPlayersDigit { position }`, `MarksNotAccepted { position }`, `MarkAlreadyWritten { position, digit }`, `MarkNotThere { position, digit }`, `NothingToUndo`, `NothingToRedo` | `board::error::PlayError` |

The choices the open points left to this ticket:

- **How a cell is addressed.** The five operations on a cell take a `Position` and
  refuse one that names no cell with `PlayError::NoSuchCell`, as `Puzzle` does. Undo and
  redo take nothing. There is no cell handle.
- **What a refused operation returns.** One variant for each `requires` clause, and one
  for a position off the grid. `holds_players_digit` and `accepts_marks` are one clause
  and one variant each, so erasing a given is `NoPlayersDigit` and not `GivenCell`.
  The first that fails is reported, in this order: `NoSuchCell`, then `Solved`, then the
  rule's clauses as `board.allium` writes them. `refusals_come_in_a_fixed_order` fixes
  it and each method's `# Errors` section states it.
- **Opening's refusal** is the solver's own `SolveError`. The module defines no opening
  error.
- **A check is handed back.** `Board::check` returns the `Check` it kept.
- **Reading back off the grid.** `Move::digit_after` gives nothing and `note_after` the
  empty note for a position that names no cell.

**What T28 needs of the board's inside.** A child module, `src/board/record.rs`, can
read the board's private fields.

| Inside | What it is | What a record wants of it |
| --- | --- | --- |
| `Board.puzzle: Puzzle` | The puzzle in play | `puzzle.givens()` for the record's givens; `puzzle.is_solution_digit` for a reopened check's answer |
| `Board.notes: note::Notes` | The eighty-one notes | Nothing: reopening derives them by replaying the moves |
| `Board.journal: journal::Journal` | `made: Vec<Made>` and `standing: usize`, the moves in order and how many stand | The moves' kind, target and digit, which `Board::moves` already hands out as `Move`; the undone count is the moves' count less `journal.standing()` |
| `Board.checks: Vec<Check>` | Every check, in order | `after_move`, `target` and `digit` of each; reopening pushes checks built with `Check::new(index, after_move, target, digit, is_right)`, which is `pub(super)` |
| `journal::Made` | One move as made: `Place { target, digit, before, struck }`, `Erase { target, before }`, `WriteMark { target, digit }`, `StrikeMark { target, digit }` | Nothing directly: `before` and `struck` are what a record leaves out and replay rebuilds |

Reopening as T28 describes it needs no new way into the board: `Board::open` on the
record's givens, the four public moves in order, `undo` once for each undone move, and
the checks pushed. A move after the puzzle is solved, and an undo on a solved puzzle,
are refused by the board already. `Board` has 17 methods; T28's two make 19 of the 20
the gate allows. The tests' full view is `view` in `tests/board.rs`, the arbitrary
script is the strategy `ask` beside it, and the scripted game is `THE_SCRIPTED_GAME` in
`tests/snapshots.rs`, played by `played`.

### Snapshots taken

All four are in `crates/pawdoku/tests/snapshots/`, taken by tests in
`crates/pawdoku/tests/snapshots.rs` through `Board` alone after the list was empty, and
each file was read before it was committed. The render helper, `render_board`, draws
from `Board::status`, `is_full`, `is_consistent`, `can_undo`, `can_redo`, `cells`,
`moves` and `checks`; nothing was added to the library for it. No picture holds
anything read from the solution but a check's answer. The solved board shows the full
grid because the script placed it; the script takes its digits from `solver::solve`.

The named script is `THE_SCRIPTED_GAME`: eighteen steps that leave eleven moves, the
last undone, two checks, a note waiting beneath a digit and a conflict standing.

| Test | File | Shows |
| --- | --- | --- |
| `snapshot_the_board_as_opened` | `snapshots__tests__snapshot_the_board_as_opened.snap` | The fixture as opened: unsolved, nothing to undo or redo, the grid, the givens, no conflict, no note, no move, no check |
| `snapshot_the_board_after_the_scripted_game` | `snapshots__tests__snapshot_the_board_after_the_scripted_game.snap` | The board `THE_SCRIPTED_GAME` leaves: a 4 and a conflicting 5 placed, three notes of which one waits, eleven moves of every kind with the last undone, and a check answered each way |
| `snapshot_the_board_played_to_the_end` | `snapshots__tests__snapshot_the_board_played_to_the_end.snap` | The fixture solved: full, consistent, nothing offered, fifty-one placements |
| `snapshot_the_boards_refusals` | `snapshots__tests__snapshot_the_boards_refusals.snap` | The `Display` text of each of the eleven `PlayError` variants, each got by asking a board for it |

No earlier snapshot changed: T25's four under `src/sudoku/snapshots/`, T26's six and
the randomness boundary's one are as they were.

### What was verified, and how

Done on 2026-10-01 in the supplied Supacode worktree, on `449aec0`, the merge of T26.
`.pixi/` was absent, so `just initialize` was run, as step 1 authorises. `just check`
was green before any edit. Commands used rustup's cargo through
`PATH="$HOME/.cargo/bin:$PATH"`.

The `Undo` comment of `board.allium` carries the sentence the ticket asks for: "Whether
it is put back through the rules' own placing and erasing or written directly is not
said here." So undo and redo go through `Puzzle::place` and `Puzzle::erase`, and
`sudoku` gained no second way to write a cell.

`just plan-spec board` printed 78 obligations and an empty `diagnostics` array; the
table above accounts for each.

Closing lines of each recipe, from the last run:

| Recipe | Closing output |
| --- | --- |
| `just fmt-check` | `cargo fmt --all --check` (no diagnostic) |
| `just clippy` | ``Finished `dev` profile [unoptimized + debuginfo] target(s)``, no warning |
| `just metrics` | `::notice::Quality score: 100.0% (510 functions analyzed)`, no finding, and the probe satisfied |
| `just test` | `206 tests run: 206 passed, 0 skipped`; `test result: ok. 107 passed; 0 failed` and `ok. 1 passed` (the `compile_fail` example) for the doctests |
| `just snapshots-check` | `info: no unreferenced snapshots found`; `info: no snapshots to review` |
| `just wasm-check` | both feature sets on `wasm32-unknown-unknown` and on `wasm32v1-none`, each `Finished` |
| `just features` | `--no-default-features` and `--features serde`, each `Finished` |
| `just coverage` | `TOTAL` lines `99.83%` (2301 lines, 4 missed); `Finished report saved to target/llvm-cov/lcov.info` |
| `just doc` | `Generated .../target/doc/pawdoku/index.html` |
| `just check-docs` | `Validated 41 pages and 42 canonical topics.` |
| `just check` | `The worktree matches the check baseline.`; `All checks passed and the worktree is unchanged.` |

`just test` takes about a second once built. The slowest test is the property, at 0.9
seconds for 128 scripts, and about three under coverage; nothing else costs a tenth of that.

Coverage of the module alone, from the same report:

| File | Lines | Missed | Cover |
| --- | --- | --- | --- |
| `board.rs` | 158 | 0 | 100.00% |
| `board/cell.rs` | 46 | 0 | 100.00% |
| `board/check.rs` | 30 | 0 | 100.00% |
| `board/error.rs` | 85 | 0 | 100.00% |
| `board/journal.rs` | 93 | 0 | 100.00% |
| `board/moves.rs` | 31 | 0 | 100.00% |
| `board/note.rs` | 97 | 0 | 100.00% |
| `board/reading.rs` | 24 | 0 | 100.00% |

The four missed lines are three in `random.rs` and the one T25 reported in
`sudoku/puzzle.rs`; this ticket touched neither.

**The tests were shown to bite.** Twenty-three deliberate breaks before the review, and
two after it that are described below, each made, run through `just test` and reverted. Those runs stop at the first failures, so a row is what failed
before the run stopped. P is the property.

| Break | Tests that failed |
| --- | --- |
| Upkeep strikes nothing | P, `a_placed_digit_is_struck_from_every_peer_that_holds_it_and_from_no_other_cell`, and two more |
| Upkeep strikes every cell that holds the mark, peer or not | P, `a_placed_digit_is_struck_from_every_peer_that_holds_it_and_from_no_other_cell` |
| Upkeep passes over a note hidden beneath a digit | P, `upkeep_reaches_a_note_hidden_beneath_a_digit` |
| A cell is its own peer, so its own mark is struck | P, `the_targets_own_note_is_kept_beneath_its_digit`, `the_note_beneath_a_digit_shows_again_when_it_is_erased` |
| The standing digit may be placed again | P, `placing_the_standing_digit_again_is_refused_and_discards_nothing`, `an_offered_operation_may_still_be_refused` |
| Undo gives no struck mark back | P, `everything_playing_exposes_can_be_read_through_the_board` |
| Undo leaves the digit the move put there | P, `a_new_move_discards_every_undone_move`, and one more |
| Redo does not strike again | P, `everything_playing_exposes_can_be_read_through_the_board` |
| A new move keeps the undone moves | P, `a_new_move_discards_every_undone_move`, `after_move_is_a_count_and_not_a_reference` |
| `can_undo` is true on a solved puzzle | P, the whole game |
| Redo on a solved puzzle says there is nothing to redo | the whole game |
| A check answers the opposite | P, `a_check_answers_yes_for_the_solutions_digit_and_no_for_another`, and three more |
| A check may be asked of a given | P, `a_check_is_refused_on_an_empty_cell_and_on_a_given` |
| A check counts every move on the record, undone included | P |
| A reading leaves the notes out | P, `everything_playing_exposes_can_be_read_through_the_board` |
| A note beneath a digit is shown | P, `everything_playing_exposes_can_be_read_through_the_board` |
| A solved puzzle is reported before a position off the grid | `refusals_come_in_a_fixed_order` |
| A mark may be written beneath a digit | P, `writing_a_mark_is_refused_on_a_cell_that_does_not_accept_marks`, `refusals_come_in_a_fixed_order` |
| A mark may be struck that is not there | P, `an_offered_operation_may_still_be_refused` |
| An erasure writes a mark in a peer's note | P, `erasing_gives_nothing_back_to_the_peers`, `an_erasure_empties_the_cell_and_joins_the_record` |
| Redo re-takes the latest undone move, not the earliest | P, `redo_retakes_each_kind_of_move_exactly`, and one more |
| An erasure of a given reaches the rules | `erasing_is_refused_on_a_cell_with_no_digit_of_the_players`, on the refusal's variant; the rules still refused it |
| `Board::undo` does not ask whether the puzzle is solved | None, and none can: see below |

**One break no test can see.** With the solved guard taken out of `Board::undo`, every
test still passes. On a solved puzzle the latest standing move is the placement that
solved it, since nothing is accepted afterwards. Undo then asks the rules to erase or
place, the rules refuse because the puzzle is solved, and the conversion reports
`PlayError::Solved`: the same refusal, by a longer road. The guard stays because
`board.allium` states it, `requires: board.can_undo`, and because it makes the refusal
independent of what the latest move was.

The `proptest-regressions/` files the breaks left were deleted; they recorded failures
of code that was broken on purpose.

**A Codex adversarial review, round 1**, was run at the maintainer's request on the
three commits through `9c33e66`. Its report is scratch, under `ai_tmp/`. It found no
defect in the board and three things to fix, all of which were fixed:

- **A note beneath a digit was only ever read from the moves.** The picture reads a
  waiting note through `note_after`, so a board whose own notes had drifted from its
  moves beneath a digit would have passed. The property now erases every digit of the
  player's on a copy after each step and compares the notes that shows with the
  picture, and it keeps the cells beside each picture and compares them after an undo
  and a redo. R `undo_and_redo_reach_a_note_hidden_beneath_a_digit` is the fixed case.
  The break the review described, undo giving a struck mark back only to an empty
  peer, was then made: the property and the new test both failed.
- **`a_board_does_not_print_its_solution` looked only for a row as a string of
  digits.** A grid printed as a list of numbers would have passed. It now reads the
  plain and the pretty `Debug` with the white space taken out and looks for a row both
  ways. With the solution added to `Puzzle`'s `Debug`, the run stopped at the two tests
  inside `sudoku` that hold the same thing, before this one ran; that this one fails
  too is reasoned from its text and was not seen.
- **Two sentences were wrong.** The architecture page counted `Board`'s seventeen
  methods as one, seven and seven, where the readings are nine; and it, the module's
  documentation and these notes said every operation takes a `Position`, which undo and
  redo do not.

**The pull request's reviews, round 1.** Copilot and Codex each reviewed `19c8520` on
pull request 28 and each made the same one finding: the doc comment on
`BoardCell::position` still said every operation of the board takes a position. It was
the fourth copy of the sentence the local review had found three of. It now says every
operation on a cell does, and that undo and redo take none.

The local review could not run `just check` in its sandbox; every figure here is from this
session's own runs.

Acceptance criteria read against the code:

- **What `src/board` names.** `crate::sudoku` and `crate::solver`, and no other engine
  module; not `crate::random` either. `just metrics` holds the boundary rule
  `board_imports_sudoku_and_solver`.
- **No way to the puzzle.** No public item returns a `Puzzle`, a reference to one, a
  `&mut` to anything, or a digit read from the solution; `Board` has no constructor
  that takes a `Puzzle`; no public item returns a count of checks or of wrong answers.
- **No recursion**, in code or tests. `just metrics` reports no finding.
- **Suppressions.** No `#[allow]`, no `qual:allow` and no `#[expect]` added. The
  answer's expectation in `src/sudoku/puzzle.rs` is gone, the whole `cfg_attr`
  attribute, and nothing else under `src/sudoku` or `src/solver` changed.
- **The method cap.** `Board` has 17 methods, `BoardCell` 14, `Note` 10 and its `Debug`,
  `Move` 8, `Check` 6, `Journal` 9, `Made` 6, `Notes` 4 and `Reading` 4, private ones
  counted.
- **Doc examples.** Each of the 50 public items of `board` has one, and the module a
  fifty-first; `just test` runs them.

### Deviations, and why

- **One file outside the list: `crates/pawdoku/tests/board_rules.rs`.** The ticket
  lists one test file for the board. The rule-by-rule tests alone come to about 880
  code lines, and the gate allows a test file a thousand, so the whole game, the
  surface and the property could not join them; and T28 adds reopening to
  `tests/board.rs`. The rules went to a second file. Each carries the helpers it uses,
  as `docs/reference/testing.md` asks of files under `tests/`, so `Op`, the picture and
  the full view are written twice. The alternative inside the list was to move tests
  into `src/board/`, where they could reach private items; from outside they cannot.
- **The ticket names the wrong file for the attribute.** It says the answer's
  `dead_code` expectation is in `crates/pawdoku/src/sudoku.rs`. It was in
  `crates/pawdoku/src/sudoku/puzzle.rs`, on `Puzzle::is_solution_digit`, and that is the
  one line of `sudoku` this ticket changed.
- **Two sentences outside the list**, one in `README.md` and one in
  `crates/pawdoku/README.md`. Each says what the crate holds today, and each would have
  left the board out. T26 made the same two edits for the solver.
- **Step 3, one line at a time.** Three things were not one loop to a line. The values
  the surface names, the journal and the reading were written whole with opening,
  because the readings `moves()` and `checks()` could not compile without them, and the
  unit tests of the note and of the refusal were written with their code. `Erase`,
  `WriteMark` and `StrikeMark` were one run. And eight lines were green on arrival:
  upkeep's four and reading back's four. Every operation's own tests failed on their
  assertions before its code was written, and the lines that arrived green were broken
  on purpose afterwards; the table above lists the breaks.
- **The refusals of `Undo` and `Redo` are two each.** The rule is one error to a
  `requires` clause, and each has one clause, `can_undo` or `can_redo`. A solved puzzle
  is reported as `Solved`, as every other operation reports it, and an unsolved one
  with nothing to act on as `NothingToUndo` or `NothingToRedo`. The clause is a
  conjunction and a caller is told which half failed.
- **The peers are worked out in `board`.** Upkeep needs the peer relation, `sudoku`
  keeps `Position::is_peer_of` private, and this ticket may not widen it. So
  `BoardCell::is_peer_of` reads the relation off what a puzzle's `Cell` exposes: its
  row, column, band and stack. It is `sudoku.allium`'s definition applied to the
  rules' own facts, not a second definition, but it is a second place the definition is
  written. Widening `is_peer_of` to `pub(crate)` would remove it, and is the
  maintainer's to allow.
- **A conversion no caller reaches.** `impl From<MoveError> for PlayError` carries the
  rules' refusal of a move the board had admitted. The board checks the rules' guards
  first, so no operation takes that road; it is there so that the `Result` the puzzle
  returns is propagated and never unwrapped. A unit test calls it directly, as T26's
  does for the proof's error.

### Handed back

- **Peers.** Whether `sudoku` should offer its peer relation to the crate, so the board
  need not restate it. One line in `src/sudoku.rs`; the maintainer's.
- **The refusal's grain.** `PlayError` follows the clauses, so erasing or checking a
  given cell says "holds no digit of the player's" where `sudoku::MoveError` says
  "holds a given". A consumer that wants the finer reason reads the cell's facts. If
  the game wants it in the refusal, that is two more variants and a later change.
- **Opening from a proof**, the first open point, is untouched: a caller that has
  solved the givens pays for a second search when it opens a board.
- **The open question** on what is counted from the checks is unanswered, and nothing
  counts.
- **`Note::default()`** is public: the empty note. Nothing else makes a note outside
  the crate, and no operation takes one.
- **For T28.** `tests/board.rs` is at about 550 code lines of the thousand a test file
  is allowed, with the full view and the script strategy in it. The record's property
  has room there; the rules file does not have room for more.

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
