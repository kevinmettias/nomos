//! What was recorded about choosing one correction candidate over its alternatives.

use crate::{CorrectionId, RankingCriterion};

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

/// `COR-012`'s own five recorded fields, apart from `selected` itself -- grouped into one
/// value so [`CorrectionChoice::New`] stays within this crate's own parameter-count limit.
pub struct ChoiceRecord
{
    pub objective_weights: Vec<(RankingCriterion, u32)>,
    pub rejected_alternatives: Vec<CorrectionId>,
    pub predicted_side_effects: Vec<String>,
    pub unresolved_tradeoffs: Vec<String>,
    pub verification_obligations: Vec<String>,
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

    fn Candidate(description: &str) -> CorrectionCandidate
    {
        let edit = Edit::New("a.rs", None, Some(description.to_owned()));
        let change = ChangeSet::Empty().With(edit);
        return CorrectionCandidate::New(description, change, CorrectionClass::Mechanical, vec![]);
    }

    #[test]
    fn Test_A_Choice_Carries_Exactly_What_It_Was_Given()
    {
        let selected = Candidate("winner").Id();
        let rejected = Candidate("loser").Id();

        let choice = CorrectionChoice::New(
            selected,
            ChoiceRecord {
                objective_weights: vec![(RankingCriterion::BehaviorPreservation, 3)],
                rejected_alternatives: vec![rejected],
                predicted_side_effects: vec!["may slow the hot path".to_owned()],
                unresolved_tradeoffs: vec!["unclear whether callers rely on the old error message".to_owned()],
                verification_obligations: vec!["run the integration suite".to_owned()],
            },
        );

        assert_eq!(choice.Selected(), selected);
        assert_eq!(choice.Objective_Weights(), [(RankingCriterion::BehaviorPreservation, 3)]);
        assert_eq!(choice.Rejected_Alternatives(), [rejected]);
        assert_eq!(choice.Predicted_Side_Effects(), ["may slow the hot path"]);
        assert_eq!(choice.Unresolved_Tradeoffs(), ["unclear whether callers rely on the old error message"]);
        assert_eq!(choice.Verification_Obligations(), ["run the integration suite"]);
    }
}
