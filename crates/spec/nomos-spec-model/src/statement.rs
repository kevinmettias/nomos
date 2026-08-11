use crate::normalize::{ContentHash, Is_Normalized, Normalize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatementKind
{
    Requirement,
    Story,
    Acceptance,
    Criterion,
    Concept,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatementId(String);

impl StatementId
{
    /// Accepts `PREFIX-NNN`, where the prefix is upper-case ASCII with optional inner
    /// hyphens and the suffix is at least three digits.
    #[must_use]
    pub fn Parse(text: &str) -> Option<Self>
    {
        let (prefix, number) = text.rsplit_once('-')?;

        if prefix.is_empty() || !Is_A_Suffix(number)
        {
            return None;
        }

        let prefix_ok = prefix
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()));

        return prefix_ok.then(|| Self(text.to_owned()));
    }

    #[must_use]
    pub fn Prefix(&self) -> &str
    {
        return self.0.rsplit_once('-').map_or("", |(prefix, _)| prefix);
    }

    #[must_use]
    pub fn As_Str(&self) -> &str
    {
        return &self.0;
    }
}

/// Whether a trailing segment is the number a statement identifier ends in.
///
/// Three digits at least, because two would let a section number read as a statement.
fn Is_A_Suffix(number: &str) -> bool
{
    return number.len() >= 3 && All_Digits(number);
}

/// A non-empty run of ASCII digits and nothing else.
fn All_Digits(text: &str) -> bool
{
    return !text.is_empty() && text.bytes().all(|byte| return byte.is_ascii_digit());
}

impl core::fmt::Display for StatementId
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return formatter.pad(&self.0);
    }
}

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
