//! `COR-013`'s own two named conditions that stop automatic selection.

/// `COR-013`'s own two named conditions that stop automatic selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReviewReason
{
    /// No candidate dominates the others under the declared ranking.
    NoDominantCandidate,
    /// The choice would change public behavior or architecture.
    ChangesPublicBehaviorOrArchitecture,
}

impl ReviewReason
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::NoDominantCandidate => "no_dominant_candidate",
            Self::ChangesPublicBehaviorOrArchitecture => "changes_public_behavior_or_architecture",
        };
    }
}

impl core::fmt::Display for ReviewReason
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

    #[test]
    fn Test_Label_Should_Be_Distinct_Per_Variant()
    {
        let mut labels = vec![ReviewReason::NoDominantCandidate.Label(), ReviewReason::ChangesPublicBehaviorOrArchitecture.Label()];
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two reasons share a wire spelling");
    }
}
