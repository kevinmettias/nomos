//! Recording C#'s two namespace forms.
//!
//! The block form (`namespace X { ... }`) encloses its declarations the way every other container
//! in this grammar does. The file-scoped form (`namespace X;`) does not: it has no body, and
//! everything after it in the file belongs to it. Both are recorded as one
//! [`ItemKind::Namespace`] item, and the difference shows up only in who walks what.

use super::super::{Documentation_Of_Declaration, ItemKind, Visibility};
use super::recording::Recording;
use super::site::Site;
use super::support::{ItemRecord, Nested_Scope};
use tree_sitter::Node;

/// The node kind whose declarations follow it as siblings rather than sitting inside it.
const FILE_SCOPED: &str = "file_scoped_namespace_declaration";

/// Records a block namespace and everything it encloses.
pub(super) fn Record_Namespace(recording: &mut Recording, site: Site)
{
    let Some(name) = Namespace_Name(site.node, site.source)
    else
    {
        return;
    };

    Push_Namespace(recording, site, &name);
    super::Walk_Body_Of(recording, site, &name);
}

/// Records a file-scoped namespace and hands back the scope its siblings now sit in, or `None`
/// when `site` is not one.
///
/// Returning the scope rather than walking anything is what the form requires: there is no body to
/// descend into, so the caller's own loop has to carry the change forward.
pub(super) fn Entered_File_Scoped_Namespace(recording: &mut Recording, site: Site) -> Option<Vec<String>>
{
    if site.node.kind() != FILE_SCOPED
    {
        return None;
    }

    let name = Namespace_Name(site.node, site.source)?;
    Push_Namespace(recording, site, &name);

    return Some(Nested_Scope(site.scope, &name));
}

/// The namespace's name exactly as written, dots included.
///
/// `namespace Acme.Widgets` stays one segment rather than becoming two. C# permits either spelling
/// for the same nesting, and splitting the dotted form would report a nesting the file does not
/// have while the nested form reported the nesting it does — two different answers for what the
/// language treats as one declaration.
fn Namespace_Name(node: Node, source: &[u8]) -> Option<String>
{
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;

    return Some(name.to_owned());
}

/// A namespace carries no accessibility: C# has no modifier a `namespace` may take, so
/// [`Visibility::NotApplicable`] is the honest mark rather than "stated nothing".
fn Push_Namespace(recording: &mut Recording, site: Site, name: &str)
{
    recording.Push(ItemRecord {
        kind: ItemKind::Namespace,
        scope: site.scope.to_vec(),
        name: name.to_owned(),
        visibility: Visibility::NotApplicable,
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: None,
    });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};

    #[test]
    fn Test_Record_Namespace_Should_Scope_What_It_Encloses()
    {
        let source = "namespace Acme.Widgets { public class Widget { } }\n";
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "namespace_declaration").expect("the fixture declares a namespace");
        let mut recording = Recording::New();

        Record_Namespace(&mut recording, Site::At_Root(node, source.as_bytes()));

        let facts = recording.Into_Facts();
        let widget = facts.items.get(1).expect("the namespace and the class are both recorded");
        assert_eq!(widget.Qualified_Name(), "Acme.Widgets::Widget");
    }

    #[test]
    fn Test_A_Node_That_Is_Not_File_Scoped_Should_Not_Be_Entered()
    {
        let source = "namespace Acme { }\n";
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "namespace_declaration").expect("the fixture declares a namespace");
        let mut recording = Recording::New();

        let entered = Entered_File_Scoped_Namespace(&mut recording, Site::At_Root(node, source.as_bytes()));

        assert_eq!(entered, None);
        assert!(recording.Into_Facts().Has_No_Declarations(), "a refused node must record nothing");
    }

    #[test]
    fn Test_A_File_Scoped_Namespace_Should_Hand_Back_The_Scope_Its_Siblings_Sit_In()
    {
        let source = "namespace Acme.Core;\n";
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), FILE_SCOPED).expect("the fixture declares a file-scoped namespace");
        let mut recording = Recording::New();

        let entered = Entered_File_Scoped_Namespace(&mut recording, Site::At_Root(node, source.as_bytes()));

        assert_eq!(entered, Some(vec!["Acme.Core".to_owned()]));
    }
}
