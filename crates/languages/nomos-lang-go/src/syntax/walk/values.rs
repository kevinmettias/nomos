//! Recording Go `const` and `var` declarations.

use super::super::{Documentation_Of_Declaration, ItemKind, Item, Visibility};
use super::support::{ItemRecord, Named_Field_Children, Push_Item_Record};
use tree_sitter::Node;

pub(super) fn Record_Const_Or_Var_Spec(items: &mut Vec<Item>, spec: Node, source: &[u8], kind: ItemKind)
{
    let name_nodes = Named_Field_Children(spec, "name");
    let shape = spec.child_by_field_name("type").map(Type_Shape);
    let documentation = Documentation_Of_Declaration(spec, source);

    for name_node in name_nodes
    {
        let Ok(name) = name_node.utf8_text(source)
        else
        {
            continue;
        };

        Push_Item_Record(
            items,
            ItemRecord {
                kind,
                scope: Vec::new(),
                name: name.to_owned(),
                visibility: Visibility::Of_Name(name),
                documentation: documentation.clone(),
                shape: shape.clone(),
            },
        );
    }
}

/// The shape a declared type has, in the payload's vocabulary — [`nomos_cap_syntax::SLICE`]
/// for a slice or array, however many pointers deep, and [`nomos_cap_syntax::VALUE`]
/// otherwise. The one distinction a consumer cannot recover from the rest of the record,
/// mirroring `nomos-lang-rust`'s own `Type_Shape`.
fn Type_Shape(node: Node) -> String
{
    return match node.kind()
    {
        "pointer_type" => node.named_child(0).map_or_else(|| return nomos_cap_syntax::VALUE.to_owned(), Type_Shape),
        "slice_type" | "array_type" => nomos_cap_syntax::SLICE.to_owned(),
        _ => nomos_cap_syntax::VALUE.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Record_Const_Or_Var_Spec_Should_Record_Each_Name_Sharing_One_Value_List()
    {
        let source = "package main\n\nconst A, B = 1, 2\n";
        let tree = Parsed_Fixture_Tree(source);
        let spec = Find_Kind(tree.root_node(), "const_spec").expect("the fixture declares a const spec");
        let mut items = Vec::new();

        Record_Const_Or_Var_Spec(&mut items, spec, source.as_bytes(), ItemKind::Constant);

        let names: Vec<&str> = items.iter().map(|item| return item.name.as_str()).collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    fn Parsed_Fixture_Tree(source: &str) -> tree_sitter::Tree
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
