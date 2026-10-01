//! What `nomos spec table` produced, or why it did not.

mod table_refusal;

pub use table_refusal::TableRefusal;

use nomos_spec_store::{DocumentSource, PathMatch, RowCensus, TableLine};

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
