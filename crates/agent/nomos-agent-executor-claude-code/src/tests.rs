//! What this crate still decides, now that the dispatch is the engine's.
//!
//! # Why the invocation's own shape is no longer asserted here
//!
//! It is not this crate's any more. That an allow-list grants nothing, that a
//! permission bypass never appears, that a goal reaches the command line as one
//! line, that each effort maps to a documented value — all of it moved down with
//! the dispatch and is asserted in `xvpe-agent-backend-claude-code`'s own suite.
//! Keeping a second copy here would be two suites drifting apart over one
//! behaviour, and the one further from the code would be the one that lied.
//!
//! What remains is the half that is genuinely this workspace's: the envelope
//! going in, and the work result coming out.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use std::cell::RefCell;

use nomos_contracts::SchemaId;
use nomos_ledger::Territory;
use nomos_platform::{Command, ExitOutcome, ProcessOutput};
use xvpe_agent_execution::{AgentWorkspace, ToolGrant};

use super::*;

/// A well-formed response, carrying exactly what the schema permits.
const A_VALID_RESPONSE: &str = r#"{"result":"done","structured_output":{"assumptions":["a ping wants a pong"],"unresolved_questions":[]},"is_error":false,"total_cost_usd":0.01,"duration_ms":500,"permission_denials":[]}"#;

/// The assumption that response carries.
const THE_ASSUMPTION: &str = "a ping wants a pong";

/// The goal these cases dispatch.
const A_GOAL: &str = "say PONG";

/// The envelope's goal must survive into the engine's task.
const GOAL_MUST_SURVIVE: &str = "the envelope's goal reaches the engine's task";
/// And the schema must travel with it, or the answer is not routable.
const SCHEMA_MUST_TRAVEL: &str = "the schema this crate requires travels with the task";
/// The boundary must be the tightest one, every time.
const BOUNDARY_MUST_BE_TIGHT: &str =
    "this crate dispatches into an empty directory with nothing granted";
/// A spend ceiling must always be set.
const CEILING_MUST_BE_SET: &str = "a dispatch is always bounded by what it may spend";
/// The default effort asks for nothing rather than naming a default.
const DEFAULT_ASKS_NOTHING: &str = "the backend default omits the request entirely";
/// Minimal has no counterpart and is approximated, deliberately.
const MINIMAL_IS_APPROXIMATED: &str = "minimal maps to low, this crate's own approximation";
/// A validated answer becomes a work result.
const ANSWER_BECOMES_A_RESULT: &str = "a validated answer builds the work result";
/// A work result claims only what this dispatch can ground.
const RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED: &str =
    "plan, claims, tests and requested verification stay structurally absent";
/// An answer that never validated is refused rather than coerced.
const MALFORMED_IS_REFUSED: &str = "an answer that is not the promised document is refused";
/// A caller-chosen directory must actually be used.
const CHOSEN_DIRECTORY_IS_USED: &str = "a caller-chosen directory is the one dispatched into";

/// A launcher that answers from a script and remembers what it was asked.
struct Scripted
{
    stdout: String,
    seen: RefCell<Vec<Command>>,
}

impl Scripted
{
    fn Saying(stdout: &str) -> Self
    {
        return Self { stdout: stdout.to_owned(), seen: RefCell::new(Vec::new()) };
    }
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Scripted
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProcessLauncher for Scripted
{
    fn Run(&self, command: &Command) -> Result<ProcessOutput, String>
    {
        self.seen.borrow_mut().push(command.clone());
        return Ok(ProcessOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: self.stdout.clone(),
            stderr: String::new(),
        });
    }
}

/// An envelope carrying nothing but a goal and an effort.
fn Bare_Task(goal: &str, effort: EffortLevel) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.agent.executor.v1"),
        effort,
    };
}

#[test]
fn Test_The_Envelope_Should_Reach_The_Engine_As_A_Task()
{
    let envelope = Bare_Task(A_GOAL, EffortLevel::High);

    let task = Task_For(&envelope);

    assert_eq!(task.goal, A_GOAL, "{GOAL_MUST_SURVIVE}");
    assert_eq!(task.answer_schema.as_deref(), Some(JSON_SCHEMA), "{SCHEMA_MUST_TRAVEL}");
}

#[test]
fn Test_The_Boundary_Should_Be_An_Empty_Directory_With_Nothing_Granted()
{
    let capability = Capability();

    assert_eq!(capability.workspace, AgentWorkspace::Isolated, "{BOUNDARY_MUST_BE_TIGHT}");
    assert_eq!(capability.tools, ToolGrant::Nothing, "{BOUNDARY_MUST_BE_TIGHT}");
    assert!(capability.spend_ceiling.is_some(), "{CEILING_MUST_BE_SET}");
}

#[test]
fn Test_The_Backend_Default_Should_Ask_For_No_Effort_At_All()
{
    // Naming a value meaning "the default" would be a request this crate cannot
    // honestly make on the caller's behalf.
    assert_eq!(Effort_For(EffortLevel::BackendDefault), None, "{DEFAULT_ASKS_NOTHING}");
}

#[test]
fn Test_Minimal_Should_Map_To_Low_As_An_Approximation()
{
    // There is no counterpart below low, so this is an approximation stated
    // rather than an exact match claimed.
    assert_eq!(
        Effort_For(EffortLevel::Minimal),
        Effort_For(EffortLevel::Low),
        "{MINIMAL_IS_APPROXIMATED}"
    );
}

#[test]
fn Test_A_Validated_Answer_Should_Build_The_Work_Result()
{
    let launcher = Scripted::Saying(A_VALID_RESPONSE);

    let outcome = Execute_Task(&Bare_Task(A_GOAL, EffortLevel::BackendDefault), &launcher)
        .expect(ANSWER_BECOMES_A_RESULT);

    assert_eq!(
        outcome.result.assumptions,
        [THE_ASSUMPTION.to_owned()],
        "{ANSWER_BECOMES_A_RESULT}"
    );
    // Nothing here saw a real file or computed a real digest, so nothing here
    // has an honest grounding for the other four.
    assert!(outcome.result.plan.is_none(), "{RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED}");
    assert!(outcome.result.claims.is_empty(), "{RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED}");
    assert!(outcome.result.tests.is_empty(), "{RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED}");
    assert!(
        outcome.result.requested_verification.is_none(),
        "{RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED}"
    );
}

#[test]
fn Test_An_Answer_That_Never_Validated_Should_Be_Refused()
{
    let launcher = Scripted::Saying(
        r#"{"result":"done","is_error":false,"total_cost_usd":0.01,"duration_ms":5}"#,
    );

    let failure = Execute_Task(&Bare_Task(A_GOAL, EffortLevel::BackendDefault), &launcher);

    assert!(matches!(failure, Err(AgentExecutionError::Unparseable(_))), "{MALFORMED_IS_REFUSED}");
}

#[test]
fn Test_A_Caller_Chosen_Directory_Should_Be_The_One_Dispatched_Into()
{
    let launcher = Scripted::Saying(A_VALID_RESPONSE);
    let directory = std::env::temp_dir();

    let outcome =
        Execute_In(&Bare_Task(A_GOAL, EffortLevel::BackendDefault), &launcher, &directory)
            .expect(CHOSEN_DIRECTORY_IS_USED);

    assert_eq!(
        outcome.result.assumptions,
        [THE_ASSUMPTION.to_owned()],
        "{CHOSEN_DIRECTORY_IS_USED}"
    );
    let seen = launcher.seen.borrow();
    let seen = seen.first().expect(CHOSEN_DIRECTORY_IS_USED);
    assert_eq!(
        seen.working_directory.as_deref(),
        Some(directory.as_path()),
        "{CHOSEN_DIRECTORY_IS_USED}"
    );
}
