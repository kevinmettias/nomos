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

/// The clock every test that does not move time shares.
///
/// A `'static` clock rather than a local one in each test: nothing here moves time, so one
/// shared reading is what every ledger these tests open is read against.
static AT_NOW: FixedClock = FixedClock(NOW);

/// The board, and the identifiers of the first two open items that write records.
///
/// Every contest below needs exactly this: a copy nobody holds, and two writers to set
/// against each other. A board without two of them cannot host a contest, so that is a
/// panic rather than a test that passes having contested nothing.
fn Two_Record_Writers() -> (LedgerDocument, ItemId, ItemId)
{
    let document = Unclaimed_Copy();
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

    return (document, first, second);
}

/// Puts a doctored board on disk and lets two agents contest it, first come first served.
///
/// The refusal handed back is the second agent's, which is what every test using this is
/// about; the directory comes back so the caller can clean it up.
fn Contested(
    name: &str,
    document: &LedgerDocument,
    first: &ItemId,
    second: &ItemId,
) -> (PathBuf, ClaimRefusal)
{
    let directory = Temp_Dir(name);
    let mut ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(document).expect("the doctored board is still a valid ledger");

    ledger
        .Claim(first, "agent-a", Duration::from_secs(3_600))
        .expect("the first claim is uncontended");
    let refusal = ledger
        .Claim(second, "agent-b", Duration::from_secs(3_600))
        .expect_err("the second of a contesting pair must be refused");

    return (directory, refusal);
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

/// Whether reserving `reserved` reserves `declared` — the coarse direction, not either one.
///
/// [`Paths_Collide`] answers a symmetric question, because exclusion is symmetric: an item
/// reserving a directory and an item reserving a file inside it exclude each other, and
/// which of them is broader does not change that. Whether a *declared serializer* is still
/// serializing is not that question. It asks whether anybody still reserves the coarse path
/// itself, and under the symmetric reading `tests/contract/surface/nomos-ledger.txt` counts
/// as reserving `tests/contract` — so narrowing a reservation never reduces the count and
/// only deleting the item does. The register could not empty, which is the one thing
/// `OD-LEDGER-007` says it is for. Found while closing `P10-SURFACE-GRAIN`; `OD-LEDGER-011`
/// records it.
///
/// The direction is taken from the collision rather than decided again beside it. If two
/// paths collide under this ledger's rule then one contains the other, and the shorter
/// normalized spelling is the container — the same tie-break [`Shared_Paths`] already uses
/// to name the broader of two paths. A containment check written out here would be a second
/// answer waiting to disagree with `Territory::Intersect`.
fn Covers(reserved: &str, declared: &str) -> bool
{
    return Paths_Collide(reserved, declared)
        && Normalize_Path(reserved).len() <= Normalize_Path(declared).len();
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
    let mut ledger = Ledger_At(&directory, &AT_NOW);
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
// The second acceptance criterion: the snapshot is an artefact, not a directory.
// ---------------------------------------------------------------------------

/// The directory holding the whole harness, and the coarse reservation this item removed.
const HARNESS_DIRECTORY: &str = "tests/contract";

/// Where `P9-PUBLIC-API` checks each crate's public API, one file per crate.
const SNAPSHOT_DIRECTORY: &str = "tests/contract/surface";

/// The snapshot *files* an item reserves, as against the directory holding them.
///
/// An item reserving `tests/contract` reserves every crate's snapshot and appears here as
/// nothing, which is the whole of the distinction the test below is about: the directory is
/// twenty-one crates' surfaces, and an item that widens one API will write one of them.
fn Reserved_Snapshots(item: &LedgerItem) -> BTreeSet<String>
{
    return item
        .territory
        .paths
        .iter()
        .map(|path| return Normalize_Path(path))
        .filter(|path| {
            return Covers(SNAPSHOT_DIRECTORY, path) && path.as_str() != SNAPSHOT_DIRECTORY;
        })
        .collect();
}

/// Two record writers that widen different crates' public APIs and share no territory.
///
/// Derived rather than named. Two identifiers written here would be right until one of them
/// finished, and an acceptance criterion that expires the moment its example is done is the
/// failure `OD-LEDGER-007` recorded about the pair search this replaces.
fn A_Pair_Widening_Different_Crates(document: &LedgerDocument) -> Option<(ItemId, ItemId)>
{
    let widening: Vec<(&LedgerItem, BTreeSet<String>)> = Record_Writers(document)
        .into_iter()
        .map(|item| return (item, Reserved_Snapshots(item)))
        .filter(|(_, snapshots)| return !snapshots.is_empty())
        .collect();

    for (index, (left, mine)) in widening.iter().enumerate()
    {
        for (right, theirs) in widening.iter().skip(index.saturating_add(1))
        {
            if mine.intersection(theirs).next().is_some()
            {
                continue;
            }
            if left.territory.Intersect(&right.territory).Permits_Concurrency()
            {
                return Some((left.id.clone(), right.id.clone()));
            }
        }
    }
    return None;
}

/// The property `P10-SURFACE-GRAIN` bought, claimed through the ledger rather than argued.
///
/// Two items each widening one crate's public API are two items writing two different files
/// in one directory. Reserving the directory made them exclude each other over nineteen
/// snapshots neither would touch; reserving the file they will write does not. Nothing in
/// the mechanism ever prevented the finer grain — `Territory::Intersect` has always compared
/// by containment — so this is a property of what the items *say*, which is why it is
/// asserted against the repository's own board and not a fixture.
///
/// Stated over the whole territory and not over a projection, unlike
/// [`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`]. That test had to project
/// because two record writers genuinely do share code; here the pair is required to be
/// independent outright, because a snapshot grain that only works once the rest is ignored
/// would buy nobody a concurrent claim.
#[test]
fn Test_Two_Items_Widening_Different_Crates_Should_Be_Held_At_Once()
{
    let document = Unclaimed_Copy();

    let Some((first, second)) = A_Pair_Widening_Different_Crates(&document)
    else
    {
        panic!(
            "no two open record writers widen different crates' APIs and are otherwise \
             independent. Either every such item is back to reserving `{HARNESS_DIRECTORY}` \
             — which is the defect OD-LEDGER-011 closed — or the board no longer holds two \
             items that widen an API at all."
        )
    };

    let directory = Temp_Dir("snapshot-grain");
    let mut ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&document).expect("the real board is a valid ledger");

    for (writer, agent) in [(&first, "agent-a"), (&second, "agent-b")]
    {
        ledger
            .Claim(writer, agent, Duration::from_secs(3_600))
            .unwrap_or_else(|refusal| {
                panic!(
                    "{first} and {second} widen different crates' APIs and {writer} was \
                     still refused: {}",
                    refusal.Describe()
                )
            });
    }

    ledger
        .Validate_Current()
        .expect("two independent claims are a valid ledger");

    let _ = std::fs::remove_dir_all(&directory);
}

/// The control that keeps the test above from passing for some other reason.
///
/// Takes the same pair and puts one of them back on the bare `tests/contract`, which is how
/// every one of these items was authored before `OD-LEDGER-011`. The second claim must then
/// be refused and must name the holder of the first. If it is not, the pair above was
/// independent for a reason that has nothing to do with the snapshot grain and the
/// acceptance test is reporting a success it did not earn.
#[test]
fn Test_Restoring_The_Snapshot_Directory_Should_Refuse_The_Pair()
{
    let mut document = Unclaimed_Copy();

    let Some((first, second)) = A_Pair_Widening_Different_Crates(&document)
    else
    {
        panic!("the acceptance test's pair must exist for its control to mean anything")
    };

    for item in &mut document.items
    {
        if item.id != first
        {
            continue;
        }
        let snapshots = Reserved_Snapshots(item);
        let mut paths: Vec<String> = item
            .territory
            .paths
            .iter()
            .filter(|path| return !snapshots.contains(&Normalize_Path(path)))
            .cloned()
            .collect();
        paths.push(HARNESS_DIRECTORY.to_owned());
        item.territory = Territory::Of_Files(paths);
    }

    let (directory, refusal) = Contested("snapshot-directory-restored", &document, &first, &second);

    assert!(
        refusal.Describe().contains("agent-a"),
        "{second} must be refused by name once {first} is back on `{HARNESS_DIRECTORY}`, \
         because the two then share nineteen snapshots neither will write: {}",
        refusal.Describe()
    );

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
///
/// # It is empty, and that is a state rather than an absence
///
/// Both entries came out in `P10-SURFACE-GRAIN`, in the commit that earned each of them.
/// `crates/spec/nomos-spec-store` was a code coupling until `OD-SPEC-007` dissolved it and
/// a declared-territory coupling for as long as twelve items still reserved the whole
/// crate to seed one record; `tests/contract` was the same shape one level up, an item
/// reserving the snapshot directory to write one file inside it. `OD-LEDGER-011` re-authored
/// both to the artefact — `records/<ID>.record` and `surface/<crate>.txt` — and the two
/// tests below are what confirmed the entries were gone rather than merely deleted.
///
/// An empty register does not make this file vacuous. The register is the *declared* half;
/// [`Test_Every_Universal_Reservation_Should_Be_Declared`] is the derived half, and it is
/// the one that fails when a third structural serializer arrives. The declaration is what
/// makes growth a decision, and a decision has to be able to start from nothing.
const KNOWN_SERIALIZERS: &[(&str, &str)] = &[];

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
///
/// Counted with [`Covers`] and not with [`Paths_Collide`]. See that function for why the
/// symmetric reading made the register unemptiable, which is the defect `OD-LEDGER-011`
/// found while closing.
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
                        .any(|path| return Covers(path, declared));
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
    let (mut document, first, second) = Two_Record_Writers();

    let contested = format!("{RECORD_DIRECTORY}/OD-CONTESTED-001");
    for item in &mut document.items
    {
        if item.id == first || item.id == second
        {
            item.territory = Territory::Of_Files([contested.clone()]);
        }
    }

    let (directory, refusal) = Contested("contested-record", &document, &first, &second);

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

// ---------------------------------------------------------------------------
// OD-LEDGER-016: the identifier and the file it names are one record.
// ---------------------------------------------------------------------------

/// The record identifier an item reserves, and the file that identifier was allocated for.
///
/// Both spellings written out, because the whole of the defect is that they used to be
/// three characters of shared text away from each other and `Territory::Intersect` compared
/// them as siblings. `OD-LEDGER-006` is a record that exists, which is what makes this the
/// amendment case rather than the allocation case.
const AN_IDENTIFIER: &str = "docs/records/OD-LEDGER-006";
const THE_FILE_IT_NAMES: &str =
    "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md";

/// The property `OD-LEDGER-016` buys, claimed through the ledger rather than argued.
///
/// One item reserves a record by the identifier `OD-LEDGER-001`'s authoring rule tells it
/// to reserve; another names the file outright, which is what an item amending a record
/// that already exists was reduced to doing when the identifier turned out to reserve
/// nothing. The second claim must be refused by name.
///
/// Stated over a records-only projection for the same reason
/// [`Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`] is: two items on this board
/// share code as well, and a refusal caused by `nomos-cli` would let this pass while
/// proving nothing about records.
#[test]
fn Test_An_Item_Naming_A_Records_File_Should_Be_Refused_By_Its_Identifiers_Holder()
{
    let (mut document, first, second) = Two_Record_Writers();

    for item in &mut document.items
    {
        if item.id == first
        {
            item.territory = Territory::Of_Files([AN_IDENTIFIER]);
        }
        else if item.id == second
        {
            item.territory = Territory::Of_Files([THE_FILE_IT_NAMES]);
        }
    }

    let directory = Temp_Dir("record-stem");
    let mut ledger = Ledger_At(&directory, &AT_NOW);
    ledger.Save(&document).expect("still a valid ledger");

    ledger
        .Claim(&first, "agent-a", Duration::from_secs(3_600))
        .expect("the first claim is uncontended");

    let refusal = ledger
        .Claim(&second, "agent-b", Duration::from_secs(3_600))
        .expect_err(
            "`docs/records/OD-LEDGER-006` and the file it names are one record, so the \
             second of them must not be claimable",
        );

    assert!(
        matches!(refusal, ClaimRefusal::HeldBy { .. }),
        "the refusal must name the holder: {}",
        refusal.Describe()
    );
    assert!(
        refusal.Describe().contains("agent-a"),
        "{second} named `{THE_FILE_IT_NAMES}` while agent-a held `{AN_IDENTIFIER}`, and the \
         two are one record: {}",
        refusal.Describe()
    );

    let _ = std::fs::remove_dir_all(&directory);
}

/// Every record in `docs/records`, as the identifier it declares and the file it is.
///
/// The identifier is read from the record's own front matter rather than from its filename,
/// so the two sides of the comparison below are written down independently — which is the
/// distinction `OD-SPEC-007` drew between a check and a derivation that compares a
/// directory against itself.
fn Records_On_Disk() -> Vec<(String, String)>
{
    let directory = Repository_Root().join("docs").join("records");
    let mut records = Vec::new();

    for entry in std::fs::read_dir(&directory).expect("the record directory must be readable")
    {
        let path = entry.expect("a record directory entry must read").path();
        let Some(name) = path.file_name().and_then(|name| return name.to_str()).map(str::to_owned)
        else
        {
            continue;
        };
        // Case-insensitively, because the reservation is folded case-insensitively and a
        // record shouted onto disk is still a record.
        let is_record = path
            .extension()
            .is_some_and(|extension| return extension.eq_ignore_ascii_case("md"));
        if !is_record
        {
            continue;
        }

        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{name} must be readable: {error}"));
        let identifier = text
            .lines()
            .find_map(|line| return line.trim().strip_prefix("id:").map(|id| return id.trim().to_owned()))
            .unwrap_or_else(|| panic!("{name} must declare an id in its front matter"));

        records.push((identifier, format!("{RECORD_DIRECTORY}/{name}")));
    }

    return records;
}

/// The rule against every record this repository actually has.
///
/// A fixture would prove the folding works on the one filename somebody thought to write
/// into it. This asks the ledger's own comparison about all of them, and so it is also the
/// guard on the identifier grammar: a record named in a shape the reduction does not read —
/// or a record whose filename has drifted from the identifier in its own front matter —
/// fails here rather than by silently reserving nothing when somebody comes to amend it.
#[test]
fn Test_Every_Record_On_Disk_Should_Be_One_Subject_With_Its_Identifier()
{
    let records = Records_On_Disk();

    assert!(
        records.len() >= 20,
        "the record directory must hold the records this compares; got {}",
        records.len()
    );

    let divergent: Vec<String> = records
        .iter()
        .filter(|(identifier, path)| {
            return !Paths_Collide(&format!("{RECORD_DIRECTORY}/{identifier}"), path);
        })
        .map(|(identifier, path)| return format!("{identifier} != {path}"))
        .collect();

    assert!(
        divergent.is_empty(),
        "reserving these records by identifier reserves nothing, because the identifier and \
         the file do not fold onto one subject: {divergent:?}.\n\
         Either the filename does not begin with the identifier its front matter declares, \
         or the identifier is not in a shape `Normalize_Path` reads."
    );
}
