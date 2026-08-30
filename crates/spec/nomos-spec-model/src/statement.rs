//! One normative statement, as the specification store holds it.

// A normative statement's identity and its kind.
mod statement_id;
mod statement_kind;
mod normative_statement;

pub use statement_id::StatementId;
pub use statement_kind::StatementKind;
pub use normative_statement::NormativeStatement;

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

    #[test]
    fn Test_A_Fixed_Point_Should_Be_Recognized()
    {
        assert!(Statement_From_Text("Nomos shall do the thing.").Is_Text_Canonical());
        assert!(!Statement_From_Text("Nomos  shall\ndo it.").Is_Text_Canonical());
    }

    #[test]
    fn Test_Canonicalizing_Should_Reach_A_Fixed_Point()
    {
        let fixed = Statement_From_Text("Nomos  shall\ndo it.").Canonicalized();

        assert_eq!(fixed.canonical_text, "Nomos shall do it.");
        assert!(fixed.Is_Text_Canonical());
    }

    #[test]
    fn Test_A_Pinned_Statement_Hash_Should_Reproduce()
    {
        let statement = Statement_From_Text(
            "Agents shall return structured plans, changes, claims, tests, requested \
             verification, assumptions, and unresolved questions.",
        );

        assert_eq!(
            statement.Canonical_Hash().As_String_Slice(),
            "sha256:f712ecde70e8f375d216a5636e3aff78c07cd2b8d235e9db5db64eeb0cdd1288"
        );
    }

    fn Statement_From_Text(text: &str) -> NormativeStatement
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
