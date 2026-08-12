//! What this module promises, exercised.

use super::*;
use crate::syntax::SyntaxItem;
use crate::reading::Reading;

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
    return Parsed(source)
        .items
        .iter()
        .map(SyntaxItem::Qualified_Name)
        .collect();
}

#[test]
fn Test_Items_Should_Be_Named_By_Their_Syntactic_Nesting()
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

    assert_eq!(ordinals, vec![0, 1, 2]);
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

/// A file that declares nothing parses. This is the variant that must never be how
/// a failure looks.
#[test]
fn Test_A_File_That_Declares_Nothing_Should_Parse()
{
    for source in ["", "\n\n", "// nothing here\n", "//! only a doc comment\n"]
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
    for source in [
        "fn unclosed( {",
        "struct S { field: }",
        "this is not rust at all",
        "fn f() { let x = ; }",
    ]
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
    let facts = Parsed("\u{feff}pub fn after_the_mark() {}\n");

    assert_eq!(
        facts.items.first().map(|item| return item.name.clone()),
        Some("after_the_mark".to_owned()),
        "a leading mark is an encoding announcement and the file is ordinary Rust"
    );

    let stray = "use super::*;\n\n\u{feff}//! documentation\npub fn hidden() {}\n";

    match Read_Source(stray)
    {
        Reading::Unparseable(failure) => assert_eq!(
            failure.line, 3,
            "the refusal names the line the mark is on: {failure}"
        ),
        Reading::Parsed(facts) => panic!(
            "a mark in the middle of a file is not whitespace and this does not \
             compile; reading it as {} items would report a broken file as sound",
            facts.items.len()
        ),
    }
}

/// The measured reason completeness is `Unknown`, and the negative control for the
/// claim that this provider is honest about it. If the walk stopped at item level,
/// the count inside the function body would be zero and the declared weakness would
/// be undetectable from the output.
#[test]
fn Test_Unexpanded_Regions_Should_Be_Counted_Wherever_They_Are()
{
    let facts = Parsed(
        "#[derive(Clone, Debug)]\n\
         pub struct Held;\n\
         fn body() { println!(\"one\"); vec![1, 2]; }\n\
         generated_items!();\n",
    );

    assert_eq!(
        facts.unexpanded, 4,
        "one derive, two invocations inside a body, one at item position: {:?}",
        facts.items
    );
}
