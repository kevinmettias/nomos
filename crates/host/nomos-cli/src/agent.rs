//! `nomos agent` — dispatching a task to this workspace's one real `AgentExecutor`
//! (`nomos-agent-executor-claude-code`, `--executor claude-code`, the default) or its one real
//! `ModelBackend` (`nomos-model-backend-ollama`, `--model-backend ollama`), through
//! `nomos-agent-orchestration`'s own shared seam.
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
//! flags parse into the same two-variant [`Backend`], and `nomos_agent_orchestration::
//! Run_Agent_Execute`/`Run_Agent_Judgment` are exactly the match they always were, unaffected
//! by which flag supplied the value or which host now calls them.
//!
//! # `P43-AGENT-CANONICAL-SEAM-2`: this module used to own the whole dispatch
//!
//! Before this item, `dispatch`/`judge_role` assembled a `TaskEnvelope`, matched on
//! `Backend`, called `nomos_agent_executor_claude_code::Execute_Task`/
//! `nomos_model_backend_ollama::Execute_Task` directly with a `StdProcessLauncher` fixed
//! here, and rendered whichever outcome came back -- the only major verb family in this
//! workspace with no orchestration crate between it and this binary, so `nomos-api` had no
//! way to dispatch an agent task, choose a backend, or read a result at all. That
//! composition now lives in `nomos-agent-orchestration`, the identical migration
//! `P40-CORRECTIONS-CANONICAL-SEAM` already made for `correct.rs`. This module supplies
//! `nomos_platform_std::StdProcessLauncher` for the seam's own one platform port -- still
//! this crate's own choice, the same reason `check.rs` and `work.rs` make theirs -- and
//! renders an `nomos_agent_orchestration::AgentDispatchOutcome` into the exact text this
//! command has always produced. [`Backend`] and [`DispatchConfig`] are both
//! `nomos_agent_orchestration`'s own, re-exported under this module's own names rather
//! than a second, host-local copy of either.
//!
//! `execute` is the only real caller either backend crate has anywhere in this workspace
//! today, other than their own tests and `nomos_workflow_orchestration`'s own dispatch. It
//! renders each backend's own outcome type directly rather than assembling a
//! `nomos_agent_contracts::WorkResult` — `OD-CONTRACTS-003` made `WorkResult.plan`
//! representable as absent, but this command has no `RuleId` or `SubjectId` to give a
//! `Finding` either, since nothing dispatched it as a rule's judgment; it is a person, asking
//! a question directly. Reporting a `Finding` about nothing in particular would be inventing
//! a subject nobody named, the same category of dishonesty `OD-EXECUTOR-001` already named
//! for trusting a process's own account of itself.
//!
//! `judge-role` is different: it *does* start from a rule's own finding.
//! `nomos_rules::Check_Declared_Role_Matches_Surface` unconditionally reports
//! `Applicability::AgentRequired` for a crate, per its own module doc, and never reaches a
//! verdict — this composition root is the real, separate dispatch `role_surface.rs` names as
//! deliberately not this rule's own job. It reads exactly the two files that finding's own
//! `locations` name (`README.md`'s band-table row, the crate's committed surface snapshot),
//! builds the real `nomos_rules::RoleSurfacePair` the rule was given, and hands both it and
//! the finding to `Run_Agent_Judgment`, which asks the chosen backend the question the rule
//! could not answer.
//!
//! # How this module is organized
//!
//! [`parsing`] turns argv into a [`Command`]; [`dispatch`] chooses a launcher for `execute`
//! and renders whatever `nomos_agent_orchestration::AgentDispatchOutcome` either command's
//! own dispatch produces -- the one rendering both commands end at; [`judge_role`] is
//! `judge-role`'s own read of a crate's declared role and actual surface, judged here and
//! dispatched through that same rendering. [`exit_code`] is the shared vocabulary all three
//! render into.

mod dispatch;
mod exit_code;
mod judge_role;
mod parsing;

#[cfg(test)]
mod tests;

pub(crate) use exit_code::ExitCode;
pub(crate) use nomos_agent_orchestration::{Backend, DispatchConfig};
pub(crate) use parsing::Command_From_String_Arguments;

use std::path::PathBuf;

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

#[cfg(test)]
mod run_coverage
{
    use super::*;

    /// `Run` dispatched through its `JudgeRole` arm, driving the real top-level function
    /// end to end. A root with no `README.md` fails inside `Judge_Role` before either arm
    /// ever reaches a live backend subprocess, the same safe path `agent/tests.rs`'s own
    /// `Run` coverage and `judge_role.rs`'s own test both use.
    #[test]
    fn Test_Run_Should_Dispatch_A_Judge_Role_Command_To_Judge_Role()
    {
        let command = Command::JudgeRole {
            crate_name: "nomos-does-not-exist".to_owned(),
            root: PathBuf::from("no-such-directory-anywhere-for-run-coverage-test"),
            effort: nomos_model_package::EffortLevel::BackendDefault,
            backend: Backend::ClaudeCode,
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();

        let code = Run(&command, &mut output, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
    }
}
