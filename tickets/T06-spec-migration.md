---
id: T06
title: "Specification migration: the seven engine modules, adapted, and the Allium gate"
status: open
depends_on: [T00]
parallel_with: [T01, T02, T03, T04, T05, T07, T08, T09]
branch: ticket/t06-spec-migration
estimated_size: L
---

# T06: Specification migration: the seven engine modules, adapted, and the Allium gate

## Context

T00 step 11 copied seven Allium modules byte-for-byte from the game (G) into
`docs/specs/`, and step 10 copied the two explanation pages they cite as Sources into
`docs/explanation/`. This ticket makes them the library's: every clause is judged for
relevance, the few game-shaped sentences are reworded, and the Allium gate is proved
green on the result. The maintainer's instruction, verbatim: the specs move here "but we
will need to be sure and adapt the spec to this project to make sure that they do not
include anything irrelevant." So this is a clause-by-clause relevance review, not a
find-and-replace; CONVENTIONS.md §9 lists the adaptations already found and says they are
not the whole job.

The seven modules, each in one line from its own Scope, at G `78d03cdf`:

- `sudoku.allium` (351 lines): classic Sudoku and nothing else; what the grid is, what a
  setter may pose, what a player may do to it and when it is solved, stated without a
  screen.
- `solver.allium` (652 lines): what decides whether a set of givens has no solution, one
  or many, and what those solutions are; the search `sudoku.allium` leaves a black box.
- `technique.allium` (1162 lines): the named techniques a person solves Sudoku by, stated
  once, and the profile of a player; the ground four models of a player stand on.
- `reach.allium` (425 lines): the first model, in which a limit hides things; how far a
  player gets, by which techniques, and what they would find next, which is a hint.
- `effort.allium` (316 lines): the second model, in which a limit makes things dear; a
  puzzle priced end to end for a player, coarse and with its figures open.
- `lapse.allium` (531 lines): the third model, in which a limit leads to error; marks that
  fall behind, guesses, and repairs, without chance.
- `human-solving.allium` (1280 lines): the fourth model, in which a limit changes what is
  noticed, kept in mind and got wrong, by chance; seeded attempts and their assessment.

Their import graph, read from the `use` lines (`solver.allium` 61, `technique.allium`
86, `reach.allium` 57-58, `effort.allium` 57-58, `lapse.allium` 78-79,
`human-solving.allium` 72-73):

```text
sudoku ──┬── solver
         ├── technique ──┬── reach
         │               ├── effort
         │               ├── lapse
         │               └── human-solving
         └───────────────┘   (each of the four also imports sudoku directly)
```

`sudoku.allium` imports nothing; `solver` and `technique` import it; the four models
import `sudoku` and `technique` and never each other, nor `solver`. G's root module
`docs/specs/pawdoku.allium` (88 lines) stays in G. It imports nothing (`Dependencies:
None`, lines 27-29) and names one of the seven, at lines 9-11: "The rules of the game are
sudoku.allium's, a module beside this one that imports nothing. How those rules are drawn
is still to be specified, in modules that import both." It is also the only module that
names `tests/platformSpecs.test.ts` (lines 16, 49, 55, 61). Allium has no cross-repository
import (CONVENTIONS.md §1 fact 6): G will restate the clauses it needs and hold them equal
to this repository's text by test, as it holds the platform's figures today.

The gate is `just check-specs` and `just analyse-specs` (CONVENTIONS.md §4), which run
`bg-run-allium check` and `bg-run-allium analyse` (decision 0004). Read B's
`src/biscuit_games_tooling/run_allium.py` at `6c5c07f6`: it runs the pinned binary over
`docs/specs/`, prints the binary's output whole, parses it as back-to-back JSON objects
(one per module, each carrying `spec_file`, `diagnostics` and `findings`), refuses a block
missing either array, refuses a report whose file set differs from what `docs/specs/`
holds, and passes only when every array is empty, whatever the exit code (lines 130-193).
The binary is allium-tools `3.6.1`, pinned with a SHA-256 per target in B's
`install_allium.py` line 34 and lines 51-56, installed by `just install-allium` into the
gitignored `.tools/bin/`. Every module's header is `-- allium: 3`, the language version,
which is a different number from the tool's.

Sources, read-only, at the commits CONVENTIONS.md §0 pins (`git -C <clone> show
<commit>:<path>` if a clone has moved): G `docs/specs/*.allium` (all eight),
`docs/explanation/solving-sudoku.md`, `docs/explanation/human-solving.md`,
`docs/how-to/work-with-the-specs.md` (waiver terms, lines 77-136),
`docs/explanation/specifications.md`, `AGENTS.md`, `docs/manifest.yml`, `docs/README.md`,
`docs/project/terminology.md`, `.agents/skills/spec-change/SKILL.md`; B `run_allium.py`
and `install_allium.py`. Read every module in full before judging a clause.

## Goal

The seven modules and the two pages adapted so that nothing game-only or irrelevant to a
library remains; the relevance review recorded in this ticket with every Includes and
Excludes clause judged; the gate green with empty `diagnostics` and `findings` for every
module, on allium 3.6.1; every open question carried; and the hand-back to G written out
as concrete edits, not performed.

## Non-goals

- No edit to any file in G, T, B, P or H. The hand-back is a list.
- No new module, no root module for the library (Open points), no restatement test:
  how G consumes the engine is S04's.
- No implementation. `crates/pawdoku/src/random.rs` is T02's; this ticket fixes the
  wording the code must satisfy and nothing else.
- No open question answered, added or removed. Each is carried as it stands.
- No waiver, unless the pinned checker is verifiably wrong on the terms G's
  `work-with-the-specs.md` lines 94-125 state, and none is expected: the copies are
  green in G.
- No change to a frozen file (CONVENTIONS.md §11): `docs/manifest.yml`, `docs/README.md`,
  the `Justfile` and the hook configs. A change needed there is a `main` follow-up.
- No reflow, reformatting or rewording beyond the sentences the review names. A diff a
  reviewer cannot read against G is a failed ticket.

## Files touched

| Path | Change |
| --- | --- |
| `docs/specs/sudoku.allium` | Lines 36-37 and 40-41 reworded (step 5) |
| `docs/specs/solver.allium` | Verbatim; reviewed |
| `docs/specs/technique.allium` | Verbatim; reviewed |
| `docs/specs/reach.allium` | Verbatim; reviewed |
| `docs/specs/effort.allium` | Verbatim; reviewed; seven open questions carried |
| `docs/specs/lapse.allium` | Lines 45-46 reworded (step 5) |
| `docs/specs/human-solving.allium` | Lines 39-40, 476-478 and 1096-1099 reworded (step 5) |
| `docs/explanation/solving-sudoku.md` | Verbatim; links checked |
| `docs/explanation/human-solving.md` | Lines 15-16 and 255-256 reworded (step 6) |
| `tickets/T06-spec-migration.md` | `status:`, the review table, hand-back notes |

## Steps

1. Create the worktree on `ticket/t06-spec-migration` from `main` (README.md "How to pick
   up a ticket"). Confirm the starting point is T00's:

   ```sh
   for m in sudoku solver technique reach effort lapse human-solving; do cmp "docs/specs/$m.allium" <(git -C /Users/scutting/projects/pawdoku show "78d03cdf:docs/specs/$m.allium") && echo "$m identical"; done
   for p in solving-sudoku human-solving; do cmp "docs/explanation/$p.md" <(git -C /Users/scutting/projects/pawdoku show "78d03cdf:docs/explanation/$p.md") && echo "$p identical"; done
   ```

   Nine `identical` lines. Anything else means T00 did not merge as written: stop.

2. The gate on the verbatim copies. `just install-allium` (network; once per worktree),
   then `just check-specs` and `just analyse-specs`. Both must end
   `allium <command>: 7 specifications, no diagnostics and no findings.` Quote the two
   closing lines in the hand-back notes. A diagnostic here is a fault in the toolchain,
   not in the modules, since G's gate is green at `78d03cdf` on the same binary.

3. The relevance review. Re-check the table under "The relevance review" below, which
   is pre-filled with the ticket writer's verdicts from reading the modules at
   `78d03cdf`. For every row: read the clause in place, confirm or change the verdict,
   and give a reason for every change. Then run the word search and account for every
   hit:

   ```sh
   grep -n -E "pawdoku|\bgames?\b|\bsurfaces?\b|\bports?\b|browser|device|screen|platform|src/|tests/|\bPlay\b|storage|Svelte" docs/specs/*.allium
   ```

   The hits at `78d03cdf`, each with the verdict the table gives it. Every `surface`
   declaration line is a keep (`sudoku` 315, 329; `solver` 607, 640; `technique` 846;
   `reach` 344, 376, 398; `effort` 261, 281; `lapse` 428, 461; `human-solving` 792,
   1065, 1150, 1192, 1203): the word is Allium's. The rest: `sudoku.allium` 7-8 ("the
   game as it is played on paper, stated without a screen": Sudoku the game, keep), 19
   ("Play:", the act of playing, keep), 29 ("a different game": a variant, keep), 35-37
   and 40 (adapt), 196 (the `-- Play ---` section, keep); `reach.allium` 44 ("anything a
   surface draws" is excluded already, keep) and 375 ("the surface a puzzle is played
   on": a consumer's, keep; Open points); `lapse.allium` 45-46 (adapt);
   `human-solving.allium` 30 ("Runtime/UI implementation, persistence" excluded, keep),
   31 ("in-progress games": puzzles in play, keep), 39-40, 476-477 and 1096-1099 (adapt),
   1109-1110 ("platform rounding", "platform iteration order": the computing platform,
   keep). A hit the table does not account for is a new row.

4. The randomness boundary, the wording CONVENTIONS.md §9 fixes and every edit below
   uses: **the randomness boundary**, a trait the library defines (`random.rs`): a stream
   of draws in `[0, 1)` begun from a seed, indexed from zero, the same for the same seed
   and `random_version`. The caller chooses the implementation; the library ships one
   named by `random_version` so that `ExactReplay` holds across callers; a test supplies
   draws through the fake. The contract the code must satisfy is already in
   `human-solving.allium` and is not changed by this ticket, only re-homed:

   - `random_version: String` on `AttemptRequest` (line 195) and `PolicyInput` (line
     208), defaulted at line 478 to `"seeded-stream-1"`.
   - `value Draw { index: Integer, purpose: String, value: Decimal }` (lines 308-312),
     and `draws: List<Draw>` on `Plan`, `Effect` and `Microstep` (317, 325, 397).
   - `ExactReplay` (lines 1091-1117): "Identical givens, resolved
     profile/environment/costs/limits, seed, model_version, catalogue_version and
     random_version yield identical action order, draws, state changes, effort and
     outcome"; "a stream of u in [0,1), begun from the seed, indexed from zero, the same
     for the same seed and random_version"; "A Bernoulli outcome is u<p; a uniform n-way
     index is floor(u*n); weighted choice is the first cumulative interval containing
     u*sum(weights), intervals closed left/open right. Draw even for p=0 or p=1."; "No
     wall clock, platform iteration order or hidden random source may affect a run."

   T02 implements the trait in `crates/pawdoku/src/random.rs` and tests it in
   `crates/pawdoku/tests/random.rs`; the clauses above are what those tests derive from.
   Nothing in this ticket may weaken them.

5. The module edits, exactly these and nothing else. Line numbers are G's at `78d03cdf`
   and were verified by reading; where a rewrap moves a following line, the rewrapped
   lines are named. Keep every header `-- allium: 3`, the `--` comment prefix, the
   three-space indent of header bullets and the wrap width the surrounding text uses.

   `sudoku.allium` lines 35-37, before:

   ```text
   --   - Whether a surface refuses, flags or merely allows a conflicting
   --     placement. The rules allow it; pawdoku.allium's Play surface and the
   --     modules beneath it decide what a player is shown.
   ```

   after:

   ```text
   --   - Whether a surface refuses, flags or merely allows a conflicting
   --     placement. The rules allow it; a consumer's surfaces decide what a
   --     player is shown; nothing here draws.
   ```

   `sudoku.allium` lines 39-41, before:

   ```text
   -- Dependencies
   --   None. The rules need no figure pawdoku.allium states, so this module does
   --   not import it; a module that draws the rules imports both.
   ```

   after:

   ```text
   -- Dependencies
   --   None. No figure a consumer's root module states is read here; a consumer
   --   that draws the rules imports this module beside its own.
   ```

   `lapse.allium` lines 44-48 (the edit is at 45-46; 46-48 rewrap), before:

   ```text
   --   - Slips: a digit misread, a candidate struck by mistake, a mark never
   --     written. They need chance, which this game reaches through its
   --     randomness port, and they are human-solving.allium's, which draws them
   --     by seed. Nothing here is drawn at random, and this model stays the
   --     account of what error there is without chance.
   ```

   after:

   ```text
   --   - Slips: a digit misread, a candidate struck by mistake, a mark never
   --     written. They need chance, which this library reaches through its
   --     randomness boundary, and they are human-solving.allium's, which draws
   --     them by seed. Nothing here is drawn at random, and this model stays
   --     the account of what error there is without chance.
   ```

   `human-solving.allium` lines 39-40 (becomes three lines), before:

   ```text
   --   - The random generator. Draws come from this game's randomness port;
   --     what this module states is how a draw becomes a decision.
   ```

   after:

   ```text
   --   - The random generator. Draws come from the library's randomness
   --     boundary: a caller supplies the stream; what this module states is
   --     how a draw becomes a decision.
   ```

   `human-solving.allium` lines 476-478 (the comment is 476-477 and becomes three lines;
   478 is the field and is unchanged), before:

   ```text
       -- Names the randomness port's seeded stream. The generator is the
       -- port's; a different generator is a different name.
       random_version: String = "seeded-stream-1"
   ```

   after:

   ```text
       -- Names the seeded stream the randomness boundary supplies. The
       -- generator is the library's, named here so a different one is a
       -- different name.
       random_version: String = "seeded-stream-1"
   ```

   `human-solving.allium` lines 1096-1099 (shifted by the two insertions above; find
   them inside `ExactReplay`), before:

   ```text
           -- Draws are the randomness port's: a stream of u in [0,1), begun
           -- from the seed, indexed from zero, the same for the same seed and
           -- random_version. Which generator is the port's and not stated
           -- here; a test supplies the draws through the port's fake.
   ```

   after:

   ```text
           -- Draws are the randomness boundary's: a stream of u in [0,1),
           -- begun from the seed, indexed from zero, the same for the same seed
           -- and random_version. Which generator is the library's to name, not
           -- stated here; a test supplies the draws through the boundary's fake.
   ```

   `solver.allium`, `technique.allium`, `reach.allium` and `effort.allium` are not
   edited. Their Source paths (`solver` 11, `technique` 20, `reach` 17, `effort` 21,
   `lapse` 20, `human-solving` 15) name `docs/explanation/solving-sudoku.md` or
   `explanation/human-solving.md`, which are valid here because both pages moved.
   `effort.allium`'s seven open questions (lines 304-316, including 312, "An unpriced
   run") and `human-solving.allium`'s six (1270-1280, the last at 1280 asking whether
   `PlayerProfile`, `Environment` and the projection should move to a small module) are
   carried as they stand, as are `technique.allium`'s two (1160, 1162), `reach.allium`'s
   three (421-425) and `lapse.allium`'s one (531). `sudoku.allium` and `solver.allium`
   carry none.

6. The two pages. Frontmatter is unchanged on both (title, kind, audience,
   `canonical_for`, `requires: []`), because `docs/manifest.yml` is frozen and T00 wrote
   the entries from G's. `docs/explanation/solving-sudoku.md` is verbatim. In
   `docs/explanation/human-solving.md`, lines 15-16, before:

   ```text
   These specifications define future simulator behaviour; no executable simulator or
   game surface exists yet.
   ```

   after:

   ```text
   These specifications define future simulator behaviour. No executable simulator
   exists yet; a surface is the game's.
   ```

   Lines 255-256, before:

   ```text
   Draws come from the game's randomness port, as every side effect does: the port
   owns the generator and its name, and a test supplies draws through the port's fake.
   ```

   after:

   ```text
   Draws come from the library's randomness boundary, the one effect the engine has: a
   caller supplies the stream, the library names the generator it ships through
   `random_version`, and a test supplies draws through the boundary's fake.
   ```

   Links, read from both pages at `78d03cdf`, all of which resolve inside this handbook
   (T00 step 9 registers every target): `solving-sudoku.md` links `specifications.md`
   (462) and the seven modules under `../specs/` (463-465); `human-solving.md` links the
   seven modules (11, 19, 27-30, 268), `solving-sudoku.md` (18, 356),
   `specifications.md` (355), `../project/terminology.md` (357) and
   `../how-to/work-with-the-specs.md` (358). Neither page links a page CONVENTIONS.md §6
   drops (`project/platform.md`, `explanation/accessibility.md`, the three how-to pages).
   The external links (HoDoKu at 272-281, the seven citations at 96-102) are untouched;
   the hook gate runs lychee offline and `just check-links-online` is outside `check`.
   `just check-docs` must pass.

7. The gate on the edited modules. `just check-specs` and `just analyse-specs` again.
   Quote the seven JSON blocks of each in the hand-back notes, each of the shape

   ```json
   {"spec_file": "docs/specs/<name>.allium", "diagnostics": [], "findings": []}
   ```

   with whatever other keys 3.6.1 prints, and the closing line. This is CONVENTIONS.md
   §12's claim for T06 ("allium 3.6.1 reports empty `diagnostics` and `findings` for the
   seven migrated modules after the §9 edits"): record it as met, or, if a diagnostic
   appears, stop and write it up as a design change rather than waiving it. A comment
   edit cannot produce a diagnostic; one here means an edit strayed outside a comment.

8. The acceptance search, over this ticket's files and then over the whole set
   CONVENTIONS.md §9 names:

   ```sh
   grep -n -i "pawdoku\.allium\|this game\|randomness port\|src/lib\|platformSpecs" docs/specs/*.allium docs/explanation/solving-sudoku.md docs/explanation/human-solving.md
   grep -n -i "pawdoku\.allium\|this game\|randomness port\|src/lib\|platformSpecs" docs/specs/*.allium docs/explanation/*.md
   ```

   The first prints nothing. The second may hit another lane's stub or page under
   `docs/explanation/`; such a hit is that lane's and is named in the hand-back notes,
   not fixed here. `git diff --stat main` lists the nine files above and this ticket
   and nothing else.

9. `just check` from the top; quote its closing lines. Fill in the hand-back notes:
   what was verified, the completed review table, deviations, the hand-back list with
   any corrections to the line numbers below, and the open points settled. Set
   `status: done`, commit on the ticket branch with short imperative subjects (one commit
   per module is fine), and stop before pushing.

## Acceptance criteria

- The first grep of step 8 prints nothing.
- `just check-specs` and `just analyse-specs` each print seven blocks with empty
  `diagnostics` and `findings` arrays and end with `7 specifications, no diagnostics
  and no findings.`; `.tools/bin/allium --version` reports `3.6.1`; no
  `allium-ignore` directive exists in `docs/specs/`.
- `head -1 docs/specs/*.allium` prints `-- allium: 3` seven times.
- The relevance review table is complete: every Includes and Excludes bullet of every
  module has a verdict and a reason, and no word-search hit is unaccounted for.
- Every `open question` block present at `78d03cdf` is present, unchanged, and listed in
  the hand-back notes by module and line: 2 + 3 + 7 + 1 + 6 = 19.
- No link in the two pages points outside this handbook; `just check-docs` passes.
- `solver.allium`, `technique.allium`, `reach.allium`, `effort.allium` and
  `solving-sudoku.md` are byte-identical to G's, unless the review changed a verdict to
  adapt, and then the hand-back notes say which clause and why.
- `just check` is green; `git status --porcelain` is empty after it.
- The hand-back to G is written as concrete edits with G paths and line numbers.

## Verification

```sh
git -C /Users/scutting/projects/pawdoku rev-parse --short=8 HEAD
.tools/bin/allium --version
just check-specs
just analyse-specs
head -1 docs/specs/*.allium
grep -rn "allium-ignore" docs/specs/ || echo "no waivers"
grep -n -i "pawdoku\.allium\|this game\|randomness port\|src/lib\|platformSpecs" docs/specs/*.allium docs/explanation/solving-sudoku.md docs/explanation/human-solving.md || echo "no game words"
grep -c "^open question" docs/specs/*.allium
for m in solver technique reach effort; do cmp "docs/specs/$m.allium" <(git -C /Users/scutting/projects/pawdoku show "78d03cdf:docs/specs/$m.allium") && echo "$m identical"; done
cmp docs/explanation/solving-sudoku.md <(git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/explanation/solving-sudoku.md) && echo "solving-sudoku identical"
git diff main --stat
just check-docs
just check
git status --porcelain
```

Expected: `78d03cdf` (or a note that the clone moved and `git show` was used); `allium
3.6.1 (language versions: 1, 2, 3)`; two runs ending `7 specifications, no diagnostics
and no findings.`; seven `-- allium: 3`; `no waivers`; `no game words`; the open-question
counts `sudoku 0, solver 0, technique 2, reach 3, effort 7, lapse 1, human-solving 6`;
five `identical` lines; a diff stat naming only the nine files and this ticket; both
gates green; nothing from `git status`.

## Hand-back notes

### What was verified, and how

### The relevance review

Pre-filled from the modules at `78d03cdf`; the executing agent confirms or changes each
verdict in place and states the reason for any change. Verdicts: **keep** (relevant to a
library as written), **adapt** (relevant, but the wording presumes the game), **drop**
(not the library's; none proposed).

`sudoku.allium`

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 13-14 | Includes "The grid: config.side rows..." | keep | The engine's data model |
| 15-16 | "Units and peers..." | keep | Read by every technique |
| 17-18 | "Setting: a setter supplies givens..." | keep | `SetPuzzle` is the library's entry point for a puzzle |
| 19-20 | "Play: placing a digit in a cell..." | keep | `PlaceDigit` and `EraseDigit` are what a hint reads (`reach` `BeginHint`); "Play" is the act, not G's surface (Open points) |
| 21-22 | "The outcome: a puzzle is solved when..." | keep | `PuzzleSolved` |
| 23-25 | "Two boundaries, PuzzleSetting and PuzzleSolving..." | keep | The library's API shape: where stimuli come from and what is visible; "Neither says anything of how it looks" |
| 28-29 | Excludes "Variants: other sizes..." | keep | Still excluded |
| 30-32 | "How a setter finds givens, how hard..." | keep | `solver.allium`'s and the models' |
| 33-34 | "Notes, hints, undo, timers, counts of mistakes..." | keep | None is a rule; hints are `reach`'s, the rest a consumer's |
| 35-37 | "Whether a surface refuses, flags or merely allows..." | adapt | Names `pawdoku.allium`'s `Play` surface (step 5) |
| 40-41 | Dependencies "None. The rules need no figure pawdoku.allium states..." | adapt | Names the root module (step 5) |
| 7-8, 29 | Scope "the game as it is played on paper"; "a different game" | keep | Sudoku the game and its variants, not Pawdoku |
| 324-328 | Comment on `PuzzleSolving`: "the module that draws them owes a non-colour indication and an accessible name" | keep | An obligation on whoever draws, true wherever that module lives (Open points) |

`solver.allium` (no game words; verbatim)

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 16-17 | Includes "The search: one for each set of givens..." | keep | The verdict is the library's first deliverable |
| 18-20 | "Branches: a grid of candidates each..." | keep | Observable shape of the search |
| 21-23 | "Propagation: a placed digit leaves..." | keep | |
| 24-26 | "Contradiction: a cell with no candidate..." | keep | |
| 27-29 | "What makes the search quick and can be seen..." | keep | Guarantees a consumer may rely on |
| 30-31 | "Two boundaries: Solving ... and SearchResult" | keep | The library API: `Solve(givens)` in, verdict and solutions out |
| 34-36 | Excludes "How any of it is stored or made fast..." | keep | The implementation's, as it should be for Rust |
| 37-38 | "The order in which tied cells and tied branches..." | keep | Deterministic tie-breaking is the implementation's |
| 39-41 | "Deductions past singles..." | keep | `technique.allium`'s |
| 42-45 | "Steps a person could follow, hints, and how hard..." | keep | The models' |
| 46-47 | "Finding givens, and solving from a puzzle in play." | keep | Generation is out of scope for the library today |
| 48 | "Variants, as sudoku.allium excludes them." | keep | |

`technique.allium` (no game words; verbatim)

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 26-28 | Includes "The supported catalogue in ladder order..." | keep | The catalogue is the library's core |
| 29-31 | "The grid a technique is read from..." | keep | |
| 32-34 | "Twenty-nine techniques, through basic fish..." | keep | |
| 35-36 | "Deductions: one for each place a technique holds..." | keep | The library's output type for a step |
| 37-40 | "The profile: repertoire, capacity, spans, marking, order, fixation, upkeep, budget, fatigue and patience, and two presets" | keep | What rating and hinting read; marking and fixation are player description, not UI (Open points) |
| 43-44 | Excludes "Which deduction a player takes..." | keep | The models' |
| 45-49 | "Keeping candidates true..." | keep | What `lapse` varies |
| 50-52 | "Finned, sashimi and other complex fish..." | keep | Deferred families stay deferred |
| 53-54 | "Chance. Nothing here is drawn at random..." | keep | |
| 55-56 | "How a grid is stored, how deductions are found quickly..." | keep | |
| 57 | "Variants, as sudoku.allium excludes them." | keep | |
| 106-108, 115-117 | `enum Marking`, `enum Fixation` | keep | Read by the Profile |
| 288-289 | "A hint over unverified player entries cannot assert this premise." | keep | Engine semantics of `uniqueness_promised` |
| 971 | "Labels are logical values, not a requirement to display colour." | keep | Already says it is not UI |
| 1160, 1162 | Open questions | keep | Carried |

`reach.allium` (no game words; verbatim)

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 21-22 | Includes "The run: one for each rating or hint..." | keep | |
| 23-24 | "Sight: the four ways a profile hides..." | keep | |
| 25-27 | "Choice: a systematic player takes..." | keep | |
| 28-29 | "Steps: the deductions taken, in order..." | keep | The trace a rating reads; the one step a hint is |
| 30-31 | "Three boundaries: Rating and Hinting ... and RunResult" | keep | The library API for rating and hinting |
| 34-35 | Excludes "Cost..." | keep | `effort`'s |
| 36-40 | "Error and guessing..." | keep | `lapse`'s |
| 41-43 | "Which of several equally preferred deductions..." | keep | Deterministic tie-breaking |
| 44 | "How a hint is worded or shown, and anything a surface draws." | keep | Already excludes the UI; "a hint pitched at them" (12) is engine output |
| 45 | "Chance..." | keep | |
| 375 | "Where a hint is asked for: the surface a puzzle is played on." | keep | A consumer's surface is exactly that (Open points) |
| 397 | "It draws nothing and words nothing." | keep | |
| 421, 423, 425 | Open questions | keep | Carried |

`effort.allium` (no game words; verbatim)

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 26-28 | Includes "The run: one for each pricing..." | keep | |
| 29-30 | "Price: a base figure for the technique..." | keep | Open figures; nothing built on them (Scope 18-20) |
| 31-32 | "Escalation: a step from beyond the profile..." | keep | |
| 33-35 | "What a rating reads: the dearest step, the total..." | keep | The `PricedRun` surface |
| 38-41 | Excludes "Missing and erring..." | keep | |
| 42 | "Hints..." | keep | `reach`'s |
| 43 | "Time. A price is effort and no clock is read." | keep | Matches the `no_std` core: no clock |
| 44 | "Chance, and which of several equally cheap..." | keep | |
| 304-316 | Seven open questions, 312 among them | keep | Carried; the maintainer is not asked to answer them here (Open points) |

`lapse.allium`

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 24-28 | Includes "Where error comes from..." | keep | |
| 29-30 | "Upkeep: after a placement..." | keep | |
| 31-32 | "Four kinds of step..." | keep | |
| 33-34 | "Choice: the deduction taken..." | keep | |
| 35-37 | "Being stuck..." | keep | |
| 38-39 | "The guess: in a cell with the fewest marks..." | keep | |
| 40-41 | "The run, which ends solved, or abandoned..." | keep | |
| 44-48 | Excludes "Slips: a digit misread..." | adapt | "this game reaches through its randomness port" (step 5) |
| 49-52 | "Cost, which is effort.allium's, and hints..." | keep | |
| 53-56 | "A faulty repair, and a contradiction that must be looked for." | keep | |
| 57-60 | "Which of several equal deductions..." | keep | |
| 531 | Open question | keep | Carried |

`human-solving.allium`

| Lines | Clause | Verdict | Reason |
| --- | --- | --- | --- |
| 18 | Includes "Independent cognitive ability, learned expertise, biases and coping." | keep | Simulation inputs |
| 19 | "Separate physical sheet, bounded mental state, and observer judgement." | keep | |
| 20 | "Explicit microsteps, effort, provenance, seeds and finite budgets." | keep | Budgets are step budgets, as the core requires |
| 21 | "Manual/automatic notes, highlighting, feedback and uniqueness premise." | keep | The modelled conditions the simulated person solves under, an input (`Environment`, 164-171, default `paper` at 497-500), not the consumer's UI (Open points) |
| 22 | "Named pattern recognition, bounded elementary reasoning, and guesses." | keep | |
| 23 | "Within-attempt fatigue, frustration and rest..." | keep | |
| 24 | "Four provisional experience presets..." | keep | |
| 25-27 | "The projection from this module's description..." | keep | |
| 30 | Excludes "Runtime/UI implementation, persistence, puzzle generation and hints." | keep | Exactly the library's exclusions |
| 31 | "Importing in-progress games, Sudoku variants, or learning across games." | keep | "games" are puzzles in play |
| 32-33 | "Diagnoses, intelligence scores, predictions in minutes..." | keep | |
| 34 | "An exhaustive expert repertoire or an unbounded solution search." | keep | |
| 35-38 | "Changing reach, effort or lapse into this model..." | keep | |
| 39-40 | "The random generator. Draws come from this game's randomness port" | adapt | Step 5 |
| 43-50 | Dependencies, including "An explicitly enabled feedback aid may reveal only the queried entry's error signal." | keep | Engine semantics of the aid |
| 476-478 | `random_version` comment | adapt | Step 5 |
| 890-906 | `AidsChangeTheEnvironment` | keep | What each aid reveals to the simulated player; a consumer maps its aids onto these (Open points) |
| 1096-1099 | `ExactReplay`, the port sentences | adapt | Step 5 |
| 1109-1111 | "no platform rounding"; "No wall clock, platform iteration order or hidden random source" | keep | The computing platform; the clause is the `no_std` contract in the spec's own words |
| 1270-1280 | Six open questions | keep | Carried |

### Deviations, and why

### Handed back to pawdoku

Performed by a later G ticket, separately authorised, never by this one. G paths and
line numbers at `78d03cdf`; the executing agent corrects any that G has since moved.

1. Delete `docs/specs/sudoku.allium`, `solver.allium`, `technique.allium`,
   `reach.allium`, `effort.allium`, `lapse.allium`, `human-solving.allium`, and
   `docs/explanation/solving-sudoku.md` and `docs/explanation/human-solving.md`.
2. `docs/manifest.yml` lines 50-51: remove the two entries (and the trailing comma on
   line 48).
3. `docs/README.md` lines 15-28: rewrite the specifications paragraph so that it is
   rooted at `pawdoku.allium`, says the engine's seven modules live in
   `steven-cutting/libpawdoku` under `docs/specs/` and are restated here and held equal
   by test, and drops the six links; lines 84-87: remove the two bullets under "This
   game" (the heading may stay for pages the game adds later).
4. `docs/how-to/work-with-the-specs.md` lines 11-12: "the root module named after its
   slug, and the modules the game adds beside it" becomes the root module alone, with a
   sentence naming where the engine's are; table rows 20-26 deleted, leaving row 19;
   step 2 at line 37 gains "and a restated engine clause is held to libpawdoku's text
   the same way".
5. `docs/explanation/specifications.md` line 12 ("rooted at the module named after this
   game" stays true); lines 33-68: "Eight are the game's today" becomes one, the
   paragraph on `sudoku.allium` and `solver.allium` (37-43) and the paragraph "Five more
   say how a person solves" (45-68) become a short account of where the engine's live
   and that the text is shared truth across two repositories until S04 ships the specs
   in the wasm package (CONVENTIONS.md §13); line 95: remove the link to
   `human-solving.md`.
6. `docs/project/terminology.md` lines 17-19, 36, 44, 59-71: the engine words stay
   (the game uses them) but the sentences that say which module owns each now say the
   module is libpawdoku's; line 96: remove the link to `../explanation/human-solving.md`.
7. `AGENTS.md` lines 170-173: the deviations list "none yet" gains "The engine's
   specifications live in `steven-cutting/libpawdoku`, restated here and held equal by
   test (decision 0011)"; invariant 1 (lines 30-32) may say the engine's rules are
   libpawdoku's `docs/specs/`.
8. `docs/specs/pawdoku.allium` Scope lines 9-11: "The rules of the game are
   sudoku.allium's, a module beside this one that imports nothing. How those rules are
   drawn is still to be specified, in modules that import both." becomes "The rules of
   the game are the engine's, specified in libpawdoku's sudoku.allium, restated here
   where needed and held equal by test, not imported. How those rules are drawn is still
   to be specified."; Dependencies lines 27-29 add that the engine's seven modules are
   held equal by test for the same reason as the platform's three.
9. `.agents/skills/spec-change/SKILL.md` line 10 ("a clause the platform states is
   owned by `@steven-cutting/biscuit-games` and only restated here") and line 13 gain
   the engine as a second owner held by test.
10. Two decisions G owes, numbered from 0011 (G's `docs/decisions/README.md` line 44):
    "The engine lives in libpawdoku" and the `allium-skill-reference/` relocation that
    G's commit `189348e` called "a template departure worth a decision record" and did
    not write.
11. `pyproject.toml` lines 63-66, the `[tool.typos.default.extend-words] mis = "mis"`
    allowlist: at `78d03cdf` the hyphenated `mis-` uses are in
    `allium-skill-reference/allium/actioning-findings.md` line 64 and
    `recommended-loops.md` line 63, not in the seven modules or the two pages, so the
    migration does not change whether it is needed; G decides.
12. `.pre-commit-config.yaml` lines 60-65 and the `analyse-specs` hook: unchanged;
    `pawdoku.allium` stays, so `docs/specs/` is not empty and the `check-specs` gate
    still has something to walk. The restatement test itself, and how the engine's text
    reaches G (the wasm package shipping `specs/`), are S04's.

### Open points settled

## Open points

- **`human-solving.allium`'s environment and feedback model.** Includes line 21, the
  `Environment` value (164-171: `manual_notes`, `automatic_candidates`, `highlighting`,
  `feedback`, `uniqueness_disclosed`, `branch_notes`), the default `paper` (497-500) and
  `AidsChangeTheEnvironment` (890-906). A surface's concern or a solver input? The
  ticket writer's verdict is keep: it is an input to `Simulate`, describing the
  conditions the simulated person solves under, defaulted to paper, and a consumer maps
  whatever aids it draws onto these six fields. The maintainer confirms, or names the
  fields that should not be the library's.
- **`technique.allium`'s marking and fixation.** `Marking` (106-108), `Fixation`
  (115-117), the Profile fields (226-228) and the partial-marking open question (1160).
  Keep: rating reads them, and a player who keeps no marks is a real player, not a UI
  setting. Confirm.
- **Hint wording.** `reach.allium` 12 ("a hint pitched at them"), 44 (worded or shown is
  excluded), 375 ("the surface a puzzle is played on") and 383-395; `technique.allium`
  288-289 and 1147-1149. Keep verbatim, as CONVENTIONS.md §9 says: a hint is engine
  output and its wording is a consumer's. The one sentence worth a second look is 375,
  which could read "a consumer's surface, where a puzzle is played"; the recommendation
  is to leave it.
- **`effort.allium`'s open figures.** Seven questions (304-316). Carried, none answered;
  the library builds nothing on `base_figure` or `search_effort` until they are settled.
  Whether S03 or a later ticket takes them up is not this ticket's.
- **`Play`-adjacent vocabulary in `sudoku.allium`.** The Includes bullet (19-20), the
  `-- Play ---` section (196), `PuzzleSolving` (329-351) and the comment at 324-328 that
  "the module that draws them owes a non-colour indication and an accessible name".
  Recommendation: keep all four. "Play" names the two moves, which the library must
  model because a hint reads a puzzle in play; `PuzzleSolving` is the boundary a
  consumer's surface provides; the accessibility sentence is an obligation on whoever
  draws and is true of a consumer. If the maintainer prefers, 324-326 becomes "a
  consumer that draws them owes", a comment-only change.
- **A single player on a single device.** Nothing in the seven modules presumes one. An
  `Attempt` is one seeded solve, an `Assessment` runs many; no module names a device, a
  browser or storage; `human-solving.allium` 1109-1111 says "platform" in the computing
  sense. Recorded here so the search is on the record; the executing agent re-checks.
- **Whether the library should gain a thin root module of its own.** Recommendation:
  not now. There is no figure for it to state and no surface for it to guarantee, so the
  analyser would have nothing to check, and `sudoku.allium` already imports nothing.
  S04 decides how consumers restate what they need, and a root module can be added the
  day something is shared by all seven.
- **The acceptance grep over `docs/explanation/*.md`.** CONVENTIONS.md §9 runs it over
  every explanation page, of which this ticket owns two; T08's pages may legitimately
  say "the game's root module". If the second grep of step 8 hits another lane's page,
  the hand-back names it and T11 reconciles.
