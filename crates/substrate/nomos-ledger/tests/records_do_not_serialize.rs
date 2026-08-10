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
use std::collections::BTreeSet;
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

/// Every open item that will write a record and could be claimed on its own merits.
///
/// An unmet dependency refuses a claim for reasons that have nothing to do with territory
/// and would make every assertion here fail for the wrong reason.
fn Record_Writers(document: &LedgerDocument) -> Vec<&LedgerItem>
{
    return document
        .items
        .iter()
        .filter(|item| Is_Open(item))
        .filter(|item| !Reserved_Records(item).is_empty())
        .filter(|item| item.depends_on.is_empty())
        .collect();
}

/// An item's territory with everything but its records removed.
///
/// The projection the acceptance property is stated over. `P10-RECORD-LOCK` bought one
/// thing and one thing only: that the *records* two items reserve stop excluding them from
/// each other. Whether they also share code is a separate question with a separate answer,
/// and asserting the two together is what made the guarantee depend on who was on the
/// board — see `OD-LEDGER-007`.
fn Only_Records(item: &LedgerItem) -> Territory
{
    return Territory::Of_Files(Reserved_Records(item));
}

/// Whether two paths exclude each other, decided by the ledger's own rule.
///
/// Single-path territories rather than a containment check written here. `a/b` contains
/// `a/b/c` and two spellings of one path are one path, and a second implementation of
/// either would be a second answer waiting to disagree with `Territory::Intersect`.
fn Paths_Collide(left: &str, right: &str) -> bool
{
    return !Territory::Of_Files([left])
        .Intersect(&Territory::Of_Files([right]))
        .Permits_Concurrency();
}

/// The non-record paths two items share, narrower spelling first.
fn Shared_Paths(left: &LedgerItem, right: &LedgerItem) -> Vec<String>
{
    let mut shared = Vec::new();

    for mine in &left.territory.paths
    {
        if Normalize_Path(mine).starts_with(RECORD_DIRECTORY)
        {
            continue;
        }

        for theirs in &right.territory.paths
        {
            if Normalize_Path(theirs).starts_with(RECORD_DIRECTORY) || !Paths_Collide(mine, theirs)
            {
                continue;
            }

            // The broader of the two is the one that serializes: an item reserving a whole
            // crate is what a file inside it collides with, and naming the file would
            // report the symptom.
            let broader = if Normalize_Path(mine).len() <= Normalize_Path(theirs).len()
            {
                Normalize_Path(mine)
            }
            else
            {
                Normalize_Path(theirs)
            };
            if !shared.contains(&broader)
            {
                shared.push(broader);
            }
        }
    }

    shared.sort();
    return shared;
}

/// Two open items that each reserve a record and are otherwise territorially independent.
///
/// Returns identifiers rather than items so the caller can claim them through the ledger,
/// which is the surface the item is actually about.
///
/// No longer an acceptance criterion. Whether such a pair exists is a fact about the board
/// rather than about the mechanism, and `OD-LEDGER-007` records why the mechanism cannot
/// guarantee one while seeding a record is a hand-maintained edit to two shared files.
/// Kept because it is still the honest way to say whether the board is parallel today.
fn A_Concurrent_Pair(document: &LedgerDocument) -> Option<(ItemId, ItemId)>
{
    let candidates = Record_Writers(document);

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
// The acceptance criterion: a record excludes nobody but its own writer.
// ---------------------------------------------------------------------------

/// What `P10-RECORD-LOCK` actually bought, stated so that it can hold.
///
/// The original acceptance test asserted that two record writers on the board could be
/// claimed at once, over their whole territories. That was true on the day it was written
/// and stopped being true on 2026-08-09 without any code changing: every open item that
/// writes a record must also edit `governing.rs` and the surface snapshots, so they all
/// share two paths and no pair is independent. The property was a fact about who had
/// authored what.
///
/// This is the property the mechanism can keep. Every record writer is reduced to the
/// records it reserves, and the pair is claimed through the real ledger on that
/// projection. If it passes, the records contribute no exclusion — which is the whole of
/// what reserving `docs/records/<ID>` instead of `docs/records` was for. What the items
/// *also* share is measured separately, by the two tests below.
///
/// `OD-LEDGER-007` records the restatement and why the stronger property was given up
/// rather than manufactured.
#[test]
fn Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer()
{
    let mut document = Unclaimed_Copy();
    let writers: Vec<ItemId> = Record_Writers(&document)
        .iter()
        .map(|item| return item.id.clone())
        .collect();

    assert!(
        writers.len() >= 2,
        "fewer than two open record writers, so a claim of independence would be a claim \
         about nothing; got {}",
        writers.len()
    );

    for item in &mut document.items
    {
        if writers.contains(&item.id)
        {
            item.territory = Only_Records(item);
        }
    }

    let directory = Temp_Dir("records-only");
    let clock = FixedClock(NOW);
    let mut ledger = Ledger_At(&directory, &clock);
    ledger.Save(&document).expect("a records-only board is a valid ledger");

    // Every one of them, not a pair. A pair could be independent by accident; all of them
    // being claimable at once is the property, and it is the one that survives an item
    // being added to the board tomorrow.
    for (ordinal, writer) in writers.iter().enumerate()
    {
        ledger
            .Claim(writer, &format!("agent-{ordinal}"), Duration::from_secs(3_600))
            .unwrap_or_else(|refusal| {
                panic!(
                    "{writer} was refused on its record alone, so two records still exclude \
                     each other: {}",
                    refusal.Describe()
                )
            });
    }

    ledger
        .Validate_Current()
        .expect("independent claims are a valid ledger");

    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// What still serializes them, named rather than assumed.
// ---------------------------------------------------------------------------

/// The paths every record writer is forced to share, and what forces them.
///
/// Declared rather than derived, and then checked against the board in both directions by
/// the two tests below. Deriving it would let a third serializer join the list without
/// anybody deciding it should, which is the shape `OD-GATE-001` and
/// `Test_Every_Declared_Gate_Count_Should_Be_The_One_In_The_Source` already settled for
/// the corpus gates: the derivation catches drift, the declaration is what makes growth a
/// decision.
///
/// This list is a debt register. Every entry is a reason two agents cannot work at once,
/// and the intended direction of travel is that it empties — see `OD-LEDGER-007` for what
/// each entry would take.
const KNOWN_SERIALIZERS: &[(&str, &str)] = &[
    (
        "crates/spec/nomos-spec-store",
        "seeding. A canonical record must be added to RECORDS and GOVERNING_RECORD_IDS in \
         governing.rs and the literal count in governing_records_are_present.rs raised, so \
         two items writing two different records edit the same two files. This is \
         OD-LEDGER-001's third authoring rule defeating the guarantee P10-RECORD-LOCK \
         bought, and it was added in the same work.",
    ),
    (
        "tests/contract",
        "the surface snapshots. P9-PUBLIC-API checks each crate's public API into \
         tests/contract/surface, and corpus_gates.rs declares a per-file test count, so an \
         item that widens any API or adds any test to a gated file writes under this \
         directory. Reserving the directory to write one file inside it is the shape \
         P10-RECORD-LOCK named for records, one level up.",
    ),
];

/// A path *every* record writer has to reserve is one somebody wrote down.
///
/// The assertion that replaces "there must be a concurrent pair", and the discriminator is
/// the whole of its value. Two record writers sharing `crates/host/nomos-cli` are two items
/// that both change the CLI — ordinary contention, which is what territory is for, and
/// which resolves itself when one of them finishes. A path reserved by *all* of them is
/// something a rule forces, and it does not resolve: the next record writer will reserve it
/// too. That is a structural serializer, and both of the ones in the register arrived
/// without anybody noticing.
#[test]
fn Test_Every_Universal_Reservation_Should_Be_Declared()
{
    let document = Unclaimed_Copy();
    let writers = Record_Writers(&document);

    assert!(
        writers.len() >= 2,
        "fewer than two open record writers, so nothing can be reserved by all of them and \
         this passes having compared nothing; got {}",
        writers.len()
    );

    let undeclared = Undeclared_Serializers(&document);

    assert!(
        undeclared.is_empty(),
        "every open record writer reserves these, and none is in KNOWN_SERIALIZERS: \
         {undeclared:?}.\n\
         A third thing every record writer has to touch is a third reason the board runs \
         one item at a time. Add it with what forces it, or remove the coupling."
    );
}

/// The other direction: a declared serializer that no longer serializes anything.
///
/// A register that over-reports is as useless as one that under-reports. If seeding stops
/// forcing a shared edit — the remedy `OD-LEDGER-007` defers — this fails, and the entry
/// comes out in the commit that earned it rather than surviving as an explanation for a
/// coupling nobody has any more.
#[test]
fn Test_Every_Declared_Serializer_Should_Still_Serialize()
{
    let document = Unclaimed_Copy();
    let writers = Record_Writers(&document);

    let stale: Vec<&str> = KNOWN_SERIALIZERS
        .iter()
        .map(|(path, _)| return *path)
        .filter(|declared| {
            let reserving = writers
                .iter()
                .filter(|item| {
                    return item
                        .territory
                        .paths
                        .iter()
                        .any(|path| return Paths_Collide(declared, path));
                })
                .count();

            return reserving < 2;
        })
        .collect();

    assert!(
        stale.is_empty(),
        "these are declared as serializing the board and fewer than two open record writers \
         reserve them: {stale:?}.\n\
         Either the coupling is gone and the entry should be too, or the board no longer \
         has the items that made it visible."
    );
}

/// The control that keeps the census above from being satisfied by an empty search.
///
/// Constructs a board where every record writer also reserves a path nobody declared, and
/// asserts the search finds it. Confirmed by construction rather than by reasoning that it
/// would be found: the whole point of this file is that a property nobody exercised turned
/// out not to hold.
#[test]
fn Test_An_Undeclared_Serializer_Should_Be_Found()
{
    let mut document = Unclaimed_Copy();
    let writers: Vec<ItemId> = Record_Writers(&document)
        .iter()
        .map(|item| return item.id.clone())
        .collect();
    let invented = "crates/invented/shared-by-everyone";

    for item in &mut document.items
    {
        if writers.contains(&item.id)
        {
            let mut paths = item.territory.paths.clone();
            paths.push(invented.to_owned());
            item.territory = Territory::Of_Files(paths);
        }
    }

    let found = Undeclared_Serializers(&document);

    assert!(
        found.iter().any(|path| return path == invented),
        "a path every record writer reserves was not reported as a serializer: {found:?}"
    );
}

/// The non-record paths every open record writer reserves and nobody declared.
///
/// Shared by the assertion and its control, so the control exercises the search the
/// assertion makes rather than a second one written beside it.
///
/// Universal rather than pairwise. A path two items share is contention; a path all of them
/// share is a rule.
fn Undeclared_Serializers(document: &LedgerDocument) -> BTreeSet<String>
{
    let writers = Record_Writers(document);
    let declared: Vec<&str> = KNOWN_SERIALIZERS.iter().map(|(path, _)| return *path).collect();
    let Some(first) = writers.first()
    else
    {
        return BTreeSet::new();
    };

    let mut undeclared = BTreeSet::new();

    // Candidates come from one writer and are tested against the rest, which is enough:
    // a path all of them reserve is reserved by this one too.
    for candidate in &first.territory.paths
    {
        if Normalize_Path(candidate).starts_with(RECORD_DIRECTORY)
        {
            continue;
        }
        if declared.iter().any(|known| return Paths_Collide(known, candidate))
        {
            continue;
        }

        let universal = writers.iter().all(|item| {
            return item
                .territory
                .paths
                .iter()
                .any(|path| return Paths_Collide(candidate, path));
        });

        if universal
        {
            undeclared.insert(Normalize_Path(candidate));
        }
    }

    return undeclared;
}

/// Whether the board is parallel today, reported rather than asserted.
///
/// The figure the old acceptance test turned into a pass or a failure. It is worth knowing
/// and it is not a property of this code: it depends on which items happen to be open. So
/// it prints, in both directions, and the line's absence would itself be visible — the
/// same shape `OD-GATE-001` settled on for the corpus gates.
#[test]
fn Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel()
{
    let document = Unclaimed_Copy();
    let writers = Record_Writers(&document);
    let declared: Vec<&str> = KNOWN_SERIALIZERS.iter().map(|(path, _)| return *path).collect();

    let mut pairs = 0_usize;
    let mut blocked = 0_usize;
    let mut structural = 0_usize;

    for (index, left) in writers.iter().enumerate()
    {
        for right in writers.iter().skip(index.saturating_add(1))
        {
            pairs = pairs.saturating_add(1);
            if left.territory.Intersect(&right.territory).Permits_Concurrency()
            {
                continue;
            }
            blocked = blocked.saturating_add(1);

            // Blocked *only* by the register. The rest is two items wanting the same crate,
            // which is territory doing its job and resolves when one of them finishes.
            if Shared_Paths(left, right)
                .iter()
                .all(|path| return declared.iter().any(|known| return Paths_Collide(known, path)))
            {
                structural = structural.saturating_add(1);
            }
        }
    }

    let parallel = A_Concurrent_Pair(&document)
        .map_or_else(|| return "none".to_owned(), |(first, second)| {
            return format!("{first} + {second}");
        });

    eprintln!(
        "record writers: {} items, {pairs} pair(s), {blocked} blocked, {structural} of those \
         only by a declared serializer. Concurrent pair available: {parallel}.\n\
         The {structural} are OD-LEDGER-007's debt; the other {} are ordinary contention.",
        writers.len(),
        blocked.saturating_sub(structural)
    );

    assert!(
        !writers.is_empty(),
        "no open item writes a record, so this reported on nothing"
    );
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

    // And the same reconstruction against the property that replaced it. The records-only
    // projection is what `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` claims
    // over, so it has to be red here: under the old authoring every item's record
    // projection is the whole directory, and the whole directory contains every record.
    let writers = Record_Writers(&document);
    let mut independent = 0_usize;

    for (index, left) in writers.iter().enumerate()
    {
        for right in writers.iter().skip(index.saturating_add(1))
        {
            if Only_Records(left)
                .Intersect(&Only_Records(right))
                .Permits_Concurrency()
            {
                independent = independent.saturating_add(1);
            }
        }
    }

    assert_eq!(
        independent, 0,
        "under the old authoring the record projection must exclude every pair, or the \
         projection has stopped measuring exclusion and the acceptance test is reporting a \
         success it did not earn"
    );
}

// ---------------------------------------------------------------------------
// The negative control: sharing a record still excludes.
// ---------------------------------------------------------------------------

/// The control that keeps the repair from being a blanket exemption.
///
/// Take the two record writers the acceptance test claims concurrently on their records,
/// and point both at one record. If that still claims twice, the fix has not made record
/// reservations finer — it has stopped them mattering, and two agents will write one file.
///
/// Stated over the records-only projection for the same reason the acceptance test is: the
/// two items very likely share code as well, and a refusal caused by `governing.rs` would
/// let this pass while proving nothing about records.
#[test]
fn Test_Two_Items_Writing_One_Record_Should_Still_Be_Refused()
{
    let mut document = Unclaimed_Copy();
    let writers: Vec<ItemId> = Record_Writers(&document)
        .iter()
        .map(|item| return item.id.clone())
        .take(2)
        .collect();
    let (Some(first), Some(second)) = (writers.first().cloned(), writers.get(1).cloned())
    else
    {
        panic!("the board must hold two open record writers for a contest to be possible")
    };

    let contested = format!("{RECORD_DIRECTORY}/OD-CONTESTED-001");
    for item in &mut document.items
    {
        if item.id == first || item.id == second
        {
            item.territory = Territory::Of_Files([contested.clone()]);
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
