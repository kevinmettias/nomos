//! One member, and what became of reading it.

use crate::rollup::Outcome;
use nomos_contracts::SubjectId;
/// A member of the module, and what came of reading it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberReading
{
    pub subject: SubjectId,
    pub outcome: Outcome,
}
