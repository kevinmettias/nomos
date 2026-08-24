//! The named dimensions correction candidates are ranked along.

/// `COR-011`: "Candidate ranking shall consider normative correctness, behavior
/// preservation, edit scope, API stability, architecture fit, secondary findings,
/// reversibility, confidence, verification cost, and user/repository preference. Lowest
/// finding count alone is not a sufficient objective."
///
/// Ten dimensions, in the corpus's own order. This crate does not rank anything with
/// them -- the same declared-not-computed boundary [`crate::CorrectionClass`] and
/// [`crate::CandidateLabel`] already draw -- it only names the vocabulary a future
/// ranking function's objective weights would be keyed by. The second sentence is a
/// constraint on that future function (it must not degenerate to a single dimension),
/// not an eleventh variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RankingCriterion
{
    /// Whether the candidate is normatively correct.
    NormativeCorrectness,
    /// Whether the candidate preserves observable behavior.
    BehaviorPreservation,
    /// How much of the workspace the candidate's edits touch.
    EditScope,
    /// Whether the candidate preserves a public interface's stability.
    ApiStability,
    /// How well the candidate fits the existing architecture.
    ArchitectureFit,
    /// Findings the candidate would introduce or resolve beyond the one it targets.
    SecondaryFindings,
    /// How easily the candidate can be undone.
    Reversibility,
    /// How confident the candidate's proposer is in it.
    Confidence,
    /// The cost of verifying the candidate.
    VerificationCost,
    /// A standing preference the user or repository has already declared.
    UserPreference,
}

impl RankingCriterion
{
    /// The variant's stable, lowercase wire spelling.
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::NormativeCorrectness => "normative_correctness",
            Self::BehaviorPreservation => "behavior_preservation",
            Self::EditScope => "edit_scope",
            Self::ApiStability => "api_stability",
            Self::ArchitectureFit => "architecture_fit",
            Self::SecondaryFindings => "secondary_findings",
            Self::Reversibility => "reversibility",
            Self::Confidence => "confidence",
            Self::VerificationCost => "verification_cost",
            Self::UserPreference => "user_preference",
        };
    }
}

impl core::fmt::Display for RankingCriterion
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

    const ALL: [RankingCriterion; 10] = [
        RankingCriterion::NormativeCorrectness,
        RankingCriterion::BehaviorPreservation,
        RankingCriterion::EditScope,
        RankingCriterion::ApiStability,
        RankingCriterion::ArchitectureFit,
        RankingCriterion::SecondaryFindings,
        RankingCriterion::Reversibility,
        RankingCriterion::Confidence,
        RankingCriterion::VerificationCost,
        RankingCriterion::UserPreference,
    ];

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels: Vec<&str> = ALL.iter().map(|criterion| return criterion.Label()).collect();
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two criteria share a wire spelling");
    }
}
