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

use crate::columns::Columns;
use crate::document_source::DocumentSource;
use crate::record::{Disposition, Kind_Label, Kind_Of};
use crate::store::{Collected, EXTERNAL, Inverse_Of, SpecificationStore, Write_Blob, Write_Node, Write_Relation, Write_Source_Blocks, Write_Source_Document};
use crate::block_change::BlockChange;
use crate::claimed_record::ClaimedRecord;
use crate::commit_report::CommitReport;
use crate::edit_error::EditError;
use crate::edit_preview::EditPreview;
use crate::identity_change::IdentityChange;
use crate::node_row::NodeRow;
use crate::normative_movement::NormativeMovement;
use crate::normative_outcome::NormativeOutcome;
use crate::record_projection::RecordProjection;
use crate::record_write::RecordWrite;
use crate::store_error::StoreError;
use nomos_spec_model::{
    BlockKind, ContentHash, Normalize, Parse_Record, Record, RecordFrontMatter, RecordRelation,
    Render_Record, Segment, SourceBlock,
};
use rusqlite::{Connection, OptionalExtension, params};

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

/// Writes one record through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Record(
    connection: &Connection,
    authored: Authored<'_>,
    record: &Record,
) -> Result<RecordWrite, StoreError>
{
    let Authored { path, revision, markdown } = authored;
    let front_matter = &record.front_matter;
    let node_uid = Write_Node(connection, NodeRow {
        node_id: &front_matter.id,
        kind: &front_matter.kind,
        authority: &front_matter.authority,
        representation: "document",
        title: &front_matter.title,
    })?;
    let document_uid = Write_Source_Document(connection, path, revision, markdown)?;
    let blocks = Segment(&record.body);
    Write_Source_Blocks(connection, document_uid, &blocks)?;
    let headings = Write_Headings(connection, document_uid, node_uid, &blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, front_matter)?;
    let relations = Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;

    return Ok(RecordWrite {
        node_uid,
        document_uid,
        blocks: u32::try_from(blocks.len()).unwrap_or(u32::MAX),
        headings,
        relations,
    });
}

/// Writes each heading and points it at the record it belongs to.
///
/// The disposition is written against the heading's own identity — its title and depth —
/// rather than against the ordinal it happened to have. An edit that moves a section changes
/// every following ordinal, and a lineage row that only matches the old number is a lineage
/// row an edit silently drops.
fn Write_Headings(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<u32, StoreError>
{
    let mut insert_heading = connection.prepare(
        "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut dispose_heading = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition, target_node_uid)
         SELECT uid, ?4, ?5 FROM source_headings
         WHERE document_uid = ?1 AND title = ?2 AND depth = ?3",
    )?;
    let headings: Vec<Heading<'_>> = blocks
        .iter()
        .filter(|block| return block.kind == BlockKind::Heading)
        .map(Heading_Of)
        .collect();
    for heading in &headings
    {
        insert_heading
            .execute(params![document_uid, heading.ordinal, heading.depth, heading.title])?;
        dispose_heading.execute(params![
            document_uid,
            heading.title,
            heading.depth,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(u32::try_from(headings.len()).unwrap_or(u32::MAX));
}

/// A heading as the schema files it: where it sat, and the identity the lineage row matches
/// on.
struct Heading<'a>
{
    ordinal: u32,
    depth: i64,
    title: &'a str,
}

/// Reads a heading's depth from its markers and its title from what follows them.
fn Heading_Of(block: &SourceBlock) -> Heading<'_>
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();

    return Heading {
        ordinal: block.ordinal,
        depth: i64::try_from(depth).unwrap_or(0),
        title: block.text.trim_start_matches('#').trim(),
    };
}

/// Points every block at the record it belongs to.
fn Dispose_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut dispose = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, ?3, ?4 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
    )?;

    for block in blocks
    {
        dispose.execute(params![
            document_uid,
            block.ordinal,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(());
}

/// Writes the three front matter fields nothing else keeps.
fn Write_Front_Matter(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    front_matter: &RecordFrontMatter,
) -> Result<(), StoreError>
{
    let tags = serde_json::to_string(&front_matter.tags)
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    connection.execute(
        "INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(document_uid) DO UPDATE SET
             node_uid = excluded.node_uid,
             status = excluded.status,
             version = excluded.version,
             tags_json = excluded.tags_json",
        params![
            document_uid,
            node_uid,
            front_matter.status,
            front_matter.version,
            tags
        ],
    )?;

    return Ok(());
}

/// Replaces the declared relation list, in order.
///
/// Replaced rather than merged: the list is a document's own sentence about itself, and
/// merging would make a removed relation unremovable through the surface that wrote it.
fn Write_Declared_Relations(
    connection: &Connection,
    document_uid: i64,
    relations: &[RecordRelation],
) -> Result<u32, StoreError>
{
    connection.execute(
        "DELETE FROM record_relations WHERE document_uid = ?1",
        params![document_uid],
    )?;

    let mut insert = connection.prepare(
        "INSERT INTO record_relations (document_uid, ordinal, target, relation)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    let mut ordinal = 0_u32;
    for relation in relations
    {
        ordinal = ordinal.saturating_add(1);
        insert.execute(params![document_uid, ordinal, relation.target, relation.relation])?;
    }

    return Ok(ordinal);
}

/// Writes a previewed edit through a caller's transaction.
fn Apply(connection: &Connection, preview: &EditPreview) -> Result<CommitReport, EditError>
{
    let claimed = &preview.staged.claimed;
    let document_uid = claimed.document_uid;
    let front_matter = &preview.staged.record.front_matter;

    if let Some((_, after)) = preview.Rename()
    {
        Rename_Document(connection, document_uid, after, &claimed.projection.revision)?;
    }
    Rewrite_Bytes(connection, document_uid, &preview.staged.markdown)?;
    let node_uid = Restate_Node(connection, front_matter)?;
    let blocks = Segment(&preview.staged.record.body);
    let blocks_removed = Rewrite_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, front_matter)?;
    Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;
    Update_Graph(connection, &front_matter.id, preview)?;

    return Ok(Reported(preview, blocks.len(), blocks_removed));
}

/// Replaces the document's stored bytes with the ones the author staged.
fn Rewrite_Bytes(connection: &Connection, document_uid: i64, markdown: &str)
    -> Result<(), StoreError>
{
    let blob_uid = Write_Blob(connection, markdown.as_bytes())?;

    connection.execute(
        "UPDATE source_documents SET blob_uid = ?2 WHERE uid = ?1",
        params![document_uid, blob_uid],
    )?;

    return Ok(());
}

/// Moves the title and the kind with the author's own edit.
///
/// `Write_Node` refuses to overwrite a real node deliberately — the first writer of a record
/// is its author and a later ingest pass must not restate it — and this is that author,
/// arriving through the door the record decided they would use.
fn Restate_Node(connection: &Connection, front_matter: &RecordFrontMatter)
    -> Result<i64, StoreError>
{
    let node_uid = Node_Uid_Of(connection, &front_matter.id)?;

    connection.execute(
        "UPDATE nodes SET kind = ?2, authority = ?3, title = ?4 WHERE uid = ?1",
        params![
            node_uid,
            front_matter.kind,
            front_matter.authority,
            front_matter.title
        ],
    )?;

    return Ok(node_uid);
}

/// Replaces the document's blocks with the staged ones, and reports how many the edit
/// shortened it past.
fn Rewrite_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, EditError>
{
    Write_Source_Blocks(connection, document_uid, blocks)?;
    Write_Headings(connection, document_uid, node_uid, blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, blocks)?;

    return Prune_Blocks_Beyond(connection, document_uid, blocks.len());
}

/// What the commit did, as the caller reads it back.
fn Reported(preview: &EditPreview, blocks: usize, blocks_removed: usize) -> CommitReport
{
    return CommitReport {
        node_id: preview.staged.record.front_matter.id.clone(),
        path: preview.staged.path.clone(),
        blocks,
        blocks_removed,
        relations_added: preview.relations_added.len(),
        relations_removed: preview.relations_removed.len(),
        renamed: preview.Rename().is_some(),
    };
}

/// Moves a document to a new path, keeping its surrogate.
///
/// `UPDATE` rather than insert-and-delete, because `uid` is what every block, lineage and
/// omission row hangs from. A rename that minted a new document row would be a rename that
/// silently dropped the record's history — which is what makes a rename a migration
/// elsewhere and an ordinary edit here.
fn Rename_Document(
    connection: &Connection,
    document_uid: i64,
    path: &str,
    revision: &str,
) -> Result<(), EditError>
{
    let holder = Document_At(connection, path, revision)?;

    if holder.is_some_and(|found| return found != document_uid)
    {
        return Err(EditError::PathTaken {
            path: path.to_owned(),
        });
    }

    connection
        .execute(
            "UPDATE source_documents SET path = ?2 WHERE uid = ?1",
            params![document_uid, path],
        )
        .map_err(StoreError::from)?;

    return Ok(());
}

/// Which document, if any, already lives at this address.
fn Document_At(connection: &Connection, path: &str, revision: &str)
    -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            params![path, revision],
            |row| row.get(0),
        )
        .optional()?);
}

/// Removes the blocks an edit shortened the document past.
///
/// Refuses rather than deletes where a block carries a justified omission. An omission row
/// is the only sanctioned way for content to leave this store, carrying its reason and the
/// decision that allowed it; deleting the block underneath it would destroy the justification
/// and leave the content gone — the exact pair of events this schema exists to prevent.
fn Prune_Blocks_Beyond(
    connection: &Connection,
    document_uid: i64,
    keep: usize,
) -> Result<usize, EditError>
{
    let keep = i64::try_from(keep).unwrap_or(i64::MAX);
    let justified = Justified_Omissions_Beyond(connection, document_uid, keep)?;

    if justified > 0
    {
        return Err(EditError::Store(StoreError::Sql(format!(
            "{justified} block(s) this edit removes carry a justified omission; deleting them \
             would destroy the justification and the content together"
        ))));
    }

    Detach_Blocks_Beyond(connection, document_uid, keep)?;

    let removed = connection
        .execute(
            "DELETE FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2",
            params![document_uid, keep],
        )
        .map_err(StoreError::from)?;

    return Ok(removed);
}

/// How many blocks past `keep` carry a justified omission.
fn Justified_Omissions_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<u32, StoreError>
{
    return Ok(connection.query_row(
        "SELECT count(*) FROM omissions o
         JOIN source_blocks b ON b.uid = o.source_block_uid
         WHERE b.document_uid = ?1 AND b.ordinal > ?2",
        params![document_uid, keep],
        |row| row.get(0),
    )?);
}

/// Removes everything hanging from the blocks past `keep`, so the blocks themselves can go.
///
/// Written out rather than iterated over a list. These are three different statements run
/// once each, in an order the foreign keys require — a row's lineage before the row, and both
/// before the block they hang from — and a loop said "repeat this" about three steps that are
/// not repetitions of one another.
fn Detach_Blocks_Beyond(connection: &Connection, document_uid: i64, keep: i64)
    -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM lineage WHERE source_table_row_uid IN (
             SELECT r.uid FROM source_table_rows r
             JOIN source_blocks b ON b.uid = r.source_block_uid
             WHERE b.document_uid = ?1 AND b.ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM lineage WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;
    connection.execute(
        "DELETE FROM source_table_rows WHERE source_block_uid IN (
             SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal > ?2)",
        params![document_uid, keep],
    )?;

    return Ok(());
}

/// Brings the graph into line with what the record now declares.
fn Update_Graph(
    connection: &Connection,
    node_id: &str,
    preview: &EditPreview,
) -> Result<(), StoreError>
{
    for relation in &preview.relations_added
    {
        Add_Relation(connection, node_id, relation)?;
    }
    for relation in &preview.relations_removed
    {
        Remove_Relation(connection, node_id, relation)?;
    }

    return Ok(());
}

/// Declares a relation, minting the target as an external node when nothing holds it yet.
fn Add_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    if Optional_Node_Uid(connection, &relation.target)?.is_none()
    {
        Write_Node(connection, NodeRow {
            node_id: &relation.target,
            kind: "unknown",
            authority: EXTERNAL,
            representation: "record",
            title: &relation.target,
        })?;
    }

    return Write_Relation(connection, node_id, &relation.relation, &relation.target);
}

/// Withdraws a relation, and the inverse the schema keeps beside it.
fn Remove_Relation(connection: &Connection, node_id: &str, relation: &RecordRelation)
    -> Result<(), StoreError>
{
    Delete_Relation(connection, node_id, &relation.relation, &relation.target)?;

    if let Some(inverse) = Inverse_Of(connection, &relation.relation)?
    {
        Delete_Relation(connection, &relation.target, &inverse, node_id)?;
    }

    return Ok(());
}

fn Delete_Relation(
    connection: &Connection,
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    connection.execute(
        "DELETE FROM relations
         WHERE from_node_uid = (SELECT uid FROM nodes WHERE node_id = ?1)
           AND relation_type = ?2
           AND to_node_uid = (SELECT uid FROM nodes WHERE node_id = ?3)",
        params![from_node_id, relation_type, to_node_id],
    )?;

    return Ok(());
}

fn Node_Uid_Of(connection: &Connection, node_id: &str) -> Result<i64, StoreError>
{
    return Optional_Node_Uid(connection, node_id)?.ok_or_else(|| {
        return StoreError::Sql(format!(
            "{node_id} was read out of this store and is no longer in it"
        ));
    });
}

fn Optional_Node_Uid(connection: &Connection, node_id: &str) -> Result<Option<i64>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| row.get(0),
        )
        .optional()?);
}

/// What became of each block, before and after.
pub(crate) fn Block_Changes(before: &[SourceBlock], after: &[SourceBlock]) -> Vec<BlockChange>
{
    let mut taken = vec![false; before.len()];
    let mut changes = Vec::new();
    let mut added: Vec<&SourceBlock> = Vec::new();

    for block in after
    {
        match Claimed(before, &mut taken, block)
        {
            Match::Held(change) => changes.extend(change),
            Match::New => added.push(block),
        }
    }

    let removed = Untaken(before, &taken);
    let paired = Paired(&added, &removed);

    changes.extend(paired);
    changes.sort_by_key(Position_Of);

    return changes;
}

/// What a staged block turned out to be against the blocks that were already there.
enum Match
{
    /// One of them is this block, and this is what became of it — nothing worth reporting,
    /// where it neither moved nor was reflowed.
    Held(Option<BlockChange>),
    /// Nothing before this edit carries this wording.
    New,
}

/// Claims the block this staged one is — verbatim first, then under the normalizer — so that
/// no earlier block is matched twice, and says what became of it.
fn Claimed(before: &[SourceBlock], taken: &mut [bool], block: &SourceBlock) -> Match
{
    if let Some(index) = Unmatched(before, taken, |candidate| return candidate.text == block.text)
    {
        Take(taken, index);
        let found = before.get(index);
        let moved = Displaced(found, block.ordinal);

        return Match::Held(moved);
    }
    if let Some(index) = Unmatched(before, taken, |candidate| {
        return Normalize(&candidate.text) == Normalize(&block.text);
    })
    {
        Take(taken, index);

        return Match::Held(Some(BlockChange::Reflowed {
            ordinal: block.ordinal,
        }));
    }

    return Match::New;
}

/// The same wording at a different ordinal moved. At the same ordinal, nothing happened to
/// it, and reporting that would be noise in a preview an author has to read every time.
fn Displaced(found: Option<&SourceBlock>, to: u32) -> Option<BlockChange>
{
    return found
        .filter(|block| return block.ordinal != to)
        .map(|block| {
            return BlockChange::Moved {
                from: block.ordinal,
                to,
            };
        });
}

/// The blocks no staged block claimed, which are the ones the edit removed.
fn Untaken<'a>(before: &'a [SourceBlock], taken: &[bool]) -> Vec<&'a SourceBlock>
{
    return before
        .iter()
        .enumerate()
        .filter(|(index, _)| return Is_Free(taken, *index))
        .map(|(_, block)| return block)
        .collect();
}

/// An addition and a removal at one ordinal are a rewording, and saying so is more use to a
/// reader than two lines that do not mention each other.
fn Paired(added: &[&SourceBlock], removed: &[&SourceBlock]) -> Vec<BlockChange>
{
    let mut changes = Vec::new();

    for block in added
    {
        let change = Added_Or_Reworded(block, removed);
        changes.push(change);
    }
    for gone in removed
    {
        if !added.iter().any(|block| return block.ordinal == gone.ordinal)
        {
            changes.push(BlockChange::Removed {
                ordinal: gone.ordinal,
                kind: Kind_Label(gone.kind).to_owned(),
            });
        }
    }

    return changes;
}

/// An addition at an ordinal something was removed from is that block, reworded.
fn Added_Or_Reworded(block: &SourceBlock, removed: &[&SourceBlock]) -> BlockChange
{
    let Some(gone) = removed.iter().find(|gone| return gone.ordinal == block.ordinal)
    else
    {
        return BlockChange::Added {
            ordinal: block.ordinal,
            kind: Kind_Label(block.kind).to_owned(),
        };
    };

    return BlockChange::Reworded {
        ordinal: block.ordinal,
        before: ContentHash::Of(&gone.text).As_Str().to_owned(),
        after: ContentHash::Of(&block.text).As_Str().to_owned(),
    };
}

const fn Position_Of(change: &BlockChange) -> u32
{
    return match change
    {
        BlockChange::Added { ordinal, .. }
        | BlockChange::Removed { ordinal, .. }
        | BlockChange::Reworded { ordinal, .. }
        | BlockChange::Reflowed { ordinal } => *ordinal,
        BlockChange::Moved { to, .. } => *to,
    };
}

fn Unmatched(
    before: &[SourceBlock],
    taken: &[bool],
    predicate: impl Fn(&SourceBlock) -> bool,
) -> Option<usize>
{
    for (index, block) in before.iter().enumerate()
    {
        if Is_Free(taken, index) && predicate(block)
        {
            return Some(index);
        }
    }

    return None;
}

/// Whether the block at this index is still unclaimed.
///
/// An index past the end reads as taken, so a walk that runs off the slice matches nothing
/// rather than pairing a change with a block that is not there.
fn Is_Free(taken: &[bool], index: usize) -> bool
{
    return !taken.get(index).copied().unwrap_or(true);
}

fn Take(taken: &mut [bool], index: usize)
{
    if let Some(slot) = taken.get_mut(index)
    {
        *slot = true;
    }
}

/// Which front matter fields the edit changes.
pub(crate) fn Identity_Changes(before: &RecordFrontMatter, after: &RecordFrontMatter) -> Vec<IdentityChange>
{
    let mut changes = Vec::new();

    for (field, was, now) in [
        ("type", before.kind.clone(), after.kind.clone()),
        ("title", before.title.clone(), after.title.clone()),
        ("status", before.status.clone(), after.status.clone()),
        ("authority", before.authority.clone(), after.authority.clone()),
        ("version", before.version.to_string(), after.version.to_string()),
        ("tags", before.tags.join(", "), after.tags.join(", ")),
    ]
    {
        if was != now
        {
            changes.push(IdentityChange {
                field: field.to_owned(),
                before: was,
                after: now,
            });
        }
    }

    return changes;
}

pub(crate) fn Relation_Changes(before: &[RecordRelation], after: &[RecordRelation]) -> RelationChanges
{
    let added = after
        .iter()
        .filter(|relation| return !before.contains(relation))
        .cloned()
        .collect();
    let removed = before
        .iter()
        .filter(|relation| return !after.contains(relation))
        .cloned()
        .collect();

    return RelationChanges { added, removed };
}

/// Where a statement's canonical text sits before and after the edit.
///
/// Compared under the normalizer, because that is what decides whether two spellings are one
/// statement everywhere else in this system. A statement whose block was merely reflowed is
/// held, not moved.
fn Located(canonical_text: &str, before: &[SourceBlock], after: &[SourceBlock]) -> NormativeOutcome
{
    let needle = Normalize(canonical_text);
    let carrying = |blocks: &[SourceBlock]| {
        return blocks
            .iter()
            .find(|block| return Normalize(&block.text).contains(&needle))
            .map(|block| return block.ordinal);
    };

    return match (carrying(before), carrying(after))
    {
        (None, _) => NormativeOutcome::Unlocatable,
        (Some(from), None) => NormativeOutcome::Gone { from },
        (Some(from), Some(to)) if from == to => NormativeOutcome::Held { block: from },
        (Some(from), Some(to)) => NormativeOutcome::Moved { from, to },
    };
}

/// Why the staged text is not what this surface would have written.
pub(crate) fn Why_Not_Canonical(markdown: &str, record: &Record) -> String
{
    return match Render_Record(&record.front_matter, &Segment(&record.body))
    {
        Err(error) => error.to_string(),
        Ok(rendered) => First_Difference(&rendered, markdown),
    };
}

fn First_Difference(expected: &str, found: &str) -> String
{
    for (index, (left, right)) in expected.lines().zip(found.lines()).enumerate()
    {
        if left != right
        {
            return format!(
                "line {}: this surface writes {left:?} and the staged text has {right:?}",
                index.saturating_add(1)
            );
        }
    }

    let (written, staged) = (expected.lines().count(), found.lines().count());
    if written != staged
    {
        return format!("this surface writes {written} line(s) and the staged text has {staged}");
    }

    return "every line matches, so the difference is in the leading or trailing whitespace"
        .to_owned();
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
