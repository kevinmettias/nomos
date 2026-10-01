---
id: OD-WORKFLOW-003
type: decision
title: WorkflowStep is admitted to band 0 as WF-008's declared contract, not the engine OD-WORKFLOW-002 declined
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - workflow
  - contracts
  - band-zero
  - roadmap
relations:
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-PACKAGE-010
    type: relates-to
---

# WorkflowStep is admitted to band 0 as WF-008's declared contract, not the engine OD-WORKFLOW-002 declined

## Question

`OD-WORKFLOW-002` re-surveyed the workflow tier and found no genuine second increment: `RunId`
still has one real consumer, and `WF-008` through `WF-012` remain exactly as unbuilt as
`OD-WORKFLOW-001` found them. Its own "What This Does Not Do" section states, without
qualification: "It does not build `WorkflowStep`, any part of the `WF-009`..`012` engine ...
manufacturing that consumer now would repeat the ... mistake this workspace has already
declined to make elsewhere."

That sentence answers one question -- does the workflow *execution engine* have a next real
increment -- and the answer stays no; nothing here reopens it. It does not answer a separate
question `OD-WORKFLOW-002` never examined: `OD-CONTRACTS-001` admits a type to band 0 when it
"crosses a subsystem, process or plugin boundary and the parties on both sides need one stable
shared representation of it," a test independent of whether a runtime consumer exists inside
this workspace today -- `RunId` itself sat in `identity.rs`, band 0, with zero real consumers
("nothing constructs one") until `OD-WORKFLOW-001` gave it one. Does `WorkflowStep`, `WF-008`'s
own named contract, independently qualify under that criterion, distinct from whether the
engine that would run one exists yet?

The user was asked directly, given a partially-written `WorkflowStep` implementation already
sitting uncommitted in this tree with no ledger claim behind it -- written after
`OD-WORKFLOW-002` was already committed, and not matching any of the three triggers that record
itself named for a next increment -- whether to discard it, gate it properly, or set it aside.
They chose to gate it properly rather than build blind on their say-so alone or discard
disciplined work unread. This record is that gate.

## What Was Measured

Read directly from `NOMOS_V14_CORPUS`'s requirements directory rather than from either prior
record's own paraphrase:

- `WF-006`'s full statement: "API-hosted, subscription-agent, human, and recorded-replay
  executors shall use the same task/result protocol." This is the corpus's own framing of a
  step as cross-executor protocol truth, independent of which executor kind is built first --
  the same "would a peer that never compiles this crate be unable to agree with us without it?"
  test `OD-CONTRACTS-001` states, answered by the requirement's own text rather than by this
  record inventing a peer.
- `WF-008`'s full statement is a single flat sentence naming exactly eleven properties: typed
  inputs/outputs, side effects, idempotency, retry policy, timeout, cacheability/cache key
  inputs, privileges, cancellation behavior, compensation/rollback, determinism class, and
  evidence emitted. No sub-structure beyond that sentence is stated anywhere in `WF-008` itself
  -- the same "one bounded, statable shape" character `OD-PACKAGE-010` found in
  `MODEL-ROUTE-037`'s opening clause and built from, while declining `038`-`049` as invention
  with no second real case.
- `WF-009` (artifact immutability and versioned mutation), `WF-010` (engine branch/merge,
  bounded parallelism, daemon-restart recovery, partial-failure states) and `WF-011` (workflow
  definitions versioned independently from runs, historical replay pinning) are, read again in
  full, statements about running and publishing workflows -- engine behavior with no data
  contract a step's own declaration could carry. They remain exactly the unbuilt subsystem
  `OD-WORKFLOW-001` and `OD-WORKFLOW-002` both found, and nothing here builds any part of them.
- `WF-012`'s full statement has two clauses: "Retries shall not repeat non-idempotent effects
  without a compensation or deduplication token" (a static coherence rule over a step's own
  declared fields -- checkable without an engine, the same shape
  `crate::determinism::Declaration_Is_Coherent` already checks for a strategy declaration) and
  "Cancellation shall report which side effects completed and what remains reversible" (a
  runtime reporting obligation an engine discharges over a real execution, not something a
  step's own contract can state in advance).
- The uncommitted draft already in this tree
  (`crates/contracts/nomos-contracts/src/workflow_step*`) was read in full against this split.
  Its eleven `WorkflowStep` fields map one-to-one onto `WF-008`'s eleven named properties,
  reusing `crate::determinism::Strategy`'s existing three-field split
  (`determinism_strength`/`reproducibility_scope`/`trace_equivalence`) for "determinism class"
  and `EvidenceClass` for "evidence emitted" rather than inventing parallel vocabulary. Its
  `Is_Coherent` method implements only `WF-012`'s first, static clause;
  `CancellationBehavior`'s own doc comment explicitly declines the second, reporting clause as
  the engine's job "over a real run this workspace does not have yet."
  `Cacheability::key_inputs` and `WorkflowStep::privileges` both stay raw, unresolved strings
  rather than a typed taxonomy -- the same "first maturity, unresolved sub-shape" discipline
  `OD-PACKAGE-010` used for `ModelSelection::Catalog`'s entries -- and `Compensation`
  deliberately does not name which step provides external compensation, deferring that to
  `WF-011`'s definition-graph concern by name in its own doc comment. No field, method or test
  in the draft reaches into `WF-009`, `WF-010`, `WF-011`, or `WF-012`'s second clause.
- Checked directly: nothing in this crate or workspace today constructs a `WorkflowStep`, the
  same population of zero `RunId` had before `OD-WORKFLOW-001`. `OD-CONTRACTS-001`'s criterion
  does not require a workspace-internal consumer -- it requires that a peer on the other side of
  a boundary would need to agree on the shape, which `WF-006` states as a corpus-level
  requirement independent of this workspace's own build order.

## The Decision

**`WorkflowStep`, scoped to exactly `WF-008`'s eleven named properties plus `WF-012`'s first
(static coherence) clause, is admitted to `nomos-contracts` at band 0.** It is vocabulary --
what a step's own declaration promises -- not the engine. `OD-WORKFLOW-002`'s "What This Does
Not Do" clause is narrowed, not reversed: it correctly found no next increment for the workflow
*execution engine*, and that finding stands unchanged. It did not separately examine whether
`WorkflowStep`'s own declared shape independently satisfies `OD-CONTRACTS-001`'s band-0
criterion, and this record answers that narrower question on its own terms.

The already-written draft is accepted as this decision's implementation, checked field by field
against `WF-008` and `WF-012` above rather than re-derived from scratch, because it already
carries the discipline this record's own analysis would have demanded: raw/unresolved
sub-shapes where no closed taxonomy exists yet, explicit doc-comment call-outs of exactly which
corpus clause each type does and does not cover, and reuse of existing crate vocabulary
(`Strategy`, `EvidenceClass`, `Declaration_Is_Coherent`) over inventing parallel forms.

## What This Does Not Do

It does not build `WF-009`, `WF-010`, `WF-011`, or `WF-012`'s second (reporting) clause. It does
not build the `WF-009`..`012` execution engine, a phase concept for `Gate`, or a `nomos-workflow`
crate -- `OD-WORKFLOW-002`'s finding that no such increment exists stands. It does not give
`WorkflowStep` a real constructor, publisher, or consumer anywhere in this workspace; nothing
here wires a `WorkflowStep` value into `Gate`, `nomos-cli`, or `nomos-api`, and doing so would be
the same "manufacture a consumer" mistake `OD-WORKFLOW-002` already declined. It does not
resolve `Cacheability::key_inputs` or `WorkflowStep::privileges` into a typed taxonomy -- both
stay raw strings until a second real case exists to check field boundaries against, the same
wait `OD-PACKAGE-008` held for `RulePackage` and `OD-PACKAGE-010` held for
`ModelSelection::Catalog`'s entries. It does not reopen `OD-WORKFLOW-001` or the rest of
`OD-WORKFLOW-002`'s survey.

## Status

Accepted. `WorkflowStep` is admitted to band 0 as `WF-008`'s declared contract, under the
`OD-CONTRACTS-001` criterion applied on its own terms rather than the "does a runtime consumer
exist" test `OD-WORKFLOW-001`/`OD-WORKFLOW-002` correctly applied to the engine. `OD-WORKFLOW-002`
is amended to narrow its "does not build `WorkflowStep`" clause to the engine question it was
actually answering; its survey of the execution-engine tier is otherwise unchanged and this
record does not reopen it.
