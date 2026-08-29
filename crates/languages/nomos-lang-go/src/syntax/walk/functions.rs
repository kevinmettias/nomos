//! Recording Go function and method declarations — a top-level `func`, and a method bound
//! to a receiver type.

use super::super::{Documentation_Of_Declaration, ItemKind, SyntaxItem, Visibility};
use super::support::{Function_Name, ItemRecord, Parameter_Arity, Push_Item_Record};
use tree_sitter::Node;

pub(super) fn Record_Function(items: &mut Vec<SyntaxItem>, node: Node, source: &[u8])
{
    let Some(name) = Function_Name(node, source)
    else
    {
        return;
    };

    let arity = node.child_by_field_name("parameters").map_or(0, Parameter_Arity);
    let documentation = Documentation_Of_Declaration(node, source);

    Push_Item_Record(
        items,
        ItemRecord {
            kind: ItemKind::Function,
            scope: Vec::new(),
            visibility: Visibility::Of_Name(&name),
            name,
            documentation,
            shape: Some(nomos_cap_syntax::Function_Shape(arity)),
        },
    );
}

pub(super) fn Record_Method(items: &mut Vec<SyntaxItem>, node: Node, source: &[u8])
{
    let Some(name) = Method_Name(node, source)
    else
    {
        return;
    };

    let found = Method_Scope_And_Arity(node, source);
    let documentation = Documentation_Of_Declaration(node, source);

    Push_Item_Record(
        items,
        ItemRecord {
            kind: ItemKind::Function,
            scope: found.scope,
            visibility: Visibility::Of_Name(&name),
            name,
            documentation,
            shape: Some(nomos_cap_syntax::Function_Shape(found.arity)),
        },
    );
}

fn Method_Name(node: Node, source: &[u8]) -> Option<String>
{
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;

    return Some(name.to_owned());
}

/// [`Method_Scope_And_Arity`]'s own result, named so the caller reads `found.scope` and
/// `found.arity` rather than an unnamed pair whose order is only a convention.
struct MethodScopeAndArity
{
    scope: Vec<String>,
    arity: usize,
}

fn Method_Scope_And_Arity(node: Node, source: &[u8]) -> MethodScopeAndArity
{
    let receiver = node.child_by_field_name("receiver");
    let scope = receiver
        .and_then(|found| return Receiver_Type_Name(found, source))
        .map_or_else(Vec::new, |type_name| return vec![type_name]);
    let receiver_arity = usize::from(receiver.is_some());
    let parameter_arity = node.child_by_field_name("parameters").map_or(0, Parameter_Arity);

    return MethodScopeAndArity {
        scope,
        arity: receiver_arity.saturating_add(parameter_arity),
    };
}

/// The receiver's type name, unwrapped through the pointer and generic-instantiation forms
/// that do not change what the type is *of* — `*T` and `*Container[U]` both name `T` and
/// `Container`, mirroring `nomos-lang-rust`'s own `Type_Head` recursion through wrappers.
fn Receiver_Type_Name(receiver_list: Node, source: &[u8]) -> Option<String>
{
    let mut cursor = receiver_list.walk();
    let declaration = receiver_list
        .children(&mut cursor)
        .find(|child| return child.kind() == "parameter_declaration")?;
    let type_node = declaration.child_by_field_name("type")?;

    return Unwrapped_Type_Name(type_node, source);
}

fn Unwrapped_Type_Name(node: Node, source: &[u8]) -> Option<String>
{
    return match node.kind()
    {
        "type_identifier" => node.utf8_text(source).ok().map(str::to_owned),
        "pointer_type" => Unwrapped_Type_Name(node.named_child(0)?, source),
        "generic_type" => Unwrapped_Type_Name(node.child_by_field_name("type")?, source),
        _ => None,
    };
}
