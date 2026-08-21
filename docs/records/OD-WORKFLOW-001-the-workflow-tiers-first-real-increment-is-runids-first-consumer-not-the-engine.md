---
id: OD-WORKFLOW-001
type: decision
title: The workflow tier's first real increment is RunId's first real consumer, not the engine
status: accepted
version: 2
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

## Amendment: A Bare Clock Reading Cannot Give RunId Real Per-Execution Uniqueness

The "Concretely, for the follow-up" paragraph above was written before checking the one type it
depends on. `nomos_platform::Timestamp`'s own module doc
(`crates/platform/nomos-platform/src/clock/timestamp.rs`) is explicit: "Second resolution is
deliberate ... Where ordering matters more finely than a second, the ordering is carried
explicitly rather than inferred from a clock." A `RunId` computed purely from `clock.Now()`
therefore collides for any two runs that start within the same wall-clock second -- not a rare
case. `nomos-api`'s own test suite calls `Handle_Gate_Run` twice inside one process, well under a
second apart, and `.github/workflows/gate.yml` runs several gate-adjacent steps in quick
succession. That paragraph's plan is withdrawn: it would have shipped a `RunId` that fails this
record's own stated purpose -- distinguishing two real executions -- the first time two runs
happened to land in the same second, which is routine rather than rare.

Grepped directly: no `Random` or nonce port exists anywhere in `nomos-platform`, and the
workspace's root `Cargo.toml` carries no `rand` or `uuid` dependency. Adding one is its own
decision -- a new external dependency, subject to `cargo-deny`'s advisories/bans/licenses/sources
check -- and out of scope for a record naming a first increment rather than building it.

**The corrected decision: `Run_Gate` does not compute a `RunId` at all.** It accepts one as a
plain caller-supplied value -- a `run: RunId` parameter, the same composition-root-supplied shape
`variant` and `launcher` already are -- and `nomos-gate-orchestration` gains no `Clock`
dependency and no generic parameter for one. Each composition root constructs its own `RunId`
however it can. The honest construction for a first increment -- named as what it is rather than
implied to be more -- is a new `nomos_contracts::RunId::Fresh(now: Timestamp) -> RunId`,
alongside `Digest_Identity!`'s existing `From_Digest`/`Digest`, combining the clock reading,
`std::process::id()` (already used this way for scratch-path uniqueness in this workspace's own
tests, e.g. `crates/host/nomos-cli/tests/check_command.rs`), and a process-local monotonic
counter: collision-free within one process, reduced but not eliminated in probability across
processes that start in the same second. Naming that limitation plainly is the same honesty this
workspace already holds `Applicability::PartiallySupported` to, rather than a caveat this record
would rather not state.

## Status

Accepted, amended. Names the workflow tier's first real increment -- giving `RunId` its first
real consumer through Gate -- and, after the amendment above, the corrected shape a follow-up
implementation needs: `Run_Gate` takes a caller-supplied `RunId`, not a `Clock`, and
`RunId::Fresh` is the first-increment construction each composition root can call. Still builds
neither the increment nor the engine `WF-*` describes beyond it.
