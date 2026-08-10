//! The answer to "do these two pieces of work touch the same things?".

use nomos_contracts::SubjectId;
use serde::{Deserialize, Serialize};

use crate::unknown_reason::UnknownReason;

/// The result of asking whether two subject sets overlap.
///
/// Three arms, and the third is the reason this is an enum rather than a `bool`.
///
/// A caller cannot reach [`Intersection::Disjoint`] by defaulting, by unwrapping, or by
/// treating an error as "probably fine". Not knowing whether two pieces of work
/// conflict is not the same as knowing they do not, and a `bool` makes those two
/// indistinguishable at exactly the moment the distinction decides whether an edit
/// survives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intersection
{
    /// The sets provably share no member.
    Disjoint,
    /// The sets share these members.
    Overlaps(Vec<SubjectId>),
    /// Independence could not be established.
    Unknown(UnknownReason),
}

impl Intersection
{
    /// Whether the two sets may be worked on concurrently.
    ///
    /// Only [`Intersection::Disjoint`] permits it. `Unknown` deliberately does not:
    /// **unknown independence is not safe parallelism.** This is the same rule as
    /// "unknown is not pass", applied to scheduling instead of to judgment, and it has
    /// the same failure mode when violated — a system that is confidently wrong.
    #[must_use]
    pub const fn Permits_Concurrency(&self) -> bool
    {
        return matches!(self, Self::Disjoint);
    }

    /// The members that conflict, if the answer was an overlap.
    #[must_use]
    pub fn Conflicting(&self) -> &[SubjectId]
    {
        return match self
        {
            Self::Overlaps(members) => members,
            Self::Disjoint | Self::Unknown(_) => &[],
        };
    }
}
