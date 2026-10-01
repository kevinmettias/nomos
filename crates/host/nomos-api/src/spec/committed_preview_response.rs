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
            wording_moved: preview.Is_Wording_Moved(),
            changes_nothing: preview.Has_No_Changes(),
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::{Area, Staged_Heading_Rename, Unique_Scratch_Directory};
    use nomos_composer_std::FILE_SYSTEM;
    use nomos_spec_orchestration::{EditRequest, SpecCommand};

    /// The area this module's one scratch path is named under.
    const COMMITTED_PREVIEW_AREA: Area = Area("committed-preview-response");

    /// Mirrors [`crate::spec::preview_response::Handle_Spec_Preview`]'s own fixture: a real
    /// heading rename staged against `D-132`, a real embedded governing record, so this needs
    /// no corpus. `EditPreview`'s own fields are `pub(crate)` to `nomos-spec-store`, so this is
    /// the only way to hand this function a real one from outside that crate.
    #[test]
    fn Test_From_Should_Carry_Wording_Moved_For_A_Real_Heading_Rename()
    {
        let into = Unique_Scratch_Directory(COMMITTED_PREVIEW_AREA, "from")
            .expect("the temp directory is writable and this call's own name is fresh");
        let staged = Staged_Heading_Rename("D-132", &into)
            .expect("D-132 is a governing record embedded in this binary");
        let request = EditRequest { id: "D-132".to_owned(), from: staged, rename: None };
        let corpus_request = crate::spec::Build_Corpus_Request();

        let outcome = nomos_spec_orchestration::Run(&SpecCommand::Preview(request), &corpus_request, &FILE_SYSTEM);

        let nomos_spec_orchestration::SpecOutcome::Preview(Ok(preview)) = outcome
        else
        {
            // A canonical heading rename against a real, freshly staged file has nothing to
            // refuse: reaching any other variant here means the preview path itself
            // regressed, not a condition this test should assert around.
            panic!("a canonical heading rename previews cleanly");
        };

        let response = CommittedPreviewResponse::From(&preview);

        assert_eq!(response.node_id, "D-132");
        assert!(response.wording_moved, "a heading rename must count as wording moved");
    }
}
