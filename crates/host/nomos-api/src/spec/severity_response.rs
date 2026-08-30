//! [`SeverityResponse`], carried only by [`super::decision_gap_response::DecisionGapResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Map_Every_Domain_Variant_To_Its_Own_Response_Variant()
    {
        assert!(matches!(SeverityResponse::From(Severity::Blocking), SeverityResponse::Blocking));
        assert!(matches!(SeverityResponse::From(Severity::NonBlocking), SeverityResponse::NonBlocking));
    }
}
