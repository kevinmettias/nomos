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
