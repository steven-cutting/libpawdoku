---
id: S10
title: "Spike: prose pages, diagrams and raw HTML inside rustdoc"
status: done
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
  `mermaid.min.js` is about 5.5 MB in 12.1.0 and about 3.6 MB in 11.17.2, the newest 11
  (corrected in step 2). `securityLevel` defaults to `strict`.
- `aquamarine` 0.6.0 (2024-10-08, no commit since) is a procedural macro. While it
  expands it writes Mermaid's files into `target/doc/static.files.mermaid/` if
  `target/doc` exists; the page it writes tries those files and falls back to
  `mermaid@11.1` on unpkg when it is read (corrected in step 2). `simple-mermaid`
  0.2.0 (2024-12-04) is a `no_std` declarative macro that loads `mermaid@11` from
  jsDelivr when the page is viewed.
- Mermaid's own command line, `@mermaid-js/mermaid-cli`, drives a headless Chrome
  through puppeteer; the conda-forge package (11.17.0) is built without one, and has no
  `osx-arm64` build (added in step 2). `mmdr`
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
| A vendored `mermaid.min.js` | The script committed and copied into the site | 3.6 MB to 5.5 MB against the 768 kB ceiling; docs.rs could not serve it | not at all |
| Mermaid's command line | SVG made before publishing | A headless Chrome in the toolchain | not at all |
| `mmdr` | SVG made before publishing from the same `mermaid` source, by a renderer written in Rust, with no browser | Young: it calls itself "under active early development" and says its output may differ from Mermaid's; a release binary with no checksum; generated files to keep byte-stable | on a trigger: a requirement that pages work with scripts off, and then the first to try, because the source stays Mermaid |
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

### What was verified, and how

Run on 2026-10-03 in the Supacode worktree for this ticket, from `main` at `b4eecc2`.
No tracked file changed except this one and `tickets/README.md`. Every experiment ran in
a scratch copy in the session's scratch directory, outside this worktree. Nothing was
installed into this worktree's `.pixi/` or `.tools/`.

The maintainer authorised, on 2026-10-03, before any step: read-only network reads (the
rustdoc book, crates.io, npm, GitHub, jsDelivr, `pixi search`); `just sync` in the
scratch copy from the caches, with the network as a fallback; `mmdr` installed into the
scratch directory only; and the agent looking at the rendered pages itself.

```text
$ git rev-parse --short HEAD
b4eecc2
$ rustup show active-toolchain
1.98.1-aarch64-apple-darwin (overridden by '.../rust-toolchain.toml')
$ rustdoc --version
rustdoc 1.98.1 (48a229cea 2026-09-01)
```

**Step 1.** The worktree existed, with `.pixi/` and the four binaries of `.tools/bin`,
so `just initialize` was not needed.

**Step 2. The Context's facts.** Each read on 2026-10-03. Three moved, and the Context
above is corrected where it stated them.

| Fact | Result | Source |
| --- | --- | --- |
| The four `--html-*` and `--extend-css` flags are stable | confirmed | <https://doc.rust-lang.org/rustdoc/command-line-arguments.html>; none is on the unstable-features page |
| rust-lang/rust issue 32104 is open | confirmed, last updated 2026-02-12 | `gh api repos/rust-lang/rust/issues/32104` |
| `rustdoc::invalid_html_tags` | confirmed, and it warns by default, so `-D warnings` makes it an error | <https://doc.rust-lang.org/rustdoc/lints.html>; step 4 saw it fire |
| docs.rs passes `rustdoc-args` | confirmed; the page says nothing of the directory a relative path is read from | <https://docs.rs/about/metadata> |
| Cargo does not document where a path in `RUSTDOCFLAGS` resolves | confirmed undocumented; step 5 measured it | the Cargo book's configuration, environment and `cargo doc` pages |
| Mermaid 12.1.0 on 2026-10-02, 12.0.0 on 2026-09-10 | confirmed; the newest 11 is 11.17.2, 2026-08-25 | `https://registry.npmjs.org/mermaid` |
| 12.0.0 raised the baseline and changed layout and colours | confirmed: ES2024 and Safari 17.4; ELK is the default layout; a new default look | the `mermaid@12.0.0` release notes |
| ESM entry about 31 kB, single file about 5.5 MB | **corrected**: 30,936 B and 5,493,176 B for 12.1.0; 30,255 B and 3,572,661 B for 11.17.2. The entry loads from about 200 further files | jsDelivr's data API, and the files downloaded and hashed |
| `securityLevel` defaults to `strict` | confirmed | `config.schema.yaml` at `mermaid@12.1.0` |
| `aquamarine` 0.6.0, 2024-10-08, no commit since | confirmed, MIT, a procedural macro with six direct dependencies. **Corrected**: it does not choose between the local files and the CDN when it expands. The page it writes tries the local files first and falls back to `mermaid@11.1` on unpkg when viewed | crates.io; `src/attrs.rs` at `v0.6.0` |
| `simple-mermaid` 0.2.0, 2024-12-04 | confirmed: `no_std`, declarative, no dependency, loads a floating `mermaid@11` from jsDelivr, one script per diagram | crates.io; `src/lib.rs` at `v0.2.0` |
| Mermaid's command line needs a headless Chrome; conda-forge has 11.17.0 without one | confirmed, and **corrected**: conda-forge has no `osx-arm64` build, so pixi could not install it on this machine | npm; `pixi search -p osx-arm64 mermaid-cli` |
| `mmdr` 0.3.1, "under active early development" | confirmed: crate `mermaid-rs-renderer`, MIT, 2026-07-06. Its release carries binaries for macOS arm64 and Linux x86_64 and no checksum file. Not on conda-forge | crates.io; the `v0.3.1` release |
| D2 v0.9.0, MPL-2.0; conda-forge 0.7.1 | confirmed on both platforms | the release; `pixi search d2` |
| Graphviz 14.1.2 and Typst 0.15.1 on conda-forge | confirmed on both platforms | `pixi search` |
| PlantUML needs a JVM; Kroki is a server | confirmed | <https://plantuml.com/starting>; Kroki's README |
| GitHub renders four fence types | confirmed. GitHub does not say which Mermaid it runs | the page the Context cites |

A crate whose docs.rs pages load a script from a CDN through this flag:
`curve25519-dalek`. Its manifest has `rustdoc-args = ["--html-in-header",
"docs/assets/rustdoc-include-katex-header.html", "--cfg", "docsrs"]`, a path relative
to the package root, and its page on docs.rs carries the header's script tag, KaTeX
0.16.3 from jsDelivr with an integrity hash.

**Step 3. The scratch copy.** `git archive HEAD | tar -x`, then `git init` and
`git add -A`, then `.tools/bin` copied from this worktree. `just sync` ran behind a dead
proxy (`HTTPS_PROXY=http://127.0.0.1:9`) and installed `.pixi/` from pixi's cache, so
nothing was fetched. `just lint` passed on the clean copy, every hook. Then the proof
that the gate is live:

```text
$ printf '\n<div>planted</div>\n' >> README.md; git add README.md; just lint
markdownlint.............................................................Failed
  README.md:100:1 error MD033/no-inline-html Inline HTML [Element: div]
```

The file was put back before anything else ran.

**Step 4. The page, both ways.** A public module `guide`, holding prose, an intra-doc
link in each form, a `mermaid` fence, a grid as a `text` fence and one example.

*As an included file:* `crates/pawdoku/src/guide.md`, and a `guide.rs` that includes it.
*As doc comments:* the same text as `//!` lines, with a grid as an HTML table and as an
inline SVG added.

| Recipe | Included file | Doc comments |
| --- | --- | --- |
| `just doc` | 0 | 0 |
| `just test-doc` | 0; `test crates/pawdoku/src/guide.md - guide (line 37) ... ok` | 0; `test crates/pawdoku/src/guide.rs - guide (line 81) ... ok` |
| `just clippy` | 0 | 0 |
| `just metrics` | 0 | 0 |
| `just features` | 0 | 0 |
| `just wasm-check` | 0 | 0 |
| `just lint` | 1, then 0 (below) | 0 |
| `just fmt-check` | 0 | 0 |

What was recorded:

- **A file whose first bytes are `#![doc = ...]` fails the gate.** The hook reads `#!`
  as a shebang:

  ```text
  check that scripts with shebangs are executable..........................Failed
    crates/pawdoku/src/guide.rs has a shebang but is not marked executable!
  ```

  With a `//` comment on the first line and the attribute on the second, `just lint`
  passed. The follow-up writes the file that way.
- **Sidebar and search.** The module is listed under Modules on the crate's page and in
  the sidebar, between `board` and `random`, and its name and its first line are in
  the search index. The summary beside it is the page's first line. The file's level-one
  heading becomes a level-two heading under rustdoc's own "Module guide", and the page's
  headings are listed under Sections in the sidebar.
- **The example runs** under `just test-doc`, with the module always compiled and with
  it under `#[cfg(doc)]` alike. Under `#[cfg(doc)]`, `doc`, `clippy`, `metrics`,
  `features` and `wasm-check` also exited 0; it was tried on the included file only.
- **What a consumer sees.** Always compiled, `pawdoku::guide` is a path a consumer can
  name: `use pawdoku::guide as _guide;` in an integration test compiled. Under
  `#[cfg(doc)]` the same line fails with `unresolved import pawdoku::guide`, so the
  module exists in the documentation and nowhere else. The module has no item either
  way.
- **The hooks and the `.md` file.** markdownlint, typos, lychee and EditorConfig all
  read it, and all passed with both link forms in it:
  `` [`Board`](crate::board::Board) `` and the shortcut `` [`crate::board::Board`] ``.
  Rustdoc resolved both to `../board/struct.Board.html`. Not seen, and so an inference:
  GitHub would show the first as a dead relative link and the second as bracketed text,
  since neither means anything outside rustdoc.
- **Packaging.** `cargo package --list -p pawdoku --allow-dirty` listed `src/guide.md`
  and `src/guide.rs`, and later `rustdoc/header.html`. `publish = false` does not stop
  the command. Without `--allow-dirty` it refuses an uncommitted tree, which the scratch
  copy is.
- **The metrics gate does not count documentation: verified.** A `guide.rs` of 715
  lines, all doc comments, passed `just metrics`. With 620 lines of constants in place
  of the padding, rustqual reported
  `SRP module length: src/guide.rs has 620 lines, 0 independent clusters`.

The three planted failures, each in the included file, one at a time:

```text
$ just doc          # See [`crate::board::Bored`].
error: unresolved link to `crate::board::Bored`
  --> crates/pawdoku/src/guide.md:53:7
   = note: requested on the command line with `-D rustdoc::broken-intra-doc-links`
error: recipe `doc` failed on line 216 with exit code 101

$ just test-doc     # board.plaice(...) in the example
test crates/pawdoku/src/guide.md - guide (line 37) ... FAILED
error[E0599]: no method named `plaice` found for struct `Board` in the current scope
  --> crates/pawdoku/src/guide.md:49:7

$ just doc          # An open <div> tag.
error: unclosed HTML tag `div`
  --> crates/pawdoku/src/guide.md:53:9
   = note: `-D rustdoc::invalid-html-tags` implied by `-D warnings`
error: recipe `doc` failed on line 216 with exit code 101
```

All three fail, each names the `.md` file and the line, and the HTML lint is live on
1.98.1.

**Step 5. The diagram.** The scratch copy's `doc` recipe became:

```text
doc:
    RUSTDOCFLAGS="-D warnings --cfg docsrs --html-in-header crates/pawdoku/rustdoc/header.html" cargo doc --workspace --no-deps --all-features --locked
```

and `crates/pawdoku/rustdoc/header.html` is the file below, in full. `just doc` exits 0
with it, `-D warnings` stays quiet, and `just lint` passes on it. EditorConfig reads it:
a comment indented by five spaces failed that hook, and two spaces passed.

```html
<!--
  Given to rustdoc with --html-in-header, so it is in the head of every page of
  the API reference. It styles a sudoku grid written as an HTML table or an
  inline SVG, and draws every `mermaid` fence in the reader's browser with a
  pinned Mermaid from jsDelivr. With no network or no scripts the fence stays
  as rustdoc wrote it: the diagram's source text.
-->
<style>
  table.sudoku { border-collapse: collapse; width: auto; margin: 1em 0; font-family: "Source Code Pro", monospace; }
  table.sudoku caption { caption-side: bottom; padding-top: 0.5em; font-family: inherit; text-align: left; }
  table.sudoku td { width: 2em; height: 2em; padding: 0; text-align: center; border: 1px solid var(--border-color); }
  table.sudoku tr:nth-child(3n) td { border-bottom: 2px solid var(--main-color); }
  table.sudoku tr:first-child td { border-top: 2px solid var(--main-color); }
  table.sudoku td:nth-child(3n) { border-right: 2px solid var(--main-color); }
  table.sudoku td:first-child { border-left: 2px solid var(--main-color); }
  table.sudoku td.mark { background-color: var(--main-color); color: var(--main-background-color); font-weight: bold; }
  svg.sudoku { display: block; margin: 1em 0; max-width: 100%; height: auto; color: var(--main-color); }
  .mermaid-diagram { margin: 1em 0; overflow-x: auto; }
  .mermaid-diagram svg { max-width: 100%; height: auto; }
</style>
<script>
  // Rustdoc writes a `mermaid` fence as <pre class="language-mermaid"><code>, and
  // keeps the theme in data-theme on the root element: light, dark or ayu.
  // Mermaid is fetched only on a page that has a fence. It is the single-file
  // build because one integrity hash can cover one file; the ESM entry loads
  // further files that no hash on this page would cover.
  document.addEventListener("DOMContentLoaded", () => {
    const fences = [...document.querySelectorAll("pre.language-mermaid")].map((pre, index) => {
      const figure = document.createElement("div");
      figure.className = "mermaid-diagram";
      pre.after(figure);
      return { pre, figure, source: pre.textContent, id: `mermaid-diagram-${index}` };
    });
    if (fences.length === 0) {
      return;
    }

    async function draw() {
      const dark = document.documentElement.dataset.theme !== "light";
      window.mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: dark ? "dark" : "default" });
      for (const { pre, figure, source, id } of fences) {
        try {
          const { svg } = await window.mermaid.render(id, source);
          figure.innerHTML = svg;
          pre.hidden = true;
        } catch (error) {
          // A fence Mermaid cannot read stays visible as text.
          figure.replaceChildren();
          pre.hidden = false;
          console.error(error);
        }
      }
    }

    const script = document.createElement("script");
    script.src = "https://cdn.jsdelivr.net/npm/mermaid@11.17.2/dist/mermaid.min.js";
    script.integrity = "sha384-EOXBFmc3gx5mb+vn0vPvvGqACToJD24hhacX5Yx+8NUUQrHIle/Qi5Bg9o3zKwW2";
    script.crossOrigin = "anonymous";
    script.addEventListener("load", async () => {
      await draw();
      new MutationObserver(draw).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    });
    document.head.append(script);
  });
</script>
```

- **`just test-doc` does not compile the fence.** The one test it reports from the page
  is the Rust example.
- **The element rustdoc 1.98.1 writes** for a `mermaid` fence:

  ```text
  <div class="example-wrap"><pre class="language-mermaid"><code>flowchart TD
  ```

  A `text` fence is `<pre class="language-text">`. So the script looks for
  `pre.language-mermaid`. Mermaid's own default looks for the class `mermaid` and would
  find nothing.
- **The rendering, and who saw it.** The agent saw it, as screenshots from Chrome
  154.0.8037.95 run headless against the scratch `target/doc` served on
  `127.0.0.1:8733`. The Chrome extension was not connected, so nobody clicked through
  the pages. The theme was set through rustdoc's own stored setting (`rustdoc-theme` in
  local storage) by a scratch page that framed the reference. No other browser was
  tried, and the maintainer has not looked.
  - *Light:* the flowchart in Mermaid's default colours, lavender nodes with dark text,
    on rustdoc's white.
  - *Dark and ayu:* Mermaid's `dark` theme, near-black nodes with light text and light
    edges, legible on both backgrounds. The header draws both with the one theme.
  - *A change of theme without a reload:* the page was loaded in light and the root
    element's `data-theme` set to `ayu` five seconds later. The screenshot is
    byte-identical to the one of a page loaded in ayu. The attribute was set by the
    scratch page, not through rustdoc's settings menu.
  - *The CDN unreachable:* with the script's host changed to a name that does not
    resolve, the fence stays a code block showing the diagram's source. With the right
    host and a wrong integrity hash the page is the same, byte for byte: the browser
    refuses the script and the source stays.
- **The path: one spelling cannot serve both builds.** Rustdoc reads the path relative
  to the directory cargo runs it in, which is the workspace root here and the package
  root on docs.rs.

  ```text
  # from the workspace root
  $ RUSTDOCFLAGS="-D warnings --cfg docsrs --html-in-header crates/pawdoku/rustdoc/header.html" cargo doc --workspace --no-deps --all-features --locked
     Generated .../copy/target/doc/pawdoku/index.html
  $ RUSTDOCFLAGS="-D warnings --cfg docsrs --html-in-header rustdoc/header.html" cargo doc --workspace --no-deps --all-features --locked
  error: error reading `rustdoc/header.html`: No such file or directory (os error 2)
  error: `ExternalHtml::load` failed

  # docs.rs imitated: packaged, unpacked outside the workspace, built there
  $ cargo package -p pawdoku --allow-dirty --no-verify --locked
      Packaged 52 files, 476.6KiB (102.8KiB compressed)
  $ tar -xzf target/package/pawdoku-0.1.0.crate -C <scratch>/unpacked
  $ cd <scratch>/unpacked/pawdoku-0.1.0
  $ cargo rustdoc --lib --all-features --locked --offline -- -D warnings --cfg docsrs --html-in-header rustdoc/header.html
     Generated .../unpacked/pawdoku-0.1.0/target/doc/pawdoku/index.html
  $ cargo rustdoc --lib --all-features --locked --offline -- -D warnings --cfg docsrs --html-in-header crates/pawdoku/rustdoc/header.html
  error: `ExternalHtml::load` failed
  ```

  So the `doc` recipe needs `crates/pawdoku/rustdoc/header.html` and the docs.rs table
  needs `rustdoc/header.html`. The same build passed for the table's second target,
  `wasm32-unknown-unknown`, and with the arguments given as
  `--config build.rustdocflags=[...]`. The package carries the file because the
  manifest has no `include` or `exclude`. What docs.rs itself runs was not seen; the
  spelling is the one `curve25519-dalek` uses there.
- **The version.** Both built and both drew the page's flowchart: 11.17.2 with the
  layout GitHub's renderings have had, 12.1.0 with squared edges and its new layout.

  | | 11.17.2 | 12.1.0 |
  | --- | --- | --- |
  | Released | 2026-08-25 | 2026-10-02 |
  | Browsers ruled out | none named by its release notes | older than Safari 17.4, and any without ES2024 |
  | Single file | 3,572,661 B, 935 kB compressed | 5,493,176 B, 1.49 MB compressed |
  | Default look | as before | ELK layout, new colours |

  **Integrity.** A hash covers one file. The ESM entry is 30 kB and imports the rest, so
  a hash on it leaves about 200 files unchecked. The single file is one file and one
  hash covers all of it. The header above therefore loads the single file, with
  `integrity` and `crossorigin`, and only on a page that has a fence. A first header,
  written with the ESM import, drew the same pictures; its ayu screenshot is
  byte-identical to the final header's. The recommendation is **11.17.2, the single
  file, with the hash**.

**Step 6. The grid and raw HTML.** One grid three ways in the doc-comment page, seen in
the same screenshots:

- *The `text` fence* reads in all three themes exactly as the handbook's does.
- *The HTML table* (`<table class="sudoku">`, nine `<tr>` lines of nine `<td>`, one
  `<td class="mark">`), styled by the header's `style` element through rustdoc's own
  colour variables: thin cell lines and thick box lines in all three themes.
  Rustdoc's own striping of alternate table rows shows through. The first mark, a
  shaded background, was too faint to tell from that striping. The header quoted in
  step 5 marks the cell by swapping the text and background colours instead, and that
  mark was then seen in all three themes: a solid black cell on light, a solid pale
  cell on dark and on ayu. The marked cell of the sample is empty, so a digit inside a
  marked cell was not seen.
- *The inline `svg`* with `stroke="currentColor"` and `fill="currentColor"`: dark lines
  and digits on light, light on dark and on ayu, the marked cell shaded in all three.

All of it passes `-D warnings`. Both need each element closed and no blank line inside
the block. The table is nine short lines a reader can check against a puzzle; the SVG
is a line of coordinates for each rule and each row of digits. A plain `.html` file copied into the served tree answered 200
beside the reference, and a doc comment linked to it as `[extra](../../extra.html)`
without a warning. Nothing checks that such a link's target exists, the file is in no
sidebar and no search, and docs.rs would not have it.

**Step 7. The routes not taken.**

- *The macro crates.* Either can sit behind an optional feature
  (`#[cfg_attr(feature = "...", aquamarine::aquamarine)]`; not built here). The feature
  would join `just features`' powerset and `wasm-check`'s, clippy would compile the
  macro under `--all-features`, and cargo-deny would read `aquamarine`'s six direct
  dependencies and what they pull. `aquamarine` also writes into `target/doc` while it
  expands, in any build where that directory exists. What a macro can do that the
  header cannot: its script travels in the item's own documentation, so the diagram
  draws on docs.rs with no `rustdoc-args`, and in another workspace's `cargo doc` of
  this crate as a dependency. Nothing in step 5 was out of the header's reach. The
  header has one exact version, a hash, a redraw on a change of theme (aquamarine's
  README says to reload) and no dependency.
- *`mmdr`.* The release binary for macOS arm64 was unpacked into the scratch directory
  (`mmdr 0.3.1`; the archive's SHA-256 begins `562d0250`). It is a `tools.txt` line by
  the binary-only route, with cargo-hack's caveat that the release publishes no
  checksum. Both handbook diagrams rendered twice, and each pair is byte-identical
  (6,623 B and 30,509 B). `human-solving.md`'s is clean. `solving-sudoku.md`'s is
  legible with defects: one edge is drawn through the "Undo most recent decision" box,
  the "Elegant exact cover" edge loops round its own node, and several arrowheads sit
  off their edges. It was compared with no rendering by GitHub, whose Mermaid version
  is unknown.
- *D2, Graphviz, Typst.* All three are on conda-forge for both platforms (table above).
  None was installed: no verdict turns on it.

**Step 8. The rule.** The handbook explains the project and its subject, and is where a
contributor reads; a rustdoc page is for someone using the crate, and it is written
around examples that compile. A topic has one home. If a page would teach how sudoku is
solved, how the engine is layered or how to work on it, it belongs in the handbook. If
it would show a caller how to do something with the public API, it belongs in rustdoc,
and anything it needs from the handbook it links to and does not repeat. A rustdoc page
links to a handbook page by its address on GitHub,
`https://github.com/steven-cutting/libpawdoku/blob/main/docs/...`, because a relative
path means nothing on the hosted site or docs.rs. The handbook links to the hosted
reference by its address, `https://steven-cutting.github.io/libpawdoku/pawdoku/`, to a
whole page and never a heading, as "Links that leave the repository" in the
documentation contract already requires. A diagram is drawn once, in the page that owns
its topic.

**Step 9. Verdicts,** each on the evidence above, 2026-10-03.

| Mechanism | Verdict | Evidence |
| --- | --- | --- |
| A page as an included file | **adopt now**, the first choice for prose | Every recipe passes; all three planted failures fail and name the file. The `.rs` file must not begin with `#!` |
| A page as doc comments | **adopt now**, for a page that needs raw HTML | Every recipe passes with a table and an SVG; gate 7 does not count the lines |
| A standalone `rustdoc page.md` | not at all | Not run. Nothing found changes the Context's reasons |
| A plain `.html` file beside the reference | on a trigger: a page rustdoc cannot express, and leave for the hosted site to carry what docs.rs lacks | Served and linked; unchecked, unlisted, absent from docs.rs |
| A handbook page included in rustdoc | not at all | Not run. A file under `docs/` is outside the package, so docs.rs could not build it |
| Mermaid in the browser | **adopt now** | Draws in three themes, redraws on a change, falls back to the source; one file, one hash |
| `aquamarine` or `simple-mermaid` | not at all | The header reached everything step 5 asked. Reopen if diagrams must draw where the flag is not passed |
| A vendored `mermaid.min.js` | not at all | 3.57 MB or 5.49 MB against 768 kB |
| Mermaid's command line | not at all | Needs a Chrome, and conda-forge has no build for this machine |
| `mmdr` | on a trigger: a requirement that pages work with scripts off; first to try | Byte-stable; one diagram of two drawn with visible defects |
| D2, Graphviz or Typst | on the same trigger, after `mmdr` | Available on both platforms; not run |
| Raw HTML or inline SVG by hand | **adopt now**, for a grid whose cells need marking: the table first, an SVG when something must be drawn across cells | Both read in the themes seen and pass `-D warnings` |
| A `text` fence | **adopt now**; already in use | Reads as the handbook's does |
| Kroki, PlantUML | not at all | A server; a JVM |

**Step 10. The follow-up, drafted.** **T34, "Pages and diagrams in the API reference: a
header file, its flag and the first guide page"**, depends on T33 and S10. It adds no
dependency.

- **`crates/pawdoku/rustdoc/header.html`**, new: the file quoted in step 5. The three
  `.sudoku` rules stay out until a page marks a cell; the `.mermaid-diagram` rules and
  the script land now. The comment at its head says how to move the pin: the version
  and the hash change together, and
  the file is fetched with `curl -fsSL -o mermaid.min.js <address>`, which fails on an
  HTTP error and so cannot hash an error page, and
  `openssl dgst -sha384 -binary mermaid.min.js | base64` then gives the hash.
- **`Justfile`** (T00 follow-up; frozen under CONVENTIONS.md §11): the `doc` recipe's
  one line gains `--html-in-header crates/pawdoku/rustdoc/header.html`, as in step 5,
  with a comment that the path is read from the workspace root. T33's `site` recipe
  calls `doc`, so the hosted site follows with no change of its own.
- **`crates/pawdoku/Cargo.toml`**: `rustdoc-args = ["--cfg", "docsrs",
  "--html-in-header", "rustdoc/header.html"]`, with a comment that docs.rs reads the
  path from the package root and that the two spellings must move together.
- **`crates/pawdoku/src/guide.md`** and **`crates/pawdoku/src/guide.rs`**: the sample
  page, as an included file. `guide.rs` is a `//` comment line and then
  `#![doc = include_str!("guide.md")]`. `lib.rs` gains `#[cfg(doc)] pub mod guide;`, so
  the page is in the reference and adds no path to the crate (the Rust choice is made
  here, not left open: the example still runs under `just test-doc`). The page holds
  one `mermaid` fence, one grid as a `text` fence and one example. Which page it is,
  and its text, are the maintainer's.
- **A probe that the flag is live**, in the `doc` recipe or a test of T34's choosing:
  `just doc` exits 0 whether or not the header is given, so something must fail when
  `target/doc/pawdoku/guide/index.html` lacks the pinned address.
- **`docs/reference/api.md`**: "Building it" names the header file and says a `mermaid`
  fence is drawn in the reader's browser; a new section, "Pages and diagrams", carries
  the rule of step 8 in the page's own words. The docs.rs section says the table passes
  the same header by a path of its own.
- **`docs/how-to/deploy-to-github-pages.md`** (T33's page): one sentence under "What
  the workflow does", that a page with a diagram loads Mermaid from jsDelivr when it is
  read, at a pinned version with an integrity hash, and that nothing is fetched while
  the site is built.
- **`docs/explanation/security-model.md`**: "No network at runtime" keeps its sentence
  about the engine and gains one about its documentation: a reference page that holds a
  diagram asks the reader's browser for one pinned script from jsDelivr, checked against
  a hash in the page, and shows the diagram's source if that fails.
- **`SECURITY.md`**: one bullet under "What this project already does", in the same
  words, shorter.
- **A how-to, `docs/how-to/add-a-reference-page.md`**: adding a page (the two files,
  the `lib.rs` line, the first-line rule), adding a diagram (a `mermaid` fence, and
  what it looks like with no network), adding a marked grid, and moving the Mermaid pin.
  With it, **`docs/manifest.yml`** and **`docs/README.md`** (both T00 follow-ups,
  frozen under §11), in the same commit as the page.
- **A decision record**, new, at the next free number on the day (T31 and T33 both
  draft 0015): pages are documentation-only modules; diagrams are `mermaid` fences
  drawn in the browser from one pinned, hashed file; the macro crates, a vendored
  script and build-time rendering are turned down, each with its reason from step 9;
  what would reopen it is a requirement that pages work with scripts off, which is
  `mmdr`'s trigger. T33's record is not amended: it decides the hosting, and this
  decides the content. With it, `docs/decisions/README.md`'s row and its "next decision"
  sentence, and the record's manifest entry.
- **`CHANGELOG.md`**, under Unreleased, Added: "The API reference can carry guide pages
  and diagrams: `pawdoku::guide`, and `mermaid` fences drawn in the reader's browser."
- **`docs/reference/configuration.md`** and **`docs/project/repository-map.md`**: the
  header file's row and line, if those pages list files of its kind on the day.
- **`tickets/README.md`**: T34's row, depending on T33 and S10.

**Step 11.** `status: done` here and in the index; committed on this branch; not pushed.

The Verification commands, run in this worktree on 2026-10-03:

```text
$ rg -n '"MD033"' .markdownlint-cli2.jsonc
11:    "MD033": { "allowed_elements": ["br"] },
$ rg -n -B1 '^file_length' rustqual.toml
52-# Code lines before the first #[cfg(test)]; comments, docs and blanks are not counted.
53:file_length = 500
--
60-max_function_lines = 100
61:file_length = 1000
$ rg -n 'rustdoc-args' crates/pawdoku/Cargo.toml
32:rustdoc-args = ["--cfg", "docsrs"]
$ rg -n '^```mermaid' docs
docs/explanation/solving-sudoku.md:388:```mermaid
docs/explanation/human-solving.md:63:```mermaid
$ rg -n 'RUSTDOCFLAGS' Justfile
216:    RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --workspace --no-deps --all-features --locked
```

### Deviations, and why

- **Branch name.** The Supacode worktree is on `ticket/S10-pages-and-diagrams-in-rustdoc`,
  with a capital the frontmatter's `branch:` lacks. The field is left as written, as in
  earlier tickets.
- **Headless Chrome, not the maintainer's browser.** The maintainer chose that the
  agent drive Chrome. The extension was not connected, so the agent ran the installed
  Chrome headless and read its screenshots. What that leaves unseen is said in step 5.
  To look: build the follow-up, run `just doc`, serve it with
  `python3 -m http.server 8733 --bind 127.0.0.1 --directory target/doc`, and open
  `http://127.0.0.1:8733/pawdoku/guide/index.html`. The scratch copy is in a session
  directory and will not outlast the session.
- **Two header files were written.** The first used Mermaid's ESM import. Step 2's
  finding that a hash cannot cover what the entry loads led to the second, which is the
  one quoted and recommended.
- **`cargo package` reached crates.io's index** in the scratch copy ("Updating
  crates.io index"), within the leave given for the scratch copy's network use.
- **`mmdr` came as a release binary,** not a source build: the release had one.
- **Step 2 was read by a sub-agent,** read-only, and its notes checked against step 5's
  own results. Its notes are in the ignored `ai_tmp/` and are not part of the change.
- **Changes after Copilot's review of the pull request,** each run in the same scratch
  copy on 2026-10-03: `just fmt-check` on the included-file variant, exit 0; the
  header's `securityLevel` set to `strict` in so many words, and the default read for
  11.17.2 as well (`config.schema.yaml` at `mermaid@11.17.2` says strict is the
  default); the stronger cell mark of step 6; `just doc` and `just lint` exit 0 with
  the header as now quoted. The diagram was not screenshotted again after the
  `securityLevel` change.

## Open points

The spike's answer is given under each point that has one. Each stays the maintainer's.

- **Which Mermaid.** 12 drops browsers older than Safari 17.4 and redraws with new
  defaults; 11 is the line the macro crates and the other documentation tools still
  load. GitHub renders the handbook's two diagrams with whatever version it runs. The
  spike recommends one exact version; the maintainer chooses. **Recommended: 11.17.2**,
  as the single file with its hash. Both drew the sample. 12.1.0 was a day old, rules
  out browsers 11 does not, redraws with a new layout, and is 1.9 MB larger.
- **Markdown file or doc comments.** The first choice keeps prose in a file that GitHub
  also renders and every Markdown hook reads, and so cannot hold raw HTML. Relaxing
  `MD033` for one directory is a change to a dotfile every page is held to, and is the
  maintainer's call if the spike finds pages need it. **It did not:** a page that needs
  raw HTML is written as doc comments, which passed every recipe, so `MD033` stays as
  it is.
- **Hosted site and docs.rs alike.** The recommendation assumes a page must render the
  same in both, which rules out anything `just site` adds after rustdoc has run. If the
  hosted site may carry more than docs.rs, the plain `.html` row and the vendored script
  read differently. The docs.rs imitation of step 5 shows the header serves both, so
  the recommendation holds the two alike.
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
  By step 8's rule it stays in the handbook, and a rustdoc page links to it.
- **`#[cfg(doc)]` on a page's module.** The follow-up drafts it, so a page adds no path
  to the crate. The other choice is an always-compiled empty module, which a consumer
  can name and which clippy also reads. Both passed `doc`, `test-doc`, `clippy`,
  `metrics`, `features` and `wasm-check`.
- **A build that does not pass the flag shows the source.** Another workspace
  documenting this crate as a dependency, or an editor's hover, gets the fence as text.
  That is the fallback the maintainer accepted for a reader with no network, met by a
  different road.
- **Nothing fails when the flag is dropped.** `just doc` exits 0 without the header.
  The follow-up drafts a probe; whether it is a recipe line or a test is T34's.
