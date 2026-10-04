---
title: "Decision 0016: The API reference on GitHub Pages"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_api_reference_on_pages]
requires: []
---

# Decision 0016: The API reference on GitHub Pages

## Context

The API reference is the one rustdoc generates, and until now reading it took a clone
and `just doc`. Every crate is `publish = false`, so docs.rs holds nothing. The
repository is public and had no Pages site, and none of its workflows held a scope
beyond `contents: read`: the gate and the advisory audit only read.

The game this engine serves already deploys to Pages, through a workflow of its own
whose build job holds no publishing scope and whose deploy job holds two. Its build is a
frontend's and cannot be called from here.

The maintainer settled the shape on 2026-10-02.

## Decision

**The site is rustdoc alone, from `main`, on every push, as a project site** at
`https://steven-cutting.github.io/libpawdoku/`. There is no site generator, and the
handbook under `docs/` stays where it is, read on GitHub.

**It is built by `just site`**, which empties `target/doc`, runs `just doc` with the
gate's own flags, and adds a root `index.html` that sends a reader to the crate, because
stable rustdoc writes none. No second build of the documentation exists: what is hosted
is what gate 12 proved.

**The deploy has a workflow of its own**, `.github/workflows/pages.yml`. Its build job
holds `contents: read`. Its deploy job holds `pages: write` and `id-token: write`,
checks nothing out and runs no code from the repository.

**The push is the authorisation.** `AGENTS.md` lists deploying among the actions that
need explicit authorisation each time, and pushing beside it. A deployment here follows
from a push to `main`, whether typed or made by GitHub when a pull request merges.
Pushing is on that list already, and merging a pull request is the maintainer's own
act, so no action an agent takes deploys on its own. Starting the workflow by hand from
`main` is a deployment and is authorised as one.

The workflow leaves the game's in three places, for one reason: this one can be started
by hand from a branch that must not publish.

- The deploy job carries `if: github.ref == 'refs/heads/main'`. A setting can be changed
  where no reviewer sees it; a line in the workflow cannot.
- The concurrency group carries the ref. A group keeps one run waiting and a newer one
  takes its place, so one group for every ref would let a run from another branch, which
  deploys nothing, displace a waiting deployment of `main`.
- The `github-pages` environment's branch rule is checked by hand before the first
  deployment and admits `main` alone, as
  [Deploy to GitHub Pages](../how-to/deploy-to-github-pages.md) records.

Turned down:

- **A site generator for the handbook.** A second tool and a second build, for a
  handbook whose frontmatter and manifest are made for GitHub's rendering.
- **Publishing on release only.** There would be nothing to read until the first
  release.
- **A tree per version.** docs.rs does that for releases once the crate is published.

## Consequences

The ones that hurt.

- **The first workflow that holds a write scope.** Every sentence that said the
  workflows only read had to be reworded, and the security model now has a job to
  describe rather than an absence.
- **The site describes `main` and says so nowhere on its pages.** Rustdoc prints the
  crate's version and nothing about the branch, so a reader may find an item no release
  has. Only the handbook says it.
- **The reference will be in two places** once the crate is published: `main` here,
  each release on docs.rs.
- **The handbook's explanations are not on the site.** A reader of the hosted reference
  who wants the reasoning follows a path to GitHub.
- **A branch that differs from `main` only in case is not told apart.** Expressions and
  concurrency groups ignore case, so a hand-started run from a branch named `Main`
  passes the deploy job's `if:` and shares `main`'s group. The environment's rule is
  then the only guard, and that is the setting the `if:` was written not to lean on.
  Nothing in an expression compares with case, so the workflow cannot close this itself.
- **Two settings live outside the repository.** The Pages source and the environment's
  rule are not in any file, so no gate reads them.
- **The build installs the whole gate's tooling to run one `cargo doc`**, because the
  setup action is the one tested path to `just` and the pinned toolchain.

## What would reopen this

Publishing the handbook as a site. A second documented crate in the workspace, which
makes the root redirect a choice between crates. A custom domain for this repository. A
need for versioned documentation that docs.rs does not meet.

## Related pages

- [Deploy to GitHub Pages](../how-to/deploy-to-github-pages.md)
- [API reference](../reference/api.md)
- [Security model](../explanation/security-model.md)
- [Quality gates](../reference/quality-gates.md)
