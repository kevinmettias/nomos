//! One entity as it stood under one snapshot, build variant and configuration.

use nomos_contracts::{BuildVariantId, ConfigurationId, SnapshotEntityId, SnapshotId};
use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

/// One canonical entity as it appears under one snapshot, build variant and
/// configuration.
///
/// Findings, architecture nodes, feature members and test paths all reference this
/// rather than a bare [`EntityId`]. A function compiled for two targets is one thing to
/// talk about and two things to measure, and a metric that cannot say which one it
/// measured is not comparable with anything.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEntity
{
    /// Stable identity of this occurrence.
    pub id: SnapshotEntityId,
    /// The entity that occurred.
    pub entity: EntityId,
    /// The snapshot it occurred in.
    pub snapshot: SnapshotId,
    /// The build variant it was analyzed under.
    pub variant: BuildVariantId,
    /// The effective configuration it was analyzed under.
    pub configuration: ConfigurationId,
}
