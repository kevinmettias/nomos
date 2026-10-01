//! Go source-text rules from code-standards' suppression-justified family.
//!
//! The Go form of the same "a marker needs an adjacent or attached reason" shape
//! `rust_text.rs` already generalizes for Rust (`every-allow-carries-a-justification`,
//! `unsafe-justification`, `panics-are-justified-documented-and-validated`, `shared-
//! interior-mutability-says-why`): five constructs, three needing an adjacent comment
//! block, one needing a same-line trailing reason after a directive, and one needing
//! either a call argument or (when the call cannot carry one) an adjacent comment.
//! Deliberately not a capability: nothing about whether these markers need a reason is a
//! value a repository would configure -- it is always true, so the generalization is
//! shared code, not shared configuration, the same distinction `OD-RULES-011`'s own
//! naming/limits capabilities do not apply here. All five rules share `Lines_Of`,
//! `Comment_Text_Of`, `Has_Adjacent_Explanation` and `Finding_For_Line` below; two of them
//! whose subject is a bare comment directive have been split into [`exclusion`] and
//! [`markers`], each carrying the tests for its own rules. The discarded-error rule is split
//! into [`discarded_error`] for a different reason: which values are errors is a question about
//! types, so it reads them from a capability and keeps only the explanation's reading here.

mod discarded_error;
mod exclusion;
mod markers;

pub use discarded_error::{A_DISCARDED_ERROR_IS_EXPLAINED, Check_A_Discarded_Error_Is_Explained};
pub use exclusion::Check_An_Excluded_File_Says_Why;
pub use markers::{Check_Suppression_Directives_Carry_A_Reason, Check_Workspace_Markers_Carry_A_Reason};

use super::code_prefix::Code_Prefix;
use crate::{GO_LANGUAGE, SourceFile};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};

/// The code-standards skipped-test rule id.
pub const A_SKIPPED_TEST_STATES_WHY: &str = "a-skipped-test-states-why";
/// The code-standards excluded-file rule id.
pub const AN_EXCLUDED_FILE_SAYS_WHY: &str = "an-excluded-file-says-why";
/// The code-standards `//nolint` rule id.
pub const SUPPRESSION_DIRECTIVES_CARRY_A_REASON: &str = "suppression-directives-carry-a-reason";
/// The code-standards workspace-marker rule id.
pub const WORKSPACE_MARKERS_CARRY_A_REASON: &str = "workspace-markers-carry-a-reason";

/// Reports `t.Skip()`/`t.Skipf()` with an empty argument list, and `t.SkipNow()` (which
/// cannot carry an argument at all) with no adjacent comment.
#[must_use]
pub fn Check_A_Skipped_Test_States_Why(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();
    for source in sources
    {
        if source.Is_Written_In(GO_LANGUAGE)
        {
            findings.extend(Skip_Findings_In(source));
        }
    }
    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Skip_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let lines = Lines_Of(source);
    let mut findings = Vec::new();

    for (index, line) in lines.iter().enumerate()
    {
        if let Some(finding) = Skip_Finding_For(source, &lines, index, line)
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Skip_Finding_For(source: &SourceFile, lines: &[&str], index: usize, line: &str) -> Option<Finding>
{
    let code = Code_Prefix(line);

    if Has_Empty_Skip_Call(CodeText(&code), CallName("t.Skip(")) || Has_Empty_Skip_Call(CodeText(&code), CallName("t.Skipf("))
    {
        let finding = Finding_For_Line(source, A_SKIPPED_TEST_STATES_WHY, Line_Number(index), "calls t.Skip/t.Skipf with no explanatory message");
        return Some(finding);
    }

    if code.contains("t.SkipNow()") && !Has_Adjacent_Explanation(lines, index)
    {
        let finding = Finding_For_Line(
            source,
            A_SKIPPED_TEST_STATES_WHY,
            Line_Number(index),
            "calls t.SkipNow(), which takes no message, with no adjacent comment saying why",
        );
        return Some(finding);
    }

    return None;
}

/// `code` and `call` are both `&str`; without a distinct type per position, a call site
/// like `Has_Empty_Skip_Call(code, call)` reads as two interchangeable strings and a swap
/// compiles silently.
struct CodeText<'a>(&'a str);
struct CallName<'a>(&'a str);

fn Has_Empty_Skip_Call(code: CodeText<'_>, call: CallName<'_>) -> bool
{
    let code = code.0;
    let call = call.0;
    let Some(start) = code.find(call)
    else
    {
        return false;
    };
    let after = &code[start.saturating_add(call.len())..];
    let close = after.find(')').unwrap_or(after.len());
    return after[..close].trim().is_empty();
}

fn Lines_Of(source: &SourceFile) -> Vec<&str>
{
    return source.text.lines().collect();
}

fn Line_Number(index: usize) -> usize
{
    return index.saturating_add(1);
}

fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();
    let comment = trimmed.strip_prefix("//")?;
    return Some(comment.trim_start());
}

/// A trailing same-line comment with non-empty text, or a non-empty comment on the line
/// immediately above — the two shapes `a-discarded-error-is-explained` and `a-skipped-
/// test-states-why` (for `SkipNow`) both accept.
fn Has_Adjacent_Explanation(lines: &[&str], index: usize) -> bool
{
    if let Some(reason) = lines.get(index).and_then(|line| return line.split_once("//").map(|(_, rest)| return rest))
    {
        if !reason.trim().is_empty()
        {
            return true;
        }
    }

    if let Some(previous) = index.checked_sub(1)
    {
        if lines.get(previous).is_some_and(|line| return Comment_Text_Of(line).is_some_and(|comment| return !comment.trim().is_empty()))
        {
            return true;
        }
    }

    return false;
}

fn Finding_For_Line(source: &SourceFile, rule: &str, line_number: usize, because: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(rule),
        subject: source.subject,
        subject_name: format!("{}:{line_number}", source.path),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: format!("{} line {line_number} {because}", source.path),
        locations: vec![format!("{}:{line_number}", source.path)],
    };
}

#[cfg(test)]
mod tests;
