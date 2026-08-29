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
