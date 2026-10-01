//! P2-STORE acceptance. Every guard has a negative control.

use nomos_spec_model::Segment;
use nomos_spec_store::{Disposition, Latest_Version, SpecificationStore, StoreError, Table};

fn Temporary_Database(name: &str) -> std::path::PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-spec-{name}-{}.db", std::process::id()));
    // A file left behind by an interrupted run is the only failure this removal may report.
    if let Err(cause) = std::fs::remove_file(&path)
    {
        assert_eq!(cause.kind(), std::io::ErrorKind::NotFound, "a leftover database: {cause}");
    }
    return path;
}

const DOCUMENT: &str = "---\nid: X\n---\n# Title\n\nOne.\n\n## Section\n\nTwo.\n";

/// How many tables the schema must report at least, below which the completeness guard below
/// would be comparing two empty lists rather than checking anything.
const SCHEMA_TABLE_FLOOR: usize = 5;

/// The three tables one `Three_Rows` transaction writes: blobs, nodes and node history.
const WRITTEN_TABLE_COUNT: usize = 3;

/// The two distinct byte strings the blob-identity test stores: one repeated, one different.
const DISTINCT_BLOB_COUNT: u32 = 2;

/// How many times the re-ingest test asks for the same document.
const RE_INGEST_ATTEMPTS: u32 = 3;

/// The two revisions of one path that must stay distinct documents.
const REVISIONS_OF_ONE_PATH: u32 = 2;

#[test]
fn Test_A_Fresh_Store_Should_Be_At_The_Latest_Version()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory() applies this crate's schema in process, so it cannot fail to open");

    assert_eq!(store.Version(), Latest_Version());
    assert!(Latest_Version() > 0, "a store with no migrations is not a store");
}

#[test]
fn Test_Every_Declared_Table_Should_Exist()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory() applies this crate's schema in process, so every table is declared");

    for table in Table::All()
    {
        assert_eq!(
            // A `Count` over a declared table errors when the schema has no such table, which
            // is exactly what this loop is looking for. The name has to be in the message:
            // comparing the `Result` itself would report a rusqlite error without saying
            // which of the declared tables the migrations never created.
            store.Count(*table).unwrap_or_else(|error| panic!("{}: {error}", table.Name())),
            0
        );
    }
}

/// The other direction, which is the one that bites.
///
/// `Table::All()` is a list kept beside the enum, and a migration that adds a table
/// without adding it here leaves that table out of every completeness guard built on
/// `All()` — the bundle's row count, its column coverage, the import's landed check — all
/// of which then pass by not looking. Nothing else notices, because each of them is
/// exactly as blind as the list.
#[test]
fn Test_Every_Table_In_The_Schema_Should_Be_Declared()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory() applies this crate's schema in process, so its tables can be asked");
    let present = Tables_In_The_Schema(&store);
    let declared: Vec<&str> = Table::All().iter().map(|table| return table.Name()).collect();
    let undeclared: Vec<&String> = present
        .iter()
        .filter(|name| return !declared.contains(&name.as_str()))
        .collect();

    assert!(
        present.len() > SCHEMA_TABLE_FLOOR,
        "the schema reported almost nothing, so this checked nothing"
    );
    assert!(
        undeclared.is_empty(),
        "in the schema and absent from Table::All(), so every guard built on it is blind \
         to them: {undeclared:?}"
    );
}

/// Every table sqlite says the store has, asked of the database rather than of the code that
/// built it.
fn Tables_In_The_Schema(store: &SpecificationStore) -> Vec<String>
{
    return store
        .Connection()
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads the schema");
}

/// Migration must be idempotent across process restarts, or a second open destroys or
/// duplicates the schema.
#[test]
fn Test_Reopening_Should_Not_Re_Run_Migrations()
{
    let path = Temporary_Database("reopen");

    let mut first = SpecificationStore::Open(&path)
        .expect("Open creates the file and applies the schema at that path");
    let uid = first
        .Put_Source_Document("a.md", "v14.36", DOCUMENT)
        .expect("the path is fresh in this store, so the insert conflicts with nothing");
    assert!(uid > 0);
    drop(first);

    let second =
        SpecificationStore::Open(&path).expect("the file the first handle wrote is still there");
    assert_eq!(second.Version(), Latest_Version());
    assert_eq!(second.Count(Table::SourceDocuments).expect("counts"), 1);
    drop(second);

    std::fs::remove_file(&path)
        .expect("the handle is dropped, so the database this test made is removable");
}

/// A store written by a newer build must be refused, not read. Reading tables whose
/// meaning may have changed is how a migration bug becomes a data-loss bug.
#[test]
fn Test_A_Future_Schema_Version_Should_Be_Refused()
{
    let path = Temporary_Database("future");
    {
        let store = SpecificationStore::Open(&path)
            .expect("Open creates the file whose pragma this test writes");
        store
            .Connection()
            .pragma_update(None, "user_version", Latest_Version().saturating_add(1))
            .expect("the connection belongs to this test, so the pragma is writable");
    }

    let result = SpecificationStore::Open(&path);

    assert!(
        matches!(result, Err(StoreError::TooNew { .. })),
        "a newer schema must be refused"
    );

    std::fs::remove_file(&path)
        .expect("the store was dropped with that block, so the file is removable");
}

/// One transaction writing a blob, a node and a history row, in that order.
///
/// The reason on the history row is the only thing that varies: the schema refuses a blank
/// one, which is how these tests get a transaction to fail at its last statement rather
/// than its first. Rolling back has to reach the two rows written before it.
fn Three_Rows(store: &mut SpecificationStore, reason: &str) -> Result<(), StoreError>
{
    return store.In_Transaction(|transaction| {
        transaction.execute(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'0011')",
            [],
        )?;
        transaction.execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('N-1', 'requirement', 'canonical', 'record', 'a node')",
            [],
        )?;
        transaction.execute(
            "INSERT INTO node_history (node_uid, ordinal, event, reason, event_hash, recorded_at)
             VALUES (1, 1, 'created', ?1, 'sha256:bb', '2026-08-08')",
            [reason],
        )?;

        Ok(())
    });
}

/// What the three tables [`Three_Rows`] writes to hold, read together.
///
/// Together because the claim is about all three at once: a rollback that reached the last
/// statement and not the two before it is the defect, and three separate assertions report
/// it as one table being wrong rather than as the rollback being partial.
fn Counts_Of_Written_Tables(store: &SpecificationStore) -> [u32; WRITTEN_TABLE_COUNT]
{
    return [
        store.Count(Table::Blobs).expect("Count runs a SELECT over a table the schema declares"),
        store
            .Count(Table::Nodes)
            .expect("the nodes table is one the schema declares, so counting it is a query"),
        store
            .Count(Table::NodeHistory)
            .expect("the history table is one the schema declares too"),
    ];
}

/// The item's stated acceptance: a transaction that fails rolls back every table it
/// touched, not merely the statement that failed.
#[test]
fn Test_A_Failed_Transaction_Should_Roll_Back_Every_Table()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the rollback test a store it can write into");

    // The history row carries a blank reason. The schema refuses it, which fails the
    // whole transaction at its last statement rather than its first.
    let outcome = Three_Rows(&mut store, "   ");

    assert!(outcome.is_err(), "a blank reason must not be accepted");
    assert_eq!(Counts_Of_Written_Tables(&store), [0, 0, 0], "the rollback did not reach every table");
}

/// The negative control for the test above. Without it, an `In_Transaction` that always
/// rolled back would pass.
#[test]
fn Test_A_Successful_Transaction_Should_Commit_Every_Table()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the commit test a store it can write into");

    let outcome = Three_Rows(&mut store, "ingested from v14.36");

    assert!(outcome.is_ok(), "a well-formed transaction must commit");
    assert_eq!(Counts_Of_Written_Tables(&store), [1, 1, 1], "the commit did not reach every table");
}

/// Content addressing: the same bytes stored twice are one blob.
#[test]
fn Test_Identical_Blobs_Should_Be_Stored_Once()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the blob-identity test a store it can write into");

    let first = store.Put_Blob(b"the same bytes").expect("Put_Blob takes the first copy of them");
    let second =
        store.Put_Blob(b"the same bytes").expect("Put_Blob takes a repeated copy of the bytes");
    let other =
        store.Put_Blob(b"different bytes").expect("Put_Blob takes the first copy of other bytes");

    assert_eq!(first, second);
    assert_ne!(first, other);
    assert_eq!(store.Count(Table::Blobs).expect("counts"), DISTINCT_BLOB_COUNT);
}

/// Re-ingesting a document must be a no-op, or ingest is not restartable.
#[test]
fn Test_Re_Ingesting_A_Document_Should_Be_Idempotent()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the re-ingest test a store it can write into");
    let blocks = Segment(DOCUMENT);

    for _ in 0..RE_INGEST_ATTEMPTS
    {
        let uid = store
            .Put_Source_Document("a.md", "v14.36", DOCUMENT)
            .expect("Put_Source_Document takes a repeat of a path it already holds");
        store
            .Put_Source_Blocks(uid, &blocks)
            .expect("Put_Source_Blocks takes a repeat of the same blocks");
    }

    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 1);
    assert_eq!(
        store.Count(Table::SourceBlocks).expect("the ingest wrote the blocks of one document"),
        u32::try_from(blocks.len()).expect("the fixture's block count fits the column's width")
    );
}

/// Two revisions of one path are two documents. Collapsing them would make the
/// regression report unable to compare a file against its own past.
#[test]
fn Test_Two_Revisions_Of_One_Path_Should_Be_Distinct()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the revision test a store it can write into");

    let old = store
        .Put_Source_Document("a.md", "v14.36", DOCUMENT)
        .expect("the path is fresh in this store, so the insert conflicts with nothing");
    let new = store
        .Put_Source_Document("a.md", "v15.0", "# Title\n\nReplaced.\n")
        .expect("a second revision of that path is a second document, not a conflict");

    assert_ne!(old, new);
    assert_eq!(
        store.Count(Table::SourceDocuments).expect("counts"),
        REVISIONS_OF_ONE_PATH
    );
}

/// Block hashes must survive the round trip through `SQLite` unchanged. If they do not,
/// every preservation rule compares a hash against a corrupted copy of itself.
#[test]
fn Test_Block_Hashes_Should_Survive_Storage()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the block-hash test a store it can write into");
    let blocks = Segment(DOCUMENT);
    let uid = store
        .Put_Source_Document("a.md", "v14.36", DOCUMENT)
        .expect("Put_Source_Document accepts a path this store has not seen");
    store
        .Put_Source_Blocks(uid, &blocks)
        .expect("Put_Source_Blocks inserts every block it is handed");

    let stored = Stored_Hashes(&store, uid);

    for block in &blocks
    {
        let hash = stored.get(&block.ordinal).expect("every block written has a row");
        assert_eq!(hash, block.Content_Hash().As_String_Slice());
    }
}

/// Every `content_hash` the store holds for `uid`, keyed by the block's own ordinal — read
/// once, so the loop above compares hashes rather than issuing one query per block.
fn Stored_Hashes(store: &SpecificationStore, uid: i64) -> std::collections::HashMap<u32, String>
{
    let mut statement = store
        .Connection()
        .prepare("SELECT ordinal, content_hash FROM source_blocks WHERE document_uid = ?1")
        .expect("the projection names two columns this connection holds");

    return statement
        .query_map([uid], |row| return Ok((row.get(0)?, row.get(1)?)))
        .expect("the query above compiles against the schema the store applies")
        .map(|row| return row.expect("query_map yields one row per source_blocks row"))
        .collect();
}

/// Foreign keys must be enforced. `SQLite` disables them by default, so a schema full of
/// REFERENCES clauses can be decorative.
#[test]
fn Test_A_Dangling_Reference_Should_Be_Refused()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the foreign-key test a store it can write into");

    let result = store.Connection().execute(
        "INSERT INTO source_documents (path, revision, blob_uid) VALUES ('a.md', 'v1', 9999)",
        [],
    );

    assert!(result.is_err(), "foreign keys are not being enforced");
}

/// An omission must carry a reason, a justification and a decision. Without all three it
/// is a deletion with paperwork.
#[test]
fn Test_An_Unjustified_Omission_Should_Be_Refused()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory() gives the omission test a store it can write into");

    let blank = store.Connection().execute(
        "INSERT INTO omissions (source_block_uid, reason, justification, decision_record)
         VALUES (NULL, 'dropped', '  ', 'D-129')",
        [],
    );
    assert!(blank.is_err(), "a blank justification must be refused");

    let targetless = store.Connection().execute(
        "INSERT INTO omissions (reason, justification, decision_record)
         VALUES ('dropped', 'superseded by D-129', 'D-129')",
        [],
    );
    assert!(targetless.is_err(), "an omission must name what was omitted");
}

#[test]
fn Test_Filler_Should_Never_Count_As_Preservation()
{
    assert!(!Disposition::RegressionFiller.Is_Preserving_Content());
    assert!(Disposition::PreservedVerbatim.Is_Preserving_Content());
}
