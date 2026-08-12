//! A table row, addressed by the block that carries it and its position within.

use serde::{Deserialize, Serialize};

use crate::ordinal_ref::OrdinalRef;

/// A table row, addressed by the block that carries it and its position within.
///
/// Not an [`OrdinalRef`] with a different meaning: an `OrdinalRef` is a position inside a
/// document, and a row's position is inside a block. Reusing the type would make the two
/// interchangeable at the call site, and a lineage row pointing at block 7 when it meant
/// row 7 resolves to something rather than failing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRowRef
{
    pub block: OrdinalRef,
    pub ordinal: i64,
}
