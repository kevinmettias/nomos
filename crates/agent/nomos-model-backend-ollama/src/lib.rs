//! Zone: Agent — this workspace's locally hosted model backend.
//!
//! Not a second `AgentExecutor`: its real mechanism is a fixed model, one
//! forwarded field, no tool-use loop and no configuration surface, which is a
//! *model backend*, and it is named to say so.
//!
//! The dispatch itself is **XVPE's** as of 2026-09-10. Every clause of the
//! boundary — never passing the experimental flags that are the only way to open
//! a capability, launching from a freshly created isolated directory, bounding
//! the call with a wall clock since local inference has no dollar cost to bound
//! instead, and reading a clean non-empty stdout as the entire evidence of what
//! happened — moved down into `xvpe-agent-backend-ollama`. Nomos is an
//! application over that engine, and a general capability sitting up here was
//! unreachable by everything down there.
//!
//! What stays is this workspace's own vocabulary on either side of it: the
//! [`TaskEnvelope`] that goes in and the [`AgentExecutionOutcome`] that comes
//! out.
//!
//! # What this crate reads from an envelope, and what it does not
//!
//! `goal` is read. `available_tools` is *refused* when it names anything, per
//! `OD-EXECUTOR-007`: this backend runs one model turn with no tool-use loop, so
//! a declared capability is not merely unmet here but unmeetable, and saying so
//! is the difference between a gap and a false guarantee. That refusal is the
//! envelope's mechanism rather than this backend's own, which is why it reads
//! the same as `nomos-agent-executor-claude-code`'s.
//!
//! Everything else — `scope`, `prohibited_changes`, `knowledge_context`,
//! `applicable_rules` and `effort` — is accepted and ignored. `effort`
//! specifically: nothing in this command line has a control that `EffortLevel`
//! honestly maps onto, and inventing one ahead of measuring it would be an
//! approximation asserted rather than observed. A real gap, not a silent one.
//!
//! # The response is never evidence
//!
//! Adversarial measurement found this backend's model narrating a file write and
//! a shell command it structurally could not perform. A clean exit with something
//! to say is the whole of what happened; the text is what it said, not a report
//! of what it did.
//!
//! # How a generic path reaches this crate
//!
//! Through [`OllamaModelBackend`], which implements `nomos-agent-contracts`'
//! [`nomos_agent_contracts::ModelBackend`] port over [`Execute_Task`] and
//! declares itself as the routing target a composition root offers.
//! `OD-ROADMAP-005` decision 2 is why: until it,
//! `nomos-agent-orchestration` named this crate in its own manifest and
//! called [`Execute_Task`] from a match arm.
//!
//! The port's answer is a `ModelAnswer`, and what it does *not* carry is the
//! point. It has no spend, no duration and no denial list, because this
//! mechanism establishes none of them -- so a caller holding one cannot ask it
//! for a cost and be handed a zero. The sibling port an `AgentExecutorPackage`
//! answers through carries all four, and the two are separate traits rather
//! than one trait with optional fields for exactly that reason.

#![forbid(unsafe_code)]

mod agent_execution_error;
mod agent_execution_outcome;
mod ollama_model_backend;

pub use agent_execution_error::AgentExecutionError;
pub use agent_execution_outcome::AgentExecutionOutcome;
pub use ollama_model_backend::{FAMILY, OllamaModelBackend};

use std::path::Path;

use nomos_agent_contracts::TaskEnvelope;
use nomos_platform::ProgramLauncher;
use nomos_platform_xvpe::XvpeLauncher;
use xvpe_agent_backend_ollama::{DEFAULT_TIMEOUT, OllamaDispatch};
use xvpe_agent_execution::{
    AgentCapability, AgentExecutorStrategy, AgentTask, AgentWorkspace,
};

/// Dispatches `task.goal` to a locally hosted model in its own isolated
/// directory, and reads back what it produced.
///
/// # Errors
///
/// [`AgentExecutionError::UnsupportedTools`] if `task.available_tools` names
/// anything, before any process is started.
/// [`AgentExecutionError::Unavailable`] if the isolated directory could not be
/// created, the process could not be started, it exited non-zero — including
/// when the daemon this backend requires is unreachable — or a bound killed it.
pub fn Execute_Task<Launcher: ProgramLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    return Dispatch_Task(task, launcher, Capability());
}

/// [`Execute_Task`], over a caller-chosen directory rather than a freshly
/// generated one.
///
/// Public for the same reason its sibling's is: a caller that wants to inspect
/// what a dispatch left behind cannot use one whose directory is created and
/// removed where nothing could name it.
///
/// # Errors
///
/// The same as [`Execute_Task`].
pub fn Execute_In<Launcher: ProgramLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    working_directory: &Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let mut capability = Capability();
    capability.workspace = AgentWorkspace::Existing(working_directory.to_path_buf());
    return Dispatch_Task(task, launcher, capability);
}

/// The envelope, as the engine's own task.
///
/// The goal and nothing else: no schema, because a free-text answer would not
/// conform to one, and no effort, because none maps here honestly.
pub(crate) fn Task_For(task: &TaskEnvelope) -> AgentTask
{
    return AgentTask::Of(task.goal.clone());
}

/// The boundary this dispatch runs inside.
///
/// An empty directory that goes away afterward, and nothing granted. No spend
/// ceiling: local inference has no metered charge a runaway invocation could
/// inflate, so a ceiling here would be a number that means nothing.
pub(crate) fn Capability() -> AgentCapability
{
    return AgentCapability::Isolated(DEFAULT_TIMEOUT);
}

/// The one call into the engine, over a boundary already decided on, after the one
/// envelope constraint this backend can answer.
fn Dispatch_Task<Launcher: ProgramLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    capability: AgentCapability,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    Refuse_Ungrantable_Tools(task)?;

    let bridged = XvpeLauncher::Wrapping(launcher);

    let outcome = OllamaDispatch::Through(&bridged)
        .Execute(&Task_For(task), &capability)
        .map_err(AgentExecutionError::From_Engine)?;

    return Ok(AgentExecutionOutcome { response: outcome.answer });
}

/// Refuses a declared tool need this backend cannot honor, before anything runs.
///
/// Not a narrower case of its sibling's: there the grant is missing a bridge that could
/// one day exist, here the backend has no tool-use loop for a bridge to reach. Both refuse,
/// because what a caller is owed is the same answer either way — the capability it declared
/// was not granted, and the dispatch did not quietly proceed without it.
fn Refuse_Ungrantable_Tools(task: &TaskEnvelope) -> Result<(), AgentExecutionError>
{
    if task.available_tools.is_empty()
    {
        return Ok(());
    }

    let named =
        task.available_tools.iter().map(|tool| tool.As_Str()).collect::<Vec<_>>().join(", ");
    return Err(AgentExecutionError::UnsupportedTools(named));
}

#[cfg(test)]
mod tests;
