//! [`RefusalResponse`], carried only by [`super::submit_response::SubmitResponse::Refused`].

use nomos_spec_model::Refusal;
use serde::Serialize;

use super::FailureResponse;

/// A serializable twin of [`nomos_spec_model::Refusal`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct RefusalResponse
{
    /// Which submission was refused.
    pub submission: String,
    /// Every rule it failed.
    pub failures: Vec<FailureResponse>,
}

impl RefusalResponse
{
    pub(crate) fn From(refusal: Refusal) -> Self
    {
        return Self {
            submission: refusal.submission,
            failures: refusal.failures.into_iter().map(FailureResponse::From).collect(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_model::Failure;

    #[test]
    fn Test_From_Should_Copy_The_Submission_And_Map_Every_Nested_Failure()
    {
        let refusal = Refusal {
            submission: "FR-API-003".to_owned(),
            failures: vec![Failure {
                field: "goal".to_owned(),
                rule: "required-field".to_owned(),
                remedy: "give goal a value".to_owned(),
            }],
        };

        let response = RefusalResponse::From(refusal.clone());

        assert_eq!(response.submission, refusal.submission);
        assert_eq!(response.failures.len(), 1);
        assert_eq!(
            response.failures.first().expect("asserted above to contain exactly one failure").field,
            "goal"
        );
    }
}
