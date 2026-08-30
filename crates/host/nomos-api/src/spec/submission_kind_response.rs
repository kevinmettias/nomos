//! [`SubmissionKindResponse`], carried only by [`super::submission_response::SubmissionResponse`].

use nomos_spec_model::SubmissionKind;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::SubmissionKind`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionKindResponse
{
    FeatureRequest,
    DesignSpec,
    FeatureResult,
}

impl SubmissionKindResponse
{
    pub(crate) fn From(kind: SubmissionKind) -> Self
    {
        return match kind
        {
            SubmissionKind::FeatureRequest => Self::FeatureRequest,
            SubmissionKind::DesignSpec => Self::DesignSpec,
            SubmissionKind::FeatureResult => Self::FeatureResult,
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
        assert!(matches!(
            SubmissionKindResponse::From(SubmissionKind::FeatureRequest),
            SubmissionKindResponse::FeatureRequest
        ));
        assert!(matches!(
            SubmissionKindResponse::From(SubmissionKind::DesignSpec),
            SubmissionKindResponse::DesignSpec
        ));
        assert!(matches!(
            SubmissionKindResponse::From(SubmissionKind::FeatureResult),
            SubmissionKindResponse::FeatureResult
        ));
    }
}
