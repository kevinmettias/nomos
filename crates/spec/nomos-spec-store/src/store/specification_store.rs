// file-size: allow this file pairs its production code with its own inline #[cfg(test)]
// module; check-test-coverage keys a test's companion unit off the exact file it is
// textually written in, so these tests cannot move to a sibling file without losing
// their attribution to every function this file declares.
// responsibility: allow same reason -- the coupling that keeps this file whole is
// check-test-coverage's stem-based companion attribution, not a design choice.

use crate::DocumentPath;
use crate::DocumentRevision;
use crate::NodeRow;
use crate::RowCensus;
use crate::RowScope;
use crate::StoreError;
use crate::SuiteAuthority;
use crate::Table;
use crate::{SuiteId, SuiteTitle};
use crate::{Latest_Version, MIGRATIONS, Migration};
use nomos_spec_model::SourceBlock;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

use super::relation;
use super::{Write_Blob, Write_Node, Write_Relation, Write_Source_Blocks, Write_Source_Document};

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
    pub fn In_Transaction<Value, Error>(
        &mut self,
        work: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<Value, Error>,
    ) -> Result<Value, Error>
    where
        Error: From<StoreError>,
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
    pub fn Put_Source_Document<'a>(
        &mut self,
        path: impl Into<DocumentPath<'a>>,
        revision: impl Into<DocumentRevision<'a>>,
        content: &str,
    ) -> Result<i64, StoreError>
    {
        return Write_Source_Document(&self.connection, path.into(), revision.into(), content);
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

        for candidate in MIGRATIONS.iter().filter(|candidate| candidate.version > current)
        {
            self.Apply(candidate, current)?;
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
        return crate::table::rows::Counted_Rows(&self.connection, scope);
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
    pub fn Put_Suite<'a>(
        &mut self,
        suite_id: impl Into<SuiteId<'a>>,
        title: impl Into<SuiteTitle<'a>>,
        authority: SuiteAuthority,
    ) -> Result<i64, StoreError>
    {
        let suite_id = suite_id.into().0;
        let title = title.into().0;

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
        use super::EXTERNAL;

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
    /// `constraint`'s domain, range and cardinality are required rather than defaulted, and
    /// an empty declaration is refused here rather than admitted and left permissive
    /// downstream.
    ///
    /// Idempotent like the rest of seeding: registering the same name twice keeps the first
    /// declaration rather than erroring or silently overwriting it.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError::UnconstrainedRelationType`] if the constraint's domain, range
    /// or `max_per_node` is empty or zero, and [`StoreError`] on any SQL failure.
    pub fn Put_Relation_Type<'a>(
        &mut self,
        name: impl Into<relation::TypeName<'a>>,
        tier: impl Into<relation::Tier<'a>>,
        constraint: &relation::Constraint<'_>,
    ) -> Result<(), StoreError>
    {
        let name = name.into().0;
        let tier = tier.into().0;

        relation::Assert_Constraint_Is_Declared(name, constraint)?;

        let domain_json = relation::Sorted_Kinds_Json(constraint.domain)?;
        let range_json = relation::Sorted_Kinds_Json(constraint.range)?;

        self.connection.execute(
            "INSERT OR IGNORE INTO relation_types
                 (name, tier, domain_kinds_json, range_kinds_json, max_per_node)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, tier, domain_json, range_json, constraint.max_per_node],
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
    pub fn Pair_Relation_Type<'a>(
        &mut self,
        name: impl Into<relation::TypeName<'a>>,
        inverse: impl Into<relation::InverseRelationType<'a>>,
    ) -> Result<(), StoreError>
    {
        let name = name.into().0;
        let inverse = inverse.into().0;

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
    pub fn Put_Relation<'a>(
        &mut self,
        from_node_id: impl Into<relation::FromNodeId<'a>>,
        relation_type: impl Into<relation::TypeName<'a>>,
        to_node_id: impl Into<relation::ToNodeId<'a>>,
    ) -> Result<(), StoreError>
    {
        return Write_Relation(&self.connection, from_node_id.into(), relation_type.into(), to_node_id.into());
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

    /// This store's own SQLite handle, for a caller that writes its own SQL.
    ///
    /// This is the widest escape in the workspace and it is deliberate, not an oversight.
    /// `OD-SPEC-016` measured it: 107 production calls outside this crate, writing 159
    /// distinct statements over 22 tables, from `nomos-spec-bundle`, `nomos-spec-ingest`,
    /// `nomos-spec-project` and `nomos-spec-validate`.
    ///
    /// # Why it is not sealed behind named operations
    ///
    /// Because that is 159 methods against a public surface that is 266 lines today, and
    /// more than half of them would be `nomos-spec-bundle`'s schema traversal re-typed as an
    /// API. That crate's contract is "every column of the store reaches the bundle"; an API
    /// restating it would be a second authority over the schema, not a seam over it.
    ///
    /// # What this actually costs, which is not what the call count suggests
    ///
    /// All 107 sites reach this handle for five operations and no others -- `prepare`,
    /// `execute`, `query_row`, `execute_batch`, `last_insert_rowid` -- so the number of
    /// callers is not what stands between here and the store-backend equivalence test
    /// `OD-SPEC-001` names. 151 of the 159 statements are portable SQL that any backend
    /// carrying this schema would run unchanged. What is not portable is that these five
    /// operations are spelled on a concrete type: 84 `rusqlite::` mentions across those four
    /// crates, plus 8 statements in SQLite-only syntax.
    ///
    /// `OD-SPEC-016` records the three steps that would close it and the order they have to
    /// be paid in. It waits on `OD-SPEC-001`'s second backend, because that backend is the
    /// only thing the seam would be for.
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
    use crate::{Constraint as RelationConstraint, NodeRow, SuiteAuthority, TableLine};

    #[test]
    fn Test_Open_Should_Persist_Across_A_Reopen()
    {
        let mut path = std::env::temp_dir();
        path.push(format!("nomos-spec-store-open-test-{}.db", std::process::id()));
        // error-info: allow this is a best-effort clean slate before creating the file fresh below
        let _ = std::fs::remove_file(&path);

        let mut first = SpecificationStore::Open(&path).expect("opens");
        first.Put_Blob(b"hello").expect("writes");
        drop(first);

        let second = SpecificationStore::Open(&path).expect("reopens");
        assert_eq!(second.Count(Table::Blobs).expect("counts"), 1);

        // error-info: allow this is best-effort cleanup after the assertions already ran
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn Test_In_Memory_Should_Open_At_The_Latest_Schema_Version()
    {
        let store = SpecificationStore::In_Memory().expect("opens");

        assert_eq!(store.Version(), Latest_Version());
    }

    #[test]
    fn Test_Version_Should_Report_The_Schema_Version_The_Store_Is_At()
    {
        let store = SpecificationStore::In_Memory().expect("opens");

        assert!(store.Version() > 0);
    }

    #[test]
    fn Test_In_Transaction_Should_Roll_Back_Everything_When_Work_Fails()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let outcome: Result<(), StoreError> = store.In_Transaction(|transaction| {
            transaction.execute(
                "INSERT INTO nodes (node_id, kind, authority, representation, title)
                 VALUES ('N-1', 'requirement', 'canonical-normative-record', 'record', 'a node')",
                [],
            )?;

            return Err(StoreError::Sql("deliberate failure".to_owned()));
        });

        assert!(outcome.is_err());
        assert_eq!(
            store.Count(Table::Nodes).expect("counts"),
            0,
            "the insert must not survive the rollback"
        );
    }

    #[test]
    fn Test_Put_Blob_Should_Store_The_Same_Bytes_Once()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let first = store.Put_Blob(b"same bytes").expect("writes");
        let second = store.Put_Blob(b"same bytes").expect("writes");

        assert_eq!(first, second);
    }

    #[test]
    fn Test_Put_Source_Document_Should_Record_A_New_Document()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let uid = store.Put_Source_Document("a.md", "v1", "hello").expect("writes");

        assert!(uid > 0);
    }

    #[test]
    fn Test_Put_Source_Blocks_Should_Write_Every_Block()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let markdown = "# Title\n\nOne.\n\n## Section\n\nTwo.\n";
        let uid = store.Put_Source_Document("a.md", "v1", markdown).expect("writes");
        let blocks = nomos_spec_model::Segment(markdown);

        let written = store.Put_Source_Blocks(uid, &blocks).expect("writes");

        assert_eq!(written, blocks.len());
    }

    #[test]
    fn Test_Table_Row_Uid_Should_Address_One_Row_By_Its_Position()
    {
        let StoreWithContentRow { store, uid, content } = A_Store_With_One_Content_Row();

        let row_uid =
            store.Table_Row_Uid(uid, content.block_ordinal, content.row_ordinal).expect("queries");

        assert!(row_uid.is_some());
        assert!(store.Table_Row_Uid(uid, content.block_ordinal, 9999).expect("queries").is_none());
    }

    #[test]
    fn Test_Put_Row_Lineage_Should_Record_What_A_Row_Became()
    {
        let StoreWithContentRow { mut store, uid, content } = A_Store_With_One_Content_Row();
        let row_uid = store
            .Table_Row_Uid(uid, content.block_ordinal, content.row_ordinal)
            .expect("queries")
            .expect("the row exists");

        store.Put_Row_Lineage(row_uid, "preserved-verbatim", None).expect("writes");

        let disposition: String = store
            .Connection()
            .query_row(
                "SELECT disposition FROM lineage WHERE source_table_row_uid = ?1",
                [row_uid],
                |row| return row.get(0),
            )
            .expect("reads");
        assert_eq!(disposition, "preserved-verbatim");
    }

    #[test]
    fn Test_Row_Census_Should_Count_Rows_In_Scope()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";
        let uid = store.Put_Source_Document("a.md", "v1", markdown).expect("writes");
        store
            .Put_Source_Blocks(uid, &nomos_spec_model::Segment(markdown))
            .expect("writes");

        let census = store.Row_Census(RowScope::Everything).expect("counts");

        assert_eq!(census.lines, 3);
    }

    #[test]
    fn Test_Upsert_Node_Should_Insert_A_New_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let uid = store.Upsert_Node(Node("D-1", "decision")).expect("writes");

        assert!(uid > 0);
        assert_eq!(store.Node_Uid("D-1").expect("queries"), Some(uid));
    }

    #[test]
    fn Test_Put_Suite_Should_Record_A_Suite_And_Its_Authority()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let uid = store.Put_Suite("my-suite", "My Suite", SuiteAuthority::Root).expect("writes");
        let again =
            store.Put_Suite("my-suite", "My Suite Renamed", SuiteAuthority::Root).expect("writes");

        assert_eq!(uid, again, "the same suite id must resolve to the same row");
    }

    #[test]
    fn Test_Assign_Suite_Should_Place_A_Node_In_A_Suite()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let node_uid = store.Upsert_Node(Node("D-1", "decision")).expect("mints");
        let suite_uid =
            store.Put_Suite("my-suite", "My Suite", SuiteAuthority::Root).expect("writes");

        store.Assign_Suite(node_uid, suite_uid).expect("assigns");

        let (suite_id, _) = store.Suite_Of("D-1").expect("queries").expect("assigned");
        assert_eq!(suite_id, "my-suite");
    }

    #[test]
    fn Test_Suite_Of_Should_Report_The_Suite_A_Node_Belongs_To()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let node_uid = store.Upsert_Node(Node("D-1", "decision")).expect("mints");
        assert!(store.Suite_Of("D-1").expect("queries").is_none());

        let suite_uid =
            store.Put_Suite("my-suite", "My Suite", SuiteAuthority::Sibling).expect("writes");
        store.Assign_Suite(node_uid, suite_uid).expect("assigns");

        let (suite_id, authority) = store.Suite_Of("D-1").expect("queries").expect("assigned");
        assert_eq!(suite_id, "my-suite");
        assert_eq!(authority, SuiteAuthority::Sibling);
    }

    #[test]
    fn Test_Reference_Node_Should_Mint_A_Placeholder_Node()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        let uid = store.Reference_Node("D-999").expect("mints");

        assert!(uid > 0);
        let summary = store.Node_Summary("D-999").expect("queries").expect("minted");
        assert_eq!(summary.kind, "unknown");
    }

    #[test]
    fn Test_Node_Uid_Should_Look_Up_A_Node_By_Its_Identifier()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        store.Upsert_Node(Node("D-1", "decision")).expect("mints");

        assert!(store.Node_Uid("D-1").expect("queries").is_some());
        assert!(store.Node_Uid("D-999").expect("queries").is_none());
    }

    #[test]
    fn Test_Put_Relation_Type_Should_Register_A_Constrained_Type()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");

        store
            .Put_Relation_Type(
                "relates-to",
                "seed",
                &RelationConstraint { domain: &["widget"], range: &["widget"], max_per_node: 3 },
            )
            .expect("registers");
        // Idempotent: registering the same name twice keeps the first declaration.
        store
            .Put_Relation_Type(
                "relates-to",
                "seed",
                &RelationConstraint { domain: &["widget"], range: &["widget"], max_per_node: 3 },
            )
            .expect("registers again");

        let tier: String = store
            .Connection()
            .query_row(
                "SELECT tier FROM relation_types WHERE name = 'relates-to'",
                [],
                |row| return row.get(0),
            )
            .expect("reads");
        assert_eq!(tier, "seed");
    }

    #[test]
    fn Test_Pair_Relation_Type_Should_Link_A_Type_With_Its_Inverse()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let constraint = RelationConstraint { domain: &["widget"], range: &["widget"], max_per_node: 3 };
        store.Put_Relation_Type("relates-to", "seed", &constraint).expect("registers");
        store.Put_Relation_Type("relates-from", "seed", &constraint).expect("registers");

        store.Pair_Relation_Type("relates-to", "relates-from").expect("pairs");

        let inverse: Option<String> = store
            .Connection()
            .query_row(
                "SELECT inverse_of FROM relation_types WHERE name = 'relates-to'",
                [],
                |row| return row.get(0),
            )
            .expect("reads");
        assert_eq!(inverse, Some("relates-from".to_owned()));

        let missing = store.Pair_Relation_Type("nonexistent", "relates-from");
        assert!(missing.is_err());
    }

    #[test]
    fn Test_Put_Relation_Should_Record_An_Edge_Between_Two_Nodes()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        store.Upsert_Node(Node("A", "widget")).expect("mints");
        store.Upsert_Node(Node("B", "widget")).expect("mints");
        store
            .Put_Relation_Type(
                "relates-to",
                "seed",
                &RelationConstraint { domain: &["widget"], range: &["widget"], max_per_node: 3 },
            )
            .expect("registers");

        store.Put_Relation("A", "relates-to", "B").expect("writes");

        assert_eq!(store.Count(Table::Relations).expect("counts"), 1);
    }

    #[test]
    fn Test_Count_Should_Tally_Rows_In_A_Declared_Table()
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        assert_eq!(store.Count(Table::Blobs).expect("counts"), 0);

        store.Put_Blob(b"hello").expect("writes");

        assert_eq!(store.Count(Table::Blobs).expect("counts"), 1);
    }

    #[test]
    fn Test_Connection_Should_Expose_The_Underlying_Handle()
    {
        let store = SpecificationStore::In_Memory().expect("opens");

        let version: i64 = store
            .Connection()
            .pragma_query_value(None, "user_version", |row| return row.get(0))
            .expect("reads");

        assert_eq!(u32::try_from(version).unwrap_or(0), store.Version());
    }

    /// A store already holding one two-column, one-row table, that row's own `TableLine`, and
    /// the document's uid — named rather than a tuple so a caller is not counting positions.
    struct StoreWithContentRow
    {
        store: SpecificationStore,
        uid: i64,
        content: TableLine,
    }

    /// A store already holding one two-column, one-row table, and that row's own `TableLine` —
    /// the setup every test that addresses a table row by position shares.
    fn A_Store_With_One_Content_Row() -> StoreWithContentRow
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";
        let uid = store.Put_Source_Document("a.md", "v1", markdown).expect("writes");
        store.Put_Source_Blocks(uid, &nomos_spec_model::Segment(markdown)).expect("writes");
        let content = store
            .Table_Lines(uid, None, None)
            .expect("queries")
            .into_iter()
            .find(|line| return line.kind == "content")
            .expect("a content row");

        return StoreWithContentRow { store, uid, content };
    }

    fn Node<'a>(id: &'a str, kind: &'a str) -> NodeRow<'a>
    {
        return NodeRow {
            node_id: id,
            kind,
            authority: "canonical-normative-record",
            representation: "record",
            title: id,
        };
    }
}
