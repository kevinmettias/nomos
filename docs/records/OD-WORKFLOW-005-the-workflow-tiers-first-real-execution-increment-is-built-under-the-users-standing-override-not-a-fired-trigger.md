---
id: OD-WORKFLOW-005
type: decision
title: The workflow tier's first real execution increment is built under the user's standing override, not a fired OD-WORKFLOW-002 trigger
status: accepted
version: 1
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

No immutable published artifacts (`WF-009`). No branch/merge semantics or bounded parallelism
(`WF-010`) — `Run` is one ordered sequence, nothing more. No independently versioned workflow
definitions with pinned historical replay (`WF-011`). No `WF-012` retry or compensation
*runtime* — `RetryPolicy`, `Timeout`, `Cacheability`, `CancellationBehavior`, and
`Compensation` are read by `Is_Coherent` and carried on each step's declaration; none of their
runtime behavior (an actual retry, an actual cache hit, an actual cancellation, an actual
compensating run) is implemented. No `nomos-check-orchestration::Run` step body — named above
as a real, heavier next step, not built here. No shared `AgentExecutor` trait — `Body` names
both crates directly, the same restraint `OD-EXECUTOR-001`/`OD-EXECUTOR-004` already hold, and
this record does not reach into the separate, live `OD-EXECUTOR-004` shared-trait question
another session is independently re-checking. No CLI verb (`nomos workflow ...`) — this
crate's own scripted tests are its real exercise; a command surface is a separate, later
increment with its own argv design, not required to prove this mechanism for real. No
`WorkResult` construction — `OD-WORKFLOW-004`'s own gap stays exactly as open as that record
left it; a step's real `AgentExecutionOutcome` is carried in `StepOutcome`, never assembled
into a `WorkResult`, because neither `AgentExecutor` produces the material a honest assembly
would need any more than either did before this record.

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

## Status

Accepted. Names the workflow tier's first real execution mechanism and records, in the same
voice this workspace already uses for `OD-PACKAGE-010`'s and `OD-ROADMAP-001`'s overrides,
that it was built ahead of any of `OD-WORKFLOW-002`'s three named conditions firing, under the
user's own direct instruction to this session. Does not claim `OD-ROADMAP-001`'s generalized
retirement already covered this decision, and does not itself generalize past the workflow
tier.
