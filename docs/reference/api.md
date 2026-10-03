---
title: "API reference"
kind: "reference"
audience: [user, contributor, maintainer, agent]
canonical_for: [api_reference]
requires: []
---

# API reference

The API reference is the one rustdoc generates from the crate's doc comments. This page
does not copy it: a list of items written out here would be wrong after the first change.
It says how to build the reference, where it lands, where it is hosted, what the public
API promises, and what a doc example is for.

## Building it

`just doc` runs `cargo doc` over the workspace with every feature and no dependencies,
with `RUSTDOCFLAGS="-D warnings --cfg docsrs"`. The result opens at
`target/doc/pawdoku/index.html`. It is gate 12 of `just check`, so a broken intra-doc
link, a link to a private item or an undocumented public item fails the gate rather than
producing a page with a hole in it.

## What is documented

`missing_docs` is set to warn in the `[workspace.lints.rust]` table, and both `just doc`
and `just clippy` turn warnings into errors. Removing the doc comment from one public
constant fails both recipes with `missing documentation for a constant`. So every public
item carries documentation, and the crate root carries the overview. The rustdoc lints
in `[workspace.lints.rustdoc]` add broken and private intra-doc links, which fail
outright, and a missing crate-level doc and unescaped backticks, which warn.

## Doc examples are tests

Every public item with a contract worth stating carries an example — today every type,
trait, method and constant does — and `just test-doc` (gate 10) compiles and runs each
one, so an example that stops being true stops the gate. Examples are not counted toward
the coverage floor, because the coverage tooling cannot measure doctests on a stable
toolchain. The division of labour follows: an example shows the contract to a reader and
proves it, and a unit test earns the coverage. See [Testing](testing.md).

## Hosted

The reference for `main` is hosted at <https://steven-cutting.github.io/libpawdoku/>,
and a push to `main` republishes it. It documents `main`, not a release, so it may
describe an item no release has. `just site` builds what is hosted: an empty
`target/doc`, then `just doc` with the flags above, then a root `index.html` that sends
a reader to the crate. [Deploy to GitHub Pages](../how-to/deploy-to-github-pages.md) has
the workflow and the rollback.

## docs.rs

`crates/pawdoku/Cargo.toml` carries a `[package.metadata.docs.rs]` table: every feature,
`--cfg docsrs`, and two targets, `x86_64-unknown-linux-gnu` and
`wasm32-unknown-unknown`, so docs.rs shows what a native and a WebAssembly consumer
each get. This is prospective. Every crate is `publish = false` until ticket
S02 decides the release process, so nothing is on docs.rs yet. When a release is, docs.rs
is the reference for it and the hosted site stays the reference for `main`.

## What the API promises

Every public type is `Send + Sync + 'static`, `Clone` and `Debug`, because a binding may
move a value between threads, copy it across a language boundary or print it. Public
structs and enums are `#[non_exhaustive]`, so a field or variant added later breaks no
consumer. Errors are `thiserror` enums whose `Display` text is stable, because the Python
and WebAssembly bindings build their exceptions from it; a unit test pins each variant's
text. `tests/api_bounds.rs` holds the bounds at compile time, so a type that loses one
fails the build of the test. The crate has no default features; `serde` is optional and
additive, and adds `Serialize` and `Deserialize` to the types that can carry state.

## Related pages

- [Deploy to GitHub Pages](../how-to/deploy-to-github-pages.md)
- [Architecture](../explanation/architecture.md)
- [Testing](testing.md)
- [Purpose and scope](../project/purpose-and-scope.md)
