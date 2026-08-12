//! The specification store: one `SQLite` database, opened, migrated and queried.

pub(crate) mod error;

mod write;

pub(crate) use write::{
    Collected, Inverse_Of, Write_Blob, Write_Node, Write_Relation, Write_Source_Blocks,
    Write_Source_Document,
};

use crate::read::node_row::NodeRow;
use crate::table::row_census::RowCensus;
use crate::table::row_scope::RowScope;
use crate::store::error::StoreError;
use crate::table::suite_authority::SuiteAuthority;
use crate::table::Table;
use crate::schema::{Latest_Version, MIGRATIONS, Migration};
use nomos_spec_model::SourceBlock;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// The revision label for content this repository authors itself.
///
/// Distinct from an ingested corpus revision like `v14.36`, so a query can tell what
/// Nomos said about itself from what it read out of an archive.
pub const AUTHORED: &str = "authored";

/// The authority of a node that exists only because something points at it.
pub const EXTERNAL: &str = "external";

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
