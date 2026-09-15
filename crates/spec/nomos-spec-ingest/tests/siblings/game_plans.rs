//! I8's half of `siblings`: every game-plan block is commentary, and no statement rests on
//! a plan alone.
//!
//! The parent file declares this module, so the two halves remain one test target and one
//! item — the split is where the file crossed the 500-line review trigger, not a second
//! authority over the same store. The fixtures are the parent's: [`Ecosystem`] builds the
//! store, [`Plan_Text`] reads a plan, and [`Counted_For`] runs the counts.

use nomos_spec_ingest::{
    COMMENTARY, Ingest_Game_Plan, LINEAGE_NOTES, Prepare_Commentary_View, ROOT_SUITE,
    Statements_Sourced_Only_From_Commentary,
};
use nomos_spec_store::{NodeRow, SpecificationStore, SuiteAuthority, Table};
use std::path::Path;

use super::{Archives, Counted_For, Ecosystem, Plan_Text, Sql};

/// A floor under the game-plan blocks, so a plan that landed nothing fails loudly rather
/// than agreeing with a store the plans never reached.
const GAME_PLAN_BLOCK_FLOOR: u32 = 100;

/// I8. Every game-plan block is commentary, and none of them is anything else.
#[test]
fn Test_Every_Game_Plan_Block_Should_Be_Commentary()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };
    let blocks = Counted_For(
        &store,
        Sql(
            "SELECT count(*) FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.revision = ?1",
        ),
        LINEAGE_NOTES,
    );
    let uncommentary: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.revision = ?1
               AND NOT EXISTS (
                   SELECT 1 FROM lineage l JOIN nodes n ON n.uid = l.target_node_uid
                   WHERE l.source_block_uid = b.uid AND n.authority = ?2
               )",
            rusqlite::params![LINEAGE_NOTES, COMMENTARY],
            |row| return row.get(0),
        )
        .expect("the statement is written in this file over the store's own tables");

    assert!(
        blocks > GAME_PLAN_BLOCK_FLOOR,
        "only {blocks} game-plan block(s), so the plans did not land"
    );
    assert_eq!(uncommentary, 0, "{uncommentary} game-plan block(s) carry no commentary authority");
}

/// D-120 as a rule rather than a convention: nothing normative rests on a plan alone.
#[test]
fn Test_No_Statement_Should_Rest_On_A_Game_Plan_Alone()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };

    Prepare_Commentary_View(&store)
        .expect("the store holds the blocks and the lineage the view is built from");

    assert!(
        store.Count(Table::NormativeStatements).expect("the store answers a count over the tables it owns") == 0,
        "the ecosystem store carries statements, so this assertion must be re-read"
    );
    assert!(
        Statements_Sourced_Only_From_Commentary(&store)
            .expect("the statement is written in this file over the store's own tables")
            .is_empty()
    );
}

/// The negative control the assertion above cannot be without. A store with no statements
/// satisfies it vacuously, so the rule is exercised against one that has a violation.
#[test]
fn Test_A_Statement_Resting_On_A_Plan_Alone_Should_Be_Caught()
{
    let Some(archives) = Archives()
    else
    {
        return;
    };
    let mut store = A_Plan_Only_Store(&archives);

    Rest_A_Statement_On_The_Plan(&mut store);
    Prepare_Commentary_View(&store)
        .expect("the store holds the blocks and the lineage the view is built from");
    assert_eq!(
        Statements_Sourced_Only_From_Commentary(&store)
            .expect("the statement is written in this file over the store's own tables"),
        vec!["AGT-999".to_owned()],
        "a statement resting on the plan alone was not caught"
    );
}

/// The nomos game plan in a store of its own, under the root suite and nothing else.
fn A_Plan_Only_Store(archives: &Path) -> SpecificationStore
{
    let mut store = SpecificationStore::In_Memory()
        .expect("an in-memory store opens over no file, so this construction has no failure path");
    let root = store
        .Put_Suite(ROOT_SUITE, "The Nomos specification", SuiteAuthority::Root)
        .expect("records the root suite");
    let name = "nomos full game plan.txt";
    let text = Plan_Text(archives, name);

    Ingest_Game_Plan(&mut store, root, name, &text)
        .expect("the plan text was read above and is markdown the ingester parses");

    return store;
}

/// One normative statement whose only lineage is a game-plan block.
fn Rest_A_Statement_On_The_Plan(store: &mut SpecificationStore)
{
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "AGT-999",
            kind: "requirement",
            authority: "canonical",
            representation: "record",
            title: "AGT-999",
        })
        .expect("AGT-999 is an identifier this store does not hold, so the upsert mints a row");

    store
        .Connection()
        .execute(
            "INSERT INTO normative_statements
             (node_uid, statement_id, kind, canonical_text, canonical_hash)
             VALUES (?1, 'AGT-999', 'Requirement', 'Nomos shall.', 'sha256:aa')",
            rusqlite::params![node],
        )
        .expect("inserts the statement");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition, target_statement)
             SELECT b.uid, 'preserved-verbatim', s.uid
             FROM source_blocks b, source_documents d, normative_statements s
             WHERE d.revision = ?1 AND b.document_uid = d.uid AND b.ordinal = 1
               AND s.statement_id = 'AGT-999'",
            rusqlite::params![LINEAGE_NOTES],
        )
        .expect("rests it on the plan");
}
