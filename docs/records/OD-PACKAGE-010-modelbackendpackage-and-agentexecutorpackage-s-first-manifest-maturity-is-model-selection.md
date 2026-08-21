---
id: OD-PACKAGE-010
type: decision
title: ModelBackendPackage and AgentExecutorPackage's first manifest maturity is model selection
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - model
  - agent
  - roadmap
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-007
    type: relates-to
  - target: OD-PACKAGE-008
    type: relates-to
---

# ModelBackendPackage and AgentExecutorPackage's first manifest maturity is model selection

## Question

`ARC-ROADMAP-001` names model backend and agent executor infrastructure as a near-term-tier
item. `PackageKind::ModelBackendPackage` and `PackageKind::AgentExecutorPackage`
(`crates/contracts/nomos-contracts/src/package.rs`) have existed, declared and unconsumed,
since that enum's own creation. No implementation of either exists anywhere in this
workspace, the same population-of-zero this workspace has repeatedly declined to build a
manifest ahead of (`OD-PACKAGE-006`, `OD-PACKAGE-008`, both about `RulePackage`). The user
was asked directly whether to build this now, run a performance audit instead, or stop, and
chose to build it now -- an explicit, informed override of that caution, the same shape
`OD-PACKAGE-006`'s own wait was already overridden once for language/rule plugin
infrastructure. This record draws the boundary the override does not itself draw: *what*
gets built first.

## What Was Measured

Read directly from `NOMOS_V14_CORPUS`'s requirements directory: `MODEL-ROUTE-037` through
`MODEL-ROUTE-049` describe a mature model-routing system, not a manifest alone -- resolution
invalidation on catalog, entitlement or capability change (`044`); a normative
`RoutingPolicyConformanceSuite` with deterministic selector-precedence, budget-fallback and
data-boundary fixtures (`045`-`047`); conformance results preserving resolution hashes and
semantic diffs (`048`); and conformance gating at package publication, engine release and
canary validation (`049`). None of it has a real implementation anywhere in this workspace
to check a shape against, so building past a manifest's first maturity now would be
invention with no second real case -- the override authorizes building *something* now, not
building all thirteen requirements at once.

`MODEL-ROUTE-037`'s own opening clause is the one piece with a bounded, statable shape:
"Every `ModelBackendPackage` and `AgentExecutorPackage` shall expose a versioned discovered
model catalog or explicitly declare that model selection is opaque or executor-controlled."
Its second clause -- catalog entries distinguishing configured identity, discovered
identity, exact revision, mutable alias, deprecation, temporary unavailability, removal,
entitlement availability and executor-selected identity, nine states -- is its own later
maturity, named but not built here, the same way `OD-PACKAGE-001` scoped `LanguagePackage`
to identity, kind, `PKG-007`'s four version domains and provider registration, and named
`PKG-022`'s larger field list (recognition, file mappings, capability claims, canonical
mappings, environment requirements, projections, quality objectives, conformance suites) as
explicitly out of scope for that first step.

Read in full before drafting any code: `crates/packages/nomos-package`'s real shape
(`PackageManifest`, `Parse_Manifest`, `ProviderRegistration`) and
`crates/packages/nomos-lang-package`'s real shape, the one existing precedent for a second
`PackageKind` gaining a manifest maturity. Two things do not carry over cleanly.
`nomos_package::Parse_Manifest`'s `Package_Kind_Field` hard-refuses every `PackageKind`
except `LanguagePackage` today -- a one-line check (`crates/packages/nomos-package/src/
reader.rs`), not an architectural wall this record needs to argue past. And `PKG-007`'s
fourth version domain, as `nomos-package` and `nomos-lang-package` both build it
(`language_versions` / `providers`, `ProviderRegistration`'s `tool_version`), is genuinely
Rust/language-tooling-shaped: a model backend does not register a `nomos_capability`
provider the way `nomos-lang-rust` does. Reusing that field for this kind would produce a
manifest that "resolves" while carrying none of `MODEL-ROUTE-037`'s real content -- the
same hollow-surface risk this workspace's honesty vocabularies (`Applicability::
PartiallySupported`) exist to name rather than paper over.

## The Decision

**The first `ModelBackendPackage`/`AgentExecutorPackage` manifest maturity carries identity,
`PackageKind` (restricted to those two), `PKG-007`'s first two version domains unchanged
from `nomos-package` (`package_version`, `protocol_range`), and a new `ModelSelection`
domain replacing the fourth -- `Opaque`, `ExecutorControlled`, or `Catalog` of raw,
unresolved model identifiers -- satisfying `MODEL-ROUTE-037`'s opening clause and nothing
past it.**

A new crate, `nomos-model-package`, band 26 (a peer of `nomos-lang-package`, both
manifest-maturity crates wrapping `nomos-package`'s generic core), carries this. It reuses
`nomos_package::{PackageVersion, ProtocolRange}` unchanged -- both domains are genuinely
kind-agnostic, the same reason `nomos-lang-package` re-exports them rather than redefining
them. It does not reuse `PackageManifest`, `Parse_Manifest` or `ProviderRegistration`: the
fourth domain's real shape for this kind is `ModelSelection`, not a provider list, and the
reader's accepted `PackageKind` set is `{ModelBackendPackage, AgentExecutorPackage}`, not
`{LanguagePackage}`. Catalog entries stay raw strings, unresolved -- the same choice
`nomos_package::PackageManifest::language_versions` made for its own third domain before a
language-specific crate existed to resolve it, because there is no one typed shape a catalog
entry takes yet, only `MODEL-ROUTE-037`'s nine-state list of what it would eventually need
to distinguish.

## What This Does Not Do

It does not build `MODEL-ROUTE-038` through `049`: no resolution invalidation, no
`RoutingPolicyConformanceSuite`, no conformance-result preservation, no publication or
canary gating. It does not resolve a catalog entry's nine-state shape -- `Catalog` carries
raw identifiers, not a typed entry, the same way `language_versions: Vec<String>` did before
`RustEdition` existed to resolve it. It does not reopen `OD-PACKAGE-006` or `OD-PACKAGE-008`
for `RulePackage`, which stays exactly as deferred as those records left it -- this override
is scoped to `ModelBackendPackage`/`AgentExecutorPackage` alone. It does not give either
package kind a real backend or executor implementation; nothing in this workspace registers
one, and this manifest format is checkable against a fixture, not against a second real
instance, until one exists.

## Status

Accepted. Names the first manifest maturity for `ModelBackendPackage` and
`AgentExecutorPackage`, under the user's explicit override of the population-of-zero caution
this workspace otherwise holds, and the two clauses (`MODEL-ROUTE-037`'s catalog-entry
detail, `038`-`049`'s routing/conformance system) it deliberately leaves for a later
increment to name against a real case.
