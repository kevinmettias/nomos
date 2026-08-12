//! Where a restored member was found.

/// Where a restored member came from.
///
/// A row is addressed as a row and not as the block around it. Thirty concepts all
/// pointing at one table block is not a lineage, and it is the property the restoration
/// exists to establish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin
{
    Block
    {
        ordinal: u32,
    },
    Row
    {
        block_ordinal: u32,
        row_ordinal: u32,
    },
}
