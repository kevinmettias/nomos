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

pub(super) fn Text(row: &Row<'_>, index: usize) -> rusqlite::Result<String>
{
    return row.get::<usize, Option<String>>(index).map(Option::unwrap_or_default);
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
