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

use nomos_platform::{DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, ItemId, ItemKind, ItemOrigin, ItemState,
    LedgerDocument, LedgerItem, ReleaseOutcome, SCHEMA_VERSION, VerificationRecord,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Held still, so that "when it was declined" is arithmetic rather than a sleep.
struct FixedClock(i64);

/// Fixed instants, so both the values and their timing reproduce.
impl Strategy for FixedClock
{
    const STRENGTH: DeterminismStrength = DeterminismStrength::StateTemporal;
    const SCOPE: ReproducibilityScope = ReproducibilityScope::SingleRun;
    const TRACE: TraceEquivalence = TraceEquivalence::BitIdentical;
}

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
    use nomos_ledger::Territory as ItemTerritory;

    return LedgerItem {
        id: ItemId::New(id),
        title: format!("work item {id}"),
        why: "it needs doing".to_owned(),
        done_when: "the tests pass".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: ItemTerritory::Of_Files(files.iter().copied()),
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
        revision: None,
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

fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-decline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// The clock every test that does not move time shares.
///
/// A `'static` clock is what lets [`Board_At`] hand back a ledger: the ledger borrows its
/// clock, so a local one could not outlive the call that built it.
static AT_NOW: FixedClock = FixedClock(NOW);

/// The one-hour lease every test here takes, said once.
const LEASE: Duration = Duration::from_secs(3_600);

/// A ledger on a fresh temporary directory, already holding the board it starts from.
fn Board_At(
    name: &str,
    items: Vec<LedgerItem>,
) -> (PathBuf, FileLedger<StdFileSystem, &'static FixedClock, FileLock>)
{
    let directory = Temporary_Directory(name);
    let ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&Document(items)).expect("a fresh ledger is valid");

    return (directory, ledger);
}

/// One agent claims one item for the standard lease, and it is expected to succeed.
fn Take<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str)
{
    ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .unwrap_or_else(|refusal| {
            panic!("the fixture claim was refused: {}", refusal.Describe())
        });
}

/// A claim that is expected to be refused, with the refusal handed back as the value the
/// test is about.
fn Refused<Ledger: ExclusionLedger>(ledger: &mut Ledger, item: &str, holder: &str) -> ClaimRefusal
{
    return ledger
        .Claim(&ItemId::New(item), holder, LEASE)
        .expect_err("this claim is contended and must be refused");
}

/// The ledger every test here builds, named once so the verbs below can take it.
type Board = FileLedger<StdFileSystem, &'static FixedClock, FileLock>;

/// An item ended with the standard reason, which is expected to succeed.
fn Decline(ledger: &mut Board, item: &str, holder: &str)
{
    ledger
        .Decline(&ItemId::New(item), holder, REASON)
        .expect("an unclaimed item is the case this verb exists for");
}

/// A decline expected to be refused, with the refusal handed back as the value the test is
/// about.
fn Decline_Refused(ledger: &mut Board, item: &str, holder: &str, reason: &str) -> ClaimRefusal
{
    return ledger
        .Decline(&ItemId::New(item), holder, reason)
        .expect_err("this decline is contended and must be refused");
}

/// A holder giving up its own claim, which is the remedy every refusal here names.
fn Give_Up(ledger: &mut Board, item: &str, holder: &str, reason: &str)
{
    ledger
        .Release(&ItemId::New(item), holder, ReleaseOutcome::Abandoned {
            reason: reason.to_owned(),
        })
        .expect("a holder may give up its own claim");
}

/// The board's only item, read back off disk.
fn Only_Item(ledger: &Board) -> LedgerItem
{
    let after = ledger.Load().expect("readable");

    return after.items.into_iter().next().expect("the item survives");
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
    let (directory, mut ledger) = Board_At("ends-it", vec![Item("T-1", &["src/a.rs"])]);

    Decline(&mut ledger, "T-1", "agent-a");

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
    let (directory, mut ledger) = Board_At("one-copy", vec![Item("T-1", &["src/a.rs"])]);
    Decline(&mut ledger, "T-1", "agent-a");

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
    let (directory, mut ledger) = Board_At("not-claimable", vec![Item("T-1", &["src/a.rs"])]);

    // The control: it is claimable right up until it is declined, so the refusal below is
    // the decline's doing and not the fixture's.
    Take(&mut ledger, "T-1", "agent-a");
    Give_Up(&mut ledger, "T-1", "agent-a", "stopped to check whether the successor landed it");
    Decline(&mut ledger, "T-1", "agent-a");

    let refusal = Refused(&mut ledger, "T-1", "agent-b");

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
    let (directory, mut ledger) = Board_At("stops-excluding", vec![
        Item("T-1", &["src/shared.rs"]),
        Item("T-2", &["src/shared.rs"]),
    ]);

    Take(&mut ledger, "T-1", "agent-a");

    // The control: while T-1 is held, the overlapping item is refused.
    let _refused = Refused(&mut ledger, "T-2", "agent-b");

    Give_Up(&mut ledger, "T-1", "agent-a", "the successor reserves this ground correctly");
    Decline(&mut ledger, "T-1", "agent-a");
    Take(&mut ledger, "T-2", "agent-b");

    Assert_The_Decline_Left_No_Claim(&ledger);

    let _ = std::fs::remove_dir_all(&directory);
}

/// A claim that survives a decline goes on excluding, and erasing the abandonment reason
/// while ending the item is the loss OD-LEDGER-006 exists to stop.
fn Assert_The_Decline_Left_No_Claim(ledger: &Board)
{
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
}

/// Ending somebody's live work is their call, and the refusal says so.
///
/// `OD-LEDGER-001`: territory is declared and not enforced, so the ledger's answer *is* the
/// exclusion. A verb that ended a held item from outside would overrule the only party that
/// knows whether the work is still running.
#[test]
fn Test_Declining_A_Held_Item_Should_Be_Refused_Retryably()
{
    let (directory, mut ledger) = Board_At("held", vec![Item("T-1", &["src/a.rs"])]);
    Take(&mut ledger, "T-1", "agent-a");

    let refusal = Decline_Refused(&mut ledger, "T-1", "agent-b", REASON);

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
    Give_Up(&mut ledger, "T-1", "agent-a", "there is nothing here to do");
    Decline(&mut ledger, "T-1", "agent-b");

    let _ = std::fs::remove_dir_all(&directory);
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
    let (directory, mut ledger) = Board_At("done", vec![Finished_Item("T-1", &["src/a.rs"])]);

    let refusal = Decline_Refused(&mut ledger, "T-1", "agent-a", REASON);
    let item = Only_Item(&ledger);

    assert!(
        !refusal.Is_Retryable(),
        "a Done item never stops being Done, so retrying is a loop and a person is needed"
    );
    assert_eq!(item.state, ItemState::Done, "the verdict was overwritten with prose");
    assert!(item.verified.is_some(), "the verification record was discarded");

    let _ = std::fs::remove_dir_all(&directory);
}

/// A second decline is a duplicate or a disagreement, and both are for a person.
#[test]
fn Test_Declining_A_Declined_Item_Should_Keep_The_First_Reason()
{
    let (directory, mut ledger) = Board_At("twice", vec![Item("T-1", &["src/a.rs"])]);
    Decline(&mut ledger, "T-1", "agent-a");

    let refusal = Decline_Refused(&mut ledger, "T-1", "agent-b", "a different reading entirely");

    assert!(
        refusal.Describe().contains(REASON),
        "the refusal must report the reason already recorded, or the caller cannot tell a \
         duplicate from a disagreement: {}",
        refusal.Describe()
    );
    assert!(!refusal.Is_Retryable());
    Assert_The_Refusal_Stays_One_Line(&mut ledger);
    Assert_The_First_Reason_Survived(&ledger);

    let _ = std::fs::remove_dir_all(&directory);
}

/// The reason `P10-DERIVED-FACT` was declined with runs to five paragraphs, and this refusal
/// printed every one of them with the newlines escaped before `ItemState::Describe` existed.
fn Assert_The_Refusal_Stays_One_Line(ledger: &mut Board)
{
    let long = Decline_Refused(ledger, "T-1", "agent-c", "first line\n\nand four more paragraphs");
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
    let (directory, mut ledger) = Board_At("valid", vec![Item("T-1", &["src/a.rs"])]);
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
    let (directory, mut ledger) = Board_At("unknown", vec![Item("T-1", &["src/a.rs"])]);

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
    let (directory, mut ledger) = Board_At("reachable", vec![Item("T-1", &["src/a.rs"])]);
    ledger
        .Decline(&ItemId::New("T-1"), "agent-a", REASON)
        .expect("the verb is the only thing that may produce this state");

    let after = ledger.Load().expect("readable");
    let item = after.items.first().expect("the item survives");

    assert!(item.state.Is_Finished(), "a declined item is not terminal");
    assert!(!item.state.Is_Claimable(), "a declined item is claimable");

    let _ = std::fs::remove_dir_all(&directory);
}
