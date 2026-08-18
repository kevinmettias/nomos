//! What `nomos spec record` produced, or why it did not.

use nomos_spec_store::{DocumentSource, NodeSummary, StoreError};

/// The one document behind an identifier, resolved.
///
/// Content goes out verbatim: [`RecordAnswer::document`] is the source exactly as it was
/// ingested, byte for byte, which is `record`'s whole point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordAnswer
{
    /// The identifier that was asked about.
    pub id: String,
    /// The document it resolved to.
    pub document: DocumentSource,
}

/// Why `nomos spec record` did not resolve to one document.
///
/// Moved from `nomos-cli::spec::verb::record`'s own `Nothing_Behind` and
/// `Ambiguous_Revision`: which of these happened is a fact about the store, not about how a
/// terminal reports it.
#[derive(Debug)]
pub enum RecordRefusal
{
    /// No document backs this identifier at the requested revision.
    ///
    /// `node` is the graph's own summary of the identifier, when the identifier is known at
    /// all. A node the store holds with no source document recorded against it is a
    /// different situation from an identifier nothing in the store recognizes, and a
    /// renderer needs both to tell them apart.
    NotFound
    {
        id: String,
        revision: Option<String>,
        node: Option<NodeSummary>,
    },
    /// The identifier is held at more than one revision, so resolving one of them without a
    /// narrower request would be a guess.
    Ambiguous
    {
        id: String,
        documents: Vec<DocumentSource>,
    },
    /// The store could not be read at all.
    Store(StoreError),
}
