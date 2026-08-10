//! A module as the rollup addresses it.

use crate::rollup::module_member::ModuleMember;
use nomos_contracts::SubjectId;
/// The subject a rollup is about, and the files it is over.
///
/// The module's own subject must not be any member's. A rollup keyed on one of its inputs
/// would be named by the same [`nomos_analysis::GenerationCause`] that names the input,
/// and would be reported as directly invalidated by a change it was actually reached by —
/// which is the transitive half of invalidation disappearing into the direct half.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module
{
    pub subject: SubjectId,
    pub members: Vec<ModuleMember>,
}
