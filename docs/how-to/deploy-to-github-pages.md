---
title: "Deploy to GitHub Pages"
kind: "how-to"
audience: [maintainer, operator, agent]
canonical_for: [deployment_procedure]
requires: []
---

# Deploy to GitHub Pages

This library publishes one thing: the API reference rustdoc generates, built from `main`.
`.github/workflows/pages.yml` builds it with `just site` on every push to `main` and
hands `target/doc` to the Pages deployment action; nothing is committed to a branch.

The site is a project site at <https://steven-cutting.github.io/libpawdoku/>: the
repository `steven-cutting/libpawdoku`, beneath `/libpawdoku/` on its owner's Pages host.

The site documents `main`, so it may describe an item no release has. Once the crate is
published, docs.rs is the reference for a release.

## One-time setup

Two repository settings, each made once, before the first push that should deploy. The
workflow can make neither for itself.

1. The Pages source is **GitHub Actions**:

   ```console
   gh api -X POST repos/steven-cutting/libpawdoku/pages -f build_type=workflow
   gh api repos/steven-cutting/libpawdoku/pages --jq '.build_type, .html_url'
   ```

   The second line prints `workflow` and the address. Where a site exists already the
   `POST` answers HTTP 409, and the same body goes by `PUT`.

2. The `github-pages` environment, which the deploy job names, lets only `main` deploy:

   ```console
   gh api repos/steven-cutting/libpawdoku/environments/github-pages --jq .deployment_branch_policy
   gh api repos/steven-cutting/libpawdoku/environments/github-pages/deployment-branch-policies --jq '.branch_policies[].name'
   ```

   When this repository's site was created, on 2026-10-03, GitHub made the environment
   with that rule already on it: the first line printed
   `{"custom_branch_policies":true,"protected_branches":false}` and the second `main`. A
   rule of `null` admits every branch. If the first line ever prints that, or the
   environment is missing, set a rule that admits protected branches alone, and `main` is
   the one protected branch:

   ```console
   printf '%s\n' '{"deployment_branch_policy": {"protected_branches": true, "custom_branch_policies": false}}' | gh api -X PUT repos/steven-cutting/libpawdoku/environments/github-pages --input -
   ```

Until the source is set, the build job succeeds and uploads its artefact but the deploy
job fails with `Failed to create deployment (status: 404)`, even though the workflow
itself is correct.

## Where the site is served from

A project site lives beneath the repository's name on the owner's Pages host. Rustdoc's
links are relative, and each page carries its own path back to the root, so the build is
told nothing about `/libpawdoku/` and the same tree opens from a local directory.

This repository sets no custom domain, and adding one would be a change to the
repository's settings alone, for the same reason. The owner's Pages host carries a domain
of its own, though, and every project site beneath it follows: the address above answers
with a redirect to the same path on `stevencutting.com`. The `github.io` address is the
one this handbook gives, because it stays true whatever domain the host carries.

## What the workflow does

- **`build`** checks the repository out, runs the composite setup action, then
  `just check-toolchain` and `just site`. `just site` empties `target/doc`, runs
  `just doc`, the gate's own recipe with the gate's own flags, and writes a root
  `index.html` that sends a reader to `pawdoku/index.html`, because stable rustdoc
  writes no page at the root. The job uploads `target/doc` as the Pages artefact; the
  upload leaves dotfiles out, so cargo's `.lock` file is not published. This job holds
  `contents: read` and nothing else.
- **`deploy`** publishes the artefact. It holds `pages: write` and `id-token: write`, the
  only write scopes in any workflow here, and it checks nothing out and runs no code from
  the repository. It proves itself to Pages with the run's own identity token; no secret
  is stored. It runs only when the ref is `main`.

A push to `main` deploys, and so does a run started by hand from `main`. A run started by
hand from any other branch builds, and deploys nothing: the deploy job's own `if:` skips
it, and the environment's branch rule would refuse it besides.

One kind of branch slips past the first of those. GitHub compares strings in a workflow
expression, and the names of concurrency groups, without regard to case, and Git allows
a branch named `Main` beside `main`. A run started by hand from such a branch passes the
`if:` and shares `main`'s group, so it can take the place of a waiting deployment of
`main`, and the environment's rule is the one guard left against its publishing. Making
such a branch takes write access to the repository. Do not make one.

Deployments are serialised by a concurrency group and never cancelled mid-flight: a
half-published site is worse than a slightly stale one. The group carries the ref. A
group keeps one run waiting and a newer run takes its place, so with one group for every
ref a hand-started run from another branch, which deploys nothing, could take the place
of a waiting deployment of `main`.

Neither job is a gate. The workflow sits outside the `check` job that protects `main`,
and a failed deployment blocks no merge; see [Quality gates](../reference/quality-gates.md).

## Reproduce a deployment locally

```console
just site
```

Then open `target/doc/index.html` in a browser. It lands on the crate's page, as the
site's root does.

## Rolling back

Re-run the last good deployment from the Actions tab, or revert the commit and let the
push trigger a fresh build. There is no state to migrate and no cache to clear beyond the
browser's.

## Related pages

- [Decision 0016: The API reference on GitHub Pages](../decisions/0016-api-reference-on-github-pages.md)
- [API reference](../reference/api.md)
- [Quality gates](../reference/quality-gates.md)
- [Maintenance](../operations/maintenance.md)
