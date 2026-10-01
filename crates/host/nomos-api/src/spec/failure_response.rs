//! [`FailureResponse`], carried only by [`super::refusal_response::RefusalResponse`].

use nomos_spec_model::Failure;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_model::Failure`], which does not derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct FailureResponse
{
    /// The field that failed, or the rule's subject when no single field owns it.
    pub field: String,
    pub rule: String,
    /// What would satisfy it.
    pub remedy: String,
}

impl FailureResponse
{
    pub(crate) fn From(failure: Failure) -> Self
    {
        return Self { field: failure.field, rule: failure.rule, remedy: failure.remedy };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Failure()
    {
        let failure = Failure {
            field: "goal".to_owned(),
            rule: "required-field".to_owned(),
            remedy: "give goal a value".to_owned(),
        };

        let response = FailureResponse::From(failure.clone());

        assert_eq!(response.field, failure.field);
        assert_eq!(response.rule, failure.rule);
        assert_eq!(response.remedy, failure.remedy);
    }
}
