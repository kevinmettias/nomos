//! What the work verbs promise, exercised.

use super::*;
use super::parse::{Parse_Duration, Usage_Text};
use nomos_ledger::{DEFAULT_LEASE, ItemKind, ItemOrigin, ItemState};
use nomos_platform::Timestamp;

fn Arguments(text: &str) -> Vec<String>
{
    return text.split_whitespace().map(str::to_owned).collect();
}

#[test]
fn Test_Claim_Should_Parse_With_A_Default_Lease()
{
    let parsed = Work_Command_From_String_Arguments(&Arguments("claim --item T-1 --holder agent-a")).unwrap();

    assert_eq!(
        parsed,
        WorkCommand::Claim(ClaimRequest {
            item: ItemId::New("T-1"),
            holder: "agent-a".to_owned(),
            lease: DEFAULT_LEASE,
        })
    );
}

/// `takeover` shares `claim`'s parse, so it must share `claim`'s defaulting too. A verb
/// that silently required `--lease` where its two siblings default it is a verb agents
/// learn by being refused.
#[test]
fn Test_Takeover_Should_Parse_With_A_Default_Lease()
{
    let parsed = Work_Command_From_String_Arguments(&Arguments("takeover --item T-1 --holder agent-b")).unwrap();

    assert_eq!(
        parsed,
        WorkCommand::TakeOver(ClaimRequest {
            item: ItemId::New("T-1"),
            holder: "agent-b".to_owned(),
            lease: DEFAULT_LEASE,
        })
    );
}

/// One word, no hyphen, matching the other nine verbs — and not a spelling of `claim`.
///
/// The second assertion is the one worth making: the shared parse arm hands three verbs
/// the same arguments, so the only thing keeping them distinct is which variant it
/// returns. A `takeover` that parsed as a `Claim` would refuse every lapsed item while
/// appearing to be the remedy for one.
#[test]
fn Test_Takeover_Should_Not_Parse_As_A_Claim()
{
    let parsed = Work_Command_From_String_Arguments(&Arguments("takeover --item T-1 --holder agent-b --lease 30m")).unwrap();

    assert!(matches!(parsed, WorkCommand::TakeOver { .. }), "{parsed:?}");
    assert!(Work_Command_From_String_Arguments(&Arguments("takeover --holder agent-b")).is_err());
    assert!(Work_Command_From_String_Arguments(&Arguments("takeover --item T-1")).is_err());
}

/// A lease with no unit is ambiguous. Guessing seconds would silently give an agent
/// a two-second lease when it asked for two hours.
///
/// Each input fails inside `Parse_Duration` for a different reason, and the assertion
/// names which: `"2"` and `""` both leave nothing before the character `split_at`
/// carves off as the unit, so the *number* fails to parse; `"2d"` parses a number fine
/// and fails because `d` is not a unit `Parse_Duration` recognises. A bare `.is_err()`
/// would keep passing the day either message stopped naming its own reason — including
/// the day the two swapped, which is exactly the confusion a shared "it failed" hides.
#[test]
fn Test_A_Unitless_Lease_Should_Be_Refused()
{
    let no_digits = Parse_Duration("2").unwrap_err();
    assert!(
        no_digits.contains("is not a duration"),
        "a single character leaves nothing before the unit to parse as a number: {no_digits}"
    );

    let empty = Parse_Duration("").unwrap_err();
    assert!(
        empty.contains("is not a duration"),
        "an empty string leaves nothing to parse as a number either: {empty}"
    );

    let unrecognized_unit = Parse_Duration("2d").unwrap_err();
    assert!(
        unrecognized_unit.contains("has no unit"),
        "a number with an unrecognized unit character must name the unit as the problem: \
         {unrecognized_unit}"
    );
}

/// Every verb's own required arguments, and the flag missing from each.
fn Missing_Required_Arguments() -> Vec<(&'static str, &'static str)>
{
    return vec![
        ("claim --holder agent-a", "--item"),
        ("claim --item T-1", "--holder"),
        ("show", "--item"),
        ("finish --item T-1", "--holder"),
        ("finish --holder agent-a", "--item"),
        ("abandon --item T-1 --holder agent-a", "--reason"),
    ];
}

/// A missing required argument must name what is missing, not just print usage.
#[test]
fn Test_A_Missing_Argument_Should_Name_Itself()
{
    for (arguments, flag) in Missing_Required_Arguments()
    {
        let error = Work_Command_From_String_Arguments(&Arguments(arguments)).unwrap_err();

        assert!(
            error.contains(flag),
            "`{arguments}` is missing {flag} and the refusal does not name it: {error}"
        );
    }
}

/// One word, no hyphen, matching the other ten verbs — and not a spelling of `abandon`.
///
/// The second half is the assertion worth making. These two take the same three
/// arguments and do opposite things to the item: a `decline` that parsed as an `Abandon`
/// would put the item back on the board while reporting that it had been ended, which is
/// exactly the state `OD-LEDGER-019` was written to stop an item being left in.
#[test]
fn Test_Decline_Should_Not_Parse_As_An_Abandon()
{
    let parsed = Work_Command_From_String_Arguments(&Arguments("decline --item T-1 --holder agent-a --reason done"))
        .unwrap();

    assert_eq!(
        parsed,
        WorkCommand::Decline(EndingRequest {
            item: ItemId::New("T-1"),
            holder: "agent-a".to_owned(),
            reason: "done".to_owned(),
        })
    );
}

/// The state carries its reason so that an item cannot be declined reasonlessly, and the
/// flag is required so that the guarantee reaches somebody typing rather than stopping at
/// the type.
#[test]
fn Test_Decline_Should_Require_A_Reason()
{
    let error =
        Work_Command_From_String_Arguments(&Arguments("decline --item T-1 --holder agent-a")).unwrap_err();

    assert!(error.contains("--reason"), "{error}");
    assert!(Work_Command_From_String_Arguments(&Arguments("decline --holder agent-a --reason r")).is_err());
    assert!(Work_Command_From_String_Arguments(&Arguments("decline --item T-1 --reason r")).is_err());
}

fn Added(text: &str) -> LedgerItem
{
    return match Work_Command_From_String_Arguments(&Arguments(text)).unwrap()
    {
        WorkCommand::Add { item, .. } => *item,
        // Every call site hands this an `add` line, so the variant is fixed by construction
        // and the only thing a fallible signature would buy is an `unwrap` at each of them.
        // The other variant is printed because the useful failure is which command the
        // parser chose instead — that names the flag it started reading differently.
        other => panic!("expected an add, got {other:?}"),
    };
}

#[test]
fn Test_Add_Should_Collect_Repeated_Territory_Flags()
{
    let item = Added(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs --territory src/b.rs",
    );

    assert_eq!(item.territory.paths, vec!["src/a.rs", "src/b.rs"]);
    assert_eq!(item.state, ItemState::Ready);
}

/// The command and the declaration it carries beside the item.
struct AddedItem
{
    item: LedgerItem,
    amending: Territory,
}

/// The command and the declaration it carries beside the item.
fn Add_Of(text: &str) -> AddedItem
{
    return match Work_Command_From_String_Arguments(&Arguments(text)).unwrap()
    {
        WorkCommand::Add { item, amending } => AddedItem {
            item: *item,
            amending,
        },
        // `amending` exists only on this variant, so there is no degraded `AddedItem` to
        // return from any other one — a caller would have nothing to inspect. That matters
        // here because the assertion below is that `--amends` reserved a path, and a helper
        // that fell back to an empty declaration would report the guard as working.
        other => panic!("expected an add, got {other:?}"),
    };
}

/// `--amends` reserves the record it declares, so an author writes it once.
///
/// The alternative was requiring `--territory` beside it, and that makes the dangerous
/// omission the easy one: an item that declared an amendment without reserving the file
/// would edit a record with nothing keeping a second writer off it, which is the whole
/// reason the guard has to admit amendments rather than refuse them.
#[test]
fn Test_Amends_Should_Reserve_The_Record_It_Declares()
{
    let AddedItem { item, amending } = Add_Of(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --amends docs/records/ARC-HARNESS-001-a-slug.md",
    );

    assert_eq!(
        item.territory.paths,
        vec!["docs/records/ARC-HARNESS-001-a-slug.md"],
        "a declared amendment must reserve the file it edits"
    );
    assert_eq!(
        amending.paths, item.territory.paths,
        "the declaration must reach the store, not only the reservation"
    );
}

/// An amendment satisfies the reservation requirement on its own.
///
/// `--territory` is required because an item that reserves nothing excludes nobody, and
/// an item that only amends is not that item — it reserves exactly one file and excludes
/// every other writer of it.
#[test]
fn Test_An_Item_That_Only_Amends_Should_Not_Be_Refused_As_Reserving_Nothing()
{
    assert!(
        Work_Command_From_String_Arguments(&Arguments(
            "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
             --amends docs/records/ARC-HARNESS-001-a-slug.md",
        ))
        .is_ok(),
        "an item reserving a record through --amends reserves something"
    );
}

/// Territory and amendments combine, and only the declared ones are declared.
///
/// The case that distinguishes reserving from declaring. An item amending one record
/// while editing ordinary code reserves both and says only the record is an amendment;
/// a declaration that swept in the code would be claiming the repository had published
/// a source file.
#[test]
fn Test_Amends_Should_Declare_Only_What_It_Names()
{
    let AddedItem { item, amending } = Add_Of(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs \
         --amends docs/records/ARC-HARNESS-001-a-slug.md",
    );

    assert_eq!(
        item.territory.paths,
        vec!["src/a.rs", "docs/records/ARC-HARNESS-001-a-slug.md"]
    );
    assert_eq!(
        amending.paths,
        vec!["docs/records/ARC-HARNESS-001-a-slug.md"]
    );
}

/// The predicate is everything after `--`, so a command carrying its own flags needs
/// no quoting and no escaping — and the named arguments before the separator are
/// still parsed normally.
#[test]
fn Test_Add_Should_Take_The_Predicate_After_The_Separator()
{
    let item = Added(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs \
         -- cargo test -p nomos-spec-model --lib",
    );

    assert_eq!(
        item.verification.map(|predicate| predicate.argv),
        Some(vec![
            "cargo".to_owned(),
            "test".to_owned(),
            "-p".to_owned(),
            "nomos-spec-model".to_owned(),
            "--lib".to_owned(),
        ])
    );
}

/// `--timeout` overrides the predicate's default ten-minute bound.
#[test]
fn Test_Timeout_Should_Bound_The_Predicate()
{
    let item = Added(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs --timeout 40m \
         -- cargo test -p nomos-workspace",
    );

    assert_eq!(
        item.verification.map(|predicate| predicate.timeout_seconds),
        Some(2_400)
    );
}

/// Left unset, the predicate keeps the ten-minute default `VerificationPredicate::From_String_Arguments`
/// gives it — `--timeout` overrides, it does not replace, the construction.
#[test]
fn Test_Timeout_Should_Default_When_Omitted()
{
    let item = Added(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs \
         -- cargo test -p nomos-workspace",
    );

    assert_eq!(
        item.verification.map(|predicate| predicate.timeout_seconds),
        Some(600)
    );
}

/// `add` lines that name `--timeout` with no predicate after `--` to bound.
fn Timeouts_With_No_Predicate_To_Bound() -> Vec<&'static str>
{
    return vec![
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs --timeout 40m",
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs --timeout 2h",
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs --timeout 10s",
    ];
}

/// A bound with nothing to bound is a mistake, not a no-op: an author who typed
/// `--timeout` meant to give the predicate a longer wall bound, and silently discarding it
/// because there is no predicate would leave that author believing the item is more
/// patient than it is.
#[test]
fn Test_Timeout_Without_A_Predicate_Should_Be_Refused()
{
    for arguments in Timeouts_With_No_Predicate_To_Bound()
    {
        let error = Work_Command_From_String_Arguments(&Arguments(arguments)).unwrap_err();

        assert!(
            error.contains("--timeout"),
            "the refusal must name the flag that has nothing to bound: {error}"
        );
    }
}

/// A flag that looks like a named argument but sits after the separator belongs to
/// the predicate. Without the split, `--lib` above would be read as an option to
/// `nomos work`.
#[test]
fn Test_Arguments_After_The_Separator_Should_Not_Be_Read_As_Options()
{
    let item = Added(
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed \
         --territory src/a.rs \
         -- prog --title stolen",
    );

    assert_eq!(item.title, "t");
}

/// `add` lines that name no territory at all, by any route.
fn Adds_That_Reserve_Nothing() -> Vec<&'static str>
{
    return vec![
        "add --item T-1 --title t --why w --done-when d",
        "add --item T-1 --title t --why w --done-when d --kind correction --origin proposed",
        "add --item T-1 --title t --why w --done-when d --depends-on T-0",
    ];
}

/// An item that reserves nothing excludes nobody, so the ledger would hand two
/// agents the same files and call it disjoint.
#[test]
fn Test_Add_Should_Refuse_An_Item_With_No_Territory()
{
    for arguments in Adds_That_Reserve_Nothing()
    {
        let error = Work_Command_From_String_Arguments(&Arguments(arguments)).unwrap_err();

        assert!(error.contains("--territory"), "`{arguments}`: {error}");
    }
}

/// The flag `OD-LEDGER-013` withdrew.
///
/// A pattern reaches `Territory::Intersect`, which short-circuits to `Unknown` before
/// comparing a single path, and `Unknown` refuses non-retryably. So this one flag made an
/// item nobody could claim *and* refused every other claim on the board, and told each
/// refused agent to stop and fetch a person rather than try another item.
///
/// Pinned as a **usage** error rather than a ledger one: the mistake is in what was
/// typed, and an exit code of 1 or 5 here would read as the board being broken, which is
/// the confusion this item exists to remove.
#[test]
fn Test_Add_Should_Refuse_A_Territory_Pattern()
{
    let error = Work_Command_From_String_Arguments(&Arguments(
        "add --item T-1 --title t --why w --done-when d \
         --territory src/a.rs --territory-pattern crates/spec/**",
    ))
    .unwrap_err();

    assert!(
        error.contains("--territory-pattern is not supported"),
        "the refusal must name the flag that is unsupported: {error}"
    );
    assert!(
        error.contains("crates/spec/**"),
        "and quote the pattern it refused, or an author cannot tell which one: {error}"
    );
}

/// `add` lines naming only a `--territory-pattern`, with no `--territory` beside it.
fn Territory_Patterns_Given_Alone() -> Vec<&'static str>
{
    return vec![
        "add --item T-1 --title t --why w --done-when d --territory-pattern crates/**",
        "add --item T-1 --title t --why w --done-when d --territory-pattern crates/spec/**",
        "add --item T-1 --title t --why w --done-when d --territory-pattern docs/**",
    ];
}

/// The pattern is refused even when it is the only territory given.
///
/// The ordering control. `--territory` was checked first before this item, so a pattern
/// on its own reported "an item that reserves nothing excludes nobody" — a true sentence
/// about the wrong problem, which sends the author to add a path rather than to drop the
/// flag. Swapping the two checks back turns this red while the test above stays green.
#[test]
fn Test_A_Pattern_Alone_Should_Be_Refused_As_A_Pattern()
{
    for arguments in Territory_Patterns_Given_Alone()
    {
        let error = Work_Command_From_String_Arguments(&Arguments(arguments)).unwrap_err();

        assert!(
            error.contains("--territory-pattern is not supported"),
            "a pattern alone must be refused for being a pattern: {error}"
        );
    }
}

/// The flag is gone from the usage text as well as from the parser.
///
/// `done_when` requires the flag to leave the advertised surface, and the usage string is
/// the copy an agent actually reads — it is printed on every refusal above.
#[test]
fn Test_The_Usage_Text_Should_Not_Advertise_A_Territory_Pattern()
{
    assert!(
        !Usage_Text().contains("--territory-pattern"),
        "the usage text still advertises a flag that is refused: {}",
        Usage_Text()
    );
}

#[test]
fn Test_Finish_Should_Work_Command_From_String_Arguments()
{
    assert_eq!(
        Work_Command_From_String_Arguments(&Arguments("finish --item T-1 --holder agent-a")).unwrap(),
        WorkCommand::Finish {
            item: ItemId::New("T-1"),
            holder: "agent-a".to_owned(),
        }
    );
}

/// The alternatives a `--flag a|b|c` segment of the help text lists.
///
/// `OD-AGENT-004` version 2 asks a printed vocabulary to be compared against its authority in
/// both directions. That comparison needs the printed side as data, and the printed side is a
/// pipe-joined run of words ending at the first whitespace — `[--state a|b]` closes with a
/// bracket, which comes off here so a caller compares words rather than punctuation.
///
/// The first occurrence wins, which is what the callers want: every flag below is spelled bare
/// in its own usage line and backticked (`` `--kind` ``) in the prose underneath, and the bare
/// one is the line a reader copies from.
fn Alternatives_After(usage: &str, flag: &str) -> Vec<String>
{
    let Some((_, rest)) = usage.split_once(&format!("{flag} "))
    else
    {
        return Vec::new();
    };
    let listed = rest.split_whitespace().next().unwrap_or_default().trim_end_matches(']');

    return listed.split('|').map(str::to_owned).collect();
}

/// A list of words in ascending order, so that two of them can be compared as sets.
fn Sorted_Words(words: impl Iterator<Item = String>) -> Vec<String>
{
    let mut sorted: Vec<String> = words.collect();
    sorted.sort();
    sorted.dedup();

    return sorted;
}

/// The instant every state fixture below treats as now.
const LISTING_NOW: i64 = 2_000;

/// A bare `Ready` item reserving one file named after it, and nothing else.
///
/// Every fixture below starts here and changes exactly the one thing whose label it is after,
/// so a fixture that produces the wrong word is wrong about that one thing rather than about
/// its whole shape.
fn Listing_Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("item {id}"),
        why: "because".to_owned(),
        done_when: "it prints".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files(vec![format!("src/{id}.rs")]),
        state: ItemState::Ready,
        depends_on: Vec::new(),
        blocked: None,
        claim: None,
        verification: None,
        verified: None,
        abandoned: Vec::new(),
        displaced: Vec::new(),
        declined: None,
    };
}

/// A claim held by somebody else, live or lapsed as of [`LISTING_NOW`].
fn Listing_Claim(live: bool) -> nomos_ledger::Claim
{
    let expires = if live { LISTING_NOW + 1_000 } else { LISTING_NOW - 1 };

    return nomos_ledger::Claim {
        holder: "agent-b".to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(LISTING_NOW - 2_000),
        lease_expires_at: Timestamp::From_Unix_Seconds(expires),
    };
}

/// What `nomos work list` would call the first item on a board of `items`.
fn Labelled_First(items: Vec<LedgerItem>) -> &'static str
{
    let document = LedgerDocument {
        schema_version: nomos_ledger::SCHEMA_VERSION,
        items,
    };
    let first = document.items.first().expect("the fixture has an item");

    return Listing_Label(&document, first, Timestamp::From_Unix_Seconds(LISTING_NOW));
}

/// Every word `nomos work list` can print in its claimability column, produced by running
/// [`Listing_Label`] rather than by copying the two functions that decide it.
///
/// This is the authority `--state` is compared against, and it is deliberately observed rather
/// than restated. `Listing_Label` answers out of `State_Label` (an exhaustive match over
/// [`ItemState`], in `listing.rs`) or `Refusal_Label` (a match over `ClaimRefusal` in
/// `report.rs`), and copying either list here would be the second authority `OD-AGENT-004` is
/// about. So each fixture below is a board that really produces one word, and the word is
/// whatever the function says it is.
///
/// **Why ten is the whole set, and not merely ten the author thought of.** The two ranges are
/// closed from opposite directions. `State_Label`'s match is exhaustive over `ItemState`, so a
/// sixth state fails `listing.rs` to compile — and the `match` in [`Listing_Item`]'s caller
/// below fails this file too, which is what brings an author here. `Refusal_Label`'s match ends
/// in a `(_, _)` catch-all returning `snagged`, so a new `ClaimRefusal` variant *cannot* add a
/// word: adding one means writing a new arm, in the function this list sits beside.
fn Every_Listing_Label() -> Vec<&'static str>
{
    let mut claimed = Listing_Item("b");
    claimed.state = ItemState::Claimed;
    claimed.claim = Some(Listing_Claim(true));

    let mut lapsed = Listing_Item("c");
    lapsed.state = ItemState::Claimed;
    lapsed.claim = Some(Listing_Claim(false));

    let mut blocked = Listing_Item("d");
    blocked.state = ItemState::Blocked;

    let mut done = Listing_Item("e");
    done.state = ItemState::Done;

    let mut declined = Listing_Item("f");
    declined.Decline("superseded", "agent-a", Timestamp::From_Unix_Seconds(LISTING_NOW - 1));

    let mut waiting = Listing_Item("g");
    waiting.depends_on = vec![ItemId::New("h")];

    let mut stranded = Listing_Item("i");
    stranded.depends_on = vec![ItemId::New("j")];
    let mut declined_dependency = Listing_Item("j");
    declined_dependency.Decline("not work", "agent-a", Timestamp::From_Unix_Seconds(LISTING_NOW - 1));

    let mut contested = Listing_Item("k");
    contested.territory = Territory::Of_Files(vec!["src/shared.rs".to_owned()]);
    let mut holder = Listing_Item("l");
    holder.territory = Territory::Of_Files(vec!["src/shared.rs".to_owned()]);
    holder.state = ItemState::Claimed;
    holder.claim = Some(Listing_Claim(true));

    let mut unprovable = Listing_Item("m");
    unprovable.territory = Territory::Of_Files(vec!["src/m.rs".to_owned()]).With_Pattern("src/**/*.rs");

    return vec![
        Labelled_First(vec![Listing_Item("a")]),
        Labelled_First(vec![claimed]),
        Labelled_First(vec![lapsed]),
        Labelled_First(vec![blocked]),
        Labelled_First(vec![done]),
        Labelled_First(vec![declined]),
        Labelled_First(vec![waiting, Listing_Item("h")]),
        Labelled_First(vec![stranded, declined_dependency]),
        Labelled_First(vec![contested, holder.clone()]),
        Labelled_First(vec![unprovable, holder]),
    ];
}

/// The states `work list --state` prints are the words the listing can actually print.
///
/// This is the case that was wrong rather than merely unguarded. The printed list held nine
/// words and `Listing_Label` produces ten: `lapsed` was missing, and it is not a word a reader
/// can do without, because `Listed_As` filters by comparing the requested string against this
/// same function's output — so `nomos work list --state lapsed` worked, and the help text did
/// not say so. `OD-LEDGER-012` is the record that gave `lapsed` its own word in the first
/// place, for the reason that a lapsed item and a merely unclaimable one have opposite
/// remedies.
///
/// The fixture set is asserted to be ten distinct words before it is compared against
/// anything. Without that, a fixture that silently stopped producing its word would shrink
/// both sides of a set comparison and pass.
#[test]
fn Test_The_Listed_States_Should_Be_Every_Word_The_Listing_Can_Print()
{
    let observed = Every_Listing_Label();

    assert_eq!(
        Sorted_Words(observed.iter().map(|word| return (*word).to_owned())).len(),
        observed.len(),
        "two fixtures produced the same word, so this covers fewer labels than it claims: \
         {observed:?}"
    );

    let listed = Alternatives_After(&Usage_Text(), "--state");

    assert!(
        !listed.is_empty(),
        "no --state alternative was parsed out of the usage text, so this compared nothing: {}",
        Usage_Text()
    );
    assert_eq!(
        Sorted_Words(listed.into_iter()),
        Sorted_Words(observed.iter().map(|word| return (*word).to_owned())),
        "the usage text and the listing disagree about what an item can be called"
    );
}

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

/// An `add` line carrying one `--kind` and one `--origin`, so a spelling can be parsed back.
fn Add_Line(kind: &str, origin: &str) -> String
{
    return format!(
        "add --item T-1 --title t --why w --done-when d --kind {kind} --origin {origin} \
         --territory src/a.rs"
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
    let listed = Alternatives_After(&Usage_Text(), "--kind");

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

    for spelling in &listed
    {
        let item = Added(&Add_Line(spelling, "proposed"));
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
    let listed = Alternatives_After(&Usage_Text(), "--origin");

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

    for spelling in &listed
    {
        let item = Added(&Add_Line("correction", spelling));
        let named = Every_Item_Origin()
            .iter()
            .find(|origin| return Spelled_Origin(**origin) == spelling.as_str())
            .expect("the set comparison above already established the spelling is one of these");

        assert_eq!(item.origin, *named, "--origin {spelling} does not parse to the origin spelled for it here");
    }
}

/// Every code this group can leave the process with.
///
/// `work::ExitCode` carries no census of its own the way `check::ExitCode::All()` does,
/// and its declaration is not this item's territory, so the census is here. [`Labelled`]'s
/// match has no wildcard arm, so a variant added to [`ExitCode`] fails this file to
/// *compile* rather than to pass — that is the forcing function that brings an author to
/// this array in the same edit. It is honestly weaker than a census living beside the
/// declaration, and it is what a test file can do without editing what it judges.
fn Every_Exit_Code() -> &'static [ExitCode]
{
    return &[
        ExitCode::Ok,
        ExitCode::ValidationError,
        ExitCode::Usage,
        ExitCode::ClaimUnavailable,
        ExitCode::Conflict,
        ExitCode::StoreError,
    ];
}

/// A code's name, as an exhaustive match, so that adding one stops the build here.
fn Labelled(code: ExitCode) -> &'static str
{
    return match code
    {
        ExitCode::Ok => "Ok",
        ExitCode::ValidationError => "ValidationError",
        ExitCode::Usage => "Usage",
        ExitCode::ClaimUnavailable => "ClaimUnavailable",
        ExitCode::Conflict => "Conflict",
        ExitCode::StoreError => "StoreError",
    };
}

/// A list of codes in ascending order, so that two of them can be compared as sets.
fn Sorted(codes: impl Iterator<Item = i32>) -> Vec<i32>
{
    let mut sorted: Vec<i32> = codes.collect();
    sorted.sort_unstable();

    return sorted;
}

/// The codes this group's help text documents are the codes this group can exit with.
///
/// The same comparison `check` and `gate` have each carried for a while, against this
/// group's own enum. Eight groups print an exit-code list and only those two mirrored it;
/// the other six were correct rather than guarded, which is a different thing, and
/// `OD-AGENT-004`'s amendment says a printed vocabulary is admissible only where a test
/// compares it against its authority. The usage text is prose a person reads and
/// [`ExitCode`] is what the process returns, the two were written separately, and a code
/// added or renumbered in one of them and not the other is the failure that actually
/// happens — and this group's codes are the ones agents branch on rather than parsing
/// output, which is `ExitCode`'s own doc's reason for existing.
#[test]
fn Test_The_Documented_Exit_Codes_Should_Be_The_Ones_This_Group_Can_Exit_With()
{
    let usage = Usage_Text();

    assert!(
        usage.starts_with("usage: nomos work"),
        "this compared some other group's help text: {usage}"
    );

    let (_, spelled) = usage
        .split_once("exit codes:")
        .expect("the usage text documents the exit codes");
    let documented = Sorted(spelled.split_whitespace().filter_map(|word| return word.parse().ok()));
    let implemented = Sorted(Every_Exit_Code().iter().map(|code| return code.Value()));

    assert!(
        !documented.is_empty(),
        "no exit code was parsed out of the usage text, so this compared nothing: {spelled}"
    );
    assert_eq!(
        documented,
        implemented,
        "the usage text and ExitCode disagree about what this command can exit with; the \
         enum declares {:?}",
        Every_Exit_Code().iter().map(|code| return Labelled(*code)).collect::<Vec<_>>()
    );
}
