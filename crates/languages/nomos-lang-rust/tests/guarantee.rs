//! The declared [`Declared_Guarantee`] checked against what the provider actually
//! produces — one property per axis, including the axis it fails.
//!
//! A guarantee nothing exercises is a comment with a type. The registry checks that this
//! provider does not claim more than its capability permits; these tests check the other
//! half, which is that what it does claim is true. The two are different questions and
//! neither answers the other: the registry would happily accept a provider that declared
//! `Syntactic` and secretly resolved names, and it would equally accept one that
//! declared `Sound` and invented items.
//!
//! The completeness axis is the one worth reading. It is declared `Unknown`, and
//! [`Test_Completeness_Should_Be_Unknown_Because_Macros_Hide_Items`] fails if the output
//! turns out to be complete after all — because a weakness nobody can demonstrate is
//! indistinguishable from modesty, and a caller cannot plan around modesty.

use nomos_contracts::{Assurance, FactVariant, IncrementalGranularity};
use nomos_lang_rust::{
    Declared_Guarantee, ItemKind, Read_Source, Reading, SyntaxFacts, SyntaxItem,
};

fn Parsed(source: &str) -> SyntaxFacts
{
    return match Read_Source(source)
    {
        Reading::Parsed(facts) => facts,
        // Each fixture here is written to demonstrate one axis of the declared guarantee, so
        // a refusal means the demonstration never ran. That has to be loud: the assertions
        // built on this helper are mostly of the form "the output does *not* contain X", and
        // an empty reading satisfies every one of them — a provider that had started refusing
        // valid Rust would look like a provider whose modesty had been proved.
        Reading::Unparseable(failure) => panic!("expected a parse: {failure}"),
    };
}

fn Names(facts: &SyntaxFacts) -> Vec<String>
{
    return facts.items.iter().map(SyntaxItem::Qualified_Name).collect();
}

// ---------------------------------------------------------------------------------
// variant: Syntactic
// ---------------------------------------------------------------------------------

/// The axis where overclaiming does the most damage, so it is asserted from the
/// direction of the overclaim: nothing in the output is a name the file does not spell
/// at the site it was read from.
///
/// `impl Display for Foo` in a file that imports `std::fmt::Display` is, to a resolver,
/// an implementation of `core::fmt::Display`. This provider says `Foo` and says nothing
/// about which `Display`, because a file three lines longer could have declared its own
/// and every such claim would then be wrong.
#[test]
fn Test_The_Variant_Should_Be_Syntactic_Because_No_Name_Is_Resolved()
{
    let facts = Parsed(
        "use std::fmt::Display;\n\
         use std::collections::HashMap as Map;\n\
         pub struct Foo;\n\
         impl Display for Foo {\n\
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }\n\
         }\n",
    );
    let names = Names(&facts);

    assert_eq!(Declared_Guarantee().variant, FactVariant::Syntactic);
    assert_eq!(
        names,
        vec!["Display", "Map", "Foo", "Foo", "Foo::fmt"],
        "every name is the identifier introduced or written at that site"
    );
    Assert_Nothing_Was_Expanded(&names);
}

/// A syntactic provider records what the file spells; expanding `Display` to
/// `std::fmt::Display` is resolution, and this provider has no basis for it.
fn Assert_Nothing_Was_Expanded(names: &[String])
{
    for name in names
    {
        assert!(
            !name.contains("std") && !name.contains("fmt::"),
            "`{name}` is an expanded path. A syntactic provider records what the file \
             spells; expanding `Display` to `std::fmt::Display` is resolution, and this \
             provider has no basis for it"
        );
    }
}

/// The other half of `Syntactic`: the answer is a function of the file's bytes and of
/// nothing else — not the order files arrive in, not what was read before.
///
/// A provider that accumulated state across files would still look correct on any single
/// file and would produce a corpus whose results depended on directory iteration order.
#[test]
fn Test_The_Reading_Should_Not_Depend_On_What_Was_Read_Before()
{
    let sources = [
        "pub fn alpha() {}\n",
        "mod beta { pub struct Inner; }\n",
        "impl Gamma { fn method() {} }\n",
        "fn unclosed( {\n",
        "",
    ];

    let forwards: Vec<Reading> = sources.iter().map(|source| Read_Source(source)).collect();
    let backwards: Vec<Reading> = sources
        .iter()
        .rev()
        .map(|source| Read_Source(source))
        .collect();

    assert_eq!(
        forwards,
        backwards.into_iter().rev().collect::<Vec<Reading>>(),
        "reading the same files in the opposite order must reach the same readings"
    );
}

// ---------------------------------------------------------------------------------
// soundness: Sound
// ---------------------------------------------------------------------------------

/// Every item reported is in the text. Checked by finding each name as a token in the
/// source, which is weaker than proving it came from the right place and is the strongest
/// thing available without reimplementing the parser to check the parser.
#[test]
fn Test_Soundness_Should_Hold_Every_Reported_Name_Occurs_In_The_Source()
{
    let source = "pub mod outer {\n\
                      pub(crate) struct Held { field: u8 }\n\
                      pub enum Choice { One, Two }\n\
                      impl Held { pub const LIMIT: u8 = 4; fn read(&self) {} }\n\
                      pub trait Contract { type Output; fn run(&self); }\n\
                  }\n\
                  pub type Alias = u32;\n\
                  static COUNT: u8 = 0;\n";
    let facts = Parsed(source);

    assert_eq!(Declared_Guarantee().soundness, Assurance::Sound);
    assert!(!facts.items.is_empty(), "the sample declares items");
    for item in &facts.items
    {
        assert!(
            source.contains(&item.name),
            "`{}` ({}) was reported and does not occur in the source",
            item.name,
            item.kind
        );
    }
}

/// The negative control for soundness. If `contains` were the whole test it would pass
/// for a provider that reported every substring of the file, so the sample also has to
/// show that things which are *not* items are not reported as items.
#[test]
fn Test_Soundness_Should_Not_Report_Things_That_Are_Not_Items()
{
    let facts = Parsed(
        "fn holder() {\n\
             let local_binding = 1;\n\
             struct NotHoisted;\n\
         }\n",
    );

    let names = Names(&facts);

    assert!(
        !names.iter().any(|name| return name.contains("local_binding")),
        "a `let` binding is not an item: {names:?}"
    );
    assert!(
        names.iter().any(|name| return name == "holder"),
        "the function itself is an item: {names:?}"
    );
}

// ---------------------------------------------------------------------------------
// completeness: Unknown
// ---------------------------------------------------------------------------------

/// The declared weakness, demonstrated.
///
/// This file genuinely declares `pub fn generated`. The provider does not report it, and
/// no amount of care short of expanding macros would let it. That is what `Unknown`
/// means here, and this test is what stops the declaration from being unearned modesty —
/// if a future change made the output complete, this test fails and the guarantee has to
/// be revised rather than silently understating what the provider can do.
#[test]
fn Test_Completeness_Should_Be_Unknown_Because_Macros_Hide_Items()
{
    assert_eq!(Declared_Guarantee().completeness, Assurance::Unknown);

    let facts = Parsed(
        // This text is a fixture handed to the provider, not a macro this suite defines, and
        // it has to be a real `macro_rules!` because the weakness being demonstrated is
        // exactly that `syn` sees the definition and never the body it would expand to.
        "macro_rules! declare {\n\
             () => { pub fn generated() {} };\n\
         }\n\
         declare!();\n\
         pub fn visible() {}\n",
    );

    let names = Names(&facts);

    assert!(
        names.iter().any(|name| return name == "visible"),
        "an item outside a macro is reported: {names:?}"
    );
    assert!(
        !names.iter().any(|name| return name == "generated"),
        "`generated` is in this file and the provider cannot see it. If it now can, the \
         completeness axis is no longer Unknown and must be redeclared: {names:?}"
    );
}

/// The gap is reported rather than left for a caller to infer from a suspiciously small
/// number. A file with two items and forty unexpanded regions is a different situation
/// from a file with two items and none, and only one of them is worth a second look.
#[test]
fn Test_The_Unexpanded_Count_Should_Expose_How_Much_Was_Out_Of_Reach()
{
    let reachable = Parsed("pub fn one() {}\npub fn two() {}\n");
    let mostly_hidden = Parsed(
        "#[derive(Clone)]\n\
         pub struct One;\n\
         pub fn two() { assert!(true); format!(\"x\"); }\n",
    );

    assert_eq!(reachable.unexpanded, 0);
    assert!(
        mostly_hidden.unexpanded >= 3,
        "one derive and two invocations, at least: {}",
        mostly_hidden.unexpanded
    );
}

/// The count is honest about being a lower bound. `#[tokio::main]` generates code and is
/// syntactically identical to `#[allow(dead_code)]`, which does not — telling them apart
/// is the name resolution this provider does not do.
///
/// Asserted rather than only documented, because "lower bound" is the difference between
/// a caller treating zero as "nothing was hidden" and treating it as "nothing detectable
/// was hidden".
#[test]
fn Test_The_Unexpanded_Count_Should_Be_A_Lower_Bound()
{
    let indistinguishable = Parsed(
        "#[allow(dead_code)]\n\
         pub fn inert() {}\n",
    );

    assert_eq!(
        indistinguishable.unexpanded, 0,
        "an attribute macro that generates items would also count zero here, which is \
         exactly why completeness cannot be Sound"
    );
}

// ---------------------------------------------------------------------------------
// incremental: File
// ---------------------------------------------------------------------------------

/// A file's reading depends on that file and completes for that file, so the unit of
/// refresh is the file.
///
/// The strong form of this is structural — [`Read_Source`] takes a `&str` and has no way
/// to reach a second file — so what is left to assert is that the implementation carries
/// nothing across calls: a file read after a hundred others reads identically to the same
/// file read first.
#[test]
fn Test_The_Granularity_Should_Be_File_Because_A_Reading_Carries_Nothing_Across()
{
    assert_eq!(
        Declared_Guarantee().incremental,
        IncrementalGranularity::File
    );

    let subject = "pub mod held { pub fn only() {} }\n";
    let first = Read_Source(subject);

    for index in 0..100_u32
    {
        let _ = Read_Source(&format!("pub fn noise_{index}() {{ vec![{index}]; }}\n"));
    }
    let _ = Read_Source("fn broken( {");

    assert_eq!(
        Read_Source(subject),
        first,
        "reading a hundred other files, one of them unparseable, must not change what \
         this file says"
    );
}

/// Granularity is `File` and not `Symbol`. `syn` parses a whole file or refuses it, so
/// there is no reading of half a file to offer — and claiming `Symbol` would let the
/// invalidation engine refresh one function and treat the rest of the file as current.
#[test]
fn Test_The_Granularity_Should_Not_Be_Symbol_Because_A_Failure_Costs_The_Whole_File()
{
    let one_bad_function = "pub fn fine() {}\npub fn broken( {\npub fn also_fine() {}\n";

    match Read_Source(one_bad_function)
    {
        Reading::Unparseable(_) =>
        {}
        // Reaching this arm means the provider recovered items from a file with a broken
        // function in it, which is the one observation that would make `File` an understated
        // granularity. The declaration would then have to be revisited rather than the test
        // relaxed, so the names that survived are printed: they are the evidence of how far
        // the reader actually got.
        Reading::Parsed(facts) => panic!(
            "a symbol-granular provider would have kept `fine` and `also_fine`; this one \
             cannot, which is why it declares File: {:?}",
            Names(&facts)
        ),
    }
}

// ---------------------------------------------------------------------------------
// the three outcomes
// ---------------------------------------------------------------------------------

/// The property the item exists for: a file that could not be read never looks like a
/// file that had nothing to say.
///
/// Both halves in one test, because either alone is satisfiable by a broken provider. A
/// provider that returned `Unparseable` for everything passes the first half; one that
/// returned `Parsed(empty)` for everything passes the second.
#[test]
fn Test_A_Failure_And_An_Empty_File_Should_Never_Be_The_Same_Answer()
{
    let broken = [
        "fn unclosed( {",
        "struct S { field: }",
        "impl { }",
        "not rust",
        "pub pub fn twice() {}",
    ];
    for source in broken
    {
        assert!(
            matches!(Read_Source(source), Reading::Unparseable(_)),
            "`{source}` must refuse rather than read as empty"
        );
    }
    for source in ["", "\n", "// a comment\n", "#![allow(dead_code)]\n"]
    {
        Assert_It_Parses_And_Declares_Nothing(source);
    }
}

/// Valid Rust that declares nothing is a reading, never a refusal.
fn Assert_It_Parses_And_Declares_Nothing(source: &str)
{
    match Read_Source(source)
    {
        Reading::Parsed(facts) => assert!(
            facts.Declares_Nothing(),
            "`{source:?}` declares nothing and parses"
        ),
        Reading::Unparseable(failure) =>
        {
            // The empty file, the lone newline, the comment and the inner attribute all
            // compile, so a refusal here is the provider calling sound source damaged — the
            // mirror image of the defect the enclosing test guards, and the direction that
            // would make a corpus walk report files as broken that CI builds every day.
            panic!("`{source:?}` is valid Rust: {failure}")
        }
    }
}

/// A refusal names where it refused. A corpus walk reporting a count of unparseable files
/// and nothing else has produced a number nobody can act on.
#[test]
fn Test_A_Refusal_Should_Name_Where_It_Refused()
{
    let Reading::Unparseable(failure) =
        Read_Source("pub fn fine() {}\n\npub fn broken( {\n")
    else
    {
        // A let-else has to diverge, so no error can be returned from here. The binding is
        // what the assertions below read `failure.line` out of, and this test is about the
        // refusal naming line 3 — with nothing to name, there is no test left to run.
        panic!("this source does not parse")
    };

    assert!(
        failure.line >= 3,
        "the failure is on the third line, not the first: {failure}"
    );
    assert!(!failure.message.is_empty());
    assert!(
        failure.to_string().contains(&failure.message),
        "the rendered form must carry the reason"
    );
}

/// Every item form Rust has reaches a distinct [`ItemKind`]. A form that fell through to
/// another kind would be reported under a name that is not its own, and nothing would
/// notice.
#[test]
fn Test_Every_Item_Form_Should_Reach_Its_Own_Kind()
{
    let facts = Parsed(ONE_OF_EVERY_FORM);
    let kinds: std::collections::BTreeSet<ItemKind> =
        facts.items.iter().map(|item| return item.kind).collect();

    for expected in EVERY_KIND
    {
        assert!(
            kinds.contains(&expected),
            "{expected} was declared in the sample and not reported: {kinds:?}"
        );
    }
}

/// One declaration of every item form the language has, in one file.
const ONE_OF_EVERY_FORM: &str = "extern crate alloc;\n\
                                 use std::fmt;\n\
                                 pub mod inner {}\n\
                                 pub const LIMIT: u8 = 1;\n\
                                 pub static NAME: u8 = 2;\n\
                                 pub type Alias = u8;\n\
                                 pub struct Shape;\n\
                                 pub enum Choice { One }\n\
                                 pub union Overlap { left: u8 }\n\
                                 pub trait Contract {}\n\
                                 impl Shape {}\n\
                                 macro_rules! declared { () => {}; }\n\
                                 extern \"C\" { pub fn external(); }\n";

/// The kinds that sample must reach. A form the reader drops is a declaration nobody would
/// notice.
const EVERY_KIND: [ItemKind; 14] = [
    ItemKind::Constant,
    ItemKind::Enum,
    ItemKind::ExternCrate,
    ItemKind::ForeignModule,
    ItemKind::Function,
    ItemKind::Implementation,
    ItemKind::MacroDefinition,
    ItemKind::Module,
    ItemKind::Static,
    ItemKind::Struct,
    ItemKind::Trait,
    ItemKind::TypeAlias,
    ItemKind::Union,
    ItemKind::Use,
];

