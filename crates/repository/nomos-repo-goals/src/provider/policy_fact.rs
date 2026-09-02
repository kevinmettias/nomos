//! The one fact this crate's `IncrementalGranularity::WholeWorkspace` ceiling allows, as
//! [`super::Materialize_Workspace`] produces it.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// The one fact this capability's `IncrementalGranularity::WholeWorkspace` ceiling allows,
/// together with the subject it was filed under — `nomos_model::Subject_Of_Path("")`, the
/// same whole-tree subject `nomos_repo_standards::PolicyFact`, `nomos_repo_limits::
/// PolicyFact`, `nomos_repo_scripting::PolicyFact` and `nomos_repo_words::PolicyFact`
/// already attribute their own workspace-wide answers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}
