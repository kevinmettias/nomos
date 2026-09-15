//! [`RenderedProjectionResponse`], carried only by [`super::submit_response::SubmitResponse::
//! Accepted`].

use nomos_spec_orchestration::RenderAnswer;
use nomos_spec_project::Stamp;
use serde::Serialize;
use std::path::PathBuf;

/// Both halves of a built projection, placed where a submission's own `into` asked for them
/// -- the same fields [`crate::spec::render_response::RenderResponse::Placed`] carries, kept as
/// its own type here rather than shared with it for the same reason
/// [`crate::spec::committed_preview_response::CommittedPreviewResponse`] is not shared with
/// `PreviewResponse`: `Render`'s own wire shape is already fixed by its own finished
/// item.
#[derive(Debug, Serialize)]
pub struct RenderedProjectionResponse
{
    /// The profile identifier that was built -- always `"subject-dossier"` here.
    pub id: String,
    /// Where the body landed.
    pub body: PathBuf,
    /// Where its sidecar landed.
    pub sidecar: PathBuf,
    /// What the build selected, and what it hashes to. Already `Serialize`, reused directly.
    pub stamp: Stamp,
}

impl RenderedProjectionResponse
{
    pub(crate) fn From(answer: RenderAnswer) -> Self
    {
        return Self { id: answer.id, body: answer.body, sidecar: answer.sidecar, stamp: answer.stamp };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_project::Format;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Of_The_Domain_Render_Answer()
    {
        let stamp = Built_Stamp();
        let answer = Render_Answer_For(stamp.clone());

        let response = RenderedProjectionResponse::From(answer.clone());

        assert_eq!(response.id, answer.id);
        assert_eq!(response.body, answer.body);
        assert_eq!(response.sidecar, answer.sidecar);
        assert_eq!(response.stamp.profile, stamp.profile);
    }

    /// The stamp a real `subject-dossier` build reports, with no sections and no inputs.
    fn Built_Stamp() -> Stamp
    {
        return Stamp {
            profile: "subject-dossier".to_owned(),
            profile_digest: "digest".to_owned(),
            format: Format::Markdown,
            output: "subject-dossier.md".to_owned(),
            content_digest: "content".to_owned(),
            inputs_digest: "inputs".to_owned(),
            sections: Vec::new(),
            inputs: Vec::new(),
        };
    }

    /// The domain answer over `stamp` that [`RenderedProjectionResponse::From`] maps.
    fn Render_Answer_For(stamp: Stamp) -> RenderAnswer
    {
        return RenderAnswer {
            id: "subject-dossier".to_owned(),
            body: PathBuf::from("into/subject-dossier.md"),
            sidecar: PathBuf::from("into/subject-dossier.stamp.json"),
            stamp,
        };
    }
}
