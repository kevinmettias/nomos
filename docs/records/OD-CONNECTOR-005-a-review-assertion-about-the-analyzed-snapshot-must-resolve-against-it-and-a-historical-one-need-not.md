---
id: OD-CONNECTOR-005
type: decision
title: A review assertion about the analyzed snapshot must resolve against it, and a historical one need not
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - connector
  - review
  - evidence
  - contracts
relations:
  - target: ARC-CONNECTOR-001
    type: relates-to
  - target: OD-CONNECTOR-004
    type: relates-to
  - target: OD-SPEC-017
    type: relates-to
  - target: D-134
    type: relates-to
---

# A review assertion about the analyzed snapshot must resolve against it, and a historical one need not

## Question

An external architectural review of this workspace named five analysis crates that do not
exist -- a dataflow, complexity, concurrency, memory and security crate under an analysis
directory -- and described a connector type and its four fields that are not the type or the
fields in the tree. The only crate of that name is `crates/substrate/nomos-analysis`, which
is the incremental fact store rather than a semantic analysis pipeline, and the rule modules
`concurrency_text.rs` and `security_text.rs` are the nearest real things to two of the
invented names.

What makes the case worth a record is that the review's structural conclusions were largely
sound. The failure was not bad judgment; it was unresolved concrete reference, and the two
are separable. Eight governing records already name an external review as the origin of the
question they answer or as the input that caught the defect they correct -- `OD-RULES-009`
is titled for one, and `OD-GATE-023` records eight rounds of the same one. External review
is a recurring load-bearing input to this repository's decisions, not a hypothetical second
consumer. Nothing states what claim Nomos may make about an assertion that arrives that way.

## Why the obvious invariant is already refuted

The invariant this case first suggests -- every concrete repository entity a review cites
must resolve -- is falsified by this repository's own census before it could be enforced.
`OD-SPEC-017` measured that 80 of 253 paths cited inside governing records do not resolve,
found every measured cause to be a legitimate change to the tree, and decided that holding
them live would make every module reorganization a record-editing exercise.
`tests/contract/tests/record_citation.rs` implements that decision by reading no path at all.

So the distinguishing property is not citation. It is tense. A claim presented as true of
the analyzed snapshot is checkable against that snapshot; a report of what was true earlier
is not, and its failure to resolve is not a defect. `OD-SPEC-017` already had to draw this
line grammatically for its own purpose -- present tense claims coverage, past tense reports
history -- and this record scopes on the same axis rather than introducing a second one.

## The invariant

**An assertion presented as true of the analyzed snapshot must resolve against that
snapshot.** An assertion whose provenance scopes it to an earlier revision, and an assertion
explicitly stated as historical, are exempt and are not defects.

## Three categories an assertion falls into

**Resolvable.** A canonical fact or relation capable of establishing the assertion exists,
and the assertion is presented as true of the analyzed snapshot. It is checkable, and this
record's invariant applies to it.

**Historically scoped.** Provenance identifies an earlier revision or snapshot. Current
resolution is not required and its absence is not a defect. This is `OD-SPEC-017`'s case,
reached from the other side.

**Judgment.** Semantic interpretation not reducible to an existing canonical fact.
Provenance and evidence class are retained; mechanical verification is not claimed for it.

## What is mechanically resolvable, and what is not

Resolvable against a snapshot today, each by a mechanism that already exists: a workspace
member, a path, a symbol on a crate's published surface (`tests/contract/surface/`), a
dependency edge and a declared band (`nomos.cap.dependency.edges`), a registered record, a
rule identifier (`nomos_rules::DESCRIPTORS`), a capability identifier, a test name
(`tests/contract/tests/record_citation.rs`), and a cited contract at the version its record
declares (`tests/contract/tests/rule_contract_citation.rs`).

Not resolvable, and not to be claimed as such: an arbitrary natural-language proposition
about behaviour -- that a function implements intended retry semantics, say -- unless a
typed fact or contract already expresses that proposition. `OD-HOST-010` met the same wall
from inside: reconstructing which fact backed one judgment is not something a caller outside
that rule can do without duplicating the rule's own private requirement. An assertion of
this kind is a judgment assertion and stays one.

## Severity, inherited rather than invented

`D-134` already ranks this asymmetry and states it directly: a false claim of coverage
blocks, an admitted gap does not. A universe naming a check that does not exist is
`GateCategory::Blocking`; a universe naming nothing is `Advisory`.

The same ranking applies here. A present-tense assertion whose concrete reference does not
resolve is the phantom, and outranks an honest inability to establish one. An inability to
establish is not a weaker finding but a different thing entirely: it is an `Applicability`
state -- `MissingCapability`, `ProviderUnavailable`, `AnalysisFailed`,
`PartiallySupported` -- which that type already carries.

## Resolution does not promote

`ARC-CONNECTOR-001` invariant 1 decided that a bears-on relation is independently
evidence-bearing, and that its `EvidenceClass` is never inherited from the classes of what it
joins. The same non-inheritance runs in the other direction, and stating it is this record's
own addition: **mechanically resolving an assertion's concrete references establishes only
that those references resolve.** It does not promote the assertion above the evidence class
its origin permits. An external review's assertion enters as `Observed` under invariant 3,
and a resolved reference does not make it `Verified`. Without this, a validation step would
read as a verification step, which is the failure `EvidenceClass::AgentJudged` already exists
to prevent one level down.

## The precondition this invariant does not have

`ARC-CONNECTOR-001` invariant 1 already decides the shape: an external artifact and the Nomos
subject it concerns are distinct entities, and what the artifact bears on is a relation from
one identity to the other, never folded into either endpoint. Its own "what it is waiting on"
leaves the concrete representation to whichever real connector needs one first.

Nothing has needed one yet. `nomos-connector-coderabbit` files every fact under the
whole-workspace placeholder subject, and
`crates/connectors/nomos-connector-coderabbit/src/provider.rs` states why in its own words: a
connector's fact about a finding has no Nomos subject until a bears-on relation names a
narrower one. A code finding at least carries a path in its payload. An architectural
assertion -- that one product owns a responsibility rather than another, the subject
`ARC-ECOSYSTEM-001` governs -- has no file subject at all.

So this invariant is stated and not yet enforceable, and the gap is exactly one relation
wide. This record does not decide that relation's representation, for the same reason
`ARC-CONNECTOR-001` did not.

## What this record does not do

No code accompanies it. It defines no type, no enum, no rule and no capability, and it does
not claim any part of the invariant is mechanically enforced today.

It does not amend `ARC-CONNECTOR-001`, `OD-SPEC-017` or `D-134`. Each already says what this
record cites it for; this record adds the snapshot scoping, the three categories, the
non-promotion statement, and the enumeration of what is resolvable.

It does not decide the bears-on relation, the assertion-extraction mechanism, or how a review
artifact is admitted.

It does not settle `OD-CONNECTOR-004`'s open question. A finding's disposition and an
assertion's groundedness are different properties, and a provider comparison remains blocked
on a second real review provider.

It does not bundle the historical-regression rule, runtime fact production, or
requirement-to-rule keying. Each is a separate consequence with its own cause.

## Status

Accepted.
