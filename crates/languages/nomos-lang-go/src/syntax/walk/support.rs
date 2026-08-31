//! Shared plumbing every declaration recorder in [`super`] uses: reading a node's own
//! named fields out of the parse tree, and turning what was found into one recorded item.

use super::super::{ItemKind, Item, Visibility};
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
pub(super) fn Push_Item_Record(items: &mut Vec<Item>, record: ItemRecord)
{
    let ordinal = u32::try_from(items.len()).unwrap_or(u32::MAX);

    items.push(Item {
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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Push_Item_Record_Should_Assign_Dense_Zero_Based_Ordinals()
    {
        let mut items = Vec::new();
        Push_Item_Record(
            &mut items,
            ItemRecord {
                kind: ItemKind::Function,
                scope: Vec::new(),
                name: "First".to_owned(),
                visibility: Visibility::Public,
                documentation: None,
                shape: None,
            },
        );
        Push_Item_Record(
            &mut items,
            ItemRecord {
                kind: ItemKind::Function,
                scope: Vec::new(),
                name: "Second".to_owned(),
                visibility: Visibility::Public,
                documentation: None,
                shape: None,
            },
        );

        let first = items.first().expect("two items were pushed");
        let second = items.get(1).expect("two items were pushed");
        assert_eq!(first.ordinal, 0);
        assert_eq!(second.ordinal, 1);
        assert_eq!(second.name, "Second");
    }

    #[test]
    fn Test_Function_Name_Should_Read_The_Nodes_Own_Name_Field()
    {
        let source = "package main\n\nfunc One() {}\n";
        let tree = Parse(source);
        let declaration = Find_Kind(tree.root_node(), "function_declaration").expect("the fixture declares a function");

        assert_eq!(Function_Name(declaration, source.as_bytes()), Some("One".to_owned()));
    }

    #[test]
    fn Test_Parameter_Arity_Should_Count_Every_Name_Sharing_One_Type()
    {
        let source = "package main\n\nfunc f(a, b int, c string) {}\n";
        let tree = Parse(source);
        let list = Find_Kind(tree.root_node(), "parameter_list").expect("the fixture declares parameters");

        assert_eq!(Parameter_Arity(list), 3);
    }

    #[test]
    fn Test_Named_Field_Children_Should_Find_Every_Name_A_Shared_Type_Declares()
    {
        let source = "package main\n\nfunc f(a, b int) {}\n";
        let tree = Parse(source);
        let declaration = Find_Kind(tree.root_node(), "parameter_declaration").expect("the fixture declares a parameter");

        let names = Named_Field_Children(declaration, "name");

        assert_eq!(names.len(), 2, "{names:?}");
    }

    fn Parse(source: &str) -> tree_sitter::Tree
    {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_go::LANGUAGE.into())
            .expect("the Go grammar is compiled into this crate");

        return parser.parse(source, None).expect("well-formed fixture source parses");
    }

    fn Find_Kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>>
    {
        if node.kind() == kind
        {
            return Some(node);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor)
        {
            if let Some(found) = Find_Kind(child, kind)
            {
                return Some(found);
            }
        }

        return None;
    }
}
