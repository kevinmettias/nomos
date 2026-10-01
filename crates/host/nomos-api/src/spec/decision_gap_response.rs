//! [`DecisionGapResponse`], carried only by [`super::submission_response::SubmissionResponse`].

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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::Severity;

    #[test]
    fn Test_From_Should_Copy_Every_Field_And_Map_The_Nested_Severity()
    {
        let gap = DecisionGap {
            question: "who owns this rule".to_owned(),
            blocks: vec!["owner".to_owned()],
            severity: Severity::Blocking,
            closed_by: Some("D-132".to_owned()),
        };

        let response = DecisionGapResponse::From(gap.clone());

        assert_eq!(response.question, gap.question);
        assert_eq!(response.blocks, gap.blocks);
        assert!(matches!(response.severity, SeverityResponse::Blocking));
        assert_eq!(response.closed_by, gap.closed_by);
    }
}
