//! Go constants and top-level variables carry the workspace's data-name conventions.
//!
//! code-standards' `constants-split-by-export` gives a Go constant `SCREAMING_SNAKE_CASE`
//! when exported and `lower_snake_case` when not — the same visibility-carries-in-the-name
//! split [`super::function_names`] already judges for functions, applied to Go's own
//! constant casing because Go's own convention does not shout its constants.
//! `variables-use-lower-snake-case` is narrower here: the syntax payload exposes only
//! top-level Go `var` declarations as `Variable` items — locals and parameters never reach
//! it, and struct fields are already judged by [`super::data_names`] regardless of language
//! — so this rule judges that one visible slice against `lower_snake_case` without a split
//! by export status, the same way the standard itself does not split it.

use crate::checks::finding_shape::{Finding_Shape, Own_Name_Finding};
use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const CONSTANTS_SPLIT_BY_EXPORT: &str = "constants-split-by-export";
/// This rule's own identifier, matching the code-standards rule id.
pub const GO_VARIABLES_USE_LOWER_SNAKE_CASE: &str = "variables-use-lower-snake-case";

const CONSTANT: &str = "Constant";
const VARIABLE: &str = "Variable";

/// Judges Go constant names against `SCREAMING_SNAKE_CASE` when exported and
/// `lower_snake_case` when not.
#[must_use]
pub fn Check_Go_Constants_Split_By_Export(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return Judged_Go_Sources(sources, facts, Constant_Violations_In, Unread_As_Constant_Rule);
}

/// Judges top-level Go `var` declarations against `lower_snake_case`.
#[must_use]
pub fn Check_Go_Variables_Use_Lower_Snake_Case(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    return Judged_Go_Sources(sources, facts, Variable_Violations_In, Unread_As_Variable_Rule);
}

fn Constant_Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return item.kind == CONSTANT)
        .filter(|item| return !Has_Constant_Case(item))
        .map(|item| return Constant_Violation_Finding(path, item))
        .collect();
}

fn Has_Constant_Case(item: &PayloadItem) -> bool
{
    if item.Is_Public()
    {
        return Is_Screaming_Snake_Case(item.Own_Name());
    }

    return Is_Lower_Snake_Case(item.Own_Name());
}

fn Is_Screaming_Snake_Case(name: &str) -> bool
{
    if Has_Malformed_Snake_Boundary(name)
    {
        return false;
    }

    return name
        .chars()
        .all(|character| return character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_');
}

fn Constant_Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let name = item.Own_Name();
    let expected = if item.Is_Public() { "SCREAMING_SNAKE_CASE" } else { "lower_snake_case" };
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        address: Some(qualified.clone()),
        rule: RuleId::New(CONSTANTS_SPLIT_BY_EXPORT),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: name.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("Go constant `{name}` is not {expected}"),
        locations: vec![path.to_owned()],
    };
}

fn Variable_Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return item.kind == VARIABLE)
        .filter(|item| return !Is_Lower_Snake_Case(item.Own_Name()))
        .map(|item| return Variable_Violation_Finding(path, item))
        .collect();
}

/// One Go variable that is not lower_snake_case.
fn Variable_Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let name = item.Own_Name();

    return Own_Name_Finding(
        Finding_Shape { rule: GO_VARIABLES_USE_LOWER_SNAKE_CASE, path, summary: format!("Go variable `{name}` is not lower_snake_case") },
        item,
    );
}

fn Unread_As_Constant_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(CONSTANTS_SPLIT_BY_EXPORT);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's constant names could not be judged");
    return finding;
}

fn Unread_As_Variable_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(GO_VARIABLES_USE_LOWER_SNAKE_CASE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's variable names could not be judged");
    return finding;
}

/// The shape both checks above share: skip a non-Go source, read each remaining source's
/// own syntax fact, and fold either a real reading failure or `judge`'s own findings into
/// one sorted list.
fn Judged_Go_Sources(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
    judge: fn(&SyntaxPayload, &str) -> Vec<Finding>,
    unread: fn(Finding) -> Finding,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !source.Is_Written_In(GO_LANGUAGE)
        {
            continue;
        }

        match super::super::reading::Payload_Of(source, facts)
        {
            Ok(payload) =>
            {
                let violations = judge(&payload, &source.path);
                findings.extend(violations);
            }
            Err(finding) => findings.push(unread(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Is_Lower_Snake_Case(name: &str) -> bool
{
    if Has_Malformed_Snake_Boundary(name)
    {
        return false;
    }

    return name
        .chars()
        .all(|character| return character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_');
}

/// Whether `name` breaks snake-case shape before its casing is even judged: empty, a
/// leading or trailing underscore, or a doubled one.
fn Has_Malformed_Snake_Boundary(name: &str) -> bool
{
    return name.is_empty() || name.starts_with('_') || name.ends_with('_') || name.contains("__");
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Constant_Violations_In_Should_Report_An_Exported_Constant_That_Is_Not_Screaming_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tConstant\tPublic\tKindRule\t.\t.\n");

        let findings = Constant_Violations_In(&payload, "kinds.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "KindRule");
    }

    #[test]
    fn Test_Constant_Violations_In_Should_Report_An_Unexported_Constant_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tConstant\tPrivate\tTableRow\t.\t.\n");

        let findings = Constant_Violations_In(&payload, "kinds.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "TableRow");
    }

    #[test]
    fn Test_Constant_Violations_In_Should_Accept_Both_Halves_Of_The_Split()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tConstant\tPublic\tKIND_RULE\t.\t.\n\
             item\t1\tConstant\tPrivate\ttable_row\t.\t.\n",
        );

        let findings = Constant_Violations_In(&payload, "kinds.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Variable_Violations_In_Should_Report_A_Variable_That_Is_Not_Lower_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tVariable\tPrivate\tentityID\t.\t.\n");

        let findings = Variable_Violations_In(&payload, "state.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "entityID");
    }

    #[test]
    fn Test_Variable_Violations_In_Should_Accept_A_Lower_Snake_Variable_Regardless_Of_Export()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tVariable\tPublic\tentity_id\t.\t.\n");

        let findings = Variable_Violations_In(&payload, "state.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_Go_Constants_Split_By_Export_Should_Ignore_Non_Go_Sources()
    {
        use nomos_analysis::{MemoryFactStore, Reader};
        use nomos_capability::Registry;
        use nomos_model::Content_Digest;

        let mut source = SourceFile::New("kinds.rs", SubjectId::From_Digest(Content_Digest(b"kinds.rs")), "pub const kind_rule: &str = \"rule\";");
        source.language = crate::Recognized_Language_In_Tests("kinds.rs");
        let store = MemoryFactStore::New();
        let registry = Registry::New();
        let mut reader = Reader::On(&store, &registry, crate::checks::test_support::Test_Context());

        let findings = Check_Go_Constants_Split_By_Export(&[source], &mut reader);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes()).expect("this fixture payload is well formed");
    }
}
