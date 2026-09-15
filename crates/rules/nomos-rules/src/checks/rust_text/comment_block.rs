//! The primitives every comment-shaped rule in this family is written over.
//!
//! Split out of [`super`], which states the family's shared reasoning. Nothing here knows
//! which rule is asking: each function walks the contiguous block immediately above a line,
//! tolerating the blank and bare-attribute lines that sit inside one, and hands the caller
//! either a verdict or the texts it found. A block ends at the first line that is neither a
//! comment, a blank, nor a bare attribute.

/// Every `///` line of the contiguous doc-comment block immediately above `index`, in file
/// order, `///` and surrounding whitespace stripped — tolerant of an intervening attribute
/// line (`#[must_use]`, ...) between the doc block and the declaration it documents, the
/// same way [`Is_Skippable_Block_Line`] already is for a `//` comment block. A line that sits
/// inside the block without being a doc line contributes an empty entry, which the one caller
/// reads exactly as it reads a blank one.
pub(super) fn Preceding_Documentation_Comment_Block<'a>(lines: &[&'a str], index: usize) -> Vec<&'a str>
{
    let mut doc_lines = Vec::new();
    let mut cursor = index;

    while let Some(previous) = cursor.checked_sub(1)
    {
        let Some(doc) = Documentation_Line_At(lines, previous)
        else
        {
            break;
        };

        doc_lines.push(doc);
        cursor = previous;
    }

    doc_lines.reverse();
    return doc_lines;
}

/// The doc text of the `///` line at `previous`, trimmed; an empty string for a blank or
/// bare-attribute line, which sits inside the block without carrying text of its own; `None`
/// where the block ends.
fn Documentation_Line_At<'a>(lines: &[&'a str], previous: usize) -> Option<&'a str>
{
    let line = lines.get(previous)?;
    if Is_Skippable_Block_Line(line)
    {
        return Some("");
    }

    return line.trim_start().strip_prefix("///").map(str::trim);
}

/// Every comment line ([`Comment_Text_Of`]) of the contiguous block immediately above
/// `index`, in file order, marker and surrounding whitespace stripped — tolerant of an
/// intervening blank or attribute line the same way [`Is_Skippable_Block_Line`] already is.
/// A skippable line contributes an empty entry, which is what "sits inside the block but
/// carries no comment" looks like to the caller.
pub(super) fn Preceding_Comment_Block<'a>(lines: &[&'a str], index: usize) -> Vec<&'a str>
{
    let mut comment_lines = Vec::new();
    let mut cursor = index;

    while let Some(previous) = cursor.checked_sub(1)
    {
        let Some(comment) = Comment_Line_In_Block_At(lines, previous)
        else
        {
            break;
        };

        comment_lines.push(comment);
        cursor = previous;
    }

    comment_lines.reverse();
    return comment_lines;
}

/// The comment text of the line at `previous`, trimmed; an empty string for a blank or
/// bare-attribute line, which sits inside the block without carrying a comment of its own;
/// `None` where the block ends.
fn Comment_Line_In_Block_At<'a>(lines: &[&'a str], previous: usize) -> Option<&'a str>
{
    let line = lines.get(previous)?;
    if Is_Skippable_Block_Line(line)
    {
        return Some("");
    }

    return Comment_Text_Of(line).map(str::trim);
}

/// `every-allow-carries-a-justification`'s own example is plain prose with no special
/// marker, unlike the panic and smart-pointer rules' `panic:`/`smart-pointer: allow:`
/// keywords — so any non-empty comment satisfies it.
pub(super) fn Comment_Is_Non_Empty(line: &str) -> bool
{
    return Comment_Text_Of(line).is_some_and(|comment| return !comment.trim().is_empty());
}

pub(super) fn Previous_Comment_Block_Has(lines: &[&str], index: usize, predicate: fn(&str) -> bool) -> bool
{
    let mut cursor = index;
    while let Some(previous) = cursor.checked_sub(1)
    {
        if let Some(verdict) = Comment_Block_Step(lines, previous, predicate)
        {
            return verdict;
        }

        cursor = previous;
    }

    return false;
}

/// One backward step through the comment block above a flagged line: `None` means keep
/// walking upward, `Some(verdict)` means the walk has its answer (the block ended, or the
/// predicate matched).
fn Comment_Block_Step(lines: &[&str], previous: usize, predicate: fn(&str) -> bool) -> Option<bool>
{
    let Some(line) = lines.get(previous)
    else
    {
        return Some(false);
    };

    if Is_Skippable_Block_Line(line)
    {
        return None;
    }

    return Comment_Line_Verdict(line, predicate);
}

/// `None` means this comment line did not carry the reason and the walk should keep
/// scanning upward through the rest of the block.
fn Comment_Line_Verdict(line: &str, predicate: fn(&str) -> bool) -> Option<bool>
{
    if !Is_Comment_Line(line)
    {
        return Some(false);
    }

    if predicate(line)
    {
        return Some(true);
    }

    return None;
}

fn Is_Comment_Line(line: &str) -> bool
{
    return Comment_Text_Of(line).is_some();
}

pub(super) fn Comment_Text_Of(line: &str) -> Option<&str>
{
    let trimmed = line.trim_start();

    for marker in ["//", "///", "//!"]
    {
        if let Some(comment) = trimmed.strip_prefix(marker)
        {
            return Some(comment.trim_start());
        }
    }

    return None;
}

/// A blank line or a bare attribute (`#[...]`) is not itself a comment, but sits inside the
/// contiguous block the walk is scanning and does not end it.
fn Is_Skippable_Block_Line(line: &str) -> bool
{
    let is_blank = line.trim().is_empty();
    return is_blank || Is_Attribute_Line(line);
}

fn Is_Attribute_Line(line: &str) -> bool
{
    return line.trim_start().starts_with("#[");
}
