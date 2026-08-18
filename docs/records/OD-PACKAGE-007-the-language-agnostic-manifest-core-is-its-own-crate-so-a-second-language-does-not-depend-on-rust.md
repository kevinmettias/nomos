---
id: OD-PACKAGE-007
type: decision
title: The language-agnostic manifest core is its own crate, so a second language does not depend on Rust to read its own manifest
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - packages
  - languages
  - architecture
relations:
  - target: OD-PACKAGE-001
    type: affects
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
---

# The language-agnostic manifest core is its own crate, so a second language does not depend on Rust to read its own manifest

## Question

`OD-PACKAGE-001` built `nomos-lang-package` (band 26) as "the first `LanguagePackage` manifest
format," naming `nomos-lang-rust` and `nomos-lang-rust-scan` as its first subject. What that
record did not settle, because nothing forced the question yet, is what the *second* language's
package would depend on.

Checked directly against the crate as it stands: `crates/packages/nomos-lang-package`'s
`Cargo.toml` names `nomos-lang-rust` and `nomos-lang-rust-scan` as direct dependencies, and
`known_providers.rs`'s `KNOWN_PROVIDERS` constant is `[nomos_lang_rust::PROVIDER,
nomos_lang_rust_scan::PROVIDER]` — pulled from those crates' own constants rather than a
parameter the reader takes. `language_version.rs`'s `RustEdition` is a four-variant closed enum
of Rust editions, and `reader.rs::Language_Versions_Field` resolves every `language_versions`
entry against it by name. A second language's package crate could not depend on
`nomos-lang-package` at all without also depending on Rust's two providers and being told every
manifest it reads recognizes only Rust editions — the crate is not a generic manifest reader
that happens to have been exercised on Rust first; it is a Rust reader, and its name is the only
generic thing about it.

Everything else `PKG-007`'s four version domains name is already language-agnostic and was
verified as such by reading each type directly: `PackageVersion` (major/minor/patch, no Rust
content), `ProtocolRange` (a pair of `nomos_contracts::ContractVersion`, reused rather than
retyped), and `ProviderRegistration` (a `nomos_contracts::ProviderId` plus a `PackageVersion`).
None of the three names Rust, imports a language provider crate, or would need to change for a
second language. `reader.rs`'s own `ManifestError` variants are also already general —
`UnresolvedProvider`, `MalformedVersion` and the rest read as prose that names no language.

## The decision

**A new crate, `nomos-package` (band 24, below `nomos-lang-rust`/`nomos-lang-rust-scan` at 25
and above `nomos-cap-syntax` at 23), carries the language-agnostic core**:
`PackageVersion`, `ProtocolRange`, `ProviderRegistration` (moved verbatim, unchanged), a
`PackageManifest` type carrying `language_versions: Vec<String>` (raw, non-empty, unresolved
labels — no typed edition domain at this layer, because there is no one typed shape a version
label takes across languages: Rust's four editions, a semver string, a single integer and a
date-stamped release are all real shapes a future language's package might need, and inventing
a generic version-domain type now would be designing against a population of one, the same
caution `OD-PACKAGE-006` already states for `KNOWN_PROVIDERS`), and `ManifestError` (moved
verbatim). `Parse_Manifest` and `Read_Manifest` take one more parameter than their
`nomos-lang-package` counterparts: `known_providers: &[&str]`, supplied by the caller rather
than hardcoded, so this crate never depends on any specific provider crate. `nomos-package`
depends only on `nomos-contracts`, `serde` and `serde_json` — nothing above band 0.

**`nomos-lang-package` becomes a thin Rust-specific wrapper over it**, keeping its exact
existing public surface: `Parse_Manifest(text, at)` and `Read_Manifest(path)` keep their
current two- and one-argument signatures (never taking `known_providers` from a caller — that
is this crate's whole reason to exist over the generic one), `LanguagePackage` keeps
`language_versions: Vec<RustEdition>`, and `RustEdition`, `KNOWN_PROVIDERS` and `Is_Known` all
stay exactly where they are. Internally, `Parse_Manifest` calls `nomos_package::Parse_Manifest`
with `KNOWN_PROVIDERS`, then resolves each raw `language_versions` label against
`RustEdition::Of_Label`, refusing with the same `ManifestError::MalformedVersion` shape
(same field name, same indexed position, same cause wording) the reader produced before this
change — a behavior-preserving move, not a new refusal. `PackageVersion`, `ProtocolRange`,
`ProviderRegistration` and `ManifestError` are re-exported from `nomos-package` rather than
redefined, so the two crates cannot drift on what these four types mean.

**A second language's package crate depends on `nomos-package` directly, never on
`nomos-lang-package`.** It supplies its own version-label type and parsing (its own
`RustEdition`-shaped enum, or a raw string, or whatever its ecosystem's version scheme actually
needs) and its own known-providers list, the same two things `nomos-lang-package` supplies for
Rust. Nothing under `crates/packages/` should ever need to depend on another language's package
crate to read its own manifest.

## What this does not decide

This is not `OD-PACKAGE-006`'s question answered. Whether `KNOWN_PROVIDERS`-shaped lists need a
self-registering mechanism once a third provider exists is unrelated to whether the *reader*
that resolves them needs to import specific provider crates — this record removes the second
problem; the first stays open exactly as `OD-PACKAGE-006` left it.

This does not build a second language's manifest format, a typed version-domain abstraction, or
`PKG-022`'s larger field list. It builds the seam a second language's package crate would depend
on, the same way `OD-HOST-001` built a seam before a second adapter existed to call it.

## What Holds It

`crates/packages/nomos-lang-package/tests/manifest.rs` is unchanged by this record and must
still pass byte-for-byte: it is the behavior-preservation proof, asserting the exact same
`ManifestError` variants, field names and cause text this crate produced before the split.
`crates/packages/nomos-package`'s own test suite exercises the generic reader directly, with a
`known_providers` list of its own choosing rather than Rust's, so the split is proven to work
for *some* provider set other than the one it was extracted from — not only reproven against
the one case it already had.

## Status

Closed by `P13-PACKAGE-GENERIC-CORE`.
