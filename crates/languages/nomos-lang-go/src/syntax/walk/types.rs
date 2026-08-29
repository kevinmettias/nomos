//! Recording Go type declarations: a struct, an interface (with its own method set), a
//! plain type definition, and a type alias.

use super::super::{Documentation, ItemKind, SyntaxItem, Visibility};
use super::support::{Function_Name, ItemRecord, Named_Field_Children, Parameter_Arity, Push};
use tree_sitter::Node;

pub(super) fn Record_Type_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(name) = Type_Spec_Name(spec, source)
    else
    {
        return;
    };

    let type_node = spec.child_by_field_name("type");
    let fields = Type_Spec_Fields(type_node, spec, source);

    Push(
        items,
        ItemRecord {
            kind: fields.kind,
            scope: Vec::new(),
            name: name.clone(),
            visibility: Visibility::Of_Name(&name),
            documentation: fields.documentation,
            shape: fields.shape,
        },
    );

    Record_Interface_Methods_If_Interface(items, (fields.kind, type_node), &name, source);
}

/// `type_node`'s kind and shape, alongside `spec`'s own documentation -- [`Record_Type_Spec`]'s
/// own gathering step, named so its body reads as "gather the fields, then push them."
/// [`Type_Spec_Fields`]'s own result, named so the caller reads `fields.kind` and the rest
/// rather than an unnamed triple whose order is only a convention.
struct TypeSpecFields
{
    kind: ItemKind,
    shape: Option<String>,
    documentation: Option<String>,
}

fn Type_Spec_Fields(type_node: Option<Node>, spec: Node, source: &[u8]) -> TypeSpecFields
{
    let kind = Type_Spec_Kind(type_node);
    let shape = Type_Spec_Shape(kind, type_node, source);
    let documentation = Documentation(spec, source);

    return TypeSpecFields { kind, shape, documentation };
}

/// `type_node`'s methods, recorded when [`Record_Type_Spec`] just built an interface --
/// its own trailing, conditional step, named so the parent's body ends at "record it."
fn Record_Interface_Methods_If_Interface(items: &mut Vec<SyntaxItem>, kind_and_type: (ItemKind, Option<Node>), name: &str, source: &[u8])
{
    if let (ItemKind::Interface, Some(interface)) = kind_and_type
    {
        Record_Interface_Methods(items, interface, name, source);
    }
}

fn Type_Spec_Name(spec: Node, source: &[u8]) -> Option<String>
{
    let name_node = spec.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;

    return Some(name.to_owned());
}

fn Type_Spec_Kind(type_node: Option<Node>) -> ItemKind
{
    return match type_node.map(|found| return found.kind())
    {
        Some("struct_type") => ItemKind::Struct,
        Some("interface_type") => ItemKind::Interface,
        _ => ItemKind::TypeDefinition,
    };
}

fn Type_Spec_Shape(kind: ItemKind, type_node: Option<Node>, source: &[u8]) -> Option<String>
{
    return match (kind, type_node)
    {
        (ItemKind::Struct, Some(struct_type)) => Struct_Shape(struct_type, source),
        _ => None,
    };
}

/// A method an interface's method set declares.
///
/// Recorded with [`ItemKind::Function`], scoped into the interface's own name, the same
/// choice `nomos-lang-rust` makes for a trait member. Unlike a Rust trait member, this one
/// carries a real, non-[`Visibility::NotApplicable`] visibility — see [`Visibility`]'s own
/// doc for why that is not a copy-paste of the Rust provider's choice but a different fact
/// about a different language. An embedded interface (`type_elem`) is not a named method and
/// is not recorded — it declares no name of its own for this provider to attribute an item
/// to.
fn Record_Interface_Methods(items: &mut Vec<SyntaxItem>, interface: Node, interface_name: &str, source: &[u8])
{
    let mut cursor = interface.walk();

    for member in interface.children(&mut cursor)
    {
        if member.kind() != "method_elem"
        {
            continue;
        }

        Record_Interface_Method(items, member, interface_name, source);
    }
}

fn Record_Interface_Method(items: &mut Vec<SyntaxItem>, member: Node, interface_name: &str, source: &[u8])
{
    let Some(name) = Function_Name(member, source)
    else
    {
        return;
    };

    let arity = member.child_by_field_name("parameters").map_or(0, Parameter_Arity);
    let documentation = Documentation(member, source);

    Push(
        items,
        ItemRecord {
            kind: ItemKind::Function,
            scope: vec![interface_name.to_owned()],
            visibility: Visibility::Of_Name(&name),
            name,
            documentation,
            shape: Some(nomos_cap_syntax::Function_Shape(arity)),
        },
    );
}

/// The `shape` a struct's own named fields declare, `OD-CAPABILITY-010`'s extension to
/// `nomos.cap.syntax.items`' per-kind vocabulary — this provider's own real second writer of
/// it, alongside `nomos-lang-rust`.
///
/// `type` is the field's own source text, verbatim (`tree-sitter`'s node span sliced
/// straight out of `source`) — unlike `nomos-lang-rust`'s `Type_Head`, this provider has no
/// "no printing" boundary to respect, since it never renders a type back out of a parse
/// tree; it reads bytes that were already there. An embedded field (`Embedded`, no `name`
/// field of its own — Go's field name is then implied by the type) is skipped rather than
/// given an invented name: this reader records fields it observed a real name for, the same
/// restraint `nomos-lang-rust` already takes for a tuple or unit struct's fields.
fn Struct_Shape(struct_type: Node, source: &[u8]) -> Option<String>
{
    let mut cursor = struct_type.walk();
    let list = struct_type
        .children(&mut cursor)
        .find(|child| return child.kind() == "field_declaration_list")?;
    let mut cursor = list.walk();
    let mut fields = Vec::new();

    for declaration in list.children(&mut cursor)
    {
        if declaration.kind() != "field_declaration"
        {
            continue;
        }

        Push_Struct_Field(declaration, source, &mut fields);
    }

    return nomos_cap_syntax::Struct_Shape(&fields);
}

fn Push_Struct_Field(declaration: Node, source: &[u8], fields: &mut Vec<(String, String)>)
{
    let Some(type_node) = declaration.child_by_field_name("type")
    else
    {
        return;
    };
    let Ok(type_text) = type_node.utf8_text(source)
    else
    {
        return;
    };

    for name_node in Named_Field_Children(declaration, "name")
    {
        if let Ok(name_text) = name_node.utf8_text(source)
        {
            fields.push((name_text.to_owned(), type_text.to_owned()));
        }
    }
}

pub(super) fn Record_Type_Alias(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(name) = Function_Name(spec, source)
    else
    {
        return;
    };

    let documentation = Documentation(spec, source);

    Push(
        items,
        ItemRecord {
            kind: ItemKind::TypeAlias,
            scope: Vec::new(),
            visibility: Visibility::Of_Name(&name),
            name,
            documentation,
            shape: None,
        },
    );
}
