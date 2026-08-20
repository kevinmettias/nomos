//! What `nomos request submit` produced, or why it did not.

mod submit_refusal;

pub use submit_refusal::SubmitRefusal;

use nomos_spec_model::Submission;

use crate::outcome::RenderAnswer;

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
