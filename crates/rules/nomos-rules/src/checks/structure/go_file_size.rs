//! Go's own, lower pair of the same two line-count triggers `structure.rs` states for Rust.

use super::{
    FILE_SIZE_HARD_LINES_KEY, FILE_SIZE_REVIEW_LINES_KEY, FIVE_HUNDRED_LINE_REVIEW_TRIGGER, Findings_For_Threshold,
    GO, GO_HARD_TRIGGER_LINES, GO_REVIEW_TRIGGER_LINES, LineThreshold, ONE_THOUSAND_LINE_HARD_TRIGGER, Resolve_Limit,
};
use crate::{GO_LANGUAGE, SourceFile};
use nomos_analysis::FactReader;
use nomos_contracts::Finding;

/// Reports Go files whose line count exceeds Go's own, lower review trigger.
#[must_use]
pub fn Check_Go_File_Size_Review_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_REVIEW_LINES_KEY, GO_REVIEW_TRIGGER_LINES);
    let because = format!("exceeds Go's {threshold}-line review trigger for splitting");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: FIVE_HUNDRED_LINE_REVIEW_TRIGGER, lines: threshold, because: &because },
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
}

/// Reports Go files whose line count exceeds Go's own, lower hard trigger.
#[must_use]
pub fn Check_Go_File_Size_Hard_Trigger(sources: &[SourceFile], facts: &mut dyn FactReader) -> Vec<Finding>
{
    let threshold = Resolve_Limit(facts, Some(GO), FILE_SIZE_HARD_LINES_KEY, GO_HARD_TRIGGER_LINES);
    let because = format!("exceeds Go's {threshold}-line trigger and needs decomposition or a documented locality justification");
    return Findings_For_Threshold(
        sources,
        LineThreshold { rule: ONE_THOUSAND_LINE_HARD_TRIGGER, lines: threshold, because: &because },
        |source| return source.Is_Written_In(GO_LANGUAGE),
    );
}

/// Narrow, file-local proofs for this file's own public functions, addressed by name.
///
/// `structure/tests.rs` is a separate physical file whose behavioural suite this does not repeat
/// or replace. `check-test-coverage`'s Rust front end keys a test's companion unit off the literal
/// file it is textually written in, so a test living in that separate file can never address a
/// function declared here, however it is named — this module gives both Go triggers above the
/// one-file address the check reads.
#[cfg(test)]
mod self_tests
{
    use super::*;
    use crate::checks::structure::tests::Fixture_Over_Threshold;
    use nomos_contracts::RuleId;

    #[test]
    fn Test_Check_Go_File_Size_Review_Trigger_Should_Report_A_Go_File_One_Line_Over()
    {
        let findings = Fixture_Over_Threshold(Check_Go_File_Size_Review_Trigger, "index.go", GO_REVIEW_TRIGGER_LINES);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above").rule.clone();
        assert!(reported == RuleId::New(FIVE_HUNDRED_LINE_REVIEW_TRIGGER), "a 501-line .go file must report Go's own review trigger, got {reported:?}");
    }

    #[test]
    fn Test_Check_Go_File_Size_Hard_Trigger_Should_Report_A_Go_File_One_Line_Over()
    {
        let findings = Fixture_Over_Threshold(Check_Go_File_Size_Hard_Trigger, "index.go", GO_HARD_TRIGGER_LINES);

        assert_eq!(findings.len(), 1, "{findings:?}");
        let reported = findings.first().expect("asserted len 1 above").rule.clone();
        assert!(reported == RuleId::New(ONE_THOUSAND_LINE_HARD_TRIGGER), "a 1001-line .go file must report Go's own hard trigger, got {reported:?}");
    }
}
