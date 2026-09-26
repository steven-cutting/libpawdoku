---
id: T12
title: "Board migration: board.allium adapted, and the alignment with the game at add73be7"
status: done
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

Run on 2026-09-25 in the Supacode worktree for this ticket, on branch `T12-board-migration`
(see Deviations), from `main` at `cf1f702` after T06 merged, with the environment the
maintainer had installed: `.tools/bin/allium` 3.6.1 and cargo-hack. The maintainer
instructed that the claimed dependencies on T07, T08 and T11 be ignored and the migration
run now; the consequences are under Deviations.

**Step 1.** `git -C /Users/scutting/projects/pawdoku rev-parse --short=8 HEAD` printed
`add73be7`, `git diff --stat add73be7 HEAD -- docs/specs/` printed nothing, and the
library's `sudoku.allium` differed from G's only by T06's two comment hunks at 37-38 and
41-42. Neither module has moved since the ticket was written.

**Step 2.** The verbatim copy (`cmp` clean, 785 lines) was gated before any edit.
`just check-specs` and `just analyse-specs` each printed eight blocks of the shape

```json
{"command": "check", "diagnostics": [], "findings": [], "spec_file": "docs/specs/board.allium"}
```

(the binary pretty-prints each over six lines; `board.allium` sits fourth, between
`lapse` and `solver`) and closed:

```text
allium check: 8 specifications, no diagnostics and no findings.
allium analyse: 8 specifications, no diagnostics and no findings.
```

**Step 3.** Every Includes and Excludes bullet of `board.allium` was read in place and has
a row in the table below; the word search, with `\bstores?\b` added to T06's pattern, was
run on the verbatim copy and every hit is a row. Two hits the ticket's step 4 did not name
were found and adapted: line 653, "and the words are the game's", and line 773, "not the
store it came from" (Deviations).

**Step 4.** The maintainer chose reading (a) before the edits ran (Open points settled).
Seven comment edits were applied by exact replacement, each matching once; the diff
against G names lines 6, 80, 82-84, 119-120, 619-620, 653 and 773 and nothing else, and
the file keeps `-- allium: 3`, the `--` prefix, the bullet indent and the wrap width. The
gate on the edited module printed the same eight blocks and the same two closing lines as
step 2. The word search after the edits prints only the keep rows, 16, 55, 70, 500, 654,
702, 704, 755, 760 and 762, plus library line 83, the word "surfaces" in the adapted
81-84 sentence.

**Steps 5 and 6.** The four pages are still T00's stubs (Deviations), so each stub's one
paragraph was edited to say eight modules and name `board.allium`; frontmatter is
byte-identical. `just check-docs`:

```text
markdownlint.............................................................Passed
typos....................................................................Passed
lychee...................................................................Passed
bg-validate-docs
Validated 37 pages and 38 canonical topics.
```

**Step 7.** T06's hand-back items 1, 3, 4, 5, 6, 8 and 13 were re-read against G's HEAD,
`add73be7`, unmoved since T06 ran. Every item already names `board.allium` and says the
eight modules leave together, and every G line number was derived at `add73be7`: item 4's
table rows 20-27 with row 21 `board.allium` and item 5's paragraphs 40-46 match what this
ticket read (module table 15-30, `board.allium` paragraph 40-46, verbatim run through 75).
No edit was needed.

**Step 8.** The verification block, in order: `add73be7`; an empty specs diff; the diff
above; the two gate runs above; `-- allium: 3` eight times; `no waivers`; the eleven keep
lines; `no game words`; the counts `board 1, effort 7, lapse 1, human-solving 6, solver 0,
reach 3, technique 2, sudoku 0`; one "seven" hit, `docs/reference/quality-gates.md` 11,
"the seventeen gates", which does not count modules (`lapse.allium` 493, "seven
questions", is outside the `*.md` glob and does not count them either); `just check-docs`
as quoted; `just check` as quoted below; `git status --porcelain` empty after the commits.

```text
allium analyse: 8 specifications, no diagnostics and no findings.

==> just check-clean
bg-project-check clean "$1"
The worktree matches the check baseline.

All checks passed and the worktree is unchanged.
```

### The relevance review

Verdicts as in T06: **keep** (relevant to a library as written), **adapt** (relevant, but
the wording presumes the game), **drop** (not the library's; none). Lines are G's at
`add73be7`, which are the library's before the edits; after them 80 becomes 80-81 and
81-84 becomes 82-84, so every later line keeps its number.

`board.allium` (base: G `add73be7`)

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 6 | Scope "a player at a screen" | adapt | "a player": a consumer's surface is where the screen is |
| 21-24 | Includes "The board: one for every puzzle set..." | keep | The entity the library's board API is; a reopened board opens the same way |
| 25-29 | "Notes: each cell's marks..." | keep | Player state the rules leave out; two of the four moves |
| 30-34 | "Upkeep: when a digit is placed..." | keep | Deterministic; what undo must put back exactly |
| 35-37 | "The note beneath a digit..." | keep | Observable through `shows_note`; no UI |
| 38-43 | "Moves: every placement, erasure, written mark and struck mark..." | keep | The record undo and redo read; "solved is final" is the rules' |
| 44-46 | "Reading back: the board as it stood..." | keep | `digit_after` and `note_after`, engine output |
| 47-50 | "The check: the player asks whether one cell's digit is the solution's..." | keep | Kept as G has it on the maintainer's decision; the open question at 785 is carried |
| 51-54 | "The record: a board written down whole..." | keep | Already says "nothing of the form a record takes or where one is kept", which is reading (a) |
| 55-58 | "Two boundaries: Playing ... and Reopening ..." | keep | The library's API shape; "a play surface" is a consumer's and Allium's word |
| 61-68 | Excludes "The rules: what a unit, a peer, a conflict and solved are..." | keep | `sudoku.allium`'s; the one board-own guard (a placement changes the digit) is stated |
| 69-72 | "Where a puzzle comes from..." | keep | `PuzzleSetting` is the way in; no setting surface here |
| 73-75 | "Finding the solution. solution_digit_at below is a black box..." | keep | `solver.allium` decides it and is not imported, as `solution_count` in `sudoku.allium` (Open points) |
| 76-78 | "Hints, which are reach.allium's ...; timers; and any count read off the checks" | keep | `reach`'s, no clock, the open question |
| 79-80 | "How a record is kept ... The record is what a store is handed; the store is a port's." | adapt | Reading (a): "The record is what a caller is handed; where it is kept is the caller's." |
| 81-84 | "Anything drawn ... pawdoku.allium's Play surface and the modules beneath it decide that, and the platform's marks are theirs to translate." | adapt | "A consumer's surfaces decide that; nothing here draws.", as T06 reworded `sudoku.allium` 36-38 |
| 16, 500, 654, 702, 704, 755, 760, 762 | `surface`, `Playing`, "In play" | keep | Allium's word and the surface's own name; "in play" is the state of a puzzle |
| 55 | "what a play surface can see of a board" | keep | A consumer's surface; the boundary is what the library states |
| 70 | "no setting surface of its own" | keep | Says the module has none |
| 119-120 | `external entity Record` comment, "a store behind a port decides the rest" | adapt | "the caller it is handed to decides the rest, where it is kept included" |
| 619-620 | `contract Recording` comment, "a record is whatever a store is handed, and the store is a port's" | adapt | "a record is whatever a caller is handed, and where it is kept is the caller's" |
| 653 | `surface Playing` comment, "and the words are the game's" | adapt | "and the words are a consumer's"; a hit the ticket's step 4 did not list |
| 773 | `ARecordIsEnoughOnItsOwn`, "not the store it came from" | adapt | "not wherever it was kept"; under reading (a) nothing presumes a store |
| 476, 488 | `solution_digit_at`, a black box | keep | Carried; whether the Rust board reaches the solver by import or only in the implementation is the `rust-change`'s (Open points) |
| 785 | Open question | keep | Carried, unchanged |

### Deviations, and why

- **The branch is `T12-board-migration`, not `ticket/t12-board-migration`.** It keeps the
  Supacode name, as T01, T04 and T06 did.
- **The dependencies on T07, T08 and T11 were not waited for.** The maintainer's
  instruction on 2026-09-25: "ignore the claimed dependencies on other tickets I want to
  go ahead and port/migrate the board.allium spec now and update the others as well."
  `tickets/README.md`'s graph still shows T12 after them; the row's status is what moved.
- **The four pages were edited as stubs, not as G's pages.** Step 5 assumes T07's and
  T08's content (a module table, an import graph, a terminology table); none exists yet.
  On the maintainer's choice each stub's paragraph now says eight modules and names
  `board.allium`, and the briefs in `tickets/T07-handbook-a.md` and
  `tickets/T08-handbook-b.md` were amended so the full pages land with the board: the
  module table and the "Diagnostics and waivers" section based on G `add73be7` (the
  `allium.field.unused` projection paragraph included), the seven terminology rows and two
  edits with the two rewordings step 5 names, "Eight are the library's" with the
  `board.allium` paragraph, and `board` in the import graph. Those two tickets are other
  lanes' files (CONVENTIONS.md §11); the same instruction is the authority.
- **Two comment edits beyond the ticket's five,** at 653 and 773, both word-search hits
  the ticket's step 4 did not account for. Line 653 named the game; line 773 named a
  store inside a guarantee, which reading (a) removes.
- **The "seven" grep.** With `--include='*.md'` it hits only `quality-gates.md` 11
  ("seventeen gates"); the ticket's expected `lapse.allium` 493 is not a Markdown file.
  Neither counts modules.
- **`git diff --stat main` lists nine files,** not the ticket's eight: the module, the
  four stubs, the two amended briefs, this ticket and `tickets/README.md`, which gains the
  status flip; `tickets/T06-spec-migration.md` needed no edit (step 7).

### Handed back to pawdoku

Nothing new. T06's list of thirteen items, re-read on 2026-09-25 against `add73be7`,
already removes the eight modules together and its line numbers hold. Two notes for the
day G performs it: `board.allium` in this repository differs from G's by the seven
comment edits the review names, so the restated text G holds equal by test is this
repository's; and item 13, the prototype under `prototypes/board/`, remains G's decision.

### Open points settled

Each answered by the maintainer on 2026-09-25, in this session.

- **A record and a reopening in a library with no persistence.** Reading (a), as
  recommended: the record is a value the engine writes and reopens, the caller keeps it
  wherever it likes, and no second effect trait is added. `contract Recording` and
  `surface Reopening` stay as G has them; the three "a port's" sentences and the
  `store` at 773 are reworded to say so. `AGENTS.md` line 13 and invariant 2 stay true.
- **`solution_digit_at`.** Carried as a black box; the `rust-change` that builds the
  board decides how it reaches the solver.
- **The check itself.** Kept as G has it; the open question at 785 is carried.
- **G's prototype.** Not this ticket's; T06's hand-back item 13 asks G. The maintainer
  named it as background for the Rust board, which is a later ticket, not yet written.
- **`docs/README.md` is frozen.** Untouched; it names seven modules at lines 18-24, and
  adding `board.allium` to the paragraph and the module list is a `main` follow-up.
- **`AGENTS.md` line 13.** Unchanged under reading (a).

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
