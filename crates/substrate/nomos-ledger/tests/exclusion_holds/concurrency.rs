//! Two writers, and what must survive both.
//!
//! Written against the interleaving harness rather than against timing, so a failure here is
//! the defect and not the machine.

use crate::board::{
    Abandon, AddRefusal, At, Claimant, ExclusionLedger, Held_By, Item, ItemId, ItemTerritory, LEASE, LEASE_ENDS_AT,
    Mutex, NOW, REASON, Take, Validate_Document,
};
use crate::interleaving::{Holder_Of, InterleavedLedger, Two_Writers};

/// A lease with seconds left on it, so the renewal below is visibly the thing that moved the
/// expiry rather than a value the fixture already carried.
const LEASE_ABOUT_TO_RUN_OUT: i64 = NOW + 10;

/// The defect `P10-LOCK-BYPASS` is open for, at the verb that starts every piece of work.
///
/// `Claim` read the whole document, decided against it and wrote the whole document back,
/// with no lock held anywhere in between — while [`FileLedger::With_Lock`] sat beside it
/// saying every mutation went through it. Two sessions overlapping therefore lost one of the
/// two writes, and what was lost was not a field: it was every change the other session had
/// made to any item, because the document written back was a snapshot taken before that
/// session existed.
///
/// The two items here reserve disjoint territory, so exclusion has nothing to say about
/// them. Both claims are legitimate and both must survive. That is the point: this is not a
/// test about refusing a claim, it is a test about not losing one that was granted.
#[test]
fn Test_Two_Concurrent_Claims_Should_Both_Survive()
{
    let after = Two_Writers(
        "concurrent-claims",
        vec![
        Item("T-1", &["src/a.rs"]),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            Take(ledger, "T-1", Claimant("agent-a"));
        },
        |ledger| {
            Take(ledger, "T-2", Claimant("agent-b"));
        },
    );

    assert_eq!(
        Holder_Of(&after, "T-1"),
        Some("agent-a".to_owned()),
        "the first writer's claim is not in the ledger it wrote"
    );
    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "the second writer was told its claim was granted and the ledger does not have it: \
         one writer wrote back a document it had read before the other one existed"
    );
}

/// The same loss at the verb that ends a piece of work.
///
/// `Release` is the write `Finish` performs after its predicate passes, so a lost one is an
/// agent that ran its verification, was told the item was recorded as done, and left behind
/// a board that still calls the item claimed — or, as here, a board that has forgotten
/// somebody else's claim entirely.
#[test]
fn Test_A_Release_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let after = Two_Writers(
        "concurrent-release",
        vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", LEASE_ENDS_AT),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            Abandon(ledger, "T-1", Claimant("agent-a"), REASON);
        },
        |ledger| {
            Take(ledger, "T-2", Claimant("agent-b"));
        },
    );

    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "a release wrote back a document read before the other writer's claim, so the claim \
         it was granted is gone"
    );
    assert_eq!(
        after
            .items
            .iter()
            .find(|item| return item.id == ItemId::New("T-1"))
            .map(|item| return item.abandoned.len()),
        Some(1),
        "the abandonment the first writer was told had been recorded is not in the ledger"
    );
}

/// And at the verb an agent runs most often, which is the one that hides best.
///
/// A renewal that is lost does not look like a lost write. It looks like a lease that ran out
/// early, which reads as an agent that died — so the wrong thing gets investigated.
#[test]
fn Test_A_Renewal_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let after = Two_Writers(
        "concurrent-renew",
        vec![
        Held_By(Item("T-1", &["src/a.rs"]), "agent-a", LEASE_ABOUT_TO_RUN_OUT),
        Item("T-2", &["src/b.rs"]),
    ],
        |ledger| {
            ledger
                .Renew(&ItemId::New("T-1"), "agent-a", LEASE)
                .expect("a holder may renew its own claim");
        },
        |ledger| {
            Take(ledger, "T-2", Claimant("agent-b"));
        },
    );

    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "a renewal wrote back a document read before the other writer's claim, so the claim \
         it was granted is gone"
    );
    assert_eq!(
        after
            .items
            .iter()
            .find(|item| return item.id == ItemId::New("T-1"))
            .and_then(|item| return item.claim.as_ref())
            .map(|claim| return claim.lease_expires_at),
        Some(At(LEASE_ENDS_AT)),
        "the renewal the first writer was told had been recorded is not in the ledger"
    );
}

/// The same loss at the verb that puts work on the board, which `OD-LEDGER-015` left behind.
///
/// `Claim`, `Renew` and `Release` were moved behind the lock and `add` was not, because the
/// three were named as "the verbs that change the board" and adding an item was not counted
/// as changing it. It is: the document `add` writes back is the whole board, so an add that
/// read before somebody else's claim erases that claim exactly as a stale `Claim` would.
///
/// Observed on the real ledger rather than reasoned about. Two adds ran back to back, both
/// printed their success line and both exited 0, and only the first was ever on the board —
/// a peer's locked verb had read the file between them and written its snapshot back over
/// the second. `OD-LEDGER-021`.
#[test]
fn Test_An_Add_Should_Not_Erase_A_Claim_Taken_While_It_Ran()
{
    let after = Two_Writers(
        "concurrent-add",
        vec![Item("T-2", &["src/b.rs"])],
        |ledger| {
            let item = Item("T-1", &["src/a.rs"]);
            ledger
                .Add(&item, "agent-a", &ItemTerritory::Empty(), &ItemTerritory::Empty())
                .expect("T-1 is not on the board yet");
        },
        |ledger| {
            Take(ledger, "T-2", Claimant("agent-b"));
        },
    );

    assert!(
        after
            .items
            .iter()
            .any(|item| return item.id == ItemId::New("T-1")),
        "the add was told the item was recorded and the item is not on the board: an add \
         wrote back a document it had read before the other writer existed, or was written \
         over by one"
    );
    assert_eq!(
        Holder_Of(&after, "T-2"),
        Some("agent-b".to_owned()),
        "an add wrote back a document read before the other writer's claim, so the claim it \
         was granted is gone"
    );
}

/// The duplicate check has to travel inside the lock with the write it guards.
///
/// Deciding it outside is the same defect one level down, and it is worse than a lost item:
/// two sessions adding one identifier both read a board without it, both are told it is
/// theirs, and the document that results has the identifier twice. [`Validate`] calls that
/// invalid and every operation loads before it does anything, so the next agent to touch the
/// board — any agent, on any item — is refused by a ledger that will not load. Two callers
/// were each told they succeeded and the board is unusable.
///
/// The interleaving is the one the harness always builds, and what it proves is different
/// here: the second writer is not merely made to wait, it is made to *see* the first writer's
/// item and refuse on it.
#[test]
fn Test_Two_Concurrent_Adds_Of_One_Identifier_Should_Not_Both_Be_Accepted()
{
    let second_outcome = Mutex::new(None);
    let recorded = &second_outcome;
    let after = Two_Writers(
        "concurrent-duplicate-add",
        Vec::new(),
        |ledger| {
            Adds_T_1(ledger, "src/a.rs", Claimant("agent-a")).expect("the board is empty, so T-1 is free");
        },
        |ledger| {
            let outcome = Adds_T_1(ledger, "src/b.rs", Claimant("agent-b"));
            *recorded.lock().expect("the harness never panics under this lock") = Some(outcome);
        },
    );

    assert_eq!(
        second_outcome
            .into_inner()
            .expect("the harness never panics under this lock")
            .expect("the second writer ran"),
        Err(AddRefusal::AlreadyPresent {
            item: ItemId::New("T-1")
        }),
        "the second add read the board before the first one's write and was told an \
         identifier that was already taken was free"
    );
    assert_eq!(
        after.items.iter().filter(|item| return item.id == ItemId::New("T-1")).count(),
        1,
        "one identifier is on the board twice, so the board no longer loads for anybody"
    );
    assert!(
        Validate_Document(&after, At(NOW)).is_empty(),
        "the board two accepted adds left behind is one the ledger itself calls invalid"
    );
}

/// Both writers add `T-1`; only the territory and the holder differ, which is what makes the
/// second one's refusal a statement about the identifier rather than about the files.
fn Adds_T_1(
    ledger: &mut InterleavedLedger<'_>,
    file: &str,
    holder: Claimant<'_>,
) -> Result<(), AddRefusal>
{
    let item = Item("T-1", &[file]);

    return ledger.Add(&item, holder.0, &ItemTerritory::Empty(), &ItemTerritory::Empty());
}
