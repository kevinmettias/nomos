//! Building one filtered query, and reading its rows back.

use super::{
    Connection, Content, Filter, Gather_Blocks, Gather_Documents, Gather_Headings, Gather_Lineage, Gather_Nodes,
    Gather_Families, Gather_Neighbourhood, Gather_Omissions, Gather_Relations, Gather_Rows, Gather_Statements, Gather_Suites, Item, params_from_iter,
    ProjectError, Row,
};
use core::fmt::Write as _;

pub(super) struct Query
{
    sql: String,
    values: Vec<String>,
}

/// The first of the two columns [`Query::Either`] matches a value against, kept distinct
/// from [`SecondColumn`] so the two positions cannot be swapped at a call site.
pub(super) struct FirstColumn<'a>(pub(super) &'a str);

/// The second of the two columns [`Query::Either`] matches a value against.
pub(super) struct SecondColumn<'a>(pub(super) &'a str);

/// The other table [`Query::Among`] matches through: where to look, which column names the
/// row this query is already selecting, and which column carries the value being matched.
///
/// Three fields in one value rather than three parameters, so `Among` stays within this
/// workspace's own `parameter-count` limit and so two same-typed column names cannot be
/// swapped at a call site -- the same reason [`FirstColumn`] and [`SecondColumn`] exist.
pub(super) struct AmongSource<'a>
{
    pub(super) table: &'a str,
    pub(super) key_column: &'a str,
    pub(super) value_column: &'a str,
}

impl Query
{
    pub(super) fn On(base: &str) -> Self
    {
        return Self {
            sql: base.to_owned(),
            values: Vec::new(),
        };
    }

    pub(super) fn Equal(&mut self, column: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(value.clone());
            let _ = write!(self.sql, " AND {column} = ?{}", self.values.len());
        }
    }

    pub(super) fn Prefix(&mut self, column: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(format!("{}%", value.replace('%', "\\%")));
            let _ = write!(
                self.sql,
                " AND {column} LIKE ?{} ESCAPE '\\'",
                self.values.len()
            );
        }
    }

    /// One value matched against either of two columns.
    ///
    /// A relation has two ends and a subject sits at one or the other. Narrowing only the
    /// end an edge starts from would show a subject what it declares and hide what is
    /// declared about it, which is the half of a graph a reader is usually looking for.
    pub(super) fn Either(&mut self, first: FirstColumn<'_>, second: SecondColumn<'_>, value: Option<&String>)
    {
        let first = first.0;
        let second = second.0;

        if let Some(value) = value
        {
            self.values.push(value.clone());
            let position = self.values.len();
            let _ = write!(self.sql, " AND ({first} = ?{position} OR {second} = ?{position})");
        }
    }

    /// One value matched through a table this query does not itself select from.
    ///
    /// # Why a semi-join and not a `JOIN` in the base query
    ///
    /// A `JOIN` multiplies a row by however many rows the other table holds for it, which
    /// would change what an *unfiltered* section selects -- and every committed projection
    /// is built from unfiltered sections. `IN` cannot change the row count whatever the
    /// other table holds, and when the value is unset it writes nothing at all, so a query
    /// that does not use this filter is byte-identical to the one before this existed.
    ///
    /// # What it does to a row the other table has nothing for
    ///
    /// Excludes it. That is the point rather than a side effect: the caller is asking which
    /// rows *declared* something, and a row that declared nothing has not declared the value
    /// being asked for. Admitting it would mean reporting it under a value nobody wrote.
    pub(super) fn Among(&mut self, key: &str, source: AmongSource<'_>, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(value.clone());
            let AmongSource { table, key_column, value_column } = source;
            let _ = write!(
                self.sql,
                " AND {key} IN (SELECT {key_column} FROM {table} WHERE {value_column} = ?{})",
                self.values.len()
            );
        }
    }

    pub(super) fn Ordered_By(mut self, columns: &str) -> Self
    {
        self.sql.push_str(" ORDER BY ");
        self.sql.push_str(columns);

        return self;
    }

    pub(super) fn Run<RowReader>(&self, connection: &Connection, read: RowReader) -> Result<Vec<Item>, ProjectError>
    where
        RowReader: Fn(&Row<'_>) -> rusqlite::Result<Item>,
    {
        let mut statement = connection.prepare(&self.sql)?;
        let rows = statement.query_map(params_from_iter(self.values.iter()), |row| return read(row))?;

        let mut items = Vec::new();
        for item in rows
        {
            items.push(item?);
        }

        return Ok(items);
    }
}

pub(super) fn Gather_Items(
    connection: &Connection,
    content: Content,
    filter: &Filter,
) -> Result<Vec<Item>, ProjectError>
{
    return match content
    {
        Content::Suites => Gather_Suites(connection, filter),
        Content::Documents => Gather_Documents(connection, filter),
        Content::Headings => Gather_Headings(connection, filter),
        Content::Blocks => Gather_Blocks(connection, filter),
        Content::Rows => Gather_Rows(connection, filter),
        Content::Nodes => Gather_Nodes(connection, filter),
        Content::Statements => Gather_Statements(connection, filter),
        Content::Relations => Gather_Relations(connection, filter),
        Content::Lineage => Gather_Lineage(connection, filter),
        Content::Omissions => Gather_Omissions(connection, filter),
        Content::Neighbourhood => Gather_Neighbourhood(connection, filter),
        Content::Families => Gather_Families(connection, filter),
    };
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
    row: &'row Row<'statement>,
    next: usize,
}

impl<'row, 'statement> Columns<'row, 'statement>
{
    pub(super) fn Of(row: &'row Row<'statement>) -> Self
    {
        return Self { row, next: 0 };
    }

    /// The next column as text, reading a NULL as the empty string.
    pub(super) fn Text(&mut self) -> rusqlite::Result<String>
    {
        return self.Next::<Option<String>>().map(Option::unwrap_or_default);
    }

    /// The next column the query names, as whatever type receives it.
    pub(super) fn Next<Value: rusqlite::types::FromSql>(&mut self) -> rusqlite::Result<Value>
    {
        let at = self.next;

        self.next = at.saturating_add(1);
        return self.row.get(at);
    }
}

/// Every way a node section may be narrowed, in one place.
///
/// `identifier_prefix` is a prefix and the rest are equalities, which is the whole of what
/// distinguishes them: a section selecting `OD-LEDGER-` wants a family and one selecting
/// `OD-LEDGER-019` wants a record. Spelled out at each call site, the two were free to
/// disagree about which columns a node section honours — and `Content::Honours` is checked
/// against that list.
pub(super) fn Narrow_To_Nodes(query: &mut Query, filter: &Filter)
{
    for (column, value) in [
        ("n.kind", filter.kind.as_ref()),
        ("n.authority", filter.authority.as_ref()),
        ("n.representation", filter.representation.as_ref()),
        ("s.suite_id", filter.suite.as_ref()),
        ("n.node_id", filter.node_id.as_ref()),
    ]
    {
        query.Equal(column, value);
    }

    query.Prefix("n.node_id", filter.identifier_prefix.as_ref());

    // The one filter answered from outside the graph. A node's identity, kind and authority
    // are the graph's; a record's declared status is its own front matter's, which the
    // authoring surface fills from the file and nothing has read back until now.
    query.Among(
        "n.uid",
        AmongSource { table: "record_front_matter", key_column: "node_uid", value_column: "status" },
        filter.status.as_ref(),
    );
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn Test_On_Should_Start_A_Query_From_The_Given_Base_Sql()
    {
        let query = Query::On("SELECT 1");

        assert_eq!(query.sql, "SELECT 1");
        assert!(query.values.is_empty());
    }

    #[test]
    fn Test_Equal_Should_Add_An_Equality_Clause_When_A_Value_Is_Set()
    {
        let mut query = Query::On("SELECT 1 WHERE 1 = 1");

        query.Equal("kind", Some(&"concept".to_owned()));
        query.Equal("authority", None);

        assert_eq!(query.sql, "SELECT 1 WHERE 1 = 1 AND kind = ?1");
        assert_eq!(query.values, vec!["concept".to_owned()]);
    }

    #[test]
    fn Test_Prefix_Should_Escape_A_Percent_Sign_In_The_Value()
    {
        let mut query = Query::On("SELECT 1 WHERE 1 = 1");

        query.Prefix("node_id", Some(&"50%".to_owned()));

        assert_eq!(query.sql, "SELECT 1 WHERE 1 = 1 AND node_id LIKE ?1 ESCAPE '\\'");
        assert_eq!(query.values, vec!["50\\%%".to_owned()]);
    }

    #[test]
    fn Test_Either_Should_Match_A_Value_Against_Two_Candidate_Columns()
    {
        let mut query = Query::On("SELECT 1 WHERE 1 = 1");

        query.Either(FirstColumn("f.node_id"), SecondColumn("t.node_id"), Some(&"CDM-ONE".to_owned()));

        assert_eq!(query.sql, "SELECT 1 WHERE 1 = 1 AND (f.node_id = ?1 OR t.node_id = ?1)");
        assert_eq!(query.values, vec!["CDM-ONE".to_owned()]);
    }

    #[test]
    fn Test_Ordered_By_Should_Append_An_Order_By_Clause()
    {
        let query = Query::On("SELECT 1").Ordered_By("node_id");

        assert_eq!(query.sql, "SELECT 1 ORDER BY node_id");
    }

    #[test]
    fn Test_Run_Should_Read_Every_Matching_Row_Through_The_Given_Reader()
    {
        let connection = Connection::open_in_memory().expect("opens");
        connection
            .execute_batch("CREATE TABLE t (n TEXT); INSERT INTO t (n) VALUES ('a'), ('b');")
            .expect("seeds");

        let query = Query::On("SELECT n FROM t");
        let items = query
            .Run(&connection, |row| {
                let text: String = row.get(0)?;
                return Ok(Item::Of(&text));
            })
            .expect("reads");

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn Test_Gather_Items_Should_Dispatch_By_Content_Kind()
    {
        let store = nomos_spec_store::SpecificationStore::In_Memory().expect("opens");
        store
            .Connection()
            .execute(
                "INSERT INTO suites (suite_id, title, authority_root) VALUES ('nomos', 'The Nomos specification', 1)",
                [],
            )
            .expect("seeds");

        let items = Gather_Items(store.Connection(), Content::Suites, &Filter::default()).expect("gathers");

        assert_eq!(items.len(), 1);
        assert_eq!(
            items.first().expect("asserted above to contain exactly one item").identity,
            "nomos"
        );
    }

    #[test]
    fn Test_Of_Should_Position_Reading_At_The_Rows_First_Column()
    {
        let connection = Connection::open_in_memory().expect("opens");
        connection
            .execute_batch("CREATE TABLE t (a TEXT, b TEXT); INSERT INTO t VALUES ('first', 'second');")
            .expect("seeds");

        let first = connection
            .query_row("SELECT a, b FROM t", [], |row| {
                return Columns::Of(row).Text();
            })
            .expect("reads");

        assert_eq!(first, "first");
    }

    #[test]
    fn Test_Text_Should_Read_A_Null_Column_As_An_Empty_String()
    {
        let connection = Connection::open_in_memory().expect("opens");
        connection
            .execute_batch("CREATE TABLE t (a TEXT); INSERT INTO t (a) VALUES (NULL);")
            .expect("seeds");

        let text = connection
            .query_row("SELECT a FROM t", [], |row| {
                return Columns::Of(row).Text();
            })
            .expect("reads");

        assert_eq!(text, "");
    }

    #[test]
    fn Test_Next_Should_Advance_Past_Each_Column_It_Reads()
    {
        let connection = Connection::open_in_memory().expect("opens");
        connection
            .execute_batch("CREATE TABLE t (a INTEGER, b INTEGER); INSERT INTO t VALUES (10, 20);")
            .expect("seeds");

        let (first, second): (i64, i64) = connection
            .query_row("SELECT a, b FROM t", [], |row| {
                let mut columns = Columns::Of(row);
                let first: i64 = columns.Next()?;
                let second: i64 = columns.Next()?;
                return Ok((first, second));
            })
            .expect("reads");

        assert_eq!((first, second), (10, 20));
    }

    #[test]
    fn Test_Narrow_To_Nodes_Should_Add_A_Clause_Per_Filter_Field_Set()
    {
        let filter = Filter {
            kind: Some("concept".to_owned()),
            node_id: Some("CDM-ONE".to_owned()),
            ..Filter::default()
        };
        let mut query = Query::On("SELECT 1 FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid WHERE 1 = 1");

        Narrow_To_Nodes(&mut query, &filter);

        assert!(query.sql.contains("n.kind = ?1"), "{}", query.sql);
        assert!(query.sql.contains("n.node_id = ?2"), "{}", query.sql);
        assert_eq!(query.values, vec!["concept".to_owned(), "CDM-ONE".to_owned()]);
    }
}
