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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The ingested document's own row id. `DocumentSourceResponse` has no `uid` field for it
    /// to land in, so the number exists only to prove the conversion drops it.
    const SOURCE_UID: i64 = 99;

    #[test]
    fn Test_From_Should_Copy_Every_Field_Except_The_Never_Exported_Uid()
    {
        let document = DocumentSource {
            uid: SOURCE_UID,
            path: "docs/records/d-132.md".to_owned(),
            revision: "1".to_owned(),
            content_hash: "abc123".to_owned(),
            text: "---\nid: D-132\n---\n".to_owned(),
        };

        let response = DocumentSourceResponse::From(document.clone());

        assert_eq!(response.path, document.path);
        assert_eq!(response.revision, document.revision);
        assert_eq!(response.content_hash, document.content_hash);
        assert_eq!(response.text, document.text);
    }
}
