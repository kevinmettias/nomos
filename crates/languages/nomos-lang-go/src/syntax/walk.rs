//! One pass over a parsed file, recording every package-level declaration it meets.
//!
//! Not a generic recursive descent over every node. The entry point walks `source_file`'s
//! direct children — each one a `_declaration` node — and each declaration's own recorder
//! searches only its own subtree for the leaf spec kinds it can contain
//! (`const_spec`, `var_spec`, `import_spec`, `type_spec`, `type_alias`), stopping at a
//! `block` or `func_literal` wherever one is met. That stop is load-bearing: a value like
//! `var Handler = func() { var scratch int }` embeds a `var_declaration` for `scratch`
//! inside the outer `var_declaration`'s own subtree, and `scratch` is a local, not a
//! package-level item — the same "not statements" boundary `nomos-lang-rust` states for
//! Rust's local bindings.

use super::{Documentation, ItemKind, SyntaxFacts, SyntaxItem, Visibility};
use crate::ParseFailure;
use crate::Reading;
use tree_sitter::Node;

/// Parses `source` and records every package-level declaration it finds.
///
/// Any parse error refuses the whole file rather than reporting the part before it —
/// `tree-sitter` does error-recovery parsing and will hand back a tree for genuinely broken
/// input, and a recovered tree's nodes near the error are not a fact about the source, they
/// are the parser's guess. Reading them as facts would be unsound, and this provider claims
/// [`nomos_contracts::Assurance::Sound`]. See [`crate::Declared_Guarantee`].
///
/// # Panics
///
/// Never, in practice: the only panic path is the compiled-in `tree-sitter-go` grammar
/// failing its own version check against the linked `tree-sitter` runtime, which is a build
/// configuration defect this crate's own `Cargo.toml` pins against, not a runtime input.
#[must_use]
pub fn Read_Source(source: &str) -> Reading
{
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .expect("the Go grammar is compiled into this crate");

    let Some(tree) = parser.parse(source, None)
    else
    {
        return Reading::Unparseable(ParseFailure {
            line: 0,
            column: 0,
            message: "tree-sitter produced no tree at all".to_owned(),
        });
    };

    let root = tree.root_node();

    if root.has_error()
    {
        let failure = First_Error(root).unwrap_or_else(|| {
            return ParseFailure {
                line: 0,
                column: 0,
                message: "the file is not well-formed Go source".to_owned(),
            };
        });

        return Reading::Unparseable(failure);
    }

    let items = Walk_Source_File(root, source.as_bytes());

    return Reading::Parsed(SyntaxFacts { items, unexpanded: 0 });
}

/// The first place recovery left a mark, depth-first in source order.
fn First_Error(node: Node) -> Option<ParseFailure>
{
    if node.is_error() || node.is_missing()
    {
        let point = node.start_position();

        return Some(ParseFailure {
            line: point.row.saturating_add(1),
            column: point.column.saturating_add(1),
            message: format!("unexpected {}", node.kind()),
        });
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor)
    {
        if let Some(found) = First_Error(child)
        {
            return Some(found);
        }
    }

    return None;
}

fn Walk_Source_File(root: Node, source: &[u8]) -> Vec<SyntaxItem>
{
    let mut items = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor)
    {
        match child.kind()
        {
            "function_declaration" => Record_Function(&mut items, child, source),
            "method_declaration" => Record_Method(&mut items, child, source),
            "const_declaration" | "var_declaration" | "import_declaration" | "type_declaration" =>
            {
                Record_Declaration_Body(&mut items, child, source);
            }
            _ =>
            {}
        }
    }

    return items;
}

/// Searches one `_declaration` node's subtree for the spec kinds it can contain, recording
/// each as it is found — so a grouped block (`const ( ... )`) and a single declaration
/// (`const X = 1`) are handled by the same walk, in true source order, without this crate
/// needing to know which declaration kinds `tree-sitter-go` wraps in a list node and which
/// it does not. Stops at `block` and `func_literal`; see the module doc.
fn Record_Declaration_Body(items: &mut Vec<SyntaxItem>, node: Node, source: &[u8])
{
    let mut cursor = node.walk();

    for child in node.children(&mut cursor)
    {
        match child.kind()
        {
            "const_spec" => Record_Const_Or_Var_Spec(items, child, source, ItemKind::Constant),
            "var_spec" => Record_Const_Or_Var_Spec(items, child, source, ItemKind::Variable),
            "import_spec" => Record_Import_Spec(items, child, source),
            "type_spec" => Record_Type_Spec(items, child, source),
            "type_alias" => Record_Type_Alias(items, child, source),
            "comment" | "block" | "func_literal" =>
            {}
            _ => Record_Declaration_Body(items, child, source),
        }
    }
}

fn Record_Function(items: &mut Vec<SyntaxItem>, node: Node, source: &[u8])
{
    let Some(name_node) = node.child_by_field_name("name")
    else
    {
        return;
    };
    let Ok(name) = name_node.utf8_text(source)
    else
    {
        return;
    };

    let arity = node.child_by_field_name("parameters").map_or(0, Parameter_Arity);

    Push(
        items,
        ItemKind::Function,
        Vec::new(),
        name.to_owned(),
        Visibility::Of_Name(name),
        Documentation(node, source),
        Some(nomos_cap_syntax::Function_Shape(arity)),
    );
}

fn Record_Method(items: &mut Vec<SyntaxItem>, node: Node, source: &[u8])
{
    let Some(name_node) = node.child_by_field_name("name")
    else
    {
        return;
    };
    let Ok(name) = name_node.utf8_text(source)
    else
    {
        return;
    };

    let receiver = node.child_by_field_name("receiver");
    let scope = receiver
        .and_then(|found| return Receiver_Type_Name(found, source))
        .map_or_else(Vec::new, |type_name| return vec![type_name]);
    let receiver_arity = usize::from(receiver.is_some());
    let parameter_arity = node.child_by_field_name("parameters").map_or(0, Parameter_Arity);

    Push(
        items,
        ItemKind::Function,
        scope,
        name.to_owned(),
        Visibility::Of_Name(name),
        Documentation(node, source),
        Some(nomos_cap_syntax::Function_Shape(receiver_arity.saturating_add(parameter_arity))),
    );
}

fn Record_Const_Or_Var_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8], kind: ItemKind)
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
            kind,
            Vec::new(),
            name.to_owned(),
            Visibility::Of_Name(name),
            documentation.clone(),
            shape.clone(),
        );
    }
}

/// `import "path"` binds the package's own name, as declared at `path`'s own `package`
/// clause — this provider does not read that file, so it falls back to the path's final
/// segment, which is what an unaliased import binds in every real Go source tree. An
/// aliased import (`import x "path"`) states its binding directly and is read from there.
fn Record_Import_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(path_node) = spec.child_by_field_name("path")
    else
    {
        return;
    };
    let Ok(raw_path) = path_node.utf8_text(source)
    else
    {
        return;
    };
    let path = raw_path.trim_matches('"');

    let name = match spec.child_by_field_name("name")
    {
        Some(alias) => alias.utf8_text(source).unwrap_or(path).to_owned(),
        None => path.rsplit('/').next().unwrap_or(path).to_owned(),
    };

    // An import declares no visibility of its own — Go has no `pub import`, and unlike
    // Rust's `pub use`, an imported name cannot be re-exported at all.
    Push(
        items,
        ItemKind::Import,
        Vec::new(),
        name,
        Visibility::NotApplicable,
        Documentation(spec, source),
        None,
    );
}

fn Record_Type_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(name_node) = spec.child_by_field_name("name")
    else
    {
        return;
    };
    let Ok(name) = name_node.utf8_text(source)
    else
    {
        return;
    };

    let type_node = spec.child_by_field_name("type");
    let kind = match type_node.map(|found| return found.kind())
    {
        Some("struct_type") => ItemKind::Struct,
        Some("interface_type") => ItemKind::Interface,
        _ => ItemKind::TypeDefinition,
    };

    Push(
        items,
        kind,
        Vec::new(),
        name.to_owned(),
        Visibility::Of_Name(name),
        Documentation(spec, source),
        None,
    );

    if let (ItemKind::Interface, Some(interface)) = (kind, type_node)
    {
        Record_Interface_Methods(items, interface, name, source);
    }
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

        let Some(name_node) = member.child_by_field_name("name")
        else
        {
            continue;
        };
        let Ok(name) = name_node.utf8_text(source)
        else
        {
            continue;
        };

        let arity = member.child_by_field_name("parameters").map_or(0, Parameter_Arity);

        Push(
            items,
            ItemKind::Function,
            vec![interface_name.to_owned()],
            name.to_owned(),
            Visibility::Of_Name(name),
            Documentation(member, source),
            Some(nomos_cap_syntax::Function_Shape(arity)),
        );
    }
}

fn Record_Type_Alias(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(name_node) = spec.child_by_field_name("name")
    else
    {
        return;
    };
    let Ok(name) = name_node.utf8_text(source)
    else
    {
        return;
    };

    Push(
        items,
        ItemKind::TypeAlias,
        Vec::new(),
        name.to_owned(),
        Visibility::Of_Name(name),
        Documentation(spec, source),
        None,
    );
}

/// How many parameters a `parameter_list` declares.
///
/// A shared type binds several names to one `parameter_declaration` (`func f(a, b int)`),
/// so this counts `name` fields rather than declaration nodes — a declaration with no name
/// at all (`func f(int)`) still declares one unnamed parameter and counts as one.
fn Parameter_Arity(list: Node) -> usize
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

/// Every child carrying `field`, as an actually-named node.
///
/// Not `Node::children_by_field_name`: in `tree-sitter-go`'s real grammar, a repeated field
/// is not always contiguous with itself — `const A, B = 1, 2` tags the separating `,` with
/// the same `name` field its neighbours carry, and the convenience iterator hands it back
/// along with them. Confirmed against the real grammar rather than assumed, after that
/// token showed up as a third "name". Filtering to named nodes is what a hand-walked
/// `TreeCursor` buys back: the grammar's own token/rule distinction, which
/// `is_named` reads directly rather than re-deriving from `kind()` strings.
fn Named_Field_Children<'a>(node: Node<'a>, field: &str) -> Vec<Node<'a>>
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

/// Records one declaration, with what this provider observed about it.
///
/// `shape` is `None` where the form has no shape to describe rather than where none could be
/// seen. This provider parses, so everything it does not record is an absence it looked for
/// — the distinction the payload spells `.` rather than `-`.
fn Push(
    items: &mut Vec<SyntaxItem>,
    kind: ItemKind,
    scope: Vec<String>,
    name: String,
    visibility: Visibility,
    documentation: Option<String>,
    shape: Option<String>,
)
{
    let ordinal = u32::try_from(items.len()).unwrap_or(u32::MAX);

    items.push(SyntaxItem { ordinal, kind, scope, name, visibility, documentation, shape });
}
