//! What a profile section selects.

use serde::Serialize;
use serde::Deserialize;
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Content
{
    Suites,
    Documents,
    Headings,
    Blocks,
    Rows,
    Nodes,
    Statements,
    Relations,
    Lineage,
    Omissions,
}

impl Content
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Suites => "suites",
            Self::Documents => "documents",
            Self::Headings => "headings",
            Self::Blocks => "blocks",
            Self::Rows => "rows",
            Self::Nodes => "nodes",
            Self::Statements => "statements",
            Self::Relations => "relations",
            Self::Lineage => "lineage",
            Self::Omissions => "omissions",
        };
    }

    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Suites,
            Self::Documents,
            Self::Headings,
            Self::Blocks,
            Self::Rows,
            Self::Nodes,
            Self::Statements,
            Self::Relations,
            Self::Lineage,
            Self::Omissions,
        ];
    }
}
