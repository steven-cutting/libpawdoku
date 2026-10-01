---
id: T24
title: "A plan-spec recipe: the test obligations allium derives from one module"
status: done
depends_on: [T23]
parallel_with: []
branch: ticket/t24-plan-spec-recipe
estimated_size: S
---

# T24: A plan-spec recipe: the test obligations allium derives from one module

## Context

T25 to T28 build the first engine modules test-first, each from a list of expected tests
written before any code. The list is made by hand from the module's clauses, and the
maintainer decided on 2026-09-30 that the agent should also read what the Allium binary
suggests. The pinned binary (allium-tools 3.6.1, `.tools/bin/allium`) has a `plan`
subcommand that prints the test obligations a module implies as JSON, and a `model`
subcommand that prints its domain model. `.agents/skills/propagate/SKILL.md` names both
as prerequisites.

No recipe runs either, and `AGENTS.md` says: "Never invent a command: if a recipe does
not exist, add it to the `Justfile`". The house wrapper cannot help: `bg-run-allium`
accepts only `check` and `analyse` and asserts both report nothing, which is the opposite
of what `plan` is for. So the recipe calls the binary.

Facts verified on 2026-09-30 with the 3.6.1 binary:

- `allium plan` takes exactly one file (`Usage: allium plan <file.allium>`).
  `board.allium` and `solver.allium`, which import `sudoku.allium`, each plan with an
  empty `diagnostics` array and exit 0. Not verified: whether `plan` reads the imported
  file or only does not report the import. `allium check` on one file alone does report
  `allium.use.unresolvedPath` for each `use` line, and the obligations seen for
  `board.allium` name its own constructs and carry no type from `sudoku.allium`. Step 3
  settles it.
- The output is one JSON document with a `diagnostics` array and an `obligations` array.
  Each obligation has an `id`, a `category`, a `description`, the `source_construct` it
  comes from and a `source_span`.
- It printed 48 obligations for `sudoku.allium`, 101 for `solver.allium` and 78 for
  `board.allium`. The categories include `rule_success`, `rule_failure`, `invariant`,
  `transition_edge`, `derived`, `surface_exposure` and `surface_provides`.
- It writes nothing to the tree.
- It exits non-zero only when the module carries an error-severity diagnostic.

The `Justfile` is frozen (CONVENTIONS.md §11). T22 reopened it for `metrics`; this is a
second T00 follow-up of the same class, smaller, because the new recipe is not a gate.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §4 and §11; the `Justfile`, the
"documents" block and the `metrics` recipe (how T22 kept a missing binary from falling
through to another on `PATH`); `docs/reference/commands.md`;
`docs/how-to/work-with-the-specs.md`; `.agents/skills/propagate/SKILL.md`.

## Goal

`just plan-spec <module>` prints the obligations `allium plan` derives from
`docs/specs/<module>.allium`. It is outside `just check`, asserts nothing and writes
nothing. The commands page lists it, the specifications how-to says what it is for, and
the propagate skill names the recipe where it names the subcommand.

## Non-goals

- Not a gate. `pyproject.toml`'s recipe list is unchanged and `check` runs the same
  nineteen gates.
- No assertion on the output, no filtering and no summary: the recipe prints what the
  binary prints.
- No change to the pinned binary, to `bg-run-allium` or to the tooling package.
- No test written from the output. T25 is the first ticket to use it.
- No recipe for `allium model` unless the maintainer answers the first open point.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `Justfile` | T00 follow-up | The `plan-spec` recipe, after `analyse-specs` |
| `docs/reference/commands.md` | reference | A row in the Documents table |
| `docs/how-to/work-with-the-specs.md` | how-to | One paragraph: what the recipe prints and that it is a suggestion list |
| `.agents/skills/propagate/SKILL.md` | skill | The Prerequisites item for test obligations names `just plan-spec <module>` |
| `CHANGELOG.md` | maintainer docs | Under Unreleased, Added |
| `tickets/README.md` | ticket index | T24's row set `done` |
| `tickets/T24-plan-spec-recipe.md` | ticket | `status:`, hand-back notes |

## Steps

1. Create the worktree on `ticket/t24-plan-spec-recipe` from `main` after T23 has merged.
   Run `just initialize` if `.pixi/` is absent; it uses the network and this ticket
   authorises it. Confirm `.tools/bin/allium` exists.

2. Add the recipe to the `Justfile`, after `analyse-specs`:

   ```text
   # The test obligations allium derives from one module, as JSON: what each
   # rule, invariant, transition and surface clause would have a test for. A
   # suggestion list for a test list, not a gate: nothing asserts on it and it
   # is outside `just check`. `allium plan` takes one file and resolves the
   # module's imports from beside it. The binary is named by path so that a
   # missing .tools/bin/allium fails here and does not fall through to another
   # allium on PATH.
   plan-spec module:
       .tools/bin/allium plan "docs/specs/$1.allium"
   ```

   The file sets `positional-arguments`, so `$1` is the module, as `check-clean` reads
   its argument.

3. Run it for `sudoku`, `solver` and `board` and record, for each, the exit status, the
   length of `obligations` and that `diagnostics` is empty. Run it for a module that
   does not exist and record how it fails. Run `just check` and confirm the worktree is
   unchanged afterwards, which proves the recipe wrote nothing a gate can see.

   Then settle the unverified claim under Context. Read `board`'s obligations for the
   constructs that reach into `sudoku.allium` (`BoardCell.puzzle_cell`, the `Place` rule
   that chains to `sudoku/PlaceDigit`) and say whether anything in them comes from the
   imported file. If `plan` does not read imports, the recipe is still right, since each
   module is planned by its own ticket; say so on the commands page, so that nobody
   expects `plan-spec board` to list the rules' obligations. The recipe's comment is
   then wrong where it says `plan` "resolves the module's imports from beside it":
   replace that sentence with one that says `plan` takes one file and plans that
   module alone, reading no import. The comment and the commands page say the same
   thing, whichever way the experiment falls.

4. The pages:

   - `docs/reference/commands.md`: a row under Documents. It says what the recipe prints,
     that it takes the module's name without the extension, that it is outside
     `just check`, and that it needs `just install-allium`.
   - `docs/how-to/work-with-the-specs.md`: a short paragraph where the page describes
     deriving tests from a clause. The output is a starting list. An obligation that
     names storage a module excludes, or that no behaviour can show, is struck with a
     reason, and a clause the output misses is still owed a test.
   - `.agents/skills/propagate/SKILL.md`: in Prerequisites, "from `allium plan <spec>`"
     gains "run here as `just plan-spec <module>`". The `model` item is left as it is
     unless the first open point is answered.

5. `CHANGELOG.md`, under Unreleased, Added: the recipe, in one line.

6. `just check-agents`, `just check-docs`, `just lint`, `just check`. Fill in the
   hand-back notes, set `status: done` here and in `tickets/README.md`, commit on the
   ticket branch, and stop before pushing.

## Acceptance criteria

- `just plan-spec sudoku`, `just plan-spec solver` and `just plan-spec board` each exit
  0 and print one JSON document whose `diagnostics` array is empty and whose
  `obligations` array is not empty.
- `just plan-spec` with a module that does not exist fails with a non-zero exit.
- `pyproject.toml` is unchanged, and `just check` runs the same gates as before and is
  green.
- `docs/reference/commands.md` has the row, and it says the recipe is outside
  `just check`.
- `just check-agents` and `just check-docs` pass.

## Verification

```sh
just plan-spec sudoku
just plan-spec solver
just plan-spec board
just check-agents
just check-docs
just lint
just check
```

Expected: three JSON documents with empty `diagnostics` and 48, 101 and 78 obligations,
or the counts the modules give on the day if T23's comment edits moved none of them;
`Validated AGENTS.md, 2 adapters, and 14 skills.`; the documents gate and the hook gate
green; `All checks passed and the worktree is unchanged.`

## Hand-back notes

### What was verified, and how

On 2026-10-01, the recipe printed one JSON document per module:

| Command | Exit status | Obligations | Diagnostics |
| --- | --- | --- | --- |
| `just plan-spec sudoku` | 0 | 48 | `[]` |
| `just plan-spec solver` | 0 | 101 | `[]` |
| `just plan-spec board` | 0 | 78 | `[]` |
| `just plan-spec t24-no-such-module` | 1 | No JSON | No JSON |

The missing-module run reported
`docs/specs/t24-no-such-module.allium: No such file or directory (os error 2)`.

`BoardCell`'s `entity_fields` obligation lists `puzzle_cell` but carries no imported
type information. `Place` has success, four guard-failure and entity-creation
obligations; their dependencies name `Move`, with no obligation for
`sudoku/PlaceDigit`. No source construct in the board's plan names `sudoku` or
`PlaceDigit`. To check whether imports were read, a byte-identical copy of
`board.allium` was placed under the ignored `ai_tmp/t24/`, with neither imported
file beside it. `just plan-spec ../../ai_tmp/t24/board` exited 0 and produced
identical parsed JSON, including empty diagnostics. Planning reads this module
alone; imported modules must be planned separately. The recipe comment and commands
page state that limit.

Verification output:

```text
just check-agents
Validated AGENTS.md, 2 adapters, and 14 skills.

just check-docs
markdownlint, typos and lychee: Passed
Validated 41 pages and 42 canonical topics.

just lint
All 23 hooks: Passed

just check
The worktree matches the check baseline.
All checks passed and the worktree is unchanged.
```

`pyproject.toml` is unchanged: the same eighteen configured gate recipes run,
followed by `check-clean`. Plan output and the import probe stayed under ignored
`ai_tmp/`; no worktree-visible output was created by the recipe or the full gate.

### Deviations, and why

The existing worktree was already on `ticket/T24-plan-spec-recipe`, based on the
merged T23, with `.pixi/` and `.tools/bin/allium` installed. No new worktree or
network initialization was needed. The shell put pixi's Cargo ahead of rustup;
the full check used `~/.cargo/bin` first, as the local-development page requires.
Checks needed sandbox access to the existing prek and Rust caches outside the
worktree; they passed with that access.

The proposed import-resolution sentence was replaced after the isolated-module
experiment. No `model-spec` recipe or test-list procedure relocation was added;
those open points remain deferred. Raw JSON remains the intended output.

### Handed back

The recipe, commands row, test-derivation paragraph, propagate prerequisite and
changelog entry are complete. T24 is marked done here and in the ticket index.
Committed locally on the ticket branch; stopped before pushing.

## Open points

- **`allium model`.** The propagate skill names it beside `plan`, for entity shapes and
  state machines. The ticket adds no recipe for it, because the maintainer asked for
  `plan` alone. If one is wanted, it is the same three lines under the name `model-spec`
  and one more row on the commands page.
- **Where the test-list procedure lives.** T25 to T28 each state it in full, so each is
  executable alone. A durable home would be `.agents/skills/rust-change/SKILL.md`, whose
  step 4 says "derive the test from the clause before implementing". Moving it there is
  the maintainer's call and a change to agent guidance, not this ticket's.
- **Raw JSON.** The output for `solver` is about 43 kB. The recipe prints it as it is;
  a summary by category would be a script, which this ticket does not write.
