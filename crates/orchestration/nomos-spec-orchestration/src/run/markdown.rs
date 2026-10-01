//! Resolving `nomos spec markdown`'s request against an assembled store.

use nomos_spec_store::{EditError, RecordProjection};

use crate::corpus::Assembly;
use crate::request::RecordRequest;

/// `request.id` rendered from the store's own rows, or why it could not be.
///
/// A thin wrapper around [`nomos_spec_store::SpecificationStore::Record_Markdown`] rather
/// than new logic: that method already returns the typed answer this verb needs, and a
/// second type here would only restate it. Moved from
/// `nomos-cli::spec::verb::markdown::Markdown`, minus the writing and the
/// [`RecordProjection::Is_Matching_Source`] check, which is a rendering decision (`Ok` versus
/// `Stale`) rather than a resolution one.
///
/// # Errors
///
/// Returns [`EditError`] naming why no single record answered -- see
/// [`nomos_spec_store::SpecificationStore::Record_Markdown`].
pub fn Rendered_Markdown(assembly: &Assembly, request: &RecordRequest) -> Result<RecordProjection, EditError>
{
    let revision = request.revision.as_deref();

    return assembly.store.Record_Markdown(&request.id, revision);
}

#[cfg(test)]
mod tests
{
    use super::{RecordRequest, Rendered_Markdown};
    use crate::corpus::{Assemble_Corpus, CorpusRequest};

    #[test]
    fn Test_Rendered_Markdown_Should_Render_A_Governing_Record_Back_Out()
    {
        let request = CorpusRequest { variable: "A_MARKDOWN_TEST_CORPUS_VARIABLE".to_owned(), root: None, revision: "v14.36".to_owned() };
        let assembly = Assemble_Corpus(&request).expect("assembles from the embedded records alone");

        let projection = Rendered_Markdown(&assembly, &RecordRequest { id: "D-132".to_owned(), revision: None })
            .expect("D-132 is a governing record with declared front matter");

        assert_eq!(projection.node_id, "D-132");
    }
}
