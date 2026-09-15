//! Reading a record out: from the rows, not from the blob, and not at all where nothing
//! was authored.

use crate::seeded::{Edge_Count, Only_Document, Seeded};
use nomos_spec_store::{EditError, GOVERNING_RECORD_IDS, NodeRow, SpecificationStore};

/// The claim this whole item rests on. Rendering happens from the declared front matter and
/// the block rows, so passing means the store holds enough to reproduce the file — which is
/// what `D-129` means by the store being the substrate and markdown being a surface.
#[test]
fn Test_Every_Governing_Record_Should_Project_To_Its_Own_Bytes()
{
    let store = Seeded();
    let mut checked = 0_usize;

    for id in GOVERNING_RECORD_IDS
    {
        Assert_Projects_To_Its_Own_Bytes(&store, id);
        checked = checked.saturating_add(1);
    }

    assert_eq!(
        checked,
        GOVERNING_RECORD_IDS.len(),
        "the loop skipped records, so a pass here would mean less than it says"
    );
}

/// One record renders back to the bytes it was seeded from, and its two content addresses
/// agree about that.
fn Assert_Projects_To_Its_Own_Bytes(store: &SpecificationStore, id: &str)
{
    let projection = store
        .Record_Markdown(id, None)
        // Every id reaching here comes from `GOVERNING_RECORD_IDS` over a seeded store, so a
        // refusal is a seeded record that will not render back at all — a stronger failure
        // than the byte comparison below. The id is in the message because the caller loops
        // over the whole governing set and counts the iterations.
        .unwrap_or_else(|error| panic!("{id}: {error}"));
    let source = Only_Document(store, id);

    assert_eq!(
        projection.markdown, source.text,
        "{id} does not render back to the bytes it was seeded from"
    );
    assert!(projection.Is_Matching_Source(), "{id}: the content addresses disagree");
}

/// The negative control for the test above, and the reason the projection is worth having:
/// point the document at entirely different bytes and the markdown is unchanged, because it
/// was never read from there. Without this, an implementation that echoed the blob would pass
/// every other assertion in this suite.
#[test]
fn Test_A_Projection_Should_Come_From_The_Rows_And_Not_The_Blob()
{
    let mut store = Seeded();
    let before = store
        .Record_Markdown("D-132", None)
        .expect("D-132 is one of the records the seed authored, so its rows render markdown back");

    let other = store.Put_Blob(b"not a record at all").expect("writes a blob");
    store
        .Connection()
        .execute(
            "UPDATE source_documents SET blob_uid = ?2 WHERE path LIKE '%D-132%'",
            rusqlite::params![0_i64, other],
        )
        .expect("repoints the document");

    let after = store.Record_Markdown("D-132", None).expect("still projects");

    assert_eq!(after.markdown, before.markdown, "the projection followed the blob");
    assert!(
        !after.Is_Matching_Source(),
        "the document now holds different bytes, and the projection must say so"
    );
}

/// Why `record_relations` exists rather than reading the graph. `relations` is completed with
/// inverses on the way in, and `relates-to` is its own inverse, so a record that is merely the
/// target of one has an outgoing edge it never wrote. Rendering front matter from the graph
/// would put that edge in the file.
#[test]
fn Test_The_Graph_Holds_An_Edge_The_Record_Never_Declared()
{
    let store = Seeded();
    let in_graph = Edge_Count(&store, "OD-LEDGER-001", "OD-LEDGER-009");
    let document = Only_Document(&store, "OD-LEDGER-001");
    let declared = store
        .Declared_Relations(document.uid)
        .expect("the seed stored OD-LEDGER-001's declared relations, so its uid has rows to read");

    assert_eq!(in_graph, 1, "the inverse edge is not there, so this proves nothing");
    assert!(
        !declared
            .iter()
            .any(|relation| return relation.target == "OD-LEDGER-009"),
        "OD-LEDGER-001 declares an edge its file does not carry"
    );
    assert!(
        !store
            .Record_Markdown("OD-LEDGER-001", None)
            .expect("projects")
            .markdown
            .contains("OD-LEDGER-009"),
        "the projection invented a relation"
    );
}

/// Reading out a record nothing wrote through this door is a distinct answer from reading out
/// one that does not exist. A corpus document arrives as blocks and lineage and declares no
/// front matter, so it can be read, hashed and preserved but not rendered back.
#[test]
fn Test_A_Document_With_No_Declared_Front_Matter_Should_Say_So()
{
    let store = An_Ingested_Document();
    let refusal = store.Record_Markdown("VOL-001", None).expect_err("must refuse");

    assert!(matches!(refusal, EditError::NotAuthored { .. }), "{refusal}");
    assert!(
        matches!(
            store.Record_Markdown("VOL-999", None).expect_err("must refuse"),
            EditError::NoSuchRecord { .. }
        ),
        "an unknown identifier and an unauthored document report the same way"
    );
}

/// A document that arrived as blocks and lineage rather than through the authoring door, so
/// it can be read, hashed and preserved but not rendered back.
fn An_Ingested_Document() -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory() applies this crate's schema in process, so there is no file to fail on");
    let uid = store
        .Put_Source_Document("volumes/one.md", "v14.36", "# Volume\n\nBody.\n")
        .expect("the store was just opened empty, so this path and its document uid are both free");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "VOL-001",
            kind: "volume",
            authority: "canonical",
            representation: "document",
            title: "Volume",
        })
        .expect("writes a node");
    store
        .Put_Source_Blocks(uid, &nomos_spec_model::Segment("# Volume\n\nBody.\n"))
        .expect("writes blocks");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_node_uid)
             SELECT uid, 'preserved-verbatim', ?1 FROM source_blocks WHERE document_uid = ?2",
            rusqlite::params![node, uid],
        )
        .expect("Put_Source_Blocks just wrote this document's blocks, so the SELECT has rows to dispose");

    return store;
}
