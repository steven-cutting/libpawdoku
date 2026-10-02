---
title: "Agent contract"
kind: "reference"
audience: [contributor, maintainer, agent]
canonical_for: [agent_contract]
requires: []
---

# Agent contract

Enforced by `bg-validate-agents`, a console script of the `biscuit-games-tooling`
package, run by `just check-agents` and by a pre-commit hook. `AGENTS.md` is the single
source of truth for how an agent works in this repository; everything else in the agent
surface exists only so a particular tool can find it.

## The four surfaces

| Path | Role |
| --- | --- |
| `AGENTS.md` | Canonical. Read natively by Codex, and by anything following the convention. |
| `CLAUDE.md` | A pointer, byte-pinned to `@AGENTS.md` and nothing else. |
| `.github/copilot-instructions.md` | A pointer, byte-pinned to one paragraph. |
| `.agents/skills/` | Canonical task procedures, mirrored by thin bridges under `.claude/skills/` and `.codex/skills/`. |

A `CODEX.md` at the repository root is forbidden: Codex reads `AGENTS.md` directly, and a
second file would be a second source of truth.

## What `AGENTS.md` must contain

- At least 300 words.
- Six phrases, each naming an invariant that is expensive to rediscover: `untrusted`,
  `just check`, `explicit authorization`, `ai_tmp/`, `docs/specs/`, and `runes`.

The phrase list is a crude check and is meant to be. It does not verify that the guidance
is good; it verifies that the six topics were not dropped in an edit.

The sixth phrase names the games' reactivity rule, which has nothing to govern in a Rust
library. `AGENTS.md` satisfies it with one honest sentence saying the rule does not apply
here, which stays until the tooling package makes the list configurable (decision 0004).

## What a skill must be

This library carries fourteen canonical skills, one directory each under
`.agents/skills/`, in two groups. Seven are the house's own: `review-change`,
`fix-quality`, `plan-change`, `project-check`, `review-docs`, `spec-change` and
`rust-change`. `review-change` is the games' `code-review` renamed, so that it does not
shadow Claude Code's built-in `/code-review`. Seven are vendored from `juxt/allium`,
with `skills-lock.json` recording their source: `allium`, `distill`, `elicit`,
`propagate`, `tend`, `weed` and `witness`. Five are byte for byte; `allium` and
`propagate` each carry one edited line that names this repository's recipes and test
layout. Each skill has two bridges, so there are twenty-eight.

Frontmatter of exactly two keys:

```markdown
---
name: rust-change
description: Implement or review a change to the engine with the trait boundary, no_std, tests and documentation evidence the invariants require.
---
```

- `name` equals the directory name.
- `description` is at least eight words and states a real trigger.
- The body cites `AGENTS.md` and names at least one `just` recipe. A procedure that ends
  without saying how to verify it is not a procedure.

The frontmatter parser is deliberately literal. It splits every line between the `---`
markers on the first colon and treats a line without one as an error, so a blank line
inside the block fails the check with `invalid frontmatter line`. Keep the two keys
adjacent, with nothing between them. The same parser reads the bridges, so the rule holds
there too.

A key that appears twice is an error rather than an overwrite. Left to the usual last-wins
behaviour, a block of three lines would satisfy a rule about two keys on whichever copy
happened to survive, which is the opposite of what "exactly two" is for.

## What a bridge must be

Each of `.claude/skills/<name>/SKILL.md` and `.codex/skills/<name>/SKILL.md` carries the
canonical frontmatter verbatim, then exactly this sentence and nothing else:

```markdown
Follow `../../../.agents/skills/<name>/SKILL.md`. That file is canonical and this bridge adds nothing to it.
```

The body is compared against that template rather than measured against a word budget. A
budget was the earlier rule and it enforced the wrong thing: a bridge could carry an
instruction of its own — an extra step, a caveat, a second pointer — and pass on being
brief. This is an instruction surface, so a clause smuggled into it is read as guidance and
becomes a second source of truth for the skill it points at. A bridge that grows content
fails whether or not it is short.

## The inventory

The validator lists managed files from Git, honouring only this repository's
`.gitignore`, and compares that against what it expects.

- Every expected file must exist. A skill without its two bridges fails.
- No unexpected file may exist under `.agents/`, `.claude/` or `.codex/`. The single
  exception is `.claude/settings.json`, which is tolerated but not required — it carries
  provider configuration rather than agent guidance.
- Managed files must be regular files, never symlinks.

Local assistant state stays out of the inventory by being listed in `.gitignore`. That is
deliberate: the check reads Git rather than walking the filesystem, so an ignored file is
invisible to it.

Because nothing else may exist under `.agents/`, the vendored skills' reference material
lives outside it, at `allium-skill-reference/` at the repository root. Git marks that
directory as vendored, and markdownlint, the link checker and `typos` all ignore it;
[Decision 0010](../decisions/0010-skill-reference-material.md) is the record. On this
tree `just check-agents` prints `Validated AGENTS.md, 2 adapters, and 14 skills.`

## Related pages

- [Documentation contract](documentation-contract.md)
- [Quality gates](quality-gates.md)
- [Decision 0010](../decisions/0010-skill-reference-material.md)
