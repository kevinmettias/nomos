//! What a plan would do, rendered without touching any workspace.

use crate::{CorrectionCandidate, CorrectionPlan, Edit};
use nomos_contracts::MutationClass;

/// A deterministic rendering of a plan's candidates and their edits.
///
/// Pure over the plan alone. Previewing never touches a [`nomos_workspace::Workspace`],
/// so it carries no [`crate::CorrectionError`] and cannot go stale — a live check of
/// whether the plan still applies is what [`crate::CorrectionPlan::Stage`] is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview
{
    rendered: Vec<u8>,
}

impl Preview
{
    #[must_use]
    pub(crate) fn Of(plan: &CorrectionPlan) -> Self
    {
        let mut rendered = Vec::new();

        for candidate in plan.Candidates()
        {
            Render_Candidate(&mut rendered, candidate);
        }

        return Self { rendered };
    }

    #[must_use]
    pub fn Rendered(&self) -> &[u8]
    {
        return &self.rendered;
    }

    /// `AGT-EXEC-001`'s capability-class question, answered for this operation:
    /// `nomos_contracts::MutationClass::Preview`, the same class
    /// `CorrectionPlan::Preview` -- the method that produces a value of this type --
    /// belongs to. `Self::Mutation_Class().Required_Authority()` is the
    /// `nomos_contracts::AuthorityClass` an actor would need to have called it.
    #[must_use]
    pub const fn Mutation_Class() -> MutationClass
    {
        return MutationClass::Preview;
    }
}

/// Appends one candidate's header and every one of its edits to `rendered`.
fn Render_Candidate(rendered: &mut Vec<u8>, candidate: &CorrectionCandidate)
{
    Render_Header(rendered, candidate);

    for edit in candidate.Change().Edits()
    {
        Render_Edit(rendered, edit);
    }
}

/// Appends a candidate's identity and description lines to `rendered`.
fn Render_Header(rendered: &mut Vec<u8>, candidate: &CorrectionCandidate)
{
    rendered.extend_from_slice(format!("candidate\t{}\n", candidate.Id()).as_bytes());
    rendered.extend_from_slice(format!("description\t{}\n", candidate.Description()).as_bytes());
}

/// Appends one edit's line to `rendered`.
fn Render_Edit(rendered: &mut Vec<u8>, edit: &Edit)
{
    rendered.extend_from_slice(
        format!(
            "edit\t{}\t{}\t{}\n",
            edit.Path(),
            edit.Before().unwrap_or("-"),
            edit.After().unwrap_or("-")
        )
        .as_bytes(),
    );
}

#[cfg(test)]
mod tests
{
    use crate::{ChangeSet, CorrectionCandidate, CorrectionClass, CorrectionPlan, Edit, Preview};
    use nomos_contracts::{AuthorityClass, MutationClass};

    #[test]
    fn Test_A_Preview_Names_Every_Candidate_And_Edit()
    {
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", Some("old".to_owned()), Some("new".to_owned()))),
            CorrectionClass::Mechanical,
            vec![],
        )])
        .expect("one candidate is a valid plan");

        let rendered = String::from_utf8(plan.Preview().Rendered().to_vec()).expect("utf8");

        assert!(rendered.contains("description\tfix a\n"));
        assert!(rendered.contains("edit\ta.rs\told\tnew\n"));
    }

    #[test]
    fn Test_Previewing_The_Same_Plan_Twice_Should_Agree()
    {
        let plan = CorrectionPlan::New(vec![CorrectionCandidate::New(
            "fix a",
            ChangeSet::Empty().With(Edit::New("a.rs", None, Some("new".to_owned()))),
            CorrectionClass::Mechanical,
            vec![],
        )])
        .expect("one candidate is a valid plan");

        assert_eq!(plan.Preview(), plan.Preview());
    }

    #[test]
    fn Test_A_Preview_Belongs_To_The_Preview_Mutation_Class()
    {
        assert_eq!(Preview::Mutation_Class(), MutationClass::Preview);
        assert_eq!(
            Preview::Mutation_Class().Required_Authority(),
            AuthorityClass::Preview,
            "previewing must never require Mutate authority, or callers skip it"
        );
    }
}
