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
