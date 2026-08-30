//! What the work verbs promise, exercised.

use super::*;
use super::parse::{Parse_Duration, Usage_Text};
use nomos_ledger::{DEFAULT_LEASE, ItemState};

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
