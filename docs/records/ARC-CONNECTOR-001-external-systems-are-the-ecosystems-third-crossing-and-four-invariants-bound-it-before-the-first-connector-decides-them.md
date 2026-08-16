---
id: ARC-CONNECTOR-001
type: architecture
title: External systems are the ecosystem's third crossing, and four invariants bound it before the first connector decides them by default
status: accepted
version: 1
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

### 1. External identity is derived from semantic addressing, not minted from a vendor key

**The rule.** An external item's identity in this workspace must be derived from what the
item is *about*, not copied from the tracker's own primary key. This is not a new
principle invented for connectors — it is the same one `crates/substrate/nomos-analysis`
already applies to a fact (`FactIdentity`/`FactKey`, keyed on what is measured rather than
on an incidental label) and the same one `D-136`/`ARC-ECOSYSTEM-002` already state for a
claim and a concept: identity is content-derived. A vendor's issue number is exactly the
incidental label that pattern already refuses to key on, because it identifies a database
row in a system this workspace does not own, not the thing the row is about.

**What it is waiting on.** The concrete addressing scheme is connector-specific — a GitHub
issue's semantic address is not shaped like a Jira ticket's or a SharePoint document's — so
this record settles the principle and leaves the scheme itself to the first connector that
needs one. That connector's addressing is checked against this rule; it does not get to
establish the rule by being first.

### 2. Vendor schemas and authentication stay below the seam

**The rule.** No canonical Nomos type may name a vendor. The vendor-to-canonical translation
layer is the only place a vendor's field names, status enumerations, authentication scheme,
or API shape may appear in source at all; nothing above that layer may import or reference
them, directly or in a type name. A `Finding`, an `EvidenceClass` assignment, or any other
canonical value that mentions "GitHub" or "Jira" in its type has let the translation leak
past its own layer.

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

## Status

Closed by `P12-CONNECTOR-SEAM`.
