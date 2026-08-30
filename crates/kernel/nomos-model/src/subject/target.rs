//! The thing itself, once a subject has been resolved to one.

use serde::{Deserialize, Serialize};

use nomos_contracts::SubjectId;

use crate::EntityId;
use crate::SubjectKind;

/// What a subject points at.
///
/// A reference, never a copy. The prototype's lesson here is small and expensive: any
/// field duplicated from the entity into the subject is a second place for it to be
/// wrong, and the two will disagree the first time one of them is updated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target
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

impl Target
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Content_Digest;

    fn Arbitrary_Entity() -> EntityId
    {
        return EntityId::From_Digest(Content_Digest(b"arbitrary"));
    }

    #[test]
    fn Test_Kind_Should_Report_What_Each_Variant_Denotes()
    {
        assert_eq!(Target::Artifact(Arbitrary_Entity()).Kind(), SubjectKind::Artifact);
        assert_eq!(Target::Symbol(Arbitrary_Entity()).Kind(), SubjectKind::Symbol);
        assert_eq!(Target::Resource(Arbitrary_Entity()).Kind(), SubjectKind::Resource);
        assert_eq!(
            Target::Aggregate {
                kind: "directory".to_owned(),
                members: Vec::new(),
            }
            .Kind(),
            SubjectKind::Aggregate
        );
    }
}
