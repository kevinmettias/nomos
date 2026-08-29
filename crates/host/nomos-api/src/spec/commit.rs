//! [`Handle_Spec_Commit`] and its own [`SpecCommitResponse`].

use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::{
    CommitAnswer, CommitRefusal, CommitRefusalError, CommitRefusalKind, CommitRequest, SpecCommand,
};
use serde::Serialize;
use std::path::PathBuf;

use super::{Build_Corpus_Request, CommitReportResponse, CommittedPreviewResponse, ReproductionResponse, VacatedResponse};

/// Previews a staged edit, commits it to the store, and writes it where its own path says,
/// exactly as `nomos spec commit` would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::record::Handle_Spec_Record`]'s own composition. Like
/// [`crate::spec::render::Handle_Spec_Render`], this verb writes real bytes through
/// `StdFileSystem` -- `run::commit::Commit` writes the committed record at
/// `request.into.join(&report.path)` via `Replace_Atomically`, the same shape `Render`'s own
/// write already has, and this crate already has real, unauthenticated `StdFileSystem` writes
/// as precedent (`Handle_Work_Claim`, `Handle_Spec_Render`). Unlike `Render`, a commit's write
/// is not a derived, regenerable artifact: it replaces the governing record's own bytes, and
/// a rename's old path is permanently unlinked via a raw `std::fs::remove_file` with no
/// port-level guard -- `run::commit`'s own documentation names the known failure mode as "two
/// files now declare this record" when that removal fails.
#[must_use]
pub fn Handle_Spec_Commit(request: &CommitRequest) -> SpecCommitResponse
{
    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Commit(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Commit(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecCommitResponse::From(result);
}

/// What a real `nomos spec commit` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecCommitResponse
{
    /// The store accepted the transaction and its bytes were written where the record
    /// belongs.
    Committed
    {
        /// What committing this edit changed, as `Preview` found it.
        preview: CommittedPreviewResponse,
        /// What the store's own transaction changed.
        report: CommitReportResponse,
        /// Where the record's bytes were written.
        destination: PathBuf,
        /// The path a rename left behind, and what became of removing it.
        vacated: Option<VacatedResponse>,
        /// Whether the store renders the just-committed record back as the same bytes.
        reproduction: ReproductionResponse,
    },
    /// `--from` could not be read.
    Unreadable
    {
        path: String,
        cause: String,
    },
    /// Staging or previewing the edit was refused, before there was anything to commit.
    Refused
    {
        cause: String,
    },
    /// The edit previewed cleanly and the store refused to commit it.
    StoreRefused
    {
        preview: CommittedPreviewResponse,
        cause: String,
    },
    /// The store accepted the transaction and its bytes could not be written where the
    /// record belongs.
    Unwritable
    {
        preview: CommittedPreviewResponse,
        report: CommitReportResponse,
        path: String,
        cause: String,
    },
}

impl SpecCommitResponse
{
    pub(crate) fn From(result: Result<CommitAnswer, CommitRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Committed {
                preview: CommittedPreviewResponse::From(&answer.preview),
                report: CommitReportResponse::From(answer.report),
                destination: answer.destination,
                vacated: answer.vacated.map(VacatedResponse::From),
                reproduction: ReproductionResponse::From(answer.reproduction),
            },
            Err(CommitRefusal { kind: CommitRefusalKind::Unreadable { path }, error }) =>
            {
                Self::Unreadable { path: path.display().to_string(), cause: Refusal_Cause(error) }
            }
            Err(CommitRefusal { kind: CommitRefusalKind::Edit, error }) => Self::Refused { cause: Refusal_Cause(error) },
            Err(CommitRefusal { kind: CommitRefusalKind::Refused { preview }, error }) => Self::StoreRefused {
                preview: CommittedPreviewResponse::From(&preview),
                cause: Refusal_Cause(error),
            },
            Err(CommitRefusal { kind: CommitRefusalKind::Unwritable { preview, report, path }, error }) =>
            {
                Self::Unwritable {
                    preview: CommittedPreviewResponse::From(&preview),
                    report: CommitReportResponse::From(report),
                    path: path.display().to_string(),
                    cause: Refusal_Cause(error),
                }
            }
        };
    }
}

/// `CommitRefusalError`'s own `Display`-equivalent: it wraps one of two error types, each
/// with its own `Display`, and does not implement the trait itself.
fn Refusal_Cause(error: CommitRefusalError) -> String
{
    return match error
    {
        CommitRefusalError::FileSystem(error) => error.to_string(),
        CommitRefusalError::Edit(error) => error.to_string(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::spec::SpecMarkdownResponse;
    use nomos_spec_orchestration::EditRequest;

    /// An empty, unique scratch directory of this test's own.
    fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
    {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let directory = std::env::temp_dir().join(format!(
            "nomos-api-spec-commit-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a fresh scratch directory can always be created");

        return directory;
    }

    /// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
    /// through this crate's own `Handle_Spec_Markdown`, so this test needs no corpus and
    /// touches no file this repository tracks), writes it to `staged.md` under `into`, and
    /// hands back the path it was written to. Mirrors `crate::spec::preview::tests`'s own
    /// copy of the same fixture -- kept apart rather than shared for the reason this crate's
    /// own composition-root walks already are (`crate::sources`'s own doc): a test fixture
    /// pinned to one file is a composition-root concern of that file's own test module.
    fn Staged_Heading_Rename(id: &str, into: &std::path::Path) -> std::path::PathBuf
    {
        let request = nomos_spec_orchestration::RecordRequest { id: id.to_owned(), revision: None };
        let SpecMarkdownResponse::Resolved { markdown, .. } = crate::spec::Handle_Spec_Markdown(&request)
        else
        {
            panic!("{id} is a governing record, embedded even with no corpus");
        };
        let edited = markdown.replace("## Decision", "## The decision");
        let staged = into.join("staged.md");
        std::fs::write(&staged, &edited).expect("writes the staged edit");

        return staged;
    }

    #[test]
    fn Test_A_Real_Commit_Should_Write_The_Record_And_Close_The_Round_Trip()
    {
        let into = Unique_Scratch_Directory("commit");
        let staged = Staged_Heading_Rename("D-132", &into);
        let edited = std::fs::read_to_string(&staged).expect("the staged file was just written");
        let request = CommitRequest { edit: EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, into };

        let response = Handle_Spec_Commit(&request);

        let SpecCommitResponse::Committed { report, destination, vacated, reproduction, .. } = response
        else
        {
            panic!("a canonical heading rename commits cleanly: {response:?}");
        };
        assert_eq!(report.node_id, "D-132");
        assert!(vacated.is_none(), "this edit did not rename the record's path");
        let written = std::fs::read_to_string(&destination).expect("the record was written");
        assert_eq!(written, edited, "the bytes on disk must be exactly what was staged");
        assert!(matches!(reproduction, ReproductionResponse::Matched { .. }), "{reproduction:?}");
    }

    #[test]
    fn Test_A_Missing_Staged_Commit_File_Should_Report_Unreadable()
    {
        let request = CommitRequest {
            edit: EditRequest {
                id: "D-132".to_owned(),
                from: std::path::PathBuf::from("no-such-staged-file-anywhere.md"),
                rename: None,
            },
            into: Unique_Scratch_Directory("commit-unreadable"),
        };

        let response = Handle_Spec_Commit(&request);

        assert!(matches!(response, SpecCommitResponse::Unreadable { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Committed_Response_Should_Round_Trip_As_Json()
    {
        let into = Unique_Scratch_Directory("commit-json");
        let staged = Staged_Heading_Rename("D-132", &into);
        let request = CommitRequest { edit: EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, into };

        let response = Handle_Spec_Commit(&request);

        let json = serde_json::to_string(&response).expect("a SpecCommitResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecCommitResponse always has this field");

        assert_eq!(outcome, "committed", "{json}");
    }
}
