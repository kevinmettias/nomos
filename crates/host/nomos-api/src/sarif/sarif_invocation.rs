//! [`SarifInvocation`], whether the run reached a clean judgment and, if not, why.

use super::sarif_notification::SarifNotification;
use crate::check::{CheckResponse, ClaimResponse};
use crate::response::{CheckOutcomeResponse, GateRunResponse, NoVerdictResponse};
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
    /// Why the execution was not successful, one entry per reason the response carries.
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
    pub(crate) fn Of_Gate_Run(response: &GateRunResponse) -> Self
    {
        let mut notifications = Vec::new();
        notifications.extend(Check_Outcome_Notification(&response.check_outcome));
        notifications.extend(response.no_verdict.as_ref().map(No_Verdict_Notification));

        return Self::From_Notifications(notifications);
    }

    /// The invocation for a check run, which applies no policy and so has only the check's own
    /// outcome to report on.
    pub(crate) fn Of_Check_Run(response: &CheckResponse) -> Self
    {
        return Self::From_Notifications(Check_Response_Notification(response).into_iter().collect());
    }

    fn From_Notifications(tool_execution_notifications: Vec<SarifNotification>) -> Self
    {
        return Self { is_execution_successful: tool_execution_notifications.is_empty(), tool_execution_notifications };
    }
}

/// Why a gate run's check judged nothing, or judged incompletely; `None` for a complete judgment.
fn Check_Outcome_Notification(outcome: &CheckOutcomeResponse) -> Option<SarifNotification>
{
    return match outcome
    {
        CheckOutcomeResponse::Unreadable => Some(SarifNotification::Unreadable_Root()),
        CheckOutcomeResponse::Contradictory { cause } => Some(SarifNotification::Contradictory(cause)),
        CheckOutcomeResponse::NoSource => Some(SarifNotification::No_Source()),
        CheckOutcomeResponse::NoFacts { files } => Some(SarifNotification::No_Facts(*files)),
        CheckOutcomeResponse::Judged { complete: false, .. } => Some(SarifNotification::Incomplete_Claim()),
        CheckOutcomeResponse::Judged { complete: true, .. } => None,
    };
}

/// Why a judged gate run reached no verdict.
fn No_Verdict_Notification(cause: &NoVerdictResponse) -> SarifNotification
{
    return match cause
    {
        NoVerdictResponse::UnreadablePolicy { detail } => SarifNotification::Unreadable_Policy(detail),
        NoVerdictResponse::MalformedPolicy { detail } => SarifNotification::Malformed_Policy(detail),
        NoVerdictResponse::IncompleteCoverage => SarifNotification::Incomplete_Coverage(),
    };
}

/// Why a check run judged nothing, or judged incompletely; `None` for a complete judgment.
fn Check_Response_Notification(response: &CheckResponse) -> Option<SarifNotification>
{
    return match response
    {
        CheckResponse::Unreadable => Some(SarifNotification::Unreadable_Root()),
        CheckResponse::Contradictory { cause } => Some(SarifNotification::Contradictory(cause)),
        CheckResponse::NoSource => Some(SarifNotification::No_Source()),
        CheckResponse::NoFacts { files } => Some(SarifNotification::No_Facts(*files)),
        CheckResponse::Judged { claim: ClaimResponse::Incomplete, .. } => Some(SarifNotification::Incomplete_Claim()),
        CheckResponse::Judged { claim: ClaimResponse::Complete, .. } => None,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::check::ExaminedResponse;
    use crate::test_support::{Complete_Gate_Response, Empty_Gate_Findings};

    /// How many files the `NoFacts` fixtures below report having read.
    const READ_FILES: usize = 2;

    #[test]
    fn Test_Of_Gate_Run_Should_Be_Successful_With_No_Notification_For_A_Complete_Judged_Run()
    {
        let invocation = SarifInvocation::Of_Gate_Run(&Complete_Gate_Response(Empty_Gate_Findings()));

        assert!(invocation.is_execution_successful);
        assert!(invocation.tool_execution_notifications.is_empty());
    }

    /// Every way a gate run's check can stop short of a complete judgment is unsuccessful and
    /// says why -- the falsifier for each arm of `Check_Outcome_Notification`.
    #[test]
    fn Test_Of_Gate_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_Short_Check_Outcome()
    {
        for (outcome, word) in [
            (CheckOutcomeResponse::Unreadable, "could not be read"),
            (CheckOutcomeResponse::Contradictory { cause: "a cause".to_owned() }, "a cause"),
            (CheckOutcomeResponse::NoSource, "no source"),
            (CheckOutcomeResponse::NoFacts { files: READ_FILES }, "no fact"),
            (CheckOutcomeResponse::Judged { files: 1, facts: 1, complete: false }, "incomplete claim"),
        ]
        {
            let mut response = Complete_Gate_Response(Empty_Gate_Findings());
            response.check_outcome = outcome;

            let invocation = SarifInvocation::Of_Gate_Run(&response);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(word)), "{word}: {invocation:?}");
        }
    }

    /// Every way a judged run can reach no verdict is unsuccessful and says why -- the
    /// falsifier for each arm of `No_Verdict_Notification`.
    #[test]
    fn Test_Of_Gate_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_No_Verdict_Cause()
    {
        for (cause, word) in [
            (NoVerdictResponse::UnreadablePolicy { detail: "permission denied".to_owned() }, "permission denied"),
            (NoVerdictResponse::MalformedPolicy { detail: "unknown field `basline`".to_owned() }, "basline"),
            (NoVerdictResponse::IncompleteCoverage, "require-completeness"),
        ]
        {
            let mut response = Complete_Gate_Response(Empty_Gate_Findings());
            response.no_verdict = Some(cause);

            let invocation = SarifInvocation::Of_Gate_Run(&response);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(word)), "{word}: {invocation:?}");
        }
    }

    /// An incomplete claim under a coverage floor is two facts, and both reach the consumer.
    #[test]
    fn Test_Of_Gate_Run_Should_Carry_Both_Notifications_When_Both_Sources_Speak()
    {
        let mut response = Complete_Gate_Response(Empty_Gate_Findings());
        response.check_outcome = CheckOutcomeResponse::Judged { files: 1, facts: 1, complete: false };
        response.no_verdict = Some(NoVerdictResponse::IncompleteCoverage);

        let invocation = SarifInvocation::Of_Gate_Run(&response);

        assert_eq!(invocation.tool_execution_notifications.len(), 2, "{invocation:?}");
    }

    #[test]
    fn Test_Of_Check_Run_Should_Be_Successful_Only_For_A_Complete_Judged_Run()
    {
        let complete = CheckResponse::Judged { findings: vec![], examined: ExaminedResponse { files: 1, facts: 1 }, claim: ClaimResponse::Complete };
        let incomplete = CheckResponse::Judged { findings: vec![], examined: ExaminedResponse { files: 1, facts: 1 }, claim: ClaimResponse::Incomplete };

        assert!(SarifInvocation::Of_Check_Run(&complete).is_execution_successful);
        assert!(!SarifInvocation::Of_Check_Run(&incomplete).is_execution_successful);
        assert!(SarifInvocation::Of_Check_Run(&incomplete).tool_execution_notifications.iter().any(|notification| return notification.message.text.contains("incomplete claim")));
    }

    #[test]
    fn Test_Of_Check_Run_Should_Be_Unsuccessful_And_Say_Why_For_Every_Non_Judged_Outcome()
    {
        for (response, word) in [
            (CheckResponse::Unreadable, "could not be read"),
            (CheckResponse::Contradictory { cause: "a cause".to_owned() }, "a cause"),
            (CheckResponse::NoSource, "no source"),
            (CheckResponse::NoFacts { files: READ_FILES }, "no fact"),
        ]
        {
            let invocation = SarifInvocation::Of_Check_Run(&response);

            assert!(!invocation.is_execution_successful, "{word}");
            assert!(invocation.tool_execution_notifications.iter().any(|notification| return notification.message.text.contains(word)), "{word}: {invocation:?}");
        }
    }

    #[test]
    fn Test_An_Invocation_Should_Serialize_Under_The_Specifications_Names()
    {
        let rendered = serde_json::to_value(SarifInvocation::Of_Check_Run(&CheckResponse::NoSource))
            .expect("a derived Serialize over owned data has nothing to refuse");

        assert_eq!(rendered.pointer("/executionSuccessful").and_then(serde_json::Value::as_bool), Some(false), "{rendered}");
        assert_eq!(rendered.pointer("/toolExecutionNotifications").and_then(serde_json::Value::as_array).map(Vec::len), Some(1), "{rendered}");
    }
}
