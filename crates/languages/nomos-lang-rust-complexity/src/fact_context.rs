//! [`FactContext`], where in the workspace's history a fact is being produced.

use nomos_contracts::{BuildVariantId, ConfigurationId, GenerationId, SnapshotId};

/// Where in the workspace's history a fact is being produced.
///
/// One value rather than four parameters, because the four always travel together and a call
/// site that transposed two of them would compile. Every provider crate declares its own, which
/// is the convention `OD-CAPABILITY-008` measured: a provider names no sibling's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactContext
{
    /// The tree this file was read from -- provenance, not identity.
    pub snapshot: SnapshotId,
    /// The build variant the fact is keyed under.
    pub variant: BuildVariantId,
    /// The configuration the fact is keyed under.
    pub configuration: ConfigurationId,
    /// The generation the fact is filed at.
    pub generation: GenerationId,
}
