//! Recording Go `import` declarations.

use super::super::{Documentation_Of_Declaration, ItemKind, Item, Visibility};
use super::support::{ItemRecord, Push_Item_Record};
use tree_sitter::Node;

/// `import "path"` binds the package's own name, as declared at `path`'s own `package`
/// clause — this provider does not read that file, so it falls back to the path's final
/// segment, which is what an unaliased import binds in every real Go source tree. An
/// aliased import (`import x "path"`) states its binding directly and is read from there.
pub(super) fn Record_Import_Spec(items: &mut Vec<Item>, spec: Node, source: &[u8])
{
    let Some(path) = Import_Path(spec, source)
    else
    {
        return;
    };

    let name = Import_Name(spec, source, &path);
    let documentation = Documentation_Of_Declaration(spec, source);

    // An import declares no visibility of its own — Go has no `pub import`, and unlike
    // Rust's `pub use`, an imported name cannot be re-exported at all.
    Push_Item_Record(
        items,
        ItemRecord {
            kind: ItemKind::Import,
            scope: Vec::new(),
            name,
            visibility: Visibility::NotApplicable,
            documentation,
            shape: None,
        },
    );
}

fn Import_Path(spec: Node, source: &[u8]) -> Option<String>
{
    let path_node = spec.child_by_field_name("path")?;
    let raw_path = path_node.utf8_text(source).ok()?;

    return Some(raw_path.trim_matches('"').to_owned());
}

fn Import_Name(spec: Node, source: &[u8], path: &str) -> String
{
    return match spec.child_by_field_name("name")
    {
        Some(alias) => alias.utf8_text(source).unwrap_or(path).to_owned(),
        None => path.rsplit('/').next().unwrap_or(path).to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Record_Import_Spec_Should_Bind_An_Unaliased_Imports_Final_Path_Segment()
    {
        let source = "package main\n\nimport \"path/filepath\"\n";
        let tree = Parsed_Fixture_Tree(source);
        let spec = Find_Kind(tree.root_node(), "import_spec").expect("the fixture declares an import");
        let mut items = Vec::new();

        Record_Import_Spec(&mut items, spec, source.as_bytes());

        let item = items.first().expect("one item recorded");
        assert_eq!(item.name, "filepath");
        assert_eq!(item.visibility, Visibility::NotApplicable);
    }

    #[test]
    fn Test_Record_Import_Spec_Should_Bind_An_Aliased_Imports_Own_Alias()
    {
        let source = "package main\n\nimport str \"strings\"\n";
        let tree = Parsed_Fixture_Tree(source);
        let spec = Find_Kind(tree.root_node(), "import_spec").expect("the fixture declares an import");
        let mut items = Vec::new();

        Record_Import_Spec(&mut items, spec, source.as_bytes());

        let item = items.first().expect("one item recorded");
        assert_eq!(item.name, "str");
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
