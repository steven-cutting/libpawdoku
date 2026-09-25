---
id: D01
title: "Hook runner and checkers: keep the Python toolchain or go Python-free"
status: done
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

Executed on 2026-09-24 on the branch `ticket/d01-hook-runner` (the Supacode worktree was
created as `D01-hook-runner` and renamed before the commit, so the `branch:` field is
true). Nothing was installed; no clone was modified; every source was read with
`git -C <clone> show <commit>:<path>` at the commits CONVENTIONS.md §0 pins.

### What was verified, and how

The Verification block, run from this worktree. Output, verbatim:

```text
$ git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/validate_agents.py | sed -n 47,54p
REQUIRED_GUIDANCE = (
    "untrusted",
    "just check",
    "explicit authorization",
    "ai_tmp/",
    "docs/specs/",
    "runes",
)
$ git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/_project.py | sed -n 44,50p
def settings(project_root: Path) -> dict[str, Any]:
    """The `[tool.biscuit-games-tooling]` table of the consumer's pyproject.toml, or {}."""
    manifest = project_root / "pyproject.toml"
    if not manifest.is_file():
        return {}
    with manifest.open("rb") as stream:
        return tomllib.load(stream).get("tool", {}).get("biscuit-games-tooling", {})
$ for f in validate_docs validate_agents run_project_check run_allium install_allium run_ripsecrets_redacted; do
    printf '%s %s\n' "$f" "$(git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/$f.py | wc -l)"
  done
validate_docs      332
validate_agents      260
run_project_check      151
run_allium      213
install_allium      238
run_ripsecrets_redacted       29
$ git -C /Users/scutting/projects/pawdoku show 78d03cdf:docs/decisions/0004-python-toolchain.md | sed -n 9,30p
# Decision 0004: A Python toolchain in a frontend repository

*Carried from Poodl's decision 0004 at `0a46a485`, and restated for a game rendered from the Biscuit Games template. Poodl's own record stands where it is.*

## Context

This game ships no Python. The hook gate it inherits runs on `prek` under `uv`, and the
documentation and agent contracts are enforced by two Python checkers. A frontend
repository could avoid Python entirely by moving the hook runner to a Node equivalent and
rewriting both validators in TypeScript.

## Decision

Keep the Python toolchain. `pyproject.toml` declares a virtual project — `package = false`
— whose dependencies are `prek`, `ruff` and `biscuit-games-tooling`, each pinned exactly
and locked in `uv.lock`. The first two are tools; the third is the package whose console
scripts are the checkers themselves, pinned to a release tag of
`steven-cutting/biscuit_games_tooling`.

Ruff is added on top of the inherited gate list so that Python this game adds is linted in
a repository that gates everything else. None ships today: the checkers moved into the
package, and `scripts/` holds the first-run script and the browser preflight.
$ git status --porcelain
 M tickets/D01-hook-runner.md
```

The comment above the tuple (`validate_agents.py` lines 43-46) says the last phrase names
"which reactivity model the components use", which is the game-shaped requirement
CONVENTIONS.md §1 fact 1 describes. `settings()` reads `pyproject.toml` and nothing else,
and returns `{}` when the file is absent, so under any option without a `pyproject.toml`
the recipe list and the predicates have nowhere to live (C02's second constraint).

**Sizing a port.** Each of B's six scripts at `6c5c07f6`, its line count, and what it
depends on. `_project.py` (64 lines; `subprocess`, `sys`, `tomllib`, `pathlib`; `root()` via
`git rev-parse --show-toplevel`, `settings()`, `recipes()`, `predicates()`) is shared by
five of them, and `tomllib` is the reason the configuration surface is `pyproject.toml`
and nothing else.

| Script | Lines | Standard library | In package |
| --- | --- | --- | --- |
| `validate_docs.py` | 332 | `json`, `re`, `sys`, `collections.deque`, `pathlib`, `urllib.parse.unquote` | `_project` (predicates) |
| `validate_agents.py` | 260 | `os`, `re`, `subprocess`, `sys`, `pathlib` | `_project` |
| `run_project_check.py` | 151 | `hashlib`, `json`, `os`, `subprocess`, `sys`, `tempfile`, `pathlib` | `_project` (recipes) |
| `run_allium.py` | 213 | `argparse`, `json`, `subprocess`, `sys`, `pathlib` | `_project`, `install_allium` (the pin has exactly one reader: `install_allium.check()`) |
| `install_allium.py` | 238 | `argparse`, `hashlib`, `platform`, `subprocess`, `sys`, `tarfile`, `tempfile`, `urllib.request`, `pathlib` | `_project` |
| `run_ripsecrets_redacted.py` | 29 | `shutil`, `subprocess`, `sys` | none |

Totals: 1,223 lines in the six scripts, 1,287 with `_project.py`. The two contract
validators are 592 of them; the four scripts option B names as POSIX ports are 631. The
docs validator parses `docs/manifest.yml` as strict JSON, walks links with a queue,
percent-decodes anchors and matches case exactly, none of which a POSIX shell does
without `jq` or `python3`, which is why the ticket lists it as dropped or rewritten in
Rust rather than ported. The Allium runner decodes back-to-back JSON objects and compares
the reported module set to the files on disk; a shell port needs `jq` for that too, which
would be an eighth pinned tool.

**prek 0.5.3 as a binstall target** (the D01 half of CONVENTIONS.md §12's prek claim).
Checked on 2026-09-24 with `gh release view v0.5.3 -R j178/prek --json assets`: the
release carries `prek-aarch64-apple-darwin.tar.gz`, `prek-x86_64-apple-darwin.tar.gz`,
`prek-aarch64-unknown-linux-gnu.tar.gz`, `prek-x86_64-unknown-linux-gnu.tar.gz`, the two
musl variants, two Windows zips, a `.sha256` beside each, `dist-manifest.json` and
`sha256.sum`. crates.io lists `prek` with max stable version 0.5.3 and repository
`j178/prek`; its `Cargo.toml` at `v0.5.3` carries no `[package.metadata.binstall]`
table, so cargo-binstall's default `{ name }-{ target }.tar.gz` template resolves each
asset by name. The claim holds: `prek@0.5.3` could sit in `tools.txt`. It is recorded here
as evidence that the runner was never the lock-in, and T03 step 8e is "not applicable"
under the option chosen.

**prek 0.5.3 on PyPI** (the option-A pin). Checked on 2026-09-24 with
`curl -s https://pypi.org/pypi/prek/0.5.3/json`: version `0.5.3`, nine files (eight
wheels and an sdist), so `prek==0.5.3` resolves for `uv lock`. B at `v0.3.0` requires
Python `>=3.14` and builds with `uv_build>=0.11,<0.12`, both satisfied by uv 0.11.18 and
Python 3.14.

**G's argument** (`docs/decisions/0004-python-toolchain.md` at `78d03cdf`): the value is
the two contracts coming from a working implementation shared by every game, and prek's
pinned hook ecosystem; what would reopen it is a Node-native runner or the validators
becoming trivial, and either "would have to replace the whole package, not only the two
validators".

**G's `pyproject.toml`** at `78d03cdf`: `[project] name = "pawdoku-tooling"`,
`requires-python = ">=3.14"`, `dependencies = []`; `[dependency-groups] dev` with
`biscuit-games-tooling @ git+...@v0.2.0`, `prek==0.4.12`, `ruff==0.16.2`; `[tool.uv]
package = false`, `default-groups = ["dev"]`; `[tool.ruff]` and `[tool.typos]` tables;
`[tool.biscuit-games-tooling]` with the eleven-recipe list and `predicates = {}`. The
`Justfile` has fifteen `uv run --frozen` sites (`install-hooks`, `install-allium`,
`format` twice, `fix` twice, `lint`, `check-docs` twice, `check-agents`, `check-specs`,
`analyse-specs`, `check-links-online`, `check-clean`, `check`).

### The comparison

| Criterion | A: keep the toolchain | B: Python-free, validators dropped | B: Python-free, validators rewritten | C: hybrid via `uvx` |
| --- | --- | --- | --- | --- |
| Fidelity to the house gate | `just check` means exactly what it means in G: the same runner, the same six scripts at a pinned tag, the same recipe list mechanism. A maintainer who knows G's `Justfile` knows this one. | The gate has fifteen recipes, not seventeen; `check-docs` is markdownlint, typos and lychee only; no `check-agents`. The runner is a fork with its own bugs. | Seventeen recipes, but through a binary nobody else in the house runs; its verdicts diverge from B's the first time either changes. | The same scripts, but no lockfile pins them and no configuration surface reaches them: `bg-project-check run` executes Poodl's eleven frontend recipes. |
| Onboarding | rustup, cargo-binstall, just, gh, uv (five). `just initialize` runs `uv lock` and `uv sync --frozen` once. | rustup, cargo-binstall, just, gh (four). One fewer tool; `.tools/bin` gains prek. | Four, plus the Rust checker either built from source in `initialize` or pinned as an eighth binary. | Five: uv is still required; it just has less to do. |
| CI cost | One `astral-sh/setup-uv` step with a cache keyed on `uv.lock`, `uv python install 3.14`, and `uv sync` inside `just sync`. About 20 to 30 seconds warm, one more action pin to maintain. | Neither step; prek comes through the `.tools` cache with the other binaries. | As B-dropped, plus a build or download of the checker. | The `setup-uv` step stays; each `uvx` call resolves the git dependency, so a cache keyed on nothing reliable. |
| What is lost | Nothing. | The documentation contract and the agent contract as gates: manifest-frontmatter equality, reachability, the 40-word floor, the adapter byte pins, the bridge bodies, the six phrases. They become conventions a reviewer checks by eye. | Nothing in the end state; T00 cannot be green until the rewrite lands, so it goes before T00 on the critical path. | `uv.lock` and `[tool.biscuit-games-tooling]`; the recipe list has nowhere to live without a B change first. |
| What comes back anyway | uv is already here. `crates/pawdoku-py` (S04, in this workspace per the maintainer) uses maturin through uv; a second `pyproject.toml` beside the crate joins the root virtual project as a uv workspace member or is excluded, S04's question (b). | uv returns the day S04 lands, and with it the question this ticket answers, so option B buys a Python-free window of a few tickets. | As B-dropped. | uv never left. |
| The `runes` constraint | `AGENTS.md` carries one honest sentence until C02 ships a MINOR release of B that makes the list configurable; the pin bump after T05 removes it. C02 is a dependency of T05's final state. | Never arises. | Never arises; the rewrite chooses its own phrases. | As A, since the same scripts run. |
| Drift risk | None: a fix in B is a moved pin here, and `uv lock --check` proves the pin. | Four ported scripts (631 lines' worth of behaviour) are a fork; every fix B makes to the runner, the installer or the Allium reader is a re-port. | Four ports plus a 592-line checker rewrite, all forks. | None for the scripts; the recipe list and predicates live nowhere, which is a different drift. |

In the maintainer's terms: A costs nothing to port and nothing to explain beyond one
extra prerequisite that returns with the bindings crate regardless. B saves one tool on
a fresh machine and one CI step, and pays for it with either the two contracts or a
checker project of its own, plus four forks to keep in step with B. C is A with the
lockfile and the configuration surface removed, which is strictly worse than A. The
prek asset check shows the runner could leave Python today; it does not show the
checkers could, and the checkers are the gate.

### The decision, and decision record 0004

**Option A: keep the Python toolchain.** It keeps `just check` identical to every other
house repository (fidelity), adds one prerequisite that the Python bindings crate brings
back anyway (onboarding, what returns), costs one cached CI step (CI cost), and loses
nothing (what is lost); the alternative loses the two contracts or spends a project
rewriting 592 lines of checker to reach the same verdicts, and forks four more scripts to
drift. C02 is filed and hangs off this decision.

Decision record 0004, the full text T09 writes to
`docs/decisions/0004-hook-runner-and-checkers.md`. It is in a fenced block so the two
relative links in Related pages are verbatim text here, not links the offline lychee run
over `tickets/` would follow.

```markdown
---
title: "Decision 0004: Hook runner and checkers"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_python_toolchain]
requires: []
---

# Decision 0004: Hook runner and checkers

*Carried from Pawdoku's decision 0004 at `78d03cdf`, and restated for the library the game's engine moved to. Pawdoku's own record stands where it is. This repository adds the Rust facts: the hook runner could leave Python, the checkers cannot, and the bindings crate brings Python back regardless.*

## Context

This library ships no Python. Its quality gate is the one every Biscuit Games repository
runs: `prek` runs the hooks, and six console scripts from the package
`biscuit-games-tooling` do the work no hook does: the documentation contract, the agent
contract, the ordered gate with a worktree snapshot between recipes, the pinned Allium
binary and its JSON-reading runner, and a ripsecrets wrapper that keeps matched secrets
out of logs. The package is a git dependency in `pyproject.toml`, resolved by `uv`, and
every recipe that needs it runs `uv run --frozen`.

A Rust repository could go Python-free. `prek` 0.5.3 is a Rust binary whose release
assets cargo-binstall resolves by name, so it could sit in `tools.txt` beside nextest
and taplo. The gate runner, the Allium installer and runner, and the ripsecrets wrapper
are 631 lines of Python that a POSIX shell could port, with `jq` for the JSON. The two
contract validators are 592 lines with no implementation in any other language anywhere
in the house; a Rust rewrite would be a project of its own, and it would have to land
before the first commit of this repository could pass its own gate.

Two facts are new here. The agent contract's checker requires the literal word `runes`
in `AGENTS.md`, because the games are Svelte and the phrase names their reactivity rule;
a Rust library has no such rule. And the Python bindings crate `crates/pawdoku-py`,
which this workspace will hold, is built by maturin, a Python tool installed and run
through `uv`. Whatever this record decided, `uv` would return on the day that crate
lands.

## Decision

Keep the Python toolchain. `pyproject.toml` declares a virtual project (`package = false`)
whose only purpose is to pin `prek==0.5.3` and `biscuit-games-tooling` at the release tag
`v0.3.0`, locked in `uv.lock` beside `.python-version` (`3.14`). No `ruff`: no `.py` file
ships, and the shell scripts are checked by shellcheck. `[tool.biscuit-games-tooling]`
lists the seventeen recipes `just check` runs, in order, and declares no predicates.
The hook runner and the Allium pin stay where the package puts them.

Until the package ships a release in which the required phrases are configurable,
`AGENTS.md` carries one honest sentence in its Provenance section: "This repository has
no Svelte runes; the games' reactivity rule does not apply here." The ticket that makes
the list configurable is filed; the pin bump that follows it removes the sentence.

## Consequences

Contributors need `uv` as well as rustup, cargo-binstall, just and gh, and
`just initialize` runs it to lock and install the two dependencies. CI carries one
`setup-uv` step with a cache keyed on `uv.lock`, and `.venv/` and `uv.lock` appear in
every checker's exclusion list. Neither the crate nor anything it publishes contains
any Python.

The gate is the same gate every other house repository runs, from the same release of
the same package. `just check` here means what it means in Pawdoku, and a fix to a
checker arrives as a moved pin rather than as an edit merged into a fork. The
documentation and agent contracts are enforced mechanically from the first commit,
which is what lets the foundation ticket ship a shape-complete skeleton and every later
lane replace stubs under a green gate.

The cost is one toolchain that a Rust maintainer would not otherwise install, and one
interim sentence in `AGENTS.md` that exists only to satisfy a checker written for a
different stack. Both are accepted deliberately, and the second is temporary.

## What would reopen this

The checkers being rewritten in Rust inside `biscuit-games-tooling` itself, so that every
consumer could take them as binaries; there is no plan to do that today, so this record
is the steady state and not a stopgap. The two validators becoming simple enough that a
rewrite is cheaper than an interpreter. The bindings crates moving to repositories of
their own, which would remove the maturin argument and leave the checkers as the only
reason. Any of these would have to replace the whole package, because the gate runner,
the Allium installer and runner and the ripsecrets wrapper come from it as well; the
runner alone leaving Python is already possible and changes nothing.

## Related pages

- [Quality gates](../reference/quality-gates.md)
- [Maintain dependencies](../how-to/maintain-dependencies.md)
```

### Files T00, T03 and T04 create

Every option-conditional path in CONVENTIONS.md §3, resolved. Unconditional paths are
unchanged and not repeated.

| Path | Owner | Under option A |
| --- | --- | --- |
| `pyproject.toml` | T00 -> T03 | **create**, content below; T03 adds the two comments its step 6 names and changes no key |
| `uv.lock` | T00 -> T03 | **create** with `uv lock` after `pyproject.toml` exists (network); committed; `linguist-generated` |
| `.python-version` | T00 -> T03 | **create**, exactly `3.14` and a newline; kept (see Handed back, T03) |
| `scripts/project-check.sh` | T03 | **omit** |
| `scripts/install_allium.sh` | T03 | **omit** |
| `scripts/run_allium.sh` | T03 | **omit** |
| `scripts/ripsecrets_redacted.sh` | T03 | **omit** |
| `scripts/initialize.sh`, `scripts/install_tools.sh` | T00 -> T03, T03 | create, unconditional; `initialize.sh` keeps G's `uv lock` and `uv sync --frozen` lines and G's `install-allium` comment verbatim |

The option-conditional lines elsewhere in CONVENTIONS.md, each resolved to its option-A
text:

| Where | Resolved |
| --- | --- |
| §2 `tools.txt` | six lines as printed; **no** `prek@0.5.3` line |
| §2 prerequisites | uv 0.11.18 is a maintainer prerequisite; T07's `develop-locally.md` lists it |
| §4 `Justfile` | the text exactly as §4 prints it (the `uv run --frozen` form, `uv` in `sync`, `lock`, `lock-upgrade`, `lock-check`); the recipe list lives in `pyproject.toml`, not in an exported variable |
| §5 `.pre-commit-config.yaml` `exclude` | includes `\.venv/` and `uv\.lock$` |
| §5 ripsecrets hook | `entry: uv run --frozen bg-ripsecrets` |
| §5 `check-specs`, `analyse-specs` `files` | the option-A trigger, written out below this table because it contains a pipe (the Allium pin is the package version, which `pyproject.toml` pins) |
| §5 `validate-docs`, `validate-agents` entries | `uv run --frozen bg-validate-docs`, `uv run --frozen bg-validate-agents` |
| §5 `.editorconfig` | `[*.py]` block stays |
| §5 `.gitattributes` | `uv.lock linguist-generated=true` stays |
| §5 `.gitignore` | `.venv/`, `__pycache__/`, `*.py[cod]`, `.ruff_cache/` stay |
| §5 `.markdownlint-cli2.jsonc`, `lychee.toml` | `.venv` in `ignores` and `exclude_path` |
| §5 `_typos.toml` | `uv.lock` in `extend-exclude`; `pyproject.toml` carries no `[tool.typos]` |
| §7 `AGENTS.md` | the interim sentence in Provenance, quoted under Handed back |
| §8 record 0004 | slug `decision_python_toolchain`; title `Hook runner and checkers`; the file name `0004-hook-runner-and-checkers.md` |
| §10 composite action | `astral-sh/setup-uv` (`version: 0.11.18`, cache on `uv.lock`) then `uv python install 3.14`, before `just sync`; T04's step 3 text is the option-A text |
| §12 prek binstall half | verified above; T03 step 8e records "not applicable: prek is a uv dependency" |

The `files` trigger for the `check-specs` and `analyse-specs` hooks, exactly:

```text
^(docs/specs/|pyproject\.toml$)
```

`pyproject.toml`, the content T00 writes. G's shape at `78d03cdf` without the `[tool.ruff]`
and `[tool.typos]` tables (no Python ships; typos is configured in `_typos.toml` only) and
without `ruff` in the group. No comments: T03 step 6 adds the two it names, so a comment
here would be duplicated there.

```toml
[project]
name = "libpawdoku-tooling"
version = "0.1.0"
description = "Repository tooling for libpawdoku. Not the library; see Cargo.toml for that."
requires-python = ">=3.14"
dependencies = []

[dependency-groups]
dev = [
  "biscuit-games-tooling @ git+https://github.com/steven-cutting/biscuit_games_tooling@v0.3.0",
  "prek==0.5.3",
]

[tool.uv]
package = false
default-groups = ["dev"]

[tool.biscuit-games-tooling]
recipes = [
  "check-toolchain", "lock-check", "lint", "fmt-check", "toml-check", "clippy",
  "features", "wasm-check", "test-doc", "coverage", "doc", "deny", "deps-unused",
  "check-docs", "check-agents", "check-specs", "analyse-specs",
]
predicates = {}
```

`.python-version` is the single line `3.14`. `uv.lock` is generated, never hand-edited;
its first `uv lock` needs GitHub reachable for the git dependency, and `uv lock --check`
in `lock-check` proves it afterwards.

Two things T00 should know before running `uv lock`: B's own `pyproject.toml` at `v0.3.0`
requires Python `>=3.14` and builds with `uv_build`, so `uv` 0.11.18 resolves it without
a build backend install; and `default-groups = ["dev"]` is what lets `uv run --frozen
prek` find prek without `--group dev` on every call, which is how G's recipes are
written.

### Handed back

- **T00.** Build option A: the `Justfile` exactly as CONVENTIONS.md §4 prints it; the
  `pyproject.toml` above, `.python-version`, then `uv lock` and `uv sync --frozen`; no
  `scripts/` ports; `tools.txt` without prek. Write the manifest row for 0004 as
  `Decision 0004: Hook runner and checkers`, kind `decision`, audience
  `[maintainer, agent]`, slug `decision_python_toolchain` (T09's open point: the manifest
  is frozen at T00, so this row must be right the first time). `AGENTS.md`'s stub carries
  the interim sentence so `check-agents` is green from the first commit.
- **T03.** Keep `.python-version` (its open point): `requires-python` states a floor and
  uv would resolve the newest interpreter it finds, while the file pins the series so two
  machines and CI resolve the same one; T04's `uv python install 3.14` reads the same
  value. Record the reason in T07's `develop-locally.md`. Step 6 (the two `pyproject.toml`
  comments) applies; step 7 (the four ports) does not; step 8e records "not applicable:
  prek is a uv dependency", with the asset check above as the evidence it would have
  gathered.
- **T04.** The composite action keeps the `setup-uv` and `uv python install 3.14` steps
  and `just sync`; `uv.lock` is the cache key; the `documents` job needs
  `just install-allium` as §10 says.
- **T05.** `AGENTS.md`'s Provenance ends with the sentence, exactly: "This repository has
  no Svelte runes; the games' reactivity rule does not apply here." `runes` occurs nowhere
  else. C02 is a dependency of T05's final state: the follow-up pull request that bumps
  the B pin and adds `agent_guidance_drop = ["runes"]` removes the sentence, after T05
  has merged.
- **T09.** Copy the record above verbatim into
  `docs/decisions/0004-hook-runner-and-checkers.md`; it is a carried record, so the
  line-11 opener and the `decision_python_toolchain` slug are already in the text.
- **C02.** Proceeds: option A is chosen, the interim sentence exists, and the pin bump is
  owed after T05.
- **S04.** Both of its open points that touch this ticket are settled: the bindings
  crates live in this workspace, and uv is already present, so S04's question (b) (how uv
  treats `crates/pawdoku-py/pyproject.toml` beside the root virtual project) is a real
  question to answer, not a hypothetical.
- **T07, T08.** No handbook change beyond what CONVENTIONS.md already assigns: `uv` in the
  prerequisites of `develop-locally.md`, the uv lines in `commands.md` and
  `maintain-dependencies.md`, and the contracts described as gates.

### Open points settled

Both open points were put to the maintainer on 2026-09-24 and answered:

- **A Rust port of the checkers inside B.** No plan to. The record therefore treats
  option A as the steady state and lists a Rust B as a hypothetical reopener, not as the
  second half of "A now, B later".
- **Where the bindings crates live.** In this workspace, as CONVENTIONS.md §1 decision 1
  and S04 assume. The maturin argument for A holds and the record uses it.

Also settled: the option itself (A, confirmed by the maintainer against the comparison
above), and the branch name (renamed from the worktree's `D01-hook-runner` to
`ticket/d01-hook-runner` before the commit).

## Open points

- Whether B's maintainer (the same person) would rather port the checkers to Rust inside
  B, which would make option B free later. Answered above: no plan to; the record says A
  is the steady state.
- Whether the bindings crate will live in this workspace (S04) or in its own repository;
  the maturin argument for A only holds in the first case. Answered above: this
  workspace.
