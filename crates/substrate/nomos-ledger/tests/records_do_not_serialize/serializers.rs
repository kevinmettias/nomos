//! What still serializes them, named rather than assumed.

use crate::board::{
    Constructed_Writers, Is_Covering, CONTESTING_WRITERS, DeclaredPath, Is_Open, Only_Records,
    PathText, Is_Colliding, RECORD_DIRECTORY, Record_Writers, ReservedPath, Unclaimed_Copy,
    Writer_Ids,
};
use nomos_ledger::{ItemId, LedgerDocument, LedgerItem, Normalize_Path, Territory};
use std::collections::BTreeSet;

mod census;
use census::{Exclusions, Exclusions_Among};

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
///
/// # The rules facade, and the argument it left unfinished
///
/// It did not stay empty that time either. `crates/rules/nomos-rules/src/lib.rs` was declared
/// on 2026-09-22 over three open record writers -- `P110-B`, `P113` and `P114` -- because the
/// file re-exports every check the crate publishes by name, so adding a rule, renaming one or
/// regrouping the modules rewrites it, and any record writer doing one of those three inherited
/// the reservation. `OD-LEDGER-029` records why the entry was keyed to that file rather than to
/// the crate: [`Is_Covering`] is coarser-reserves-finer, so a crate-keyed entry would have
/// counted only `P110-B` and read as stale in the commit that wrote it.
///
/// **Two readings of that report were live at once and the disagreement was never settled.**
/// One session argued `P110-B` alone caused it, by reserving eleven whole crates where the
/// other two reserve narrow files, so that folding took everything up to the crate root; that
/// session had first proposed raising [`WRITERS_MAKING_A_SERIALIZER`] instead and withdrew it.
/// A later measurement recorded here reached the opposite conclusion: that setting `P110-B`
/// aside left two writers and *two* undeclared universal paths rather than none,
/// `crates/rules/nomos-rules/src/checks` and `src/lib.rs`, so the territory of that one item was
/// not what produced the report. Both measurements are preserved because they cannot both be
/// right and nothing since has adjudicated them.
///
/// **The entry expired on its own stated terms rather than on that argument.** All three writers
/// ended -- `P113` and `P114` `Done`, `P110-B` `Declined` on 2026-09-26 in favour of a successor
/// reserving three paths instead of fifteen -- and the entry's condition was fewer than two open
/// writers covering the file. Measured at that point: zero. So it came out, and which reading of
/// the original report was correct is still open. `P152` records the removal.
///
/// A caution for whoever meets the next report on this path, because the green that followed is
/// weaker than it looks: with fewer than two open record writers the census below cannot be
/// stated at all and returns empty, so it passes identically whether a coupling is gone or
/// merely unobservable. The facade is still a hand-maintained list of check names, which is the
/// shape `OD-LEDGER-007` defers the structural remedy for, so the forcing condition has not been
/// removed -- only the population that made it visible.
const KNOWN_SERIALIZERS: &[(&str, &str)] = &[];

/// How many open record writers reserving one path makes it a serializer.
///
/// Below this the path is ordinary contention between two items that happen to want the
/// same ground, which resolves when one of them finishes. At this many it is a rule, and the
/// next record writer will reserve it too.
const WRITERS_MAKING_A_SERIALIZER: usize = 2;

/// How many writers the search can be stated over.
///
/// "Reserved by *all* of them" is a claim about a population, and one item is not one: with a
/// single writer every path it happens to reserve reads as universal.
const WRITERS_MAKING_A_POPULATION: usize = 2;

/// The paths the register declares, without what forces each of them.
fn Declared() -> Vec<&'static str>
{
    return KNOWN_SERIALIZERS.iter().map(|(path, _)| return *path).collect();
}

/// An item's territory with one more path on it.
fn Territory_Widened_By_Path(item: &LedgerItem, path: &str) -> Territory
{
    let mut paths = item.territory.paths.clone();
    paths.push(path.to_owned());

    return Territory::Of_Files(paths);
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
/// something a rule forces, and it does not resolve when one of them finishes: the rule that
/// forced it forces it again on the next writer that rule reaches. That is a structural
/// serializer. The first two arrived without anybody noticing, which is why this assertion
/// exists; the one the register holds today is the first this assertion found for itself, and
/// "does not resolve" was measured of it rather than assumed — a writer landing and the
/// widest territory set aside both left it standing. How far its rule reaches, which is
/// narrower than every future writer, is in the register's own comment beside the entry.
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
    let writers = Record_Writers(&document);
    Report_Whether_The_Search_Could_Be_Stated(writers.len());
    let undeclared = Undeclared_Serializers(&document);

    assert!(
        undeclared.is_empty(),
        "every open record writer reserves these, and none is in KNOWN_SERIALIZERS: \
         {undeclared:?}.\n\
         A third thing every record writer has to touch is a third reason the board runs \
         one item at a time. Add it with what forces it, or remove the coupling."
    );
}

/// Prints whether the census above asked its question or could not.
///
/// A pass here means one of two things and they are not the same: no open writer reserves an
/// undeclared path, or there were too few writers for "reserved by all of them" to mean
/// anything, in which case [`Undeclared_Serializers`] returns empty without comparing anything.
/// The second reads exactly like the first. So the scope of the green prints beside it, the way
/// [`Report_The_Board`] prints the figure it refuses to assert, and for the same reason
/// `OD-LEDGER-032` gives: the line's absence would itself be visible.
///
/// Deliberately not an assertion. Failing because the board went quiet would redden every
/// commit made on a board at rest, for a reason no commit contained, which is the defect that
/// record exists to record.
fn Report_Whether_The_Search_Could_Be_Stated(writers: usize)
{
    if writers < WRITERS_MAKING_A_POPULATION
    {
        eprintln!(
            "universal-reservation census: {writers} open record writer(s), below the {} this \
             search can be stated over, so nothing was compared and this pass is about the \
             population rather than about any coupling.",
            WRITERS_MAKING_A_POPULATION
        );
        return;
    }

    eprintln!("universal-reservation census: compared {writers} open record writers.");
}

/// The other direction: a declared serializer that no longer serializes anything.
///
/// A register that over-reports is as useless as one that under-reports. If seeding stops
/// forcing a shared edit — the remedy `OD-LEDGER-007` defers — this fails, and the entry
/// comes out in the commit that earned it rather than surviving as an explanation for a
/// coupling nobody has any more.
///
/// Counted with [`Is_Covering`] and not with [`Is_Colliding`]. See that function for why the
/// symmetric reading made the register unemptiable, which is the defect `OD-LEDGER-011`
/// found while closing.
#[test]
fn Test_Every_Declared_Serializer_Should_Still_Serialize()
{
    let document = Unclaimed_Copy();
    let writers = Record_Writers(&document);

    let stale = Stale_Declarations(&Declared(), &writers);

    assert!(
        stale.is_empty(),
        "these are declared as serializing the board and fewer than two open record writers \
         reserve them: {stale:?}.\n\
         Either the coupling is gone and the entry should be too, or the board no longer \
         has the items that made it visible."
    );
}

/// The declared entries too few open writers still cover to be serializing.
///
/// Shared by the assertion and its control, so the control exercises the filter the assertion
/// uses rather than a second copy of it -- the same reason [`Undeclared_Serializers`] is shared
/// with its own control.
fn Stale_Declarations<'a>(declared: &[&'a str], writers: &[&LedgerItem]) -> Vec<&'a str>
{
    return declared
        .iter()
        .copied()
        .filter(|entry| return Writers_Reserving_Path(writers, entry) < WRITERS_MAKING_A_SERIALIZER)
        .collect();
}

/// The control that keeps the assertion above from passing on an empty register.
///
/// [`KNOWN_SERIALIZERS`] is empty, so that assertion now iterates nothing and would pass whether
/// [`Stale_Declarations`] works or not. Every other guard in this file has at some point passed
/// when it should not have, which is why an empty population gets a control rather than a note.
///
/// The subject is constructed rather than borrowed from the live board, for the reason
/// [`Test_An_Undeclared_Serializer_Should_Be_Found`]'s doc gives: a control that depends on what
/// `Unclaimed_Copy` happens to hold proves the filter has teeth only while somebody is mid-work,
/// and proves nothing on a board at rest. `OD-LEDGER-032`.
#[test]
fn Test_A_Declared_Serializer_Nobody_Reserves_Should_Be_Reported()
{
    let document = Constructed_Writers(CONTESTING_WRITERS);
    let writers = Record_Writers(&document);
    let abandoned = "crates/invented/declared-but-nobody-reserves-it";

    let stale = Stale_Declarations(&[abandoned], &writers);

    assert!(
        stale.contains(&abandoned),
        "a declared serializer that no open writer reserves was not reported as stale: \
         {stale:?}. The register's other direction cannot report an entry whose coupling is \
         gone, so an expired entry would survive as an explanation for nothing."
    );
}

/// How many of these items reserve a path coarsely enough to still be serializing on it.
fn Writers_Reserving_Path(writers: &[&LedgerItem], declared: &str) -> usize
{
    return writers
        .iter()
        .filter(|item| {
            return item
                .territory
                .paths
                .iter()
                .any(|path| return Is_Covering(ReservedPath(path.as_str()), DeclaredPath(declared)));
        })
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
    let mut document = Constructed_Writers(CONTESTING_WRITERS);
    let writers = Writer_Ids(&document);
    let invented = "crates/invented/shared-by-everyone";
    for item in &mut document.items
    {
        if writers.contains(&item.id)
        {
            item.territory = Territory_Widened_By_Path(item, invented);
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
        .filter(|candidate| return Is_Serialized(candidate, &writers, &declared))
        .map(|candidate| return Normalize_Path(candidate))
        .collect();
}

/// The writer candidates are drawn from, if there are at least two writers to compare —
/// below that, "reserved by all of them" is not a claim a population of one can make, per
/// [`Undeclared_Serializers`]'s own doc comment.
fn First_Writer_If_Comparable<'a>(writers: &[&'a LedgerItem]) -> Option<&'a LedgerItem>
{
    if writers.len() < WRITERS_MAKING_A_POPULATION
    {
        return None;
    }

    return writers.first().copied();
}

/// Whether every record writer reserves this path, and nobody declared it.
fn Is_Serialized(candidate: &str, writers: &[&LedgerItem], declared: &[&str]) -> bool
{
    if Normalize_Path(candidate).starts_with(RECORD_DIRECTORY)
    {
        return false;
    }
    if declared
        .iter()
        .any(|known| return Is_Colliding(PathText(known), PathText(candidate)))
    {
        return false;
    }

    return writers.iter().all(|item| {
        return item
            .territory
            .paths
            .iter()
            .any(|path| return Is_Colliding(PathText(candidate), PathText(path)));
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

    Report_The_Board(&writers, &counted, &document);
    Assert_The_Counts_Agree(&counted);
}

/// Prints the figure, in both directions, so a reader sees what the board looked like.
fn Report_The_Board(writers: &[&LedgerItem], counted: &Exclusions, document: &LedgerDocument)
{
    eprintln!(
        "record writers: {} items, {} pair(s), {} blocked, {} of those only by a declared \
         serializer. Concurrent pair available: {}.\n\
         The {} are OD-LEDGER-007's debt; the other {} are ordinary contention.",
        writers.len(),
        counted.pairs,
        counted.blocked,
        counted.structural,
        Parallel_Pair_Named(document),
        counted.structural,
        counted.blocked.saturating_sub(counted.structural)
    );
}

/// The concurrent pair the board currently offers, named for the report, or `none`.
fn Parallel_Pair_Named(document: &LedgerDocument) -> String
{
    return A_Concurrent_Pair(document)
        .map_or_else(|| return "none".to_owned(), |(first, second)| {
            return format!("{first} + {second}");
        });
}

/// The arithmetic relationship `Count_Exclusion` promises whatever happens to be on the board.
///
/// The figure `Report_The_Board` prints is deliberately not asserted against a threshold --
/// `OD-LEDGER-032` is the record of what demanding one cost (red on 26 of the last 30
/// commits, for a reason no commit contained), and reasserting it here would be the same
/// defect in miniature. What *is* asserted is the counting's own consistency: a pair the
/// register alone excludes is still an excluded pair, and an excluded pair is still one of
/// the pairs counted. Either inequality breaking would mean the counting itself regressed,
/// which the report would not by itself reveal -- it would simply print a different number
/// and look no less legitimate for it.
fn Assert_The_Counts_Agree(counted: &Exclusions)
{
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
