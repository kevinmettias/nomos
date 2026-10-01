//! One pass over a parsed file, recording every declaration it meets.
//!
//! Not a generic recursive descent over every node. The entry point walks the
//! `compilation_unit`'s direct children and each declaration that has a body of its own walks
//! that body, so nesting is followed exactly as far as declarations nest and no further: a
//! statement block, an expression, a lambda and a property's accessor list are never entered.
//! That boundary is load-bearing, and for the same reason `nomos-lang-go` states for `block`
//! and `func_literal`: a local function inside a method body is not a member declaration, and
//! recording it would report a local as part of the type's surface.
//!
//! # The one node kind this walk refuses to enter
//!
//! A `preproc_if` subtree carries *every* branch of a `#if`/`#elif`/`#else` chain at once, and
//! at most one branch is in any compilation. [`Read_Source`] counts the region and reads
//! nothing inside it; [`crate::Declared_Guarantee`] carries the full reasoning and the two
//! guarantee axes that choice decides.
//!
//! Recording is split by declaration category, one submodule per concern: [`namespaces`] for
//! both namespace forms, [`usings`] for `using` directives, [`types`] for the six type forms
//! and an enum's own members, and [`members`] for what a type declares inside itself.
//! [`support`] holds what every one of those shares. This file keeps only the entry point,
//! parse-error handling, and the dispatch that hands each node to the recorder owning its kind.

use super::ItemKind;
use crate::ParseFailure;
use crate::Reading;
use recording::Recording;
use site::Site;
use support::Nested_Scope;
use tree_sitter::Node;

mod members;
mod namespaces;
mod recording;
mod site;
mod support;
mod types;
mod usings;

/// Parses `source` and records every declaration it finds.
///
/// Any parse error refuses the whole file rather than reporting the part before it —
/// `tree-sitter` does error-recovery parsing and will hand back a tree for genuinely broken
/// input, and a recovered tree's nodes near the error are not a fact about the source, they are
/// the parser's guess. Reading them as facts would be unsound, and this provider claims
/// [`nomos_contracts::Assurance::Sound`] on that axis. See [`crate::Declared_Guarantee`].
///
/// Panics nowhere: both ways the parse itself can fail — a runtime that will not install the
/// compiled-in grammar, and a parser that hands back no tree — reach the caller as
/// [`Reading::Unparseable`], so a build configuration defect is reported as an unreadable file
/// rather than as a crash this crate's own `Cargo.toml` pins against.
#[must_use]
pub fn Read_Source(source: &str) -> Reading
{
    let tree = match Parsed_Tree(source)
    {
        Ok(tree) => tree,
        Err(failure) => return Reading::Unparseable(failure),
    };

    let root = tree.root_node();

    if let Some(failure) = Root_Error(root)
    {
        return Reading::Unparseable(failure);
    }

    let mut recording = Recording::New();
    Walk_Declarations(&mut recording, Site::At_Root(root, source.as_bytes()));

    return Reading::Parsed(recording.Into_Facts());
}

/// Parses `source` with the compiled-in C# grammar.
///
/// Both ways this can fail are returned rather than raised: a linked `tree-sitter` runtime that
/// will not install the compiled-in grammar, which is a build configuration defect this crate's
/// `Cargo.toml` pins against rather than a runtime input, and a parser that hands back no tree
/// at all. Neither is a fact about `source`, so neither gets to decide the process's fate.
fn Parsed_Tree(source: &str) -> Result<tree_sitter::Tree, ParseFailure>
{
    let mut parser = tree_sitter::Parser::new();

    if let Err(error) = parser.set_language(&tree_sitter_c_sharp::LANGUAGE.into())
    {
        return Err(ParseFailure {
            line: 0,
            column: 0,
            message: format!("the linked tree-sitter runtime refused the compiled-in C# grammar: {error}"),
        });
    }

    let Some(tree) = parser.parse(source, None)
    else
    {
        return Err(No_Tree_Failure());
    };

    return Ok(tree);
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
            message: "the file is not well-formed C# source".to_owned(),
        };
    }));
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

/// Records every declaration among one list's direct children.
///
/// The scope is a `let mut` rather than the site's own, because of one C# form: a file-scoped
/// namespace (`namespace X;`) has no body, and every declaration after it in the file belongs to
/// it. So entering one changes the scope the *remaining siblings* are recorded in, which is a
/// thing no declaration in Rust or Go does.
fn Walk_Declarations(recording: &mut Recording, list: Site)
{
    let mut cursor = list.node.walk();
    let mut current = list.scope.to_vec();

    for child in list.node.children(&mut cursor)
    {
        let site = list.Moved_To(child, &current);

        if let Some(entered) = namespaces::Entered_File_Scoped_Namespace(recording, site)
        {
            current = entered;
            continue;
        }

        Record_One(recording, site);
    }
}

/// Walks whatever body `site`'s own declaration has, in the scope its name opens.
///
/// Shared by every container form — a namespace, a type, an enum — because all three nest their
/// members under a `body` field and all three qualify them by their own name.
fn Walk_Body_Of(recording: &mut Recording, site: Site, name: &str)
{
    let Some(body) = site.node.child_by_field_name("body")
    else
    {
        return;
    };

    let inner = Nested_Scope(site.scope, name);
    Walk_Declarations(recording, site.Moved_To(body, &inner));
}

/// One node, handed to whichever recorder owns its kind — or to none of them.
fn Record_One(recording: &mut Recording, site: Site)
{
    if site.node.kind() == CONDITIONAL_REGION
    {
        recording.Note_Conditional_Region();
        return;
    }

    if let Some(kind) = Type_Kind_Of(site.node.kind())
    {
        types::Record_Type(recording, site, kind);
        return;
    }

    if let Some(kind) = Member_Kind_Of(site.node.kind())
    {
        members::Record_Member(recording, site, kind);
        return;
    }

    Record_Remaining(recording, site);
}

/// The node kind whose subtree this walk will not enter.
const CONDITIONAL_REGION: &str = "preproc_if";

/// The type forms whose node shape is the same: a name, modifiers, and a body of members.
///
/// `record`, `record class` and `record struct` all parse as `record_declaration`, so one arm
/// covers the three — the difference between them is what the compiler generates rather than
/// anything the file declares differently.
fn Type_Kind_Of(kind: &str) -> Option<ItemKind>
{
    return match kind
    {
        "class_declaration" => Some(ItemKind::Class),
        "struct_declaration" => Some(ItemKind::Struct),
        "interface_declaration" => Some(ItemKind::Interface),
        "record_declaration" => Some(ItemKind::Record),
        _ => None,
    };
}

/// The member forms a type declares, other than a field, an event or an enum member, each of
/// which needs a reader of its own.
fn Member_Kind_Of(kind: &str) -> Option<ItemKind>
{
    return match kind
    {
        "method_declaration" => Some(ItemKind::Function),
        "constructor_declaration" => Some(ItemKind::Constructor),
        "destructor_declaration" => Some(ItemKind::Finalizer),
        "operator_declaration" | "conversion_operator_declaration" => Some(ItemKind::Operator),
        "indexer_declaration" => Some(ItemKind::Indexer),
        "property_declaration" => Some(ItemKind::Property),
        _ => None,
    };
}

/// Everything whose reader takes no kind because its kind is fixed by the node.
///
/// Anything not named here is passed over: a comment, a `#region`, a `#pragma`, a `#nullable`, a
/// `#define`, an `extern alias`, an assembly-level attribute list, and a top-level statement —
/// none of which declares a named member, and the last of which is a statement by name.
fn Record_Remaining(recording: &mut Recording, site: Site)
{
    match site.node.kind()
    {
        "using_directive" => usings::Record_Using(recording, site),
        "namespace_declaration" => namespaces::Record_Namespace(recording, site),
        "enum_declaration" => types::Record_Enum(recording, site),
        "enum_member_declaration" => types::Record_Enum_Member(recording, site),
        "delegate_declaration" => types::Record_Delegate(recording, site),
        "field_declaration" => members::Record_Field(recording, site),
        "event_field_declaration" => members::Record_Event_Field(recording, site),
        "event_declaration" => members::Record_Event(recording, site),
        _ =>
        {}
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Reading;

    #[test]
    fn Test_Read_Source_Should_Parse_A_Well_Formed_Csharp_File()
    {
        let reading = Read_Source("public class Widget { }\n");

        let Reading::Parsed(facts) = reading
        else
        {
            // rust-panic: allow: the fixture above is well-formed C#, so a non-`Parsed`
            // reading is this test's own fixture broken, not a reachable outcome its caller
            // needs to handle — panicking names which reading and points straight at the
            // fixture that regressed.
            panic!("expected a parse: {reading:?}");
        };
        assert_eq!(facts.items.len(), 1);
    }

    #[test]
    fn Test_Read_Source_Should_Refuse_A_File_Tree_Sitter_Could_Not_Parse_Cleanly()
    {
        let reading = Read_Source("public class Unclosed {");

        assert!(matches!(reading, Reading::Unparseable(_)), "got {reading:?}");
    }
}
