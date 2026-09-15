//! What this module promises, exercised.

use super::*;
use crate::Reading;

fn Parsed(source: &str) -> Facts
{
    return match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        // Callers of this helper go straight on to read `.items`, so an empty `Facts`
        // handed back instead of a panic would be indistinguishable from a file that parsed
        // and declared nothing — which is the one confusion this module exists to rule out.
        // The failure carries the line it stopped on, and that is the only thing that tells
        // an author which of their fixture's lines they mistyped.
        Reading::Unparseable(failure) => panic!("expected a parse: {failure}"),
    };
}

fn Names(source: &str) -> Vec<String>
{
    use crate::Item;

    return Parsed(source)
        .items
        .iter()
        .map(Item::Qualified_Name)
        .collect();
}

#[test]
fn Test_Qualified_Name_Should_Include_Every_Items_Syntactic_Nesting()
{
    let names = Names(
        "mod outer { pub mod inner { pub fn deep() {} } }\n\
         struct Top;\n\
         impl Top { fn method(&self) {} }\n",
    );

    assert_eq!(
        names,
        vec![
            "outer",
            "outer::inner",
            "outer::inner::deep",
            "Top",
            "Top",
            "Top::method",
        ]
    );
}

#[test]
fn Test_Ordinals_Should_Be_Dense_And_Zero_Based()
{
    let facts = Parsed("fn a() {}\nfn b() {}\nfn c() {}\n");

    let ordinals: Vec<u32> = facts.items.iter().map(|item| return item.ordinal).collect();

    assert_eq!(ordinals, DENSE_ORDINALS_OF_THREE_DECLARATIONS);
}

/// The ordinals the fixture above's three declarations must carry, spelled out rather than
/// derived: dense says no gap between them, and zero-based says where the first one starts,
/// and a vector counted off the fixture itself would assert neither.
const DENSE_ORDINALS_OF_THREE_DECLARATIONS: [u32; 3] = [0, 1, 2];

/// `OD-CAPABILITY-010`'s extension: a named-field struct's own field names and head types,
/// read back through `nomos_cap_syntax::Struct_Fields` the same way a real consumer would,
/// not by inspecting the encoded string directly.
#[test]
fn Test_A_Named_Field_Structs_Fields_Should_Be_Recorded()
{
    let facts = Parsed("pub struct Counter { pub n: u32, label: String }\n");
    let item = facts.items.first().expect("the fixture is one struct declaration, so its first item is that struct");

    let shape = item.shape.clone().map_or(nomos_cap_syntax::Observation::Absent, nomos_cap_syntax::Observation::Present);
    let fields = nomos_cap_syntax::Struct_Fields(&shape).expect("a named-field struct records its fields");

    assert_eq!(
        fields,
        vec![("n".to_owned(), "u32".to_owned()), ("label".to_owned(), "String".to_owned())]
    );
}

/// A tuple struct and a unit struct both declare no named field a cross-language
/// comparison could key on — `Struct_Shape`'s own documented default, exercised here
/// against this provider's real output rather than assumed from its source.
#[test]
fn Test_A_Tuple_Or_Unit_Struct_Should_Record_No_Fields()
{
    let facts = Parsed("struct Pair(u32, u32);\nstruct Marker;\n");

    assert_eq!(facts.items.len(), FIXTURE_STRUCT_COUNT, "{facts:?}");
    for item in &facts.items
    {
        assert_eq!(item.shape, None, "{item:?}");
    }
}

/// The fixture above declares exactly this many structs. The loop below passes vacuously
/// over an empty `items`, so without this the test would be green for a parser that recorded
/// one struct and for one that recorded none.
const FIXTURE_STRUCT_COUNT: usize = 2;

/// A field's type is its head, not its full generic spelling — `Type_Head`'s own existing
/// boundary, reused rather than widened.
#[test]
fn Test_A_Generic_Fields_Type_Should_Record_Its_Head_Only()
{
    let facts = Parsed("pub struct Wrapper { pub inner: Vec<String> }\n");
    let item = facts.items.first().expect("the fixture is one struct declaration, so its first item is that struct");

    let shape = item.shape.clone().map_or(nomos_cap_syntax::Observation::Absent, nomos_cap_syntax::Observation::Present);
    let fields = nomos_cap_syntax::Struct_Fields(&shape).expect("a named-field struct records its fields");

    assert_eq!(fields, vec![("inner".to_owned(), "Vec".to_owned())]);
}

#[test]
fn Test_Visibility_Should_Be_Recorded_As_Declared()
{
    let facts = Parsed(
        "pub fn exported() {}\n\
         fn hidden() {}\n\
         pub(crate) fn within() {}\n\
         pub(in some::place) fn nested() {}\n\
         trait Contract { fn required(&self); }\n",
    );
    let visibilities: Vec<String> = facts
        .items
        .iter()
        .map(|item| return item.visibility.Label())
        .collect();

    assert_eq!(
        visibilities,
        vec![
            "Public",
            "Private",
            "Restricted(crate)",
            "Restricted(in some::place)",
            "Private",
            "NotApplicable",
        ],
        "a trait member declares no visibility, and saying `Private` would be \
         recording a keyword the source does not contain"
    );
}

/// A `use` introduces the binding it introduces, and nothing else. The prefix
/// segments are not declarations this file makes.
#[test]
fn Test_A_Use_Should_Record_The_Binding_It_Introduces()
{
    assert_eq!(
        Names("use std::collections::HashMap;\n"),
        vec!["HashMap".to_owned()]
    );
    assert_eq!(
        Names("use std::collections::HashMap as Map;\n"),
        vec!["Map".to_owned()],
        "the file now has `Map`; what `Map` refers to is a resolution away"
    );
    assert_eq!(
        Names("use std::collections::{HashMap, BTreeSet as Ordered};\n"),
        vec!["HashMap".to_owned(), "Ordered".to_owned()]
    );
    assert_eq!(Names("use std::fmt::*;\n"), vec!["*".to_owned()]);
}

/// Sources that declare nothing at all: empty, blank, or only a comment.
const EMPTY_SOURCES: &[&str] = &["", "\n\n", "// nothing here\n", "//! only a doc comment\n"];

/// A file that declares nothing parses. This is the variant that must never be how
/// a failure looks.
#[test]
fn Test_Has_No_Declarations_Should_Be_True_For_A_File_That_Declares_Nothing()
{
    for source in EMPTY_SOURCES
    {
        let facts = Parsed(source);

        assert!(facts.Has_No_Declarations(), "`{source:?}` declares nothing");
        assert_eq!(facts.unexpanded, 0);
    }
}

/// Sources broken in different ways, none of which should ever parse.
const UNPARSEABLE_SOURCES: &[&str] = &[
    "fn unclosed( {",
    "struct S { field: }",
    "this is not rust at all",
    "fn f() { let x = ; }",
];

/// The property the whole outcome type exists for.
#[test]
fn Test_Read_Source_Should_Report_Broken_Source_As_Unparseable_Rather_Than_Empty()
{
    for source in UNPARSEABLE_SOURCES
    {
        match Read_Source(source)
        {
            Reading::Unparseable(failure) =>
            {
                assert!(
                    failure.line >= 1,
                    "a refusal must name where it refused: {failure:?}"
                );
                assert!(!failure.message.is_empty());
            }
            Reading::Parsed(facts) =>
            {
                // The arm that must never be taken: each of these four sources is broken and
                // a reading of one is the provider claiming to have understood text rustc
                // rejects. The item count is printed rather than a bare failure because it
                // separates a reader that salvaged part of the file from one that read it as
                // empty, and only the second could be mistaken for `Has_No_Declarations`.
                panic!("`{source}` parsed to {} items", facts.items.len())
            }
        }
    }
}

/// A byte order mark belongs at offset zero or nowhere, and the difference decides
/// whether a file is source or damage.
///
/// Found by the corpus walk rather than reasoned about: seven files under
/// `F:/repos/xvpe` carry a `U+FEFF` at byte 15, immediately after a `use super::*;`
/// somebody prepended to a file that already began with one. They are the only seven
/// refusals in 7,580 files, and rustc will not compile them either — so the refusal
/// is this provider agreeing with the compiler, not falling behind it.
///
/// D-131 records the same distinction for the specification corpus, where the mark
/// belongs to the front matter fence. It is the same rule twice because it is a fact
/// about byte order marks rather than about either format.
#[test]
fn Test_A_Byte_Order_Mark_Should_Be_Leading_Or_Refused()
{
    Assert_Leading_Mark_Is_Ordinary_Rust();
    Assert_Stray_Mark_Is_Refused();
}

fn Assert_Leading_Mark_Is_Ordinary_Rust()
{
    let facts = Parsed("\u{feff}pub fn after_the_mark() {}\n");

    assert_eq!(
        facts.items.first().map(|item| return item.name.clone()),
        Some("after_the_mark".to_owned()),
        "a leading mark is an encoding announcement and the file is ordinary Rust"
    );
}

fn Assert_Stray_Mark_Is_Refused()
{
    let stray = "use super::*;\n\n\u{feff}//! documentation\npub fn hidden() {}\n";

    match Read_Source(stray)
    {
        Reading::Unparseable(failure) => assert_eq!(
            failure.line, STRAY_MARK_LINE,
            "the refusal names the line the mark is on: {failure}"
        ),
        // The half of the byte-order-mark rule that has to be red: a stray mark reported as
        // items is this provider disagreeing with rustc about whether the file compiles, and
        // the seven files under `F:/repos/xvpe` named above are where that would be believed.
        Reading::Parsed(facts) => panic!(
            "a mark in the middle of a file is not whitespace and this does not \
             compile; reading it as {} items would report a broken file as sound",
            facts.items.len()
        ),
    }
}

/// The line the stray mark occupies in `Assert_Stray_Mark_Is_Refused`'s fixture -- counted
/// out, so that a refusal naming the file's first line, or the item after the mark, is a
/// wrong line rather than a shifted one this assertion would follow.
const STRAY_MARK_LINE: usize = 3;

/// The measured reason completeness is `Unknown`, and the negative control for the
/// claim that this provider is honest about it. If the walk stopped at item level,
/// the count inside the function body would be zero and the declared weakness would
/// be undetectable from the output.
#[test]
fn Test_New_Should_Produce_A_Walk_That_Counts_Unexpanded_Regions_Wherever_They_Are()
{
    let facts = Parsed(
        "#[derive(Clone, Debug)]\n\
         pub struct Held;\n\
         fn body() { println!(\"one\"); vec![1, 2]; }\n\
         generated_items!();\n",
    );

    assert_eq!(
        facts.unexpanded, FIXTURE_UNEXPANDED_COUNT,
        "one derive, two invocations inside a body, one at item position: {:?}",
        facts.items
    );
}

/// The fixture above's four unexpanded regions: the derive, the two macro invocations
/// inside `body`, and the one at item position. Named so that a walk which stopped counting
/// at item level reports a number this test can name rather than an off-by-three.
const FIXTURE_UNEXPANDED_COUNT: u32 = 4;
