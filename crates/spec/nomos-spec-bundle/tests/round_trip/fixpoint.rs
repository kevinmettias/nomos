//! The round trip is a fixpoint, and it is one for reasons other than luck.

use crate::populated::{BINARY, Populated, Populated_In_Reverse};
use nomos_spec_bundle::{Bundle, Export, Import};
use nomos_spec_store::{SpecificationStore, Table};

/// The guard against a vacuous round trip.
#[test]
fn Test_Every_Table_Should_Be_Exercised()
{
    let store = Populated();

    let empty: Vec<&str> = Table::All()
        .iter()
        .filter(|table| store.Count(**table).unwrap_or(0) == 0)
        .map(|table| table.Name())
        .collect();

    assert!(
        empty.is_empty(),
        "these tables hold nothing, so the round trip says nothing about them: {empty:?}"
    );
}

/// P1.
#[test]
fn Test_A_Bundle_Should_Survive_A_Database_Round_Trip_Byte_For_Byte()
{
    let source = Populated();
    let first = Export(&source).expect("exports").Write().expect("writes");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    let report = Import(&mut rebuilt, &Bundle::Parse(&first).expect("parses")).expect("imports");

    let second = Export(&rebuilt).expect("re-exports").Write().expect("writes");

    assert_eq!(first, second, "the bundle is not a fixpoint of the round trip");
    assert!(report.records > 20, "only {} record(s) round-tripped", report.records);
}

/// P1 does not catch ordering by `uid`, because exporting in surrogate order and then
/// importing in that same order is a fixpoint too. This is the test that does: the same
/// corpus written in a different order must produce the same bundle.
#[test]
fn Test_Two_Stores_Of_The_Same_Corpus_Should_Export_Identically()
{
    let forward = Populated_In_Reverse(false);
    let backward = Populated_In_Reverse(true);

    assert_ne!(
        Surrogates(&forward),
        Surrogates(&backward),
        "both stores assigned the same surrogates, so this test proved nothing"
    );

    let first = Export(&forward).expect("exports").Write().expect("writes");
    let second = Export(&backward).expect("exports").Write().expect("writes");

    assert_eq!(first, second, "the bundle is ordered by something that is not identity");
    assert!(
        !first.contains("\"uid\""),
        "a join surrogate reached the portable authority"
    );
}

fn Surrogates(store: &SpecificationStore) -> Vec<i64>
{
    return store
        .Connection()
        .prepare("SELECT uid FROM source_documents ORDER BY path")
        .and_then(|mut statement| {
            return statement
                .query_map([], |row| row.get(0))
                .and_then(std::iter::Iterator::collect);
        })
        .expect("reads uids");
}

/// Every row counted, both directions.
#[test]
fn Test_Import_Should_Land_Every_Row_The_Bundle_Declared()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &bundle).expect("imports");

    for table in Table::All()
    {
        assert_eq!(
            source.Count(*table).expect("counts"),
            rebuilt.Count(*table).expect("counts"),
            "{} did not survive the round trip",
            table.Name()
        );
    }
}

/// `OD-SPEC-012`'s constraint columns are not a column the row-count guard would catch —
/// the row is there either way, and only its values say whether the constraint survived.
/// `verifies` carries a non-trivial domain, range and cardinality in the populated fixture
/// specifically so this has something to assert against.
#[test]
fn Test_A_Relation_Types_Constraint_Should_Survive_The_Round_Trip()
{
    let source = Populated();
    let bundle = Export(&source).expect("exports");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &bundle).expect("imports");

    let (domain, range, max_per_node): (String, String, i64) = rebuilt
        .Connection()
        .query_row(
            "SELECT domain_kinds_json, range_kinds_json, max_per_node FROM relation_types
             WHERE name = 'verifies'",
            [],
            |row| return Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("reads the constraint back");

    assert_eq!(domain, "[\"concept\"]", "the domain did not survive the round trip");
    assert_eq!(range, "[\"requirement\"]", "the range did not survive the round trip");
    assert_eq!(max_per_node, 4, "the cardinality did not survive the round trip");
}

/// A blob that is not valid UTF-8 must come back byte-exact.
#[test]
fn Test_Binary_Blobs_Should_Survive_As_Bytes()
{
    let source = Populated();
    let text = Export(&source).expect("exports").Write().expect("writes");
    assert!(text.contains("\"base64\""), "the binary blob took the utf8 arm");

    let mut rebuilt = SpecificationStore::In_Memory().expect("opens");
    Import(&mut rebuilt, &Bundle::Parse(&text).expect("parses")).expect("imports");

    let restored: Vec<u8> = rebuilt
        .Connection()
        .query_row("SELECT content FROM blobs WHERE byte_length = 5", [], |row| {
            row.get(0)
        })
        .expect("reads the blob");

    assert_eq!(restored, BINARY);
}
