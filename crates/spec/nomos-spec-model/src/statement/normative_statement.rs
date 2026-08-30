use super::StatementId;
use super::kind::Kind as StatementKind;
use crate::{ContentHash, Is_Normalized, Normalize_Whitespace};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormativeStatement
{
    pub id: StatementId,
    pub kind: StatementKind,
    pub canonical_text: String,
    pub source_document: String,
    pub heading_path: Vec<String>,
}

impl NormativeStatement
{
    #[must_use]
    pub fn Canonical_Hash(&self) -> ContentHash
    {
        return ContentHash::Of(&self.canonical_text);
    }

    /// Whether `canonical_text` is already what the normalizer would produce.
    ///
    /// Required, not incidental: `Canonical_Hash` hashes the text verbatim, so a text
    /// that is not a fixed point would hash differently from the same content written
    /// with different spacing, and two spellings of one statement would be two
    /// statements.
    #[must_use]
    pub fn Is_Text_Canonical(&self) -> bool
    {
        return Is_Normalized(&self.canonical_text);
    }

    #[must_use]
    pub fn Canonicalized(mut self) -> Self
    {
        self.canonical_text = Normalize_Whitespace(&self.canonical_text);
        return self;
    }
}
