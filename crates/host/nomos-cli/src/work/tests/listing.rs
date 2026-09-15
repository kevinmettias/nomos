//! What `nomos work list` and `show` print, held against the board they print it from.

use super::super::{ItemId, LedgerDocument, LedgerItem, Listing_Label, Territory};
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
