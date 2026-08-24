---
id: OD-PACKAGE-008
type: decision
title: Whether RulePackage needs its manifest crate now that nomos-package exists, or stays a bare rule bounded to a population of one
status: accepted
version: 5
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
  - target: D-134
    type: relates-to
  - target: OD-RULES-003
    type: relates-to
  - target: OD-RULES-008
    type: relates-to
  - target: OD-ROADMAP-001
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

## Corroborating Evidence

`code-standards`/`nomos-proto` (`github.com/kevinmettias/nomos-proto`), the Go-era
predecessor this workspace rebuilds, tried independent per-artifact packaging at
approximately this population and reversed it, for a concrete technical reason rather than
a population-of-one caution alone. Its `go.work` carries this comment verbatim, verified
directly against the file rather than secondhand:

> This file carried 45 entries — one per check — because every check was its own Go module.
> That boundary was packaging, never design: it forced the gate to `go run` each check, and
> it is why a shared driver could not simply be imported.

What this shows: a mature, production system directly analogous to `nomos-rules` — checks
there play the same role rules do here — packaged each unit independently as its own Go
module, and collapsed that back to one module/one binary because the packaging boundary
itself broke composition (a shared driver could no longer be `import`ed across it, only
invoked as a subprocess via `go run`). The reversal was driven by a driver-composition cost,
not merely by having too few checks to design a schema against.

What this does not show: that `nomos-rules`' own eventual second rule will diverge from
`Check_Completeness_Mirrors` the same way, or that a `nomos-rule-package` wrapper crate
would reproduce Go's specific failure — Rust's crate/module system is not Go's, and this
record's own reasoning above (a `RulePackage`'s contents list containing data corpora and
executable fixtures no version comparison can reason about) is a different, independently
sufficient caution. This evidence corroborates waiting for a second rule; it does not by
itself decide what shape the wait should end in.

## What Would Decide It

Historical — see the amendment below for what governs now. A second rule joining
`crates/rules/nomos-rules` — or a second rule crate — was the trigger this record originally
named, adjusted down by one from `OD-PACKAGE-006`'s third-provider trigger because rules
numbered one rather than two at the time.

## Resolution

Accepted, on the trigger this record itself named. `crates/rules/nomos-rules/src/naming.rs`
(`Check_Naming_Convention`, `P13-RULE-NAMING-CONVENTION`) is now the second rule this record
waited for, and the field-by-field comparison against `ARCH-002`'s contents list is possible
on real evidence rather than a population of one.

The two rules converge on some fields and diverge sharply on others, and the divergence lands
exactly where `ARCH-002` and `PKG-007` care most. Convergent: both state
`required canonical capabilities` as `crate::Syntax_Requirement()`, unchanged between them;
both carry a `deterministic judgment implementation` of the same shape, a pure function of
`&[SourceFile]` and a `FactReader` returning `Vec<Finding>`; both use the same
`evidence schema`, `EvidenceClass::Derived`; neither has an `optional enhanced
implementation`, an `external diagnostic mapping`, or a `correction and suppression
contract`; both carry `examples/counterexamples` only as hand-written fixture text inside
their own `#[cfg(test)]` modules, not as an external evaluation corpus — `mirror.rs`'s three
historical instances live in its own test fixtures the same way `naming.rs`'s do, so this is
convergence in shape, not merely absence in both.

Divergent, and divergent on `identity/version` and `normative specification` themselves —
`ARCH-002`'s first two contents and the two `PKG-007` names directly: `Check_Completeness_
Mirrors` cites a versioned governing record through `nomos_rules::CONTRACT_RECORD` /
`CONTRACT_RECORD_VERSION` (`"D-134"`, version 2), mechanically checked against that record's
own front matter by `tests/contract/tests/rule_contract_citation.rs`. `Check_Naming_
Convention` deliberately has neither — its own module doc's `# Why this has no
CONTRACT_RECORD` section states its contract is `README.md`'s Conventions section, prose
with no `version:` field the citation mechanism could read, and that `PKG-014`'s
traceability requirement is accordingly not mechanically checked for it, "the same way it was
not checked for the first one before that citation existed." This is not a superficial
difference the way a naming or module-layout choice would be: `nomos_rules::CONTRACT_RECORD`
is a single crate-level constant today, not a per-rule field, so the citation mechanism itself
is singular and already shaped around exactly one rule having a versioned contract. A second
rule that structurally lacks one is the concrete case `ARCH-002`'s independent-versioning
requirement and `PKG-007`'s four version domains assume will not happen — a manifest domain
built from `mirror.rs`'s shape alone would have a `contract_record`/`contract_record_version`
pair with nothing for `Check_Naming_Convention` to put in it.

That is the second outcome this record's own "What Would Decide It" section named: the
second rule's shape diverges enough from `Check_Completeness_Mirrors`'s that no single schema
built from the first alone would have fit it. The wait was load-bearing, not merely cautious.
No `nomos-rule-package` crate is scaffolded now.

One prediction that section made does not hold, and is corrected here rather than left to
read as settled: it stated "either outcome also gives `PackageKind::RulePackage` the first
consumer `OD-PACKAGE-001` said it was waiting for." That is true of the outcome not taken —
scaffolding a wrapper crate with a reader that resolves `PackageKind::RulePackage` would have
been a real consumer — but not of this one. Declining to build leaves `RulePackage` exactly
as unconsumed as `OD-PACKAGE-001` and `crates/contracts/nomos-contracts/src/package.rs`'s own
module doc already found: no manifest, no reader, no `PackageId` constructed. That half of the
condition recorded on `PackageKind` stays open, and this resolution does not close it.

## A Third And Fourth Rule Arrive: What The Population Now Shows

The trigger this record named at version 3 — "a third rule ... is found to need a
version-bearing contract citation" — has fired, twice over. `Check_Dependency_Direction`
(`crates/rules/nomos-rules/src/dependency.rs`) cites `DEPENDENCY_CONTRACT_RECORD = "OD-RULES-003"`,
`DEPENDENCY_CONTRACT_RECORD_VERSION = 1`. `Check_Unread_Reaches_A_Finding`
(`crates/rules/nomos-rules/src/reachability.rs`) cites
`UNREAD_REACHES_FINDING_CONTRACT_RECORD = "OD-RULES-008"`,
`UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION = 2`. Both are checked the same way `D-134`'s
citation is, by `tests/contract/tests/rule_contract_citation.rs` against each record's own
front matter. Re-reading this record's own comparison at the population it names — four rules,
not two — rather than re-affirming the version-3 outcome by citation count alone changes what
converges and what diverges.

Identity/version and normative specification, the field the trigger itself names, is the
field that most changes shape. At version 3 it was a 1-of-2 split — one rule cited, one did
not — and this record read that as evidence the field was unsettled. At four rules it is 3-of-4:
`Check_Completeness_Mirrors`, `Check_Dependency_Direction` and `Check_Unread_Reaches_A_Finding`
all cite a versioned governing record; `Check_Naming_Convention` remains the sole exception, and
it remains a documented one — its own module doc's "Why this has no `CONTRACT_RECORD`" section
still gives the same reason version 3 already read. Three independent rules, designed at
different times against different capabilities, converging on the same citation shape is
materially more evidence for that field's stability than one rule doing so alone. Read on its
own, this axis would argue for building the wrapper now rather than against it.

It is not read on its own, because two other fields moved the opposite direction over the same
two rules. Required canonical capabilities, which version 3 found unchanged between the first
two rules (both `crate::Syntax_Requirement()`, against `nomos_cap_syntax`), is no longer
convergent across four. `Check_Dependency_Direction` requires `nomos_cap_dependency` via its own
`Dependency_Requirement()` — `FactVariant::SemanticallyResolved`, soundness and completeness both
`Sound`, `IncrementalGranularity::Project`. `Check_Unread_Reaches_A_Finding` requires
`nomos_cap_controlflow` via its own `Reachability_Requirement()` — `FactVariant::Syntactic`,
soundness `Sound`, completeness `Unknown`, `IncrementalGranularity::File`. These are not two
variations on one shape; they are two more distinct capability families, each with its own
`FactVariant`/`Assurance`/`IncrementalGranularity` combination, joining `nomos_cap_syntax`. A
manifest field for "required canonical capabilities" built from the first two rules alone would
have had one shape to generalize from; it now has three, and nothing yet says three is the
ceiling rather than a running count.

Applicability semantics is a field version 3 did not examine, because it had not yet diverged.
`Check_Completeness_Mirrors`, `Check_Naming_Convention` and `Check_Dependency_Direction` each
raise `Applicability::Supported` for a genuine violation. `Check_Unread_Reaches_A_Finding`
structurally never does — every finding it raises carries `Applicability::PartiallySupported`
(`reachability.rs`'s own `Violation`), by the rule's own stated design: its tier-1 provider is a
heuristic over one file's parse tree, evaluating part of `OD-RULES-008`'s question rather than
the whole of it, and the rule reports that honestly rather than rounding up. This is a second
axis version 3's population of two could not have shown, because no rule needed it until this
one.

What still converges, with no exception across all four: the judgment implementation shape — a
pure function of `&[SourceFile]` and a `FactReader` returning `Vec<Finding>`, split into a
`Payload_Of`/`Violations_In` pair each later rule's own doc comment names as reused from the one
before it (`naming.rs`'s split, then `dependency.rs`'s, then `reachability.rs`'s, each citing the
last); the evidence schema, `EvidenceClass::Derived`, unchanged in every rule; and the complete
absence, in every one of the four, of any real instance of an optional enhanced implementation,
an external diagnostic mapping, a correction and suppression contract, an evaluation corpus, or
agent-guidance fragments — five of `ARCH-002`'s contents-list items with zero real shape to build
a manifest field against anywhere in the workspace today, a caution this record raised from a
population of one and can now report checked against a population of four rather than assumed.

This does not reverse the resolution above at the time it was written: no `nomos-rule-package`
crate was scaffolded at version 4. But the reasoning the resolution stood on was corrected
rather than merely reaffirmed. One of the two original axes — contract-citation instability —
was no longer well supported by the real population, and two other axes that version 3 could
not see, because the population was too small to show them, newly diverged as the population
grew from two rules to four. See the amendment below for what that means now that the wait
itself, not merely the evidence for it, is retired.

## Amendment: The Wait Is Retired; Build From What Four Rules Actually Show

Added at version 5. `OD-ROADMAP-001` retires waiting for a fifth rule, or for any further
convergence, before scaffolding `nomos-rule-package`. The population-of-one caution this
record originally raised, and the population-of-four re-measurement the section above
performed, stay exactly as useful as they were — they are the actual field-by-field evidence
for how to shape `RulePackage`'s manifest well, not evidence for whether to build it at all.
Read together, four real rules already show: `identity/version` should be optional or
per-rule rather than assumed present, since `Check_Naming_Convention` genuinely lacks it;
`required canonical capabilities` needs to carry a real `FactVariant`/`Assurance`/
`IncrementalGranularity` triple per rule, since three distinct ones are already observed;
`applicability semantics` needs to distinguish a rule that always raises `Supported` from one
that structurally cannot, since `Check_Unread_Reaches_A_Finding` is the latter; and five of
`ARCH-002`'s contents-list fields (optional enhanced implementation, external diagnostic
mapping, correction and suppression contract, evaluation corpus, agent-guidance fragments)
have no real instance across any of the four rules to shape a field from, and building them
speculatively is exactly what `OD-ROADMAP-001` now authorizes doing anyway — those fields
should be built from `ARCH-002`'s own corpus text and this workspace's nearest analogous
types (`nomos-corrections` for a correction contract, `nomos_contracts::Finding` for
diagnostic mapping) rather than left unbuilt for want of a fifth rule that exercises them.

## Status

Accepted. The trigger this record named at version 3 fired twice — `Check_Dependency_
Direction` and `Check_Unread_Reaches_A_Finding` both cite a version-bearing contract record —
and the comparison was redone at the real population of four rather than reaffirmed by count
alone. Amended to version 5 by `P13-ROADMAP-001-POPULATION-CAUTION-RETIRED`: the wait for a
fifth rule or further convergence is retired via `OD-ROADMAP-001`, and `nomos-rule-package`
is in scope to build now from the four-rule field-by-field measurement this record already
performed, not from an invented shape.
