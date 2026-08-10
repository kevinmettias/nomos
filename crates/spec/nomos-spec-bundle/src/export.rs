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
    let mut records: Vec<Record> = Vec::new();

    Blobs(connection, &mut records)?;
    Source_Documents(connection, &mut records)?;
    Source_Headings(connection, &mut records)?;
    Source_Blocks(connection, &mut records)?;
    Source_Table_Rows(connection, &mut records)?;
    Suites(connection, &mut records)?;
    Nodes(connection, &mut records)?;
    Node_Aliases(connection, &mut records)?;
    Node_Histories(connection, &mut records)?;
    Relation_Types(connection, &mut records)?;
    Relations(connection, &mut records)?;
    Normative_Statements(connection, &mut records)?;
    Lineages(connection, &mut records)?;
    Omissions(connection, &mut records)?;
    Record_Front_Matter(connection, &mut records)?;
    Record_Relations(connection, &mut records)?;
    Submissions(connection, &mut records)?;
    Submission_Values(connection, &mut records)?;
    Submission_Gaps(connection, &mut records)?;

    Assert_Complete(store, &records)?;
    Assert_Columns_Covered(connection, &records)?;

    return Bundle::New(store.Version(), records);
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

fn Blobs(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement =
        connection.prepare("SELECT sha256, byte_length, content FROM blobs ORDER BY sha256")?;
    let rows = statement
        .query_map([], |row| {
            let sha256: String = row.get(0)?;
            let byte_length: i64 = row.get(1)?;
            let content: Vec<u8> = row.get(2)?;
            return Ok((sha256, byte_length, content));
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (sha256, byte_length, content) in rows
    {
        let (encoding, spelled) = match String::from_utf8(content)
        {
            Ok(text) => (BlobEncoding::Utf8, text),
            Err(error) => (BlobEncoding::Base64, STANDARD.encode(error.as_bytes())),
        };

        records.push(Record::Blob(Blob {
            sha256,
            byte_length,
            encoding,
            content: spelled,
        }));
    }

    return Ok(());
}

fn Source_Documents(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, b.sha256
         FROM source_documents d JOIN blobs b ON b.uid = d.blob_uid
         ORDER BY d.path, d.revision",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(SourceDocument {
                path: row.get(0)?,
                revision: row.get(1)?,
                blob_sha256: row.get(2)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SourceDocument));
    return Ok(());
}

fn Source_Headings(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, h.ordinal, h.depth, h.title
         FROM source_headings h JOIN source_documents d ON d.uid = h.document_uid
         ORDER BY d.path, d.revision, h.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(SourceHeading {
                document: DocumentRef {
                    path: row.get(0)?,
                    revision: row.get(1)?,
                },
                ordinal: row.get(2)?,
                depth: row.get(3)?,
                title: row.get(4)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SourceHeading));
    return Ok(());
}

fn Source_Blocks(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, b.ordinal, b.kind, b.heading_path, b.text,
                b.content_hash, b.normalized_hash
         FROM source_blocks b JOIN source_documents d ON d.uid = b.document_uid
         ORDER BY d.path, d.revision, b.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(SourceBlock {
                document: DocumentRef {
                    path: row.get(0)?,
                    revision: row.get(1)?,
                },
                ordinal: row.get(2)?,
                kind: row.get(3)?,
                heading_path: row.get(4)?,
                text: row.get(5)?,
                content_hash: row.get(6)?,
                normalized_hash: row.get(7)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SourceBlock));
    return Ok(());
}

fn Source_Table_Rows(connection: &Connection, records: &mut Vec<Record>)
    -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, b.ordinal, r.ordinal, r.table_ordinal, r.kind,
                r.cells_json, r.text, r.content_hash, r.normalized_hash
         FROM source_table_rows r
         JOIN source_blocks b ON b.uid = r.source_block_uid
         JOIN source_documents d ON d.uid = b.document_uid
         ORDER BY d.path, d.revision, b.ordinal, r.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            let cells: String = row.get(6)?;
            return Ok((
                SourceTableRow {
                    block: OrdinalRef {
                        document: DocumentRef {
                            path: row.get(0)?,
                            revision: row.get(1)?,
                        },
                        ordinal: row.get(2)?,
                    },
                    ordinal: row.get(3)?,
                    table_ordinal: row.get(4)?,
                    kind: row.get(5)?,
                    cells: Vec::new(),
                    text: row.get(7)?,
                    content_hash: row.get(8)?,
                    normalized_hash: row.get(9)?,
                },
                cells,
            ));
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (mut record, cells) in rows
    {
        record.cells =
            serde_json::from_str(&cells).map_err(|error| BundleError::Sql(error.to_string()))?;
        records.push(Record::SourceTableRow(record));
    }

    return Ok(());
}

fn Suites(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement =
        connection.prepare("SELECT suite_id, title, authority_root FROM suites ORDER BY suite_id")?;
    let rows = statement
        .query_map([], |row| {
            let root: i64 = row.get(2)?;
            return Ok(Suite {
                suite_id: row.get(0)?,
                title: row.get(1)?,
                authority_root: root != 0,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Suite));
    return Ok(());
}

fn Nodes(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, n.deleted_at,
                s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         ORDER BY n.node_id",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(Node {
                node_id: row.get(0)?,
                kind: row.get(1)?,
                authority: row.get(2)?,
                representation: row.get(3)?,
                title: row.get(4)?,
                deleted_at: row.get(5)?,
                suite_id: row.get(6)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Node));
    return Ok(());
}

fn Node_Aliases(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT a.alias, n.node_id
         FROM node_aliases a JOIN nodes n ON n.uid = a.node_uid
         ORDER BY a.alias",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(NodeAlias {
                alias: row.get(0)?,
                node_id: row.get(1)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::NodeAlias));
    return Ok(());
}

fn Node_Histories(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT n.node_id, h.ordinal, h.event, h.reason, h.previous_event_hash,
                h.event_hash, h.recorded_at
         FROM node_history h JOIN nodes n ON n.uid = h.node_uid
         ORDER BY n.node_id, h.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(NodeHistory {
                node_id: row.get(0)?,
                ordinal: row.get(1)?,
                event: row.get(2)?,
                reason: row.get(3)?,
                previous_event_hash: row.get(4)?,
                event_hash: row.get(5)?,
                recorded_at: row.get(6)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::NodeHistory));
    return Ok(());
}

fn Relation_Types(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement =
        connection.prepare("SELECT name, tier, inverse_of FROM relation_types ORDER BY name")?;
    let rows = statement
        .query_map([], |row| {
            return Ok(RelationType {
                name: row.get(0)?,
                tier: row.get(1)?,
                inverse_of: row.get(2)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::RelationType));
    return Ok(());
}

fn Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT f.node_id, r.relation_type, t.node_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         ORDER BY f.node_id, r.relation_type, t.node_id",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(Relation {
                from_node_id: row.get(0)?,
                relation_type: row.get(1)?,
                to_node_id: row.get(2)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Relation));
    return Ok(());
}

fn Normative_Statements(
    connection: &Connection,
    records: &mut Vec<Record>,
) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT s.statement_id, n.node_id, s.kind, s.canonical_text, s.canonical_hash,
                s.supersedes_hash
         FROM normative_statements s JOIN nodes n ON n.uid = s.node_uid
         ORDER BY s.statement_id",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(NormativeStatement {
                statement_id: row.get(0)?,
                node_id: row.get(1)?,
                kind: row.get(2)?,
                canonical_text: row.get(3)?,
                canonical_hash: row.get(4)?,
                supersedes_hash: row.get(5)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::NormativeStatement));
    return Ok(());
}

/// Every join here is a LEFT JOIN because both source references are nullable. An inner
/// join would drop exactly the rows that record a disposition and nothing else, which is
/// the same silent-loss shape this crate exists to make impossible.
fn Lineages(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
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
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(Lineage {
                source_block: Ordinal_Reference(row, 0, 1, 2)?,
                source_heading: Ordinal_Reference(row, 3, 4, 5)?,
                source_table_row: Table_Row_Reference(row, 6, 7, 8, 9)?,
                disposition: row.get(10)?,
                target_node_id: row.get(11)?,
                target_statement_id: row.get(12)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Lineage));
    return Ok(());
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
            return Ok(Omission {
                source_block: Ordinal_Reference(row, 0, 1, 2)?,
                source_heading: Ordinal_Reference(row, 3, 4, 5)?,
                reason: row.get(6)?,
                justification: row.get(7)?,
                decision_record: row.get(8)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Omission));
    return Ok(());
}

fn Table_Row_Reference(
    row: &rusqlite::Row<'_>,
    path: usize,
    revision: usize,
    block_ordinal: usize,
    row_ordinal: usize,
) -> rusqlite::Result<Option<TableRowRef>>
{
    let block = Ordinal_Reference(row, path, revision, block_ordinal)?;
    let ordinal: Option<i64> = row.get(row_ordinal)?;

    return Ok(match (block, ordinal)
    {
        (Some(block), Some(ordinal)) => Some(TableRowRef { block, ordinal }),
        _ => None,
    });
}

fn Ordinal_Reference(
    row: &rusqlite::Row<'_>,
    path: usize,
    revision: usize,
    ordinal: usize,
) -> rusqlite::Result<Option<OrdinalRef>>
{
    let path: Option<String> = row.get(path)?;
    let revision: Option<String> = row.get(revision)?;
    let ordinal: Option<i64> = row.get(ordinal)?;

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
fn Record_Front_Matter(
    connection: &Connection,
    records: &mut Vec<Record>,
) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, n.node_id, f.status, f.version, f.tags_json
         FROM record_front_matter f
         JOIN source_documents d ON d.uid = f.document_uid
         JOIN nodes n ON n.uid = f.node_uid
         ORDER BY d.path, d.revision",
    )?;
    let rows = statement
        .query_map([], |row| {
            let tags: String = row.get(5)?;
            return Ok((
                RecordFrontMatter {
                    document: DocumentRef {
                        path: row.get(0)?,
                        revision: row.get(1)?,
                    },
                    node_id: row.get(2)?,
                    status: row.get(3)?,
                    version: row.get(4)?,
                    tags: Vec::new(),
                },
                tags,
            ));
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for (mut record, tags) in rows
    {
        record.tags =
            serde_json::from_str(&tags).map_err(|error| BundleError::Sql(error.to_string()))?;
        records.push(Record::RecordFrontMatter(record));
    }

    return Ok(());
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
            return Ok(RecordRelation {
                document: DocumentRef {
                    path: row.get(0)?,
                    revision: row.get(1)?,
                },
                ordinal: row.get(2)?,
                target: row.get(3)?,
                relation: row.get(4)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::RecordRelation));
    return Ok(());
}

/// The submissions, ordered by the node they are.
fn Submissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let mut statement = connection.prepare(
        "SELECT n.node_id, s.kind, s.form_contract_version, s.state, s.submitted_by,
                s.submitted_through
         FROM submissions s
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(Submission {
                node_id: row.get(0)?,
                kind: row.get(1)?,
                form_contract_version: row.get(2)?,
                state: row.get(3)?,
                submitted_by: row.get(4)?,
                submitted_through: row.get(5)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::Submission));
    return Ok(());
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
            return Ok(SubmissionValue {
                node_id: row.get(0)?,
                field: row.get(1)?,
                ordinal: row.get(2)?,
                origin: row.get(3)?,
                value: row.get(4)?,
                value_hash: row.get(5)?,
                supersedes_hash: row.get(6)?,
                recorded_at: row.get(7)?,
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
    let mut statement = connection.prepare(
        "SELECT n.node_id, g.ordinal, g.question, g.blocks, g.severity, g.closed_by
         FROM submission_gaps g
         JOIN submissions s ON s.uid = g.submission_uid
         JOIN nodes n ON n.uid = s.node_uid
         ORDER BY n.node_id, g.ordinal",
    )?;
    let rows = statement
        .query_map([], |row| {
            return Ok(SubmissionGap {
                node_id: row.get(0)?,
                ordinal: row.get(1)?,
                question: row.get(2)?,
                blocks: row.get(3)?,
                severity: row.get(4)?,
                closed_by: row.get(5)?,
            });
        })?
        .collect::<Result<Vec<_>, _>>()?;

    records.extend(rows.into_iter().map(Record::SubmissionGap));
    return Ok(());
}
