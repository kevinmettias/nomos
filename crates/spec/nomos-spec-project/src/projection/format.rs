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
