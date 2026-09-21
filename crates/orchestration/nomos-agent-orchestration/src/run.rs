//! Assembling a `TaskEnvelope` for one of this workspace's two real dispatch shapes and
//! dispatching it to a chosen [`Backend`], apart from choosing a platform or rendering the
//! answer.
//!
//! Moved here from `nomos-cli`'s own `agent/dispatch.rs` and `agent/judge_role.rs`, the
//! same migration `P40-CORRECTIONS-CANONICAL-SEAM` made for `correct.rs`: that module's
//! own former `Execute_Goal` (assemble a bare envelope, dispatch it) and `Judge_Role`'s own
//! `Judgment_Task` (assemble an envelope naming a rule's own finding, dispatch it through
//! the identical primitive) are [`Run_Agent_Execute`] and [`Run_Agent_Judgment`] now, and
//! `nomos-cli`'s own `agent` module is a thin renderer over both.
//!
//! # A deliberate change from what `dispatch.rs`'s own comment used to defend
//!
//! The former `Dispatch_Task` this crate's [`Dispatched_Task`] replaces was fixed to
//! `nomos_platform_std::StdProgramLauncher` rather than generic, and that file's own doc
//! comment defended the choice as "the same composition-root choice `check.rs` and
//! `work.rs` make for their own subprocesses" -- correct advice for a CLI-only module with
//! exactly one caller. It stopped being correct the moment this dispatch needed a second
//! caller: [`Run_Agent_Execute`] and [`Run_Agent_Judgment`] are generic over
//! [`nomos_platform::ProgramLauncher`], the identical reason
//! `nomos_correction_orchestration::Run_Correction` already is, so `nomos-api` can supply
//! its own `StdProgramLauncher` at its own call site instead of depending on `nomos-cli`'s
//! choice, or on `nomos-cli` at all.
//!
//! Genericizing also retires a gap `dispatch.rs`'s own `// check-test-coverage:
//! allow-untested` comments named as unavoidable there: both match arms were untestable in
//! that file only because they were fixed to a real launcher, so exercising them meant
//! either depending on which binaries happened to be on the running machine's `PATH` or
//! risking a real, costly invocation. Now that [`Dispatched_Task`] is generic, this crate's own
//! tests reach both arms with a scripted [`nomos_platform::ProgramLauncher`] instead -- the
//! identical fake-launcher shape `nomos_agent_executor_claude_code`'s and
//! `nomos_model_backend_ollama`'s own `address_tests` already use for the identical reason
//! -- so no exclusion marker survives the move. What is still never invoked in a test is
//! the real `claude`/`ollama` subprocess either backend crate's own `Execute_Task` spawns
//! through whichever launcher a caller supplies; only the substitutable launcher parameter
//! is.

use std::path::Path;

use crate::{AgentDispatchOutcome, AgentEnvironment, Backend, BackendSelection, Selected_Dispatch};
use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{Finding, SchemaId};
use nomos_model_package::EffortLevel;
use nomos_platform::ProgramLauncher;
use nomos_rules::RoleSurfacePair;
use nomos_scope_verification::Territory;

/// Assembles a bare `TaskEnvelope` naming only `goal` and `config.effort`, and dispatches
/// it to `config.backend`.
///
/// `execute` is the only real caller either backend crate has anywhere in this workspace
/// today, other than their own tests and `nomos_workflow_orchestration`'s own `Body::
/// ClaudeCode`/`Body::Ollama` dispatch. It renders each backend's own outcome type
/// directly rather than assembling a `nomos_agent_contracts::WorkResult` --
/// `OD-CONTRACTS-003` made `WorkResult.plan` representable as absent, but a bare `goal`
/// carries no `RuleId` or `SubjectId` to give a `Finding` either, since nothing dispatched
/// it as a rule's judgment; it is a person, asking a question directly.
#[must_use]
pub fn Run_Agent_Execute<Launcher: ProgramLauncher>(
    goal: &str, selection: &BackendSelection<'_>, environment: &AgentEnvironment<'_, Launcher>,
) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };
    let task = Bare_Task(goal, config.effort);

    return Dispatched_Task(&task, config.backend, environment);
}

/// A bare `TaskEnvelope` naming only `goal` and `effort`. `scope`, `prohibited_changes`
/// and `available_tools` are the empty value `OD-EXECUTOR-001` already reads as "nothing
/// enumerated, nothing granted" -- a direct call has no configuration surface to fill them
/// from yet, and inventing one ahead of a real need would repeat a mistake this workspace
/// has already declined to make elsewhere. `expected_output_schema` names this call site
/// rather than a real schema, since nothing here validates a response against one. Named
/// `Bare_Task` rather than `Execute_Task`, the name this held in `nomos-cli`'s own
/// `dispatch.rs`, so it is never mistaken for either backend crate's own, differently
/// shaped, public `Execute_Task`.
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
/// own summary, and dispatches it to `config.backend`.
///
/// `pair` and `finding` are already built and already judged, the same "already walked"
/// contract `nomos_correction_orchestration::Run_Correction` holds for the source it is
/// handed: reading a crate's `README.md` row and its committed surface snapshot, and
/// running the rule over them, is a composition root's own file-reading concern
/// (`OD-HOST-002`), not this seam's.
#[must_use]
pub fn Run_Agent_Judgment<Launcher: ProgramLauncher>(
    pair: &RoleSurfacePair, finding: &Finding, selection: &BackendSelection<'_>,
    environment: &AgentEnvironment<'_, Launcher>,
) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };
    let task = Judgment_Task(pair, finding, config.effort);

    return Dispatched_Task(&task, config.backend, environment);
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
/// repository root, and the executor refuses rather than uses it should either task ever
/// declare a path to protect -- which is the point of naming it here rather than passing a
/// dot. Threading a real root is `AgentEnvironment`'s own change, and reaches this crate's
/// surface snapshot and `nomos-cli`'s construction of it.
const NO_ROOT: &str = "";

/// Runs `task` against `backend` and reports whichever of the two outcome shapes it
/// produces, or why neither could answer. The two crates share no trait --
/// `OD-EXECUTOR-001`/`OD-EXECUTOR-004` both decline to invent one ahead of a real need, and
/// `OD-EXECUTOR-005` found that trigger has not fired even once `--executor`/
/// `--model-backend` replaced `--backend`: there is still only one real `AgentExecutor`, so
/// this match is the entire dispatch, not a stand-in for a trait either flag's own
/// vocabulary would need.
/// Dispatches a task a caller already built, to whatever `selection` resolves to.
///
/// The seam a workflow step reaches, and the reason it exists: before this,
/// `nomos-workflow-orchestration` matched its own `Body` variant straight to an executor
/// crate, so the variant a step was written as *was* its backend choice and no resolution
/// happened anywhere. `OD-PACKAGE-016` decision 9's wiring is that a step declares what it
/// wants and the declared set decides what answers.
///
/// Distinct from [`Run_Agent_Execute`] only in where the task comes from. That one builds a
/// bare task from a goal, which is what a person at a command line has; this one takes a
/// whole [`TaskEnvelope`], which is what a workflow step carries. Both resolve the same way,
/// through the same [`Selected_Dispatch`], so neither can reach a backend the other could
/// not.
#[must_use]
pub fn Run_Agent_Task<Launcher: ProgramLauncher>(
    task: &TaskEnvelope, selection: &BackendSelection<'_>, environment: &AgentEnvironment<'_, Launcher>,
) -> AgentDispatchOutcome
{
    let config = match Selected_Dispatch(selection)
    {
        Ok(config) => config,
        Err(absence) => return AgentDispatchOutcome::NotSelected(absence),
    };

    return Dispatched_Task(task, config.backend, environment);
}

fn Dispatched_Task<Launcher: ProgramLauncher>(task: &TaskEnvelope, backend: Backend, environment: &AgentEnvironment<'_, Launcher>) -> AgentDispatchOutcome
{
    return match backend
    {
        Backend::ClaudeCode => match nomos_agent_executor_claude_code::Execute_Task(task, environment.launcher, Path::new(NO_ROOT))
        {
            Ok(outcome) => AgentDispatchOutcome::ClaudeCode(outcome),
            Err(error) => AgentDispatchOutcome::Unavailable(error.to_string()),
        },
        Backend::Ollama => match nomos_model_backend_ollama::Execute_Task(task, environment.launcher)
        {
            Ok(outcome) => AgentDispatchOutcome::Ollama(outcome),
            Err(error) => AgentDispatchOutcome::Unavailable(error.to_string()),
        },
    };
}

#[cfg(test)]
mod tests
{
    use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
    use super::*;
    use crate::ProfileAbsence;
    use nomos_model_package::{ModelExecutionProfile, ModelSelector};
    use nomos_platform::{Command, ExitOutcome, ProgramOutput};

    struct Scripted
    {
        outcome: ExitOutcome,
        stdout: String,
    }

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for Scripted
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for Scripted
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Ok(ProgramOutput { outcome: self.outcome, stdout: self.stdout.clone(), stderr: String::new() });
        }
    }

    struct Unreachable;

    /// Answers from fixed data, so its outputs reproduce byte for byte.
    impl Strategy for Unreachable
    {
        const STRENGTH: DeterminismStrength = DeterminismStrength::State;
        const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
        const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
    }

    impl ProgramLauncher for Unreachable
    {
        fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
        {
            return Err("no such program".to_owned());
        }
    }

    #[test]
    fn Test_Run_Agent_Execute_Should_Reach_Claude_Code_With_A_Scripted_Launcher()
    {
        let launcher = Scripted { outcome: ExitOutcome::Exited { code: 0 }, stdout: Claude_Code_Success_Json() };

        let outcome = Run_Agent_Execute("say PONG", &Selecting::Preferring(Backend::ClaudeCode).Selection(), &AgentEnvironment { launcher: &launcher });

        match outcome
        {
            AgentDispatchOutcome::ClaudeCode(outcome) => assert_eq!(outcome.result.assumptions, ["PONG".to_owned()]),
            other => panic!("expected ClaudeCode, got {other:?}"),
        }
    }

    #[test]
    fn Test_Run_Agent_Execute_Should_Reach_Ollama_With_A_Scripted_Launcher()
    {
        let launcher = Scripted { outcome: ExitOutcome::Exited { code: 0 }, stdout: "PONG\n".to_owned() };

        let outcome = Run_Agent_Execute("say PONG", &Selecting::Preferring(Backend::Ollama).Selection(), &AgentEnvironment { launcher: &launcher });

        match outcome
        {
            AgentDispatchOutcome::Ollama(outcome) => assert_eq!(outcome.response, "PONG"),
            other => panic!("expected Ollama, got {other:?}"),
        }
    }

    /// Neither backend can be started at all -- the launcher itself refuses, never a real
    /// subprocess. Exercised for both backends: `Unavailable` folds both crates' own error
    /// type down to text, and this proves the fold holds from either arm of [`Dispatched_Task`].
    #[test]
    fn Test_Run_Agent_Execute_Should_Report_Unavailable_When_The_Launcher_Cannot_Start_Either_Backend()
    {
        for backend in [Backend::ClaudeCode, Backend::Ollama]
        {
            let outcome = Run_Agent_Execute("say PONG", &Selecting::Preferring(backend).Selection(), &AgentEnvironment { launcher: &Unreachable });

            assert!(matches!(outcome, AgentDispatchOutcome::Unavailable(_)), "{backend:?}: {outcome:?}");
        }
    }

    #[test]
    fn Test_Run_Agent_Judgment_Should_Reach_Claude_Code_With_A_Scripted_Launcher()
    {
        let launcher = Scripted { outcome: ExitOutcome::Exited { code: 0 }, stdout: Claude_Code_Success_Json() };
        let pair = Fixture_Pair();
        let finding = Fixture_Finding();

        let outcome = Run_Agent_Judgment(&pair, &finding, &Selecting::Preferring(Backend::ClaudeCode).Selection(), &AgentEnvironment { launcher: &launcher });

        match outcome
        {
            AgentDispatchOutcome::ClaudeCode(outcome) => assert_eq!(outcome.result.assumptions, ["PONG".to_owned()]),
            other => panic!("expected ClaudeCode, got {other:?}"),
        }
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
    /// backend, and it reaches it because the profile resolved rather than because a default
    /// was written down. Before this, `nomos_cli::agent::parsing` returned
    /// `Backend::ClaudeCode` when neither flag was given, which is a backend nothing chose.
    #[test]
    fn Test_A_Profile_With_No_Preference_Should_Still_Reach_A_Backend()
    {
        let selecting = Selecting::Resolving(Backend::Ollama.Label());

        let config = Selected_Dispatch(&selecting.Selection()).expect("the declared set offers the family");

        assert_eq!(config.backend, Backend::Ollama);
    }

    /// Every declared backend is reachable by its own family name with no preference, so the
    /// resolution is answering rather than one arm of it happening to be first.
    #[test]
    fn Test_Every_Declared_Backend_Should_Be_Reachable_By_Family_Alone()
    {
        for backend in Backend::ALL
        {
            let selecting = Selecting::Resolving(backend.Label());

            let config = Selected_Dispatch(&selecting.Selection()).expect("a declared family resolves");

            assert_eq!(config.backend, backend, "{}", backend.Label());
        }
    }

    /// A family nothing declares does not quietly become a backend. This is the falsifier for
    /// the clause above: if resolution were bypassed in favour of any default, this would
    /// return that default instead of refusing.
    #[test]
    fn Test_An_Undeclared_Family_Should_Reach_No_Backend()
    {
        let selecting = Selecting::Resolving("no-such-family");

        let absence = Selected_Dispatch(&selecting.Selection()).expect_err("nothing declares that family");

        assert!(
            matches!(absence, crate::BackendAbsence::Unresolved { absence: ProfileAbsence::NoDeclaredTargetOfThatFamily, .. }),
            "{absence:?}"
        );
    }

    /// What a preference naming a backend the declared set cannot offer returns, which the
    /// item required be reported rather than left to be discovered.
    ///
    /// It fails rather than falling back. `nomos_capability::Selection::Over` does fall back
    /// for a provider preference, and doing that here would run a different backend than the
    /// one a person typed after `--executor`.
    #[test]
    fn Test_A_Preference_The_Declared_Set_Cannot_Offer_Should_Fail_Rather_Than_Fall_Back()
    {
        let profile = ModelExecutionProfile::New(
            ModelSelector::BackendFamily(Backend::Ollama.Label().to_owned()),
            EffortLevel::BackendDefault,
        );
        let declared: Vec<crate::DeclaredTarget> = crate::Declared_Targets()
            .into_iter()
            .filter(|target| return target.backend != Backend::ClaudeCode)
            .collect();

        let absence = Selected_Dispatch(&BackendSelection {
            profile: &profile,
            preferred: Some(Backend::ClaudeCode.Label()),
            declared: &declared,
        })
        .expect_err("the set no longer offers the preferred backend");

        assert_eq!(absence, crate::BackendAbsence::PreferenceNotDeclared { preferred: Backend::ClaudeCode.Label().to_owned() });
    }

    fn Claude_Code_Success_Json() -> String
    {
        return r#"{"result": "PONG", "structured_output": {"assumptions": ["PONG"], "unresolved_questions": []}, "is_error": false, "total_cost_usd": 0.01, "duration_ms": 500, "permission_denials": []}"#.to_owned();
    }

    /// A request that reaches exactly `backend`, with the owned parts a
    /// [`BackendSelection`] borrows kept alive beside it.
    ///
    /// Names `backend` as the preference rather than writing a selector that happens to
    /// resolve to it, because these tests are about what the seam does once a backend is
    /// chosen. The resolution path is exercised by
    /// `Test_A_Profile_With_No_Preference_Should_Still_Reach_A_Backend`, which is the one
    /// that matters for the default a person never typed.
    struct Selecting
    {
        profile: ModelExecutionProfile,
        declared: Vec<crate::DeclaredTarget>,
        preferred: Option<String>,
    }

    impl Selecting
    {
        fn Preferring(backend: Backend) -> Self
        {
            return Self {
                profile: ModelExecutionProfile::New(
                    ModelSelector::BackendFamily(backend.Label().to_owned()),
                    EffortLevel::BackendDefault,
                ),
                declared: crate::Declared_Targets(),
                preferred: Some(backend.Label().to_owned()),
            };
        }

        fn Resolving(family: &str) -> Self
        {
            return Self {
                profile: ModelExecutionProfile::New(
                    ModelSelector::BackendFamily(family.to_owned()),
                    EffortLevel::BackendDefault,
                ),
                declared: crate::Declared_Targets(),
                preferred: None,
            };
        }

        fn Selection(&self) -> BackendSelection<'_>
        {
            return BackendSelection {
                profile: &self.profile,
                preferred: self.preferred.as_deref(),
                declared: &self.declared,
            };
        }
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
