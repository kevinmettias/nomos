//! Zone: Application Service — the workflow tier's first real execution increment: an ordered sequence of
//! `WorkflowStep` declarations, each paired with a real dispatch to one of this
//! workspace's four real dispatch targets.
//!
//! `OD-WORKFLOW-005` names why this exists ahead of any of `OD-WORKFLOW-002`'s three
//! named conditions firing, and what it does and does not build. In short:
//! `nomos_contracts::WorkflowStep` (`OD-WORKFLOW-003`) declares what a step promises but
//! dispatches nothing on its own; `nomos-agent-orchestration::Run_Agent_Task`,
//! `nomos-check-orchestration::Run`, `nomos-correction-orchestration::Run_Correction` and
//! `nomos-gate-orchestration::Run_Gate`, the canonical orchestration seams every host
//! already calls, each dispatch their own input but share no trait. [`Body`] pairs a
//! `WorkflowStep` with one of them, and [`Run`] iterates a plan of them in order, giving
//! `WorkflowStep::Is_Coherent` its first real consumer anywhere in this workspace.
//!
//! This crate names no agent backend, in its manifest or in its source. It used to declare
//! both adapter crates as dependencies, left over from the two per-backend bodies
//! `OD-PACKAGE-016` decision 9 replaced with a declared profile and unread by any line of
//! Rust here by the end; `OD-ROADMAP-005` decision 2 retired them, so an agent step
//! carries a profile, the composition root supplies the targets and the ports that answer
//! them, and `nomos-agent-orchestration` resolves one against the other. Above band 41 because `Body::Gate` depends on
//! `nomos-gate-orchestration` (41) and a band may not depend on its own band --
//! `P40-WORKFLOW-CHECK-BODY` and `P40-WORKFLOW-CORRECTION-BODY` already moved this crate up
//! twice for the identical reason, one dependency at a time.
//!
//! `nomos_cli::workflow` (`P40-WORKFLOW-CLI-VERB`) is the one real caller of [`Run`]
//! outside this crate's own tests -- a thin renderer over it, composing exactly one step
//! per invocation today.
//!
//! `Body::Gate` is the one dispatch target whose failure ends the workflow rather than
//! merely completing as a step: `run::Dispatch` reports a `GateRunOutcome::Failed`
//! disposition as [`DispatchError::Gate`], not [`StepOutcome::Gate`] -- see [`run::
//! Platform`] and `run::Dispatch`'s own doc for why, and `P40-WORKFLOW-GATE-BODY`'s own
//! done_when for the requirement it satisfies.
//!
//! `WF-012`'s retry, timeout and compensation are honored here, not merely declared:
//! `P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` built the three halves of that
//! clause this crate can honor from the declared vocabulary alone, under the user's own
//! direct, session-specific override that `OD-WORKFLOW-005` records for this tier.
//!
//! This doc used to name `OD-ROADMAP-001`'s standing override instead, and it was the one
//! kind of wrong a reader could not detect from here: two authorities disagreeing, neither
//! of them the mechanical one. Read against each other, they do not. `OD-ROADMAP-001`
//! retires the population-of-zero caution for the AgentExecutor / ModelBackend /
//! RulePackage / corrections cluster it supersedes record by record, and its own text says
//! it does not touch a decision outside that cluster; the workflow tier is not named in it.
//! `OD-WORKFLOW-005` refuses the same claim from its own side -- its title says the
//! increment rests on the user's standing override rather than a fired `OD-WORKFLOW-002`
//! trigger, and its "What This Does Not Build" and Status both decline to claim that the
//! generalized retirement already covers this decision. Its version 2 amendment, which is
//! what records this runtime as landed, widens nothing. So the record was right and this
//! doc was wrong, and the licence is the narrow one named above.
//!
//! [`Run`] re-dispatches a failed step while its own
//! `RetryPolicy::Retry` permits another attempt, reports a step that broke its declared
//! `Timeout::Seconds` as [`StepTiming::Exceeded`] when a clock was supplied to measure it,
//! and unwinds the completed steps of a failed run in reverse through whichever
//! compensating mode each body actually supports. What honoring the three did is the
//! [`WorkflowRun`] that [`Run_Unclocked`] and [`Run_With_Clock`] report; [`Run`] itself
//! still answers with [`WorkflowOutcome`] alone, unchanged, because two hosts match that
//! type's three variants field by field.
//!
//! # What stays out
//!
//! No immutable published artifacts (`WF-009`), no branch/merge or bounded parallelism
//! (`WF-010`), no independently versioned or replayable definitions (`WF-011`), no shared
//! dispatch trait across any of the four bodies, no `WorkResult` assembly --
//! `OD-WORKFLOW-005`'s "What This Does Not Build" names each and why, apart from the
//! check-body, correction-body and gate-body steps and the CLI verb that section named as
//! later, heavier increments: those are now built, here and in `nomos-cli::workflow`.
//!
//! Three of the five declarations that section grouped under "no `WF-012` runtime" are
//! still read by `Is_Coherent` and by nothing else, and the shape of what stays out is now
//! narrower than that grouping. No cache runtime: `Cacheability::Cacheable`'s `key_inputs`
//! name fields of a schema this crate never resolves, and a prior result is never
//! substituted for a dispatch. No cancellation runtime: `CancellationBehavior` still says
//! only whether a step's declaration would permit being cut short, and nothing here cuts a
//! dispatch short -- which is also why a broken `Timeout` is *reported* after the dispatch
//! ends rather than interrupting it. No deduplication tokens: `Is_Coherent` refuses a
//! retryable non-idempotent side-effecting step that declares none, so every retry here
//! runs under a cover the contract already checked, but this crate mints no per-attempt
//! token and hands none to any dispatch target. And no compensating *step*:
//! `Compensation::ExternallyCompensated` is reported as owed to whatever assembled the
//! plan, never composed into one, because wiring one step's failure to another step's
//! compensating run is a workflow definition's concern that `Compensation`'s own doc
//! declines to name.

#![forbid(unsafe_code)]

mod body;
mod run;
mod workflow_outcome;
mod workflow_run;
mod workflow_step_plan;

#[cfg(test)]
mod tests;

pub use body::{AgentBody, Body, CheckBody, CommitIntent, CorrectionBody, GateBody};
pub use run::{ClockedPlatform, Platform, Run, Run_Unclocked, Run_With_Clock};
pub use workflow_outcome::{DispatchError, StepOutcome, WorkflowOutcome};
pub use workflow_run::{StepAttempt, StepCompensation, StepTiming, WorkflowRun};
pub use workflow_step_plan::WorkflowStepPlan;
