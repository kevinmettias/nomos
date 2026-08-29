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
