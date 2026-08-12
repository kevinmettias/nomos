//! What became of one block between two revisions.

pub(crate) mod document;
pub(crate) mod member;

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
