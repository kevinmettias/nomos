//! Band 40 — the workflow tier's first real execution increment: an ordered sequence of
//! `WorkflowStep` declarations, each paired with a real dispatch to one of this
//! workspace's two real dispatch targets.
//!
//! `OD-WORKFLOW-005` names why this exists ahead of any of `OD-WORKFLOW-002`'s three
//! named conditions firing, and what it does and does not build. In short:
//! `nomos_contracts::WorkflowStep` (`OD-WORKFLOW-003`) declares what a step promises but
//! dispatches nothing on its own; `nomos-agent-executor-claude-code` (`OD-EXECUTOR-001`),
//! the one real `AgentExecutor`, and `nomos-model-backend-ollama` (`OD-PACKAGE-013`), the
//! one real `ModelBackend`, each dispatch a `TaskEnvelope` but share no trait. [`Body`]
//! pairs a `WorkflowStep` with a concrete dispatch target
//! the same way `nomos_cli::agent::Backend` already does for a person's own single call,
//! and [`Run`] iterates a plan of them in order, giving `WorkflowStep::Is_Coherent` its
//! first real consumer anywhere in this workspace.
//!
//! # What stays out
//!
//! No immutable published artifacts, no branch/merge or bounded parallelism, no
//! independently versioned or replayable definitions, no retry or compensation runtime,
//! no `nomos-check-orchestration::Run` step body, no shared `AgentExecutor` trait, no CLI
//! verb, no `WorkResult` assembly — `OD-WORKFLOW-005`'s "What This Does Not Build" names
//! each and why.

#![forbid(unsafe_code)]

mod body;
mod outcome;
mod plan;
mod run;

#[cfg(test)]
mod tests;

pub use body::Body;
pub use outcome::{DispatchError, StepOutcome, WorkflowOutcome};
pub use plan::WorkflowStepPlan;
pub use run::Run;
