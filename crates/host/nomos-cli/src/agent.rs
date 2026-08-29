//! `nomos agent` — dispatching a task to this workspace's one real `AgentExecutor`
//! (`nomos-agent-executor-claude-code`, `--executor claude-code`, the default) or its one real
//! `ModelBackend` (`nomos-model-backend-ollama`, `--model-backend ollama`).
//!
//! `--executor` and `--model-backend` used to be one flag, `--backend`, spelling
//! `claude-code` and `ollama` as if they were peer choices of the same kind. `OD-PACKAGE-013`
//! found they are not: Ollama's real mechanism — a fixed model, one forwarded field, every
//! other `TaskEnvelope` field read and ignored, no tool-use loop, no MCP surface — matches
//! `PackageKind::ModelBackendPackage`, not `PackageKind::AgentExecutorPackage`.
//! `OD-EXECUTOR-005` then checked what that does to `OD-EXECUTOR-001`/`OD-EXECUTOR-004`'s
//! shared-trait trigger and found it has not fired: with only one real `AgentExecutor` in
//! this workspace, there is nothing to build a dispatch trait generic over. What `--backend`
//! actually needed was not an abstraction, but an honest surface — two flags naming which
//! family a caller is choosing from, so passing `--model-backend ollama` cannot be read as
//! choosing an alternative agent. Internally this is still a plain `match`, not a trait: both
//! flags parse into the same two-variant [`Backend`], and [`dispatch::Dispatch`] is exactly
//! the match it always was, unaffected by which flag supplied the value.
//!
//! Neither composition root can supply its own launcher: both depend on `nomos-platform` and
//! not on any concrete implementation of it, the same reason `nomos-lang-rust-cargo` and every
//! other `ProcessLauncher`-driven crate stays generic. This module supplies
//! `nomos_platform_std::StdProcessLauncher`, the same choice `check.rs` and `work.rs` already
//! make for their own subprocesses.
//!
//! `execute` is the only real caller either backend crate has anywhere in this workspace
//! today, other than their own tests. It renders each backend's own outcome type directly
//! rather than assembling a `nomos_agent_contracts::WorkResult` — `OD-CONTRACTS-003` made
//! `WorkResult.plan` representable as absent, but this command has no `RuleId` or `SubjectId`
//! to give a `Finding` either, since nothing dispatched it as a rule's judgment; it is a
//! person, asking a question directly. Reporting a `Finding` about nothing in particular
//! would be inventing a subject nobody named, the same category of dishonesty `OD-EXECUTOR-001`
//! already named for trusting a process's own account of itself.
//!
//! `judge-role` is different: it *does* start from a rule's own finding.
//! `nomos_rules::Check_Declared_Role_Matches_Surface` unconditionally reports
//! `Applicability::AgentRequired` for a crate, per its own module doc, and never reaches a
//! verdict — this composition root is the real, separate dispatch `role_surface.rs` names as
//! deliberately not this rule's own job. It reads exactly the two files that finding's own
//! `locations` name (`README.md`'s band-table row, the crate's committed surface snapshot),
//! builds the real `nomos_rules::RoleSurfacePair` the rule was given, and asks the chosen
//! backend the question the rule could not answer.
//!
//! # How this module is organized
//!
//! [`parsing`] turns argv into a [`Command`]; [`dispatch`] builds a bare `TaskEnvelope` for
//! `execute` and carries the one primitive both commands end at, talking to a chosen
//! [`Backend`] and rendering what it returns; [`judge_role`] is `judge-role`'s own read of a
//! crate's declared role and actual surface, judged and dispatched through that same
//! primitive. [`exit_code`] is the shared vocabulary all three render into.

mod dispatch;
mod exit_code;
mod judge_role;
mod parsing;

#[cfg(test)]
mod tests;

pub(crate) use exit_code::ExitCode;
pub(crate) use parsing::Command_From_String_Arguments;

use std::path::PathBuf;

/// Which real backend a call dispatches to. `ClaudeCode` is the one real `AgentExecutor` in
/// this workspace (`nomos-agent-executor-claude-code`, chosen by `--executor claude-code`);
/// `Ollama` is the one real `ModelBackend` (`nomos-model-backend-ollama`, chosen by
/// `--model-backend ollama`) -- a `ModelBackendPackage` instance dispatched through the same
/// `Execute<P: ProcessLauncher>` shape, not a second `AgentExecutor` (`OD-PACKAGE-013`).
/// `ClaudeCode` is every caller's default before either flag existed, and stays the default
/// now: neither flag given must reach `nomos-agent-executor-claude-code` exactly as every
/// invocation did before `nomos-model-backend-ollama` existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Backend
{
    ClaudeCode,
    Ollama,
}

/// What `nomos agent` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Execute
    {
        goal: String, effort: nomos_model_package::EffortLevel, backend: Backend
    },
    JudgeRole
    {
        crate_name: String, root: PathBuf, effort: nomos_model_package::EffortLevel, backend: Backend
    },
}

/// The effort and backend a dispatched call carries -- grouped because every caller of
/// [`dispatch::Dispatch`] threads them together, never one without the other.
#[derive(Clone, Copy)]
pub(crate) struct DispatchConfig
{
    pub(crate) effort: nomos_model_package::EffortLevel,
    pub(crate) backend: Backend,
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub(crate) fn Run(command: &Command, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match command
    {
        Command::Execute { goal, effort, backend } => dispatch::Execute_Goal(goal, DispatchConfig { effort: *effort, backend: *backend }, output, notes),
        Command::JudgeRole { crate_name, root, effort, backend } => judge_role::Judge_Role(
            judge_role::RoleRequest { crate_name, root },
            DispatchConfig { effort: *effort, backend: *backend },
            output,
            notes,
        ),
    };
}
