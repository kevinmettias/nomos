//! Recording Go `const` and `var` declarations.

use super::super::{Documentation, ItemKind, SyntaxItem, Visibility};
use super::support::{ItemRecord, Named_Field_Children, Push};
use tree_sitter::Node;

pub(super) fn Record_Const_Or_Var_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8], kind: ItemKind)
{
    let name_nodes = Named_Field_Children(spec, "name");
    let shape = spec.child_by_field_name("type").map(Type_Shape);
    let documentation = Documentation(spec, source);

    for name_node in name_nodes
    {
        let Ok(name) = name_node.utf8_text(source)
        else
        {
            continue;
        };

        Push(
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
