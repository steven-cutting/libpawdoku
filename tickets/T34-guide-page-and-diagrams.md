---
id: T34
title: "The engine map in the API reference: the first guide page"
status: done
depends_on: [T33, S10]
parallel_with: []
branch: ticket/t34-guide-page-and-diagrams
estimated_size: M
---

# T34: The engine map in the API reference: the first guide page

## Context

T33 put the API reference on GitHub Pages, and decision 0016 says the site is rustdoc
alone: the handbook under `docs/` is not hosted. S10 found how a page of prose and
diagrams goes inside rustdoc, gave a verdict for each mechanism, and drafted this ticket
in its hand-back notes (step 10). Read S10 first; nothing it measured is measured again
here.

The first page is a map of the engine: what the crate is, what it does and how, drawn
at four levels. Level 0 is the engine seen from outside, level 1 its modules, level 2
each module that is built, and level 3 three calls followed through the code.

The maintainer settled the shape on 2026-10-03:

- **One guide page,** `pawdoku::guide`, carrying all four levels in order.
- **Two versions are built and compared before one is kept.** One carries its figures
  as inline SVG drawn by hand, the other as `mermaid` fences. The maintainer compares
  them in a browser, from the local build, and picks. The one not picked is removed
  before anything is committed.
- **Mermaid is pinned at 12.1.0.** S10 recommended 11.17.2; the maintainer chose the
  newer line for the comparison.
- **The frozen files may be edited here:** the `Justfile`, `docs/manifest.yml` and
  `docs/README.md`, in the same change, as T33 had leave to.
- **Network:** the tool environment from local caches, with the network as a fallback;
  one fetch of the pinned Mermaid file from jsDelivr to compute its hash; a headless
  Chrome may load that file from jsDelivr to draw the Mermaid page.
- **The map of modules goes in the reference.** S10's rule (its step 8) keeps "how the
  engine is layered" in the handbook. The maintainer admits a map of how the public API
  fits together into the reference. `docs/explanation/layering.md` stays the owner of
  the import rule, and the guide links to it by its address on GitHub. A decision
  record says so.

What S10 found that this ticket leans on, each in its hand-back notes:

- A page is a documentation-only module. One that needs raw HTML is written as doc
  comments in a `.rs` file, which no Markdown hook reads; one that does not is a `.md`
  file included with `include_str!`. A `.rs` file whose first bytes are `#!` fails the
  shebang hook, so it begins with a `//` line.
- `#[cfg(doc)]` on the module's declaration keeps the page in the reference and out of
  the crate: no consumer can name it, and its example still runs under `just test-doc`.
- An inline `svg` passes `-D warnings` when every element is closed and no blank line
  falls inside the HTML block.
- `--html-in-header` puts a file in the head of every page. Its path is read from the
  directory cargo runs rustdoc in: `crates/pawdoku/rustdoc/header.html` for the `doc`
  recipe, `rustdoc/header.html` for `[package.metadata.docs.rs]`.
- `just doc` exits 0 with the flag dropped, so something must fail when it is.
- Rustdoc writes a `mermaid` fence as `pre.language-mermaid`, and the header's script
  draws it in the reader's browser from one pinned file with an integrity hash.

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §11;
`tickets/S10-pages-and-diagrams-in-rustdoc.md`, the hand-back notes;
`docs/decisions/0016-api-reference-on-github-pages.md`; `docs/reference/api.md`;
`docs/explanation/layering.md` and `docs/explanation/architecture.md`;
`docs/reference/documentation-contract.md`; the `Justfile`'s `doc` and `site` recipes;
`crates/pawdoku/Cargo.toml` and `crates/pawdoku/src/lib.rs`.

## Goal

The hosted reference carries a page, `pawdoku::guide`, that shows the engine in
figures at four levels, with every item it names linked to its own documentation and
one example that compiles and runs. The figures follow rustdoc's light, dark and ayu
themes, and the page renders the same wherever rustdoc builds it: on the hosted site,
on docs.rs, and in a consumer's own `cargo doc`. The handbook says how a page and a
figure are added, and a decision record says what a page in the reference may hold.

## Non-goals

- Pushing, opening the pull request or deploying. Each is separately authorised, and a
  deployment follows only from a push to `main` (decision 0016).
- A second guide page.
- Figures for the five modules that are specified and not built. They appear on the map
  of modules and nowhere else.
- Any change to the engine's code or its public API. The page's module is
  documentation-only and exists under `cfg(doc)` alone.
- Hosting the handbook. Decision 0016 stands.
- A dependency. None is added.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `crates/pawdoku/src/guide.rs` | new | The page, its figures and their `style` block |
| `crates/pawdoku/src/lib.rs` | engine | `#[cfg(doc)] pub mod guide;` and one sentence of the crate's documentation |
| `crates/pawdoku/rustdoc/header.html` | new, then removed | The figures' stylesheet and the Mermaid loader, built for the comparison and taken out after the adversarial review (hand-back notes) |
| `crates/pawdoku/Cargo.toml`, `Justfile` | edited, then restored | The header's flag in the docs.rs table and the `doc` recipe, with its probe; both files are as `main` has them |
| `docs/reference/api.md` | reference | A new section on pages and figures, and the docs.rs sentence |
| `docs/how-to/add-a-reference-page.md` | how-to | New |
| `docs/decisions/0017-guide-pages-in-the-api-reference.md` | decision | New |
| `docs/decisions/README.md` | decision index | The record's row; the "next decision" sentence |
| `docs/manifest.yml`, `docs/README.md` | T00 follow-up | The two new pages. Frozen; leave given |
| `docs/project/repository-map.md`, `docs/explanation/architecture.md` | handbook | The new file, and the module that is not behaviour (step 9) |
| `CHANGELOG.md` | maintainer docs | Under Unreleased, Added |
| `tickets/README.md`, this file | ticket | T34's row; the hand-back notes; `status:` |

## Steps

1. Create the branch `ticket/t34-guide-page-and-diagrams` from `main`. If `.pixi/` is
   absent the gate cannot run: install it from the caches behind a dead proxy
   (`HTTPS_PROXY=http://127.0.0.1:9 just sync`) and copy `.tools/bin` from a worktree
   that has `rustqual`. The network is the fallback, and leave for it is given above.

2. Write `crates/pawdoku/rustdoc/header.html`. Its first comment line is
   `pawdoku-rustdoc-header`, the marker the probe looks for. It holds:

   - a stylesheet for the figures. A figure is an inline `svg` with the class `pg`
     inside a `div` with the class `pg-fig`, and every class it uses begins `pg-`, so
     none can meet one of rustdoc's. Lines, text and neutral fills take rustdoc's own
     variables (`--main-color`, `--border-color`, `--code-block-background-color`,
     `--font-family`, `--font-family-code`). Three meanings have colours of their own,
     defined once for light and once for dark and ayu: built, a stored solution, and a
     refusal;
   - S10's Mermaid loader as quoted in its step 5, repinned to 12.1.0, the single file,
     with the `sha384` hash of the file fetched on the day. The comment at the head of
     the file says how to move the pin.

3. Pass the flag in both spellings, with a comment at each that the two move together:
   `--html-in-header crates/pawdoku/rustdoc/header.html` in the `Justfile`'s `doc`
   recipe, and `--html-in-header rustdoc/header.html` in `crates/pawdoku/Cargo.toml`.
   Add the probe to `doc`: after `cargo doc`, the recipe fails unless
   `target/doc/pawdoku/guide/index.html` holds the marker.

4. Write the page twice.

   - `crates/pawdoku/src/guide.rs`: a `//` comment first, then the page as `//!` lines,
     with each figure one HTML block.
   - `crates/pawdoku/src/guide_mermaid.md`, included by `guide_mermaid.rs`: the same
     text, with flowcharts and one sequence diagram as `mermaid` fences, the journal
     and the upkeep example as `text` fences, and the orders of refusal as tables.

   Declare both in `lib.rs` under `#[cfg(doc)]`. Both hold one example, a game from
   opening to reopening, which `just test-doc` runs. Every item a page names in its
   prose is an intra-doc link, so a renamed item fails `just doc`. A link to the
   handbook is an address on GitHub.

5. What the page must say. Each line below was checked against the source on
   2026-10-03, and several correct a first draft that an adversarial review found
   wrong or overclaimed:

   - Five modules are built (`sudoku`, `solver`, `board`, `generation`, `random`) and
     five are specified and not built. The map has nine specification modules and the
     randomness boundary; `random` has no specification file and imports nothing.
     Every specification module imports `sudoku`.
   - The import rule is checked by a gate for paths written `crate::`; a relative
     path, an alias of the crate and a re-export are left to review.
   - Only a solution stored in a `Puzzle` or a `Board` is hidden. `SearchResult` and
     `WellPosed` hand out whole solution grids.
   - The solver makes the proof. No consumer can; inside the crate that is a rule
     review holds.
   - The search goes on after a contradiction, a solution or a split alike, and stops
     at two solutions or when no branch waits.
   - Moves can be undone while the puzzle is unsolved; solving is final; checks are
     never undone.
   - A new board opens from givens or reopens from a record, and a board in hand can
     be cloned.
   - Opening runs the solver once, playing never does, and reopening runs it again.
   - In play, `Board::check` alone consults the stored solution; reopening consults it
     too, to answer a record's checks again.
   - Placing keeps the cell's own note beneath the digit and erasing shows it again;
     erasing gives peers nothing back; undo returns a struck mark to exactly the notes
     that lost it.
   - A draw is a number in `[0, 1)` or a `RandomError`. `SeededStream` always has one;
     `ReplayStream` refuses a script out of range and reports when it runs out.
   - Well-posed is exactly one solution; a puzzle to play also needs a cell left.
   - A record's erasure holds only its cell. A record carries no version, and
     reopening across versions of the engine is not promised.
   - Reopening replays moves through the board's own operations; a record's checks are
     tested on their own and are not matched to the moves that are left.
   - Loading a record checks at most its shape; `Board::reopen` judges it as a game.
   - `generate` makes a puzzle from a tier and a stream of draws, a tier claims nothing
     about difficulty, and `GenerateError::NotProved` is reached by no tier and no
     stream.

6. Run `just doc`, `just test-doc` and `just fmt-check`. Serve `target/doc` on
   localhost and look at both pages in light, dark and ayu, as S10's step 5 did. Fix
   what that shows.

7. **Stop.** Open both pages in the maintainer's browser and ask which stays, and, if
   the SVG page stays, whether the Mermaid loader stays in the header for a later
   page.

8. Remove the page not chosen and its line in `lib.rs`. If the Mermaid page stays it
   becomes `guide.md` and `guide.rs`, and the figures' stylesheet leaves the header.
   If the SVG page stays and the loader goes, the header holds the stylesheet alone,
   and no page of the reference asks a reader's browser for a third-party script.

9. The handbook. `docs/reference/api.md` names the header file under "Building it",
   gains a section on pages and figures with the rule for what a page in the reference
   may hold, and says the docs.rs table passes the same header by a path of its own.
   `docs/how-to/add-a-reference-page.md` is new, with its manifest entry and its link
   in `docs/README.md` in the same commit. The decision record is the next free number,
   0017, with its row and the "next decision" sentence. Sweep for every sentence that
   quotes the `doc` recipe's flags or lists the crate's files, and reword what the
   change makes false:

   ```sh
   rg -n 'RUSTDOCFLAGS|cfg docsrs|just doc' docs AGENTS.md README.md crates/pawdoku/README.md
   ```

   If the Mermaid loader ships, `docs/how-to/deploy-to-github-pages.md`,
   `docs/explanation/security-model.md` and `SECURITY.md` each gain the sentence S10's
   step 10 drafts about the pinned, hashed script. `CHANGELOG.md` gains its line.

10. Prove the probe: take the flag out of the recipe, run `just doc`, see it fail on
    the probe's line, put the flag back. Quote the output.

11. `just check`, then `just site` and a last look at the page. Fill the hand-back
    notes, set `status: done` here and in `tickets/README.md`, commit on the ticket
    branch, and stop before pushing.

## Acceptance criteria

- `just check` exits 0.
- `just test-doc` reports the guide's example as run and passing.
- The page was seen in light, dark and ayu, and the notes say who saw it and how.
- A build with none of the gate's flags, `cargo doc -p pawdoku --no-deps --all-features`,
  draws the figures as the gate's build does: the page carries its own styles.
- `cargo package --list -p pawdoku --allow-dirty` lists the guide's source, so the
  packaged crate carries what docs.rs would build.
- One guide page is committed, not two, and `lib.rs` declares it under `cfg(doc)`.
- The `Justfile` and `crates/pawdoku/Cargo.toml` are unchanged from `main`.
- `git status --porcelain` is empty after the commits.

## Verification

```sh
just check
just test-doc 2>&1 | rg 'guide'
rg -n 'cfg\(doc\)' crates/pawdoku/src/lib.rs
rg -c '<style>' crates/pawdoku/src/guide.rs
git diff --stat main -- Justfile crates/pawdoku/Cargo.toml
cargo package --list -p pawdoku --allow-dirty | rg 'src/guide'
git status --porcelain
```

Expected: exit 0; one line naming the guide's example as `ok`; one `cfg(doc)` line; a
count of one; no output, because neither file differs from `main`; the guide's source;
nothing.

## Hand-back notes

### Built 2026-10-03 to 2026-10-05

The ticket was written and built in one pass, on the maintainer's instruction of
2026-10-03, on the branch `ticket/t34-guide-page-and-diagrams` from `main` at `f580c02`.
The worktree is the Supacode worktree `show-me`, not one named for the ticket: it was
clean and level with `main`, so the branch was made in it. A recorded deviation.

**Leave given by the maintainer on 2026-10-03, before the act:** the three frozen files
(`Justfile`, `docs/manifest.yml`, `docs/README.md`) edited here; the tool environment
from caches with the network as a fallback; one fetch of the pinned Mermaid file; a
headless Chrome loading it from jsDelivr. The environment came from the caches behind a
dead proxy (`HTTPS_PROXY=http://127.0.0.1:9 just sync`), with `.tools/bin` copied from
the `docs-site` worktree, so the fallback was not needed.

**The Mermaid pin.** The maintainer chose 12.1.0 over S10's recommended 11.17.2. The
single file was fetched once from jsDelivr on 2026-10-04: 5,493,176 bytes, matching S10's
figure, `sha384-EbBpjO7rlR6eqZEcG7GaPpyk9H9WrMyPWX4d3KvPYltgt8Z8l0z6R56B1qP40pR4`.

**Two pages were built** (step 4) and both passed `just doc`, `just test-doc`,
`just clippy`, `just metrics`, `just lint` and `just fmt-check`, with every intra-doc
link resolving and each page's example reported `ok`. Both were served from `target/doc`
on `127.0.0.1:8734` and screenshotted by the installed Chrome run headless, with the
theme set through rustdoc's own stored setting by a scratch page, as S10 did; the agent
read the screenshots. The SVG page was seen in light, dark and ayu, the Mermaid page in
light and dark. Then both were opened in the maintainer's browser.

Rendering the Mermaid page inside rustdoc needed two corrections to S10's loader, each
found from a screenshot:

- Rustdoc wraps a fence in `div.example-wrap`, a flex row. A figure inserted after the
  `pre` inside it has no width of its own and shrinks: every diagram came out about 300
  pixels wide. The figure goes after the wrapper, and the wrapper is hidden with
  `style.display = "none"` (the `hidden` attribute loses to rustdoc's `display: flex`).
- Rustdoc's stylesheet defines `.hidden { display: none !important }`, so a Mermaid
  `classDef` named `hidden` made its nodes vanish. Class names must avoid rustdoc's.

With those, and `flowchart: { wrappingWidth: 320 }` so that labels did not wrap at
Mermaid's 200 pixels, every diagram drew.

**The pick, 2026-10-05.** The maintainer chose the SVG page, and chose to remove the
Mermaid loader from the header rather than keep it for a later page. The Mermaid page,
`guide_mermaid.md` and `guide_mermaid.rs`, and its `lib.rs` line were removed before
anything was committed; the header holds the stylesheet alone, so no page of the
reference loads a third-party script and the security pages needed no sentence. The
loader as it stood when the comparison was made, for the page that brings it back:

```html
<script>
  document.addEventListener("DOMContentLoaded", () => {
    const fences = [...document.querySelectorAll("pre.language-mermaid")].map((pre, index) => {
      const block = pre.closest(".example-wrap") ?? pre;
      const figure = document.createElement("div");
      figure.className = "mermaid-diagram";
      block.after(figure);
      return { block, figure, source: pre.textContent, id: `mermaid-diagram-${index}` };
    });
    if (fences.length === 0) {
      return;
    }
    async function draw() {
      const dark = document.documentElement.dataset.theme !== "light";
      window.mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: dark ? "dark" : "default", flowchart: { wrappingWidth: 320 } });
      for (const { block, figure, source, id } of fences) {
        try {
          const { svg } = await window.mermaid.render(id, source);
          figure.innerHTML = svg;
          block.style.display = "none";
        } catch (error) {
          figure.replaceChildren();
          block.style.display = "";
          console.error(error);
        }
      }
    }
    const script = document.createElement("script");
    script.src = "https://cdn.jsdelivr.net/npm/mermaid@12.1.0/dist/mermaid.min.js";
    script.integrity = "sha384-EbBpjO7rlR6eqZEcG7GaPpyk9H9WrMyPWX4d3KvPYltgt8Z8l0z6R56B1qP40pR4";
    script.crossOrigin = "anonymous";
    script.addEventListener("load", async () => {
      await draw();
      new MutationObserver(draw).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    });
    document.head.append(script);
  });
</script>
```

with, in the stylesheet, `.mermaid-diagram { margin: 1em 0; overflow-x: auto; }` and
`.mermaid-diagram svg { max-width: 100%; height: auto; }`.

**The figures' fonts.** Rustdoc's `--font-family` variable is its serif body face, which
made the figures' labels thin at eleven to thirteen pixels. The stylesheet names
rustdoc's own sans face, `"Fira Sans"`, for the figures, and its code face through
`--font-family-code`.

**The probe** (step 10), as built before the review below took the header out. With
`--html-in-header` taken out of the recipe:

```text
$ just doc
RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --workspace --no-deps --all-features --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
   Generated .../target/doc/pawdoku/index.html
grep -q 'pawdoku-rustdoc-header' target/doc/pawdoku/guide/index.html || { ... }
doc: target/doc/pawdoku/guide/index.html lacks the header file; is --html-in-header passed?
error: recipe `doc` failed on line 222 with exit code 1
```

With the flag put back, `just doc` exits 0.

**What was corrected in the page** (step 5): every line of that list is in the page's
text or a figure, and the sixteen findings of the first draft's review are each
answered. `generation` is drawn as built, with a component figure of its own; the
figures that showed four built modules show five.

**The handbook.** `docs/reference/api.md` gained the section "Pages and figures" and a
word in the docs.rs sentence; `docs/how-to/add-a-reference-page.md` is new; decision
0017 is new, and "the next decision" is 0018; `CHANGELOG.md` gained its line and its
counts moved to seventeen records and twenty-six pages. The sweep of step 9 reworded
`docs/project/repository-map.md` (the new file) and `docs/explanation/architecture.md`
(the module that is not behaviour). While the header stood, `docs/reference/api.md`'s
"Building it", `docs/reference/configuration.md`, `docs/reference/commands.md` and
`docs/reference/quality-gates.md` carried the flag and the probe too; those sentences
went back to `main`'s with the header.

### Codex adversarial review, 2026-10-05

Run by the plugin's `codex-companion.mjs adversarial-review` on the branch against
`main`, after the first commit, `2a7d317`. One finding, medium: **a consumer's own
`cargo doc` renders the figures unreadable.** The guide module exists under `cfg(doc)`,
so rustdoc builds it when a project that depends on `pawdoku` documents itself, and that
build gets no header: Cargo ignores `[package.metadata.docs.rs]` and never runs this
repository's `Justfile`. An unstyled SVG draws its boxes solid black, with the labels
lost in them. Codex could not build to confirm it (a read-only sandbox); the mechanism
was confirmed here.

The maintainer chose, among four answers offered, to make the page self-contained. The
stylesheet moved into `guide.rs` as a `style` block, which rustdoc passes through and
`-D warnings` accepts; the header file, the flag in both spellings and the probe were
removed, and the `Justfile` and `crates/pawdoku/Cargo.toml` are as `main` has them, so
no frozen file changed after all. A build with none of the gate's flags,
`cargo doc -p pawdoku --no-deps --all-features --locked`, was screenshotted in dark: the
figures draw as the gate's build draws them. Decision 0017 records the header as turned
down, and the how-to says where the loader would go if a page ever needs a script.

**Verification**, run on 2026-10-05 in this worktree, after the review was answered:

```text
$ just check
...
All checks passed and the worktree is unchanged.
$ just test-doc 2>&1 | rg 'guide'
test crates/pawdoku/src/guide.rs - guide (line 216) ... ok
$ rg -n 'cfg\(doc\)' crates/pawdoku/src/lib.rs
86:#[cfg(doc)]
$ rg -c '<style>' crates/pawdoku/src/guide.rs
1
$ git diff --stat main -- Justfile crates/pawdoku/Cargo.toml
$ cargo package --list -p pawdoku --allow-dirty --offline | rg 'src/guide'
src/guide.rs
$ git status --porcelain
```

### Deviations, and why

- **One pull request, ticket and build together,** on the maintainer's instruction.
  CONVENTIONS.md §11 has a ticket written before it is picked up.
- **The worktree keeps the name `show-me`.** See above.
- **Headless Chrome read by the agent,** as in S10, and the maintainer's own browser for
  the pick. The Mermaid page was not seen in ayu; S10 found dark and ayu draw alike.
- **A sans face named in the stylesheet,** where the plan had rustdoc's variable.
- **The page was generated once** from an HTML draft by a scratch script, then edited
  as a file. The committed `guide.rs` is the source; nothing generates it again.
- **The header file was built and then removed.** Steps 2, 3 and 10 describe it as the
  ticket was written; the adversarial review above is why the page carries its own
  styles instead, and why the two frozen-file edits the ticket asked leave for were not
  needed in the end.

## Open points

- **Which page stays** is the maintainer's, at step 7.
- **Whether the Mermaid loader stays** when the SVG page is kept. Removing it means no
  page of the reference loads a third-party script until a page has a fence, and the
  loader returns with that page from S10's notes. Keeping it means the next page with a
  fence needs no change to the header.
- **A hand-drawn figure is edited by its coordinates.** When a module is built or an
  operation changes, the figure that shows it changes by hand. The how-to says where
  each figure is and how to look at the result.
- **Nothing ties a figure's text to the code.** Prose names are intra-doc links and
  fail the gate when an item is renamed. A name inside a figure is plain text and does
  not. The page's example and its links are what the gate holds.
