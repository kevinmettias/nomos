//! The behavioural suite for the file-backed ledger, as a file of its own.
//!
//! It is `store/tests/file_ledger.rs` and not an inline `mod tests` for one reason: the type
//! it tests grew past what one file should hold. `check-test-coverage` keys a unit off the
//! FILE STEM -- a test's companion is the file it is textually written in -- so the stem here
//! is `file_ledger`, the same stem as `file_ledger.rs`, and every test below addresses the
//! same unit it addressed when it sat inside that file. The stem is the whole address; the
//! directory it sits in is not read.

use nomos_platform::{Clock, DeterminismStrength, ReproducibilityScope, Strategy, TraceEquivalence};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::time::Duration;

use super::*;
use crate::Claim;
use crate::{ItemKind, ItemOrigin, ItemState};

/// The instant nearly every test here works at, named so that the number is a decision rather
/// than a value that happens to be spelled the same way in every fixture.
const NOW_SECONDS: i64 = 1_000;

/// A second instant, so a test can show the ledger answering its own clock rather than this one.
const LATER_SECONDS: i64 = 4_242;

/// The instant a lapsed claim's lease ran out at.
const LAPSE_SECONDS: i64 = 2_000;

/// The instant the takeover test judges at: past [`LAPSE_SECONDS`], so the claim has lapsed.
const AFTER_LAPSE_SECONDS: i64 = 10_000;

/// The lease a takeover asks for -- an hour, comfortably under the ceiling the ledger enforces.
const ONE_HOUR: Duration = Duration::from_secs(3_600);

/// A clock that never moves, so a test's fixture and its assertions read the same instant the
/// ledger did.
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

#[test]
fn Test_At_Should_Remember_The_Ledger_File_Location()
{
    let directory = Temporary_Directory("at");
    let clock = FixedClock(NOW_SECONDS);
    let target = directory.join("ledger.json");

    let ledger = Ledger_At(&directory, &clock);

    assert_eq!(ledger.Path(), target.as_path());
}

#[test]
fn Test_Path_Should_Return_The_File_This_Ledger_Reads_And_Writes()
{
    let directory = Temporary_Directory("path");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);

    assert_eq!(ledger.Path(), directory.join("ledger.json").as_path());
}

#[test]
fn Test_Read_File_Should_Surface_The_Underlying_Cause_As_Text()
{
    let directory = Temporary_Directory("read-file");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);
    let missing = directory.join("missing.txt");

    let error = ledger.Read_File(&missing).expect_err("a missing file cannot be read");

    assert!(error.contains("does not exist"), "the cause must be legible, got: {error}");
}

#[test]
fn Test_Now_Should_Reflect_The_Ledgers_Own_Clock()
{
    let directory = Temporary_Directory("now");
    let clock = FixedClock(LATER_SECONDS);
    let ledger = Ledger_At(&directory, &clock);

    assert_eq!(ledger.Now(), Timestamp::From_Unix_Seconds(LATER_SECONDS));
}

#[test]
fn Test_Load_Should_Answer_An_Empty_Document_When_Nothing_Was_Written()
{
    let directory = Temporary_Directory("load-missing");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);

    let document = ledger.Load().expect("a missing ledger is an empty one, not an error");

    assert_eq!(document.schema_version, SCHEMA_VERSION);
    assert!(document.items.is_empty());
}

#[test]
fn Test_Save_Should_Write_A_Document_That_Reads_Back_Unchanged()
{
    let directory = Temporary_Directory("save");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);
    let document = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: vec![Workable_Item("SAVE-1")],
    };

    ledger.Save(&document).expect("a valid document must be writable");
    let reloaded = ledger.Load().expect("what was just written must be readable");

    assert_eq!(reloaded, document);
}

#[test]
fn Test_With_Lock_Should_Run_The_Modification_And_Persist_A_Real_Change()
{
    let directory = Temporary_Directory("with-lock");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);
    let item = Workable_Item("LOCK-1");

    let (outcome, takeover) = ledger
        .With_Lock("agent-a", |document| {
            document.items.push(item.clone());
            return Ok(item.id.clone());
        })
        .expect("a plain modification must succeed");

    assert_eq!(outcome, item.id);
    assert!(takeover.is_none(), "a fresh lock is never a stale takeover");
    let reloaded = ledger.Load().expect("the modification must have been written");
    assert_eq!(reloaded.items.len(), 1);
}

#[test]
fn Test_Take_Over_Should_Replace_A_Lapsed_Claim_With_A_Fresh_One()
{
    let directory = Temporary_Directory("take-over");
    let clock = FixedClock(AFTER_LAPSE_SECONDS);
    let mut ledger = Ledger_Holding_A_Lapsed_Claim(&directory, &clock);

    let reservation = ledger
        .Take_Over(&ItemId::New("TAKE-1"), "agent-b", ONE_HOUR)
        .expect("a lapsed claim must be takeable");

    Assert_Taken_Over(&ledger, &reservation);
}

#[test]
fn Test_Decline_Should_End_A_Ready_Item_With_No_Claim_At_All()
{
    let directory = Temporary_Directory("decline");
    let clock = FixedClock(NOW_SECONDS);
    let mut ledger = Ledger_At(&directory, &clock);
    ledger
        .Save(&LedgerDocument {
            schema_version: SCHEMA_VERSION,
            items: vec![Workable_Item("DECLINE-1")],
        })
        .expect("a ready item is a valid document");

    ledger
        .Decline(&ItemId::New("DECLINE-1"), "agent-a", "superseded")
        .expect("a ready, unclaimed item may be declined");

    let reloaded = ledger.Load().expect("the decline must have been written");
    let declined = reloaded.items.first().expect("the item survives its decline");
    assert_eq!(
        declined.state,
        ItemState::Declined { reason: "superseded".to_owned() }
    );
}

#[test]
fn Test_Add_Should_Put_A_New_Item_On_The_Board()
{
    let directory = Temporary_Directory("add");
    let clock = FixedClock(NOW_SECONDS);
    let mut ledger = Ledger_At(&directory, &clock);
    let item = Workable_Item("ADD-1");

    ledger
        .Add(&item, "agent-a", &Territory::Empty(), &Territory::Empty())
        .expect("a fresh identifier over an empty board must be accepted");

    let reloaded = ledger.Load().expect("the add must have been written");
    assert_eq!(reloaded.items.len(), 1);
    assert_eq!(reloaded.items.first().expect("the assertion above found exactly one item").id, item.id);
}

#[test]
fn Test_Validate_Current_Should_Report_A_Duplicate_Identifier_Written_To_Disk()
{
    let directory = Temporary_Directory("validate-current");
    let clock = FixedClock(NOW_SECONDS);
    let ledger = Ledger_At(&directory, &clock);
    Write_Duplicated_Identifiers_Directly(&ledger);

    let error = ledger
        .Validate_Current()
        .expect_err("a repeated identifier violates an invariant");

    Assert_Refused_A_Duplicate_Identifier(&error);
}

/// A ledger holding one item whose claim by `dead-agent` ran out at [`LAPSE_SECONDS`], read at a
/// clock past that instant.
///
/// The claim is written through `Save` rather than hand-edited, so what the takeover faces is a
/// document the store itself calls valid.
fn Ledger_Holding_A_Lapsed_Claim<'clock>(
    directory: &Path,
    clock: &'clock FixedClock,
) -> FileLedger<StdFileSystem, &'clock FixedClock, FileLock>
{
    let ledger = Ledger_At(directory, clock);
    let mut item = Workable_Item("TAKE-1");
    item.state = ItemState::Claimed;
    item.claim = Some(Claim {
        holder: "dead-agent".to_owned(),
        acquired_at: Timestamp::From_Unix_Seconds(NOW_SECONDS),
        lease_expires_at: Timestamp::From_Unix_Seconds(LAPSE_SECONDS),
    });
    ledger
        .Save(&LedgerDocument { schema_version: SCHEMA_VERSION, items: vec![item] })
        .expect("a claimed item is still a valid document");
    return ledger;
}

/// What a takeover must have done: the new holder holds it, and the displaced claim is kept.
fn Assert_Taken_Over(
    ledger: &FileLedger<StdFileSystem, &FixedClock, FileLock>,
    reservation: &Reservation,
)
{
    assert_eq!(reservation.holder, "agent-b");
    let reloaded = ledger.Load().expect("the takeover must have been written");
    let taken = reloaded.items.first().expect("the item survives its takeover");
    assert_eq!(
        taken.claim.as_ref().map(|claim| return claim.holder.as_str()),
        Some("agent-b")
    );
    assert_eq!(taken.displaced.len(), 1, "the displaced claim must be kept, not dropped");
}

/// The fixture document written straight past `Save`, which would refuse it.
///
/// This test is about `Validate_Current` surfacing what is already on disk, not about `Save`'s
/// own guard, so the corruption has to reach the file without going through the guard.
fn Write_Duplicated_Identifiers_Directly(ledger: &FileLedger<StdFileSystem, &FixedClock, FileLock>)
{
    let duplicated = LedgerDocument {
        schema_version: SCHEMA_VERSION,
        items: vec![Workable_Item("DUP-1"), Workable_Item("DUP-1")],
    };
    let raw = serde_json::to_string(&duplicated).expect("the fixture document serializes");
    std::fs::write(ledger.Path(), raw).expect("test can write the raw fixture directly");
}

/// The refusal must name the repeated identifier, not merely report that something is wrong.
fn Assert_Refused_A_Duplicate_Identifier(error: &LedgerError)
{
    assert!(
        matches!(
            error,
            LedgerError::Invalid { violations } if violations.iter().any(|line| line.contains("more than once"))
        ),
        "expected a duplicate-identifier violation, got {error:?}"
    );
}

/// A scratch directory this test owns outright, named for the test so two tests running at once
/// never share one file.
fn Temporary_Directory(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-file-ledger-{name}-{}", std::process::id()));
    if path.exists()
    {
        std::fs::remove_dir_all(&path).expect("the stale scratch directory must be removable");
    }
    std::fs::create_dir_all(&path).expect("test needs a temp directory");
    return path;
}

/// A real ledger over a real file, exactly as every caller of this type gets one.
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

/// A `Ready` item with a non-empty territory of its own, so it can sit on a board without itself
/// violating [`Validate_Document`]'s "reserves nothing" rule.
fn Workable_Item(id: &str) -> LedgerItem
{
    return LedgerItem {
        id: ItemId::New(id),
        title: "an item".to_owned(),
        why: "because".to_owned(),
        done_when: "when it is done".to_owned(),
        kind: ItemKind::Correction,
        origin: ItemOrigin::Proposed,
        territory: Territory::Of_Files([format!("src/{id}.rs")]),
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
