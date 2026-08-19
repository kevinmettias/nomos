---
id: OD-PACKAGE-009
type: decision
title: The manifest JSON carries its own schema version, distinct from PKG-007's four content domains
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - packages
  - architecture
relations:
  - target: OD-PACKAGE-001
    type: affects
  - target: OD-PACKAGE-007
    type: relates-to
  - target: OD-LEDGER-008
    type: relates-to
---

# The manifest JSON carries its own schema version, distinct from PKG-007's four content domains

## Question

`crates/packages/nomos-package/src/manifest.rs`'s `PackageManifest` models `PKG-007`'s four
version domains — package version, protocol range, language versions, provider tool
versions — none of which answer a different question: what shape is the JSON file itself?
`crates/packages/nomos-package/src/reader.rs::Parse_Manifest` reads `package_id`,
`package_kind` and the rest positionally, by field name, with no marker anywhere in the
document saying which revision of that field layout a given file was written against. A
real, hand-authored manifest already exists on disk at `packages/nomos.lang.rust.json`, read
by a real consumer (`nomos_lang_package::Read_Manifest`, exercised by
`crates/packages/nomos-lang-package/tests/manifest.rs::Test_The_Real_Rust_Manifest_Should_
Round_Trip`) — this is not a hypothetical future format, it is a format with one real file
already committed to this repository.

`code-standards`/`nomos-proto` (`github.com/kevinmettias/nomos-proto`), the Go-era
predecessor this workspace rebuilds, names exactly this as a third, independent version
domain on its own out-of-process plugin manifest: `MANIFEST_SCHEMA_VERSION`, distinct from
both a plugin's own `Version` and its `Engine{Minimum,Ceiling}` compatibility range —
verified directly against `kernel/plugin/installation_contract.go`.

This workspace already has its own, closer precedent for the identical problem:
`crates/substrate/nomos-ledger/src/store/ledger_document.rs`'s `LedgerDocument` carries a
required `schema_version: u32` field, checked against a `SCHEMA_VERSION` constant
(`store.rs`) before a document is trusted, specifically so a build reading a newer-schema
file refuses with a legible reason (`document.rs::Explain`, `LedgerError::Unrecognized`)
rather than misreading it as a merely malformed one. `OD-LEDGER-008` is why the ledger's own
container is `#[serde(deny_unknown_fields)]` in the first place — the same "a build that
cannot account for every key does not get to write it back" discipline a schema marker
exists to make legible across a format change.

## The Decision

`PackageManifest`'s JSON format gains a required `schema_version` field, checked first, with
a distinct refusal — `ManifestError::UnknownSchema { at, understood, found }` — when a
manifest names a schema version newer than this build understands. A missing
`schema_version` is refused the same way any other missing required field already is
(`ManifestError::MissingField`); there is no default and no silent inference, the same
choice `nomos-ledger` already made for its own schema marker.

Built now rather than logged as an open question, unlike `OD-CAPABILITY-007` and
`OD-RULES-006`'s decline-with-reason and `EnforcementPlacementRationale` questions decided
the same session: those concern a mechanism with no real consumer and no second real
instance to design against. This is different in kind. `Parse_Manifest` has a real consumer
today, and a real, already-published manifest file exists with zero forward-compatibility
marker on it. The cost of waiting is not symmetric the way it is for an unconsumed struct
field: every manifest published before a schema marker exists is a file a future reader can
never distinguish from one written against whatever shape came first, the same reason a
ledger schema field could not usefully be added after the first real schema change rather
than before it. `nomos-ledger`'s own `schema_version` is not analogous evidence borrowed
from elsewhere; it is this workspace's own prior, independent arrival at the identical
design for the identical class of problem — a persistent, hand-authored, on-disk document
read by code that will itself keep changing.

`SCHEMA_VERSION` starts at `1`. Nothing about the JSON shape `Parse_Manifest` reads today
changes; this only adds the one new required field and the one new refusal path.

## What This Record Does Not Do

It does not build a migration mechanism between schema versions, forward or backward
compatible reading of an older schema, or any equivalent of `nomos-ledger`'s
`READS_MAJOR`-style escape hatch — `code-standards`' own `HANDOFF.md` documents that
mechanism as having shipped unreachable there (an equality check ran before the range check
meant to allow it through), and this record deliberately does not attempt the more complex
version of the problem before the simpler one — refuse cleanly on mismatch — has a real
second schema to be checked against.

It does not touch `PKG-007`'s four content domains, `nomos-lang-package`'s Rust-specific
reading, or any provider/language-specific manifest content.

## Status

Accepted, landed by `P13-PACKAGE-SCHEMA-VERSION`.
