//! File-size thresholds from code-standards' structure rules.
//!
//! The standards define two line-count triggers: files above roughly 500 lines are review
//! candidates for splitting, and files above roughly 1500 lines must carry an explicit
//! justification. Counting lines is text-local, so these rules take only [`SourceFile`]s.

use crate::SourceFile;
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards review-trigger rule id.
pub const FILE_SIZE_REVIEW_TRIGGER: &str = "500-lines";
/// The code-standards hard-trigger rule id.
pub const FILE_SIZE_JUSTIFICATION_TRIGGER: &str = "1500-lines";

const REVIEW_TRIGGER_LINES: usize = 500;
const JUSTIFICATION_TRIGGER_LINES: usize = 1500;

/// Reports files whose line count exceeds the review trigger.
#[must_use]
pub fn Check_File_Size_Review_Trigger(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For_Threshold(
        sources,
        FILE_SIZE_REVIEW_TRIGGER,
        REVIEW_TRIGGER_LINES,
        "exceeds the ~500 line review trigger for splitting",
    );
}

/// Reports files whose line count exceeds the hard justification trigger.
#[must_use]
pub fn Check_File_Size_Justification_Trigger(sources: &[SourceFile]) -> Vec<Finding>
{
    return Findings_For_Threshold(
        sources,
        FILE_SIZE_JUSTIFICATION_TRIGGER,
        JUSTIFICATION_TRIGGER_LINES,
        "exceeds the ~1500 line trigger and needs an explicit splitting justification",
    );
}

fn Findings_For_Threshold(sources: &[SourceFile], rule: &str, threshold: usize, because: &str) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        let line_count = Line_Count(source);
        if line_count > threshold
        {
            findings.push(Finding_For_Source(source, rule, line_count, because));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Line_Count(source: &SourceFile) -> usize
{
    return source.text.lines().count();
}

fn Finding_For_Source(source: &SourceFile, rule: &str, line_count: usize, because: &str) -> Finding
{
    return Finding {
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: source.path.clone(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} has {line_count} lines and {because}", source.path),
        locations: vec![source.path.clone()],
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_contracts::SubjectId;
    use nomos_model::Content_Digest;

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Report_A_File_Over_500_Lines()
    {
        let source = Source("src/large.rs", Lines(501));

        let findings = Check_File_Size_Review_Trigger(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_SIZE_REVIEW_TRIGGER));
        assert!(found.summary.contains("501 lines"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_File_Size_Review_Trigger_Should_Accept_A_File_At_500_Lines()
    {
        let source = Source("src/medium.rs", Lines(500));

        let findings = Check_File_Size_Review_Trigger(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn Test_Check_File_Size_Justification_Trigger_Should_Report_A_File_Over_1500_Lines()
    {
        let source = Source("src/huge.rs", Lines(1501));

        let findings = Check_File_Size_Justification_Trigger(&[source]);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let found = findings.first().expect("asserted len 1 above");
        assert_eq!(found.rule, RuleId::New(FILE_SIZE_JUSTIFICATION_TRIGGER));
        assert!(found.summary.contains("1501 lines"), "{}", found.summary);
    }

    #[test]
    fn Test_Check_File_Size_Justification_Trigger_Should_Accept_A_File_At_1500_Lines()
    {
        let source = Source("src/large.rs", Lines(1500));

        let findings = Check_File_Size_Justification_Trigger(&[source]);

        assert!(findings.is_empty(), "{findings:?}");
    }

    fn Lines(count: usize) -> String
    {
        return (0..count).map(|_| return "line").collect::<Vec<_>>().join("\n");
    }

    fn Source(path: &str, text: String) -> SourceFile
    {
        return SourceFile::New(path, SubjectId::From_Digest(Content_Digest(path.as_bytes())), text);
    }
}
