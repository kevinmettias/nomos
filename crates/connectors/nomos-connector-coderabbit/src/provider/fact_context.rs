//! Where in the workspace's history a fact is being produced.

use nomos_contracts::{BuildVariantId, ConfigurationId, GenerationId, SnapshotId};

/// Where in the workspace's history a fact is being produced -- the same four-field shape
/// every other real provider's own `FactContext` carries, for the identical reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}
