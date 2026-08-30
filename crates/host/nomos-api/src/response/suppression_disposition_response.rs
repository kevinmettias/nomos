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
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::InlineSuppression),
            SuppressionDispositionResponse::InlineSuppression
        );
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::RepositoryPolicyException),
            SuppressionDispositionResponse::RepositoryPolicyException
        );
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::TemporaryWaiver),
            SuppressionDispositionResponse::TemporaryWaiver
        );
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::AcceptedBaselineDebt),
            SuppressionDispositionResponse::AcceptedBaselineDebt
        );
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::FalsePositiveDisposition),
            SuppressionDispositionResponse::FalsePositiveDisposition
        );
        assert_eq!(
            SuppressionDispositionResponse::From(SuppressionDisposition::FormalRiskAcceptance),
            SuppressionDispositionResponse::FormalRiskAcceptance
        );
    }
}
