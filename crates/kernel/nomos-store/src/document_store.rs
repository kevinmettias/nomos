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
        if self.authority.Can_Admit(kind)
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
    pub fn Length(&self) -> usize
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Recorded;
    use nomos_contracts::{BuildVariantId, ConfigurationId, Digest128, GenerationId, SchemaId, SnapshotId};

    #[test]
    fn Test_For_Should_Open_An_Empty_Store()
    {
        assert_eq!(Observed().Length(), 0);
    }

    #[test]
    fn Test_Authority_Should_Report_The_Authority_The_Store_Was_Opened_For()
    {
        assert_eq!(DocumentStore::For(Authority::Authored).Authority(), Authority::Authored);
    }

    #[test]
    fn Test_Commit_Should_Write_Every_Record_And_A_Manifest()
    {
        let mut store = Observed();

        let id = store.Commit(&Taken(1)).expect("commits");

        assert!(store.Read(id).is_ok());
        assert!(store.Length() >= 2);
    }

    #[test]
    fn Test_Read_Should_Return_A_Committed_Bytes_By_Id()
    {
        let mut store = Observed();
        let id = store.Commit(&Taken(1)).expect("commits");

        assert!(store.Read(id).is_ok());
    }

    #[test]
    fn Test_Documents_Should_Expose_Every_Document_The_Store_Holds()
    {
        let mut store = Observed();
        store.Commit(&Taken(1)).expect("commits");

        assert_eq!(store.Documents().len(), store.Length());
    }

    #[test]
    fn Test_Length_Should_Count_The_Stores_Entries()
    {
        let mut store = Observed();
        assert_eq!(store.Length(), 0);

        store.Commit(&Taken(1)).expect("commits");

        assert!(store.Length() > 0);
    }

    #[test]
    fn Test_Index_Should_Derive_When_None_Is_Cached()
    {
        let mut store = Observed();
        store.Commit(&Taken(1)).expect("commits");
        store.Drop_Index();

        assert!(!store.Index().expect("indexes").Is_Empty());
    }

    #[test]
    fn Test_Drop_Index_Should_Clear_The_Cached_Index()
    {
        let mut store = Observed();
        store.Commit(&Taken(1)).expect("commits");
        store.Index().expect("indexes");

        store.Drop_Index();

        assert!(!store.Has_Index());
    }

    #[test]
    fn Test_Has_Index_Should_Report_Whether_An_Index_Is_Cached()
    {
        let mut store = Observed();
        assert!(store.Has_Index(), "a fresh store starts with an empty cached index");

        store.Drop_Index();

        assert!(!store.Has_Index());
    }

    #[test]
    fn Test_Unreachable_Should_Be_Empty_When_Every_Document_Is_Reachable()
    {
        let mut store = Observed();
        store.Commit(&Taken(1)).expect("commits");

        assert!(store.Unreachable().expect("indexes").is_empty());
    }

    fn Digest(seed: u8) -> Digest128
    {
        return Digest128::From_Bytes([seed; Digest128::BYTE_LENGTH]);
    }

    fn Fact(payload: &str) -> Recorded
    {
        return Recorded::New(DocumentKind::Fact, SchemaId::New("nomos.syntax.v1"), payload.as_bytes().to_vec());
    }

    fn Taken(seed: u8) -> Commit
    {
        return Commit::Under(
            SnapshotId::From_Digest(Digest(seed)),
            BuildVariantId::From_Digest(Digest(2)),
            ConfigurationId::From_Digest(Digest(3)),
            GenerationId::INITIAL,
        )
        .Recording(Fact("fn main() {}"));
    }

    fn Observed() -> DocumentStore
    {
        return DocumentStore::For(Authority::Observed);
    }
}
