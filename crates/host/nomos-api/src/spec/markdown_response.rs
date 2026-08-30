//! [`Handle_Spec_Markdown`] and its own [`MarkdownResponse`].

use nomos_spec_orchestration::{RecordRequest, SpecCommand};
use nomos_spec_store::{EditError, RecordProjection};
use serde::Serialize;

/// Renders one record's markdown from the store's own rows, exactly as `nomos spec markdown`
/// would, and hands back a JSON-serializable response.
///
/// Follows [`crate::spec::record_response::Handle_Spec_Record`]'s own composition, reusing the same
/// [`RecordRequest`]: builds a `CorpusRequest` from the environment, dispatches through the
/// shared `Run` entry point, matches `SpecOutcome::Markdown`. `run::markdown::Markdown` itself
/// never touches a `FileSystem`, the same as `Record` and `Table`.
#[must_use]
pub fn Handle_Spec_Markdown(request: &RecordRequest) -> MarkdownResponse
{
    use super::Build_Corpus_Request;
    use nomos_platform_std::StdFileSystem;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Markdown(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Markdown(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return MarkdownResponse::From(result);
}

/// What a real `nomos spec markdown` produced, in a shape `serde_json` can hand across a
/// wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum MarkdownResponse
{
    /// The record, rendered from the store's own rows.
    Resolved
    {
        node_id: String,
        path: String,
        revision: String,
        /// The markdown, rendered from the store's rows.
        markdown: String,
        /// What the ingested bytes hash to.
        source_hash: String,
        /// What this projection hashes to.
        projected_hash: String,
        /// Whether the projection is the ingested bytes -- `source_hash == projected_hash`.
        matches_source: bool,
    },
    /// The identifier did not resolve to one record's markdown.
    Refused
    {
        /// What went wrong, as `EditError`'s own `Display` renders it.
        cause: String,
    },
}

impl MarkdownResponse
{
    pub(crate) fn From(result: Result<RecordProjection, EditError>) -> Self
    {
        return match result
        {
            Ok(projection) =>
            {
                let matches_source = projection.Matches_Source();

                Self::Resolved {
                    node_id: projection.node_id,
                    path: projection.path,
                    revision: projection.revision,
                    markdown: projection.markdown,
                    source_hash: projection.source_hash,
                    projected_hash: projection.projected_hash,
                    matches_source,
                }
            }
            Err(error) => Self::Refused { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Assert_Round_Trips_As_Json;

    /// `D-132` is a real, embedded governing record with declared front matter --
    /// `nomos_spec_orchestration`'s own `tests.rs` already renders it with no corpus present.
    #[test]
    fn Test_A_Real_Governing_Record_Should_Render_As_Markdown()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        let MarkdownResponse::Resolved { node_id, markdown, .. } = response
        else
        {
            // D-132 is a real, embedded governing record with declared front matter, so
            // reaching any other variant here means the markdown path itself regressed, not
            // a condition this test should assert around.
            panic!("D-132 is a governing record with declared front matter: {response:?}");
        };
        assert_eq!(node_id, "D-132");
        assert!(markdown.starts_with("---\nid: D-132\n"), "{markdown}");
    }

    #[test]
    fn Test_An_Unknown_Identifier_Should_Report_Refused()
    {
        let request = RecordRequest { id: "D-9999999-DOES-NOT-EXIST".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        assert!(matches!(response, MarkdownResponse::Refused { .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Resolved_Markdown_Response_Should_Round_Trip_As_Json()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Markdown(&request);

        Assert_Round_Trips_As_Json(&response, "resolved");
    }
}
