//! Rendering what `nomos-spec-orchestration::Submit_Corpus_Request` answered.

use nomos_spec_orchestration::{RenderRefusal, SubmitAnswer};
use nomos_spec_project::SIDECAR_SUFFIX;

use super::ExitCode;

/// A submission accepted, and its `subject-dossier` projection reported if one was written.
pub(super) fn Report_Accepted(answer: &SubmitAnswer, output: &mut impl std::io::Write) -> ExitCode
{
    let _ = writeln!(
        output,
        "accepted {} as {} ({}), uid {}",
        answer.submission.id,
        answer.submission.kind.Label(),
        answer.submission.state.Label(),
        answer.uid
    );

    if let Some(written) = &answer.written
    {
        let _ = writeln!(
            output,
            "{} -> {}\nsidecar ({SIDECAR_SUFFIX}) -> {}",
            written.id,
            written.body.display(),
            written.sidecar.display()
        );
    }

    return ExitCode::Ok;
}

/// The submission was accepted and its `subject-dossier` projection could not be built or
/// placed.
pub(super) fn Report_Unwritten(refusal: &RenderRefusal, notes: &mut impl std::io::Write) -> ExitCode
{
    return match refusal
    {
        RenderRefusal::Unwritable { path, error } =>
        {
            let _ = writeln!(notes, "cannot write {}: {error}", path.display());
            ExitCode::Unwritable
        }
        RenderRefusal::Store(error) =>
        {
            let _ = writeln!(notes, "{error}");
            ExitCode::StoreError
        }
        RenderRefusal::NoSuchProfile { requested, .. } =>
        {
            let _ = writeln!(notes, "the shipped catalogue no longer carries {requested}");
            ExitCode::StoreError
        }
        RenderRefusal::Project(error) =>
        {
            let _ = writeln!(notes, "{error}");
            ExitCode::StoreError
        }
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_orchestration::RenderAnswer;
    use nomos_spec_model::{Submission, SubmissionKind, SubmissionState};
    use nomos_spec_project::{Format, Stamp};
    use std::path::PathBuf;

    /// An accepted submission with no `--into` renders its id, kind, state and uid, and
    /// nothing about a projection that was never asked for.
    #[test]
    fn Test_Report_Accepted_Should_Name_The_Submission_Its_Kind_State_And_Uid()
    {
        let answer = SubmitAnswer { submission: Example_Submission(), uid: 42, written: None };
        let mut output = Vec::new();

        let code = Report_Accepted(&answer, &mut output);

        let rendered = String::from_utf8_lossy(&output).into_owned();
        assert_eq!(code, ExitCode::Ok, "{rendered}");
        assert!(rendered.contains("accepted FR-1 as feature-request (draft), uid 42"), "{rendered}");
        assert!(!rendered.contains("sidecar"), "{rendered}");
    }

    /// An accepted submission with a `--into` projection also names where its body and
    /// sidecar landed -- the half of [`Report_Accepted`] the no-projection test above cannot
    /// reach.
    #[test]
    fn Test_Report_Accepted_Should_Name_Where_The_Projection_Landed_When_One_Was_Written()
    {
        let written = RenderAnswer {
            id: "subject-dossier".to_owned(),
            body: PathBuf::from("out/FR-1.md"),
            sidecar: PathBuf::from("out/FR-1.sidecar.json"),
            stamp: Stamp {
                profile: "subject-dossier".to_owned(),
                profile_digest: "0".repeat(32),
                format: Format::Markdown,
                output: "out/FR-1.md".to_owned(),
                content_digest: "0".repeat(32),
                inputs_digest: "0".repeat(32),
                sections: Vec::new(),
                inputs: Vec::new(),
            },
        };
        let answer = SubmitAnswer { submission: Example_Submission(), uid: 7, written: Some(written) };
        let mut output = Vec::new();

        let code = Report_Accepted(&answer, &mut output);

        let rendered = String::from_utf8_lossy(&output).into_owned();
        assert_eq!(code, ExitCode::Ok, "{rendered}");
        assert!(rendered.contains("subject-dossier -> out/FR-1.md"), "{rendered}");
        assert!(rendered.contains(&format!("sidecar ({SIDECAR_SUFFIX}) -> out/FR-1.sidecar.json")), "{rendered}");
    }

    /// The shipped catalogue no longer carrying the requested profile is a store-shaped
    /// refusal, and the message must still name the profile a caller asked for.
    #[test]
    fn Test_Report_Unwritten_Should_Name_The_Missing_Profile()
    {
        let refusal = RenderRefusal::NoSuchProfile { requested: "subject-dossier".to_owned(), known: Vec::new() };
        let mut notes = Vec::new();

        let code = Report_Unwritten(&refusal, &mut notes);

        let rendered = String::from_utf8_lossy(&notes).into_owned();
        assert_eq!(code, ExitCode::StoreError, "{rendered}");
        assert!(rendered.contains("subject-dossier"), "{rendered}");
    }

    /// A submission with nothing about its own content load-bearing to either test above --
    /// only `id`, `kind` and `state` are ever rendered by [`Report_Accepted`].
    fn Example_Submission() -> Submission
    {
        return Submission {
            id: "FR-1".to_owned(),
            kind: SubmissionKind::FeatureRequest,
            form_contract_version: 1,
            state: SubmissionState::Draft,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values: Vec::new(),
            gaps: Vec::new(),
        };
    }
}
