//! [`SuppressedBecause`], the disposition that kept a finding from blocking, as a wire caller
//! reads it.

use nomos_gate_orchestration::SuppressionDisposition;
use serde::Serialize;

/// A serializable twin of [`nomos_gate_orchestration::SuppressionDisposition`].
///
/// A twin for the reason [`super::FindingBucket`] is one. Six variants in the same order, so a
/// reordering or a dropped arm is a compile error in [`SuppressedBecause::From`] rather than
/// the wrong word on a wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressedBecause
{
    /// Suppressed at the finding's own site.
    InlineSuppression,
    /// Exempted by a repository-wide policy.
    RepositoryPolicyException,
    /// Accepted for a bounded period, and carrying an end date.
    TemporaryWaiver,
    /// Existing debt a baseline tolerates.
    AcceptedBaselineDebt,
    /// The finding does not hold.
    FalsePositiveDisposition,
    /// A deliberate, owned decision to accept the risk.
    FormalRiskAcceptance,
}

impl SuppressedBecause
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
