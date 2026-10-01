//! The authoring round trip `D-129` decided: read a record out as markdown, write an edit
//! back as a transaction against the store.
//!
//! # Four steps, as types
//!
//! `D-129` says an author's edit is *claim, stage, preview, commit*, and that the preview
//! is mandatory. Four functions in a row would leave that a convention: a caller could stage
//! and commit, and the only thing standing between them and an unpreviewed write would be
//! remembering. So each step is a type that only the previous step can produce, and
//! [`SpecificationStore::Commit_Edit`] takes an [`EditPreview`] — a value nothing outside
//! this module can construct. The preview is not checked for; it is required to exist.
//!
//! That also settles staleness without a check. The preview owns the staged bytes rather
//! than pointing at them, so there is no window in which the thing committed is not the
//! thing previewed.
//!
//! # What the projection is made of
//!
//! [`SpecificationStore::Record_Markdown`] renders from the store's own rows — the declared
//! front matter, the blocks — and never from the ingested blob. Reading the blob back would
//! be an echo, and would prove nothing about whether the store holds enough to be the
//! substrate. The projection carries both content addresses so a caller can see that they
//! agree; `Test_Every_Governing_Record_Should_Project_To_Its_Own_Bytes` asserts they do,
//! for all of them.
//!
//! # Why the read-back half is a submodule
//!
//! [`SpecificationStore::Record_Markdown`] and the accessors beside it are read out of the
//! store's rows; the four steps above are the write path. They are two jobs, and
//! `read_back` is where the first one lives. The split is by responsibility, not by size:
//! the four authoring steps are the ones that must be understood together, because the
//! preview's type is what makes the commit unpreviewable.

mod commit;
mod difference;
mod read_back;
mod write;

pub(crate) use difference::{Block_Changes, Identity_Changes, Relation_Changes, Why_Not_Canonical};

use nomos_spec_model::{Parse_Record, RecordRelation};

use crate::ClaimedRecord;
use crate::CommitReport;
use crate::DocumentPath;
use crate::DocumentRevision;
use crate::EditError;
use crate::EditPreview;
use crate::SpecificationStore;
use crate::StoreError;
use crate::Write as RecordWrite;

/// Which relations the edit adds and which it removes.
///
/// Both halves are `Vec<RecordRelation>`, so returning them as a pair let a caller take them
/// in either order and still compile. Named, the mistake cannot be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RelationChanges
{
    pub(crate) added: Vec<RecordRelation>,
    pub(crate) removed: Vec<RecordRelation>,
}

impl SpecificationStore
{
    /// Writes an authored record: identity, bytes, blocks, dispositions and the front matter
    /// it declared.
    ///
    /// The one door. The seed uses it, a re-ingest uses it, and a commit uses the same
    /// writers underneath, so there is no second place that decides what a record is.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Record`] if the markdown is not a readable record, and
    /// [`StoreError`] on any SQL failure.
    pub fn Put_Record<'a>(
        &mut self,
        path: impl Into<DocumentPath<'a>>,
        revision: impl Into<DocumentRevision<'a>>,
        markdown: &str,
    ) -> Result<RecordWrite, StoreError>
    {
        use write::Write_Record;

        let path = path.into().0;
        let revision = revision.into().0;
        let record = Parse_Record(markdown).map_err(|error| {
            return StoreError::Record {
                path: path.to_owned(),
                cause: error.to_string(),
            };
        })?;

        return self.In_Transaction(|transaction| {
            return Write_Record(
                transaction,
                Authored {
                    path,
                    revision,
                    markdown,
                },
                &record,
            );
        });
    }

    /// Step one of the round trip: read the record out and hold it for editing.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] naming why no single record answered.
    pub fn Claim_For_Edit(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<ClaimedRecord, EditError>
    {
        let document = self.Sole_Document(node_id, revision)?;
        let projection = self.Projection_Of(&document)?;
        let front_matter = self
            .Declared_Front_Matter(document.uid)?
            .ok_or_else(|| {
                return EditError::NotAuthored {
                    path: document.path.clone(),
                };
            })?;

        return Ok(ClaimedRecord {
            projection,
            front_matter,
            document_uid: document.uid,
        });
    }

    /// Writes a previewed edit, atomically.
    ///
    /// Takes the preview rather than the staged text, so an unpreviewed commit is not a
    /// discipline but an unwritable program.
    ///
    /// # Errors
    ///
    /// Returns [`EditError::PathTaken`] if a rename would collide, and [`EditError::Store`]
    /// on any SQL failure. Nothing is written if anything fails.
    pub fn Commit_Edit(&mut self, preview: &EditPreview) -> Result<CommitReport, EditError>
    {
        use commit::Apply_Preview;

        return self.In_Transaction(|transaction| return Apply_Preview(transaction, preview));
    }
}

/// The authored document a record arrived as: where it lives, at which revision, and the
/// bytes themselves.
#[derive(Clone, Copy)]
pub(crate) struct Authored<'a>
{
    pub(crate) path: &'a str,
    pub(crate) revision: &'a str,
    pub(crate) markdown: &'a str,
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Seed_Governing_Records;
    use crate::store::AUTHORED;

    #[test]
    fn Test_Put_Record_Should_Write_An_Authored_Record_The_Store_Can_Read_Back()
    {
        let store = With_Synthetic();

        let documents = store
            .Documents_Behind("D-900", None)
            .expect("Put_Record wrote D-900 as one document in this store");

        assert_eq!(documents.len(), 1);
        assert_eq!(
            documents.first().expect("asserted above to contain exactly one document").path,
            PATH
        );
    }

    #[test]
    fn Test_Claim_For_Edit_Should_Read_The_Record_Out_For_Editing()
    {
        let store = With_Synthetic();

        let claimed = store
            .Claim_For_Edit("D-900", None)
            .expect("D-900 is held at one revision with a declared front matter");

        assert_eq!(claimed.Markdown(), CANONICAL);
        assert_eq!(claimed.Node_Id(), "D-900");
    }

    #[test]
    fn Test_Commit_Edit_Should_Apply_A_Previewed_Edit_Atomically()
    {
        let mut store = With_Synthetic();
        let edited = CANONICAL.replace("First paragraph.", "First paragraph, edited.");

        let preview = store
            .Claim_For_Edit("D-900", None)
            .expect("D-900 is held at one revision with a declared front matter")
            .Stage(&edited, None)
            .expect("the edited text parses as a record named D-900")
            .Preview(&store)
            .expect("the staged edit names one record and one target path");
        store
            .Commit_Edit(&preview)
            .expect("the preview's target path is free in this store");

        let projection = store
            .Record_Markdown("D-900", None)
            .expect("Put_Record gave D-900 a declared front matter and blocks");
        assert_eq!(projection.markdown, edited);
    }

    const CANONICAL: &str = "---\nid: D-900\ntype: decision\ntitle: A synthetic record\n\
                             status: accepted\nversion: 1\n\
                             authority: canonical-normative-record\ntags:\n  - testing\n\
                             relations:\n  - target: D-129\n    type: relates-to\n---\n\n\
                             # A synthetic record\n\n## Decision\n\nFirst paragraph.\n\n\
                             ## Rationale\n\nSecond paragraph.\n";
    const PATH: &str = "docs/records/D-900-a-synthetic-record.md";

    fn Seeded() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory()
            .expect("In_Memory builds its own schema, so opening touches no file");
        Seed_Governing_Records(&mut store)
            .expect("the governing record set is compiled in, so seeding reads no file");
        return store;
    }

    fn With_Synthetic() -> SpecificationStore
    {
        let mut store = Seeded();
        store.Put_Record(PATH, AUTHORED, CANONICAL).expect("writes the synthetic record");
        return store;
    }
}
