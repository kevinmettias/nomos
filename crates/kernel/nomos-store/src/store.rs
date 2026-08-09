use crate::document::{Authority, Document, DocumentId, DocumentKind};
use crate::index::Index;
use crate::commit::{Commit, COMMIT_SCHEMA};
use crate::StoreError;
use nomos_contracts::SchemaId;
use std::collections::BTreeMap;

pub struct DocumentStore
{
    authority: Authority,
    documents: BTreeMap<DocumentId, Document>,
    index: Option<Index>,
}

impl DocumentStore
{
    #[must_use]
    pub fn For(authority: Authority) -> Self
    {
        return Self {
            authority,
            documents: BTreeMap::new(),
            index: Some(Index::default()),
        };
    }

    #[must_use]
    pub const fn Authority(&self) -> Authority
    {
        return self.authority;
    }

    pub fn Commit(&mut self, commit: &Commit) -> Result<DocumentId, StoreError>
    {
        if commit.records.is_empty()
        {
            return Err(StoreError::Vacuous {
                snapshot: commit.snapshot.to_string(),
            });
        }

        for record in &commit.records
        {
            if !self.authority.Admits(record.kind)
            {
                return Err(StoreError::WrongAuthority {
                    store: self.authority,
                    kind: record.kind,
                    document: record.kind.Authority(),
                });
            }
        }

        let manifest = Document::New(
            DocumentKind::Commit,
            SchemaId::New(COMMIT_SCHEMA),
            commit.Encode()?,
        );
        if !self.authority.Admits(manifest.kind)
        {
            return Err(StoreError::WrongAuthority {
                store: self.authority,
                kind: manifest.kind,
                document: manifest.kind.Authority(),
            });
        }

        for record in &commit.records
        {
            let document = record.Document();
            self.documents.insert(document.Id(), document);
        }

        let id = manifest.Id();
        self.documents.insert(id, manifest);
        self.index = None;

        return Ok(id);
    }

    pub fn Read(&self, id: DocumentId) -> Result<&Document, StoreError>
    {
        return self
            .documents
            .get(&id)
            .ok_or(StoreError::NoSuchDocument { document: id });
    }

    #[must_use]
    pub fn Documents(&self) -> &BTreeMap<DocumentId, Document>
    {
        return &self.documents;
    }

    #[must_use]
    pub fn Len(&self) -> usize
    {
        return self.documents.len();
    }

    pub fn Index(&mut self) -> Result<&Index, StoreError>
    {
        if self.index.is_none()
        {
            self.index = Some(Index::Derive(&self.documents)?);
        }

        return self
            .index
            .as_ref()
            .ok_or_else(|| return StoreError::Malformed("the index did not derive".to_owned()));
    }

    pub fn Drop_Index(&mut self)
    {
        self.index = None;
    }

    #[must_use]
    pub fn Has_Index(&self) -> bool
    {
        return self.index.is_some();
    }

    pub fn Unreachable(&mut self) -> Result<Vec<DocumentId>, StoreError>
    {
        let reachable: Vec<DocumentId> = {
            let index = self.Index()?;
            index
                .Snapshots()
                .into_iter()
                .flat_map(|snapshot| return index.In_Snapshot(snapshot))
                .collect()
        };

        return Ok(self
            .documents
            .keys()
            .filter(|id| return !reachable.contains(id))
            .copied()
            .collect());
    }
}
