//! Honoring `Timeout`: what a bound measures, what it reports when it is broken, and what
//! it reports when nothing measured it at all.

use super::support::*;
use super::*;

/// The bound every case here declares.
const DECLARED_SECONDS: u32 = 30;

/// When a measured dispatch started. An arbitrary real instant, far from zero so a case
/// cannot pass by accident on an unread clock.
const STARTED_AT: i64 = 1_000;

/// When a dispatch that took exactly its declared bound finished -- the boundary case, and
/// the one that distinguishes "no longer waited on *after* this many seconds" from "no
/// longer waited on *at* this many seconds".
const FINISHED_AT_THE_BOUND: i64 = 1_030;

/// When a dispatch that took one second more than its declared bound finished.
const FINISHED_PAST_THE_BOUND: i64 = 1_031;

/// How long the dispatch that broke its bound took.
const OVERRUN_SECONDS: u64 = 31;

/// A dispatch that took exactly its declared bound stayed inside it: `Timeout::Seconds`'s
/// own doc is that the step is no longer waited on *after* that many seconds.
#[test]
fn Test_A_Bounded_Step_Whose_Dispatch_Stays_Inside_Its_Bound_Should_Report_It_Honored()
{
    let plan = [WorkflowStepPlan {
        declaration: Bounded_Step(DECLARED_SECONDS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("prompt")),
    }];

    let run = Ran_Report_With_Clock(&plan, vec![Clean_Executor_Answer("answer")], vec![STARTED_AT, FINISHED_AT_THE_BOUND]);

    let timing = Only_Timing(&run);
    assert_eq!(timing, StepTiming::Honored { declared_seconds: Declared(), elapsed_seconds: u64::from(DECLARED_SECONDS) });
    assert!(!timing.Has_Timed_Out());
}

/// One second past the bound is a timeout, and it is reported as one rather than as a
/// dispatch that merely took a while.
#[test]
fn Test_A_Bounded_Step_Whose_Dispatch_Exceeds_Its_Bound_Should_Be_Reported_As_Timed_Out()
{
    let plan = [WorkflowStepPlan {
        declaration: Bounded_Step(DECLARED_SECONDS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("prompt")),
    }];

    let run = Ran_Report_With_Clock(&plan, vec![Clean_Executor_Answer("answer")], vec![STARTED_AT, FINISHED_PAST_THE_BOUND]);

    let timing = Only_Timing(&run);
    assert_eq!(timing, StepTiming::Exceeded { declared_seconds: Declared(), elapsed_seconds: OVERRUN_SECONDS });
    assert!(timing.Has_Timed_Out());
}

/// The distinction the whole of [`StepTiming`] exists for: a dispatch that broke its bound
/// *and* failed is reported as having timed out, not merely as one more failure. The
/// dispatch error is still carried, because the two answers are about different things --
/// why the dispatch failed, and whether the step was still being waited on when it did.
#[test]
fn Test_A_Bounded_Step_That_Failed_Past_Its_Bound_Should_Be_Reported_As_Timed_Out_Not_Merely_Failed()
{
    let plan = [WorkflowStepPlan {
        declaration: Bounded_Step(DECLARED_SECONDS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("prompt")),
    }];

    let run = Ran_Report_With_Clock(&plan, vec![Failing_Answer("claude exited 1")], vec![STARTED_AT, FINISHED_PAST_THE_BOUND]);

    assert!(matches!(run.outcome, WorkflowOutcome::Failed { index: 0, .. }), "{:?}", run.outcome);
    let attempt = run.attempts.first().expect("the one step was dispatched once");
    assert!(attempt.timing.Has_Timed_Out(), "the failing dispatch broke its bound and says so: {:?}", attempt.timing);
    assert!(attempt.failure.is_some(), "the dispatch error is kept beside the timing, not replaced by it");
}

/// `Timeout::Unbounded` is honored as it always was: there is no bound, so there is
/// nothing to measure against, and a supplied clock does not invent one.
#[test]
fn Test_An_Unbounded_Step_Should_Report_That_There_Was_No_Bound_To_Measure()
{
    let plan = [WorkflowStepPlan {
        declaration: Coherent_Step(),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("prompt")),
    }];

    let run = Ran_Report_With_Clock(&plan, vec![Clean_Executor_Answer("answer")], vec![STARTED_AT, FINISHED_PAST_THE_BOUND]);

    assert_eq!(Only_Timing(&run), StepTiming::Unbounded);
}

/// The requirement that an unmeasured bound is never silently treated as honored: run
/// without a clock, the identical declaration reports [`StepTiming::Unmeasured`], which is
/// a different answer from both `Honored` and `Exceeded`.
#[test]
fn Test_A_Bounded_Step_Run_With_No_Clock_Should_Be_Reported_Unmeasured_Rather_Than_Honored()
{
    let plan = [WorkflowStepPlan {
        declaration: Bounded_Step(DECLARED_SECONDS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("prompt")),
    }];

    let run = Ran_Report(&plan, vec![Clean_Executor_Answer("answer")]);

    let timing = Only_Timing(&run);
    assert_eq!(timing, StepTiming::Unmeasured { declared_seconds: Declared() });
    assert!(!timing.Has_Timed_Out(), "an unmeasured bound is not a broken one either");
}

/// A bound is measured per attempt, not once per step: the clock is read either side of
/// each dispatch, so a retried step's first attempt can break its bound while the retry
/// that answered stayed inside it.
#[test]
fn Test_Each_Attempt_Of_A_Retried_Bounded_Step_Should_Be_Timed_On_Its_Own()
{
    let mut declaration = Retryable_Step(TWO_ATTEMPTS);
    declaration.timeout = Timeout::Seconds(Declared());
    let plan = [WorkflowStepPlan { declaration, body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("flaky")) }];
    let answers = vec![Failing_Answer("first attempt"), Clean_Executor_Answer("second attempt")];
    let readings = vec![STARTED_AT, FINISHED_PAST_THE_BOUND, STARTED_AT, FINISHED_AT_THE_BOUND];

    let run = Ran_Report_With_Clock(&plan, answers, readings);

    let timings: Vec<StepTiming> = run.attempts.iter().map(|attempt| return attempt.timing).collect();
    assert_eq!(
        timings,
        vec![
            StepTiming::Exceeded { declared_seconds: Declared(), elapsed_seconds: OVERRUN_SECONDS },
            StepTiming::Honored { declared_seconds: Declared(), elapsed_seconds: u64::from(DECLARED_SECONDS) },
        ]
    );
}

/// The attempt limit the per-attempt timing case declares.
const TWO_ATTEMPTS: u32 = 2;

/// The bound these cases declare, as the type `Timeout::Seconds` carries.
fn Declared() -> NonZeroU32
{
    return NonZeroU32::new(DECLARED_SECONDS).expect("DECLARED_SECONDS is nonzero");
}

/// The one timing a one-step, one-attempt plan produced.
fn Only_Timing(run: &WorkflowRun) -> StepTiming
{
    let attempt = run.attempts.first().expect("the one step in these plans was dispatched at least once");
    assert_eq!(run.attempts.len(), 1, "these cases declare one step and expect one attempt at it");

    return attempt.timing;
}
