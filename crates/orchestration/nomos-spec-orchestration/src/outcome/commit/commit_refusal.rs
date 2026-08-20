//! Why `nomos spec commit` did not produce a [`CommitAnswer`].

use nomos_platform::FileSystemError;
use nomos_spec_store::{CommitReport, EditError, EditPreview};
use std::path::PathBuf;

/// Why `nomos spec commit` did not produce a [`crate::outcome::CommitAnswer`].
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
