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
use nomos_platform::{Clock, Command, ProgramLauncher, ProgramOutput};
use nomos_platform_std::StdFileSystem;
use nomos_workspace::BuildVariant;

use crate::{
    Body, BranchArm, BranchChoice, CheckBody, CommitIntent, CorrectionBody, DefinitionRefusal, DefinitionRun, DispatchError, GateBody, GroupVisitOrder,
    NodeDisposition, Parallelism, Platform, ProducedState, ReplayRefusal, Run, StepCompensation, StepOutcome, StepTiming, WorkflowDefinition,
    WorkflowDefinitionId, WorkflowExecution, WorkflowNode, WorkflowOutcome, WorkflowRun, WorkflowRunRecord, WorkflowStepPlan,
};

/// The cases for each of the three declarations
/// `P123-WORKFLOW-RETRY-TIMEOUT-COMPENSATION-RUNTIME` made this crate honor, and the
/// doubles all three share.
///
/// Beside this file rather than in it: this one already carries the dispatch cases for all
/// four bodies and sits close to the five-hundred-line review trigger, and three more
/// families of case would push it past one.
mod compensation;
mod definition;
mod graph;
mod replay;
mod retry;
mod support;
mod timeout;

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
/// per call in the order queued. Unlike an adapter crate's own single-answer scripted
/// launcher, a multi-step plan dispatches its launcher once per step, so each needs its own
/// scripted answer — and a call with nothing left queued is a test bug (a body dispatched
/// that should not have), not a silently-repeated answer.
///
/// No agent step reaches this any more. An agent body resolves to one of the ports
/// [`Declared_Ports`] declares, and those answer from [`ScriptedPorts`]' own queue; what is
/// left here is the platform a check, correction or gate body is handed.
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

/// The family label the executor port this file declares answers to. Deliberately not a
/// shipped backend's label: this crate names no adapter, so a case that only passed because
/// it happened to spell a real family would be testing a coincidence.
const EXECUTOR_FAMILY: &str = "harness-agent";

/// The family label the model backend port this file declares answers to.
const MODEL_FAMILY: &str = "harness-model";

/// One scripted answer for whichever port the next dispatch resolves to.
///
/// Replaces the `ProgramOutput`s this file used to queue. Those were subprocess output read
/// back through a real adapter's own parser, because a step dispatched straight into an
/// adapter crate and there was no seam closer than the process boundary to substitute at.
/// `OD-ROADMAP-005` decision 2 put a port there, so a case now says what the backend answered
/// rather than what a process printed -- and this crate no longer depends on either adapter to
/// say it.
#[derive(Clone, Debug)]
enum PortAnswer
{
    /// An `AgentExecutorPackage` answered, carrying the text as its one assumption so a case
    /// can still correlate which queued answer a step received.
    Executed(String),
    /// A `ModelBackendPackage` answered with this text.
    Answered(String),
    /// Whichever port was reached produced nothing, for this reason.
    Refused(String),
    /// An `AgentExecutorPackage` answered and raised its own error flag -- a dispatch that
    /// completed and reported that something went wrong, which is not the same thing as a
    /// dispatch that failed. The distinction is what a branch reads.
    Errored(String),
}

fn Clean_Executor_Answer(result: &str) -> PortAnswer
{
    return PortAnswer::Executed(result.to_owned());
}

fn Clean_Model_Answer(response: &str) -> PortAnswer
{
    return PortAnswer::Answered(response.to_owned());
}

fn Failing_Answer(reason: &str) -> PortAnswer
{
    return PortAnswer::Refused(reason.to_owned());
}

/// An executor that answered and raised its own error flag, which is what makes a node
/// publish `ProducedState::Flagged` without the dispatch having failed.
fn Errored_Executor_Answer(result: &str) -> PortAnswer
{
    return PortAnswer::Errored(result.to_owned());
}

/// Both ports, answering from one queue the case wrote down, consumed one per dispatch in the
/// order queued -- the identical discipline [`Scripted`] holds for a launcher, and for the
/// identical reason: a multi-step plan dispatches once per step, and a dispatch with nothing
/// left queued is a test bug rather than a silently repeated answer.
struct ScriptedPorts
{
    answers: RefCell<VecDeque<PortAnswer>>,
}

impl ScriptedPorts
{
    fn Of(answers: Vec<PortAnswer>) -> Self
    {
        return Self { answers: RefCell::new(answers.into_iter().collect()) };
    }

    fn Next(&self) -> PortAnswer
    {
        return self
            .answers
            .borrow_mut()
            .pop_front()
            .expect("the case queued an answer for every dispatch the plan makes");
    }
}

impl nomos_agent_contracts::AgentExecutor for ScriptedPorts
{
    fn Execute(
        &self, _task: &TaskEnvelope, _root: &std::path::Path,
    ) -> Result<nomos_agent_contracts::AgentExecution, nomos_agent_contracts::DispatchRefusal>
    {
        return match self.Next()
        {
            PortAnswer::Executed(result) => Ok(Executed_With(&result, false)),
            PortAnswer::Errored(result) => Ok(Executed_With(&result, true)),
            PortAnswer::Refused(reason) => Err(nomos_agent_contracts::DispatchRefusal::Of(reason)),
            PortAnswer::Answered(response) =>
            {
                panic!("a model backend answer {response:?} was queued for an executor dispatch")
            }
        };
    }
}

impl nomos_agent_contracts::ModelBackend for ScriptedPorts
{
    fn Answer(
        &self, _task: &TaskEnvelope,
    ) -> Result<nomos_agent_contracts::ModelAnswer, nomos_agent_contracts::DispatchRefusal>
    {
        return match self.Next()
        {
            PortAnswer::Answered(response) => Ok(nomos_agent_contracts::ModelAnswer { response }),
            PortAnswer::Refused(reason) => Err(nomos_agent_contracts::DispatchRefusal::Of(reason)),
            PortAnswer::Executed(result) | PortAnswer::Errored(result) =>
            {
                panic!("an executor execution {result:?} was queued for a model backend dispatch")
            }
        };
    }
}

/// An execution carrying `result` as its one assumption, `is_error` as the flag the
/// executor raised, and the four measurements an `AgentExecutorPackage` establishes and a
/// `ModelBackendPackage` does not.
fn Executed_With(result: &str, is_error: bool) -> nomos_agent_contracts::AgentExecution
{
    use nomos_agent_contracts::{PortionSubstantiation, Substantiation, UnsubstantiatedReason, WorkResult};

    return nomos_agent_contracts::AgentExecution {
        result: WorkResult {
            plan: None,
            claims: Vec::new(),
            tests: Vec::new(),
            requested_verification: None,
            assumptions: vec![result.to_owned()],
            unresolved_questions: Vec::new(),
            substantiation: Substantiation {
                plan: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                claims: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                tests: PortionSubstantiation::Unsubstantiated(UnsubstantiatedReason::ProducerCannotGround),
                requested_verification: PortionSubstantiation::Unsubstantiated(
                    UnsubstantiatedReason::ProducerCannotGround,
                ),
                assumptions: PortionSubstantiation::Substantiated,
                unresolved_questions: PortionSubstantiation::Substantiated,
            },
        },
        denied_tool_uses: Vec::new(),
        is_error,
        spend: nomos_agent_contracts::MicroDollars::From_Micros(10_000),
        duration_ms: 10,
    };
}

/// `ports` as the two declared targets every case in this file resolves against -- the
/// declaration a composition root supplies, which this crate reads and never writes.
fn Declared_Ports(ports: &ScriptedPorts) -> Vec<nomos_agent_contracts::DeclaredTarget<'_>>
{
    use nomos_agent_contracts::{DeclaredTarget, DispatchPort};

    return vec![
        DeclaredTarget {
            family: EXECUTOR_FAMILY.to_owned(),
            package: Harness_Package(EXECUTOR_FAMILY, nomos_contracts::PackageKind::AgentExecutorPackage),
            port: DispatchPort::Executor(ports),
        },
        DeclaredTarget {
            family: MODEL_FAMILY.to_owned(),
            package: Harness_Package(MODEL_FAMILY, nomos_contracts::PackageKind::ModelBackendPackage),
            port: DispatchPort::Model(ports),
        },
    ];
}

fn Harness_Package(family: &str, kind: nomos_contracts::PackageKind) -> nomos_model_package::ModelRoutePackage
{
    return nomos_model_package::ModelRoutePackage {
        package_id: nomos_contracts::PackageId::New(format!("harness.{family}")),
        package_kind: kind,
        package_version: nomos_model_package::PackageVersion::New(1, 0, 0),
        protocol_range: nomos_model_package::ProtocolRange::New(
            nomos_contracts::ContractVersion::New(1, 0),
            nomos_contracts::ContractVersion::New(1, 0),
        ),
        model_selection: nomos_model_package::ModelSelection::Opaque,
    };
}

/// Runs `plan` through [`Run`] with a launcher scripted to answer its steps in the order
/// queued, and hands back the run's own outcome.
///
/// The platform every test in this file needs is the same one — a scripted launcher, the real
/// standard filesystem, this process's own environment, and one fixed moment — so building it
/// in a single place is what makes a test's result depend on its plan and its script alone.
/// A body that declares the family it wants answered, rather than naming a backend.
///
/// The difference is the item's whole point: a step states what it wants, and
/// `nomos_agent_orchestration::Run_Agent_Task` decides what answers it against the declared
/// set.
fn Agent_Step(family: &str, task: nomos_agent_contracts::TaskEnvelope) -> Body
{
    return Body::Agent(crate::AgentBody {
        task,
        profile: nomos_model_package::ModelExecutionProfile::New(
            nomos_model_package::ModelSelector::BackendFamily(family.to_owned()),
            nomos_model_package::EffortLevel::BackendDefault,
        ),
    });
}

fn Ran_Outcome(plan: &[WorkflowStepPlan], answers: Vec<PortAnswer>) -> WorkflowOutcome
{
    let launcher = Scripted::Of(Vec::new());
    let ports = ScriptedPorts::Of(answers);
    let declared = Declared_Ports(&ports);
    let platform = Platform {
        launcher: &launcher,
        filesystem: &StdFileSystem,
        environment: &nomos_platform_std::StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(0),
        declared: &declared,
    };

    return Run(plan, &platform, &Test_Variant(), Test_Run_Id());
}

/// The steps `plan` completed through [`Run`], asserting the run reached the end of the plan.
///
/// Asserts rather than reports the `Completed` outcome: the tests reusing this are proving what
/// a completed run carries, so a refusal or a failure has to stop here with its own message
/// rather than surfacing later as a puzzling assertion about a step.
fn Ran_To_Completion(plan: &[WorkflowStepPlan], answers: Vec<PortAnswer>) -> Vec<StepOutcome>
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
fn Only_Step(body: Body, answers: Vec<PortAnswer>) -> StepOutcome
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
    let step = Only_Step(Agent_Step(EXECUTOR_FAMILY, Task_Envelope("say PONG")), vec![Clean_Executor_Answer("PONG")]);

    assert!(matches!(step, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Executed { ref execution, .. }) if execution.result.assumptions == ["PONG".to_owned()]));
}

#[test]
fn Test_A_Single_Coherent_Step_Against_Ollama_Dispatches_And_Completes()
{
    let step = Only_Step(Agent_Step(MODEL_FAMILY, Task_Envelope("say PONG")), vec![Clean_Model_Answer("PONG")]);

    assert!(matches!(step, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Answered { ref answer, .. }) if answer.response == "PONG"));
}

#[test]
fn Test_A_Two_Step_Sequence_Completes_In_Order()
{
    let bodies = [Agent_Step(EXECUTOR_FAMILY, Task_Envelope("first")), Agent_Step(MODEL_FAMILY, Task_Envelope("second"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let completed = Ran_To_Completion(&plan, vec![Clean_Executor_Answer("first"), Clean_Model_Answer("second")]);

    let first = completed.first().expect("the run above asserted a step for every body");
    let second = completed.get(1).expect("the run above asserted a step for every body");
    assert!(matches!(first, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Executed { execution, .. }) if execution.result.assumptions == ["first".to_owned()]));
    assert!(matches!(second, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Answered { answer, .. }) if answer.response == "second"));
}

#[test]
fn Test_An_Incoherent_Step_Is_Refused_Before_Dispatch()
{
    let coherent = WorkflowStepPlan { declaration: Incoherent_Step(), body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("never runs")) };

    let outcome = Ran_Outcome(&[coherent], Vec::new());

    assert_eq!(outcome, WorkflowOutcome::Refused { completed: Vec::new(), index: 0 });
}

#[test]
fn Test_A_Mid_Sequence_Refusal_Preserves_Prior_Completions()
{
    let plan = [
        WorkflowStepPlan { declaration: Coherent_Step(), body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("first")) },
        WorkflowStepPlan { declaration: Incoherent_Step(), body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("never runs")) },
    ];

    let outcome = Ran_Outcome(&plan, vec![Clean_Executor_Answer("first")]);

    let WorkflowOutcome::Refused { completed, index } = outcome
    else
    {
        panic!("expected Refused: {outcome:?}")
    };
    assert_eq!(index, 1, "the second step is the incoherent one");
    assert_eq!(completed.len(), 1, "only the step before the refusal ran");
    let first = completed.first().expect("the assertion above fixes the length at one");
    assert!(matches!(first, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Executed { execution, .. }) if execution.result.assumptions == ["first".to_owned()]));
}

#[test]
fn Test_A_Failed_Dispatch_Stops_The_Run()
{
    let plan = [WorkflowStepPlan { declaration: Coherent_Step(), body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("fails")) }];

    let outcome = Ran_Outcome(&plan, vec![Failing_Answer("claude exited 1")]);

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
    let bodies = [Agent_Step(EXECUTOR_FAMILY, Task_Envelope("fails")), Agent_Step(MODEL_FAMILY, Task_Envelope("never runs"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let outcome = Ran_Outcome(&plan, vec![Failing_Answer("claude exited 1")]);

    assert!(
        matches!(outcome, WorkflowOutcome::Failed { ref completed, index: 0, .. } if completed.is_empty()),
        "the first step's failure ends the run before the second step could run: {outcome:?}"
    );
}

/// `P40-WORKFLOW-CHECK-BODY`'s own `done_when`: a two-step workflow whose first step is a
/// check runs through `nomos-check-orchestration::Run` against the canonical check seam,
/// its outcome carried in the same `StepOutcome` shape an agent body already uses, and its
/// second step still dispatches through a model backend port afterward -- proving the new body
/// composes with the one that already existed rather than replacing it.
#[test]
fn Test_A_Two_Step_Workflow_Whose_First_Step_Is_A_Check_Runs_Through_The_Canonical_Seam()
{
    let bodies = [Body::Check(Check_Body_Over("pub fn Ok() {}\n")), Agent_Step(MODEL_FAMILY, Task_Envelope("second"))];
    let plan: Vec<WorkflowStepPlan> = bodies.into_iter().map(|body| return WorkflowStepPlan { declaration: Coherent_Step(), body }).collect();

    let completed = Ran_To_Completion(&plan, vec![Clean_Model_Answer("second")]);

    let first = completed.first().expect("the run above asserted a step for every body");
    let second = completed.get(1).expect("the run above asserted a step for every body");
    assert!(
        matches!(first, StepOutcome::Check(nomos_check_orchestration::CheckOutcome::Judged { findings, .. }) if findings.is_empty()),
        "{first:?}"
    );
    assert!(matches!(second, StepOutcome::Agent(nomos_agent_orchestration::AgentDispatchOutcome::Answered { answer, .. }) if answer.response == "second"));
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
