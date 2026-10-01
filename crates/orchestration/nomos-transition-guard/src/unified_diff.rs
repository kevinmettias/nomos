//! Reads a `-U0` unified diff for what it adds: every added line, and every file it adds or
//! changes. Removed lines are never judged, and a file git reports as binary is skipped.

use crate::judge::Judge;
use crate::refusal::{Place, Refusal};

/// Every refusal in the lines and paths `diff` adds, attributed to `commit`.
pub(crate) fn Judge_Diff(judge: &Judge<'_>, commit: Option<&str>, diff: &str) -> Vec<Refusal>
{
    let mut refusals = Vec::new();
    let mut path: Option<String> = None;
    let mut line_number: usize = 0;
    let mut in_header = false;
    for text in diff.lines()
    {
        if text.starts_with("diff --git ")
        {
            path = None;
            in_header = true;
        }
        else if in_header && text.starts_with("+++ ")
        {
            path = New_Path(text.trim_start_matches("+++ "));
            if let Some(added) = &path
            {
                let place = Place::Path { commit: commit.map(str::to_owned), path: added.clone() };
                refusals.extend(judge.Text(&place, added));
            }
        }
        else if in_header && text.starts_with("Binary files ")
        {
            path = None;
        }
        else if text.starts_with("@@ ")
        {
            in_header = false;
            line_number = Hunk_Start(text);
        }
        else if !in_header && let (Some(file), Some(added)) = (&path, text.strip_prefix('+'))
        {
            let place = Place::Line { commit: commit.map(str::to_owned), path: file.clone(), line: line_number };
            refusals.extend(judge.Text(&place, added));
            line_number = line_number.saturating_add(1);
        }
        else if !in_header && text.starts_with(' ')
        {
            line_number = line_number.saturating_add(1);
        }
    }

    return refusals;
}

/// The path in a `+++ ` header: `b/<path>`, a C-quoted `"b/<path>"`, or `/dev/null` for a
/// deleted file, which adds nothing.
fn New_Path(field: &str) -> Option<String>
{
    let unquoted = field.strip_prefix('"').and_then(|rest| return rest.strip_suffix('"')).map_or_else(|| return field.to_owned(), Unescape_C);
    if unquoted == "/dev/null"
    {
        return None;
    }
    return Some(unquoted.strip_prefix("b/").unwrap_or(&unquoted).to_owned());
}

/// Undoes the C escapes git quotes a path with, leaving any other text as it is.
fn Unescape_C(quoted: &str) -> String
{
    let mut out = String::new();
    let mut characters = quoted.chars();
    while let Some(character) = characters.next()
    {
        if character != '\\'
        {
            out.push(character);
            continue;
        }
        match characters.next()
        {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    return out;
}

/// The new-side first line of `@@ -a,b +c,d @@`.
fn Hunk_Start(header: &str) -> usize
{
    return header
        .split_whitespace()
        .nth(2)
        .and_then(|range| return range.trim_start_matches('+').split(',').next())
        .and_then(|start| return start.parse().ok())
        .unwrap_or(0);
}
