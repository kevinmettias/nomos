---
id: OD-CONTRACTS-005
type: decision
title: A type is admitted to the protocol kernel only when an independent peer must read it without linking Nomos
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - contracts
  - layering
  - protocol
  - admission
relations:
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-WORKFLOW-003
    type: relates-to
  - target: ARC-ECOSYSTEM-001
    type: relates-to
---

# A type is admitted to the protocol kernel only when an independent peer must read it without linking Nomos

## Question

`nomos-contracts` started as a thin protocol kernel and has grown: authority classes,
semantic change classes, knowledge-source roles, the workflow step contract, the finding
and reporting vocabulary, package kinds, peer synchronisation, determinism and
enforcement, several files now hundreds of lines. `OD-CONTRACTS-001` already states an
admission test in the abstract -- "a type is admitted when it crosses a subsystem, process
or plugin boundary and the parties on both sides need one stable shared representation of
it," decided by asking "would a peer that never compiles this crate be unable to agree
with us without this type?" -- but has never been walked through the types that have
since accumulated here. The specification distinguishes a protocol and identity kernel
from canonical models, the provider and package layer, and the analysis engines; it does
not say every stable domain model belongs in the kernel by default, and without a
type-by-type accounting the lowest crate becomes the path of least resistance for
anything two parts of this workspace happen to share.

This record sharpens `OD-CONTRACTS-001`'s own question into one with a concrete artifact
to check against -- *does an independently implemented peer need to understand this
type's serialized (JSON Schema) form without linking Nomos domain code?* -- and applies
it, by name, to the four families this session's own audit named: the workflow step
contract, the knowledge-context types, package kinds, and the finding and reporting
structures. No type moves under this record; the outcome is the accounting itself, and a
named destination for anything that fails it.

## The test

**A type earns a place in `nomos-contracts` only when a system that has never linked this
crate -- a knowledge service in another language, an independently implemented platform
adapter, a client reading a JSON-RPC or MCP response -- must construct or interpret its
serialized form correctly in order to agree with Nomos about what happened.** The artifact
that crosses the boundary is the wire shape, not the Rust type: `lib.rs`'s own doc already
names this exception for `serde` itself -- "the artifact a peer really reads is the JSON
Schema generated from these declarations." A type nothing outside this workspace ever
deserializes, however central to Nomos's own semantics, fails the test and belongs beside
the code that gives it meaning instead.

This does not replace `OD-CONTRACTS-001`; it is that record's own question, asked with the
serialized artifact named explicitly rather than left as "one stable shared
representation." A type failing this test also fails `OD-CONTRACTS-001`'s, and a type
passing this one satisfies it by the same reasoning `OD-CONTRACTS-001`'s own worked
example (`nomos-cap-syntax::SyntaxPayload`, admitted one band up because the parties who
must agree about it are one capability's own providers, not every peer that speaks to
Nomos) already uses.

## Applying it

**`WorkflowStep` (and the `Cacheability`, `CancellationBehavior`, `Compensation`,
`RetryPolicy` and `Timeout` types it carries as fields) passes.** `OD-WORKFLOW-003`
already made this argument directly, before this record existed to ask for it: "the
contract a peer executor -- API-hosted, subscription-agent, human, or recorded-replay --
must agree with Nomos about before either side can speak about 'a step' at all... the same
admission `OD-CONTRACTS-001` already gives `RunId`, `EvidenceClass` and `GateCategory` for
the same reason." A future executor peer that has never linked `nomos-workflow-
orchestration` still has to construct or read a `WorkflowStep`'s serialized declaration
correctly to participate in Check-then-Correction-then-Gate at all. Its own five field
types are entailed by the same admission -- a peer agreeing about the step must agree
about what each of its fields means -- and are not a separate question.

**`KnowledgeContextItem` and `KnowledgeSourceRole` pass.** Their own module doc already
states the boundary directly: "vocabulary shared with an external knowledge system across
the process boundary `ARC-ECOSYSTEM-001` names." The knowledge system is exactly the
independent peer this test asks about -- a service that has never compiled this crate and
must still read what role a piece of borrowed context plays.

**`PackageKind` passes.** Its own doc states the same shape again: "a peer reimplementing
this enum in another language reads the label and never sees this Rust, so a label is a
protocol commitment rather than a local identifier." Twelve of its sixteen kinds have no
consumer inside this workspace yet, which is a different question `OD-PACKAGE-001` already
owns; the type's admission does not wait for an internal consumer, the same distinction
`WorkflowStep`'s own doc draws for a runtime engine that does not exist yet either.

**`Finding` and the reporting vocabulary it carries (`Applicability`, `EvidenceClass`,
`GateCategory`, `EnforcementBreach`, `EnforcementReach`, `EnforcerRef`) pass, and are the
clearest case of the four.** `nomos-api-transport`, `nomos-mcp` and every `--output-format
json` surface this workspace has already serialize `Finding` directly to a caller that may
never have compiled a line of this workspace's Rust -- an MCP client, a CI dashboard, a
script reading a JSON-RPC response. A `Finding` a peer could not correctly parse is a
finding that peer cannot act on, which is the exact failure this crate's own "honesty
vocabularies" section says the five look-alike enums exist to prevent.

## What This Does Not Do

It does not move any type. Every one of the four families named above stays exactly where
it is, because every one of them passes the sharpened test on inspection -- the audit this
session's own why names has been carried out, and its answer is that this crate's current
population is not the problem `OD-CONTRACTS-001` was written to prevent. It does not
audit the whole crate: `AuthorityClass`, `SemanticChangeClass`, the determinism vocabulary,
`Guarantee`, peer synchronisation and the digest-identity types were not re-examined here,
because nothing in this session's own why named them as suspect. A future session
finding a fifth candidate applies this same test to it rather than re-deriving one. It
does not change `OD-CONTRACTS-001` itself, which stays the canonical statement of the
abstract rule; this record is its first systematic application, not its replacement.

## Status

Accepted. `WorkflowStep`, the knowledge-context types, `PackageKind`, and `Finding` with its
reporting vocabulary each pass the sharpened admission test; no type moves.
