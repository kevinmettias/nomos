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
