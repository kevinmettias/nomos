//! [`Handle_Spec_Commit`] and its own [`CommitResponse`].

use nomos_spec_orchestration::{
    CommitAnswer, CommitRefusal, CommitRefusalError, CommitRefusalKind, CommitRequest, SpecCommand,
};
use serde::Serialize;
use std::path::PathBuf;

use super::{Build_Corpus_Request, CommitReportResponse, CommittedPreviewResponse, ReproductionResponse, VacatedResponse};

/// Previews a staged edit, commits it to the store, and writes it where its own path says,
/// exactly as `nomos spec commit` would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::record_response::Handle_Spec_Record`]'s own composition. Like
/// [`crate::spec::render_response::Handle_Spec_Render`], this verb writes real bytes through
/// `nomos_composer_std::FILE_SYSTEM` -- `run::commit::Commit_Staged_Edit` writes the committed record at
/// `request.into.join(&report.path)` via `Replace_Atomically`, the same shape `Render`'s own
/// write already has, and this crate already has real, unauthenticated filesystem writes
/// as precedent (`Handle_Work_Claim`, `Handle_Spec_Render`). Unlike `Render`, a commit's write
/// is not a derived, regenerable artifact: it replaces the governing record's own bytes, and
/// a rename's old path is permanently unlinked via a raw `std::fs::remove_file` with no
/// port-level guard -- `run::commit`'s own documentation names the known failure mode as "two
/// files now declare this record" when that removal fails.
#[must_use]
pub fn Handle_Spec_Commit(request: &CommitRequest) -> CommitResponse
{
    use nomos_composer_std::FILE_SYSTEM;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Commit(request.clone()), &corpus_request, &FILE_SYSTEM);

    let nomos_spec_orchestration::SpecOutcome::Commit(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return CommitResponse::From(result);
}

/// What a real `nomos spec commit` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum CommitResponse
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

impl CommitResponse
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
    use crate::test_support::{Assert_Round_Trips_As_Json, Staged_Heading_Rename, Unique_Scratch_Directory};
    use nomos_spec_orchestration::EditRequest;

    #[test]
    fn Test_Handle_Spec_Commit_Should_Write_The_Record_And_Close_The_Round_Trip()
    {
        let into = Unique_Scratch_Directory("spec-commit", "commit");
        let staged = Staged_Heading_Rename("D-132", &into);
        let edited = std::fs::read_to_string(&staged).expect("the staged file was just written");
        let request = CommitRequest { edit: EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, into };

        let response = Handle_Spec_Commit(&request);

        let CommitResponse::Committed { report, destination, vacated, reproduction, .. } = response
        else
        {
            // A canonical heading rename against a real, freshly staged file has nothing to
            // refuse: reaching any other variant here means the commit path itself regressed,
            // not a condition this test should assert around.
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
            into: Unique_Scratch_Directory("spec-commit", "commit-unreadable"),
        };

        let response = Handle_Spec_Commit(&request);

        assert!(matches!(response, CommitResponse::Unreadable { .. }), "{response:?}");
    }

    #[test]
    fn Test_From_Should_Round_Trip_As_Json()
    {
        let into = Unique_Scratch_Directory("spec-commit", "commit-json");
        let staged = Staged_Heading_Rename("D-132", &into);
        let request = CommitRequest { edit: EditRequest { id: "D-132".to_owned(), from: staged, rename: None }, into };

        let response = Handle_Spec_Commit(&request);

        Assert_Round_Trips_As_Json(&response, "committed");
    }
}
