---
name: fix-quality
description: Diagnose and repair a failing quality gate at its root instead of suppressing the finding.
---

# Fix a failing gate

1. Read `AGENTS.md`, then reproduce with the narrowest recipe rather than the whole gate. `just check` names the recipe that failed; run that one alone.
2. Dispatch on which gate failed:
   - `fmt-check` or `toml-check` — run `just fix`; never hand-format to match.
   - `clippy` — fix the code. A lint that is wrong for one item gets `#[expect(lint, reason = "...")]` on that item; a lint that is wrong for the crate is changed in the workspace lint table in `Cargo.toml` with a comment saying why, never disabled at the call site with `allow`.
   - `features` or `wasm-check` — a `std` path, or a feature that does not compile alone. The fix is `cfg`, `core` and `alloc`; a target is never dropped.
   - `coverage` below the floor — add the missing test. Never lower `coverage_floor` in the `Justfile`.
   - `doc` — a missing doc comment or a broken intra-doc link; `missing_docs` is on and warnings are errors there.
   - `deny` or `deps-unused` — a licence, a ban or an unused dependency; a new dependency is a decision recorded per decision 0007, and `just fix` runs `cargo shear --fix`.
   - `check-docs` — a frontmatter list that disagrees with `docs/manifest.yml` is usually list order; the comparison is order-sensitive.
   - `check-agents` — a bridge under `.claude/` or `.codex/` has grown content, or a managed file is missing from the inventory.
   - `check-specs` or `analyse-specs` — hand to the `spec-change` skill; a diagnostic is a regression and a finding cannot be waived.
   - `lock-check` — run `just lock`, and read the lockfile diff before accepting it.
3. Distinguish an unreachable branch from an untested one. Defensive code no input can reach should be removed, not covered by a contrived test.
4. Never disable a gate to make a run green. A suppression is a last resort: one rule, one line, with a stated reason.
5. If a recipe changed the worktree, that is a defect in the recipe — checks are read-only. Fix the recipe.
6. Run the narrow recipe, then `just check` to confirm nothing else moved.
