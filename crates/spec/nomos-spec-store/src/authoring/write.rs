//! Writing one record into the schema: its bytes, its blocks, its headings and its edges.

use nomos_spec_model::{
    BlockKind, FrontMatter as RecordFrontMatter, Record, RecordRelation, Segment, SourceBlock,
};
use rusqlite::{Connection, params};

use crate::Disposition;
use crate::Write as RecordWrite;
use crate::store::{
    Write_Node,
    Write_Source_Blocks, Write_Source_Document,
};
use crate::StoreError;

use super::Authored;

/// A heading as the schema files it: where it sat, and the identity the lineage row matches
/// on.
struct Heading<'a>
{
    ordinal: u32,
    depth: i64,
    title: &'a str,
}

/// Writes one record through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Record(
    connection: &Connection,
    authored: Authored<'_>,
    record: &Record,
) -> Result<RecordWrite, StoreError>
{
    use crate::DocumentPath;
    use crate::DocumentRevision;

    let Authored { path, revision, markdown } = authored;
    let front_matter = &record.front_matter;

    let node_uid = Write_Record_Node(connection, front_matter)?;
    let document_uid = Write_Source_Document(connection, DocumentPath(path), DocumentRevision(revision), markdown)?;
    let (blocks, headings) = Write_Record_Content(connection, document_uid, node_uid, record)?;
    let relations = Write_Declared_Relations(connection, document_uid, &front_matter.relations)?;

    return Ok(RecordWrite { node_uid, document_uid, blocks, headings, relations });
}

/// The record's own node: its identifier, kind, authority and title.
fn Write_Record_Node(connection: &Connection, front_matter: &RecordFrontMatter) -> Result<i64, StoreError>
{
    use crate::NodeRow;

    return Write_Node(connection, NodeRow {
        node_id: &front_matter.id,
        kind: &front_matter.kind,
        authority: &front_matter.authority,
        representation: "document",
        title: &front_matter.title,
    });
}

/// The record's body: its blocks, its headings, their dispositions and its front matter —
/// everything written once the node and the document both have surrogates. Returns the
/// block count and the heading count.
fn Write_Record_Content(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    record: &Record,
) -> Result<(u32, u32), StoreError>
{
    let blocks = Segment(&record.body);
    Write_Source_Blocks(connection, document_uid, &blocks)?;
    let headings = Write_Headings(connection, document_uid, node_uid, &blocks)?;
    Dispose_Blocks(connection, document_uid, node_uid, &blocks)?;
    Write_Front_Matter(connection, document_uid, node_uid, &record.front_matter)?;

    return Ok((u32::try_from(blocks.len()).unwrap_or(u32::MAX), headings));
}

/// Writes each heading and points it at the record it belongs to.
///
/// The disposition is written against the heading's own identity — its title and depth —
/// rather than against the ordinal it happened to have. An edit that moves a section changes
/// every following ordinal, and a lineage row that only matches the old number is a lineage
/// row an edit silently drops.
pub(super) fn Write_Headings(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<u32, StoreError>
{
    let mut insert_heading = connection.prepare(
        "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut dispose_heading = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition, target_node_uid)
         SELECT uid, ?4, ?5 FROM source_headings
         WHERE document_uid = ?1 AND title = ?2 AND depth = ?3",
    )?;
    let headings: Vec<Heading<'_>> = blocks
        .iter()
        .filter(|block| return block.kind == BlockKind::Heading)
        .map(Heading_Of)
        .collect();
    for heading in &headings
    {
        insert_heading
            .execute(params![document_uid, heading.ordinal, heading.depth, heading.title])?;
        dispose_heading.execute(params![
            document_uid,
            heading.title,
            heading.depth,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(u32::try_from(headings.len()).unwrap_or(u32::MAX));
}

/// Reads a heading's depth from its markers and its title from what follows them.
fn Heading_Of(block: &SourceBlock) -> Heading<'_>
{
    let depth = block.text.chars().take_while(|character| return *character == '#').count();

    return Heading {
        ordinal: block.ordinal,
        depth: i64::try_from(depth).unwrap_or(0),
        title: block.text.trim_start_matches('#').trim(),
    };
}

/// Points every block at the record it belongs to.
pub(super) fn Dispose_Blocks(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut dispose = connection.prepare(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, ?3, ?4 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
    )?;

    for block in blocks
    {
        dispose.execute(params![
            document_uid,
            block.ordinal,
            Disposition::PreservedVerbatim.Label(),
            node_uid
        ])?;
    }

    return Ok(());
}

/// Writes the three front matter fields nothing else keeps.
pub(super) fn Write_Front_Matter(
    connection: &Connection,
    document_uid: i64,
    node_uid: i64,
    front_matter: &RecordFrontMatter,
) -> Result<(), StoreError>
{
    let tags = serde_json::to_string(&front_matter.tags)
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    connection.execute(
        "INSERT INTO record_front_matter (document_uid, node_uid, status, version, tags_json)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(document_uid) DO UPDATE SET
             node_uid = excluded.node_uid,
             status = excluded.status,
             version = excluded.version,
             tags_json = excluded.tags_json",
        params![
            document_uid,
            node_uid,
            front_matter.status,
            front_matter.version,
            tags
        ],
    )?;

    return Ok(());
}

/// Replaces the declared relation list, in order.
///
/// Replaced rather than merged: the list is a document's own sentence about itself, and
/// merging would make a removed relation unremovable through the surface that wrote it.
pub(super) fn Write_Declared_Relations(
    connection: &Connection,
    document_uid: i64,
    relations: &[RecordRelation],
) -> Result<u32, StoreError>
{
    connection.execute(
        "DELETE FROM record_relations WHERE document_uid = ?1",
        params![document_uid],
    )?;

    let mut insert = connection.prepare(
        "INSERT INTO record_relations (document_uid, ordinal, target, relation)
         VALUES (?1, ?2, ?3, ?4)",
    )?;

    let mut ordinal = 0_u32;
    for relation in relations
    {
        ordinal = ordinal.saturating_add(1);
        insert.execute(params![document_uid, ordinal, relation.target, relation.relation])?;
    }

    return Ok(ordinal);
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::SpecificationStore;

    #[test]
    fn Test_Write_Record_Should_Write_The_Node_The_Document_And_Its_Blocks()
    {
        let store = Store();

        let write = Written(store.Connection());

        assert!(write.node_uid > 0);
        assert!(write.document_uid > 0);
        assert!(write.blocks > 0);
        assert_eq!(write.relations, 1);
    }

    #[test]
    fn Test_Write_Headings_Should_Record_One_Row_Per_Heading()
    {
        let store = Store();

        let write = Written(store.Connection());

        let headings: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM source_headings WHERE document_uid = ?1",
                [write.document_uid],
                |row| return row.get(0),
            )
            .expect("the SELECT counts the heading rows the write just inserted");

        assert_eq!(headings, write.headings);
        assert!(headings > 0);
    }

    #[test]
    fn Test_Dispose_Blocks_Should_Record_A_Lineage_Row_Per_Block()
    {
        let store = Store();

        let write = Written(store.Connection());

        let dispositions: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM lineage WHERE source_block_uid IS NOT NULL",
                [],
                |row| return row.get(0),
            )
            .expect("the SELECT counts the lineage rows the write just inserted");

        assert_eq!(dispositions, write.blocks);
    }

    #[test]
    fn Test_Write_Front_Matter_Should_Record_Status_Version_And_Tags()
    {
        let store = Store();

        let write = Written(store.Connection());

        let (status, version): (String, u32) = store
            .Connection()
            .query_row(
                "SELECT status, version FROM record_front_matter WHERE document_uid = ?1",
                [write.document_uid],
                |row| return Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("the front matter row the write just inserted is there to read");

        assert_eq!(status, "accepted");
        assert_eq!(version, 1);
    }

    #[test]
    fn Test_Write_Declared_Relations_Should_Replace_The_Whole_List()
    {
        let store = Store();
        let write = Written(store.Connection());

        let replaced = Write_Declared_Relations(store.Connection(), write.document_uid, &[])
            .expect("the document row exists, so replacing its relations runs");

        let remaining: u32 = store
            .Connection()
            .query_row(
                "SELECT count(*) FROM record_relations WHERE document_uid = ?1",
                [write.document_uid],
                |row| return row.get(0),
            )
            .expect("the count runs against record_relations in this store");

        assert_eq!(replaced, 0);
        assert_eq!(remaining, 0, "an empty list must replace, not merge with, what was there");
    }

    fn Store() -> SpecificationStore
    {
        return SpecificationStore::In_Memory()
            .expect("In_Memory builds its own schema, so opening touches no file");
    }

    fn A_Record() -> Record
    {
        return Record {
            front_matter: RecordFrontMatter {
                id: "D-1".to_owned(),
                kind: "decision".to_owned(),
                title: "A record".to_owned(),
                status: "accepted".to_owned(),
                authority: "canonical-normative-record".to_owned(),
                version: 1,
                tags: vec!["testing".to_owned()],
                relations: vec![RecordRelation {
                    target: "D-2".to_owned(),
                    relation: "relates-to".to_owned(),
                }],
            },
            body: "# A record\n\n## Decision\n\nOne.\n".to_owned(),
        };
    }

    fn Written(connection: &Connection) -> RecordWrite
    {
        let record = A_Record();

        return Write_Record(
            connection,
            Authored {
                path: "docs/records/D-1.md",
                revision: "authored",
                markdown: "# A record\n",
            },
            &record,
        )
        .expect("the record parses and this connection's schema takes every row");
    }
}
