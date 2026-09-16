//! Go functions keep the workspace's operation-name convention on both sides of
//! visibility.
//!
//! code-standards' `exported-functions-use-upper-snake-case` and
//! `unexported-functions-lowercase-only-the-first-letter` are Go-specific and split the
//! same convention by visibility, because Go decides visibility by a name's first letter
//! rather than a keyword: an unexported name lowercases only that first letter and keeps
//! the rest of the shape intact (`Run_With_Backend` becomes `run_With_Backend`).
//!
//! # `OD-RULES-011` fixed a real drift here
//!
//! This crate's own prior reading of "`Upper_Snake_Case`" for the exported half was every
//! character uppercase, digit, or `_` — `screaming-snake` in code-standards' own closed
//! vocabulary, not `upper-snake` (`Compute_Total`, `^[A-Z][A-Za-z0-9]*(_[A-Z0-9]
//! [A-Za-z0-9]*)*$`), which is what the rule id actually names and what `Check_Naming_
//! Convention`'s own `Pascal_Snake_Case` already implements correctly. The unexported half
//! mirrored that same wrong shape rather than [`nomos_cap_naming_policy::Case::MixedSnake`]
//! (`compute_Total`), the style code-standards' own vocabulary names for exactly "the
//! first word lower, the rest cased normally." Both are corrected to their real defaults
//! here, the same way every casing rule in this crate now resolves its case from a
//! repository's own `nomos.cap.naming.policy` rather than a hand-rolled predicate.

use crate::checks::naming::Resolve_Case;
use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_cap_naming_policy::Case;
use nomos_cap_syntax::{FUNCTION, PayloadItem, SyntaxPayload};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId, SubjectId};

/// This rule's own identifier, matching the code-standards rule id.
pub const EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE: &str = "exported-functions-use-upper-snake-case";
/// The code-standards identifier for the unexported half of the same convention.
pub const UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER: &str = "unexported-functions-lowercase-only-the-first-letter";

/// Judges exported Go function and method names against `Upper_Snake_Case` — a
/// repository's own `nomos.cap.naming.policy` when it declares `function.exported` for
/// `go`, this rule's own corrected default otherwise.
#[must_use]
pub fn Check_Exported_Go_Functions_Use_Upper_Snake_Case(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let case = Resolve_Case(facts, Some("go"), "function.exported", Case::UpperSnake);
    return Judged_Go_Function_Sources(
        sources,
        facts,
        Judgment { case, judge: ViolationConstructor(Violations_In), unread: UnreadWrapper(Unread_As_This_Rule) },
    );
}

/// Judges unexported Go function and method names against the same convention with only
/// its first word lowercased — a repository's own `nomos.cap.naming.policy` when it
/// declares `function.unexported` for `go`, this rule's own corrected default otherwise.
#[must_use]
pub fn Check_Unexported_Go_Functions_Lowercase_Only_The_First_Letter(
    sources: &[SourceFile],
    facts: &mut dyn FactReader,
) -> Vec<Finding>
{
    let case = Resolve_Case(facts, Some("go"), "function.unexported", Case::MixedSnake);
    return Judged_Go_Function_Sources(
        sources,
        facts,
        Judgment { case, judge: ViolationConstructor(Unexported_Violations_In), unread: UnreadWrapper(Unread_As_Unexported_Rule) },
    );
}

/// The shape both checks above share: skip a non-Go source, read each remaining source's
/// own syntax fact, and fold either a real reading failure or `judge`'s own findings
fn Violations_In(payload: &SyntaxPayload, path: &str, case: Case) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Exported_Go_Function(item))
        .filter(|item| return !case.Conforms(item.Own_Name()))
        .map(|item| return Violation_Finding(path, item))
        .collect();
}

fn Is_Exported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && item.Is_Public();
}

fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let name = item.Own_Name();
    let summary = format!("exported Go function `{name}` is not Upper_Snake_Case");
    return Function_Naming_Finding(EXPORTED_FUNCTIONS_USE_UPPER_SNAKE_CASE, path, item, summary);
}

fn Unexported_Violations_In(payload: &SyntaxPayload, path: &str, case: Case) -> Vec<Finding>
{
    return payload
        .items
        .iter()
        .filter(|item| return Is_Unexported_Go_Function(item))
        .filter(|item| return !case.Conforms(item.Own_Name()))
        .map(|item| return Unexported_Violation_Finding(path, item))
        .collect();
}

fn Is_Unexported_Go_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && !item.Is_Public();
}

fn Unexported_Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    let name = item.Own_Name();
    let summary = format!("unexported Go function `{name}` lowercases more than its first letter from Upper_Snake_Case");
    return Function_Naming_Finding(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER, path, item, summary);
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

/// The shape both checks above share: skip a non-Go source, read each remaining source's
/// own syntax fact, and fold either a real reading failure or `judge`'s own findings
/// (against the already-resolved `case`) into one sorted list.
/// Constructs the violations one payload's function names hold against an already-resolved
/// case, named so it reads as a collaborator with one documented operation rather than a
/// bare stored callable.
struct ViolationConstructor(fn(&SyntaxPayload, &str, Case) -> Vec<Finding>);

impl ViolationConstructor
{
    fn Violations(&self, payload: &SyntaxPayload, path: &str, case: Case) -> Vec<Finding>
    {
        return (self.0)(payload, path, case);
    }
}

/// Wraps a reading failure into this rule's own finding, named for the same reason as
/// [`ViolationConstructor`].
struct UnreadWrapper(fn(Finding) -> Finding);

impl UnreadWrapper
{
    fn Wrap(&self, finding: Finding) -> Finding
    {
        return (self.0)(finding);
    }
}

/// How to judge one payload's function names: the already-resolved case a name must match,
/// the violation constructor, and how to wrap a reading failure into its own finding.
/// Grouped so the two callers above and this function stay under the parameter-count
/// ceiling.
struct Judgment
{
    case: Case,
    judge: ViolationConstructor,
    unread: UnreadWrapper,
}

fn Judged_Go_Function_Sources(sources: &[SourceFile], facts: &mut dyn FactReader, judgment: Judgment) -> Vec<Finding>
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
                let violations = judgment.judge.Violations(&payload, &source.path, judgment.case);
                findings.extend(violations);
            }
            Err(finding) => findings.push(judgment.unread.Wrap(finding)),
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

/// The `Finding` shape both violation constructors above share: only the rule id and the
/// already-composed summary differ between an exported and an unexported Go function name.
fn Function_Naming_Finding(rule: &'static str, path: &str, item: &PayloadItem, summary: String) -> Finding
{
    use nomos_model::Content_Digest;

    let qualified = format!("{path}::{}", item.qualified_name);

    return Finding {
        rule: RuleId::New(rule),
        subject: SubjectId::From_Digest(Content_Digest(qualified.as_bytes())),
        subject_name: item.qualified_name.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary,
        locations: vec![path.to_owned()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Violations_In_Should_Report_Exported_Go_Functions_That_Are_Not_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\trun_With_Backend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Case::UpperSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings.first().expect("asserted len 1 above").subject_name, "run_With_Backend");
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Exported_Go_Functions_In_Upper_Snake()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRun_With_Backend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Case::UpperSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Unexported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trunWithBackend\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "main.go", Case::UpperSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    /// The doc's own worked example: only the first word lowercases.
    #[test]
    fn Test_Unexported_Violations_In_Should_Accept_Only_The_First_Word_Lowercased()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Case::MixedSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_Recased_Name_With_No_Word_Boundary()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\trowBreaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Case::MixedSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(
            findings.first().expect("asserted len 1 above").rule,
            RuleId::New(UNEXPORTED_FUNCTIONS_LOWERCASE_ONLY_THE_FIRST_LETTER)
        );
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Report_A_First_Word_That_Is_Not_Fully_Lowercased()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tRow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Case::MixedSnake);

        assert_eq!(findings.len(), 1, "{findings:?}");
    }

    #[test]
    fn Test_Unexported_Violations_In_Should_Ignore_Exported_Go_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPublic\tRow_Breaches\t.\t+fn/0\n");

        let findings = Unexported_Violations_In(&payload, "index.go", Case::MixedSnake);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
