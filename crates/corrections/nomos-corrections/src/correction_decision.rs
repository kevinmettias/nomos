//! Whether a correction was selected automatically, or stopped for review.

use crate::{CorrectionChoice, CorrectionId};

/// `COR-013`: "If no candidate dominates or the choice changes public behavior or
/// architecture, Nomos shall stop at a reviewed proposal instead of selecting
/// automatically."
///
/// The two-way branch the corpus sentence itself states: a choice was made
/// automatically, or the process stopped short of one and left a reviewed proposal
/// instead. This type does not compute which arm applies -- the same declared-not-computed
/// boundary [`crate::CorrectionClass`] and [`crate::CandidateLabel`] already draw --
/// it only carries whichever the caller already decided.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorrectionDecision
{
    /// A candidate was selected without stopping for review.
    Automatic(CorrectionChoice),
    /// No candidate was selected; these candidates were left for a reviewer instead.
    ReviewedProposal
    {
        candidates: Vec<CorrectionId>,
        reason: ReviewReason,
    },
}

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
    use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, Edit};

    fn Candidate(description: &str) -> CorrectionCandidate
    {
        let edit = Edit::New("a.rs", None, Some(description.to_owned()));
        let change = ChangeSet::Empty().With(edit);
        return CorrectionCandidate::New(description, change, CorrectionClass::Mechanical, vec![]);
    }

    #[test]
    fn Test_Labels_Are_Distinct()
    {
        let mut labels = vec![ReviewReason::NoDominantCandidate.Label(), ReviewReason::ChangesPublicBehaviorOrArchitecture.Label()];
        let count = labels.len();
        labels.sort_unstable();
        labels.dedup();

        assert_eq!(labels.len(), count, "two reasons share a wire spelling");
    }

    #[test]
    fn Test_An_Automatic_Decision_Carries_Its_Choice()
    {
        let selected = Candidate("winner").Id();
        let choice = CorrectionChoice::New(selected, vec![], vec![], vec![], vec![], vec![]);

        let decision = CorrectionDecision::Automatic(choice.clone());

        assert_eq!(decision, CorrectionDecision::Automatic(choice));
    }

    #[test]
    fn Test_A_Reviewed_Proposal_Carries_Its_Candidates_And_Reason()
    {
        let candidate = Candidate("either could work").Id();

        let decision = CorrectionDecision::ReviewedProposal {
            candidates: vec![candidate],
            reason: ReviewReason::NoDominantCandidate,
        };

        match decision
        {
            CorrectionDecision::ReviewedProposal { candidates, reason } =>
            {
                assert_eq!(candidates, [candidate]);
                assert_eq!(reason, ReviewReason::NoDominantCandidate);
            }
            CorrectionDecision::Automatic(_) => panic!("expected a reviewed proposal"),
        }
    }
}
