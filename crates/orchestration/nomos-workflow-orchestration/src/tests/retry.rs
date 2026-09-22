//! Honoring `RetryPolicy`: what a failed dispatch is re-dispatched under, how many times,
//! and what it is never re-dispatched under at all.

use super::support::*;
use super::*;

/// The attempt limit the retrying cases declare.
const THREE_ATTEMPTS: u32 = 3;

/// The attempt limit the second-cover case declares -- two, so one failure and one
/// success exhausts it exactly.
const TWO_ATTEMPTS: u32 = 2;

/// `RetryPolicy::Retry`'s own `max_attempts` counts the first attempt, so a step declaring
/// three attempts that fails twice has one left and must complete on it.
#[test]
fn Test_A_Retryable_Step_Should_Be_Redispatched_Up_To_Its_Declared_Attempt_Limit()
{
    let plan = [WorkflowStepPlan {
        declaration: Retryable_Step(THREE_ATTEMPTS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("flaky")),
    }];
    let answers = vec![Failing_Answer("first attempt"), Failing_Answer("second attempt"), Clean_Executor_Answer("third attempt")];

    let run = Ran_Report(&plan, answers);

    assert!(matches!(run.outcome, WorkflowOutcome::Completed { .. }), "the third attempt answered, so the step completed: {:?}", run.outcome);
    assert_eq!(Dispatched_Attempts(&run), vec![(0, 1), (0, 2), (0, 3)], "one step, three attempts, in order");
    let attempts: Vec<bool> = run.attempts.iter().map(|attempt| return attempt.failure.is_some()).collect();
    assert_eq!(attempts, vec![true, true, false], "the first two attempts failed and the third did not");
}

/// The limit is a limit, not a suggestion: a fourth scripted answer that would have
/// succeeded is never reached, which is why a failed outcome here is the proof rather than
/// merely the expectation.
#[test]
fn Test_A_Retryable_Step_Whose_Every_Attempt_Fails_Should_Stop_The_Run_At_Its_Limit()
{
    let plan = [WorkflowStepPlan {
        declaration: Retryable_Step(THREE_ATTEMPTS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("never answers")),
    }];
    let answers = vec![
        Failing_Answer("first attempt"),
        Failing_Answer("second attempt"),
        Failing_Answer("third attempt"),
        Clean_Executor_Answer("a fourth attempt that must never be dispatched"),
    ];

    let run = Ran_Report(&plan, answers);

    assert!(
        matches!(run.outcome, WorkflowOutcome::Failed { index: 0, .. }),
        "a fourth dispatch would have consumed the clean answer and completed the run: {:?}",
        run.outcome
    );
    assert_eq!(Dispatched_Attempts(&run), vec![(0, 1), (0, 2), (0, 3)], "three declared attempts, three dispatches, no fourth");
}

/// `RetryPolicy::NoRetry` never re-dispatches. The second scripted answer would have
/// completed the run had the failed first attempt been retried.
#[test]
fn Test_A_Step_Declaring_No_Retry_Should_Never_Be_Redispatched()
{
    let plan = [WorkflowStepPlan {
        declaration: Coherent_Step(),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("fails once")),
    }];
    let answers = vec![Failing_Answer("the only attempt"), Clean_Executor_Answer("a retry that must never happen")];

    let run = Ran_Report(&plan, answers);

    assert!(matches!(run.outcome, WorkflowOutcome::Failed { index: 0, .. }), "{:?}", run.outcome);
    assert_eq!(Dispatched_Attempts(&run), vec![(0, 1)], "NoRetry permits the first attempt and nothing after it");
}

/// `WF-012`'s own named failure, and the requirement that a retried non-idempotent
/// side-effecting step is dispatched only under the cover `WorkflowStep::Is_Coherent`
/// requires: with neither a deduplication token nor a compensation, the step is refused
/// and its body never dispatches at all. An empty attempt list is the direct evidence --
/// not that the dispatch's result went unchecked, but that no dispatch happened.
#[test]
fn Test_An_Uncovered_Retry_Of_A_Non_Idempotent_Side_Effecting_Step_Should_Never_Be_Dispatched()
{
    let plan = [WorkflowStepPlan {
        declaration: Incoherent_Step(),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("never runs")),
    }];

    let run = Ran_Report(&plan, Vec::new());

    assert_eq!(run.outcome, WorkflowOutcome::Refused { completed: Vec::new(), index: 0 });
    assert!(run.attempts.is_empty(), "an uncovered retryable step is refused before any attempt: {:?}", run.attempts);
}

/// The other half of the same requirement: the identical side-effecting, non-idempotent
/// step *is* retried once a compensation covers it, which is the second of the two covers
/// `Is_Coherent` accepts. So retrying such a step is reachable only through a cover, and
/// both covers reach it.
#[test]
fn Test_A_Compensation_Covered_Retry_Of_The_Same_Step_Should_Be_Redispatched()
{
    let plan = [WorkflowStepPlan {
        declaration: Compensated_Retryable_Step(TWO_ATTEMPTS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("flaky")),
    }];
    let answers = vec![Failing_Answer("first attempt"), Clean_Executor_Answer("second attempt")];

    let run = Ran_Report(&plan, answers);

    assert!(matches!(run.outcome, WorkflowOutcome::Completed { .. }), "{:?}", run.outcome);
    assert_eq!(Dispatched_Attempts(&run), vec![(0, 1), (0, 2)], "a compensation covers the retry the same way a token does");
}

/// [`crate::Run`] is what `nomos_cli::workflow` and `nomos_api::workflow` call, and it
/// honors a declaration exactly as the reporting entry points do -- it reports less, it
/// does not do less.
#[test]
fn Test_The_Outcome_Only_Entry_Point_Should_Honor_A_Retry_Too()
{
    let plan = [WorkflowStepPlan {
        declaration: Retryable_Step(TWO_ATTEMPTS),
        body: Agent_Step(EXECUTOR_FAMILY, Task_Envelope("flaky")),
    }];
    let answers = vec![Failing_Answer("first attempt"), Clean_Executor_Answer("second attempt")];

    let outcome = Ran_Outcome(&plan, answers);

    assert!(matches!(outcome, WorkflowOutcome::Completed { .. }), "the retry ran through the narrow entry point too: {outcome:?}");
}
