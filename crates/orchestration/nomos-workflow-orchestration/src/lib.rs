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
//! # The definition engine, beside the ordered line
//!
//! `OD-ROADMAP-006` decision 2 supersedes four of `OD-WORKFLOW-005`'s "What This Does Not
//! Build" clauses by name and by version -- `WF-009`'s immutable published artifacts,
//! `WF-010`'s branch, merge and bounded parallelism, `WF-011`'s independently versioned
//! definitions with pinned historical replay, and the **cache** half of its `WF-012`
//! clause. [`WorkflowDefinition`] is what that authorizes, and it is built beside [`Run`]
//! rather than inside it.
//!
//! A definition is an ordered list of [`WorkflowNode`]s, published through
//! [`WorkflowDefinition::Publish`] with its own [`WorkflowDefinitionId`] and version, and
//! immutable once published. A node's own name is the value it publishes; a
//! [`WorkflowNode::Branch`] chooses an arm by the [`ProducedState`] an earlier node
//! published; a [`WorkflowNode::Join`] reconverges the arms it names. Publishing takes
//! every refusal [`DefinitionRefusal`] names -- a branch on a value no node produces most
//! of all -- so a definition that exists can always be run, and
//! [`WorkflowOutcome::Refused`] never appears in a definition run. That is the same
//! guarantee `Is_Coherent` already gave [`Run`], moved from the step that reaches it to
//! the whole plan before any of it dispatches.
//!
//! Nodes that read none of each other's values fall into one dependency wave, and a wave
//! is cut into [`DispatchGroup`]s of at most the bound a caller states through
//! [`Parallelism`]. Determinism survives that: every report is indexed and ordered by the
//! declaration, never by the order a group's members were visited in, which
//! [`GroupVisitOrder`] exists to make falsifiable rather than merely asserted. And
//! [`Run_Definition`] records its run beside the definition it ran under, so [`Replay`]
//! runs *that* definition and refuses when the one a caller offers now has moved.
//!
//! A third report type, [`DefinitionRun`], rather than new variants: `WorkflowOutcome`'s
//! three variants and `DispatchError`'s three are matched field by field, with no wildcard
//! arm, by three sites across `nomos-api` and `nomos-cli`, so widening either is a
//! breaking change to two crates this increment may not edit. The sequential path keeps
//! its behaviour, its signatures and its tests unchanged.
//!
//! # What stays out
//!
//! No shared dispatch trait across any of the four bodies, and no `WorkResult` assembly --
//! `OD-WORKFLOW-005`'s "What This Does Not Build" names each and why, and
//! `OD-ROADMAP-006` leaves both untouched.
//!
//! No cancellation runtime, which is the half of `WF-012` that `OD-ROADMAP-006`
//! deliberately did **not** supersede: `CancellationBehavior` still says only whether a
//! step's declaration would permit being cut short, and nothing here cuts a dispatch
//! short -- which is also why a broken `Timeout` is *reported* after the dispatch ends
//! rather than interrupting it, and why no definition node can be cancelled either.
//!
//! No deduplication tokens: `Is_Coherent` refuses a retryable non-idempotent
//! side-effecting step that declares none, so every retry here runs under a cover the
//! contract already checked, but this crate mints no per-attempt token and hands none to
//! any dispatch target. And no compensating *step*: `Compensation::ExternallyCompensated`
//! is reported as owed to whatever assembled the plan, never composed into one, because
//! wiring one step's failure to another step's compensating run is a workflow
//! definition's concern that `Compensation`'s own doc declines to name.
//!
//! Two limits on what *was* built, stated here rather than left to be discovered. No
//! thread is spawned and no executor is composed, so a group's members are dispatched one
//! after another; what the bound buys is an explicit, checkable statement of how much
//! independent work may be in flight, not work in flight. And the cache key is the whole
//! body and the whole input schema rather than `Cacheable::key_inputs`, whose field names
//! this crate has no resolver for -- strictly stronger than the declaration, so nothing is
//! ever substituted that the declaration forbade.

#![forbid(unsafe_code)]

mod body;
mod definition_run;
mod run;
mod run_definition;
mod workflow_definition;
mod workflow_outcome;
mod workflow_run;
mod workflow_step_plan;

#[cfg(test)]
mod tests;

pub use body::{AgentBody, Body, CheckBody, CommitIntent, CorrectionBody, GateBody};
pub use definition_run::{BranchChoice, DefinitionRun, DispatchGroup, NodeDisposition, NodeReport};
pub use run::{ClockedPlatform, Platform, Run, Run_Unclocked, Run_With_Clock};
pub use run_definition::{GroupVisitOrder, Parallelism, Replay, ReplayRefusal, Run_Definition, WorkflowExecution, WorkflowRunRecord};
pub use workflow_definition::{BranchArm, DefinitionRefusal, ProducedState, WorkflowDefinition, WorkflowDefinitionId, WorkflowNode};
pub use workflow_outcome::{DispatchError, StepOutcome, WorkflowOutcome};
pub use workflow_run::{StepAttempt, StepCompensation, StepTiming, WorkflowRun};
pub use workflow_step_plan::WorkflowStepPlan;
