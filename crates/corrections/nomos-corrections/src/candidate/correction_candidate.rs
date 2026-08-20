//! A proposed correction: why, and what it would change.

use super::CorrectionId;
use crate::ChangeSet;
use nomos_model::Digest_Of_Parts;

/// A proposed correction: a description of why, and the [`ChangeSet`] that would carry it
/// out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrectionCandidate
{
    id: CorrectionId,
    description: String,
    change: ChangeSet,
}

impl CorrectionCandidate
{
    #[must_use]
    pub fn New(description: impl Into<String>, change: ChangeSet) -> Self
    {
        let description = description.into();
        let id = Identity_Of(&description, &change);

        return Self {
            id,
            description,
            change,
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
}

/// A digest of the description and every edit, each part length-framed so that one edit's
/// boundary cannot be mistaken for another's.
fn Identity_Of(description: &str, change: &ChangeSet) -> CorrectionId
{
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
        let edit = Edit::New("a.rs", None, Some("x".to_owned()));
        let change = ChangeSet::Empty().With(edit);

        let one = CorrectionCandidate::New("fix a", change.clone());
        let other = CorrectionCandidate::New("fix a", change);

        assert_eq!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Description_Should_Change_The_Identity()
    {
        let edit = Edit::New("a.rs", None, Some("x".to_owned()));
        let change = ChangeSet::Empty().With(edit);

        let one = CorrectionCandidate::New("fix a", change.clone());
        let other = CorrectionCandidate::New("fix a differently", change);

        assert_ne!(one.Id(), other.Id());
    }

    #[test]
    fn Test_A_Different_Change_Should_Change_The_Identity()
    {
        let edit_one = Edit::New("a.rs", None, Some("x".to_owned()));
        let one = CorrectionCandidate::New("fix a", ChangeSet::Empty().With(edit_one));

        let edit_other = Edit::New("a.rs", None, Some("y".to_owned()));
        let other = CorrectionCandidate::New("fix a", ChangeSet::Empty().With(edit_other));

        assert_ne!(one.Id(), other.Id());
    }
}
