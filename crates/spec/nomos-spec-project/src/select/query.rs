//! Building one filtered query, and reading its rows back.

use super::{
    Blocks, Connection, Content, Documents, Filter, Headings, Item, Lineage, Nodes, Omissions, params_from_iter,
    ProjectError, Relations, Row, Rows, Statements, Suites,
};
use core::fmt::Write as _;

pub(super) struct Query
{
    sql: String,
    values: Vec<String>,
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
    pub(super) fn Either(&mut self, first: &str, second: &str, value: Option<&String>)
    {
        if let Some(value) = value
        {
            self.values.push(value.clone());
            let position = self.values.len();
            let _ = write!(self.sql, " AND ({first} = ?{position} OR {second} = ?{position})");
        }
    }

    pub(super) fn Ordered_By(mut self, columns: &str) -> Self
    {
        self.sql.push_str(" ORDER BY ");
        self.sql.push_str(columns);

        return self;
    }

    pub(super) fn Run<F>(&self, connection: &Connection, read: F) -> Result<Vec<Item>, ProjectError>
    where
        F: Fn(&Row<'_>) -> rusqlite::Result<Item>,
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

pub(super) fn Gather(
    connection: &Connection,
    content: Content,
    filter: &Filter,
) -> Result<Vec<Item>, ProjectError>
{
    return match content
    {
        Content::Suites => Suites(connection, filter),
        Content::Documents => Documents(connection, filter),
        Content::Headings => Headings(connection, filter),
        Content::Blocks => Blocks(connection, filter),
        Content::Rows => Rows(connection, filter),
        Content::Nodes => Nodes(connection, filter),
        Content::Statements => Statements(connection, filter),
        Content::Relations => Relations(connection, filter),
        Content::Lineage => Lineage(connection, filter),
        Content::Omissions => Omissions(connection, filter),
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
}
