---
id: OD-PACKAGE-014
type: decision
title: A LanguagePackage's activation semantics are a conformance claim, not an installer action
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - package
  - capability
  - conformance
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-PACKAGE-001
    type: relates-to
  - target: OD-PACKAGE-007
    type: relates-to
  - target: OD-HOST-004
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
---

# A LanguagePackage's activation semantics are a conformance claim, not an installer action

## Question

`ARC-ECOSYSTEM-001` v4's adopted `D-091` clause keeps "package activation semantics" on
Nomos's side of the mechanism/meaning seam — `P14-ROADMAP-001-ACTIVATION-SEMANTICS-
CORRECTION` fixed `ARC-ROADMAP-001`'s own restatement of exactly this carve-out. Nothing
before this record says what that phrase means for the one real manifest type this
workspace has today, `nomos-lang-rust-package::LanguagePackage`. `OD-PACKAGE-001`'s own
four-step path — an installable manifest exists, something reads and refuses it, the one
rule this tree has cites a contract version, only then does `publish` arise — never mentions
activation at all, so this is a genuinely open question rather than one already answered and
merely unnamed.

## What Was Measured

**Nothing calls the reader outside the package crates themselves.** Grepped across every
`.rs` file in this workspace for `Parse_Manifest`/`Read_Manifest`: every call site is inside
`nomos-package`, `nomos-lang-rust-package`, `nomos-model-package` or `nomos-rule-package`'s
own source. No composition root, no CLI verb, nothing under `nomos-check-orchestration` or
`nomos-cli` ever reads a `LanguagePackage` manifest. A manifest can be parsed and validated
today, and nothing in this workspace ever does.

**`LanguagePackage::providers` already carries exactly what would need checking.**
`ProviderRegistration { provider: ProviderId, tool_version: PackageVersion }` — a manifest's
own claim about which providers it registers. Composition today is entirely hand-written:
`crates/orchestration/nomos-check-orchestration/src/composition.rs::Registered()` calls
`registry.Offer(nomos_lang_rust::Provider_Offer())` and its siblings directly, with no
manifest anywhere in that path. `nomos_capability::Registry::Offers(&CapabilityId) ->
&[ProviderOffer]` already exists and already answers, for any capability, exactly what is
actually registered — the observed half of a comparison a manifest's declared `providers`
list is the declared half of.

**`OD-HOST-004` already forecloses "activation drives composition."** It decided composition
needs no selection mechanism at any provider count: the registry ranks, the caller states a
`Requirement`, one more hand-written `Offer` call is composition, not choice. A manifest that
caused `Registered()` to add or omit an `Offer` call would be exactly the selection mechanism
`OD-HOST-004` declined to build, regardless of how many real cases exist. So "a
`LanguagePackage`'s activation semantics" cannot mean "the manifest's declared providers
become the registered ones" — that reading contradicts an already-decided record, not merely
an unbuilt one.

**The declared-vs-observed shape this workspace already uses elsewhere fits without
inventing anything.** `OD-RULES-003` designed exactly this composition for architecture:
declared data compared against an observed fact, judged into a `Finding`. `P14-RULES-
DEPENDENCY-COMPLETENESS` applied the identical shape a second time, for coverage instead of
direction. A `LanguagePackage`'s declared `providers` compared against `Registry::Offers`'
actually-registered set is the same shape a third time: declared package content is data: a
mismatch — a manifest claiming a provider composition never registered, or a registered
provider no installed package claims — is a fact worth a `Finding`, not silence and not a
build failure.

## The Decision

**A `LanguagePackage`'s activation semantics are the conformance claim that its declared
`providers` match what `nomos_capability::Registry` actually offers for each one's
capability — checked and reported, never enacted.** Nomos owns judging whether an installed
package's declared content agrees with the running composition it describes; it does not own
making the composition agree with the package, which would be the installer/materializer
mechanism `D-091` already places on XVPE's side of the seam. This is meaning, not mechanism,
exactly the line `ARC-ECOSYSTEM-001` draws: `PackageKind`'s variant semantics and Nomos's own
compatibility/capability-declaration/permission rules are what "activation" describes here,
not a privileged manifest-reading composition step.

Concretely, when this is built, it takes `Check_Dependency_Direction`/`Check_Every_Member_
Declares_A_Band`'s own shape: a rule reading a manifest's declared `providers` (the "declared
architecture" half) against `Registry::Offers` (the "observed fact" half), producing a
`Finding` on mismatch in either direction. It needs no new capability contract — `Registry::
Offers` is already a public method of an existing, composed type — and no change to `OD-
HOST-004`'s hand-written composition.

## What This Record Does Not Do

It does not build the rule, add a capability, or wire anything into `nomos-check-
orchestration::Run`. It names the shape a future increment takes, the same way `OD-RULES-003`
named `Check_Dependency_Direction`'s shape before `P13-DEPENDENCY-EDGES-2` built it.

It does not read `LanguagePackage`'s `language_versions` or `protocol_range` fields into this
judgment. Whether those carry their own activation semantics (a package claiming Rust
editions or a protocol range the running build does not actually satisfy) is a question this
record leaves open rather than answers by omission — a future item's own measurement, once a
real manifest exists anywhere outside a test fixture to check either field against.

It does not decide where a `LanguagePackage` manifest would be found on a real, installed
target repository (a path convention, a well-known filename) — that is squarely
`D-091`'s installation-mechanism half, XVPE's once built, and orthogonal to what activation
means once a manifest is in hand.

It does not reopen `OD-PACKAGE-001`, `OD-PACKAGE-007`, or `OD-HOST-004`. It answers a
question none of the three asked, using conclusions all three already reached.

## Status

Accepted. Names what a `LanguagePackage`'s activation semantics are; builds nothing.
