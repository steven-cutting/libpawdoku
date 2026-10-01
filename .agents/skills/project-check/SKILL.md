---
name: project-check
description: Bring a workspace to a state where the full gate runs, and interpret what it reports.
---

# Run the full gate

1. Read `AGENTS.md`. The `Justfile` is the only supported interface to the checks; do not assemble an equivalent pipeline by hand.
2. Check the prerequisites exist: rustup's cargo first on `PATH` (`just check-toolchain` says so), the pixi environment at `.pixi/envs/default/bin` (nextest, llvm-cov, deny, shear, taplo, prek, just, the `bg-*` scripts), `.tools/bin/cargo-hack`, `.tools/bin/rustqual` (built from source by `just install-tools`), the pinned checker at `.tools/bin/allium`, `Cargo.lock` and `pixi.lock`. If any is missing, `just initialize` creates them — it normalises formatting and installs the hook, and it never stages, commits, tags or pushes — but it downloads, so it is a network operation and needs explicit authorization before it runs, unless the task already grants it (a ticket's Prepare step does). The checker alone is `just install-allium` and rustqual alone is `just install-tools`, under the same rule; both are gitignored and per-worktree, so a fresh worktree needs the checker before gates 3, 17 and 18 can pass, and rustqual before gate 7 can.
3. Run `just check`. It runs each recipe in order and snapshots the worktree between them.
4. Read only the first failure. The gates are ordered so that a later failure is often a consequence of an earlier one.
5. A report that a recipe changed the worktree is a defect in that recipe, not in the change under test. Checks are read-only; `just fix` is where mutation belongs.
6. Hand a failing gate to the `fix-quality` skill rather than working around it.
7. Confirm with `just check-clean` that the worktree is unchanged before reporting success.
