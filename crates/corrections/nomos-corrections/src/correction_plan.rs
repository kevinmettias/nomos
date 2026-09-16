//! A group of candidates, checked against each other before it is checked against anything
//! live.

use crate::{CorrectionCandidate, CorrectionError, Preview, StagedPlan};
use nomos_workspace::Workspace;

/// One or more candidates meant to be staged, validated, committed and rolled back
/// together.
///
/// Constructing a plan is the first refusal a correction can hit, and the cheapest: two
/// candidates that touch the same path disagree with each other regardless of what
/// workspace either was built against, so that is caught here rather than carried forward
/// to [`CorrectionPlan::Stage`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionPlan
{
    candidates: Vec<CorrectionCandidate>,
}

impl CorrectionPlan
{
    /// # Errors
    ///
    /// Returns [`CorrectionError::Vacuous`] if `candidates` is empty, or
    /// [`CorrectionError::Conflicting`] if two candidates touch the same path.
    pub fn New(candidates: Vec<CorrectionCandidate>) -> Result<Self, CorrectionError>
    {
        if candidates.is_empty()
        {
            return Err(CorrectionError::Vacuous);
        }

        let overlapping = Overlapping_Paths(&candidates);
        if !overlapping.is_empty()
        {
            return Err(CorrectionError::Conflicting {
                paths: overlapping,
            });
        }

        return Ok(Self { candidates });
    }

    #[must_use]
    pub fn Candidates(&self) -> &[CorrectionCandidate]
    {
        return &self.candidates;
    }

    /// What this plan would do, rendered without touching `base`.
    #[must_use]
    pub fn Preview(&self) -> Preview
    {
        return Preview::Of(self);
    }

    /// Checks every candidate's declared prior content against `base`, and carries the
    /// combined forward and reverse changes forward for [`StagedPlan::Validate`].
    ///
    /// # Errors
    ///
    /// Returns [`CorrectionError::StaleCandidate`] if a candidate's declared prior content
    /// at a path does not match what `base` currently holds there.
    pub fn Stage(&self, base: &Workspace) -> Result<StagedPlan, CorrectionError>
    {
        return StagedPlan::Of(self, base);
    }
}

/// Every path touched by more than one candidate, in a stable order.
fn Overlapping_Paths(candidates: &[CorrectionCandidate]) -> Vec<String>
{
    use std::collections::BTreeSet;

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut overlapping: BTreeSet<&str> = BTreeSet::new();

    for candidate in candidates
    {
        for path in candidate.Change().Touched()
        {
            if !seen.insert(path)
            {
                overlapping.insert(path);
            }
        }
    }

    return overlapping.into_iter().map(str::to_owned).collect();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{ChangeSet, CorrectionClass, Edit};

    const CANDIDATE_COUNT: usize = 2;

    /// A configuration seed no other case in this file uses, so this case's snapshot
    /// digest cannot collide with one a sibling test built.
    const STAGED_WORKSPACE_SEED_BYTE: u8 = 0x44;

    struct Description<'a>(&'a str);
    struct Path<'a>(&'a str);

    fn Candidate_Touching(description: Description<'_>, path: Path<'_>) -> CorrectionCandidate
    {
        let edit = Edit::New(path.0, None, Some("x".to_owned()));

        return CorrectionCandidate::New(description.0, ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, vec![]);
    }

    #[test]
    fn Test_New_Should_Refuse_An_Empty_Candidate_List()
    {
        let refusal = CorrectionPlan::New(Vec::new()).expect_err("no candidates is vacuous");

        assert_eq!(refusal, CorrectionError::Vacuous);
    }

    #[test]
    fn Test_Preview_Should_Render_Without_Touching_Any_Workspace()
    {
        let plan = CorrectionPlan::New(vec![Candidate_Touching(Description("first"), Path("a.rs"))]).expect("one candidate is a valid plan");

        let preview = plan.Preview();

        assert!(!preview.Rendered().is_empty());
    }

    #[test]
    fn Test_Stage_Should_Check_Prior_Content_Against_The_Base_Workspace()
    {
        let workspace =
            crate::test_support::Base(STAGED_WORKSPACE_SEED_BYTE).expect("one Present applies to an empty workspace");
        let edit = Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()));
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New("fix a", ChangeSet::Empty().With(edit), CorrectionClass::Mechanical, vec![])])
            .expect("one candidate is a valid plan");

        let staged = plan.Stage(&workspace).expect("the candidate's declared prior content matches");

        assert_eq!(staged.Base(), workspace.Id());
    }

    #[test]
    fn Test_Two_Candidates_Touching_The_Same_Path_Should_Be_Refused()
    {
        let refusal = CorrectionPlan::New(vec![
            Candidate_Touching(Description("first"), Path("a.rs")),
            Candidate_Touching(Description("second"), Path("a.rs")),
        ])
        .expect_err("two candidates on one path conflict");

        assert_eq!(
            refusal,
            CorrectionError::Conflicting {
                paths: vec!["a.rs".to_owned()]
            }
        );
    }

    #[test]
    fn Test_Candidates_Touching_Different_Paths_Should_Be_Accepted()
    {
        let plan = CorrectionPlan::New(vec![
            Candidate_Touching(Description("first"), Path("a.rs")),
            Candidate_Touching(Description("second"), Path("b.rs")),
        ])
            .expect("disjoint candidates form a plan");

        assert_eq!(plan.Candidates().len(), CANDIDATE_COUNT);
    }
}
