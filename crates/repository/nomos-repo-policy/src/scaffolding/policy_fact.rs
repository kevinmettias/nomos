//! The one fact a whole-workspace policy provider answers.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// The one fact each of this crate's five `IncrementalGranularity::WholeWorkspace` ceilings
/// allows, together with the subject it was filed under -- `nomos_model::Subject_Of_Path("")`
/// for every one of the five, the whole-tree subject a workspace-wide answer is always
/// attributed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}
