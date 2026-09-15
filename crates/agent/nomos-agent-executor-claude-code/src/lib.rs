//! Zone: Agent — this workspace's Claude Code dispatch.
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
//! `goal` and `effort` are read. `prohibited_changes` is enforced by comparison
//! around the dispatch, `available_tools` by refusing what cannot be granted, and
//! `applicable_rules` reaches the model as advisory context inside the goal —
//! each the mechanism `OD-EXECUTOR-007` decided for it.
//!
//! **`scope` is accepted and structurally unenforced**, and this crate says so
//! rather than implying otherwise by enforcing its three siblings. Its decided
//! mechanism was one `--add-dir` per named path under `--restricted`; the
//! 2026-09-12 dispatch migration removed both flags, and the engine's boundary is
//! now one whole directory, which an arbitrary file list does not reduce to.
//! `OD-EXECUTOR-007` version 2 returned it to undecided rather than elect a
//! replacement no adversarial run has tested. `knowledge_context` was never given
//! a mechanism at all — `OD-EXECUTOR-009` is why.
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

use std::path::{Path, PathBuf};

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
/// `root` is what `task.prohibited_changes`'s repository-relative paths resolve
/// against. It is a parameter rather than this crate's own discovery because a
/// `Territory` says nothing about which checkout it describes, and a crate that
/// guessed would protect the wrong tree in a worktree.
///
/// # Errors
///
/// [`AgentExecutionError::UnsupportedTools`] if `task.available_tools` names
/// anything, before any process is started.
/// [`AgentExecutionError::Unavailable`] if the dispatch produced no answer at
/// all — the working directory could not be created, the process could not be
/// started, it exited non-zero, or a bound killed it.
/// [`AgentExecutionError::Unparseable`] if it answered and the answer could not
/// be read.
/// [`AgentExecutionError::ProhibitedChange`] if a path `task.prohibited_changes`
/// names differs after the dispatch from before it.
pub fn Execute_Task<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    root: &Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    return Dispatch(task, launcher, Capability(), root);
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
    root: &Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    let mut capability = Capability();
    capability.workspace = AgentWorkspace::Existing(working_directory.to_path_buf());
    return Dispatch(task, launcher, capability, root);
}

/// The envelope, as the engine's own task.
pub(crate) fn Task_For(task: &TaskEnvelope) -> AgentTask
{
    let asked = AgentTask::Of(Goal_For(task)).Answering(String::from(JSON_SCHEMA));
    return match Effort_For(task.effort)
    {
        Some(effort) => asked.With_Effort(effort),
        None => asked,
    };
}

/// The goal, with the rules the envelope declares named inside it.
///
/// Advisory and stated as such: telling a model what binds it is not enforcement,
/// and `OD-EXECUTOR-007` decided the validation half stays undecided until there
/// is a real `WorkResult` to check rules against. The goal's own text is the only
/// channel left for it — the separate system-prompt flag this originally used went
/// with the 2026-09-12 dispatch migration.
fn Goal_For(task: &TaskEnvelope) -> String
{
    if task.applicable_rules.is_empty()
    {
        return task.goal.clone();
    }

    let named =
        task.applicable_rules.iter().map(|rule| rule.As_Str()).collect::<Vec<_>>().join(", ");
    return format!("{} (bound by these rules: {named})", task.goal);
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

/// The one call into the engine, over a boundary already decided on, between the
/// two envelope constraints this crate enforces itself.
fn Dispatch<Launcher: ProcessLauncher>(
    task: &TaskEnvelope,
    launcher: &Launcher,
    capability: AgentCapability,
    root: &Path,
) -> Result<AgentExecutionOutcome, AgentExecutionError>
{
    Refuse_Ungrantable_Tools(task)?;
    Refuse_An_Undecidable_Root(task, root)?;

    let before = Contents_Of(&task.prohibited_changes.paths, root);

    let bridged = XvpeLauncher::Wrapping(launcher);

    let outcome = ClaudeCodeDispatch::Through(&bridged)
        .Execute(&Task_For(task), &capability)
        .map_err(AgentExecutionError::From_Engine)?;

    Refuse_Changed_Paths(&before, root)?;

    return response::Outcome_For(&outcome);
}

/// Refuses a declared tool need this crate cannot honor, before anything runs.
///
/// Building the real grant needs a bridge that does not exist: `available_tools`
/// names Nomos capabilities, the command line grants its own tool names, and the
/// one thing that could carry the former to a subprocess is an MCP server this
/// dispatch reaches none of. `OD-EXECUTOR-007` decided refusal over silently
/// dropping the field until that bridge is built.
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

/// Refuses paths to protect whose tree is not decidable.
///
/// A caller with nothing to protect may pass any root, including one naming
/// nothing — there is no path for it to resolve. The moment it declares one, the
/// root has to say which checkout, and only an absolute path does.
fn Refuse_An_Undecidable_Root(
    task: &TaskEnvelope,
    root: &Path,
) -> Result<(), AgentExecutionError>
{
    if task.prohibited_changes.paths.is_empty() || root.is_absolute()
    {
        return Ok(());
    }

    return Err(AgentExecutionError::UnresolvableRoot(root.display().to_string()));
}

/// What every path `prohibited_changes` names holds right now, by path.
///
/// `None` for a path that is not there, so a prohibited path the dispatch
/// *creates* is a difference too, rather than one that only counts once it
/// already exists.
///
/// Whole contents rather than `OD-EXECUTOR-007`'s word "hash": the comparison is
/// the point, and this crate cannot add a hashing dependency without reaching
/// outside its own manifest. Byte equality answers the same question with no
/// collision to reason about, over a list that names specific files.
fn Contents_Of(prohibited: &[String], root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)>
{
    return prohibited
        .iter()
        .map(|path| root.join(path))
        .map(|path|
        {
            let content = std::fs::read(&path).ok();
            (path, content)
        })
        .collect();
}

/// Refuses if anything recorded before the dispatch is different now.
fn Refuse_Changed_Paths(
    before: &[(PathBuf, Option<Vec<u8>>)],
    root: &Path,
) -> Result<(), AgentExecutionError>
{
    for (path, content) in before
    {
        if std::fs::read(path).ok().as_ref() != content.as_ref()
        {
            let named = path.strip_prefix(root).unwrap_or(path);
            return Err(AgentExecutionError::ProhibitedChange(named.display().to_string()));
        }
    }
    return Ok(());
}

#[cfg(test)]
mod tests;
