//! What became of one block between two revisions.

pub(crate) mod document_fate;
pub(crate) mod member_fate;

use crate::Hollow;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fate
{
    Preserved
    {
        document: String,
    },
    Hollowed
    {
        document: String,
        evidence: Hollow,
    },
    Mentioned
    {
        documents: Vec<String>,
    },
    Gone,
}

impl Fate
{
    #[must_use]
    pub const fn Label(&self) -> &'static str
    {
        return match self
        {
            Self::Preserved { .. } => "preserved",
            Self::Hollowed { .. } => "hollowed",
            Self::Mentioned { .. } => "mentioned",
            Self::Gone => "gone",
        };
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Label_Should_Name_Each_Fate_Variant()
    {
        assert_eq!(
            Fate::Preserved { document: "d.md".to_owned() }.Label(),
            "preserved"
        );
        assert_eq!(
            Fate::Hollowed { document: "d.md".to_owned(), evidence: Hollow::NoBody }.Label(),
            "hollowed"
        );
        assert_eq!(Fate::Mentioned { documents: vec!["d.md".to_owned()] }.Label(), "mentioned");
        assert_eq!(Fate::Gone.Label(), "gone");
    }
}
