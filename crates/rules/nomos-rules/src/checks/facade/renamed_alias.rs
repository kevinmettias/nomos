//! A renamed public re-export that states no contract for the name it publishes.
//!
//! Split out of [`super`], which states the family's shared reasoning. This file is the
//! reading and the judgment behind `facade-aliases-name-the-contract` alone, including the
//! `facade-alias: allow` marker reader no other rule here shares.

use crate::{RUST_LANGUAGE, SourceFile};
use nomos_contracts::Finding;

use super::text::Re_Export;
use super::{Code_Lines, Finding_At, Line_Number, FACADE_ALIASES_NAME_THE_CONTRACT, FACADE_ALIAS_MARKER};

/// Reports a renamed public re-export carrying no adjacent `facade-alias: allow` reason --
/// the alias is a public name, so it owes the contract it states.
#[must_use]
pub fn Check_A_Renamed_Facade_Re_Export_Names_The_Contract(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        if source.Is_Written_In(RUST_LANGUAGE)
        {
            findings.extend(Unexplained_Alias_Findings_In(source));
        }
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Unexplained_Alias_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let code_lines = Code_Lines(&source.text);
    let raw_lines: Vec<&str> = source.text.lines().collect();

    let mut findings = Vec::new();
    for (index, line) in code_lines.iter().enumerate()
    {
        let finding = Unexplained_Alias_Finding_For_Line(source, line, index, &raw_lines);
        if let Some(finding) = finding
        {
            findings.push(finding);
        }
    }

    return findings;
}

fn Unexplained_Alias_Finding_For_Line(source: &SourceFile, line: &str, index: usize, raw_lines: &[&str]) -> Option<Finding>
{
    let re_export = Re_Export(line)?;
    let alias = re_export.alias?;
    if Has_Marker_Reason(raw_lines, index)
    {
        return None;
    }

    let line_number = Line_Number(index);
    return Some(Finding_At(
        source,
        FACADE_ALIASES_NAME_THE_CONTRACT,
        line_number,
        &format!(
            "renames the facade re-export to `{alias}` and carries no adjacent `{FACADE_ALIAS_MARKER}` reason; an \
             alias is a public name, so either rename the item at its declaration or state the contract this \
             boundary name gives it"
        ),
    ));
}

/// The statement's own line, or a contiguous run of blank, comment and attribute lines
/// walking upward from it, carries the literal `facade-alias: allow` marker with a non-empty
/// reason -- the same shape `concurrency_text` and `error_text` read their own markers with.
fn Has_Marker_Reason(lines: &[&str], index: usize) -> bool
{
    if lines.get(index).is_some_and(|line| return Marker_Reason_In(line).is_some_and(|reason| return !reason.is_empty()))
    {
        return true;
    }

    return Marker_Reason_Found_Above(lines, index);
}

/// Walks upward from `index` (exclusive) over a contiguous run of blank/comment/attribute
/// lines, stopping at the first line that is not skippable -- returning whether a marker
/// reason was found with a non-empty reason before that happened.
fn Marker_Reason_Found_Above(lines: &[&str], index: usize) -> bool
{
    let mut cursor = index;
    while cursor > 0
    {
        cursor = cursor.saturating_sub(1);
        let Some(line) = lines.get(cursor)
        else
        {
            break;
        };

        if let Some(reason) = Marker_Reason_In(line)
        {
            return !reason.is_empty();
        }

        if !Is_Skippable_Above(line)
        {
            break;
        }
    }

    return false;
}

fn Is_Skippable_Above(line: &str) -> bool
{
    let trimmed = line.trim();

    return trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with("#[")
        || trimmed.starts_with("#![");
}

/// The marker must lead a comment, never merely appear somewhere on the line, so prose or a
/// string literal spelling its words cannot silence a real finding.
fn Marker_Reason_In(line: &str) -> Option<&str>
{
    let comment = line.split_once("//").map(|(_before, after)| return after)?;
    let after_marker = comment.trim_start().strip_prefix(FACADE_ALIAS_MARKER)?;
    let reason = after_marker.trim_start().strip_prefix(':').unwrap_or(after_marker);

    return Some(reason.trim());
}
