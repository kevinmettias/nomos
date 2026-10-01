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
use std::path::PathBuf;

use nomos_contracts::{CapabilityId, RuleId, SchemaId};
use nomos_ledger::Territory;
use nomos_platform::{Command, ExitOutcome, ProgramOutput};
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
/// The declaration the result carries is the one this crate publishes.
const RESULT_CARRIES_ITS_DECLARATION: &str =
    "the result carries the substantiation declaration this crate publishes";
/// A substantiated portion is what makes its emptiness mean the task produced none.
const SUBSTANTIATED_MEANS_PRODUCED_NONE: &str =
    "a portion the dispatch did read is declared substantiated, so its emptiness means the model produced none";
/// An answer that never validated is refused rather than coerced.
const MALFORMED_IS_REFUSED: &str = "an answer that is not the promised document is refused";
/// A caller-chosen directory must actually be used.
const CHOSEN_DIRECTORY_IS_USED: &str = "a caller-chosen directory is the one dispatched into";
/// A prohibited path that changed must be a refusal, not a note inside an outcome.
const PROHIBITED_CHANGE_IS_REFUSED: &str =
    "a path prohibited_changes names and the dispatch changed is refused by name";
/// A prohibited path left alone must not be reported as changed.
const UNTOUCHED_IS_NOT_A_CHANGE: &str = "a prohibited path the dispatch left alone is not a change";
/// A tool grant this crate cannot make must be refused before anything runs.
const UNGRANTABLE_TOOLS_ARE_REFUSED: &str =
    "available_tools this crate cannot grant refuses instead of dispatching";
/// Paths to protect against a root that names no tree must be refused.
const UNDECIDABLE_ROOT_IS_REFUSED: &str =
    "prohibited paths against a relative root refuse rather than resolve against the process";
/// The rules the envelope declares must reach the model.
const RULES_REACH_THE_MODEL: &str = "applicable_rules are named in the goal the model is given";

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

impl ProgramLauncher for Scripted
{
    fn Run(&self, command: &Command) -> Result<ProgramOutput, String>
    {
        self.seen.borrow_mut().push(command.clone());
        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: self.stdout.clone(),
            stderr: String::new(),
        });
    }
}

/// A launcher that changes a file on disk before answering, the way a real
/// dispatch granted edits could.
struct Meddling
{
    writes_to: PathBuf,
}

/// Answers from fixed data, so its outputs reproduce byte for byte.
impl Strategy for Meddling
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl ProgramLauncher for Meddling
{
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        std::fs::write(&self.writes_to, b"changed by the dispatch").expect("the fixture file");
        return Ok(ProgramOutput {
            outcome: ExitOutcome::Exited { code: 0 },
            stdout: A_VALID_RESPONSE.to_owned(),
            stderr: String::new(),
        });
    }
}

/// A root for the cases that declare nothing to protect, so none resolves against it.
const NO_ROOT: &str = "";

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

    let envelope = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    let outcome = Execute_Task(&envelope, &launcher, Path::new(NO_ROOT))
        .expect("the scripted launcher above answers with the well-formed document this case is about, so its dispatch cannot fail");

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
    // The four assertions above are what `OD-EXECUTOR-011` found indistinguishable from a task
    // that produced none. What follows is the declaration that separates them, asserted in
    // both directions because either half alone is passable by a declaration that says nothing:
    // the portions that were read are declared substantiated, so their emptiness means the
    // model produced none, and the four that were never read are declared unsubstantiated, so
    // their emptiness cannot be misread as a task that produced none. The first assertion is
    // decision 4 -- the result carries the crate's own published declaration rather than an
    // equal-looking one built at this call site -- and it is checked for all six portions, so a
    // call site that diverged from `response.rs` in any single entry reddens here.
    assert_eq!(outcome.result.substantiation, WORK_RESULT_SUBSTANTIATION, "{RESULT_CARRIES_ITS_DECLARATION}");
    assert!(
        outcome.result.substantiation.assumptions.Is_Substantiated(),
        "{SUBSTANTIATED_MEANS_PRODUCED_NONE}"
    );
    assert!(
        !outcome.result.substantiation.plan.Is_Substantiated(),
        "{RESULT_CLAIMS_ONLY_WHAT_IS_GROUNDED}"
    );
}

#[test]
fn Test_An_Answer_That_Never_Validated_Should_Be_Refused()
{
    let launcher = Scripted::Saying(
        r#"{"result":"done","is_error":false,"total_cost_usd":0.01,"duration_ms":5}"#,
    );

    let envelope = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    let failure = Execute_Task(&envelope, &launcher, Path::new(NO_ROOT));

    assert!(matches!(failure, Err(AgentExecutionError::Unparseable(_))), "{MALFORMED_IS_REFUSED}");
}

#[test]
fn Test_A_Caller_Chosen_Directory_Should_Be_The_One_Dispatched_Into()
{
    let launcher = Scripted::Saying(A_VALID_RESPONSE);
    let directory = std::env::temp_dir();

    let envelope = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    let outcome = Execute_In(&envelope, &launcher, &directory, Path::new(NO_ROOT))
        .expect("the scripted launcher above answers with the well-formed document this case is about, so its dispatch cannot fail");

    assert_eq!(
        outcome.result.assumptions,
        [THE_ASSUMPTION.to_owned()],
        "{CHOSEN_DIRECTORY_IS_USED}"
    );
    let seen = launcher.seen.borrow();
    let seen = seen.first().expect("the dispatch above ran the launcher once, and a launcher that never ran would record no command");
    assert_eq!(
        seen.working_directory.as_deref(),
        Some(directory.as_path()),
        "{CHOSEN_DIRECTORY_IS_USED}"
    );
}

#[test]
fn Test_A_Prohibited_Path_The_Dispatch_Changed_Should_Be_Refused_By_Name()
{
    let root = std::env::temp_dir().join("nomos-prohibited-change-is-refused");
    std::fs::create_dir_all(&root).expect("the fixture directory");
    let protected = root.join("protected.txt");
    std::fs::write(&protected, b"as it was before").expect("the fixture file");

    let launcher = Meddling { writes_to: protected.clone() };
    let mut task = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    task.prohibited_changes = Territory::Of_Files(["protected.txt"]);

    let refusal = Execute_Task(&task, &launcher, &root);

    // By name, so a caller reads which path went rather than that something did.
    assert_eq!(
        refusal,
        Err(AgentExecutionError::ProhibitedChange("protected.txt".to_owned())),
        "{PROHIBITED_CHANGE_IS_REFUSED}"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn Test_A_Prohibited_Path_Left_Alone_Should_Not_Be_Reported_As_Changed()
{
    let root = std::env::temp_dir().join("nomos-prohibited-untouched-is-clean");
    std::fs::create_dir_all(&root).expect("the fixture directory");
    let protected = root.join("protected.txt");
    std::fs::write(&protected, b"as it was before").expect("the fixture file");

    // Writes somewhere else entirely, so the comparison has a real chance to be wrong.
    let launcher = Meddling { writes_to: root.join("untracked.txt") };
    let mut task = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    task.prohibited_changes = Territory::Of_Files(["protected.txt"]);

    let outcome = Execute_Task(&task, &launcher, &root)
        .expect("the meddling launcher above leaves the prohibited path alone and answers a valid document, so nothing here refuses");

    assert_eq!(
        outcome.result.assumptions,
        [THE_ASSUMPTION.to_owned()],
        "{UNTOUCHED_IS_NOT_A_CHANGE}"
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn Test_Declared_Tools_This_Crate_Cannot_Grant_Should_Refuse_Before_Dispatching()
{
    let launcher = Scripted::Saying(A_VALID_RESPONSE);
    let mut task = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    task.available_tools = vec![CapabilityId::New("nomos.cap.example.claude_code_refusal_test_only")];

    let refusal = Execute_Task(&task, &launcher, Path::new(NO_ROOT));

    assert_eq!(
        refusal,
        Err(AgentExecutionError::UnsupportedTools("nomos.cap.example.claude_code_refusal_test_only".to_owned())),
        "{UNGRANTABLE_TOOLS_ARE_REFUSED}"
    );
    // Before anything runs, not after: a refused grant must not have dispatched.
    assert!(launcher.seen.borrow().is_empty(), "{UNGRANTABLE_TOOLS_ARE_REFUSED}");
}

#[test]
fn Test_Paths_To_Protect_Against_A_Root_Naming_No_Tree_Should_Be_Refused()
{
    let launcher = Scripted::Saying(A_VALID_RESPONSE);
    let mut task = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    task.prohibited_changes = Territory::Of_Files(["protected.txt"]);

    let refusal = Execute_Task(&task, &launcher, Path::new("relative/root"));

    assert!(
        matches!(refusal, Err(AgentExecutionError::UnresolvableRoot(_))),
        "{UNDECIDABLE_ROOT_IS_REFUSED}"
    );
    assert!(launcher.seen.borrow().is_empty(), "{UNDECIDABLE_ROOT_IS_REFUSED}");
}

#[test]
fn Test_The_Declared_Rules_Should_Be_Named_In_The_Goal_The_Model_Is_Given()
{
    let mut envelope = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    envelope.applicable_rules =
        vec![RuleId::New("check-naming-convention"), RuleId::New("check-file-size")];

    let task = Task_For(&envelope);

    assert!(task.goal.contains(A_GOAL), "{RULES_REACH_THE_MODEL}");
    assert!(task.goal.contains("check-naming-convention"), "{RULES_REACH_THE_MODEL}");
    assert!(task.goal.contains("check-file-size"), "{RULES_REACH_THE_MODEL}");
}

#[test]
fn Test_An_Envelope_Declaring_No_Rules_Should_Reach_The_Model_Unchanged()
{
    // The common case today, and it must not grow a trailing clause about nothing.
    let envelope = Bare_Task(A_GOAL, EffortLevel::BackendDefault);
    let task = Task_For(&envelope);

    assert_eq!(task.goal, A_GOAL, "{GOAL_MUST_SURVIVE}");
}
