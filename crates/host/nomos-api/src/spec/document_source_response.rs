//! [`DocumentSourceResponse`], shared by [`super::record_response::RecordResponse`] and
//! [`super::table_response::TableResponse`] -- the one document a resolved identifier or a
//! selected table both carry.

use nomos_spec_store::DocumentSource;
use serde::Serialize;

/// A serializable twin of [`nomos_spec_store::DocumentSource`], which does not derive
/// `Serialize`. Omits `uid`: that field's own doc comment says it is "never exported and
/// never printed."
#[derive(Debug, Serialize)]
pub struct DocumentSourceResponse
{
    pub path: String,
    pub revision: String,
    /// The document's content address.
    pub content_hash: String,
    /// The document exactly as it was ingested, byte for byte.
    pub text: String,
}

impl DocumentSourceResponse
{
    pub(crate) fn From(document: DocumentSource) -> Self
    {
        return Self {
            path: document.path,
            revision: document.revision,
            content_hash: document.content_hash,
            text: document.text,
        };
    }
}
