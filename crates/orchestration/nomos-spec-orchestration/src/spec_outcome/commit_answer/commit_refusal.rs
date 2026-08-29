//! Why `nomos spec commit` did not produce a [`CommitAnswer`].

mod commit_refusal_error;
mod commit_refusal_kind;

pub use commit_refusal_error::CommitRefusalError;
pub use commit_refusal_kind::CommitRefusalKind;

use nomos_platform::FileSystemError;
use nomos_spec_store::{CommitReport, EditError, EditPreview};
use std::path::PathBuf;

/// Why `nomos spec commit` did not produce a [`crate::spec_outcome::CommitAnswer`].
///
/// Moved from `nomos-cli::spec::verb::editing`'s own error paths through `Commit`,
/// `Committed` and `Written`: which of these happened is a fact about the filesystem and the
/// store, not about how a terminal reports it.
///
/// `error` is the type's own field, not any one variant's: every way this refuses IS an
/// error, from one of two places -- the filesystem, or the store's own edit staging -- and
/// [`CommitRefusalKind`] carries only what genuinely differs between the four ways of
/// getting there. [`CommitRefusalKind::Edit`] carries nothing beyond that shared error,
/// because staging refused before there was anything else about the attempt to say.
#[derive(Debug)]
pub struct CommitRefusal
{
    pub kind: CommitRefusalKind,
    pub error: CommitRefusalError,
}

impl CommitRefusal
{
    /// `--from` could not be read.
    #[must_use]
    pub fn Unreadable(path: PathBuf, error: FileSystemError) -> Self
    {
        return Self {
            kind: CommitRefusalKind::Unreadable { path },
            error: CommitRefusalError::FileSystem(error),
        };
    }

    /// Staging or previewing the edit was refused, before there was anything to commit.
    #[must_use]
    pub fn Edit(error: EditError) -> Self
    {
        return Self {
            kind: CommitRefusalKind::Edit,
            error: CommitRefusalError::Edit(error),
        };
    }

    /// The edit previewed cleanly and the store refused to commit it.
    #[must_use]
    pub fn Refused(preview: EditPreview, error: EditError) -> Self
    {
        return Self {
            kind: CommitRefusalKind::Refused { preview },
            error: CommitRefusalError::Edit(error),
        };
    }

    /// The store accepted the transaction and its bytes could not be written where the
    /// record belongs.
    #[must_use]
    pub fn Unwritable(preview: EditPreview, report: CommitReport, path: PathBuf, error: FileSystemError) -> Self
    {
        return Self {
            kind: CommitRefusalKind::Unwritable { preview, report, path },
            error: CommitRefusalError::FileSystem(error),
        };
    }
}

impl From<crate::spec_outcome::PreviewRefusal> for CommitRefusal
{
    /// `Commit_Staged_Edit` runs [`crate::run::Preview_Staged_Edit`] first, so every way staging or previewing can
    /// refuse is a way committing can refuse too, before either has written anything.
    fn from(refusal: crate::spec_outcome::PreviewRefusal) -> Self
    {
        return match refusal
        {
            crate::spec_outcome::PreviewRefusal::Unreadable { path, error } => Self::Unreadable(path, error),
            crate::spec_outcome::PreviewRefusal::Edit(error) => Self::Edit(error),
        };
    }
}
