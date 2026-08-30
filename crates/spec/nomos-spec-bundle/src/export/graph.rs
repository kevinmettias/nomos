//! Reading the graph out: the nodes, what joins them, and what they were restored from.

use crate::BundleError;
use crate::DocumentRef;
use crate::OrdinalRef;
use crate::Record;
use crate::FrontMatter as RecordFrontMatter;
use crate::TableRowRef;
use rusqlite::Connection;

use super::{Collect_Rows, Columns, Decode_Json_Column};

pub(super) fn Collect_Suites(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::Suite;

    return Collect_Rows(
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

pub(super) fn Collect_Nodes(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::Node;

    return Collect_Rows(
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
    use crate::Alias as NodeAlias;

    return Collect_Rows(
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
    use crate::History as NodeHistory;

    return Collect_Rows(
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
    for row in Raw_Relation_Type_Rows(connection)?
    {
        Push_Relation_Type(records, row)?;
    }

    return Ok(());
}

/// A relation type row, still carrying its domain and range as undecoded JSON.
type RawRelationTypeRow = (String, String, Option<String>, String, String, u32);

/// Every relation type row, in the order named.
fn Raw_Relation_Type_Rows(connection: &Connection) -> Result<Vec<RawRelationTypeRow>, BundleError>
{
    let mut statement = connection.prepare(
        "SELECT name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node
         FROM relation_types ORDER BY name",
    )?;
    return Ok(statement
        .query_map([], |row| {
            let mut columns = Columns::Of(row);
            let name = columns.Next()?;
            let tier = columns.Next()?;
            let inverse_of = columns.Next()?;
            let domain_json: String = columns.Next()?;
            let range_json: String = columns.Next()?;
            let max_per_node = columns.Next()?;

            return Ok((name, tier, inverse_of, domain_json, range_json, max_per_node));
        })?
        .collect::<Result<Vec<_>, _>>()?);
}

/// One relation type row, decoded and appended.
fn Push_Relation_Type(records: &mut Vec<Record>, row: RawRelationTypeRow) -> Result<(), BundleError>
{
    use crate::Type as RelationType;
    let (name, tier, inverse_of, domain_json, range_json, max_per_node) = row;

    records.push(Record::RelationType(RelationType {
        name,
        tier,
        inverse_of,
        domain: Decode_Json_Column(&domain_json)?,
        range: Decode_Json_Column(&range_json)?,
        max_per_node,
    }));

    return Ok(());
}

pub(super) fn Collect_Relations(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::Relation;

    return Collect_Rows(
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
    use crate::NormativeStatement;

    return Collect_Rows(
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
pub(super) fn Collect_Lineages(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    use crate::Lineage;

    return Collect_Rows(
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

pub(super) fn Collect_Omissions(connection: &Connection, records: &mut Vec<Record>) -> Result<(), BundleError>
{
    let rows = Omission_Rows(connection)?;

    records.extend(rows.into_iter().map(Record::Omission));
    return Ok(());
}

/// Every omission row, in the order named.
fn Omission_Rows(connection: &Connection) -> Result<Vec<crate::Omission>, BundleError>
{
    use crate::Omission;

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
    return Ok(statement
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
        .collect::<Result<Vec<_>, _>>()?);
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
        record.tags = Decode_Json_Column(&tags)?;
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
    let rows = Record_Relation_Rows(connection)?;

    records.extend(rows.into_iter().map(Record::RecordRelation));
    return Ok(());
}

/// Every declared relation row, in the order the record declared them.
fn Record_Relation_Rows(connection: &Connection) -> Result<Vec<crate::row::record::relation::Relation>, BundleError>
{
    use crate::row::record::relation::Relation as RecordRelation;

    let mut statement = connection.prepare(
        "SELECT d.path, d.revision, r.ordinal, r.target, r.relation
         FROM record_relations r
         JOIN source_documents d ON d.uid = r.document_uid
         ORDER BY d.path, d.revision, r.ordinal",
    )?;
    return Ok(statement
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
        .collect::<Result<Vec<_>, _>>()?);
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_store::SpecificationStore;

    /// One row of everything this file reads: a suite, two nodes (one inside it and one
    /// not), an alias, a history entry, a relation type, a relation between the two nodes,
    /// a normative statement, a heading-anchored lineage row, a heading-anchored omission,
    /// and one record's declared front matter and one declared relation.
    fn Fixture() -> SpecificationStore
    {
        let store = SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute_batch(
                "INSERT INTO blobs (sha256, byte_length, content) VALUES ('sha256:aa', 2, x'6869');
                 INSERT INTO source_documents (path, revision, blob_uid) VALUES ('doc.md', 'v1', 1);
                 INSERT INTO source_headings (document_uid, ordinal, depth, title)
                     VALUES (1, 1, 1, 'Intro');
                 INSERT INTO suites (suite_id, title, authority_root)
                     VALUES ('nomos', 'The Nomos Specification', 1);
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N1', 'requirement', 'canonical', 'record', 'Node One', NULL, 1);
                 INSERT INTO nodes
                     (node_id, kind, authority, representation, title, deleted_at, suite_uid)
                     VALUES ('N2', 'concept', 'canonical', 'record', 'Node Two', NULL, NULL);
                 INSERT INTO node_aliases (alias, node_uid) VALUES ('N1-OLD', 1);
                 INSERT INTO node_history
                     (node_uid, ordinal, event, reason, previous_event_hash, event_hash, recorded_at)
                     VALUES (1, 1, 'created', 'seeded', NULL, 'sha256:01', '2026-01-01T00:00:00Z');
                 INSERT INTO relation_types
                     (name, tier, inverse_of, domain_kinds_json, range_kinds_json, max_per_node)
                     VALUES ('verifies', 'core', NULL, '[]', '[]', 4);
                 INSERT INTO relations (from_node_uid, relation_type, to_node_uid)
                     VALUES (2, 'verifies', 1);
                 INSERT INTO normative_statements
                     (node_uid, statement_id, kind, canonical_text, canonical_hash, supersedes_hash)
                     VALUES (1, 'STMT-1', 'Requirement', 'Text', 'sha256:aa', NULL);
                 INSERT INTO lineage (source_heading_uid, disposition, target_node_uid)
                     VALUES (1, 'preserved-normalized', 1);
                 INSERT INTO omissions (source_heading_uid, reason, justification, decision_record)
                     VALUES (1, 'superseded', 'replaced', 'D-1');
                 INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
                     VALUES (1, 1, 'accepted', 1, '[]');
                 INSERT INTO record_relations (document_uid, ordinal, target, relation)
                     VALUES (1, 1, 'N2', 'affects');",
            )
            .expect("populates every table this file reads");

        return store;
    }

    #[test]
    fn Test_Collect_Suites_Should_Read_A_Suite_By_Its_Natural_Key()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Suites(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::Suite(crate::Suite {
                suite_id: "nomos".to_owned(),
                title: "The Nomos Specification".to_owned(),
                authority_root: true,
            })]
        );
    }

    #[test]
    fn Test_Collect_Nodes_Should_Read_A_Node_And_The_Suite_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Nodes(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![
                Record::Node(crate::Node {
                    node_id: "N1".to_owned(),
                    kind: "requirement".to_owned(),
                    authority: "canonical".to_owned(),
                    representation: "record".to_owned(),
                    title: "Node One".to_owned(),
                    deleted_at: None,
                    suite_id: Some("nomos".to_owned()),
                }),
                Record::Node(crate::Node {
                    node_id: "N2".to_owned(),
                    kind: "concept".to_owned(),
                    authority: "canonical".to_owned(),
                    representation: "record".to_owned(),
                    title: "Node Two".to_owned(),
                    deleted_at: None,
                    suite_id: None,
                }),
            ]
        );
    }

    #[test]
    fn Test_Node_Aliases_Should_Read_An_Alias_By_The_Node_It_Names()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Node_Aliases(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::NodeAlias(crate::Alias {
                alias: "N1-OLD".to_owned(),
                node_id: "N1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Node_Histories_Should_Read_An_Event_By_The_Node_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Node_Histories(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::NodeHistory(crate::History {
                node_id: "N1".to_owned(),
                ordinal: 1,
                event: "created".to_owned(),
                reason: "seeded".to_owned(),
                previous_event_hash: None,
                event_hash: "sha256:01".to_owned(),
                recorded_at: "2026-01-01T00:00:00Z".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Relation_Types_Should_Decode_The_Domain_And_Range_Json_Columns()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Relation_Types(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::RelationType(crate::Type {
                name: "verifies".to_owned(),
                tier: "core".to_owned(),
                inverse_of: None,
                domain: Vec::new(),
                range: Vec::new(),
                max_per_node: 4,
            })]
        );
    }

    #[test]
    fn Test_Collect_Relations_Should_Read_Both_Ends_By_Their_Node_Id()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Relations(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::Relation(crate::Relation {
                from_node_id: "N2".to_owned(),
                relation_type: "verifies".to_owned(),
                to_node_id: "N1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Normative_Statements_Should_Read_A_Statement_By_The_Node_It_Belongs_To()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Normative_Statements(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::NormativeStatement(crate::NormativeStatement {
                statement_id: "STMT-1".to_owned(),
                node_id: "N1".to_owned(),
                kind: "Requirement".to_owned(),
                canonical_text: "Text".to_owned(),
                canonical_hash: "sha256:aa".to_owned(),
                supersedes_hash: None,
            })]
        );
    }

    #[test]
    fn Test_Collect_Lineages_Should_Read_A_Heading_Anchored_Row()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Lineages(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::Lineage(crate::Lineage {
                source_block: None,
                source_heading: Some(OrdinalRef {
                    document: DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                source_table_row: None,
                disposition: "preserved-normalized".to_owned(),
                target_node_id: Some("N1".to_owned()),
                target_statement_id: None,
            })]
        );
    }

    #[test]
    fn Test_Collect_Omissions_Should_Read_A_Heading_Anchored_Row()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Collect_Omissions(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::Omission(crate::Omission {
                source_block: None,
                source_heading: Some(OrdinalRef {
                    document: DocumentRef {
                        path: "doc.md".to_owned(),
                        revision: "v1".to_owned(),
                    },
                    ordinal: 1,
                }),
                reason: "superseded".to_owned(),
                justification: "replaced".to_owned(),
                decision_record: "D-1".to_owned(),
            })]
        );
    }

    #[test]
    fn Test_Record_Front_Matter_Should_Decode_The_Tags_Json_Column()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Record_Front_Matter(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::RecordFrontMatter(RecordFrontMatter {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                node_id: "N1".to_owned(),
                status: "accepted".to_owned(),
                version: 1,
                tags: Vec::new(),
            })]
        );
    }

    #[test]
    fn Test_Record_Relations_Should_Read_A_Declared_Relation_By_Its_Document()
    {
        let store = Fixture();
        let mut records = Vec::new();

        Record_Relations(store.Connection(), &mut records).expect("collects");

        assert_eq!(
            records,
            vec![Record::RecordRelation(crate::RecordRelation {
                document: DocumentRef {
                    path: "doc.md".to_owned(),
                    revision: "v1".to_owned(),
                },
                ordinal: 1,
                target: "N2".to_owned(),
                relation: "affects".to_owned(),
            })]
        );
    }
}
