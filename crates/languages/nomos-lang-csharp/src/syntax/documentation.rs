//! The doc comment a declaration carries, if it carries one.
//!
//! C# has no attribute syntax a doc comment compiles into the way `///` becomes `#[doc]` for
//! `syn` to hand back as already-separated text. `tree-sitter-c-sharp` parses a comment as an
//! ordinary sibling node, so this module recovers what a C# documentation tool recovers:
//! comment lines immediately above a declaration, with no blank source line between them — a
//! gap ends the run rather than being skipped over, because a comment separated from its
//! declaration by a blank line is conventionally about something else.
//!
//! # Why there is no climb here, where `nomos-lang-go`'s equivalent needs one
//!
//! Go's reader records a `var_spec` nested inside a `var_declaration`, so a comment written
//! above the line is a sibling of the *outer* node and its own reader has to climb to find it.
//! Every declaration this crate records is already a direct child of the list that encloses it
//! — a `compilation_unit`, a `declaration_list` or an `enum_member_declaration_list` — so the
//! comment above it is its own immediate prior sibling and there is nothing to climb.
//!
//! An attribute list does not break that, and it looked as though it might: `[Obsolete]` parses
//! as a *child* of the declaration it decorates rather than a sibling before it, so the
//! declaration node still begins at the attribute's own row and the comment above the attribute
//! is still the declaration's prior sibling. Confirmed against the real tree rather than
//! assumed.

use tree_sitter::Node;

/// The comment openers a C# line comment can carry, longest first — `///` is the XML
/// documentation form and would otherwise be stripped as an ordinary `//` with a stray slash
/// left at the front of every line.
const LINE_COMMENT_MARKERS: &[&str] = &["///", "//"];

/// The declaration's documentation, as one string, or `None` when it has none.
///
/// Walks backward through siblings collecting a contiguous block of `comment` nodes, then
/// reverses it back into source order. Contiguous means row-adjacent: the last line of one
/// comment must end on the row immediately before the next thing in the run starts, or the run
/// stops there.
pub(super) fn Documentation_Of_Declaration(declaration: Node, source: &[u8]) -> Option<String>
{
    let boundary_row = declaration.start_position().row.checked_sub(1)?;
    let mut lines = Contiguous_Comment_Lines(declaration, boundary_row, source);

    if lines.is_empty()
    {
        return None;
    }

    lines.reverse();
    return Some(lines.join("\n"));
}

/// Every contiguous comment line above `anchor`, nearest first — [`Documentation_Of_Declaration`]'s
/// own walk, named so its body reads as "collect the run, then reverse it into source order."
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

/// One comment line eligible to join the run, and the boundary row the next candidate above it
/// must end on — or `None` when `previous` is not a contiguous comment at all.
///
/// The boundary row saturates rather than checking, and the difference is a real defect this
/// crate's fixtures caught: a comment on the *first* row of a file has no row above it, so a
/// checked subtraction returns `None` there and the whole run is refused — which means a file
/// opening with a doc comment on line one loses its documentation. Saturating to zero is safe
/// because a node on row zero has no prior sibling for the next turn of the loop to find.
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
    let above = previous.start_position().row.saturating_sub(1);

    return Some((Stripped_Comment_Text(text), above));
}

/// One comment's text with its opener removed and the result trimmed.
///
/// A block comment spanning several lines keeps its interior newlines — stripping only the
/// delimiters preserves whatever paragraph the author wrote rather than flattening it. The XML
/// tags inside a `///` line are left exactly as written: they are what the author put there,
/// and rendering them would be this provider deciding what the documentation means.
fn Stripped_Comment_Text(raw: &str) -> String
{
    for marker in LINE_COMMENT_MARKERS
    {
        if let Some(body) = raw.strip_prefix(marker)
        {
            return body.trim().to_owned();
        }
    }

    if let Some(body) = Block_Comment_Body(raw)
    {
        return body.trim().to_owned();
    }

    return raw.trim().to_owned();
}

/// A block comment's interior, whichever of the two openers it carries.
fn Block_Comment_Body(raw: &str) -> Option<&str>
{
    let opened = raw.strip_prefix("/**").or_else(|| return raw.strip_prefix("/*"))?;

    return opened.strip_suffix("*/");
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};

    #[test]
    fn Test_Documentation_Of_Declaration_Should_Read_A_Comment_Directly_Above_It()
    {
        let source = "/// <summary>A widget.</summary>\npublic class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(
            Documentation_Of_Declaration(declaration, source.as_bytes()),
            Some("<summary>A widget.</summary>".to_owned())
        );
    }

    /// Two `///` lines are two sibling nodes, and the run has to join them rather than report
    /// only the nearest.
    #[test]
    fn Test_Documentation_Of_Declaration_Should_Join_A_Run_Of_Comment_Lines_In_Source_Order()
    {
        let source = "/// First.\n/// Second.\npublic class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(
            Documentation_Of_Declaration(declaration, source.as_bytes()),
            Some("First.\nSecond.".to_owned())
        );
    }

    /// The falsifier for the row-adjacency guard: with it removed, this comment would be read
    /// as the class's documentation although a blank line separates them.
    #[test]
    fn Test_A_Comment_Separated_By_A_Blank_Line_Should_Not_Be_Documentation()
    {
        let source = "// About the file.\n\npublic class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(Documentation_Of_Declaration(declaration, source.as_bytes()), None);
    }

    /// An attribute list is a child of the declaration, not a sibling before it, so the comment
    /// above the attribute still belongs to the declaration.
    #[test]
    fn Test_A_Comment_Above_An_Attribute_Should_Still_Be_The_Declarations_Documentation()
    {
        let source = "/// Documented.\n[Obsolete]\npublic class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(
            Documentation_Of_Declaration(declaration, source.as_bytes()),
            Some("Documented.".to_owned())
        );
    }

    #[test]
    fn Test_Documentation_Of_Declaration_Should_Be_None_With_No_Comment_Directly_Above_It()
    {
        let source = "public class Widget { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let declaration = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");

        assert_eq!(Documentation_Of_Declaration(declaration, source.as_bytes()), None);
    }
}
