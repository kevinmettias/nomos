//! Where a block came from.
//!
//! Named `BlockLineage` rather than `Lineage` because this crate publishes its whole
//! vocabulary at one flat root, where the block's lineage and a section's would otherwise
//! share a name.

use crate::RecordedBlock;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BlockLineage
{
    pub blocks: Vec<RecordedBlock>,
}
