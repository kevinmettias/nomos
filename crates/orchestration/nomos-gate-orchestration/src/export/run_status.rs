//! [`RunStatus`], whether a run reached a complete judgment and, if not, every reason why.

use crate::sarif::SarifInvocation;
use crate::GateRunResult;
use nomos_check_orchestration::CheckOutcome;

/// A run's own status, as every export states it beside the findings.
///
/// Read from the SARIF log's own invocation rather than derived again, so a run the SARIF log
/// calls unsuccessful is unsuccessful in every other export, for the same stated reasons. An
/// export that carried only findings would write a run that judged nothing as an empty file,
/// which is the one reading of an absence every consumer gets wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RunStatus
{
    /// `false` exactly when `reasons` is non-empty.
    pub(crate) is_execution_successful: bool,
    /// Why the run stopped short of a complete judgment, one entry per reason.
    pub(crate) reasons: Vec<String>,
}

impl RunStatus
{
    /// A gate run's status: its check's outcome and, for a judged run, why no verdict was reached.
    pub(crate) fn Of_Gate_Run(result: &GateRunResult) -> Self
    {
        return Self::Read(SarifInvocation::Of_Gate_Run(result));
    }

    /// A check run's status: its outcome alone, since a check applies no policy.
    pub(crate) fn Of_Check_Run(outcome: &CheckOutcome) -> Self
    {
        return Self::Read(SarifInvocation::Of_Check_Run(outcome));
    }

    /// The status `invocation` states, in its own words.
    fn Read(invocation: SarifInvocation) -> Self
    {
        return Self {
            is_execution_successful: invocation.is_execution_successful,
            reasons: invocation.tool_execution_notifications.into_iter().map(|notification| return notification.message.text).collect(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::sarif::fixtures::{Complete_Gate_Result, Empty_Gate_Findings};
    use crate::NoVerdict;

    #[test]
    fn Test_A_Complete_Judged_Run_Should_Be_Successful_With_No_Reason()
    {
        let status = RunStatus::Of_Gate_Run(&Complete_Gate_Result(Empty_Gate_Findings()));

        assert!(status.is_execution_successful);
        assert!(status.reasons.is_empty());
    }

    /// A run that reached no verdict is unsuccessful and says why, in the SARIF log's own words.
    #[test]
    fn Test_A_Run_That_Reached_No_Verdict_Should_Be_Unsuccessful_And_Say_Why()
    {
        let mut result = Complete_Gate_Result(Empty_Gate_Findings());
        result.no_verdict = Some(NoVerdict::IncompleteCoverage);

        let status = RunStatus::Of_Gate_Run(&result);

        assert!(!status.is_execution_successful);
        assert!(status.reasons.iter().any(|reason| return reason.contains("require-completeness")), "{status:?}");
    }
}
