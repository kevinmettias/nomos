//! The specification store: one `SQLite` database, opened, migrated and queried.

// Seeding this build's own governing records into a store. It sits here rather than under
// `registration` because the build script reaches registration.rs by #[path] and compiles
// it with no crate around it, so a child of registration cannot name `crate::`.
mod governing;

pub use governing::{GOVERNING_RECORD_IDS, Seed_Governing_Records, SeedReport};

pub(crate) mod error;

mod write;

pub(crate) use write::{
    Collected, Inverse_Of, Write_Blob, Write_Node, Write_Relation, Write_Source_Blocks,
    Write_Source_Document,
};

use crate::NodeRow;
use crate::RowCensus;
use crate::RowScope;
use crate::StoreError;
use crate::SuiteAuthority;
use crate::Table;
use crate::{Latest_Version, MIGRATIONS, Migration};
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
        return crate::table::rows::Census(&self.connection, scope);
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

    /// Registers a relation type together with what it constrains: the node kinds it may
    /// join at each end, and how many edges of it one node may carry.
    ///
    /// `OD-SPEC-012`: a relation type that declares neither is not a lighter-weight
    /// registration, it is the absence of the thing this function exists to record — so
    /// `domain`, `range` and `max_per_node` are required rather than defaulted, and an empty
    /// declaration is refused here rather than admitted and left permissive downstream.
    ///
    /// Idempotent like the rest of seeding: registering the same name twice keeps the first
    /// declaration rather than erroring or silently overwriting it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::UnconstrainedRelationType`] if `domain`, `range` or
    /// `max_per_node` is empty or zero, and [`StoreError`] on any SQL failure.
    pub fn Put_Relation_Type(
        &mut self,
        name: &str,
        tier: &str,
        domain: &[&str],
        range: &[&str],
        max_per_node: u32,
    ) -> Result<(), StoreError>
    {
        if domain.is_empty() || range.is_empty() || max_per_node == 0
        {
            return Err(StoreError::UnconstrainedRelationType { name: name.to_owned() });
        }

        let mut domain_sorted: Vec<&str> = domain.to_vec();
        domain_sorted.sort_unstable();
        let mut range_sorted: Vec<&str> = range.to_vec();
        range_sorted.sort_unstable();

        // Sorted before serializing so the same set of kinds always writes the same bytes,
        // regardless of the order a caller happened to list them in — the bundle's byte-
        // identical round trip depends on it exactly the way it depends on every other
        // ordering in this store being by natural key rather than by insertion order.
        let domain_json = serde_json::to_string(&domain_sorted)
            .map_err(|error| return StoreError::Sql(error.to_string()))?;
        let range_json = serde_json::to_string(&range_sorted)
            .map_err(|error| return StoreError::Sql(error.to_string()))?;

        self.connection.execute(
            "INSERT OR IGNORE INTO relation_types
                 (name, tier, domain_kinds_json, range_kinds_json, max_per_node)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, tier, domain_json, range_json, max_per_node],
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

#[cfg(test)]
mod tests
{
    use super::*;

    fn Node(store: &mut SpecificationStore, node_id: &str, kind: &str)
    {
        store
            .Upsert_Node(NodeRow {
                node_id,
                kind,
                authority: AUTHORED,
                representation: "record",
                title: node_id,
            })
            .expect("mints a node");
    }

    /// `OD-SPEC-012`: a relation type declaring nothing is refused where it is registered,
    /// not left to write an edge that later discovers there was nothing to check.
    #[test]
    fn Test_Registering_A_Relation_Type_With_No_Domain_Should_Be_Refused()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let refusal = store.Put_Relation_Type("nothing", "seed", &[], &["widget"], 1);
        let message = format!("{refusal:?}");

        assert!(
            matches!(refusal, Err(StoreError::UnconstrainedRelationType { ref name }) if name == "nothing"),
            "an empty domain must be refused at registration: {message}"
        );
    }

    #[test]
    fn Test_Registering_A_Relation_Type_With_Zero_Cardinality_Should_Be_Refused()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let refusal = store.Put_Relation_Type("nothing", "seed", &["widget"], &["widget"], 0);

        assert!(
            matches!(refusal, Err(StoreError::UnconstrainedRelationType { .. })),
            "a zero cardinality must be refused at registration: {refusal:?}"
        );
    }

    /// The refusal names the type, the endpoint that failed, its kind, and the role it
    /// failed — everything `done_when` asks a domain/range refusal to say.
    #[test]
    fn Test_An_Edge_Whose_Endpoint_Kind_Is_Not_Admitted_Should_Be_Refused_By_Name()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Node(&mut store, "A", "widget");
        Node(&mut store, "B", "gadget");
        store.Put_Relation_Type("joins", "seed", &["widget"], &["widget"], 4).expect("registers");

        let error = store.Put_Relation("A", "joins", "B").expect_err("B is a gadget, not a widget");

        let StoreError::RelationEndpoint { relation_type, role, node_id, kind, admits } = error
        else
        {
            panic!("wrong variant: {error:?}");
        };
        assert_eq!(relation_type, "joins", "which type");
        assert_eq!(role, "range", "which end");
        assert_eq!(node_id, "B", "which endpoint");
        assert_eq!(kind, "gadget", "what it is");
        assert_eq!(admits, vec!["widget".to_owned()], "what would satisfy it");
    }

    /// The refusal names the type, the node that is full, and the cap it declared.
    #[test]
    fn Test_An_Edge_Past_The_Declared_Cardinality_Should_Be_Refused_By_Name()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Node(&mut store, "A", "widget");
        Node(&mut store, "B", "widget");
        Node(&mut store, "C", "widget");
        store.Put_Relation_Type("joins", "seed", &["widget"], &["widget"], 1).expect("registers");
        store.Put_Relation("A", "joins", "B").expect("the first edge fits the cap of 1");

        let error =
            store.Put_Relation("A", "joins", "C").expect_err("a second edge exceeds the cap of 1");

        let StoreError::RelationCardinality { relation_type, node_id, max_per_node } = error
        else
        {
            panic!("wrong variant: {error:?}");
        };
        assert_eq!(relation_type, "joins", "which type");
        assert_eq!(node_id, "A", "which node is full");
        assert_eq!(max_per_node, 1, "what cap it declared");
    }

    /// Re-ingesting the same edge is idempotent, the same property `INSERT OR IGNORE`
    /// already gives every other edge — it must not count a second time against the cap.
    #[test]
    fn Test_Re_Writing_The_Same_Edge_Should_Not_Count_Against_Cardinality()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Node(&mut store, "A", "widget");
        Node(&mut store, "B", "widget");
        store.Put_Relation_Type("joins", "seed", &["widget"], &["widget"], 1).expect("registers");
        store.Put_Relation("A", "joins", "B").expect("first write");

        store.Put_Relation("A", "joins", "B").expect("an idempotent re-write must not refuse");
    }

    /// A placeholder minted by [`SpecificationStore::Reference_Node`] carries no real kind
    /// yet, so the domain/range check defers rather than judging it against the sentinel.
    #[test]
    fn Test_A_Placeholder_Endpoint_Should_Be_Exempt_From_The_Kind_Check()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Node(&mut store, "A", "widget");
        store.Reference_Node("B").expect("mints a placeholder");
        store.Put_Relation_Type("joins", "seed", &["widget"], &["widget"], 4).expect("registers");

        store
            .Put_Relation("A", "joins", "B")
            .expect("a placeholder endpoint is exempt from the range check");
    }
}
