//! A repository's whole requirement-trace judgment, as the one fact this capability answers.

use super::Problem;

/// A repository's whole requirement-trace judgment: every stale or incomplete assessment
/// [`crate::predicates`] found, in a fixed, deterministic order (every unresolved site, then
/// every unresolved gap, then every unresolved record, then every divergence with no record,
/// then every partial with no gap -- each group itself ordered by requirement, since
/// [`crate::registry::Entries`] sorts the assessments it reads before any predicate runs).
///
/// Empty for a repository with no `tests/contract/requirements/` directory at all, or one
/// whose committed assessments all resolve -- the two cases `crate::provider`'s own module
/// doc says are the same fact to a caller that only reads this payload: nothing to report.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RequirementTracePayload
{
    /// Every problem the five predicates found, in the order above.
    pub problems: Vec<Problem>,
}
