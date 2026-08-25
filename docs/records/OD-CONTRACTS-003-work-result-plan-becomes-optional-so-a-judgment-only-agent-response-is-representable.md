---
id: OD-CONTRACTS-003
type: decision
title: WorkResult.plan becomes Option<CorrectionPlan>, so a judgment-only agent response is representable
status: accepted
version: 1
authority: canonical-normative-record
tags:
  - agent
  - contracts
  - corrections
relations:
  - target: OD-CONTRACTS-002
    type: relates-to
  - target: OD-EXECUTOR-001
    type: relates-to
---

# WorkResult.plan becomes Option<CorrectionPlan>, so a judgment-only agent response is representable

## Question

`AGT-002` requires an agent's structured response to carry "plans, changes, claims, tests,
requested verification, assumptions, and unresolved questions." `nomos_agent_contracts::
WorkResult` bundles these into `plan: CorrectionPlan`, `claims: Vec<Finding>`, `tests:
Vec<VerificationPredicate>`, `requested_verification: Option<VerificationPredicate>`,
`assumptions: Vec<String>`, `unresolved_questions: Vec<String>`. Whether `plan`'s required
cardinality is correct, or whether an agent's response can legitimately have no plan at all,
was unmeasured until `nomos-agent-executor` — this crate's own first real caller — tried to
construct one.

## What Was Measured

Verified directly against the real code: `nomos_corrections::CorrectionPlan::New` refuses an
empty candidate list, returning `CorrectionError::Vacuous`. That refusal is correct for what
it guards — a *proposed* plan with nothing in it is a defect, the same "silence reads as
clean" failure `OD-COMPLETENESS-004` names for a different surface, not a real answer.

But a judgment-only task — an agent asked to assess something and report a claim, proposing
no code change — was never proposing a plan to begin with. `nomos-agent-executor::Execute`
dispatches exactly this shape: a single, tool-free, judgment-only invocation of Claude Code,
per `OD-EXECUTOR-001`. Its own module doc states the consequence directly: it cannot
construct a `WorkResult`, because there is no honest way to fill `plan: CorrectionPlan` for a
response that proposed nothing — an empty `CorrectionPlan` is refused, and fabricating a
one-candidate plan with an empty `ChangeSet` just to satisfy the field's type would report a
correction nobody proposed, the same category of dishonesty `OD-EXECUTOR-001`'s own amendment
found in trusting a process's free-text self-report over what actually happened.

`requested_verification` already answers the identical question at `Option` cardinality, for
`WorkResult`'s own stated reason: "the same shape `nomos_ledger::LedgerItem::verification`
already uses." A submission's own verification predicate may legitimately be absent; a
submission's own plan can be absent for the same reason, and nothing about `plan`'s previous
required cardinality reflected a decision that it could not be — only that no caller had yet
needed it to be.

## The Decision

`WorkResult.plan` is `Option<CorrectionPlan>`. `None` represents a response that proposed no
change — a judgment, a claim, an answer to the task's `goal`, nothing more. `Some(plan)`
remains exactly what it always was, and `CorrectionPlan::New`'s own refusal of an empty
candidate list is untouched: a *present* plan must still be a real one.

## What This Does Not Do

It does not touch `CorrectionPlan`, `CorrectionCandidate`, or any of `nomos-corrections`'s
own stage/validate/commit lifecycle. `CorrectionPlan::New`'s `Vacuous` refusal stays exactly
as strict as it was; this record only lets `WorkResult` say "no plan," rather than forcing a
plan that does not exist.

It does not decide how `nomos-agent-executor` or any other caller assembles the rest of a
`WorkResult` from an executor's free-text response — `claims`, `assumptions`, and
`unresolved_questions` still have no honest, general mapping from unstructured text, and
building one is a separate question `TaskEnvelope.expected_output_schema` and Claude Code's
own `--json-schema` support point toward, not answered here.

It does not retroactively require any existing caller to change. Verified directly: the only
production or test construction site for `WorkResult` in this workspace is its own crate's
test, updated alongside this record.

## Status

Accepted.
