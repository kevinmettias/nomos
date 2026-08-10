use crate::commit::Commit;
use crate::document::Document;
use crate::document_id::DocumentId;
use crate::document_kind::DocumentKind;
use crate::StoreError;
use nomos_contracts::{Digest128, SnapshotId};
use nomos_model::Digest_Of_Parts;
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
        let mut parts: Vec<Vec<u8>> = Vec::new();

        for (kind, ids) in &self.kinds
        {
            parts.push(kind.Label().as_bytes().to_vec());
            parts.push(Joined(ids));
        }
        for (schema, ids) in &self.schemas
        {
            parts.push(schema.as_bytes().to_vec());
            parts.push(Joined(ids));
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
        parts.push(Joined(ids));
    }
}

fn Joined(ids: &BTreeSet<DocumentId>) -> Vec<u8>
{
    let mut joined = Vec::new();
    for id in ids
    {
        joined.extend_from_slice(id.Digest().Bytes());
    }

    return joined;
}
