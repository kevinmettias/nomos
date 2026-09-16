//! What was recorded about choosing one correction candidate over its alternatives.

use crate::{ChoiceRecord, CorrectionId, RankingCriterion};

/// `COR-012`: "Correction choice shall record the selected objective weights, rejected
/// alternatives, predicted side effects, unresolved tradeoffs, and verification
/// obligations."
///
/// Five fields, each a direct transcription of one named noun. Constructed and carried
/// by the caller, the same declared-not-computed pattern
/// [`crate::CorrectionCandidate::New`]'s `class`/`labels` parameters and
/// [`crate::ValidatedPlan::Commit`]'s `Evidence` parameter already use -- this type does
/// not decide which candidate wins, only records that a decision was made and what it
/// weighed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionChoice
{
    selected: CorrectionId,
    objective_weights: Vec<(RankingCriterion, u32)>,
    rejected_alternatives: Vec<CorrectionId>,
    predicted_side_effects: Vec<String>,
    unresolved_tradeoffs: Vec<String>,
    verification_obligations: Vec<String>,
}

impl CorrectionChoice
{
    /// Records a choice. `selected` is the winning candidate's identity; `record` is
    /// `COR-012`'s own five recorded fields, declared by the caller.
    #[must_use]
    pub fn New(selected: CorrectionId, record: ChoiceRecord) -> Self
    {
        return Self {
            selected,
            objective_weights: record.objective_weights,
            rejected_alternatives: record.rejected_alternatives,
            predicted_side_effects: record.predicted_side_effects,
            unresolved_tradeoffs: record.unresolved_tradeoffs,
            verification_obligations: record.verification_obligations,
        };
    }

    #[must_use]
    pub const fn Selected(&self) -> CorrectionId
    {
        return self.selected;
    }

    #[must_use]
    pub fn Objective_Weights(&self) -> &[(RankingCriterion, u32)]
    {
        return &self.objective_weights;
    }

    #[must_use]
    pub fn Rejected_Alternatives(&self) -> &[CorrectionId]
    {
        return &self.rejected_alternatives;
    }

    #[must_use]
    pub fn Predicted_Side_Effects(&self) -> &[String]
    {
        return &self.predicted_side_effects;
    }

    #[must_use]
    pub fn Unresolved_Tradeoffs(&self) -> &[String]
    {
        return &self.unresolved_tradeoffs;
    }

    #[must_use]
    pub fn Verification_Obligations(&self) -> &[String]
    {
        return &self.verification_obligations;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, Edit};

    /// The weight `Choice_Over` records against `RankingCriterion::BehaviorPreservation`.
    const BEHAVIOR_PRESERVATION_WEIGHT: u32 = 3;

    #[test]
    fn Test_New_Should_Carry_Every_Field_It_Was_Constructed_With()
    {
        let selected = Candidate_Named("winner").Id();
        let rejected = Candidate_Named("loser").Id();
        let choice = Choice_Over(selected, rejected);

        assert_eq!(choice.Selected(), selected);
        assert_eq!(choice.Rejected_Alternatives(), [rejected]);
    }

    #[test]
    fn Test_Selected_Should_Report_The_Winning_Candidates_Identity()
    {
        let selected = Candidate_Named("winner").Id();
        let choice = Choice_Over(selected, Candidate_Named("loser").Id());

        assert_eq!(choice.Selected(), selected);
    }

    #[test]
    fn Test_Objective_Weights_Should_Report_What_The_Choice_Was_Given()
    {
        let choice = Choice_Over(Candidate_Named("winner").Id(), Candidate_Named("loser").Id());

        assert_eq!(choice.Objective_Weights(), [(RankingCriterion::BehaviorPreservation, BEHAVIOR_PRESERVATION_WEIGHT)]);
    }

    #[test]
    fn Test_Rejected_Alternatives_Should_List_Every_Candidate_That_Lost()
    {
        let rejected = Candidate_Named("loser").Id();
        let choice = Choice_Over(Candidate_Named("winner").Id(), rejected);

        assert_eq!(choice.Rejected_Alternatives(), [rejected]);
    }

    #[test]
    fn Test_Predicted_Side_Effects_Should_Report_What_The_Choice_Was_Given()
    {
        let choice = Choice_Over(Candidate_Named("winner").Id(), Candidate_Named("loser").Id());

        assert_eq!(choice.Predicted_Side_Effects(), ["may slow the hot path"]);
    }

    #[test]
    fn Test_Unresolved_Tradeoffs_Should_Report_What_The_Choice_Was_Given()
    {
        let choice = Choice_Over(Candidate_Named("winner").Id(), Candidate_Named("loser").Id());

        assert_eq!(choice.Unresolved_Tradeoffs(), ["unclear whether callers rely on the old error message"]);
    }

    #[test]
    fn Test_Verification_Obligations_Should_Report_What_The_Choice_Was_Given()
    {
        let choice = Choice_Over(Candidate_Named("winner").Id(), Candidate_Named("loser").Id());

        assert_eq!(choice.Verification_Obligations(), ["run the integration suite"]);
    }

    /// The one choice record every test above builds against: one objective weight, one
    /// rejected alternative, and the same predicted side effect, unresolved tradeoff and
    /// verification obligation -- encapsulated once so the fields under test are not
    /// repeated at every call site.
    fn Choice_Over(selected: crate::CorrectionId, rejected: crate::CorrectionId) -> CorrectionChoice
    {
        return CorrectionChoice::New(
            selected,
            ChoiceRecord {
                objective_weights: vec![(RankingCriterion::BehaviorPreservation, BEHAVIOR_PRESERVATION_WEIGHT)],
                rejected_alternatives: vec![rejected],
                predicted_side_effects: vec!["may slow the hot path".to_owned()],
                unresolved_tradeoffs: vec!["unclear whether callers rely on the old error message".to_owned()],
                verification_obligations: vec!["run the integration suite".to_owned()],
            },
        );
    }

    fn Candidate_Named(description: &str) -> CorrectionCandidate
    {
        let edit = Edit::New("a.rs", None, Some(description.to_owned()));
        let change = ChangeSet::Empty().With(edit);
        return CorrectionCandidate::New(description, change, CorrectionClass::Mechanical, vec![]);
    }
}
