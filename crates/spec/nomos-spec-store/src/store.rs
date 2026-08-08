use crate::schema::{Latest_Version, MIGRATIONS};
use nomos_spec_model::{ContentHash, SourceBlock};
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
    /// An authored record this build embeds does not read.
    Record
    {
        path: String,
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

    /// # Errors
    ///
    /// Returns [`StoreError`] on any SQL failure.
    pub fn Put_Source_Blocks(
        &mut self,
        document_uid: i64,
        blocks: &[SourceBlock],
    ) -> Result<usize, StoreError>
    {
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
        }
        transaction.commit()?;

        return Ok(blocks.len());
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
