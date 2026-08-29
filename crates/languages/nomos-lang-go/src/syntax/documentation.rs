//! The doc comment a declaration carries, if it carries one.
//!
//! Go has no attribute syntax a doc comment compiles into the way `///` becomes `#[doc]`
//! for `syn` to hand back as already-separated text. `tree-sitter-go` parses a comment as an
//! ordinary sibling node, so this module recovers what godoc's own convention recovers:
//! comment lines immediately above a declaration, with no blank source line between them —
//! a gap ends the run rather than being skipped over, because a comment separated from its
//! declaration by a blank line is conventionally about something else.

use tree_sitter::Node;

/// The declaration's documentation, as one string, or `None` when it has none.
///
/// Walks backward through named siblings collecting a contiguous block of `comment` nodes,
/// then reverses it back into source order. Contiguous means row-adjacent: the last line of
/// one comment must end on the row immediately before the next thing in the run starts, or
/// the run stops there.
///
/// Starts from [`Search_Anchor`] rather than from `declaration` itself. A spec that is the
/// first (or only) one under its `_declaration` node has no prior sibling of its own to
/// check — `var Tables []string` parses as `var_declaration(var_spec(name, type))`, and a
/// comment written above the line attaches as a sibling of `var_declaration`, not of the
/// `var_spec` inside it. A spec further into a grouped block (`const ( A = 1\n// B\nB = 2 )`)
/// needs no such climb — its own comment is already its own immediate prior sibling — and
/// [`Search_Anchor`] leaves it exactly where it was for that case.
pub(super) fn Documentation(declaration: Node, source: &[u8]) -> Option<String>
{
    let anchor = Search_Anchor(declaration);
    let boundary_row = anchor.start_position().row.checked_sub(1)?;
    let mut lines = Contiguous_Comment_Lines(anchor, boundary_row, source);

    if lines.is_empty()
    {
        return None;
    }

    lines.reverse();
    return Some(lines.join("\n"));
}

/// Climbs from a spec to the outermost ancestor with nothing but grammar punctuation before
/// it.
///
/// `var Tables []string`'s `var_spec` has a prior sibling — the `var` keyword itself, an
/// *unnamed* token rather than a real declaration. Climbing only while there is no sibling
/// at all would stop right there and never reach `var_declaration`, whose own prior sibling
/// is the comment. A node with a real, *named* prior sibling — another spec, or a comment —
/// stops the climb immediately: there is live content directly above it, and anything
/// further up belongs to that content rather than to this one. `source_file` is never
/// climbed past, since above it there is nothing.
fn Search_Anchor(node: Node) -> Node
{
    let mut anchor = node;

    while let Some(next) = Next_Anchor(anchor)
    {
        anchor = next;
    }

    return anchor;
}

/// The next ancestor to climb to, or `None` when the climb should stop at `anchor` itself.
fn Next_Anchor(anchor: Node) -> Option<Node>
{
    let blocked_by_real_content = anchor.prev_sibling().is_some_and(|previous| return previous.is_named());

    if blocked_by_real_content
    {
        return None;
    }

    let parent = anchor.parent()?;

    if parent.kind() == "source_file"
    {
        return None;
    }

    return Some(parent);
}

/// Every contiguous comment line above `anchor`, nearest first -- [`Documentation`]'s own
/// walk, named so its body reads as "collect the run, then reverse it into source order."
fn Contiguous_Comment_Lines(anchor: Node, mut boundary_row: usize, source: &[u8]) -> Vec<String>
{
    let mut lines = Vec::new();
    let mut cursor = anchor;

    while let Some(previous) = cursor.prev_sibling()
    {
        let Some((text, next_boundary_row)) = Comment_Line(previous, boundary_row, source)
        else
        {
            break;
        };

        lines.push(text);
        cursor = previous;
        boundary_row = next_boundary_row;
    }

    return lines;
}

/// One comment line eligible to join the run, and the boundary row the next candidate above
/// it must end on — or `None` when `previous` is not a contiguous comment at all.
fn Comment_Line(previous: Node, boundary_row: usize, source: &[u8]) -> Option<(String, usize)>
{
    if previous.kind() != "comment"
    {
        return None;
    }

    if previous.end_position().row != boundary_row
    {
        return None;
    }

    let text = previous.utf8_text(source).ok()?;
    let above = previous.start_position().row.checked_sub(1)?;

    return Some((Stripped(text), above));
}

/// One comment's text with its `//` or `/* ... */` marker removed and the result trimmed.
///
/// A block comment spanning several lines keeps its interior newlines — stripping only the
/// delimiters preserves whatever paragraph the author wrote rather than flattening it.
fn Stripped(raw: &str) -> String
{
    if let Some(body) = raw.strip_prefix("//")
    {
        return body.trim().to_owned();
    }

    if let Some(body) = raw.strip_prefix("/*").and_then(|rest| return rest.strip_suffix("*/"))
    {
        return body.trim().to_owned();
    }

    return raw.trim().to_owned();
}
