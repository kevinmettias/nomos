---
id: OD-PACKAGE-007
type: decision
title: The language-agnostic manifest core is its own crate, so a second language does not depend on Rust to read its own manifest
status: accepted
version: 2
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

## Amendment: A Real Second Language Landed, And The Core Needed No Adjustment

Added at version 2. This record's own "second language's package crate" was a projection
against a population of one when it was written; `nomos-lang-go-package`
(`P14-PACKAGE-GO-LANGUAGE-MANIFEST-3`) is that party, real rather than hypothetical, and this
amendment measures the projection against it the same honest-correction discipline
`OD-PACKAGE-008`'s own amendment already models: name what, if anything, needed adjusting once
real evidence existed, rather than reaffirming the original decision by citation alone.

**The dependency shape held exactly.** Checked directly against
`crates/packages/nomos-lang-go-package/Cargo.toml`: it depends on `nomos-package`,
`nomos-contracts`, `nomos-lang-go`, `serde` and `serde_json` — never on `nomos-lang-package`,
never on any Rust provider crate. Every one of `nomos_package::{PackageVersion, ProtocolRange,
ProviderRegistration, ManifestError, Parse_Manifest, Read_Manifest}` is used unchanged, byte-
for-byte the same public surface this record fixed for the split. `nomos-package` itself needed
zero changes to serve a second, unrelated language — not a line of it is in
`P14-PACKAGE-GO-LANGUAGE-MANIFEST-3`'s own territory, because none of it needed touching.

**The version-label domain is a real third shape, not one of the two this record named.**
This record's own text offered two candidates for what a second language's version domain
might look like: "its own `RustEdition`-shaped enum, or a raw string." What
`nomos-lang-go-package::GoVersion` actually is is neither: a parsed, validated `{major, minor}`
pair, checked against `go.mod`'s own grammar, chosen because Go ships a new minor version
roughly twice a year and a closed enum enumerating them would already be stale, while an
unvalidated raw string would accept nonsense a `LanguagePackage` should refuse. The real
population of typed version-label shapes this workspace has evidence for is now three — a
small closed enum, a raw unvalidated string, and a parsed-and-validated structured value — not
the two this record anticipated. This is not evidence `nomos_package::PackageManifest::
language_versions` should have tried to generalize over that shape: leaving it an unresolved
`Vec<String>`, exactly as this record already decided, is what let a third real shape arrive
without needing to touch the generic core at all — the design this record made *because* it
could not know the second language's shape in advance is exactly what absorbed a shape it did
not specifically predict.

**`KNOWN_PROVIDERS`'s scoping convention repeated independently, not by copying.**
`nomos-lang-go-package`'s own allowlist carries exactly one provider,
`nomos_lang_go::PROVIDER`, and excludes `nomos-lang-go-modules` (`nomos.cap.dependency.edges`)
the same way `nomos-lang-package`'s own allowlist excludes `nomos-lang-rust-cargo` — a
`LanguagePackage` registers the language's syntax provider, not every capability provider that
happens to read its ecosystem's files. This was reasoned independently against Go's own real
provider population, not copied from the Rust crate's file, and landing on the identical
scoping rule twice is real evidence the rule is a property of what a `LanguagePackage` is
for, rather than an accident of the first case.

## Status

Closed by `P13-PACKAGE-GENERIC-CORE`. Amended to version 2 by
`P14-LANG-GO-RECORD-2`: measured directly against `nomos-lang-go-package`, the real second
consumer this record was written for, and found to need no adjustment.
