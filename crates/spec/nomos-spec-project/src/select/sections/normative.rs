//! What a node declares: its normative statements and the relations it stands in.

use super::super::{
    Columns, Connection, FirstColumn, Filter, Item, Name, ProjectError, Query, SecondColumn, Value,
};

pub(crate) fn Gather_Statements(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT s.statement_id, s.kind, n.node_id, s.canonical_text, s.canonical_hash,
                s.supersedes_hash
         FROM normative_statements s JOIN nodes n ON n.uid = s.node_uid
         WHERE 1 = 1",
    );
    query.Equal("s.kind", filter.kind.as_ref());
    query.Prefix("s.statement_id", filter.identifier_prefix.as_ref());
    query.Equal("n.node_id", filter.node_id.as_ref());

    return query.Ordered_By("s.statement_id").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let statement = columns.Text()?;
        let kind = columns.Text()?;
        let node = columns.Text()?;
        let text = columns.Text()?;
        let hash = columns.Text()?;
        let supersedes = columns.Text()?;

        return Ok(Item::Of(&statement)
            .With(Name("kind"), Value(&kind))
            .With(Name("node"), Value(&node))
            .With(Name("supersedes"), Value(&supersedes))
            .With(Name("hash"), Value(&hash))
            .Carrying(&text));
    });
}

pub(crate) fn Gather_Relations(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT f.node_id, r.relation_type, t.node_id, y.tier, s.suite_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         JOIN relation_types y ON y.name = r.relation_type
         LEFT JOIN suites s ON s.uid = f.suite_uid
         WHERE 1 = 1",
    );
    query.Equal("r.relation_type", filter.relation_type.as_ref());
    query.Equal("s.suite_id", filter.suite.as_ref());
    query.Prefix("f.node_id", filter.identifier_prefix.as_ref());
    query.Either(FirstColumn("f.node_id"), SecondColumn("t.node_id"), filter.node_id.as_ref());

    return query
        .Ordered_By("f.node_id, r.relation_type, t.node_id")
        .Run(connection, |row| {
            let mut columns = Columns::Of(row);
            let from = columns.Text()?;
            let relation = columns.Text()?;
            let to = columns.Text()?;
            let tier = columns.Text()?;

            return Ok(Item::Of(&format!("{from} {relation} {to}"))
                .With(Name("from"), Value(&from))
                .With(Name("relation"), Value(&relation))
                .With(Name("to"), Value(&to))
                .With(Name("tier"), Value(&tier)));
        });
}
