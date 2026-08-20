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
        assert_eq!(set.Edits().len(), 2);
    }
}
