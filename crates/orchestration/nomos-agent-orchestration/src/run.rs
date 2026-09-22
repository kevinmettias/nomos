//! Assembling a `TaskEnvelope` for one of this workspace's two real dispatch shapes and
//! dispatching it through whichever port a declared target carries, apart from choosing a
//! platform or rendering the answer.
//!
//! Moved here from `nomos-cli`'s own `agent/dispatch.rs` and `agent/judge_role.rs`, the
//! same migration `P40-CORRECTIONS-CANONICAL-SEAM` made for `correct.rs`: that module's
//! own former `Execute_Goal` (assemble a bare envelope, dispatch it) and `Judge_Role`'s own
//! `Judgment_Task` (assemble an envelope naming a rule's own finding, dispatch it through
//! the identical primitive) are [`Run_Agent_Execute`] and [`Run_Agent_Judgment`] now, and
//! `nomos-cli`'s own `agent` module is a thin renderer over both.
//!
//! # The launcher left, because the port arrived
//!
//! Every function here used to be generic over [`nomos_platform::ProgramLauncher`] and to
//! take an `AgentEnvironment` carrying one, because [`Dispatched_Task`] called
//! `nomos_agent_executor_claude_code::Execute_Task` and
//! `nomos_model_backend_ollama::Execute_Task` directly and those need a launcher. Neither is
//! named here since `OD-ROADMAP-005` decision 2: a declared target arrives carrying a port,
//! the adapter behind that port already holds whatever platform a composition root bound into
//! it, and dispatching an agent task is no longer a thing this crate needs a platform to do.
//! `AgentEnvironment` is gone with the parameter, since its one field was that launcher.
//!
//! What is *not* given up is the substitutability the launcher parameter bought. Before, a
//! test could substitute a scripted launcher under a real adapter; now it substitutes the port
//! itself, which is a seam one layer further out and reaches the same two arms of the match
//! below. `test_support` says what that gives up and where the real adapters are still
//! exercised.

use std::path::Path;

use crate::{AgentDispatchOutcome, BackendSelection, Selected_Dispatch};
use nomos_agent_contracts::{DispatchPort, TaskEnvelope};
use nomos_contracts::{Finding, SchemaId};
use nomos_model_package::EffortLevel;
use nomos_rules::RoleSurfacePair;
use nomos_scope_verification::Territory;

/// Assembles a bare `TaskEnvelope` naming only `goal` and the resolved effort, and dispatches
/// it through whatever `selection` resolves to.
///
/// Renders no `nomos_agent_contracts::WorkResult` of its own -- `OD-CONTRACTS-003` made
/// `WorkResult.plan` representable as absent, but a bare `goal` carries no `RuleId` or
/// `SubjectId` to give a `Finding` either, since nothing dispatched it as a rule's judgment;
/// it is a person, asking a question directly.
#[must_use]
pub fn Run_Agent_Execute(goal: &str, selection: &BackendSelection<'_>) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };
    let task = Bare_Task(goal, config.effort);

    return Dispatched_Task(&task, config.family, config.port);
}

/// A bare `TaskEnvelope` naming only `goal` and `effort`. `scope`, `prohibited_changes`
/// and `available_tools` are the empty value `OD-EXECUTOR-001` already reads as "nothing
/// enumerated, nothing granted" -- a direct call has no configuration surface to fill them
/// from yet, and inventing one ahead of a real need would repeat a mistake this workspace
/// has already declined to make elsewhere. `expected_output_schema` names this call site
/// rather than a real schema, since nothing here validates a response against one. Named
/// `Bare_Task` rather than `Execute_Task`, the name this held in `nomos-cli`'s own
/// `dispatch.rs`, so it is never mistaken for either adapter's own, differently shaped,
/// public `Execute_Task`.
fn Bare_Task(goal: &str, effort: EffortLevel) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
}

/// Assembles the judgment `role_surface.rs`'s own module doc says
/// `Check_Declared_Role_Matches_Surface` cannot reach for itself -- whether `pair`'s
/// declared role and actual surface agree -- into a `TaskEnvelope` carrying `finding`'s
/// own summary, and dispatches it through whatever `selection` resolves to.
///
/// `pair` and `finding` are already built and already judged, the same "already walked"
/// contract `nomos_correction_orchestration::Run_Correction` holds for the source it is
/// handed: reading a crate's `README.md` row and its committed surface snapshot, and
/// running the rule over them, is a composition root's own file-reading concern
/// (`OD-HOST-002`), not this seam's.
#[must_use]
pub fn Run_Agent_Judgment(
    pair: &RoleSurfacePair, finding: &Finding, selection: &BackendSelection<'_>,
) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };
    let task = Judgment_Task(pair, finding, config.effort);

    return Dispatched_Task(&task, config.family, config.port);
}

/// The judgment `role_surface.rs`'s own module doc says this rule cannot reach itself --
/// whether `pair`'s declared role and actual surface agree -- carrying `finding.summary`
/// so the dispatched question is traceably the rule's own, not a paraphrase invented here.
fn Judgment_Task(pair: &RoleSurfacePair, finding: &Finding, effort: EffortLevel) -> TaskEnvelope
{
    let goal = format!(
        concat!(
            "A Rust crate's declared role, from its workspace README's band table: {}\n\n",
            "The crate's actual public surface, as a list of every item it exports:\n{}\n\n",
            "{}. Does the declared role accurately and completely describe what the surface ",
            "exports? Name anything the role claims that the surface does not show, or anything ",
            "the surface exports that the role does not mention, in 2-4 sentences.",
        ),
        pair.declared_role, pair.actual_surface, finding.summary
    );

    return TaskEnvelope {
        goal,
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: vec![finding.rule.clone()],
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.cli.v1"),
        effort,
    };
}

/// The root this seam has to resolve `prohibited_changes` against: none.
///
/// [`Bare_Task`] and [`Judgment_Task`] both name an empty `prohibited_changes`, so no path
/// resolves against this and the value is never read. It is not a stand-in for the
/// repository root, and an executor refuses rather than uses it should either task ever
/// declare a path to protect -- which is the point of naming it here rather than passing a
/// dot. Threading a real root is a change to this crate's own surface and to what a
/// composition root constructs.
const NO_ROOT: &str = "";

/// Dispatches a task a caller already built, through whatever `selection` resolves to.
///
/// The seam a workflow step reaches, and the reason it exists: before this,
/// `nomos-workflow-orchestration` matched its own `Body` variant straight to an adapter
/// crate, so the variant a step was written as *was* its backend choice and no resolution
/// happened anywhere. `OD-PACKAGE-016` decision 9's wiring is that a step declares what it
/// wants and the declared set decides what answers.
///
/// Distinct from [`Run_Agent_Execute`] only in where the task comes from. That one builds a
/// bare task from a goal, which is what a person at a command line has; this one takes a
/// whole [`TaskEnvelope`], which is what a workflow step carries. Both resolve the same way,
/// through the same [`Selected_Dispatch`], so neither can reach a target the other could
/// not.
#[must_use]
pub fn Run_Agent_Task(task: &TaskEnvelope, selection: &BackendSelection<'_>) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };

    return Dispatched_Task(task, config.family, config.port);
}

/// Runs `task` through `port` and reports what it answered, or why it answered nothing.
///
/// The two arms are the two `PackageKind`s, not two vendors, which is the whole of what
/// `OD-ROADMAP-005` decision 2 changed here: this function used to match a `Backend` enum
/// onto `nomos_agent_executor_claude_code::Execute_Task` and
/// `nomos_model_backend_ollama::Execute_Task`, so the generic path named both adapters and
/// carried their own outcome types. It names neither now, and the two answers stay apart
/// because the two ports return different types -- an executor's execution carries a spend and
/// a denial list, a model backend's answer carries a response, and neither is the other with
/// fields left empty.
fn Dispatched_Task(task: &TaskEnvelope, family: &str, port: DispatchPort<'_>) -> AgentDispatchOutcome
{
    return match port
    {
        DispatchPort::Executor(executor) => match executor.Execute(task, Path::new(NO_ROOT))
        {
            Ok(execution) => AgentDispatchOutcome::Executed { family: family.to_owned(), execution },
            Err(refusal) => AgentDispatchOutcome::Unavailable { family: family.to_owned(), reason: refusal.to_string() },
        },
        DispatchPort::Model(model) => match model.Answer(task)
        {
            Ok(answer) => AgentDispatchOutcome::Answered { family: family.to_owned(), answer },
            Err(refusal) => AgentDispatchOutcome::Unavailable { family: family.to_owned(), reason: refusal.to_string() },
        },
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{
        Declared_Executor, Declared_Model, RefusingExecutor, RefusingModel, ScriptedExecutor, ScriptedModel,
    };
    use crate::ProfileAbsence;
    use nomos_agent_contracts::DeclaredTarget;
    use nomos_model_package::{ModelExecutionProfile, ModelSelector};

    const EXECUTOR_FAMILY: &str = "acme-agent";
    const MODEL_FAMILY: &str = "acme-model";

    #[test]
    fn Test_Run_Agent_Execute_Should_Reach_An_Executor_Port()
    {
        let executor = ScriptedExecutor { assumption: "PONG".to_owned() };
        let declared = [Declared_Executor(EXECUTOR_FAMILY, &executor)];
        let profile = Family_Profile(EXECUTOR_FAMILY);

        let outcome = Run_Agent_Execute("say PONG", &Selection(&profile, None, &declared));

        match outcome
        {
            AgentDispatchOutcome::Executed { family, execution } =>
            {
                assert_eq!(family, EXECUTOR_FAMILY, "the outcome names the family that answered");
                assert_eq!(execution.result.assumptions, ["PONG".to_owned()]);
            }
            other => panic!("expected an execution, got {other:?}"),
        }
    }

    #[test]
    fn Test_Run_Agent_Execute_Should_Reach_A_Model_Backend_Port()
    {
        let model = ScriptedModel { response: "PONG".to_owned() };
        let declared = [Declared_Model(MODEL_FAMILY, &model)];
        let profile = Family_Profile(MODEL_FAMILY);

        let outcome = Run_Agent_Execute("say PONG", &Selection(&profile, None, &declared));

        match outcome
        {
            AgentDispatchOutcome::Answered { family, answer } =>
            {
                assert_eq!(family, MODEL_FAMILY);
                assert_eq!(answer.response, "PONG");
            }
            other => panic!("expected an answer, got {other:?}"),
        }
    }

    /// Neither port can produce anything -- both refuse before any real process exists.
    /// Exercised for both, because `Unavailable` folds either port's refusal down to text and
    /// this proves the fold holds from both arms of [`Dispatched_Task`].
    #[test]
    fn Test_Run_Agent_Execute_Should_Report_Unavailable_When_Either_Port_Refuses()
    {
        let executor = RefusingExecutor;
        let model = RefusingModel;
        let declared = [Declared_Executor(EXECUTOR_FAMILY, &executor), Declared_Model(MODEL_FAMILY, &model)];

        for family in [EXECUTOR_FAMILY, MODEL_FAMILY]
        {
            let profile = Family_Profile(family);

            let outcome = Run_Agent_Execute("say PONG", &Selection(&profile, None, &declared));

            match outcome
            {
                AgentDispatchOutcome::Unavailable { family: named, reason } =>
                {
                    assert_eq!(named, family, "the refusal names which target was reached");
                    assert_eq!(reason, "no such program");
                }
                other => panic!("{family}: expected Unavailable, got {other:?}"),
            }
        }
    }

    #[test]
    fn Test_Run_Agent_Judgment_Should_Reach_An_Executor_Port()
    {
        let executor = ScriptedExecutor { assumption: "PONG".to_owned() };
        let declared = [Declared_Executor(EXECUTOR_FAMILY, &executor)];
        let profile = Family_Profile(EXECUTOR_FAMILY);
        let pair = Fixture_Pair();
        let finding = Fixture_Finding();

        let outcome = Run_Agent_Judgment(&pair, &finding, &Selection(&profile, None, &declared));

        match outcome
        {
            AgentDispatchOutcome::Executed { execution, .. } =>
            {
                assert_eq!(execution.result.assumptions, ["PONG".to_owned()]);
            }
            other => panic!("expected an execution, got {other:?}"),
        }
    }

    /// A whole envelope a caller already built reaches the same two arms, which is what a
    /// workflow step carries.
    #[test]
    fn Test_Run_Agent_Task_Should_Dispatch_An_Envelope_A_Caller_Already_Built()
    {
        let model = ScriptedModel { response: "carried".to_owned() };
        let declared = [Declared_Model(MODEL_FAMILY, &model)];
        let profile = Family_Profile(MODEL_FAMILY);
        let task = Bare_Task("carried", EffortLevel::Low);

        let outcome = Run_Agent_Task(&task, &Selection(&profile, None, &declared));

        assert!(matches!(outcome, AgentDispatchOutcome::Answered { ref answer, .. } if answer.response == "carried"), "{outcome:?}");
    }

    /// The dispatched goal is traceably the rule's own question, not a paraphrase invented
    /// by this seam -- [`Judgment_Task`]'s own doc names this directly.
    #[test]
    fn Test_Judgment_Task_Should_Carry_The_Pair_And_The_Findings_Own_Summary()
    {
        let pair = Fixture_Pair();
        let finding = Fixture_Finding();

        let task = Judgment_Task(&pair, &finding, EffortLevel::High);

        assert!(task.goal.contains(&pair.declared_role), "{}", task.goal);
        assert!(task.goal.contains(&pair.actual_surface), "{}", task.goal);
        assert!(task.goal.contains(&finding.summary), "{}", task.goal);
        assert_eq!(task.applicable_rules, vec![finding.rule.clone()]);
        assert_eq!(task.effort, EffortLevel::High);
    }

    #[test]
    fn Test_Bare_Task_Should_Carry_Only_The_Goal_And_Effort()
    {
        let task = Bare_Task("say hello", EffortLevel::Minimal);

        assert_eq!(task.goal, "say hello");
        assert_eq!(task.effort, EffortLevel::Minimal);
        assert!(task.applicable_rules.is_empty());
        assert!(task.available_tools.is_empty());
    }

    /// The clause the whole item turns on: with nothing named, a dispatch still reaches a
    /// target, and it reaches it because the profile resolved rather than because a default
    /// was written down.
    #[test]
    fn Test_A_Profile_With_No_Preference_Should_Still_Reach_A_Target()
    {
        let model = ScriptedModel { response: "resolved".to_owned() };
        let declared = [Declared_Model(MODEL_FAMILY, &model)];
        let profile = Family_Profile(MODEL_FAMILY);

        let config = Selected_Dispatch(&Selection(&profile, None, &declared)).expect("the declared set offers the family");

        assert_eq!(config.family, MODEL_FAMILY);
    }

    /// A family nothing declares does not quietly become a target. This is the falsifier for
    /// the clause above: if resolution were bypassed in favour of any default, this would
    /// return that default instead of refusing.
    #[test]
    fn Test_An_Undeclared_Family_Should_Reach_No_Target()
    {
        let executor = RefusingExecutor;
        let declared = [Declared_Executor(EXECUTOR_FAMILY, &executor)];
        let profile = Family_Profile("no-such-family");

        let absence = Absence_Of(&Selection(&profile, None, &declared));

        assert!(
            matches!(absence, crate::BackendAbsence::Unresolved { absence: ProfileAbsence::NoDeclaredTargetOfThatFamily, .. }),
            "{absence:?}"
        );
    }

    /// What a preference naming a target the declared set cannot offer returns.
    ///
    /// It fails rather than falling back. `nomos_capability::Selection::Over` does fall back
    /// for a provider preference, and doing that here would run a different backend than the
    /// one a person typed after `--executor`.
    #[test]
    fn Test_A_Preference_The_Declared_Set_Cannot_Offer_Should_Fail_Rather_Than_Fall_Back()
    {
        let model = RefusingModel;
        let declared = [Declared_Model(MODEL_FAMILY, &model)];
        let profile = Family_Profile(MODEL_FAMILY);

        let absence = Absence_Of(&Selection(&profile, Some(EXECUTOR_FAMILY), &declared));

        assert_eq!(absence, crate::BackendAbsence::PreferenceNotDeclared { preferred: EXECUTOR_FAMILY.to_owned() });
    }

    /// A preference the set does offer is honoured ahead of what the profile would have
    /// resolved to on its own.
    #[test]
    fn Test_A_Declared_Preference_Should_Be_Honoured_Ahead_Of_The_Resolution()
    {
        let executor = RefusingExecutor;
        let model = RefusingModel;
        let declared = [Declared_Executor(EXECUTOR_FAMILY, &executor), Declared_Model(MODEL_FAMILY, &model)];
        let profile = Family_Profile(MODEL_FAMILY);

        let config = Selected_Dispatch(&Selection(&profile, Some(EXECUTOR_FAMILY), &declared)).expect("the set offers it");

        assert_eq!(config.family, EXECUTOR_FAMILY);
    }

    /// The absence a selection reported, or a panic. Written as a match rather than as
    /// `expect_err`, because that method needs the success type to carry `Debug` and
    /// [`DispatchConfig`] deliberately does not -- a port is a trait object.
    fn Absence_Of(selection: &BackendSelection<'_>) -> crate::BackendAbsence
    {
        return match Selected_Dispatch(selection)
        {
            Err(absence) => absence,
            Ok(config) => panic!("expected an absence, and it resolved to {}", config.family),
        };
    }

    fn Family_Profile(family: &str) -> ModelExecutionProfile
    {
        return ModelExecutionProfile::New(
            ModelSelector::BackendFamily(family.to_owned()),
            EffortLevel::BackendDefault,
        );
    }

    fn Selection<'port>(
        profile: &'port ModelExecutionProfile, preferred: Option<&'port str>,
        declared: &'port [DeclaredTarget<'port>],
    ) -> BackendSelection<'port>
    {
        return BackendSelection { profile, preferred, declared };
    }

    fn Fixture_Pair() -> RoleSurfacePair
    {
        return RoleSurfacePair {
            crate_root: "crates/example/nomos-example".to_owned(),
            crate_name: "nomos-example".to_owned(),
            declared_role: "An example crate.".to_owned(),
            actual_surface: "pub fn Something();\n".to_owned(),
        };
    }

    /// The real `Finding` `nomos_rules::Check_Declared_Role_Matches_Surface` produces for
    /// [`Fixture_Pair`] -- built through the real rule rather than hand-guessed, the same
    /// "already judged" input [`Run_Agent_Judgment`] itself expects from a caller.
    fn Fixture_Finding() -> Finding
    {
        let pair = Fixture_Pair();
        let mut findings = nomos_rules::Check_Declared_Role_Matches_Surface(std::slice::from_ref(&pair));

        return findings.pop().expect("the rule reports exactly one finding per subject");
    }
}
