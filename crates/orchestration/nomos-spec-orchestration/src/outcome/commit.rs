//! What `nomos spec commit` produced, or why it did not.

use nomos_platform::FileSystemError;
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

/// The path a rename left behind, and what became of removing it.
#[derive(Debug)]
pub struct Vacated
{
    /// The path the rename moved the record away from.
    pub path: PathBuf,
    /// What removing it did.
    pub outcome: VacateOutcome,
}

/// What became of the path a rename vacated.
///
/// Outside [`nomos_platform::FileSystem`] on purpose -- deletion is not one of the port's
/// three declared operations, so this is [`std::fs::remove_file`] directly. See
/// `crate::run::commit`'s own documentation for why that is the arrangement rather than a
/// defect.
#[derive(Debug)]
pub enum VacateOutcome
{
    /// The old path was removed.
    Removed,
    /// The old path was already gone.
    AlreadyGone,
    /// The old path could not be removed, so two files now declare this record.
    Failed(String),
}

/// Whether the store renders a just-committed record back as the bytes that were staged.
#[derive(Debug)]
pub enum Reproduction
{
    /// The store's own rendering matches what was staged, byte for byte.
    Matched
    {
        hash: String,
    },
    /// The store renders something else. The commit already happened; this says the round
    /// trip did not close.
    Mismatched
    {
        hash: String,
    },
}

/// Why `nomos spec commit` did not produce a [`CommitAnswer`].
///
/// Moved from `nomos-cli::spec::verb::editing`'s own error paths through `Commit`,
/// `Committed` and `Written`: which of these happened is a fact about the filesystem and the
/// store, not about how a terminal reports it.
#[derive(Debug)]
pub enum CommitRefusal
{
    /// `--from` could not be read.
    Unreadable
    {
        path: PathBuf,
        error: FileSystemError,
    },
    /// Staging or previewing the edit was refused, before there was anything to commit.
    Edit(EditError),
    /// The edit previewed cleanly and the store refused to commit it.
    ///
    /// Carries the preview: a caller that already described it to an author does not have to
    /// decide whether to describe it again from a bare [`EditError`].
    Refused
    {
        preview: EditPreview,
        error: EditError,
    },
    /// The store accepted the transaction and its bytes could not be written where the
    /// record belongs.
    Unwritable
    {
        preview: EditPreview,
        report: CommitReport,
        path: PathBuf,
        error: FileSystemError,
    },
}

impl From<crate::outcome::PreviewRefusal> for CommitRefusal
{
    /// `Commit` runs [`crate::run::Preview`] first, so every way staging or previewing can
    /// refuse is a way committing can refuse too, before either has written anything.
    fn from(refusal: crate::outcome::PreviewRefusal) -> Self
    {
        return match refusal
        {
            crate::outcome::PreviewRefusal::Unreadable { path, error } => Self::Unreadable { path, error },
            crate::outcome::PreviewRefusal::Edit(error) => Self::Edit(error),
        };
    }
}
