//! A lease that ran out, and what the rest of the board may still do.
//!
//! A lapsed claim must not brick the ledger. One agent dying holding one item is the
//! ordinary case, and a board that stops until somebody intervenes has turned a routine
//! failure into an outage.

use crate::board::{
    Timestamp_From_Seconds, AT_LATER_SECONDS, After_The_Lapse, Board, Board_At, ClaimRefusal, Claimant, ExclusionLedger,
    Holder_Of_Item, Item_Reserving_Files, ItemId, ItemState, LEASE, LEASE_ENDS_AT, LedgerItem, NOW, Only_Item,
    Refusal_From_Claim, Standing, Standing_Of, Claim_For_Holder, Take_Over_In, Timestamp, Validate_Document,
};

/// The defect, and the reason it needed an experiment rather than a reading.
///
/// Validation used to call `Claimed` with no *active* claim a violation, and a lease
/// expiring is precisely that. So a document written valid stopped being valid on its own,
/// `Save` refuses an invalid document, and every claim saves — which meant one lapsed lease
/// refused every claim on the board, including items sharing no territory with it. The
/// lease expiring caused exactly what `MAXIMUM_LEASE` exists to prevent.
///
/// The unit level was right the whole time and that is what hid it: `Has_Active_Claim`
/// returns false on a lapsed claim, exclusion honours that, and validation refused the
/// document before exclusion was ever consulted.
#[test]
fn Test_A_Lapsed_Lease_Should_Not_Stop_The_Rest_Of_The_Board()
{
    let Board { directory, mut ledger } = Board_At("lapse-bricks", vec![
        Item_Reserving_Files("T-1", &["src/a.rs"]),
        Item_Reserving_Files("T-2", &["src/b.rs"]),
    ]);
    Claim_For_Holder(&mut ledger, "T-1", Claimant("dead-agent"));

    let mut after = After_The_Lapse(directory.As_Path());

    // One: the document is not called invalid because time passed.
    assert_eq!(
        Validate_Document(&after.Load().expect("the file is still readable"), Timestamp_From_Seconds(AT_LATER_SECONDS)),
        Vec::<String>::new(),
        "a lapsed lease made the whole document invalid, so nothing can be written to it"
    );
    after
        .Validate_Current()
        .expect("validate must not call a board with a lapsed lease broken");
    // Two: an unrelated item is still claimable. `src/b.rs` shares nothing with `src/a.rs`,
    // so a refusal here is not exclusion — it is the board refusing to be written at all.
    Claim_For_Holder(&mut after, "T-2", Claimant("agent-b"));
}

/// Three: what the lapsed item itself does, which is a decision rather than a consequence.
///
/// It stays `Claimed` and a plain `claim` on it is refused. `OD-LEDGER-009` states the
/// grounds: `Claim` overwrites `claim`, and `claim` is the only thing recording that the work
/// was ever started — which `OD-LEDGER-006` decided must survive, having refused to synthesize
/// an `Abandonment` for a lapse because `Abandonment::reason` is the words the holder gave
/// and a lapse has none. Taking a lapsed item over is a different operation from claiming a
/// free one, and `OD-LEDGER-012` is where it became one.
///
/// Asserted rather than left implicit, because the refusal is now deliberate. What must not
/// happen is that it becomes claimable by accident and quietly erases who was working on it.
///
/// # What changed here, and why it is this test working rather than a regression
///
/// This test was `…Should_Refuse_A_New_Holder_And_Keep_The_Old_One_Visible`, and "refuse a new
/// holder" became false once `takeover` existed: a new holder is exactly what a takeover
/// installs. Its subject — `claim` is refused — is **not** reversed, and the refusal is
/// stronger than it was, because `Lapsed` names the holder and the remedy where `NotClaimable`
/// said only that the item was `Claimed`. The assertion that the lapsed claim was not replaced
/// is kept word for word: nothing in `OD-LEDGER-012` erases a claim, and the test was written
/// to stop it being erased *by accident*.
///
/// This test's own previous doc comment said it "stops short of the claimability, because an
/// assertion either way would pin the behaviour before that decision is made". The decision is
/// made, in `OD-LEDGER-012`. `Test_A_Lapsed_Item_Should_Be_Taken_Over_And_Still_Name_Its_\
/// Previous_Holder` covers the case this one could not, because the operation did not exist.
#[test]
fn Test_A_Lapsed_Item_Should_Refuse_A_Plain_Claim_And_Name_The_Takeover()
{
    let Board { directory, mut ledger } = Board_At("lapse-takeover", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Claim_For_Holder(&mut ledger, "T-1", Claimant("dead-agent"));

    let mut after = After_The_Lapse(directory.As_Path());

    let refusal = Refusal_From_Claim(&mut after, "T-1", Claimant("agent-b"));
    Says_The_Item_Is_Lapsed_And_Names_The_Verb(&refusal);

    assert_eq!(
        Standing_Of(&Only_Item(&after)),
        Standing {
            held_by: Some("dead-agent"),
            displaced: Vec::new(),
            widened: Vec::new(),
        },
        "the lapsed claim was replaced, or a refused claim recorded a displacement — either \
         way `claim` has quietly become `takeover`"
    );
}

/// The three things the refusal has to say, and each is a different way of getting it wrong:
/// blaming territory, giving no remedy, or advising a wait that will never end.
fn Says_The_Item_Is_Lapsed_And_Names_The_Verb(refusal: &ClaimRefusal)
{
    assert!(
        matches!(refusal, ClaimRefusal::Lapsed { .. }),
        "the refusal must say the item is not in a claimable state rather than blame \
         territory or the identifier: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("takeover"),
        "a refusal whose remedy is a verb has to name the verb: {}",
        refusal.Describe()
    );
    assert!(
        !refusal.Is_Retryable(),
        "waiting does not revive a dead holder; somebody has to decide to take the work"
    );
}

/// The holder's own recovery still works, and is now the whole recovery story.
///
/// `Renew` and `Release` match on the holder and never took the validating path, so an
/// agent that came back could always rescue its own claim. That was the only recovery there
/// was while the board was bricked; it is still the only way a lapsed item returns to the
/// pool, and the lapse now blocks only that item rather than every item.
#[test]
fn Test_The_Holder_Should_Still_Recover_Its_Own_Lapsed_Claim()
{
    let Board { directory, mut ledger } = Board_At("lapse-recover", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Claim_For_Holder(&mut ledger, "T-1", Claimant("agent-a"));

    let mut after = After_The_Lapse(directory.As_Path());

    after
        .Renew(&ItemId::New("T-1"), "agent-a", LEASE)
        .unwrap_or_else(|refusal| {
            panic!("the holder must be able to renew: {}", refusal.Describe())
        });

    let held = after.Load().expect("the ledger the renewal wrote reads back");
    assert!(
        held.items
            .first()
            .is_some_and(|item| return item.Has_Active_Claim(Timestamp_From_Seconds(AT_LATER_SECONDS))),
        "renewing a lapsed claim must make it active again"
    );
}

// ---------------------------------------------------------------------------
// A lapse is taken over, and the claim it replaces is kept. P10-LAPSE-TAKEOVER,
// decided in `OD-LEDGER-012`.
//
// The two tests above this comment are the boundary the decision was made against: a lapse
// must not brick the board, and a plain `claim` must not quietly erase a dead agent's claim.
// Everything below is the operation that does take the item, and the assertion running through
// all of it is that nothing it does is silent.
// ---------------------------------------------------------------------------

/// The subject, and exactly the sequence `done_when` names.
///
/// Claim it, force the lease into the past, take it over as somebody else, and assert both
/// halves: the takeover succeeded, and the previous holder is still named on the item. The
/// second half is the whole point — *"a new claim overwriting the old one silently is the
/// outcome this must not have."*
#[test]
fn Test_A_Lapsed_Item_Should_Be_Taken_Over_And_Still_Name_Its_Previous_Holder()
{
    let Board { directory, mut ledger } =
        Board_At("takeover-keeps-predecessor", vec![Item_Reserving_Files("T-1", &["src/a.rs"])]);
    Claim_For_Holder(&mut ledger, "T-1", Claimant("dead-agent"));

    let mut after = After_The_Lapse(directory.As_Path());

    let reservation = Take_Over_In(&mut after, "T-1", Claimant("agent-b")).unwrap_or_else(|refusal| {
        panic!(
            "an item whose holder died must return to the pool without a person editing the \
             file: {}",
            refusal.Describe()
        )
    });
    assert_eq!(reservation.holder, "agent-b");

    let item = Only_Item(&after);
    Installed_The_Taker(&item, Timestamp_From_Seconds(AT_LATER_SECONDS));
    Kept_The_Claim_It_Replaced(&item);
}

/// The takeover installed its new holder, on a live lease, without moving the state.
///
/// Three assertions rather than one because each names a different way the verb could be
/// half-done, and a takeover that leaves the lease in the past has taken nothing.
fn Installed_The_Taker(item: &LedgerItem, now: Timestamp)
{
    assert_eq!(
        Holder_Of_Item(item),
        Some("agent-b"),
        "the takeover did not install the new holder"
    );
    assert!(
        item.Has_Active_Claim(now),
        "a takeover that leaves the lease in the past has taken nothing"
    );
    assert_eq!(
        item.state,
        ItemState::Claimed,
        "a takeover does not move the state; the item was claimed and still is"
    );
}

/// The half `P10-LAPSE-TAKEOVER` exists for: who held it, when they took it, and when the
/// lease ran out all survive, and they survive as the claim itself rather than as a summary.
fn Kept_The_Claim_It_Replaced(item: &LedgerItem)
{
    let displaced = item.displaced.first().unwrap_or_else(|| {
        panic!(
            "the takeover kept {} displaced claim(s) rather than exactly the one it replaced, \
             so nothing records who walked away from this",
            item.displaced.len()
        )
    });

    assert_eq!(item.displaced.len(), 1, "something records it twice");
    assert_eq!(
        displaced.holder, "dead-agent",
        "the takeover dropped the previous holder, which is the outcome this must not have"
    );
    assert_eq!(displaced.acquired_at, Timestamp_From_Seconds(NOW), "when they took it");
    assert_eq!(
        displaced.lease_expires_at,
        Timestamp_From_Seconds(LEASE_ENDS_AT),
        "when the lease ran out"
    );
}
