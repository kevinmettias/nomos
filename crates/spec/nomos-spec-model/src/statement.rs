//! One normative statement, as the specification store holds it.

// A normative statement's identity and its kind.
mod id;
mod kind;

pub use id::StatementId;
pub use kind::StatementKind;

use crate::normalize::{ContentHash, Is_Normalized, Normalize};

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
    pub fn Text_Is_Canonical(&self) -> bool
    {
        return Is_Normalized(&self.canonical_text);
    }

    #[must_use]
    pub fn Canonicalized(mut self) -> Self
    {
        self.canonical_text = Normalize(&self.canonical_text);
        return self;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Real_Identifiers_Should_Parse()
    {
        for text in ["AGT-001", "PKG-014", "ARC-DOC-001", "NSV-PRESERVE-001", "D-129"]
        {
            assert!(StatementId::Parse(text).is_some(), "{text} should parse");
        }
    }

    #[test]
    fn Test_Malformed_Identifiers_Should_Be_Refused()
    {
        for text in ["agt-001", "AGT-1", "AGT_001", "AGT-", "-001", "AGT001", ""]
        {
            assert!(StatementId::Parse(text).is_none(), "{text} should not parse");
        }
    }

    #[test]
    fn Test_Prefix_Should_Be_Everything_Before_The_Number()
    {
        assert_eq!(
            StatementId::Parse("NSV-PRESERVE-001").map(|id| id.Prefix().to_owned()),
            Some("NSV-PRESERVE".to_owned())
        );
    }

    fn Statement(text: &str) -> NormativeStatement
    {
        return NormativeStatement {
            id: StatementId::Parse("AGT-001").expect("valid"),
            kind: StatementKind::Requirement,
            canonical_text: text.to_owned(),
            source_document: "x.md".to_owned(),
            heading_path: Vec::new(),
        };
    }

    #[test]
    fn Test_A_Fixed_Point_Should_Be_Recognized()
    {
        assert!(Statement("Nomos shall do the thing.").Text_Is_Canonical());
        assert!(!Statement("Nomos  shall\ndo it.").Text_Is_Canonical());
    }

    #[test]
    fn Test_Canonicalizing_Should_Reach_A_Fixed_Point()
    {
        let fixed = Statement("Nomos  shall\ndo it.").Canonicalized();

        assert_eq!(fixed.canonical_text, "Nomos shall do it.");
        assert!(fixed.Text_Is_Canonical());
    }

    #[test]
    fn Test_A_Pinned_Statement_Hash_Should_Reproduce()
    {
        let statement = Statement(
            "Agents shall return structured plans, changes, claims, tests, requested \
             verification, assumptions, and unresolved questions.",
        );

        assert_eq!(
            statement.Canonical_Hash().As_Str(),
            "sha256:f712ecde70e8f375d216a5636e3aff78c07cd2b8d235e9db5db64eeb0cdd1288"
        );
    }
}
