//! Every edit one correction candidate proposes, as one thing to reason about.

use crate::Edit;
use std::collections::BTreeSet;

/// A batch of edits one candidate proposes.
///
/// A correction's own change payload, distinct from [`nomos_workspace::WorkspaceChangeSet`]:
/// this one carries both directions of every edit and no [`nomos_workspace::ChangeSource`],
/// because a candidate is not yet a submission to the workspace door — [`crate::StagedPlan`]
/// is what turns a plan's edits into one.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ChangeSet
{
    edits: Vec<Edit>,
}

impl ChangeSet
{
    #[must_use]
    pub fn Empty() -> Self
    {
        return Self::default();
    }

    #[must_use]
    pub fn With(mut self, edit: Edit) -> Self
    {
        self.edits.push(edit);

        return self;
    }

    #[must_use]
    pub fn Edits(&self) -> &[Edit]
    {
        return &self.edits;
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.edits.is_empty();
    }

    /// Every path this change set touches.
    #[must_use]
    pub fn Touched(&self) -> BTreeSet<&str>
    {
        return self.edits.iter().map(Edit::Path).collect();
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// How many edits `Test_Touched_Names_Every_Edited_Path` appends to its set.
    const EDITS_APPENDED: usize = 2;

    #[test]
    fn Test_An_Empty_Change_Set_Touches_Nothing()
    {
        let set = ChangeSet::Empty();

        assert!(set.Is_Empty());
        assert!(set.Touched().is_empty());
    }

    #[test]
    fn Test_Touched_Names_Every_Edited_Path()
    {
        let edit_a = Edit::New("a.rs", None, Some("x".to_owned()));
        let edit_b = Edit::New("b.rs", Some("y".to_owned()), None);
        let set = ChangeSet::Empty().With(edit_a).With(edit_b);

        assert_eq!(set.Touched(), BTreeSet::from(["a.rs", "b.rs"]));
        assert!(!set.Is_Empty());
        assert_eq!(set.Edits().len(), EDITS_APPENDED);
    }

    #[test]
    fn Test_Is_Empty_Should_Be_False_Once_An_Edit_Is_Added()
    {
        let edit = Edit::New("a.rs", None, Some("x".to_owned()));
        let set = ChangeSet::Empty().With(edit);

        assert!(!set.Is_Empty());
    }

    #[test]
    fn Test_With_Should_Append_The_Given_Edit()
    {
        let edit = Edit::New("a.rs", None, Some("x".to_owned()));
        let set = ChangeSet::Empty().With(edit.clone());

        assert_eq!(set.Edits(), [edit]);
    }

    #[test]
    fn Test_Edits_Should_Report_Every_Appended_Edit_In_Order()
    {
        let edit_a = Edit::New("a.rs", None, Some("x".to_owned()));
        let edit_b = Edit::New("b.rs", Some("y".to_owned()), None);
        let set = ChangeSet::Empty().With(edit_a.clone()).With(edit_b.clone());

        assert_eq!(set.Edits(), [edit_a, edit_b]);
    }
}
