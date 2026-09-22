//! What `nomos work list` and `show` print, held against the board they print it from.

use super::super::{
    Bounds, ExitCode, ItemId, LedgerDocument, LedgerItem, ListingScope, Listing_Label, Print_Board,
    Render_Show, ShowView, Territory,
};
use super::{Alternatives_After, FlagAlternatives, Sorted_Words};
use super::super::parse::Usage_Text;
use nomos_ledger::{ItemKind, ItemOrigin, ItemState};
use nomos_platform::Timestamp;

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
        widened: Vec::new(),
        declined: None,
    };
}

/// How far past [`LISTING_NOW`] the competing claim's lease runs when the fixture wants it
/// live — far enough that no rounding can put the two on the same side of the boundary.
const CLAIM_LEASE_SECONDS: i64 = 1_000;

/// How far before [`LISTING_NOW`] the competing claim was acquired, which is what makes it
/// the older of the two claims on the fixture board.
const CLAIM_ACQUIRED_SECONDS_BEFORE_NOW: i64 = 2_000;

/// A claim held by somebody else, live or lapsed as of [`LISTING_NOW`].
fn Listing_Claim(live: bool) -> nomos_ledger::Claim
{
    let expires = if live { LISTING_NOW + CLAIM_LEASE_SECONDS } else { LISTING_NOW - 1 };

    return nomos_ledger::Claim {
        holder: "agent-b".to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(LISTING_NOW - CLAIM_ACQUIRED_SECONDS_BEFORE_NOW),
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
    return vec![
        Labelled_First(vec![Listing_Item("a")]),
        Labelled_First(vec![Claimed("b")]),
        Labelled_First(vec![Lapsed("c")]),
        Labelled_First(vec![Blocked("d")]),
        Labelled_First(vec![Done("e")]),
        Labelled_First(vec![Declined("f")]),
        Labelled_First(vec![Waiting("g", ItemId::New("h")), Listing_Item("h")]),
        Labelled_First(vec![Waiting("i", ItemId::New("j")), Declined("j")]),
        Labelled_First(vec![Reserving_A_Shared_Path("k"), Held_Shared_Path("l")]),
        Labelled_First(vec![Unprovable("m"), Held_Shared_Path("l")]),
    ];
}

/// An item somebody else holds, inside its lease.
fn Claimed(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.state = ItemState::Claimed;
    item.claim = Some(Listing_Claim(true));

    return item;
}

/// An item somebody else held and whose lease has run out -- takeable rather than held.
fn Lapsed(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.state = ItemState::Claimed;
    item.claim = Some(Listing_Claim(false));

    return item;
}

fn Blocked(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.state = ItemState::Blocked;

    return item;
}

fn Done(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.state = ItemState::Done;

    return item;
}

fn Declined(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.Decline("superseded", "agent-a", Timestamp::From_Unix_Seconds(LISTING_NOW - 1));

    return item;
}

/// An item waiting on a dependency that has not ended, whether or not it ever will.
///
/// The dependency is an [`ItemId`] rather than a second `&str`: the two positions are both
/// strings otherwise, so a caller could transpose them and the board would carry an item
/// waiting on an id nothing else names.
fn Waiting(id: &str, dependency: ItemId) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.depends_on = vec![dependency];

    return item;
}

/// An item reserving `src/shared.rs`, which the claim beside it also reserves.
fn Reserving_A_Shared_Path(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.territory = Territory::Of_Files(vec!["src/shared.rs".to_owned()]);

    return item;
}

/// `src/shared.rs` held live by somebody else, so whatever is filed against it is contested.
fn Held_Shared_Path(id: &str) -> LedgerItem
{
    let mut item = Reserving_A_Shared_Path(id);
    item.state = ItemState::Claimed;
    item.claim = Some(Listing_Claim(true));

    return item;
}

/// An item whose territory carries a pattern, against which no comparison can be decided.
fn Unprovable(id: &str) -> LedgerItem
{
    let mut item = Listing_Item(id);
    item.territory = Territory::Of_Files(vec![format!("src/{id}.rs")]).With_Pattern("src/**/*.rs");

    return item;
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

    let listed = Alternatives_After(FlagAlternatives { usage: &Usage_Text(), flag: "--state" });

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

/// The board every bound below is drawn from: two items that have ended, and two live ones
/// that each depend on one of them.
///
/// The dependencies are the point rather than scenery. `T-2` reads `ready` only because `T-1`
/// is on the board and `Done`, and `T-4` reads `stranded` only because `T-3` is on the board
/// and `Declined` -- so a bound that removed the ended items *from the document* instead of
/// from the output would change what the two surviving rows are called. That is the mutation
/// [`Test_The_Default_Listing_Should_Label_And_Choose_Next_Over_The_Whole_Board`] performs.
fn A_Board_Half_Ended() -> LedgerDocument
{
    return LedgerDocument {
        schema_version: nomos_ledger::SCHEMA_VERSION,
        items: vec![
            Done("T-1"),
            Waiting("T-2", ItemId::New("T-1")),
            Declined("T-3"),
            Waiting("T-4", ItemId::New("T-3")),
        ],
    };
}

/// The same board with every ended item taken out of the document rather than out of the
/// output: the naive fix this change must not be, built here so a test can run it.
fn Pruned(document: &LedgerDocument) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: document.schema_version,
        items: document
            .items
            .iter()
            .filter(|item| return !item.state.Is_Finished())
            .cloned()
            .collect(),
    };
}

/// What `nomos work list` prints for `document` under `bounds`.
fn Printed(document: &LedgerDocument, bounds: Bounds<'_>) -> String
{
    let mut output = Vec::new();

    Print_Board(document, bounds, Timestamp::From_Unix_Seconds(LISTING_NOW), &mut output);

    return String::from_utf8(output).expect("Print_Board writes only str into the buffer");
}

/// The word printed in `item`'s claimability column, or nothing when no row named it.
///
/// `None` and a row with no label are different failures and this keeps them apart: the rows
/// are `<id> <label> ...`, so a row that named the item at all yields its second word.
fn Label_In<'a>(printed: &'a str, item: &str) -> Option<&'a str>
{
    return printed
        .lines()
        .find(|line| return line.starts_with(item))
        .and_then(|line| return line.split_whitespace().nth(1));
}

/// The default answers with the live board, which is the whole of what `OD-LEDGER-041` asked
/// for: 1,464 of the 1,486 rows it measured were items nobody could act on, printed by the
/// command `AGENTS.md` step 2 sends every session to first.
#[test]
fn Test_The_Default_Listing_Should_Print_Only_The_Items_That_Have_Not_Ended()
{
    let printed = Printed(&A_Board_Half_Ended(), Bounds { state: None, scope: ListingScope::Live });

    assert_eq!(Label_In(&printed, "T-2"), Some("ready"), "{printed}");
    assert_eq!(Label_In(&printed, "T-4"), Some("stranded"), "{printed}");
    assert_eq!(Label_In(&printed, "T-1"), None, "a Done item is not on the live board:\n{printed}");
    assert_eq!(
        Label_In(&printed, "T-3"),
        None,
        "a Declined item is not on the live board:\n{printed}"
    );
}

/// The control the bound is only honest with: every row is still reachable, by a spelling a
/// reader can find. A bound with no way past it would be the archive `OD-LEDGER-041` refused,
/// arrived at through the renderer instead of through the file.
#[test]
fn Test_The_Whole_Board_Should_Stay_Reachable_By_The_Scope_That_Asks_For_It()
{
    let printed = Printed(&A_Board_Half_Ended(), Bounds { state: None, scope: ListingScope::Whole });

    assert_eq!(Label_In(&printed, "T-1"), Some("done"), "{printed}");
    assert_eq!(Label_In(&printed, "T-3"), Some("declined"), "{printed}");
    assert_eq!(Label_In(&printed, "T-2"), Some("ready"), "{printed}");
    assert_eq!(Label_In(&printed, "T-4"), Some("stranded"), "{printed}");
}

/// Every label and the `next:` line are computed over the whole document, and the bound
/// governs only which rows are printed. `OD-LEDGER-023`.
///
/// The second half is the mutation, performed rather than described: the same rendering over a
/// document the ended items were pruned from. `T-2` stops being `ready` and `T-4` stops being
/// `stranded` -- both become `waiting`, because a dependency that is not on the board at all is
/// reported as unmet -- and the `next:` line stops naming anything. So the four assertions
/// above it would fail on the naive fix, and pass here only because the document reaching the
/// listing is still whole.
#[test]
fn Test_The_Default_Listing_Should_Label_And_Choose_Next_Over_The_Whole_Board()
{
    let whole = A_Board_Half_Ended();
    let bounds = Bounds { state: None, scope: ListingScope::Live };

    let printed = Printed(&whole, bounds);

    assert_eq!(Label_In(&printed, "T-2"), Some("ready"), "{printed}");
    assert_eq!(Label_In(&printed, "T-4"), Some("stranded"), "{printed}");
    assert!(printed.contains("next: T-2 "), "{printed}");

    let pruned = Printed(&Pruned(&whole), bounds);

    assert_eq!(
        Label_In(&pruned, "T-2"),
        Some("waiting"),
        "pruning the document must change this row, or the assertion above it proves nothing:\n{pruned}"
    );
    assert_eq!(
        Label_In(&pruned, "T-4"),
        Some("waiting"),
        "a declined dependency that is no longer on the board reads as merely unfinished, \
         which is the OD-LEDGER-020 distinction the bound must not cost:\n{pruned}"
    );
    assert!(
        pruned.contains("next: nothing is eligible"),
        "pruning the document must empty the eligible set, or the `next:` assertion above \
         proves nothing:\n{pruned}"
    );
}

/// The bounded listing says what it withheld and how to see it.
///
/// A spelling nobody can discover is not a bound but a hole, and this is the channel a reader
/// meets without having gone looking: it is printed by the command they already ran.
#[test]
fn Test_A_Bounded_Listing_Should_Report_How_Many_Rows_It_Withheld_And_The_Flag_That_Prints_Them()
{
    let printed = Printed(&A_Board_Half_Ended(), Bounds { state: None, scope: ListingScope::Live });

    assert!(printed.contains("2 ended items not shown"), "{printed}");
    assert!(printed.contains("--all"), "{printed}");
}

/// A listing that withheld nothing must not say it did, or the count stops being read.
#[test]
fn Test_An_Unbounded_Listing_Should_Not_Report_Withholding_Anything()
{
    let printed = Printed(&A_Board_Half_Ended(), Bounds { state: None, scope: ListingScope::Whole });

    assert!(!printed.contains("not shown"), "{printed}");
}

/// A `--state` naming a bucket that has ended answers with it, default scope or not.
///
/// Two of the ten words the usage text promises name terminal rows. A bound that swallowed
/// them would leave the help text describing filters that answer nothing.
#[test]
fn Test_A_Terminal_State_Filter_Should_Answer_Without_Asking_For_The_Whole_Board()
{
    let printed = Printed(
        &A_Board_Half_Ended(),
        Bounds { state: Some("declined"), scope: ListingScope::Live },
    );

    assert_eq!(Label_In(&printed, "T-3"), Some("declined"), "{printed}");
    assert_eq!(Label_In(&printed, "T-2"), None, "a filter still excludes what it excluded:\n{printed}");
    assert!(
        !printed.contains("next:"),
        "a filtered listing carries no `next:` line, which this bound does not change:\n{printed}"
    );
}

/// A `why` as they really arrive on this board: prose long enough that a width limit, a wrap
/// or an ellipsis anywhere in the renderer would cut it, and ending in a clause a truncating
/// implementation could not reach.
///
/// Written out as one string rather than assembled from parts, because what the assertions
/// below check is that these exact characters come back out of the verb.
const A_LONG_WHY: &str = "work show prints the title, the state, the kind, the origin, the claim \
     history with its reasons, and the verification line, and it prints neither the why nor the \
     done_when. The two fields that decide what an item is and when it is finished are therefore \
     unreachable from every verb, and the only way to read them is to parse the board file by \
     hand. The failure is quiet rather than loud, which is the whole of why it survived: a \
     session that does not notice proceeds from whatever paraphrase its instructions carried.";

/// A `done_when` at the length these routinely run to, for [`A_LONG_WHY`]'s reason.
const A_LONG_DONE_WHEN: &str = "show prints the item's why and its done_when in full, unwrapped \
     and unelided, so that the instruction to read the contract before editing can be followed \
     through the verb it routes to. A done_when is routinely several hundred words and a \
     truncated contract is worse than none, because a reader cannot tell which clause they are \
     missing, so nothing is summarized or line-clipped however long it runs. The output stays \
     readable when a field is empty and when it is enormous, and the existing lines keep their \
     order and their spelling so that a reader's habits and any script reading them still work.";

/// One item carrying a whole contract: prose in both fields, a territory of more than one
/// path, and a declared predicate.
fn An_Item_With_A_Contract() -> LedgerItem
{
    let mut item = Listing_Item("T-1");
    item.why = A_LONG_WHY.to_owned();
    item.done_when = A_LONG_DONE_WHEN.to_owned();
    item.territory = Territory::Of_Files(vec![
        "crates/host/nomos-cli/src/work/listing.rs".to_owned(),
        "crates/host/nomos-cli/src/work/tests/listing.rs".to_owned(),
    ]);
    item.verification = Some(nomos_ledger::VerificationPredicate::From_String_Arguments(vec![
        "cargo".to_owned(),
        "test".to_owned(),
        "-p".to_owned(),
        "nomos-cli".to_owned(),
    ]));

    return item;
}

/// What `nomos work show` prints for `item`, rendered through the verb rather than through any
/// one of the functions it is assembled from.
///
/// That is the point of this helper and not an incidental choice. The defect this group is
/// about was in what `show` *printed*, so a test reaching past the renderer -- one asserting
/// that the stored item holds a `done_when`, say -- would have passed unchanged on the day the
/// field was invisible, and proved nothing about the thing that was wrong.
fn Shown(item: LedgerItem) -> String
{
    let id = item.id.clone();
    let view = ShowView {
        document: LedgerDocument {
            schema_version: nomos_ledger::SCHEMA_VERSION,
            items: vec![item],
        },
        now: Timestamp::From_Unix_Seconds(LISTING_NOW),
        current_revision: None,
    };
    let mut output = Vec::new();

    let code = Render_Show(&id, Ok(view), &mut output);

    assert_eq!(code, ExitCode::Ok, "the fixture board holds the item the fixture asks for");

    return String::from_utf8(output).expect("Render_Show writes only str into the buffer");
}

/// The two fields that are the contract are printed, and printed whole.
///
/// Both halves matter and the second is the one that is easy to lose. Asserting the label
/// would pass over a renderer that printed `done_when:` and then the first eighty characters
/// of it; asserting the stored string back out, character for character, fails on a clip, on
/// an ellipsis and on a re-wrap alike.
#[test]
fn Test_Show_Should_Print_The_Items_Why_And_Done_When_Whole()
{
    let item = An_Item_With_A_Contract();

    let printed = Shown(item.clone());

    assert!(printed.contains("why:"), "{printed}");
    assert!(printed.contains("done_when:"), "{printed}");
    assert!(
        printed.contains(&item.why),
        "the why must come back out character for character, unclipped and unwrapped:\n{printed}"
    );
    assert!(
        printed.contains(&item.done_when),
        "the done_when must come back out character for character, unclipped and \
         unwrapped:\n{printed}"
    );
}

/// The other two terms the `done_when` names, in the same block as the prose.
#[test]
fn Test_Show_Should_Print_The_Territory_And_The_Declared_Predicate()
{
    let printed = Shown(An_Item_With_A_Contract());

    assert!(printed.contains("territory: 2 path(s)"), "{printed}");
    assert!(printed.contains("crates/host/nomos-cli/src/work/listing.rs"), "{printed}");
    assert!(printed.contains("crates/host/nomos-cli/src/work/tests/listing.rs"), "{printed}");
    assert!(printed.contains("verification: `cargo test -p nomos-cli`"), "{printed}");
}

/// An empty field says which field it is, rather than printing a label over a blank line.
///
/// The two absences are different: a `why` nobody wrote is a field left empty, and a missing
/// predicate is an item nothing was declared to judge. Output that showed neither would leave
/// a reader unable to tell either one from a renderer that had stopped working.
#[test]
fn Test_Show_Should_Name_A_Contract_Field_That_Is_Empty()
{
    let mut item = An_Item_With_A_Contract();
    item.why = String::new();
    item.verification = None;

    let printed = Shown(item);

    assert!(printed.contains("why: (empty)"), "{printed}");
    assert!(printed.contains("verification: (none declared)"), "{printed}");
}

/// The lines that were already printed keep their order and their spelling, at the top where
/// they were.
///
/// Other sessions read this output by habit and at least one reads it with a script, so the
/// contract is an addition and not a redesign. It is printed after these rather than before
/// them for the same reason: a `done_when` of several hundred words inserted above would push
/// every one of them off the screen.
#[test]
fn Test_Show_Should_Keep_The_Lines_It_Already_Printed_Ahead_Of_The_Contract()
{
    let printed = Shown(An_Item_With_A_Contract());
    let lines: Vec<&str> = printed.lines().collect();

    assert_eq!(lines.first().copied(), Some("T-1 item T-1"), "{printed}");
    assert_eq!(lines.get(1).copied(), Some("state: ready"), "{printed}");
    assert_eq!(lines.get(2).copied(), Some("kind: Correction  origin: Proposed"), "{printed}");
    assert!(
        printed.find("state: ") < printed.find("why:"),
        "the contract follows the lines that were already there:\n{printed}"
    );
}
