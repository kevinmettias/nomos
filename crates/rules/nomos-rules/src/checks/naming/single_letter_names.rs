//! Single-letter names are forbidden where the syntax payload can see them.
//!
//! code-standards also governs locals, parameters, generic parameters and closure
//! arguments. The current syntax payload exposes declared item names and named struct
//! fields, so this rule judges that subset and leaves the rest to a richer syntax shape.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const SINGLE_LETTER_NAMES: &str = "single-letter-names";

const STRUCT: &str = "Struct";

/// Reports declared item names and named struct fields that are a single character.
#[must_use]
pub fn Check_Single_Letter_Names(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, &source.path)),
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for item in &payload.items
    {
        if Is_Single_Letter(item.Own_Name())
        {
            findings.push(Violation_Finding(path, item, item.Own_Name()));
        }

        if item.kind == STRUCT
        {
            findings.extend(Field_Violations_In(path, item));
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
        .filter(|(name, _type_name)| return Is_Single_Letter(name))
        .map(|(name, _type_name)| return Violation_Finding(path, item, name))
        .collect();
}

fn Is_Single_Letter(name: &str) -> bool
{
    return name.chars().count() == 1;
}

fn Violation_Finding(path: &str, item: &PayloadItem, name: &str) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}::{name}", item.qualified_name);

    return Finding {
        rule: RuleId::New(SINGLE_LETTER_NAMES),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{name}` is a single-letter name outside the local-variable exception this payload can judge"),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(SINGLE_LETTER_NAMES);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's single-letter names could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Single_Letter_Item_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tX\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "X");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Single_Letter_Struct_Field()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tPoint\t.\t+fields\\nx\\tf64\n");

        let findings = Violations_In(&payload, "src/point.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "x");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Multi_Letter_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tFunction\tPrivate\tRun\t.\t+fn/0\n\
             item\t1\tStruct\tPublic\tPoint\t.\t+fields\\nx_coord\\tf64\n",
        );

        let findings = Violations_In(&payload, "src/point.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
