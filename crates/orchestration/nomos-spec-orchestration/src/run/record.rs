//! Resolving `nomos spec record`'s request against an assembled store.

use crate::corpus::Assembly;
use crate::spec_outcome::{RecordAnswer, RecordRefusal};
use crate::request::RecordRequest;

/// The one document behind `request.id`, or why none answered.
///
/// Moved verbatim from `nomos-cli::spec::verb::record::Record`, minus the writing: three
/// document counts are three different answers -- exactly one, none, or several -- and
/// which of them happened is decided here rather than re-derived by whoever renders it.
///
/// # Errors
///
/// Returns [`RecordRefusal::NotFound`] when no document backs `request.id` at the requested
/// revision, [`RecordRefusal::Ambiguous`] when more than one does, and
/// [`RecordRefusal::Store`] when the store could not be read at all.
// `RecordRefusal::NotFound` carries an owned `NodeSummary` -- five owned `String` fields --
// because a node the store holds with no source document behind it is a real, common answer
// that a renderer needs the graph's own words for, the same reason
// `nomos_spec_store::SpecificationStore::Node_Summary` returns it unboxed. That pushes the
// variant past `clippy::result_large_err`'s default threshold without there being a smaller
// type that says the same thing.
#[allow(clippy::result_large_err)]
pub fn Resolved_Record(assembly: &Assembly, request: &RecordRequest) -> Result<RecordAnswer, RecordRefusal>
{
    let revision = request.revision.as_deref();
    let documents = assembly
        .store
        .Documents_Behind(&request.id, revision)
        .map_err(RecordRefusal::Store)?;

    return match documents.as_slice()
    {
        [only] => Ok(RecordAnswer {
            id: request.id.clone(),
            document: only.clone(),
        }),
        [] => Err(Nothing_Behind(assembly, request)),
        _ => Err(RecordRefusal::Ambiguous {
            id: request.id.clone(),
            documents,
        }),
    };
}

/// What to say when a record read produced no document.
///
/// Two different situations, told apart by whether the graph knows the identifier at all:
/// a node with no source document recorded against it is not the same claim as an
/// identifier nothing in the store recognizes.
fn Nothing_Behind(assembly: &Assembly, request: &RecordRequest) -> RecordRefusal
{
    let summary = match assembly.store.Node_Summary(&request.id)
    {
        Ok(summary) => summary,
        Err(error) => return RecordRefusal::Store(error),
    };

    return RecordRefusal::NotFound {
        id: request.id.clone(),
        revision: request.revision.clone(),
        node: summary,
    };
}

#[cfg(test)]
mod tests
{
    use super::{RecordRequest, Resolved_Record};
    use crate::corpus::{Assemble_Corpus, CorpusRequest};

    fn Assembled() -> crate::corpus::Assembly
    {
        let request = CorpusRequest { variable: "A_RECORD_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        return Assemble_Corpus(&request).expect("assembles from the embedded records alone");
    }

    #[test]
    fn Test_Resolved_Record_Should_Resolve_A_Governing_Record_With_No_Corpus()
    {
        let assembly = Assembled();

        let answer = Resolved_Record(&assembly, &RecordRequest { id: "D-132".to_owned(), revision: None }).expect("D-132 is embedded even with no corpus");

        assert_eq!(answer.id, "D-132");
    }

    #[test]
    fn Test_Resolved_Record_Should_Refuse_An_Identifier_Nothing_Holds()
    {
        let assembly = Assembled();

        let error = Resolved_Record(&assembly, &RecordRequest { id: "D-9999".to_owned(), revision: None }).expect_err("nothing in the store is identified D-9999");

        assert!(matches!(error, super::RecordRefusal::NotFound { node: None, .. }), "{error:?}");
    }
}
