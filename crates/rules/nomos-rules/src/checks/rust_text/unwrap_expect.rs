//! `unwrap-expect-discipline`: `unwrap()`, and an `expect(...)` that names no invariant.
//!
//! Split out of [`super`], which states the family's shared reasoning. This is the one rule
//! of the eight that reads neither a comment block nor a detector pair, so it depends on
//! neither of the two files that hold them.

use crate::checks::{code_prefix::Code_Prefix, Is_Test_Or_Example_Source};
use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::{Finding_For_Line, Line_Number, UNWRAP_EXPECT_DISCIPLINE};

/// Reports `unwrap()` and placeholder `expect(...)` outside test and example Rust sources.
#[must_use]
pub fn Check_Unwrap_Expect_Discipline(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE) && !Is_Test_Or_Example_Source(source)
        {
            findings.extend(Unwrap_Expect_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Unwrap_Expect_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for (index, line) in source.text.lines().enumerate()
    {
        let code = Code_Prefix(line);
        Push_Unwrap_Finding(source, &code, index, &mut findings);
        Push_Placeholder_Expect_Finding(source, &code, index, &mut findings);
    }

    return findings;
}

fn Push_Unwrap_Finding(source: &SourceFile, code: &str, index: usize, findings: &mut Vec<Finding>)
{
    if code.contains(".unwrap()")
    {
        let finding = Finding_For_Line(source, UNWRAP_EXPECT_DISCIPLINE, Line_Number(index), "uses `unwrap()` outside tests/examples");
        findings.push(finding);
    }
}

fn Push_Placeholder_Expect_Finding(source: &SourceFile, code: &str, index: usize, findings: &mut Vec<Finding>)
{
    if Placeholder_Expect(code)
    {
        let finding = Finding_For_Line(
            source,
            UNWRAP_EXPECT_DISCIPLINE,
            Line_Number(index),
            "uses `expect(...)` without naming an invariant",
        );
        findings.push(finding);
    }
}

fn Placeholder_Expect(code: &str) -> bool
{
    let Some(after_call) = code.split(".expect(").nth(1)
    else
    {
        return false;
    };

    let lower = after_call.to_ascii_lowercase();
    return lower.contains("\"should not happen\"")
        || lower.contains("\"impossible\"")
        || lower.contains("\"unreachable\"")
        || lower.contains("\"todo\"")
        || lower.contains("\"fixme\"");
}
