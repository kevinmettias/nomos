//! [`SeverityResponse`], carried only by [`super::decision_gap::DecisionGapResponse`].

use nomos_spec_model::Severity;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::Severity`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityResponse
{
    Blocking,
    NonBlocking,
}

impl SeverityResponse
{
    pub(crate) fn From(severity: Severity) -> Self
    {
        return match severity
        {
            Severity::Blocking => Self::Blocking,
            Severity::NonBlocking => Self::NonBlocking,
        };
    }
}
