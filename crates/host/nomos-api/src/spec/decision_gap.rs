//! [`DecisionGapResponse`], carried only by [`super::submission::SubmissionResponse`].

use nomos_spec_model::DecisionGap;
use serde::Serialize;

use super::SeverityResponse;

/// A serializable twin of [`nomos_spec_model::DecisionGap`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
pub struct DecisionGapResponse
{
    pub question: String,
    pub blocks: Vec<String>,
    pub severity: SeverityResponse,
    pub closed_by: Option<String>,
}

impl DecisionGapResponse
{
    pub(crate) fn From(gap: DecisionGap) -> Self
    {
        return Self {
            question: gap.question,
            blocks: gap.blocks,
            severity: SeverityResponse::From(gap.severity),
            closed_by: gap.closed_by,
        };
    }
}
