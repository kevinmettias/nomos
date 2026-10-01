//! [`SarifInvocation`], whether the run reached a clean judgment and, if not, why.

use super::sarif_notification::SarifNotification;
use crate::{GateRunResult, NoVerdict};
use nomos_check_orchestration::{CheckOutcome, Claim};
use serde::Serialize;

/// SARIF 2.1.0 §3.20's `invocation` object.
///
/// `executionSuccessful` is the property a CI consumer reads before it reads a single result,
/// and it is `false` for every run that stopped short of a complete judgment: a tree that
/// could not be walked, a policy that could not be read, a coverage floor that was not met,
/// a rule that could not look. Each of those has a clean-looking result list -- often an empty
/// one -- and the whole reason `nomos_contracts::Applicability` exists is that an empty answer
/// and an unexamined one must never render the same. A `false` here always travels with at
/// least one notification naming why, so a consumer is never told only that something went
/// wrong.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SarifInvocation
{
    /// `false` exactly when `tool_execution_notifications` is non-empty.
    #[serde(rename = "executionSuccessful")]
    pub(crate) is_execution_successful: bool,
    /// Why the execution was not successful, one entry per reason the judgment carries.
    pub(crate) tool_execution_notifications: Vec<SarifNotification>,
}

impl SarifInvocation
{
    /// The invocation for a gate run.
    ///
    /// Two sources, both consulted: the check outcome says whether and how completely the
    /// tree was judged, and `no_verdict` says why a judged tree still reached no verdict. They
    /// are kept as two notifications when both speak -- a judged run under `require-completeness`
    /// carries an incomplete claim *and* a coverage-floor refusal, and those are two facts a
    /// person at a terminal is shown separately.
    pub(crate) fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        let mut notifications = Vec::new();
        notifications.extend(Check_Outcome_Notification(&result.check_outcome));
        notifications.extend(result.no_verdict.as_ref().map(No_Verdict_Notification));

        return Self::From_Notifications(notifications);
    }

    /// The invocation for a check run, which applies no policy and so has only the check's own
    /// outcome to report on.
    pub(crate) fn Of_Check_Run(outcome: &CheckOutcome) -> Self
    {
        return Self::From_Notifications(Check_Outcome_Notification(outcome).into_iter().collect());
    }

    fn From_Notifications(tool_execution_notifications: Vec<SarifNotification>) -> Self
    {
        return Self { is_execution_successful: tool_execution_notifications.is_empty(), tool_execution_notifications };
    }
}

/// Why a check judged nothing, or judged incompletely; `None` for a complete judgment.
///
/// One function serves both runs because both carry the check service's own outcome. Two
/// existed while this projection lived in `nomos-api`, because a gate response and a check
/// response each held a twin of that outcome, and the twins disagreed on one word: a
/// contradictory registry's cause was its `Display` in one and its `Debug` in the other.
/// Reading the error itself leaves one answer, its own `Display`.
fn Check_Outcome_Notification(outcome: &CheckOutcome) -> Option<SarifNotification>
{
    return match outcome
    {
        CheckOutcome::Unreadable => Some(SarifNotification::Unreadable_Root()),
        CheckOutcome::Contradictory(error) => Some(SarifNotification::Contradictory(&error.to_string())),
        CheckOutcome::NoSource => Some(SarifNotification::No_Source()),
        CheckOutcome::NoFacts { files } => Some(SarifNotification::No_Facts(*files)),
        CheckOutcome::Judged { claim: Claim::Incomplete, .. } => Some(SarifNotification::Incomplete_Claim()),
        CheckOutcome::Judged { claim: Claim::Complete, .. } => None,
    };
}

/// Why a judged gate run reached no verdict.
fn No_Verdict_Notification(cause: &NoVerdict) -> SarifNotification
{
    return match cause
    {
        NoVerdict::UnreadablePolicy(detail) => SarifNotification::Unreadable_Policy(detail),
        NoVerdict::MalformedPolicy(detail) => SarifNotification::Malformed_Policy(detail),
        NoVerdict::IncompleteCoverage => SarifNotification::Incomplete_Coverage(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Contradictory_Outcome, Empty_Gate_Findings, Judged_Outcome};

    /// How many files the `NoFacts` fixtures below report having read.
    const READ_FILES: usize = 2;

    #[test]
    fn Test_Of_Gate_Run_Should_Be_Successful_With_No_Notification_For_A_Complete_Judged_Run()
    {
        let invocation = SarifInvocation::Of_Gate_Run(&Complete_Gate_Result(Empty_Gate_Findings()));

        assert!(invocation.is_execution_successful);
        assert!(invocation.tool_execution_notifications.is_empty());
    }

    /// Every way a gate run's check can stop short of a complete judgment is unsuccessful and
    /// says why -- the falsifier for each arm of `Check_Outcome_Notification`.
    #[test]
    fn Test_Of_Gate_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_Short_Check_Outcome()
    {
        for (outcome, word) in Short_Check_Outcomes()
        {
            let mut result = Complete_Gate_Result(Empty_Gate_Findings());
            result.check_outcome = outcome;

            let invocation = SarifInvocation::Of_Gate_Run(&result);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(&word)), "{word}: {invocation:?}");
        }
    }

    /// Every way a judged run can reach no verdict is unsuccessful and says why -- the
    /// falsifier for each arm of `No_Verdict_Notification`.
    #[test]
    fn Test_Of_Gate_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_No_Verdict_Cause()
    {
        for (cause, word) in [
            (NoVerdict::UnreadablePolicy("permission denied".to_owned()), "permission denied"),
            (NoVerdict::MalformedPolicy("unknown field `basline`".to_owned()), "basline"),
            (NoVerdict::IncompleteCoverage, "require-completeness"),
        ]
        {
            let mut result = Complete_Gate_Result(Empty_Gate_Findings());
            result.no_verdict = Some(cause);

            let invocation = SarifInvocation::Of_Gate_Run(&result);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(word)), "{word}: {invocation:?}");
        }
    }

    /// An incomplete claim under a coverage floor is two facts, and both reach the consumer.
    #[test]
    fn Test_Of_Gate_Run_Should_Carry_Both_Notifications_When_Both_Sources_Speak()
    {
        let mut result = Complete_Gate_Result(Empty_Gate_Findings());
        result.check_outcome = Judged_Outcome(Claim::Incomplete);
        result.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let invocation = SarifInvocation::Of_Gate_Run(&result);

        assert_eq!(invocation.tool_execution_notifications.len(), 2, "{invocation:?}");
    }

    #[test]
    fn Test_Of_Check_Run_Should_Be_Successful_Only_For_A_Complete_Judged_Run()
    {
        let complete = Judged_Outcome(Claim::Complete);
        let incomplete = Judged_Outcome(Claim::Incomplete);

        assert!(SarifInvocation::Of_Check_Run(&complete).is_execution_successful);
        assert!(!SarifInvocation::Of_Check_Run(&incomplete).is_execution_successful);
        assert!(SarifInvocation::Of_Check_Run(&incomplete).tool_execution_notifications.iter().any(|notification| return notification.message.text.contains("incomplete claim")));
    }

    #[test]
    fn Test_Of_Check_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_Non_Judged_Outcome()
    {
        for (outcome, word) in Short_Check_Outcomes().into_iter().filter(|(outcome, _)| return !matches!(outcome, CheckOutcome::Judged { .. }))
        {
            let invocation = SarifInvocation::Of_Check_Run(&outcome);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(&word)), "{word}: {invocation:?}");
        }
    }

    #[test]
    fn Test_An_Invocation_Should_Serialize_Under_The_Specifications_Names()
    {
        let rendered = serde_json::to_value(SarifInvocation::Of_Check_Run(&CheckOutcome::NoSource))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{rendered}");
        assert_eq!(rendered.pointer("/toolExecutionNotifications").and_then(serde_json::Value::as_array).map(Vec::len), Some(1), "{rendered}");
    }

    /// Every check outcome short of a complete judgment, beside a word its notification must
    /// carry. The contradictory one is a real registry refusal, so its word is that refusal's
    /// own message rather than one a fixture chose.
    fn Short_Check_Outcomes() -> Vec<(CheckOutcome, String)>
    {
        let contradictory = Contradictory_Outcome();
        let cause = match &contradictory
        {
            CheckOutcome::Contradictory(error) => error.to_string(),
            other => panic!("the fixture is a contradictory outcome, not {other:?}"),
        };

        return vec![
            (CheckOutcome::Unreadable, "could not be read".to_owned()),
            (contradictory, cause),
            (CheckOutcome::NoSource, "no source".to_owned()),
            (CheckOutcome::NoFacts { files: READ_FILES }, "no fact".to_owned()),
            (Judged_Outcome(Claim::Incomplete), "incomplete claim".to_owned()),
        ];
    }
}
