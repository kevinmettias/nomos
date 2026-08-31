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
//!
//! Recording is split by declaration kind, one submodule per concern:
//! [`functions`] for `func` and method declarations, [`values`] for `const`/`var`,
//! [`imports`] for `import`, and [`types`] for `struct`/`interface`/type-definition/alias.
//! [`support`] holds what every one of those recorders shares — reading a node's named
//! fields, and turning what was found into one recorded item. This file itself keeps only
//! the entry point, parse-error handling, and the top-level dispatch that hands each
//! declaration node to the recorder that owns its kind.

use super::{ItemKind, Facts, Item};
use crate::ParseFailure;
use crate::Reading;
use tree_sitter::Node;

mod functions;
mod imports;
mod support;
mod types;
mod values;

use functions::{Record_Function, Record_Method};
use types::{Record_Type_Alias, Record_Type_Spec};

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
    let Some(tree) = Parsed_Tree(source)
    else
    {
        return Reading::Unparseable(No_Tree_Failure());
    };

    let root = tree.root_node();

    if let Some(failure) = Root_Error(root)
    {
        return Reading::Unparseable(failure);
    }

    let items = Walk_Source_File(root, source.as_bytes());

    return Reading::Parsed(Facts { items, unexpanded: 0 });
}

fn Parsed_Tree(source: &str) -> Option<tree_sitter::Tree>
{
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .expect("the Go grammar is compiled into this crate");

    return parser.parse(source, None);
}

fn No_Tree_Failure() -> ParseFailure
{
    return ParseFailure {
        line: 0,
        column: 0,
        message: "tree-sitter produced no tree at all".to_owned(),
    };
}

fn Root_Error(root: Node) -> Option<ParseFailure>
{
    if !root.has_error()
    {
        return None;
    }

    return Some(First_Error(root).unwrap_or_else(|| {
        return ParseFailure {
            line: 0,
            column: 0,
            message: "the file is not well-formed Go source".to_owned(),
        };
    }));
}

fn Walk_Source_File(root: Node, source: &[u8]) -> Vec<Item>
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

/// Searches one `_declaration` node's subtree for the spec kinds it can contain, recording
/// each as it is found — so a grouped block (`const ( ... )`) and a single declaration
/// (`const X = 1`) are handled by the same walk, in true source order, without this crate
/// needing to know which declaration kinds `tree-sitter-go` wraps in a list node and which
/// it does not. Stops at `block` and `func_literal`; see the module doc.
fn Record_Declaration_Body(items: &mut Vec<Item>, node: Node, source: &[u8])
{
    use imports::Record_Import_Spec;
    use values::Record_Const_Or_Var_Spec;

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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Reading;

    #[test]
    fn Test_Read_Source_Should_Parse_A_Well_Formed_Go_File()
    {
        let reading = Read_Source("package main\n\nfunc One() {}\n");

        let Reading::Parsed(facts) = reading
        else
        {
            // rust-panic: allow: the fixture above is well-formed Go, so a non-`Parsed`
            // reading is this test's own fixture broken, not a reachable outcome its
            // caller needs to handle — panicking names which reading and points straight
            // at the fixture that regressed.
            panic!("expected a parse: {reading:?}");
        };
        assert_eq!(facts.items.len(), 1);
    }

    #[test]
    fn Test_Read_Source_Should_Refuse_A_File_Tree_Sitter_Could_Not_Parse_Cleanly()
    {
        let reading = Read_Source("func unclosed( {");

        assert!(matches!(reading, Reading::Unparseable(_)), "got {reading:?}");
    }
}
