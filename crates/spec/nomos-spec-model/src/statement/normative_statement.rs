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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Canonical_Hash_Should_Hash_The_Canonical_Text()
    {
        let statement = Sample("Nomos shall do it.");

        assert_eq!(statement.Canonical_Hash(), ContentHash::Of("Nomos shall do it."));
    }

    #[test]
    fn Test_Is_Text_Canonical_Should_Detect_Whitespace_That_Is_Not_Normalized()
    {
        assert!(Sample("Nomos shall do it.").Is_Text_Canonical());
        assert!(!Sample("Nomos  shall\ndo it.").Is_Text_Canonical());
    }

    #[test]
    fn Test_Canonicalized_Should_Reach_A_Fixed_Point()
    {
        let fixed = Sample("Nomos  shall\ndo it.").Canonicalized();

        assert_eq!(fixed.canonical_text, "Nomos shall do it.");
        assert!(fixed.Is_Text_Canonical());
    }

    fn Sample(text: &str) -> NormativeStatement
    {
        return NormativeStatement {
            id: StatementId::Parse("AGT-001").expect("valid"),
            kind: StatementKind::Requirement,
            canonical_text: text.to_owned(),
            source_document: "x.md".to_owned(),
            heading_path: Vec::new(),
        };
    }
}
