use nomos_contracts::{
    BuildVariantId, ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId,
};
use nomos_store::{
    Authority, Commit, DocumentKind, DocumentStore, Recorded, StoreError, COMMIT_SCHEMA,
};

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
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
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

    let id = store.Commit(&snapshot).expect("commits");
    let written = store.Read(id).expect("reads").bytes.clone();

    assert_eq!(
        written,
        snapshot.Encode().expect("encodes"),
        "the snapshot the store holds is not the snapshot that was committed"
    );
    assert!(!written.is_empty());
}

#[test]
fn Test_A_Reread_Snapshot_Should_Decode_To_What_Was_Committed()
{
    let snapshot = Taken(1);
    let mut store = Observed();
    let id = store.Commit(&snapshot).expect("commits");

    let manifest = Commit::Decode(&store.Read(id).expect("reads").bytes).expect("decodes");

    assert_eq!(manifest.schema, COMMIT_SCHEMA);
    assert_eq!(manifest.snapshot, snapshot.snapshot);
    assert_eq!(manifest.generation, snapshot.generation);
    assert_eq!(manifest.records.len(), snapshot.records.len());
    for (reference, record) in manifest.records.iter().zip(&snapshot.records)
    {
        assert_eq!(reference.document, record.Document().Id());
        assert_eq!(
            store.Read(reference.document).expect("reads").bytes,
            record.bytes,
            "a recorded document is not the bytes the snapshot named"
        );
    }
}

#[test]
fn Test_Committing_The_Same_Snapshot_Twice_Should_Address_The_Same_Document()
{
    let mut store = Observed();

    let first = store.Commit(&Taken(1)).expect("commits");
    let documents = store.Len();
    let second = store.Commit(&Taken(1)).expect("commits again");

    assert_eq!(first, second, "content addressing produced two addresses for one content");
    assert_eq!(store.Len(), documents, "a re-commit duplicated every document");
}

#[test]
fn Test_A_Changed_Record_Should_Address_A_Different_Snapshot()
{
    let mut store = Observed();
    let edited = Commit::Under(
        SnapshotId::From_Digest(Digest(1)),
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
        GenerationId::INITIAL,
    )
    .Recording(Fact("fn main() { edited }"));

    let first = store.Commit(&Taken(1)).expect("commits");
    let second = store.Commit(&edited).expect("commits");

    assert_ne!(first, second);
}

#[test]
fn Test_The_Index_Should_Be_Rebuildable_From_The_Documents_Alone()
{
    let mut store = Observed();
    store.Commit(&Taken(1)).expect("commits");
    store.Commit(&Taken(4)).expect("commits");

    let before = store.Index().expect("indexes").clone();
    store.Drop_Index();
    assert!(!store.Has_Index(), "the index survived being dropped");
    let after = store.Index().expect("rebuilds").clone();

    assert_eq!(before, after, "the index does not rebuild to itself");
    assert_eq!(before.Digest(), after.Digest());
    assert!(!before.Is_Empty(), "an empty index rebuilds to an empty index and proves nothing");
}

#[test]
fn Test_The_Index_Should_Reach_Every_Recorded_Document()
{
    let snapshot = Taken(1);
    let mut store = Observed();
    let id = store.Commit(&snapshot).expect("commits");

    let index = store.Index().expect("indexes");
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
    store.Commit(&Taken(1)).expect("commits");

    let index = store.Index().expect("indexes");

    assert_eq!(index.Of_Kind(DocumentKind::Fact).len(), 2);
    assert_eq!(index.Of_Kind(DocumentKind::Finding).len(), 1);
    assert_eq!(index.Of_Kind(DocumentKind::Commit).len(), 1);
    assert_eq!(index.Of_Schema("nomos.syntax.v1").len(), 2);
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
    let first = Commit::Under(
        state,
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
        GenerationId::INITIAL,
    )
    .Recording(Fact("fn main() {}"));
    let second = Commit::Under(
        state,
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
        GenerationId::INITIAL,
    )
    .Recording(Finding("unused import at nomos.rs"));

    let mut store = Observed();
    let one = store.Commit(&first).expect("commits");
    let other = store.Commit(&second).expect("commits");

    assert_ne!(one, other, "two different commits addressed as one document");

    let commits = store.Index().expect("indexes").Commits_Under(state);

    assert_eq!(commits.len(), 2, "one commit under this state is unreachable: {commits:?}");
    assert!(commits.contains(&one) && commits.contains(&other));
    assert_eq!(
        store.Index().expect("indexes").Snapshots(),
        vec![state],
        "and both are under one workspace state, not two"
    );
    assert!(
        store.Unreachable().expect("indexes").is_empty(),
        "a commit that fell out of the index takes its records with it"
    );
}

#[test]
fn Test_Two_Stores_Given_The_Same_Snapshots_In_Any_Order_Should_Index_Identically()
{
    let mut forwards = Observed();
    forwards.Commit(&Taken(1)).expect("commits");
    forwards.Commit(&Taken(4)).expect("commits");

    let mut backwards = Observed();
    backwards.Commit(&Taken(4)).expect("commits");
    backwards.Commit(&Taken(1)).expect("commits");

    assert_eq!(
        forwards.Index().expect("indexes").Digest(),
        backwards.Index().expect("indexes").Digest(),
        "the index depends on the order the snapshots arrived in"
    );
}

#[test]
fn Test_Every_Document_Should_Be_Reachable_From_A_Snapshot()
{
    let mut store = Observed();
    store.Commit(&Taken(1)).expect("commits");
    store.Commit(&Taken(4)).expect("commits");

    assert!(
        store.Unreachable().expect("indexes").is_empty(),
        "a document is in the store that no snapshot records, so something wrote by another \
         route"
    );
    assert!(store.Len() >= 4, "the store holds too little for that to mean anything");
}

#[test]
fn Test_An_Authored_Document_Should_Not_Enter_An_Observed_Store()
{
    let mut store = Observed();
    let authored = Commit::Under(
        SnapshotId::From_Digest(Digest(1)),
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
        GenerationId::INITIAL,
    )
    .Recording(Recorded::New(
        DocumentKind::Record,
        SchemaId::New("nomos.record.v1"),
        b"D-129".to_vec(),
    ));

    let refusal = store.Commit(&authored).expect_err("must refuse");

    assert!(
        matches!(refusal, StoreError::WrongAuthority { .. }),
        "{refusal}"
    );
    assert!(format!("{refusal}").contains("never share a store"), "{refusal}");
    assert_eq!(store.Len(), 0, "the refused snapshot wrote documents anyway");
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
            .filter(|authority| return authority.Admits(*kind))
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
        BuildVariantId::From_Digest(Digest(2)),
        ConfigurationId::From_Digest(Digest(3)),
        GenerationId::INITIAL,
    );

    let refusal = store.Commit(&empty).expect_err("must refuse");

    assert!(format!("{refusal}").contains("never happened"), "{refusal}");
    assert_eq!(store.Len(), 0);
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
    let id = store.Commit(&snapshot).expect("commits");
    let before = store.Read(id).expect("reads").clone();

    store.Commit(&Taken(4)).expect("commits");

    assert_eq!(&before, store.Read(id).expect("reads"));
}

#[test]
fn Test_Decoding_Something_That_Is_Not_A_Snapshot_Should_Be_Refused()
{
    let refusal = Commit::Decode(b"{\"schema\":\"nomos.other.v1\"}").expect_err("must refuse");

    assert!(matches!(refusal, StoreError::Malformed(_)), "{refusal}");
}
