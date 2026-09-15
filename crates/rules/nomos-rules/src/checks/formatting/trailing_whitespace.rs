//! `no-trailing-whitespace`: a line whose last character is a space or a tab.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::SourceFile;
use nomos_contracts::Finding;

use super::{Finding_For_Line, For_Each_Line_Number, NO_TRAILING_WHITESPACE};

/// Reports every line in `sources` whose content ends in a space or tab.
#[must_use]
pub fn Check_No_Trailing_Whitespace(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    For_Each_Line_Number(&source.text, |line_number, line| {
        if Has_Trailing_Whitespace(line)
        {
            let finding = Finding_For_Line(source, NO_TRAILING_WHITESPACE, line_number, "ends with trailing whitespace");
            findings.push(finding);
        }
    });

    return findings;
}

fn Has_Trailing_Whitespace(line: &str) -> bool
{
    let content = line.trim_end_matches(['\r', '\n']);

    return content.ends_with(' ') || content.ends_with('\t');
}
