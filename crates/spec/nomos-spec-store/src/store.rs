use crate::schema::{Latest_Version, MIGRATIONS};
use nomos_spec_model::{ContentHash, SourceBlock};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

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
            let mut insert = transaction.prepare(
                "INSERT OR REPLACE INTO source_blocks
                 (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
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
