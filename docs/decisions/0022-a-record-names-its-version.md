---
title: "Decision 0022: A record names its format version, and nothing is promised across versions"
kind: "decision"
audience: [contributor, maintainer, agent]
canonical_for: [decision_record_version]
requires: []
---

# Decision 0022: A record names its format version, and nothing is promised across versions

## Context

Play from a record ([decision 0021](0021-play-from-a-record.md)) makes a record a
consumer's state between calls, and a game keeps it between visits, so a record written
by one release of the engine may be handed to another. Nothing said what happens then.
`Record`'s documentation said no promise was made yet that a record written by one
version of the engine reopens in another, and that a record carried no version;
`board.allium` said nothing either way.

On 2026-10-31 the maintainer was asked what a saved record promises across versions of
the engine, and chose: version it, and promise nothing. A record names its format
version, a record of another version is refused with a clear message, and no
compatibility is promised. That is behaviour a consumer sees, so decision 0002 makes it
a specification change first: `board.allium`'s `config` gains `record_version`, and
`contract Recording` the invariant `ARecordNamesItsVersion`.

Three further questions were put with a proposal each, and answered on 2026-11-06:

- **The version string and the field's name.** `"record-1"`, in the shape of
  `"generation-1"`, and `version` as the field's name, first among a serialised record's
  fields, as proposed.
- **Whether the refusal names the engine's crate version too.** No, format versions
  only, as proposed. The crate version changes with every release while the format may
  not, so naming it would suggest a record of an earlier release is refused when it is
  not.
- **A record of a later shape may not deserialise at all.** A later engine may rename or
  drop a field, and an earlier one would then report the format's error, not the version
  refusal. The proposal left reading the version alone to each binding; the maintainer
  chose the alternative, a small value in the core that reads the version alone, so that
  every binding shares that reading.

The engine already names what replay depends on with version strings, each raised by
whoever changes what it names: `generation_version` and `random_version` in
`generation.allium`'s `config`, and `GENERATION_VERSION` and `RANDOM_VERSION` in Rust. A
record's version follows them.

## Decision

1. **A record names its format version.** `pawdoku::board::RECORD_VERSION`,
   `"record-1"`, equal to `board.allium`'s `config.record_version`. Every record
   `Board::write` gives names it. Under the `serde` feature it is the record's first
   field, `version`, and a record serialised without one reads as naming none.
2. **Reopening refuses another version first.** `Board::reopen` refuses a record that
   names another format version, or none, with `ReopenError::Version`, before it reads
   anything else, and the refusal's stable text names both versions. Nothing else in such
   a record can be trusted to mean what this engine reads.
3. **No compatibility is promised.** An engine reads its own version and no other: no
   migration, and no list of versions an engine accepts.
4. **The version is raised by whoever changes what it names**, in the same commit:
   anything a record holds, or anything reopening reads from it. The snapshot of a record
   as JSON shows a reviewer such a change.
5. **The version is read alone in the core.** Under the `serde` feature,
   `pawdoku::board::RecordVersion` deserialises a record's `version` and ignores every
   other field, and gives the refusal `Board::reopen` gives. A binding reads a record
   through it before the rest, so a record of a later shape is refused by the version's
   text. It is a reading, not a second check: `Board::reopen` still checks the record it
   is handed.

## Consequences

A game discards a record of another version and must say so to the player; the refusal
names both versions so that it can. Any change to what a record holds raises the version
and orphans every record kept under the last one.

A record kept from a release that wrote no version is refused. In a format that names
its fields, such as JSON, it deserialises and is refused with the version refusal naming
none. In a positional format, which hands a struct's fields over in order, the new first
field shifts every other, so such a record fails to deserialise in the consumer's format
before reopening is reached; the core cannot change that. The same holds for a later
shape in a positional format: `RecordVersion` reads the first field, which is the version
while every later version keeps the field's name, type and place.

`ReopenError` gains a variant and `Record` a private field. Both are `#[non_exhaustive]`
and the field is private, so no Rust API breaks; the serialised shape does change. The
variant holds a `String`, so the private helper that builds reopening's refusals is no
longer a `const fn`.

## What would reopen this

A consumer that needs records to survive an upgrade of the engine, which would bring a
promise and a reader for each earlier version. Or a record format of the consumer's own
that carries its version outside the record.

## Related pages

- [Decision 0021: Play from a record, beside the board](0021-play-from-a-record.md)
- [Decision 0014: The board imports the solver, and well-posedness is a proof value](0014-board-imports-solver.md)
- [Architecture](../explanation/architecture.md)
