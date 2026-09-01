//! Go functions keep the workspace's operation-name convention on both sides of
//! visibility.
//!
//! code-standards' `exported-functions-use-upper-snake-case` and
//! `unexported-functions-lowercase-only-the-first-letter` are Go-specific and split the
//! same convention by visibility, because Go decides visibility by a name's first letter
//! rather than a keyword: an unexported name lowercases only that first letter and keeps
//! the rest of the shape intact. [`Is_Upper_Snake_Case`] is this crate's own, pre-existing
//! reading of that shape — every character uppercase, digit, or `_` — so the unexported
//! check mirrors it exactly with only the first character's case flipped
//! (`RUN_WITH_BACKEND` becomes `rUN_WITH_BACKEND`, not `run_with_backend` or
//! `rUN_with_BACKEND`). The syntax payload carries function names and Go visibility, so
//! both checks can judge that exact surface for `.go` sources.

use crate::SourceFile;
use nomos_analysis::FactReader;
use nomos_cap_syntax::{FUNCTION, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE: &str = "exported-functions-use-upper-snake-case";
/// The code-standards identifier for the unexported half of the same convention.
pub const UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER: &str = "unexported-functions-lowercase-only-the-first-letter";

/// Judges exported Go function and method names against `Upper_Snake_Case`.
#[must_use]
pub fn Check_Exported_Go_Functions_Use_Upper_Snake_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Go_File(&source.path)
        {
            continue;
        }

        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Violations_In(&payload, &source.path)),
            Err(finding) => findings.push(Unread_As_This_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// Judges unexported Go function and method names against the same convention with only
/// its first letter lowercased.
#[must_use]
pub fn Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if !Is_Go_File(&source.path)
        {
            continue;
        }

        match super::reading::Payload_Of(source, facts)
        {
            Ok(payload) => findings.extend(Unexported_Violations_In(&payload, &source.path)),
            Err(finding) => findings.push(Unread_As_Unexported_Rule(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Exported_Go_Function(item))
        .filter(|item| return !Is_Upper_Snake_Case(item.Own_Name()))
        .map(|item| return Violation_Finding(path, item))
        .collect();
}

fn Unexported_Violations_In(payload: &SyntaxPayload, path: &str) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Unexported_Go_Function(item))
        .filter(|item| return !Is_Lowercase_First_Letter_Convention(item.Own_Name()))
        .map(|item| return Unexported_Violation_Finding(path, item))
        .collect();
}

fn Is_Go_File(path: &str) -> bool
{
    return std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| return extension.eq_ignore_ascii_case("go"));
}

fn Is_Exported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && item.Is_Public();
}

fn Is_Unexported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && !item.Is_Public();
}

fn Is_Upper_Snake_Case(name: &str) -> bool
{
    if name.is_empty() || name.starts_with('_') || name.ends_with('_') || name.contains("__")
    {
        return false;
    }

    return name
        .chars()
        .all(|character| return character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_');
}

/// An unexported name conforms when only its first letter has been lowercased from what
/// would otherwise be `Upper_Snake_Case`: the first character is a lowercase letter, and
/// everything after it still obeys `Upper_Snake_Case`'s own shape.
fn Is_Lowercase_First_Letter_Convention(name: &str) -> bool
{
    let Some(first) = name.chars().next()
    else
    {
        return false;
    };

    if !first.is_ascii_lowercase()
    {
        return false;
    }

    let rest = &name[first.len_utf8()..];
    if rest.is_empty()
    {
        return true;
    }

    if rest.ends_with('_') || rest.contains("__")
    {
        return false;
    }

    return rest
        .chars()
        .all(|character| return character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_');
}

fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let name = item.Own_Name();
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("exported Go function `{name}` is not Upper_Snake_Case"),
        locations: vec![path.to_owned()],
    };
}

fn Unexported_Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    use nomos_model::Content_Digest;

    let name = item.Own_Name();
    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("unexported Go function `{name}` lowercases more than its first letter from Upper_Snake_Case"),
        locations: vec![path.to_owned()],
    };
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's exported function names could not be judged");
    return finding;
}

fn Unread_As_Unexported_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this Go file's unexported function names could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_Exported_Go_Functions_That_Are_Not_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRunWithBackend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "RunWithBackend");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Exported_Go_Functions_In_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRUN_WITH_BACKEND\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Unexported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trunWithBackend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Accept_Only_The_First_Letter_Lowercased()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trUN_WITH_BACKEND\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_Fully_Lowercased_Name()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trowbreaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER)
        );
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_Lowercase_Letter_After_The_First()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trUN_With_BACKEND\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go");

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Ignore_Exported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go");

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
