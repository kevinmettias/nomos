use nomos_contracts::{
    BuildVariantId, ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId,
};
use nomos_store::{
    Authority, Commit, DocumentKind, DocumentStore, Recorded, StoreError, COMMIT_SCHEMA,
};

/// The seeds the fixture identities are built from, one per identity component. Distinct
/// seeds are what keep two identifiers from colliding by construction.
const BUILD_VARIANT_SEED: u8 = 2;
const CONFIGURATION_SEED: u8 = 3;

/// A second workspace state, distinct from the initial-state seed the other fixtures use.
const SECOND_STATE_SEED: u8 = 4;

/// What `Taken` records: two syntax facts under one schema, and one finding.
const FACTS_IN_ONE_SNAPSHOT: usize = 2;

/// Two commits of three records each. Four documents is the fewest that make "no document
/// in the store is unaccounted for" a claim with content behind it.
const DOCUMENTS_IN_TWO_SNAPSHOTS: usize = 4;

/// The commits `Test_Two_Commits_Under_One_Workspace_State_Should_Both_Be_Reachable`
/// writes against one state. That the count exceeds one is the whole point of the test.
const COMMITS_UNDER_ONE_STATE: usize = 2;

fn Digest(seed: u8) -> Digest128
{
    return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
}

fn Fact(payload: &str) -> Recorded
{
    return Recorded::New(
        DocumentKind::Fact,
        SchemaId::New("nomos.syntax.v1"),
        payload.as_bytes().to_vec(),
    );
}

fn Finding(payload: &str) -> Recorded
{
    return Recorded::New(
        DocumentKind::Finding,
        SchemaId::New("nomos.finding.v1"),
        payload.as_bytes().to_vec(),
    );
}

fn Taken(seed: u8) -> Commit
{
    return Commit::Under(
        SnapshotId::From_Digest(Digest(seed)),
        BuildVariantId::From_Digest(Digest(BUILD_VARIANT_SEED)),
        ConfigurationId::From_Digest(Digest(CONFIGURATION_SEED)),
        GenerationId::INITIAL,
    )
    .Recording(Fact("fn main() {}"))
    .Recording(Fact("struct Workspace;"))
    .Recording(Finding("unused import at nomos.rs"));
}

fn Observed() -> DocumentStore
{
    return DocumentStore::For(Authority::Observed);
}

#[test]
fn Test_A_Snapshot_Written_And_Reread_Should_Be_Byte_Identical()
{
    let snapshot = Taken(1);
    let mut store = Observed();

    let id = store.Commit(&snapshot).expect("Authority::Observed admits every kind recorded here");
    let written = store.Read(id).expect("this store holds what it committed").bytes.clone();

    assert_eq!(
        written,
        snapshot.Encode().expect("the store committed this snapshot already"),
        "the snapshot the store holds is not the snapshot that was committed"
    );
    assert!(!written.is_empty());
}

#[test]
fn Test_A_Reread_Snapshot_Should_Decode_To_What_Was_Committed()
{
    let snapshot = Taken(1);
    let mut store = Observed();
    let id = store.Commit(&snapshot).expect("Authority::Observed admits every kind recorded here");

    let held = store.Read(id).expect("this store holds what it committed");
    let manifest = Commit::Decode(&held.bytes).expect("the store accepted these bytes");

    assert_eq!(manifest.schema, COMMIT_SCHEMA);
    assert_eq!(manifest.snapshot, snapshot.snapshot);
    assert_eq!(manifest.generation, snapshot.generation);
    assert_eq!(manifest.records.len(), snapshot.records.len());
    for (reference, record) in manifest.records.iter().zip(&snapshot.records)
    {
        assert_eq!(reference.document, record.Document().Id());
        assert_eq!(
            store.Read(reference.document).expect("this store holds what it committed").bytes,
            record.bytes,
            "a recorded document is not the bytes the snapshot named"
        );
    }
}

#[test]
fn Test_Committing_The_Same_Snapshot_Twice_Should_Address_The_Same_Document()
{
    let mut store = Observed();

    let first = store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");
    let documents = store.Length();
    let second = store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");

    assert_eq!(first, second, "content addressing produced two addresses for one content");
    assert_eq!(store.Length(), documents, "a re-commit duplicated every document");
}

#[test]
fn Test_A_Changed_Record_Should_Address_A_Different_Snapshot()
{
    let mut store = Observed();
    let edited = Commit::Under(
        SnapshotId::From_Digest(Digest(1)),
        BuildVariantId::From_Digest(Digest(BUILD_VARIANT_SEED)),
        ConfigurationId::From_Digest(Digest(CONFIGURATION_SEED)),
        GenerationId::INITIAL,
    )
    .Recording(Fact("fn main() { edited }"));

    let first = store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");
    let second = store.Commit(&edited).expect("Authority::Observed admits every kind recorded here");

    assert_ne!(first, second);
}

#[test]
fn Test_The_Index_Should_Be_Rebuildable_From_The_Documents_Alone()
{
    let mut store = Observed();
    store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");
    store.Commit(&Taken(SECOND_STATE_SEED)).expect("Authority::Observed admits every kind recorded here");

    let before = store.Index().expect("the index derives from these documents").clone();
    store.Drop_Index();
    assert!(!store.Has_Index(), "the index survived being dropped");
    let after = store.Index().expect("the index derives from these documents").clone();

    assert_eq!(before, after, "the index does not rebuild to itself");
    assert_eq!(before.Digest(), after.Digest());
    assert!(!before.Is_Empty(), "an empty index rebuilds to an empty index and proves nothing");
}

#[test]
fn Test_The_Index_Should_Reach_Every_Recorded_Document()
{
    let snapshot = Taken(1);
    let mut store = Observed();
    let id = store.Commit(&snapshot).expect("Authority::Observed admits every kind recorded here");

    let index = store.Index().expect("the index derives from these documents");
    let members = index.In_Snapshot(snapshot.snapshot);

    assert_eq!(index.Commits_Under(snapshot.snapshot), vec![id]);
    assert_eq!(members.len(), snapshot.records.len() + 1, "{members:?}");
    for record in &snapshot.records
    {
        assert!(
            members.contains(&record.Document().Id()),
            "a document the snapshot recorded is not in the snapshot's index"
        );
    }
}

#[test]
fn Test_The_Index_Should_Group_By_Kind_And_Schema()
{
    let mut store = Observed();
    store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");

    let index = store.Index().expect("the index derives from these documents");

    assert_eq!(index.Of_Kind(DocumentKind::Fact).len(), FACTS_IN_ONE_SNAPSHOT);
    assert_eq!(index.Of_Kind(DocumentKind::Finding).len(), 1);
    assert_eq!(index.Of_Kind(DocumentKind::Commit).len(), 1);
    assert_eq!(index.Of_Schema("nomos.syntax.v1").len(), FACTS_IN_ONE_SNAPSHOT);
    assert!(index.Of_Schema("nomos.absent.v1").is_empty());
}

/// A workspace state is a tree. A commit is one write against it. Nothing says there is
/// one of the second per one of the first.
///
/// # Why this test exists
///
/// The index held one `DocumentId` per `SnapshotId` and silently kept the last. Nothing
/// looked wrong, because while a commit manifest was called a snapshot, "the snapshot
/// document for snapshot S" read as a tautology — obviously one per state.
///
/// It is not obvious and it is not true. Analyzing a tree, recording what was found, then
/// analyzing it again for something else produces two commits and no edit between them.
/// Under the old map the first became unreachable through `Snapshots`, and
/// `Test_Every_Document_Should_Be_Reachable_From_A_Snapshot` would have started reporting
/// documents nothing recorded.
#[test]
fn Test_Two_Commits_Under_One_Workspace_State_Should_Both_Be_Reachable()
{
    let state = SnapshotId::From_Digest(Digest(1));
    let first = Under(state, Fact("fn main() {}"));
    let second = Under(state, Finding("unused import at nomos.rs"));
    let mut store = Observed();
    let one = store.Commit(&first).expect("Authority::Observed admits every kind recorded here");
    let other = store.Commit(&second).expect("Authority::Observed admits every kind recorded here");

    assert_ne!(one, other, "two different commits addressed as one document");

    let commits = store.Index().expect("the index derives from these documents").Commits_Under(state);

    assert_eq!(commits.len(), COMMITS_UNDER_ONE_STATE, "one commit here is unreachable: {commits:?}");
    assert!(commits.contains(&one) && commits.contains(&other));
    assert_eq!(
        store.Index().expect("the index derives from these documents").Snapshots(),
        vec![state],
        "and both are under one workspace state, not two"
    );
    assert!(
        store.Unreachable().expect("the index derives from these documents").is_empty(),
        "a commit that fell out of the index takes its records with it"
    );
}

/// One commit under a named workspace state, recording one document.
fn Under(state: SnapshotId, recorded: Recorded) -> Commit
{
    return Commit::Under(
        state,
        BuildVariantId::From_Digest(Digest(BUILD_VARIANT_SEED)),
        ConfigurationId::From_Digest(Digest(CONFIGURATION_SEED)),
        GenerationId::INITIAL,
    )
    .Recording(recorded);
}

#[test]
fn Test_Two_Stores_Given_The_Same_Snapshots_In_Any_Order_Should_Index_Identically()
{
    let mut forwards = Observed();
    forwards.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");
    forwards.Commit(&Taken(SECOND_STATE_SEED)).expect("Authority::Observed admits every kind recorded here");

    let mut backwards = Observed();
    backwards.Commit(&Taken(SECOND_STATE_SEED)).expect("Authority::Observed admits every kind recorded here");
    backwards.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");

    assert_eq!(
        forwards.Index().expect("the index derives from these documents").Digest(),
        backwards.Index().expect("the index derives from these documents").Digest(),
        "the index depends on the order the snapshots arrived in"
    );
}

#[test]
fn Test_Every_Document_Should_Be_Reachable_From_A_Snapshot()
{
    let mut store = Observed();
    store.Commit(&Taken(1)).expect("Authority::Observed admits every kind recorded here");
    store.Commit(&Taken(SECOND_STATE_SEED)).expect("Authority::Observed admits every kind recorded here");

    assert!(
        store.Unreachable().expect("the index derives from these documents").is_empty(),
        "a document is in the store that no snapshot records, so something wrote by another \
         route"
    );
    assert!(
        store.Length() >= DOCUMENTS_IN_TWO_SNAPSHOTS,
        "the store holds too little for that to mean anything"
    );
}

#[test]
fn Test_An_Authored_Document_Should_Not_Enter_An_Observed_Store()
{
    let mut store = Observed();
    let recorded = Recorded::New(
        DocumentKind::Record,
        SchemaId::New("nomos.record.v1"),
        b"D-129".to_vec(),
    );
    let authored = Under(SnapshotId::From_Digest(Digest(1)), recorded);

    let refusal = store.Commit(&authored).expect_err("must refuse");

    assert!(
        matches!(refusal, StoreError::WrongAuthority { .. }),
        "{refusal}"
    );
    assert!(format!("{refusal}").contains("never share a store"), "{refusal}");
    assert_eq!(store.Length(), 0, "the refused snapshot wrote documents anyway");
}

#[test]
fn Test_An_Observed_Document_Should_Not_Enter_An_Authored_Store()
{
    let mut store = DocumentStore::For(Authority::Authored);

    let refusal = store.Commit(&Taken(1)).expect_err("must refuse");

    assert!(matches!(refusal, StoreError::WrongAuthority { .. }), "{refusal}");
}

#[test]
fn Test_Every_Document_Kind_Should_Belong_To_Exactly_One_Authority()
{
    for kind in DocumentKind::All()
    {
        let admitting: Vec<Authority> = [Authority::Observed, Authority::Authored]
            .into_iter()
            .filter(|authority| return authority.Can_Admit(*kind))
            .collect();

        assert_eq!(
            admitting.len(),
            1,
            "{} is admitted by {admitting:?}",
            kind.Label()
        );
    }
}

#[test]
fn Test_A_Snapshot_That_Records_Nothing_Should_Be_Refused()
{
    let mut store = Observed();
    let empty = Commit::Under(
        SnapshotId::From_Digest(Digest(1)),
        BuildVariantId::From_Digest(Digest(BUILD_VARIANT_SEED)),
        ConfigurationId::From_Digest(Digest(CONFIGURATION_SEED)),
        GenerationId::INITIAL,
    );

    let refusal = store.Commit(&empty).expect_err("must refuse");

    assert!(format!("{refusal}").contains("never happened"), "{refusal}");
    assert_eq!(store.Length(), 0);
}

#[test]
fn Test_An_Absent_Document_Should_Be_Refused_Rather_Than_Empty()
{
    let store = Observed();

    let refusal = store
        .Read(Fact("never committed").Document().Id())
        .expect_err("must refuse");

    assert!(matches!(refusal, StoreError::NoSuchDocument { .. }), "{refusal}");
}

#[test]
fn Test_A_Committed_Document_Should_Not_Change_When_Another_Snapshot_Arrives()
{
    let snapshot = Taken(1);
    let mut store = Observed();
    let id = store.Commit(&snapshot).expect("Authority::Observed admits every kind recorded here");
    let before = store.Read(id).expect("this store holds what it committed").clone();

    store.Commit(&Taken(SECOND_STATE_SEED)).expect("Authority::Observed admits every kind recorded here");

    assert_eq!(&before, store.Read(id).expect("this store holds what it committed"));
}

#[test]
fn Test_Decoding_Something_That_Is_Not_A_Snapshot_Should_Be_Refused()
{
    let refusal = Commit::Decode(b"{\"schema\":\"nomos.other.v1\"}").expect_err("must refuse");

    assert!(matches!(refusal, StoreError::Malformed(_)), "{refusal}");
}
