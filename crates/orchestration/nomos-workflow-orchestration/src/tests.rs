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
use nomos_platform::{Clock, Command, ExitOutcome, ProcessLauncher, ProcessOutput};
use nomos_platform_std::StdFileSystem;
use nomos_workspace::BuildVariant;

use crate::{Body, CheckBody, CorrectionBody, DispatchError, GateBody, Platform, Run, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

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

fn Task(goal: &str) -> TaskEnvelope
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
        max_attempts: NonZeroU32::new(3).expect("3 is nonzero"),
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
    answers: RefCell<VecDeque<ProcessOutput>>,
}

impl Scripted
{
    fn Of(answers: Vec<ProcessOutput>) -> Self
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

impl ProcessLauncher for Scripted
{
    fn Run(&self, _command: &Command) -> Result<ProcessOutput, String>
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
fn Clean_Claude_Code_Response(result: &str) -> ProcessOutput
{
    return ProcessOutput {
        outcome: ExitOutcome::Exited { code: 0 },
        stdout: format!(
            r#"{{"result": "{result}", "structured_output": {{"assumptions": ["{result}"], "unresolved_questions": []}}, "is_error": false, "total_cost_usd": 0.01, "duration_ms": 10, "permission_denials": []}}"#
        ),
        stderr: String::new(),
    };
}

fn Clean_Ollama_Response(response: &str) -> ProcessOutput
{
    return ProcessOutput { outcome: ExitOutcome::Exited { code: 0 }, stdout: response.to_owned(), stderr: String::new() };
}

fn Failing_Response(stderr: &str) -> ProcessOutput
{
    return ProcessOutput { outcome: ExitOutcome::Exited { code: 1 }, stdout: String::new(), stderr: stderr.to_owned() };
}

#[test]
fn Test_An_Empty_Plan_Completes_Vacuously()
{
    let launcher = Scripted::Of(Vec::new());

    let outcome = Run(&[], &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    assert_eq!(outcome, WorkflowOutcome::Completed { completed: Vec::new() });
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Claude_Code_Dispatches_And_Completes()
{
    let launcher = Scripted::Of(vec![Clean_Claude_Code_Response("PONG")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("say PONG")) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 1);
    let first = completed.first().expect("asserted len 1 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.result.assumptions == ["PONG".to_owned()]));
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Ollama_Dispatches_And_Completes()
{
    let launcher = Scripted::Of(vec![Clean_Ollama_Response("PONG")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Ollama(Task("say PONG")) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 1);
    let first = completed.first().expect("asserted len 1 above");
    assert!(matches!(first, StepOutcome::Ollama(answer) if answer.response == "PONG"));
}

#[test]
fn Test_A_Two_Step_Sequence_Completes_In_Order()
{
    let launcher = Scripted::Of(vec![Clean_Claude_Code_Response("first"), Clean_Ollama_Response("second")]);
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("first")) },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Ollama(Task("second")) },
    ];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 2);
    let first = completed.first().expect("asserted len 2 above");
    let second = completed.get(1).expect("asserted len 2 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.result.assumptions == ["first".to_owned()]));
    assert!(matches!(second, StepOutcome::Ollama(answer) if answer.response == "second"));
}

#[test]
fn Test_An_Incoherent_Step_Is_Refused_Before_Dispatch()
{
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Incoherent_Step(), body: Body::ClaudeCode(Task("never runs")) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    assert_eq!(outcome, WorkflowOutcome::Refused { completed: Vec::new(), index: 0 });
}

#[test]
fn Test_A_Mid_Sequence_Refusal_Preserves_Prior_Completions()
{
    let launcher = Scripted::Of(vec![Clean_Claude_Code_Response("first")]);
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("first")) },
        WorkflowStepPlan { declaration: Incoherent_Step(), body: Body::ClaudeCode(Task("never runs")) },
    ];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Refused { completed, index } = outcome
    else
    {
        panic!("expected Refused: {outcome:?}")
    };
    assert_eq!(index, 1);
    assert_eq!(completed.len(), 1);
    let first = completed.first().expect("asserted len 1 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.result.assumptions == ["first".to_owned()]));
}

#[test]
fn Test_A_Failed_Dispatch_Stops_The_Run()
{
    let launcher = Scripted::Of(vec![Failing_Response("claude exited 1")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("fails")) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Failed { completed, index, error } = outcome
    else
    {
        panic!("expected Failed: {outcome:?}")
    };
    assert_eq!(index, 0);
    assert!(completed.is_empty());
    assert!(matches!(error, DispatchError::ClaudeCode(_)));
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
    let launcher = Scripted::Of(vec![Clean_Ollama_Response("second")]);
    let sources = vec![nomos_rules::SourceFile::New(
        "a.rs",
        nomos_model::Subject_Of_Path("a.rs"),
        "pub fn Ok() {}\n",
    )];
    let check = CheckBody::New(
        std::path::PathBuf::from("."),
        sources,
        vec![nomos_contracts::RuleId::New(nomos_rules::COMPLETENESS_MIRROR)],
    );
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Check(check) },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Ollama(Task("second")) },
    ];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 2);
    let first = completed.first().expect("asserted len 2 above");
    let second = completed.get(1).expect("asserted len 2 above");
    assert!(
        matches!(first, StepOutcome::Check(nomos_check_orchestration::CheckOutcome::Judged { findings, .. }) if findings.is_empty()),
        "{first:?}"
    );
    assert!(matches!(second, StepOutcome::Ollama(answer) if answer.response == "second"));
}

/// Only one scripted answer: if the second step's body were ever dispatched, the
/// launcher would have nothing left queued for it and `Scripted::Run` would report that
/// as its own failure instead of the second step ever producing a real outcome — proof
/// the second step's body never ran, not merely that its result went unchecked.
#[test]
fn Test_A_Failure_Prevents_A_Later_Step_From_Running()
{
    let launcher = Scripted::Of(vec![Failing_Response("claude exited 1")]);
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("fails")) },
        WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Ollama(Task("never runs")) },
    ];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Failed { completed, index, .. } = outcome
    else
    {
        panic!("expected Failed: {outcome:?}")
    };
    assert_eq!(index, 0);
    assert!(completed.is_empty());
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

/// `P40-WORKFLOW-CORRECTION-BODY`'s own `done_when`, the committed half: a workflow step
/// whose body is a correction reaches Preview, Stage, Validate and Commit through
/// `nomos-correction-orchestration::Run_Correction`, and its outcome is carried in the
/// same `StepOutcome` shape the other three bodies already use.
#[test]
fn Test_A_Correction_Step_Should_Commit_A_Real_Phantom_Claim()
{
    let root = Fresh_Root("nomos-workflow-orchestration-correction-body-commit");
    let path = root.join("a.rs");
    std::fs::write(&path, PHANTOM_FIXTURE).expect("writable");
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), PHANTOM_FIXTURE.to_owned())];
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Correction(CorrectionBody::New(root.clone(), sources, true)) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());
    let corrected = std::fs::read_to_string(&path).expect("still readable");

    let _ignored = std::fs::remove_dir_all(&root);
    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    let first = completed.first().expect("one step ran");
    match first
    {
        StepOutcome::Correction(nomos_correction_orchestration::CorrectionOutcome::Committed { path: committed_path, .. }) =>
        {
            assert_eq!(committed_path, "a.rs");
        }
        other => panic!("expected Committed: {other:?}"),
    }
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
    std::fs::write(&path, &ambiguous).expect("writable");
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), ambiguous.clone())];
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Correction(CorrectionBody::New(root.clone(), sources, true)) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());
    let untouched = std::fs::read_to_string(&path).expect("still readable");

    let _ignored = std::fs::remove_dir_all(&root);
    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    let first = completed.first().expect("one step ran");
    assert!(matches!(first, StepOutcome::Correction(nomos_correction_orchestration::CorrectionOutcome::Refused(_))), "{first:?}");
    assert_eq!(untouched, ambiguous, "a refused correction must not touch the file");
}

/// `parameter-count`, narrowed to alone so this test dispatches no subprocess-backed
/// section (`GateCommand::default`'s own empty selection would otherwise also select
/// `dependency-policy`, `lint-diagnostics` and `dependency-direction`, each of which
/// launches a real `cargo` subprocess against a fixture tree that has no `Cargo.toml` at
/// all).
fn Narrowed_To_Parameter_Count() -> GateCommand
{
    return GateCommand { rules: RuleSelector { include: vec![RuleId::New(nomos_rules::PARAMETER_COUNT)] }, ..GateCommand::default() };
}

/// `P40-WORKFLOW-GATE-BODY`'s own `done_when`, the passing half: a gate step whose own
/// disposition is `Passed` reports its full result through `StepOutcome::Gate`, and the
/// workflow completes exactly as a check or correction step would.
#[test]
fn Test_A_Passing_Gate_Step_Completes_As_A_Step_Outcome()
{
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), "pub fn Ok() {}\n".to_owned())];
    let gate = GateBody::New(sources, Narrowed_To_Parameter_Count());
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(gate) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    let first = completed.first().expect("one step ran");
    assert!(matches!(first, StepOutcome::Gate(result) if result.disposition == GateRunOutcome::Passed), "{first:?}");
}

/// `P40-WORKFLOW-GATE-BODY`'s own `done_when`, the failing half: a gate step whose own
/// disposition is `Failed` ends the workflow as a `DispatchError::Gate` rather than being
/// reported as a step that merely ran -- the one dispatch target this crate composes
/// where completing and failing are not the same shape of answer.
#[test]
fn Test_A_Failing_Gate_Step_Ends_The_Workflow_Rather_Than_Completing()
{
    let over_limit = "pub fn Something(a: i32, b: i32, c: i32, d: i32, e: i32) {}\n";
    let sources = vec![nomos_rules::SourceFile::New("a.rs", nomos_model::Subject_Of_Path("a.rs"), over_limit.to_owned())];
    let gate = GateBody::New(sources, Narrowed_To_Parameter_Count());
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Gate(gate) }];

    let outcome = Run(&plan, &Platform { launcher: &launcher, filesystem: &StdFileSystem, environment: &nomos_platform_std::StdEnvironment }, &Test_Variant(), Test_Run_Id());

    let WorkflowOutcome::Failed { completed, index, error } = outcome
    else
    {
        panic!("expected Failed: {outcome:?}")
    };
    assert_eq!(index, 0);
    assert!(completed.is_empty());
    assert!(matches!(error, DispatchError::Gate(ref result) if result.disposition == GateRunOutcome::Failed), "{error:?}");
}
