//! What `nomos spec table` was asked for.

/// Which rows are being asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableRequest
{
    /// A path, a file name, or a fragment of one.
    pub document: String,
    /// Only this block of the document.
    pub block: Option<u32>,
    /// Only this table within a block. Counted per block, so it narrows rather than
    /// addresses on its own.
    pub table: Option<u32>,
    /// Only this revision.
    pub revision: Option<String>,
}
