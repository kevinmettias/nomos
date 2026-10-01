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
    Neighbourhood,
    Families,
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
            Self::Neighbourhood => "neighbourhood",
            Self::Families => "families",
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
            Self::Neighbourhood,
            Self::Families,
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Where `Content::All()` puts each variant.
    ///
    /// Named rather than written into the mirror below as bare numbers: a variant's place is
    /// a fact about the universe rather than a value in the arm, and `10` beside `2` reads as
    /// a quantity where `ORDINAL_NEIGHBOURHOOD` beside `ORDINAL_HEADINGS` reads as an order.
    const ORDINAL_SUITES: usize = 0;
    const ORDINAL_DOCUMENTS: usize = 1;
    const ORDINAL_HEADINGS: usize = 2;
    const ORDINAL_BLOCKS: usize = 3;
    const ORDINAL_ROWS: usize = 4;
    const ORDINAL_NODES: usize = 5;
    const ORDINAL_STATEMENTS: usize = 6;
    const ORDINAL_RELATIONS: usize = 7;
    const ORDINAL_LINEAGE: usize = 8;
    const ORDINAL_OMISSIONS: usize = 9;
    const ORDINAL_NEIGHBOURHOOD: usize = 10;
    const ORDINAL_FAMILIES: usize = 11;

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
                Content::Suites => ORDINAL_SUITES,
                Content::Documents => ORDINAL_DOCUMENTS,
                Content::Headings => ORDINAL_HEADINGS,
                Content::Blocks => ORDINAL_BLOCKS,
                Content::Rows => ORDINAL_ROWS,
                Content::Nodes => ORDINAL_NODES,
                Content::Statements => ORDINAL_STATEMENTS,
                Content::Relations => ORDINAL_RELATIONS,
                Content::Lineage => ORDINAL_LINEAGE,
                Content::Omissions => ORDINAL_OMISSIONS,
                Content::Neighbourhood => ORDINAL_NEIGHBOURHOOD,
                Content::Families => ORDINAL_FAMILIES,
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

    // `Test_Every_Content_Should_Be_Matched_Exhaustively` above already exercises both
    // `All` and `Label`, but it is a load-bearing mirror named and checked by literal
    // string in `tests/contract/tests/completeness_universes/table.rs`, outside this
    // crate's territory, so it is left untouched rather than renamed to address either.
    #[test]
    fn Test_All_Should_List_Every_Variant_Exactly_Once()
    {
        use std::collections::BTreeSet;

        let distinct: BTreeSet<&'static str> = Content::All().iter().map(|content| return content.Label()).collect();

        assert_eq!(distinct.len(), Content::All().len(), "a variant is missing or repeated");
    }

    #[test]
    fn Test_Label_Should_Spell_Each_Variant_In_Lowercase()
    {
        assert_eq!(Content::Suites.Label(), "suites");
        assert_eq!(Content::Omissions.Label(), "omissions");
        assert_eq!(Content::Neighbourhood.Label(), "neighbourhood");
        assert_eq!(Content::Families.Label(), "families");
    }
}
