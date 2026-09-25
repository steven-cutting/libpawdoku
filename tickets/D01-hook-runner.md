---
id: D01
title: "Hook runner and checkers: keep the Python toolchain or go Python-free"
status: open
depends_on: []
parallel_with: []
branch: ticket/d01-hook-runner
estimated_size: M
---

# D01: Hook runner and checkers: keep the Python toolchain or go Python-free

## Context

Every Biscuit Games repository runs its quality gate the same way: `prek` (a Rust
reimplementation of pre-commit) runs the hook gate, and six console scripts from the
Python package `biscuit-games-tooling` (B, CONVENTIONS.md §0) do the work no hook does:
`bg-validate-docs` (the documentation contract), `bg-validate-agents` (the agent
contract), `bg-project-check` (the ordered gate with a worktree snapshot between
recipes), `bg-run-allium` and `bg-install-allium` (the specification gate and its pinned
binary), and `bg-ripsecrets` (a wrapper that keeps matched secrets out of logs). The
package is pinned as a git dependency in `pyproject.toml`, resolved by `uv`, and every
recipe that needs it runs `uv run --frozen <script>`. Pawdoku (G) ships no Python either;
its decision 0004 (`G/docs/decisions/0004-python-toolchain.md`) keeps the toolchain
anyway because the real lock-in is the checkers, not the runner.

This repository is a Rust library. The maintainer asked whether a Rust repository should
carry a `pyproject.toml` and a `.venv` for its hook gate, and asked for the question to be
answered as a decision with a record rather than assumed. This ticket is that decision.
It merges before T00, because T00 writes the `Justfile` in one of two forms and creates
either `pyproject.toml`, `uv.lock` and `.python-version` or a set of `scripts/` ports
(CONVENTIONS.md §3, §4, §11).

Facts the decision rests on, each verified in source (CONVENTIONS.md §1):

- B's `validate_agents.py` lines 47-54 require the literal word `runes` in `AGENTS.md`,
  and `_project.py` lines 44-50 read the package's configuration only from
  `pyproject.toml`. Keeping B unmodified means a `pyproject.toml` exists regardless and
  `AGENTS.md` carries the word until C02 ships.
- B's `validate_docs.py` and `run_project_check.py` are language-neutral.
- The Python bindings crate (`crates/pawdoku-py`, S04) will use maturin, which is a
  Python tool installed and run through `uv`. Whatever this ticket decides, uv returns to
  this repository the day the bindings crate lands.
- prek 0.5.3 is a Rust binary with GitHub release assets; it can be pinned in
  `tools.txt` and installed by cargo-binstall like every other tool (CONVENTIONS.md §12
  asks T03 to confirm the asset names resolve).
- The two contract validators have no non-Python implementation anywhere in the house.

Sources to read, at the pinned commits: `G/pyproject.toml` (the virtual project and the
`[tool.biscuit-games-tooling]` table), `G/docs/decisions/0004-python-toolchain.md`
(the argument this ticket either extends or overturns), `G/Justfile` (every `uv run
--frozen` site), `B/README.md` (the package section), `B/src/biscuit_games_tooling/`
(each script, to size a port), `B/pyproject.toml` lines 8-15 (the console scripts).

## Goal

A decision, recorded in this ticket's hand-back notes as the full text of decision
record 0004 (CONVENTIONS.md §8) and as the exact list of files T00, T03 and T04 create
under the chosen option, so that T00 can start. The decision is one of:

- **Option A: keep the Python toolchain.** `pyproject.toml` declares a virtual project
  (`package = false`) whose only purpose is to pin `prek==0.5.3` and
  `biscuit-games-tooling @ git+https://github.com/steven-cutting/biscuit_games_tooling@v0.3.0`;
  `uv.lock` and `.python-version` (`3.14`) beside it; no `ruff`, because no `.py` file
  ships (scripts are POSIX sh, checked by shellcheck). `[tool.biscuit-games-tooling]`
  carries the seventeen-recipe gate list of CONVENTIONS.md §4 and `predicates = {}`.
  `AGENTS.md` carries the honest `runes` sentence of CONVENTIONS.md §7 until C02 ships.
  The `Justfile` is CONVENTIONS.md §4 as written.
- **Option B: Python-free.** `prek@0.5.3` joins `tools.txt`; `scripts/project-check.sh`
  is a POSIX port of `run_project_check.py` (snapshot = `git status --porcelain=v1 -z
  --untracked-files=all` plus `git ls-files -z --cached --others --exclude-standard |
  xargs -0 git hash-object`, recipe list in an exported Justfile variable);
  `scripts/install_allium.sh` and `scripts/run_allium.sh` port B's installer (version
  3.6.1 and the checksums copied from `B/src/biscuit_games_tooling/install_allium.py`
  line 34 and lines 51-56, targets at 61-69) and JSON-reading runner; `scripts/ripsecrets_redacted.sh` is a six-line
  port. The two contract validators are either **dropped** (no `check-docs` contract
  half, no `check-agents`; the handbook and agent surface are review-only) or
  **rewritten**, which is a small Rust binary outside the workspace and a project of its
  own. The `Justfile` is CONVENTIONS.md §4 with the option-B substitutions it names.
- **Option C: hybrid.** No `pyproject.toml`; the scripts run through
  `uvx --from git+https://github.com/steven-cutting/biscuit_games_tooling@v0.3.0 bg-<name>`.
  This still needs uv on every machine and in CI, loses the lockfile and the
  `[tool.biscuit-games-tooling]` configuration surface (B reads it only from
  `pyproject.toml`), and so gets the recipe list nowhere. Listed for completeness;
  expected to lose to A or B.

The recommendation this ticket starts from is **option A**, with C02 filed: it costs
nothing to port, keeps the gate identical to every other house repository, and the
argument in G's decision 0004 applies unchanged, strengthened by the maturin fact. The
ticket exists so that the maintainer can overrule it with the trade-offs written down.

## Non-goals

- No file outside `tickets/D01-hook-runner.md` is created or changed. The decision
  record itself is written by T09 from this ticket's text; the files are created by T00,
  T03 and T04.
- No tool is installed. Sizing a port is done by reading B's source, not by running it.
- No change to B (that is C02, and only under option A).
- The bindings crate's own Python needs (S04) are evidence here, not decided here.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/D01-hook-runner.md` | ticket | the decision, the record text and the file list, in Hand-back notes; `status: done` |

## Steps

1. Create the worktree on `ticket/d01-hook-runner` from `main` (README.md "How to pick
   up a ticket"). There is nothing to run yet; `just` does not exist in this repository.

2. Read the sources named in Context at the pinned commits. Record, for each of B's six
   scripts, its line count and what it depends on (`_project.py`, `tomllib`, `json`,
   `subprocess`, `urllib`), so that the cost of a port is a number rather than a feeling.

3. Weigh the three options against these criteria, in a table in the hand-back notes:
   - **Fidelity to the house gate.** Does `just check` here mean what it means in G? Does
     a maintainer who knows G's `Justfile` know this one?
   - **Onboarding.** What a new contributor installs before `just initialize` works
     (CONVENTIONS.md §2 lists the Rust prerequisites; add uv under A and C).
   - **CI cost.** Steps in the composite action (CONVENTIONS.md §10 step 6 exists only
     under A), cache keys, minutes.
   - **What is lost.** Under B, the two validators unless rewritten; under C, the
     configuration surface and the lockfile.
   - **What comes back anyway.** maturin under S04, and with it uv.
   - **The `runes` constraint** and the C02 dependency it creates under A.
   - **Drift risk.** A port under B is a fork: every fix B makes to a checker is a
     re-port here.

4. Decide. Write the decision record text in the hand-back notes in the house shape
   (CONVENTIONS.md §8: frontmatter, `# Decision 0004: <title>`, Context, Decision,
   Consequences, What would reopen this, Related pages), using the slug
   `decision_python_toolchain` under A or `decision_hook_runner` under B or C. Under A,
   open with "Carried from Pawdoku's decision 0004 at `78d03cdf`" and say what this
   repository adds to the argument.

5. Write the file list for T00, T03 and T04 under the chosen option, as a table of
   exact paths, taken from CONVENTIONS.md §3 with the option-conditional rows resolved.
   Under A the list includes `pyproject.toml` (with its full content: the virtual
   project, the two pinned dependencies, `[tool.uv] package = false`, the
   `[tool.biscuit-games-tooling]` table with the seventeen recipes), `uv.lock` (generated
   by T00 with `uv lock`), `.python-version`. Under B it includes the four `scripts/`
   ports with a one-paragraph specification each and the `tools.txt` line for prek.

6. If the decision is B with the validators dropped, write the two handbook changes it
   forces (`docs/reference/documentation-contract.md` and `docs/reference/agent-contract.md`
   become descriptions of a convention rather than of a gate) and the AGENTS.md invariant
   it removes, as hand-back items for T07, T08 and T05. If B with a rewrite, write the
   rewrite as a new build ticket draft in the hand-back notes and note that T00 cannot
   reach a green `check-docs` until it lands, which puts it before T00 on the critical
   path.

7. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- The hand-back notes name exactly one option and give the reason in the maintainer's
  terms (fidelity, onboarding, CI cost, what is lost, what returns anyway).
- The full text of decision record 0004 is in the hand-back notes and follows
  CONVENTIONS.md §8's shape and slug rule.
- The file list for T00, T03 and T04 is exact: every option-conditional row of
  CONVENTIONS.md §3 is resolved to "create" or "omit", and every created file that has
  content this ticket decides (`pyproject.toml` under A; the four scripts under B) has
  that content or its specification written out.
- Under A, C02 is named as a dependency of T05's final state and the interim `runes`
  sentence is quoted.
- Under B, the fate of the two validators (dropped or rewritten) is decided and its
  consequences are listed as hand-back items with their owning tickets.
- Nothing outside this file changed: `git status --porcelain` on the ticket branch
  lists only `tickets/D01-hook-runner.md`.

## Verification

```sh
git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/validate_agents.py | sed -n 47,54p
git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/_project.py | sed -n 44,50p
for f in validate_docs validate_agents run_project_check run_allium install_allium run_ripsecrets_redacted; do
  printf '%s %s\n' "$f" "$(git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/$f.py | wc -l)"
done
git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/decisions/0004-python-toolchain.md | sed -n 9,30p
git status --porcelain
```

Expected: the first two commands print the `REQUIRED_GUIDANCE` tuple ending in `"runes"`
and the `settings()` function reading `pyproject.toml`; the loop prints six line counts
(quote them); the decision excerpt prints G's argument; `git status --porcelain` prints
one line naming this file.

## Hand-back notes

### What was verified, and how

### The comparison

### The decision, and decision record 0004

### Files T00, T03 and T04 create

### Handed back

### Open points settled

## Open points

- Whether B's maintainer (the same person) would rather port the checkers to Rust inside
  B, which would make option B free later. If so, A now and B later is the path, and the
  record should say so under "What would reopen this".
- Whether the bindings crate will live in this workspace (S04) or in its own repository;
  the maturin argument for A only holds in the first case.
