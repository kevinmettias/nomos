use std::cell::RefCell;
use std::collections::VecDeque;
use std::num::NonZeroU32;

use nomos_agent_contracts::TaskEnvelope;
use nomos_contracts::{
    Cacheability, CancellationBehavior, Compensation, DeterminismStrength, EvidenceClass, ReproducibilityScope, RetryPolicy, SchemaId,
    Timeout, TraceEquivalence, WorkflowStep,
};
use nomos_ledger::Territory;
use nomos_model_package::EffortLevel;
use nomos_platform::{Command, ExitOutcome, ProcessLauncher, ProcessOutput};

use crate::{Body, DispatchError, Run, StepOutcome, WorkflowOutcome, WorkflowStepPlan};

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

fn Clean_Claude_Code_Response(result: &str) -> ProcessOutput
{
    return ProcessOutput {
        outcome: ExitOutcome::Exited { code: 0 },
        stdout: format!(
            r#"{{"result": "{result}", "is_error": false, "total_cost_usd": 0.01, "duration_ms": 10, "permission_denials": []}}"#
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

    let outcome = Run(&[], &launcher);

    assert_eq!(outcome, WorkflowOutcome::Completed { completed: Vec::new() });
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Claude_Code_Dispatches_And_Completes()
{
    let launcher = Scripted::Of(vec![Clean_Claude_Code_Response("PONG")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("say PONG")) }];

    let outcome = Run(&plan, &launcher);

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 1);
    let first = completed.first().expect("asserted len 1 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.response == "PONG"));
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Ollama_Dispatches_And_Completes()
{
    let launcher = Scripted::Of(vec![Clean_Ollama_Response("PONG")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::Ollama(Task("say PONG")) }];

    let outcome = Run(&plan, &launcher);

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

    let outcome = Run(&plan, &launcher);

    let WorkflowOutcome::Completed { completed } = outcome
    else
    {
        panic!("expected Completed: {outcome:?}")
    };
    assert_eq!(completed.len(), 2);
    let first = completed.first().expect("asserted len 2 above");
    let second = completed.get(1).expect("asserted len 2 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.response == "first"));
    assert!(matches!(second, StepOutcome::Ollama(answer) if answer.response == "second"));
}

#[test]
fn Test_An_Incoherent_Step_Is_Refused_Before_Dispatch()
{
    let launcher = Scripted::Of(Vec::new());
    let plan = [WorkflowStepPlan { declaration: Incoherent_Step(), body: Body::ClaudeCode(Task("never runs")) }];

    let outcome = Run(&plan, &launcher);

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

    let outcome = Run(&plan, &launcher);

    let WorkflowOutcome::Refused { completed, index } = outcome
    else
    {
        panic!("expected Refused: {outcome:?}")
    };
    assert_eq!(index, 1);
    assert_eq!(completed.len(), 1);
    let first = completed.first().expect("asserted len 1 above");
    assert!(matches!(first, StepOutcome::ClaudeCode(answer) if answer.response == "first"));
}

#[test]
fn Test_A_Failed_Dispatch_Stops_The_Run()
{
    let launcher = Scripted::Of(vec![Failing_Response("claude exited 1")]);
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Body::ClaudeCode(Task("fails")) }];

    let outcome = Run(&plan, &launcher);

    let WorkflowOutcome::Failed { completed, index, error } = outcome
    else
    {
        panic!("expected Failed: {outcome:?}")
    };
    assert_eq!(index, 0);
    assert!(completed.is_empty());
    assert!(matches!(error, DispatchError::ClaudeCode(_)));
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

    let outcome = Run(&plan, &launcher);

    let WorkflowOutcome::Failed { completed, index, .. } = outcome
    else
    {
        panic!("expected Failed: {outcome:?}")
    };
    assert_eq!(index, 0);
    assert!(completed.is_empty());
}
