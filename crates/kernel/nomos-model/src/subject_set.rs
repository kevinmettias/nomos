//! The one exclusion primitive: do these two pieces of work touch the same things?
//!
//! Three subsystems ask this question and they must not each answer it their own way:
//!
//! - the **work ledger**, deciding whether two agents may claim overlapping territory;
//! - the **correction scheduler**, deciding whether two corrections may run in the same
//!   parallel wave;
//! - **agent leases**, deciding whether a delegated task's write scope is free.
//!
//! They differ in lifetime and authority — a ledger claim outlives every process, a
//! wave reservation dies with the run — but the predicate is identical, and three
//! implementations of one predicate is three chances to disagree about whether two
//! writers may proceed. That disagreement is a lost edit.

use nomos_contracts::SubjectId;
use serde::{Deserialize, Serialize};

use crate::intersection::Intersection;
use crate::set_resolution::SetResolution;
use crate::unknown_reason::UnknownReason;
use std::collections::BTreeSet;

/// A set of subjects some piece of work reads or writes.
///
/// Membership is explicit. There is no "everything under this path" that resolves
/// lazily at comparison time — a pattern that has not been expanded produces
/// [`UnknownReason::UnexpandedPattern`] rather than an answer, because a comparison
/// against an unexpanded pattern is a guess wearing the costume of a computation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectSet
{
    resolution: SetResolution,
    members: BTreeSet<SubjectId>,
    unexpanded: Vec<String>,
}

impl SubjectSet
{
    /// An empty set at the given resolution.
    #[must_use]
    pub fn Empty(resolution: SetResolution) -> Self
    {
        return Self {
            resolution,
            members: BTreeSet::new(),
            unexpanded: Vec::new(),
        };
    }

    /// A set with known members at the given resolution.
    #[must_use]
    pub fn Of(resolution: SetResolution, members: impl IntoIterator<Item = SubjectId>) -> Self
    {
        return Self {
            resolution,
            members: members.into_iter().collect(),
            unexpanded: Vec::new(),
        };
    }

    /// Records a pattern whose membership has not been resolved.
    ///
    /// The set remains usable — it can still be displayed, stored and reasoned about —
    /// but every comparison involving it answers [`Intersection::Unknown`] until the
    /// pattern is expanded. That is deliberate: the alternative is a comparison that
    /// quietly ignores whatever the pattern would have matched.
    #[must_use]
    pub fn With_Unexpanded_Pattern(mut self, pattern: impl Into<String>) -> Self
    {
        self.unexpanded.push(pattern.into());
        return self;
    }

    /// Adds a member.
    pub fn Insert(&mut self, subject: SubjectId)
    {
        self.members.insert(subject);
    }

    /// The resolution at which this set states membership.
    #[must_use]
    pub const fn Resolution(&self) -> SetResolution
    {
        return self.resolution;
    }

    /// The known members, in a stable order.
    pub fn Members(&self) -> impl Iterator<Item = &SubjectId>
    {
        return self.members.iter();
    }

    /// Whether the set has no known members and no unexpanded patterns.
    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.members.is_empty() && self.unexpanded.is_empty();
    }

    /// The number of known members.
    #[must_use]
    pub fn Len(&self) -> usize
    {
        return self.members.len();
    }

    /// Whether two sets share a member.
    ///
    /// Answers [`Intersection::Unknown`] rather than guessing whenever an honest answer
    /// is unavailable: an unexpanded pattern on either side, or two different
    /// resolutions. The second case is the subtle one — a file-resolution set and a
    /// symbol-resolution set cannot be compared by matching identifiers, because a file
    /// identifier and a symbol identifier are different things and their absence from
    /// each other's membership means nothing.
    /// Why two sets cannot be compared at all, if they cannot.
    ///
    /// Both answers are about the sets rather than about their members: one of them still
    /// holds an unexpanded pattern, or they were resolved to different kinds of thing.
    /// Neither can be settled by looking at membership, which is why they are asked first.
    fn Incomparable(&self, other: &Self) -> Option<UnknownReason>
    {
        if let Some(pattern) = self.unexpanded.first().or_else(|| return other.unexpanded.first())
        {
            return Some(UnknownReason::UnexpandedPattern {
                pattern: pattern.clone(),
            });
        }

        if self.resolution != other.resolution
        {
            return Some(UnknownReason::IncomparableResolution {
                left: self.resolution,
                right: other.resolution,
            });
        }

        return None;
    }

    #[must_use]
    pub fn Intersect(&self, other: &Self) -> Intersection
    {
        if let Some(reason) = self.Incomparable(other)
        {
            return Intersection::Unknown(reason);
        }

        let shared: Vec<SubjectId> = self
            .members
            .intersection(&other.members)
            .copied()
            .collect();
        if shared.is_empty()
        {
            return Intersection::Disjoint;
        }

        return Intersection::Overlaps(shared);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::digest::Content_Digest;

    fn Subject(name: &str) -> SubjectId
    {
        return SubjectId::From_Digest(Content_Digest(name.as_bytes()));
    }

    #[test]
    fn Test_Disjoint_Sets_Should_Permit_Concurrency()
    {
        let left = SubjectSet::Of(SetResolution::File, [Subject("a.rs")]);
        let right = SubjectSet::Of(SetResolution::File, [Subject("b.rs")]);

        assert_eq!(left.Intersect(&right), Intersection::Disjoint);
        assert!(left.Intersect(&right).Permits_Concurrency());
    }

    #[test]
    fn Test_Overlapping_Sets_Should_Report_The_Shared_Members()
    {
        let left = SubjectSet::Of(SetResolution::File, [Subject("a.rs"), Subject("b.rs")]);
        let right = SubjectSet::Of(SetResolution::File, [Subject("b.rs"), Subject("c.rs")]);

        let result = left.Intersect(&right);

        assert!(!result.Permits_Concurrency());
        assert_eq!(result.Conflicting(), &[Subject("b.rs")]);
    }

    /// The property the whole type exists for. If this ever passes, two agents can be
    /// told they may proceed when nobody established that they may.
    #[test]
    fn Test_Unknown_Should_Never_Permit_Concurrency()
    {
        let reasons = [
            UnknownReason::IncomparableResolution {
                left: SetResolution::File,
                right: SetResolution::Symbol,
            },
            UnknownReason::ResolutionUnavailable {
                requested: SetResolution::Region,
            },
            UnknownReason::IncomparableSnapshots,
            UnknownReason::UnexpandedPattern {
                pattern: "src/**".to_owned(),
            },
        ];

        for reason in reasons
        {
            assert!(
                !Intersection::Unknown(reason).Permits_Concurrency(),
                "unknown independence is not safe parallelism"
            );
        }
    }

    /// A file set and a symbol set look disjoint if you just compare identifiers, and
    /// they are not — the identifiers denote different kinds of thing, so absence from
    /// each other's membership carries no information.
    #[test]
    fn Test_Mismatched_Resolutions_Should_Be_Unknown_Not_Disjoint()
    {
        let by_file = SubjectSet::Of(SetResolution::File, [Subject("a.rs")]);
        let by_symbol = SubjectSet::Of(SetResolution::Symbol, [Subject("a.rs::foo")]);

        let result = by_file.Intersect(&by_symbol);

        assert!(matches!(result, Intersection::Unknown(_)));
        assert!(!result.Permits_Concurrency());
    }

    /// An unexpanded pattern could match anything. Comparing against it as though it
    /// matched nothing is the quiet version of the same bug.
    #[test]
    fn Test_An_Unexpanded_Pattern_Should_Make_The_Answer_Unknown()
    {
        let wildcard =
            SubjectSet::Empty(SetResolution::File).With_Unexpanded_Pattern("src/**/*.rs");
        let concrete = SubjectSet::Of(SetResolution::File, [Subject("src/main.rs")]);

        assert!(!wildcard.Intersect(&concrete).Permits_Concurrency());
        assert!(
            !concrete.Intersect(&wildcard).Permits_Concurrency(),
            "the pattern must be caught from either side"
        );
    }

    /// Two empty sets genuinely do not conflict, and must not be dragged into `Unknown`
    /// by over-caution — a scheduler that cannot prove the trivial case will serialize
    /// everything.
    #[test]
    fn Test_Empty_Sets_Should_Be_Disjoint()
    {
        let left = SubjectSet::Empty(SetResolution::File);
        let right = SubjectSet::Empty(SetResolution::File);

        assert_eq!(left.Intersect(&right), Intersection::Disjoint);
    }

    #[test]
    fn Test_Intersection_Should_Be_Symmetric()
    {
        let left = SubjectSet::Of(SetResolution::File, [Subject("a.rs"), Subject("b.rs")]);
        let right = SubjectSet::Of(SetResolution::File, [Subject("b.rs")]);

        assert_eq!(
            left.Intersect(&right).Permits_Concurrency(),
            right.Intersect(&left).Permits_Concurrency()
        );
        assert_eq!(
            left.Intersect(&right).Conflicting(),
            right.Intersect(&left).Conflicting()
        );
    }

    /// A set overlaps itself unless it is empty. Trivial, and it is the sanity check
    /// that catches an `Intersect` accidentally written as a difference.
    #[test]
    fn Test_A_Nonempty_Set_Should_Overlap_Itself()
    {
        let set = SubjectSet::Of(SetResolution::File, [Subject("a.rs")]);

        assert!(!set.Intersect(&set).Permits_Concurrency());
    }

    /// Members must iterate in a stable order regardless of insertion order, or a
    /// territory rendered into the ledger file would churn under `git diff` and two
    /// agents writing the same claim would produce different bytes.
    #[test]
    fn Test_Members_Should_Iterate_In_A_Stable_Order()
    {
        let forward = SubjectSet::Of(
            SetResolution::File,
            [Subject("a.rs"), Subject("b.rs"), Subject("c.rs")],
        );
        let reverse = SubjectSet::Of(
            SetResolution::File,
            [Subject("c.rs"), Subject("b.rs"), Subject("a.rs")],
        );

        let forward_order: Vec<&SubjectId> = forward.Members().collect();
        let reverse_order: Vec<&SubjectId> = reverse.Members().collect();

        assert_eq!(forward_order, reverse_order);
    }

    #[test]
    fn Test_Every_Unknown_Reason_Should_Describe_Itself_Usefully()
    {
        let reasons = [
            UnknownReason::IncomparableResolution {
                left: SetResolution::File,
                right: SetResolution::Region,
            },
            UnknownReason::ResolutionUnavailable {
                requested: SetResolution::Symbol,
            },
            UnknownReason::IncomparableSnapshots,
            UnknownReason::UnexpandedPattern {
                pattern: "src/**".to_owned(),
            },
        ];

        for reason in &reasons
        {
            assert!(
                reason.Describe().len() > 20,
                "{} is too terse to act on",
                reason.Describe()
            );
        }
    }
}
