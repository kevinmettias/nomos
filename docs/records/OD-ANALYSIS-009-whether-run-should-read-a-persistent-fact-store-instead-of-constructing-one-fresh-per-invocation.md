---
id: OD-ANALYSIS-009
type: decision
title: Whether nomos-check-orchestration's Run should read a persistent fact store instead of constructing MemoryFactStore fresh per invocation
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - analysis
  - incrementality
  - check-orchestration
  - architecture
relations:
  - target: OD-ANALYSIS-001
    type: relates-to
  - target: OD-ANALYSIS-002
    type: relates-to
  - target: OD-ANALYSIS-008
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-HOST-002
    type: relates-to
  - target: OD-HOST-003
    type: relates-to
  - target: OD-RULES-009
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
---

# Whether nomos-check-orchestration's Run should read a persistent fact store instead of constructing MemoryFactStore fresh per invocation

## Question

An external architecture review (read against this workspace rather than accepted on read)
names `nomos-check-orchestration::run::Run`'s `let mut store = MemoryFactStore::New();` as
evidence that `nomos-analysis`'s incremental substrate — generations, invalidation,
dependency-aware recomputation — is architecturally proven but never operationalized:
despite that machinery being real, "the normal user-facing check path still creates a fresh
fact store for every invocation." It names a persistent incremental daemon or service as one
of the largest remaining gaps between what this workspace's substrate can do and what `nomos
check` actually does today. `ARC-ROADMAP-001` separately lists "analysis + incremental fact
infrastructure" in Nomos Core's near-term tier. Whether either of those is new evidence that
`Run` should stop owning its own store's construction, or a restatement of scope
`ARC-ROADMAP-001` already named without ordering, was unmeasured before this record — the
same question `OD-RULES-009` asked of the same review's `RunPlanner` claim, asked here of its
persistence claim.

## What Was Measured

**The review's claim is accurate.** Verified directly against the real code at this record's
own HEAD: `crates/orchestration/nomos-check-orchestration/src/run.rs:51` constructs
`MemoryFactStore::New()` fresh inside `Run`, on every call, with no parameter through which a
caller could supply one instead.

**The incremental machinery this store would reuse is real, not aspirational.**
`OD-ANALYSIS-001` closed a bug that defeated cross-edit reuse entirely (a workspace-snapshot
component in `FactKey` that recomputed 9,440 facts per keystroke over a six-file corpus);
`OD-ANALYSIS-002` built the dependent half of invalidation; `OD-ANALYSIS-008` (open,
narrowed) tracks one remaining internal cost in that same machinery.
`tests/invalidation_order.rs`, `tests/recomputation_equivalence.rs` and
`tests/fact_identity/*` exercise generation-based reuse and dependency-aware invalidation
directly, and pass.

**But nothing in this workspace has ever asked one `MemoryFactStore` to survive across two
separate calls to `Run`.** A workspace-wide search for `MemoryFactStore::New()` finds
twenty-three call sites: one in production code (`run.rs:51`) and twenty-two in test modules,
each constructing its own store, using it within one test function, and dropping it. None is
held by anything that outlives one call. `nomos-cli`'s two composition roots that reach
`Run` — `check::Run` and `gate::run` — each call it exactly once per process, and
`main.rs` returns one `std::process::ExitCode` per invocation with no loop, no server and no
state that survives past `return`. The "fresh store per invocation" cost the review measures
is therefore not currently a same-process repeated-computation waste; it is the ordinary cost
of a fresh OS process on every shell invocation of `nomos check`, which passing a
caller-supplied store into `Run` would not by itself change — nothing would exist to hold
that store between one shell invocation and the next unless a long-lived process held it, and
no such process, nor any vocabulary for one, exists anywhere in this workspace. A search of
`docs/records` for "daemon", "watch mode" and "persistent workspace" finds nothing proposing
one.

**A persistent store would be licensed, if a real caller ever needed one — but none does
yet.** `OD-HOST-002`'s cache/privileged-state test draws the line a persistent `FactStore`
would have to clear: losing it must cost latency, not information. It would clear that line
(a dropped store is rebuilt from source, not lost data), so nothing forbids building one.
`OD-HOST-003` goes further and already reasons about exactly this shape for a long-lived
editor process, settling that a resolved capability registry or fact set held across such a
process's lifetime is a cache under `OD-HOST-002`'s test, not privileged state — real
precedent for what the first caller of a persistent store would look like, reasoned about
before that caller exists rather than invented to justify building the store first.

**Naming a category as near-term is not the same as ordering this increment inside it.**
`ARC-ROADMAP-001` lists "analysis + incremental fact infrastructure" in Nomos Core's
near-term tier, but its own fourth "What This Record Does Not Do" clause states plainly that
it "does not order the near-term tier internally... a separate judgment, informed by this
record but not fixed by it." `OD-RULES-009` already drew this same distinction against the
same review's `RunPlanner` claim, over the same record; it applies identically here.

## Decision

**Declined to build now.** `Run` keeps constructing `MemoryFactStore::New()` internally; no
persistent or cross-invocation fact store, no caller-supplied-store parameter, and no
daemon or long-lived-process concept is scheduled by this record.

This is not a verdict that persistence is the wrong eventual shape. `ARC-ROADMAP-001` already
names the category as near-term, and `OD-HOST-002`/`OD-HOST-003` already establish that a
persistent store would be licensed as a cache rather than forbidden as privileged state when
one is built. It is a verdict that no caller anywhere in this workspace has yet asked `Run`
to be invoked more than once within one process's lifetime, so there is no real
repeated-computation cost to fix today and no real caller's shape to design a
caller-supplied-store seam from — the same "wait for a real case, not the population of
zero" reasoning this workspace applies elsewhere (`OD-PACKAGE-006`, `OD-PACKAGE-008`,
`OD-RULES-007`, `OD-RULES-008`). `OD-ROADMAP-001`'s override does not reach this question: it
names one explicit cluster — `AgentExecutor`/`ModelBackend`/`RulePackage`/corrections — and
this is not in it.

## What Would Decide It

- **A real long-lived caller.** Most plausibly the editor surface `OD-HOST-003` already
  reasons about, or any IDE/LSP-shaped client, that would invoke `Run` (or something
  functionally like it) more than once within its own process lifetime — the concrete case
  that would turn a per-call `MemoryFactStore::New()` into a measured cost rather than a
  theoretical one.
- **A workflow engine or a Gate phase concept** (`OD-WORKFLOW-002`'s still-unfired third
  trigger) that runs multiple analysis passes within one execution and would benefit from
  sharing a store across them.
- **Measured evidence from a real corpus-scale workload** — an edit-then-recheck run timed
  against a fresh-store baseline — the same evidentiary bar `OD-ANALYSIS-001` and
  `OD-ANALYSIS-008` both hold themselves to rather than arguing from complexity alone.
- **`ARC-ROADMAP-001`'s near-term tier being internally sequenced** by a later decision that
  specifically orders "analysis + incremental fact infrastructure" ahead of the items beside
  it.

## A Direct Override Built The First Real Increment, Narrower Than Any Trigger Above

None of the four triggers named above had fired on their own by 2026-08-26. What changed is
not evidence — no editor surface, no workflow engine, no measured corpus workload, no
internal sequencing of `ARC-ROADMAP-001`'s near-term tier arrived. What changed is that the
user directly and explicitly overrode this record's "wait for a real caller" conclusion,
the same shape `OD-ROADMAP-001` already used for a different cluster of decisions. This is
not `OD-ROADMAP-001` reaching a question its own text already said it does not reach — this
record's own "Decision" section above already measured that and it is still true, unchanged
by this section. It is a second, distinct override, over this specific question, recorded
here rather than folded into `OD-ROADMAP-001`'s text or assumed to already be covered by it.

**What was built** (`P14-ANALYSIS-009-STORE-WORKSPACE-REUSE-FIRST-INCREMENT`):
`nomos_check_orchestration::Run` now takes `workspace: &mut Option<Workspace>` and
`store: &mut MemoryFactStore` as parameters instead of constructing a
`nomos_workspace::Workspace` and a `MemoryFactStore` internally on every call. `Ingested`
(`crates/orchestration/nomos-check-orchestration/src/facts.rs`) reuses an existing
`Workspace` when one is handed in, via `Option::get_or_insert_with`, rather than always
starting from `Workspace::Empty`. Both of this crate's real callers —
`nomos-cli::check::Run` and `nomos-gate-orchestration::run_gate::Judged` — construct a fresh
`Workspace` (`&mut None`) and a fresh `MemoryFactStore` for every call, unchanged: each is
still one process per invocation, exactly the finding "What Was Measured" made above, so
nothing about *their* behavior is different after this increment. What is different is that
`Run` no longer forces every caller to.

A new test,
`nomos_check_orchestration::tests::Test_A_Store_And_Workspace_Reused_Across_An_Edit_Agrees_With_A_Clean_Recomputation`,
is the first caller this workspace has ever had that reuses either object across two calls.
It carries one `Workspace` and one `MemoryFactStore` across a call that edits one source and
leaves another alone, and checks the result against an independent third call that
recomputes the post-edit tree from nothing. Two things are proven, both new:

- The claim and the findings agree between the reused-state call and the from-nothing call —
  `IncrementalResult(S) == CleanRecomputation(S)`, the literal invariant `crates/substrate/
  nomos-analysis/tests/recomputation_equivalence.rs` already proved over synthetic `FactKey`s
  built by hand, exercised here for the first time through this crate's own real composition
  (the real syntax provider, the real `Workspace`, `Run`'s real ingestion and judging) rather
  than bypassing it.
- The reused `Workspace`'s generation genuinely advances across the two calls
  (`Workspace::Apply`'s own diff — an untouched file is `Redundant`, an edited one is
  `Modified` — is what makes the second call a real second generation, not the fresh-store-
  every-time behavior computing the identical generation twice).

**What this does not close.** `Materialize_Syntax` and its sibling materialization steps
still re-derive every source's fact on every call, whether or not `store` already holds a
live one for that identity — this increment proves reuse is *safe*, not that anything is
*skipped*. `IncrementalResult` above costs exactly what `CleanRecomputation` costs. And no
long-lived process exists anywhere in this workspace that could hold a `Workspace` and a
`MemoryFactStore` across two *shell* invocations — `nomos check` and `nomos gate run` remain
one process each, so the first trigger above ("a real long-lived caller") has still not
fired; this increment is what a real caller of that kind would need to exist, not that
caller itself.

## Status

Accepted, amended. The four triggers named above remain unfired on their own evidence.
Revisit on any of them, on `OD-HOST-003`'s editor surface or `OD-WORKFLOW-002`'s engine
trigger next changing status, or on either of this section's own two named remainders: a
materialization step that consults `store` before recomputing a fact, or a real long-lived
process that could be the caller this record's first trigger describes.
