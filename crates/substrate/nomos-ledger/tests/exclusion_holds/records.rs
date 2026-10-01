//! Reserving a record identifier, and the two ways it can already be spoken for.
//!
//! An identifier already published and one another open item reserves are different
//! refusals, because they are fixed differently: the first is an amendment, the second is a
//! wait. Collapsing them would tell an author to wait for an item that will never release
//! what it is holding.

use crate::board::{
    AddRefusal, Timestamp_From_Seconds, AT_NOW, Board, Board_At, Declination, Document_Holding_Items, FileLedger,
    FileLock, FixedClock, Item_Reserving_Files, ItemId, ItemState, ItemTerritory, Ledger_At, LedgerItem, NOW,
    StdFileSystem, Temporary_Directory, VerificationRecord,
};

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
    let Board { directory: _directory, mut ledger } = Board_At("add-would-not-validate", Vec::new());

    let item = Item_Reserving_Files("T-1", &[]);
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

/// A record that has been published, spelled as the file it actually is.
const PUBLISHED_RECORD_FILE: &str = "docs/records/OD-LEDGER-006-a-reason-does-not-survive.md";

/// A record identifier some other item is holding, spelled bare.
const RESERVED_RECORD: &str = "docs/records/OD-LEDGER-020";

/// An item reserving `docs/records/<ID>` alongside whatever else it touches.
///
/// The identifier is typed rather than a second string, so a call site cannot hand the two
/// positions to each other and get an item that reads correctly while reserving something else.
fn Reserving_Record(id: &ItemId, identifier: &str) -> LedgerItem
{
    return Item_Reserving_Files(id.As_Text(), &["crates/a/src/lib.rs", identifier]);
}

/// A published record, spelled as the file it actually is rather than as its identifier.
fn Territory_Of_Published_Records(files: &[&str]) -> ItemTerritory
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
    let Board { directory: _directory, mut ledger } = Board_At("add-record-published", Vec::new());

    let item = Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Territory_Of_Published_Records(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
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
    let Board { directory: _directory, mut ledger } = Board_At("add-record-unspent", Vec::new());

    let item = Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-007");
    let added = ledger.Add(
        &item,
        "agent-a",
        &Territory_Of_Published_Records(&[
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
fn Territory_Of_Amended_Records(files: &[&str]) -> ItemTerritory
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
/// The two spellings `--amends` accepts for the same published record: the bare identifier,
/// and the filename it was actually published under.
///
/// A named provider rather than an inline literal, so a third accepted spelling is a value
/// added here rather than a change to the loop that reads them.
const SPELLINGS_OF_ONE_RECORD: usize = 2;

fn Spellings_Of_The_Published_Record() -> [(&'static str, &'static str); SPELLINGS_OF_ONE_RECORD]
{
    return [
        ("the bare identifier", "docs/records/OD-LEDGER-006"),
        ("the published filename", PUBLISHED_RECORD_FILE),
    ];
}

#[test]
fn Test_A_Published_Record_Declared_As_An_Amendment_Should_Be_Accepted()
{
    for (described, spelled) in Spellings_Of_The_Published_Record()
    {
        let Board { directory: _directory, mut ledger } = Board_At("add-record-amended", Vec::new());
        let item = Reserving_Record(&ItemId::New("T-1"), spelled);

        assert_eq!(
            ledger.Add(
                &item,
                "agent-a",
                &Territory_Of_Published_Records(&[PUBLISHED_RECORD_FILE]),
                &Territory_Of_Amended_Records(&[spelled])
            ),
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
    let Board { directory: _directory, mut ledger } = Board_At("add-record-amend-contended", vec![
        Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-006"),
    ]);

    let item = Reserving_Record(&ItemId::New("T-2"), "docs/records/OD-LEDGER-006");
    let refused = ledger.Add(
        &item,
        "agent-b",
        &Territory_Of_Published_Records(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Territory_Of_Amended_Records(&["docs/records/OD-LEDGER-006"]),
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
    let Board { directory: _directory, mut ledger } = Board_At("add-record-amend-absent", Vec::new());

    let item = Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-099");
    let refused = ledger.Add(
        &item,
        "agent-a",
        &Territory_Of_Published_Records(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Territory_Of_Amended_Records(&["docs/records/OD-LEDGER-099"]),
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

/// A declared amendment that names the right record by the wrong filename is refused, and
/// the refusal carries the right one.
///
/// The reservation this guards is correct without it, which is the whole difficulty:
/// `OD-LEDGER-016`'s fold makes every slug for one identifier the same subject, so the
/// mistyped declaration excludes exactly the writers the correct one would have and nothing
/// else has any reason to object. What survives the refusal's absence is an item carrying a
/// path that opens nothing, asserted by its own success line as the file it reserved. Both
/// halves are checked here: that it refuses, and that its text hands back the spelling,
/// since a refusal that only said "wrong" would leave the author doing the lookup that
/// produced the mistake.
#[test]
fn Test_A_Declared_Amendment_Spelling_The_Wrong_Filename_Should_Be_Refused_With_The_Right_One()
{
    let Board { directory: _directory, mut ledger } = Board_At("add-record-amend-misspelled", Vec::new());

    let refused = Amend_With_A_Slug_Nobody_Published(&mut ledger, PUBLISHED_RECORD_FILE);

    Hands_Back_The_Spelling_It_Refused(refused, PUBLISHED_RECORD_FILE);
    assert!(
        ledger.Load().expect("readable").items.is_empty(),
        "the refusal was reported and the item landed anyway"
    );
}

/// The declaration itself: `--amends` naming the right record by a filename nobody published.
fn Amend_With_A_Slug_Nobody_Published(
    ledger: &mut FileLedger<StdFileSystem, &'static FixedClock, FileLock>,
    published: &str,
) -> AddRefusal
{
    let item = Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-006");

    return ledger
        .Add(
            &item,
            "agent-a",
            &Territory_Of_Published_Records(&[published]),
            &Territory_Of_Amended_Records(&["docs/records/OD-LEDGER-006-a-slug-nobody-published.md"]),
        )
        .expect_err("the misspelled amendment must not be accepted as an allocation");
}

/// Both halves of what the refusal owes: that it refuses, and that its text hands the author
/// back the spelling, since a refusal that only said "wrong" would leave them doing the lookup
/// that produced the mistake.
fn Hands_Back_The_Spelling_It_Refused(refused: AddRefusal, published: &str)
{
    let AddRefusal::AmendmentMisspelled { identifier, declared, file } = refused
    else
    {
        panic!("a misspelled amendment must be refused as such: {refused:?}");
    };
    assert_eq!(identifier, "docs/records/od-ledger-006");
    assert_eq!(declared, "docs/records/OD-LEDGER-006-a-slug-nobody-published.md");
    assert_eq!(file, published);

    let described = AddRefusal::AmendmentMisspelled { identifier, declared, file }.Describe();
    assert!(
        described.contains(published),
        "the refusal must hand back the spelling, not only withhold it: {described}"
    );
}

/// The bare identifier spelling stays accepted, because it never claimed a filename.
///
/// The other half of the rule above, and the reason that one judges the `.md` suffix rather
/// than judging every declaration against the published filename. `Refuse_A_Spent_Record`'s
/// own doc states that a declaration may be spelled either way, so a check that refused the
/// bare form would withdraw a spelling this crate documents while fixing a different
/// mistake.
#[test]
fn Test_A_Declared_Amendment_Spelled_As_A_Bare_Identifier_Should_Still_Be_Accepted()
{
    let Board { directory: _directory, mut ledger } = Board_At("add-record-amend-bare", Vec::new());

    let item = Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-006");
    let added = ledger.Add(
        &item,
        "agent-a",
        &Territory_Of_Published_Records(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
        &Territory_Of_Amended_Records(&["docs/records/OD-LEDGER-006"]),
    );

    assert!(added.is_ok(), "the bare spelling is one of the two the fold admits: {added:?}");
    assert_eq!(ledger.Load().expect("readable").items.len(), 1, "the item must land");
}

/// A record identifier another open item reserves is refused, and that item is named.
///
/// Five collisions bought this. Three open items reserved `OD-LEDGER-020` and two reserved
/// `OD-LEDGER-021`, each authored by a session taking the next free number; clearing them
/// meant declining and re-authoring four items, because there is no `work edit`.
#[test]
fn Test_A_Record_Identifier_Another_Open_Item_Reserves_Should_Be_Refused_By_Its_Item()
{
    let Board { directory: _directory, mut ledger } = Board_At("add-record-reserved", vec![
        Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-020"),
    ]);

    // The second author writes the identifier's file spelling rather than its bare form.
    // `OD-LEDGER-016` makes those one subject, so this must still be refused — an author
    // who reserved the file they were about to write has taken the identifier.
    let item = Reserving_Record(&ItemId::New("T-2"), "docs/records/OD-LEDGER-020-the-same-number.md");
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
    let Board { directory: _directory, mut ledger } = Board_At("add-record-distinct", vec![
        Reserving_Record(&ItemId::New("T-1"), "docs/records/OD-LEDGER-020"),
    ]);

    let holding = Reserving_Record(&ItemId::New("T-2"), "docs/records/OD-LEDGER-020");
    let publishing = Reserving_Record(&ItemId::New("T-3"), "docs/records/OD-LEDGER-006");
    let reserved =
        ledger.Add(&holding, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());
    let published = ledger.Add(
        &publishing,
        "agent-b",
        &Territory_Of_Published_Records(&["docs/records/OD-LEDGER-006-a-reason-does-not-survive.md"]),
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
/// The two states under which a record reservation is history rather than a hold: finished
/// with a passing verification, or declined with a reason.
///
/// A named provider rather than an inline literal, so a third closed state — should one ever
/// exist — is a value added here rather than a change to the loop that reads them.
const CLOSED_STATES_UNDER_TEST: usize = 2;

fn Closed_States() -> [ItemState; CLOSED_STATES_UNDER_TEST]
{
    return [
        ItemState::Done,
        ItemState::Declined {
            reason: "it turned out not to be work".to_owned(),
        },
    ];
}

#[test]
fn Test_A_Closed_Items_Record_Reservation_Should_Not_Reserve_Anything()
{
    let directory = Temporary_Directory("add-record-closed");
    let mut ledger = Ledger_At(directory.As_Path(), &AT_NOW);

    // The board holds one closed item reserving a record, and the next author allocates it.
    for state in Closed_States()
    {
        let described = format!("{state:?}");
        ledger
            .Save(&Document_Holding_Items(vec![Closed_Reserving(RESERVED_RECORD, state)]))
            .expect("the closed item the fixture built is a valid document");

        let item = Reserving_Record(&ItemId::New("T-2"), RESERVED_RECORD);
        assert_eq!(
            ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty()),
            Ok(()),
            "a {described} item's reservation outlived it, so the number is claimed forever"
        );
    }
}

/// An item in a closed state, carrying whatever that state's own invariants require.
///
/// A `Done` item with no verification and a `Declined` one with no declination are both
/// refused by the board before this test's subject is ever reached, so each arm has to be
/// completed here — but the completing is scaffolding, not what the test is about.
fn Closed_Reserving(record: &str, state: ItemState) -> LedgerItem
{
    let mut closed = Reserving_Record(&ItemId::New("T-1"), record);
    if matches!(state, ItemState::Declined { .. })
    {
        closed.declined = Some(Declination {
            holder: "agent-a".to_owned(),
            declined_at: Timestamp_From_Seconds(NOW),
        });
    }
    else
    {
        closed.verified = Some(VerificationRecord {
            argv: vec!["cargo".to_owned(), "test".to_owned()],
            exit_code: 0,
            output_tail: "test result: ok".to_owned(),
            verified_at: Timestamp_From_Seconds(NOW),
            gate: None,
            revision: None,
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
    let Board { directory: _directory, mut ledger } =
        Board_At("add-shared-territory", vec![Item_Reserving_Files("T-1", &["crates/a/src/lib.rs"])]);

    let item = Item_Reserving_Files("T-2", &["crates/a/src/lib.rs"]);
    let added =
        ledger.Add(&item, "agent-b", &ItemTerritory::Empty(), &ItemTerritory::Empty());

    assert_eq!(
        added,
        Ok(()),
        "two items may reserve one path; a claim is what decides who holds it"
    );
}
