//! The one fact this crate's `IncrementalGranularity::WholeWorkspace` ceiling allows, as
//! [`Materialize_Workspace`](super::Materialize_Workspace) produces it.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// The one fact this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows,
/// together with the subject it was filed under — `nomos_model::Subject_Of_Path("")`, the
/// same whole-tree subject `nomos_check_orchestration`'s own `Lint_Capability_Unavailable`
/// already attributes a failed materialization to, reused here for a successful one: there
/// is exactly one subject a workspace-wide answer could honestly be filed under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}
