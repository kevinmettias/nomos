//! [`Handle_Spec_Submit`] and its own [`SubmitResponse`].

use nomos_spec_orchestration::{RenderRefusal, SubmitAnswer, SubmitRefusal, SubmitRequest};
use serde::Serialize;

use super::{Build_Corpus_Request, RefusalResponse, RenderedProjectionResponse, SubmissionResponse};

/// Accepts a submission through the one door `OD-SPEC-009` decided, and places its
/// `subject-dossier` projection when `request.into` asks for one, exactly as `nomos request
/// submit` would, and hands back a JSON-serializable response.
///
/// `Submit_Corpus_Request` is not a `SpecCommand` variant -- `OD-HOST-005`'s own resolution says a `nomos
/// request submit` invocation is not a `nomos spec` verb by the CLI's own naming, so
/// [`nomos_spec_orchestration::Submit_Corpus_Request`] is a sibling of `Run`, not one of its cases, and
/// takes an already-assembled `&mut Assembly` directly rather than being dispatched through
/// `Run`. This function therefore assembles the store itself, the one step every other
/// `Handle_Spec_*` function in this crate gets from `Run`. Writes real bytes through
/// `StdFileSystem` when `request.into` is given, the same `Render`-shaped write
/// [`crate::spec::render_response::Handle_Spec_Render`] already performs (`Submit_Corpus_Request` calls
/// `run::render::Rendered_Projection` internally for exactly that reason, addressed at the submission's
/// own id under the same `into` root).
#[must_use]
pub fn Handle_Spec_Submit(request: &SubmitRequest) -> SubmitResponse
{
    use nomos_platform_std::StdFileSystem;
    use nomos_spec_orchestration::corpus::Assemble_Corpus;

    let corpus_request = Build_Corpus_Request();

    let mut assembly = match Assemble_Corpus(&corpus_request)
    {
        Ok(assembly) => assembly,
        Err(error) => return SubmitResponse::Unreadable { cause: error.to_string() },
    };

    let submitted = nomos_spec_orchestration::Submit_Corpus_Request(&mut assembly, request, &StdFileSystem);
    return SubmitResponse::From(submitted);
}

/// What a real `nomos request submit` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SubmitResponse
{
    /// The submission was accepted, and its projection placed if one was asked for.
    Accepted
    {
        /// The submission as it was accepted -- every field carrying the origin it was given.
        submission: SubmissionResponse,
        /// The row identifier the store assigned it.
        uid: i64,
        /// Both halves of its `subject-dossier` projection, when `request.into` asked for
        /// one.
        written: Option<RenderedProjectionResponse>,
    },
    /// The store could not be assembled or used at all.
    Unreadable
    {
        /// What went wrong, as the underlying error's own `Display` renders it.
        cause: String,
    },
    /// The submission failed `OD-SPEC-010`'s rule set. Nothing was stored.
    Refused
    {
        refusal: RefusalResponse,
    },
    /// The submission was accepted and its `subject-dossier` projection could not be built
    /// or placed.
    WrittenRefused
    {
        /// What went wrong, as the underlying `RenderRefusal`'s own parts render.
        cause: String,
    },
}

impl SubmitResponse
{
    pub(crate) fn From(result: Result<SubmitAnswer, SubmitRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Accepted {
                submission: SubmissionResponse::From(answer.submission),
                uid: answer.uid,
                written: answer.written.map(RenderedProjectionResponse::From),
            },
            Err(SubmitRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
            Err(SubmitRefusal::Refused(refusal)) => Self::Refused { refusal: RefusalResponse::From(refusal) },
            Err(SubmitRefusal::Written(refusal)) => Self::WrittenRefused { cause: Render_Refusal_Cause(refusal) },
        };
    }
}

/// A readable cause from any [`RenderRefusal`] variant. `RenderRefusal` itself has no
/// `Display` (only `Debug`); its own parts do.
fn Render_Refusal_Cause(refusal: RenderRefusal) -> String
{
    return match refusal
    {
        RenderRefusal::NoSuchProfile { requested, known } =>
        {
            format!("no profile named {requested} (known: {})", known.join(", "))
        }
        RenderRefusal::Project(error) => error.to_string(),
        RenderRefusal::Unwritable { path, error } => format!("{}: {error}", path.display()),
        RenderRefusal::Store(error) => error.to_string(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Assert_Round_Trips_As_Json, Unique_Scratch_Directory};
    use nomos_spec_model::{SubmissionKind, SubmissionState};

    #[test]
    fn Test_Handle_Spec_Submit_Should_Accept_A_Complete_Submission_With_Submitted_Origin()
    {
        let request = Complete_Feature_Request("FR-API-001", None);

        let response = Handle_Spec_Submit(&request);

        let SubmitResponse::Accepted { submission, written, .. } = response
        else
        {
            // This request carries every universal field and every field OD-SPEC-010
            // requires of a FeatureRequest, so reaching any other variant here means the
            // submission path itself regressed, not a condition this test should assert
            // around.
            panic!("a complete feature request is accepted: {response:?}");
        };
        assert_eq!(submission.submitted_through, "nomos-api-test");
        assert!(
            submission
                .values
                .iter()
                .all(|value| matches!(value.origin, crate::spec::OriginResponse::Submitted))
        );
        assert!(written.is_none(), "no into was given");
    }

    #[test]
    fn Test_A_Real_Submission_With_Into_Should_Place_Its_Subject_Dossier_Projection()
    {
        let into = Unique_Scratch_Directory("spec-submit", "submit");
        let request = Complete_Feature_Request("FR-API-002", Some(into));

        let response = Handle_Spec_Submit(&request);

        let SubmitResponse::Accepted { written, .. } = response
        else
        {
            // This request carries every universal field and every field OD-SPEC-010
            // requires of a FeatureRequest, so reaching any other variant here means the
            // submission path itself regressed, not a condition this test should assert
            // around.
            panic!("a complete feature request is accepted: {response:?}");
        };
        let written = written.expect("into was given");
        assert_eq!(written.id, "subject-dossier");
        let body = std::fs::read_to_string(&written.body).expect("the body was written");
        assert!(body.contains("FR-API-002"), "{body}");
    }

    #[test]
    fn Test_An_Incomplete_Submission_Should_Be_Refused_And_Write_Nothing()
    {
        let mut request = Complete_Feature_Request("FR-API-003", None);
        request.fields.truncate(1);

        let response = Handle_Spec_Submit(&request);

        let SubmitResponse::Refused { refusal } = response
        else
        {
            // This request was truncated to just its first field, dropping the ones
            // OD-SPEC-010 requires (including `goal`), so reaching any other variant here
            // means the completeness rule itself stopped enforcing, not a condition this
            // test should assert around.
            panic!("an incomplete submission must be refused: {response:?}");
        };
        assert!(refusal.failures.iter().any(|failure| failure.field == "goal"), "{refusal:?}");
    }

    #[test]
    fn Test_From_Should_Round_Trip_As_Json()
    {
        let request = Complete_Feature_Request("FR-API-004", None);

        let response = Handle_Spec_Submit(&request);

        Assert_Round_Trips_As_Json(&response, "accepted");
    }

    /// Every universal field and every field `OD-SPEC-010` requires of `SubmissionKind::
    /// FeatureRequest` -- the same set `nomos_spec_orchestration`'s own `tests.rs` submits.
    fn Complete_Feature_Request(id: &str, into: Option<std::path::PathBuf>) -> SubmitRequest
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
            submitted_through: "nomos-api-test".to_owned(),
            into,
        };
    }
}
