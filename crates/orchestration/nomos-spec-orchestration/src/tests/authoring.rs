//! `D-129`'s authoring round trip: `Preview` describes a staged edit and writes nothing, and
//! `Commit` applies it as a transaction against the store.

use std::path::PathBuf;

use nomos_platform_std::StdFileSystem;

use crate::SpecCommand;
use crate::request::{CommitRequest, EditRequest, RecordRequest};
use crate::run::Run;
use crate::spec_outcome::{CommitAnswer, PreviewRefusal, Reproduction, SpecOutcome};

use super::{No_Corpus, Scratch};

/// A governing record, rendered from the store's own rows -- the same bytes
/// `Run(Markdown, ..)` would answer, fetched here so a preview/commit test can stage an
/// edited copy of something real rather than an invented fixture.
fn Governing_Markdown(id: &str) -> String
{
    let outcome = Run(
        &SpecCommand::Markdown(RecordRequest { id: id.to_owned(), revision: None }),
        &No_Corpus(),
        &StdFileSystem,
    );
    let SpecOutcome::Markdown(projection) = outcome
    else
    {
        panic!("Run(Markdown, ..) must answer SpecOutcome::Markdown");
    };

    return projection.expect("a governing record is embedded even with no corpus").markdown;
}

/// A canonical heading rename -- the same edit
/// `crates/host/nomos-cli/tests/authoring_surface.rs` stages against a real record on disk,
/// here staged against a governing record's own embedded copy so this crate's tests need no
/// corpus and touch no file this repository tracks.
fn Renamed_Decision_Heading(markdown: &str) -> String
{
    return markdown.replace("## Decision", "## The decision");
}

/// A staged heading rename: where the edited copy was written, and the edited text itself.
struct StagedHeading
{
    /// The path the edited copy was written to.
    path: PathBuf,
    /// The edited text, byte-for-byte as it was written.
    markdown: String,
}

/// Stages `Renamed_Decision_Heading(Governing_Markdown(id))` at `into`, as `staged.md`, and
/// hands back both the path it was written to and the edited text -- so a preview test can
/// check the staging and a commit test can also check the bytes landed unchanged.
fn Staged_Heading_Rename(id: &str, into: &std::path::Path) -> StagedHeading
{
    let markdown = Renamed_Decision_Heading(&Governing_Markdown(id));
    let path = into.join("staged.md");
    std::fs::write(&path, &markdown).expect("writes the staged edit");

    return StagedHeading { path, markdown };
}

#[test]
fn Test_Run_Of_Preview_Should_Describe_A_Staged_Edit_And_Write_Nothing()
{
    let staged = Staged_Heading_Rename("D-132", &Scratch("preview")).path;

    let outcome = Run(
        &SpecCommand::Preview(EditRequest {
            id: "D-132".to_owned(),
            from: staged,
            rename: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Preview(preview) = outcome
    else
    {
        panic!("Run(Preview, ..) must answer SpecOutcome::Preview");
    };
    let preview = preview.expect("a canonical heading rename previews cleanly");

    assert!(preview.Is_Wording_Moved(), "a heading rename must count as wording moved");
    assert!(preview.Describe().contains("normative wording moved"), "{}", preview.Describe());
}

#[test]
fn Test_Run_Of_Preview_Should_Refuse_A_Staged_File_That_Cannot_Be_Read()
{
    let missing = PathBuf::from("no-such-staged-file-anywhere.md");
    let outcome = Run(
        &SpecCommand::Preview(EditRequest {
            id: "D-132".to_owned(),
            from: missing.clone(),
            rename: None,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Preview(Err(PreviewRefusal::Unreadable { path, .. })) = outcome
    else
    {
        panic!("a --from naming nothing must refuse PreviewRefusal::Unreadable");
    };
    assert_eq!(path, missing);
}

/// Runs `Run(Commit, ..)` over a staged edit already written at `staged`, into `into`, and
/// unwraps down to the [`CommitAnswer`] a clean commit produces.
fn Committed(id: &str, staged: PathBuf, into: PathBuf) -> CommitAnswer
{
    let outcome = Run(
        &SpecCommand::Commit(CommitRequest {
            edit: EditRequest {
                id: id.to_owned(),
                from: staged,
                rename: None,
            },
            into,
        }),
        &No_Corpus(),
        &StdFileSystem,
    );

    let SpecOutcome::Commit(answer) = outcome
    else
    {
        panic!("Run(Commit, ..) must answer SpecOutcome::Commit");
    };

    return answer.expect("a canonical heading rename commits cleanly");
}

#[test]
fn Test_Run_Of_Commit_Should_Write_The_Record_And_Close_The_Round_Trip()
{
    let into = Scratch("commit");
    let staged = Staged_Heading_Rename("D-132", &into);

    let answer = Committed("D-132", staged.path, into);

    assert_eq!(answer.report.node_id, "D-132");
    assert!(answer.vacated.is_none(), "this edit did not rename the record's path");
    let written = std::fs::read_to_string(&answer.destination).expect("the record was written");
    assert_eq!(written, staged.markdown, "the bytes on disk must be exactly what was staged");
    assert!(
        matches!(answer.reproduction, Ok(Reproduction::Matched { .. })),
        "the round trip must close: {:?}",
        answer.reproduction
    );
}
