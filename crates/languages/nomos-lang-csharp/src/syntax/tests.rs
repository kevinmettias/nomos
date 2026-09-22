//! What [`Read_Source`] makes of whole files — the assertions no single recorder can make,
//! because each of them is about the walk as a whole.

use super::{ItemKind, Read_Source, Facts, Visibility};
use crate::Reading;

/// The declarations of `EVERY_CATEGORY`, as `<kind> <qualified name>` pairs in source order.
fn Read(source: &str) -> Facts
{
    return match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        // Every fixture in this module is C# its author wrote to be parseable, so a refusal is
        // a broken fixture and not a reading worth handing back to an assertion, which would
        // then pass vacuously over a walk that had begun refusing everything. The parser's own
        // message is printed because it names what stopped it.
        Reading::Unparseable(failure) => panic!("expected a parse: {failure}"),
    };
}

/// One declaration of each of the four categories this crate states it reads.
const EVERY_CATEGORY: &str = "using System;\n\
                              namespace Acme.Widgets\n\
                              {\n\
                                  public class Widget\n\
                                  {\n\
                                      private int slots;\n\
                                      public void Reset() { }\n\
                                  }\n\
                              }\n";

#[test]
fn Test_Read_Source_Should_Record_Every_Declaration_Category_In_Source_Order()
{
    let facts = Read(EVERY_CATEGORY);

    let recorded: Vec<(ItemKind, String)> = facts
        .items
        .iter()
        .map(|item| return (item.kind, item.Qualified_Name()))
        .collect();

    assert_eq!(
        recorded,
        vec![
            (ItemKind::Using, "System".to_owned()),
            (ItemKind::Namespace, "Acme.Widgets".to_owned()),
            (ItemKind::Class, "Acme.Widgets::Widget".to_owned()),
            (ItemKind::Field, "Acme.Widgets::Widget::slots".to_owned()),
            (ItemKind::Function, "Acme.Widgets::Widget::Reset".to_owned()),
        ]
    );
    assert_eq!(facts.unexpanded, 0, "this fixture has no conditional region");
}

/// A class whose two fields sit in the two branches of one conditional region, and a third
/// that does not.
const CONDITIONAL_FIELDS: &str = "public class Widget\n\
                                  {\n\
                                  #if DEBUG\n\
                                      public int Debugging;\n\
                                  #else\n\
                                      public int Shipping;\n\
                                  #endif\n\
                                      public int Always;\n\
                                  }\n";

/// The load-bearing one, and the falsifier for the guard `crate::Declared_Guarantee`'s
/// soundness claim rests on.
///
/// Both branches of this region are in the parse tree at once and at most one of them is in any
/// compilation. Measured against the two ways the guard can be broken: with the `preproc_if` arm
/// deleted the region count falls to zero, which is the claim that nothing was skipped; with the
/// arm changed to walk the region instead of counting it, `Debugging` is reported as an
/// unconditional field of `Widget`, which it is only under one definition set this provider
/// never saw.
#[test]
fn Test_A_Declaration_Inside_A_Conditional_Region_Should_Not_Be_Reported()
{
    let facts = Read(CONDITIONAL_FIELDS);

    let names: Vec<&str> = facts.items.iter().map(|item| return item.name.as_str()).collect();

    assert_eq!(names, vec!["Widget", "Always"]);
    assert_eq!(facts.unexpanded, 1, "the one region declined must be counted, not silently dropped");
}

/// A region, a pragma and a nullable directive around declarations that are not gated by any of
/// them.
const UNGATED_DIRECTIVES: &str = "#nullable enable\n\
                                  #pragma warning disable CS1591\n\
                                  #region Types\n\
                                  public class Alpha { }\n\
                                  public class Beta { }\n\
                                  #endregion\n";

/// The negative control for the test above. `#region`, `#pragma` and `#nullable` gate nothing,
/// and the grammar leaves declarations around them as ordinary siblings — so a walk that
/// declined every `preproc_*` node would lose two real classes and report a gap that is not
/// there.
#[test]
fn Test_A_Non_Conditional_Directive_Should_Hide_Nothing()
{
    let facts = Read(UNGATED_DIRECTIVES);

    let names: Vec<&str> = facts.items.iter().map(|item| return item.name.as_str()).collect();

    assert_eq!(names, vec!["Alpha", "Beta"]);
    assert_eq!(facts.unexpanded, 0, "none of these directives is a conditional region");
}

/// The form with no body, whose scope reaches the rest of the file.
const FILE_SCOPED_NAMESPACE: &str = "namespace Acme.Core;\n\
                                     \n\
                                     public class Widget { }\n\
                                     \n\
                                     public enum Color { Red }\n";

#[test]
fn Test_A_File_Scoped_Namespace_Should_Scope_Every_Declaration_After_It()
{
    let facts = Read(FILE_SCOPED_NAMESPACE);

    let names: Vec<String> = facts.items.iter().map(|item| return item.Qualified_Name()).collect();

    assert_eq!(
        names,
        vec![
            "Acme.Core".to_owned(),
            "Acme.Core::Widget".to_owned(),
            "Acme.Core::Color".to_owned(),
            "Acme.Core::Color::Red".to_owned(),
        ]
    );
}

/// A method whose body declares a local function and a local variable.
const LOCAL_DECLARATIONS: &str = "public class Widget\n\
                                  {\n\
                                      public void Outer()\n\
                                      {\n\
                                          int scratch = 0;\n\
                                          void Inner() { }\n\
                                      }\n\
                                  }\n";

/// The statement boundary, and the falsifier for it: a walk that descended into a `block` would
/// report `Inner` as a member of `Widget`, which it is not.
#[test]
fn Test_A_Local_Function_Should_Not_Be_Recorded_As_A_Member()
{
    let facts = Read(LOCAL_DECLARATIONS);

    let names: Vec<&str> = facts.items.iter().map(|item| return item.name.as_str()).collect();

    assert_eq!(names, vec!["Widget", "Outer"]);
}

/// A file declaring one half of a `partial` type.
const PARTIAL_HALF: &str = "public partial class Widget\n\
                            {\n\
                                public void First() { }\n\
                            }\n";

/// `partial` is a modifier and not an accessibility, and a per-file reading reports the half
/// this file declares — which is what `IncrementalGranularity::File` claims and nothing more.
#[test]
fn Test_A_Partial_Declaration_Should_Record_What_Its_Own_File_Declares()
{
    let facts = Read(PARTIAL_HALF);

    let widget = facts.items.first().expect("the class is recorded");
    assert_eq!(widget.visibility, Visibility::Public, "`partial` is not an accessibility word");
    assert_eq!(facts.items.len(), 2, "the class and its one method");
}

/// `OD-RULES-001`'s refusal honesty, for this provider: a file the parser could not read is a
/// refusal and never a clean answer over no items.
#[test]
fn Test_Broken_Source_Should_Be_Unparseable_Rather_Than_Empty()
{
    let reading = Read_Source("public class Widget { public void (");

    assert!(matches!(reading, Reading::Unparseable(_)), "got {reading:?}");
}

/// The third outcome, and the one that makes the second meaningful: a file that really declares
/// nothing is read, not refused.
#[test]
fn Test_A_File_Of_Only_Comments_Should_Be_Parsed_With_No_Items()
{
    let facts = Read("// Nothing here yet.\n");

    assert!(facts.Has_No_Declarations());
    assert_eq!(facts.unexpanded, 0);
}

/// The documentation a real XML doc comment carries, read through the whole walk rather than
/// through one recorder.
#[test]
fn Test_A_Documented_Member_Should_Carry_Its_Comment()
{
    let facts = Read("public class Widget\n{\n    /// <summary>Resets it.</summary>\n    public void Reset() { }\n}\n");

    let reset = facts.items.get(1).expect("the class and its method are both recorded");
    assert_eq!(reset.documentation.as_deref(), Some("<summary>Resets it.</summary>"));
}
