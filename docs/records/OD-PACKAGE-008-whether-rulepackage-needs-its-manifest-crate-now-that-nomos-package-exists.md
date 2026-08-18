---
id: OD-PACKAGE-008
type: decision
title: Whether RulePackage needs its manifest crate now that nomos-package exists, or stays a bare rule bounded to a population of one
status: open
version: 1
authority: canonical-normative-record
tags:
  - packages
  - rules
  - architecture
relations:
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-006
    type: relates-to
  - target: OD-PACKAGE-007
    type: relates-to
---

# Whether RulePackage needs its manifest crate now that nomos-package exists, or stays a bare rule bounded to a population of one

## Question

`OD-PACKAGE-001` accepted `ARCH-001` and `ARCH-002` as corpus requirements binding this
build: a `LanguagePackage` and a `RulePackage` must each be independently versioned, in the
sense both requirements actually specify — a declared manifest carrying `PKG-007`'s four
version domains, not a Cargo `version` field. `OD-PACKAGE-007` (`P13-PACKAGE-GENERIC-CORE`)
satisfied `ARCH-001`'s mechanism: `nomos-package` (band 24) now holds `PackageVersion`,
`ProtocolRange`, `ProviderRegistration` and `ManifestError` as a language-agnostic core, and
`nomos-lang-package` is a thin Rust-specific wrapper over it that never depends on Rust for
anything but its own `RustEdition` version-label domain.

`ARCH-002`'s `RulePackage` half is untouched by that work. Measured directly at HEAD:
`crates/rules/nomos-rules` declares exactly one rule, `Check_Completeness_Mirrors`, with a
stable id (`COMPLETENESS_MIRROR`) and a contract citation (`CONTRACT_RECORD = "D-134"`,
`CONTRACT_RECORD_VERSION = 2`) that `tests/contract/tests/rule_contract_citation.rs` checks
against the record's own front matter on every run. That citation discharges `PKG-014`'s
traceability requirement in the narrow sense of "checked, not merely asserted in prose," but
there is no manifest anywhere: no `PackageId` for a rule package, no `PackageKind::RulePackage`
consumer anywhere in the workspace, and nothing that reads a declared `RulePackage` manifest
the way `nomos-lang-package::reader` reads a `LanguagePackage` one. The only occurrence of
`RulePackage` outside `nomos-contracts` itself is a negative test in
`crates/packages/nomos-lang-package/tests/manifest.rs` asserting that the Rust reader
*refuses* a manifest declaring `RulePackage` as its kind — proof the seam is absent, not a use
of it.

The question `nomos-package`'s existence raises directly: does the generic core built for
`ARCH-001` make an `ARCH-002` wrapper crate — a `nomos-rule-package`, shaped like
`nomos-lang-package` but for rules — cheap enough to build now regardless of how many rules
exist to shape it against, or should it wait for a second rule the same way `OD-PACKAGE-006`
waits for a third provider before building `KNOWN_PROVIDERS` a self-registering mechanism?

## Current Position

`nomos-package`'s three reused types are already rule-agnostic on inspection, not merely
Rust-agnostic. `PackageVersion` is major/minor/patch with no language or rule content.
`ProtocolRange` is a pair of `nomos_contracts::ContractVersion`, already shared. And
`ProviderRegistration` pairs a `nomos_contracts::ProviderId` with a `PackageVersion` — nothing
about it names a language provider specifically, so a `RulePackage` manifest listing the
`ToolProvider`s a rule's enhanced implementation depends on could reuse the same type
unchanged. A `nomos-rule-package` wrapper would not need to touch `nomos-package`'s core at
all; it would only need to supply its own equivalent of `KNOWN_PROVIDERS` and its own
version-label domain, exactly the shape `OD-PACKAGE-007` reserved for a second language.
That lowers the mechanical cost of building the wrapper below what it was before
`OD-PACKAGE-007` landed.

Cost is not the only variable, and the harder one points the other way. `ARCH-002`'s contents
list is not `ARCH-001`'s. `LanguagePackage`'s contents are declarations: identity, recognition,
mappings, registrations, requirements, compatibility — exactly what
`nomos-lang-package::LanguagePackage` already models as fields. `RulePackage`'s contents —
rule identity/version, normative specification, applicability semantics, required canonical
capabilities, deterministic judgment implementations, optional enhanced implementations,
external diagnostic mappings, correction and suppression contracts, evidence schema,
examples/counterexamples, conformance fixtures, evaluation corpus, agent-guidance fragments,
presentation/protocol metadata — is a longer and structurally different bundle: several of
those fields are data corpora and executable fixtures, not manifest-shaped declarations at
all, and `OD-PACKAGE-001` already noted a Rust crate can carry them only as opaque bytes no
version comparison can reason about. `nomos-rules` today has exactly one rule and exhibits
none of the variation a manifest schema would need to generalize over — one applicability
shape, one judgment implementation, no enhanced implementation, no external diagnostic
mapping, no correction or suppression contract. A schema built from a population of one rule
would be `Check_Completeness_Mirrors`'s own shape wearing a general name, the same trap
`OD-PACKAGE-006` named for `KNOWN_PROVIDERS` at a population of two providers — and this
population is smaller than that one.

`OD-HOST-004` is relevant precedent for the opposite conclusion in a different shape, the same
way `OD-PACKAGE-006` used it: a second hand-written call was not treated as seam-by-accretion
because participation there depended on the request and a registry already existed
underneath. That does not transfer here either. A `RulePackage` manifest is not a
selection mechanism over existing rules; it would be a new declared-artifact format for a
domain this workspace has observed exactly once. There is no second rule to check field
boundaries against, the way `nomos-lang-package`'s split was checked against `nomos-rules`
existing already as a second, structurally different consumer of `nomos-contracts` before
`nomos-package` was extracted.

A separate, narrower question is in flight on the board at the time of this writing and is
worth distinguishing rather than leaving this record to look silently at odds with it: the
`P13-RULE-PACKAGE-DECISION` item proposes a rule-*registration* contract — a `RuleId` and a
`Registry` a rule crate offers into, mirroring `nomos_capability::Registry`'s shape — built
ahead of a second rule, on explicit direction to extract early the way `nomos-add-plugin`
section 2 states. That is `Run()`'s discovery mechanism, the `OD-HOST-004` axis of the
problem (participation varying by request), not `ARCH-002`'s independently-versioned
*manifest* with `PKG-007`'s four version domains, which is this record's question. The two
can be decided on different schedules without contradiction: a rule could gain a declared
registration contract before it gains a versioned manifest, the same way `nomos-lang-rust`
registered with `nomos_capability::Registry` long before `nomos-lang-package` gave it a
manifest to be named in. Whether that registration contract, once built, changes the
population this record reasons from is itself a fact for a later reader to check, not one
this record assumes.

## What Would Decide It

A second rule joining `crates/rules/nomos-rules` — or a second rule crate — is the natural
trigger, the same role a third provider plays for `OD-PACKAGE-006`, adjusted down by one
because rules number one today rather than two. At that point either the two rules turn out
to share enough manifest-shaped structure (applicability semantics, capability requirements,
evidence schema) that a thin `nomos-rule-package` wrapper over `nomos-package`'s existing core
is a proportionate build the same way `nomos-lang-package` was, or the second rule's shape
diverges enough from `Check_Completeness_Mirrors`'s that no single schema built from the first
alone would have fit it — in which case the wait was load-bearing rather than merely cautious.
Either outcome also gives `PackageKind::RulePackage` the first consumer `OD-PACKAGE-001` said
it was waiting for, closing that half of the condition recorded on `PackageKind` itself.

## Status

Open. Revisit when a second rule (function or crate) is added under `crates/rules/`, or if
`ARCH-002`'s contents list is found to need a manifest sooner for a reason unrelated to rule
count. Recorded here so the asymmetry between `ARCH-001`'s now-satisfied mechanism and
`ARCH-002`'s untouched one is legible rather than silently inherited by whichever item next
touches `crates/rules/nomos-rules` or `crates/contracts/nomos-contracts/src/package.rs`.
