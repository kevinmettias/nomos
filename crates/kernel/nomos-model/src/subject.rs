//! Addressing a thing a rule, metric or finding can be about.

use crate::subject_kind::SubjectKind;
use crate::subject_target::SubjectTarget;
use nomos_contracts::{SnapshotEntityId, SubjectId};
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::entity::EntityId;
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
