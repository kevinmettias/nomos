//! Where in the workspace's history a fact is being produced.

use nomos_contracts::{BuildVariantId, ConfigurationId, GenerationId, SnapshotId};

/// The same four fields every `crates/repository/` provider's own fact needs, because they
/// are provenance rather than policy. See `crate::scaffolding`'s own doc for why one shape
/// now serves all five.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}
