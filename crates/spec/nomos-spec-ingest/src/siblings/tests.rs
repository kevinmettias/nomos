//! What this module promises, exercised.

use super::*;

const PLAN: &str = "4. How Nomos relates to KnowledgeWorkbench\n\n\
                    Nomos deterministically enforces the standards.\n";

/// A store holding the root suite, and the row that suite landed on.
///
/// Named rather than a pair, so that a caller reading one member is reading a name and
/// not a position.
struct RootedStore
{
    store: SpecificationStore,
    root: i64,
}

fn Rooted() -> RootedStore
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let root = store
        .Put_Suite(ROOT_SUITE, "The Nomos specification", SuiteAuthority::Root)
        .expect("records the root suite");

    return RootedStore { store, root };
}

#[test]
fn Test_A_Game_Plan_Should_Enter_As_Commentary()
{
    let RootedStore { mut store, root } = Rooted();

    let node = Ingest_Game_Plan(&mut store, root, "nomos full game plan.txt", PLAN)
        .expect("ingests");

    assert_eq!(node, "PLAN-NOMOS-FULL-GAME-PLAN-TXT");
    let authority: String = store
        .Connection()
        .query_row(
            "SELECT authority FROM nodes WHERE node_id = ?1",
            rusqlite::params![node],
            |row| row.get(0),
        )
        .expect("queries");
    assert_eq!(authority, COMMENTARY);
}

/// Every block of a game plan is disposed, or NSV-PRESERVE-002 reports the plan as
/// silently dropped content the moment it is ingested.
#[test]
fn Test_Every_Game_Plan_Block_Should_Be_Disposed_To_The_Commentary_Node()
{
    let RootedStore { mut store, root } = Rooted();
    Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");

    let undisposed: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_blocks b
             WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.source_block_uid = b.uid)",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(undisposed, 0);
    assert!(store.Count(nomos_spec_store::Table::SourceBlocks).expect("counts") > 0);
}

/// D-120, mechanized. A statement resting on the plan and nothing else is reported.
#[test]
fn Test_A_Statement_Sourced_Only_From_A_Game_Plan_Should_Be_Reported()
{
    let RootedStore { mut store, root } = Rooted();
    Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
    Statement(&mut store, "AGT-001");
    Trace_To_Plan(&store, "AGT-001");
    Prepare_Commentary_View(&store).expect("prepares");

    assert_eq!(
        Statements_Sourced_Only_From_Commentary(&store).expect("queries"),
        vec!["AGT-001".to_owned()]
    );
}

/// The other half. A statement that also rests on real source is not reported, or the
/// rule would forbid citing the plan at all rather than forbidding relying on it.
#[test]
fn Test_A_Statement_With_A_Real_Source_Too_Should_Not_Be_Reported()
{
    let RootedStore { mut store, root } = Rooted();
    Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
    crate::phases::Ingest_Source_Document(&mut store, "v.md", "v14.36", "# T\n\nReal.\n")
        .expect("ingests");
    Statement(&mut store, "AGT-001");
    Trace_To_Plan(&store, "AGT-001");

    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_statement)
             SELECT b.uid, 'preserved-verbatim', s.uid
             FROM source_blocks b, source_documents d, normative_statements s
             WHERE d.path = 'v.md' AND b.document_uid = d.uid AND b.ordinal = 2
               AND s.statement_id = 'AGT-001'",
            [],
        )
        .expect("links the real source");
    Prepare_Commentary_View(&store).expect("prepares");

    assert!(
        Statements_Sourced_Only_From_Commentary(&store)
            .expect("queries")
            .is_empty()
    );
}

/// A statement with no lineage at all belongs to NSV-PRESERVE-006, not here.
#[test]
fn Test_A_Statement_With_No_Lineage_Should_Not_Be_Reported_Here()
{
    let RootedStore { mut store, root } = Rooted();
    Ingest_Game_Plan(&mut store, root, "plan.txt", PLAN).expect("ingests");
    Statement(&mut store, "AGT-002");
    Prepare_Commentary_View(&store).expect("prepares");

    assert!(
        Statements_Sourced_Only_From_Commentary(&store)
            .expect("queries")
            .is_empty()
    );
}

fn Statement(store: &mut SpecificationStore, id: &str)
{
    let node = store
        .Upsert_Node(id, "requirement", "canonical", "record", id)
        .expect("mints");
    store
        .Connection()
        .execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             VALUES (?1, ?2, 'Requirement', 'Nomos shall.', 'sha256:aa')",
            rusqlite::params![node, id],
        )
        .expect("inserts");
}

fn Trace_To_Plan(store: &SpecificationStore, id: &str)
{
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_statement)
             SELECT b.uid, 'preserved-verbatim', s.uid
             FROM source_blocks b, source_documents d, normative_statements s
             WHERE d.revision = ?1 AND b.document_uid = d.uid AND b.ordinal = 1
               AND s.statement_id = ?2",
            rusqlite::params![LINEAGE_NOTES, id],
        )
        .expect("links the plan");
}

#[test]
fn Test_Suite_Identifiers_Should_Be_Distinct()
{
    let mut seen = std::collections::BTreeSet::new();
    for sibling in Sibling::All()
    {
        assert!(seen.insert(sibling.Suite_Id()), "{} is declared twice", sibling.Suite_Id());
        assert_ne!(sibling.Suite_Id(), ROOT_SUITE, "a sibling claimed the root suite");
    }
}

#[test]
fn Test_A_Qualified_Identifier_Should_Name_Its_Suite()
{
    assert_eq!(
        Qualified(Sibling::Xvpe, "xvpe-spec-seed-v0.1/machine/target-adapter.schema.json"),
        "xvpe-spec-seed:target-adapter.schema.json"
    );
}
