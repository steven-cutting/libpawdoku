---
title: "Decision 0017: Guide pages and figures in the API reference"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_guide_pages_in_reference]
requires: []
---

# Decision 0017: Guide pages and figures in the API reference

## Context

Decision 0016 put the API reference on GitHub Pages and kept the handbook off it. The
maintainer then asked for a map of the engine in the hosted docs: what the crate is,
what it does and how, in figures at four levels. Spike S10 had found how a page of prose
and diagrams goes inside rustdoc, and had ruled that "how the engine is layered" belongs
in the handbook, which the map's picture of the modules collides with.

The maintainer settled it on 2026-10-03, and chose between two builds of the page on
2026-10-05: one with figures drawn by hand as inline SVG, one with the same figures
redrawn as Mermaid fences, both built and compared in a browser from the local build. An
adversarial review of the first build then found that its figures took their styles from
a header file rustdoc was given by flag, and the page was made self-contained.

## Decision

**A page in the reference is a documentation-only module.** It is written as doc comments
in a `.rs` file when it holds raw HTML, and declared in `lib.rs` under `cfg(doc)`, so it
exists when rustdoc builds and nowhere else. It carries at least one example, and names
every item it mentions through an intra-doc link, so that gates 10 and 12 hold what they
can. `pawdoku::guide`, the engine map, is the first.

**What a page may hold.** A page in the reference is for someone using the crate: it
shows how the public API fits together, or how to do something with it. The handbook
explains the project and its subject and is where a contributor reads. A topic has one
home, and the other side links to it by address. This amends S10's rule in one respect:
a map of the public API may show the modules and which may import which, because a
caller reads the crate module by module; the handbook's layering page stays the owner of
the import rule, its enforcement and its reasons.

**Figures are inline SVG, drawn by hand, and the page carries their styles** in a
`style` block of its own, which takes rustdoc's colour variables so a figure follows the
light, dark and ayu themes. Nothing about the page depends on how rustdoc was invoked: it
renders the same on the hosted site, on docs.rs, and in the `cargo doc` of a consumer
who depends on the crate and passes rustdoc no flag of ours. No page of the reference
asks a reader's browser for anything from a third party.

Turned down, each on evidence:

- **A header file passed with `--html-in-header`**, S10's mechanism, carrying the figures'
  stylesheet. It was built first, with the flag in the `doc` recipe and the docs.rs table
  and a probe that failed when the flag was dropped. The adversarial review of that build
  found what the probe could not: Cargo ignores `[package.metadata.docs.rs]` and never
  runs this repository's `Justfile`, so a consumer's own `cargo doc` builds the
  `cfg(doc)` page with no header, and an unstyled SVG draws black boxes with its labels
  lost in them. A `style` block in the page passes `-D warnings` and travels with it.
  A header file returns only for a script: one in the page would run in that consumer's
  build too, and fetch from a third party there, so a script belongs in a file the
  build is given and nowhere else.
- **Mermaid fences drawn in the browser**, S10's first choice for diagrams and the
  handbook's own notation. The Mermaid page was built and compared. Its figures came
  out at sizes Mermaid chose, the wide ones shrunk to fit the column, two figures (the
  journal and the upkeep example) could only be text, and every reader of the page
  would fetch a 5.5 MB script from jsDelivr. Rendering it inside rustdoc also needed
  two corrections to S10's loader, recorded in ticket T34. The hand-drawn figures read
  at one size throughout and need nothing fetched. What they cost is upkeep: a change
  to the engine is a change to a figure, by coordinates.
- **Keeping the Mermaid loader in a header for a later page.** Nothing uses it, and
  shipping it would oblige the security pages to describe a script no page loads. It
  returns with the first page that has a fence.
- **The page as a plain `.html` file copied beside the reference.** Unlisted, unsearched,
  unchecked and absent from docs.rs; S10's verdict stands.
- **Figures beside the items they explain, or split between reference and handbook.**
  The maintainer wanted the four levels read in order on one page.

## Consequences

- **A figure's names are not held by the gate.** Prose names are intra-doc links and
  fail when an item is renamed; a name inside an `svg` is plain text and does not. A
  figure that falls behind the code is a defect review has to catch.
- **Editing a figure means editing coordinates.** The how-to says where each class is
  and how to look at the result in three themes.
- **The styles live in the page.** A second page with figures copies the `style` block,
  because sharing one through a header file is what was turned down.
- **The map of modules now lives in two places**, as a table in the handbook and as a
  figure in the reference. The table is the owner; the figure links to it.
- **The `Justfile` did not change.** Of the frozen files, only `docs/manifest.yml` and
  `docs/README.md` did, for the two new pages, with the maintainer's leave.

## What would reopen this

A page that needs a diagram no hand-drawn figure can carry well, such as a state machine
that changes often: Mermaid returns with a header file, its flag in two places, a probe,
and the sentences the security pages then need. A requirement that figures be generated
from a source the gate can check against the code. A second documented crate in the
workspace, which would ask whether pages share their styles.

## Related pages

- [API reference](../reference/api.md)
- [Add a reference page](../how-to/add-a-reference-page.md)
- [Decision 0016: The API reference on GitHub Pages](0016-api-reference-on-github-pages.md)
- [Layering and dependency direction](../explanation/layering.md)
