//! `deprecation`: a `#[deprecated]` attribute or a Go `// Deprecated:` marker with no
//! reason for the caller.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::SourceFile;
use nomos_contracts::Finding;

use super::{Comment_Text_Of, Finding_For_Line, For_Each_Line_Number, DEPRECATION};

/// Reports a `#[deprecated]`/`#[deprecated(...)]` attribute with no `note`, or a Go
/// `// Deprecated:` marker with nothing after the colon.
#[must_use]
pub fn Check_Deprecation_Carries_A_Reason(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Deprecation_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Deprecation_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    For_Each_Line_Number(&source.text, |line_number, line| {
        if Rust_Deprecated_Without_Reason(line)
        {
            let finding = Finding_For_Line(source, DEPRECATION, line_number, "marks `#[deprecated]` with no `note` for the caller");
            findings.push(finding);
        }
        else if let Some(comment) = Comment_Text_Of(line)
        {
            if Go_Deprecated_Without_Reason(comment)
            {
                let finding = Finding_For_Line(source, DEPRECATION, line_number, "marks `// Deprecated:` with nothing after the colon");
                findings.push(finding);
            }
        }
    });

    return findings;
}

/// A bare `#[deprecated]`, or `#[deprecated(...)]` whose parenthesized arguments close on
/// this same line and do not mention `note`. An attribute whose arguments do not close on
/// this line is not decidable from one line alone and is left unjudged rather than guessed.
fn Rust_Deprecated_Without_Reason(line: &str) -> bool
{
    let after = line.trim_start().strip_prefix("#[deprecated");
    let Some(after) = after
    else
    {
        return false;
    };

    if after.starts_with(']')
    {
        return true;
    }

    return Single_Line_Arguments_Lack_Note(after);
}

/// Whether `after` (the text following `#[deprecated`) opens a parenthesized argument list
/// that closes on this same line and does not mention `note`. An argument list that does not
/// close on this line is not decidable from one line alone and is left unjudged rather than
/// guessed.
fn Single_Line_Arguments_Lack_Note(after: &str) -> bool
{
    let Some(arguments) = after.strip_prefix('(')
    else
    {
        return false;
    };
    let Some((arguments, _rest)) = arguments.split_once(")]")
    else
    {
        return false;
    };

    return !arguments.contains("note");
}

/// A Go `Deprecated:` doc-comment marker with nothing but whitespace after the colon.
fn Go_Deprecated_Without_Reason(comment: &str) -> bool
{
    let Some(after) = comment.strip_prefix("Deprecated:")
    else
    {
        return false;
    };

    return after.trim().is_empty();
}
