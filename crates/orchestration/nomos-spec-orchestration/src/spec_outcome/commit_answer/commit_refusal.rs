//! Why `nomos spec commit` did not produce a [`CommitAnswer`].

mod error;
mod kind;

pub use error::Error as CommitRefusalError;
pub use kind::Kind as CommitRefusalKind;

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

#[cfg(test)]
mod tests
{
    use super::{CommitRefusal, CommitRefusalError, CommitRefusalKind};
    use nomos_platform::FileSystemError;
    use nomos_spec_store::{CommitReport, EditError};
    use std::path::PathBuf;

    /// A real, staged edit preview -- the same D-132 heading rename `run::commit`'s own
    /// colocated tests build -- so `Refused` and `Unwritable` carry a genuine `EditPreview`
    /// rather than a value nothing outside this module could construct.
    fn Real_Preview() -> nomos_spec_store::EditPreview
    {
        let request = crate::corpus::CorpusRequest {
            variable: "A_COMMIT_REFUSAL_TEST_CORPUS_VARIABLE".to_owned(),
            root: None,
            revision: "v14.36".to_owned(),
        };
        let assembly = crate::corpus::Assemble_Corpus(&request).expect("assembles from the embedded records alone");
        let markdown = crate::run::Rendered_Markdown(&assembly, &crate::request::RecordRequest { id: "D-132".to_owned(), revision: None })
            .expect("D-132 is embedded")
            .markdown;
        let edited = markdown.replace("## Decision", "## The decision");

        return assembly
            .store
            .Claim_For_Edit("D-132", None)
            .and_then(|claimed| return claimed.Stage(&edited, None))
            .and_then(|edit| return edit.Preview(&assembly.store))
            .expect("a canonical heading rename previews cleanly");
    }

    #[test]
    fn Test_Unreadable_Should_Carry_The_Path_And_Error()
    {
        let path = PathBuf::from("no-such-staged-file.md");
        let error = FileSystemError::NotFound { path: path.display().to_string() };

        let refusal = CommitRefusal::Unreadable(path.clone(), error);

        assert!(matches!(refusal.kind, CommitRefusalKind::Unreadable { path: kind_path } if kind_path == path));
        assert!(matches!(refusal.error, CommitRefusalError::FileSystem(_)));
    }

    #[test]
    fn Test_Edit_Should_Carry_The_Underlying_Edit_Error()
    {
        let refusal = CommitRefusal::Edit(EditError::NoSuchRecord { node_id: "D-9999".to_owned() });

        assert!(matches!(refusal.kind, CommitRefusalKind::Edit));
        assert!(matches!(refusal.error, CommitRefusalError::Edit(_)));
    }

    #[test]
    fn Test_Refused_Should_Carry_The_Preview_The_Store_Refused()
    {
        let preview = Real_Preview();
        let error = EditError::NoSuchRecord { node_id: "D-132".to_owned() };

        let refusal = CommitRefusal::Refused(preview, error);

        assert!(matches!(refusal.kind, CommitRefusalKind::Refused { .. }));
        assert!(matches!(refusal.error, CommitRefusalError::Edit(_)));
    }

    #[test]
    fn Test_Unwritable_Should_Carry_The_Preview_Report_And_Path()
    {
        let preview = Real_Preview();
        let report = CommitReport {
            node_id: "D-132".to_owned(),
            path: "d-132.md".to_owned(),
            blocks: 1,
            blocks_removed: 0,
            relations_added: 0,
            relations_removed: 0,
            renamed: false,
        };
        let path = PathBuf::from("into/d-132.md");
        let error = FileSystemError::NotFound { path: path.display().to_string() };

        let refusal = CommitRefusal::Unwritable(preview, report, path.clone(), error);

        assert!(matches!(refusal.kind, CommitRefusalKind::Unwritable { path: kind_path, .. } if kind_path == path));
        assert!(matches!(refusal.error, CommitRefusalError::FileSystem(_)));
    }
}
