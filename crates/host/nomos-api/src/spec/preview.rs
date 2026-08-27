//! [`Handle_Spec_Preview`] and its own [`SpecPreviewResponse`].

use nomos_platform_std::StdFileSystem;
use nomos_spec_orchestration::{EditRequest, PreviewRefusal, SpecCommand};
use nomos_spec_store::EditPreview;
use serde::Serialize;

use super::{
    Build_Corpus_Request, BlockChangeResponse, IdentityChangeResponse, NormativeMovementResponse,
    RecordRelationResponse,
};

/// Says what committing a staged edit would change, without writing anything, exactly as
/// `nomos spec preview` would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::record::Handle_Spec_Record`]'s own composition. `run::preview::
/// Preview` is generic over `FileSystem` (it reads `request.from` through `nomos_platform::
/// FileSystem::Read_To_String`), but that is its only contact with a filesystem --
/// everything else is an in-memory `assembly.store` operation. Unlike `Render` and `Commit`,
/// which place or overwrite files at a caller-known output path, `Preview` never writes, so
/// exposing it over nomos-api carries none of the "does a wire call write to this host's
/// disk" hazard those two do -- the same reasoning [`crate::spec::freshness::
/// Handle_Spec_Freshness`] already gives for its own `FileSystem` parameter.
#[must_use]
pub fn Handle_Spec_Preview(request: &EditRequest) -> SpecPreviewResponse
{
    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Preview(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Preview(result) = outcome
    else
    {
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecPreviewResponse::From(result);
}

/// What a real `nomos spec preview` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecPreviewResponse
{
    /// What committing the staged edit would change.
    Previewed
    {
        node_id: String,
        path: String,
        /// The paths before and after, when this edit is also a rename.
        rename: Option<(String, String)>,
        /// The staged bytes.
        markdown: String,
        blocks: Vec<BlockChangeResponse>,
        identity: Vec<IdentityChangeResponse>,
        relations_added: Vec<RecordRelationResponse>,
        relations_removed: Vec<RecordRelationResponse>,
        statements: Vec<NormativeMovementResponse>,
        /// The question `D-129` calls mandatory: does this edit move wording that was there?
        wording_moved: bool,
        /// Whether this edit changes anything at all.
        changes_nothing: bool,
    },
    /// `--from` could not be read.
    Unreadable
    {
        path: String,
        /// What went wrong, as `FileSystemError`'s own `Display` renders it.
        cause: String,
    },
    /// The store refused the staged edit.
    Refused
    {
        /// What went wrong, as `EditError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecPreviewResponse
{
    pub(crate) fn From(result: Result<EditPreview, PreviewRefusal>) -> Self
    {
        return match result
        {
            Ok(preview) => Self::From_Preview(&preview),
            Err(PreviewRefusal::Unreadable { path, error }) =>
            {
                Self::Unreadable { path: path.display().to_string(), cause: error.to_string() }
            }
            Err(PreviewRefusal::Edit(error)) => Self::Refused { cause: error.to_string() },
        };
    }

    fn From_Preview(preview: &EditPreview) -> Self
    {
        return Self::Previewed {
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::spec::SpecMarkdownResponse;

    /// An empty, unique scratch directory of this test's own -- never the real shared `work/`
    /// directory, which live sessions write to concurrently.
    fn Unique_Scratch_Directory(label: &str) -> std::path::PathBuf
    {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let directory = std::env::temp_dir().join(format!(
            "nomos-api-spec-preview-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("a fresh scratch directory can always be created");

        return directory;
    }

    /// Stages a canonical heading rename against `id`'s own real, embedded markdown (read
    /// through this crate's own `Handle_Spec_Markdown`, so this test needs no corpus and
    /// touches no file this repository tracks), writes it to `staged.md` under `into`, and
    /// hands back the path it was written to. Mirrors
    /// `nomos_spec_orchestration`'s own `Staged_Heading_Rename`.
    fn Staged_Heading_Rename(id: &str, into: &std::path::Path) -> std::path::PathBuf
    {
        let request = nomos_spec_orchestration::RecordRequest { id: id.to_owned(), revision: None };
        let SpecMarkdownResponse::Resolved { markdown, .. } = crate::spec::Handle_Spec_Markdown(&request)
        else
        {
            panic!("{id} is a governing record, embedded even with no corpus");
        };
        let edited = markdown.replace("## Decision", "## The decision");
        let staged = into.join("staged.md");
        std::fs::write(&staged, &edited).expect("writes the staged edit");

        return staged;
    }

    #[test]
    fn Test_A_Real_Heading_Rename_Should_Preview_Wording_Moved()
    {
        let staged = Staged_Heading_Rename("D-132", &Unique_Scratch_Directory("preview"));
        let request = EditRequest { id: "D-132".to_owned(), from: staged, rename: None };

        let response = Handle_Spec_Preview(&request);

        let SpecPreviewResponse::Previewed { wording_moved, .. } = response
        else
        {
            panic!("a canonical heading rename previews cleanly: {response:?}");
        };
        assert!(wording_moved, "a heading rename must count as wording moved");
    }

    #[test]
    fn Test_A_Missing_Staged_File_Should_Report_Unreadable()
    {
        let request = EditRequest {
            id: "D-132".to_owned(),
            from: std::path::PathBuf::from("no-such-staged-file-anywhere.md"),
            rename: None,
        };

        let response = Handle_Spec_Preview(&request);

        assert!(matches!(response, SpecPreviewResponse::Unreadable { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Previewed_Response_Should_Round_Trip_As_Json()
    {
        let staged = Staged_Heading_Rename("D-132", &Unique_Scratch_Directory("preview-json"));
        let request = EditRequest { id: "D-132".to_owned(), from: staged, rename: None };

        let response = Handle_Spec_Preview(&request);

        let json = serde_json::to_string(&response).expect("a SpecPreviewResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecPreviewResponse always has this field");

        assert_eq!(outcome, "previewed", "{json}");
    }
}
