//! One source document, and the bytes it was ingested from.

/// One source document, and the bytes it was ingested from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSource
{
    /// The join surrogate, so a caller can scope a second query to this document without
    /// resolving the path again. Never exported and never printed.
    pub uid: i64,
    pub path: String,
    pub revision: String,
    /// The document's content address.
    pub content_hash: String,
    /// The document exactly as it was ingested, byte for byte.
    pub text: String,
}
