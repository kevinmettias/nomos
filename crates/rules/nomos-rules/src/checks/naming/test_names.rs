//! Test functions name both the subject and the behavior they assert.
//!
//! This is the mechanically provable half of code-standards'
//! `test-functions-use-test-subject-should-behavior`: once a function is visibly a test by
//! this workspace's `Test_` prefix, its own name should carry `_Should_` or
//! `_Should_Not_` so the failing test report names the expectation that broke.

use crate::SourceFile;
use crate::checks::finding_shape::{Finding_Shape, Qualified_Name_Finding};
use nomos_analysis::FactReader;
use nomos_cap_syntax::{PayloadItem, SyntaxPayload, FUNCTION};
use nomos_contracts::{Finding, GateCategory, RuleId};

/// This rule's own identifier, matching the code-standards rule id.
pub const TEST_NAME_DESCRIBES_BEHAVIOR: &str = "test-functions-use-test-subject-should-behavior";

const TEST_PREFIX: &str = "Test_";
const POSITIVE_EXPECTATION: &str = "_Should_";
const NEGATIVE_EXPECTATION: &str = "_Should_Not_";

/// Judges every `Test_*` function in `sources` against the test-name behavior pattern.
#[must_use]
pub fn Check_Test_Names_Describe_Behavior(
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
        if !Is_Test_Function(item)
        {
            continue;
        }

        if !Is_Stating_An_Expectation(item.Own_Name())
        {
            let finding = Violation_Finding(path, item);
            findings.push(finding);
        }
    }

    return findings;
}

fn Is_Test_Function(item: &PayloadItem) -> bool
{
    return item.kind == FUNCTION && !item.Declares_No_Visibility() && item.Own_Name().starts_with(TEST_PREFIX);
}

fn Is_Stating_An_Expectation(name: &str) -> bool
{
    return name.contains(POSITIVE_EXPECTATION) || name.contains(NEGATIVE_EXPECTATION);
}

/// One test function whose name states no expectation.
fn Violation_Finding(path: &str, item: &PayloadItem) -> Finding
{
    return Qualified_Name_Finding(
        Finding_Shape {
            rule: TEST_NAME_DESCRIBES_BEHAVIOR,
            path,
            summary: format!(
                "`{}` is a test function whose name does not state a `_Should_` or \
                 `_Should_Not_` expectation",
                item.Own_Name()
            ),
        },
        item,
        GateCategory::Blocking,
    );
}

fn Unread_As_This_Rule(mut finding: Finding) -> Finding
{
    finding.rule = RuleId::New(TEST_NAME_DESCRIBES_BEHAVIOR);
    finding.summary = finding
        .summary
        .replace("this file's naming could not be judged", "this file's test names could not be judged");
    return finding;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::{Applicability, EvidenceClass, SubjectId};

    #[test]
    fn Test_Violations_In_Should_Report_A_Test_Name_With_No_Should_Phrase()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tTest_Insert_Works\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(TEST_NAME_DESCRIBES_BEHAVIOR));
        assert_eq!(found.subject_name, "Test_Insert_Works");
        assert_eq!(found.gate, GateCategory::Blocking);
    }

    #[test]
    fn Test_Violations_In_Should_Accept_Positive_And_Negative_Should_Names()
    {
        for name in ["Test_Arena_Should_Recycle_Freed_Handles", "Test_Arena_Should_Not_Recycle_Live_Handles"]
        {
            let payload = Payload_From_Text(&format!("unexpanded\t0\nitem\t0\tFunction\tPrivate\t{name}\t.\t+fn/0\n"));

            let findings = Violations_In(&payload, "src/lib.rs");

            assert!(findings.is_empty(), "{name}: {findings:?}");
        }
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Non_Test_Functions()
    {
        let payload = Payload_From_Text("unexpanded\t0\nitem\t0\tFunction\tPrivate\tHelper_Works\t.\t+fn/0\n");

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Violations_In_Should_Ignore_Trait_Method_Signatures()
    {
        let payload = Payload_From_Text(
            "unexpanded\t0\n\
             item\t0\tTrait\tPublic\tHarness\t.\t.\n\
             item\t1\tFunction\tNotApplicable\tHarness::Test_Insert_Works\t.\t+fn/0\n",
        );

        let findings = Violations_In(&payload, "src/lib.rs");

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Unread_As_This_Rule_Should_Relabel_The_Finding()
    {
        let finding = Finding {
            rule: RuleId::New(super::super::NAMING_CONVENTION),
            subject: SubjectId::From_Digest(nomos_model::Content_Digest(b"src/lib.rs")),
            subject_name: "src/lib.rs".to_owned(),
            applicability: Applicability::DependencyUnavailable,
            evidence: EvidenceClass::Derived,
            gate: GateCategory::Advisory,
            summary: "this file's naming could not be judged: no fact".to_owned(),
            locations: vec!["src/lib.rs".to_owned()],
        };

        let relabeled = Unread_As_This_Rule(finding);

        assert_eq!(relabeled.rule, RuleId::New(TEST_NAME_DESCRIBES_BEHAVIOR));
        assert!(relabeled.summary.contains("test names"), "{}", relabeled.summary);
    }

    fn Payload_From_Text(text: &str) -> SyntaxPayload
    {
        return nomos_cap_syntax::Parse_Payload(text.as_bytes())
            .expect("this fixture payload is well formed");
    }
}
