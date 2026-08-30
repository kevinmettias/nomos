//! What still serializes them, named rather than assumed.

use crate::board::{
    Constructed_Writers, Is_Open, Only_Records, Paths_Collide, RECORD_DIRECTORY, Record_Writers,
    Unclaimed_Copy, Writer_Ids, Covers,
};
use nomos_ledger::{ItemId, LedgerDocument, LedgerItem, Normalize_Path, Territory};
use std::collections::BTreeSet;

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
/// # Empty, then not, then empty again
///
/// The first two came out in `P10-SURFACE-GRAIN`, in the commit that earned each of them.
/// `crates/spec/nomos-spec-store` was a code coupling until `OD-SPEC-007` dissolved it and
/// a declared-territory coupling for as long as twelve items still reserved the whole
/// crate to seed one record; `tests/contract` was the same shape one level up, an item
/// reserving the snapshot directory to write one file inside it. `OD-LEDGER-011` re-authored
/// both to the artefact — `records/<ID>.record` and `surface/<crate>.txt` — and the two
/// tests below are what confirmed the entries were gone rather than merely deleted.
///
/// The register emptied and then did not stay empty, and the entry that arrived did not
/// outlast the pairing that forced it. `crates/host/nomos-cli` was declared because
/// `P10-VACUITY-HOME` and `P10-SERVICE-SEAM` both reserved the whole crate; `P10-VACUITY-HOME`
/// reached `Done` with `P10-SERVICE-SEAM` alone left reserving it, which is ordinary
/// territory rather than a structural serializer, and `OD-LEDGER-028` named that exact
/// condition as the entry's own expiry in advance.
///
/// What replaced it was `P10-SERVICE-SEAM` against `P11-NEXT-WORK`, over the same
/// not-yet-implemented question as before, keyed to the two narrow files `OD-LEDGER-029`
/// records why it chose over the directory spelling. That pairing has since closed too:
/// `P10-SERVICE-SEAM` was abandoned and re-authored as `P10-SERVICE-SEAM-2`, which reached
/// `Done`, and `P11-NEXT-WORK` reached `Done` separately. `OD-LEDGER-031` records the
/// removal — this time with no third open item taking the collision's place, so the
/// register empties rather than replaces, the state this comment described once before,
/// between `OD-LEDGER-011` and `OD-LEDGER-028`.
const KNOWN_SERIALIZERS: &[(&str, &str)] = &[];

/// The paths the register declares, without what forces each of them.
fn Declared() -> Vec<&'static str>
{
    return KNOWN_SERIALIZERS.iter().map(|(path, _)| return *path).collect();
}

/// An item's territory with one more path on it.
fn Widened(item: &LedgerItem, path: &str) -> Territory
{
    let mut paths = item.territory.paths.clone();
    paths.push(path.to_owned());

    return Territory::Of_Files(paths);
}

/// The non-record paths two items share, narrower spelling first.
fn Shared_Paths(left: &LedgerItem, right: &LedgerItem) -> Vec<String>
{
    let mut shared = Vec::new();

    for mine in &left.territory.paths
    {
        for broader in Broader_Of(mine, right)
        {
            if !shared.contains(&broader)
            {
                shared.push(broader);
            }
        }
    }

    shared.sort();
    return shared;
}

/// The broader spelling of every non-record path in `right` that `mine` collides with.
fn Broader_Of(mine: &str, right: &LedgerItem) -> Vec<String>
{
    if Normalize_Path(mine).starts_with(RECORD_DIRECTORY)
    {
        return Vec::new();
    }

    return right
        .territory
        .paths
        .iter()
        .filter(|theirs| return !Normalize_Path(theirs).starts_with(RECORD_DIRECTORY))
        .filter(|theirs| return Paths_Collide(mine, theirs))
        .map(|theirs| return Broader(mine, theirs))
        .collect();
}

/// The broader of two colliding paths, which is the one that serializes.
///
/// An item reserving a whole crate is what a file inside it collides with, and naming the
/// file would report the symptom. The shorter normalized spelling is the container — the
/// same tie-break [`Covers`] takes the direction of a containment from.
fn Broader(mine: &str, theirs: &str) -> String
{
    if Normalize_Path(mine).len() <= Normalize_Path(theirs).len()
    {
        return Normalize_Path(mine);
    }

    return Normalize_Path(theirs);
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

/// A path *every* record writer has to reserve is one somebody wrote down.
///
/// The assertion that replaces "there must be a concurrent pair", and the discriminator is
/// the whole of its value. Two record writers sharing `crates/host/nomos-cli` are two items
/// that both change the CLI — ordinary contention, which is what territory is for, and
/// which resolves itself when one of them finishes. A path reserved by *all* of them is
/// something a rule forces, and it does not resolve: the next record writer will reserve it
/// too. That is a structural serializer, and both of the ones in the register arrived
/// without anybody noticing.
///
/// Honest over a board with fewer than two writers rather than refusing one.
/// [`Undeclared_Serializers`] answers "nothing" there, because "reserved by *all* of them"
/// is a claim about a population and one item is not a population — with a single writer
/// every path it happens to reserve would read as universal, which is a false positive, not
/// a weaker true answer. `OD-LEDGER-032` measured the cost of demanding two instead: red on
/// 26 of the last 30 commits, for a reason no commit contained. The search's teeth are
/// proved by `Test_An_Undeclared_Serializer_Should_Be_Found`, on a subject built for it.
#[test]
fn Test_Every_Universal_Reservation_Should_Be_Declared()
{
    let document = Unclaimed_Copy();
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

    let stale: Vec<&str> = Declared()
        .into_iter()
        .filter(|declared| return Reserving(&writers, declared) < 2)
        .collect();

    assert!(
        stale.is_empty(),
        "these are declared as serializing the board and fewer than two open record writers \
         reserve them: {stale:?}.\n\
         Either the coupling is gone and the entry should be too, or the board no longer \
         has the items that made it visible."
    );
}

/// How many of these items reserve a path coarsely enough to still be serializing on it.
fn Reserving(writers: &[&LedgerItem], declared: &str) -> usize
{
    return writers
        .iter()
        .filter(|item| return item.territory.paths.iter().any(|path| return Covers(path, declared)))
        .count();
}

/// The control that keeps the census above from being satisfied by an empty search.
///
/// Constructs a board where every record writer also reserves a path nobody declared, and
/// asserts the search finds it. Confirmed by construction rather than by reasoning that it
/// would be found: the whole point of this file is that a property nobody exercised turned
/// out not to hold.
///
/// The subject is built rather than borrowed from the live board. It used to widen whatever
/// `Unclaimed_Copy` happened to hold, which meant the control proved the search had teeth
/// only while somebody was mid-work and proved nothing — while failing — on a board at rest.
/// `OD-LEDGER-032`.
#[test]
fn Test_An_Undeclared_Serializer_Should_Be_Found()
{
    let mut document = Constructed_Writers(2);
    let writers = Writer_Ids(&document);
    let invented = "crates/invented/shared-by-everyone";
    for item in &mut document.items
    {
        if writers.contains(&item.id)
        {
            item.territory = Widened(item, invented);
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
///
/// Nothing, below two writers, and that is the answer rather than a refusal to answer.
/// "Reserved by all of them" is a claim about a population, and one item is not one: with a
/// single writer every path it happens to reserve is trivially reserved by all writers, so
/// the search reports its whole territory as structural. Measured while closing
/// `OD-LEDGER-032` — one open item produced eight false positives, including the record
/// registration and the seven test files the item itself was editing. A false positive here
/// is worse than silence, because the remedy it demands is to remove a coupling that does
/// not exist.
fn Undeclared_Serializers(document: &LedgerDocument) -> BTreeSet<String>
{
    let writers = Record_Writers(document);
    let declared = Declared();

    let Some(first) = First_Writer_If_Comparable(&writers)
    else
    {
        return BTreeSet::new();
    };

    // Candidates come from one writer and are tested against the rest, which is enough:
    // a path all of them reserve is reserved by this one too.
    return first
        .territory
        .paths
        .iter()
        .filter(|candidate| return Serializes(candidate, &writers, &declared))
        .map(|candidate| return Normalize_Path(candidate))
        .collect();
}

/// The writer candidates are drawn from, if there are at least two writers to compare —
/// below that, "reserved by all of them" is not a claim a population of one can make, per
/// [`Undeclared_Serializers`]'s own doc comment.
fn First_Writer_If_Comparable<'a>(writers: &[&'a LedgerItem]) -> Option<&'a LedgerItem>
{
    if writers.len() < 2
    {
        return None;
    }

    return writers.first().copied();
}

/// Whether every record writer reserves this path, and nobody declared it.
fn Serializes(candidate: &str, writers: &[&LedgerItem], declared: &[&str]) -> bool
{
    if Normalize_Path(candidate).starts_with(RECORD_DIRECTORY)
    {
        return false;
    }
    if declared.iter().any(|known| return Paths_Collide(known, candidate))
    {
        return false;
    }

    return writers.iter().all(|item| {
        return item
            .territory
            .paths
            .iter()
            .any(|path| return Paths_Collide(candidate, path));
    });
}

/// Whether the board is parallel today, reported rather than asserted.
///
/// The figure the old acceptance test turned into a pass or a failure. It is worth knowing
/// and it is not a property of this code: it depends on which items happen to be open. So
/// it prints, in both directions, and the line's absence would itself be visible — the
/// same shape `OD-GATE-001` settled on for the corpus gates.
///
/// It said all of that and then asserted the count was non-zero anyway, which is the defect
/// `OD-LEDGER-032` records in miniature: the reasoning that condemns the assertion was
/// already written above it. Reporting is now the whole of what this does.
#[test]
fn Test_A_Run_Should_Report_Whether_The_Board_Is_Parallel()
{
    let document = Unclaimed_Copy();
    let writers = Record_Writers(&document);
    let counted = Exclusions_Among(&writers);
    let parallel = A_Concurrent_Pair(&document)
        .map_or_else(|| return "none".to_owned(), |(first, second)| {
            return format!("{first} + {second}");
        });

    eprintln!(
        "record writers: {} items, {} pair(s), {} blocked, {} of those only by a declared \
         serializer. Concurrent pair available: {parallel}.\n\
         The {} are OD-LEDGER-007's debt; the other {} are ordinary contention.",
        writers.len(),
        counted.pairs,
        counted.blocked,
        counted.structural,
        counted.structural,
        counted.blocked.saturating_sub(counted.structural)
    );

    // The figure printed above is deliberately not asserted against a threshold --
    // `OD-LEDGER-032` is the record of what demanding one cost (red on 26 of the last 30
    // commits, for a reason no commit contained), and reasserting it here would be the same
    // defect in miniature. What *is* asserted is the arithmetic relationship `Count_Exclusion`
    // promises regardless of which items happen to be on the board: a pair the register alone
    // excludes is still an excluded pair, and an excluded pair is still one of the pairs
    // counted. Either inequality breaking would mean the counting itself regressed, which the
    // report above would not by itself reveal -- it would simply print a different number and
    // look no less legitimate for it.
    assert!(
        counted.blocked <= counted.pairs,
        "a blocked pair must be one of the pairs counted: {} blocked of {} pairs",
        counted.blocked,
        counted.pairs
    );
    assert!(
        counted.structural <= counted.blocked,
        "a pair excluded only by a declared serializer is still an excluded pair, so \
         structural must never exceed blocked: {} structural of {} blocked",
        counted.structural,
        counted.blocked
    );
}

/// How many pairs of record writers there are, how many exclude each other, and how many of
/// those are excluded only by a path the register declares.
#[derive(Default)]
struct Exclusions
{
    pairs: usize,
    blocked: usize,
    structural: usize,
}

fn Exclusions_Among(writers: &[&LedgerItem]) -> Exclusions
{
    let declared = Declared();
    let mut counted = Exclusions::default();

    for (index, left) in writers.iter().enumerate()
    {
        for right in writers.iter().skip(index.saturating_add(1))
        {
            counted.pairs = counted.pairs.saturating_add(1);
            Count_Exclusion(&mut counted, (left, right), &declared);
        }
    }

    return counted;
}

/// Whether one pair excludes the other, and whether the register is the only reason.
///
/// Anything else is two items wanting the same crate, which is territory doing its job and
/// resolves when one of them finishes.
fn Count_Exclusion(counted: &mut Exclusions, pair: (&LedgerItem, &LedgerItem), declared: &[&str])
{
    let (left, right) = pair;
    if left.territory.Intersect(&right.territory).Permits_Concurrency()
    {
        return;
    }

    counted.blocked = counted.blocked.saturating_add(1);
    if Shared_Paths(left, right)
        .iter()
        .all(|path| return declared.iter().any(|known| return Paths_Collide(known, path)))
    {
        counted.structural = counted.structural.saturating_add(1);
    }
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
        if Is_Open(item)
        {
            item.territory = On_The_Record_Directory(item);
        }
    }

    assert!(
        A_Concurrent_Pair(&document).is_none(),
        "with every open item reserving `{RECORD_DIRECTORY}` the board must serialize \
         completely, which is the defect this item closed"
    );
    // And the same reconstruction against the property that replaced it. The records-only
    // projection is what `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` claims
    // over, so it has to be red here: under the old authoring every item's record projection
    // is the whole directory, and the whole directory contains every record.
    assert_eq!(
        Independent_Record_Pairs(&document),
        0,
        "under the old authoring the record projection must exclude every pair, or the \
         projection has stopped measuring exclusion and the acceptance test is reporting a \
         success it did not earn"
    );
}

/// An item's territory with its records replaced by the directory holding them, which is the
/// authoring `P10-RECORD-LOCK` replaced.
fn On_The_Record_Directory(item: &LedgerItem) -> Territory
{
    let mut paths: Vec<String> = item
        .territory
        .paths
        .iter()
        .filter(|path| return !Normalize_Path(path).starts_with(RECORD_DIRECTORY))
        .cloned()
        .collect();

    paths.push(RECORD_DIRECTORY.to_owned());

    return Territory::Of_Files(paths);
}

/// How many pairs of record writers are independent over their records alone.
fn Independent_Record_Pairs(document: &LedgerDocument) -> usize
{
    let writers = Record_Writers(document);
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

    return independent;
}
