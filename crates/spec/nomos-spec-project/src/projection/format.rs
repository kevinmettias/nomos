//! The forms a profile can render into.

use serde::Serialize;
use serde::Deserialize;
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Format
{
    Markdown,
    Yaml,
    Json,
    Html,
    Mermaid,
    Contextpack,
}

impl Format
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Markdown => "markdown",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Html => "html",
            Self::Mermaid => "mermaid",
            Self::Contextpack => "contextpack",
        };
    }

    #[must_use]
    pub const fn Extension(self) -> &'static str
    {
        return match self
        {
            Self::Markdown => "md",
            Self::Yaml => "yaml",
            Self::Json | Self::Contextpack => "json",
            Self::Html => "html",
            Self::Mermaid => "mmd",
        };
    }

    /// Every form a profile can render into.
    ///
    /// Mirrored by `Test_Every_Format_Should_Be_Matched_Exhaustively`, an exhaustive match
    /// over every variant with no wildcard arm, in this file. It fails to compile, not
    /// merely to pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Markdown,
            Self::Yaml,
            Self::Json,
            Self::Html,
            Self::Mermaid,
            Self::Contextpack,
        ];
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    /// Where each variant sits in [`Format::All`], which is what the loop below asserts the
    /// match's ordinals against.
    const POSITION_OF_MARKDOWN: usize = 0;
    const POSITION_OF_YAML: usize = 1;
    const POSITION_OF_JSON: usize = 2;
    const POSITION_OF_HTML: usize = 3;
    const POSITION_OF_MERMAID: usize = 4;
    const POSITION_OF_CONTEXTPACK: usize = 5;

    /// `Format::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Format` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Format_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal_Of_Format(format: Format) -> usize
        {
            return match format
            {
                Format::Markdown => POSITION_OF_MARKDOWN,
                Format::Yaml => POSITION_OF_YAML,
                Format::Json => POSITION_OF_JSON,
                Format::Html => POSITION_OF_HTML,
                Format::Mermaid => POSITION_OF_MERMAID,
                Format::Contextpack => POSITION_OF_CONTEXTPACK,
            };
        }

        for (index, format) in Format::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal_Of_Format(*format),
                index,
                "{} is not matched at the position Format::All() puts it, so the exhaustive \
                 match and the universe have drifted apart",
                format.Label()
            );
        }
    }

    // `Test_Every_Format_Should_Be_Matched_Exhaustively` above already exercises both
    // `All` and `Label`, but it is a load-bearing mirror named and checked by literal
    // string in `tests/contract/tests/completeness_universes/table.rs`, outside this
    // crate's territory, so it is left untouched rather than renamed to address either.
    #[test]
    fn Test_All_Should_List_Every_Variant_Exactly_Once()
    {
        use std::collections::BTreeSet;

        let distinct: BTreeSet<&'static str> = Format::All().iter().map(|format| return format.Label()).collect();

        assert_eq!(distinct.len(), Format::All().len(), "a variant is missing or repeated");
    }

    #[test]
    fn Test_Label_Should_Spell_Each_Variant_In_Lowercase()
    {
        assert_eq!(Format::Markdown.Label(), "markdown");
        assert_eq!(Format::Contextpack.Label(), "contextpack");
    }

    #[test]
    fn Test_Extension_Should_Match_Each_Formats_File_Suffix()
    {
        assert_eq!(Format::Markdown.Extension(), "md");
        assert_eq!(Format::Json.Extension(), "json");
        assert_eq!(Format::Contextpack.Extension(), "json");
        assert_eq!(Format::Mermaid.Extension(), "mmd");
    }
}
