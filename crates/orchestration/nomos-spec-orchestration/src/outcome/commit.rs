//! What `nomos spec commit` produced, or why it did not.

mod commit_refusal;
mod reproduction;
mod vacate_outcome;
mod vacated;

pub use commit_refusal::CommitRefusal;
pub use reproduction::Reproduction;
pub use vacate_outcome::VacateOutcome;
pub use vacated::Vacated;

use nomos_spec_store::{CommitReport, EditError, EditPreview};
use std::path::PathBuf;

/// What committing an edit actually did: previewed, applied to the store, and written where
/// the record belongs.
///
/// Carries the preview it ran ([`CommitAnswer::preview`]) rather than making a caller ask for
/// one separately -- `commit` refuses to write an edit it has not previewed, and an author
/// reading a commit's own report reads the same preview `nomos spec preview` would have shown.
#[derive(Debug)]
pub struct CommitAnswer
{
    /// What committing this edit would change, as [`crate::run::Preview`] found it.
    pub preview: EditPreview,
    /// What the store's own transaction changed.
    pub report: CommitReport,
    /// Where the record's bytes were written.
    pub destination: PathBuf,
    /// The path a rename left behind, and what became of removing it -- `None` when this
    /// commit did not rename the record.
    pub vacated: Option<Vacated>,
    /// Whether the store renders the just-committed record back as the same bytes.
    ///
    /// `Err` when the store could not be asked at all -- a defect in this run rather than in
    /// the edit, since the transaction that produced this answer already committed.
    pub reproduction: Result<Reproduction, EditError>,
}
