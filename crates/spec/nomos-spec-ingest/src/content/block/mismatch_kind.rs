//! Which of the three disagreements it was.

use crate::BlockField;

/// Which of the three disagreements it was.
///
/// The three per-block comparisons are one variant carrying a [`BlockField`] rather than
/// three variants, because they differ only in which field disagreed. Each of them read
/// the same ordinal and the same recorded-against-recomputed pair, and the ordinal is what
/// a reader needs first from every one of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockMismatchKind
{
    /// A document the manifest records and the source tree does not have.
    DocumentMissing,
    /// A document whose block count moved, so no ordinal is comparable past the shorter.
    CountDiffers
    {
        recorded: usize,
        recomputed: usize,
    },
    /// One block at one ordinal, disagreeing in one field.
    Block
    {
        ordinal: u32,
        field: BlockField,
        recorded: String,
        recomputed: String,
    },
}
