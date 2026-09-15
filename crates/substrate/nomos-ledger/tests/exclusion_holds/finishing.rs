//! Ending a claim: finished, abandoned, or lapsed.
//!
//! Three different endings, and the ledger keeps them apart. A finish records a verification
//! that actually ran, an abandonment records who stopped and why, and a lapse invents no
//! reason at all — nobody was there to give one.

use crate::board::{
    At, AT_LATER_SECONDS, Abandon, Abandonments, After_The_Lapse, Board, Board_At, ClaimRefusal, Claimant, Duration,
    ExclusionLedger, Exits_With, Finish_In, FinishRefusal, Item, ItemId, ItemState, Item_Verified_By, LEASE, NOW,
    Only_Item, REASON, Release_As_Finished, Take,
};

/// The Phase 0 acceptance criterion: a completion whose predicate exits non-zero is
/// refused, and the item does not become done.
#[test]
fn Test_Finishing_Should_Be_Refused_When_The_Predicate_Fails()
{
    let Board { directory, mut ledger } = Board_At("finish-fails", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        Exits_With(1),
    )]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = Finish_In(&mut ledger, directory.As_Path(), "T-1", Claimant("agent-a"))
    .expect_err("a predicate that exits non-zero must refuse the completion");

    assert!(matches!(refusal, FinishRefusal::PredicateFailed { .. }));
    assert!(refusal.Has_Judged_The_Work());

    let after = ledger.Load().expect("the ledger a refused finish left behind reads back");
    assert_eq!(
        after.items.first().map(|item| &item.state),
        Some(&ItemState::Claimed),
        "a refused completion must leave the item claimed, not done"
    );
}

/// The negative control. Without it, a `Finish` that refused everything unconditionally
/// would pass the test above.
#[test]
fn Test_Finishing_Should_Succeed_When_The_Predicate_Passes()
{
    let Board { directory, mut ledger } = Board_At("finish-passes", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        Exits_With(0),
    )]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let record = Finish_In(&mut ledger, directory.As_Path(), "T-1", Claimant("agent-a"))
    .expect("a passing predicate must finish the item");

    assert_eq!(record.exit_code, 0);
    assert_eq!(record.verified_at, At(NOW));

    let finished = Only_Item(&ledger);
    assert_eq!(finished.state, ItemState::Done);
    assert!(
        finished.verified.is_some(),
        "a done item carries the evidence that made it done"
    );
    ledger
        .Validate_Current()
        .expect("a verified done item is a valid ledger");
}

/// An item with nothing to run cannot be shown to be finished. "There was nothing to
/// check" must not read the same as "everything checked out".
#[test]
fn Test_Finishing_Should_Be_Refused_Without_A_Predicate()
{
    let Board { directory, mut ledger } = Board_At("finish-no-predicate", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = Finish_In(&mut ledger, directory.As_Path(), "T-1", Claimant("agent-a"))
    .expect_err("an item with no predicate cannot be finished");

    assert!(matches!(refusal, FinishRefusal::NoPredicate { .. }));
    assert!(
        !refusal.Has_Judged_The_Work(),
        "nothing was learned about the work"
    );
}

/// A predicate that cannot be started says nothing about the work. Reporting it as a
/// failed check would tell an author their code is wrong when their tooling is missing.
#[test]
fn Test_An_Unstartable_Predicate_Should_Not_Judge_The_Work()
{
    let Board { directory, mut ledger } = Board_At("finish-unstartable", vec![Item_Verified_By(
        "T-1",
        &["src/a.rs"],
        vec!["nomos-no-such-program-exists".to_owned()],
    )]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = Finish_In(&mut ledger, directory.As_Path(), "T-1", Claimant("agent-a"))
    .expect_err("a missing program is not a verdict");

    assert!(matches!(refusal, FinishRefusal::CouldNotRun { .. }));
    assert!(!refusal.Has_Judged_The_Work());
}

/// The state transition to `Done` carries its own evidence, so an item cannot arrive
/// there by any route that skipped verification.
#[test]
fn Test_Releasing_As_Finished_Should_Record_The_Verification()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("finish-records", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    Release_As_Finished(&mut ledger, "T-1", Claimant("agent-a"));

    let finished = Only_Item(&ledger);
    assert_eq!(finished.state, ItemState::Done);
    assert_eq!(
        finished.verified.as_ref().map(|record| record.exit_code),
        Some(0)
    );
}


/// The arm adjacent to the one above, which used to throw its evidence away.
///
/// [`ReleaseOutcome::Abandoned`] has always carried `reason: String` non-optionally — the
/// same technique the doc comment praises the finished arm for — and the store matched it
/// with `{ .. }` and set the state and nothing else. One match, two arms, one keeping its
/// evidence and one discarding it.
#[test]
fn Test_Releasing_As_Abandoned_Should_Record_Who_Stopped_And_Why()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("abandon-records", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    Abandon(&mut ledger, "T-1", Claimant("agent-a"), REASON);

    let item = Only_Item(&ledger);
    let abandonment = item
        .abandoned
        .first()
        .expect("the abandonment must survive the release that produced it");

    assert_eq!(abandonment.reason, REASON, "the reason the holder gave was not kept");
    assert_eq!(abandonment.holder, "agent-a", "the record does not say who stopped");
    assert_eq!(abandonment.abandoned_at, At(NOW), "the record does not say when");
}

/// The second control, holding a line the crate already drew.
///
/// An abandonment is a record of something that stopped. A record that went on excluding
/// people would be a worse defect than the one it replaced, so the item must go back to
/// `Ready`, the claim must go, and — the part worth checking rather than inferring —
/// somebody else must actually be able to take it.
#[test]
fn Test_An_Abandoned_Item_Should_Return_To_Ready_And_Stop_Excluding()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("abandon-releases", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));
    Abandon(&mut ledger, "T-1", Claimant("agent-a"), REASON);

    let item = Only_Item(&ledger);
    assert_eq!(item.state, ItemState::Ready);
    assert!(item.claim.is_none(), "a claim that survives an abandonment goes on excluding");

    Take(&mut ledger, "T-1", Claimant("agent-b"));

    let again = Only_Item(&ledger);
    assert_eq!(
        again.abandoned.len(),
        1,
        "the next claim erased the record of the last one"
    );
}

/// Why the field is a list and not the most recent one.
///
/// An item abandoned twice was abandoned twice. Keeping only the latest would discard the
/// earlier reason, which is the loss this whole item is about, one scale down.
/// The two claims on the same item, oldest first, and why each one ended.
///
/// A named provider rather than an inline literal, so a third abandonment is a value added
/// here rather than a change to the loop that reads them.
const ABANDONMENTS_ON_THE_ITEM: usize = 2;

fn Two_Abandonments() -> [(&'static str, &'static str); ABANDONMENTS_ON_THE_ITEM]
{
    return [("agent-a", "ran out of lease"), ("agent-b", REASON)];
}

#[test]
fn Test_An_Item_Abandoned_Twice_Should_Keep_Both_Reasons()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("abandon-twice", vec![Item("T-1", &["src/a.rs"])]);

    for (holder, reason) in Two_Abandonments()
    {
        ledger
            .Claim(&ItemId::New("T-1"), holder, LEASE)
            .expect("an abandoned item is claimable again");
        Abandon(&mut ledger, "T-1", Claimant(holder), reason);
    }

    let item = Only_Item(&ledger);
    assert_eq!(
        Abandonments(&item),
        vec![("agent-a", "ran out of lease"), ("agent-b", REASON)],
        "both abandonments must survive, oldest first"
    );
}

/// The question `OD-LEDGER-006` settles, asserted here rather than left in the prose.
///
/// A lapsed claim is given no synthesized abandonment. Nobody was there to write a reason,
/// and inventing one — "the lease expired" — would be filler wearing a record's clothes.
/// What a lapse leaves is the claim itself: it stops counting as active without being
/// removed, so a reader can still see who held it and when they stopped. The two paths now
/// differ by what is actually knowable rather than by which one ran.
///
/// What this test deliberately does not assert is that the next agent can take the item.
/// It cannot: a lapse leaves `state` at `Claimed`, and `Claim_Refusal` rejects anything
/// that is not `Ready` before it ever reaches the lease. That contradicts `item.rs`, which
/// says a lapse "stops excluding, which is what lets the next agent take the item", and it
/// is a different defect from this one — recorded in `OD-LEDGER-006` and on the ledger,
/// not asserted here, because an assertion would pin the behaviour in place.
#[test]
fn Test_A_Lapsed_Claim_Should_Stay_Visible_And_Invent_No_Reason()
{
    let Board { directory, mut ledger } = Board_At("abandon-lapse", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let lapsed = After_The_Lapse(directory.As_Path());
    let item = Only_Item(&lapsed);

    assert!(
        item.abandoned.is_empty(),
        "a lapse wrote a reason nobody gave: {:?}",
        item.abandoned
    );
    assert!(
        item.claim.is_some(),
        "the lapsed claim was removed, so nothing says the work was ever started"
    );
    assert!(
        !item.Has_Active_Claim(At(AT_LATER_SECONDS)),
        "a lapsed claim must stop counting as an active claim"
    );
}

#[test]
fn Test_A_Lease_Beyond_The_Ceiling_Should_Be_Refused()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("claim-lease", vec![Item("T-1", &["src/a.rs"])]);

    let refusal = ledger
        .Claim(
            &ItemId::New("T-1"),
            "agent-a",
            nomos_ledger::MAXIMUM_LEASE + Duration::from_secs(1),
        )
        .expect_err("an unbounded lease defeats the ledger");

    assert!(matches!(refusal, ClaimRefusal::LeaseTooLong { .. }));
}

#[test]
fn Test_Renewing_Someone_Elses_Claim_Should_Be_Refused()
{
    let Board { directory: _directory, mut ledger } =
        Board_At("renew-foreign", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", Claimant("agent-a"));

    let refusal = ledger
        .Renew(&ItemId::New("T-1"), "agent-b", LEASE)
        .expect_err("renewing another holder's claim must be refused");

    assert!(matches!(refusal, ClaimRefusal::HeldBy { .. }));
}

// ---------------------------------------------------------------------------
// Durability — a corrupt ledger is never mistaken for an empty one.
// ---------------------------------------------------------------------------
