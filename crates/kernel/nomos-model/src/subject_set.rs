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
use std::collections::BTreeSet;

const FILE_LABEL: &str = "File";
const REGION_LABEL: &str = "Region";
const SYMBOL_LABEL: &str = "Symbol";

/// The granularity at which membership of a [`SubjectSet`] is decided.
///
/// Nomos starts at [`SetResolution::File`] — one writer per file — and stays there
/// until the architecture graph can produce region facts with a proven guarantee.
/// Requests at a finer resolution than the system can actually resolve do not silently
/// degrade to file comparison; they return [`Intersection::Unknown`], which forces
/// serialization. Claiming precision you do not have is how two edits to one function
/// get applied concurrently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SetResolution
{
    /// Whole files. The only resolution available today.
    File,
    /// Named symbols within a file.
    Symbol,
    /// Sub-symbol regions.
    Region,
}

impl SetResolution
{
    /// The variant's stable `PascalCase` name.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::File => FILE_LABEL,
            Self::Symbol => SYMBOL_LABEL,
            Self::Region => REGION_LABEL,
        };
    }
}

impl core::fmt::Display for SetResolution
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

/// Why an overlap question could not be answered.
///
/// Every variant is a reason a caller can act on, because the response to "we cannot
/// resolve symbols yet" is different from the response to "these sets were computed
/// against different snapshots".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownReason
{
    /// The two sets are stated at different granularities and neither can be lowered to
    /// the other without inventing membership.
    IncomparableResolution
    {
        /// The resolution of the first set.
        left: SetResolution,
        /// The resolution of the second set.
        right: SetResolution,
    },
    /// A set names a resolution the system cannot currently compute membership at.
    ResolutionUnavailable
    {
        /// The resolution that was asked for.
        requested: SetResolution,
    },
    /// The sets were derived from different snapshots, so their members are not
    /// comparable even where the identifiers look equal.
    IncomparableSnapshots,
    /// A set includes a pattern whose expansion is not known without touching the
    /// filesystem, and the caller asked for an answer without doing so.
    UnexpandedPattern
    {
        /// The pattern that was not expanded.
        pattern: String,
    },
}

impl UnknownReason
{
    /// A one-line description naming what is unknown and why.
    #[must_use]
    pub fn Describe(&self) -> String
    {
        return match self
        {
            Self::IncomparableResolution { left, right } => format!(
                "sets are stated at {left} and {right} resolution and cannot be compared \
                 without inventing membership"
            ),
            Self::ResolutionUnavailable { requested } => {
                format!("{requested}-resolution membership cannot be computed yet")
            }
            Self::IncomparableSnapshots =>
            {
                "sets were derived from different snapshots, so equal identifiers are not \
                 necessarily the same subject"
                    .to_owned()
            }
            Self::UnexpandedPattern { pattern } => {
                format!("pattern `{pattern}` was not expanded, so its members are not known")
            }
        };
    }
}

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
