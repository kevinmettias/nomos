//! Reserving a record identifier, and the two ways it can already be spoken for.
//!
//! An identifier already published and one another open item reserves are different
//! refusals, because they are fixed differently: the first is an amendment, the second is a
//! wait. Collapsing them would tell an author to wait for an item that will never release
//! what it is holding.

use crate::board::*;

/// An item the board cannot hold is the caller's to correct, not a broken store.
///
/// The two are different exit codes and opposite next actions — fix your item, or stop and
/// fetch a person — so routing `add` through the store had to keep them apart. Carrying
/// [`LedgerError::Invalid`] out as [`AddRefusal::LedgerUnusable`] with everything else would
/// have turned "your territory is empty" into "the ledger is unusable", which is
/// `OD-LEDGER-009`'s conflation arriving by a new route.
///
/// An empty territory is the instance that actually happens: `AGENTS.md` states it as a rule
/// of the board, so it is the refusal an author hits by writing a plausible item.
#[test]
fn Test_An_Item_That_Would_Not_Validate_Should_Refuse_As_The_Authors_Mistake()
{
    let (_directory, mut ledger) = Board_At("add-would-not-validate", Vec::new());

    let item = Item("T-1", &[]);
    let refused =
        ledger.Add(&item, "agent-a", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    assert!(
        matches!(refused, Err(AddRefusal::WouldBeInvalid { .. })),
        "an item that reserves nothing is a violation of the board's own rules, and \
         reporting it as an unusable store sends its author to the wrong remedy: {refused:?}"
    );
    assert!(
        ledger
            .Load()
            .expect("readable")
            .items
            .is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

// ---------------------------------------------------------------------------
// A record identifier is allocated once, and `add` is where that is enforced.
// ---------------------------------------------------------------------------

/// An item reserving `docs/records/<ID>` alongside whatever else it touches.
fn Reserving_Record(id: &str, identifier: &str) -> LedgerItem
{
    return Item(id, &["crates/a/src/lib.rs", identifier]);
}

/// A published record, spelled as the file it actually is rather than as its identifier.
fn Published(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().map(|file| return (*file).to_owned()));
}

/// A published identifier reserved without declaring an amendment is refused, by its file.
///
/// The half nothing could have caught. A published identifier excludes nobody, so
/// `Test_A_Record_Should_Exclude_Nobody_But_Its_Own_Writer` — which reddens on two *open*
/// items sharing one — is blind to it by construction. What happened instead is that the
/// author found the identifier taken mid-claim, with no `work edit` to move it.
///
/// What this asserts narrowed when the declaration arrived, and the assertion did not have to
/// change: the item here declares nothing, so it is allocating, and an allocation onto a spent
/// number is still the defect this was written for. The case it no longer covers is the one
/// directly below.
#[test]
fn Test_A_Record_Identifier_Already_Published_Should_Be_Refused_By_Its_File()
{
    let (_directory, mut ledger) = Board_At("add-record-published", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &ItemTerritory::Empty(),
    );

    let Err(AddRefusal::RecordPublished { identifier, file }) = refused
    else
    {
        panic!("a spent identifier must be refused as spent: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-006");
    assert_eq!(file, "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md");
    assert!(
        ledger.Load().expect("readable").items.is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

/// The negative control. An unspent identifier is still accepted, beside published ones.
///
/// Without it every assertion above is satisfied by an `add` that refuses everything, and
/// the guard would be indistinguishable from a broken one on the day it mattered.
#[test]
fn Test_An_Unspent_Record_Identifier_Should_Still_Be_Accepted()
{
    let (_directory, mut ledger) = Board_At("add-record-unspent", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-007");
    let added = ledger.Add(
        &item,
        "agent-a",
        &Published(&[
            "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md",
            // The ordinal is compared as a whole component, so this must not make `007`
            // look taken. `0071` is not `007`, and a prefix rule would say it is.
            "docs/records/OD-LEDGER-0071-something-else.md",
        ]),
        &ItemTerritory::Empty(),
    );

    assert_eq!(added, Ok(()), "the next free identifier is free");
    assert_eq!(ledger.Load().expect("readable").items.len(), 1);
}

/// What the item says it is editing rather than allocating.
///
/// The same shape as [`Published`] and deliberately a different name at the call site: the
/// two arguments are both record files and mean opposite things, and a reader who sees
/// `Published` twice has to work out which one is the claim and which is the repository.
fn Amending(files: &[&str]) -> ItemTerritory
{
    return ItemTerritory::Of_Files(files.iter().map(|file| return (*file).to_owned()));
}

/// A published record reserved for amendment is accepted, in either spelling.
///
/// The clause this item exists for. `ARC-ECOSYSTEM-001` is at version 2 and
/// `OD-CAPABILITY-001` at version 2, so amending a record is ordinary work here, and an
/// amendment must reserve the file it edits because territory is the only thing keeping two
/// writers off one file. Both spellings are driven, because `OD-LEDGER-016` makes the
/// identifier and its file one subject and an author who had to guess which one `--amends`
/// wanted would be following a convention rather than a rule.
#[test]
fn Test_A_Published_Record_Declared_As_An_Amendment_Should_Be_Accepted()
{
    const FILE: &str = "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md";

    for (described, spelled) in [
        ("the bare identifier", "docs/records/OD-LEDGER-006"),
        ("the published filename", FILE),
    ]
    {
        let (_directory, mut ledger) = Board_At("add-record-amended", Vec::new());
        let item = Reserving_Record("T-1", spelled);

        assert_eq!(
            ledger.Add(&item, "agent-a", &Published(&[FILE]), &Amending(&[spelled])),
            Ok(()),
            "an amendment declared by {described} was refused as an allocation"
        );
        assert_eq!(ledger.Load().expect("readable").items.len(), 1);
    }
}

/// Declaring an amendment does not exempt the record from the open-item comparison.
///
/// The half a reader is most likely to expect the other way round. Two items amending one
/// record are two writers on one file, which is exactly what territory serializes, so the
/// declaration answers *which act this is* and never *whether somebody else is already doing
/// it*. Asserted separately from the acceptance above rather than folded into it, because one
/// assertion covering both is satisfied by an `add` that ignores the declaration entirely.
#[test]
fn Test_An_Amendment_Should_Not_Exempt_A_Record_Another_Open_Item_Reserves()
{
    let (_directory, mut ledger) = Board_At("add-record-amend-contended", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-006",
    )]);

    let item = Reserving_Record("T-2", "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-b",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Amending(&["docs/records/OD-LEDGER-006"]),
    );

    let Err(AddRefusal::RecordReserved { identifier, item }) = refused
    else
    {
        panic!("a second amender must still be refused by the first: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-006");
    assert_eq!(item, ItemId::New("T-1"));
}

/// An amendment declared against a record nobody has published is refused.
///
/// Without this the declaration would be the cheapest way to defeat the guard the whole
/// check exists to be: declare every reservation an amendment and no identifier is ever
/// spent again. It is also the ordinary mistake — an author who mistyped the number is told
/// so here rather than getting an item that allocates while saying it amends.
#[test]
fn Test_A_Declared_Amendment_Of_An_Unpublished_Record_Should_Be_Refused()
{
    let (_directory, mut ledger) = Board_At("add-record-amend-absent", Vec::new());

    let item = Reserving_Record("T-1", "docs/records/OD-LEDGER-099");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Amending(&["docs/records/OD-LEDGER-099"]),
    );

    let Err(AddRefusal::AmendmentNotPublished { identifier }) = refused
    else
    {
        panic!("an amendment of nothing must be refused as such: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-099");
    assert!(
        ledger.Load().expect("readable").items.is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

/// A record identifier another open item reserves is refused, and that item is named.
///
/// Five collisions bought this. Three open items reserved `OD-LEDGER-020` and two reserved
/// `OD-LEDGER-021`, each authored by a session taking the next free number; clearing them
/// meant declining and re-authoring four items, because there is no `work edit`.
#[test]
fn Test_A_Record_Identifier_Another_Open_Item_Reserves_Should_Be_Refused_By_Its_Item()
{
    let (_directory, mut ledger) = Board_At("add-record-reserved", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-020",
    )]);

    // The second author writes the identifier's file spelling rather than its bare form.
    // `OD-LEDGER-016` makes those one subject, so this must still be refused — an author
    // who reserved the file they were about to write has taken the identifier.
    let item = Reserving_Record("T-2", "docs/records/OD-LEDGER-020-the-same-number.md");
    let refused =
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    let Err(AddRefusal::RecordReserved { identifier, item }) = refused
    else
    {
        panic!("an identifier another open item holds must be refused as held: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-020");
    assert_eq!(item, ItemId::New("T-1"));
}

/// The two refusals are different values, because the remedies are different.
///
/// Choosing another identifier fixes one; the other may resolve itself when the item
/// holding it is retired. An author told only "taken" picks the wrong remedy half the time,
/// which is the mis-subject `OD-LEDGER-014` measured one verb over.
#[test]
fn Test_A_Published_Identifier_And_A_Reserved_One_Should_Be_Different_Refusals()
{
    let (_directory, mut ledger) = Board_At("add-record-distinct", vec![Reserving_Record(
        "T-1",
        "docs/records/OD-LEDGER-020",
    )]);

    let holding = Reserving_Record("T-2", "docs/records/OD-LEDGER-020");
    let publishing = Reserving_Record("T-3", "docs/records/OD-LEDGER-006");
    let reserved =
        ledger.Add(&holding, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());
    let published = ledger.Add(
        &publishing,
        "agent-b",
        &Published(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &ItemTerritory::Empty(),
    );

    assert_ne!(reserved, published);
    assert_ne!(
        reserved.as_ref().err().map(AddRefusal::Describe),
        published.as_ref().err().map(AddRefusal::Describe),
        "two refusals with one sentence send both authors to one remedy"
    );
}

/// A closed item's territory is history, not a reservation.
///
/// The rule that keeps this guard from refusing the whole board: almost every item ever
/// finished reserved a record, so counting closed items would make every allocated number a
/// permanent claim and the next author could allocate nothing at all.
#[test]
fn Test_A_Closed_Items_Record_Reservation_Should_Not_Reserve_Anything()
{
    let directory = Temp_Dir("add-record-closed");
    let mut ledger = Ledger_At(&directory, &AT_NOW);

    let closed_states = [
        ItemState::Done,
        ItemState::Declined {
            reason: "it turned out not to be work".to_owned(),
        },
    ];

    for state in closed_states
    {
        Allocates_Over(&mut ledger, state);
    }
}

/// The board holds one closed item reserving a record, and the next author allocates it.
fn Allocates_Over<Clock: nomos_platform::Clock>(
    ledger: &mut FileLedger<StdFileSystem, Clock, FileLock>,
    state: ItemState,
)
{
    const RECORD: &str = "docs/records/OD-LEDGER-020";

    let described = format!("{state:?}");
    ledger
        .Save(&Document(vec![Closed_Reserving(RECORD, state)]))
        .expect("valid");

    let item = Reserving_Record("T-2", RECORD);
    assert_eq!(
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty()),
        Ok(()),
        "a {described} item's reservation outlived it, so the number is claimed forever"
    );
}

/// An item in a closed state, carrying whatever that state's own invariants require.
///
/// A `Done` item with no verification and a `Declined` one with no declination are both
/// refused by the board before this test's subject is ever reached, so each arm has to be
/// completed here — but the completing is scaffolding, not what the test is about.
fn Closed_Reserving(record: &str, state: ItemState) -> LedgerItem
{
    let mut closed = Reserving_Record("T-1", record);
    if matches!(state, ItemState::Declined { .. })
    {
        closed.declined = Some(Declination {
            holder: "agent-a".to_owned(),
            declined_at: At(NOW),
        });
    }
    else
    {
        closed.verified = Some(VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: "test result: ok".to_owned(),
            verified_at: At(NOW),
            gate: None,
        });
    }
    closed.state = state;

    return closed;
}

/// Ordinary overlapping territory is still accepted, and that is deliberate.
///
/// `add` does not refuse shared territory in general and must not start: items overlap
/// constantly and claims are what serialize them. `P10-ADD-PROMISE` narrowed the doc comment
/// that once promised otherwise. What is guarded is only the reservation an author cannot
/// recover from mid-claim.
#[test]
fn Test_Ordinary_Shared_Territory_Should_Still_Be_Accepted()
{
    let (_directory, mut ledger) = Board_At("add-shared-territory", vec![Item("T-1", &["crates/a/src/lib.rs"])]);

    let item = Item("T-2", &["crates/a/src/lib.rs"]);
    let added =
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    assert_eq!(
        added,
        Ok(()),
        "two items may reserve one path; a claim is what decides who holds it"
    );
}
