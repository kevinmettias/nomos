use crate::document::{Document, DocumentId, DocumentKind};
use crate::snapshot::Snapshot;
use crate::StoreError;
use nomos_contracts::{Digest128, SnapshotId};
use nomos_model::Digest_Of_Parts;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Index
{
    by_kind: BTreeMap<DocumentKind, BTreeSet<DocumentId>>,
    by_schema: BTreeMap<String, BTreeSet<DocumentId>>,
    by_snapshot: BTreeMap<SnapshotId, BTreeSet<DocumentId>>,
    snapshots: BTreeMap<SnapshotId, DocumentId>,
}

impl Index
{
    pub fn Derive(documents: &BTreeMap<DocumentId, Document>) -> Result<Self, StoreError>
    {
        let mut index = Self::default();

        for (id, document) in documents
        {
            index.by_kind.entry(document.kind).or_default().insert(*id);
            index
                .by_schema
                .entry(document.schema.As_Str().to_owned())
                .or_default()
                .insert(*id);
        }

        for (id, document) in documents
        {
            if document.kind != DocumentKind::Snapshot
            {
                continue;
            }

            let manifest = Snapshot::Decode(&document.bytes)?;
            index.snapshots.insert(manifest.snapshot, *id);
            let members = index.by_snapshot.entry(manifest.snapshot).or_default();
            members.insert(*id);
            for reference in &manifest.records
            {
                members.insert(reference.document);
            }
        }

        return Ok(index);
    }

    #[must_use]
    pub fn Of_Kind(&self, kind: DocumentKind) -> Vec<DocumentId>
    {
        return self
            .by_kind
            .get(&kind)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    #[must_use]
    pub fn Of_Schema(&self, schema: &str) -> Vec<DocumentId>
    {
        return self
            .by_schema
            .get(schema)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    #[must_use]
    pub fn In_Snapshot(&self, snapshot: SnapshotId) -> Vec<DocumentId>
    {
        return self
            .by_snapshot
            .get(&snapshot)
            .map(|ids| return ids.iter().copied().collect())
            .unwrap_or_default();
    }

    #[must_use]
    pub fn Snapshot_Document(&self, snapshot: SnapshotId) -> Option<DocumentId>
    {
        return self.snapshots.get(&snapshot).copied();
    }

    #[must_use]
    pub fn Snapshots(&self) -> Vec<SnapshotId>
    {
        return self.snapshots.keys().copied().collect();
    }

    #[must_use]
    pub fn Is_Empty(&self) -> bool
    {
        return self.by_kind.is_empty()
            && self.by_schema.is_empty()
            && self.by_snapshot.is_empty()
            && self.snapshots.is_empty();
    }

    #[must_use]
    pub fn Digest(&self) -> Digest128
    {
        let mut parts: Vec<Vec<u8>> = Vec::new();

        for (kind, ids) in &self.by_kind
        {
            parts.push(kind.Label().as_bytes().to_vec());
            parts.push(Joined(ids));
        }
        for (schema, ids) in &self.by_schema
        {
            parts.push(schema.as_bytes().to_vec());
            parts.push(Joined(ids));
        }
        for (snapshot, ids) in &self.by_snapshot
        {
            parts.push(snapshot.to_string().into_bytes());
            parts.push(Joined(ids));
        }
        for (snapshot, id) in &self.snapshots
        {
            parts.push(snapshot.to_string().into_bytes());
            parts.push(id.Digest().Bytes().to_vec());
        }

        let borrowed: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();

        return Digest_Of_Parts(&borrowed);
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
