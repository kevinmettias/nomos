//! What sort of thing a stored document is.

use serde::{Deserialize, Serialize};

use crate::Authority;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentKind
{
    Commit,
    Fact,
    Finding,
    Run,
    Specification,
    Record,
    Projection,
}

impl DocumentKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Commit => "commit",
            Self::Fact => "fact",
            Self::Finding => "finding",
            Self::Run => "run",
            Self::Specification => "specification",
            Self::Record => "record",
            Self::Projection => "projection",
        };
    }

    #[must_use]
    pub const fn Authority(self) -> Authority
    {
        return match self
        {
            Self::Commit | Self::Fact | Self::Finding | Self::Run => Authority::Observed,
            Self::Specification | Self::Record | Self::Projection => Authority::Authored,
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Commit,
            Self::Fact,
            Self::Finding,
            Self::Run,
            Self::Specification,
            Self::Record,
            Self::Projection,
        ];
    }
}
