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
