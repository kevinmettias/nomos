//! `nomos agent` — dispatching a task to this workspace's one real `AgentExecutor`
//! (`nomos-agent-executor-claude-code`, `--executor claude-code`, the default) or its one real
//! `ModelBackend` (`nomos-model-backend-ollama`, `--model-backend ollama`), through
//! `nomos-agent-orchestration`'s own shared seam.
//!
//! # This module is the composition root for both adapters
//!
//! `OD-ROADMAP-005` decision 2: the generic path names a port and a composition root supplies
//! the concrete pair. [`Shipped_Targets`] is that supply. `nomos-agent-orchestration` used to
//! name both adapter crates itself and match a backend enum onto their functions, so the seam
//! two hosts share knew which vendors existed; it names `nomos_agent_contracts`' two ports
//! now, and this file is where Claude Code and Ollama are chosen.
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
//! choosing an alternative agent. Each flag names a family label that a declared target
//! answers to, and `nomos_agent_orchestration::Selected_Dispatch` matches the two against the
//! set [`Shipped_Targets`] declares. `OD-ROADMAP-005` decision 2 then made
//! `OD-EXECUTOR-005`'s reason structural rather than only documented: an executor and a model
//! backend answer through different ports returning different types, so there is no shape in
//! which one could report the other's absent measurements as zeroes.
//!
//! # `P43-AGENT-CANONICAL-SEAM-2`: this module used to own the whole dispatch
//!
//! Before this item, `dispatch`/`judge_role` assembled a `TaskEnvelope`, matched on a
//! backend enum, called each adapter's `Execute_Task` directly with one concrete launcher
//! fixed here, and rendered whichever outcome came back -- the only major verb family in this
//! workspace with no orchestration crate between it and this binary, so `nomos-api` had no
//! way to dispatch an agent task, choose a backend, or read a result at all. That
//! composition now lives in `nomos-agent-orchestration`, the identical migration
//! `P40-CORRECTIONS-CANONICAL-SEAM` already made for `correct.rs`. This module binds
//! `nomos_composer_std::LAUNCHER` into each adapter -- still this crate's own choice, the
//! same reason `check.rs` and `work.rs` make theirs -- and renders an
//! `nomos_agent_orchestration::AgentDispatchOutcome` into the exact text this command has
//! always produced.
//!
//! `execute` renders each port's own answer directly rather than assembling a
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
pub(crate) use parsing::Command_From_String_Arguments;

use nomos_agent_contracts::DeclaredTarget;
use nomos_agent_executor_claude_code::ClaudeCodeExecutor;
use nomos_composer_std::{LAUNCHER, LauncherType};
use nomos_model_backend_ollama::OllamaModelBackend;
use std::path::PathBuf;

/// This binary's own `AgentExecutor`, bound to the platform this composition root chose.
///
/// A `static` rather than a value built per call, because a
/// [`nomos_agent_contracts::DeclaredTarget`] borrows the adapter it names and every caller
/// here wants one that outlives the call. Constructing it costs nothing -- the adapter holds
/// a reference to a unit-struct launcher and opens nothing until it is used.
static EXECUTOR: ClaudeCodeExecutor<'static, LauncherType> = ClaudeCodeExecutor::Through(&LAUNCHER);

/// This binary's own `ModelBackend`, bound to the same platform.
static MODEL_BACKEND: OllamaModelBackend<'static, LauncherType> = OllamaModelBackend::Through(&LAUNCHER);

/// Every dispatch target this binary ships, in the order a resolution reads them.
///
/// **This is the composition root `OD-ROADMAP-005` decision 2 names.** The two adapters are
/// named here, in a host, and nowhere in `nomos-agent-orchestration` or
/// `nomos-workflow-orchestration` -- which is the whole of what that decision moved. Each
/// adapter states its own family label, package kind, version and model selection; this
/// function only says which of them this binary ships, the same shape
/// `nomos_capability::Registry` already uses when a root calls
/// `registry.Offer(nomos_lang_rust::Provider_Offer())`.
///
/// `nomos-api` declares the identical pair for its own verbs, and that is a deliberate twin
/// rather than a shared dependency, the same division `judge_role`'s own file reading already
/// draws: two composition roots naming the same adapters is not one of them borrowing the
/// other's choice.
pub(crate) fn Shipped_Targets() -> Vec<DeclaredTarget<'static>>
{
    return vec![EXECUTOR.Declared_Target(), MODEL_BACKEND.Declared_Target()];
}

/// What `nomos agent` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Command
{
    Execute
    {
        goal: String, effort: nomos_model_package::EffortLevel, preferred: Option<String>
    },
    JudgeRole
    {
        crate_name: String, root: PathBuf, effort: nomos_model_package::EffortLevel, preferred: Option<String>
    },
}

/// The profile this group declares when a person states none.
///
/// A profile is not a backend, and that difference is the whole of what changed here. This
/// says which family the group asks for; `nomos_agent_orchestration::Selected_Dispatch`
/// answers it against the declared set, and answers with nothing if that set does not offer
/// the family. The line this replaced named a dispatch target directly, which no declaration
/// had to offer and no resolution had to agree to -- a run against a set that had dropped
/// that target would still have dispatched to it.
///
/// Read from the adapter rather than spelled here, so the label a person types, the label a
/// declaration states and the label this group asks for are one string with one origin.
const DECLARED_FAMILY: &str = nomos_agent_executor_claude_code::FAMILY;

/// What a command asks a dispatch for: the effort it stated, the family this group declares,
/// and whatever backend a person named as a preference.
fn Requested(effort: nomos_model_package::EffortLevel, preferred: Option<String>) -> Requested_Dispatch
{
    return Requested_Dispatch {
        profile: nomos_model_package::ModelExecutionProfile::New(
            nomos_model_package::ModelSelector::BackendFamily(DECLARED_FAMILY.to_owned()),
            effort,
        ),
        declared: Shipped_Targets(),
        preferred,
    };
}

/// The owned parts a [`nomos_agent_orchestration::BackendSelection`] borrows, kept together
/// so a caller holds one value rather than three with the same lifetime.
pub(super) struct Requested_Dispatch
{
    pub(super) profile: nomos_model_package::ModelExecutionProfile,
    pub(super) declared: Vec<DeclaredTarget<'static>>,
    pub(super) preferred: Option<String>,
}

impl Requested_Dispatch
{
    /// This request as the borrowed shape the orchestration seam takes.
    pub(super) fn Selection(&self) -> nomos_agent_orchestration::BackendSelection<'_>
    {
        return nomos_agent_orchestration::BackendSelection {
            profile: &self.profile,
            preferred: self.preferred.as_deref(),
            declared: &self.declared,
        };
    }
}

/// Runs a command, writing content to `output` and everything about it to `notes`.
///
/// Returns the exit code rather than exiting, so the whole surface is testable.
pub(crate) fn Run(command: &Command, output: &mut impl std::io::Write, notes: &mut impl std::io::Write) -> ExitCode
{
    return match command
    {
        Command::Execute { goal, effort, preferred } => {
            dispatch::Execute_Goal(goal, Requested(*effort, preferred.clone()), output, notes)
        }
        Command::JudgeRole { crate_name, root, effort, preferred } => judge_role::Judge_Role(
            judge_role::RoleRequest { crate_name, root },
            Requested(*effort, preferred.clone()),
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
            preferred: Some("claude-code".to_owned()),
        };
        let mut output = Vec::new();
        let mut notes = Vec::new();

        let code = Run(&command, &mut output, &mut notes);

        assert_eq!(code, ExitCode::NotFound);
    }
}
