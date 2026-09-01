//! Data-like names stay lower snake where the syntax fact exposes them.
//!
//! code-standards' `data-names-stay-lower-snake` also covers locals, parameters, enum
//! variant fields and package names. The current `nomos.cap.syntax.items` payload exposes
//! module declarations and named struct fields, so this rule judges that precise subset and
//! leaves the rest for a richer provider.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, Struct_Fields, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const DATA_NAMES_STAY_LOWER_SNAKE: &str = "data-names-stay-lower-snake";

const MODULE: &str = "Module";
const STRUCT: &str = "Struct";

/// Judges module declarations and named struct fields against lower snake case.
#[must_use]
pub fn Check_Data_Names_Stay_Lower_Snake(
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
        if item.kind == MODULE && !Is_Lower_Snake_Case(item.Own_Name())
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
        .filter(|(name, _type_name)| return !Is_Lower_Snake_Case(name))
        .map(|(name, _type_name)| return Violation_Finding(path, item, name))
        .collect();
}

fn Is_Lower_Snake_Case(name: &str) -> bool
{
    if name.is_empty() || name.starts_with('_') || name.ends_with('_') || name.contains("__")
    {
        return false;
    }

    return name
        .chars()
        .all(|character| return character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_');
}

fn Violation_Finding(path: &str, item: &PayloadItem, name: &str) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}::{name}", item.qualified_name);

    return Finding {
        rule: RuleId::New(DATA_NAMES_STAY_LOWER_SNAKE),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("`{name}` is a data name that is not lower snake case"),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(DATA_NAMES_STAY_LOWER_SNAKE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's data names could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_A_Module_Name_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tModule\tPrivate\tBadModule\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadModule");
    }

    #[test]
    fn Test_Violations_In_Should_Report_A_Struct_Field_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tStruct\tPublic\tConfig\t.\t+fields\\nBadField\\tString\\nworker_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "BadField");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Lower_Snake_Module_And_Field_Names()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tModule\tPrivate\tread_model\t.\t.\n\
             item\t1\tStruct\tPublic\tConfig\t.\t+fields\\nworker_count\\tusize\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Tuple_Structs()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tStruct\tPublic\tConfig\t.\t.\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
