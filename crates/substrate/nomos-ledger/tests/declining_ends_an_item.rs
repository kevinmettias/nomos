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

use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, ItemId, ItemState, LedgerDocument, LedgerItem,
    ReleaseOutcome, SCHEMA_VERSION, Territory as ItemTerritory, VerificationRecord,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Held still, so that "when it was declined" is arithmetic rather than a sleep.
struct FixedClock(i64);

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

const NOW: i64 = 1_000_000;

const REASON: &str =
    "superseded by T-2, which landed the whole of this item's done_when and was verified first";

fn At(seconds: i64) -> Timestamp
{
    return Timestamp::From_Unix_Seconds(seconds);
}

fn Item(id: &str, files: &[&str]) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        territory: ItemTerritory::Of_Files(files.iter().copied()),
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

/// A `Done` item, built rather than finished.
///
/// Running a real predicate would make this file about `finish`, which `gate_covers_finish.rs`
/// already is. What matters here is only that the state is `Done` and validation accepts it,
/// which needs the verification record store validation requires beside that state.
fn Finished_Item(id: &str, files: &[&str]) -> LedgerItem
{
    let mut item = Item(id, files);
    item.state = ItemState::Done;
    item.verified = Some(VerificationRecord {
        argv: vec!["cargo".to_owned(), "--version".to_owned()],
        exit_code: 0,
        output_tail: String::new(),
        verified_at: At(NOW),
        gate: None,
    });
    return item;
}

fn Document(items: Vec<LedgerItem>) -> LedgerDocument
{
    return LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items,
    };
}

fn Temp_Dir(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-decline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

fn Ledger_At<'clock>(
    directory: &Path,
    clock: &'clock FixedClock,
) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
{
    return FileLedger::At(
        directory.join("ledger.json"),
        StdFileSystem,
        clock,
        FileLock::At(directory.join("ledger.lock")),
    );
}

/// The whole point, on the shape that motivated it: an unclaimed `Ready` item.
///
/// Both items `OD-LEDGER-019` measures were released before anybody knew they were
/// superseded, so a verb that could only reach a held item could not reach either of them.
#[test]
fn Test_Declining_An_Unclaimed_Item_Should_End_It_And_Say_Who_Ended_It()
{
    let directory = Temp_Dir("ends-it");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");

    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item is the case this verb exists for");

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");

    assert_eq!(
        item.state,
        ItemState::Declined {
            reason: REASON.to_owned(),
        },
        "the reason must be in the state, which is what makes it unreachable reasonlessly"
    );

    let declination = item
        .declined
        .as_ref()
        .expect("who ended it and when must survive the transition that ended it");
    assert_eq!(declination.holder, "agent-a", "the item does not say who declined it");
    assert_eq!(declination.declined_at, At(NOW), "the item does not say when");

    let _ = std::fs::remove_dir_all(&directory);
}

/// The negative control for the test above, and the reason `Declination` carries no reason
/// of its own.
///
/// One fact, one home. A reader asking *why* asks the state and a reader asking *who* asks
/// the item, and neither can be told two different things because there is only one copy.
#[test]
fn Test_A_Declination_Should_Not_Carry_A_Second_Copy_Of_The_Reason()
{
    let directory = Temp_Dir("one-copy");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    let raw = std::fs::read_to_string(directory.join("ledger.json")).expect("readable");

    assert_eq!(
        raw.matches(REASON).count(),
        1,
        "the reason is stored twice, so the two copies can come to disagree"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// A declined item is not work, so nothing may take it.
#[test]
fn Test_A_Declined_Item_Should_Not_Be_Claimable()
{
    let directory = Temp_Dir("not-claimable");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");

    // The control: it is claimable right up until it is declined, so the refusal below is
    // the decline's doing and not the fixture's.
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("a Ready item is claimable");
    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Abandoned {
                reason: "stopped to check whether the successor already landed it".to_owned(),
            },
        )
        .expect("a holder may give up its own claim");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    let refusal = ledger
        .Claim(&ItemId::New("T-1"), "agent-b", Duration::from_secs(3_600))
        .expect_err("a declined item is not work and must not be claimable");

    assert!(
        matches!(refusal, ClaimRefusal::NotClaimable { .. }),
        "{}",
        refusal.Describe()
    );
    assert!(
        !refusal.Is_Retryable(),
        "a declined item never becomes claimable, so telling an agent to retry is a loop"
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Declining must not resurrect the claim the abandonment removed.
///
/// An item that went on excluding people after it was ended would be a worse defect than the
/// one this verb closes, which is the line `ReleaseOutcome::Record_On` already draws for
/// abandonment.
#[test]
fn Test_A_Declined_Item_Should_Stop_Excluding()
{
    let directory = Temp_Dir("stops-excluding");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![
            Item("T-1", &["src/shared.rs"]),
            Item("T-2", &["src/shared.rs"]),
        ]))
        .expect("a fresh ledger is valid");

    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    // The control: while T-1 is held, the overlapping item is refused.
    ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect_err("overlapping territory is refused while T-1 is held");

    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Abandoned {
                reason: "the successor reserves this ground correctly".to_owned(),
            },
        )
        .expect("a holder may give up its own claim");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    ledger
        .Claim(&ItemId::New("T-2"), "agent-b", Duration::from_secs(3_600))
        .expect("a declined item reserves nothing");

    let after = ledger.Load().expect("readable");
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

    let _ = std::fs::remove_dir_all(&directory);
}

/// Ending somebody's live work is their call, and the refusal says so.
///
/// `OD-LEDGER-001`: territory is declared and not enforced, so the ledger's answer *is* the
/// exclusion. A verb that ended a held item from outside would overrule the only party that
/// knows whether the work is still running.
#[test]
fn Test_Declining_A_Held_Item_Should_Be_Refused_Retryably()
{
    let directory = Temp_Dir("held");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Claim(&ItemId::New("T-1"), "agent-a", Duration::from_secs(3_600))
        .expect("uncontended");

    let refusal = ledger
        .Decline(&ItemId::New("T-1"), "agent-b", REASON)
        .expect_err("a held item may not be ended by somebody who is not holding it");

    assert!(
        refusal.Describe().contains("agent-a"),
        "the refusal must name the holder, because the remedy is a command only they can run: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Is_Retryable(),
        "the claim will be released or will lapse, so this is a queue and not a dead end"
    );

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");
    assert_eq!(item.state, ItemState::Claimed, "a refused decline changed the state");
    assert!(item.declined.is_none(), "a refused decline was recorded anyway");

    // The remedy the refusal names, run in full: the holder releases, and then it declines.
    ledger
        .Release(
            &ItemId::New("T-1"),
            "agent-a",
            ReleaseOutcome::Abandoned {
                reason: "there is nothing here to do".to_owned(),
            },
        )
        .expect("a holder may give up its own claim");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-b", REASON)
        .expect("once released, anybody may end it");

    let _ = std::fs::remove_dir_all(&directory);
}

/// Declining finished work would put prose where a verdict is.
///
/// `finish` exists so that a `Done` item cannot claim a check that never ran, and a decline
/// that overwrote it would hand back exactly that.
#[test]
fn Test_Declining_A_Done_Item_Should_Be_A_Conflict()
{
    let directory = Temp_Dir("done");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Finished_Item("T-1", &["src/a.rs"])]))
        .expect("a finished item is a valid ledger");

    let refusal = ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect_err("finished work is not declinable");

    assert!(
        !refusal.Is_Retryable(),
        "a Done item never stops being Done, so retrying is a loop and a person is needed"
    );

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");
    assert_eq!(item.state, ItemState::Done, "the verdict was overwritten with prose");
    assert!(item.verified.is_some(), "the verification record was discarded");

    let _ = std::fs::remove_dir_all(&directory);
}

/// A second decline is a duplicate or a disagreement, and both are for a person.
#[test]
fn Test_Declining_A_Declined_Item_Should_Keep_The_First_Reason()
{
    let directory = Temp_Dir("twice");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    let refusal = ledger
        .Decline(&ItemId::New("T-1"), "agent-b", "a different reading entirely")
        .expect_err("a declined item is already ended");

    assert!(
        refusal.Describe().contains(REASON),
        "the refusal must report the reason already recorded, or the caller cannot tell a \
         duplicate from a disagreement: {}",
        refusal.Describe()
    );
    assert!(!refusal.Is_Retryable());

    // …and it must stay one line while doing it. The reason `P10-DERIVED-FACT` was declined
    // with runs to five paragraphs, and this refusal printed every one of them with the
    // newlines escaped before `ItemState::Describe` existed.
    let long = ledger
        .Decline(&ItemId::New("T-1"), "agent-c", "first line\n\nand four more paragraphs")
        .expect_err("still declined");
    assert_eq!(long.Describe().lines().count(), 1, "{}", long.Describe());
    assert!(long.Describe().contains(REASON.lines().next().unwrap()));

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");
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

    let _ = std::fs::remove_dir_all(&directory);
}

/// A declined board is a valid board.
///
/// The failure this guards is the one `ReleaseOutcome::Finished` was reshaped for: a
/// transition that produces a state the document's own validation then refuses, so the only
/// path to it can never succeed.
#[test]
fn Test_A_Declined_Item_Should_Survive_Validation()
{
    let directory = Temp_Dir("valid");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("an unclaimed item may be declined");

    ledger
        .Validate_Current()
        .expect("a declined item must be a valid ledger, or the verb writes what nothing can read");

    let _ = std::fs::remove_dir_all(&directory);
}

/// A misspelled identifier is told it is a misspelled identifier.
#[test]
fn Test_Declining_An_Unknown_Item_Should_Name_The_Identifier()
{
    let directory = Temp_Dir("unknown");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");

    let refusal = ledger
        .Decline(&ItemId::New("T-2"), "agent-a", REASON)
        .expect_err("there is no such item");

    assert!(matches!(refusal, ClaimRefusal::NoSuchItem { .. }), "{}", refusal.Describe());
    assert!(refusal.Describe().contains("T-2"));

    let _ = std::fs::remove_dir_all(&directory);
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
    let directory = Temp_Dir("reachable");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);

    ledger
        .Save(&Document(vec![Item("T-1", &["src/a.rs"])]))
        .expect("a fresh ledger is valid");
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("the verb is the only thing that may produce this state");

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");

    assert!(item.state.Is_Finished(), "a declined item is not terminal");
    assert!(!item.state.Is_Claimable(), "a declined item is claimable");

    let _ = std::fs::remove_dir_all(&directory);
}
