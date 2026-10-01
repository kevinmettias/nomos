//! Tying a restored node back to the text it was minted from.
//!
//! A member's origin is either a whole block or one row of a table, and each is disposed in
//! the lineage table its own way. These came out of `record.rs` when it crossed the
//! file-size review trigger, and their tests came with them -- a Rust unit test's companion
//! is the file the test is written in, so a function and its test have to move together.

use super::super::{IngestError, Member, Origin, SpecificationStore};

/// A row's address in the store.
///
/// Three numbers that only mean anything together, and carrying them as one value is what
/// keeps the row tracer inside the argument budget.
#[derive(Clone, Copy)]
pub(super) struct RowAt
{
    document_uid: i64,
    block_ordinal: u32,
    row_ordinal: u32,
}

/// Ties a node to the text it was minted from.
pub(super) fn Trace_Member(
    store: &mut SpecificationStore,
    document_uid: i64,
    member: &Member,
    node_uid: i64,
) -> Result<(), IngestError>
{
    match member.origin
    {
        Origin::Block { ordinal } => Dispose_Block(store, document_uid, ordinal, node_uid)?,
        Origin::Row {
            block_ordinal,
            row_ordinal,
        } =>
        {
            let at = RowAt {
                document_uid,
                block_ordinal,
                row_ordinal,
            };
            Trace_Row(store, at, member, node_uid)?;
        }
    }

    return Ok(());
}

/// Ties a node to the one row it was minted from.
///
/// A row the store does not hold is refused rather than skipped: the node would stand with
/// no text behind it, and "which row did this come from" is the question the restoration
/// exists to answer.
pub(super) fn Trace_Row(
    store: &mut SpecificationStore,
    at: RowAt,
    member: &Member,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let found = store.Table_Row_Uid(at.document_uid, at.block_ordinal, at.row_ordinal)?;
    let Some(row_uid) = found
    else
    {
        return Err(IngestError::Parse(format!(
            "{} block {} row {} is not in the store, so {} would trace to nothing",
            member.document, at.block_ordinal, at.row_ordinal, member.id
        )));
    };
    store.Put_Row_Lineage(row_uid, "preserved-verbatim", Some(node_uid))?;

    return Ok(());
}

pub(super) fn Dispose_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), IngestError>
{
    use super::Sql_Result;

    let disposed = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, node_uid],
    );
    Sql_Result(disposed)?;

    return Ok(());
}

#[cfg(test)]
mod tests
{
    use super::super::{
        A_Located_Member_With_Its_Node, Core, DocumentPath, DocumentRevision, Document_Uid, LocatedMemberWithNode,
        NodeRow, Store_With_Core,
    };
    use super::*;

    #[test]
    fn Test_Trace_Member_Should_Dispatch_By_Origin_Kind()
    {
        let LocatedMemberWithNode { mut store, document_uid, member: row_member, node_uid: node_for_row } =
            A_Located_Member_With_Its_Node();

        Trace_Member(&mut store, document_uid, &row_member, node_for_row)
            .expect("the fixture's member was located from a table row the store already holds");

        let traced_row = Traced_Row_Lineage_Count(&store, node_for_row);
        assert_eq!(traced_row, 1);

        let BlockMemberWithNode { member: block_member, node: node_for_block } =
            Block_Member_With_Node(&mut store, row_member);

        Trace_Member(&mut store, document_uid, &block_member, node_for_block)
            .expect("the block member carries the ordinal the fixture's only block has");

        let traced_block = Traced_Block_Lineage_Count(&store, node_for_block);
        assert_eq!(traced_block, 1);
    }

    fn Traced_Row_Lineage_Count(store: &SpecificationStore, node_for_row: i64) -> i64
    {
        let traced_row: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage WHERE source_table_row_uid IS NOT NULL AND target_node_uid = ?1",
                rusqlite::params![node_for_row],
                |row| row.get(0),
            )
            .expect("the count runs over the store's own lineage table, which this test just wrote");

        return traced_row;
    }

    /// A member re-pointed at a whole block, and the node minted for it.
    struct BlockMemberWithNode
    {
        member: Member,
        node: i64,
    }

    fn Block_Member_With_Node(store: &mut SpecificationStore, row_member: Member) -> BlockMemberWithNode
    {
        let member = Member {
            origin: Origin::Block { ordinal: 1 },
            ..row_member
        };
        let node = store
            .Upsert_Node(NodeRow {
                node_id: "APX-D-TEST",
                kind: "schema",
                authority: "canonical",
                representation: "record",
                title: "Test",
            })
            .expect("APX-D-TEST is an identifier this test has not minted, so the upsert writes a row");

        return BlockMemberWithNode { member, node };
    }

    fn Traced_Block_Lineage_Count(store: &SpecificationStore, node_for_block: i64) -> i64
    {
        let traced_block: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage WHERE source_block_uid IS NOT NULL AND target_node_uid = ?1",
                rusqlite::params![node_for_block],
                |row| row.get(0),
            )
            .expect("the count runs over the store's own lineage table, which this test just wrote");

        return traced_block;
    }

    /// A row ordinal no table in the fixture reaches, so the lookup finds nothing.
    const BEYOND_THE_TABLE: u32 = 100;

    #[test]
    fn Test_Trace_Row_Should_Point_The_Row_At_Its_Node_Or_Refuse_A_Missing_One()
    {
        let LocatedMemberWithNode { mut store, document_uid, member, node_uid } = A_Located_Member_With_Its_Node();
        let at = Row_At(&member, document_uid);

        Trace_Row(&mut store, at, &member, node_uid)
            .expect("the fixture's member was located from a table row the store already holds");

        let traced = Traced_Row_Uid(&store, at);
        assert_eq!(traced, Some(node_uid));

        let mut missing = at;
        missing.row_ordinal = missing.row_ordinal.saturating_add(BEYOND_THE_TABLE);
        let refusal = Trace_Row(&mut store, missing, &member, node_uid)
            .expect_err("must refuse a row that is not in the store");
        assert!(format!("{refusal}").contains("is not in the store"), "{refusal}");
    }

    /// Where the member's own row sits, as the store addresses it.
    fn Row_At(member: &Member, document_uid: i64) -> RowAt
    {
        let Origin::Row {
            block_ordinal,
            row_ordinal,
        } = member.origin
        else
        {
            // The fixture is the canonical domain model, whose members are table rows; a member
            // from a heading would mean the fixture stopped being the one this test needs.
            panic!("the domain model member must trace to a row");
        };

        return RowAt {
            document_uid,
            block_ordinal,
            row_ordinal,
        };
    }

    fn Traced_Row_Uid(store: &SpecificationStore, at: RowAt) -> Option<i64>
    {
        let row_uid = store
            .Table_Row_Uid(at.document_uid, at.block_ordinal, at.row_ordinal)
            .expect("the fixture's row was ingested, so the store answers its identifier")
            .expect("the member's own row is the one its document ingest wrote");
        let traced: Option<i64> = store
            .Connection()
            .query_row(
                "SELECT target_node_uid FROM lineage WHERE source_table_row_uid = ?1",
                rusqlite::params![row_uid],
                |row| row.get(0),
            )
            .expect("the row was traced just above, so it carries a lineage row here");

        return traced;
    }

    #[test]
    fn Test_Dispose_Block_Should_Trace_A_Whole_Block_To_One_Node()
    {
        let Core { mut store, documents: _documents } = Store_With_Core();
        let document_uid = Document_Uid(&store, DocumentRevision("v14.36"), DocumentPath("02-core.md"))
            .expect("the fixture ingested 02-core.md at v14.36, so the row is there");
        let node_uid = store
            .Upsert_Node(NodeRow {
                node_id: "APX-D-TEST",
                kind: "schema",
                authority: "canonical",
                representation: "record",
                title: "Test",
            })
            .expect("APX-D-TEST is an identifier this test has not minted, so the upsert writes a row");

        Dispose_Block(&mut store, document_uid, 1, node_uid)
            .expect("the fixture's document ingest wrote a block at ordinal one");

        let traced = Disposed_Lineage_Count(&store, document_uid, node_uid);
        assert_eq!(traced, 1);
    }

    fn Disposed_Lineage_Count(store: &SpecificationStore, document_uid: i64, node_uid: i64) -> i64
    {
        let traced: i64 = store
            .Connection()
            .query_row(
                "SELECT COUNT(*) FROM lineage l JOIN source_blocks b ON b.uid = l.source_block_uid \
                 WHERE b.document_uid = ?1 AND b.ordinal = 1 AND l.target_node_uid = ?2",
                rusqlite::params![document_uid, node_uid],
                |row| row.get(0),
            )
            .expect("the count runs over the store's own lineage and block tables");

        return traced;
    }
}
