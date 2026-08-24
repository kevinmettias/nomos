//! Why a candidate that looked available could not actually serve one operation.

use serde::{Deserialize, Serialize};

/// `MODEL-ROUTE-034`: "An available candidate that cannot satisfy the exact operation
/// at runtime shall produce a typed `RuntimeCandidateDisqualification` rather than
/// `ProviderUnavailable` or a generic fallback reason. Supported reasons shall include
/// effective context-limit overflow, structured-output loss, unsupported tool-call
/// depth or parallelism, changed data boundary, projected budget breach, unsatisfiable
/// latency deadline, provider degradation, oversized evidence attachment, and
/// empirical reliability below the operation-specific threshold."
///
/// A dedicated "supported reasons shall include" sentence, the same closed-list
/// transcription pattern that already licensed `EffortLevel`. `ProviderUnavailable`
/// and "a generic fallback reason" are named non-members -- the requirement rules
/// them out by name, so neither belongs as a variant here even though both would
/// otherwise look like plausible additions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCandidateDisqualification
{
    pub reason: DisqualificationReason,
}

/// `MODEL-ROUTE-034`'s nine named reasons, in the corpus's own order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisqualificationReason
{
    EffectiveContextLimitOverflow,
    StructuredOutputLoss,
    UnsupportedToolCallDepthOrParallelism,
    ChangedDataBoundary,
    ProjectedBudgetBreach,
    UnsatisfiableLatencyDeadline,
    ProviderDegradation,
    OversizedEvidenceAttachment,
    EmpiricalReliabilityBelowThreshold,
}

impl DisqualificationReason
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::EffectiveContextLimitOverflow => "EffectiveContextLimitOverflow",
            Self::StructuredOutputLoss => "StructuredOutputLoss",
            Self::UnsupportedToolCallDepthOrParallelism => "UnsupportedToolCallDepthOrParallelism",
            Self::ChangedDataBoundary => "ChangedDataBoundary",
            Self::ProjectedBudgetBreach => "ProjectedBudgetBreach",
            Self::UnsatisfiableLatencyDeadline => "UnsatisfiableLatencyDeadline",
            Self::ProviderDegradation => "ProviderDegradation",
            Self::OversizedEvidenceAttachment => "OversizedEvidenceAttachment",
            Self::EmpiricalReliabilityBelowThreshold => "EmpiricalReliabilityBelowThreshold",
        };
    }
}

impl core::fmt::Display for DisqualificationReason
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [DisqualificationReason; 9] = [
        DisqualificationReason::EffectiveContextLimitOverflow,
        DisqualificationReason::StructuredOutputLoss,
        DisqualificationReason::UnsupportedToolCallDepthOrParallelism,
        DisqualificationReason::ChangedDataBoundary,
        DisqualificationReason::ProjectedBudgetBreach,
        DisqualificationReason::UnsatisfiableLatencyDeadline,
        DisqualificationReason::ProviderDegradation,
        DisqualificationReason::OversizedEvidenceAttachment,
        DisqualificationReason::EmpiricalReliabilityBelowThreshold,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|reason| return reason.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two reasons share a wire spelling");
    }

    #[test]
    fn Test_A_Disqualification_Carries_Its_Reason()
    {
        let disqualification = RuntimeCandidateDisqualification {
            reason: DisqualificationReason::ProjectedBudgetBreach,
        };

        assert_eq!(disqualification.reason, DisqualificationReason::ProjectedBudgetBreach);
    }
}
