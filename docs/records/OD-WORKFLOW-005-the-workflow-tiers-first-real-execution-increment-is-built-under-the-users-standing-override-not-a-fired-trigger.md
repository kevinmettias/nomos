---
id: OD-WORKFLOW-005
type: decision
title: The workflow tier's first real execution increment is built under the user's standing override, not a fired OD-WORKFLOW-002 trigger
status: accepted
version: 2
authority: canonical-normative-record
tags:
  - workflow
  - agent
  - executor
  - roadmap
relations:
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-WORKFLOW-003
    type: relates-to
  - target: OD-WORKFLOW-004
    type: relates-to
  - target: OD-ROADMAP-001
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
  - target: OD-EXECUTOR-004
    type: relates-to
  - target: OD-EXECUTOR-003
    type: relates-to
  - target: OD-PACKAGE-016
    type: relates-to
---

# The workflow tier's first real execution increment is built under the user's standing override, not a fired OD-WORKFLOW-002 trigger

## Question

`OD-WORKFLOW-002` (v3, amended twice) named three conditions that would decide the workflow
tier's next real increment, and re-checked them twice more since: `OD-WORKFLOW-003` narrowed
its `WorkflowStep` clause to the engine question it was actually answering, and
`OD-WORKFLOW-004` found the third condition satisfied only at the letter of its wording, not
its substance — a real `AgentExecutor` exists but constructs no real `WorkResult`. Re-checked
directly once more for this record: `nomos-agent-executor-ollama` now exists as a second real
`AgentExecutor` (this workspace's own `git log` shows `b5566ae`, "Merge track-b-agent-model:
second AgentExecutor backend (Ollama)"), but it constructs no `WorkResult` either — grepped
across `crates` for every real `WorkResult` construction site, the only two remain
`nomos-agent-contracts`' own `#[cfg(test)]` module, exactly where `OD-WORKFLOW-004` already
found them. **None of the three conditions has fired.** `nomos_gate_orchestration::Run_Gate`
still has exactly the same two real callers `OD-WORKFLOW-002` found; a word-boundary grep for
`phase`/`Phase` across `crates/orchestration/nomos-gate-orchestration/src` and
`crates/orchestration/nomos-check-orchestration/src` still returns nothing.

This record exists anyway, because the user who owns this repository directed this session
directly to build the workflow tier's first real execution increment now, in full, explicit
awareness that no condition has organically fired — the same shape this workspace has already
recorded for other subsystems. `OD-PACKAGE-010` was "built only after the user overrode it
once, explicitly and narrowly," before `OD-ROADMAP-001` later generalized that override across
the `AgentExecutor`/`ModelBackend`/`RulePackage`/corrections cluster. `OD-ROADMAP-001` does not
name the workflow tier or this record's own cluster — grepped directly, no record under
`docs/records/` cites `OD-ROADMAP-001` as authority for anything workflow-tier-shaped — so this
record does not claim that generalized retirement already covers this decision. It follows
the same *pattern* `OD-PACKAGE-010` walked before `OD-ROADMAP-001` existed: a narrow, explicit,
one-session override, recorded honestly as exactly that, not inflated into a claim that a
wider retirement already licensed it.

## What Was Measured

Read directly from the live tree rather than assumed from any prior record's own summary of
it:

- `crates/contracts/nomos-contracts/src/workflow_step.rs` and its four sibling files
  (`cacheability.rs`, `cancellation_behavior.rs`, `compensation.rs`, `retry_policy.rs`,
  `timeout.rs`) declare `WorkflowStep` exactly as `OD-WORKFLOW-003` admitted it: `WF-008`'s
  eleven properties plus `WF-012`'s first, static-coherence clause only (`Is_Coherent`).
  Grepped across the whole workspace for `Is_Coherent`: the only call sites are
  `workflow_step.rs`'s own `#[cfg(test)]` module. Nothing outside this crate's own tests has
  ever called it — the identical "vocabulary with zero real consumers" state `RunId` was in
  before `OD-WORKFLOW-001` gave it one through `Gate`.
- Two real `AgentExecutor` crates exist, `nomos-agent-executor-claude-code` (band 37,
  `OD-EXECUTOR-001`) and `nomos-agent-executor-ollama` (band 37, `OD-EXECUTOR-004`), each
  exposing `pub fn Execute<P: ProcessLauncher>(task: &TaskEnvelope, launcher: &P) ->
  Result<AgentExecutionOutcome, AgentExecutionError>`. Read in full: the two crates share no
  trait and no common `AgentExecutionOutcome`/`AgentExecutionError` type — both module docs
  say so explicitly, and `OD-EXECUTOR-001`'s and `OD-EXECUTOR-004`'s own text each decline a
  shared dispatch trait "ahead of a real need."
- `crates/host/nomos-cli/src/agent.rs` is this workspace's one real precedent for combining
  the two: `Backend` is "a plain match over two already-independent free functions, each
  crate's own... There is no such trait," and `Dispatch` is "the entire dispatch, not a
  stand-in for one." This is the only place in the workspace today that calls both
  `nomos_agent_executor_claude_code::Execute` and `nomos_agent_executor_ollama::Execute` from
  one caller.
- `nomos-check-orchestration::Run` (band 40) was read in full
  (`crates/orchestration/nomos-check-orchestration/src/run.rs`). Its real signature —
  `Run<P: ProcessLauncher>(sources: &[SourceFile], variant: BuildVariant, root: &Path,
  launcher: &P, selected: &[RuleId]) -> CheckOutcome` — takes source already walked by a
  composition root, a build variant read through `env!` at the calling binary, and a rule
  selection; a caller wanting to use it as a workflow step body would itself need to become a
  second composition root capable of a directory walk, which is a materially heavier shape
  than a bare `AgentExecutor` dispatch for a first increment to carry honestly.
- A live, separate, unmerged ledger claim (`P14-EXECUTOR-005-BACKEND-FLAG-TRIGGER-RECHECK`,
  held by another session as of this writing) is independently re-checking whether
  `OD-EXECUTOR-004`'s own "no shared trait" restraint should be revisited. This record does
  not depend on, preempt, or assume any outcome of that question — the mechanism this record
  names below reads both `AgentExecutor` crates through the same plain-match shape
  `nomos-cli::agent::Backend` already uses today, regardless of what a future shared trait
  might look like.

## The Decision

**Under the user's direct, session-specific instruction to build the workflow tier's first
real execution increment now, the mechanism is: a new crate, `nomos-workflow-orchestration`
(band 40, `crates/orchestration/nomos-workflow-orchestration`, a peer of
`nomos-work-orchestration`/`nomos-check-orchestration`/`nomos-spec-orchestration` — none of
the four may name another), that runs an ordered sequence of `WorkflowStep` declarations, each
paired with a real dispatch to one of this workspace's two real `AgentExecutor` crates,
refusing whichever step first declares itself incoherent before that step's body ever runs.**

Concretely:

- `Body`, an enum naming `ClaudeCode(TaskEnvelope)` and `Ollama(TaskEnvelope)` directly —
  `nomos-cli::agent::Backend`'s own shape, reused rather than reinvented, not a trait generic
  over either crate.
- `WorkflowStepPlan { declaration: WorkflowStep, body: Body }` — `WorkflowStep` declares no
  dispatch mechanism of its own (`OD-WORKFLOW-003`), so pairing one with a concrete `Body` is
  this crate's own decision, not something the contract states.
- `Run(plan: &[WorkflowStepPlan], launcher: &impl ProcessLauncher) -> WorkflowOutcome` —
  iterates `plan` in order; before dispatching a step, calls
  `step.declaration.Is_Coherent()` and, if false, stops and reports `Refused` without ever
  calling `step.body`; otherwise dispatches through the real executor `Body` names and
  continues on success, or stops and reports `Failed` on the first dispatch error. An empty
  `plan` completes vacuously. This gives `WorkflowStep::Is_Coherent` its first real consumer
  anywhere in this workspace, the same "vocabulary, then its first real consumer" shape
  `OD-WORKFLOW-001` already walked for `RunId` through `Gate`.

This is real sequencing and a real enforcement of the one static rule `WorkflowStep` carries
today — more than "give the vocabulary a token consumer," and deliberately not more than
that. It is scoped by explicit subtraction, named in full in "What This Does Not Build" below.

## What This Does Not Build

Corrected in place at version 2: four of the clauses version 1 wrote here either named an
absence this workspace has since built, or gave a reason that has since stopped holding. The
amendment section below quotes each of the four and names what moved it, so this list is what
stays out and the correction can still be checked against the words it replaced.

No immutable published artifacts (`WF-009`). No branch/merge semantics or bounded parallelism
(`WF-010`) — `Run` is one ordered sequence, nothing more. No independently versioned workflow
definitions with pinned historical replay (`WF-011`). No `WF-012` cache or cancellation
*runtime* — `Cacheability` and `CancellationBehavior` are read by `Is_Coherent` and carried on
each step's declaration, and nothing here substitutes a prior result for a dispatch or cuts a
dispatch short; version 1's single clause grouped `RetryPolicy`, `Timeout` and `Compensation`
under the same absence, and those three have been honored since `29bc3e20`. No deduplication
token minted — `Is_Coherent` refuses a retryable, non-idempotent, side-effecting step that
requires none and declares no compensation, so every retry runs under a cover the contract
already checked, but nothing here mints a per-attempt token or hands one to any dispatch
target. No compensating *step* — `Compensation::ExternallyCompensated` is reported as owed to
whatever assembled the plan, never composed into another step's compensating run, which
`Compensation`'s own doc declines to name. No shared dispatch trait — `Body` names each real
dispatch target directly, the same restraint `OD-EXECUTOR-001`/`OD-EXECUTOR-004` already hold,
and this record does not reach into the separate `OD-EXECUTOR-004` shared-trait question. No
`WorkResult` assembly — a step's real outcome is carried in `StepOutcome` exactly as its
dispatch target reported it, and nothing here builds a `WorkResult` out of it; version 1 gave
as the reason that neither `AgentExecutor` produces the material an honest assembly would
need, and that reason no longer holds — `OD-EXECUTOR-003` decided the first real `WorkResult`
and `nomos-agent-executor-claude-code` constructs one — so this exclusion stands on this crate
assembling none, not on there being none to carry.

Two of version 1's clauses are no longer absences at all, and are corrected here rather than
left to read as current. The `nomos-check-orchestration::Run` step body version 1 named as "a
real, heavier next step, not built here" was built by `P40-WORKFLOW-CHECK-BODY`, with
`P40-WORKFLOW-CORRECTION-BODY` and `P40-WORKFLOW-GATE-BODY` adding a correction body and a
gate body beside it; and the CLI verb version 1 declined as "a separate, later increment with
its own argv design" was built by `P40-WORKFLOW-CLI-VERB`, which is `nomos workflow`. Those
increments moved this crate's band and its dependency edges with them, so `README.md` and
`OD-RULES-020` are the authorities on where this crate sits and which same-zone edges it is
allowed — not the band-and-peer parenthetical version 1 wrote in "The Decision" above, which
named a band this crate has since left and an exclusion those bodies' own edges replaced.

It does not reopen `OD-WORKFLOW-001` through `004`, `OD-EXECUTOR-001`, or `OD-EXECUTOR-004`.
It does not claim `OD-ROADMAP-001` already covers this decision — checked directly above, it
does not name the workflow tier, and this record does not amend `OD-ROADMAP-001` to add it.

## Amendment to OD-WORKFLOW-002

`OD-WORKFLOW-002`'s own text describes a tier that "stands exactly where `OD-WORKFLOW-001`
left it" until one of its three named conditions fires. This record does not claim any of the
three has fired — the survey above re-confirms none has. `OD-WORKFLOW-002` is amended in place
(version bumped, a relation added, an amendment section appended, the same shape
`OD-WORKFLOW-003` and `OD-WORKFLOW-004` each already used) to record that a real execution
increment now exists in this workspace anyway, built under the user's own direct,
session-specific override rather than under any of the three conditions `OD-WORKFLOW-002`
itself named. The three conditions, and everything else that record found, are otherwise
unchanged: they remain the honest triggers for the *next* increment past this one.

## Amendment: The Retry, Timeout and Compensation Runtime Version 1 Declined Is Built

`P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` landed at `29bc3e20` and built three of
the things version 1 of this record named among what it does not build. That item reserved no
record territory, so its own claimant measured the staleness and could not repair it, which
left this record false in a way a reader could not detect from the record itself.
`P123-OD-WORKFLOW-005-SAYS-THE-RETRY-AND-COMPENSATION-RUNTIME-IS-UNBUILT-AND-IT-LANDED` is
that repair. Version 1 was true at the revision it was written against; the corrections are
made in place above and the words they replaced are quoted here, so what changed can be
checked against what it said rather than taken on trust.

**What version 1 said.** "No `WF-012` retry or compensation *runtime* — `RetryPolicy`,
`Timeout`, `Cacheability`, `CancellationBehavior`, and `Compensation` are read by
`Is_Coherent` and carried on each step's declaration; none of their runtime behavior (an
actual retry, an actual cache hit, an actual cancellation, an actual compensating run) is
implemented." "No `nomos-check-orchestration::Run` step body — named above as a real, heavier
next step, not built here." "No CLI verb (`nomos workflow ...`) — this crate's own scripted
tests are its real exercise; a command surface is a separate, later increment with its own
argv design, not required to prove this mechanism for real." And, of the `WorkResult`
exclusion, "because neither `AgentExecutor` produces the material a honest assembly would need
any more than either did before this record."

**Retry is honored.** A failed step is re-dispatched while its own `RetryPolicy::Retry`
permits another attempt, up to `max_attempts` counting the first, and `RetryPolicy::NoRetry`
never re-dispatches. Every attempt is reported in the order it ran, with the failure of each
attempt that did not survive. `WF-012`'s own named failure — repeating a non-idempotent effect
with nothing to tell two attempts apart — is not re-guarded in the runner, because it is
guarded earlier: `Is_Coherent` refuses `RetryPolicy::Retry` on a side-effecting,
non-idempotent step that requires no deduplication token and declares no compensation, and an
incoherent declaration refuses the run before any body dispatches. So a step that reaches a
second dispatch reached it under one of those two covers, and re-checking it in the runner
would be a second authority for a rule the contract already owns.

**Timeout is measured, not enforced.** A dispatch that breaks its step's declared
`Timeout::Seconds` is reported as having exceeded the bound once the dispatch ends, rather
than being cut short — cutting one short needs the cancellation runtime this record still
declines, and `CancellationBehavior`, the declaration that would say whether a given step even
permits it, is still read by `Is_Coherent` alone. A dispatch that took exactly its bound stayed
inside it, because the declaration says the step is no longer waited on *after* that many
seconds. A run given no clock reports a declared bound as unmeasured and never as honored,
which is the one reading a run that measured nothing must not give.

**Compensation is honored, on failure only.** The completed steps of a failed run are
compensated in the reverse of the order they ran, because a later step's effect may stand on an
earlier step's. A correction body is the one body with a compensating mode this crate can
reach; a body with no compensating mode that declared `Compensation::SelfCompensating` is
reported as a refused declaration rather than silently skipped;
`Compensation::ExternallyCompensated` is reported as owed to whatever assembled the plan; and
`Compensation::None` produces no entry at all. A refused run is deliberately not compensated:
the step that refused never dispatched, so no dispatch of it failed, and whether the steps
before an incoherent declaration should be unwound is a question `WF-012` does not answer and
this crate does not decide for it.

**Two design consequences, recorded because a later reader would otherwise rediscover them.**
First, the richer report is a second type beside `WorkflowOutcome` rather than a new variant or
a new field inside it: `nomos_api::workflow` and `nomos_cli::workflow` match that enum's three
variants field by field with no wildcard, so widening it is a breaking change to two crates
outside the building item's territory. `Run` keeps its exact signature and return type and
drops the report; `Run_Unclocked` and `Run_With_Clock` report it. `Run` therefore reports less
than they do, and does not do less — the declarations are honored whichever entry point ran the
plan. Second, a correction step's compensation restores the body's own already-walked source
rather than calling `nomos_corrections::CommittedPlan::Rollback`: `CorrectionOutcome::Committed`
carries rendered strings and no `CommittedPlan`, so the receiver a rollback needs never crosses
that seam. What that reverse does not have is named rather than implied — it does not assert the
workspace has not moved since the commit, and it restores only the one path the outcome reports.

**Three follow-ups this measurement identified, named and not scheduled.** Handing the rollback
receiver across the correction seam, so a correction step's compensation is the rollback its own
outcome owns rather than a reverse this crate reconstructs. A timed-out outcome, or an overrun
that stops a run rather than merely being reported, which needs the territory of the two host
crates whose exhaustive matches make that a breaking change. And minting a per-attempt
deduplication token, so the declaration `Is_Coherent` demands becomes a mechanism rather than a
cover a retry runs under. This record names them; `work/ledger.json` is where work is scheduled,
and nothing here claims any of the three.

**What this amendment does not re-take or reopen.** Version 1's "Question" and "What Was
Measured" sections are measurements dated to the revision they were read at, and this amendment
re-measures none of them; one of them the tree no longer agrees with, and it is named here so it
is not read as current: version 1 grepped every real `WorkResult` construction site and found
only `nomos-agent-contracts`' own `#[cfg(test)]` module, and `nomos-agent-executor-claude-code`
now constructs one under `OD-EXECUTOR-003`. Whether that moves `OD-WORKFLOW-004`'s own finding,
or any condition `OD-WORKFLOW-002` named, is those two records' question and belongs to an item
that reserves them; this amendment neither restates those conditions nor decides them. Version
1's per-backend `Body` variants were replaced by `OD-PACKAGE-016`'s profile resolution, which is
that record's decision and is not reopened here. And this amendment does not reopen
`OD-ROADMAP-001`'s override: the increment version 1 recorded stays recorded as the narrow,
explicit, session-specific override it was, and nothing here widens or generalizes it.

## Status

Accepted. Names the workflow tier's first real execution mechanism and records, in the same
voice this workspace already uses for `OD-PACKAGE-010`'s and `OD-ROADMAP-001`'s overrides,
that it was built ahead of any of `OD-WORKFLOW-002`'s three named conditions firing, under the
user's own direct instruction to this session. Does not claim `OD-ROADMAP-001`'s generalized
retirement already covered this decision, and does not itself generalize past the workflow
tier.

Amended to version 2 by
`P123-OD-WORKFLOW-005-SAYS-THE-RETRY-AND-COMPENSATION-RUNTIME-IS-UNBUILT-AND-IT-LANDED`,
which corrected "What This Does Not Build" against the tree: the `WF-012` retry, timeout and
compensation runtime it named as unimplemented was built by
`P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` at `29bc3e20`, the check step body and the
CLI verb it declined were built by `P40-WORKFLOW-CHECK-BODY` and `P40-WORKFLOW-CLI-VERB`, and
the reason it gave for excluding `WorkResult` assembly was overtaken by `OD-EXECUTOR-003`. The decision this record names, and the override it records
that decision as having been made under, are unchanged; what moved is which absences it may
still claim.
