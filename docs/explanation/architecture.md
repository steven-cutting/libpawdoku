---
title: "Architecture"
kind: "explanation"
audience: [contributor, maintainer, operator, agent]
canonical_for: [system_architecture]
requires: []
---

# Architecture

This library is a library and nothing else. It has no process of its own: no `main`, no
I/O, no clock, and nothing it can do until a consumer calls it. The game consumes it
through WebAssembly later; a command-line tool and Python bindings consume it later
still, as sibling crates in the same workspace.

That constraint is not a limitation waiting for a runtime to be added — it is the
architecture. See [Decision 0001](../decisions/0001-engine-as-a-library.md) for why the
engine is a library of its own, and
[Decision 0008](../decisions/0008-no-std-core.md) for why its core is `no_std`.

## Workspace

The root `Cargo.toml` is a virtual manifest: it names the members, the package fields
every crate inherits, the dependency table every crate draws from, and the
`[workspace.lints]` table every crate opts into. `Cargo.lock` beside it is the pin.

`crates/pawdoku` is the engine and today the only member. `src/lib.rs` is the crate
root, `src/random.rs` is the randomness boundary, `src/sudoku.rs` with the files under
`src/sudoku/` is the rules, `src/solver.rs` with the files under `src/solver/` is the
solver, `src/board.rs` with the files under `src/board/` is the board, and `tests/`
holds the integration tests. Behaviour arrives
as one Rust module per specification module, and a module may use only what its
specification imports, in the direction
[Layering and dependency direction](layering.md) describes.

One module is not behaviour. `src/guide.rs` is a page of the API reference, a map of the
engine in figures: it holds documentation and no item, and `lib.rs` declares it under
`cfg(doc)`, so it exists when rustdoc builds and nowhere else. Its figures and their
styles are in the file, so the page needs nothing from the build that renders it.
[API reference](../reference/api.md) says how a page of that kind is made.

Three siblings join when ticket S04 opens them: `crates/pawdoku-cli`,
`crates/pawdoku-py` (pyo3, built with maturin) and `crates/pawdoku-wasm`
(wasm-bindgen). None exists yet. Every decision below is taken so that each stays a thin
wrapper when it arrives.

## The core is `no_std`

`crates/pawdoku` is `#![no_std]` with `extern crate alloc`. A clock, threads, the
filesystem, the environment and `HashMap` are therefore not avoided by convention but
unnameable: the standard library that would name them is not in scope. The proof is
mechanical rather than a promise. `just wasm-check` checks the crate against
`wasm32v1-none`, a target that has no standard library at all, so a single `std` path
anywhere in the core fails the gate.

With no clock there can be no timeout. Where a computation needs a limit, the limit is a
step budget the caller sets, and the same budget gives the same answer on every machine.
The solver needs none: `solver.allium` shows that every search concludes, so solving
takes the givens alone.

## The randomness boundary

Randomness is the one effect the engine has, and `random.rs` is its boundary. The trait
`RandomStream` is a stream of draws in `[0, 1)`, begun from a seed, indexed from zero,
the same for the same seed and `random_version`. The caller chooses the implementation.
The library ships one generator, `SeededStream`, named by the constant `RANDOM_VERSION`,
so that exact replay holds across callers: a consumer that records the name beside the
seed can replay the same draws in a browser, in Python or on the command line. A test
supplies its draws through the fake, `ReplayStream`, which replays a script and reports
exhaustion rather than panicking.

The core never sources entropy. `deny.toml` bans `rand` and `getrandom` from everything
the workspace ships, and names `pawdoku-cli` as the only crate permitted to wrap them;
the command line is the one place a seed will come from the operating system.

## How well-posedness reaches the rules

`SetPuzzle` in `sudoku.allium` admits only givens that leave something to play and have
exactly one solution. The `sudoku` module cannot decide the second: counting solutions
is the solver's work, and `sudoku` imports nothing. So the answer is carried to the
rules as a value ([decision 0014](../decisions/0014-board-imports-solver.md)).

`sudoku` defines the proof, `WellPosed`: a set of givens and their one solution. A
`Puzzle` is set from a proof and from nothing else, and setting cannot fail, so no
puzzle exists whose givens `SetPuzzle` would refuse.

The proof has one constructor, `WellPosed::vouch`. It returns a `Result` and never
panics, and it checks everything it can see: that there are fewer givens than cells;
that the solution is full, every digit from 1 to 9, and no two peers hold one digit;
and that every given is on the grid and holds the solution's digit at its position.
That last check is also what refuses two givens that disagree about one position.
Uniqueness is the one thing it takes on its caller's word.

**"Only the solver calls the constructor" is a rule review holds, not the compiler.**
The constructor is `pub(crate)`, and Rust cannot restrict visibility to one sibling
module, so any module in the crate could name it. A reviewer who sees `vouch` called
anywhere but the solver has found a defect. Outside test code there is one call, in
`solver::solve` in `src/solver.rs`. The constructor's own checks bound the damage: it
can be handed a wrong claim of uniqueness and nothing else that is wrong. Outside the
crate the rule is mechanical, because nothing there can make a proof: `WellPosed` is
public and its constructor is not, and a doc example on the type that names the
constructor is there to fail to compile.

A `Puzzle` keeps the proof's solution and never hands it out. It answers one question,
whether one digit is the solution's at one position, yes or no, and that answer is
`pub(crate)`, for the board's check. Its `Debug` leaves the solution out. Solved is the
specification's definition, a full grid with no conflict, and never a comparison with
the stored solution. Whoever holds the proof can read its givens and its solution; what
is hidden is the solution once it is inside a puzzle.

`WellPosed`, `Puzzle` and the errors they return are public, because the solver makes
the proof and outside code can therefore make a puzzle: solve, then set. The tests
inside `sudoku` still build the proof by hand, since `sudoku` imports nothing.

## The solver

`solver` is `solver.allium` in Rust: the search that gives a set of givens its verdict
of none, one or many solutions. There is one solver. It is not a trait and not a
parameter, it takes no budget because every search concludes, and it draws on no
randomness because nothing in a search is left to chance
([decision 0014](../decisions/0014-board-imports-solver.md)). It has two entries.

| Entry | Gives | A caller wants it to |
| --- | --- | --- |
| `solver::search(givens)` | A `SearchResult`: the verdict, the solutions found (none, one or two) and the count of guesses. It never fails; the worst givens get `none`. | Learn about givens: whether they can be solved, whether the answer is open, and how much guessing it took. A generator asks this of every candidate. |
| `solver::solve(givens)` | The proof, `WellPosed`, or a refusal. | Set a puzzle: `Puzzle::set(solve(givens)?)`. |

`solve` is built on `search`. It refuses three ways: the givens have no solution, they
have more than one, or they have one and leave nothing to play. The third is the proof
constructor's own guard and reaches the caller as the constructor's error, wrapped:
eighty-one givens that agree with a valid grid get the verdict `one` from `search`, and
`SetPuzzle` still refuses them.

Both take the givens as anything that yields them, in any order, and read them as a
set, so a given handed over twice is one given.

**A result has no status.** The `SearchResult` surface exposes `search.status`, and the
functions here return only once the search has concluded, so being handed a result is
how the status is read; `solver.allium` says so in the sentence above that surface.

**The solver runs once for each puzzle**, as it is set. A `Puzzle` keeps the solution
the proof carried, so nothing asks the solver again while the puzzle is played.

**The search's shape is the specification's and its storage is not.** The count of
guesses is exposed and must be the same every time, so the code makes exactly the splits
`solver.allium` describes: propagation by the three rules until nothing is left, the
three kinds of contradiction, a split on a cell with the fewest candidates, depth first,
and a stop at the second solution. How a branch is kept is the implementation's: a grid
of candidate sets, each a bit to a digit, and the waiting branches as a stack rather
than recursion ([decision 0013](../decisions/0013-metrics-gate.md)). A branch has no
status field, no parent and no depth; where it is says what it is. The module also fixes
the one order the specification leaves open, and documents it: among tied cells the
first in grid order, and among a split's children the lowest digit first. Counts of
guesses depend on that rule, so changing it changes what consumers may have recorded.

## The board

`board` is `board.allium` in Rust: one puzzle as it is being played.
A `Board` is the playable abstraction
([decision 0014](../decisions/0014-board-imports-solver.md)). It is what a consumer
plays on, and the one thing in the engine that changes as a game goes on.

**A board owns its puzzle and hands out values.** The `Puzzle` is a private field, and
no method returns it, a reference to it, a `&mut` to anything, or one of the board's own
collections. Everything the `Playing` surface exposes is read through the board, as
values that are copies:

| The board hands out | Which is |
| --- | --- |
| `BoardCell` | One cell as it stands: its position, its digit, whether it is given, whether it conflicts, and its note while the note is shown. |
| `Note` | One cell's marks. |
| `Move` | One move on the record: its index, kind, target and digit, whether it is undone, and the board as it stood once it had been made. |
| `Check` | One check as asked and answered: its index, how many moves stood, its target, the player's digit, and yes or no. |

That shape is what holds two guarantees. Nothing reaches the puzzle in play except
through the board, so every placement and erasure is recorded, kept up and reversible
(`EveryMoveIsTheBoards`). And no public item of the module returns a digit read from the
solution: the one place the solution is consulted is `Board::check`, which calls the
puzzle's crate-only answer and keeps the yes or no (`ACheckNeverTellsTheDigit`,
`TheSolutionIsNeverShown`).

It also keeps `Board` small. `Playing` exposes five facts about the board, six about
each cell, every move with its readings and every check, and offers seven operations;
one flat type holding all of that would be a god object, and the metrics gate caps a
type at twenty methods. `Board` has nineteen: one to open, nine to read, seven
operations, and the record's two, to write and to reopen. What a cell, a move or a
check can say is a method of that value. Inside, the board is four parts: the puzzle,
the notes, the journal of moves, and the checks.

**A board opens from givens, in one call, or reopens from a record, and comes from
nothing else.** `Board::open` puts the givens to `solver::solve` and sets the puzzle
from the proof, so what it refuses is what the solver refuses. `Board::reopen` does the
same with a record's givens; [The record](#the-record) says the rest. There is no
constructor that takes a `Puzzle`. A puzzle that
had already been played on would arrive holding digits no move recorded, against
`TheMovesReplayToTheBoard` and `EveryMoveIsTheBoards`.

**A puzzle with no board is not a defect.** `OpenBoard` gives every puzzle that is set a
board, and the two-step path, solve and then set, yields a `Puzzle` with none. The
reading: `OpenBoard` is this module's view, of the puzzles set through it. A `Puzzle`
on its own is `sudoku.allium`'s `PuzzleSolving` surface, which stands as the rules' own
boundary and not as a second way into a board. `OneBoardToAPuzzle` holds by ownership:
a board owns its puzzle, a cloned board owns a copy, and no second board can reach
either.

**The moves are a journal, and the undone moves are its tail.** The board keeps every
move in the order made, each with what it displaced, and a count of how many stand. The
moves after that count are the undone ones, so they are always the latest, a move's
index is its place in the list, and a new move discards the tail by cutting the list at
the count. Each kind of move carries what it needs and nothing else, by its type.

**Undo and redo go through the puzzle's own two moves.** `Puzzle` offers placing and
erasing and no other way to write a cell, and those are enough: undoing a placement
erases, or places what stood before; undoing an erasure places the digit again; redo
does the move again. `board.allium` does not say how the digit is put back. The rules
cannot refuse any of these and none can solve the puzzle: undo restores a state that
stood before a move, every move requires an unsolved puzzle, and an unsolved puzzle is
never complete; redo restores the state after a move that was then undone, and undo is
refused once the puzzle is solved. Both calls return a `Result`, and the board converts
the rules' refusal into its own, so that a defect would be a refusal and never a panic.

**Reading back goes forward.** A `Move` says what any cell held once it had been made.
That reading is worked out when the moves are handed out, from the board as it was
opened forward through the moves, and never by taking a move back, so it is the same
for a standing move and an undone one and it cannot disturb the board.

**Peers come from the rules' cells.** Upkeep strikes a placed digit from the peers'
notes, and `sudoku` keeps its peer relation private. The board reads it off what a
puzzle's cell exposes, its row, column, band and stack: two cells are peers when they
share a row, a column, or both a band and a stack, and are not one cell.

**A refusal says which clause failed.** The five operations on a cell take a `Position`,
undo and redo take nothing, and each returns a `PlayError`, one variant for each `requires` clause of the board's rules and one for a
position that names no cell. When several fail at once the first is reported, in a fixed
order: a position off the grid, then a solved puzzle, then the rule's own clauses as
`board.allium` writes them. A refused operation changes nothing.

Nothing is counted from the checks. They are handed out in order, and whether wrong
answers or checks made are counted, and shown, is an open question of `board.allium`.

## The record

`Board::write` writes a board down as a `Record`, and `Board::reopen` makes the same
board from one: `contract Recording` and `surface Reopening` of `board.allium`. The
record is a plain value the caller is handed and keeps wherever it likes. The engine has
no storage effect, no format and no format crate; under the `serde` feature a `Record`
serialises through whatever format a consumer brings.

**A record is minimal.** It holds the givens; the moves in the order made, each with
its kind, its target and its digit; how many of them are undone; and each check's count
of moves, target and digit. It holds nothing that can be worked out from those: no
cell's digit, no note, nothing a move displaced or upkeep struck, no check's answer.
And it holds nothing read from the solution, so a stored record tells a player nothing
a board would not. Each kind of move carries what it needs by its type, so a record
cannot say a placement has no digit.

**Reopening reads the record back.** It puts the givens to the solver, as opening does,
makes every move in order through the board's own guards, takes back the undone ones,
and answers each check again from the solution the solver found. Reading every move
forward and then taking back the last few gives the board that was written, because an
undone move was made after the standing ones and nothing has moved since. Everything a
record leaves out is worked out on the way, by the same code that worked it out the
first time, which is what makes the reopened board exact
(`AReopenedBoardIsTheSameBoard`).

**Deserialising checks nothing; reopening checks everything.** A record's fields are
private, but a deserialised record is whatever its source held, so `Board::reopen` is
where a record is judged. It returns a `Result` and never panics. A record that does
not read back is refused with a `ReopenError` that says why, in a fixed order: more
moves undone than moves, or checks and no move; givens the solver refuses; a move the
board refuses, or one that follows the move that solved the puzzle; moves undone on a
solved puzzle; and a check no board answers. The two counts are asked before the search, so a
record that fails them costs no search.

**One thing cannot be read back.** A check holds the digit that stood in its cell when
it was asked, and after an undo and a new move the moves that stood then may be gone.
So a check is held to what can be known: its target is a cell that is not a given, its
digit is from 1 to 9, it was asked after at least one move, a record with a check has
a move, and its digit is not the right one for a puzzle with a single cell to play,
which would have solved the puzzle before anything could be asked. Its digit is not
compared with any move, and its count of moves may exceed
the moves the record holds. Its answer is never stored, so a record cannot lie about
it. `Record`'s documentation says the same, and that no promise is made yet that a
record written by one version of the engine reopens in another.

## The generator

`generation` is the basic way of `generation.allium` in Rust: a published method, taken
as it stands so that it can serve as a yardstick
([decision 0015](../decisions/0015-basic-generator.md)). It has one entry.

| Entry | Takes | Gives |
| --- | --- | --- |
| `generation::generate(tier, stream)` | A `Tier`, one of five construction settings, and a `&mut dyn RandomStream` | The proof, `WellPosed`, of the givens it made, or a `GenerateError`. |

**It draws through the boundary and from nowhere else**, in a fixed order. Each grid
attempt draws eleven givens that do not conflict, a position and then a digit for each,
and puts them to the solver; a solution is the solution grid, and of two the one that
holds the lower digit at the first position, row by row, where they differ. A seeding
that cannot go on or has no solution is a failed attempt, and a hundred of them are a
refusal. One draw then chooses the bound within the tier's range. Removal visits each of
the 81 positions once in the tier's order, and only tiers 1 and 2, whose order is drawn,
take a draw for each visit. So a generation that finds its grid at once takes 23 draws
in tiers 3 to 5 and 104 in tiers 1 and 2, whatever removal does.

**It takes a stream and not a seed**, because the boundary is a trait and a test supplies
its draws through the fake. It draws from wherever the stream stands, so what replays a
puzzle is the tier, `GENERATION_VERSION`, `RANDOM_VERSION`, the seed and the index the
stream stood at; or a fresh stream for each puzzle.

**The proof is the solver's.** Removal asks `solver::search` for each verdict and hands
the givens it is left with to `solver::solve` once, at the end, so the proof's
constructor keeps its one caller. That last call cannot refuse: the givens have one
solution by construction, and the first visit always empties its position, so something
is left to play. Its refusal is still carried, as `GenerateError::NotProved`, so that a
defect would be a refusal and never a panic.

**What a seed gives depends on the solver.** Eleven givens nearly always have many
solutions, the search stops at two, and which two it reaches follows from the solver's
tie-break and from what its propagation strikes before a guess. A change to either
changes the grids with nothing in `generation` touched, and is a new
`GENERATION_VERSION`. Two tests in `tests/generation.rs` pin what seeds give and say so.

A tier is a construction setting: a range for the bound, a floor for every row and
column and an order of removal. Nothing in the module rates a puzzle or says one tier is
harder than another. What removal reached is what comes back, in the tier's range or
above it; `Tier::most_givens` says which.

## What crosses each boundary

| Consumer | Crosses | Through |
| --- | --- | --- |
| WebAssembly | Values and the seed | wasm-bindgen |
| Python | Values and the seed; errors as exceptions | pyo3 |
| Command line | Values, and the one source of entropy | `pawdoku-cli` |

Python and WebAssembly build their exceptions from each error's `Display` text, which is
why that text is stable API. It is also why every public type is `Send + Sync +
'static`, `Clone`, `Debug` and `#[non_exhaustive]`: a binding may move a value between
threads, copy it across the boundary or print it, and a new field or variant must not
break a consumer that matched on the old ones. `AGENTS.md` states this as invariant 3,
and `tests/api_bounds.rs` holds it at compile time.

A game in play crosses as a `Record` and as nothing else. A `Board` and a `Puzzle` do
not serialise, which `tests/api_bounds.rs` holds at compile time under the `serde`
feature: a board holds
its puzzle's solution, and a record does not. A consumer that wants a game kept between
visits writes the board, serialises the record through the `serde` feature in a format
of its choosing, keeps it where it likes, and reopens it. Whatever comes back is
checked at reopening, not at deserialising.

## What is not here

No I/O, no clock, no threads, no `rand`, no network, no persistence and no user
interface. Those are excluded by design, as
[Purpose and scope](../project/purpose-and-scope.md) records. Puzzle generation is not
among them. The basic generator, a published method checked by the solver's verdict
([decision 0015](../decisions/0015-basic-generator.md)), is specified in
`generation.allium` and built as `generation`, above. The designed generator beside it
is a skeleton of scope, config and open questions, and nothing of it is built. Whether
puzzles are made on the device, ahead of time, or both is open
([decision 0012](../decisions/0012-generation-and-dev-time-judges.md)).

## Related pages

- [Layering and dependency direction](layering.md)
- [Security model](security-model.md)
- [Repository map](../project/repository-map.md)
- [API reference](../reference/api.md)
