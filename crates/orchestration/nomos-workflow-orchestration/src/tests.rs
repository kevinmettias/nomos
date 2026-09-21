use std::cell::RefCell;
use std::collections::VecDeque;
use std::num::NonZeroU32;

use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{
    Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, RuleId, RunId,
    SchemaId, Strategy, Timeout, TraceEquivalence, WorkflowStep,
};
use nomos_gate_orchestration::{GateCommand, GateRunOutcome, RuleSelector};
use nomos_ledger::Territory;
use nomos_model_package::EffortLevel;
use nomos_platform::{Clock, Command, ExitOutcome, ProgramLauncher, ProgramOutput};
use nomos_platform_std::StdFileSystem;
use nomos_workspace::BuildVariant;

use crate::{Body, CheckBody, CommitIntent, CorrectionBody, DispatchError, GateBody, Platform, Run, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

/// The retry limit [`Incoherent_Step`] declares. The count itself is not what makes that step
/// incoherent -- `WF-012`'s failure is a retry with neither a deduplication token nor a
/// compensation -- so any value past the first attempt would do, and naming it keeps the
/// number out of the expression.
const MAX_ATTEMPTS: u32 = 3;

/// This process's own build variant is not what a workflow step should be judged as --
/// `nomos_check_orchestration::Run`'s own doc says `variant` must come from the
/// *compiling* binary read through `env!`, and a test binary is not the composition root
/// any real caller would be. A fixed, named test variant, the same shape
/// `nomos_check_orchestration`'s own fixtures already use.
fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// A fresh `RunId`, for a test that dispatches a `Body::Gate` step -- `nomos_platform_std::
/// SystemClock` is a real clock, not a fixture, but a `RunId`'s own identity is its
/// uniqueness, not any particular timestamp, so reading the real clock once here costs
/// this file nothing a fixed one would have bought.
fn Test_Run_Id() -> RunId
{
    return nomos_gate_orchestration::Fresh_Run_Id(nomos_platform_std::SystemClock.Now());
}

fn Task_Envelope(goal: &str) -> TaskEnvelope
{
    return TaskEnvelope {
        goal: goal.to_owned(),
        scope: Territory::Of_Files(Vec::<String>::new()),
        knowledge_context: Vec::new(),
        applicable_rules: Vec::new(),
        prohibited_changes: Territory::Of_Files(Vec::<String>::new()),
        available_tools: Vec::new(),
        expected_output_schema: SchemaId::New("nomos.workflow.orchestration.test.v1"),
        effort: EffortLevel::BackendDefault,
    };
}

/// A `WorkflowStep` with nothing for `Is_Coherent` to reject — no side effects, no
/// retry, the trivially-coherent shape a caller with nothing yet to declare would
/// actually construct.
fn Coherent_Step() -> WorkflowStep
{
    return WorkflowStep {
        input_schema: SchemaId::New("nomos.workflow.orchestration.test.input.v1"),
        output_schema: SchemaId::New("nomos.workflow.orchestration.test.output.v1"),
        has_side_effects: false,
        idempotent: true,
        retry: RetryPolicy::NoRetry,
        timeout: Timeout::Unbounded,
        cacheability: Cacheability::NotCacheable,
        privileges: Vec::new(),
        cancellation: CancellationBehavior::Uncancellable,
        compensation: Compensation::None,
        determinism_strength: DeterminismStrength::None,
        reproducibility_scope: ReproducibilityScope::SingleRun,
        trace_equivalence: TraceEquivalence::NotApplicable,
        evidence: EvidenceClass::AgentJudged,
    };
}

/// The exact incoherent shape `nomos_contracts::workflow_step`'s own tests fix: a
/// side-effecting, non-idempotent step that retries with no deduplication token and no
/// compensation — `WF-012`'s own named failure.
fn Incoherent_Step() -> WorkflowStep
{
    let mut step = Coherent_Step();
    step.has_side_effects = true;
    step.idempotent = false;
    step.retry = RetryPolicy::Retry {
        max_attempts: NonZeroU32::new(MAX_ATTEMPTS).expect("the constant above is a nonzero attempt limit"),
        deduplication_token_required: false,
    };

    return step;
}

/// A launcher whose answers were written down by the test that built it, consumed one
/// per call in the order queued. Unlike `nomos_agent_executor_claude_code`'s own
/// `Scripted`, which only ever answers one call, a multi-step plan dispatches its
/// launcher once per step, so each needs its own scripted answer — and a call with
/// nothing left queued is a test bug (a body dispatched that should not have), not a
/// silently-repeated answer.
struct Scripted
{
    answers: RefCell<VecDeque<ProgramOutput>>,
}

impl Scripted
{
    fn Of(answers: Vec<ProgramOutput>) -> Self
    {
        return Self { answers: RefCell::new(answers.into_iter().collect()) };
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
    fn Run(&self, _command: &Command) -> Result<ProgramOutput, String>
    {
        return self
            .answers
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| return "Scripted launcher ran out of scripted answers".to_owned());
    }
}

/// `result` also seeds `structured_output.assumptions`, so a test can still correlate
/// which scripted answer a step received by checking `answer.result.assumptions` --
/// `OD-EXECUTOR-008`'s decision means `result` itself is no longer read into
/// `AgentExecutionOutcome` at all.
fn Clean_Claude_Code_Response(result: &str) -> ProgramOutput
{
    return ProgramOutput {
        outcome: ExitOutcome::Exited { code: 0 },
        stdout: format!(
            r#"{{"result": "{result}", "structured_output": {{"assumptions": ["{result}"], "unresolved_questions": []}}, "is_error": false, "total_cost_usd": 0.01, "duration_ms": 10, "permission_denials": []}}"#
        ),
        stderr: String::new(),
    };
}

fn Clean_Ollama_Response(response: &str) -> ProgramOutput
{
    return ProgramOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout: response.to_owned(), stderr: String::new() };
}

fn Failing_Response(stderr: &str) -> ProgramOutput
{
    return ProgramOutput { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new(), stderr: stderr.to_owned() };
}

/// Runs `plan` through [`Run`] with a launcher scripted to answer its steps in the order
/// queued, and hands back the run's own outcome.
///
/// The platform every test in this file needs is the same one — a scripted launcher, the real
/// standard filesystem, this process's own environment, and one fixed moment — so building it
/// in a single place is what makes a test's result depend on its plan and its script alone.
/// A body that declares the family `backend` answers to, rather than naming `backend` itself.
///
/// The difference is the item's whole point: a step states what it wants, and
/// `nomos_agent_orchestration::Run_Agent_Task` decides what answers it against the declared
/// set. Writing the family the intended backend already labels keeps these tests asserting
/// the same dispatches they always did, through the resolution rather than around it.
fn Agent_Step(backend: nomos_agent_orchestration::Backend, task: nomos_agent_contracts::TaskEnvelope) -> Body
{
    return Body::Agent(crate::AgentBody {
        task,
        profile: nomos_model_package::ModelExecutionProfile::New(
            nomos_model_package::ModelSelector::BackendFamily(backend.Label().to_owned()),
            nomos_model_package::EffortLevel::BackendDefault,
        ),
    });
}

fn Ran_Outcome(plan: &[WorkflowStepPlan], answers: Vec<ProgramOutput>) -> WorkflowOutcome
{
    let launcher = Scripted::Of(answers);
    let platform = Platform {
        launcher: &launcher,
        filesystem: &StdFileSystem,
        environment: &nomos_platform_std::StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(0),
        declared: &nomos_agent_orchestration::Declared_Targets(),
    };

    return Run(plan, &platform, &Test_Variant(), Test_Run_Id());
}

/// The steps `plan` completed through [`Run`], asserting the run reached the end of the plan.
///
/// Asserts rather than reports the `Completed` outcome: the tests reusing this are proving what
/// a completed run carries, so a refusal or a failure has to stop here with its own message
/// rather than surfacing later as a puzzling assertion about a step.
fn Ran_To_Completion(plan: &[WorkflowStepPlan], answers: Vec<ProgramOutput>) -> Vec<StepOutcome>
{
    let outcome = Ran_Outcome(plan, answers);

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), plan.len(), "every declared step completed");

    return completed;
}

/// The single step a one-body plan completed through [`Run`] — the one-step shape most of the
/// tests below are about.
fn Only_Step(body: Body, answers: Vec<ProgramOutput>) -> StepOutcome
{
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body }];
    let completed = Ran_To_Completion(&plan, answers);

    return completed.into_iter().next().expect("the run above asserted a step for every body");
}

/// A check body judging `text` filed under `a.rs`, restricted to the one rule whose own
/// applicability a single in-memory file can satisfy.
fn Check_Body_Over(text: &str) -> CheckBody
{
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), text.to_owned())];
    let selected = vec![RuleId::New(nomos_rules::COMPLETENESS_MIRROR)];

    return CheckBody::New(std::path::PathBuf::from("."), sources, selected);
}

/// A correction body over `root`, whose already-walked source is `text` filed under `a.rs`, and
/// which asks `Run_Correction` to commit the fix it validates rather than only stage it.
fn Correction_Body_Over(root: &std::path::Path, text: &str) -> CorrectionBody
{
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), text.to_owned())];

    return CorrectionBody::New(root.to_path_buf(), sources, CommitIntent::Commit);
}

/// A gate body judging `text` filed under `a.rs`, narrowed to `parameter-count` alone so this
/// test dispatches no subprocess-backed section (`GateCommand::default`'s own empty selection
/// would otherwise also select `dependency-policy`, `lint-diagnostics` and
/// `dependency-direction`, each of which launches a real `cargo` subprocess against a fixture
/// tree that has no `Cargo.toml` at all).
fn Gate_Body_Over(text: &str) -> GateBody
{
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), text.to_owned())];
    let narrowed = GateCommand { rules: RuleSelector { include: vec![RuleId::New(nomos_rules::PARAMETER_COUNT)] }, ..GateCommand::default() };

    return GateBody::New(sources, narrowed);
}

/// A real phantom-mirror claim: a declared universe with no test naming it as its mirror.
/// The identical fixture `nomos-correction-orchestration::run`'s own tests use.
const PHANTOM_FIXTURE: &str = "/// A list of things this crate owns.\n\
    /// Mirrored by `Test_Nonexistent_Check_That_Does_Not_Exist`.\n\
    pub const THINGS: &[&str] = &[\"a\"];\n";

/// Removes and recreates `name` under the system temp directory, so a test starts from a
/// clean, empty tree regardless of what an earlier run left behind -- `Body::Correction`
/// can write to disk, so unlike `Body::Check`'s own tests this one needs a real,
/// disposable root rather than an in-memory source list alone.
fn Fresh_Root(name: &str) -> std::path::PathBuf
{
    let root = std::env::temp_dir().join(name);
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary root is creatable");

    return root;
}

#[test]
fn Test_An_Empty_Plan_Completes_Vacuously()
{
    let outcome = Ran_Outcome(&[], Vec::new());

    assert_eq!(outcome, WorkflowOutcome::Completed { completed: Vec::new() });
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Claude_Code_Dispatches_And_Completes()
{
    let step = Only_Step(Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("say PONG")), vec![Clean_Claude_Code_Response("PONG")]);

    assert!(matches!(step, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::ClaudeCode(answer)) if answer.result.assumptions == ["PONG".to_owned()]));
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Ollama_Dispatches_And_Completes()
{
    let step = Only_Step(Agent_Step(nomos_agent_orchestration::Backend::Ollama, Task_Envelope("say PONG")), vec![Clean_Ollama_Response("PONG")]);

    assert!(matches!(step, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Ollama(answer)) if answer.response == "PONG"));
}

#[test]
fn Test_A_Two_Step_Sequence_Completes_In_Order()
{
    let bodies = [Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("first")), Agent_Step(nomos_agent_orchestration::Backend::Ollama, Task_Envelope("second"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let completed = Ran_To_Completion(&plan, vec![Clean_Claude_Code_Response("first"), Clean_Ollama_Response("second")]);

    let first = completed.first().expect("the run above asserted a step for every body");
    let second = completed.get(1).expect("the run above asserted a step for every body");
    assert!(matches!(first, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::ClaudeCode(answer)) if answer.result.assumptions == ["first".to_owned()]));
    assert!(matches!(second, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Ollama(answer)) if answer.response == "second"));
}

#[test]
fn Test_An_Incoherent_Step_Is_Refused_Before_Dispatch()
{
    let coherent = WorkflowStepPlan { declaration: Incoherent_Step(), body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("never runs")) };

    let outcome = Ran_Outcome(&[coherent], Vec::new());

    assert_eq!(outcome, WorkflowOutcome::Refused { completed: Vec::new(), index: 0 });
}

#[test]
fn Test_A_Mid_Sequence_Refusal_Preserves_Prior_Completions()
{
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("first")) },
        WorkflowStepPlan { declaration: Incoherent_Step(), body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("never runs")) },
    ];

    let outcome = Ran_Outcome(&plan, vec![Clean_Claude_Code_Response("first")]);

    let WorkflowOutcome::Refused { completed, index } = outcome
    else
    {
        panic!("expected Refused: {outcome:?}")
    };
    assert_eq!(index, 1, "the second step is the incoherent one");
    assert_eq!(completed.len(), 1, "only the step before the refusal ran");
    let first = completed.first().expect("the assertion above fixes the length at one");
    assert!(matches!(first, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::ClaudeCode(answer)) if answer.result.assumptions == ["first".to_owned()]));
}

#[test]
fn Test_A_Failed_Dispatch_Stops_The_Run()
{
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("fails")) }];

    let outcome = Ran_Outcome(&plan, vec![Failing_Response("claude exited 1")]);

    assert!(
        matches!(outcome, WorkflowOutcome::Failed { ref completed, index: 0, ref error }
            if completed.is_empty() && matches!(error, DispatchError::AgentUnavailable(_))),
        "a failing first step stops the run as Failed and completes nothing: {outcome:?}"
    );
}

/// Only one scripted answer: if the second step's body were ever dispatched, the
/// launcher would have nothing left queued for it and `Scripted::Run` would report that
/// as its own failure instead of the second step ever producing a real outcome — proof
/// the second step's body never ran, not merely that its result went unchecked.
#[test]
fn Test_A_Failure_Prevents_A_Later_Step_From_Running()
{
    let bodies = [Agent_Step(nomos_agent_orchestration::Backend::ClaudeCode, Task_Envelope("fails")), Agent_Step(nomos_agent_orchestration::Backend::Ollama, Task_Envelope("never runs"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let outcome = Ran_Outcome(&plan, vec![Failing_Response("claude exited 1")]);

    assert!(
        matches!(outcome, WorkflowOutcome::Failed { ref completed, index: 0, .. } if completed.is_empty()),
        "the first step's failure ends the run before the second step could run: {outcome:?}"
    );
}

/// `P40-WORKFLOW-CHECK-BODY`'s own `done_when`: a two-step workflow whose first step is a
/// check runs through `nomos-check-orchestration::Run` against the canonical check seam,
/// its outcome carried in the same `StepOutcome` shape the agent and model bodies already
/// use, and its second step still dispatches through `nomos-model-backend-ollama`
/// afterward -- proving the new body composes with the two that already existed rather
/// than replacing them.
#[test]
fn Test_A_Two_Step_Workflow_Whose_First_Step_Is_A_Check_Runs_Through_The_Canonical_Seam()
{
    let bodies = [Body::Check(Check_Body_Over("pub fn Ok() {}\n")), Agent_Step(nomos_agent_orchestration::Backend::Ollama, Task_Envelope("second"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let completed = Ran_To_Completion(&plan, vec![Clean_Ollama_Response("second")]);

    let first = completed.first().expect("the run above asserted a step for every body");
    let second = completed.get(1).expect("the run above asserted a step for every body");
    assert!(
        matches!(first, StepOutcome::Check(nomos_check_orchestration::CheckOutcome::Judged { findings, .. }) if findings.is_empty()),
        "{first:?}"
    );
    assert!(matches!(second, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Ollama(answer)) if answer.response == "second"));
}

/// `P40-WORKFLOW-CORRECTION-BODY`'s own `done_when`, the committed half: a workflow step
/// whose body is a correction reaches Preview, Stage, Validate and Commit through
/// `nomos-correction-orchestration::Run_Correction`, and its outcome is carried in the
/// same `StepOutcome` shape the other four bodies already use.
#[test]
fn Test_A_Correction_Step_Should_Commit_A_Real_Phantom_Claim()
{
    let root = Fresh_Root("nomos-workflow-orchestration-correction-body-commit");
    let path = root.join("a.rs");
    std::fs::write(&path, PHANTOM_FIXTURE).expect("the fixture root Fresh_Root just created holds this file");

    let body = Correction_Body_Over(&root, PHANTOM_FIXTURE);
    let step = Only_Step(Body::Correction(body), Vec::new());
    let corrected = std::fs::read_to_string(&path).expect("the committed correction left the file readable");
    let _ignored = std::fs::remove_dir_all(&root);

    let StepOutcome::Correction(nomos_correction_orchestration::CorrectionOutcome::Committed { path: committed_path, .. }) = step
    else
    {
        panic!("expected Committed: {step:?}")
    };
    assert_eq!(committed_path, "a.rs");
    assert_eq!(corrected, "/// A list of things this crate owns.\npub const THINGS: &[&str] = &[\"a\"];\n");
}

/// `P40-WORKFLOW-CORRECTION-BODY`'s own `done_when`, the refused half: a claimed
/// declaration named twice in one file is ambiguous, and the step ends without
/// committing anything rather than guessing which line is the real one.
#[test]
fn Test_A_Correction_Step_Should_Refuse_An_Ambiguous_Claim_Without_Committing()
{
    let root = Fresh_Root("nomos-workflow-orchestration-correction-body-refused");
    let path = root.join("a.rs");
    let ambiguous = format!("{PHANTOM_FIXTURE}\n{PHANTOM_FIXTURE}");
    std::fs::write(&path, &ambiguous).expect("the fixture root Fresh_Root just created holds this file");

    let body = Correction_Body_Over(&root, &ambiguous);
    let step = Only_Step(Body::Correction(body), Vec::new());
    let untouched = std::fs::read_to_string(&path).expect("a refused correction left the file readable");
    let _ignored = std::fs::remove_dir_all(&root);

    assert!(matches!(step, StepOutcome::Correction(nomos_correction_orchestration::CorrectionOutcome::Refused(_))), "{step:?}");
    assert_eq!(untouched, ambiguous, "a refused correction must not touch the file");
}

/// `P40-WORKFLOW-GATE-BODY`'s own `done_when`, the passing half: a gate step whose own
/// disposition is `Passed` reports its full result through `StepOutcome::Gate`, and the
/// workflow completes exactly as a check or correction step would.
#[test]
fn Test_A_Passing_Gate_Step_Completes_As_A_Step_Outcome()
{
    let step = Only_Step(Body::Gate(Gate_Body_Over("pub fn Ok() {}\n")), Vec::new());

    assert!(matches!(step, StepOutcome::Gate(ref result) if result.disposition == GateRunOutcome::Passed), "{step:?}");
}

/// `P40-WORKFLOW-GATE-BODY`'s own `done_when`, the failing half: a gate step whose own
/// disposition is `Failed` ends the workflow as a `DispatchError::Gate` rather than being
/// reported as a step that merely ran -- the one dispatch target this crate composes
/// where completing and failing are not the same shape of answer.
#[test]
fn Test_A_Failing_Gate_Step_Ends_The_Workflow_Rather_Than_Completing()
{
    let over_limit = "pub fn Something(a: i32, b: i32, c: i32, d: i32, e: i32) {}\n";
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(Gate_Body_Over(over_limit)) }];

    let outcome = Ran_Outcome(&plan, Vec::new());

    assert!(
        matches!(outcome, WorkflowOutcome::Failed { ref completed, index: 0, ref error }
            if completed.is_empty()
                && matches!(error, DispatchError::Gate(result) if result.disposition == GateRunOutcome::Failed)),
        "a failing gate step ends the workflow as DispatchError::Gate: {outcome:?}"
    );
}
