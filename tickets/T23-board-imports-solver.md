---
id: T23
title: "Spec change: board imports solver, the proof sentence in sudoku, and decision 0014"
status: open
depends_on: [T12, T22]
parallel_with: []
branch: ticket/t23-board-imports-solver
estimated_size: M
---

# T23: Spec change: board imports solver, the proof sentence in sudoku, and decision 0014

## Context

The crate holds the randomness boundary and nothing else; no engine module is built. The
first three to be built are `sudoku`, then `solver`, then `board` (tickets T25 to T28).
Before any of them, one thing the specifications say has to change, and the design the
three share has to be written down where a later ticket can cite it.

**What has to change.** `docs/specs/board.allium` says the solver "is not imported"
(Excludes, lines 73-75). `docs/explanation/layering.md` gives `board` the row "imports
`sudoku`", and `rustqual.toml`'s rule `board_imports_sudoku` forbids `crate::solver` in
`src/board`. But a board answers a check against the puzzle's one solution
(`CheckCell`, lines 465-490), and only the solver finds a solution. T12 left this open
on purpose: "whether the Rust board reaches the solver through an import in the spec or
only in the implementation is the `rust-change` that builds it". The maintainer decided
on 2026-09-30: the board imports the solver, and the specification says so first.

**Why it is safe.** `board` is a leaf: nothing imports it, so the new edge closes no
cycle. The layering page's load-bearing claim is that every model of a player (`reach`,
`effort`, `lapse`, `human-solving`) is testable without the others and without a
generator; `board` is not a model of a player, and no model gains an import.

**The design the three modules share**, decided by the maintainer on 2026-09-30. This
ticket builds none of it; it records it as a decision so that T25 to T28 cite a page.

1. **Build order:** `sudoku`, then `solver`, then `board`. In this repository the grid
   and the rules are `sudoku.allium`, and the board is the play layer that stands on it.
2. **The board imports the solver** (this ticket).
3. **Well-posedness is carried by a proof value.** `sudoku` defines a type, `WellPosed`,
   holding a set of givens and their solution. Only `solver` produces it. `Puzzle` is
   built from it, so no `Puzzle` exists whose givens `SetPuzzle` would refuse, and
   `sudoku` never names `crate::solver`.
   - Its constructor is `pub(crate)` and fallible: it returns a `Result` and never
     panics. It always checks that there are fewer givens than cells; that the solution
     is full, every digit in range, and free of conflict; and that every given is on the
     grid and matches the solution at its position. Uniqueness is the one thing it takes
     on the solver's word.
   - Rust cannot restrict visibility to one sibling module, so "only the solver calls
     the constructor" is held by review inside the crate; outside the crate it is
     mechanical.
4. **`Puzzle` keeps the solution privately.** It answers one question, yes or no:
   whether one digit is the solution's at one position. That answer is `pub(crate)`,
   for the board's check; the digits themselves are never exposed through `Puzzle`.
   "Solved" stays the specification's definition, full and consistent, and is not a
   comparison with the stored solution. The solver runs once per puzzle, as it is set.
5. **One solver, not swappable:** no trait and no generic parameter for the solver, and
   no optional-solver argument. `solver.allium` has no step budget, because its search
   always concludes, so solving takes the givens alone.
6. **The API's shape.** `solver` has two public entries: `search`, which returns what
   `SearchResult` exposes (the verdict, the guesses and the solutions found, at most
   two), and `solve`, built on it, which returns the proof or a refusal. The proof's
   givens and solution can be read by whoever holds it, as `SearchResult` exposes them.
   `Puzzle` is set from the proof and cannot fail. `Board` opens from givens in one call and reopens
   from a record in one call, each running the solver itself. The two-step path (solve,
   then set) stays public for code that wants a puzzle without a board.
7. **`Board` hides its puzzle.** A board is the playable abstraction: it owns its
   `Puzzle` and hands out values of its own, never the puzzle, so no question reaches a
   puzzle in play without the board recording it.
8. **The record is a plain value the caller keeps;** there is no storage effect. It
   holds the givens, the moves, how many are undone, and each check's `after_move`,
   target and digit. Reopening runs the solver, reads the moves forward through the
   board's own guards, derives everything else again and refuses a record that does not
   read back.

**What that asks of the specification text.** All of it is comment text; no entity, rule,
invariant or surface changes, and decision 4 is an implementation choice the modules
already permit: how anything is stored is excluded by each module, and
`solution_digit_at` is a function of the givens whatever keeps its answer.

- `board.allium`: the `use` line; Excludes 73-75; Dependencies 86-89; the `CheckCell`
  comment 476-480; the `reopen` comment 624-630, whose sentence "No move is re-made and
  no rule of this module runs for what the record holds" reads as a ban on reading the
  moves forward and has to become a statement about the result; and
  `ARecordIsEnoughOnItsOwn` 771-776, which says reopening asks the solver "for its
  verdict and nothing more", untrue once reopening takes the proof.
- `board.allium` again, for how T27 builds undo and redo: Excludes 66-68 and the `Undo`
  comment 420-422 say a cell's digit is written back and "no rule of the rules is
  asked to run again". T27 puts the digit back through the puzzle's own placing and
  erasing, so that `Puzzle` keeps one way to write a cell. The two cannot be told
  apart from outside, but the text forbids the second, so the text says less.
- `sudoku.allium`: one sentence in the `SetPuzzle` comment (159-168), so that the proof
  value has a clause to answer to.
- `solver.allium`: one sentence in the comment above `surface SearchResult` (638-639).
  The surface exposes `search.status`, and T26's `search` is a function that hands
  back a search only once it has concluded, so a caller has no status to read.

An unreferenced `use` is not a diagnostic: `generation.allium` imports `solver` and
`reach` and names neither, and the nine modules check clean on the pinned binary
(verified 2026-09-30).

**Every restatement of board's imports**, found by search on 2026-09-30:

| Where | What it says today |
| --- | --- |
| `docs/specs/board.allium` 73-75, 86-89, 104, 476-480, 771-776 | as above |
| `docs/explanation/layering.md` 18, 24, 30-31 | the `solver` row's "Imported by" is `generation`; the `board` row imports `sudoku`; "`board`, the puzzle in play, imports the rules alone" |
| `docs/explanation/specifications.md` 36 | "It imports the rules alone" |
| `docs/how-to/work-with-the-specs.md` 21 | "It imports `sudoku.allium` alone" |
| `rustqual.toml` 181-193 | rule `board_imports_sudoku`, `crate::solver` forbidden, reason "board may import only sudoku and random" |
| `tests/fixtures/metrics-violation/src/board.rs` 1 | the doc comment names the rule |

The done tickets T08, T12, T22 and S09 say "sudoku alone" or quote the old rule name.
They are records of their day and are not edited.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11 and §13;
`.agents/skills/spec-change/SKILL.md`; `docs/specs/board.allium`, `docs/specs/sudoku.allium`
and `docs/specs/solver.allium` in full; `docs/explanation/layering.md`; `rustqual.toml`;
`tickets/T22-metrics-gate.md` steps 5 and 6 (the probe and how it was proved live);
`docs/decisions/0012-generation-and-dev-time-judges.md` as the shape of a decision
record; `docs/decisions/README.md` ("Writing a new one").

## Goal

`board.allium` imports `solver.allium` and says why; every page and rule that restates
what the board imports agrees with it; `sudoku.allium` carries the proof sentence;
`board.allium` no longer says how undo and redo put a digit back, and `solver.allium`
says a caller handed a concluded search reads no status; a
decision record states the eight decisions above and their consequences. `just check` is
green, with `9 specifications, no diagnostics and no findings.` from both specification
gates and the metrics probe naming ten rules, the board's under its new name.

## Non-goals

- No Rust under `crates/`. The modules are T25 to T28.
- No change to any entity, rule, invariant, surface, contract or config in any module.
  Comments and one `use` line only.
- No open question answered, added or removed. `board.allium`'s one open question (what
  is counted from the checks) is carried unchanged.
- No expression that names a solver entity. `solution_digit_at` stays a black box; the
  comment ties it to the solver's one solved branch.
- No edit to a done ticket, and no edit to any file in the game's repository.
- No new architecture rule and no threshold change in `rustqual.toml`: one rule is
  renamed and loses one forbidden path.

## Files touched

| Path | Change |
| --- | --- |
| `docs/specs/board.allium` | The `use` line and seven comment edits (step 2) |
| `docs/specs/sudoku.allium` | One sentence in the `SetPuzzle` comment (step 3) |
| `docs/specs/solver.allium` | One sentence in the comment above `surface SearchResult` (step 3) |
| `docs/explanation/layering.md` | The `board` and `solver` rows, the prose, and the reason the edge is safe (step 5) |
| `docs/explanation/specifications.md` | Line 36, the board paragraph's import sentence |
| `docs/how-to/work-with-the-specs.md` | Line 21, the `board.allium` row's last sentence |
| `rustqual.toml` | The board rule: name, forbidden list, reason (step 4) |
| `tests/fixtures/metrics-violation/src/board.rs` | Line 1 names the rule under its new name |
| `docs/decisions/0014-board-imports-solver.md` | New decision record (step 6); the number is the next free one on the day |
| `docs/decisions/README.md` | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | Entry for the record. A frozen file (CONVENTIONS.md §11): a T00 follow-up, as T22's entry for 0013 was |
| `CHANGELOG.md` | Under Unreleased: the import change; the count of decision records |
| `tickets/README.md` | T23's index row set `done` |
| `tickets/T23-board-imports-solver.md` | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t23-board-imports-solver` from `main` (README.md "How
   to pick up a ticket"). Run `just initialize` if `.pixi/` is absent; it uses the
   network and this ticket authorises it. Run `just check-specs` and `just analyse-specs`
   before any edit and quote their closing lines.

2. **`board.allium`.** Follow the `spec-change` skill. Add, directly under line 104:

   ```text
   use "./solver.allium" as solver
   ```

   Then the seven comment edits. The wording below is the ticket writer's proposal; keep
   the `--` prefix, the indent and the wrap width of the lines around each.

   - Excludes 73-75 becomes: "Finding the solution. solution_digit_at below is a black
     box for that reason, as solution_count is in sudoku.allium; solver.allium decides
     it, and is imported so that a board can ask."
   - Dependencies gains a second paragraph after line 89: "solver.allium, for the one
     solution of the givens: the search runs once, as the puzzle is set, and the check
     reads what it found. Nothing imports this module, so the import closes no cycle."
   - The `CheckCell` comment, the sentence "solver.allium says how it is found." becomes:
     "solver.allium finds it: it is what the one solved branch of the search of these
     givens holds, found once as the puzzle is set and not again for each check."
   - The `reopen` comment, the sentence "No move is re-made and no rule of this module
     runs for what the record holds." becomes: "The moves and checks the board then
     holds are the record's own, with the indices they had and the same moves undone:
     reopening adds none and discards none. How the board is made whole is not said
     here. A record that no board could have been written to is refused."
   - `ARecordIsEnoughOnItsOwn` becomes: "A record carries everything reopening needs.
     Nothing else is consulted: not wherever it was kept, not any other record and not a
     board already open, since there need be none. The solution is not in the record:
     reopening puts the record's givens to the solver, as setting a puzzle does, and the
     solver finds it again."
   - Excludes 66-68, the sentence beginning "Undo and redo write a cell's digit back"
     becomes: "Undo and redo put a cell's digit back to what a move the rules
     admitted left there; they restore states the rules reached and record no move
     of their own."
   - The `Undo` comment 420-422, the sentence beginning "The cell's digit is written
     back" becomes: "The cell's digit is put back to what a move the rules admitted
     left there. That state stood on an unsolved puzzle, so no guard of the rules
     can refuse it and it cannot solve the puzzle. Whether it is put back through
     the rules' own placing and erasing or written directly is not said here."

   Re-read the whole module afterwards for any other sentence the import makes untrue,
   and record each one found under Deviations.

3. **`sudoku.allium`.** In the `SetPuzzle` comment, after the sentence ending "its
   verdict is one exactly when this is 1.", add: "Whoever sets a puzzle may bring that
   verdict with it, and the one solution it rests on, so that the count is not taken
   twice; the solution is then kept with the puzzle, and nothing this module exposes is
   read from it." Nothing else in the module changes.

   **`solver.allium`.** In the comment above `surface SearchResult`, after "since it
   draws nothing.", add: "A caller that is handed a search only once it has concluded
   has no status to read: being handed it says concluded." Nothing else in the
   module changes.

4. **`rustqual.toml`.** In the board rule: `name` becomes
   `board_imports_sudoku_and_solver`; `"crate::solver"` leaves `forbid_path_prefix`;
   `reason` becomes `layering: board may import only sudoku, solver and random`. Update
   line 1 of `tests/fixtures/metrics-violation/src/board.rs` to the new name; its
   violation names `generation`, which stays forbidden. Run `just toml-check` and accept
   taplo's form for the shorter array. Then prove the change live, as T22 did, and quote
   each run:

   - `just metrics` is green on the tree.
   - With the fixture's `board.rs` changed to name `crate::solver` in place of
     `crate::generation`, `just metrics` fails, listing ten rules named and nine fired,
     the board's missing. That is the proof the solver is no longer forbidden there.
     Revert.

5. **The pages.** Each stays within what `docs/manifest.yml` says it owns.

   - `docs/explanation/layering.md`: the `board` row imports `sudoku`, `solver`; the
     `solver` row is imported by `board`, `generation`; the sentence at 30-31 says the
     board imports the rules and the solver, and nothing imports it. Add the reason in
     the same paragraph: the board is a leaf, so no cycle arises, and the claim under
     "Why the direction matters" is about the four models, which this does not touch.
     The Enforcement section's count of ten rules is unchanged.
   - `docs/explanation/specifications.md` 36 and `docs/how-to/work-with-the-specs.md` 21:
     the board imports the rules and the solver; every placement and erasure still goes
     to the rules' own moves, and the check reads the one solution the solver finds.
   - Search `docs/`, `AGENTS.md`, `.agents/` and `README.md` once more for any sentence
     that says what the board imports or that the solver is imported by `generation`
     alone. Edit a hit that has become false and record it under Deviations.
     `docs/README.md` is frozen: if it restates the import, hand that back.

6. **The decision record**, in the shape of 0012: frontmatter with
   `canonical_for: [decision_board_imports_solver]`; the title "Decision 0014: The board
   imports the solver, and well-posedness is a proof value". Use the next free number on
   the day; S04's hand-back reserves "0013 or later" for the release record, so check
   `docs/decisions/` first and renumber the file, the title and the index if 0014 is
   taken.

   - **Context:** the three modules to be built and their order; what the board's check
     needs; what the specifications said before this change.
   - **Decision:** the eight decisions under Context above, each in a sentence or two,
     with the reason the import is safe.
   - **Consequences, the ones that hurt included:** the layering table no longer shows
     the board standing on the rules alone; "only the solver calls the proof's
     constructor" is a rule review holds, not the compiler; every puzzle costs one search
     as it is set, and every reopening one more; a `Puzzle` cannot be made without the
     solver, so `sudoku`'s own tests build the proof by hand; nothing outside the crate
     can ask a `Puzzle` about its solution, so code on the two-step path reads the
     digits from the proof before it sets the puzzle, or does without them; the
     modules no longer say how undo puts a digit back, and a search's status is
     something no caller reads.
   - **What would reopen this:** a second solver, or a need to swap one in; a module
     that must import the board; a consumer that needs the solution through a `Puzzle`;
     a measured cost of solving at reopening that a stored solution would remove.
   - **Related pages:** Layering, Architecture, Specifications, decision 0013.

   Add the manifest entry, with audience `contributor`, `maintainer` and `agent`; add the
   row to `docs/decisions/README.md` and move its "next decision" sentence on by one.

7. `CHANGELOG.md`, under Unreleased: the board may import the solver, with the rule's new
   name; the comment sentences changed in `board.allium`, `sudoku.allium` and
   `solver.allium`, in one line; the count of decision records, which reads "Thirteen" today.

8. Run `just check-specs`, `just analyse-specs`, `just check-docs` and `just check`.
   Quote the closing lines of each. Fill in the hand-back notes, including the list for
   the game below, set `status: done` here and in `tickets/README.md`, commit on the
   ticket branch with short imperative subjects, and stop before pushing.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` each end
  `9 specifications, no diagnostics and no findings.`, and no `allium-ignore` directive
  exists under `docs/specs/`.
- `board.allium` has two `use` lines, `sudoku` then `solver`, and no sentence in it says
  the solver is not imported or that reopening asks only for a verdict.
- The diff of `docs/specs/` touches comment lines and the one `use` line and nothing
  else; the count of `open question` lines per module is unchanged.
- No sentence of `board.allium` says that undo or redo runs no rule, and the comment
  above `SearchResult` in `solver.allium` carries the sentence of step 3.
- `just metrics` is green; `rustqual.toml` holds ten rules, none named
  `board_imports_sudoku`; the sabotage run of step 4 failed as described and was
  reverted.
- The layering table, its prose, `specifications.md` and `work-with-the-specs.md` agree
  with `board.allium`'s two `use` lines, and the layering page states why the edge is
  safe.
- The decision record exists, is in the manifest and the index, and states all eight
  decisions; the index names the next free number.
- `just check` is green.

## Verification

```sh
just check-specs
just analyse-specs
just toml-check
just metrics
just check-docs
just check
```

Expected: two runs ending `9 specifications, no diagnostics and no findings.`; taplo
clean; `metrics` green with no finding on the tree and the probe satisfied; the documents
gate green with one more page than before; `All checks passed and the worktree is
unchanged.`

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

## Open points

- **The wording.** Steps 2 and 3 carry the ticket writer's proposals for nine comment edits.
  The executing agent applies them as written; the maintainer may reword any of them at
  review. Two say something the modules did not say before and deserve a second look:
  "A record that no board could have been written to is refused", and the proof sentence
  in `sudoku.allium`.
- **Three sentences loosen the text to fit the build.** The two on undo and redo and
  the one on a search's status are there because T27 and T26 would otherwise differ
  from what the modules say. The maintainer may prefer to keep the text. Then T27
  needs a crate-only write on `Puzzle` that undo and redo use and no move does, and
  T26 needs a status a caller can read, which would always say concluded: one
  accessor, and the cheaper of the two paths, set against a constant in the public
  surface. Both tickets stop and report if their sentence is not in the module. The
  undo sentences rest on placing and erasing doing nothing but set a cell's digit; if
  `PlaceDigit` or `EraseDigit` ever gains another effect, undo through them would run
  it, and the choice is looked at again.
- **The game restates these clauses.** Pawdoku holds its restated text equal to this
  repository's by test (`tickets/T06-spec-migration.md`, hand-back). The edits to
  `board.allium`, `sudoku.allium` and `solver.allium` are changes the game must take
  wherever it restates them; list them under
  Handed back with the old and new sentences. Editing the game is separately authorised
  and is not this ticket's.
- **The decision's number.** 0014 unless the release record has taken it by the day this
  runs; step 6 says what to do then.
- **A reference in an expression.** The import is named in no expression, as
  `generation.allium`'s are not. Replacing `solution_digit_at` with the solver's own
  entities would need a rule chain from `Solve` to the check; the ticket does not do it.
  If the maintainer wants it, it is a second spec change.
