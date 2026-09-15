//! `todo-format-is-todo-name-description-ticket`: a `TODO` marker missing owner,
//! description or ticket.
//!
//! Split out of [`super`], which states the family's shared reasoning.

use crate::SourceFile;
use nomos_contracts::Finding;

use super::{Comment_Text_Of, Finding_For_Line, For_Each_Line_Number, TODO_FORMAT};

/// Reports every deferred-work marker comment in `sources` that does not carry owner,
/// description and ticket.
///
/// The marker itself and the exact shape are the rule id's own words. This prose does not
/// spell the marker, because detection and acceptance agree on what counts as one: the
/// marker must open the comment. A comment that merely mentions it in passing is not a
/// marker and is not judged, so a comment that does open with it and is missing owner,
/// description or ticket always has a conforming edit.
#[must_use]
pub fn Check_Todo_Format(sources: &[SourceFile]) -> Vec<Finding>
{
    let mut findings = Vec::new();

    for source in sources
    {
        findings.extend(Todo_Findings_In(source));
    }

    findings.sort_by(|left, right| return left.subject_name.cmp(&right.subject_name));
    return findings;
}

fn Todo_Findings_In(source: &SourceFile) -> Vec<Finding>
{
    let mut findings = Vec::new();

    For_Each_Line_Number(&source.text, |line_number, line| {
        if let Some(comment) = Comment_Text_Of(line)
        {
            if Starts_With_Todo(comment) && !Has_Valid_Todo_Format(comment)
            {
                let finding = Finding_For_Line(
                    source,
                    TODO_FORMAT,
                    line_number,
                    "contains a TODO without `TODO(owner): description (#ticket)` format",
                );
                findings.push(finding);
            }
        }
    });

    return findings;
}

/// A marker opens the comment; a mention elsewhere in the comment's prose is not one, and is
/// left unjudged rather than reported with no route to a conforming edit.
fn Starts_With_Todo(comment: &str) -> bool
{
    return comment.starts_with("TODO");
}

fn Has_Valid_Todo_Format(comment: &str) -> bool
{
    let Some(after_marker) = comment.strip_prefix("TODO(")
    else
    {
        return false;
    };
    let Some((owner, after_owner)) = after_marker.split_once("):")
    else
    {
        return false;
    };
    if !Is_Valid_Owner(owner)
    {
        return false;
    }

    let description = after_owner.trim();
    if description.is_empty()
    {
        return false;
    }

    return Has_Ticket_Suffix(description);
}

fn Is_Valid_Owner(owner: &str) -> bool
{
    let owner_is_empty = owner.trim().is_empty();
    let owner_has_whitespace = owner.chars().any(char::is_whitespace);

    return !owner_is_empty && !owner_has_whitespace;
}

fn Has_Ticket_Suffix(description: &str) -> bool
{
    let Some(ticket) = description.strip_suffix(')')
    else
    {
        return false;
    };
    let Some((before_ticket, number)) = ticket.rsplit_once("(#")
    else
    {
        return false;
    };

    return !before_ticket.trim().is_empty() && !number.is_empty() && number.chars().all(|character| return character.is_ascii_digit());
}
