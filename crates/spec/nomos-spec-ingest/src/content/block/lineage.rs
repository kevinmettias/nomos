//! Where a block came from.

use crate::content::block::recorded::RecordedBlock;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BlockLineage
{
    pub blocks: Vec<RecordedBlock>,
}
