//! Reading the graph out: the nodes, what joins them, and what they were restored from.

use crate::BundleError;
use crate::row::reference::document::DocumentRef;
use crate::bundle::lineage::Lineage;
use crate::row::node::Node;
use crate::row::node::alias::NodeAlias;
use crate::row::node::history::NodeHistory;
use crate::row::normative_statement::NormativeStatement;
use crate::bundle::omission::Omission;
use crate::row::reference::ordinal::OrdinalRef;
use crate::row::record::Record;
use crate::row::record::front_matter::RecordFrontMatter;
use crate::row::record::relation::RecordRelation;
use crate::row::relation::Relation;
use crate::row::relation::kind::RelationType;
use crate::row::suite::Suite;
use crate::row::reference::table_row::TableRowRef;
use rusqlite::Connection;

use super::{Collect, Columns, Decoded};

pub(super) fn Suites(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Nodes(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Node_Aliases(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Node_Histories(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Relation_Types(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Normative_Statements(
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
pub(super) fn Lineages(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Omissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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

pub(super) fn Record_Front_Matter(
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
pub(super) fn Record_Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
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
