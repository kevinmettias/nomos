//! [`CommittedPreviewResponse`], carried only by [`super::commit_response::CommitResponse`].

use nomos_spec_store::EditPreview;
use serde::Serialize;

use super::{BlockChangeResponse, IdentityChangeResponse, NormativeMovementResponse, RecordRelationResponse};

/// The structured fields [`EditPreview`]'s own accessors expose, built the same way
/// [`crate::spec::preview_response::PreviewResponse::Previewed`]'s payload is -- kept as its own
/// type rather than shared with it, since `Preview`'s own wire shape is already fixed by
/// `P13-API-SPEC-PREVIEW-SEAM` and this crate does not widen an already-shipped response.
#[derive(Debug, Serialize)]
pub struct CommittedPreviewResponse
{
    pub node_id: String,
    pub path: String,
    pub rename: Option<(String, String)>,
    pub markdown: String,
    pub blocks: Vec<BlockChangeResponse>,
    pub identity: Vec<IdentityChangeResponse>,
    pub relations_added: Vec<RecordRelationResponse>,
    pub relations_removed: Vec<RecordRelationResponse>,
    pub statements: Vec<NormativeMovementResponse>,
    pub wording_moved: bool,
    pub changes_nothing: bool,
}

impl CommittedPreviewResponse
{
    pub(crate) fn From(preview: &EditPreview) -> Self
    {
        return Self {
            node_id: preview.Node_Id().to_owned(),
            path: preview.Path().to_owned(),
            rename: preview.Rename().map(|(before, after)| return (before.to_owned(), after.to_owned())),
            markdown: preview.Markdown().to_owned(),
            blocks: preview.Blocks().iter().cloned().map(BlockChangeResponse::From).collect(),
            identity: preview.Identity().iter().cloned().map(IdentityChangeResponse::From).collect(),
            relations_added: preview.Relations_Added().iter().cloned().map(RecordRelationResponse::From).collect(),
            relations_removed: preview
                .Relations_Removed()
                .iter()
                .cloned()
                .map(RecordRelationResponse::From)
                .collect(),
            statements: preview.Statements().iter().cloned().map(NormativeMovementResponse::From).collect(),
            wording_moved: preview.Wording_Moved(),
            changes_nothing: preview.Changes_Nothing(),
        };
    }
}
