//! Why `nomos spec table` did not answer with rows.

use nomos_spec_store::{DocumentSource, PathMatch, RowCensus, StoreError};

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
