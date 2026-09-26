---
id: T12
title: "Board migration: board.allium adapted, and the alignment with the game at add73be7"
status: open
depends_on: [T06, T07, T08, T11]
parallel_with: []
branch: ticket/t12-board-migration
estimated_size: M
---

# T12: Board migration: board.allium adapted, and the alignment with the game at add73be7

## Context

T06 migrated the seven engine modules from the game (G) at the commit CONVENTIONS.md §0
pins, `78d03cdf`. By the day T06 ran, 2026-09-25, G had moved to `add73be7` and added an
eighth engine-level module, `docs/specs/board.allium` (commits `3079b31` to `add73be7`,
merged as G's pull request #4): 785 lines, `-- allium: 3`, `use "./sudoku.allium" as
sudoku` at line 104 and no other import; `enum MoveKind` (112), `entity Board` (127),
`BoardCell` (149), `Move` (174), `Check` (212), a `config` block (239), `contract
Recording` (621), `surface Playing` (654), `surface Reopening` (762), and one open question
at 785. Its Scope, in its own words: "What stands between the rules of Sudoku and a player
at a screen: one puzzle as it is being played" — a note in every cell with upkeep, the four
moves with exact undo and redo, the board as it stood after any move, the check of one
cell against the solution, and a record a board is written to and had again from. It draws
nothing, and every placement and erasure goes through `sudoku.allium`'s own `PlaceDigit`
and `EraseDigit`.

G also added one sentence to `sudoku.allium` (its lines 34-35): "board.allium keeps the
notes, the moves, undo and the check for a puzzle in play." The maintainer chose to take
the library's `sudoku.allium` from `add73be7` and keep that sentence verbatim, so today it
names a module this repository does not have. This ticket is the maintainer's instruction
from the same day, verbatim: "create a follow up ticket to port/migrate board.allium here
and make any other alignment updates as well."

The game-shaped sentences in `board.allium` at `add73be7`, found by T06's word search
over it (the relevance review itself is this ticket's): Scope line 6, "a player at a
screen"; Excludes 79-80, "How a record is kept, and whether one is kept at all between
visits. The record is what a store is handed; the store is a port's."; Excludes 81-84,
"Anything drawn ... pawdoku.allium's Play surface and the modules beneath it decide that,
and the platform's marks are theirs to translate."; line 120, "behind a port decides the
rest"; line 620, "is a port's". The library has no storage port: `AGENTS.md` says the
engine has "no UI, no persistence and no server" and that randomness is its only effect.

What else G changed after the pin and this repository should follow, from `git -C
/Users/scutting/projects/pawdoku diff --stat 78d03cdf add73be7`: `docs/how-to/work-with-the-specs.md`
(the `board.allium` row at 21, and a paragraph at 120-130 on how `allium.field.unused`
misses a use inside a projection's `where` predicate and how an invariant answers it
better than a waiver), `docs/explanation/specifications.md` (the `board.allium` paragraph
at 40-46), `docs/project/terminology.md` (seven board rows at 36-42 and edits to the Mark
row at 55 and the Note, entry row at 78), and `docs/README.md` (names `board.allium` at
17-18). G's decision 0011 (executable prototypes under `prototypes/board/`), its
`pyproject.toml` `proto` group and its `quality-gates.md` changes are the game's own and
are not followed here; T06's hand-back item 13 asks G what happens to the prototype when
the modules leave.

Sources, read-only: G at `add73be7` (`git -C /Users/scutting/projects/pawdoku show
add73be7:<path>` if the clone has moved): `docs/specs/board.allium`, `docs/specs/sudoku.allium`,
the four pages above; this repository's `tickets/T06-spec-migration.md` (the review table's
shape, the hand-back list) and its `docs/specs/sudoku.allium`.

## Goal

`docs/specs/board.allium` in this repository, adapted by the same clause-by-clause
relevance review T06 ran, with every Includes and Excludes bullet judged and recorded in
this ticket; the gate green with `8 specifications, no diagnostics and no findings.` from
both `just check-specs` and `just analyse-specs`; the open question at 785 carried; the
handbook pages that list or count the modules saying eight where they said seven; T06's
hand-back list amended so G removes eight modules at once; and the frozen-file follow-up
named.

## Non-goals

- No edit to any file in G. The hand-back stays a list.
- No implementation. A board in Rust is a later `rust-change`; S04 decides how a consumer
  reaches it.
- No open question answered, added or removed; no open point below settled by the
  executing agent. The maintainer answers the first open point before step 4 runs.
- No change to `sudoku.allium` or any other module beyond what the review of `board.allium`
  forces (none is expected).
- No change to a frozen file (CONVENTIONS.md §11). `docs/README.md` line 21 names and
  links the seven modules; adding `board.allium` there is a `main` follow-up.
- No waiver. G's `work-with-the-specs.md` 120-130 says how `board.allium` answered the one
  diagnostic it met with an invariant; the copy is green in G on the same binary.
- No rewrite of T06's two pages, and no reflow of `board.allium` beyond the sentences the
  review names.

## Files touched

| Path | Change |
| --- | --- |
| `docs/specs/board.allium` | New: G `add73be7` text, adapted per the review (step 4) |
| `docs/how-to/work-with-the-specs.md` | Module table gains the `board.allium` row; the `allium.field.unused` projection paragraph, reworded for this repository, if T07 took the waiver terms at the pin |
| `docs/explanation/specifications.md` | "Seven are the library's" becomes eight; a `board.allium` paragraph after `sudoku.allium`'s |
| `docs/explanation/layering.md` | The import graph gains `sudoku ── board` |
| `docs/project/terminology.md` | The seven board rows and the Mark and Note, entry edits, reworded for a library (no "rendering", no "storage port") |
| `tickets/T06-spec-migration.md` | Hand-back list items 1, 3, 4, 5, 6, 8 and 13 re-read against G's HEAD on the day; corrected if G moved again |
| `tickets/T12-board-migration.md` | `status:`, the review table, hand-back notes |

## Steps

1. Create the worktree on `ticket/t12-board-migration` from `main` after T06, T07, T08 and
   T11 have merged (README.md "How to pick up a ticket"). Confirm `docs/specs/sudoku.allium`
   is identical to G's `add73be7` copy apart from T06's two comment edits, and that
   `board.allium` is unchanged in G since `add73be7`:

   ```sh
   git -C /Users/scutting/projects/pawdoku diff --stat add73be7 HEAD -- docs/specs/
   ```

   If `board.allium` or `sudoku.allium` moved, read the diff and carry the change into the
   review; do not silently take the newer text.

2. Copy the module verbatim and gate it before any edit:

   ```sh
   git -C /Users/scutting/projects/pawdoku show add73be7:docs/specs/board.allium >| docs/specs/board.allium
   just check-specs
   just analyse-specs
   ```

   Both must end `8 specifications, no diagnostics and no findings.` Quote the closing
   lines. A diagnostic here is a toolchain fault, not the module's, since G's gate is green
   on the same binary.

3. The relevance review. Tabulate every Includes bullet (lines 20-58) and every Excludes
   bullet (60-84) of `board.allium` with a verdict, keep, adapt or drop, and a reason, in a
   table under "The relevance review" below shaped as T06's. Then run T06's word search
   with two words added and account for every hit:

   ```sh
   grep -n -E "pawdoku|\bgames?\b|\bsurfaces?\b|\bports?\b|browser|device|screen|platform|src/|tests/|\bPlay\b|storage|\bstores?\b|Svelte" docs/specs/board.allium
   ```

4. The edits, all comments, once the maintainer has answered the first open point. The
   ticket writer's proposals, to be confirmed or replaced by that answer:

   - Scope line 6: "a player at a screen" becomes "a player".
   - Excludes 79-80: "The record is what a store is handed; the store is a port's."
     becomes "The record is what a caller is handed; where it is kept is the caller's."
   - Excludes 81-84: "pawdoku.allium's Play surface and the modules beneath it decide
     that, and the platform's marks are theirs to translate." becomes "a consumer's
     surfaces decide that; nothing here draws.", as T06 reworded `sudoku.allium` 36-38.
   - Lines 120 and 620, the "port" sentences: the same wording as 79-80.

   Keep `-- allium: 3`, the `--` prefix, the three-space bullet indent and the wrap width.
   A comment edit cannot produce a diagnostic; one after this step means an edit strayed.

5. The pages, each read from G at `add73be7` and reworded for a library, and each landing
   on the page `docs/manifest.yml` already registers:

   - `docs/how-to/work-with-the-specs.md`: the `board.allium` row, after `sudoku.allium`'s,
     from G row 21; the `allium.field.unused` paragraph from G 120-130 if the page carries
     the waiver terms at the pin's wording.
   - `docs/explanation/specifications.md`: "Seven are the library's" becomes eight; the
     `board.allium` paragraph from G 40-46, minus "as `human-solving.allium` does" only if
     the page does not already describe that contract.
   - `docs/explanation/layering.md`: the import graph gains `board`, importing `sudoku`
     alone and imported by nothing.
   - `docs/project/terminology.md`: the seven rows from G 36-42 and the two edits at G 55
     and 78, with "What a record is made of is a store's, behind the storage port" reworded
     to match step 4's answer, and "what draws one is a rendering" to "what draws one is a
     consumer's".

6. The gate on the edited module, `just check-specs` and `just analyse-specs` again, the
   eight blocks quoted; then `just check-docs`.

7. Amend T06's hand-back list against G's HEAD on the day, so that it still says eight
   modules leave together and its line numbers still hold.

8. `just check` from the top; quote its closing lines. Fill in the hand-back notes, set
   `status: done`, commit on the ticket branch with short imperative subjects, and stop
   before pushing.

## Acceptance criteria

- `just check-specs` and `just analyse-specs` each print eight blocks with empty
  `diagnostics` and `findings` arrays and end `8 specifications, no diagnostics and no
  findings.`; no `allium-ignore` directive exists in `docs/specs/`.
- `head -1 docs/specs/*.allium` prints `-- allium: 3` eight times.
- `grep -n -i "pawdoku\.allium\|this game\|randomness port\|storage port\|src/lib\|platformSpecs" docs/specs/*.allium docs/explanation/*.md docs/how-to/*.md docs/project/*.md`
  prints nothing, and every hit of step 3's word search is a row in the review table.
- `grep -c "^open question" docs/specs/board.allium` prints 1; the nineteen T06 counted
  are unchanged.
- No page under `docs/` counts the modules as seven:
  `grep -rn -i "seven" docs --include=*.md` hits nothing that counts them.
- `just check` is green; `git status --porcelain` is empty after it.
- The review table is complete and T06's hand-back list is amended.

## Verification

```sh
git -C /Users/scutting/projects/pawdoku rev-parse --short=8 HEAD
git -C /Users/scutting/projects/pawdoku diff --stat add73be7 HEAD -- docs/specs/
diff <(git -C /Users/scutting/projects/pawdoku show add73be7:docs/specs/board.allium) docs/specs/board.allium
just check-specs
just analyse-specs
head -1 docs/specs/*.allium
grep -rn "allium-ignore" docs/specs/ || echo "no waivers"
grep -n -E "pawdoku|\bgames?\b|\bsurfaces?\b|\bports?\b|browser|device|screen|platform|src/|tests/|\bPlay\b|storage|\bstores?\b|Svelte" docs/specs/board.allium
grep -c "^open question" docs/specs/*.allium
grep -rn -i "seven" docs --include=*.md
just check-docs
just check
git status --porcelain
```

Expected: `add73be7` or a later commit with the specs diff read; a diff naming only the
comment lines step 4 lists; two runs ending `8 specifications, no diagnostics and no
findings.`; eight `-- allium: 3`; `no waivers`; every word-search hit in the table; the
counts `board 1, effort 7, human-solving 6, lapse 1, reach 3, solver 0, sudoku 0,
technique 2`; no "seven" that counts modules; both gates green; nothing from `git status`.

## Hand-back notes

### What was verified, and how

### The relevance review

### Deviations, and why

### Handed back to pawdoku

### Open points settled

## Open points

- **A record and a reopening in a library with no persistence.** `contract Recording`
  (621), `surface Reopening` (762) and the three "a port's" sentences (80, 120, 620) name a
  store behind a port that this library does not have. Three readings: (a) keep both: the
  record is a value the engine produces and accepts, the caller keeps it wherever it likes,
  and no second effect trait is added, so "the store is a port's" becomes "where a record
  is kept is the caller's"; (b) a second effect trait beside randomness, which changes
  `AGENTS.md` invariant 2 ("Randomness is the only effect the engine has") and needs a
  decision record; (c) leave `Reopening` and `Recording` out and hand the record's shape to
  S04. The ticket writer recommends (a); the maintainer decides before step 4.
- **`solution_digit_at`.** A black box at lines 476 and 488 that the check reads; in G it
  is the solver's to answer, as `solution_count` is in `sudoku.allium`. Whether the Rust
  board reaches the solver through an import in the spec or only in the implementation is
  the `rust-change` that builds it, not this ticket's.
- **The check itself.** Asking whether one cell's digit is the solution's, and the open
  question at 785 about what is counted from the checks. Keep as G has it and carry the
  question; the maintainer may instead call it a consumer's feature over the solver's
  result, in which case it is a drop with a stated reason.
- **G's prototype.** `prototypes/board/` (G decision 0011) models `board.allium` and
  `sudoku.allium` in Python with snapshot tests. Whether a model of that kind is wanted here
  is not this ticket's; T06's hand-back item 13 asks G what happens to it when the modules
  leave.
- **`docs/README.md` is frozen** and names seven modules at line 21; a `main` follow-up
  adds `board.allium` to the paragraph and the module list.
- **`AGENTS.md` line 13** says the engine has "no persistence". Under reading (a) above
  the sentence stays true; under (b) it and invariant 2 change, which is a T05 follow-up
  and a decision record, not this ticket's.
