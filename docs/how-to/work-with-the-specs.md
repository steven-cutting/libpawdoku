---
title: "Work with the specifications"
kind: "how-to"
audience: [contributor, maintainer, agent]
canonical_for: [specification_workflow]
requires: []
---

# Work with the specifications

The nine Allium modules under `docs/specs/` decide what the engine does. The game's root
module stays in the game and restates what it needs of these, held equal by test on the
game's side. This page is the procedure;
[Specifications](../explanation/specifications.md) is the reasoning.

## Find the module that owns the behaviour

| Module | Owns |
| --- | --- |
| `sudoku.allium` | The rules of classic Sudoku and nothing about how they look: the grid, its units and peers, what a setter may pose, the two moves a player has, conflicts, and when a puzzle is solved. It imports nothing; a module that draws the rules imports it. |
| `board.allium` | The puzzle in play: one board a puzzle, a note in every cell with upkeep, the four moves with exact undo and redo, the board as it stood after any move, the check of one cell against the solution, and the record a board is written to and reopened from. It imports `sudoku.allium` and `solver.allium`: every placement and erasure still goes to the rules' own moves, and the check reads the one solution the solver finds. |
| `solver.allium` | What decides whether a set of givens is well-posed: the search, its branches and candidates, propagation by singles, contradiction, the guess on a cell with the fewest candidates, and the verdict of none, one or many with the solutions found. It imports `sudoku.allium`, which does not import it, and says nothing of how a solver is stored or made fast. |
| `technique.allium` | The shared catalogue of 29 named techniques, proof obligations and acceptance schemas; the grid and coarse profile that `reach`, `effort` and `lapse` read; and the record of how its earlier open questions were resolved. It imports `sudoku.allium` alone. |
| `reach.allium` | The player model in which a limit hides a deduction: what a profile sees, which deduction it takes next, how far it gets through a set of givens, and the hint it would be given in a puzzle in play. It imports `sudoku.allium` and `technique.allium` and none of the other three models. |
| `effort.allium` | The player model in which a limit makes a deduction dear: the price of a step, escalation past the profile, and a puzzle priced end to end. Coarse: the figures are open questions. Same imports as `reach.allium`. |
| `lapse.allium` | The player model in which a limit leads to error: upkeep of marks, checks, guesses and repairs, rated by steps and by the count of each kind, asked for by `Tackle`. Coarse and deterministic, with the record of how its open questions were resolved. Same imports as `reach.allium`. |
| `human-solving.allium` | Bounded attention and memory, independent experience, fallible beliefs and notes, explicit effort and seeded attempts, plus profile-relative repeated assessment. The fourth player model and the one with chance in it, drawn through the randomness boundary. Owns its finer profile, four presets and the projection onto `technique.allium`'s profile. Same imports as `reach.allium`. |
| `generation.allium` | How a setter that is a program finds givens, in two ways. The basic way, stated in full: a solution grid drawn through the randomness boundary, givens removed one position at a time in the order a tier names, each removal kept only while the solver's verdict stays one. The designed way, a skeleton of scope, config and open questions with no triggers: givens removed by symmetry orbit until the puzzle is well-posed and solvable within a technique contract, under a step budget. It imports `sudoku.allium`, `solver.allium`, `technique.allium` and `reach.allium`, and nothing imports it. |

Each module opens with `Scope`, `Includes` and `Excludes`. If your change falls under
another module's `Excludes`, it belongs there.

## Change behaviour

1. Change the specification first. Add or amend the rule, its triggers, its guards and
   its outcomes.
2. Check the modules that depend on it. A module others import is depended on by each of
   them, so a change to a trigger or an entity ripples. The game restates clauses of these
   modules and holds them equal by test on its side, so a reworded clause here is a change
   the game must take.
3. Derive tests from the changed clauses and confirm they fail before implementing. A
   test that is green before you write any code is either already covered or vacuous.
4. Implement until they pass, without weakening any test.
5. Run `just check-specs`. It fails on any diagnostic at all, whatever its severity, so
   a diagnostic is a regression: fix it, or — for a verified checker gap — waive it on the
   terms below. Then run `just analyse-specs`, which fails on a diagnostic or a finding;
   a finding cannot be waived.
6. Run `just test`, then `just check`.

When deriving tests from a clause, run `just plan-spec <module>` for a starting list
of test obligations as raw JSON. Treat it as suggestions: strike an obligation that
names storage the module excludes, or that no observable behaviour can show, with a
reason. A clause the output misses is still owed a test.

## Handle an open question

An `open question` block records a product decision nobody has made yet, so the gap is
visible rather than silently filled in. These modules carry more than twenty, in every
module but `sudoku.allium` and `solver.allium`: `effort.allium`'s open figures and
`human-solving.allium`'s uncalibrated defaults among them. Treat them as the decisions
still owed, never as debt to clear by guessing, and keep adding them where a gap opens.

- If your change depends on one, raise it. Do not answer it in code.
- If your change creates a new gap, add an `open question` rather than picking an answer.
- Answering one is a real change: edit the specification to state the decision and delete
  the question in the same commit.

## Tooling

The `allium` command-line tool validates and analyses these files, and this project owns a
pinned copy of it. `just initialize` installs it; afterwards, or after a version change,
`just install-allium` puts it in the gitignored `.tools/bin/`. It is a checksummed binary
rather than a package in a lockfile — see
[decision 0005](../decisions/0005-project-managed-allium-cli.md).

```console
just check-specs
just analyse-specs
```

The first runs `allium check` over every module, which reports on structure: syntax,
references, and names a module reaches for that no import defines. The second runs
`allium analyse`, which repeats every one of those diagnostics and adds process-level
findings on top — data flow, edge reachability, deadlocks, conflicts and invariants. The
`spec-change` skill in `.agents/skills/` carries the procedure for agents.

### Diagnostics and waivers

Both recipes are part of `just check`, and both run as hooks in the read-only gate, so a
commit that touches `docs/specs/` is held to them. That is what
[decision 0005](../decisions/0005-project-managed-allium-cli.md) records, and it means a
worktree needs `just install-allium` before `just lint` or `just check` will pass. Both are
clean on an untouched checkout: every module reports an empty `diagnostics` array and an
empty `findings` array, and both recipes print one JSON block per module and exit 0.

Neither recipe takes the exit code as its verdict, because neither exit code carries what
this project means by clean. `allium check` exits 0 on an `info` diagnostic — `allium.field.unused` is
one, so the waiver the modules used to carry for it was never what kept the recipe green —
and `allium analyse` keys its status on findings alone, so a module that does not parse
passes it with the `error` sitting in the JSON it has just printed.
`bg-run-allium` reads the arrays instead, and treats a non-zero status beside an
empty report as a fault in the tool, never as a pass.

Either recipe reporting anything at all is therefore a regression in the change under
review. Fix it at the root. A finding cannot be waived. A diagnostic can, but only when the
diagnostic itself is wrong — the construct is valid Allium that the pinned checker cannot
resolve — and then it is waived in place:

```text
-- Why the checker is wrong here, in a sentence.
-- allium-ignore allium.reference.unknownName
```

The directive is a whole-line comment holding the full diagnostic code and nothing else —
prose on the directive line disables it, which is why the reason sits on its own line
above — and it covers only the line directly beneath it. One rule, one line, one stated
reason. Upstream documents none of this: the directive was found in the binary and
re-verified against 3.6.1, so every waiver must be re-verified whenever the pinned version
moves — see [Maintain dependencies](maintain-dependencies.md). No waiver is currently in
the modules.

Where a shape can be retired rather than waived, that is the better route. A `related:`
clause that names a surface through a module alias is one the checker cannot resolve, and
the language reference cannot be read to sanction it either: rule 31 asks only that a
surface in `related:` be defined, and no example anywhere qualifies a surface name with an
alias. Where a waiver would assert that the checker is wrong, the honest form is prose —
state the adjacency in the guarantees of the surfaces concerned — and no waiver is written.

`allium.field.unused` counts uses within one module only, so a definition whose only
readers sit in another module is reported, although the language has always allowed a
module to read another's fields. It also does not count a use inside a projection's
`where` predicate, so a derived value read only as `moves where is_next_to_redo` is
reported as unused; `board.allium` answers that with an invariant that states a true
property of the value, which is the better route than a waiver. A waiver for it is
legitimate on these terms, but read the definition twice before writing one: the
diagnostic can be wrong about the language and still right that the declaring module has
something true to say about the field, such as the property its readers exist to
maintain. Waive only what the reference plainly permits, and only when there is nothing
truthful to say instead.

Two more gaps are handled without a waiver at all. The checker sees a `.created(...)` call
only when it stands alone as an ensures statement, at 3.6.1 exactly as at 3.5.3: bind the
creation — `let entity = Entity.created(...)`, or assign it straight into a field — and
both the status it sets and every field it establishes vanish from the checker's status
scan and from the analyser's producer search, so create unbound and let a `.created` rule
pick the entity up. And 3.6.1 resolves `alias/config.field` without checking the name behind
the dot, so a mistyped cross-module config reference draws nothing and is read by eye; a
local one is reported as `allium.config.undefinedReference` from a derived value, a rule or
a module-level invariant, but not from inside an entity-level `invariant` block, which is
read by eye too.

## Related pages

- [Specifications](../explanation/specifications.md)
- [Quality gates](../reference/quality-gates.md)
