//! A proposed correction: why, and what it would change.

use super::CorrectionId;
use crate::{CandidateLabel, ChangeSet, CorrectionClass};

/// A proposed correction: a description of why, and the [`ChangeSet`] that would carry it
/// out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionCandidate
{
    id: CorrectionId,
    description: String,
    change: ChangeSet,
    class: CorrectionClass,
    labels: Vec<CandidateLabel>,
}

impl CorrectionCandidate
{
    /// Constructs a candidate. `class` is `COR-001`'s fix-action class and `labels` is
    /// `COR-010`'s independent set of descriptive labels -- both declared by the caller,
    /// never computed here, the same way `ValidatedPlan::Commit`'s `Evidence` parameter
    /// is declared rather than judged (`OD-CORRECTIONS-002`).
    #[must_use]
    pub fn New(
        description: impl Into<String>,
        change: ChangeSet,
        class: CorrectionClass,
        labels: Vec<CandidateLabel>,
    ) -> Self
    {
        let description = description.into();
        let id = Identity_Of(&description, &change);

        return Self {
            id,
            description,
            change,
            class,
            labels,
        };
    }

    #[must_use]
    pub const fn Id(&self) -> CorrectionId
    {
        return self.id;
    }

    #[must_use]
    pub fn Description(&self) -> &str
    {
        return &self.description;
    }

    #[must_use]
    pub const fn Change(&self) -> &ChangeSet
    {
        return &self.change;
    }

    /// `COR-001`'s fix-action class this candidate was declared under.
    #[must_use]
    pub const fn Class(&self) -> CorrectionClass
    {
        return self.class;
    }

    /// `COR-010`'s independent descriptive labels this candidate was declared with.
    #[must_use]
    pub fn Labels(&self) -> &[CandidateLabel]
    {
        return &self.labels;
    }
}

/// A digest of the description and every edit, each part length-framed so that one edit's
/// boundary cannot be mistaken for another's.
fn Identity_Of(description: &str, change: &ChangeSet) -> CorrectionId
{
    use nomos_model::Digest_Of_Parts;

    let mut parts: Vec<&[u8]> = vec![description.as_bytes()];

    for edit in change.Edits()
    {
        parts.push(edit.Path().as_bytes());
        parts.push(edit.Before().unwrap_or_default().as_bytes());
        parts.push(edit.After().unwrap_or_default().as_bytes());
    }

    return CorrectionId::From_Digest(Digest_Of_Parts(&parts));
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Edit;

    #[test]
    fn Test_Identical_Candidates_Should_Share_An_Identity()
    {
        let change = Change_Setting_A_To("x");

        let one = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let other = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, vec![]);

        assert_eq!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Description_Should_Change_The_Identity()
    {
        let change = Change_Setting_A_To("x");

        let one = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let other = CorrectionCandidate::New("fix a differently", change, CorrectionClass::Mechanical, vec![]);

        assert_ne!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Change_Should_Change_The_Identity()
    {
        let one = CorrectionCandidate::New("fix a", Change_Setting_A_To("x"), CorrectionClass::Mechanical, vec![]);

        let other = CorrectionCandidate::New("fix a", Change_Setting_A_To("y"), CorrectionClass::Mechanical, vec![]);

        assert_ne!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Class_Or_Labels_Should_Not_Change_The_Identity()
    {
        let change = Change_Setting_A_To("x");

        let mechanical = CorrectionCandidate::New("fix a", change.clone(), CorrectionClass::Mechanical, vec![]);
        let agent = CorrectionCandidate::New(
            "fix a",
            change,
            CorrectionClass::Agent,
            vec![CandidateLabel::AgentProposed, CandidateLabel::Speculative],
        );

        assert_eq!(
            mechanical.Id(),
            agent.Id(),
            "class and labels are declared metadata, not part of what identifies a correction"
        );
    }

    #[test]
    fn Test_Class_And_Labels_Are_Carried_Rather_Than_Computed()
    {
        let change = Change_Setting_A_To("x");
        let labels = vec![CandidateLabel::MechanicallySafe, CandidateLabel::BehaviorPreserving];

        let candidate = CorrectionCandidate::New("fix a", change, CorrectionClass::Mechanical, labels.clone());

        assert_eq!(candidate.Class(), CorrectionClass::Mechanical);
        assert_eq!(candidate.Labels(), labels.as_slice());
    }

    /// The one-edit changeset every test above builds: `a.rs` set to `content`, with no
    /// prior content declared. Encapsulated once so the same `Edit::New` /
    /// `ChangeSet::Empty().With` pairing is not repeated at every call site.
    fn Change_Setting_A_To(content: &str) -> ChangeSet
    {
        let edit = Edit::New("a.rs", None, Some(content.to_owned()));
        return ChangeSet::Empty().With(edit);
    }
}
