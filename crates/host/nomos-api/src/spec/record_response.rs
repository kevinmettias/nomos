//! [`Handle_Spec_Record`] and its own [`RecordResponse`].

use nomos_spec_orchestration::{RecordAnswer, RecordRefusal, RecordRequest, SpecCommand};
use serde::Serialize;

use super::{Build_Corpus_Request, DocumentSourceResponse, NodeSummaryResponse};

/// Resolves one record by identifier, exactly as `nomos spec record` would, and hands back a
/// JSON-serializable response.
///
/// Builds its own `CorpusRequest` from the environment, the same composition every other
/// store-touching `Handle_Spec_*` function uses -- an API handler holds no pre-assembled
/// `Assembly` the way `nomos-cli`'s own multi-verb dispatch does, so it pays for one assembly
/// per call. `run::record::Resolved_Record` itself never touches a `FileSystem`, but the shared entry
/// point it is dispatched through, `nomos_spec_orchestration::Run`, is generic over one for
/// every verb regardless.
#[must_use]
pub fn Handle_Spec_Record(request: &RecordRequest) -> RecordResponse
{
    use nomos_platform_std::StdFileSystem;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Record(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Record(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return RecordResponse::From(result);
}

/// What a real `nomos spec record` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum RecordResponse
{
    /// The one document behind the identifier, resolved.
    Resolved
    {
        /// The identifier that was asked about.
        id: String,
        /// The document it resolved to.
        document: DocumentSourceResponse,
    },
    /// No document backs this identifier at the requested revision.
    ///
    /// `node` is the graph's own summary of the identifier, when the identifier is known at
    /// all -- a node the store holds with no source document recorded against it is a
    /// different situation from an identifier nothing in the store recognizes.
    NotFound
    {
        id: String,
        revision: Option<String>,
        node: Option<NodeSummaryResponse>,
    },
    /// The identifier is held at more than one revision, so resolving one of them without a
    /// narrower request would be a guess.
    Ambiguous
    {
        id: String,
        documents: Vec<DocumentSourceResponse>,
    },
    /// The store could not be read at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl RecordResponse
{
    pub(crate) fn From(result: Result<RecordAnswer, RecordRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Resolved { id: answer.id, document: DocumentSourceResponse::From(answer.document) },
            Err(RecordRefusal::NotFound { id, revision, node }) =>
            {
                Self::NotFound { id, revision, node: node.map(NodeSummaryResponse::From) }
            }
            Err(RecordRefusal::Ambiguous { id, documents }) => Self::Ambiguous {
                id,
                documents: documents.into_iter().map(DocumentSourceResponse::From).collect(),
            },
            Err(RecordRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_support::Assert_Round_Trips_As_Json;

    /// `D-132` is a real, embedded governing record -- `nomos_spec_orchestration`'s own
    /// `tests.rs` already resolves it with no corpus present -- so this needs no scratch
    /// directory and no `NOMOS_V14_CORPUS` to be set, the same zero-setup shape every other
    /// test in this file already has.
    #[test]
    fn Test_A_Real_Governing_Record_Should_Resolve()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        let RecordResponse::Resolved { id, document } = response
        else
        {
            // D-132 is a real, embedded governing record, so reaching any other variant here
            // means the record path itself regressed, not a condition this test should
            // assert around.
            panic!("D-132 is a governing record, embedded even with no corpus: {response:?}");
        };
        assert_eq!(id, "D-132");
        assert!(document.text.starts_with("---\nid: D-132\n"), "{}", document.text);
    }

    #[test]
    fn Test_An_Unknown_Identifier_Should_Report_Not_Found_With_No_Node()
    {
        let request = RecordRequest { id: "D-9999999-DOES-NOT-EXIST".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        assert!(matches!(response, RecordResponse::NotFound { node: None, .. }), "{response:?}");
    }

    #[test]
    fn Test_A_Real_Resolved_Response_Should_Round_Trip_As_Json()
    {
        let request = RecordRequest { id: "D-132".to_owned(), revision: None };

        let response = Handle_Spec_Record(&request);

        Assert_Round_Trips_As_Json(&response, "resolved");
    }
}
