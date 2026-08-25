---
id: OD-WORKFLOW-004
type: decision
title: The first real agent executor is not yet a real instance of WF-006's task/result protocol
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - workflow
  - agent
  - executor
  - roadmap
relations:
  - target: OD-WORKFLOW-002
    type: relates-to
  - target: OD-WORKFLOW-001
    type: relates-to
  - target: OD-CONTRACTS-003
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
---

# The first real agent executor is not yet a real instance of WF-006's task/result protocol

## Question

`OD-WORKFLOW-002` (v2) named three conditions that would decide the workflow tier's next real
increment. The third read: "`ModelBackend`/`AgentExecutor` infrastructure reaches a real
executor ... giving `WF-006`'s 'API-hosted, subscription-agent, human, and recorded-replay
executors' a first real instance to check a shared task/result protocol against." Since that
record was accepted, `P14-AGENT-EXECUTOR-CLAUDE-FIRST-INCREMENT` shipped
`nomos-agent-executor`'s real `Execute` function and `P14-CLI-AGENT-EXECUTE` gave it a real
CLI caller. An executor now genuinely exists. Whether that satisfies condition 3 in substance,
not only at the letter of its wording, had not been checked against the live tree before this
record -- the same discipline `OD-WORKFLOW-002` itself insisted on rather than trusting that
time passing answers the question.

## What Was Measured

Read directly from the live tree, not assumed from either record's own prior description of
it:

- `nomos-agent-executor`'s own module doc (`crates/agent/nomos-agent-executor/src/lib.rs`)
  states plainly: "It does not assemble a `nomos_agent_contracts::WorkResult`." `Execute`
  reads `TaskEnvelope.goal` and, since `OD-CONTRACTS-004`, `TaskEnvelope.effort`, and returns
  `AgentExecutionOutcome` -- a free-text result, never the `WorkResult` type `WF-006`'s
  "task/result protocol" names on its result side.
- `OD-CONTRACTS-003`, accepted the same window `nomos-agent-executor` shipped, already
  measured this precisely and left it open on purpose: "It does not decide how
  `nomos-agent-executor` or any other caller assembles the rest of a `WorkResult` from an
  executor's free-text response -- `claims`, `assumptions`, and `unresolved_questions` still
  have no honest, general mapping from unstructured text ... not answered here."
- Grepped across the whole workspace (`crates --include=*.rs`) for every real `WorkResult`
  construction site: exactly two, both inside `nomos-agent-contracts`' own
  `#[cfg(test)]` module (`work_result.rs`). Zero production constructors exist anywhere,
  including in `nomos-cli`'s own two real callers of the executor -- `nomos agent execute`
  and the in-progress `nomos agent judge-role` -- both of which say directly in their own
  comments that they do not assemble one ("It does not assemble a `WorkResult` or a
  `Finding`: this is a person's direct ... question").
- `WF-006`'s own corpus text (`04-checks-gates-corrections-and-governance.md`) names four
  executor kinds sharing one protocol: "API-hosted, subscription-agent, human, and
  recorded-replay executors shall use the same task/result protocol." Grepped for all four
  terms across `crates` and `docs/records`: the only hits are `WorkflowStep`'s own module doc
  (`crates/contracts/nomos-contracts/src/workflow_step.rs`) quoting `WF-006`'s list verbatim
  as part of `OD-WORKFLOW-003`'s vocabulary admission -- not an implementation of any of the
  four. Exactly one kind, subscription-agent (dispatching to a local Claude Code subprocess),
  has a real instance, and that instance realizes only the task half of the named protocol.
- The other two named conditions were re-verified rather than assumed stale. Grepped directly:
  `nomos_gate_orchestration::Run_Gate` still has exactly the same two real callers
  `OD-WORKFLOW-002` found -- `nomos-cli`'s `gate.rs`, `nomos-api`'s `lib.rs` -- unchanged. A
  word-boundary grep for `phase`/`Phase` across
  `crates/orchestration/nomos-gate-orchestration/src` and
  `crates/orchestration/nomos-check-orchestration/src` still returns zero matches in either;
  `Gate` still has no phase concept.

## The Finding

**Condition 3 is satisfied at the letter of `OD-WORKFLOW-002`'s wording and not at its
substance.** A real executor exists, but it produces no real result in the `WorkResult` shape
the protocol names, no second executor kind exists to demonstrate the protocol is actually
*shared* rather than merely used once, and none of `WF-006`'s other three named kinds has any
trace anywhere in this workspace. Reading the condition as fired on the strength of one
tool-dispatching, result-less executor would license starting the `WF-009`..`012` engine on
evidence that cannot yet answer the question that engine exists to serve. Conditions 1 and 2
remain exactly as unfired as `OD-WORKFLOW-002` found them. **No genuine second increment for
the workflow tier has arrived.**

## What This Does Not Do

It does not build `WorkResult` construction for `nomos-agent-executor` or any CLI caller --
`OD-CONTRACTS-003` already named that as a separate, unanswered question, and it stays
unanswered here. It does not build `WorkflowStep`, any part of the `WF-009`..`012` engine, or
a phase concept for `Gate`. It does not add a second executor kind, real or stubbed, to
demonstrate sharing -- inventing one to satisfy this record's own question would repeat the
"no invented shape ahead of a real case" mistake `OD-WORKFLOW-002` itself already declined to
make. It does not reopen `OD-CONTRACTS-003`, `OD-EXECUTOR-001`, or `OD-WORKFLOW-001`.

## Amendment to OD-WORKFLOW-002

`OD-WORKFLOW-002`'s condition 3 is narrowed by this record to require a real executor that
also constructs a real `WorkResult` -- not merely a real executor -- before the trigger counts
as fired. `OD-WORKFLOW-002` is amended in place (version bumped, a relation added, an
amendment section appended) to carry this narrower wording, the same way `OD-WORKFLOW-003`
already amended it once for a different clause.

## Status

Accepted. Re-checks `OD-WORKFLOW-002`'s third named condition against the live tree after
`nomos-agent-executor`'s arrival, finds it fired only at the letter and not the substance of
its own wording, and sharpens the condition's text with the evidence this audit found rather
than either declaring victory or leaving a now-misleading condition unchanged.
