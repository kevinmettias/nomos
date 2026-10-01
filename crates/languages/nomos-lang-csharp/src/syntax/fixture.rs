//! The two helpers every test module under [`super`] needs: a parsed fixture, and the first
//! node of a given kind inside it.
//!
//! Declared once rather than copied into each test module. `nomos-lang-go`'s own walk repeats
//! them five times, and a fixture helper that disagrees with its four copies about which node
//! a test is looking at is a test measuring a different subject than its name claims.

use tree_sitter::Node;

/// The tree a fixture's own source parses to.
///
/// Panics rather than returning a result: every caller writes C# it intends to be well formed,
/// so a parser that refuses it is a broken fixture and not an outcome a test needs to handle.
pub(super) fn Parsed_Fixture_Tree(source: &str) -> tree_sitter::Tree
{
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
        .expect("the C# grammar is compiled into this crate");

    return parser.parse(source, None).expect("well-formed fixture source parses");
}

/// The first node of `kind` in `node`'s subtree, depth-first in source order.
pub(super) fn Find_Kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>>
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
