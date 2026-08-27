//! Recording Go `import` declarations.

use super::super::{Documentation, ItemKind, SyntaxItem, Visibility};
use super::support::{ItemRecord, Push};
use tree_sitter::Node;

/// `import "path"` binds the package's own name, as declared at `path`'s own `package`
/// clause — this provider does not read that file, so it falls back to the path's final
/// segment, which is what an unaliased import binds in every real Go source tree. An
/// aliased import (`import x "path"`) states its binding directly and is read from there.
pub(super) fn Record_Import_Spec(items: &mut Vec<SyntaxItem>, spec: Node, source: &[u8])
{
    let Some(path) = Import_Path(spec, source)
    else
    {
        return;
    };

    let name = Import_Name(spec, source, &path);
    let documentation = Documentation(spec, source);

    // An import declares no visibility of its own — Go has no `pub import`, and unlike
    // Rust's `pub use`, an imported name cannot be re-exported at all.
    Push(
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
