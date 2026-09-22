//! The doubles and declarations the retry, timeout and compensation cases share.
//!
//! Beside [`super`]'s own doubles rather than inside them: `super` is already near this
//! workspace's five-hundred-line review trigger, and the three case modules each need all
//! of these, so a helper reached by three siblings belongs in one place they all name
//! rather than in whichever of them happened to be written first.

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
fn Test_Platform<'a>(launcher: &'a Scripted, declared: &'a [nomos_agent_contracts::DeclaredTarget<'a>]) -> Platform<'a, Scripted, StdFileSystem, nomos_platform_std::StdEnvironment>
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
