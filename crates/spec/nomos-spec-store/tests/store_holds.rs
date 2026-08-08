//! P2-STORE acceptance. Every guard has a negative control.

use nomos_spec_model::Segment;
use nomos_spec_store::{Disposition, Latest_Version, SpecificationStore, StoreError, Table};
use rusqlite::params;

fn Temp_Db(name: &str) -> std::path::PathBuf
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-spec-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    return path;
}

const DOCUMENT: &str = "---\nid: X\n---\n# Title\n\nOne.\n\n## Section\n\nTwo.\n";

#[test]
fn Test_A_Fresh_Store_Should_Be_At_The_Latest_Version()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    assert_eq!(store.Version(), Latest_Version());
    assert!(Latest_Version() > 0, "a store with no migrations is not a store");
}

#[test]
fn Test_Every_Declared_Table_Should_Exist()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    for table in Table::All()
    {
        assert_eq!(
            store.Count(*table).unwrap_or_else(|error| panic!("{}: {error}", table.Name())),
            0
        );
    }
}

/// Migration must be idempotent across process restarts, or a second open destroys or
/// duplicates the schema.
#[test]
fn Test_Reopening_Should_Not_Re_Run_Migrations()
{
    let path = Temp_Db("reopen");

    let mut first = SpecificationStore::Open(&path).expect("opens");
    let uid = first.Put_Source_Document("a.md", "v14.36", DOCUMENT).expect("writes");
    assert!(uid > 0);
    drop(first);

    let second = SpecificationStore::Open(&path).expect("reopens");
    assert_eq!(second.Version(), Latest_Version());
    assert_eq!(second.Count(Table::SourceDocuments).expect("counts"), 1);

    let _ = std::fs::remove_file(&path);
}

/// A store written by a newer build must be refused, not read. Reading tables whose
/// meaning may have changed is how a migration bug becomes a data-loss bug.
#[test]
fn Test_A_Future_Schema_Version_Should_Be_Refused()
{
    let path = Temp_Db("future");
    {
        let store = SpecificationStore::Open(&path).expect("opens");
        store
            .Connection()
            .pragma_update(None, "user_version", Latest_Version().saturating_add(1))
            .expect("bumps");
    }

    let result = SpecificationStore::Open(&path);

    assert!(
        matches!(result, Err(StoreError::TooNew { .. })),
        "a newer schema must be refused"
    );

    let _ = std::fs::remove_file(&path);
}

/// The item's stated acceptance: a transaction that fails rolls back every table it
/// touched, not merely the statement that failed.
#[test]
fn Test_A_Failed_Transaction_Should_Roll_Back_Every_Table()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let outcome: Result<(), StoreError> = store.In_Transaction(|transaction| {
        transaction.execute(
            "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'0011')",
            [],
        )?;
        transaction.execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('N-1', 'requirement', 'canonical', 'record', 'a node')",
            [],
        )?;
        // A history row with a blank reason. The schema refuses it, which fails the
        // whole transaction.
        transaction.execute(
            "INSERT INTO node_history (node_uid, ordinal, event, reason, event_hash, recorded_at)
             VALUES (1, 1, 'created', '   ', 'sha256:bb', '2026-08-08')",
            [],
        )?;
        Ok(())
    });

    assert!(outcome.is_err(), "a blank reason must not be accepted");
    assert_eq!(store.Count(Table::Blobs).expect("counts"), 0);
    assert_eq!(store.Count(Table::Nodes).expect("counts"), 0);
    assert_eq!(store.Count(Table::NodeHistory).expect("counts"), 0);
}

/// The negative control for the test above. Without it, an `In_Transaction` that always
/// rolled back would pass.
#[test]
fn Test_A_Successful_Transaction_Should_Commit_Every_Table()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let outcome: Result<(), StoreError> = store.In_Transaction(|transaction| {
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
             VALUES (1, 1, 'created', 'ingested from v14.36', 'sha256:bb', '2026-08-08')",
            [],
        )?;
        Ok(())
    });

    assert!(outcome.is_ok(), "a well-formed transaction must commit");
    assert_eq!(store.Count(Table::Blobs).expect("counts"), 1);
    assert_eq!(store.Count(Table::Nodes).expect("counts"), 1);
    assert_eq!(store.Count(Table::NodeHistory).expect("counts"), 1);
}

/// Content addressing: the same bytes stored twice are one blob.
#[test]
fn Test_Identical_Blobs_Should_Be_Stored_Once()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let first = store.Put_Blob(b"the same bytes").expect("writes");
    let second = store.Put_Blob(b"the same bytes").expect("writes");
    let other = store.Put_Blob(b"different bytes").expect("writes");

    assert_eq!(first, second);
    assert_ne!(first, other);
    assert_eq!(store.Count(Table::Blobs).expect("counts"), 2);
}

/// Re-ingesting a document must be a no-op, or ingest is not restartable.
#[test]
fn Test_Re_Ingesting_A_Document_Should_Be_Idempotent()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let blocks = Segment(DOCUMENT);

    for _ in 0..3
    {
        let uid = store.Put_Source_Document("a.md", "v14.36", DOCUMENT).expect("writes");
        store.Put_Source_Blocks(uid, &blocks).expect("writes blocks");
    }

    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 1);
    assert_eq!(
        store.Count(Table::SourceBlocks).expect("counts"),
        u32::try_from(blocks.len()).expect("small")
    );
}

/// Two revisions of one path are two documents. Collapsing them would make the
/// regression report unable to compare a file against its own past.
#[test]
fn Test_Two_Revisions_Of_One_Path_Should_Be_Distinct()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let old = store.Put_Source_Document("a.md", "v14.36", DOCUMENT).expect("writes");
    let new = store
        .Put_Source_Document("a.md", "v15.0", "# Title\n\nReplaced.\n")
        .expect("writes");

    assert_ne!(old, new);
    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 2);
}

/// Block hashes must survive the round trip through `SQLite` unchanged. If they do not,
/// every preservation rule compares a hash against a corrupted copy of itself.
#[test]
fn Test_Block_Hashes_Should_Survive_Storage()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let blocks = Segment(DOCUMENT);
    let uid = store.Put_Source_Document("a.md", "v14.36", DOCUMENT).expect("writes");
    store.Put_Source_Blocks(uid, &blocks).expect("writes blocks");

    for block in &blocks
    {
        let stored: String = store
            .Connection()
            .query_row(
                "SELECT content_hash FROM source_blocks WHERE document_uid = ?1 AND ordinal = ?2",
                params![uid, block.ordinal],
                |row| row.get(0),
            )
            .expect("reads back");

        assert_eq!(stored, block.Content_Hash().As_Str());
    }
}

/// Foreign keys must be enforced. `SQLite` disables them by default, so a schema full of
/// REFERENCES clauses can be decorative.
#[test]
fn Test_A_Dangling_Reference_Should_Be_Refused()
{
    let store = SpecificationStore::In_Memory().expect("opens");

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
    let store = SpecificationStore::In_Memory().expect("opens");

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
    assert!(!Disposition::RegressionFiller.Preserves_Content());
    assert!(Disposition::PreservedVerbatim.Preserves_Content());
}
