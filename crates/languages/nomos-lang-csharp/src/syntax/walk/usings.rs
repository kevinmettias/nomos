//! Recording C# `using` directives.

use super::super::ItemKind;
use super::recording::Recording;
use super::site::Site;
use super::support::Push_Unmodifiable_Declaration;
use tree_sitter::Node;

/// Records what a `using` directive binds.
///
/// Four spellings, one item. `using A = X;` states its binding outright and is read from the
/// `name` field. `using X;`, `using static X;` and `global using X;` bind the final segment of the
/// name they import, which is what C# itself binds — this provider resolves nothing, so the final
/// segment is read from the text rather than confirmed against whatever is at `X`.
///
/// A `using` directive declares no accessibility of its own: C# has no `public using`, and unlike
/// Rust's `pub use` a `using` cannot re-export anything at all.
pub(super) fn Record_Using(recording: &mut Recording, site: Site)
{
    let Some(name) = Bound_Name(site.node, site.source)
    else
    {
        return;
    };

    Push_Unmodifiable_Declaration(recording, site, (ItemKind::Using, &name));
}

/// The name this directive binds.
fn Bound_Name(node: Node, source: &[u8]) -> Option<String>
{
    if let Some(alias) = node.child_by_field_name("name")
    {
        return alias.utf8_text(source).ok().map(str::to_owned);
    }

    let imported = Imported_Name(node, source)?;

    return Some(imported.rsplit('.').next().unwrap_or(imported).to_owned());
}

/// The name the directive imports, as written.
///
/// The last named child, because the preceding tokens — `global`, `using`, `static` — are all
/// anonymous in this grammar, and the terminating `;` is too. Reading the last named child is what
/// makes one reader cover all three unaliased spellings.
fn Imported_Name<'source>(node: Node, source: &'source [u8]) -> Option<&'source str>
{
    let mut cursor = node.walk();
    let last = node.children(&mut cursor).filter(|child| return child.is_named()).last()?;

    return last.utf8_text(source).ok();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};
    use crate::Visibility;

    fn Recorded_Using(source: &str) -> (String, Visibility)
    {
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "using_directive").expect("the fixture declares a using directive");
        let mut recording = Recording::New();

        Record_Using(&mut recording, Site::At_Root(node, source.as_bytes()));

        let facts = recording.Into_Facts();
        let item = facts.items.first().expect("one item recorded").clone();

        return (item.name, item.visibility);
    }

    #[test]
    fn Test_An_Unaliased_Using_Should_Bind_The_Final_Segment_Of_What_It_Imports()
    {
        assert_eq!(Recorded_Using("using System.Collections.Generic;\n").0, "Generic");
        assert_eq!(Recorded_Using("using System;\n").0, "System");
    }

    #[test]
    fn Test_A_Static_Or_Global_Using_Should_Read_The_Same_Way()
    {
        assert_eq!(Recorded_Using("using static System.Math;\n").0, "Math");
        assert_eq!(Recorded_Using("global using System.Text;\n").0, "Text");
    }

    #[test]
    fn Test_An_Aliased_Using_Should_Bind_Its_Own_Alias()
    {
        assert_eq!(Recorded_Using("using Widgets = Acme.Widgets;\n").0, "Widgets");
    }

    #[test]
    fn Test_A_Using_Should_Declare_No_Accessibility()
    {
        assert_eq!(Recorded_Using("using System;\n").1, Visibility::NotApplicable);
    }
}
