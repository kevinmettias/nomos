---
id: OD-EXECUTOR-009
type: decision
title: Knowledge context cannot bridge to a peer that has not exposed an interface yet
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - executor
  - ecosystem
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: D-137
    type: relates-to
  - target: OD-EXECUTOR-007
    type: relates-to
---

# Knowledge context cannot bridge to a peer that has not exposed an interface yet

## Question

`TaskEnvelope.knowledge_context: Vec<KnowledgeReferenceId>` is a declared field the executor
ignores. A person required a real bridge: acquisition from a real source, provenance per
item, task-scoped selection decided by the system rather than guessed by a caller, and
promotion back across the boundary either implemented or recorded as refused, all without
normative authority leaving Nomos. `OD-EXECUTOR-007` named this same field as one of
`TaskEnvelope`'s unenforced four and declined to build it speculatively, alongside `scope`,
`prohibited_changes` and `available_tools`, which that record did decide. Whether
`knowledge_context` is now buildable needed checking directly against KWB itself, not
assumed from the field's own shape.

## What Was Measured

**`KnowledgeReferenceId` was admitted on exactly this understanding, and nothing has changed
it.** `D-137`'s own title says so outright: "`KnowledgeReferenceId` is admitted to
nomos-contracts as the one shape a KWB citation needs, and nothing yet produces or consumes
one." Two committed requirement assessments cite it as their gap for the identical reason:
`AGT-007.assessment` — "the KWB half... is unreachable: `KnowledgeReferenceId` exists (D-137)
but, by that record's own text, 'nothing yet produces or consumes one'" — and
`AGT-017.assessment`, the same citation for a handoff's cross-system links.

**`ARC-ECOSYSTEM-001` draws the boundary precisely and deliberately builds nothing across
it.** KWB owns knowledge — rationale, semantic intent, decision context, long-term epistemic
memory; Nomos owns software-specific reality. The record diagrams the crossing (KWB semantic
intent through a governed projection into a Nomos executable contract) and states the rule a
bridge must honour — a rationale is not enforceable, only a derived contract is, and the
derivation must be a recorded step — but names no concrete interface, protocol, or wire
format, and says outright: "No KWB or XVPE integration is implemented, and none is scheduled
here." The boundary is a governance decision, not yet a crossing anything could build
against.

**KWB itself was read directly, not assumed from its own architecture documents.** It is
real and structured — ten crates, band-organized, densely commented with the same
contract-first discipline this workspace uses — and every one of them is an unimplemented
scaffold today: `kwb-cli/src/main.rs` prints "no verbs are wired yet"; `kwb-mcp/src/main.rs`
prints "no tools are wired yet" and names nine planned read-only tools none of which exist;
`kwb-platform-std/src/lib.rs` says plainly "Nothing is implemented yet"; `kwb-contracts`'s own
crate stays deliberately empty because, in its own words, the shape a citation identity
string takes "is a decision rather than a guess" until `kwb-model` exists to derive it from.
There is no CLI, no HTTP endpoint, no file export, and no library surface on the KWB side
today that anything could call or read.

**This is not a design gap Nomos can close by itself.** `OD-RULES-024` and `OD-RULES-025`
both found a missing fact or convention *this workspace* could eventually decide and
materialize. Here the blocker is external: KWB has not yet decided what a
`KnowledgeReferenceId`'s own string content is, has not built retrieval or query, and has
wired no host at all. A Nomos-side `nomos-knowledge-context` crate built now would have no
real peer to acquire from, select against, or promote to — it would mean inventing both ends
of a protocol against an interface that does not exist, the identical speculative-building
`D-137` already declined when it admitted the identifier alone and stopped there.

## The Decision

**Acquisition and selection are not built here.** Both presuppose a KWB-side interface —
something that answers a query, something that names a citation's shape — that this
workspace does not control and cannot honestly stand in for. Building either now would be
guessing at a peer's contract before the peer has one, which `ARC-ECOSYSTEM-001`'s own
ownership split exists to prevent as much as to enable.

**Promotion back across the boundary is recorded as refused, and this much is genuinely
free.** `ARC-ECOSYSTEM-001` already forbids normative authority leaving Nomos, and since
nothing yet produces a `knowledge_context` entry, there is nothing a promotion path would
carry. Refusing it costs nothing and states a true fact rather than an aspiration: there is
no promotion mechanism because there is no acquisition mechanism for it to promote from.

## What This Record Does Not Do

**No code changes here.** It does not build `crates/orchestration/nomos-knowledge-context`,
which is not authored — that territory presumed a real KWB peer that does not exist yet, the
same way `OD-RULES-024`'s architecture-drift territory presumed a capability that was never
built.

It does not amend `ARC-ECOSYSTEM-001` or `D-137`. Both already say precisely what this
record measured; this record confirms their claims are still current rather than
superseding them.

It does not withdraw the requirement. `AGT-007` and `AGT-017` stay `Partial` for the
identical reason they already are: the gap is real, and closing it is gated on KWB's own
progress, not on a decision Nomos has been withholding.

It does not say KWB will never be ready. Revisiting this is conditioned on an observable
KWB-side fact — a real host binary that answers a query, or a decided citation-identity
shape in `kwb-contracts` — not on a timer or another Nomos-side design pass.

## Status

Accepted. Knowledge-context acquisition and selection are blocked on KWB exposing a real
interface, which it has not done; promotion back across the boundary is recorded as refused,
honestly, because nothing yet exists to promote.
