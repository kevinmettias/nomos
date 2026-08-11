use crate::rows::{RowCensus, RowScope};
use crate::schema::{Latest_Version, MIGRATIONS, Migration};
use nomos_spec_model::{ContentHash, SourceBlock, TableRow, Table_Defects, Table_Rows};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// The revision label for content this repository authors itself.
///
/// Distinct from an ingested corpus revision like `v14.36`, so a query can tell what
/// Nomos said about itself from what it read out of an archive.
pub const AUTHORED: &str = "authored";

/// The authority of a node that exists only because something points at it.
pub const EXTERNAL: &str = "external";

/// Whether a suite is this repository's own specification or one it merely references.
///
/// Named rather than a bool. `Put_Suite(suite_id, title, true)` said nothing at the call
/// site about what was true, and the distinction it carries is the one the store exists to
/// keep: a sibling's statement is quoted, not governed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuiteAuthority
{
    /// This repository's own specification, whose records govern here.
    Root,
    /// A suite read in from elsewhere, present so its nodes can be pointed at.
    Sibling,
}

impl SuiteAuthority
{
    /// How the `authority_root` column spells this.
    const fn Stored(self) -> i64
    {
        return match self
        {
            Self::Root => 1,
            Self::Sibling => 0,
        };
    }

    /// What the `authority_root` column meant.
    const fn Read(stored: i64) -> Self
    {
        return if stored == 0 { Self::Sibling } else { Self::Root };
    }
}

#[derive(Debug)]
pub enum StoreError
{
    Sql(String),
    Migration
    {
        from: u32,
        cause: String,
    },
    /// The database was written by a newer build than this one.
    TooNew
    {
        found: u32,
        supported: u32,
    },
    /// A document does not read: an authored record this build embeds that will not
    /// parse, or stored bytes that are not text. Both name the document and say what
    /// went wrong with it, because in either case the caller's next question is which
    /// file.
    Record
    {
        path: String,
        cause: String,
    },
    /// A block carries something shaped like a table that cannot be read as one.
    Table
    {
        document_uid: i64,
        ordinal: u32,
        cause: String,
    },
}

impl core::fmt::Display for StoreError
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        return match self
        {
            Self::Sql(cause) => write!(formatter, "store error: {cause}"),
            Self::Migration { from, cause } => {
                write!(formatter, "migration from version {from} failed: {cause}")
            }
            Self::TooNew { found, supported } => write!(
                formatter,
                "the store is at schema version {found}; this build understands {supported}. \
                 Refusing to open it rather than reading tables whose meaning may have changed"
            ),
            Self::Record { path, cause } => write!(formatter, "{path}: {cause}"),
            Self::Table {
                document_uid,
                ordinal,
                cause,
            } => write!(formatter, "document {document_uid} block {ordinal}: {cause}"),
        };
    }
}

impl std::error::Error for StoreError
{}

impl From<rusqlite::Error> for StoreError
{
    fn from(error: rusqlite::Error) -> Self
    {
        return Self::Sql(error.to_string());
    }
}

pub struct SpecificationStore
{
    connection: Connection,
}

impl SpecificationStore
{
    /// # Errors
    ///
    /// Returns [`StoreError`] if the database cannot be opened or migrated.
    pub fn Open(path: &Path) -> Result<Self, StoreError>
    {
        return Self::From_Connection(Connection::open(path)?);
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] if the schema cannot be applied.
    pub fn In_Memory() -> Result<Self, StoreError>
    {
        return Self::From_Connection(Connection::open_in_memory()?);
    }

    fn From_Connection(connection: Connection) -> Result<Self, StoreError>
    {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        let mut store = Self { connection };
        store.Migrate()?;
        return Ok(store);
    }

    fn Migrate(&mut self) -> Result<(), StoreError>
    {
        let current: u32 = self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))?;

        if current > Latest_Version()
        {
            return Err(StoreError::TooNew {
                found: current,
                supported: Latest_Version(),
            });
        }

        for migration in MIGRATIONS.iter().filter(|m| m.version > current)
        {
            self.Apply(migration, current)?;
        }

        return Ok(());
    }

    /// One migration, in a transaction of its own, stamping the version it reached.
    ///
    /// The version is written inside the transaction that ran the statements, so a store
    /// cannot come back claiming a version whose statements did not commit.
    fn Apply(&mut self, migration: &Migration, from: u32) -> Result<(), StoreError>
    {
        let transaction = self.connection.transaction()?;

        for statement in migration.statements
        {
            transaction
                .execute_batch(statement)
                .map_err(|error| StoreError::Migration {
                    from,
                    cause: format!("{}: {error}", migration.name),
                })?;
        }
        transaction.pragma_update(None, "user_version", migration.version)?;

        return Ok(transaction.commit()?);
    }

    #[must_use]
    pub fn Version(&self) -> u32
    {
        return self
            .connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0);
    }

    /// Runs `work` in a transaction, rolling back everything if it returns an error.
    ///
    /// # Errors
    ///
    /// Returns whatever `work` returns, having rolled back.
    pub fn In_Transaction<T, E>(
        &mut self,
        work: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<StoreError>,
    {
        let transaction = self.connection.transaction().map_err(StoreError::from)?;
        let outcome = work(&transaction)?;
        transaction.commit().map_err(StoreError::from)?;
        return Ok(outcome);
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Blob(&mut self, content: &[u8]) -> Result<i64, StoreError>
    {
        return Write_Blob(&self.connection, content);
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Source_Document(
        &mut self,
        path: &str,
        revision: &str,
        content: &str,
    ) -> Result<i64, StoreError>
    {
        return Write_Source_Document(&self.connection, path, revision, content);
    }

    /// Writes blocks and, for any block carrying a table, its typed rows.
    ///
    /// Rows are written here rather than by a separate call so a block cannot reach the
    /// store without them. A second entry point would mean the rows exist only where
    /// somebody remembered to ask, and a preservation rule over rows that some documents
    /// have and others do not is a rule that reports clean on the ones it cannot see.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::Table`] if a block's tables do not each carry exactly one
    /// delimiter, and [`StoreError`] on any SQL failure.
    pub fn Put_Source_Blocks(
        &mut self,
        document_uid: i64,
        blocks: &[SourceBlock],
    ) -> Result<usize, StoreError>
    {
        let transaction = self.connection.transaction()?;
        let written = Write_Source_Blocks(&transaction, document_uid, blocks)?;
        transaction.commit()?;

        return Ok(written);
    }

}

/// Writes blocks and their typed rows through a caller's transaction.
///
/// Free rather than a method, and taking a [`Connection`] rather than the store, because
/// [`rusqlite::Transaction`] dereferences to one: the seed, a re-ingest and an authoring
/// commit all reach this same writer, one of them inside a transaction that spans several
/// documents. A second entry point that knew how to write a block would be a second place
/// that could be wrong about what a block is.
///
/// # Errors
///
/// Returns [`StoreError::Table`] if a block's tables do not each carry exactly one
/// delimiter, and [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, StoreError>
{
    Assert_Tables_Are_Sound(document_uid, blocks)?;
    Insert_Blocks(connection, document_uid, blocks)?;

    let uids = Block_Uids(connection, document_uid)?;

    Insert_Table_Rows(connection, document_uid, blocks, &uids)?;

    return Ok(blocks.len());
}

/// Every table in these blocks is one this store will take.
fn Assert_Tables_Are_Sound(document_uid: i64, blocks: &[SourceBlock]) -> Result<(), StoreError>
{
    for block in blocks
    {
        let defects = Table_Defects(&Table_Rows(block));
        if let Some(defect) = defects.first()
        {
            return Err(StoreError::Table {
                document_uid,
                ordinal: block.ordinal,
                cause: defect.to_string(),
            });
        }
    }

    return Ok(());
}

/// The blocks themselves, updated in place where the document already held one.
///
/// Not `INSERT OR REPLACE`. REPLACE deletes the conflicting row and inserts a new one, which
/// hands the block a new `uid` — and `uid` is what every lineage and omission row points at.
/// Re-ingesting a document would silently renumber its blocks and take their dispositions
/// with them.
fn Insert_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut insert = connection.prepare(
        "INSERT INTO source_blocks
         (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(document_uid, ordinal) DO UPDATE SET
             kind = excluded.kind,
             heading_path = excluded.heading_path,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        insert.execute(params![
            document_uid,
            block.ordinal,
            crate::record::Kind_Label(block.kind),
            block.heading_path.join(" / "),
            block.text,
            block.Content_Hash().As_Str(),
            block.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Every block's uid in one crossing, indexed by ordinal.
///
/// Asking per block cost one round trip for each row of the document, to learn surrogates the
/// same document already decides as a set.
fn Block_Uids(
    connection: &Connection,
    document_uid: i64,
) -> Result<std::collections::BTreeMap<u32, i64>, StoreError>
{
    let mut every_uid = connection
        .prepare("SELECT ordinal, uid FROM source_blocks WHERE document_uid = ?1")?;
    let found = every_uid.query_map(params![document_uid], |row| {
        return Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?));
    })?;
    let mut block_uids = std::collections::BTreeMap::new();

    for entry in found
    {
        let (ordinal, uid) = entry?;
        block_uids.insert(ordinal, uid);
    }

    return Ok(block_uids);
}

/// The rows of every table these blocks carry, under the block that carries them.
///
/// Same reasoning as [`Insert_Blocks`]: updated in place rather than replaced, so a re-ingest
/// does not hand a row a new uid.
fn Insert_Table_Rows(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
    uids: &std::collections::BTreeMap<u32, i64>,
) -> Result<(), StoreError>
{
    let mut insert_row = connection.prepare(
        "INSERT INTO source_table_rows
         (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
          content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(source_block_uid, ordinal) DO UPDATE SET
             table_ordinal = excluded.table_ordinal,
             kind = excluded.kind,
             cells_json = excluded.cells_json,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        let rows = Table_Rows(block);
        let uid = Block_Uid_Of(uids, document_uid, block)?;

        Insert_Rows_Under(&mut insert_row, uid, &rows)?;
    }

    return Ok(());
}

/// The surrogate the block just inserted landed under.
///
/// [`Insert_Blocks`] put every block in, so a missing ordinal is a broken invariant rather
/// than a row that has not arrived yet, and it says so.
fn Block_Uid_Of(
    uids: &std::collections::BTreeMap<u32, i64>,
    document_uid: i64,
    block: &SourceBlock,
) -> Result<i64, StoreError>
{
    let found = uids.get(&block.ordinal).copied();

    return found.ok_or_else(|| {
        return StoreError::Sql(format!(
            "source block {} of document {document_uid} has no uid after insertion",
            block.ordinal
        ));
    });
}

/// One block's table rows, through a prepared insert.
fn Insert_Rows_Under(
    insert_row: &mut rusqlite::Statement<'_>,
    uid: i64,
    rows: &[TableRow],
) -> Result<(), StoreError>
{
    for row in rows
    {
        let cells = serde_json::to_string(&row.cells)
            .map_err(|error| return StoreError::Sql(error.to_string()))?;

        insert_row.execute(params![
            uid,
            row.ordinal,
            row.table_ordinal,
            row.kind.Label(),
            cells,
            row.text,
            row.Content_Hash().As_Str(),
            row.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Writes bytes, addressed by content, through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
/// The counting statement each table carries.
///
/// One constant per table rather than eighteen match arms each holding their own string:
/// the arms then say only which statement belongs to which variant, and the statements read
/// as the table of literals they are. Written out rather than interpolated from
/// [`Table::Name`] for the reason [`Table::Tally_Sql`] gives.
const TALLY_BLOBS: &str = "SELECT 'blobs' AS which, count(*) AS tally FROM blobs";
const TALLY_SOURCE_DOCUMENTS: &str =
    "SELECT 'source_documents' AS which, count(*) AS tally FROM source_documents";
const TALLY_SOURCE_HEADINGS: &str =
    "SELECT 'source_headings' AS which, count(*) AS tally FROM source_headings";
const TALLY_SOURCE_BLOCKS: &str =
    "SELECT 'source_blocks' AS which, count(*) AS tally FROM source_blocks";
const TALLY_SOURCE_TABLE_ROWS: &str =
    "SELECT 'source_table_rows' AS which, count(*) AS tally FROM source_table_rows";
const TALLY_SUITES: &str = "SELECT 'suites' AS which, count(*) AS tally FROM suites";
const TALLY_NODES: &str = "SELECT 'nodes' AS which, count(*) AS tally FROM nodes";
const TALLY_NODE_ALIASES: &str =
    "SELECT 'node_aliases' AS which, count(*) AS tally FROM node_aliases";
const TALLY_NODE_HISTORY: &str =
    "SELECT 'node_history' AS which, count(*) AS tally FROM node_history";
const TALLY_RELATIONS: &str = "SELECT 'relations' AS which, count(*) AS tally FROM relations";
const TALLY_RELATION_TYPES: &str =
    "SELECT 'relation_types' AS which, count(*) AS tally FROM relation_types";
const TALLY_NORMATIVE_STATEMENTS: &str =
    "SELECT 'normative_statements' AS which, count(*) AS tally FROM normative_statements";
const TALLY_LINEAGE: &str = "SELECT 'lineage' AS which, count(*) AS tally FROM lineage";
const TALLY_OMISSIONS: &str = "SELECT 'omissions' AS which, count(*) AS tally FROM omissions";
const TALLY_RECORD_FRONT_MATTER: &str =
    "SELECT 'record_front_matter' AS which, count(*) AS tally FROM record_front_matter";
const TALLY_RECORD_RELATIONS: &str =
    "SELECT 'record_relations' AS which, count(*) AS tally FROM record_relations";
const TALLY_SUBMISSIONS: &str = "SELECT 'submissions' AS which, count(*) AS tally FROM submissions";
const TALLY_SUBMISSION_VALUES: &str =
    "SELECT 'submission_values' AS which, count(*) AS tally FROM submission_values";
const TALLY_SUBMISSION_GAPS: &str =
    "SELECT 'submission_gaps' AS which, count(*) AS tally FROM submission_gaps";

/// Every row a mapped query produced, or the first failure it hit.
pub(crate) fn Collected<T>(
    rows: impl Iterator<Item = rusqlite::Result<T>>,
) -> Result<Vec<T>, StoreError>
{
    let mut collected = Vec::new();
    for row in rows
    {
        collected.push(row?);
    }

    return Ok(collected);
}

pub(crate) fn Write_Blob(connection: &Connection, content: &[u8]) -> Result<i64, StoreError>
{
    let digest = ContentHash::Of_Bytes(content);
    let existing: Option<i64> = connection
        .query_row(
            "SELECT uid FROM blobs WHERE sha256 = ?1",
            params![digest.As_Str()],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(uid) = existing
    {
        return Ok(uid);
    }
    connection.execute(
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        params![
            digest.As_Str(),
            i64::try_from(content.len()).unwrap_or(i64::MAX),
            content
        ],
    )?;

    return Ok(connection.last_insert_rowid());
}

/// Writes a document and its bytes through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Document(
    connection: &Connection,
    path: &str,
    revision: &str,
    content: &str,
) -> Result<i64, StoreError>
{
    let blob_uid = Write_Blob(connection, content.as_bytes())?;

    connection.execute(
        "INSERT OR IGNORE INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
        params![path, revision, blob_uid],
    )?;

    return Ok(connection.query_row(
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        params![path, revision],
        |row| row.get(0),
    )?);
}

/// The columns a node carries, as one value.
///
/// Grouped because they only mean anything together: an identifier without the authority
/// that speaks for it says nothing about whether a later writer may overwrite it, and five
/// bare strings in a fixed order is a shape a caller gets wrong silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRow<'a>
{
    pub node_id: &'a str,
    pub kind: &'a str,
    pub authority: &'a str,
    pub representation: &'a str,
    pub title: &'a str,
}

/// Writes a node, upgrading a placeholder but never overwriting a real one, through a
/// caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Node(connection: &Connection, node: NodeRow<'_>) -> Result<i64, StoreError>
{
    let NodeRow {
        node_id,
        kind,
        authority,
        representation,
        title,
    } = node;
    connection.execute(
        "INSERT INTO nodes (node_id, kind, authority, representation, title)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
             kind = excluded.kind,
             authority = excluded.authority,
             representation = excluded.representation,
             title = excluded.title
         WHERE nodes.authority = ?6",
        params![node_id, kind, authority, representation, title, EXTERNAL],
    )?;

    return Ok(connection.query_row(
        "SELECT uid FROM nodes WHERE node_id = ?1",
        params![node_id],
        |row| row.get(0),
    )?);
}

/// Records an edge and its inverse through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Relation(
    connection: &Connection,
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    connection.execute(
        "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
         SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
         WHERE f.node_id = ?1 AND t.node_id = ?3",
        params![from_node_id, relation_type, to_node_id],
    )?;

    if let Some(inverse) = Inverse_Of(connection, relation_type)?
    {
        connection.execute(
            "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
             WHERE f.node_id = ?1 AND t.node_id = ?3",
            params![to_node_id, inverse, from_node_id],
        )?;
    }

    return Ok(());
}

/// The inverse a relation type declares, if it declares one.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Inverse_Of(
    connection: &Connection,
    relation_type: &str,
) -> Result<Option<String>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT inverse_of FROM relation_types WHERE name = ?1",
            params![relation_type],
            |row| row.get(0),
        )
        .optional()?
        .flatten());
}

impl SpecificationStore
{
    /// The surrogate for one table row, addressed the way a person addresses one.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Table_Row_Uid(
        &self,
        document_uid: i64,
        block_ordinal: u32,
        row_ordinal: u32,
    ) -> Result<Option<i64>, StoreError>
    {
        return Ok(self
            .connection
            .query_row(
                "SELECT r.uid FROM source_table_rows r
                 JOIN source_blocks b ON b.uid = r.source_block_uid
                 WHERE b.document_uid = ?1 AND b.ordinal = ?2 AND r.ordinal = ?3",
                params![document_uid, block_ordinal, row_ordinal],
                |row| row.get(0),
            )
            .optional()?);
    }

    /// Records what became of one table row.
    ///
    /// Separate from a block disposition rather than a nullable extra argument on it,
    /// because the two answer different questions: a block disposition says the table
    /// survived, and this says which row a given node came out of. Restoration needs the
    /// second — thirty concepts all pointing at one block is not a lineage.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Row_Lineage(
        &mut self,
        row_uid: i64,
        disposition: &str,
        target_node_uid: Option<i64>,
    ) -> Result<(), StoreError>
    {
        self.connection.execute(
            "INSERT OR IGNORE INTO lineage (source_table_row_uid, disposition, target_node_uid)
             VALUES (?1, ?2, ?3)",
            params![row_uid, disposition, target_node_uid],
        )?;

        return Ok(());
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Row_Census(&self, scope: RowScope) -> Result<RowCensus, StoreError>
    {
        return crate::rows::Census(&self.connection, scope);
    }

    /// Writes a node, upgrading a placeholder but never overwriting a real one.
    ///
    /// A relation whose target has not been ingested yet needs somewhere to point, so
    /// [`Self::Reference_Node`] creates one at authority [`EXTERNAL`]. When the real
    /// record arrives it fills that row in. Any other node is left alone: the first
    /// writer of a real node is its author, and a later pass must not quietly restate it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Upsert_Node(&mut self, node: NodeRow<'_>) -> Result<i64, StoreError>
    {
        return Write_Node(&self.connection, node);
    }

    /// Records a suite and whether it is this repository's own.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Suite(
        &mut self,
        suite_id: &str,
        title: &str,
        authority: SuiteAuthority,
    ) -> Result<i64, StoreError>
    {
        self.connection.execute(
            "INSERT INTO suites (suite_id, title, authority_root) VALUES (?1, ?2, ?3)
             ON CONFLICT(suite_id) DO UPDATE SET title = excluded.title,
                                                 authority_root = excluded.authority_root",
            params![suite_id, title, authority.Stored()],
        )?;

        return Ok(self.connection.query_row(
            "SELECT uid FROM suites WHERE suite_id = ?1",
            params![suite_id],
            |row| row.get(0),
        )?);
    }

    /// Places a node in a suite.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Assign_Suite(&mut self, node_uid: i64, suite_uid: i64) -> Result<(), StoreError>
    {
        self.connection.execute(
            "UPDATE nodes SET suite_uid = ?2 WHERE uid = ?1",
            params![node_uid, suite_uid],
        )?;

        return Ok(());
    }

    /// The suite a node belongs to and whether that suite is the root.
    ///
    /// `None` where no suite is recorded, which is not the same as the root and must not
    /// read as it. A node nothing placed is a node whose ownership was never established.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Suite_Of(&self, node_id: &str) -> Result<Option<(String, SuiteAuthority)>, StoreError>
    {
        return Ok(self
            .connection
            .query_row(
                "SELECT s.suite_id, s.authority_root FROM nodes n
                 JOIN suites s ON s.uid = n.suite_uid
                 WHERE n.node_id = ?1",
                params![node_id],
                |row| {
                    let suite: String = row.get(0)?;
                    let root: i64 = row.get(1)?;
                    return Ok((suite, SuiteAuthority::Read(root)));
                },
            )
            .optional()?);
    }

    /// A node that exists only because something points at it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Reference_Node(&mut self, node_id: &str) -> Result<i64, StoreError>
    {
        return self.Upsert_Node(NodeRow {
            node_id,
            kind: "unknown",
            authority: EXTERNAL,
            representation: "record",
            title: node_id,
        });
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Node_Uid(&self, node_id: &str) -> Result<Option<i64>, StoreError>
    {
        return Ok(self
            .connection
            .query_row(
                "SELECT uid FROM nodes WHERE node_id = ?1",
                params![node_id],
                |row| row.get(0),
            )
            .optional()?);
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Relation_Type(&mut self, name: &str, tier: &str) -> Result<(), StoreError>
    {
        self.connection.execute(
            "INSERT OR IGNORE INTO relation_types (name, tier) VALUES (?1, ?2)",
            params![name, tier],
        )?;
        return Ok(());
    }

    /// Pairs a relation type with its inverse.
    ///
    /// Separate from [`Self::Put_Relation_Type`] because `inverse_of` points back into
    /// the same table, so no single insertion order satisfies it — the first of any pair
    /// names a row that does not exist yet. Naming both halves first and pairing them
    /// afterwards keeps the foreign key enforced the whole way through, rather than
    /// deferring it and finding out at commit.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] if either name is not a relation type.
    pub fn Pair_Relation_Type(&mut self, name: &str, inverse: &str) -> Result<(), StoreError>
    {
        let changed = self.connection.execute(
            "UPDATE relation_types SET inverse_of = ?2 WHERE name = ?1",
            params![name, inverse],
        )?;

        if changed == 0
        {
            return Err(StoreError::Sql(format!(
                "no relation type named {name}, so it cannot be paired with {inverse}"
            )));
        }

        return Ok(());
    }

    /// Records an edge and its inverse, so `verifies` and `verified_by` are one fact.
    ///
    /// Both endpoints must already exist as nodes; use [`Self::Reference_Node`] for a
    /// target that has not been ingested yet.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Relation(
        &mut self,
        from_node_id: &str,
        relation_type: &str,
        to_node_id: &str,
    ) -> Result<(), StoreError>
    {
        return Write_Relation(&self.connection, from_node_id, relation_type, to_node_id);
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Count(&self, table: Table) -> Result<u32, StoreError>
    {
        return Ok(self
            .connection
            .query_row(table.Tally_Sql(), [], |row| row.get(1))?);
    }

    #[must_use]
    pub fn Connection(&self) -> &Connection
    {
        return &self.connection;
    }
}

/// The tables a caller may count.
///
/// An enum rather than a string, so `Count` cannot become a hole through which
/// arbitrary SQL reaches the database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Table
{
    Blobs,
    SourceDocuments,
    SourceHeadings,
    SourceBlocks,
    SourceTableRows,
    Suites,
    Nodes,
    NodeAliases,
    NodeHistory,
    Relations,
    RelationTypes,
    NormativeStatements,
    Lineage,
    Omissions,
    RecordFrontMatter,
    RecordRelations,
    Submissions,
    SubmissionValues,
    SubmissionGaps,
}

impl Table
{
    #[must_use]
    pub const fn Name(self) -> &'static str
    {
        return match self
        {
            Self::Blobs => "blobs",
            Self::SourceDocuments => "source_documents",
            Self::SourceHeadings => "source_headings",
            Self::SourceBlocks => "source_blocks",
            Self::SourceTableRows => "source_table_rows",
            Self::Suites => "suites",
            Self::Nodes => "nodes",
            Self::NodeAliases => "node_aliases",
            Self::NodeHistory => "node_history",
            Self::Relations => "relations",
            Self::RelationTypes => "relation_types",
            Self::NormativeStatements => "normative_statements",
            Self::Lineage => "lineage",
            Self::Omissions => "omissions",
            Self::RecordFrontMatter => "record_front_matter",
            Self::RecordRelations => "record_relations",
            Self::Submissions => "submissions",
            Self::SubmissionValues => "submission_values",
            Self::SubmissionGaps => "submission_gaps",
        };
    }

    /// The statement counting this table's rows, labelled with the table's own name.
    ///
    /// Written out per variant rather than interpolated from [`Table::Name`] at the call
    /// site. A table identifier occupies no value position, so no driver can bind one and
    /// there is no parameterized form to prefer; the only way to keep the statement out of
    /// runtime string building is for each variant to carry its own. The label is selected
    /// rather than assumed from row order, because a compound `SELECT` without `ORDER BY`
    /// is not promised to come back in the order its arms were written.
    #[must_use]
    pub const fn Tally_Sql(self) -> &'static str
    {
        return match self
        {
            Self::Blobs => TALLY_BLOBS,
            Self::SourceDocuments => TALLY_SOURCE_DOCUMENTS,
            Self::SourceHeadings => TALLY_SOURCE_HEADINGS,
            Self::SourceBlocks => TALLY_SOURCE_BLOCKS,
            Self::SourceTableRows => TALLY_SOURCE_TABLE_ROWS,
            Self::Suites => TALLY_SUITES,
            Self::Nodes => TALLY_NODES,
            Self::NodeAliases => TALLY_NODE_ALIASES,
            Self::NodeHistory => TALLY_NODE_HISTORY,
            Self::Relations => TALLY_RELATIONS,
            Self::RelationTypes => TALLY_RELATION_TYPES,
            Self::NormativeStatements => TALLY_NORMATIVE_STATEMENTS,
            Self::Lineage => TALLY_LINEAGE,
            Self::Omissions => TALLY_OMISSIONS,
            Self::RecordFrontMatter => TALLY_RECORD_FRONT_MATTER,
            Self::RecordRelations => TALLY_RECORD_RELATIONS,
            Self::Submissions => TALLY_SUBMISSIONS,
            Self::SubmissionValues => TALLY_SUBMISSION_VALUES,
            Self::SubmissionGaps => TALLY_SUBMISSION_GAPS,
        };
    }

    /// Every table in the schema — a list kept beside the enum, which the compiler does
    /// not check against the schema it claims to enumerate.
    ///
    /// Mirrored by `Test_Every_Table_In_The_Schema_Should_Be_Declared`.
    #[must_use]
    pub const fn All() -> &'static [Self]
    {
        return &[
            Self::Blobs,
            Self::SourceDocuments,
            Self::SourceHeadings,
            Self::SourceBlocks,
            Self::SourceTableRows,
            Self::Suites,
            Self::Nodes,
            Self::NodeAliases,
            Self::NodeHistory,
            Self::Relations,
            Self::RelationTypes,
            Self::NormativeStatements,
            Self::Lineage,
            Self::Omissions,
            Self::RecordFrontMatter,
            Self::RecordRelations,
            Self::Submissions,
            Self::SubmissionValues,
            Self::SubmissionGaps,
        ];
    }
}
