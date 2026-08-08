//! The records governing this system must be in the store before the validator they
//! govern is trusted.

use nomos_spec_store::{
    AUTHORED, EXTERNAL, GOVERNING_RECORD_IDS, Seed_Governing_Records, SpecificationStore, Table,
};

fn Seeded() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    Seed_Governing_Records(&mut store).expect("seeds");
    return store;
}

fn Title(store: &SpecificationStore, node_id: &str) -> Option<String>
{
    return store
        .Connection()
        .query_row(
            "SELECT title FROM nodes WHERE node_id = ?1",
            rusqlite::params![node_id],
            |row| row.get(0),
        )
        .ok();
}

#[test]
fn Test_Every_Governing_Record_Should_Resolve_By_Id()
{
    let store = Seeded();

    let missing: Vec<&str> = GOVERNING_RECORD_IDS
        .iter()
        .filter(|id| store.Node_Uid(id).expect("queries").is_none())
        .copied()
        .collect();

    assert!(missing.is_empty(), "not in the store: {missing:?}");
    assert_eq!(GOVERNING_RECORD_IDS.len(), 8);
}

/// The negative control. Without it the assertion above would pass on a store that
/// contains everything for some other reason.
#[test]
fn Test_An_Unseeded_Store_Should_Hold_None_Of_Them()
{
    let store = SpecificationStore::In_Memory().expect("opens");

    for id in GOVERNING_RECORD_IDS
    {
        assert!(
            store.Node_Uid(id).expect("queries").is_none(),
            "{id} appeared in a store nobody seeded"
        );
    }
}

/// D-129 supersedes ADR-DOC-001. The edge has to be in the store, not only in the prose,
/// or "what superseded this?" is a question only a person reading markdown can answer.
#[test]
fn Test_Adr_Doc_001_Should_Carry_An_Explicit_Supersession_Edge()
{
    let store = Seeded();

    assert!(
        store.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the superseded record must exist for the edge to mean anything"
    );

    let forward: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = 'D-129' AND r.relation_type = 'supersedes'
               AND t.node_id = 'ADR-DOC-001'",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    let inverse: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             WHERE f.node_id = 'ADR-DOC-001' AND r.relation_type = 'superseded_by'
               AND t.node_id = 'D-129'",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(forward, 1, "D-129 does not supersede ADR-DOC-001");
    assert_eq!(inverse, 1, "the inverse edge is missing, so the fact is only half recorded");
}

/// ADR-DOC-001 is in the store by identity, not by content — its prose is restored from
/// the v14 corpus in Phase 3. Recording that as a placeholder rather than inventing a
/// body is the difference between a gap and filler.
#[test]
fn Test_The_Superseded_Record_Should_Be_A_Visible_Placeholder()
{
    let store = Seeded();

    let authority: String = store
        .Connection()
        .query_row(
            "SELECT authority FROM nodes WHERE node_id = 'ADR-DOC-001'",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(authority, EXTERNAL);
    assert_eq!(
        store
            .Connection()
            .query_row(
                "SELECT count(*) FROM source_documents WHERE path LIKE '%ADR-DOC-001%'",
                [],
                |row| row.get::<_, u32>(0)
            )
            .expect("queries"),
        0,
        "a placeholder with a body would be filler wearing a record's clothes"
    );
}

/// A real record arriving later must fill the placeholder in rather than being ignored.
#[test]
fn Test_A_Real_Record_Should_Replace_A_Placeholder_But_Not_A_Real_One()
{
    let mut store = Seeded();

    store
        .Upsert_Node(
            "ADR-DOC-001",
            "decision",
            "canonical-normative-record",
            "document",
            "Markdown is the canonical authored documentation format",
        )
        .expect("upgrades");
    store
        .Upsert_Node("D-129", "decision", "commentary", "document", "Something else")
        .expect("attempts to restate");

    assert_eq!(
        Title(&store, "ADR-DOC-001").as_deref(),
        Some("Markdown is the canonical authored documentation format")
    );
    assert_ne!(
        Title(&store, "D-129").as_deref(),
        Some("Something else"),
        "a later pass overwrote a record its author already wrote"
    );
}

/// Re-ingesting a document must not renumber its blocks.
///
/// `uid` is what every lineage and omission row points at, so a write that deletes the
/// block row and inserts a replacement takes the dispositions with it. Seeding twice is
/// what exposed this; the assertion is on the uids rather than on the counts because a
/// count survives a renumbering that destroys every reference.
#[test]
fn Test_Rewriting_A_Document_Should_Not_Renumber_Its_Blocks()
{
    let mut store = Seeded();
    let before = Block_Uids(&store);

    Seed_Governing_Records(&mut store).expect("seeds again");

    assert!(!before.is_empty(), "no blocks, so this test proved nothing");
    assert_eq!(before, Block_Uids(&store), "the blocks were reinserted under new uids");
}

fn Block_Uids(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_blocks ORDER BY document_uid, ordinal")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// Seeding twice is seeding once.
#[test]
fn Test_Seeding_Should_Be_Idempotent()
{
    let mut store = Seeded();
    let before: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    Seed_Governing_Records(&mut store).expect("seeds again");

    let after: Vec<u32> = Table::All()
        .iter()
        .map(|table| store.Count(*table).expect("counts"))
        .collect();

    assert_eq!(before, after);
}

/// The records are present as content, not only as identity. Without blocks there is
/// nothing for the preservation rules to examine and every one of them reports clean
/// having looked at nothing.
#[test]
fn Test_The_Records_Should_Be_Present_As_Disposed_Content()
{
    let store = Seeded();

    assert_eq!(store.Count(Table::SourceDocuments).expect("counts"), 8);
    assert!(store.Count(Table::SourceBlocks).expect("counts") >= 40);
    assert!(store.Count(Table::SourceHeadings).expect("counts") >= 16);

    let empty: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_documents d
             WHERE NOT EXISTS (SELECT 1 FROM source_blocks b WHERE b.document_uid = d.uid)",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(empty, 0, "{empty} record(s) segmented to nothing");

    let undisposed: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_blocks b
             WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(undisposed, 0, "{undisposed} seeded block(s) have no disposition");
}

#[test]
fn Test_Authored_Content_Should_Be_Distinguishable_From_An_Ingested_Revision()
{
    let store = Seeded();

    let revisions: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_documents WHERE revision <> ?1",
            rusqlite::params![AUTHORED],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(revisions, 0, "a governing record claims to come from a corpus revision");
}

/// A carriage return in an embedded record would change every hash it produces, so the
/// same commit would validate on one machine and not on another. `.gitattributes` pins
/// it; this is what checks the pin held.
#[test]
fn Test_No_Governing_Record_Should_Carry_A_Carriage_Return()
{
    let store = Seeded();

    let mut statement = store
        .Connection()
        .prepare("SELECT path, text FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid")
        .expect("prepares");
    let offenders: Vec<String> = statement
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let text: String = row.get(1)?;
            return Ok((path, text));
        })
        .expect("queries")
        .filter_map(Result::ok)
        .filter(|(_, text)| text.contains('\r'))
        .map(|(path, _)| path)
        .collect();

    assert!(offenders.is_empty(), "CRLF reached the store from {offenders:?}");
    assert!(
        store.Count(Table::SourceBlocks).expect("counts") > 0,
        "no blocks were examined, so this found nothing by looking at nothing"
    );
}
