//! Band 41 — the workflow tier's first real execution increment: an ordered sequence of
//! `WorkflowStep` declarations, each paired with a real dispatch to one of this
//! workspace's three real dispatch targets.
//!
//! `OD-WORKFLOW-005` names why this exists ahead of any of `OD-WORKFLOW-002`'s three
//! named conditions firing, and what it does and does not build. In short:
//! `nomos_contracts::WorkflowStep` (`OD-WORKFLOW-003`) declares what a step promises but
//! dispatches nothing on its own; `nomos-agent-executor-claude-code` (`OD-EXECUTOR-001`),
//! the one real `AgentExecutor`, `nomos-model-backend-ollama` (`OD-PACKAGE-013`), the one
//! real `ModelBackend`, and `nomos-check-orchestration::Run`, the canonical check seam
//! both hosts already use, each dispatch their own input but share no trait. [`Body`]
//! pairs a `WorkflowStep` with a concrete dispatch target the same way `nomos_cli::agent::
//! Backend` already does for a person's own single call, and [`Run`] iterates a plan of
//! them in order, giving `WorkflowStep::Is_Coherent` its first real consumer anywhere in
//! this workspace. Above band 40 because `Body::Check` depends on
//! `nomos-check-orchestration` (band 40) and a band may not depend on its own band --
//! `P40-WORKFLOW-CHECK-BODY` moved this crate up rather than composing check dispatch
//! anywhere else, the same reason `nomos-gate-orchestration` and
//! `nomos-correction-orchestration` already sit at 41 for depending on the same crate.
//!
//! # What stays out
//!
//! No immutable published artifacts, no branch/merge or bounded parallelism, no
//! independently versioned or replayable definitions, no retry or compensation runtime,
//! no shared dispatch trait across any of the three bodies, no CLI verb, no `WorkResult`
//! assembly — `OD-WORKFLOW-005`'s "What This Does Not Build" names each and why, apart
//! from the check-body step that section named as a real, heavier next step: that one is
//! now built, here.

#![forbid(unsafe_code)]

mod body;
mod run;
mod workflow_outcome;
mod workflow_step_plan;

#[cfg(test)]
mod tests;

pub use body::{Body, CheckBody};
pub use run::Run;
pub use workflow_outcome::{DispatchError, StepOutcome, WorkflowOutcome};
pub use workflow_step_plan::WorkflowStepPlan;
