//! Where a block came from.

use crate::RecordedBlock;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Lineage
{
    pub blocks: Vec<RecordedBlock>,
}
