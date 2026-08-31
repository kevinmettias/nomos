//! [`SuppressionDispositionResponse`], the six-way disposition [`super::suppression_response::
//! SuppressionResponse`] carries.

use nomos_gate_orchestration::SuppressionDisposition;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::SuppressionDisposition`], kept to the
/// same six variants, in the same order, so a mismatch between the two is a compile error in
/// [`SuppressionDispositionResponse::From`] rather than a silent divergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionDispositionResponse
{
    /// Suppressed at the finding's own site, e.g. an inline annotation.
    InlineSuppression,
    /// Exempted by a repository-wide policy rather than a per-finding annotation.
    RepositoryPolicyException,
    /// Accepted for a bounded period, expected to be revisited.
    TemporaryWaiver,
    /// Existing debt a baseline tolerates rather than blocks.
    AcceptedBaselineDebt,
    /// The finding does not hold; the rule (or its inputs) were wrong here.
    FalsePositiveDisposition,
    /// A deliberate, owned decision to accept the risk the finding names.
    FormalRiskAcceptance,
}

impl SuppressionDispositionResponse
{
    pub(crate) fn From(disposition: SuppressionDisposition) -> Self
    {
        return match disposition
        {
            SuppressionDisposition::InlineSuppression => Self::InlineSuppression,
            SuppressionDisposition::RepositoryPolicyException => Self::RepositoryPolicyException,
            SuppressionDisposition::TemporaryWaiver => Self::TemporaryWaiver,
            SuppressionDisposition::AcceptedBaselineDebt => Self::AcceptedBaselineDebt,
            SuppressionDisposition::FalsePositiveDisposition => Self::FalsePositiveDisposition,
            SuppressionDisposition::FormalRiskAcceptance => Self::FormalRiskAcceptance,
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Map_Every_Suppression_Disposition_Variant_To_Its_Own_Response_Variant()
    {
        Assert_Maps(SuppressionDisposition::InlineSuppression, SuppressionDispositionResponse::InlineSuppression);
        Assert_Maps(
            SuppressionDisposition::RepositoryPolicyException,
            SuppressionDispositionResponse::RepositoryPolicyException,
        );
        Assert_Maps(SuppressionDisposition::TemporaryWaiver, SuppressionDispositionResponse::TemporaryWaiver);
        Assert_Maps(SuppressionDisposition::AcceptedBaselineDebt, SuppressionDispositionResponse::AcceptedBaselineDebt);
        Assert_Maps(
            SuppressionDisposition::FalsePositiveDisposition,
            SuppressionDispositionResponse::FalsePositiveDisposition,
        );
        Assert_Maps(SuppressionDisposition::FormalRiskAcceptance, SuppressionDispositionResponse::FormalRiskAcceptance);
    }

    /// One disposition's own round trip through [`SuppressionDispositionResponse::From`] --
    /// the shared body every arm of the test above drove separately before this existed.
    fn Assert_Maps(disposition: SuppressionDisposition, expected: SuppressionDispositionResponse)
    {
        assert_eq!(SuppressionDispositionResponse::From(disposition), expected);
    }
}
