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
