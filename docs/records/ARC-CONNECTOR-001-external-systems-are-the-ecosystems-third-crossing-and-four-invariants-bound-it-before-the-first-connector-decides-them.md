---
id: ARC-CONNECTOR-001
type: architecture
title: External systems are the ecosystem's third crossing, and four invariants bound it before the first connector decides them by default
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - ecosystem
  - connectors
  - evidence
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-130
    type: relates-to
  - target: OD-MODEL-002
    type: relates-to
  - target: D-137
    type: relates-to
  - target: OD-ANALYSIS-001
    type: relates-to
---

# External systems are the ecosystem's third crossing, and four invariants bound it before the first connector decides them by default

## Question

`ARC-ECOSYSTEM-001` draws and governs two crossings: KWB semantic intent projects down into
a Nomos executable contract, and an XVPE generic primitive is consumed upward by adaptation.
An external system of record — an issue tracker, a wiki, a document store — is neither. It
is not knowledge, and it is not a generic runtime primitive; it is a system this workspace
does not run, whose facts arrive from outside every crossing that record already named.

Nothing in this workspace names one. A search for `connector`, `external system`, `GitHub`,
`Jira`, `Confluence` or `SharePoint` across every `.rs` file in the tree returns three files
— `crates/host/nomos-cli/tests/gate_step/workflow.rs`,
`crates/host/nomos-cli/tests/gate_step/main.rs` and
`crates/substrate/nomos-ledger/src/gate.rs` — and all three are this repository's own gate
step, not an external service. There is no crate, no trait, no `EvidenceClass` assignment
and no identity rule for state that originates outside the workspace, and the first
connector written would decide all four by being the only thing that has.

That is the same shape `OD-AGENT-001` recorded for the KWB boundary before
`ARC-ECOSYSTEM-001` answered it: not a missing feature, a missing decision that the first
implementation would make silently. The specific risk here is authority. An external
tracker's status field is a fact about that tracker, not about this codebase, and a
connector that returns it as an ordinary value hands a vendor's workflow the standing of a
Nomos verdict. `EvidenceClass` already exists to keep those apart, and nothing today routes
an external read through it.

This record names the crossing and settles what can be settled about it now, the same way
`ARC-ECOSYSTEM-002` settled the landing spot for an extraction before anyone opened the old
KWB tree. It cites `ARC-ECOSYSTEM-001` rather than restating it: a seam decided in two
record sets where neither names the other is the duplication a seam record exists to
prevent, which is exactly what `ARC-ECOSYSTEM-001` itself needed a citation pass to
correct once already, over the sibling suites that had reached parts of it first.

## The Third Crossing, And Its Layers

```
External system of record  (GitHub, Jira, Confluence, SharePoint, ...)
        |
        |  generic connector substrate — protocol, auth transport, retry, rate limit
        v
Vendor schema and vendor authentication
        |
        |  vendor-to-canonical translation — a governed projection
        v
Nomos canonical fact, entering as Observed evidence
        |
        |  unchanged: the two crossings ARC-ECOSYSTEM-001 already governs
        v
KWB semantic intent <-> Nomos executable contract
XVPE generic primitive <-> Nomos-specific service (by adaptation)
```

Four layers, and ownership follows `ARC-ECOSYSTEM-001`'s existing vocabulary rather than a
new one:

**The external system of record** is outside the workspace and outside this ecosystem
entirely. Nomos does not own it, does not own its schema, and does not own its workflow
semantics. It stands to a connector the way the old C# KWB tree stands to the KWB rewrite
in `ARC-ECOSYSTEM-002`: a source a connector reads, never a system this repository is
responsible for being correct.

**The generic connector substrate** — the protocol client, the auth transport, retry and
backoff, rate limiting: the mechanics of talking to *any* external API-shaped system, with
no software-engineering or Nomos semantics attached — is XVPE's, on the same criterion
`ARC-ECOSYSTEM-001` already states for that product: "the primitives that are reusable
without any Nomos semantics attached." It is reached on the same terms `D-130` already sets
for anything XVPE, before or after Phase 5: never a `path` dependency, and only through a
single named, quarantined adapter crate if adopted early. This record does not mint that
crate's name; naming an empty one now would be exactly the mistake
`ARC-ECOSYSTEM-001`'s "Current Placement Does Not Prove Permanent Ownership" clause warns
against — deciding an address before anything needs one.

**The vendor-to-canonical translation** is Nomos's, and it is a governed projection in the
sense `ARC-ECOSYSTEM-001` already uses for the other two crossings: authority changes hands
here, so the translation must be a recorded, inspectable step rather than an inference made
once and trusted afterward. It is the only layer permitted to know a specific vendor's
field names, status enumerations, or API shape.

**The two crossings `ARC-ECOSYSTEM-001` already governs** are untouched by this record. Once
a fact is translated into canonical form it is ordinary Nomos evidence, subject to the same
downward (KWB rationale into a Nomos contract) and upward (XVPE primitive adapted into a
Nomos service) crossings as any other fact this workspace produces. This record adds a floor
beneath both. It does not touch either, and in particular it does not open the
observations-to-knowledge crossing `P11-ECOSYSTEM-UPWARD` governs — an external fact
translated to canonical form is Nomos evidence like any other, and whether *that* may
generalize into KWB knowledge is that item's question, not this one's, exactly as
`ARC-ECOSYSTEM-002` already declined to open it for an extracted KWB finding.

## The Four Invariants

### 1. External artifact identity is an honest mint from the authority's own stable key, distinct from what the artifact bears on

**The rule, corrected.** This invariant originally read: "An external item's identity in this
workspace must be derived from what the item is *about*, not copied from the tracker's own
primary key" — one identity slot asked to answer two questions that do not move together:
which external record this is, and what Nomos currently understands it to concern. Those are
distinct entities with independent lifetimes — a record's title can change without its
concern changing, and a correction to what it concerns does not change which record it is —
and the original wording was also unimplementable as stated: a specific external record must
be nameable before any semantic meaning has been derived from it, or nothing could recognize
a record it had already seen.

The corrected rule: **an external artifact and the Nomos subject it concerns are distinct
entities.** Where the external authority assigns the artifact a stable identity-bearing key,
Nomos preserves that key through a canonically constructed, namespaced `Named_Identity` —
`OD-MODEL-002`'s second identity shape, not its first. This is an honest mint in
`OD-MODEL-002`'s sense, the same shape `D-137`'s `KnowledgeReferenceId` already licenses:
Nomos owns the construction — normalization, escaping, namespace and component ordering —
but does not claim to have derived the externally assigned identity-bearing value, and does
not disguise the mint as a derived digest by hashing it (`OD-MODEL-002`: "a mint wearing a
digest's clothes").

A location or address for the artifact that can change independently of the artifact itself
— which repository an issue currently lives in, which URL currently resolves it — is
provenance recorded beside that identity, not part of it, the same identity/provenance split
`OD-ANALYSIS-001` already made for a fact's snapshot. A locator change does not by itself
restamp the artifact's identity. Where the external authority exposes no stable identity
surviving a transition, Nomos does not fabricate continuity it cannot verify: a new identity
is minted, and any supported continuity between the old and new identity is represented
explicitly, not assumed.

What the artifact bears on — a feature, a requirement, a finding, any ordinary Nomos subject
— is represented as a relation from the artifact's identity to that subject's own identity,
never folded into either endpoint's identity. That relation is independently evidence-bearing:
its `EvidenceClass` reflects how the correlation itself was established — an explicit
reference the external record carries and a connector reads, a deterministic mapping rule, an
agent's inference, a human's assertion — never inherited from the classes of what it joins.
Two records agreeing to exist does not by itself justify treating what they are about as
settled.

**What it is waiting on.** The concrete addressing scheme, the canonical spelling syntax, and
the representation of a correlation (a typed relation type, an eventual `EGRAPH` edge, or
something else) are connector-specific and representation-specific choices this record leaves
open — see "What This Record Does Not Do." What is decided now is the shape every connector's
choices are checked against, not the choices themselves.

### 2. Vendor identity may appear as provenance data; vendor ontology may not appear in canonical schema

**The rule, tightened.** The original wording — "no canonical Nomos type may name a vendor" —
is too literal once provenance must be able to say which external authority was observed: an
external artifact's identity necessarily carries a component naming its authority, something
like `"github"` or `"jira"`. The distinction this invariant actually protects is narrower and
still holds in full: **vendor identity may appear as canonical data; vendor ontology may
never appear in canonical schema.** A canonical type's *fields* may hold a value naming the
external authority a fact or an identity was observed from or minted against. A canonical
type's *shape* — its variants, its field names, its own type system — may never be built from
a specific vendor's entity types, field schemas, status enumerations, authentication
concepts, protocol objects or SDK types. A generic identity whose `system` field holds
`"github"` is provenance data and satisfies this invariant; a `GitHubIssue` variant, a
`JiraStatus` enum, or any other canonical type carrying a vendor's own ontology in its shape
violates it. The vendor-to-canonical translation layer remains the only place that ontology
may appear in source at all.

**What it is waiting on.** Nothing normative — this is fully decidable now, as a shape
constraint the first connector's translation layer is checked against. What is not yet
decided is the container: whether the quarantine is one crate per vendor or one crate per
connector family is an implementation choice for whoever writes the first one, the same way
`nomos-platform-xvpe` in `D-130` was named only when something needed to be adopted through
it.

### 3. External state enters as Observed and never as authority

**The rule.** `EvidenceClass::Observed` — "directly observed at runtime," per
`crates/contracts/nomos-contracts/src/finding/evidence.rs` — already fits an external read
exactly: state read from a system Nomos does not control and did not check. A fact produced
by the vendor-to-canonical translation layer must carry `Observed` and never
`Verified` or `Authoritative`, because nothing in this crossing checked the claim or defined
it to be true by this system's own authority; it only read what another system currently
says. This also settles the negative case directly: a connector's return value is not
permitted to skip `EvidenceClass` and be treated as an ordinary, unclassed value, because an
unclassed value is indistinguishable from one nobody thought to weaken.

**What it is waiting on.** Nothing. This is the invariant closest to already being code —
`EvidenceClass` exists, is ordered, and its floor-and-ceiling behavior (`Weaker_Of`,
`Is_Mechanical`) already does the right thing with an `Observed` value without needing to
know it came from a connector.

### 4. A write capability is omitted from the interface, not present and permission-checked

**The rule.** The generic connector substrate and the vendor-to-canonical translation
together define only an observation interface. No method named `write`, `update`, `post`,
`close`, or any vendor-specific equivalent may exist in the type signature at all. The
enforcement is the absence of the capability, not a permission check performed when a write
is attempted — a deny-checked write method is still a write method, reachable by whoever
next changes the check, in the same way `EvidenceClass::AgentJudged` is the floor precisely
because promotion is not offered rather than offered-and-refused (`Is_Mechanical` on
`AgentJudged` and `HumanAsserted` returns `false`; there is no path that overrides it).

**What it is waiting on.** Nothing. This is fully decidable now as a shape constraint on any
connector's trait or interface, and it is checked the same way any missing method is
checked: by the interface not compiling code that calls one.

## Why This Is An Ecosystem Record And Not A Connector-Specific One

No connector exists yet, so this record is deliberately not about any one vendor. The
interesting part is not GitHub or Jira; it is that a system outside the workspace is a
distinct case from either crossing `ARC-ECOSYSTEM-001` already governs, and needs its own
answer for the same reason `ARC-ECOSYSTEM-002` needed one for a legacy codebase: both are
shapes this repository will meet more than once, and deciding the shape once, in advance,
is cheaper than re-litigating it per instance. `ARC-ECOSYSTEM-001` is cited as the boundary
this record extends rather than restated, for the reason its own version-2 citation pass
already established: restating it here would be the second-authority mistake `AGENTS.md`
warns against, and a seam decided in two places that do not name each other is the
duplication a seam record exists to prevent.

## Conflicts With Existing Decisions

`ARC-ECOSYSTEM-001` is untouched. This record names a third crossing beside the two it
already governs and applies its existing vocabulary and ownership criteria to it; the seam
itself, and the two crossings it already draws, are unchanged.

`D-130` is untouched. This record applies its adapter-crate shape one layer down, to a
connector substrate that does not exist yet; it does not weaken or reinterpret the dependency
rule itself.

`P11-ECOSYSTEM-UPWARD` is untouched and unclosed. This record explicitly declines to open
the observations-to-knowledge crossing that item governs — translated external state is
ordinary Nomos evidence, and whether it may generalize into KWB knowledge is a question this
record leaves exactly where it found it.

## What This Record Does Not Do

No connector is built. No crate is created, and no trait is defined. No vendor is named as a
target. Nothing here is implemented, and this record does not claim any of the four
invariants above is enforced mechanically today — only that each is now a rule to be checked
against, rather than a decision the first connector would otherwise make by default.

Invariant 1's correction adds no new machinery either. It does not define a concrete
`ExternalArtifact` Rust type, a canonical spelling syntax for a minted identity, or how a
correlation is represented — a typed `Relation<T>`, an `EvidenceEdge`, an eventual `EGRAPH`
edge, or something else entirely is left to whichever real connector needs one first, the
same restraint `ARC-ROADMAP-001` constraint 3 already applies to `EGRAPH` generally. It does
not define correlation-discovery algorithms, a vendor's own definition of a stable key versus
a locator, or migration/transfer semantics for any specific external system. No GitHub, Jira,
or SharePoint connector implementation is decided by this amendment any more than it was by
the original record.

## Status

Closed by `P12-CONNECTOR-SEAM`. Amended to version 2 by
`P13-ARC-CONNECTOR-001-IDENTITY-CORRECTION`: invariant 1 corrected and invariant 2 tightened,
for the reasoning each section now carries inline. Every other invariant, the layering, and
the ownership boundary are unchanged.
