//! How reversible one correction operation's effect is.

/// `COR-EXEC-006`: "Each operation shall declare its rollback boundary and whether
/// reversal is Exact, Compensating, RepositoryRecoverable, ProviderDependent, or
/// Irreversible. Parallel groups shall not cross incompatible rollback boundaries."
///
/// Five variants, in the corpus's own order. The second sentence constrains a scheduler
/// this crate does not have -- [`crate::CorrectionPlan`] runs one plan at a time, with no
/// concept of a parallel group -- so this type carries only the first sentence's
/// declaration, the same second-sentence carve-out [`crate::RankingCriterion`] already
/// draws for `COR-011`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RollbackBoundary
{
    /// Reversal restores exactly what was there before, byte for byte.
    Exact,
    /// Reversal restores equivalent state through a compensating action rather than the
    /// same bytes.
    Compensating,
    /// Reversal is only possible by recovering from elsewhere in the repository, not from
    /// what the operation itself retained.
    RepositoryRecoverable,
    /// Whether reversal is possible depends on state external to this repository.
    ProviderDependent,
    /// The operation cannot be reversed.
    Irreversible,
}

impl RollbackBoundary
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Exact => "exact",
            Self::Compensating => "compensating",
            Self::RepositoryRecoverable => "repository_recoverable",
            Self::ProviderDependent => "provider_dependent",
            Self::Irreversible => "irreversible",
        };
    }
}

impl core::fmt::Display for RollbackBoundary
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.write_str(self.Label());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const ALL: [RollbackBoundary; 5] = [
        RollbackBoundary::Exact,
        RollbackBoundary::Compensating,
        RollbackBoundary::RepositoryRecoverable,
        RollbackBoundary::ProviderDependent,
        RollbackBoundary::Irreversible,
    ];

    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|boundary| return boundary.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two boundaries share a wire spelling");
    }
}
