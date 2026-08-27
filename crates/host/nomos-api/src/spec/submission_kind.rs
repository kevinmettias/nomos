//! [`SubmissionKindResponse`], carried only by [`super::submission::SubmissionResponse`].

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
