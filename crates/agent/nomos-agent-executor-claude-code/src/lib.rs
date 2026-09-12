//! Band 37 — this workspace's Claude Code dispatch.
//!
//! The dispatch itself is **XVPE's** as of 2026-09-10: an isolated working
//! directory, an allow-list granting nothing real rather than a deny-list that is
//! always one release behind, one `--print` turn, a spend ceiling, a wall bound
//! that kills, and an answer read from schema-validated `structured_output`
//! rather than from free text. Every one of those clauses, and the adversarial
//! verification behind them, moved down into `xvpe-agent-backend-claude-code`.
//! Nomos is an application over that engine, and a general capability sitting up
//! here was unreachable by everything down there.
//!
//! What stays is what is genuinely this workspace's: the [`TaskEnvelope`] it
//! dispatches from, the `WorkResult` it assembles, and the schema that says what
//! a work result may honestly claim.
//!
//! # What this crate reads from an envelope, and what it does not
//!
//! `goal` and `effort` are read. `scope`, `prohibited_changes`,
//! `available_tools`, `knowledge_context` and `applicable_rules` are accepted and
//! ignored, because nothing in this workspace enforces them yet — and this crate
//! does not pretend otherwise by honoring some and not others. Scope that *is*
//! enforced travels as the engine's capability boundary instead, which is
//! structural rather than advisory.
//!
//! # Why the work result stays narrow
//!
//! `plan`, `claims`, `tests` and `requested_verification`
//! stay structurally absent rather than model-filled. Under this boundary the model has seen no
//! real file and computed no real digest by the time it answers, so nothing here
//! has an honest grounding for any of the four. A schema can only make the
//! *shape* conform, and a conforming lie is not the goal.

#![forbid(unsafe_code)]

mod agent_execution_error;
mod agent_execution_outcome;
mod response;

pub use agent_execution_error::AgentExecutionError;
pub use agent_execution_outcome::AgentExecutionOutcome;
// The engine's exact money, re-exported so a caller reading `AgentExecutionOutcome::cost`
// or comparing one against `MAXIMUM_SPEND` can name its type without taking a dependency
// on the engine crate that declares it. `MAXIMUM_SPEND` below was already this type in
// this crate's public surface; only the name was missing.
pub use xvpe_agent_execution::MicroDollars;

use std::path::Path;

use nomos_agent_contracts::TaskEnvelope;
use nomos_model_package::EffortLevel;
use nomos_platform::ProcessLauncher;
use nomos_platform_xvpe::XvpeLauncher;
use xvpe_agent_backend_claude_code::{ClaudeCodeDispatch, DEFAULT_TIMEOUT};
use xvpe_agent_execution::{
    AgentCapability, AgentExecutorStrategy, AgentTask, AgentWorkspace,
    EffortLevel as EngineEffort,
};

/// The narrow, honest schema a work result is read from: `assumptions` and
/// `unresolved_questions`, both string arrays, additional properties refused.
///
/// Adversarially verified against the real command line: a prompt explicitly
/// instructed to also emit a top-level `plan` field bypassing the schema still
/// produced a `structured_output` carrying only these two declared fields, and
/// the model's own text named the reason — `additionalProperties: false` refused
/// the extra key structurally.
pub const JSON_SCHEMA: &str = r#"{"type":"object","properties":{"assumptions":{"type":"array","items":{"type":"string"}},"unresolved_questions":{"type":"array","items":{"type":"string"}}},"required":["assumptions","unresolved_questions"],"additionalProperties":false}"#;

/// A starting bound on what one dispatch may spend.
///
/// Real invocations cost between under a cent and a few tens of cents. A dollar
/// is generous headroom for a single judgment-only turn while still being a real
/// ceiling on what a runaway sequence of refused-capability retries could cost —
/// verified empirically: capped at an unreachably low budget, the command line
/// aborted before its expensive model call ran, incurring only a small triage
/// cost first. That overshoot is what this bounds rather than closes.
pub const MAXIMUM_SPEND: MicroDollars = MicroDollars::From_Micros(1_000_000);

/// Dispatches `task.goal` into its own isolated directory, and reads back what
/// it reported.
///
/// # Errors
///
/// [`AgentExecutionError::Unavailable`] if the dispatch produced no answer at
/// all — the working directory could not be created, the process could not be
/// started, it exited non-zero, or a bound killed it.
/// [`AgentExecutionError::Unparseable`] if it answered and the answer could not
/// be read.
pub fn Execute_Task<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    return Dispatch(task, launcher, Capability());
}

/// [`Execute_Task`], over a caller-chosen directory rather than a freshly
/// generated one.
///
/// Public because it is a real capability rather than a testing hook: a caller
/// that already has a directory it wants inspected afterward cannot use
/// [`Execute_Task`], whose isolated directory is created and then removed where
/// nothing could name it. It is also what a real, adversarial integration test
/// needs in order to look at what the dispatch left behind.
///
/// # Errors
///
/// The same as [`Execute_Task`].
pub fn Execute_In<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    working_directory: &Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let mut capability = Capability();
    capability.workspace = AgentWorkspace::Existing(working_directory.to_path_buf());
    return Dispatch(task, launcher, capability);
}

/// The one call into the engine, over a boundary already decided on.
fn Dispatch<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    capability: AgentCapability,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let bridged = XvpeLauncher::Wrapping(launcher);

    let outcome = ClaudeCodeDispatch::Through(&bridged)
        .Execute(&Task_For(task), &capability)
        .map_err(AgentExecutionError::From_Engine)?;

    return response::Outcome_For(&outcome);
}

/// The envelope, as the engine's own task.
pub(crate) fn Task_For(task: &TaskEnvelope) -> AgentTask
{
    let asked = AgentTask::Of(task.goal.clone()).Answering(String::from(JSON_SCHEMA));
    return match Effort_For(task.effort)
    {
        Some(effort) => asked.With_Effort(effort),
        None => asked,
    };
}

/// The boundary this dispatch runs inside.
///
/// The tightest the engine offers, plus a spend ceiling: an empty directory that
/// goes away afterward, and no capability granted at all. This crate has never
/// dispatched under anything wider, and stating it here rather than leaning on a
/// default is what keeps that visible.
pub(crate) fn Capability() -> AgentCapability
{
    return AgentCapability::Isolated(DEFAULT_TIMEOUT).Spending_At_Most(MAXIMUM_SPEND);
}

/// This workspace's effort, as the engine's.
///
/// [`EffortLevel::BackendDefault`] maps to `None`, which omits the request
/// entirely rather than naming a value meaning "the default".
///
/// [`EffortLevel::Minimal`] has no distinct counterpart below `Low` and is mapped
/// there — this crate's own approximation, not a claim of an exact match. The
/// engine's own `ExtraHigh` has no counterpart here, because this workspace's
/// enumeration closes at six values; it is not folded into `High` or `Maximum` to
/// pretend otherwise.
pub(crate) fn Effort_For(effort: EffortLevel) -> Option<EngineEffort>
{
    return match effort
    {
        EffortLevel::BackendDefault => None,
        EffortLevel::Minimal | EffortLevel::Low => Some(EngineEffort::Low),
        EffortLevel::Medium => Some(EngineEffort::Medium),
        EffortLevel::High => Some(EngineEffort::High),
        EffortLevel::Maximum => Some(EngineEffort::Maximum),
    };
}

#[cfg(test)]
mod tests;
