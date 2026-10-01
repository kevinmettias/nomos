use nomos_contracts::Applicability;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadOutcome
{
    Materialized,
    Absent,
    Superseded,
    Degraded(Applicability),
}

impl ReadOutcome
{
    #[must_use]
    pub const fn Is_Answered(self) -> bool
    {
        return matches!(self, Self::Materialized);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Is_Answered_Should_Be_True_Only_For_Materialized()
    {
        assert!(ReadOutcome::Materialized.Is_Answered());
        assert!(!ReadOutcome::Absent.Is_Answered());
        assert!(!ReadOutcome::Superseded.Is_Answered());
        assert!(!ReadOutcome::Degraded(Applicability::SupportedWithFallback).Is_Answered());
    }
}
