//! What each way [`super::CommitRefusal`] can happen carries beyond its shared `error`.

use nomos_spec_store::{CommitReport, EditPreview};
use std::path::PathBuf;

/// What each way [`super::CommitRefusal`] can happen carries beyond its shared `error`.
#[derive(Debug)]
pub enum Kind
{
    /// `--from` could not be read.
    Unreadable
    {
        path: PathBuf,
    },
    /// Staging or previewing the edit was refused, before there was anything to commit.
    Edit,
    /// The edit previewed cleanly and the store refused to commit it.
    ///
    /// Carries the preview: a caller that already described it to an author does not have to
    /// decide whether to describe it again from a bare [`nomos_spec_store::EditError`].
    Refused
    {
        preview: EditPreview,
    },
    /// The store accepted the transaction and its bytes could not be written where the
    /// record belongs.
    Unwritable
    {
        preview: EditPreview,
        report: CommitReport,
        path: PathBuf,
    },
}
