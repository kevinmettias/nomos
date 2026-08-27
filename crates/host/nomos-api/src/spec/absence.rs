//! [`AbsenceResponse`], one entry of [`super::sources::SpecSourcesResponse::Assembled`]'s
//! own `absent` list.

use nomos_spec_orchestration::corpus::Absence;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_orchestration::corpus::Absence`], which does not
/// derive `Serialize`.
#[derive(Debug, Serialize)]
pub struct AbsenceResponse
{
    /// What is missing, as a person would name it.
    pub subject: String,
    /// Where it was looked for, concretely.
    pub expected: String,
    /// Why it is not here.
    pub cause: String,
    /// What is therefore not in this store.
    pub cost: String,
}

impl AbsenceResponse
{
    pub(crate) fn From(absence: Absence) -> Self
    {
        return Self {
            subject: absence.subject,
            expected: absence.expected,
            cause: absence.cause,
            cost: absence.cost,
        };
    }
}
