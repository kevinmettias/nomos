//! A repository's whole requirement-trace judgment, as the one fact this capability answers.

use super::Problem;

/// A repository's whole requirement-trace judgment: every stale or incomplete assessment
/// [`crate::predicates`] found, in a fixed, deterministic order (every unresolved site, then
/// every unresolved gap, then every unresolved record, then every divergence with no record,
/// then every partial with no gap -- each group itself ordered by requirement, since
/// [`crate::registry::Entries`] sorts the assessments it reads before any predicate runs).
///
/// No problems both for a repository with no `tests/contract/requirements/` directory at all
/// and for one whose committed assessments all resolve, and [`Self::directory_absent`] tells the
/// two apart: `OD-RULES-011` version 3 decision 6 has the rule reading this payload say it judged
/// nothing in the first case, which a payload reporting zero problems in both could not let it do.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RequirementTracePayload
{
    /// Every problem the five predicates found, in the order above.
    pub problems: Vec<Problem>,
    /// Whether the repository has no `tests/contract/requirements/` directory, so there was no
    /// corpus to judge and no problem could have been found.
    ///
    /// `false` for a directory the provider could not enumerate and for one holding an entry it
    /// refused: both read, as they always have, as zero problems, and decision 6 leaves what they
    /// should read as undecided.
    pub directory_absent: bool,
}
