//! Where a block came from.

use crate::recorded_block::RecordedBlock;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BlockLineage
{
    pub blocks: Vec<RecordedBlock>,
}
