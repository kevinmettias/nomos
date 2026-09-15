use super::*;
use crate::Claim;
use crate::ItemKind;
use crate::ItemOrigin;
use crate::Territory;

/// When the fixture below acquired its claim.
const CLAIM_ACQUIRED_AT_SECONDS: i64 = 1_000;

/// When the fixture below's claim expires.
const CLAIM_EXPIRES_AT_SECONDS: i64 = 9_000;

/// The instant every test here asks its question at, which is after
/// `CLAIM_ACQUIRED_AT_SECONDS` and before `CLAIM_EXPIRES_AT_SECONDS`.
const ASKED_AT_SECONDS: i64 = 2_000;

/// The fixture `Test_Eligible_Items_Should_Exclude_What_Claim_Refusal_Would_Refuse`
/// asserts over: a `Ready` item held back by a live overlapping claim, the item
/// contesting its territory, and an item nothing touches.
///
/// A named struct rather than a three-`LedgerItem` tuple: the tuple repeats one type
/// three times, so a caller destructuring it by position could swap two members and the
/// compiler would not notice.
struct Fixture
{
    held: LedgerItem,
    contested: LedgerItem,
    free: LedgerItem,
}

/// The property [`Eligible_Items`] exists to give a name to: a `Ready` item held back by
/// a live overlapping claim is not eligible, and one nothing contests is, in the same
/// order [`Claim_Refusal`] would decide each of them individually.
#[test]
fn Test_Eligible_Items_Should_Exclude_What_Claim_Refusal_Would_Refuse()
{
    let Fixture { held, contested, free } = Held_Contested_And_Free();
    let document = Document_Of(vec![held, contested, free]);

    let eligible: Vec<&str> = Eligible_Items(&document, Timestamp_At_Seconds(ASKED_AT_SECONDS))
        .into_iter()
        .map(|item| return item.id.As_Text())
        .collect();

    assert_eq!(
        eligible,
        vec!["P3-FREE"],
        "P1-HELD is claimed and not Ready, and P2-CONTESTED shares P1-HELD's live \
         territory -- Claim_Refusal would refuse both, and Eligible_Items must agree \
         with it rather than compute a second opinion"
    );
}

fn Held_Contested_And_Free() -> Fixture
{
    let mut held = Item_Named("P1-HELD");
    held.territory = Territory::Of_Files(["a/shared.rs"]);
    held.state = ItemState::Claimed;
    held.claim = Some(Claim {
        holder: "agent-a".to_owned(),
        acquired_at: Timestamp_At_Seconds(CLAIM_ACQUIRED_AT_SECONDS),
        lease_expires_at: Timestamp_At_Seconds(CLAIM_EXPIRES_AT_SECONDS),
    });

    let mut contested = Item_Named("P2-CONTESTED");
    contested.territory = Territory::Of_Files(["a/shared.rs"]);

    let mut free = Item_Named("P3-FREE");
    free.territory = Territory::Of_Files(["b/other.rs"]);

    return Fixture { held, contested, free };
}

/// The tie-break `OD-LEDGER-023` wrote down: id order, not the order items happen to
/// sit in the document.
#[test]
fn Test_Eligible_Items_Should_Order_By_Id_Rather_Than_Document_Order()
{
    let document = Document_Of(vec![Item_Named("P9-LATER"), Item_Named("P1-EARLIER"), Item_Named("P5-MIDDLE")]);

    let eligible: Vec<&str> = Eligible_Items(&document, Timestamp_At_Seconds(ASKED_AT_SECONDS))
        .into_iter()
        .map(|item| return item.id.As_Text())
        .collect();

    assert_eq!(eligible, vec!["P1-EARLIER", "P5-MIDDLE", "P9-LATER"]);
}

#[test]
fn Test_Eligible_Items_Should_Be_Empty_Over_An_Empty_Board()
{
    let document = Document_Of(Vec::new());

    assert!(Eligible_Items(&document, Timestamp_At_Seconds(ASKED_AT_SECONDS)).is_empty());
}

#[test]
fn Test_Claim_Refusal_Should_Report_No_Such_Item_When_Absent()
{
    let document = Document_Of(Vec::new());

    let refusal = Claim_Refusal(&document, &ItemId::New("GHOST"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

    assert_eq!(refusal, Some(ClaimRefusal::NoSuchItem { item: ItemId::New("GHOST") }));
}

#[test]
fn Test_Decline_Refusal_Should_Refuse_An_Item_Someone_Else_Is_Holding()
{
    let document = Document_With_A_Held_Claim();

    let refusal = Decline_Refusal(&document, &ItemId::New("P1-HELD"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

    assert!(
        matches!(refusal, Some(ClaimRefusal::StillHeld { .. })),
        "got {refusal:?}"
    );
}

#[test]
fn Test_Takeover_Refusal_Should_Refuse_A_Verb_Used_On_A_Live_Claim()
{
    let document = Document_With_A_Held_Claim();

    // The claim above expires at CLAIM_EXPIRES_AT_SECONDS; asking before that means it
    // has not lapsed, so a takeover is the wrong verb rather than a valid recovery.
    let refusal = Takeover_Refusal(&document, &ItemId::New("P1-HELD"), Timestamp_At_Seconds(ASKED_AT_SECONDS));

    assert!(
        matches!(refusal, Some(ClaimRefusal::HeldBy { .. })),
        "a live claim asked about with the recovery verb must read as held, not takeable: {refusal:?}"
    );
}

/// A board holding exactly one item, `P1-HELD`, live-claimed by `agent-a` for the
/// duration `[CLAIM_ACQUIRED_AT_SECONDS, CLAIM_EXPIRES_AT_SECONDS)`.
///
/// Both refusal tests above ask a different verb about the same claimed item, so they
/// share this one arrangement rather than each rebuilding it -- a change to what "held"
/// means here now changes for both at once instead of silently drifting between two
/// independently hand-written copies.
fn Document_With_A_Held_Claim() -> LedgerDocument
{
    let mut held = Item_Named("P1-HELD");
    held.state = ItemState::Claimed;
    held.claim = Some(Claim {
        holder: "agent-a".to_owned(),
        acquired_at: Timestamp_At_Seconds(CLAIM_ACQUIRED_AT_SECONDS),
        lease_expires_at: Timestamp_At_Seconds(CLAIM_EXPIRES_AT_SECONDS),
    });
    return Document_Of(vec![held]);
}

fn Item_Named(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "an item".to_owned(),
        why: "because".to_owned(),
        done_when: "when it is done".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Empty(),
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

fn Timestamp_At_Seconds(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

fn Document_Of(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: crate::SCHEMA_VERSION,
        items,
    };
}
