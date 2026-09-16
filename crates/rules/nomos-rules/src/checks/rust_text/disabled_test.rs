//! `a-disabled-test-states-why`: a bare `#[ignore]` with neither a reason nor a comment.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::comment_block::{Is_A_Non_Empty_Comment, Has_A_Previous_Comment_Block};
use super::{
    ConstructDetector, Detector, Is_Own_Implementation_File, JustificationDetector, Message, Rule,
    Unjustified_Construct_Findings_In, A_DISABLED_TEST_STATES_WHY,
};

/// Reports a bare `#[ignore]` on a Rust test with neither an inline `= "reason"` value nor
/// an adjacent comment explaining why the test does not run.
#[must_use]
pub fn Check_A_Disabled_Test_States_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Own_Implementation_File(source)
        {
            findings.extend(Disabled_Test_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Disabled_Test_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    return Unjustified_Construct_Findings_In(
        source,
        Rule(A_DISABLED_TEST_STATES_WHY),
        Message("disables a test with a bare #[ignore] and no reason"),
        Detector { has_construct: ConstructDetector(Has_Bare_Ignore_Attribute), has_local_justification: JustificationDetector(Has_Local_Ignore_Justification) },
    );
}
/// A bare `#[ignore]` (or `#[ignore, ...]`) with no `= "reason"` value — the shape
/// `a-disabled-test-states-why` names as the one that needs a local comment instead.
/// `#[ignore = "..."]` already carries its own reason in the attribute itself and is never
/// flagged.
fn Has_Bare_Ignore_Attribute(code: &str) -> bool
{
    let Some(start) = code.find("#[ignore") else { return false };
    let after = code[start.saturating_add("#[ignore".len())..].trim_start();
    return after.starts_with(']') || after.starts_with(',');
}

fn Has_Local_Ignore_Justification(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Is_A_Non_Empty_Comment(line))
    {
        return true;
    }

    return Has_A_Previous_Comment_Block(lines, index, Is_A_Non_Empty_Comment);
}
