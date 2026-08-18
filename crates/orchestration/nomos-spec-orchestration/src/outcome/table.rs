//! What `nomos spec table` produced, or why it did not.

use nomos_spec_store::{DocumentSource, PathMatch, RowCensus, StoreError, TableLine};

/// The rows a `table` request selected, and the document and census they came from.
///
/// The census counts the whole document rather than the selection deliberately: it is what
/// lets a renderer distinguish "this document has no tables" from "the block you named has
/// none" once [`TableAnswer::lines`] is empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableAnswer
{
    /// The document the address resolved to.
    pub document: DocumentSource,
    /// How the address matched it.
    pub tier: PathMatch,
    /// Every pipe line in the whole document, by kind.
    pub census: RowCensus,
    /// The rows under the request's own narrowing.
    pub lines: Vec<TableLine>,
}

/// Why `nomos spec table` did not answer with rows.
///
/// Moved from `nomos-cli::spec::verb::table`'s own `Addressed` and `Nothing_Selected`: which
/// of these happened is a fact about the store, not about how a terminal reports it. The
/// address itself is not repeated here — a renderer already holds the request it built this
/// refusal from.
#[derive(Debug)]
pub enum TableRefusal
{
    /// No document in the store matches the address.
    NoSuchDocument,
    /// The address matches more than one document, which is a question rather than an
    /// answer.
    AmbiguousDocument
    {
        matched: usize,
        tier: PathMatch,
    },
    /// The document was found and read, and the request's own narrowing selected no rows.
    NoRows
    {
        document: DocumentSource,
        tier: PathMatch,
        census: RowCensus,
    },
    /// The store could not be read at all.
    Store(StoreError),
}
