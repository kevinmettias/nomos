//! The one fact this capability answers, together with the subject it was filed under.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// The one fact this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows,
/// together with the subject it was filed under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceFact
{
    /// The subject this fact was filed under.
    pub subject: SubjectId,
    /// The materialized fact itself.
    pub fact: MaterializedFact,
}
