//! Recording what a C# type declares inside itself: the six callable-or-typed member forms that
//! share one reader, and the three variable-shaped ones that do not.

use super::super::{Documentation_Of_Declaration, ItemKind, Visibility};
use super::recording::Recording;
use super::site::Site;
use super::support::{Declared_Name, ItemRecord, Modifier_Words, Parameter_Arity};
use tree_sitter::Node;

/// The name an indexer declares.
///
/// C# spells an indexer `this[...]`, so it binds no identifier of its own. `this` is what the file
/// says, and inventing a name like `Item` would be reporting the property the compiler generates
/// rather than the declaration the file contains.
const INDEXER_NAME: &str = "this";

/// Records a method, constructor, finalizer, operator, indexer or property.
pub(super) fn Record_Member(recording: &mut Recording, site: Site, kind: ItemKind)
{
    let Some(name) = Member_Name(site.node, kind, site.source)
    else
    {
        return;
    };

    recording.Push(ItemRecord {
        kind,
        scope: site.scope.to_vec(),
        name,
        visibility: Member_Visibility(site.node, kind, site.source),
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: Member_Shape(site.node, kind),
    });
}

/// Records every field one `field_declaration` declares.
pub(super) fn Record_Field(recording: &mut Recording, site: Site)
{
    Record_Variable_Declaration(recording, site, ItemKind::Field);
}

/// Records every event one `event EventHandler A, B;` declares.
pub(super) fn Record_Event_Field(recording: &mut Recording, site: Site)
{
    Record_Variable_Declaration(recording, site, ItemKind::Event);
}

/// Records an event declared with its own `add`/`remove` accessors.
///
/// A different node kind from the field form above, with the name on the declaration itself rather
/// than inside a declarator list. The accessors are not entered: an `add` block is a statement
/// body, which is outside this crate's stated scope.
pub(super) fn Record_Event(recording: &mut Recording, site: Site)
{
    let Some(name) = Declared_Name(site.node, site.source)
    else
    {
        return;
    };

    recording.Push(ItemRecord {
        kind: ItemKind::Event,
        scope: site.scope.to_vec(),
        name,
        visibility: Visibility::Of_Modifiers(&Modifier_Words(site.node, site.source)),
        documentation: Documentation_Of_Declaration(site.node, site.source),
        shape: site.node.child_by_field_name("type").map(Type_Shape),
    });
}

/// Records one item per name a `field_declaration` or `event_field_declaration` binds.
///
/// C# permits several names against one type (`private int first, second;`), so this is one
/// declaration and two items — the same unfolding `nomos-lang-go` performs for a `const` or `var`
/// spec binding several names.
fn Record_Variable_Declaration(recording: &mut Recording, site: Site, kind: ItemKind)
{
    let Some(declaration) = Variable_Declaration(site.node)
    else
    {
        return;
    };

    let shape = declaration.child_by_field_name("type").map(Type_Shape);
    let visibility = Visibility::Of_Modifiers(&Modifier_Words(site.node, site.source));
    let documentation = Documentation_Of_Declaration(site.node, site.source);

    for name in Declarator_Names(declaration, site.source)
    {
        recording.Push(ItemRecord {
            kind,
            scope: site.scope.to_vec(),
            name,
            visibility,
            documentation: documentation.clone(),
            shape: shape.clone(),
        });
    }
}

/// The `variable_declaration` inside a field or event-field declaration.
fn Variable_Declaration(node: Node) -> Option<Node>
{
    let mut cursor = node.walk();

    return node
        .children(&mut cursor)
        .find(|child| return child.kind() == "variable_declaration");
}

/// Every name a `variable_declaration` binds, in source order.
fn Declarator_Names(declaration: Node, source: &[u8]) -> Vec<String>
{
    let mut cursor = declaration.walk();
    let mut names = Vec::new();

    for child in declaration.children(&mut cursor)
    {
        if child.kind() != "variable_declarator"
        {
            continue;
        }

        names.extend(Declared_Name(child, source));
    }

    return names;
}

/// The name a member declares, for the three forms whose name is not a `name` field.
fn Member_Name(node: Node, kind: ItemKind, source: &[u8]) -> Option<String>
{
    return match kind
    {
        ItemKind::Indexer => Some(INDEXER_NAME.to_owned()),
        ItemKind::Operator => Operator_Name(node, source),
        _ => Declared_Name(node, source),
    };
}

/// An operator's own name: the operator token it overloads, or — for a conversion operator, which
/// overloads no token — the type it converts to.
fn Operator_Name(node: Node, source: &[u8]) -> Option<String>
{
    let named = node
        .child_by_field_name("operator")
        .or_else(|| return node.child_by_field_name("type"))?;

    return named.utf8_text(source).ok().map(str::to_owned);
}

/// A finalizer cannot carry an accessibility modifier in C# at all, so there is no unstated default
/// for one either; every other member form can and may simply not have.
fn Member_Visibility(node: Node, kind: ItemKind, source: &[u8]) -> Visibility
{
    if kind == ItemKind::Finalizer
    {
        return Visibility::NotApplicable;
    }

    return Visibility::Of_Modifiers(&Modifier_Words(node, source));
}

/// A member's shape: a property is a typed declaration, and every other form this reader handles is
/// a callable.
fn Member_Shape(node: Node, kind: ItemKind) -> Option<String>
{
    if kind == ItemKind::Property
    {
        return node.child_by_field_name("type").map(Type_Shape);
    }

    let arity = node.child_by_field_name("parameters").map_or(0, Parameter_Arity);

    return Some(nomos_cap_syntax::Function_Shape(arity));
}

/// The shape a declared type has, in the payload's vocabulary — [`nomos_cap_syntax::SLICE`] for an
/// array however many nullable or pointer wrappers deep, and [`nomos_cap_syntax::VALUE`] otherwise.
/// The one distinction a consumer cannot recover from the rest of the record, mirroring
/// `nomos-lang-rust`'s and `nomos-lang-go`'s own `Type_Shape`.
///
/// A `List<int>` is [`nomos_cap_syntax::VALUE`] and not `SLICE`: it is a generic type whose name
/// happens to mean a sequence, and deciding otherwise would require resolving the name, which this
/// provider does not do.
fn Type_Shape(node: Node) -> String
{
    return match node.kind()
    {
        "array_type" => nomos_cap_syntax::SLICE.to_owned(),
        "nullable_type" | "pointer_type" => node
            .child_by_field_name("type")
            .map_or_else(|| return nomos_cap_syntax::VALUE.to_owned(), Type_Shape),
        _ => nomos_cap_syntax::VALUE.to_owned(),
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::syntax::fixture::{Find_Kind, Parsed_Fixture_Tree};
    use crate::{Facts, Item};

    fn Recorded_Member(source: &str, node_kind: &str, kind: ItemKind) -> Item
    {
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), node_kind).expect("the fixture declares the requested node kind");
        let mut recording = Recording::New();

        Record_Member(&mut recording, Site::At_Root(node, source.as_bytes()), kind);

        return recording.Into_Facts().items.first().expect("one item recorded").clone();
    }

    fn Recorded_Fields(source: &str) -> Facts
    {
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "field_declaration").expect("the fixture declares a field");
        let mut recording = Recording::New();

        Record_Field(&mut recording, Site::At_Root(node, source.as_bytes()));

        return recording.Into_Facts();
    }

    #[test]
    fn Test_A_Method_Should_Carry_Its_Declared_Parameter_Count()
    {
        let item = Recorded_Member(
            "class Widget { public static int Add(int a, int b) => a + b; }\n",
            "method_declaration",
            ItemKind::Function,
        );

        assert_eq!(item.name, "Add");
        assert_eq!(item.shape.as_deref(), Some("fn/2"));
        assert_eq!(item.visibility, Visibility::Public);
    }

    #[test]
    fn Test_A_Method_With_No_Accessibility_Word_Should_Be_Unspecified_Rather_Than_Private()
    {
        let item = Recorded_Member("class Widget { void Reset() { } }\n", "method_declaration", ItemKind::Function);

        assert_eq!(item.visibility, Visibility::Unspecified);
    }

    #[test]
    fn Test_A_Finalizer_Should_Declare_No_Accessibility()
    {
        let item = Recorded_Member("class Widget { ~Widget() { } }\n", "destructor_declaration", ItemKind::Finalizer);

        assert_eq!(item.name, "Widget");
        assert_eq!(item.visibility, Visibility::NotApplicable);
    }

    #[test]
    fn Test_An_Operator_Should_Be_Named_For_The_Token_It_Overloads()
    {
        let item = Recorded_Member(
            "class Widget { public static Widget operator +(Widget a, Widget b) => a; }\n",
            "operator_declaration",
            ItemKind::Operator,
        );

        assert_eq!(item.name, "+");
        assert_eq!(item.shape.as_deref(), Some("fn/2"));
    }

    #[test]
    fn Test_A_Conversion_Operator_Should_Be_Named_For_The_Type_It_Converts_To()
    {
        let item = Recorded_Member(
            "class Widget { public static implicit operator string(Widget w) => \"\"; }\n",
            "conversion_operator_declaration",
            ItemKind::Operator,
        );

        assert_eq!(item.name, "string");
    }

    #[test]
    fn Test_An_Indexer_Should_Be_Named_For_What_The_File_Spells()
    {
        let item = Recorded_Member(
            "class Widget { public int this[int at] => at; }\n",
            "indexer_declaration",
            ItemKind::Indexer,
        );

        assert_eq!(item.name, INDEXER_NAME);
        assert_eq!(item.shape.as_deref(), Some("fn/1"));
    }

    /// A property is typed rather than callable, so its shape is the type distinction and not an
    /// arity — and the array case is the one a consumer cannot recover from anything else in the
    /// record.
    #[test]
    fn Test_A_Property_Should_Carry_Its_Declared_Type_Shape()
    {
        let scalar = Recorded_Member(
            "class Widget { public int Count { get; set; } }\n",
            "property_declaration",
            ItemKind::Property,
        );
        let sequence = Recorded_Member(
            "class Widget { public int[] Slots { get; } }\n",
            "property_declaration",
            ItemKind::Property,
        );

        assert_eq!(scalar.shape.as_deref(), Some(nomos_cap_syntax::VALUE));
        assert_eq!(sequence.shape.as_deref(), Some(nomos_cap_syntax::SLICE));
    }

    #[test]
    fn Test_Several_Names_Against_One_Type_Should_Each_Be_An_Item()
    {
        let facts = Recorded_Fields("class Widget { private int first, second; }\n");

        let names: Vec<&str> = facts.items.iter().map(|item| return item.name.as_str()).collect();
        assert_eq!(names, vec!["first", "second"]);
        assert!(facts.items.iter().all(|item| return item.visibility == Visibility::Private));
    }

    #[test]
    fn Test_A_Nullable_Array_Field_Should_Still_Be_A_Sequence()
    {
        let facts = Recorded_Fields("class Widget { public int[]? slots; }\n");

        let item = facts.items.first().expect("one item recorded");
        assert_eq!(item.shape.as_deref(), Some(nomos_cap_syntax::SLICE));
    }

    /// A generic type whose name means a sequence is not one as far as this provider can see, and
    /// the negative control that keeps the assertion above from being about any array-ish name
    /// rather than about the grammar's own `array_type`.
    #[test]
    fn Test_A_Generic_Collection_Field_Should_Be_A_Value()
    {
        let facts = Recorded_Fields("class Widget { public List<int> items; }\n");

        let item = facts.items.first().expect("one item recorded");
        assert_eq!(item.shape.as_deref(), Some(nomos_cap_syntax::VALUE));
    }

    #[test]
    fn Test_An_Event_With_Its_Own_Accessors_Should_Be_Recorded_By_Name()
    {
        let source = "class Widget { public event EventHandler Changed { add { } remove { } } }\n";
        let tree = Parsed_Fixture_Tree(source);
        let node = Find_Kind(tree.root_node(), "event_declaration").expect("the fixture declares an event");
        let mut recording = Recording::New();

        Record_Event(&mut recording, Site::At_Root(node, source.as_bytes()));

        let item = recording.Into_Facts().items.first().expect("one item recorded").clone();
        assert_eq!(item.name, "Changed");
        assert_eq!(item.kind, ItemKind::Event);
    }
}
