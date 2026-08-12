//! The second acceptance criterion: the snapshot is an artefact, not a directory.

use crate::board::{
    Claimed, Contested, Covers, Record_Writers, Saved, Unclaimed_Copy,
};
use nomos_ledger::{ItemId, LedgerDocument, LedgerItem, Normalize_Path, Territory};
use std::collections::BTreeSet;

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

/// The board, and the pair of items widening different crates that the two tests below are
/// both about.
///
/// The pair is derived rather than named, so `blame` is what each caller says when the board
/// no longer holds one.
fn A_Widening_Pair(blame: &str) -> (LedgerDocument, ItemId, ItemId)
{
    let document = Unclaimed_Copy();
    let Some((first, second)) = A_Pair_Widening_Different_Crates(&document)
    else
    {
        panic!("{blame}")
    };

    return (document, first, second);
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
/// [`super::record_exclusion::Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer`]. That
/// test had to project because two record writers genuinely do share code; here the pair is
/// required to be independent outright, because a snapshot grain that only works once the
/// rest is ignored would buy nobody a concurrent claim.
#[test]
fn Test_Two_Items_Widening_Different_Crates_Should_Be_Held_At_Once()
{
    let (document, first, second) = A_Widening_Pair(&format!(
        "no two open record writers widen different crates' APIs and are otherwise \
         independent. Either every such item is back to reserving `{HARNESS_DIRECTORY}` — \
         which is the defect OD-LEDGER-011 closed — or the board no longer holds two items \
         that widen an API at all."
    ));
    let (_scratch, mut ledger) = Saved("snapshot-grain", &document);

    for (writer, agent) in [(&first, "agent-a"), (&second, "agent-b")]
    {
        let blame = format!(
            "{first} and {second} widen different crates' APIs and {writer} was still refused"
        );
        Claimed(&mut ledger, writer, agent, &blame);
    }
    ledger
        .Validate_Current()
        .expect("two independent claims are a valid ledger");
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
    let (mut document, first, second) =
        A_Widening_Pair("the acceptance test's pair must exist for its control to mean anything");
    for item in &mut document.items
    {
        if item.id == first
        {
            item.territory = On_The_Snapshot_Directory(item);
        }
    }
    let (_scratch, refusal) = Contested("snapshot-directory-restored", &document, &first, &second);

    assert!(
        refusal.Describe().contains("agent-a"),
        "{second} must be refused by name once {first} is back on `{HARNESS_DIRECTORY}`, \
         because the two then share nineteen snapshots neither will write: {}",
        refusal.Describe()
    );
}

/// An item's territory with its snapshot files replaced by the directory holding them, which
/// is how every one of these items was authored before `OD-LEDGER-011`.
fn On_The_Snapshot_Directory(item: &LedgerItem) -> Territory
{
    let snapshots = Reserved_Snapshots(item);
    let mut paths: Vec<String> = item
        .territory
        .paths
        .iter()
        .filter(|path| return !snapshots.contains(&Normalize_Path(path)))
        .cloned()
        .collect();

    paths.push(HARNESS_DIRECTORY.to_owned());

    return Territory::Of_Files(paths);
}
