---
id: OD-WORKFLOW-001
type: decision
title: The workflow tier's first real increment is RunId's first real consumer, not the engine
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - workflow
  - identity
  - gate
  - roadmap
relations:
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-CONTRACTS-001
    type: relates-to
  - target: OD-GATE-014
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
---

# The workflow tier's first real increment is RunId's first real consumer, not the engine

## Question

`ARC-ROADMAP-001` names "headless workflow orchestration" as a near-term-tier item, and the
user's standing roadmap override (recorded after `P13-GATE-API-ADAPTER-FIRST-INCREMENT`) names
it as unblocked work, not still waiting for a trigger. Nothing in this workspace names what its
first real increment actually is. Building the wrong first piece is expensive here specifically:
this repository has repeatedly declined to design a seam ahead of a second real case
(`OD-PACKAGE-006`, `OD-PACKAGE-008`, `nomos-gate-orchestration`'s own doc on why `compare` is not
stubbed), and "workflow orchestration" has zero real code and zero real consumers to check a
design against today.

## What Was Measured

The v14 corpus's own `WF-*` and `WF-ORDER-*` requirement families, read directly from
`NOMOS_V14_CORPUS`'s requirements directory (`WF-001` through `WF-012`, `WF-ORDER-001` through
`WF-ORDER-005`) rather than summarized from `ARC-ROADMAP-001`'s own reading of them. They
describe a real execution engine: deterministic branch/merge semantics and bounded parallelism
(`WF-010`), retries that must not repeat non-idempotent effects without a compensation or
deduplication token (`WF-012`), workflow definitions versioned independently from runs with
historical replay pinning the exact definition, packages, providers, environment, inputs and
approval decisions (`WF-011`), immutable published artifacts (`WF-009`), and a
`WorkflowStep` contract (`WF-008`) declaring eleven distinct properties -- typed inputs/outputs,
side effects, idempotency, retry policy, timeout, cacheability/cache key inputs, privileges,
cancellation behavior, compensation/rollback, determinism class, and evidence emitted. Every one
of those is a real subsystem or a multi-field contract with no existing implementation anywhere
in this workspace to check a shape against -- the same absence `OD-PACKAGE-008` measured for
`RulePackage`'s manifest fields, at a larger scale.

Separately, `crates/contracts/nomos-contracts/src/identity.rs` already declares `RunId`
("Identity of one execution of a workflow or gate") -- present since this crate's very first
commit (`205e66c`, "Add nomos-contracts, the band-0 protocol vocabulary"), grepped directly: its
only reference anywhere in the workspace outside `identity.rs` itself is its re-export in
`nomos-contracts`' own `lib.rs`. Nothing constructs one. `nomos_gate_orchestration::
GateRunResult` -- this workspace's one real, shipped "one execution" today -- carries no run
identity of its own, the same "declared and deliberately unconsumed" state `OD-HOST-002` already
names for `PackageKind::ModelBackendPackage` and `PackageKind::AgentExecutorPackage`.

## The Decision

**The workflow tier's first real increment is not `WorkflowStep`, not the engine `WF-009`
through `WF-012` describe, and not a new crate. It is giving `RunId` -- vocabulary this
workspace already committed to, naming exactly the "workflow or gate" split this tier needs --
its first real consumer, through the one real "one execution" this workspace has: a Gate run.**

This is deliberately the narrowest defensible piece. It invents no new field, no new type and no
new crate: `RunId`'s shape (a `Digest128` newtype, `Digest_Identity!`'s standard pattern) is
already fixed by code committed under `OD-CONTRACTS-001`'s band-0 criterion, which this record
does not reopen. What it does decide, because nothing has decided it yet and a follow-up
implementation cannot proceed without an answer: **a `RunId` identifies one execution, not one
configuration, and is therefore not a pure content digest of `GateCommand` and the build
variant** -- two runs of an identical command are two different executions, the same distinction
`nomos-platform::Clock`'s own module doc draws between a value that is a dependency and one that
is an ambient fact. `WF-005`'s own text keeps "cache hits" as a concept distinct from a run's own
identity ("stream progress and retain execution DAG, cache hits, retries, artifacts, costs, and
exact tool provenance") -- the run is the container the cache hits are recorded *against*, not
the same digest as a cache key. A `RunId` that collapsed into a content digest of the command
would make two genuinely different runs of the same command indistinguishable, which is the same
failure mode `identity.rs`'s own module doc calls out for path-and-line identity: a real
distinction with no key to carry it.

Concretely, for the follow-up this record unblocks: `nomos_gate_orchestration::Run_Gate` gains a
`nomos_platform::Clock` parameter alongside its existing `ProcessLauncher` one -- a composition-
root-supplied dependency, the same shape `variant` and `launcher` already are, not a
`SystemTime::now()` read buried in the crate -- and `GateRunResult` gains a `pub run: RunId`
field, computed once per call from the clock reading (and nothing else content-addressed,
because content-addressing it would silently re-introduce the collapse this record just ruled
out). Every existing caller of `Run_Gate` (`nomos-cli`'s `gate.rs`, `nomos-ledger`'s
`finish/gate_step.rs`, `nomos-api`'s `Handle_Gate_Run`) supplies a real clock the same way each
already supplies a real build variant and a real process launcher.

## What This Does Not Do

It does not build `WorkflowStep`, any part of the execution engine `WF-009` through `WF-012`
describe, or a `nomos-workflow` crate. Those stay exactly as unbuilt as `ARC-ROADMAP-001` already
found them, and this record schedules none of them -- the same "does not schedule work" limit
`ARC-ROADMAP-001` itself states for its own boundary.

It does not implement the `Run_Gate`/`GateRunResult` change the previous section names. That is
real code touching a shipped, multiply-depended-on seam (three callers today), and belongs in
its own claimed item with its own verification, the same split `OD-GATE-014`'s override record
and `P13-GATE-014-SCOPE-RULE-SELECTORS` already used between naming a decision and building it.

It does not reopen `OD-CONTRACTS-001`'s admission criterion, or `RunId`'s own shape. `RunId`
already satisfies that criterion today, on the same reasoning `OD-CONTRACTS-001` gives for
`GateCategory` and `EvidenceClass`: a peer executor -- `WF-006`'s "API-hosted, subscription-agent,
human, and recorded-replay executors" -- cannot agree with Nomos about which execution a result
belongs to without a shared identity for it, so this stays band 0 rather than moving to
whichever crate builds its first consumer.

It does not commit to `EGRAPH` or any relationship-graph construction. `ARC-ROADMAP-001`
constraint 3 already reserves that question for once workflow orchestration or agent execution
reach the point of needing typed cross-entity relationships; giving `GateRunResult` an identity
of its own is not that point.

## Status

Accepted. Names the workflow tier's first real increment and the one open design question a
follow-up implementation needs answered -- content-addressed versus per-execution identity --
without building either the increment or the engine `WF-*` describes beyond it.
