//! Whether a correction was selected automatically, or stopped for review.

use crate::{CorrectionChoice, CorrectionId};

mod review_reason;

pub use review_reason::ReviewReason;

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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ChangeSet, ChoiceRecord, CorrectionCandidate, CorrectionClass, Edit};

    #[test]
    fn Test_An_Automatic_Decision_Carries_Its_Choice()
    {
        let selected = Candidate_Named("winner").Id();
        let choice = CorrectionChoice::New(
            selected,
            ChoiceRecord {
                objective_weights: vec![],
                rejected_alternatives: vec![],
                predicted_side_effects: vec![],
                unresolved_tradeoffs: vec![],
                verification_obligations: vec![],
            },
        );

        let decision = CorrectionDecision::Automatic(choice.clone());

        assert_eq!(decision, CorrectionDecision::Automatic(choice));
    }

    #[test]
    fn Test_A_Reviewed_Proposal_Carries_Its_Candidates_And_Reason()
    {
        let candidate = Candidate_Named("either could work").Id();

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
            // A test assertion, not production control flow: any other arm here is this
            // test's own failure mode, and panicking is how a test reports one.
            CorrectionDecision::Automatic(_) => panic!("expected a reviewed proposal"),
        }
    }

    fn Candidate_Named(description: &str) -> CorrectionCandidate
    {
        let edit = Edit::New("a.rs", None, Some(description.to_owned()));
        let change = ChangeSet::Empty().With(edit);
        return CorrectionCandidate::New(description, change, CorrectionClass::Mechanical, vec![]);
    }
}
