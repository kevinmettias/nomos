//! Shared plumbing every declaration recorder in [`super`] uses: reading a node's own
//! named fields out of the parse tree, and turning what was found into one recorded item.

use super::super::{ItemKind, SyntaxItem, Visibility};
use tree_sitter::Node;

/// The fields one recorded declaration needs, grouped so [`Push_Item_Record`] takes a small,
/// fixed number of parameters regardless of how many facts a declaration carries.
pub(super) struct ItemRecord
{
    pub(super) kind: ItemKind,
    pub(super) scope: Vec<String>,
    pub(super) name: String,
    pub(super) visibility: Visibility,
    pub(super) documentation: Option<String>,
    pub(super) shape: Option<String>,
}

/// Records one declaration, with what this provider observed about it.
///
/// `shape` is `None` where the form has no shape to describe rather than where none could be
/// seen. This provider parses, so everything it does not record is an absence it looked for
/// — the distinction the payload spells `.` rather than `-`.
pub(super) fn Push_Item_Record(items: &mut Vec<SyntaxItem>, record: ItemRecord)
{
    let ordinal = u32::try_from(items.len()).unwrap_or(u32::MAX);

    items.push(SyntaxItem {
        ordinal,
        kind: record.kind,
        scope: record.scope,
        name: record.name,
        visibility: record.visibility,
        documentation: record.documentation,
        shape: record.shape,
    });
}

/// Every child carrying `field`, as an actually-named node.
///
/// Not `Node::children_by_field_name`: in `tree-sitter-go`'s real grammar, a repeated field
/// is not always contiguous with itself — `const A, B = 1, 2` tags the separating `,` with
/// the same `name` field its neighbours carry, and the convenience iterator hands it back
/// along with them. Confirmed against the real grammar rather than assumed, after that
/// token showed up as a third "name". Filtering to named nodes is what a hand-walked
/// `TreeCursor` buys back: the grammar's own token/rule distinction, which
/// `is_named` reads directly rather than re-deriving from `kind()` strings.
pub(super) fn Named_Field_Children<'a>(node: Node<'a>, field: &str) -> Vec<Node<'a>>
{
    let mut cursor = node.walk();
    let mut found = Vec::new();

    if cursor.goto_first_child()
    {
        loop
        {
            let current = cursor.node();

            if cursor.field_name() == Some(field) && current.is_named()
            {
                found.push(current);
            }

            if !cursor.goto_next_sibling()
            {
                break;
            }
        }
    }

    return found;
}

/// The name a `name`-field-carrying node declares — a function or method declaration, an
/// interface method element, or a type alias spec, all of which bind their own identifier
/// the same way.
pub(super) fn Function_Name(node: Node, source: &[u8]) -> Option<String>
{
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;

    return Some(name.to_owned());
}

/// How many parameters a `parameter_list` declares.
///
/// A shared type binds several names to one `parameter_declaration` (`func f(a, b int)`),
/// so this counts `name` fields rather than declaration nodes — a declaration with no name
/// at all (`func f(int)`) still declares one unnamed parameter and counts as one.
pub(super) fn Parameter_Arity(list: Node) -> usize
{
    let mut cursor = list.walk();
    let mut total = 0_usize;

    for child in list.children(&mut cursor)
    {
        if child.kind() != "parameter_declaration" && child.kind() != "variadic_parameter_declaration"
        {
            continue;
        }

        let bound = Named_Field_Children(child, "name").len();
        total = total.saturating_add(bound.max(1));
    }

    return total;
}
