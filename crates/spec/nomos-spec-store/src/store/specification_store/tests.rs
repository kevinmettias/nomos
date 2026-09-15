use super::*;
use crate::{Constraint as RelationConstraint, NodeRow, SuiteAuthority, TableLine};

/// How many edges of one type a node may carry, as these relation-type fixtures declare it.
const MAX_PER_NODE: u32 = 3;

/// A row ordinal no table in these fixtures reaches, so the lookup must find no row.
const ROW_ORDINAL_PAST_THE_TABLE: u32 = 9999;

/// The lines a one-row, two-column table gives the census: header, separator and content row.
const LINES_IN_A_ONE_ROW_TABLE: u32 = 3;

#[test]
fn Test_Open_Should_Persist_Across_A_Reopen()
{
    let mut path = std::env::temp_dir();
    path.push(format!("nomos-spec-store-open-test-{}.db", std::process::id()));
    // A file left behind by an interrupted run is the only failure this removal may report.
    if let Err(cause) = std::fs::remove_file(&path)
    {
        assert_eq!(cause.kind(), std::io::ErrorKind::NotFound, "a leftover database: {cause}");
    }

    let mut first = SpecificationStore::Open(&path)
        .expect("Open creates the file and applies the schema at that path");
    first.Put_Blob(b"hello").expect("Put_Blob takes the first write to a fresh store");
    drop(first);

    let second =
        SpecificationStore::Open(&path).expect("the file the first handle wrote is still there");
    assert_eq!(second.Count(Table::Blobs).expect("Count reads the schema this store opened"), 1);
    drop(second);

    std::fs::remove_file(&path)
        .expect("the handle is dropped, so the database this test made is removable");
}

#[test]
fn Test_In_Memory_Should_Open_At_The_Latest_Schema_Version()
{
    let store = SpecificationStore::In_Memory().expect("In_Memory applies the schema in process");

    assert_eq!(store.Version(), Latest_Version());
}

#[test]
fn Test_Version_Should_Report_The_Schema_Version_The_Store_Is_At()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory applies the schema in process without a file to open");

    assert!(store.Version() > 0);
}

#[test]
fn Test_In_Transaction_Should_Roll_Back_Everything_When_Work_Fails()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the rollback test a store it can write into");

    let outcome: Result<(), StoreError> = store.In_Transaction(|transaction| {
        transaction.execute(
            "INSERT INTO nodes (node_id, kind, authority, representation, title)
             VALUES ('N-1', 'requirement', 'canonical-normative-record', 'record', 'a node')",
            [],
        )?;

        return Err(StoreError::Sql("deliberate failure".to_owned()));
    });

    assert!(outcome.is_err());
    assert_eq!(
        store.Count(Table::Nodes).expect("the rolled-back insert left the table as it was"),
        0,
        "the insert must not survive the rollback"
    );
}

#[test]
fn Test_Put_Blob_Should_Store_The_Same_Bytes_Once()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the blob test a store it can write into");

    let first = store.Put_Blob(b"same bytes").expect("Put_Blob takes the first copy of the bytes");
    let second = store.Put_Blob(b"same bytes").expect("Put_Blob takes a second copy of the bytes");

    assert_eq!(first, second);
}

#[test]
fn Test_Put_Source_Document_Should_Record_A_New_Document()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the document test an empty store");

    let uid = store
        .Put_Source_Document("a.md", "v1", "hello")
        .expect("Put_Source_Document answers with the uid it minted");

    assert!(uid > 0);
}

#[test]
fn Test_Put_Source_Blocks_Should_Write_Every_Block()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the block test an empty store");
    let markdown = "# Title\n\nOne.\n\n## Section\n\nTwo.\n";
    let uid = store
        .Put_Source_Document("a.md", "v1", markdown)
        .expect("Put_Source_Document accepts a path this store has not seen");
    let blocks = nomos_spec_model::Segment(markdown);

    let written = store
        .Put_Source_Blocks(uid, &blocks)
        .expect("Put_Source_Blocks inserts every block it is handed");

    assert_eq!(written, blocks.len());
}

#[test]
fn Test_Table_Row_Uid_Should_Address_One_Row_By_Its_Position()
{
    let StoreWithContentRow { store, uid, content } = A_Store_With_One_Content_Row();

    let row_uid = store
        .Table_Row_Uid(uid, content.block_ordinal, content.row_ordinal)
        .expect("the fixture's table has a row at the fixture's own position");

    assert!(row_uid.is_some());
    let past_the_end = store
        .Table_Row_Uid(uid, content.block_ordinal, ROW_ORDINAL_PAST_THE_TABLE)
        .expect("an ordinal past the table's rows resolves to no row");
    assert!(past_the_end.is_none());
}

#[test]
fn Test_Put_Row_Lineage_Should_Record_What_A_Row_Became()
{
    let StoreWithContentRow { mut store, uid, content } = A_Store_With_One_Content_Row();
    let row_uid = store
        .Table_Row_Uid(uid, content.block_ordinal, content.row_ordinal)
        .expect("the fixture's table has a row at the fixture's own position")
        .expect("the row exists");

    store
        .Put_Row_Lineage(row_uid, "preserved-verbatim", None)
        .expect("Put_Row_Lineage accepts a row uid this store minted");

    let disposition: String = store
        .Connection()
        .query_row(
            "SELECT disposition FROM lineage WHERE source_table_row_uid = ?1",
            [row_uid],
            |row| return row.get(0),
        )
        .expect("the lineage row this test wrote is the one the query reads");
    assert_eq!(disposition, "preserved-verbatim");
}

#[test]
fn Test_Row_Census_Should_Count_Rows_In_Scope()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the census test an empty store");
    let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let uid = store
        .Put_Source_Document("a.md", "v1", markdown)
        .expect("Put_Source_Document accepts a path this store has not seen");
    let blocks = nomos_spec_model::Segment(markdown);
    store
        .Put_Source_Blocks(uid, &blocks)
        .expect("Put_Source_Blocks inserts every block it is handed");

    let census = store
        .Row_Census(RowScope::Everything)
        .expect("Row_Census reads the tables this store filled");

    assert_eq!(census.lines, LINES_IN_A_ONE_ROW_TABLE);
}

#[test]
fn Test_Upsert_Node_Should_Insert_A_New_Node()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the node test an empty store");

    let decision = Node("D-1", "decision");
    let uid = store.Upsert_Node(decision).expect("Upsert_Node mints a uid for an unseen node id");

    assert!(uid > 0);
    let found = store.Node_Uid("D-1").expect("Node_Uid reads the node just inserted");
    assert_eq!(found, Some(uid));
}

#[test]
fn Test_Put_Suite_Should_Record_A_Suite_And_Its_Authority()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the suite test an empty store");

    let uid = store
        .Put_Suite("my-suite", "My Suite", SuiteAuthority::Root)
        .expect("Put_Suite records a suite id the store has not seen");
    let again = store
        .Put_Suite("my-suite", "My Suite Renamed", SuiteAuthority::Root)
        .expect("Put_Suite takes a second declaration of that id");

    assert_eq!(uid, again, "the same suite id must resolve to the same row");
}

#[test]
fn Test_Assign_Suite_Should_Place_A_Node_In_A_Suite()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the assignment test an empty store");
    let decision = Node("D-1", "decision");
    let node_uid =
        store.Upsert_Node(decision).expect("Upsert_Node mints the node this suite will hold");
    let suite_uid = store
        .Put_Suite("my-suite", "My Suite", SuiteAuthority::Root)
        .expect("Put_Suite records the suite the node is assigned to");

    store
        .Assign_Suite(node_uid, suite_uid)
        .expect("Assign_Suite accepts two uids this store minted");

    let (suite_id, _) = store
        .Suite_Of("D-1")
        .expect("Suite_Of selects from the assignment table the schema defines")
        .expect("the assignment just written is the one it reads back");
    assert_eq!(suite_id, "my-suite");
}

#[test]
fn Test_Suite_Of_Should_Report_The_Suite_A_Node_Belongs_To()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the lookup test an empty store");
    let decision = Node("D-1", "decision");
    let node_uid =
        store.Upsert_Node(decision).expect("Upsert_Node mints the node the lookup asks about");
    let unassigned = store.Suite_Of("D-1").expect("Suite_Of answers for an unassigned node");
    assert!(unassigned.is_none());

    let suite_uid = store
        .Put_Suite("my-suite", "My Suite", SuiteAuthority::Sibling)
        .expect("Put_Suite records the sibling-authority suite");
    store
        .Assign_Suite(node_uid, suite_uid)
        .expect("Assign_Suite pairs the node with that suite");

    let (suite_id, authority) = store
        .Suite_Of("D-1")
        .expect("Suite_Of selects the assignment row it just wrote")
        .expect("the assignment is the row Suite_Of reads back");
    assert_eq!(suite_id, "my-suite");
    assert_eq!(authority, SuiteAuthority::Sibling);
}

#[test]
fn Test_Reference_Node_Should_Mint_A_Placeholder_Node()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the reference test an empty store");

    let uid = store
        .Reference_Node("D-999")
        .expect("Reference_Node mints a placeholder for an unseen id");

    assert!(uid > 0);
    let summary = store
        .Node_Summary("D-999")
        .expect("Node_Summary selects the row Reference_Node inserted")
        .expect("the placeholder is the row Node_Summary reads back");
    assert_eq!(summary.kind, "unknown");
}

#[test]
fn Test_Node_Uid_Should_Look_Up_A_Node_By_Its_Identifier()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the uid test an empty store");
    let decision = Node("D-1", "decision");
    store
        .Upsert_Node(decision)
        .expect("Upsert_Node mints the node whose identifier is looked up");

    let found = store.Node_Uid("D-1").expect("Node_Uid finds the node just inserted");
    let missing = store.Node_Uid("D-999").expect("Node_Uid answers for an unseen id");
    assert!(found.is_some());
    assert!(missing.is_none());
}

#[test]
fn Test_Put_Relation_Type_Should_Register_A_Constrained_Type()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the relation-type test an empty store");
    // Idempotent: registering the same name twice keeps the first declaration.
    Register_A_Widget_Relation_Type(&mut store, "relates-to");
    Register_A_Widget_Relation_Type(&mut store, "relates-to");

    let tier = The_Registered_Tier(&store);

    assert_eq!(tier, "seed");
}

#[test]
fn Test_Pair_Relation_Type_Should_Link_A_Type_With_Its_Inverse()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the pairing test an empty store");
    Register_A_Widget_Relation_Type(&mut store, "relates-to");
    Register_A_Widget_Relation_Type(&mut store, "relates-from");

    store
        .Pair_Relation_Type("relates-to", "relates-from")
        .expect("Pair_Relation_Type links two names this store registered");

    let inverse = The_Registered_Inverse(&store);

    assert_eq!(inverse, Some("relates-from".to_owned()));

    let missing = store.Pair_Relation_Type("nonexistent", "relates-from");
    assert!(missing.is_err());
}

#[test]
fn Test_Put_Relation_Should_Record_An_Edge_Between_Two_Nodes()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the edge test an empty store");
    let source = Node("A", "widget");
    let target = Node("B", "widget");
    store.Upsert_Node(source).expect("Upsert_Node mints the edge's source node");
    store.Upsert_Node(target).expect("Upsert_Node mints the edge's target node");
    Register_A_Widget_Relation_Type(&mut store, "relates-to");

    store
        .Put_Relation("A", "relates-to", "B")
        .expect("Put_Relation links two nodes this store holds under a registered type");

    let edges = store.Count(Table::Relations).expect("the edge is the row the table gained");
    assert_eq!(edges, 1);
}

#[test]
fn Test_Count_Should_Tally_Rows_In_A_Declared_Table()
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the count test an empty store");
    assert_eq!(store.Count(Table::Blobs).expect("an untouched store holds no blobs"), 0);

    store.Put_Blob(b"hello").expect("Put_Blob takes the one blob this test counts");

    assert_eq!(
        store.Count(Table::Blobs).expect("the count sees the blob this test inserted"),
        1
    );
}

#[test]
fn Test_Connection_Should_Expose_The_Underlying_Handle()
{
    let store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the handle test an open store");

    let version: i64 = store
        .Connection()
        .pragma_query_value(None, "user_version", |row| return row.get(0))
        .expect("the opened database answers for the pragma it holds");

    assert_eq!(u32::try_from(version).unwrap_or(0), store.Version());
}

/// A store already holding one two-column, one-row table, that row's own `TableLine`, and
/// the document's uid — named rather than a tuple so a caller is not counting positions.
struct StoreWithContentRow
{
    store: SpecificationStore,
    uid: i64,
    content: TableLine,
}

/// A store already holding one two-column, one-row table, and that row's own `TableLine` —
/// the setup every test that addresses a table row by position shares.
fn A_Store_With_One_Content_Row() -> StoreWithContentRow
{
    let mut store = SpecificationStore::In_Memory()
        .expect("In_Memory gives the shared fixture an empty store");
    let markdown = "# Title\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let uid = store
        .Put_Source_Document("a.md", "v1", markdown)
        .expect("Put_Source_Document accepts a path this store has not seen");
    let blocks = nomos_spec_model::Segment(markdown);
    store
        .Put_Source_Blocks(uid, &blocks)
        .expect("Put_Source_Blocks inserts every block it is handed");
    let content = store
        .Table_Lines(uid, None, None)
        .expect("Table_Lines reads the table the fixture's markdown holds")
        .into_iter()
        .find(|line| return line.kind == "content")
        .expect("the fixture's markdown carries one content row");

    return StoreWithContentRow { store, uid, content };
}

/// Registers `name` as a widget relation type under the constraint these fixtures share.
fn Register_A_Widget_Relation_Type(store: &mut SpecificationStore, name: &str)
{
    let constraint = RelationConstraint {
        domain: &["widget"],
        range: &["widget"],
        max_per_node: MAX_PER_NODE,
    };
    store
        .Put_Relation_Type(name, "seed", &constraint)
        .expect("Put_Relation_Type accepts a name the store has not registered");
}

/// The tier the store recorded for `relates-to`, read straight from the table that holds it.
fn The_Registered_Tier(store: &SpecificationStore) -> String
{
    let tier: String = store
        .Connection()
        .query_row(
            "SELECT tier FROM relation_types WHERE name = 'relates-to'",
            [],
            |row| return row.get(0),
        )
        .expect("the relation type this test registered is the row the query reads");
    return tier;
}

/// The inverse the store recorded for `relates-to`, read straight from the table that holds it.
fn The_Registered_Inverse(store: &SpecificationStore) -> Option<String>
{
    let inverse: Option<String> = store
        .Connection()
        .query_row(
            "SELECT inverse_of FROM relation_types WHERE name = 'relates-to'",
            [],
            |row| return row.get(0),
        )
        .expect("the pairing this test wrote is the row the query reads");
    return inverse;
}

fn Node<'a>(id: &'a str, kind: &'a str) -> NodeRow<'a>
{
    return NodeRow {
        node_id: id,
        kind,
        authority: "canonical-normative-record",
        representation: "record",
        title: id,
    };
}
