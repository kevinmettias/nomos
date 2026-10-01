//! Declining ends an item, and every way of getting it wrong is refused.
//!
//! `ItemState::Declined` was declared, counted terminal, and filtered on for the whole life
//! of this ledger while nothing could produce it. The one item in it was written into the
//! file by hand. `OD-LEDGER-019` records what that cost and why the remedy is a verb of its
//! own rather than a flag on `abandon`.
//!
//! Every test here has a negative control, for the reason `exclusion_holds.rs` gives: a
//! guard nobody has watched fail is a test that would pass just as happily if the thing it
//! checks were deleted.

mod fixtures;

use crate::fixtures::{
    Timestamp_From_Seconds, Board, BoardOnDisk, Board_At, ClaimRefusal, Decline_Item_For_Reason, Decline_Refused,
    Finished_Item, Give_Up, Holder, Item_Reserving_Files, ItemId, ItemState, Only_Item, Refusal_From_Claim,
    Remove_Scratch, Claim_For_Holder, NOW, REASON,
};

/// The whole point, on the shape that motivated it: an unclaimed `Ready` item.
///
/// Both items `OD-LEDGER-019` measures were released before anybody knew they were
/// superseded, so a verb that could only reach a held item could not reach either of them.
#[test]
fn Test_Declining_An_Unclaimed_Item_Should_End_It_And_Say_Who_Ended_It()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("ends-it", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);

    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-a"));

    let item = Only_Item(&ledger);
    let declination = item
        .declined
        .as_ref()
        .expect("who ended it and when must survive the transition that ended it");

    assert_eq!(
        item.state,
        ItemState::Declined {
            reason: REASON.to_owned(),
        },
        "the reason must be in the state, which is what makes it unreachable reasonlessly"
    );
    assert_eq!(declination.holder, "agent-a", "the item does not say who declined it");
    assert_eq!(declination.declined_at, Timestamp_From_Seconds(NOW), "the item does not say when");

    Remove_Scratch(&directory);
}

/// The negative control for the test above, and the reason `Declination` carries no reason
/// of its own.
///
/// One fact, one home. A reader asking *why* asks the state and a reader asking *who* asks
/// the item, and neither can be told two different things because there is only one copy.
#[test]
fn Test_A_Declination_Should_Not_Carry_A_Second_Copy_Of_The_Reason()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("one-copy", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-a"));

    let raw = std::fs::read_to_string(directory.join("ledger.json"))
        .expect("Board_At saved this file and Decline saved it again");

    assert_eq!(
        raw.matches(REASON).count(),
        1,
        "the reason is stored twice, so the two copies can come to disagree"
    );

    Remove_Scratch(&directory);
}

/// A declined item is not work, so nothing may take it.
#[test]
fn Test_A_Declined_Item_Should_Not_Be_Claimable()
{
    let BoardOnDisk { directory, mut ledger } =
        Board_At("not-claimable", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);

    // The control: it is claimable right up until it is declined, so the refusal below is
    // the decline's doing and not the fixture's.
    Claim_For_Holder(&mut ledger, "T-1", &Holder::from("agent-a"));
    Give_Up(&mut ledger, "T-1", &Holder::from("agent-a"), "stopped to check whether the successor landed it");
    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-a"));

    let refusal = Refusal_From_Claim(&mut ledger, "T-1", &Holder::from("agent-b"));

    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "{}",
        refusal.Describe()
    );
    assert!(
        !refusal.Is_Retryable(),
        "a declined item never becomes claimable, so telling an agent to retry is a loop"
    );

    Remove_Scratch(&directory);
}

/// Declining must not resurrect the claim the abandonment removed.
///
/// An item that went on excluding people after it was ended would be a worse defect than the
/// one this verb closes, which is the line `ReleaseOutcome::Record_On` already draws for
/// abandonment.
#[test]
fn Test_A_Declined_Item_Should_Stop_Excluding()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("stops-excluding", vec![
        Item_Reserving_Files("T-1", &["src/shared.rs"]),
        Item_Reserving_Files("T-2", &["src/shared.rs"]),
    ]);

    Claim_For_Holder(&mut ledger, "T-1", &Holder::from("agent-a"));

    // The control: while T-1 is held, the overlapping item is refused.
    let _refused = Refusal_From_Claim(&mut ledger, "T-2", &Holder::from("agent-b"));

    Give_Up(&mut ledger, "T-1", &Holder::from("agent-a"), "the successor reserves this ground correctly");
    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-a"));
    Claim_For_Holder(&mut ledger, "T-2", &Holder::from("agent-b"));

    Assert_The_Decline_Left_No_Claim(&ledger);

    Remove_Scratch(&directory);
}

/// A claim that survives a decline goes on excluding, and erasing the abandonment reason
/// while ending the item is the loss OD-LEDGER-006 exists to stop.
fn Assert_The_Decline_Left_No_Claim(ledger: &Board)
{
    let after = ledger
        .Load()
        .expect("the fixture wrote this board and every change since went through a verb");
    let declined = after
        .items
        .iter()
        .find(|item| item.id == ItemId::New("T-1"))
        .expect("the item survives");

    assert!(
        declined.claim.is_none(),
        "a claim that survives a decline goes on excluding"
    );
    assert_eq!(
        declined.abandoned.len(),
        1,
        "declining erased the abandonment reason, which is the loss OD-LEDGER-006 exists to stop"
    );
}

/// Ending somebody's live work is their call, and the refusal says so.
///
/// `OD-LEDGER-001`: territory is declared and not enforced, so the ledger's answer *is* the
/// exclusion. A verb that ended a held item from outside would overrule the only party that
/// knows whether the work is still running.
#[test]
fn Test_Declining_A_Held_Item_Should_Be_Refused_Retryably()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("held", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Claim_For_Holder(&mut ledger, "T-1", &Holder::from("agent-a"));

    let refusal = Decline_Refused(&mut ledger, "T-1", &Holder::from("agent-b"), REASON);

    assert!(
        refusal.Describe().contains("agent-a"),
        "the refusal must name the holder, because the remedy is a command only they can run: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Is_Retryable(),
        "the claim will be released or will lapse, so this is a queue and not a dead end"
    );
    Assert_Nothing_Was_Recorded(&ledger);

    // The remedy the refusal names, run in full: the holder releases, and then it declines.
    Give_Up(&mut ledger, "T-1", &Holder::from("agent-a"), "there is nothing here to do");
    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-b"));

    Remove_Scratch(&directory);
}

/// A refused decline leaves the item exactly as it was.
fn Assert_Nothing_Was_Recorded(ledger: &Board)
{
    let item = Only_Item(ledger);

    assert_eq!(item.state, ItemState::Claimed, "a refused decline changed the state");
    assert!(item.declined.is_none(), "a refused decline was recorded anyway");
}

/// Declining finished work would put prose where a verdict is.
///
/// `finish` exists so that a `Done` item cannot claim a check that never ran, and a decline
/// that overwrote it would hand back exactly that.
#[test]
fn Test_Declining_A_Done_Item_Should_Be_A_Conflict()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("done", vec![Finished_Item("T-1", &["src/a.rs"])]);

    let refusal = Decline_Refused(&mut ledger, "T-1", &Holder::from("agent-a"), REASON);
    let item = Only_Item(&ledger);

    assert!(
        !refusal.Is_Retryable(),
        "a Done item never stops being Done, so retrying is a loop and a person is needed"
    );
    assert_eq!(item.state, ItemState::Done, "the verdict was overwritten with prose");
    assert!(item.verified.is_some(), "the verification record was discarded");

    Remove_Scratch(&directory);
}

/// A second decline is a duplicate or a disagreement, and both are for a person.
#[test]
fn Test_Declining_A_Declined_Item_Should_Keep_The_First_Reason()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("twice", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Decline_Item_For_Reason(&mut ledger, "T-1", &Holder::from("agent-a"));

    let refusal = Decline_Refused(&mut ledger, "T-1", &Holder::from("agent-b"), "a different reading entirely");

    assert!(
        refusal.Describe().contains(REASON),
        "the refusal must report the reason already recorded, or the caller cannot tell a \
         duplicate from a disagreement: {}",
        refusal.Describe()
    );
    assert!(!refusal.Is_Retryable());
    Assert_The_Refusal_Stays_One_Line(&mut ledger);
    Assert_The_First_Reason_Survived(&ledger);

    Remove_Scratch(&directory);
}

/// The reason `P10-DERIVED-FACT` was declined with runs to five paragraphs, and this refusal
/// printed every one of them with the newlines escaped before `ItemState::Describe` existed.
fn Assert_The_Refusal_Stays_One_Line(ledger: &mut Board)
{
    let long = Decline_Refused(ledger, "T-1", &Holder::from("agent-c"), "first line\n\nand four more paragraphs");
    let opening = REASON.lines().next().expect("the standard reason has a first line");

    assert_eq!(long.Describe().lines().count(), 1, "{}", long.Describe());
    assert!(long.Describe().contains(opening));
}

/// Neither the reason nor who recorded it may be replaced by a second decline.
fn Assert_The_First_Reason_Survived(ledger: &Board)
{
    let item = Only_Item(ledger);

    assert_eq!(
        item.state,
        ItemState::Declined {
            reason: REASON.to_owned(),
        },
        "the second decline replaced the first reason"
    );
    assert_eq!(
        item.declined
            .as_ref()
            .expect("the declination survives")
            .holder,
        "agent-a",
        "the second decline replaced who ended it"
    );
}

/// A declined board is a valid board.
///
/// The failure this guards is the one `ReleaseOutcome::Finished` was reshaped for: a
/// transition that produces a state the document's own validation then refuses, so the only
/// path to it can never succeed.
#[test]
fn Test_A_Declined_Item_Should_Survive_Validation()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("valid", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    ledger
        .Validate_Current()
        .expect("a declined item must be a valid ledger, or the verb writes what nothing can read");

    Remove_Scratch(&directory);
}

/// A misspelled identifier is told it is a misspelled identifier.
#[test]
fn Test_Declining_An_Unknown_Item_Should_Name_The_Identifier()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("unknown", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);

    let refusal = ledger
        .Decline(&ItemId::New("T-2"), "agent-a", REASON)
        .expect_err("there is no such item");

    assert!(matches!(refusal, ClaimRefusal::NoSuchItem { .. }), "{}", refusal.Describe());
    assert!(refusal.Describe().contains("T-2"));

    Remove_Scratch(&directory);
}

/// The state exists to be produced, and this is the census that says so.
///
/// `OD-LEDGER-019` decision 6: `Blocked` is the other variant nothing writes, and it is
/// knowingly left that way. This test pins the half that changed, so that a decline verb
/// deleted tomorrow fails here rather than quietly returning `Declined` to the set of states
/// the ledger can describe and cannot enter.
#[test]
fn Test_Declined_Should_Be_Reachable_Through_A_Verb()
{
    let BoardOnDisk { directory, mut ledger } = Board_At("reachable", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("the verb is the only thing that may produce this state");

    let after = ledger
        .Load()
        .expect("the fixture wrote this board and every change since went through a verb");
    let item = after.items.first().expect("the item survives");

    assert!(item.state.Is_Finished(), "a declined item is not terminal");
    assert!(!item.state.Is_Claimable(), "a declined item is claimable");

    Remove_Scratch(&directory);
}
