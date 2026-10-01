//! `Submit`, a sibling of `Run` rather than a `SpecCommand` case -- see this crate's own
//! top-level documentation, "A second command group" -- so every test here assembles the
//! store itself, the same way `nomos-cli::request::Submit` does.

use std::path::PathBuf;

use nomos_platform_std::StdFileSystem;

use nomos_spec_model::{Origin, SubmissionKind, SubmissionState};

use crate::corpus::Assemble_Corpus;
use crate::request::SubmitRequest;
use crate::run::Submit_Corpus_Request;
use crate::spec_outcome::SubmitRefusal;

use super::{No_Corpus, Scratch_Root};

/// Every universal field and every field `OD-SPEC-010` requires of `SubmissionKind::
/// FeatureRequest`, the same set `crates/host/nomos-cli/tests/request_submit.rs` submits
/// through the CLI's own surface.
fn Complete_Feature_Request(id: &str, into: Option<PathBuf>) -> SubmitRequest
{
    return SubmitRequest {
        kind: SubmissionKind::FeatureRequest,
        id: id.to_owned(),
        by: "kevin".to_owned(),
        state: SubmissionState::Draft,
        contract_version: 1,
        fields: vec![
            ("title".to_owned(), "t".to_owned()),
            ("goal".to_owned(), "g".to_owned()),
            ("behaviour".to_owned(), "b".to_owned()),
            ("acceptance".to_owned(), "a".to_owned()),
            ("invariants".to_owned(), "none".to_owned()),
        ],
        gaps: Vec::new(),
        submitted_through: "test".to_owned(),
        into,
    };
}

#[test]
fn Test_Submit_Should_Accept_A_Complete_Submission_With_Submitted_Origin()
{
    let mut assembly = Assemble_Corpus(&No_Corpus()).expect("assembles from the embedded records alone");

    let request = Complete_Feature_Request("FR-ORCH-001", None);
    let answer =
        Submit_Corpus_Request(&mut assembly, &request, &StdFileSystem).expect("a complete feature request is accepted");

    assert_eq!(answer.submission.submitted_through, "test");
    assert!(answer.submission.values.iter().all(|value| return value.origin == Origin::Submitted));
    assert!(answer.written.is_none(), "no --into was given");
}

#[test]
fn Test_Submit_Should_Place_Its_Subject_Dossier_Projection_When_Into_Is_Given()
{
    let into = Scratch_Root("submit");
    let mut assembly = Assemble_Corpus(&No_Corpus()).expect("assembles from the embedded records alone");

    let request = Complete_Feature_Request("FR-ORCH-002", Some(into));
    let answer =
        Submit_Corpus_Request(&mut assembly, &request, &StdFileSystem).expect("a complete feature request is accepted");
    let written = answer.written.expect("--into was given");

    assert_eq!(written.id, "subject-dossier");
    let body = std::fs::read_to_string(&written.body).expect("the body was written");
    assert!(body.contains("FR-ORCH-002"), "{body}");
}

#[test]
fn Test_Submit_Should_Refuse_An_Incomplete_Submission_And_Write_Nothing()
{
    let mut request = Complete_Feature_Request("FR-ORCH-003", None);
    request.fields.truncate(1);
    let mut assembly = Assemble_Corpus(&No_Corpus()).expect("assembles from the embedded records alone");

    let SubmitRefusal::Refused(refusal) = Submit_Corpus_Request(&mut assembly, &request, &StdFileSystem)
        .expect_err("an incomplete submission must be refused")
    else
    {
        panic!("an incomplete submission must refuse SubmitRefusal::Refused");
    };
    assert!(format!("{refusal}").contains("goal"), "{refusal}");
}
