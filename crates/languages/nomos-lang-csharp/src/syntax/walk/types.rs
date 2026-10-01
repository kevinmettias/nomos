//! Recording C# type declarations: the four forms that share one node shape, plus an enum and its
//! own members, plus a delegate.

use super::super::{Documentation_Of_Declaration, ItemKind, Visibility};
use super::recording::Recording;
use super::site::Site;
use super::support::{Declared_Name, ItemRecord, Modifier_Words, Parameter_Arity, Push_Modifiable_Declaration,
                     Push_Unmodifiable_Declaration};

/// Records a class, struct, interface or record, and then everything it declares inside itself.
///
/// The shape is not reported, and that is a divergence from `nomos-lang-go` worth naming: Go's
/// provider puts a struct's named fields in `shape` because it does not record them as items. C#
/// fields *are* members and are recorded as items of their own, so a `fields` shape here would be
/// the same information twice, in two vocabularies, with nothing keeping them in agreement.
pub(super) fn Record_Type(recording: &mut Recording, site: Site, kind: ItemKind)
{
    let Some(name) = Declared_Name(site.node, site.source)
    else
    {
        return;
    };

    Push_Modifiable_Declaration(recording, site, (kind, &name));
    super::Walk_Body_Of(recording, site, &name);
}

/// Records an enum and the names inside it.
///
/// Its members are recorded rather than folded into the enum's own `shape`, for the reason
/// [`Record_Type`] gives: an enum member is a declaration with a name, and every other named
/// declaration in this file's scope is an item.
pub(super) fn Record_Enum(recording: &mut Recording, site: Site)
{
    let Some(name) = Declared_Name(site.node, site.source)
    else
    {
        return;
    };

    Push_Modifiable_Declaration(recording, site, (ItemKind::Enum, &name));
    super::Walk_Body_Of(recording, site, &name);
}

/// Records one name inside an enum body.
///
/// An enum member cannot carry an accessibility modifier — it is as visible as the enum that
/// declares it — so [`Visibility::NotApplicable`] is the mark, not [`Visibility::Unspecified`].
pub(super) fn Record_Enum_Member(recording: &mut Recording, site: Site)
{
    let Some(name) = Declared_Name(site.node, site.source)
    else
    {
        return;
    };

    Push_Unmodifiable_Declaration(recording, site, (ItemKind::EnumMember, &name));
}

/// Records a delegate declaration.
///
/// A named function type, so its shape is the one every callable in this payload carries: the
/// declared parameter count. C# is the first language this workspace reads that declares a
/// function type as a top-level named form of its own.
pub(super) fn Record_Delegate(recording: &mut Recording, site: Site)
{
    let Some(name) = Declared_Name(site.node, site.source)
    else
    {
        return;
    };

    let arity = site.node.child_by_field_name("parameters").map_or(0, Parameter_Arity);

    recording.Push(ItemRecord {
        kind: ItemKind::Delegate,
        scope: site.scope.to_vec(),
        name,
        visibility: Visibility::Of_Modifiers(&Modifier_Words(site.node, site.source)),
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: Some(nomos_cap_syntax::Function_Shape(arity)),
    });
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};
    use crate::Facts;

    /// Records `source`'s first node of `node_kind` through `record`, and hands back what it
    /// produced.
    fn Recorded(source: &str, node_kind: &str, record: fn(&mut Recording, Site)) -> Facts
    {
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), node_kind).expect("the fixture declares the requested node kind");
        let mut recording = Recording::New();

        record(&mut recording, Site::At_Root(node, source.as_bytes()));

        return recording.Into_Facts();
    }

    /// The same, for the one recorder that also takes a kind.
    fn Recorded_Type(source: &str, kind: ItemKind) -> Facts
    {
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "class_declaration").expect("the fixture declares a class");
        let mut recording = Recording::New();

        Record_Type(&mut recording, Site::At_Root(node, source.as_bytes()), kind);

        return recording.Into_Facts();
    }

    #[test]
    fn Test_Record_Type_Should_Record_A_Nested_Type_Under_Its_Enclosing_One()
    {
        let facts = Recorded_Type("public class Outer { private class Inner { } }\n", ItemKind::Class);

        let inner = facts.items.get(1).expect("the outer and inner classes are both recorded");
        assert_eq!(inner.Qualified_Name(), "Outer::Inner");
        assert_eq!(inner.visibility, Visibility::Private);
    }

    #[test]
    fn Test_Record_Enum_Should_Record_Every_Member_Under_The_Enum()
    {
        let facts = Recorded("public enum Color { Red, Green }\n", "enum_declaration", Record_Enum);

        let names: Vec<String> = facts.items.iter().map(|item| return item.Qualified_Name()).collect();
        assert_eq!(names, vec!["Color".to_owned(), "Color::Red".to_owned(), "Color::Green".to_owned()]);
    }

    #[test]
    fn Test_An_Enum_Member_Should_Declare_No_Accessibility()
    {
        let facts = Recorded("enum Color { Red }\n", "enum_member_declaration", Record_Enum_Member);

        let member = facts.items.first().expect("one item recorded");
        assert_eq!(member.visibility, Visibility::NotApplicable);
    }

    #[test]
    fn Test_Record_Delegate_Should_Carry_Its_Declared_Parameter_Count()
    {
        let facts = Recorded(
            "public delegate void Handler(object sender, int code);\n",
            "delegate_declaration",
            Record_Delegate,
        );

        let item = facts.items.first().expect("one item recorded");
        assert_eq!(item.name, "Handler");
        assert_eq!(item.shape.as_deref(), Some("fn/2"));
    }

    /// A generic type is the same node with a type-parameter list added, which is the claim
    /// `crate::Declared_Guarantee`'s completeness reasoning rests on.
    #[test]
    fn Test_A_Generic_Type_Should_Be_Recorded_Under_Its_Bare_Name()
    {
        let facts = Recorded_Type("public class Box<TItem> where TItem : class { }\n", ItemKind::Class);

        let item = facts.items.first().expect("one item recorded");
        assert_eq!(item.name, "Box");
    }
}
