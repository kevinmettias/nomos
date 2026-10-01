//! What a completed run did, target by target.

use crate::placement::Placement;
use crate::publication_scope::PublicationScope;

/// Every placement a completed run made, in the order the intents declared them.
///
/// Only produced by a run that completed: a refused or rolled-back run has no report,
/// because there is nothing it placed to report. That is why the report holds no refusal
/// variant — a list mixing placements with refusals would read as a partial success, and
/// this mechanism has no partial successes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationReport
{
    /// One entry per intent, in declaration order.
    pub placements: Vec<Placement>,
}

impl MaterializationReport
{
    /// Every placement carrying one publication scope.
    ///
    /// The whole of what this crate does with a scope. `OD-PACKAGE-005`'s guard belongs at
    /// the transition into repository-distributed state, which is somewhere else entirely;
    /// what a materializer owes that guard is an honest answer about which of the assets it
    /// just placed carry which scope.
    #[must_use]
    pub fn Placements_Scoped(&self, scope: PublicationScope) -> Vec<&Placement>
    {
        return self
            .placements
            .iter()
            .filter(|placement| return placement.publication_scope == scope)
            .collect();
    }
}
