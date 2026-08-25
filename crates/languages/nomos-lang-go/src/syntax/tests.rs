//! What this module promises, exercised.

use super::*;
use crate::Reading;

fn Parsed(source: &str) -> SyntaxFacts
{
    return match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        Reading::Unparseable(failure) => panic!("expected a parse: {failure}"),
    };
}

fn Names(source: &str) -> Vec<String>
{
    return Parsed(source).items.iter().map(SyntaxItem::Qualified_Name).collect();
}

#[test]
fn Test_A_Method_Should_Be_Named_By_Its_Receiver_Type()
{
    let names = Names(
        "package main\n\n\
         type Counter struct { n int }\n\n\
         func (c *Counter) Increment() {}\n\n\
         func Free() {}\n",
    );

    assert_eq!(names, vec!["Counter", "Counter::Increment", "Free"]);
}

#[test]
fn Test_Ordinals_Should_Be_Dense_And_Zero_Based()
{
    let facts = Parsed("package main\n\nfunc a() {}\nfunc b() {}\nfunc c() {}\n");

    let ordinals: Vec<u32> = facts.items.iter().map(|item| return item.ordinal).collect();

    assert_eq!(ordinals, vec![0, 1, 2]);
}

#[test]
fn Test_Visibility_Should_Be_Recorded_From_The_Names_Own_Case()
{
    let facts = Parsed(
        "package main\n\n\
         func Exported() {}\n\
         func hidden() {}\n\
         var _ int\n",
    );
    let visibilities: Vec<&str> = facts.items.iter().map(|item| return item.visibility.Label()).collect();

    assert_eq!(
        visibilities,
        vec!["Public", "Private", "NotApplicable"],
        "the blank identifier declares no name for visibility to be a fact about"
    );
}

/// An interface's method set is recorded the same way a Rust trait's member list is — see
/// [`crate::guarantee`] for why the visibility on these is real rather than
/// `NotApplicable`, unlike the Rust provider's trait members. An embedded interface names no
/// method of its own and is not recorded.
#[test]
fn Test_An_Interfaces_Method_Set_Should_Be_Recorded_Under_Its_Name()
{
    let facts = Parsed(
        "package main\n\n\
         type Writer interface {\n\
         \tWrite(p []byte) (n int, err error)\n\
         \tio.Reader\n\
         }\n",
    );

    let names: Vec<String> = facts.items.iter().map(SyntaxItem::Qualified_Name).collect();
    assert_eq!(names, vec!["Writer".to_owned(), "Writer::Write".to_owned()]);

    let method = facts.items.get(1).expect("two items");
    assert_eq!(method.visibility, Visibility::Public);
    assert_eq!(method.shape.as_deref(), Some("fn/1"));
}

/// A generic function or type is the same node with an added `type_parameters` field, not a
/// less-transparent form — the fact [`crate::Declared_Guarantee`]'s completeness claim rests
/// on. This is the walk actually reading one, rather than the grammar merely being checked
/// by hand once during design.
#[test]
fn Test_A_Generic_Declaration_Should_Be_Read_Like_Any_Other()
{
    let facts = Parsed(
        "package main\n\n\
         type Container[T any] struct { items []T }\n\n\
         func (c *Container[T]) Add(item T) {}\n\n\
         func Map[T any, U any](items []T, f func(T) U) []U { return nil }\n",
    );

    let names: Vec<String> = facts.items.iter().map(SyntaxItem::Qualified_Name).collect();
    assert_eq!(names, vec!["Container", "Container::Add", "Map"]);
    assert_eq!(facts.unexpanded, 0);
}

/// `func f(a, b int)` shares one type across two names in a single `parameter_declaration`
/// — the same repeated-field grammar shape `const A, B = 1, 2` has, including the comma
/// that `Named_Field_Children` filters back out. Arity must count the two names, not the
/// one declaration node.
#[test]
fn Test_Shared_Type_Parameters_Should_Each_Count_Toward_Arity()
{
    let facts = Parsed("package main\n\nfunc f(a, b int, c string) {}\n");

    let shape = facts.items.first().and_then(|item| return item.shape.clone());
    assert_eq!(shape.as_deref(), Some("fn/3"), "a, b and c are three declared parameters");
}

/// Grouped and single forms are two spellings of one grammar, and this crate reads both
/// through the same recursive search rather than two code paths — see `walk.rs`'s own doc.
#[test]
fn Test_Grouped_And_Single_Declarations_Should_Read_The_Same_Way()
{
    assert_eq!(
        Names("package main\n\nconst (\n\tA = 1\n\tB = 2\n)\n"),
        vec!["A".to_owned(), "B".to_owned()]
    );
    assert_eq!(Names("package main\n\nconst Solo = 1\n"), vec!["Solo".to_owned()]);
    assert_eq!(
        Names("package main\n\nconst A, B = 1, 2\n"),
        vec!["A".to_owned(), "B".to_owned()],
        "one spec, two names, sharing one value list"
    );
}

/// `import` binds a name Go itself decides — the alias when there is one, the path's own
/// final segment when there is not, since this provider does not read the imported package
/// to learn its declared name.
#[test]
fn Test_An_Import_Should_Record_The_Name_It_Binds()
{
    assert_eq!(
        Names("package main\n\nimport \"fmt\"\n"),
        vec!["fmt".to_owned()]
    );
    assert_eq!(
        Names("package main\n\nimport str \"strings\"\n"),
        vec!["str".to_owned()],
        "an alias is what this file now has"
    );
    assert_eq!(
        Names("package main\n\nimport \"path/filepath\"\n"),
        vec!["filepath".to_owned()],
        "unaliased, a package binds its path's final segment"
    );
}

/// `type X = Y` and `type X Y` are distinct grammar nodes and distinct facts: one names one
/// type twice, the other declares a new type with `Y`'s representation.
#[test]
fn Test_A_Type_Alias_Should_Be_Distinguished_From_A_Defined_Type()
{
    let facts = Parsed("package main\n\ntype Meters float64\n\ntype Alias = string\n");

    let kinds: Vec<ItemKind> = facts.items.iter().map(|item| return item.kind).collect();
    assert_eq!(kinds, vec![ItemKind::TypeDefinition, ItemKind::TypeAlias]);
}

/// A doc comment attaches at whichever level the grammar puts it: directly above a spec
/// inside a grouped block, or above the keyword for a single declaration — `walk.rs`'s
/// `Search_Anchor` is what makes both land on the same field.
#[test]
fn Test_Documentation_Should_Be_Found_At_Either_Attachment_Point()
{
    let grouped = Parsed(
        "package main\n\n\
         const (\n\
         \tA = 1\n\
         \t// B is two.\n\
         \tB = 2\n\
         )\n",
    );
    let b = grouped.items.get(1).expect("two items");
    assert_eq!(b.documentation.as_deref(), Some("B is two."));

    let single = Parsed(
        "package main\n\n\
         // Tables lists every table.\n\
         var Tables []string\n",
    );
    let tables = single.items.first().expect("one item");
    assert_eq!(tables.documentation.as_deref(), Some("Tables lists every table."));
}

/// A blank source line ends the doc comment run — a comment that far away is conventionally
/// about something else, the same rule godoc itself applies.
#[test]
fn Test_A_Blank_Line_Should_End_The_Documentation_Run()
{
    let facts = Parsed(
        "package main\n\n\
         // Unrelated.\n\n\
         func NoDoc() {}\n",
    );

    assert_eq!(facts.items.first().and_then(|item| return item.documentation.clone()), None);
}

/// A file that declares nothing parses. This is the variant that must never be how a
/// failure looks.
#[test]
fn Test_A_File_That_Declares_Nothing_Should_Parse()
{
    for source in ["package main\n", "package main\n\n// nothing here\n"]
    {
        let facts = Parsed(source);

        assert!(facts.Declares_Nothing(), "`{source:?}` declares nothing");
        assert_eq!(facts.unexpanded, 0);
    }
}

/// The property the whole outcome type exists for.
#[test]
fn Test_Broken_Source_Should_Be_Unparseable_Rather_Than_Empty()
{
    for source in ["package main\n\nfunc unclosed( {", "this is not go at all {{{"]
    {
        match Read_Source(source)
        {
            Reading::Unparseable(failure) =>
            {
                assert!(!failure.message.is_empty(), "a refusal must say why: {failure:?}");
            }
            Reading::Parsed(facts) => panic!("`{source}` parsed to {} items", facts.items.len()),
        }
    }
}

/// The measured reason completeness is `Sound` rather than `Unknown` here, unlike
/// `nomos-lang-rust`: a `//go:build` tag is an ordinary comment, and this file's own
/// declaration beneath it is read like any other. The negative control for
/// [`crate::Declared_Guarantee`]'s completeness claim — if a build tag or a doc comment
/// silently consumed the declaration below it, this would be the test to catch it.
#[test]
fn Test_A_Build_Tag_Should_Not_Hide_The_Declaration_Below_It()
{
    let facts = Parsed("package main\n\n//go:build linux\n\nfunc OnLinux() {}\n");

    assert_eq!(facts.items.len(), 1);
    assert_eq!(facts.unexpanded, 0);
}
