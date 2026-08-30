//! Writing a whole specification store out as one portable bundle.

mod graph;
mod source;
mod submission;

use graph::{
    Collect_Lineages, Collect_Nodes, Collect_Omissions, Collect_Relations, Collect_Suites,
    Node_Aliases, Node_Histories, Normative_Statements, Record_Front_Matter, Record_Relations,
    Relation_Types,
};
use source::{
    Collect_Blobs, Source_Blocks, Source_Documents, Source_Headings, Source_Table_Rows,
};
use submission::{Collect_Submissions, Submission_Gaps, Submission_Values};

use crate::BundleError;
use crate::Bundle;
use crate::Record;
use nomos_spec_store::{SpecificationStore, Table};
use rusqlite::Connection;

/// Writes the whole store out as text.
///
/// Every ordering is by natural key rather than by `uid`, so a bundle exported from a
/// database built by importing a bundle comes back in the same order even though the
/// surrogates were assigned differently.
///
/// Two completeness guards, because they catch two different losses. The row guard counts
/// rows and cannot see a column: a table whose every row is exported one field short
/// passes it exactly. The column guard is the one that sees that.
///
/// # Errors
///
/// Returns [`BundleError::Incomplete`] if any table holds rows this function did not
/// emit, [`BundleError::UncoveredColumn`] if the schema holds a column the exporter does
/// not carry, and [`BundleError::Sql`] on any query failure.
pub fn Export_Store(store: &SpecificationStore) -> Result<Bundle, BundleError>
{
    use crate::columns::Assert_Columns_Covered;

    let connection = store.Connection();
    let records = Every_Record(connection)?;

    Assert_Complete(store, &records)?;
    Assert_Columns_Covered(connection, &records)?;

    return Bundle::New(store.Version(), records);
}

// Renamed from the vague `Export` for check-naming-clarity (it takes a parameter without
// saying what it exports). The public name stays `Export` through this alias because
// `lib.rs`'s `pub use export::Export;` is on this campaign's central-exclusion list — the
// orchestrating session owns that file, not this crate-scoped worker.
pub use self::Export_Store as Export;

/// Every table, in the order the bundle carries them. The order is the bundle's identity, so
/// moving a call here changes the bytes every store exports.
fn Every_Record(connection: &Connection) -> Result<Vec<Record>, BundleError>
{
    let mut records: Vec<Record> = Vec::new();

    The_Source_Corpus(connection, &mut records)?;
    The_Graph(connection, &mut records)?;
    The_Submissions(connection, &mut records)?;

    return Ok(records);
}

/// The corpus as it was read: the blobs, the documents, and everything segmented out of them.
fn The_Source_Corpus(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    Collect_Blobs(connection, records)?;
    Source_Documents(connection, records)?;
    Source_Headings(connection, records)?;
    Source_Blocks(connection, records)?;
    Source_Table_Rows(connection, records)?;

    return Ok(());
}

/// The graph the corpus was turned into, and the preservation ledger tying the two together.
fn The_Graph(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    Collect_Suites(connection, records)?;
    Collect_Nodes(connection, records)?;
    Node_Aliases(connection, records)?;
    Node_Histories(connection, records)?;
    Relation_Types(connection, records)?;
    Collect_Relations(connection, records)?;
    Normative_Statements(connection, records)?;
    Collect_Lineages(connection, records)?;
    Collect_Omissions(connection, records)?;
    Record_Front_Matter(connection, records)?;
    Record_Relations(connection, records)?;

    return Ok(());
}

/// What arrived through the submission door.
fn The_Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    Collect_Submissions(connection, records)?;
    Submission_Values(connection, records)?;
    Submission_Gaps(connection, records)?;

    return Ok(());
}

/// Every row in the store reached the bundle.
///
/// Without this a new table joins the schema, nothing exports it, and the round trip
/// still passes — because both sides are equally blind to it.
fn Assert_Complete(store: &SpecificationStore, records: &[Record]) -> Result<(), BundleError>
{
    for table in Table::All()
    {
        let in_store = store.Count(*table)?;
        let exported = u32::try_from(
            records
                .iter()
                .filter(|record| record.Table() == table.Name())
                .count(),
        )
        .unwrap_or(u32::MAX);

        if in_store != exported
        {
            return Err(BundleError::Incomplete {
                table: table.Name().to_owned(),
                in_store,
                exported,
            });
        }
    }

    return Ok(());
}

/// Runs one query and files every row it returns as a record.
///
/// Every uniform exporter below is this shape, and the shape is the whole of what they
/// share: prepare, read each row into its own type, wrap it in the variant that carries it,
/// extend the list. Written out per table it was the same twelve lines of plumbing around
/// the two that differ, and the SQL was the hardest thing on the screen to find.
///
/// The exporters that are not uniform keep their own bodies. `Source_Table_Rows` decodes a
/// JSON column after the query and `Collect_Relations` reads two, so folding them in would mean a
/// helper with a hole in it rather than one concept.
pub(super) fn Collect_Rows<Read>(
    connection: &Connection,
    records: &mut Vec<Record>,
    sql: &str,
    read: Read,
) -> Result<(), BundleError>
where
    Read: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Record>,
{
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], read)?.collect::<Result<Vec<_>, _>>()?;

    records.extend(rows);
    return Ok(());
}

/// A row read left to right, so a column's place is the order it is asked for rather than a
/// number typed beside the SELECT that chose it.
///
/// The number and the query drift apart in silence. A column inserted into a SELECT renumbers
/// every column after it, and nothing in the language ties `row.get(7)` to the eighth name in a
/// string literal twenty lines up — the reader keeps compiling and starts filling the wrong
/// fields. Asking in order leaves the SELECT as the only place the order is stated, which is
/// where a reader was going to look anyway.
pub(super) struct Columns<'row, 'statement>
{
    row: &'row rusqlite::Row<'statement>,
    next: usize,
}

impl<'row, 'statement> Columns<'row, 'statement>
{
    fn Of(row: &'row rusqlite::Row<'statement>) -> Self
    {
        return Self { row, next: 0 };
    }

    /// The next column the query names, as whatever type receives it.
    fn Next<Value: rusqlite::types::FromSql>(&mut self) -> rusqlite::Result<Value>
    {
        let at = self.next;

        self.next = at.saturating_add(1);
        return self.row.get(at);
    }
}

/// A JSON column decoded, reported as a SQL failure because that is where it came from.
pub(super) fn Decode_Json_Column<Value: serde::de::DeserializeOwned>(json: &str) -> Result<Value, BundleError>
{
    return serde_json::from_str(json).map_err(|error| BundleError::Sql(error.to_string()));
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_Export_Store_Should_Produce_An_Empty_Bundle_For_A_Fresh_Store()
    {
        let store = SpecificationStore::In_Memory().expect("opens");

        let bundle = Export_Store(&store).expect("exports");

        assert!(bundle.Records().is_empty());
        assert_eq!(bundle.Manifest().records, 0);
    }

    #[test]
    fn Test_Collect_Rows_Should_Append_Every_Row_The_Query_Returns()
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute(
                "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
                rusqlite::params!["sha256:aa", 2i64, b"hi".as_slice()],
            )
            .expect("inserts");

        let mut records = Vec::new();
        Collect_Rows(store.Connection(), &mut records, "SELECT sha256 FROM blobs", |row| {
            let sha256: String = row.get(0)?;
            return Ok(Record::Blob(crate::Blob {
                sha256,
                byte_length: 2,
                encoding: crate::Encoding::Utf8,
                content: "hi".to_owned(),
            }));
        })
        .expect("collects");

        assert_eq!(
            records,
            vec![Record::Blob(crate::Blob {
                sha256: "sha256:aa".to_owned(),
                byte_length: 2,
                encoding: crate::Encoding::Utf8,
                content: "hi".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Decode_Json_Column_Should_Deserialize_A_Vec_Of_Strings()
    {
        let decoded: Vec<String> = Decode_Json_Column(r#"["a","b"]"#).expect("decodes");
        assert_eq!(decoded, vec!["a".to_owned(), "b".to_owned()]);

        let refusal = Decode_Json_Column::<Vec<String>>("not json").expect_err("malformed JSON must be refused");
        assert!(matches!(refusal, BundleError::Sql(_)), "{refusal}");
    }
}
