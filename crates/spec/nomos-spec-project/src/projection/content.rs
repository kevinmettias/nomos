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

    /// Every content a profile section can select.
    ///
    /// Mirrored by `Test_Every_Content_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in this file. It fails to compile,
    /// not merely to pass, if a variant is added here without being added there.
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

#[cfg(test)]
mod tests
{
    use super::*;

    /// `Content::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Content` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Content_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of_Content(content: Content) -> usize
        {
            return match content
            {
                Content::Suites => 0,
                Content::Documents => 1,
                Content::Headings => 2,
                Content::Blocks => 3,
                Content::Rows => 4,
                Content::Nodes => 5,
                Content::Statements => 6,
                Content::Relations => 7,
                Content::Lineage => 8,
                Content::Omissions => 9,
            };
        }

        for (index, content) in Content::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal_Of_Content(*content),
                index,
                "{} is not matched at the position Content::All() puts it, so the \
                 exhaustive match and the universe have drifted apart",
                content.Label()
            );
        }
    }
}
