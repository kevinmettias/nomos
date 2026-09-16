//! Boolean data names should read as predicates where field types are visible.
//!
//! The current syntax payload does not expose local variables, parameters or function
//! return types. It does expose named struct fields with their spelled type, so this rule
//! judges boolean fields and leaves the other boolean-name sites to a richer provider.

use crate::SourceFile;
use crate::checks::finding_shape::Member_Finding;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Finding, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const BOOLEAN_PREDICATES: &str = "boolean-predicates";

const STRUCT: &str = "Struct";

/// Reports visible boolean fields whose names do not start with a predicate prefix.
#[must_use]
pub fn Check_Boolean_Predicates(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    return super::reading::Judged_Sources(sources, facts, Violations_In, Unread_As_This_Rule);
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for item in &payload.items
    {
        if item.kind == STRUCT
        {
            let field_violations = Field_Violations_In(path, item);
            findings.extend(field_violations);
        }
    }

    return findings;
}

fn Field_Violations_In(path: &str, item: &PayloadItem) -> Vec<Finding>
{
    let Some(fields) = Struct_Fields(&item.shape)
    else
    {
        return Vec::new();
    };

    return fields
        .iter()
        .filter(|(name, type_name)| return Is_Bool_Type(type_name) && !Has_Predicate_Prefix(name))
        .map(|(name, _type_name)| return Violation_Finding(path, item, name))
        .collect();
}

/// One boolean field that is not named as a predicate.
fn Violation_Finding(path: &str, item: &PayloadItem, name: &str) -> Finding
{
    return Member_Finding(
        BOOLEAN_PREDICATES,
        path,
        item,
        name,
        format!("boolean field `{name}` does not start with `is_`, `has_`, `can_`, or `should_`"),
    );
}

fn Is_Bool_Type(type_name: &str) -> bool
{
    return matches!(type_name.trim(), "bool" | "Bool" | "boolean" | "Boolean" | "System.Boolean");
}

fn Has_Predicate_Prefix(name: &str) -> bool
{
    return ["is_", "has_", "can_", "should_"]
        .iter()
        .any(|prefix| return name.starts_with(prefix));
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(BOOLEAN_PREDICATES);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's boolean predicates could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Boolean_Field_Without_Predicate_Prefix()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tFlag\t.\t+fields\\nready\\tbool\n");

        let findings = Violations_In(&payload, "src/flag.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "ready");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Boolean_Field_With_Predicate_Prefix()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tFlag\t.\t+fields\\nis_ready\\tbool\n");

        let findings = Violations_In(&payload, "src/flag.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Non_Boolean_Fields()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tFlag\t.\t+fields\\nready\\tString\n");

        let findings = Violations_In(&payload, "src/flag.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
