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
    let mut lines: Vec<String> = Vec::new();
    let mut boundary_row = anchor.start_position().row.checked_sub(1)?;
    let mut cursor = anchor;

    while let Some(previous) = cursor.prev_sibling()
    {
        if previous.kind() != "comment"
        {
            break;
        }

        if previous.end_position().row != boundary_row
        {
            break;
        }

        let Ok(text) = previous.utf8_text(source)
        else
        {
            break;
        };

        lines.push(Stripped(text));
        cursor = previous;

        let Some(above) = previous.start_position().row.checked_sub(1)
        else
        {
            break;
        };
        boundary_row = above;
    }

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

    loop
    {
        let blocked_by_real_content = anchor.prev_sibling().is_some_and(|previous| return previous.is_named());

        if blocked_by_real_content
        {
            break;
        }

        let Some(parent) = anchor.parent()
        else
        {
            break;
        };

        if parent.kind() == "source_file"
        {
            break;
        }

        anchor = parent;
    }

    return anchor;
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
