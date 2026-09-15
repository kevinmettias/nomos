//! D-129 supersedes ADR-DOC-001, and the placeholder that stands for it stays visible.

use crate::queries::{Column, Counted, NodeId, Seeded, Title};
use nomos_spec_store::{EXTERNAL, NodeRow};

/// D-129 supersedes ADR-DOC-001. The edge has to be in the store, not only in the prose,
/// or "what superseded this?" is a question only a person reading markdown can answer.
#[test]
fn Test_The_Superseded_Record_Should_Carry_An_Explicit_Supersession_Edge()
{
    let store = Seeded();
    let forward = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         WHERE f.node_id = 'D-129' AND r.relation_type = 'supersedes'
           AND t.node_id = 'ADR-DOC-001'",
    );
    let inverse = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         WHERE f.node_id = 'ADR-DOC-001' AND r.relation_type = 'superseded_by'
           AND t.node_id = 'D-129'",
    );

    assert!(
        store.Node_Uid("ADR-DOC-001").expect("queries").is_some(),
        "the superseded record must exist for the edge to mean anything"
    );
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
    let authority = Column(&store, "SELECT authority FROM nodes WHERE node_id = ?1", NodeId("ADR-DOC-001"));
    let bodies = Counted(
        &store,
        "SELECT count(*) FROM source_documents WHERE path LIKE '%ADR-DOC-001%'",
    );

    assert_eq!(authority, EXTERNAL);
    assert_eq!(
        bodies, 0,
        "a placeholder with a body would be filler wearing a record's clothes"
    );
}

/// A real record arriving later must fill the placeholder in rather than being ignored.
#[test]
fn Test_A_Real_Record_Should_Replace_A_Placeholder_But_Not_A_Real_One()
{
    let mut store = Seeded();
    store
        .Upsert_Node(NodeRow {
            node_id: "ADR-DOC-001",
            kind: "decision",
            authority: "canonical-normative-record",
            representation: "document",
            title: "Markdown is the canonical authored documentation format",
        })
        .expect("the seed left ADR-DOC-001 as an external placeholder, so this updates that row");
    store
        .Upsert_Node(NodeRow {
            node_id: "D-129",
            kind: "decision",
            authority: "commentary",
            representation: "document",
            title: "Something else",
        })
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
