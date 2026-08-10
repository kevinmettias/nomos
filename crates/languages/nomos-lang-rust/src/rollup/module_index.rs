//! Everything one module was found to declare.

use crate::rollup::index_entry::IndexEntry;
use crate::rollup::member_reading::MemberReading;
use crate::rollup::outcome::Outcome;
use nomos_contracts::SubjectId;
/// What a module declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleIndex
{
    pub module: SubjectId,
    pub members: Vec<MemberReading>,
    pub items: Vec<IndexEntry>,
}

impl ModuleIndex
{
    /// Members that contributed entries, whether exactly or approximately.
    #[must_use]
    pub fn Answered(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome.Answered())
            .count();
    }

    /// Members whose syntax fact could not be read.
    #[must_use]
    pub fn Unreachable(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome == Outcome::Unreachable)
            .count();
    }

    /// Members answered by a weaker provider than the caller asked for.
    #[must_use]
    pub fn Approximated(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome == Outcome::Approximate)
            .count();
    }
}
