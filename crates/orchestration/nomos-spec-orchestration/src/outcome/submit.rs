//! What `nomos request submit` produced, or why it did not.

use nomos_spec_model::{Refusal, Submission};
use nomos_spec_store::{AcceptError, StoreError};

use crate::outcome::{RenderAnswer, RenderRefusal};

/// A submission accepted through the one door `OD-SPEC-009` decided, and where its
/// `subject-dossier` projection landed, if `--into` asked for one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmitAnswer
{
    /// The submission as it was accepted -- every field carrying the origin it was given.
    pub submission: Submission,
    /// The row identifier the store assigned it.
    pub uid: i64,
    /// Both halves of its `subject-dossier` projection, placed where the run asked, or
    /// nothing when no `--into` was given.
    pub written: Option<RenderAnswer>,
}

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
