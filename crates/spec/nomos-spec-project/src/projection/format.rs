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

    /// `Format::All()`'s own mirror, named in the doc comment above it.
    ///
    /// The match has no wildcard arm. A variant added to `Format` without a matching arm
    /// added here fails this file to *compile*, not merely to pass — the property D-134
    /// asks a closed enum's mirror to have.
    #[test]
    fn Test_Every_Format_Should_Be_Matched_Exhaustively()
    {
        fn Ordinal(format: Format) -> usize
        {
            return match format
            {
                Format::Markdown => 0,
                Format::Yaml => 1,
                Format::Json => 2,
                Format::Html => 3,
                Format::Mermaid => 4,
                Format::Contextpack => 5,
            };
        }

        for (index, format) in Format::All().iter().enumerate()
        {
            assert_eq!(
                Ordinal(*format),
                index,
                "{} is not matched at the position Format::All() puts it, so the exhaustive \
                 match and the universe have drifted apart",
                format.Label()
            );
        }
    }
}
