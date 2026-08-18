//! What a line inside a table is.

use serde::{Deserialize, Serialize};

/// What a line inside a table is.
///
/// A separator carries no authored content — it is the delimiter telling a reader where
/// the header stops. A header names the columns; it is authored text, but it is not a
/// datum. Typing all three rather than discarding any of them keeps the line count, the
/// non-separator count and the data count as three queries over one table, so no number
/// has to be bent to match another.
///
/// Restoration is what forces the header to be its own kind. Minting a node per data row
/// of the canonical domain model, over a table whose header cannot be told from its data,
/// mints a concept named after the column titles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowKind
{
    Header,
    Content,
    Separator,
}

impl RowKind
{
    #[must_use]
    pub const fn Label(self) -> &'static str
    {
        return match self
        {
            Self::Header => "header",
            Self::Content => "content",
            Self::Separator => "separator",
        };
    }

    #[must_use]
    pub fn Parse(label: &str) -> Option<Self>
    {
        return match label
        {
            "header" => Some(Self::Header),
            "content" => Some(Self::Content),
            "separator" => Some(Self::Separator),
            _ => None,
        };
    }

    /// Every kind, so a census cannot quietly omit one.
    ///
    /// Mirrored by `Test_Every_RowKind_Should_Be_Matched_Exhaustively`, an exhaustive
    /// match over every variant with no wildcard arm, in
    /// `crates/spec/nomos-spec-model/src/table.rs`. It fails to compile, not merely to
    /// pass, if a variant is added here without being added there.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[Self::Header, Self::Content, Self::Separator];
    }
}
