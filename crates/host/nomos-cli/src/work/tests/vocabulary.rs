//! The two closed vocabularies `add` accepts, compared in both directions against the words
//! the printed usage text advertises for them -- and the one flag `list` takes, compared
//! against both the usage text and the operating contract that sends every session to the
//! verb.

use super::Added;
use super::Arguments;
use super::{Alternatives_After, FlagAlternatives, Sorted_Words};
use super::super::parse::Usage_Text;
use super::super::{ListingScope, WorkCommand, Work_Command_From_String_Arguments};
use nomos_ledger::{ItemKind, ItemOrigin};

/// Every kind an item can declare.
///
/// `nomos_ledger::item::Kind`'s own file is not this item's territory, so the census is here,
/// paired with [`Spelled_Kind`]'s wildcard-free match: a variant added to [`ItemKind`] fails
/// this file to compile rather than to pass.
fn Every_Item_Kind() -> &'static [ItemKind]
{
    return &[
        ItemKind::Capability,
        ItemKind::Decision,
        ItemKind::Validation,
        ItemKind::Correction,
        ItemKind::Cleanup,
    ];
}

/// The spelling `work add --kind` takes for a kind, as an exhaustive match.
///
/// Deliberately not trusted from here.
/// `Test_The_Listed_Add_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare` parses each spelling
/// back through the real command line and asserts it produces the variant named beside it, so
/// a word that drifted from `parse.rs`'s own match fails rather than agreeing with itself.
fn Spelled_Kind(kind: ItemKind) -> &'static str
{
    return match kind
    {
        ItemKind::Capability => "capability",
        ItemKind::Decision => "decision",
        ItemKind::Validation => "validation",
        ItemKind::Correction => "correction",
        ItemKind::Cleanup => "cleanup",
    };
}

/// Every origin an item can declare. [`Every_Item_Kind`]'s reasoning, one closed set over.
fn Every_Item_Origin() -> &'static [ItemOrigin]
{
    return &[ItemOrigin::Required, ItemOrigin::Proposed];
}

/// The spelling `work add --origin` takes for an origin, as an exhaustive match.
fn Spelled_Origin(origin: ItemOrigin) -> &'static str
{
    return match origin
    {
        ItemOrigin::Required => "required",
        ItemOrigin::Proposed => "proposed",
    };
}

/// A `--kind` word as the printed usage text spells it, to be read back through `add`.
///
/// A type of its own rather than a second adjacent `&str`: both positions are strings
/// otherwise, so a caller could transpose them and the round trip below would assert that the
/// origin spelling parses to the kind it is named after.
#[derive(Clone, Copy)]
struct KindSpelling<'a>(&'a str);

/// An `--origin` word as the printed usage text spells it, to be read back through `add`.
#[derive(Clone, Copy)]
struct OriginSpelling<'a>(&'a str);

/// An `add` line carrying one `--kind` and one `--origin`, so a spelling can be parsed back.
fn Add_Line(kind: KindSpelling<'_>, origin: OriginSpelling<'_>) -> String
{
    return format!(
        "add --item T-1 --title t --why w --done-when d --kind {} --origin {} --territory src/a.rs",
        kind.0, origin.0
    );
}

/// The kinds `work add` prints are the kinds an item can declare, both directions.
///
/// `OD-AGENT-004` version 2: a help text may enumerate a compiled vocabulary only where a test
/// compares that enumeration against the vocabulary's own authority. `--kind` is one of the two
/// closed sets `OD-LEDGER-024` decided, the help text says so in the same breath ("both are
/// closed sets: an unrecognized value is refused rather than stored"), and until this test
/// nothing compared the words to the set.
///
/// Three assertions rather than one, because they fail for different reasons a reader has to
/// tell apart: the printed list was not found at all, a kind exists that the text does not
/// offer, and a word the text offers is not one the command line takes.
#[test]
fn Test_The_Listed_Add_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare()
{
    let listed = Alternatives_After(FlagAlternatives { usage: &Usage_Text(), flag: "--kind" });

    assert!(
        !listed.is_empty(),
        "no --kind alternative was parsed out of the usage text, so this compared nothing: {}",
        Usage_Text()
    );
    assert_eq!(
        Sorted_Words(listed.iter().cloned()),
        Sorted_Words(Every_Item_Kind().iter().map(|kind| return Spelled_Kind(*kind).to_owned())),
        "the usage text and ItemKind disagree about what --kind takes"
    );

    Assert_Each_Kind_Parses_Back(&listed);
}

/// Drives each advertised `--kind` spelling back through the real command line, so a word
/// that drifted from `parse.rs`'s own match fails rather than agreeing with itself.
fn Assert_Each_Kind_Parses_Back(listed: &[String])
{
    for spelling in listed
    {
        let line = Add_Line(KindSpelling(spelling), OriginSpelling("proposed"));
        let item = Added(&line);
        let named = Every_Item_Kind()
            .iter()
            .find(|kind| return Spelled_Kind(**kind) == spelling.as_str())
            .expect("the set comparison above already established the spelling is one of these");

        assert_eq!(item.kind, *named, "--kind {spelling} does not parse to the kind spelled for it here");
    }
}

/// The origins `work add` prints are the origins an item can declare, both directions.
/// [`Test_The_Listed_Add_Kinds_Should_Be_Every_Kind_An_Item_Can_Declare`]'s reasoning, over the
/// other of `OD-LEDGER-024`'s two closed sets.
#[test]
fn Test_The_Listed_Add_Origins_Should_Be_Every_Origin_An_Item_Can_Declare()
{
    let listed = Alternatives_After(FlagAlternatives { usage: &Usage_Text(), flag: "--origin" });

    assert!(
        !listed.is_empty(),
        "no --origin alternative was parsed out of the usage text, so this compared nothing: {}",
        Usage_Text()
    );
    assert_eq!(
        Sorted_Words(listed.iter().cloned()),
        Sorted_Words(Every_Item_Origin().iter().map(|origin| return Spelled_Origin(*origin).to_owned())),
        "the usage text and ItemOrigin disagree about what --origin takes"
    );

    Assert_Each_Origin_Parses_Back(&listed);
}

/// [`Assert_Each_Kind_Parses_Back`]'s other half, over `--origin`.
fn Assert_Each_Origin_Parses_Back(listed: &[String])
{
    for spelling in listed
    {
        let line = Add_Line(KindSpelling("correction"), OriginSpelling(spelling));
        let item = Added(&line);
        let named = Every_Item_Origin()
            .iter()
            .find(|origin| return Spelled_Origin(**origin) == spelling.as_str())
            .expect("the set comparison above already established the spelling is one of these");

        assert_eq!(item.origin, *named, "--origin {spelling} does not parse to the origin spelled for it here");
    }
}

/// The flag `AGENTS.md` step 2 offers is the flag this command line takes, and the printed
/// usage text advertises the same one.
///
/// `OD-AGENT-004` version 2 again, with the enumerating document outside this crate. The
/// operating contract sends every session to `nomos work list` before it does anything else,
/// and since `OD-LEDGER-041` that listing is bounded -- so the sentence naming the way past
/// the bound is load-bearing, because a spelling nobody can reach is not a bound but a hole.
/// The authority for the spelling is private to this binary crate, which is why
/// `tests/contract/tests/boundaries/readme.rs` routes to this vocabulary rather than restating
/// it. The comparison therefore lives here, the one place both halves are in reach.
///
/// Three assertions, failing for three reasons a reader has to tell apart: the contract names
/// no flag at all, it names one this command line does not take, or it names one the printed
/// usage text has never heard of.
#[test]
fn Test_The_Contracts_Step_Two_Should_Offer_The_Flag_This_Command_Line_Takes()
{
    let step = Contract_Step("2.");
    let offered = Flags_In(&step);

    assert!(
        !offered.is_empty(),
        "AGENTS.md step 2 names no flag, so a session the loop sends there is shown a bounded \
         listing and no way past it:\n{step}"
    );

    for flag in &offered
    {
        assert_eq!(
            Work_Command_From_String_Arguments(&Arguments(&format!("list {flag}")))
                .expect("`list` takes no argument that can be wrong"),
            WorkCommand::List { state: None, scope: ListingScope::Whole },
            "AGENTS.md step 2 offers `{flag}`, and `nomos work list {flag}` does not answer \
             with the whole board"
        );
        assert!(
            Usage_Text().contains(flag.as_str()),
            "AGENTS.md step 2 offers `{flag}` and the printed usage text does not:\n{}",
            Usage_Text()
        );
    }
}

/// One numbered step of `AGENTS.md`'s loop, `marker` being the step's own `2.` opening.
///
/// Read out of the committed contract rather than out of a fixture, because the claim is about
/// the file a session actually opens. A step runs from its marker to the next line that is
/// neither the marker nor one of its indented continuations.
fn Contract_Step(marker: &str) -> String
{
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../AGENTS.md");
    let contract = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let step: Vec<&str> = contract
        .lines()
        .skip_while(|line| return !line.starts_with(marker))
        .take_while(|line| return line.starts_with(marker) || line.starts_with("   "))
        .collect();

    assert!(!step.is_empty(), "AGENTS.md carries no step opening `{marker}`");

    return step.join("\n");
}

/// Every `--flag` a text offers inside a code span.
///
/// Code spans rather than bare words, and one word rather than several: the contract writes
/// prose about the board too, and a sentence that happens to contain two dashes is not an
/// offer of a flag.
fn Flags_In(text: &str) -> Vec<String>
{
    return text
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| return span.starts_with("--") && span.split_whitespace().count() == 1)
        .map(|span| return (*span).to_owned())
        .collect();
}
