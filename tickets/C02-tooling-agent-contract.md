---
id: C02
title: "biscuit_games_tooling: a configurable agent contract"
status: open
depends_on: [D01]
parallel_with: []
branch: ticket/c02-tooling-agent-contract
estimated_size: S
---

# C02: biscuit_games_tooling: a configurable agent contract

## Context

B's agent checker is game-shaped (CONVENTIONS.md §1 fact 1). At `6c5c07f6`,
`src/biscuit_games_tooling/validate_agents.py` lines 47-54 define `REQUIRED_GUIDANCE`
as a fixed tuple of six phrases `AGENTS.md` must carry (`untrusted`, `just check`,
`explicit authorization`, `ai_tmp/`, `docs/specs/`, `runes`), and the comment above it
(lines 43-46) says the last names "which reactivity model the components use". A Rust
library has no components, so this repository's `AGENTS.md` carries
one honest sentence containing the word ("this repository has no Svelte runes; the
games' reactivity rule does not apply here", §7) until B ships a release in which the
list is configurable. The second constraint is `_project.py` lines 44-50: `settings()`
reads `[tool.biscuit-games-tooling]` from `pyproject.toml` alone and returns `{}` when
the file is missing, so a repository with no `pyproject.toml` (any future Python-free
consumer) has no configuration surface. `recipes()` (lines 53-54)
and `predicates()` (lines 57-64) are the two existing keys and the pattern a third
follows: read from `settings()`, default to today's behaviour.

B's versioning rules (`README.md` lines 274-285) decide the release: "A check added
behind a configuration key whose default keeps today's verdicts" and "a configuration
key added, with today's behaviour as the default" are Minor (lines 281-282); "a phrase
added to `REQUIRED_GUIDANCE`" is Major (line 278). Making the list configurable with
the six phrases as the default is therefore MINOR: every existing consumer's verdict
is unchanged, the golden test (README lines 234-248) stays green without re-recording,
and `tests/test_project.py` is where the new keys get their cases.

This ticket proceeds: D01 kept the Python checkers and the interim sentence exists
(D01's hand-back notes, "Handed back", C02). It hangs off D01 alone, but its last step,
the pin bump here, cannot land before T05 has written the final `AGENTS.md`, because
the interim sentence is T05's to remove.

Read first: CONVENTIONS.md §1 fact 1, §7, §11 (every edit to B is separately
authorised), §13 ("`runes`"); B's `validate_agents.py` lines 40-60 and `_project.py`
lines 40-70 at `6c5c07f6`; B's `README.md` lines 201-210 (Configuration) and 250-291
(Versions); B's `tests/test_project.py`; D01's hand-back notes; T05's `AGENTS.md` if
T05 has merged.

## Goal

A B release in which the phrases `bg-validate-agents` requires come from configuration
with today's six as the default, and a follow-up here that bumps the pin and drops the
interim sentence. Recommendation: two keys under `[tool.biscuit-games-tooling]`,
`agent_guidance_drop` (phrases removed from the default set) and
`agent_guidance_extra` (phrases added), so a consumer states what differs rather than
restating the list; and, for a consumer with no `pyproject.toml`, the same table read
from `biscuit-games-tooling.toml` at the repository root.

| Option | Consumer writes | Default keeps verdicts | Release level | Cost in B |
| --- | --- | --- | --- | --- |
| Drop and extra lists | `agent_guidance_drop = ["runes"]` | yes | MINOR | two keys in `_project.py`, one call in `validate_agents.py`, tests |
| A full replacement list | all six minus one, restated | yes, when absent | MINOR | one key; a consumer silently loses a phrase B adds later |
| A per-language default set | `agent_language = "rust"` | yes, when absent | MINOR now, MAJOR each time a set changes | B curates lists for languages it does not build |
| Leave it | the interim sentence forever | yes | none | none; the sentence exists only to satisfy the validator |

## Non-goals

- Any change to the docs validator, the runner or the allium scripts.
- Dropping `runes` from B's default: MAJOR for B and wrong for the games, which use
  runes.
- Editing this repository's `AGENTS.md` on this branch: the follow-up pull request
  that bumps the pin removes the sentence, after T05.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/C02-tooling-agent-contract.md` | ticket | the design, the release, the follow-up; `status: done` |
| `B/src/biscuit_games_tooling/_project.py` | other repository | `settings()` falls back to `biscuit-games-tooling.toml`; `agent_guidance()` returns the effective tuple; authorisation required |
| `B/src/biscuit_games_tooling/validate_agents.py` | other repository | the tuple becomes the default; the check reads `agent_guidance(root)`; authorisation required |
| `B/tests/test_project.py` | other repository | cases for both keys and the fallback file; authorisation required |
| `B/README.md`, `B/CHANGELOG.md`, `B/pyproject.toml` | other repository | two configuration rows, the fallback paragraph, the MINOR entry and version; authorisation required |
| `pyproject.toml`, `pixi.lock`, `AGENTS.md`, `docs/reference/agent-contract.md`, `tickets/T05-agent-contract.md` | follow-up here, after T05 | the pin to the new tag, the relock, the sentence removed, `agent_guidance_drop = ["runes"]` added; the reference page's phrase list goes from six entries to five and T05's third verification block expects no `runes` line |

## Steps

1. Create the worktree on `ticket/c02-tooling-agent-contract` from `main` after D01 has
   merged (README.md "How to pick up a ticket").

2. Confirm the source lines and the tag list:

   ```sh
   git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/validate_agents.py | sed -n 43,54p
   git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/_project.py | sed -n 44,64p
   git -C /Users/scutting/projects/biscuit_games_tooling tag -l
   ```

   If B has moved past `6c5c07f6`, read the current lines and note the next tag
   (`v0.4.0` if nothing MINOR or MAJOR shipped since `v0.3.0`; C01 may take it first).

3. Write the design in the hand-back notes as the exact diff: in `_project.py`, a
   `_manifest(project_root)` returning `pyproject.toml` if present else
   `biscuit-games-tooling.toml` (its table at the top level rather than under
   `[tool]`), and `agent_guidance(project_root)` returning the default tuple minus
   `agent_guidance_drop` plus `agent_guidance_extra`, order preserved, an unknown name
   in `drop` exiting 2; in `validate_agents.py`, the tuple renamed `DEFAULT_GUIDANCE`
   and the check calling `agent_guidance(root)`; in `tests/test_project.py`, a
   temporary directory per manifest shape with drop, extra, both, neither, unknown.

4. **Authorisation required.** In B, on a branch: the diff of step 3, the README rows
   in the Configuration table (lines 206-209's shape) and a paragraph on the fallback
   file, the CHANGELOG entry as MINOR, the version in `pyproject.toml`. `just check`
   and `just test-network` green in B, the golden cases unchanged (no
   `BG_GOLDEN_RECORD`). Pull request, merge, annotated tag.

5. **Authorisation required, after T05 has merged.** A follow-up pull request on
   `main` here (not this branch): the tag in `pyproject.toml`'s
   `[tool.pixi.pypi-dependencies]` moved to the new tag and
   `agent_guidance_drop = ["runes"]` added under `[tool.biscuit-games-tooling]`;
   `pixi update biscuit-games-tooling` and the committed `pixi.lock`; the interim
   sentence removed
   from `AGENTS.md`'s Provenance; `just check-agents` printing
   `Validated AGENTS.md, 2 adapters, and 14 skills.` and `just check` green. Draft
   that pull request's description in the hand-back notes.

6. Set `status: done` and commit on the ticket branch. Stop before pushing.

## Acceptance criteria

- The hand-back notes hold the exact B diff, the test cases and the release level with
  the README line that justifies it.
- If applied: a B tag exists whose `bg-validate-agents` accepts an `AGENTS.md` without
  `runes` when `agent_guidance_drop = ["runes"]` is set and still rejects one under
  the default; B's golden tests pass without re-recording.
- If applied here: `AGENTS.md` contains no `runes`, `pyproject.toml` names the new
  tag, `pixi.lock` records its commit, `just check-agents` prints the fourteen-skill line.
- Every action on B was authorised before it was taken; the follow-up waited for T05.

## Verification

```sh
git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:src/biscuit_games_tooling/validate_agents.py | sed -n 47,54p
git -C /Users/scutting/projects/biscuit_games_tooling show 6c5c07f6:README.md | sed -n 278,282p
grep -n 'runes' AGENTS.md || echo "no runes"
grep -n 'tag = ' pyproject.toml
git status --porcelain
```

Expected: the six-phrase tuple; the Major and Minor rows; the interim sentence (before
the follow-up) or `no runes` (after); the pinned tag; one line naming this file.

## Hand-back notes

### What was verified, and how

### Deviations, and why

### Handed back

### Open points settled

## Open points

- Whether B's maintainer prefers a per-language default set after all, so a Rust
  consumer writes one key rather than knowing which phrase to drop; the drop list is
  recommended because B should not curate phrases for a language it does not build,
  but the two are not exclusive.
- Whether Rust-specific phrases join this repository's `agent_guidance_extra`
  (`no_std`, `--locked`, `randomness boundary`) so the validator holds the Rust
  invariants as it holds the games'; a T05 question, raised in its open points.
