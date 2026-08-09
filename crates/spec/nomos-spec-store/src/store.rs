use crate::rows::{RowCensus, RowScope};
use crate::schema::{Latest_Version, MIGRATIONS};
use nomos_spec_model::{ContentHash, SourceBlock, Table_Defects, Table_Rows};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// The revision label for content this repository authors itself.
///
/// Distinct from an ingested corpus revision like `v14.36`, so a query can tell what
/// Nomos said about itself from what it read out of an archive.
pub const AUTHORED: &str = "authored";

/// The authority of a node that exists only because something points at it.
pub const EXTERNAL: &str = "external";

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

impl std::error::Error for StoreError {}

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
            let transaction = self.connection.transaction()?;
            for statement in migration.statements
            {
                transaction
                    .execute_batch(statement)
                    .map_err(|error| StoreError::Migration {
                        from: current,
                        cause: format!("{}: {error}", migration.name),
                    })?;
            }
            transaction.pragma_update(None, "user_version", migration.version)?;
            transaction.commit()?;
        }

        return Ok(());
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
        let digest = ContentHash::Of_Bytes(content);
        let existing: Option<i64> = self
            .connection
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

        self.connection.execute(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
            params![
                digest.As_Str(),
                i64::try_from(content.len()).unwrap_or(i64::MAX),
                content
            ],
        )?;

        return Ok(self.connection.last_insert_rowid());
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
        let blob_uid = self.Put_Blob(content.as_bytes())?;

        self.connection.execute(
            "INSERT OR IGNORE INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
            params![path, revision, blob_uid],
        )?;

        return Ok(self.connection.query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            params![path, revision],
            |row| row.get(0),
        )?);
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

        let transaction = self.connection.transaction()?;
        {
            // Not `INSERT OR REPLACE`. REPLACE deletes the conflicting row and inserts a
            // new one, which hands the block a new `uid` — and `uid` is what every
            // lineage and omission row points at. Re-ingesting a document would silently
            // renumber its blocks and take their dispositions with them.
            let mut insert = transaction.prepare(
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

            let mut block_uid = transaction.prepare(
                "SELECT uid FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
            )?;
            // Same reasoning as the blocks above: update in place rather than REPLACE, so
            // a re-ingest does not hand a row a new uid.
            let mut insert_row = transaction.prepare(
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
                if rows.is_empty()
                {
                    continue;
                }

                let uid: i64 =
                    block_uid.query_row(params![document_uid, block.ordinal], |row| row.get(0))?;

                for row in &rows
                {
                    let cells = serde_json::to_string(&row.cells)
                        .map_err(|error| StoreError::Sql(error.to_string()))?;
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
            }
        }
        transaction.commit()?;

        return Ok(blocks.len());
    }

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
    pub fn Upsert_Node(
        &mut self,
        node_id: &str,
        kind: &str,
        authority: &str,
        representation: &str,
        title: &str,
    ) -> Result<i64, StoreError>
    {
        self.connection.execute(
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

        return Ok(self.connection.query_row(
            "SELECT uid FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| row.get(0),
        )?);
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
        authority_root: bool,
    ) -> Result<i64, StoreError>
    {
        self.connection.execute(
            "INSERT INTO suites (suite_id, title, authority_root) VALUES (?1, ?2, ?3)
             ON CONFLICT(suite_id) DO UPDATE SET title = excluded.title,
                                                 authority_root = excluded.authority_root",
            params![suite_id, title, i64::from(authority_root)],
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
    pub fn Suite_Of(&self, node_id: &str) -> Result<Option<(String, bool)>, StoreError>
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
                    return Ok((suite, root != 0));
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
        return self.Upsert_Node(node_id, "unknown", EXTERNAL, "record", node_id);
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
        self.connection.execute(
            "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
             SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
             WHERE f.node_id = ?1 AND t.node_id = ?3",
            params![from_node_id, relation_type, to_node_id],
        )?;

        let inverse: Option<String> = self
            .connection
            .query_row(
                "SELECT inverse_of FROM relation_types WHERE name = ?1",
                params![relation_type],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        if let Some(inverse) = inverse
        {
            self.connection.execute(
                "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
                 SELECT f.uid, ?2, t.uid FROM nodes f, nodes t
                 WHERE f.node_id = ?1 AND t.node_id = ?3",
                params![to_node_id, inverse, from_node_id],
            )?;
        }

        return Ok(());
    }

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Count(&self, table: Table) -> Result<u32, StoreError>
    {
        let sql = format!("SELECT count(*) FROM {}", table.Name());
        return Ok(self.connection.query_row(&sql, [], |row| row.get(0))?);
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
        };
    }

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
        ];
    }
}
