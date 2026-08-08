//! Addressing a thing a rule, metric or finding can be about.

use crate::entity::EntityId;
use nomos_contracts::{BuildVariantId, ConfigurationId, SnapshotEntityId, SnapshotId, SubjectId};
use serde::{Deserialize, Serialize};

/// What kind of thing a subject denotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SubjectKind
{
    /// A persisted or generated repository object.
    Artifact,
    /// A language-semantic declaration.
    Symbol,
    /// A non-code entity.
    Resource,
    /// A named group of subjects treated as one.
    Aggregate,
}

/// What a subject points at.
///
/// A reference, never a copy. The prototype's lesson here is small and expensive: any
/// field duplicated from the entity into the subject is a second place for it to be
/// wrong, and the two will disagree the first time one of them is updated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectTarget
{
    /// A repository object.
    Artifact(EntityId),
    /// A declaration.
    Symbol(EntityId),
    /// A non-code entity.
    Resource(EntityId),
    /// Several subjects addressed together.
    Aggregate
    {
        /// What the grouping means.
        kind: String,
        /// The members.
        members: Vec<SubjectId>,
    },
}

impl SubjectTarget
{
    /// What kind of thing this target denotes.
    #[must_use]
    pub const fn Kind(&self) -> SubjectKind
    {
        return match self
        {
            Self::Artifact(_) => SubjectKind::Artifact,
            Self::Symbol(_) => SubjectKind::Symbol,
            Self::Resource(_) => SubjectKind::Resource,
            Self::Aggregate { .. } => SubjectKind::Aggregate,
        };
    }
}

/// One addressable thing, as it exists in one snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject
{
    /// Stable identity of the subject reference.
    pub id: SubjectId,
    /// What it points at.
    pub target: SubjectTarget,
    /// The occurrence under a specific snapshot, variant and configuration.
    pub occurrence: SnapshotEntityId,
}

impl Subject
{
    /// What kind of thing this subject denotes.
    #[must_use]
    pub const fn Kind(&self) -> SubjectKind
    {
        return self.target.Kind();
    }
}

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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    #[test]
    fn Test_Target_Kind_Should_Follow_The_Variant()
    {
        let entity = EntityId::From_Digest(Content_Digest(b"thing"));

        assert_eq!(
            SubjectTarget::Artifact(entity).Kind(),
            SubjectKind::Artifact
        );
        assert_eq!(SubjectTarget::Symbol(entity).Kind(), SubjectKind::Symbol);
        assert_eq!(
            SubjectTarget::Resource(entity).Kind(),
            SubjectKind::Resource
        );
        assert_eq!(
            SubjectTarget::Aggregate {
                kind: "subsystem".to_owned(),
                members: Vec::new(),
            }
            .Kind(),
            SubjectKind::Aggregate
        );
    }
}
