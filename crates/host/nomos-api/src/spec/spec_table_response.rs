//! [`Handle_Spec_Table`] and its own [`SpecTableResponse`].

use nomos_spec_orchestration::{SpecCommand, TableAnswer, TableRefusal, TableRequest};
use serde::Serialize;

use super::{Build_Corpus_Request, DocumentSourceResponse, PathMatchResponse, RowCensusResponse, TableLineResponse};

/// Selects table rows exactly as `nomos spec table` would, and hands back a
/// JSON-serializable response.
///
/// Follows [`crate::spec::spec_record_response::Handle_Spec_Record`]'s own composition: builds a
/// `CorpusRequest` from the environment, dispatches through the shared `Run` entry point,
/// matches `SpecOutcome::Table`. `run::table::Resolved_Table` itself never touches a `FileSystem`, the
/// same as `Record`.
#[must_use]
pub fn Handle_Spec_Table(request: &TableRequest) -> SpecTableResponse
{
    use nomos_platform_std::StdFileSystem;

    let corpus_request = Build_Corpus_Request();

    let outcome =
        nomos_spec_orchestration::Run(&SpecCommand::Table(request.clone()), &corpus_request, &StdFileSystem);

    let nomos_spec_orchestration::SpecOutcome::Table(result) = outcome
    else
    {
        // rust-panic: allow: Run's own contract guarantees it returns the SpecOutcome variant
        // naming the SpecCommand it was given -- any other outcome means Run itself is broken.
        unreachable!("Run always returns the SpecOutcome variant naming the SpecCommand it was given")
    };

    return SpecTableResponse::From(result);
}

/// What a real `nomos spec table` produced, in a shape `serde_json` can hand across a wire.
#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecTableResponse
{
    /// The rows the request selected, and the document and census they came from.
    Selected
    {
        document: DocumentSourceResponse,
        tier: PathMatchResponse,
        census: RowCensusResponse,
        lines: Vec<TableLineResponse>,
    },
    /// No document in the store matches the address.
    NoSuchDocument,
    /// The address matches more than one document.
    AmbiguousDocument
    {
        matched: usize,
        tier: PathMatchResponse,
    },
    /// The document was found and read, and the request's own narrowing selected no rows.
    NoRows
    {
        document: DocumentSourceResponse,
        tier: PathMatchResponse,
        census: RowCensusResponse,
    },
    /// The store could not be read at all.
    Unreadable
    {
        /// What went wrong, as `StoreError`'s own `Display` renders it.
        cause: String,
    },
}

impl SpecTableResponse
{
    pub(crate) fn From(result: Result<TableAnswer, TableRefusal>) -> Self
    {
        return match result
        {
            Ok(answer) => Self::Selected {
                document: DocumentSourceResponse::From(answer.document),
                tier: PathMatchResponse::From(answer.tier),
                census: RowCensusResponse::From(answer.census),
                lines: answer.lines.into_iter().map(TableLineResponse::From).collect(),
            },
            Err(TableRefusal::NoSuchDocument) => Self::NoSuchDocument,
            Err(TableRefusal::AmbiguousDocument { matched, tier }) =>
            {
                Self::AmbiguousDocument { matched, tier: PathMatchResponse::From(tier) }
            }
            Err(TableRefusal::NoRows { document, tier, census }) => Self::NoRows {
                document: DocumentSourceResponse::From(document),
                tier: PathMatchResponse::From(tier),
                census: RowCensusResponse::From(census),
            },
            Err(TableRefusal::Store(error)) => Self::Unreadable { cause: error.to_string() },
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// No document under any corpus state can match this name, regardless of whether this
    /// session's own `NOMOS_V14_CORPUS` happens to be set -- `Assemble_Corpus`'s own contract is
    /// that an absent corpus reports real absences rather than refusing.
    #[test]
    fn Test_A_Real_Call_For_An_Unknown_Document_Should_Report_No_Such_Document()
    {
        let request = TableRequest {
            document: "definitely-nonexistent-table-document-xyz".to_owned(),
            block: None,
            table: None,
            revision: None,
        };

        let response = Handle_Spec_Table(&request);

        assert!(matches!(response, SpecTableResponse::NoSuchDocument), "{response:?}");
    }

    #[test]
    fn Test_A_No_Such_Document_Response_Should_Round_Trip_As_Json()
    {
        let request = TableRequest {
            document: "definitely-nonexistent-table-document-xyz".to_owned(),
            block: None,
            table: None,
            revision: None,
        };

        let response = Handle_Spec_Table(&request);

        let json = serde_json::to_string(&response).expect("a SpecTableResponse always serializes");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("what was just written parses back");
        let outcome = parsed.get("outcome").expect("a serialized SpecTableResponse always has this field");

        assert_eq!(outcome, "no_such_document", "{json}");
    }
}
