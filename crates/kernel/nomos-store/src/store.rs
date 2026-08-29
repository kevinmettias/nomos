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

    /// Writes every record in `commit`, then a manifest naming them, as one document each.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] if this store's authority does not admit a record's kind or the
    /// commit's own manifest kind, or if `commit` records nothing.
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

    /// # Errors
    ///
    /// Returns [`StoreError::NoSuchDocument`] if `id` names nothing this store holds.
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

    /// The store's index, deriving it first if a mutation has dropped the cached one.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] if deriving the index fails.
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

    /// Every document this store holds that no commit's manifest reaches from any snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] if the index cannot be derived.
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
