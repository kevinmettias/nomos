//! A rolled-up module index and how it was come by.

use crate::rollup::module_index::ModuleIndex;
use nomos_analysis::Dependency;
use nomos_analysis::FactKey;
/// A rollup that has been written to the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rolled
{
    /// Where the derived fact is filed. A caller invalidating or re-reading it needs this.
    pub key: FactKey,
    pub index: ModuleIndex,
    /// What the reader observed, which is what the store was given as edges.
    pub dependencies: Vec<Dependency>,
}
