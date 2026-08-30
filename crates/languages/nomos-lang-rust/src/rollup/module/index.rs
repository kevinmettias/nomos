//! Everything one module was found to declare.

use crate::rollup::module::IndexEntry;
use crate::rollup::module::MemberReading;
use crate::rollup::Outcome;
use nomos_contracts::SubjectId;
/// What a module declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index
{
    pub module: SubjectId,
    pub members: Vec<MemberReading>,
    pub items: Vec<IndexEntry>,
}

impl Index
{
    /// Members that contributed entries, whether exactly or approximately.
    #[must_use]
    pub fn Answered(&self) -> usize
    {
        return self
            .members
            .iter()
            .filter(|member| return member.outcome.Is_Answered())
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_model::Content_Digest;

    fn Subject(path: &str) -> SubjectId
    {
        return SubjectId::From_Digest(Content_Digest(path.as_bytes()));
    }

    fn An_Index() -> Index
    {
        return Index {
            module: Subject("the/module"),
            members: vec![
                MemberReading { subject: Subject("read.rs"), outcome: Outcome::Read },
                MemberReading { subject: Subject("approx.rs"), outcome: Outcome::Approximate },
                MemberReading { subject: Subject("missing.rs"), outcome: Outcome::Unreachable },
            ],
            items: Vec::new(),
        };
    }

    #[test]
    fn Test_Answered_Should_Count_Both_Read_And_Approximate_Members()
    {
        assert_eq!(An_Index().Answered(), 2);
    }

    #[test]
    fn Test_Unreachable_Should_Count_Only_Members_With_No_Readable_Answer()
    {
        assert_eq!(An_Index().Unreachable(), 1);
    }

    #[test]
    fn Test_Approximated_Should_Count_Only_Members_Answered_By_A_Weaker_Provider()
    {
        assert_eq!(An_Index().Approximated(), 1);
    }
}
