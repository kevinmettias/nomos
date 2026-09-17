//! P3-SIBLINGS. I6 and I8, against the real seeds and the real game plans.
//!
//! The two halves are one item because they answer one question in two directions: what is
//! ours, and what is authority. A sibling's record is canonical and not ours; a game plan
//! is ours and not authority. Neither distinction exists in a store that only knows
//! documents, and both are how a statement ends up resting on something that never said it.
//!
//! I8's half — the game plans, and the rule that nothing normative rests on one alone — is
//! [`game_plans`], the module this target declares below.

use nomos_spec_ingest::{
    Archive, Ingest_Game_Plan, Ingest_Sibling_Suite, Ingest_Source_Document, ROOT_SUITE, Sibling,
};
use nomos_spec_store::{NodeRow, SpecificationStore, SuiteAuthority, Table};
use nomos_spec_validate::{Registered, Validate_Rules};
use std::path::{Path, PathBuf};

/// I8's half, in its own file: the target crossed the 500-line review trigger and the two
/// halves read better apart than as one 537-line file.
///
/// The path is spelled out because an integration test's crate root resolves `mod` beside
/// itself — in `tests/` — and a bare `tests/game_plans.rs` would be compiled as a second
/// test target rather than as this one's module.
#[path = "siblings/game_plans.rs"]
mod game_plans;

/// How many rule violations the failure message names before it stops listing them.
const VIOLATIONS_LISTED: usize = 5;

/// How many `depends_on` edges one node may carry: one per dependency this file writes.
const DEPENDENCIES_PER_NODE: u32 = 4;

/// The schema nodes the three seeds carry, and the machine documents that declare no shape.
const MACHINE_SCHEMAS: u32 = 8;
const MACHINE_DOCUMENTS: u32 = 5;

/// A SQL statement written by hand in this file.
///
/// Named so that it cannot be passed where the value the statement binds is expected: a
/// caller who transposes the two is refused by the compiler rather than by a test.
struct Sql(&'static str);

/// One counted answer, for a query that binds nothing.
fn Counted_Of_Sql(store: &SpecificationStore, sql: Sql) -> u32
{
    return store
        .Connection()
        .query_row(sql.0, [], |row| return row.get(0))
        // The statement is a literal in this file, so what it names is the schema the store
        // was created from rather than anything a caller could have supplied.
        .expect("the statement is written in this file over the store's own tables");
}

/// One counted answer, for a query that binds one value as `?1`.
fn Counted_For(store: &SpecificationStore, sql: Sql, bound: &str) -> u32
{
    return store
        .Connection()
        .query_row(sql.0, rusqlite::params![bound], |row| return row.get(0))
        // The same invariant as [`Counted`]: both the statement and the binding are written
        // here, and the store's own schema is what the statement is answerable to.
        .expect("the statement is written in this file over the store's own tables");
}

/// The root suite's surrogate.
fn Root_Suite_Uid(store: &SpecificationStore) -> i64
{
    return store
        .Connection()
        .query_row(
            "SELECT uid FROM suites WHERE suite_id = ?1",
            rusqlite::params![ROOT_SUITE],
            |row| return row.get(0),
        )
        // The statement and its binding are both written here, and the suites table is the
        // store's own — so a row answers unless the root suite was never put.
        .expect("the statement is written in this file over the store's own tables");
}

/// Names `depends_on`, admitting only `decision` at either end, at a cardinality of
/// [`DEPENDENCIES_PER_NODE`].
fn Put_Depends_On_Relation_Type(store: &mut SpecificationStore)
{
    store
        .Put_Relation_Type(
            "depends_on",
            "seed",
            &nomos_spec_store::Constraint {
                domain: &["decision"],
                range: &["decision"],
                max_per_node: DEPENDENCIES_PER_NODE,
            },
        )
        .expect("names the type");
}

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
    let mut store = SpecificationStore::In_Memory()
        .expect("an in-memory store opens over no file, so this construction has no failure path");
    let root = Put_Root_Suite(&mut store);

    Ingest_Every_Sibling(&mut store, &archives);
    Ingest_Every_Plan(&mut store, &archives, root);

    return Some(store);
}

/// This repository's own root suite.
fn Put_Root_Suite(store: &mut SpecificationStore) -> i64
{
    return store
        .Put_Suite(ROOT_SUITE, "The Nomos specification", SuiteAuthority::Root)
        .expect("records the root suite");
}

/// Every sibling's suite, ingested from its own archive.
fn Ingest_Every_Sibling(store: &mut SpecificationStore, archives: &Path)
{
    for sibling in Sibling::All()
    {
        Ingest_One_Sibling(store, archives, *sibling);
    }
}

/// Both game plans, each under the suite its lineage belongs to.
fn Ingest_Every_Plan(store: &mut SpecificationStore, archives: &Path, root: i64)
{
    for (name, owner) in PLANS
    {
        let suite = Suite_For(store, root, *owner);
        let text = Plan_Text(archives, name);

        Ingest_Game_Plan(store, suite, *name, &text)
            // I8 asserts only "the plans did not land" over a block count, and there are two
            // plans under two different authorities. Naming this one and the ingest's own error
            // is the difference between that count and a repair.
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

/// One sibling suite, ingested and checked for the two ways it could arrive empty.
fn Ingest_One_Sibling(store: &mut SpecificationStore, archives: &Path, sibling: Sibling)
{
    let mut archive = Archive::Open(&archives.join(sibling.Archive()))
        // A seed that will not open leaves its suite absent, and the suite count then reads as
        // this repository having minted the wrong number of them rather than as a missing zip.
        .unwrap_or_else(|error| panic!("{error}"));
    let report = Ingest_Sibling_Suite(store, &mut archive, sibling)
        // The two assertions below catch a suite that arrived empty. A refused ingest is the third
        // way, and the only one that can say why — named by suite, since three run through here.
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

/// The suite a plan's lineage belongs to: a sibling's own, or this repository's root.
fn Suite_For(store: &mut SpecificationStore, root: i64, owner: Option<Sibling>) -> i64
{
    let Some(sibling) = owner
    else
    {
        return root;
    };

    return store
        .Put_Suite(sibling.Suite_Id(), sibling.Title(), SuiteAuthority::Sibling)
        .expect("records the suite");
}

/// One plan's text, or a panic naming the file that would not read.
fn Plan_Text(archives: &Path, name: &str) -> String
{
    let path = archives.join(name);

    return std::fs::read_to_string(&path)
        // The plans sit beside the archives rather than in this repository, so which path was
        // tried is the whole of the answer when one of them is not there.
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
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
        // The statement is a literal here and the store is the one `Ecosystem()` built, so the
        // suites table it reads is the one this run wrote.
        .expect("the statement is written in this file over the store's own tables");

    assert_eq!(
        roots,
        vec![ROOT_SUITE.to_owned()],
        "exactly one suite is this repository's own"
    );
    assert_eq!(
        store.Count(Table::Suites).expect("the store answers a count over the tables it owns"),
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

    Assert_Machine_Counts(&store);

    for sibling in Sibling::All()
    {
        Assert_Owned_By_Its_Own_Suite(&store, *sibling);
    }
}

/// The two schema counts the three seeds settle.
///
/// Measured over the three seeds: 13 files under `machine/`, of which 8 are named
/// `*.schema.json` and declare a shape. The other 5 are instance documents — an
/// ownership matrix, a platform profile, a dependency inventory, a package envelope,
/// an evidence exchange — and counting them as schemas would answer "how many schemas"
/// with the file count.
fn Assert_Machine_Counts(store: &SpecificationStore)
{
    assert_eq!(
        Kind_Count(store, "schema"),
        MACHINE_SCHEMAS,
        "schema files across the three seeds"
    );
    assert_eq!(
        Kind_Count(store, "machine_document"),
        MACHINE_DOCUMENTS,
        "machine files that are not schemas"
    );
}

/// A sibling's schemas belong to the sibling's own suite, and none of them claims root.
fn Assert_Owned_By_Its_Own_Suite(store: &SpecificationStore, sibling: Sibling)
{
    let rooted = Counted_For(
        store,
        Sql(
            "SELECT count(*) FROM nodes n JOIN suites s ON s.uid = n.suite_uid
             WHERE n.kind = 'schema' AND s.suite_id = ?1 AND s.authority_root = 1",
        ),
        sibling.Suite_Id(),
    );
    let owned = Counted_For(
        store,
        Sql(
            "SELECT count(*) FROM nodes n JOIN suites s ON s.uid = n.suite_uid
             WHERE n.kind = 'schema' AND s.suite_id = ?1",
        ),
        sibling.Suite_Id(),
    );

    assert_eq!(rooted, 0, "{} claimed root authority", sibling.Suite_Id());
    assert!(owned > 0, "{} has no schema nodes", sibling.Suite_Id());
}

/// How many nodes of one kind the store holds.
///
/// Named rather than inlined at each call: the two counts differ by which kind they ask for,
/// and `machine_document` is the name of a kind rather than a document that is not a machine.
fn Kind_Count(store: &SpecificationStore, kind: &str) -> u32
{
    return Counted_For(store, Sql("SELECT count(*) FROM nodes WHERE kind = ?1"), kind);
}

/// A record identifier and the sibling suite it must resolve to, named for what is being
/// resolved rather than `Cases()` — what varies here is which sibling's own identifier it is.
fn Sibling_Record_Identifiers() -> Vec<(&'static str, &'static str)>
{
    return vec![
        ("D-085", Sibling::Xvpe.Suite_Id()),
        ("D-122", Sibling::Xvpe.Suite_Id()),
        ("D-102", Sibling::Kwb.Suite_Id()),
        ("D-096", Sibling::Ecosystem.Suite_Id()),
        ("ARC-ECOSYS-001", Sibling::Ecosystem.Suite_Id()),
    ];
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

    for (id, suite) in Sibling_Record_Identifiers()
    {
        let held = store
            .Suite_Of(id)
            // A statement and a binding both written here, over the store's own nodes table.
            .expect("the statement is written in this file over the store's own tables")
            // "Resolves to no suite" and "resolves to the wrong suite" are different failures, and
            // the two assertions below can only speak about the second. This is where the first
            // gets said.
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
    let root_uid = Root_Suite_Uid(&store);

    Mint_D_130(&mut store, root_uid);
    let across = Cross_Suite_Relation_Count(&store);

    assert_eq!(across, 1, "the relation crossing the suite boundary is not visible as one");
}

/// Mints `D-130` under the root suite and points it at the sibling record it depends on.
///
/// `OD-SPEC-012` (`nomos-spec-store`) made domain, range and cardinality required rather than
/// defaulted, so this test's own ad hoc type declares them too. `D-130` and the sibling record
/// it depends on are both decisions, and one dependency per test is all this fixture writes.
fn Mint_D_130(store: &mut SpecificationStore, root_uid: i64)
{
    Put_Depends_On_Relation_Type(store);
    store
        .Upsert_Node(NodeRow {
            node_id: "D-130",
            kind: "decision",
            authority: "canonical",
            representation: "record",
            title: "No XVPE before Phase 5",
        })
        // D-130 is minted here for the first time, so the row it lands on is the one this
        // statement just wrote.
        .expect("mints this repository's decision");
    let node = store
        .Node_Uid("D-130")
        .expect("the statement is written in this file over the store's own tables")
        .expect("the upsert above wrote this identifier into the store");
    store
        .Assign_Suite(node, root_uid)
        .expect("the root suite's row exists, so the assignment lands on a real node");
    store
        .Put_Relation("D-130", "depends_on", "D-085")
        .expect("the type is declared above and both endpoints resolve to rows");
}

/// How many relations cross from the root suite into a sibling's.
fn Cross_Suite_Relation_Count(store: &SpecificationStore) -> u32
{
    return Counted_Of_Sql(
        store,
        Sql(
            "SELECT count(*) FROM relations r
             JOIN nodes f ON f.uid = r.from_node_uid
             JOIN nodes t ON t.uid = r.to_node_uid
             JOIN suites fs ON fs.uid = f.suite_uid
             JOIN suites ts ON ts.uid = t.suite_uid
             WHERE fs.authority_root = 1 AND ts.authority_root = 0",
        ),
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
    Ingest_Source_Document(&mut store, "own.md", "authored", "# Ours\n\nOne.\n")
        .expect("the document is markdown written in this test, so the ingester parses it");
    store
        .Connection()
        .execute(
            "INSERT INTO lineage (source_block_uid, disposition)
             SELECT b.uid, 'preserved-verbatim' FROM source_blocks b
             JOIN source_documents d ON d.uid = b.document_uid WHERE d.path = 'own.md'",
            [],
        )
        .expect("disposes them");
    let run = Validate_Rules(&store, &Registered());

    assert!(run.Errors().is_empty(), "{:?}", run.Errors());
    assert!(
        run.Violations().is_empty(),
        "{}\nfirst: {:?}",
        run.Summary(),
        run.Violations().iter().take(VIOLATIONS_LISTED).collect::<Vec<_>>()
    );
    assert!(run.Is_Passed(), "{}", run.Summary());
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
        store.Count(Table::Nodes).expect("the store answers a count over the tables it owns"),
        store.Count(Table::Suites).expect("the store answers a count over the tables it owns"),
        store.Count(Table::SourceBlocks).expect("the store answers a count over the tables it owns"),
        store.Count(Table::Lineage).expect("the store answers a count over the tables it owns"),
    );

    for sibling in Sibling::All()
    {
        let mut archive = Archive::Open(&archives.join(sibling.Archive()))
            .expect("the sibling's seed archive is a zip this reader opens");
        let report = Ingest_Sibling_Suite(&mut store, &mut archive, *sibling)
            .expect("the same suite ingested a second time takes the same path");
        assert!(report.contested.is_empty(), "{:?}", report.contested);
    }
    assert!(before.0 > 0 && before.1 > 0, "the first pass wrote nothing");
    assert_eq!(
        (
            store.Count(Table::Nodes).expect("the store answers a count over the tables it owns"),
            store.Count(Table::Suites).expect("the store answers a count over the tables it owns"),
            store.Count(Table::SourceBlocks).expect("the store answers a count over the tables it owns"),
            store.Count(Table::Lineage).expect("the store answers a count over the tables it owns"),
        ),
        before
    );
}
