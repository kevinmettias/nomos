//! An invalidation reaching further than the change that caused it.

use nomos_contracts::IncrementalGranularity;
use crate::fact_key::FactKey;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Broadening
{
    pub key: FactKey,
    pub requested: IncrementalGranularity,
    pub applied: IncrementalGranularity,
}
