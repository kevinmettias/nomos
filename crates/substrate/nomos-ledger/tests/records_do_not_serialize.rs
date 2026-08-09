//! P10-RECORD-LOCK acceptance: writing a record must not serialize the board.
//!
//! Every assertion here runs against the repository's own `work/ledger.json` rather than
//! against a fixture. That is deliberate and it is what the item asked for: the defect was
//! never in the exclusion code, which has always compared paths correctly. It was in what
//! the items *say*, so a fixture proving that two invented territories are disjoint would
//! have passed on the day the whole board was blocked.
//!
//! The claims are exercised against a copy in a temporary directory. A test that claimed on
//! the real ledger would take territory from whoever is working the repository while it
//! runs, and a suite with side effects on the thing it measures is not a suite.

use nomos_ledger::{
    ClaimRefusal, ExclusionLedger, FileLedger, ItemId, ItemState, LedgerDocument, LedgerItem,
    Normalize_Path, Territory,
};
use nomos_platform::{Clock, Timestamp};
use nomos_platform_std::{FileLock, StdFileSystem};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The directory every record in this repository lives in, normalized.
const RECORD_DIRECTORY: &str = "docs/records";

struct FixedClock(i64);

impl Clock for &FixedClock
{
    fn Now(&self) -> Timestamp
    {
        return Timestamp::From_Unix_Seconds(self.0);
    }
}

const NOW: i64 = 1_000_000;

/// The repository root, from this crate's manifest directory.
fn Repository_Root() -> PathBuf
{
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // crates/substrate/nomos-ledger -> the repository.
    for _ in 0..3
    {
        root.pop();
    }
    return root;
}

/// The ledger this repository actually runs on.
fn Real_Ledger() -> LedgerDocument
{
    let path = Repository_Root().join("work").join("ledger.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the repository ledger must be readable: {error}"));
    return serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("the repository ledger must parse: {error}"));
}

/// Whether an item is still competing for territory.
///
/// `Done` and `Declined` items hold no claim and exclude nobody, so their territories are
/// history rather than reservations and are deliberately left as they were authored.
fn Is_Open(item: &LedgerItem) -> bool
{
    return matches!(item.state, ItemState::Ready | ItemState::Claimed);
}

/// The record paths an item reserves.
fn Reserved_Records(item: &LedgerItem) -> Vec<String>
{
    return item
        .territory
        .paths
        .iter()
        .map(|path| Normalize_Path(path))
        .filter(|path| path.starts_with(RECORD_DIRECTORY))
        .collect();
}

fn Temp_Dir(name: &str) -> PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-record-lock-{name}-{}", std::process::id()));
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

/// The real ledger with every live claim cleared.
///
/// Whoever happens to be holding an item when the suite runs must not change what it
/// concludes, and the question here is whether two items *could* be held at once — not
/// whether they are free this second.
fn Unclaimed_Copy() -> LedgerDocument
{
    let mut document = Real_Ledger();
    for item in &mut document.items
    {
        if Is_Open(item)
        {
            item.state = ItemState::Ready;
            item.claim = None;
        }
    }
    return document;
}

/// Two open items that each reserve a record and are otherwise territorially independent.
///
/// Returns identifiers rather than items so the caller can claim them through the ledger,
/// which is the surface the item is actually about.
fn A_Concurrent_Pair(document: &LedgerDocument) -> Option<(ItemId, ItemId)>
{
    let candidates: Vec<&LedgerItem> = document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| !Reserved_Records(item).is_empty())
        // An unmet dependency refuses a claim for reasons that have nothing to do with
        // territory, and would make this test fail for the wrong reason.
        .filter(|item| item.depends_on.is_empty())
        .collect();

    for (index, left) in candidates.iter().enumerate()
    {
        for right in candidates.iter().skip(index.saturating_add(1))
        {
            if left.territory.Intersect(&right.territory).Permits_Concurrency()
            {
                return Some((left.id.clone(), right.id.clone()));
            }
        }
    }
    return None;
}

// ---------------------------------------------------------------------------
// The rule: no open item reserves the directory every record lives in.
// ---------------------------------------------------------------------------

/// The straggler guard, and the reason this file reads the real ledger.
///
/// One item left claiming `docs/records` contains every record anyone else could write, so
/// it re-blocks the whole board by itself. The fix is therefore not incremental and cannot
/// be kept by review alone.
#[test]
fn Test_No_Open_Item_Should_Reserve_The_Whole_Record_Directory()
{
    let document = Real_Ledger();

    let offenders: Vec<String> = document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| {
            item.territory
                .paths
                .iter()
                .any(|path| Normalize_Path(path) == RECORD_DIRECTORY)
        })
        .map(|item| item.id.As_Str().to_owned())
        .collect();

    assert!(
        offenders.is_empty(),
        "an open item reserving `{RECORD_DIRECTORY}` excludes every other record-writing \
         item, which is the defect P10-RECORD-LOCK closed. Reserve the record the item will \
         write instead. Offenders: {offenders:?}"
    );
}

/// Guards the test above against passing because there is nothing to check. A board with
/// no open record-writing items would satisfy it vacuously.
#[test]
fn Test_The_Board_Should_Have_Record_Writing_Items_To_Talk_About()
{
    let document = Real_Ledger();

    let writers = document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| !Reserved_Records(item).is_empty())
        .count();

    assert!(
        writers >= 2,
        "fewer than two open items reserve a record, so the concurrency this file measures \
         cannot be observed; got {writers}"
    );
}

// ---------------------------------------------------------------------------
// The case that exists right now: two of them claim at once.
// ---------------------------------------------------------------------------

/// The acceptance criterion, over the ledger as it actually stands.
///
/// Before P10-RECORD-LOCK this could not pass for any pair on the board: all nine open
/// items reserved `docs/records`, so the second claim was always refused.
#[test]
fn Test_Two_Items_Writing_Different_Records_Should_Be_Claimable_At_Once()
{
    let document = Unclaimed_Copy();
    let (first, second) =
        A_Concurrent_Pair(&document).expect("the board must offer two independent record writers");

    let directory = Temp_Dir("concurrent-pair");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    ledger.Save(&document).expect("the real ledger is valid");

    ledger
        .Claim(&first, "agent-a", Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| panic!("the first claim is uncontended: {}", refusal.Describe()));

    ledger
        .Claim(&second, "agent-b", Duration::from_secs(3_600))
        .unwrap_or_else(|refusal| {
            panic!(
                "{second} writes a different record from {first} and touches different code, so \
                 it must be claimable concurrently: {}",
                refusal.Describe()
            )
        });

    ledger
        .Validate_Current()
        .expect("two independent claims are a valid ledger");

    let _ = std::fs::remove_dir_all(&directory);
}

/// The control for the acceptance test above, and the reason it cannot pass vacuously.
///
/// Reconstructs in memory the authoring this item replaced — every open item reserving the
/// record *directory* — and asserts that no pair on the board could be held at once. That
/// was the measured state on 2026-08-09: one claim, eight refusals.
///
/// Keeping it as a test rather than as a sentence in a record is the point. If it ever
/// passes, the pair search has stopped measuring exclusion and the acceptance test above is
/// reporting a success it did not earn.
#[test]
fn Test_The_Old_Authoring_Should_Offer_No_Concurrent_Pair()
{
    let mut document = Unclaimed_Copy();

    for item in &mut document.items
    {
        if !Is_Open(item)
        {
            continue;
        }
        let mut paths: Vec<String> = item
            .territory
            .paths
            .iter()
            .filter(|path| !Normalize_Path(path).starts_with(RECORD_DIRECTORY))
            .cloned()
            .collect();
        paths.push(RECORD_DIRECTORY.to_owned());
        item.territory = Territory::Of_Files(paths);
    }

    assert!(
        A_Concurrent_Pair(&document).is_none(),
        "with every open item reserving `{RECORD_DIRECTORY}` the board must serialize \
         completely, which is the defect this item closed"
    );
}

// ---------------------------------------------------------------------------
// The negative control: sharing a record still excludes.
// ---------------------------------------------------------------------------

/// The control that keeps the repair from being a blanket exemption.
///
/// Take the very pair the test above claims concurrently and point both at one record.
/// If that still claims twice, the fix has not made record reservations finer — it has
/// stopped them mattering, and two agents will write one file.
#[test]
fn Test_Two_Items_Writing_One_Record_Should_Still_Be_Refused()
{
    let mut document = Unclaimed_Copy();
    let (first, second) =
        A_Concurrent_Pair(&document).expect("the board must offer two independent record writers");

    let contested = format!("{RECORD_DIRECTORY}/OD-CONTESTED-001");
    for item in &mut document.items
    {
        if item.id == first || item.id == second
        {
            let mut kept: Vec<String> = item
                .territory
                .paths
                .iter()
                .filter(|path| !Normalize_Path(path).starts_with(RECORD_DIRECTORY))
                .cloned()
                .collect();
            kept.push(contested.clone());
            item.territory = Territory::Of_Files(kept);
        }
    }

    let directory = Temp_Dir("contested-record");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    ledger.Save(&document).expect("still a valid ledger");

    ledger
        .Claim(&first, "agent-a", Duration::from_secs(3_600))
        .expect("the first claim is uncontended");

    let refusal = ledger
        .Claim(&second, "agent-b", Duration::from_secs(3_600))
        .expect_err("two items writing one record must not both be claimable");

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the refusal must name the holder: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("agent-a"),
        "{}",
        refusal.Describe()
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// The spelling control. A finer grain must not reintroduce the hole
/// [`Normalize_Path`] exists to close: two spellings of one record are one record.
#[test]
fn Test_Two_Spellings_Of_One_Record_Should_Still_Collide()
{
    let lower = Territory::Of_Files(["docs/records/OD-LEDGER-004"]);
    let shouted = Territory::Of_Files([r"docs\Records\OD-LEDGER-004"]);

    assert!(
        !lower.Intersect(&shouted).Permits_Concurrency(),
        "case and separator folding must survive the move to record-level reservations"
    );
}
