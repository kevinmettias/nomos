//! The one fact this capability's ceiling allows for one review comment, together with the
//! subject it was filed under.

use nomos_analysis::MaterializedFact;
use nomos_contracts::SubjectId;

/// The one fact this capability's `IncrementalGranularity::None` ceiling allows for one
/// review comment, together with the subject it was filed under.
///
/// `subject` is always `nomos_model::Subject_Of_Path("")` today -- a connector's own fact
/// about the finding itself has no Nomos subject until a bears-on relation names a
/// narrower one, per `ARC-CONNECTOR-001`'s "What This Record Does Not Do". Nothing in the
/// review comment's own fields -- not even `path`, which names a file in the *reviewed*
/// repository, not necessarily a path this workspace's own analysis can address -- is
/// treated as that relation; `path` travels as payload data describing the finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewFindingFact
{
    pub subject: SubjectId,
    pub fact: MaterializedFact,
}
