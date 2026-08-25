---
id: OD-LEDGER-036
type: decision
title: The work ledger is repository bootstrap machinery, not a Nomos product Workflow convergence target
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - work-ledger
  - architecture
  - orchestration
relations:
  - target: ARC-ECOSYSTEM-001
    type: relates-to
  - target: OD-LEDGER-001
    type: relates-to
  - target: OD-WORKFLOW-002
    type: relates-to
---

# The work ledger is repository bootstrap machinery, not a Nomos product Workflow convergence target

## Question

An external architecture review asked this directly: is `nomos-ledger`/`nomos-work-
orchestration` a Nomos product feature that should converge toward the product's general
`TaskEnvelope`/`Workflow`/`AgentExecutor`/authority/artifacts/verification model, or is it
this repository's own development infrastructure that should not shape public architecture —
naming the risk plainly: "do not let the internal agent coordination tool become the
accidental prototype whose peculiarities define generic workflow semantics."

This is not a new question to this workspace. `ARC-ECOSYSTEM-001` already names the work
ledger as one of three subsystems whose "current placement does not prove ownership," framed
as an open disjunction it deliberately declined to close: "Territory-based mutual exclusion
over a committed JSON document is either repository bootstrap machinery or a generic
coordination primitive; `OD-LEDGER-001` records that its territory is declared rather than
enforced, which is a bootstrap-shaped compromise. Nothing about it requires the subject to be
software." That record states plainly that naming the three is "not a plan to move them," and
moves nothing. This record settles the one disjunction it left open for the ledger
specifically, against evidence that has accumulated since.

## What Was Measured

**No product surface depends on `nomos-ledger`.** `nomos check` and `nomos gate` — the two
verbs an end-user repository actually runs against its own code — never import
`nomos-ledger` or `nomos-work-orchestration`. Every real dependent is `nomos-work-
orchestration`, consumed by exactly two callers: `nomos-cli::work`
(`crates/host/nomos-cli/src/work.rs`) and `nomos-api::work`
(`crates/host/nomos-api/src/work.rs`) — both dispatch surfaces for this repository's own
board, `work/ledger.json`, never for a subject an end user's repository would configure.

**The product's own agent vocabulary is declared bundling, not an engine the ledger could
converge with.** `nomos-agent-contracts`'s `TaskEnvelope`/`WorkResult`
(`crates/agent/nomos-agent-contracts/src/lib.rs`) compute nothing — a caller fills a
`TaskEnvelope` in and an agent's own response fills a `WorkResult` in. No real `Workflow` type
exists anywhere in this workspace; the nearest is `RunId`
(`crates/contracts/nomos-contracts/src/identity.rs`), an identity with no scheduling,
authority, artifact, or side-effect model attached to it, and a declared-only `WorkflowStep`
contract shape (`OD-WORKFLOW-003`) with no engine behind it. There is no live `Workflow`
machinery for `LedgerItem`/`Territory`/`Claim` to converge toward; converging now would mean
inventing the product's workflow semantics from this coordination tool's own shape, which is
exactly the risk the review named.

**The two vocabularies have already collided once, by accident, and been explicitly
un-conflated twice.** `OD-WORKFLOW-001` corrects "a conflation an earlier draft of this
paragraph made from a substring grep rather than reading the file" — mistaking `nomos-ledger`'s
own `Run_Gate_Step` (an internal argv runner `nomos work finish` uses to run the gate's lint
step) for a call into the product's real `nomos_gate_orchestration::Run_Gate`. `OD-WORKFLOW-
002` names the same correction again, citing "its false `nomos-ledger` caller claim." Two
independent instances of the same name-fragment accident, each caught and reversed rather than
left standing, is evidence the vocabularies need to stay legibly apart, not evidence they are
converging on their own.

**`nomos-api::work.rs` already exposes the full verb set, but frames itself as a seam
exercise, not a product surface.** Its own module doc: "A second real caller of
`nomos_work_orchestration::Run` — `List`, `Show`, `Validate`, `Audit`, `Claim`, `Renew`,
`TakeOver`, `Abandon`, `Decline`, `Finish` and `Add`, the same 'one verb at a time, not the
whole command set' scope `Handle_Gate_Run` already uses for Gate's own `run`." It states its
own purpose as proving the orchestration seam has a second real caller, the identical
framing `nomos-api`'s `check`/`gate`/`spec` modules already carry for their own verbs — not a
claim that ledger coordination is itself a capability an end user's repository would want.

**README's own wording already reads as internal coordination, not a product capability.**
Row 20 (`nomos-ledger`): "Territory-based mutual exclusion over `work/ledger.json`." Row 40
(`nomos-work-orchestration`): runs "a `nomos work` verb against a caller-chosen platform."
Contrast row 41, `nomos-gate-orchestration`: "The seam for the first-class Gate object
`ARC-ROADMAP-001` names" — README already marks Gate as a product object in its own words, and
never uses that language for the ledger.

## Decision

**The work ledger is repository bootstrap machinery.** `nomos-ledger` and `nomos-work-
orchestration` exist to coordinate concurrent work on this repository's own tree —
`LedgerItem`, `Territory`, `Claim`/`Renew`/`TakeOver`, `VerificationPredicate` are shaped
around that one job and answer to no product surface. They are not a Nomos product feature,
and do not converge toward `TaskEnvelope`/`Workflow`/`AgentExecutor`. `ARC-ECOSYSTEM-001`'s
open disjunction — bootstrap machinery or generic XVPE-shaped coordination primitive — is
settled toward the first: nothing measured above requires the subject under coordination to be
software, but nothing measured shows a real, exercised generic-primitive shape either; every
real consumer coordinates work on *this* repository specifically, through `work/ledger.json`
specifically, which is what "bootstrap machinery" names.

This settles ownership, not disposal. `OD-LEDGER-001`'s own open question — whether territory
should stay declared-but-unenforced — is untouched by this record; that is a question about
the mechanism's own correctness, not about what the mechanism is for.

## What This Record Does Not Do

It does not move any file, rename any crate, or change one line of `nomos-ledger`'s or
`nomos-work-orchestration`'s code. `ARC-ECOSYSTEM-001` already declined to treat a subsystem's
location as proof of its ownership; this record answers the ownership question that record
left open without reopening the location question it explicitly declined to answer.

It does not forbid `nomos-api::work` from existing or from growing more verbs. It names the
condition under which this record's own answer should be revisited, below.

It does not decide whether the ledger's underlying primitive — territory-based mutual
exclusion over a committed document — would also be a reasonable generic XVPE capability
someday. It decides only that nothing in this workspace exercises it as one today, which is
the only question `ARC-ECOSYSTEM-001` asked this record to settle.

## What Would Decide It Otherwise

Named so a reader has a concrete trigger rather than a standing suspicion: if `nomos-api`
becomes a real, externally-consumed product surface — used by something other than this
repository's own tooling and this record's own seam-exercise framing — its exposure of the
full ledger verb set at band 90 stops being an internal coordination detail and starts being a
public commitment. That is the point to re-examine this record's answer, not before it.

## Status

Accepted. Settles `ARC-ECOSYSTEM-001`'s open disjunction for the work ledger specifically —
bootstrap machinery, not a generic coordination primitive with a real exercised second
subject, and not a Nomos product feature converging toward `Workflow`/`TaskEnvelope`. Revisit
if `nomos-api` becomes a real external product surface rather than a seam-exercise caller.
