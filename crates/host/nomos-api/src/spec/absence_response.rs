//! [`AbsenceResponse`], one entry of [`super::sources_response::SourcesResponse::Assembled`]'s
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Absence()
    {
        let absence = Absence {
            subject: "a source document".to_owned(),
            expected: "under crates/spec".to_owned(),
            cause: "NOMOS_V14_CORPUS is not set".to_owned(),
            cost: "no absences can be reported for it".to_owned(),
        };

        let response = AbsenceResponse::From(absence.clone());

        assert_eq!(response.subject, absence.subject);
        assert_eq!(response.expected, absence.expected);
        assert_eq!(response.cause, absence.cause);
        assert_eq!(response.cost, absence.cost);
    }
}
