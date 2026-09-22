//! The doubles and declarations the retry, timeout, compensation, definition, graph and
//! replay cases share.
//!
//! Beside [`super`]'s own doubles rather than inside them: `super` is already near this
//! workspace's five-hundred-line review trigger, and the case modules each need all of
//! these, so a helper reached by several siblings belongs in one place they all name
//! rather than in whichever of them happened to be written first.

use std::num::NonZeroUsize;

use super::*;

/// A clock whose readings were written down by the test that built it, handed out one per
/// call in the order queued -- the same "answers the test wrote down" shape [`Scripted`]
/// already is for a launcher, applied to the other port a timed run reads.
///
/// Fixed readings rather than a real clock, because a timeout case has to distinguish a
/// dispatch that stayed inside its bound from one that broke it, and a real clock can only
/// produce the second by sleeping -- which is the slow, intermittent test
/// `nomos_platform::Clock`'s own doc says this port exists to avoid. A call with nothing
/// left queued is a test bug: the run read the clock more times than the case accounted
/// for, and saying so loudly is what keeps the reading count itself an assertion.
pub(super) struct ScriptedClock
{
    readings: RefCell<VecDeque<nomos_platform::Timestamp>>,
}

impl ScriptedClock
{
    pub(super) fn Of(readings: Vec<i64>) -> Self
    {
        let queued = readings.into_iter().map(nomos_platform::Timestamp::From_Unix_Seconds).collect();

        return Self { readings: RefCell::new(queued) };
    }
}

/// Answers from fixed data, so its readings reproduce byte for byte.
impl Strategy for ScriptedClock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::State;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

impl Clock for ScriptedClock
{
    fn Now(&self) -> nomos_platform::Timestamp
    {
        return self.readings.borrow_mut().pop_front().expect("the case queued a reading for every clock read the run performs");
    }
}

/// Runs `plan` through [`crate::Run_Unclocked`] with a launcher scripted to answer its
/// steps in the order queued, and hands back the whole report.
///
/// The clockless entry point, so every bounded step in a plan run through this reports
/// [`StepTiming::Unmeasured`].
pub(super) fn Ran_Report(plan: &[WorkflowStepPlan], answers: Vec<PortAnswer>) -> WorkflowRun
{
    let launcher = Scripted::Of(Vec::new());
    let ports = ScriptedPorts::Of(answers);
    let declared = Declared_Ports(&ports);
    let platform = Test_Platform(&launcher, &declared);

    return crate::Run_Unclocked(plan, &platform, &Test_Variant(), Test_Run_Id());
}

/// Runs `plan` through [`crate::Run_With_Clock`], measuring each bounded step against
/// `readings` -- two per attempt, the one taken before the dispatch and the one taken
/// after it.
pub(super) fn Ran_Report_With_Clock(plan: &[WorkflowStepPlan], answers: Vec<PortAnswer>, readings: Vec<i64>) -> WorkflowRun
{
    let launcher = Scripted::Of(Vec::new());
    let ports = ScriptedPorts::Of(answers);
    let clock = ScriptedClock::Of(readings);
    let declared = Declared_Ports(&ports);
    let platform = Test_Platform(&launcher, &declared);
    let clocked = crate::ClockedPlatform { platform: &platform, clock: &clock };

    return crate::Run_With_Clock(plan, &clocked, &Test_Variant(), Test_Run_Id());
}

/// The platform both runners above build: `launcher`'s scripted answers, the real standard
/// filesystem a correction step writes through, this process's own environment, and one
/// fixed moment.
pub(super) fn Test_Platform<'a>(launcher: &'a Scripted, declared: &'a [nomos_agent_contracts::DeclaredTarget<'a>]) -> Platform<'a, Scripted, StdFileSystem, nomos_platform_std::StdEnvironment>
{
    return Platform {
        launcher,
        filesystem: &StdFileSystem,
        environment: &nomos_platform_std::StdEnvironment,
        now: nomos_platform::Timestamp::From_Unix_Seconds(FIXED_MOMENT),
        declared,
    };
}

/// The moment every case judges against. The one `super::Ran_Outcome` already uses.
const FIXED_MOMENT: i64 = 0;

/// A step that is side-effecting and not idempotent -- so `WF-012` applies to it -- and
/// that declares a retry covered by requiring a per-attempt deduplication token.
pub(super) fn Retryable_Step(max_attempts: u32) -> WorkflowStep
{
    let mut step = Coherent_Step();
    step.has_side_effects = true;
    step.idempotent = false;
    step.retry = RetryPolicy::Retry {
        max_attempts: NonZeroU32::new(max_attempts).expect("every case passes a nonzero attempt limit"),
        deduplication_token_required: true,
    };

    return step;
}

/// The same side-effecting, non-idempotent retry as [`Retryable_Step`], covered by the
/// other of the two covers `WorkflowStep::Is_Coherent` accepts: a declared compensation
/// instead of a deduplication token.
pub(super) fn Compensated_Retryable_Step(max_attempts: u32) -> WorkflowStep
{
    let mut step = Retryable_Step(max_attempts);
    step.retry = RetryPolicy::Retry {
        max_attempts: NonZeroU32::new(max_attempts).expect("every case passes a nonzero attempt limit"),
        deduplication_token_required: false,
    };
    step.compensation = Compensation::SelfCompensating;

    return step;
}

/// A step declaring `Timeout::Seconds(seconds)` and nothing else out of the ordinary.
pub(super) fn Bounded_Step(seconds: u32) -> WorkflowStep
{
    let mut step = Coherent_Step();
    step.timeout = Timeout::Seconds(NonZeroU32::new(seconds).expect("every case passes a nonzero bound"));

    return step;
}

/// A step declaring `compensation` and nothing else out of the ordinary.
pub(super) fn Declaring_Compensation(compensation: Compensation) -> WorkflowStep
{
    let mut step = Coherent_Step();
    step.compensation = compensation;

    return step;
}

/// Which attempt each dispatch in `run` was, in the order the dispatches happened, paired
/// with the step it dispatched.
pub(super) fn Dispatched_Attempts(run: &WorkflowRun) -> Vec<(usize, u32)>
{
    return run.attempts.iter().map(|attempt| return (attempt.index, attempt.attempt)).collect();
}

/// The identity every definition case publishes under. One spelling, so a case that varies
/// the identity has to say so.
pub(super) fn Test_Definition_Id() -> WorkflowDefinitionId
{
    return WorkflowDefinitionId::New("nomos.workflow.test.definition");
}

/// The version every definition case publishes at first.
pub(super) const FIRST_VERSION: u32 = 1;

/// A bound of `bound` nodes per dispatch group.
pub(super) fn Bound(bound: usize) -> NonZeroUsize
{
    return NonZeroUsize::new(bound).expect("every case states a nonzero bound");
}

/// A step node named `name`, dispatching `body`, depending on the nodes `requires` names.
pub(super) fn Step_Node(name: &str, requires: &[&str], body: Body) -> WorkflowNode
{
    return WorkflowNode::Step {
        name: name.to_owned(),
        requires: Owned(requires),
        step: WorkflowStepPlan { declaration: Coherent_Step(), body },
    };
}

/// A step node declaring `declaration` rather than the trivially coherent one.
pub(super) fn Declaring_Node(name: &str, declaration: WorkflowStep, body: Body) -> WorkflowNode
{
    return WorkflowNode::Step { name: name.to_owned(), requires: Vec::new(), step: WorkflowStepPlan { declaration, body } };
}

/// A branch node named `name`, choosing an arm by the state the node `on` published.
pub(super) fn Branch_Node(name: &str, on: &str, arms: Vec<BranchArm>) -> WorkflowNode
{
    return WorkflowNode::Branch { name: name.to_owned(), on: on.to_owned(), arms };
}

/// The arm chosen when the branched-on node published `when`, owning the nodes `nodes`
/// names.
pub(super) fn Arm(when: ProducedState, nodes: &[&str]) -> BranchArm
{
    return BranchArm { when, nodes: Owned(nodes) };
}

/// A join node named `name`, waiting for the nodes `arms` names.
pub(super) fn Join_Node(name: &str, arms: &[&str]) -> WorkflowNode
{
    return WorkflowNode::Join { name: name.to_owned(), arms: Owned(arms) };
}

/// `names`, owned.
fn Owned(names: &[&str]) -> Vec<String>
{
    return names.iter().map(|name| return (*name).to_owned()).collect();
}

/// `nodes` published as [`Test_Definition_Id`] at [`FIRST_VERSION`], asserting the publish
/// was accepted.
///
/// Asserts rather than reports, for the reason `super::Ran_To_Completion` asserts: a case
/// about what a *run* does has to fail here with its own message if the definition never
/// published at all, rather than surfacing later as a puzzling assertion about a node.
pub(super) fn Published(nodes: Vec<WorkflowNode>) -> WorkflowDefinition
{
    return WorkflowDefinition::Publish(Test_Definition_Id(), FIRST_VERSION, nodes).expect("this case's definition is coherent");
}

/// Runs `definition` through [`crate::Run_Definition`] under `parallelism`, with the ports
/// scripted to answer its nodes in the order queued, and hands back the whole record.
pub(super) fn Recorded(definition: &WorkflowDefinition, answers: Vec<PortAnswer>, parallelism: Parallelism) -> WorkflowRunRecord
{
    let launcher = Scripted::Of(Vec::new());
    let ports = ScriptedPorts::Of(answers);
    let declared = Declared_Ports(&ports);
    let platform = Test_Platform(&launcher, &declared);
    let variant = Test_Variant();
    let execution = WorkflowExecution { platform: &platform, variant: &variant, run: Test_Run_Id(), parallelism };

    return crate::Run_Definition(definition, &execution);
}

/// Replays `record` against `against` with the ports scripted to answer in the order
/// queued.
pub(super) fn Replayed(record: &WorkflowRunRecord, against: &WorkflowDefinition, answers: Vec<PortAnswer>) -> Result<DefinitionRun, ReplayRefusal>
{
    let launcher = Scripted::Of(Vec::new());
    let ports = ScriptedPorts::Of(answers);
    let declared = Declared_Ports(&ports);
    let platform = Test_Platform(&launcher, &declared);
    let variant = Test_Variant();
    let execution = WorkflowExecution { platform: &platform, variant: &variant, run: Test_Run_Id(), parallelism: Parallelism::Of(Bound(1)) };

    return crate::Replay(record, against, &execution);
}

/// What became of each node in `run`, by name, in the order the run reported them.
pub(super) fn Dispositions(run: &DefinitionRun) -> Vec<(String, NodeDisposition)>
{
    return run.nodes.iter().map(|node| return (node.name.clone(), node.disposition.clone())).collect();
}

/// The names of the nodes in each group `run` visited, in the order it visited them.
pub(super) fn Grouped_Names(run: &DefinitionRun) -> Vec<Vec<String>>
{
    return run.groups.iter().map(|group| return group.nodes.clone()).collect();
}
