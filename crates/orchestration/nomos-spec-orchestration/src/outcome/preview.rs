//! Why `nomos spec preview` did not produce a preview.

use nomos_platform::FileSystemError;
use nomos_spec_store::EditError;
use std::path::PathBuf;

/// Why `nomos spec preview` did not produce a preview.
///
/// Moved from `nomos-cli::spec::verb::editing`'s own `Staged_Text` and `Previewed`: which of
/// these happened is a fact about the filesystem and the store, not about how a terminal
/// reports it. `nomos spec preview`'s own success needs no wrapper of this crate's own --
/// [`nomos_spec_store::EditPreview`] already carries everything a preview says, the same
/// reuse [`crate::outcome::SpecOutcome::Markdown`] already makes for
/// [`nomos_spec_store::RecordProjection`].
#[derive(Debug)]
pub enum PreviewRefusal
{
    /// `--from` could not be read.
    Unreadable
    {
        path: PathBuf,
        error: FileSystemError,
    },
    /// The store refused the staged edit -- also carries a corpus-assembly failure, wrapped
    /// as [`EditError::Store`] the same way [`crate::outcome::SpecOutcome::Markdown`] already
    /// wraps one.
    Edit(EditError),
}
