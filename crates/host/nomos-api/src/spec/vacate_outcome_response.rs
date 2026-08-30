//! [`VacateOutcomeResponse`], carried only by [`super::vacated_response::VacatedResponse`].

use nomos_spec_orchestration::VacateOutcome;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_orchestration::VacateOutcome`], which does not derive
/// `Serialize`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VacateOutcomeResponse
{
    /// The old path was removed.
    Removed,
    /// The old path was already gone.
    AlreadyGone,
    /// The old path could not be removed, so two files now declare this record.
    Failed
    {
        message: String
    },
}

impl VacateOutcomeResponse
{
    pub(crate) fn From(outcome: VacateOutcome) -> Self
    {
        return match outcome
        {
            VacateOutcome::Removed => Self::Removed,
            VacateOutcome::AlreadyGone => Self::AlreadyGone,
            VacateOutcome::Failed(message) => Self::Failed { message },
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
        assert!(matches!(VacateOutcomeResponse::From(VacateOutcome::Removed), VacateOutcomeResponse::Removed));
        assert!(matches!(
            VacateOutcomeResponse::From(VacateOutcome::AlreadyGone),
            VacateOutcomeResponse::AlreadyGone
        ));
        assert!(matches!(
            VacateOutcomeResponse::From(VacateOutcome::Failed("two files now declare this record".to_owned())),
            VacateOutcomeResponse::Failed { message } if message == "two files now declare this record"
        ));
    }
}
