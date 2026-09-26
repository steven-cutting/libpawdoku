---
title: "Decision 0010: Skill reference material beside the skills"
kind: "decision"
audience: [maintainer, agent]
canonical_for: [decision_skill_reference]
requires: []
---

# Decision 0010: Skill reference material beside the skills

## Context

The seven Allium skills are vendored byte-for-byte from `juxt/allium`, with hashes
recorded in `skills-lock.json`. Each `SKILL.md` cites reference material of its own: a
language reference, patterns, worked examples and JSON schemas, several files per skill,
which upstream ships beside the skill.

The agent contract's checker does not allow that. `validate_agents.py` expects exactly one
`SKILL.md` per skill under `.agents/skills/` and one bridge per provider under `.claude/`
and `.codex/`, counts every versioned path under those three directories as managed, and
fails on any path it did not expect with `unexpected managed file`. A `references/`
directory beside a `SKILL.md` is a gate failure, however faithful the vendoring.

Pawdoku met the same wall first. The commit that installed its skills moved the reference
material to `allium-skill-reference/` at the repository root, called the relocation "a
template departure worth a decision record", and did not write one. This is that record.

## Decision

The reference material lives at `allium-skill-reference/` at the repository root, one
directory per skill, vendored byte-for-byte with the skills. `skills-lock.json` hashes
only each `SKILL.md`, so the reference is held to upstream by the vendoring, not by the
lock. The skills keep their upstream relative links, rewritten only in
depth: `../../../allium-skill-reference/<skill>/` from `.agents/skills/<skill>/SKILL.md`.
The directory is excluded from markdownlint, lychee and typos, and marked
`linguist-vendored` in `.gitattributes`, because it is upstream's text and not this
repository's to lint: a finding there is fixed upstream and arrives as a moved pin, not as
an edit here.

## Consequences

A root directory that is not in the house shape and has to be explained on the repository
map. An upstream typo or dead link lands here unflagged, because the three checkers that
would catch it are told to look away. Moving a skill pin means moving both the skill and
its reference in one change, or living with a skill that cites a reference from a
different version. The agent contract page carries the rule, so a contributor adding a
skill knows where its material goes.

## What would reopen this

The tooling package's validator tolerating a `references/` subdirectory beside a
`SKILL.md`, which is a change to the agent contract itself; or the Allium plugin shipping
the reference material, so that no repository has to vendor it. On either day the
directory is deleted, the links move, and this record is superseded.

## Related pages

- [Agent contract](../reference/agent-contract.md)
- [Repository map](../project/repository-map.md)
