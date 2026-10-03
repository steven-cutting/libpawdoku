---
id: S10
title: "Spike: prose pages, diagrams and raw HTML inside rustdoc"
status: open
depends_on: [T11]
parallel_with: [T33]
branch: ticket/s10-pages-and-diagrams-in-rustdoc
estimated_size: S
---

# S10: Spike: prose pages, diagrams and raw HTML inside rustdoc

## Context

T33 puts the API reference on GitHub Pages, and the maintainer decided on 2026-10-02
that the site is rustdoc and nothing else: no site generator, and the handbook under
`docs/` stays on GitHub. The same day the maintainer asked that the site be able to
carry more than the API: pages of prose, diagrams, and raw HTML where Markdown runs out.
So those pages live inside rustdoc, and this spike finds out how.

Two things are settled and are not reopened here:

- **Rustdoc only.** A mechanism that needs a second generator is out.
- **A pinned script from a public CDN is acceptable.** The maintainer accepted, on
  2026-10-02, that a docs page may load Mermaid from jsDelivr at a pinned version in the
  reader's browser. A reader with no network, or with scripts off, then sees the
  diagram's source text.

The handbook already draws with Mermaid: `docs/explanation/human-solving.md` line 63 and
`docs/explanation/solving-sudoku.md` line 388 each open a `mermaid` fence, which GitHub
renders. GitHub renders `mermaid`, `geojson`, `topojson` and `stl` fences and no other
(<https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams>).
A sudoku grid is the other thing this library draws, and the handbook draws it as a
`text` fence (`docs/explanation/solving-sudoku.md` line 66, `docs/reference/testing.md`
line 87).

What this repository puts in the way, each read in the file named:

- **Raw HTML is refused in every Markdown file.** `.markdownlint-cli2.jsonc` line 11
  sets `MD033` to allow `br` alone, and its glob is `**/*.md`, so a `.md` file inside
  the crate is linted like any other. A doc comment in a `.rs` file is not Markdown to
  that hook.
- **A `.md` under `docs/` is a handbook page.** `bg-validate-docs` is
  `src/biscuit_games_tooling/validate_docs.py` in B at `6c5c07f6` (the letters are
  CONVENTIONS.md §0's). Its lines 278 to 284 read every `docs/**/*.md` on disk, and each
  must carry the five-key frontmatter, which rustdoc would print as text. Its lines 33
  to 40 refuse `{{`, `{%` and `{#` anywhere in a registered page, fences included, and a
  Mermaid hexagon node is written `id{{text}}`. A page for rustdoc therefore does not
  live under `docs/`.
- **A file the crate includes must be inside `crates/pawdoku/`.** `cargo package`
  packs the crate's own directory, and docs.rs builds from the package.
- **The metrics gate does not count documentation.** `rustqual.toml` line 53 sets
  `file_length = 500`, and line 52 says "comments, docs and blanks are not counted". A
  long page written as doc comments is not a long file to gate 7. Verify it.
- **Every text file meets `typos` and `editorconfig-checker`,** and a new file may not
  exceed 768 kB (`.pre-commit-config.yaml` line 93). `lychee` reads Markdown links
  offline, so a relative link must name a file that exists.
- **`just doc` is gate 12** and runs with `RUSTDOCFLAGS="-D warnings --cfg docsrs"`. The
  rustdoc lints in `Cargo.toml` lines 37 to 41 deny broken and private intra-doc links.
  `crates/pawdoku/Cargo.toml` lines 30 to 33 give docs.rs `--cfg docsrs` and two
  targets.
- **A dependency costs every consumer unless it is optional.** `serde` is the crate's
  one optional dependency, behind a feature of its name (`crates/pawdoku/Cargo.toml`
  lines 19 to 23). A macro crate for diagrams could sit behind a feature the same way,
  so what it costs is not every consumer's build. It is a public feature that exists
  for documentation alone, a procedural macro in the graph cargo-deny reads and every
  `--all-features` gate compiles, an attribute on each documented item, and the note
  decision 0007 asks for with any dependency.

Facts from the tools' documentation and registries, read on 2026-10-02 and not run.
Step 2 verifies each and records its source:

- `--html-in-header`, `--html-before-content`, `--html-after-content` and `--extend-css`
  are stable rustdoc flags
  (<https://doc.rust-lang.org/rustdoc/command-line-arguments.html>).
- Stable rustdoc cannot ship a local image or any other asset file: rust-lang/rust
  issue 32104, "Include images in rustdoc output", is open. What a page shows is its
  own HTML, a data URI, or something fetched by address.
- Rustdoc passes raw HTML in documentation through, and has a lint,
  `rustdoc::invalid_html_tags`, for tags left open.
- docs.rs passes the `rustdoc-args` of `[package.metadata.docs.rs]` to rustdoc, and
  builds the packaged crate, where the package's root is the working directory. In this
  workspace `cargo doc` runs from the workspace root. Cargo does not document which
  directory a relative path in `RUSTDOCFLAGS` is resolved against.
- Mermaid 12.1.0 was released on 2026-10-02. 12.0.0 (2026-09-10) raised its baseline to
  ES2024 and Safari 17.4 and changed its default layout and colours. On jsDelivr the
  ESM entry is about 31 kB and loads the rest in pieces; the single-file
  `mermaid.min.js` is about 5.5 MB. `securityLevel` defaults to `strict`.
- `aquamarine` 0.6.0 (2024-10-08, no commit since) is a procedural macro. While it
  expands it writes Mermaid's files into `target/doc/static.files.mermaid/` if
  `target/doc` exists, and otherwise links `mermaid@11.1` on unpkg. `simple-mermaid`
  0.2.0 (2024-12-04) is a `no_std` declarative macro that loads `mermaid@11` from
  jsDelivr when the page is viewed.
- Mermaid's own command line, `@mermaid-js/mermaid-cli`, drives a headless Chrome
  through puppeteer; the conda-forge package (11.17.0) is built without one. `mmdr`
  0.3.1 renders Mermaid in Rust and describes itself as "under active early
  development".
- D2 v0.9.0 (2026-09-07, MPL-2.0) is one Go binary with a grid layout; conda-forge has
  0.7.1. Graphviz 14.1.2 and Typst 0.15.1 are on conda-forge. PlantUML needs a JVM.
  Kroki is a server.

The mechanisms, and the verdict each starts from:

| Mechanism | What it is | What it costs | Verdict to start from |
| --- | --- | --- | --- |
| A page as an included file | A documentation-only module whose text is a `.md` file inside `crates/pawdoku/`, pulled in with `#![doc = include_str!(...)]` | Linted as Markdown, so no raw HTML beyond `br`; GitHub also renders the file | candidate, first choice for prose |
| A page as doc comments | The same module with its text written as `//!` lines in the `.rs` file | No Markdown hook reads it; raw HTML is free; harder to read as source | candidate, where a page needs raw HTML |
| A standalone `rustdoc page.md` | Rustdoc run on a Markdown file by itself | Cargo does not drive it; outside the sidebar and the search; absent from docs.rs | not at all |
| A plain `.html` file beside the reference | A file `just site` copies into `target/doc` | On Pages only, never on docs.rs; no navigation to it | on a trigger: a page rustdoc cannot express |
| A handbook page included in rustdoc | `include_str!` of a file under `docs/` | Outside the package; its frontmatter prints as text | not at all |
| Mermaid in the browser | A header file given to `--html-in-header` that loads a pinned Mermaid from jsDelivr and renders every `mermaid` fence | A third-party script on every page; one more flag in `doc` and in the docs.rs table | candidate, first choice for diagrams |
| `aquamarine` or `simple-mermaid` | A macro on each documented item, as an optional dependency behind a feature for documentation | A public feature that is not the engine's; a macro crate in the graph and in every `--all-features` gate; the first has had no commit since 2024-10 and writes files while it expands; both draw with the same Mermaid script in the browser, which the header file loads with no dependency | not at all, unless step 5 finds something the header file cannot do and a macro can |
| A vendored `mermaid.min.js` | The script committed and copied into the site | 5.5 MB against the 768 kB ceiling; docs.rs could not serve it | not at all |
| Mermaid's command line | SVG made before publishing | A headless Chrome in the toolchain | not at all |
| `mmdr` | SVG made before publishing from the same `mermaid` source, by a renderer written in Rust, with no browser | Young: it calls itself "under active early development" and says its output may differ from Mermaid's; an install route to settle; generated files to keep byte-stable | on a trigger: a requirement that pages work with scripts off, and then the first to try, because the source stays Mermaid |
| D2, Graphviz or Typst with CeTZ | SVG made before publishing by a pinned tool, inlined in the page | A tool to pin, generated files to keep byte-stable, and sources GitHub does not render | on the same trigger, after `mmdr` |
| Raw HTML or inline SVG by hand | A table or an `svg` element in the documentation text | Written and themed by hand | candidate, for a sudoku grid |
| A `text` fence | The grid as characters, as the handbook has it | None | already in use; keep |
| Kroki, PlantUML | A rendering server; a JVM | A service or a runtime | not at all |

Read first: `AGENTS.md`; `tickets/CONVENTIONS.md` §1, §11 and §13;
`tickets/T33-api-reference-on-github-pages.md`; `docs/reference/api.md`;
`docs/reference/documentation-contract.md`; `docs/decisions/0007-dependency-policy.md`
and `docs/decisions/0009-rust-quality-gate.md`; the `Justfile`'s `doc` recipe;
`crates/pawdoku/Cargo.toml` and `crates/pawdoku/src/lib.rs`;
`tickets/S09-code-quality-metrics.md` as the shape.

## Goal

A verdict for every row of the table: adopt now, adopt on a named trigger, or not at
all. One sample page with one diagram and one grid, built in a scratch copy with the
gate's own flags, and the header file that draws them, both quoted in the hand-back
notes. The rule for what belongs in a rustdoc page and what stays in the handbook. A
drafted build ticket for whatever is adopted. Nothing in the repository changes outside
this file and its row in the ticket index, and nothing is installed outside the scratch
copy of step 3, as in S09.

The starting recommendation, which the spike proves or overturns:

- **Pages** are documentation-only modules. The text is a Markdown file inside
  `crates/pawdoku/`, included with `include_str!`, so that `cargo package` keeps it and
  docs.rs shows it. A page that needs raw HTML is written as doc comments instead.
- **Diagrams** are `mermaid` fences, the syntax the handbook already uses, rendered in
  the reader's browser by one small header file: Mermaid at an exact version from
  jsDelivr, `securityLevel` left at `strict`, following rustdoc's theme. The flag joins
  the `doc` recipe and the docs.rs table together, so the hosted reference and docs.rs
  show the same thing.
- **A sudoku grid** is a `text` fence where that is enough, and a hand-written HTML
  table or inline SVG where cells need marking. Mermaid draws a grid badly.
- **Build-time SVG** waits for a trigger, with `mmdr` first in line because it reads the
  same source. **The macro crates, the vendored script and Mermaid's command line** are
  not adopted. The macro crates are turned down on what they add to the crate's
  features and its graph, not on a claim that every consumer would pay for them.

## Non-goals

- Adding a module, a page, a header file, a flag, a recipe, a dependency or a decision
  record: the follow-up does that, with the `Justfile` change as a T00 follow-up.
- A site generator, or publishing the handbook. Decided on 2026-10-02.
- Publishing anything, or touching the Pages settings. T33 owns the site.
- Moving or redrawing the handbook's two diagrams.
- Deciding which pages get written. The follow-up names one sample page; the catalogue
  is the maintainer's.

## Files touched

| Path | Class | Change |
| --- | --- | --- |
| `tickets/S10-pages-and-diagrams-in-rustdoc.md` | ticket | The evidence, the verdicts, the rule, the drafted follow-up; `status: done` |
| `tickets/README.md` | ticket index | S10's row in the spike table |

## Steps

1. Create the worktree on `ticket/s10-pages-and-diagrams-in-rustdoc` from `main`
   (`tickets/README.md`, "How to pick up a ticket"). If `.pixi/` is absent,
   `just initialize` installs it over the network, which is a separately authorised
   action: ask first.

2. Verify, read-only, every fact in the Context that names a tool, a flag or a version:
   against the rustdoc book, crates.io's API, the projects' release pages, jsDelivr and
   `pixi search`. Record version, date and source for each. Where a fact has moved,
   correct the table before going on.

3. Make the scratch copy outside this worktree, in the session's scratch directory, as
   S09 did. `ai_tmp/` will not do. It is inside the repository, so `git` there finds
   this repository: `just lint` and the `bg-*` tools would take this tree for the root
   and read its files, not the copy's, and every hook's configuration leaves `ai_tmp/`
   out. The copy has to be a repository of its own, with the gate's tools:

   - `git archive HEAD | tar -x -C <scratch>`, then `git init` and `git add -A` there.
     No commit is needed. The hooks read what Git lists, so stage each experimental
     file before a hook is expected to see it.
   - `just sync` there installs the copy's own `.pixi/` from `pixi.lock`. If pixi's
     cache cannot serve it, that is the network: ask first. Copy `.tools/bin/` from
     this worktree.
   - Prove the copy's gate is live before trusting a result from it. Put a raw HTML
     element in a Markdown file there, stage it, and see `just lint` fail on that file;
     then take it out.

   Every experiment below runs in the copy. Nothing is installed into this worktree's
   `.pixi/` or `.tools/`, and nothing anywhere outside the scratch directory.

4. The page, both ways. In the scratch copy add a public module, `guide`, once with its
   text in a `.md` file beside the source and once as doc comments. For each, run the
   scratch copy's `just doc`, `just test-doc`, `just clippy`, `just metrics`,
   `just features`, `just wasm-check` and `just lint`, and record:

   - where the module appears in the sidebar and the search;
   - whether an example in the page runs under `just test-doc`, with the module
     compiled always and with it under `#[cfg(doc)]`;
   - whether an always-compiled empty module changes anything a consumer can see beyond
     the page;
   - which hooks read the `.md` file and what each says about an intra-doc link such as
     a type's name in square brackets;
   - what `cargo package --list -p pawdoku` prints for the `.md` file, and whether
     `publish = false` lets that command run at all.

   Plant three failures, one at a time, and record that each fails and where: a broken
   intra-doc link in the page (`just doc`), an example that does not compile
   (`just test-doc`), and an HTML tag left open (`just doc`, if the lint is live on the
   pinned toolchain).

5. The diagram. Write the header file and add `--html-in-header` to the scratch copy's
   `doc` recipe. Put a `mermaid` fence in the page. Record:

   - that `just test-doc` does not try to compile the fence;
   - the element and class the pinned rustdoc (1.98.1) writes for it, which is what the
     header's script must find;
   - that `-D warnings` stays quiet;
   - what the page looks like in rustdoc's light, dark and ayu themes, served from the
     scratch `target/doc` on localhost, and after a change of theme without a reload;
   - what it looks like with the CDN unreachable.

   If no browser is to hand, give the maintainer the command that serves the scratch
   copy and record what they report. Do not guess at a rendering.

   Then the path. Find the spelling of the header's path that works in both builds:
   `just doc` from the workspace root, and the docs.rs build, imitated by packaging the
   crate into the scratch directory, unpacking it, and running `cargo rustdoc` there
   with the arguments `[package.metadata.docs.rs]` would give. Quote both commands and
   their results. If one spelling cannot serve both, say what each build needs.

   Then the version. Try the newest 11 and the newest 12 at exact versions. Record
   which browsers each rules out, whether a subresource-integrity hash can cover what
   is loaded (the ESM entry loads further files; the single file is one), and name one
   published crate whose docs.rs pages already load a script from a CDN through this
   flag, as evidence that docs.rs serves it.

6. The grid and raw HTML. Draw one grid three ways in the page: a `text` fence, an HTML
   table styled by a `style` element in the header file, and an inline `svg` that uses
   `currentColor`. Record how each reads in the three themes and under `-D warnings`.
   Then copy one plain `.html` file into the scratch `target/doc` as `just site` would,
   and say what it would take to reach it from the reference.

7. The routes not taken, each turned down or kept on evidence:

   - The macro crates, read-only. From each crate's source and crates.io: what it emits
     and what it loads; whether it can sit behind an optional feature; and what
     `just features`, `just clippy`, `just wasm-check` and `cargo deny` would then see.
     Say whether anything in step 5 was out of the header file's reach and in a
     macro's.
   - `mmdr`. Its version, licence and install route: a release binary would be a
     `tools.txt` line, a source build a `tools-source.txt` line, which decision 0013
     accepted for one tool. Then install it into the scratch directory, which reaches
     the network, so ask first, and render the handbook's two diagrams with it twice.
     Record whether the output is legible, whether the two runs are byte-identical, and
     where it differs from what GitHub draws.
   - D2, Graphviz and Typst, read-only: what `pixi search` offers on both platforms the
     manifest names. Install none of them unless a verdict turns on it; if one is
     tried, it goes in the scratch directory.

8. Write the rule for what belongs where. Start from this and correct it against
   `docs/reference/documentation-contract.md` and `docs/reference/api.md`: the handbook
   explains the project and its subject and is where a contributor reads; a rustdoc
   page is for someone using the crate, and its examples compile. A topic has one home,
   and the other side links to it by address. Say how a rustdoc page links to a
   handbook page (an address on GitHub), and how the handbook links to the hosted
   reference.

9. Give the verdicts: per row, with the evidence and its date.

10. Draft the follow-up in the hand-back notes, under the next free ticket id, depending
    on T33: the header file's path and full text; the flag in the `doc` recipe (a
    `Justfile` change, so a T00 follow-up) and in `[package.metadata.docs.rs]`; the
    sample page and its module; the sentences that change in `docs/reference/api.md`,
    `docs/how-to/deploy-to-github-pages.md`, `docs/explanation/security-model.md` and
    `SECURITY.md`, which today name no third-party script; a how-to for adding a page or
    a diagram, if the rule needs one; the decision record, new or an amendment to
    T33's; the `CHANGELOG.md` line.

11. Set `status: done` here and in `tickets/README.md`, commit on the ticket branch,
    and stop before pushing.

## Acceptance criteria

- Every fact in the Context that names a tool, a flag or a version has a source and a
  date, or is corrected.
- The scratch copy was a repository of its own with the gate's tools, and a planted
  Markdown violation failed `just lint` there before any result from it was recorded.
- The sample page, with its diagram and its grid, builds in the scratch copy with the
  gate's `doc` flags and the header file, exit 0. The header file and the flags are
  quoted in full. The rendering in three themes is described by whoever saw it, and the
  notes say who.
- The docs.rs question is answered with the two commands and their results.
- The three planted failures each fail, with the message quoted, or the failure to fail
  is recorded as a finding.
- Every row of the table has a verdict of adopt now, adopt on a named trigger, or not
  at all.
- The rule for what belongs where is one paragraph a contributor can apply.
- The follow-up is drafted, lists every frozen-file change as a T00 follow-up, and adds
  no dependency.
- `git status --porcelain` on the ticket branch lists only this file and
  `tickets/README.md`.

## Verification

```sh
rg -n '"MD033"' .markdownlint-cli2.jsonc
rg -n -B1 '^file_length' rustqual.toml
rg -n 'rustdoc-args' crates/pawdoku/Cargo.toml
rg -n '^```mermaid' docs
rg -n 'RUSTDOCFLAGS' Justfile
git status --porcelain
```

Expected: the `allowed_elements` line naming `br` alone; `file_length = 500` under the
comment that documentation is not counted (and the test-file limit after it); the
`--cfg docsrs` arguments; two fences, in `human-solving.md` and `solving-sudoku.md`;
the `doc` recipe's one line; two lines naming this file and `tickets/README.md`.

## Hand-back notes

None yet.

## Open points

- **Which Mermaid.** 12 drops browsers older than Safari 17.4 and redraws with new
  defaults; 11 is the line the macro crates and the other documentation tools still
  load. GitHub renders the handbook's two diagrams with whatever version it runs. The
  spike recommends one exact version; the maintainer chooses.
- **Markdown file or doc comments.** The first choice keeps prose in a file that GitHub
  also renders and every Markdown hook reads, and so cannot hold raw HTML. Relaxing
  `MD033` for one directory is a change to a dotfile every page is held to, and is the
  maintainer's call if the spike finds pages need it.
- **Hosted site and docs.rs alike.** The recommendation assumes a page must render the
  same in both, which rules out anything `just site` adds after rustdoc has run. If the
  hosted site may carry more than docs.rs, the plain `.html` row and the vendored script
  read differently.
- **A banner for `main`.** T33 leaves the site saying nowhere that it documents `main`
  and no release. `--html-before-content` could say so on the hosted build alone. It is
  in reach of the same flag change; whether it is wanted is the maintainer's.
- **The reader's browser calls jsDelivr.** Accepted on 2026-10-02 for the hosted
  reference. `docs/explanation/security-model.md` and `SECURITY.md` describe a library
  with no network at runtime and say nothing of its documentation; the follow-up words
  the sentence, and the maintainer may want it in the decision record as well.
- **Whether the handbook's diagrams are repeated in rustdoc.** Not by default: one home
  for a topic. The rule of step 8 decides, and the maintainer may want
  `docs/explanation/solving-sudoku.md`'s flowchart in front of the crate's users anyway.
