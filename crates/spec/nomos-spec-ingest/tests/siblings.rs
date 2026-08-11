//! P3-SIBLINGS. I6 and I8, against the real seeds and the real game plans.
//!
//! The two halves are one item because they answer one question in two directions: what is
//! ours, and what is authority. A sibling's record is canonical and not ours; a game plan
//! is ours and not authority. Neither distinction exists in a store that only knows
//! documents, and both are how a statement ends up resting on something that never said it.

use nomos_spec_ingest::{
    Archive, COMMENTARY, Ingest_Game_Plan, Ingest_Sibling_Suite, Ingest_Source_Document,
    LINEAGE_NOTES, Prepare_Commentary_View, ROOT_SUITE, Sibling,
    Statements_Sourced_Only_From_Commentary,
};
use nomos_spec_store::{SpecificationStore, SuiteAuthority, Table};
use nomos_spec_validate::{Registered, Validate};
use std::path::PathBuf;

/// The two plans, and whose lineage each one is.
const PLANS: &[(&str, Option<Sibling>)] = &[
    ("nomos full game plan.txt", None),
    ("kwb full game plan.txt", Some(Sibling::Kwb)),
];

fn Archives() -> Option<PathBuf>
{
    let root = PathBuf::from(std::env::var_os("NOMOS_SPEC_ARCHIVES")?);
    assert!(root.is_dir(), "NOMOS_SPEC_ARCHIVES is not a directory: {}", root.display());
    return Some(root);
}

/// Every suite, plus both game plans, in one store.
fn Ecosystem() -> Option<SpecificationStore>
{
    let archives = Archives()?;
    let mut store = SpecificationStore::In_Memory().expect("opens");

    let root = store
        .Put_Suite(ROOT_SUITE, "The Nomos specification", SuiteAuthority::Root)
        .expect("records the root suite");

    for sibling in Sibling::All()
    {
        let mut archive = Archive::Open(&archives.join(sibling.Archive()))
            .unwrap_or_else(|error| panic!("{error}"));
        let report = Ingest_Sibling_Suite(&mut store, &mut archive, *sibling)
            .unwrap_or_else(|error| panic!("{}: {error}", sibling.Suite_Id()));

        assert!(
            report.contested.is_empty(),
            "{} declares identifiers another suite already owns: {:?}",
            sibling.Suite_Id(),
            report.contested
        );
        assert!(!report.records.is_empty(), "{} ingested nothing", sibling.Suite_Id());
        assert!(!report.schemas.is_empty(), "{} carries no machine schemas", sibling.Suite_Id());
    }

    for (name, owner) in PLANS
    {
        let path = archives.join(name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let suite = owner.map_or(root, |sibling| {
            return store
                .Put_Suite(sibling.Suite_Id(), sibling.Title(), SuiteAuthority::Sibling)
                .expect("records the suite");
        });
        Ingest_Game_Plan(&mut store, suite, name, &text)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    return Some(store);
}

/// I6's own sentence: a sibling suite is not the root.
#[test]
fn Test_Every_Sibling_Suite_Should_Be_Non_Root()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };

    let roots: Vec<String> = store
        .Connection()
        .prepare("SELECT suite_id FROM suites WHERE authority_root = 1 ORDER BY suite_id")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("queries");

    assert_eq!(
        roots,
        vec![ROOT_SUITE.to_owned()],
        "exactly one suite is this repository's own"
    );
    assert_eq!(
        store.Count(Table::Suites).expect("counts"),
        u32::try_from(Sibling::All().len().saturating_add(1)).unwrap_or(u32::MAX)
    );
}

#[test]
fn Test_Every_Sibling_Schema_Should_Resolve_As_A_Node_In_Its_Own_Suite()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };

    // Measured over the three seeds: 13 files under `machine/`, of which 8 are named
    // `*.schema.json` and declare a shape. The other 5 are instance documents — an
    // ownership matrix, a platform profile, a dependency inventory, a package envelope,
    // an evidence exchange — and counting them as schemas would answer "how many schemas"
    // with the file count.
    assert_eq!(Kind_Count(&store, "schema"), 8, "schema files across the three seeds");
    assert_eq!(Kind_Count(&store, "machine_document"), 5, "machine files that are not schemas");

    for sibling in Sibling::All()
    {
        let rooted: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM nodes n JOIN suites s ON s.uid = n.suite_uid
                 WHERE n.kind = 'schema' AND s.suite_id = ?1 AND s.authority_root = 1",
                rusqlite::params![sibling.Suite_Id()],
                |row| row.get(0),
            )
            .expect("queries");
        assert_eq!(rooted, 0, "{} claimed root authority", sibling.Suite_Id());

        let owned: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM nodes n JOIN suites s ON s.uid = n.suite_uid
                 WHERE n.kind = 'schema' AND s.suite_id = ?1",
                rusqlite::params![sibling.Suite_Id()],
                |row| row.get(0),
            )
            .expect("queries");
        assert!(owned > 0, "{} has no schema nodes", sibling.Suite_Id());
    }
}

fn Kind_Count(store: &SpecificationStore, kind: &str) -> u32
{
    return store
        .Connection()
        .query_row(
            "SELECT count(*) FROM nodes WHERE kind = ?1",
            rusqlite::params![kind],
            |row| row.get(0),
        )
        .expect("queries");
}

/// A sibling's decision resolves by the identifier it declares, so a cross-suite relation
/// is an ordinary row rather than a special case.
#[test]
fn Test_A_Sibling_Record_Should_Resolve_By_Its_Own_Identifier()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };

    for (id, suite) in [
        ("D-085", Sibling::Xvpe.Suite_Id()),
        ("D-122", Sibling::Xvpe.Suite_Id()),
        ("D-102", Sibling::Kwb.Suite_Id()),
        ("D-096", Sibling::Ecosystem.Suite_Id()),
        ("ARC-ECOSYS-001", Sibling::Ecosystem.Suite_Id()),
    ]
    {
        let held = store
            .Suite_Of(id)
            .expect("queries")
            .unwrap_or_else(|| panic!("{id} resolves to no suite"));

        assert_eq!(held.0, suite, "{id}");
        assert_eq!(
            held.1,
            SuiteAuthority::Sibling,
            "{id} is a sibling's record and reads as this repository's"
        );
    }
}

/// A cross-suite relation, written as an ordinary row and read back.
#[test]
fn Test_A_Cross_Suite_Relation_Should_Be_An_Ordinary_Row()
{
    let Some(mut store) = Ecosystem()
    else
    {
        return;
    };

    store.Put_Relation_Type("depends_on", "seed").expect("names the type");
    store
        .Upsert_Node("D-130", "decision", "canonical", "record", "No XVPE before Phase 5")
        .expect("mints this repository's decision");
    let root_uid: i64 = store
        .Connection()
        .query_row(
            "SELECT uid FROM suites WHERE suite_id = ?1",
            rusqlite::params![ROOT_SUITE],
            |row| row.get(0),
        )
        .expect("queries");
    let node = store.Node_Uid("D-130").expect("queries").expect("exists");
    store.Assign_Suite(node, root_uid).expect("places it");

    store.Put_Relation("D-130", "depends_on", "D-085").expect("relates");

    let across: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             JOIN suites fs ON fs.uid = f.suite_uid
             JOIN suites ts ON ts.uid = t.suite_uid
             WHERE fs.authority_root = 1 AND ts.authority_root = 0",
            [],
            |row| row.get(0),
        )
        .expect("queries");

    assert_eq!(across, 1, "the relation crossing the suite boundary is not visible as one");
}

/// I8. Every game-plan block is commentary, and none of them is anything else.
#[test]
fn Test_Every_Game_Plan_Block_Should_Be_Commentary()
{
    let Some(store) = Ecosystem()
    else
    {
        return;
    };

    let blocks: u32 = store
        .Connection()
        .query_row(
            "SELECT count(*) FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid
             WHERE d.revision = ?1",
            rusqlite::params![LINEAGE_NOTES],
            |row| row.get(0),
        )
        .expect("queries");
    assert!(blocks > 100, "only {blocks} game-plan block(s), so the plans did not land");

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
            |row| row.get(0),
        )
        .expect("queries");

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

    Prepare_Commentary_View(&store).expect("prepares");

    assert!(
        store.Count(Table::NormativeStatements).expect("counts") == 0,
        "the ecosystem store carries statements, so this assertion must be re-read"
    );
    assert!(
        Statements_Sourced_Only_From_Commentary(&store)
            .expect("queries")
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

    let mut store = SpecificationStore::In_Memory().expect("opens");
    let root = store.Put_Suite(ROOT_SUITE, "The Nomos specification", SuiteAuthority::Root).expect("records");

    let name = "nomos full game plan.txt";
    let text = std::fs::read_to_string(archives.join(name)).expect("reads the plan");
    Ingest_Game_Plan(&mut store, root, name, &text).expect("ingests");

    let node = store
        .Upsert_Node("AGT-999", "requirement", "canonical", "record", "AGT-999")
        .expect("mints");
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

    Prepare_Commentary_View(&store).expect("prepares");

    assert_eq!(
        Statements_Sourced_Only_From_Commentary(&store).expect("queries"),
        vec!["AGT-999".to_owned()],
        "a statement resting on the plan alone was not caught"
    );
}

/// The suites and the plans must leave the preservation ledger clean, or I6 and I8 have
/// added content the rules report as silently dropped.
#[test]
fn Test_The_Ecosystem_Store_Should_Report_No_Preservation_Errors()
{
    let Some(mut store) = Ecosystem()
    else
    {
        return;
    };

    // One root-suite document too, so the run is over a store holding both authorities
    // rather than only over siblings.
    Ingest_Source_Document(&mut store, "own.md", "authored", "# Ours\n\nOne.\n").expect("ingests");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition)
             SELECT b.uid, 'preserved-verbatim' FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid WHERE d.path = 'own.md'",
            [],
        )
        .expect("disposes them");

    let run = Validate(&store, &Registered());

    assert!(run.Errors().is_empty(), "{:?}", run.Errors());
    assert!(
        run.Violations().is_empty(),
        "{}\nfirst: {:?}",
        run.Summary(),
        run.Violations().iter().take(5).collect::<Vec<_>>()
    );
    assert!(run.Passed(), "{}", run.Summary());
}

#[test]
fn Test_Re_Ingesting_The_Suites_Should_Change_Nothing()
{
    let Some(archives) = Archives()
    else
    {
        return;
    };

    let Some(mut store) = Ecosystem()
    else
    {
        return;
    };
    let before = (
        store.Count(Table::Nodes).expect("counts"),
        store.Count(Table::Suites).expect("counts"),
        store.Count(Table::SourceBlocks).expect("counts"),
        store.Count(Table::Lineage).expect("counts"),
    );

    for sibling in Sibling::All()
    {
        let mut archive = Archive::Open(&archives.join(sibling.Archive())).expect("opens");
        let report = Ingest_Sibling_Suite(&mut store, &mut archive, *sibling).expect("re-ingests");
        assert!(report.contested.is_empty(), "{:?}", report.contested);
    }

    assert!(before.0 > 0 && before.1 > 0, "the first pass wrote nothing");
    assert_eq!(
        (
            store.Count(Table::Nodes).expect("counts"),
            store.Count(Table::Suites).expect("counts"),
            store.Count(Table::SourceBlocks).expect("counts"),
            store.Count(Table::Lineage).expect("counts"),
        ),
        before
    );
}
