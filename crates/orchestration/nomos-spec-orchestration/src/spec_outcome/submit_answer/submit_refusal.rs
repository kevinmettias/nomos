//! Why `nomos spec submit` did not accept a submission, or could not place its projection.

use nomos_spec_model::Refusal;
use nomos_spec_store::{AcceptError, StoreError};

use crate::spec_outcome::RenderRefusal;

/// Why `nomos spec submit` did not accept a submission, or could not place its projection.
#[derive(Debug)]
pub enum SubmitRefusal
{
    /// The store could not be assembled at all.
    Store(StoreError),
    /// The submission failed `OD-SPEC-010`'s rule set. Nothing was stored.
    Refused(Refusal),
    /// The submission was accepted and its `subject-dossier` projection could not be built or
    /// placed.
    Written(RenderRefusal),
}

impl From<AcceptError> for SubmitRefusal
{
    fn from(error: AcceptError) -> Self
    {
        return match error
        {
            AcceptError::Refused(refusal) => Self::Refused(refusal),
            AcceptError::Store(error) => Self::Store(error),
        };
    }
}
