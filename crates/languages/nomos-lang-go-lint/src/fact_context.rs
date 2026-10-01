//! [`FactContext`], where in the workspace's history a fact is being produced.

use nomos_contracts::{BuildVariantId, ConfigurationId, GenerationId, SnapshotId};

/// Where in the workspace's history a fact is being produced -- the four fields every provider in
/// this workspace takes together, declared here because a provider names no sibling's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    pub snapshot: SnapshotId,
    pub variant: BuildVariantId,
    pub configuration: ConfigurationId,
    pub generation: GenerationId,
}
