//! An edge across the seam arrives as a reported placeholder, and the sibling suite claims it.

use crate::common::{Column, Counted, Seeded, Title};
use nomos_spec_store::{EXTERNAL, NodeRow, SeedReport, Seed_Governing_Records, SpecificationStore, SuiteAuthority};

/// The sibling records `ARC-ECOSYSTEM-001` cites, named here rather than counted.
///
/// `D-085`, `D-086`, `D-090` and `D-096` are the decisions it restates and `D-122` is the
/// one it adopts. None of them is this repository's: they belong to `xvpe-spec-seed` and
/// `ecosystem-contracts`, which `P3-SIBLINGS` ingests as non-root suites. Over a store
/// holding only this repository's records they are therefore targets nothing has ingested.
const CITED_SIBLING_RECORDS: &[&str] = &["D-085", "D-086", "D-090", "D-096", "D-122"];

/// A relation across the seam must arrive as a reported placeholder, not as a dropped edge.
///
/// `Write_Relation` inserts by selecting both endpoints out of `nodes`, so an edge to an
/// identifier no node holds matches nothing and writes nothing — silently, at exit 0. What
/// stops that here is the second pass in `Seed_Governing_Records`, which mints the reference
/// first and reports it by name. This asserts the reporting, because a growing list of
/// unresolved targets is the only thing that distinguishes an edge into another suite from
/// an edge into a typo.
#[test]
fn Test_A_Relation_To_A_Sibling_Record_Should_Arrive_As_A_Reported_Placeholder()
{
    let mut store = SpecificationStore::In_Memory().expect("opens");
    let report = Seed_Governing_Records(&mut store).expect("seeds");

    for record in CITED_SIBLING_RECORDS
    {
        Assert_Arrived_As_A_Placeholder(&store, &report, record);
    }
}

/// One cited sibling record: named in the report, minted as external, claiming no suite.
fn Assert_Arrived_As_A_Placeholder(store: &SpecificationStore, report: &SeedReport, record: &str)
{
    assert!(
        report.references.iter().any(|reference| return reference == record),
        "{record} is cited across the seam and the seed report does not name it, so an edge \
         into a sibling suite is indistinguishable from an edge into nothing"
    );

    let authority = Column(store, "SELECT authority FROM nodes WHERE node_id = ?1", record);

    assert_eq!(authority, EXTERNAL, "{record}");
    assert_eq!(
        store.Suite_Of(record).expect("queries"),
        None,
        "{record} claims a suite in a store no suite was ingested into"
    );
}

/// The other half: a placeholder is the sibling's own node once the sibling claims it.
///
/// `Ingest_Sibling_Suite` upserts each record under its declared identifier and assigns it
/// the suite, and that only lands because `Write_Node` updates a node in place where its
/// authority is external. The two halves live in different crates and neither states the
/// property they compose to, so this is where "which suite decided this first" stops being a
/// reading of two functions and becomes a query.
///
/// The suite is created here rather than ingested from an archive on purpose: the archives
/// are outside this repository and a test that cannot find its corpus passes. This one has
/// no corpus to miss.
#[test]
fn Test_A_Placeholder_Should_Become_The_Node_Of_The_Suite_That_Claims_It()
{
    let store = Claimed_By_The_Sibling_Suite();
    let edges = Counted(
        &store,
        "SELECT count(*) FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         JOIN suites s ON s.uid = t.suite_uid
         WHERE f.node_id = 'ARC-ECOSYSTEM-001' AND r.relation_type = 'relates-to'
           AND t.node_id = 'D-090' AND s.authority_root = 0",
    );

    assert_eq!(
        store.Suite_Of("D-090").expect("queries"),
        Some(("xvpe-spec-seed".to_owned(), SuiteAuthority::Sibling)),
        "D-090 resolved to no sibling suite, so the edge to it says nothing about whose \
         decision it is"
    );
    assert_eq!(
        Title(&store, "D-090").as_deref(),
        Some("Reuse alone does not justify platform ownership"),
        "the placeholder kept its own name, so the sibling's record never arrived"
    );
    assert_eq!(
        edges, 1,
        "the cross-suite edge does not join a non-root suite, which is the whole distinction \
         P3-SIBLINGS bought"
    );
}

/// A seeded store in which the sibling suite has claimed the `D-090` placeholder as its own.
///
/// The suite is created here rather than ingested from an archive on purpose: the archives
/// are outside this repository and a test that cannot find its corpus passes. This one has
/// no corpus to miss.
fn Claimed_By_The_Sibling_Suite() -> SpecificationStore
{
    let mut store = Seeded();
    let suite = store
        .Put_Suite("xvpe-spec-seed", "XVPE specification seed", SuiteAuthority::Sibling)
        .expect("records the sibling suite");
    let node = store
        .Upsert_Node(NodeRow {
            node_id: "D-090",
            kind: "decision",
            authority: "canonical-normative-record",
            representation: "document",
            title: "Reuse alone does not justify platform ownership",
        })
        .expect("claims the placeholder");

    store.Assign_Suite(node, suite).expect("assigns");

    return store;
}
