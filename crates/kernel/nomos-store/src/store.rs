use crate::Authority;
use crate::Document;
use crate::DocumentId;
use crate::DocumentKind;
use crate::Index;
use crate::{Commit, COMMIT_SCHEMA};
use crate::StoreError;
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

    /// A document this store's authority does not hold.
    ///
    /// Checked for the manifest as well as for the records it names, because a store that
    /// admitted the manifest and not its contents — or the reverse — would hold half a
    /// commit and answer as if it held all of it.
    fn Refuse_Unadmitted(&self, kind: DocumentKind) -> Result<(), StoreError>
    {
        if self.authority.Admits(kind)
        {
            return Ok(());
        }

        return Err(StoreError::WrongAuthority {
            store: self.authority,
            kind,
            document: kind.Authority(),
        });
    }

    /// The manifest a commit would write, once the whole commit has been shown admissible.
    ///
    /// A commit recording nothing is refused rather than written as an empty one: it would
    /// advance the store's history and reach no document, which is indistinguishable from a
    /// commit whose records were lost.
    fn Admitted(&self, commit: &Commit) -> Result<Document, StoreError>
    {
        use nomos_contracts::SchemaId;

        if commit.records.is_empty()
        {
            return Err(StoreError::Vacuous {
                snapshot: commit.snapshot.to_string(),
            });
        }

        for record in &commit.records
        {
            self.Refuse_Unadmitted(record.kind)?;
        }

        let manifest = Document::New(
            DocumentKind::Commit,
            SchemaId::New(COMMIT_SCHEMA),
            commit.Encode()?,
        );
        self.Refuse_Unadmitted(manifest.kind)?;

        return Ok(manifest);
    }

    pub fn Commit(&mut self, commit: &Commit) -> Result<DocumentId, StoreError>
    {
        let manifest = self.Admitted(commit)?;

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
