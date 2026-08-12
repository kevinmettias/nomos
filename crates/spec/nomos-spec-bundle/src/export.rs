use crate::BundleError;
use crate::bundle::Bundle;
use crate::columns::Assert_Columns_Covered;
use crate::model::{
    Blob, BlobEncoding, DocumentRef, Lineage, Node, NodeAlias, NodeHistory, NormativeStatement,
    Omission, OrdinalRef, Record, RecordFrontMatter, RecordRelation, Relation, RelationType,
    SourceBlock, SourceDocument, SourceHeading, SourceTableRow, Submission, SubmissionGap,
    SubmissionValue, Suite, TableRowRef,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
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
pub fn Export(store: &SpecificationStore) -> Result<Bundle, BundleError>
{
    let connection = store.Connection();
    let records = Every_Record(connection)?;

    Assert_Complete(store, &records)?;
    Assert_Columns_Covered(connection, &records)?;

    return Bundle::New(store.Version(), records);
}

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
    Blobs(connection, records)?;
    Source_Documents(connection, records)?;
    Source_Headings(connection, records)?;
    Source_Blocks(connection, records)?;
    Source_Table_Rows(connection, records)?;

    return Ok(());
}

/// The graph the corpus was turned into, and the preservation ledger tying the two together.
fn The_Graph(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    Suites(connection, records)?;
    Nodes(connection, records)?;
    Node_Aliases(connection, records)?;
    Node_Histories(connection, records)?;
    Relation_Types(connection, records)?;
    Relations(connection, records)?;
    Normative_Statements(connection, records)?;
    Lineages(connection, records)?;
    Omissions(connection, records)?;
    Record_Front_Matter(connection, records)?;
    Record_Relations(connection, records)?;

    return Ok(());
}

/// What arrived through the submission door.
fn The_Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    Submissions(connection, records)?;
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
/// JSON column after the query and `Relations` reads two, so folding them in would mean a
/// helper with a hole in it rather than one concept.
fn Collect<Read>(
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
struct Columns<'row, 'statement>
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

fn Blobs(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement =
        connection.prepare("SELECT sha256, byte_length, content FROM blobs ORDER BY sha256")?;
    let rows = statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            let sha256: String = columns.Next()?;
            let byte_length: i64 = columns.Next()?;
            let content: Vec<u8> = columns.Next()?;
            return Ok((sha256, byte_length, content));
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (sha256, byte_length, content) in rows
    {
        let blob = A_Blob(sha256, byte_length, content);

        records.push(blob);
    }

    return Ok(());
}

/// A blob's bytes as the bundle spells them: the text itself where it is UTF-8, and base64
/// where it is not.
fn A_Blob(sha256: String, byte_length: i64, content: Vec<u8>) -> Record
{
    let (encoding, spelled) = match String::from_utf8(content)
    {
        Ok(text) => (BlobEncoding::Utf8, text),
        Err(error) => (BlobEncoding::Base64, STANDARD.encode(error.as_bytes())),
    };

    return Record::Blob(Blob {
        sha256,
        byte_length,
        encoding,
        content: spelled,
    });
}

/// A JSON column decoded, reported as a SQL failure because that is where it came from.
fn Decoded<Value: serde::de::DeserializeOwned>(json: &str) -> Result<Value, BundleError>
{
    return serde_json::from_str(json).map_err(|error| BundleError::Sql(error.to_string()));
}

fn Source_Documents(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT d.path, d.revision, b.sha256
         FROM source_documents d JOIN blobs b ON b.uid = d.blob_uid
         ORDER BY d.path, d.revision",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceDocument(SourceDocument {
                path: columns.Next()?,
                revision: columns.Next()?,
                blob_sha256: columns.Next()?,
            }));
        },
    );
}

fn Source_Headings(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT d.path, d.revision, h.ordinal, h.depth, h.title
         FROM source_headings h JOIN source_documents d ON d.uid = h.document_uid
         ORDER BY d.path, d.revision, h.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceHeading(SourceHeading {
                document: DocumentRef {
                    path: columns.Next()?,
                    revision: columns.Next()?,
                },
                ordinal: columns.Next()?,
                depth: columns.Next()?,
                title: columns.Next()?,
            }));
        },
    );
}

fn Source_Blocks(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT d.path, d.revision, b.ordinal, b.kind, b.heading_path, b.text,
                b.content_hash, b.normalized_hash
         FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
         ORDER BY d.path, d.revision, b.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SourceBlock(SourceBlock {
                document: DocumentRef {
                    path: columns.Next()?,
                    revision: columns.Next()?,
                },
                ordinal: columns.Next()?,
                kind: columns.Next()?,
                heading_path: columns.Next()?,
                text: columns.Next()?,
                content_hash: columns.Next()?,
                normalized_hash: columns.Next()?,
            }));
        },
    );
}

const TABLE_ROWS: &str =
    "SELECT d.path, d.revision, b.ordinal, r.ordinal, r.table_ordinal, r.kind,
            r.cells_json, r.text, r.content_hash, r.normalized_hash
     FROM source_table_rows r
     JOIN source_blocks b ON b.uid = r.source_block_uid
     JOIN source_documents d ON d.uid = b.document_uid
     ORDER BY d.path, d.revision, b.ordinal, r.ordinal";

fn Source_Table_Rows(connection: &Connection, records: &mut Vec<Record>)
    -> Result<(), BundleError>
{
    let mut statement = connection.prepare(TABLE_ROWS)?;
    let rows = statement
        .query_map([], Read_A_Table_Row)?
        .collect::<Result<Vec<_>, _>>()?;
    for (mut record, cells) in rows
    {
        record.cells = Decoded(&cells)?;
        records.push(Record::SourceTableRow(record));
    }

    return Ok(());
}

/// One row and the JSON cells column that travels beside it, still undecoded.
fn Read_A_Table_Row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(SourceTableRow, String)>
{
    let mut columns = Columns::Of(row);
    let block = OrdinalRef {
        document: DocumentRef {
            path: columns.Next()?,
            revision: columns.Next()?,
        },
        ordinal: columns.Next()?,
    };
    let ordinal = columns.Next()?;
    let table_ordinal = columns.Next()?;
    let kind = columns.Next()?;
    let cells: String = columns.Next()?;

    return Ok((
        SourceTableRow {
            block,
            ordinal,
            table_ordinal,
            kind,
            cells: Vec::new(),
            text: columns.Next()?,
            content_hash: columns.Next()?,
            normalized_hash: columns.Next()?,
        },
        cells,
    ));
}

fn Suites(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT suite_id, title, authority_root FROM suites ORDER BY suite_id",
        |row| {
            let mut columns = Columns::Of(row);
            let suite_id = columns.Next()?;
            let title = columns.Next()?;
            let root: i64 = columns.Next()?;

            return Ok(Record::Suite(Suite {
                suite_id,
                title,
                authority_root: root != 0,
            }));
        },
    );
}

fn Nodes(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, n.deleted_at,
                s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         ORDER BY n.node_id",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::Node(Node {
                node_id: columns.Next()?,
                kind: columns.Next()?,
                authority: columns.Next()?,
                representation: columns.Next()?,
                title: columns.Next()?,
                deleted_at: columns.Next()?,
                suite_id: columns.Next()?,
            }));
        },
    );
}

fn Node_Aliases(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT a.alias, n.node_id
         FROM node_aliases a JOIN nodes n ON n.uid = a.node_uid
         ORDER BY a.alias",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::NodeAlias(NodeAlias {
                alias: columns.Next()?,
                node_id: columns.Next()?,
            }));
        },
    );
}

fn Node_Histories(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT n.node_id, h.ordinal, h.event, h.reason, h.previous_event_hash,
                h.event_hash, h.recorded_at
         FROM node_history h JOIN nodes n ON n.uid = h.node_uid
         ORDER BY n.node_id, h.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::NodeHistory(NodeHistory {
                node_id: columns.Next()?,
                ordinal: columns.Next()?,
                event: columns.Next()?,
                reason: columns.Next()?,
                previous_event_hash: columns.Next()?,
                event_hash: columns.Next()?,
                recorded_at: columns.Next()?,
            }));
        },
    );
}

fn Relation_Types(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT name, tier, inverse_of FROM relation_types ORDER BY name",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::RelationType(RelationType {
                name: columns.Next()?,
                tier: columns.Next()?,
                inverse_of: columns.Next()?,
            }));
        },
    );
}

fn Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT f.node_id, r.relation_type, t.node_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         ORDER BY f.node_id, r.relation_type, t.node_id",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::Relation(Relation {
                from_node_id: columns.Next()?,
                relation_type: columns.Next()?,
                to_node_id: columns.Next()?,
            }));
        },
    );
}

fn Normative_Statements(
    connection: &Connection,
    records: &mut Vec<Record>,
) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT s.statement_id, n.node_id, s.kind, s.canonical_text, s.canonical_hash,
                s.supersedes_hash
         FROM normative_statements s JOIN nodes n ON n.uid = s.node_uid
         ORDER BY s.statement_id",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::NormativeStatement(NormativeStatement {
                statement_id: columns.Next()?,
                node_id: columns.Next()?,
                kind: columns.Next()?,
                canonical_text: columns.Next()?,
                canonical_hash: columns.Next()?,
                supersedes_hash: columns.Next()?,
            }));
        },
    );
}

/// Every join here is a LEFT JOIN because both source references are nullable. An inner
/// join would drop exactly the rows that record a disposition and nothing else, which is
/// the same silent-loss shape this crate exists to make impossible.
fn Lineages(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT bd.path, bd.revision, b.ordinal,
                hd.path, hd.revision, h.ordinal,
                rd.path, rd.revision, rb.ordinal, r.ordinal,
                l.disposition, n.node_id, s.statement_id
         FROM lineage l
         LEFT JOIN source_blocks b ON b.uid = l.source_block_uid
         LEFT JOIN source_documents bd ON bd.uid = b.document_uid
         LEFT JOIN source_headings h ON h.uid = l.source_heading_uid
         LEFT JOIN source_documents hd ON hd.uid = h.document_uid
         LEFT JOIN source_table_rows r ON r.uid = l.source_table_row_uid
         LEFT JOIN source_blocks rb ON rb.uid = r.source_block_uid
         LEFT JOIN source_documents rd ON rd.uid = rb.document_uid
         LEFT JOIN nodes n ON n.uid = l.target_node_uid
         LEFT JOIN normative_statements s ON s.uid = l.target_statement
         ORDER BY coalesce(bd.path, ''), coalesce(bd.revision, ''), coalesce(b.ordinal, -1),
                  coalesce(hd.path, ''), coalesce(hd.revision, ''), coalesce(h.ordinal, -1),
                  coalesce(rd.path, ''), coalesce(rd.revision, ''), coalesce(rb.ordinal, -1),
                  coalesce(r.ordinal, -1),
                  l.disposition, coalesce(n.node_id, ''), coalesce(s.statement_id, '')",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::Lineage(Lineage {
                source_block: Ordinal_Reference(&mut columns)?,
                source_heading: Ordinal_Reference(&mut columns)?,
                source_table_row: Table_Row_Reference(&mut columns)?,
                disposition: columns.Next()?,
                target_node_id: columns.Next()?,
                target_statement_id: columns.Next()?,
            }));
        },
    );
}

fn Omissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT bd.path, bd.revision, b.ordinal,
                hd.path, hd.revision, h.ordinal,
                o.reason, o.justification, o.decision_record
         FROM omissions o
         LEFT JOIN source_blocks b ON b.uid = o.source_block_uid
         LEFT JOIN source_documents bd ON bd.uid = b.document_uid
         LEFT JOIN source_headings h ON h.uid = o.source_heading_uid
         LEFT JOIN source_documents hd ON hd.uid = h.document_uid
         ORDER BY coalesce(bd.path, ''), coalesce(bd.revision, ''), coalesce(b.ordinal, -1),
                  coalesce(hd.path, ''), coalesce(hd.revision, ''), coalesce(h.ordinal, -1),
                  o.reason, o.justification, o.decision_record",
    )?;
    let rows = statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            return Ok(Omission {
                source_block: Ordinal_Reference(&mut columns)?,
                source_heading: Ordinal_Reference(&mut columns)?,
                reason: columns.Next()?,
                justification: columns.Next()?,
                decision_record: columns.Next()?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Omission));
    return Ok(());
}

/// The four columns a table-row reference spans, taken in the order a query names them.
fn Table_Row_Reference(columns: &mut Columns<'_, '_>) -> rusqlite::Result<Option<TableRowRef>>
{
    let block = Ordinal_Reference(columns)?;
    let ordinal: Option<i64> = columns.Next()?;

    return Ok(match (block, ordinal)
    {
        (Some(block), Some(ordinal)) => Some(TableRowRef { block, ordinal }),
        _ => None,
    });
}

/// The three columns an ordinal reference spans, taken in the order a query names them.
fn Ordinal_Reference(columns: &mut Columns<'_, '_>) -> rusqlite::Result<Option<OrdinalRef>>
{
    let path: Option<String> = columns.Next()?;
    let revision: Option<String> = columns.Next()?;
    let ordinal: Option<i64> = columns.Next()?;

    return Ok(match (path, revision, ordinal)
    {
        (Some(path), Some(revision), Some(ordinal)) => Some(OrdinalRef {
            document: DocumentRef { path, revision },
            ordinal,
        }),
        _ => None,
    });
}

/// The declared front matter, addressed by the document that declared it.
const FRONT_MATTER: &str =
    "SELECT d.path, d.revision, n.node_id, f.status, f.version, f.tags_json
     FROM record_front_matter f
     JOIN source_documents d ON d.uid = f.document_uid
     JOIN nodes n ON n.uid = f.node_uid
     ORDER BY d.path, d.revision";

fn Record_Front_Matter(
    connection: &Connection,
    records: &mut Vec<Record>,
) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(FRONT_MATTER)?;
    let rows = statement
        .query_map([], Read_Front_Matter)?
        .collect::<Result<Vec<_>, _>>()?;
    for (mut record, tags) in rows
    {
        record.tags = Decoded(&tags)?;
        records.push(Record::RecordFrontMatter(record));
    }

    return Ok(());
}

/// One row and the JSON tags column that travels beside it, still undecoded.
fn Read_Front_Matter(row: &rusqlite::Row<'_>) -> rusqlite::Result<(RecordFrontMatter, String)>
{
    let mut columns = Columns::Of(row);
    let record = RecordFrontMatter {
        document: DocumentRef {
            path: columns.Next()?,
            revision: columns.Next()?,
        },
        node_id: columns.Next()?,
        status: columns.Next()?,
        version: columns.Next()?,
        tags: Vec::new(),
    };
    let tags: String = columns.Next()?;

    return Ok((record, tags));
}

/// The declared relations, in the order the record declared them.
fn Record_Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, r.ordinal, r.target, r.relation
         FROM record_relations r
         JOIN source_documents d ON d.uid = r.document_uid
         ORDER BY d.path, d.revision, r.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            return Ok(RecordRelation {
                document: DocumentRef {
                    path: columns.Next()?,
                    revision: columns.Next()?,
                },
                ordinal: columns.Next()?,
                target: columns.Next()?,
                relation: columns.Next()?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::RecordRelation));
    return Ok(());
}

/// The submissions, ordered by the node they are.
fn Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
        "SELECT n.node_id, s.kind, s.form_contract_version, s.state, s.submitted_by,
                s.submitted_through
         FROM submissions s
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::Submission(Submission {
                node_id: columns.Next()?,
                kind: columns.Next()?,
                form_contract_version: columns.Next()?,
                state: columns.Next()?,
                submitted_by: columns.Next()?,
                submitted_through: columns.Next()?,
            }));
        },
    );
}

/// Every value of every field, in the order that makes the last one the current reading.
fn Submission_Values(connection: &Connection, records: &mut Vec<Record>)
-> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT n.node_id, v.field, v.ordinal, v.origin, v.value, v.value_hash,
                v.supersedes_hash, v.recorded_at
         FROM submission_values v
         JOIN submissions s ON s.uid = v.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, v.field, v.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            return Ok(SubmissionValue {
                node_id: columns.Next()?,
                field: columns.Next()?,
                ordinal: columns.Next()?,
                origin: columns.Next()?,
                value: columns.Next()?,
                value_hash: columns.Next()?,
                supersedes_hash: columns.Next()?,
                recorded_at: columns.Next()?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SubmissionValue));
    return Ok(());
}

/// The decision gaps, open and closed alike.
///
/// A closed gap travels too. `OD-SPEC-010` closes one by a citation and never by deletion, so
/// a bundle that dropped them would lose the record that a question was ever asked.
fn Submission_Gaps(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    return Collect(
        connection,
        records,
"SELECT n.node_id, g.ordinal, g.question, g.blocks, g.severity, g.closed_by
         FROM submission_gaps g
         JOIN submissions s ON s.uid = g.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, g.ordinal",
        |row| {
            let mut columns = Columns::Of(row);
            return Ok(Record::SubmissionGap(SubmissionGap {
                node_id: columns.Next()?,
                ordinal: columns.Next()?,
                question: columns.Next()?,
                blocks: columns.Next()?,
                severity: columns.Next()?,
                closed_by: columns.Next()?,
            }));
        },
    );
}
