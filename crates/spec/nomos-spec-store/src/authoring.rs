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

mod commit;
mod difference;
mod write;

pub(crate) use difference::{Block_Changes, Identity_Changes, Relation_Changes, Why_Not_Canonical};
pub(crate) use write::Write_Record;

use commit::Apply;
use difference::Located;

use nomos_spec_model::{
    BlockKind, ContentHash, Parse_Record, RecordFrontMatter, RecordRelation,
    Render_Record, SourceBlock,
};
use rusqlite::{OptionalExtension, params};

use crate::ClaimedRecord;
use crate::read::columns::Columns;
use crate::CommitReport;
use crate::DocumentSource;
use crate::EditError;
use crate::EditPreview;
use crate::NormativeMovement;
use crate::record::Kind_Of;
use crate::RecordProjection;
use crate::RecordWrite;
use crate::store::{
    Collected, SpecificationStore,
};
use crate::StoreError;

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
    pub fn Put_Record(
        &mut self,
        path: &str,
        revision: &str,
        markdown: &str,
    ) -> Result<RecordWrite, StoreError>
    {
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

    /// A record, rendered back out of the store's own rows as markdown.
    ///
    /// # Errors
    ///
    /// Returns [`EditError`] naming why no single record answered.
    pub fn Record_Markdown(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<RecordProjection, EditError>
    {
        let document = self.Sole_Document(node_id, revision)?;

        return self.Projection_Of(&document);
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
        return self.In_Transaction(|transaction| return Apply(transaction, preview));
    }

    /// The blocks of one document, in the order they were authored.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Stored_Blocks(&self, document_uid: i64) -> Result<Vec<SourceBlock>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT ordinal, kind, heading_path, text FROM source_blocks
             WHERE document_uid = ?1 ORDER BY ordinal",
        )?;
        let rows = statement.query_map(params![document_uid], |row| {
            let mut columns = Columns::Of(row);
            let ordinal = columns.Next()?;
            let kind: String = columns.Next()?;
            let path: String = columns.Next()?;

            return Ok(SourceBlock {
                ordinal,
                kind: Kind_Of(&kind).unwrap_or(BlockKind::Prose),
                heading_path: Heading_Path(&path),
                text: columns.Next()?,
            });
        })?;

        return Collected(rows);
    }

    /// The front matter a document declared, as it declared it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Declared_Front_Matter(
        &self,
        document_uid: i64,
    ) -> Result<Option<RecordFrontMatter>, StoreError>
    {
        let found = self
            .Connection()
            .query_row(
                "SELECT node.node_id, node.kind, node.title, node.authority,
                        front.status, front.version, front.tags_json
                 FROM record_front_matter front
                 JOIN nodes node ON node.uid = front.node_uid
                 WHERE front.document_uid = ?1",
                params![document_uid],
                Declared_Row,
            )
            .optional()?;
        let Some(declared) = found
        else
        {
            return Ok(None);
        };

        return Ok(Some(RecordFrontMatter {
            id: declared.id,
            kind: declared.kind,
            title: declared.title,
            status: declared.status,
            version: declared.version,
            authority: declared.authority,
            tags: serde_json::from_str(&declared.tags).unwrap_or_default(),
            relations: self.Declared_Relations(document_uid)?,
        }));
    }

    /// The relations a document declared, in the order it declared them.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Declared_Relations(
        &self,
        document_uid: i64,
    ) -> Result<Vec<RecordRelation>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT target, relation FROM record_relations
             WHERE document_uid = ?1 ORDER BY ordinal",
        )?;
        let rows = statement.query_map(params![document_uid], |row| {
            return Ok(RecordRelation {
                target: row.get(0)?,
                relation: row.get(1)?,
            });
        })?;

        return Collected(rows);
    }

    /// The one document behind an identifier, or why there is not exactly one.
    fn Sole_Document(
        &self,
        node_id: &str,
        revision: Option<&str>,
    ) -> Result<DocumentSource, EditError>
    {
        let documents = self.Documents_Behind(node_id, revision)?;

        if let [only] = documents.as_slice()
        {
            return Ok(only.clone());
        }
        if documents.len() > 1
        {
            return Err(Too_Many_Revisions(node_id, &documents));
        }

        return Err(self.Nothing_To_Read(node_id)?);
    }

    /// Which refusal a record with no document behind it deserves: an identity this store
    /// holds and has no content for, or no identity at all.
    fn Nothing_To_Read(&self, node_id: &str) -> Result<EditError, StoreError>
    {
        if self.Node_Summary(node_id)?.is_some()
        {
            return Ok(EditError::NoContent {
                node_id: node_id.to_owned(),
            });
        }

        return Ok(EditError::NoSuchRecord {
            node_id: node_id.to_owned(),
        });
    }

    /// The markdown for one document, rendered from its rows.
    fn Projection_Of(&self, document: &DocumentSource) -> Result<RecordProjection, EditError>
    {
        let front_matter = self
            .Declared_Front_Matter(document.uid)?
            .ok_or_else(|| {
                return EditError::NotAuthored {
                    path: document.path.clone(),
                };
            })?;
        let blocks = self.Stored_Blocks(document.uid)?;
        let markdown = Render_Record(&front_matter, &blocks)?;

        return Ok(RecordProjection {
            node_id: front_matter.id,
            path: document.path.clone(),
            revision: document.revision.clone(),
            projected_hash: ContentHash::Of(&markdown).As_Str().to_owned(),
            source_hash: document.content_hash.clone(),
            markdown,
        });
    }

    /// What became of each normative statement recorded against this record.
    pub(crate) fn Statement_Movements(
        &self,
        node_id: &str,
        before: &[SourceBlock],
        after: &[SourceBlock],
    ) -> Result<Vec<NormativeMovement>, StoreError>
    {
        let mut movements = Vec::new();
        for (statement_id, canonical_text, canonical_hash) in self.Recorded_Statements(node_id)?
        {
            movements.push(NormativeMovement {
                statement_id,
                canonical_hash,
                outcome: Located(&canonical_text, before, after),
            });
        }

        return Ok(movements);
    }

    /// Every normative statement recorded against a record: its identifier, its canonical
    /// text, and what that text hashes to.
    fn Recorded_Statements(&self, node_id: &str) -> Result<Vec<Recorded>, StoreError>
    {
        let mut statement = self.Connection().prepare(
            "SELECT s.statement_id, s.canonical_text, s.canonical_hash
             FROM normative_statements s
             JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1
             ORDER BY s.statement_id",
        )?;
        let rows = statement.query_map(params![node_id], |row| {
            let mut columns = Columns::Of(row);
            return Ok((columns.Next()?, columns.Next()?, columns.Next()?));
        })?;

        return Collected(rows);
    }
}

/// A normative statement as the store keeps it: identifier, canonical text, canonical hash.
type Recorded = (String, String, String);

/// The authored document a record arrived as: where it lives, at which revision, and the
/// bytes themselves.
#[derive(Clone, Copy)]
pub(crate) struct Authored<'a>
{
    pub(crate) path: &'a str,
    pub(crate) revision: &'a str,
    pub(crate) markdown: &'a str,
}

/// More than one revision answered, so editing whichever came back first would be a guess.
fn Too_Many_Revisions(node_id: &str, documents: &[DocumentSource]) -> EditError
{
    return EditError::Ambiguous {
        node_id: node_id.to_owned(),
        revisions: documents
            .iter()
            .map(|document| return document.revision.clone())
            .collect(),
    };
}

/// The row a document's declared front matter joins to.
struct DeclaredRow
{
    id: String,
    kind: String,
    title: String,
    authority: String,
    status: String,
    version: u32,
    tags: String,
}

fn Declared_Row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeclaredRow>
{
    let mut columns = Columns::Of(row);

    return Ok(DeclaredRow {
        id: columns.Next()?,
        kind: columns.Next()?,
        title: columns.Next()?,
        authority: columns.Next()?,
        status: columns.Next()?,
        version: columns.Next()?,
        tags: columns.Next()?,
    });
}

fn Heading_Path(stored: &str) -> Vec<String>
{
    if stored.is_empty()
    {
        return Vec::new();
    }

    return stored.split(" / ").map(str::to_owned).collect();
}
