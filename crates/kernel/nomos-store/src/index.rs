use crate::Commit;
use crate::Document;
use crate::DocumentId;
use crate::DocumentKind;
use crate::StoreError;
use nomos_contracts::{Digest128, SnapshotId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index
{
    kinds: BTreeMap<DocumentKind, BTreeSet<DocumentId>>,
    schemas: BTreeMap<String, BTreeSet<DocumentId>>,
    /// Every document written under a workspace state: each commit's manifest and the
    /// records it named. What `Unreachable` measures against.
    reachable: BTreeMap<SnapshotId, BTreeSet<DocumentId>>,
    /// The commits made under each workspace state.
    ///
    /// A set, and it was a single `DocumentId` until the name of the thing it holds was
    /// corrected. While a commit manifest was called a snapshot, "the snapshot document for
    /// snapshot S" read as a tautology and the map read as a lookup — one entry per state,
    /// obviously. It is not: a workspace state is a tree, a commit is one write against it,
    /// and analyzing a tree twice without editing it produces two commits under one state.
    /// The map silently kept the last.
    commits: BTreeMap<SnapshotId, BTreeSet<DocumentId>>,
}

impl Index
{
    /// # Errors
    ///
    /// Returns [`StoreError`] if a commit document among `documents` fails to decode as a
    /// manifest.
    pub fn Derive(documents: &BTreeMap<DocumentId, Document>) -> Result<Self, StoreError>
    {
        let mut index = Self::default();

        for (id, document) in documents
        {
            index.kinds.entry(document.kind).or_default().insert(*id);
            index
                .schemas
                .entry(document.schema.As_Str().to_owned())
                .or_default()
                .insert(*id);
        }

        for (id, document) in documents
        {
            if document.kind == DocumentKind::Commit
            {
                index.Note_Commit(*id, &document.bytes)?;
            }
        }

        return Ok(index);
    }

    /// What one commit manifest reaches: itself, and every record it names.
    ///
    /// The manifest is reachable from its own snapshot because it is the thing that names
    /// the others. A walk that reached the records and not the manifest would find the
    /// contents of a commit and no evidence that the commit happened.
    fn Note_Commit(&mut self, id: DocumentId, bytes: &[u8]) -> Result<(), StoreError>
    {
        let manifest = Commit::Decode(bytes)?;
        self.commits.entry(manifest.snapshot).or_default().insert(id);

        let members = self.reachable.entry(manifest.snapshot).or_default();
        members.insert(id);
        for reference in &manifest.records
        {
            members.insert(reference.document);
        }

        return Ok(());
    }

    #[must_use]
    pub fn Of_Kind(&self, kind: DocumentKind) -> Vec<DocumentId>
    {
        return self
            .kinds
            .get(&kind)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    #[must_use]
    pub fn Of_Schema(&self, schema: &str) -> Vec<DocumentId>
    {
        return self
            .schemas
            .get(schema)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    #[must_use]
    pub fn In_Snapshot(&self, snapshot: SnapshotId) -> Vec<DocumentId>
    {
        return self
            .reachable
            .get(&snapshot)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    /// Every commit made while the workspace was in this state, in a deterministic order.
    ///
    /// A list rather than an `Option`. One workspace state can be committed against any
    /// number of times — nothing has to change for a second analysis to be recorded — and an
    /// `Option` here would say at most one exists while quietly serving whichever arrived
    /// last.
    #[must_use]
    pub fn Commits_Under(&self, snapshot: SnapshotId) -> Vec<DocumentId>
    {
        return self
            .commits
            .get(&snapshot)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    /// The workspace states this store holds commits under.
    #[must_use]
    pub fn Snapshots(&self) -> Vec<SnapshotId>
    {
        return self.commits.keys().copied().collect();
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.kinds.is_empty()
            && self.schemas.is_empty()
            && self.reachable.is_empty()
            && self.commits.is_empty();
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        use nomos_model::Digest_Of_Parts;

        let mut parts: Vec<Vec<u8>> = Vec::new();

        for (kind, ids) in &self.kinds
        {
            parts.push(kind.Label().as_bytes().to_vec());
            parts.push(Joined_Ids(ids));
        }
        for (schema, ids) in &self.schemas
        {
            parts.push(schema.as_bytes().to_vec());
            parts.push(Joined_Ids(ids));
        }
        Digest_By_Snapshot(&mut parts, &self.reachable);
        Digest_By_Snapshot(&mut parts, &self.commits);

        let borrowed: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();

        return Digest_Of_Parts(&borrowed);
    }
}

/// The parts one snapshot-keyed map contributes to the digest.
///
/// The two maps are walked separately and both appear, because a snapshot's reachable set
/// and its commit set are different claims about it — folding them together would let a
/// document move from one to the other without the digest noticing.
fn Digest_By_Snapshot(parts: &mut Vec<Vec<u8>>, by_snapshot: &BTreeMap<SnapshotId, BTreeSet<DocumentId>>)
{
    for (snapshot, ids) in by_snapshot
    {
        parts.push(snapshot.to_string().into_bytes());
        parts.push(Joined_Ids(ids));
    }
}

fn Joined_Ids(ids: &BTreeSet<DocumentId>) -> Vec<u8>
{
    let mut joined = Vec::new();
    for id in ids
    {
        joined.extend_from_slice(id.Digest().Bytes());
    }

    return joined;
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{Recorded, COMMIT_SCHEMA};
    use nomos_contracts::{BuildVariantId, ConfigurationId, GenerationId, SchemaId};

    /// The seed of a snapshot distinct from the first one a case builds, so the two commits
    /// share no document and neither looks reachable from the other's snapshot.
    const OTHER_SNAPSHOT_SEED: u8 = 4;
    /// The seeds `Taken` derives its build variant and configuration from. Named so a
    /// reader can see the two digests are deliberately different.
    const BUILD_VARIANT_SEED: u8 = 2;
    const CONFIGURATION_SEED: u8 = 3;

    #[test]
    fn Test_Derive_Should_Reach_Every_Recorded_Document()
    {
        let commit = Taken_Commit(1);
        let documents = Documents_From(&commit);

        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");
        let reachable = index.In_Snapshot(commit.snapshot);

        for id in documents.keys()
        {
            assert!(reachable.contains(id), "{id} was recorded but is unreachable from its snapshot");
        }
    }

    #[test]
    fn Test_Of_Kind_Should_Group_Documents_By_Kind()
    {
        let documents = Documents_From(&Taken_Commit(1));
        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");

        assert_eq!(index.Of_Kind(DocumentKind::Fact).len(), 1);
        assert_eq!(index.Of_Kind(DocumentKind::Commit).len(), 1);
    }

    #[test]
    fn Test_Of_Schema_Should_Group_Documents_By_Schema()
    {
        let documents = Documents_From(&Taken_Commit(1));
        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");

        assert_eq!(index.Of_Schema("nomos.syntax.v1").len(), 1);
        assert!(index.Of_Schema("nomos.absent.v1").is_empty());
    }

    #[test]
    fn Test_In_Snapshot_Should_Not_Return_Members_Of_A_Different_Snapshot()
    {
        let first = Taken_Commit(1);
        let second = Taken_Commit(OTHER_SNAPSHOT_SEED);
        let second_documents = Documents_From(&second);
        let mut documents = Documents_From(&first);
        documents.extend(second_documents.clone());

        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");
        let first_members = index.In_Snapshot(first.snapshot);

        for id in second_documents.keys()
        {
            assert!(!first_members.contains(id), "{id} belongs to a different snapshot");
        }
    }

    #[test]
    fn Test_Commits_Under_Should_List_Every_Commit_Made_Against_A_Snapshot()
    {
        let commit = Taken_Commit(1);
        let documents = Documents_From(&commit);
        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");

        assert_eq!(index.Commits_Under(commit.snapshot).len(), 1);
    }

    #[test]
    fn Test_Snapshots_Should_List_Every_Workspace_State_With_A_Commit()
    {
        let commit = Taken_Commit(1);
        let documents = Documents_From(&commit);
        let index = Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads");

        assert_eq!(index.Snapshots(), vec![commit.snapshot]);
    }

    #[test]
    fn Test_Is_Empty_Should_Be_True_Only_When_Nothing_Was_Derived()
    {
        assert!(Index::default().Is_Empty());

        let documents = Documents_From(&Taken_Commit(1));

        assert!(!Index::Derive(&documents).expect("every Commit-kind document here is a manifest Commit::Decode reads").Is_Empty());
    }

    #[test]
    fn Test_Digest_Should_Change_When_The_Indexed_Documents_Change()
    {
        let first = Index::Derive(&Documents_From(&Taken_Commit(1))).expect("every Commit-kind document here is a manifest Commit::Decode reads");
        let second = Index::Derive(&Documents_From(&Taken_Commit(OTHER_SNAPSHOT_SEED))).expect("every Commit-kind document here is a manifest Commit::Decode reads");

        assert_ne!(first.Digest(), second.Digest());
    }

    fn Seeded_Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Fact_Record(payload: &str) -> Recorded
    {
        return Recorded::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), payload.as_bytes().to_vec());
    }

    /// A one-record commit under a snapshot derived from `seed`, recording content that is
    /// itself derived from `seed` -- so two different seeds never share a document by
    /// content-addressing coincidence, which would make them look reachable from each
    /// other's snapshot for a reason that has nothing to do with `In_Snapshot` itself.
    fn Taken_Commit(seed: u8) -> Commit
    {
        return Commit::Under(
            SnapshotId::From_Digest(Seeded_Digest(seed)),
            BuildVariantId::From_Digest(Seeded_Digest(BUILD_VARIANT_SEED)),
            ConfigurationId::From_Digest(Seeded_Digest(CONFIGURATION_SEED)),
            GenerationId::INITIAL,
        )
        .Recording(Fact_Record(&format!("fn seed_{seed}() {{}}")));
    }

    /// The documents a commit's own write would insert: one per record, plus its manifest.
    fn Documents_From(commit: &Commit) -> BTreeMap<DocumentId, Document>
    {
        let mut documents = BTreeMap::new();
        for record in &commit.records
        {
            let document = record.Document();
            documents.insert(document.Id(), document);
        }
        let manifest = Document::New(
            DocumentKind::Commit,
            SchemaId::New(COMMIT_SCHEMA),
            commit.Encode().expect("Manifest holds only schema strings and digest fields, which serde_json encodes"),
        );
        documents.insert(manifest.Id(), manifest);

        return documents;
    }
}
