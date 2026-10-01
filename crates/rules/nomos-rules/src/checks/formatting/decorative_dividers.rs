//! `no-decorative-section-dividers`: a standalone comment that is only punctuation and a
//! short label.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::SourceFile;
use nomos_contracts::Finding;

use super::{Comment_Text_Of, Finding_For_Line, For_Each_Line_Number, NO_DECORATIVE_SECTION_DIVIDERS};

/// Reports standalone decorative comment dividers such as `// ===== Setup =====`.
#[must_use]
pub fn Check_No_Decorative_Section_Dividers(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Divider_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Divider_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    For_Each_Line_Number(&source.text, |line_number, line| {
        if Is_Decorative_Divider(line)
        {
            let finding = Finding_For_Line(source, NO_DECORATIVE_SECTION_DIVIDERS, line_number, "is a decorative section divider");
            findings.push(finding);
        }
    });

    return findings;
}

fn Is_Decorative_Divider(line: &str) -> bool
{
    let Some(comment) = Comment_Text_Of(line)
    else
    {
        return false;
    };

    const MIN_DIVIDER_PUNCTUATION: usize = 6;

    let content = comment.trim().trim_end_matches(['\r', '\n']).trim();
    if Divider_Punctuation_Count(content) < MIN_DIVIDER_PUNCTUATION
    {
        return false;
    }

    let label = content.trim_matches(Is_Divider_Punctuation).trim();
    return label.is_empty() || Is_Short_Label(label);
}

fn Divider_Punctuation_Count(content: &str) -> usize
{
    return content
        .chars()
        .filter(|character| return Is_Divider_Punctuation(*character))
        .count();
}

fn Is_Divider_Punctuation(character: char) -> bool
{
    return matches!(character, '=' | '-' | '_' | '*' | '/');
}

fn Is_Short_Label(label: &str) -> bool
{
    const MAX_SHORT_LABEL_WORDS: usize = 4;

    let words = label.split_whitespace().count();
    return words > 0 && words <= MAX_SHORT_LABEL_WORDS && label.chars().all(|character| {
        return character.is_ascii_alphanumeric() || character.is_ascii_whitespace() || character == '_' || character == '-';
    });
}
