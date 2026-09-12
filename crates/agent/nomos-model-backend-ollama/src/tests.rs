//! What this crate still decides, now that the dispatch is the engine's.
//!
//! # Why the invocation's own shape is no longer asserted here
//!
//! It is not this crate's any more. That no flag opening a capability ever
//! reaches the invocation, that the goal passes through untouched, that a
//! boundary this backend cannot keep is refused — all of it moved down with the
//! dispatch and is asserted in `xvpe-agent-backend-ollama`'s own suite. A second
//! copy here would be two suites drifting over one behaviour.
//!
//! What remains is the envelope going in and the outcome coming out.

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use std::cell::RefCell;

use nomos_contracts::{CapabilityId, SchemaId};
use nomos_ledger::Territory;
use nomos_model_package::EffortLevel;
use nomos_platform::{Command, ExitOutcome, ProcessOutput};
use xvpe_agent_execution::{AgentWorkspace, ToolGrant};

use super::*;

/// What the scripted model says back.
const A_RESPONSE: &str = "the model's answer";
/// The goal these cases dispatch.
const A_GOAL: &str = "say hello";

/// The envelope's goal must survive into the engine's task, untouched.
const GOAL_MUST_SURVIVE: &str = "the goal reaches the engine's task exactly as written";
/// No schema is asked for, because a free-text answer would not conform to one.
const NO_SCHEMA_IS_ASKED: &str = "no schema is asked for, because the answer is free text";
/// No effort is asked for, because none maps here honestly.
const NO_EFFORT_IS_ASKED: &str = "no effort is asked for, because none maps here honestly";
/// The boundary must be the tightest one, with nothing granted.
const BOUNDARY_MUST_BE_TIGHT: &str =
    "this crate dispatches into an empty directory with nothing granted";
/// A ceiling would be a number that means nothing here.
const NO_CEILING_IS_SET: &str = "local inference has no metered charge to bound";
/// The response is what the model wrote.
const RESPONSE_IS_THE_OUTPUT: &str = "the outcome carries what the model wrote";
/// A caller-chosen directory must actually be used.
const CHOSEN_DIRECTORY_IS_USED: &str = "a caller-chosen directory is the one dispatched into";
/// A capability this backend cannot grant must refuse before anything runs.
const UNGRANTABLE_TOOLS_ARE_REFUSED: &str =
    "available_tools this backend cannot grant refuses instead of dispatching";

/// A launcher that answers from a script and remembers what it was asked.
struct Scripted
{
    seen: RefCell<Vec<Command>>,
}

impl Scripted
{
    fn New() -> Self
    {
        return Self { seen: RefCell::new(Vec::new()) };
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
            stdout: A_RESPONSE.to_owned(),
            stderr: String::new(),
        });
    }
}

/// An envelope carrying nothing but a goal.
fn Bare_Task(goal: &str) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.model.backend.v1"),
        // Read and ignored, deliberately: see this crate's own doc.
        effort: EffortLevel::High,
    };
}

#[test]
fn Test_The_Goal_Should_Reach_The_Engine_Untouched()
{
    let task = Task_For(&Bare_Task(A_GOAL));

    assert_eq!(task.goal, A_GOAL, "{GOAL_MUST_SURVIVE}");
    assert_eq!(task.answer_schema, None, "{NO_SCHEMA_IS_ASKED}");
    // Asking for an effort would mean claiming a mapping nobody has measured.
    assert_eq!(task.effort, None, "{NO_EFFORT_IS_ASKED}");
}

#[test]
fn Test_The_Boundary_Should_Be_An_Empty_Directory_With_Nothing_Granted()
{
    let capability = Capability();

    assert_eq!(capability.workspace, AgentWorkspace::Isolated, "{BOUNDARY_MUST_BE_TIGHT}");
    assert_eq!(capability.tools, ToolGrant::Nothing, "{BOUNDARY_MUST_BE_TIGHT}");
    assert_eq!(capability.spend_ceiling, None, "{NO_CEILING_IS_SET}");
}

#[test]
fn Test_The_Outcome_Should_Carry_What_The_Model_Wrote()
{
    let launcher = Scripted::New();

    let outcome = Execute_Task(&Bare_Task(A_GOAL), &launcher).expect(RESPONSE_IS_THE_OUTPUT);

    assert_eq!(outcome.response, A_RESPONSE, "{RESPONSE_IS_THE_OUTPUT}");
}

#[test]
fn Test_A_Caller_Chosen_Directory_Should_Be_The_One_Dispatched_Into()
{
    let launcher = Scripted::New();
    let directory = std::env::temp_dir();

    let outcome = Execute_In(&Bare_Task(A_GOAL), &launcher, &directory)
        .expect(CHOSEN_DIRECTORY_IS_USED);

    assert_eq!(outcome.response, A_RESPONSE, "{CHOSEN_DIRECTORY_IS_USED}");
    let seen = launcher.seen.borrow();
    let seen = seen.first().expect(CHOSEN_DIRECTORY_IS_USED);
    assert_eq!(
        seen.working_directory.as_deref(),
        Some(directory.as_path()),
        "{CHOSEN_DIRECTORY_IS_USED}"
    );
}

#[test]
fn Test_Declared_Tools_This_Backend_Cannot_Grant_Should_Refuse_Before_Dispatching()
{
    let launcher = Scripted::New();
    let mut task = Bare_Task(A_GOAL);
    task.available_tools = vec![CapabilityId::New("nomos.cap.example.ollama_refusal_test_only")];

    let refusal = Execute_Task(&task, &launcher);

    assert_eq!(
        refusal,
        Err(AgentExecutionError::UnsupportedTools("nomos.cap.example.ollama_refusal_test_only".to_owned())),
        "{UNGRANTABLE_TOOLS_ARE_REFUSED}"
    );
    // Before anything runs: a refused grant must not have reached the launcher.
    assert!(launcher.seen.borrow().is_empty(), "{UNGRANTABLE_TOOLS_ARE_REFUSED}");
}
