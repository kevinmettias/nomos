---
id: OD-WORKFLOW-002
type: decision
title: The workflow tier's second increment has not arrived since RunId's first consumer shipped
status: accepted
version: 4
authority: canonical-normative-record
tags:
  - workflow
  - identity
  - gate
  - roadmap
relations:
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: ARC-ROADMAP-001
    type: relates-to
  - target: OD-CORRECTIONS-001
    type: relates-to
  - target: OD-WORKFLOW-003
    type: relates-to
  - target: OD-WORKFLOW-004
    type: relates-to
  - target: OD-WORKFLOW-005
    type: relates-to
---

# The workflow tier's second increment has not arrived since RunId's first consumer shipped

## Question

`OD-WORKFLOW-001` (v3, accepted) named the workflow tier's first real increment narrowly:
give `RunId` its first real consumer through `Gate`, and build nothing else --
`WorkflowStep` and the `WF-009`..`012` engine stay unbuilt "as a real subsystem ... with no
existing implementation to check a shape against." `P13-WORKFLOW-001-RUNID-FRESH-AND-RUN-GATE`
shipped that increment (`Fresh_Run_Id`, `Run_Gate`'s `run: RunId` parameter,
`GateRunResult::run`, threaded through both real callers). Nothing has since named what the
tier's *second* increment is. Building the wrong next piece here is the same expensive
mistake `OD-WORKFLOW-001` corrected twice over for its own first one (its withdrawn
clock-only `RunId` plan, and its false `nomos-ledger` caller claim), so this record checks
the real code and the real corpus again before naming anything, rather than assuming the
question answers itself just because time passed.

## What Was Measured

Read directly from the live tree, not from `OD-WORKFLOW-001`'s own description of it:

- `git log` confirms `P13-WORKFLOW-001-RUNID-FRESH-AND-RUN-GATE` (`3a155fb`) is the tip of
  what this tier has built. No `OD-WORKFLOW-002` existed in `docs/records/` or
  `work/ledger.json` before this one, and `nomos work list` reports `next: nothing is
  eligible` -- every non-`done` item on the board is `declined`, including
  `P13-WORKFLOW-001-RUNID-FIRST-INCREMENT` itself (superseded by its own `-2`, which is
  `Done`). No live session has claimed or proposed a second workflow-tier increment.
- Grepped directly: `nomos_gate_orchestration::Run_Gate` still has exactly two real callers
  in this workspace -- `nomos-cli`'s `gate.rs` and `nomos-api`'s `Handle_Gate_Run` -- the
  same two `OD-WORKFLOW-001` named. `nomos-ledger`'s `Run_Gate_Step`
  (`crates/substrate/nomos-ledger/src/finish/gate_step.rs`) remains the distinct, unrelated
  function `OD-WORKFLOW-001`'s own amendment already corrected an earlier draft for
  conflating; read in full, it runs the workflow's own lint-step argv through a bare
  `ProcessLauncher` with no persisted or correlated executions anywhere to give an identity
  meaning against. Giving it a `RunId` now would manufacture a second consumer rather than
  find a real one.
- `nomos-cli`'s `gate/report.rs` still does not print `result.run` -- the same boundary
  `P13-WORKFLOW-001-RUNID-FRESH-AND-RUN-GATE`'s own `done_when` drew on purpose ("this
  increment does not add `RunId` to the CLI's printed output, only to the typed result").
  That is a real, human-visible gap, but it is mechanical -- one `writeln!` against an
  already-computed field, with no design question to settle -- not the kind of decision
  `OD-WORKFLOW-001` and `OD-PACKAGE-010`'s record pattern exists for. Naming it as a
  "decision" here would misuse that pattern rather than follow it.
- The v14 corpus's `WF-*`/`WF-ORDER-*` requirement families were re-read in full, file by
  file, from `NOMOS_V14_CORPUS`'s requirements directory (`WF-001` through `WF-012`, absent
  at `WF-007`, and `WF-ORDER-001` through `WF-ORDER-005`) rather than trusted from
  `OD-WORKFLOW-001`'s own summary of them. `WF-008`..`WF-012` (the `WorkflowStep` contract,
  immutable published artifacts, deterministic branch/merge and bounded parallelism,
  independently versioned definitions with pinned historical replay, retry/compensation)
  remain exactly the unbuilt subsystem `OD-WORKFLOW-001` already found; nothing new checks
  their shape. `WF-ORDER-001`..`005` describe a gate's *own* remediation-phase ordering --
  file-scoped phases completing before tree-scoped ones, a validator rejecting mis-ordered or
  unsatisfiable `needs` dependencies, a harness that may coalesce consecutive file-scoped
  phases into a per-file band. Grepped directly for a word-boundary match on `phase`/`Phase`
  across `crates/orchestration/nomos-gate-orchestration/src` and
  `crates/orchestration/nomos-check-orchestration/src`: no match in either. This workspace's
  one real `Gate` still judges a tree in a single pass with no phase concept at all, so
  `WF-ORDER-*` has zero real structure anywhere to check a shape against -- an earlier-stage
  absence than `WF-008`'s own, not a later one a second increment could plausibly reach.
- `ARC-ROADMAP-001`'s near-term tier and its four constraints were re-read in full and are
  unchanged; nothing in it names workflow orchestration as ready to advance past
  `OD-WORKFLOW-001`. The adjacent near-term-tier work that has landed since --
  `P13-MODEL-PACKAGE-FIRST-INCREMENT` (`OD-PACKAGE-010`, Track C's lane) and four new
  `nomos-api` seams (`P13-API-WORK-LIST-SEAM` through `P13-API-WORK-VALIDATE-SEAM`, Track A's
  lane) -- touches model/agent package infrastructure and the CLI/API/MCP
  application-service boundary respectively, and neither constructs, consumes, or references
  `RunId`, `WorkflowStep`, or anything in the `WF-*` family: checked directly by grep and by
  reading `nomos-api`'s `response.rs` and `lib.rs` in full, `RunId`'s only appearance
  anywhere in `nomos-api` is still `Handle_Gate_Run`'s existing `GateRunResponse::run` field
  from the first increment. `OD-CORRECTIONS-001` -- the same survey discipline applied to
  the corrections tier, in the same window -- likewise found zero real callers for its own
  subject and named the trigger that would produce a real increment rather than inventing
  one; this record follows the same shape for workflow.

## The Finding

**Nothing has changed.** `RunId` still has exactly one real consumer (`Gate`, through
`Run_Gate` and its two callers), the `WF-009`..`012` engine and the `WorkflowStep` contract
remain exactly as unbuilt as `OD-WORKFLOW-001` found them, and the `WF-ORDER-*` family has no
phase concept anywhere in this workspace to check a shape against -- an absence one level
more basic than `WF-008`'s own. There is no genuine second increment to name here yet. The
one real, mechanical gap this survey found -- the CLI does not print `RunId` -- is not a
second increment; it is a print statement against an already-decided field, deliberately
left out of the first increment's scope for narrowness rather than as an oversight, and
recording it as a "decision" would misuse the pattern this tier's own records exist for.

## What This Does Not Do

It does not build `WorkflowStep`, any part of the `WF-009`..`012` engine, a phase concept
for `Gate`, or a `nomos-workflow` crate. It does not give `RunId` a second consumer inside
`nomos-ledger`'s `Run_Gate_Step`, which has no persisted or correlated executions to make an
identity meaningful against -- manufacturing that consumer now would repeat the "no invented
shape ahead of a real case" mistake this workspace has already declined to make elsewhere
(`OD-PACKAGE-006`, `OD-PACKAGE-008`, `OD-CORRECTIONS-001`). It does not schedule the CLI's
`RunId` print line as a ledger item needing its own decision; that is real, narrow,
uncontroversial work a later item can simply do, with no record required to justify it. It
does not reopen `ARC-ROADMAP-001`, `OD-CORRECTIONS-001`, or `OD-WORKFLOW-001` itself.

## What Would Decide The Next Increment

Any of:

- **A second real caller of `Run_Gate` appears**, distinct from `nomos-cli` and `nomos-api`,
  giving `RunId` a genuine second consumption pattern to check a broader identity contract
  against -- persistence, comparison, or lookup by id.
- **`Gate` grows a real phase concept** -- more than one ordered stage of judgment inside one
  run -- giving `WF-ORDER-*` real structure to check file-scoped-before-tree-scoped ordering
  against, rather than a single-pass walk with nothing to order.
- **`ModelBackend`/`AgentExecutor` infrastructure reaches a real executor**
  (`ARC-ROADMAP-001`'s own near-term item, distinct from this tier), giving `WF-006`'s
  "API-hosted, subscription-agent, human, and recorded-replay executors" a first real
  instance to check a shared task/result protocol against.

Until one of those arrives, the workflow tier stands exactly where `OD-WORKFLOW-001` left
it: one real consumer, one real "one execution," and an engine with nothing yet to check its
shape against.

## Amendment (OD-WORKFLOW-003)

The "What This Does Not Do" clause above -- "It does not build `WorkflowStep`, any part of the
`WF-009`..`012` engine ..." -- was written answering one question: whether the workflow
*execution engine* has a next real increment. It did not separately examine whether
`WorkflowStep`'s own declared shape (`WF-008`'s eleven named properties) independently
qualifies for band-0 admission under `OD-CONTRACTS-001`'s cross-boundary-vocabulary criterion,
a test that does not require a workspace-internal runtime consumer -- the same test `RunId`
itself passed with zero consumers before this tier's own first increment gave it one.
`OD-WORKFLOW-003` answers that narrower question and admits `WorkflowStep`, scoped to `WF-008`
plus `WF-012`'s first (static coherence) clause only, as vocabulary. That clause is narrowed
accordingly: it continues to mean no execution engine, no `WF-009`/`010`/`011`, and no real
`WorkflowStep` consumer anywhere in this workspace. Every other finding in this record --
`RunId`'s one real consumer, the `WF-ORDER-*` phase absence, the three named triggers for a
next engine increment -- is unchanged.

## Amendment (OD-WORKFLOW-004)

Condition 3 above -- "`ModelBackend`/`AgentExecutor` infrastructure reaches a real executor"
-- was written before any real executor existed, and read literally it is satisfied by any
single tool-dispatching executor regardless of whether it ever produces a result.
`nomos-agent-executor`'s arrival exercised that gap directly: `OD-WORKFLOW-004` found a real
executor now exists but constructs no real `WorkResult`, so `WF-006`'s task/result protocol
has a real instance only on its task side, and none of the other three named executor kinds
has any trace anywhere in this workspace. Condition 3 is narrowed accordingly: it now reads
as satisfied only when a real executor also constructs a real `WorkResult`, not merely when
one exists. Conditions 1 and 2, and every other finding in this record, are unchanged --
`OD-WORKFLOW-004` re-verified both directly and found neither has moved.

## Amendment (OD-WORKFLOW-005)

None of the three conditions above has fired -- `OD-WORKFLOW-005` re-checked all three
directly against the tree that shipped `nomos-agent-executor-ollama` and found condition 3
still satisfied only at the letter, conditions 1 and 2 unchanged. A real execution increment
exists in this workspace anyway: `nomos-workflow-orchestration`, built under the user's own
direct, session-specific instruction to this workspace's workflow tier rather than under any
condition this record names. This record's own three conditions are not superseded and are
not retired -- they remain the honest triggers for the increment *after* this one, the same
distinction `OD-WORKFLOW-005` itself draws between a narrow, explicit override and a general
retirement of the caution this record's survey discipline still holds.

## Status

Accepted. Re-surveys the workflow tier against the live tree and the v14 corpus a second
time, after `OD-WORKFLOW-001`'s first increment shipped, and finds no genuine second
increment has arrived -- naming the three conditions that would produce one rather than
inventing a shape to have something to build. Amended by `OD-WORKFLOW-003` to narrow this
record's "does not build `WorkflowStep`" clause to the execution-engine question it was
actually answering. Amended again by `OD-WORKFLOW-004` to narrow condition 3's wording after
`nomos-agent-executor`'s arrival satisfied its letter without satisfying its substance.
Amended a third time by `OD-WORKFLOW-005` to record that a real execution increment was built
under the user's own override before any of the three conditions fired; the rest of this
record's survey stands.
