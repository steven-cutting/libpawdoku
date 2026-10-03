---
id: T33
title: "The API reference on GitHub Pages: a deploy workflow, a site recipe and the first workflow that publishes"
status: open
depends_on: [T11]
parallel_with: [S10]
branch: ticket/t33-api-reference-on-github-pages
estimated_size: M
---

# T33: The API reference on GitHub Pages: a deploy workflow, a site recipe and the first workflow that publishes

## Context

The API reference is the one rustdoc generates, and nobody can read it without a clone.
`docs/reference/api.md` lines 47 to 49 say so: every crate is `publish = false` until
S02's follow-up lifts it, "so nothing is on docs.rs yet, and `just doc` is the reference
in the meantime". The repository is public (T01) and has no Pages site: T01 ran the
template's bootstrap script with `--no-pages`
(`tickets/T01-github-repository.md` lines 185 and 204).

The maintainer settled the shape on 2026-10-02:

- **Rustdoc only.** The site is what `cargo doc` writes. No site generator, and the
  handbook under `docs/` stays where it is, read on GitHub.
- **Every push to `main`.** The site documents `main`. docs.rs will document each
  release once the crate is published, so the two do not overlap.
- **One build ticket and one spike.** This ticket hosts the reference. S10 weighs how
  prose pages, diagrams and raw HTML go inside rustdoc, and drafts its own build ticket.

This is the first workflow here that publishes. `AGENTS.md` line 129 says "No workflow
publishes anything", and five other sentences say the same in other words; the table
under step 7 lists them. A sentence that the change makes false is corrected in the
change.

The house already has a Pages deployment, in the game. Read it before writing a line:

| Source | Path | What to take |
| --- | --- | --- |
| G at `78d03cdf` | `.github/workflows/pages.yml` | The caller's shape: the triggers, the concurrency group, the comment on why the deploy has a file of its own |
| B at `6c5c07f6` | `.github/workflows/game-pages.yml` | The two jobs: a build that holds no publishing scope, and a deploy that holds `pages: write` and `id-token: write` and names the `github-pages` environment. It cannot be called from here: its build is `npm ci` and `npm run build` |
| G at `78d03cdf` | `docs/how-to/deploy-to-github-pages.md` | The page this ticket carries, section for section |
| G at `78d03cdf` | `docs/decisions/0010-a-project-pages-site.md` | The shape of the decision record |
| T at `2283589c` | `scripts/bootstrap_repo.sh` lines 160 to 222 | Step 1, the Pages source: the `gh api` calls and their fallbacks |

Facts, each read on 2026-10-02 in the source named. Nothing here was run in this
repository; step 2 runs what can be run.

- **Stable rustdoc writes no page at the root of `target/doc`.** `--enable-index-page`
  and `--index-page` are listed under "Unstable features" in the rustdoc book
  (<https://doc.rust-lang.org/rustdoc/unstable-features.html>). `cargo doc` for a crate
  named `pawdoku` writes `target/doc/pawdoku/index.html`. A site served from
  `target/doc` therefore needs a root `index.html` of its own, and the usual one is a
  redirect.
- **Rustdoc's links are relative.** Each page carries its own path to the root
  (`data-root-path`), so the tree can be served beneath `/libpawdoku/` with no base path
  passed to the build. That is why `actions/configure-pages` is not used here: nothing
  needs the value it computes.
- **A project site is served at `https://steven-cutting.github.io/libpawdoku/`**: the
  owner's Pages host, then the repository's name.
- **A custom workflow does not run Jekyll,** and `actions/upload-pages-artifact` has left
  dotfiles out of the artefact since v4 unless `include-hidden-files` is set
  (<https://github.com/actions/upload-pages-artifact>). So no `.nojekyll` file is
  needed, and the `.lock` file cargo leaves in `target/doc` is not uploaded. The
  artefact must hold no symbolic link.
- **The Pages source is a repository setting the workflow cannot set for itself.** Until
  it is "GitHub Actions", the build job uploads its artefact and the deploy job fails
  with `Failed to create deployment (status: 404)` (T `scripts/bootstrap_repo.sh` lines
  187 to 189). `actions/configure-pages` can enable Pages only with a token the run does
  not have, so it is not the route. The call is
  `gh api -X POST repos/steven-cutting/libpawdoku/pages -f build_type=workflow`; on
  HTTP 409 a site exists and the same body goes by `PUT`; on HTTP 422 the body is resent
  with a `source` of `main` and `/`.
- **The `github-pages` environment exists once a run reaches the deploy job** (G's
  how-to, lines 37 and 38), **and nothing here may lean on what protects it.** GitHub's
  page on managing environments says of one a workflow creates: "the newly created
  environment will not have any protection rules or secrets configured"
  (<https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments>).
  Whether the one Pages makes carries a branch rule of its own was not verified. So the
  deploy job carries its own `if:`, and step 5 gives the environment a branch rule
  before anything is deployed. S02 recorded one environment in this repository on
  2026-09-27, `copilot`.
- **A concurrency group keeps one run waiting, not a queue.** "By default, any existing
  `pending` job or workflow in the same concurrency group will be canceled and the new
  queued job or workflow will take its place"
  (<https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency>);
  `cancel-in-progress: false` spares only the run already going. The game's group is the
  one word `pages`. Here that would let a run started by hand from another branch, which
  builds and does not deploy, take the place of a waiting deployment of `main` and
  publish nothing in its stead. So the group here carries the ref.
- **The deployment cannot be proved before the merge.** The trigger is a push to
  `main`, and `workflow_dispatch` is offered only for a workflow file that exists on the
  default branch. See "Open points".
- **Releases on 2026-10-02:** `actions/upload-pages-artifact` v5.0.0 (2026-04-10) and
  `actions/deploy-pages` v5.0.1 (2026-09-01). The house pins, in B's `game-pages.yml`,
  are `fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0` for the upload and
  `cd2ce8fcbc39b97be8ca5fce6e763baed58fa128 # v5.0.0` for the deploy. Pages limits a
  site to 1 GB and a deployment to ten minutes
  (<https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits>).

Three files this ticket edits are frozen under CONVENTIONS.md §11: the `Justfile`,
`docs/manifest.yml` and `docs/README.md`. T22 reopened the `Justfile` for `metrics`, T24
for `plan-spec` and T29 for the snapshot recipes; this is another T00 follow-up of T24's
class, a recipe that is not a gate. See "Open points" for the other two.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §10 and §11; `.github/workflows/ci.yml`
and `.github/actions/setup/action.yml`; the `Justfile`, the `doc` and `audit` recipes;
`docs/reference/api.md`; `docs/reference/quality-gates.md`, "In continuous integration"
and "On `main`"; `docs/explanation/security-model.md`;
`docs/reference/documentation-contract.md`; `docs/decisions/README.md`.

## Goal

A push to `main` publishes the API reference at
`https://steven-cutting.github.io/libpawdoku/`. The root of the site sends a reader to
`pawdoku/index.html`. What is published is what `just doc` builds, with the gate's own
flags, and no second build of the documentation exists. The deploy has a workflow of its
own, in which only the job that deploys holds a write scope. The handbook says where the
reference is hosted, how it gets there and how to put it back, and no page still says
that nothing publishes.

## Non-goals

- No site generator, and no handbook page on the site. Decided; decision record below.
- No extra page, diagram, header file or change to rustdoc's flags. S10 and its
  follow-up own those.
- No versioned documentation and no tree per release. docs.rs keeps those once the crate
  is published.
- No custom domain.
- No `documentation` key in `crates/pawdoku/Cargo.toml`. Left unset, crates.io links a
  published crate to docs.rs, which is the reference for a release.
- No change to `.github/workflows/ci.yml`, to `pyproject.toml`'s recipe list or to the
  `doc` recipe. `check` runs the same twenty gates.
- No preview of a pull request's documentation. GitHub has no public feature for it.
- No change to the list of separately authorised actions in `AGENTS.md`. "Deploying" is
  already in it.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `.github/workflows/pages.yml` | workflow | New: the build job and the deploy job (step 4) |
| `Justfile` | T00 follow-up | The `site` recipe, after `doc` (step 3) |
| `docs/how-to/deploy-to-github-pages.md` | how-to | New, carried from G (step 6) |
| `docs/decisions/0015-api-reference-on-github-pages.md` | decision | New record (step 6); the number is the next free one on the day |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml` | T00 follow-up | Entries for the two new pages. Frozen under CONVENTIONS.md §11; step 1 |
| `docs/README.md` | T00 follow-up | The how-to's link under "How to". Frozen under CONVENTIONS.md §11; step 1 |
| `AGENTS.md` | agent contract | The paragraph at 129 to 132 (step 7) |
| `SECURITY.md` | maintainer docs | The bullet at 51 and 52 |
| `docs/explanation/security-model.md` | explanation | The bullets at 23 to 26 and 62 to 64 |
| `docs/reference/quality-gates.md` | reference | "In continuous integration" gains the workflow; the sentence at 170 |
| `docs/operations/maintenance.md` | operations | The sentence at 11 |
| `docs/reference/api.md` | reference | A "Hosted" section; the sentence at 47 to 49 |
| `docs/reference/commands.md` | reference | A row for `just site`; the sentence at 22 to 24 |
| `docs/reference/configuration.md` | reference | The row at 25: which workflows export the variables |
| `docs/reference/documentation-contract.md` | reference | "Links that leave the repository", at 75 to 80: the pages that point outward |
| `docs/project/repository-map.md` | project | The `.github/` line at 48 |
| `docs/how-to/maintain-dependencies.md` | how-to | The sentence at 185: the files that pin an action |
| `README.md` | maintainer docs | The hosted address beside the API reference line at 79 |
| `CHANGELOG.md` | maintainer docs | Under Unreleased, Added; the count of decision records at 60 |
| `tickets/README.md` | ticket index | T33's row set `done`, in the closing pull request (step 12) |
| `tickets/T33-api-reference-on-github-pages.md` | ticket | Hand-back notes in both pull requests; `status:` in the closing one |

## Steps

1. Create the worktree on `ticket/t33-api-reference-on-github-pages` from `main`
   (`tickets/README.md`, "How to pick up a ticket"). If `.pixi/` is absent the gate
   cannot run; `just initialize` installs it over the network, which is a separately
   authorised action: ask first.

   Before editing anything, confirm with the maintainer that the three frozen files in
   "Files touched" may be edited here: the `Justfile`, `docs/manifest.yml` and
   `docs/README.md`. Nothing in this ticket can land without them. The workflow runs
   the recipe, and a page, its manifest entry and its link from the map are one commit
   or the documentation contract fails. So there is no handing one of them back and
   landing the rest: without leave for all three, stop here and say so.

2. See what there is to serve. Run `just doc`, then record in the hand-back notes:

   - that `target/doc/pawdoku/index.html` exists and `target/doc/index.html` does not;
   - that a page under `pawdoku/` reaches its stylesheet by a relative path
     (`../static.files/`), which is what lets the tree be served beneath `/libpawdoku/`;
   - what `find target/doc -type l` prints (expected: nothing);
   - the size `du -sh target/doc` reports;
   - whether a name from a dependency in the reference (an implemented `serde` or
     `thiserror` trait) is a link to docs.rs, plain text, or a link that leads nowhere.
     `--no-deps` documents no dependency, so the third would be a dead link on the
     site. Record it; if any lead nowhere, say so under "Handed back" and do not fix it
     here.

3. Add the recipe to the `Justfile`, directly after `doc`, as `audit` sits after `deny`:

   ```text
   # What GitHub Pages serves: the API reference `doc` builds, from an empty
   # target/doc so that nothing stale is published, and a root index.html that
   # sends a reader to the crate, because stable rustdoc writes none. Not a
   # gate and outside `just check`: `doc` is the gate, and this adds one file
   # to what it built. The flags are `doc`'s own, through the recipe, so the
   # hosted reference is the one the gate proved.
   site:
       rm -rf target/doc
       just doc
       printf '%s\n' '<!doctype html>' '<html lang="en">' '<meta charset="utf-8">' '<title>pawdoku: API reference</title>' '<meta http-equiv="refresh" content="0; url=pawdoku/index.html">' '<p><a href="pawdoku/index.html">The pawdoku API reference</a></p>' '</html>' > target/doc/index.html
   ```

   `fix` already calls `just lint` from inside a recipe, so `just doc` here is the
   file's own idiom. The `rm` is there because `target/doc` keeps a page for every item
   it has ever documented, on a developer's machine and in a restored CI cache alike;
   whether `Swatinem/rust-cache` keeps `target/doc` was not checked, and the `rm` makes
   the answer not matter.

   Run `just site` twice. Confirm `target/doc/index.html` holds the seven lines, that
   opening it in a browser lands on the crate's page, and that `git status --porcelain`
   is empty afterwards, which proves the recipe writes only under an ignored path.

4. Write `.github/workflows/pages.yml`:

   ```yaml
   name: Deploy to GitHub Pages

   # Publishing needs write scopes that no other workflow here has, so the
   # deploy lives in its own file rather than as a job on the end of CI.

   on:
     push:
       # Quoted for the reason ci.yml gives.
       branches: ['main']
     workflow_dispatch:

   # Read-only unless a job says otherwise. A job-level block replaces this
   # one, so the job that checks out and builds never holds a publishing scope.
   permissions:
     contents: read

   # One deployment at a time, and never cancel one that is already running: a
   # half-published site is worse than a slightly stale one. The ref is in the
   # group because a group keeps one run waiting and a newer one takes its
   # place: a run started by hand from another branch deploys nothing, and must
   # not displace a waiting deployment of main.
   concurrency:
     group: pages-${{ github.ref }}
     cancel-in-progress: false

   env:
     CARGO_TERM_COLOR: always
     CARGO_INCREMENTAL: '0'
     CARGO_NET_RETRY: '10'
     RUST_BACKTRACE: '1'

   jobs:
     build:
       name: build
       runs-on: ubuntu-latest
       timeout-minutes: 15
       steps:
         - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
           with:
             persist-credentials: false
         - uses: ./.github/actions/setup
           with:
             cache-key: pages
         - run: just check-toolchain
         - run: just site
         # Dotfiles are left out of the artefact, so cargo's target/doc/.lock
         # is not published.
         - uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0
           with:
             path: target/doc

     deploy:
       name: deploy
       needs: build
       # A run started by hand from another branch builds and stops here. Not
       # optional: the environment's own branch rule is a setting, and a
       # setting can be changed where no reviewer sees it.
       if: github.ref == 'refs/heads/main'
       runs-on: ubuntu-latest
       timeout-minutes: 10
       # The two publishing scopes, on the one job that has no checkout and
       # runs no code from the repository.
       permissions:
         contents: read
         pages: write
         id-token: write
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       steps:
         - id: deployment
           uses: actions/deploy-pages@cd2ce8fcbc39b97be8ca5fce6e763baed58fa128 # v5.0.0
   ```

   Look each remote action up on the day and pin the latest release of the major the
   house uses, with its version comment
   (`gh api repos/<owner>/<repo>/git/ref/tags/<tag>`, dereferencing an annotated tag;
   `docs/how-to/maintain-dependencies.md`, "Actions in the workflows"). The SHAs above
   are the house pins and are the floor; the checkout pin is `ci.yml`'s and moves only
   with it. Every `run:` stays one line, one recipe: `actionlint` under `prek` reads no
   embedded shell (`docs/reference/quality-gates.md` lines 112 to 119).

   Neither job joins `check`'s `needs` in `ci.yml`. A deployment is not a gate, as
   `audit` is not.

   Two lines differ from the game's workflow on purpose, and the decision record says
   so: the `if:` on the deploy job, and the ref in the concurrency group. The game's
   workflow has neither because it has no reason to be run from another branch.

5. The Pages source and the environment. **Stop and ask the maintainer before this
   step**: it changes two repository settings. Do it before the pull request merges, so
   that the push to `main` deploys and so that the environment is restricted before the
   first deployment, not after:

   ```sh
   gh api repos/steven-cutting/libpawdoku/pages --jq .build_type
   gh api -X POST repos/steven-cutting/libpawdoku/pages -f build_type=workflow
   gh api repos/steven-cutting/libpawdoku/pages --jq '.build_type, .html_url'
   gh api repos/steven-cutting/libpawdoku/environments/github-pages --jq .deployment_branch_policy
   ```

   The first line is expected to fail with HTTP 404, no site. The third prints
   `workflow` and the address. On HTTP 409 or 422 from the second, follow the fallbacks
   in T's `scripts/bootstrap_repo.sh` lines 160 to 178, and record which ran. Enabling
   the source with no workflow on `main` publishes nothing.

   The fourth line reads the environment's branch rule. If the environment does not
   exist yet (HTTP 404), or the rule is `null`, which allows every branch, set it:

   ```sh
   printf '%s\n' '{"deployment_branch_policy": {"protected_branches": true, "custom_branch_policies": false}}' | gh api -X PUT repos/steven-cutting/libpawdoku/environments/github-pages --input -
   gh api repos/steven-cutting/libpawdoku/environments/github-pages --jq .deployment_branch_policy
   ```

   That is "Create or update an environment" in GitHub's REST reference
   (<https://docs.github.com/en/rest/deployments/environments>), read on 2026-10-02 and
   not run here: it creates the environment when there is none, and
   `protected_branches: true` lets only a protected branch deploy. `main` is the one
   protected branch (T01). If GitHub already gave the environment a rule that admits
   `main` alone, leave it and record it. Either way the last line must show a rule that
   is not `null` before the pull request merges.

6. The two new pages. Each lands in the same commit as its `docs/manifest.yml` entry and
   its link, or the documentation hook refuses the commit.

   - `docs/how-to/deploy-to-github-pages.md`, carried from G's page of the same path.
     Frontmatter: title "Deploy to GitHub Pages", kind `how-to`, audience
     `[maintainer, operator, agent]`, `canonical_for: [deployment_procedure]`, G's slug.
     Keep G's sections and rewrite each for a library:
     - the opening: what is published (the API reference, from `main`), by which
       workflow, at which address, and that nothing is committed to a branch;
     - "One-time setup": two settings, each with its `gh api` line from step 5: the
       Pages source, and the `github-pages` environment's branch rule. G's second step,
       a package grant, has no counterpart here, and G's sentence that the environment
       "needs no setup" is not carried: here it is given a rule. Keep the sentence on
       how the deploy job fails without the source;
     - "Where the site is served from": a project site beneath `/libpawdoku/`; rustdoc's
       links are relative, so the build is told nothing about the path. A custom domain
       would be a change to the repository's settings alone, for the same reason;
     - "What the workflow does": the two jobs and which holds what, `just site`, the
       artefact, the concurrency group and why it carries the ref, and what a run
       started by hand from another branch does (it builds, and deploys nothing);
     - "Reproduce a deployment locally": `just site`, then open
       `target/doc/index.html`;
     - "Rolling back": re-run the last good deployment, or revert and let the push
       rebuild;
     - "Related pages": the decision record, API reference, Quality gates, Maintenance.

     Say once, plainly, that the site documents `main` and may describe an item no
     release has.

   - `docs/decisions/0015-api-reference-on-github-pages.md`, in the house shape
     (Context, Decision, Consequences, What would reopen this, Related pages). Title
     "Decision 0015: The API reference on GitHub Pages"; audience
     `[contributor, maintainer, agent]`; `canonical_for: [decision_api_reference_on_pages]`.
     Take the next free number on the day and rename the file, the title and the row if
     0015 is taken. Its Context says what was true before in its own words: the
     acceptance search below reads `docs/`, and a record that quotes the old `AGENTS.md`
     sentence would fail it. It records:
     - the decision: rustdoc alone, from `main`, on every push, as a project site;
       built by `just site` with the `doc` gate's flags; a workflow of its own, with the
       publishing scopes on a deploy job that checks nothing out;
     - where the workflow leaves the game's and why: the deploy job's `if:`, the ref in
       the concurrency group, and the environment's branch rule set by hand, because
       this workflow can be started from a branch that must not publish;
     - what was turned down and why: a site generator for the handbook (a second tool, a
       second build, and a handbook whose frontmatter and manifest are made for GitHub's
       rendering); publishing on release only (nothing to read until the first release);
       a tree per version (docs.rs does it for releases);
     - the consequences that hurt: the first workflow that holds a write scope; a site
       that describes `main` and says so nowhere on its pages; a reference in two places
       once the crate is published; the handbook's explanations are not on the site;
     - what would reopen it: publishing the handbook as a site; a second documented
       crate in the workspace, which makes the root redirect a choice; a custom domain;
       a need for versioned documentation that docs.rs does not meet.

   Add the record's row to `docs/decisions/README.md` and move "The next decision this
   repository takes is" on by one. Add both manifest entries, the how-to after
   `how-to/maintain-dependencies.md` and the record after the last decision. Add the
   how-to's link to `docs/README.md` under "How to".

7. The sentences the workflow makes false or leaves short. Reword each so that it is
   true on the day the workflow is on `main`:

   | Path | Lines | Today | Becomes |
   | --- | --- | --- | --- |
   | `AGENTS.md` | 129 to 132 | "No workflow publishes anything: `audit.yml` only reads the advisory database." | One workflow publishes: `pages.yml` deploys the API reference to GitHub Pages on a push to `main`, and only its deploy job holds a write scope. `ci.yml` and `audit.yml` stay read-only. The crates.io sentence stands |
   | `docs/explanation/security-model.md` | 62 to 64 | "Every workflow runs with `contents: read` and nothing else. The release workflow S02 designs will be the only one to hold more" | `ci.yml` and `audit.yml` hold `contents: read` and nothing else. `pages.yml` holds more, on its deploy job alone, which checks nothing out; the release workflow will be the second, in a file of its own |
   | `docs/explanation/security-model.md` | 23 to 26 | "the workflows need nothing more" | Still no stored credential: the deploy proves itself with the run's own identity token. Say so in a clause |
   | `SECURITY.md` | 51 and 52 | "Continuous integration runs with `contents: read` and holds no stored secret" | The gate's workflows do; the Pages deployment holds two publishing scopes on one job; no workflow holds a stored secret |
   | `docs/reference/quality-gates.md` | 170 | "There is no deployment: nothing publishes on a push to `main`." | A push to `main` publishes the API reference through `pages.yml`; it is not a gate and not a required check |
   | `docs/reference/quality-gates.md` | 129 to 155 | The section names `ci.yml` and `audit.yml` | A paragraph for `pages.yml`: what it runs, and that it sits outside `check`'s `needs` |
   | `docs/operations/maintenance.md` | 11 | "There is no service to operate and nothing to deploy." | No service to operate; the one thing deployed is the API reference, by a workflow, with a link to the how-to |
   | `docs/reference/api.md` | 47 to 49 | "nothing is on docs.rs yet, and `just doc` is the reference in the meantime" | A "Hosted" section before "docs.rs": the address, that it is `main`, that `just site` builds it. The docs.rs section keeps "prospective" and loses "in the meantime" |
   | `docs/reference/commands.md` | 22 to 24, and the Check table | The recipes outside `just check` | `site` joins the list; a row after `just doc`: what it builds, that it empties `target/doc` first, that it is outside `just check` |
   | `docs/reference/configuration.md` | 25 | "Exported by `ci.yml`; `audit.yml` exports the first three." | `pages.yml` exports all four |
   | `docs/reference/documentation-contract.md` | 75 to 80 | "Three pages point outward", and names them | The count and the list as they stand after this change: the API reference page and the how-to each give the site's address, and the record does if it links one. The rule stands: a whole page, never a heading |
   | `docs/project/repository-map.md` | 48 | "CI, the audit workflow and the composite setup action" | The Pages workflow joins the list |
   | `docs/how-to/maintain-dependencies.md` | 185 | "`ci.yml`, `audit.yml` and the composite `setup` action pin every remote action" | `pages.yml` joins the list |

   `AGENTS.md` is checked for six required phrases (`untrusted`, `just check`,
   `explicit authorization`, `ai_tmp/`, `docs/specs/`, `runes`); the paragraph being
   reworded holds none of them. Leave room in its wording for the release workflow S02's
   follow-up adds, so that ticket adds a clause and does not rewrite the paragraph
   again. Nothing in `.github/workflows/ci.yml` changes: its opening comment, "nothing
   here publishes", stays true of that file.

8. `README.md` line 79: the address of the hosted reference beside the link to
   `docs/reference/api.md`. `CHANGELOG.md`, under Unreleased, Added: the hosted
   reference in one entry, and the count of decision records at line 60 set to the count
   on the day.

9. Run `just check-agents`, `just check-docs`, `just lint`, `just site` and
   `just check`. Read the whole diff. For each row of step 7's table, quote the sentence
   as it was and as it now reads in the hand-back notes: the search under Verification
   catches three phrases and proves nothing about the other rows.

10. The first pull request. Fill in the hand-back notes with what was proved: the
    build, the gate, the settings of step 5. **Leave `status: open`**, here and in
    `tickets/README.md`: the ticket's outcome is a published site, and none exists yet.
    Commit on the ticket branch. **Stop before pushing**: pushing and opening the pull
    request are separately authorised.

11. After the merge, with the maintainer's leave for each remote call: watch the run of
    `pages.yml` on `main`, then run the "After the merge" commands under Verification.
    The last of them starts the workflow by hand from a branch other than `main` and
    shows that its deploy job is skipped.

12. The closing pull request, on a branch from `main`. Add the output of step 11 to the
    hand-back notes, set `status: done` here and in `tickets/README.md`, run
    `just check`, commit, and stop before pushing. If the first deployment failed, this
    pull request carries the fix as well, and the ticket stays open until a deployment
    of `main` is green and the site answers.

## Acceptance criteria

In the first pull request:

- `just site` exits 0 and leaves `target/doc/index.html`, which sends a browser to
  `pawdoku/index.html`, beside `target/doc/pawdoku/index.html`. The tree holds no
  symbolic link. `git status --porcelain` is empty after it.
- `pyproject.toml` and `.github/workflows/ci.yml` are unchanged. `just check` runs the
  same twenty gates and is green.
- In `.github/workflows/pages.yml`: every remote `uses:` is pinned to a full commit SHA
  with a version comment; every `run:` is one line; `pages: write` and
  `id-token: write` appear on the `deploy` job and nowhere else; the `build` job holds
  no scope beyond `contents: read`. The deploy job carries the `if:` on `main`, and the
  concurrency group carries the ref. `just lint` passes, which runs `actionlint` over
  it.
- Every row of step 7's table is quoted in the hand-back notes as it was and as it now
  reads, and each new reading is true of the repository with the workflow on `main`.
  `rg -n "No workflow publishes|There is no deployment|nothing to deploy" AGENTS.md README.md SECURITY.md docs`
  prints nothing.
- `just check-agents` passes. `just check-docs` passes and reports two more pages and
  two more canonical topics than `main` does.
- The how-to is linked from `docs/README.md`, and the record has its row in
  `docs/decisions/README.md`.
- Before the merge, the Pages source is `workflow` and the `github-pages` environment
  has a branch rule that is not `null`; the hand-back notes quote both.
- `status:` is `open`.

In the closing pull request:

- The run of `pages.yml` on `main` is green in both jobs; the site's root answers 200
  and holds the redirect; `pawdoku/index.html` answers 200.
- A run started by hand from another branch completed with its deploy job skipped, and
  the site was not redeployed by it.
- The hand-back notes quote the output of every "After the merge" command, and
  `status:` is `done` here and in `tickets/README.md`.

## Verification

Before the first pull request:

```sh
just site
ls target/doc/index.html target/doc/pawdoku/index.html
find target/doc -type l
git status --porcelain
just check-agents
just check-docs
just lint
just check
git diff --stat main -- pyproject.toml .github/workflows/ci.yml
rg -n "No workflow publishes|There is no deployment|nothing to deploy" AGENTS.md README.md SECURITY.md docs
rg -n "pages: write|id-token: write" .github/workflows
```

Expected: both files listed; nothing from `find`; nothing from `git status` once the
work is committed; `Validated AGENTS.md, 2 adapters, and 14 skills.`; the page and topic
counts two above `main`'s; every hook passing; `All checks passed and the worktree is
unchanged.`; nothing from `git diff --stat`; nothing from the first `rg`; two lines from
the second, both in `pages.yml` and both under `deploy`.

After the merge, each a remote call the maintainer authorises. `<branch>` is a branch
other than `main` that holds the workflow file and is on the remote: the closing pull
request's branch serves, cut from `main` and pushed before it has a commit of its own,
and pushing it is one more thing to ask for. `<id>` is what the line before prints:

```sh
gh api repos/steven-cutting/libpawdoku/pages --jq '.build_type, .html_url'
gh api repos/steven-cutting/libpawdoku/environments/github-pages --jq .deployment_branch_policy
gh run list --workflow pages.yml --branch main --limit 1
curl -sI https://steven-cutting.github.io/libpawdoku/
curl -s https://steven-cutting.github.io/libpawdoku/
curl -sI https://steven-cutting.github.io/libpawdoku/pawdoku/index.html
gh workflow run pages.yml --ref <branch>
gh run list --workflow pages.yml --branch <branch> --limit 1 --json databaseId --jq '.[0].databaseId'
gh run view <id> --json jobs --jq '.jobs[] | [.name, .conclusion] | @tsv'
```

Expected: `workflow` and the address; a branch rule that is not `null`; a completed,
successful run on `main`; HTTP 200; the seven lines of the redirect; HTTP 200; and, for
the run started by hand once it has finished, `build` with `success` and `deploy` with
`skipped`.

## Hand-back notes

None yet.

## Open points

- **Two pull requests, and why.** The first can prove the build (`just site`,
  `actionlint`, `just check`) and nothing about the deploy job: a push to `main` is the
  trigger, and a workflow can be started by hand only once its file is on the default
  branch. The definition of done wants the acceptance criteria met and every
  verification command's output in the hand-back notes of the pull request that sets
  `status: done` (`tickets/README.md`, "Definition of done"). So the first pull request
  leaves the ticket open and the closing one, step 12, carries the proof. The
  maintainer may prefer one pull request, with the after-merge output as a comment on
  it; that is a deviation from the definition of done and is theirs to grant, not the
  agent's to take.
- **The three frozen files.** The `Justfile`, `docs/manifest.yml` and `docs/README.md`
  are frozen under CONVENTIONS.md §11, which sends a change there to a T00 follow-up on
  `main`. T22, T24 and T29 reopened the first as follow-ups carried in their own
  tickets; T19 edited the other two, and T22 and T23 each added a decision record's
  manifest entry. This ticket is the same kind of carrier, and step 1 asks before any
  edit. There is no part-way: see step 1.
- **The environment's rule rests on `main` staying protected.** `protected_branches:
  true` admits a branch only while it has a protection rule. If `main`'s protection
  were ever removed, deployments would stop, which is the safe way to fail. A rule that
  names `main` by pattern is the alternative, and takes a second call.
- **Paths other open tickets also edit.** The basic-generator tickets drafted as T31
  and T32 edit `CHANGELOG.md`, `docs/manifest.yml`, `docs/decisions/README.md`,
  `docs/README.md` and `docs/project/repository-map.md`, and T31's draft takes decision
  0015. S02's drafted follow-up, T13, rewords the same `AGENTS.md` paragraph and the
  same sentence of `docs/explanation/security-model.md`. §11 forbids two open lanes
  sharing a path. This is a stated deviation from it: each shared edit is a row, an
  entry or a sentence, and whichever pull request merges second rebases and takes the
  next decision number.
- **The setup action in the build job.** It installs the whole gate's tooling (the pixi
  environment, the hooks, cargo-hack, rustqual) to run one `cargo doc`, which costs
  minutes on a cold cache. It is used because it is the one tested path to `just` and
  the pinned toolchain, and because the job that uses it holds no publishing scope.
  S02's T13 draft raises the same question for its publishing job. A leaner setup is a
  later change if the run time is felt.
- **`deploy-pages` v5.0.1.** Newer than the house pin by one patch. Step 4 says to pin
  the latest of the major on the day, which moves this repository ahead of the game and
  the tooling repository. Say so in the hand-back notes.
- **Nothing on the site says it documents `main`.** Rustdoc prints the crate's version,
  `0.1.0`, and nothing about the branch. A banner needs `--html-before-content`, which
  is a change to rustdoc's flags and so S10's follow-up's to weigh, not this ticket's.
- **The crate's opening page names handbook paths as code spans**
  (`crates/pawdoku/src/lib.rs` lines 7, 10 and 16). On the hosted site they are not
  links. Whether the reference links to the handbook on GitHub is S10's question, under
  its rule for what belongs where.
