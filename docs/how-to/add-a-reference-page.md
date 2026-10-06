---
title: "Add a reference page"
kind: "how-to"
audience: [contributor, maintainer, agent]
canonical_for: [reference_page_authoring]
requires: []
---

# Add a reference page

A reference page is a module of the crate that holds documentation and no item, so that
rustdoc puts it in the sidebar, the search and the hosted site beside the API. The engine
map, `pawdoku::guide`, is the first; `crates/pawdoku/src/guide.rs` is the shape to copy.
[API reference](../reference/api.md) says what such a page may hold. In short: it is for
someone using the crate, it shows how the public API fits together or how to do something
with it, and anything it needs from the handbook it links to and does not repeat.

## Add a page

1. Decide the form. A page with figures or any other raw HTML is written as doc comments
   in a `.rs` file, which no Markdown hook reads. A page of prose alone can be a `.md`
   file beside the source, pulled in with `#![doc = include_str!("name.md")]`; the
   Markdown hooks then read it, and `MD033` refuses raw HTML in it.
2. Begin the `.rs` file with a `//` comment, never with `#!`: the hook that checks
   shebangs reads `#![doc = ...]` on the first line as one and fails the gate.
3. Declare the module in `crates/pawdoku/src/lib.rs` under `#[cfg(doc)]`, in
   alphabetical order with the others. The page then exists when rustdoc builds and
   nowhere else, and no consumer can name it.
4. Make the first line one sentence: rustdoc shows it beside the module's name.
5. Give the page at least one example that compiles and asserts. `just test-doc` runs it.
6. Name every item in the prose through an intra-doc link whose target is the item's
   path, such as `crate::board::Board`, inline or as a reference definition at the foot
   of the page, so a renamed item fails `just doc`.
   Link to a handbook page by its address on GitHub,
   `https://github.com/steven-cutting/libpawdoku/blob/main/docs/...`: a relative path
   means nothing on the hosted site or on docs.rs.
7. Run `just doc`, `just test-doc`, `just lint` and `just fmt-check`, then
   `just check`. A heading inside the page is one level below rustdoc's own, so start at
   `##`.

## Add a figure

A figure is an inline `svg` with the class `pg` inside a `div` with the class `pg-fig`,
styled by `crates/pawdoku/rustdoc/header.html`, which the `doc` recipe passes to rustdoc
with `--html-in-header`.

- Put the whole figure in one HTML block: no blank line from `<div` to `</div>`, since a
  blank line ends the block and the rest prints as text. Close every element, or
  `rustdoc::invalid_html_tags` fails the gate under `-D warnings`.
- Draw on a `viewBox` of `0 0 960 H`, and let the stylesheet scale it: the page's column
  is about that wide, and a narrower window scrolls the figure sideways.
- Use the header's classes and no others, so a name cannot collide with rustdoc's
  stylesheet (rustdoc defines `.hidden`, which hides an element). The classes are: `pg-h`
  for a heading, `pg-ch` and `pg-c` for a name set in code, `pg-d` and `pg-s` for a
  smaller note, `pg-lab` for a small-caps label; `pg-bx` for a plain box, `pg-built`,
  `pg-plan`, `pg-sol` and `pg-ref` for built, planned, a stored solution and a refusal;
  `pg-ln` for a line, with `pg-back` for a dashed one, `pg-ln-s` and `pg-ln-r` in the
  solution's and the refusal's colour; `pg-ah` and its variants for an arrowhead.
- Give each `<svg>` a `role="img"` and an `aria-label` that states what the figure
  shows, and give every marker an id that begins `pg-`, unique on the page.
- Colours come from rustdoc's variables (`--main-color`, `--border-color`,
  `--code-block-background-color`) and from the header's three of its own, each defined
  for light and again for dark and ayu. Use no literal colour in a figure.

Names inside a figure are plain text. Nothing fails when an item they name is renamed,
so a change to the engine is a change to its figure, by hand, by coordinates.

## Look at the result

Rustdoc's pages set their theme from the browser's storage, so a plain file open shows
one theme. Serve the build and switch themes with the gear icon on the page:

```console
just doc
python3 -m http.server 8734 --bind 127.0.0.1 --directory target/doc
```

Then open `http://127.0.0.1:8734/pawdoku/guide/index.html` and look at light, dark and
ayu. A figure that is right in one theme can be unreadable in another when a colour was
written as a literal.

## A diagram as a Mermaid fence

Nothing in the reference draws with Mermaid today, and the header file loads no script.
If a page needs a `mermaid` fence, the loader S10 wrote and T34 built for its comparison
comes back, with two corrections T34 found: rustdoc wraps a fence in `div.example-wrap`,
a flex row, so the drawn figure must be inserted after that wrapper, not after the `pre`
inside it, and the wrapper hidden with `display: none`; and no Mermaid class may be
named `hidden`. The loader is quoted in `tickets/T34-guide-page-and-diagrams.md`. With
it, [Deploy to GitHub Pages](deploy-to-github-pages.md), the security model and
`SECURITY.md` each need a sentence about the one script a reader's browser fetches, and
decision 0017 says what would reopen it.

## Move the header

The header's path is spelt two ways, and the two move together:
`crates/pawdoku/rustdoc/header.html` in the `Justfile`'s `doc` recipe, read from the
workspace root, and `rustdoc/header.html` in `[package.metadata.docs.rs]` of
`crates/pawdoku/Cargo.toml`, read from the package root. The recipe's last line fails
when a built guide page lacks the header's first line, `pawdoku-rustdoc-header`, so
dropping the flag by mistake fails gate 12.

## Related pages

- [API reference](../reference/api.md)
- [Decision 0017: Guide pages and figures in the API reference](../decisions/0017-guide-pages-in-the-api-reference.md)
- [Deploy to GitHub Pages](deploy-to-github-pages.md)
- [Quality gates](../reference/quality-gates.md)
