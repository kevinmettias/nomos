//! The record graph: its nodes, one hop around a node, and its families.

use super::super::{Columns, Connection, Filter, Item, Name, Narrow_To_Nodes, ProjectError, Query, Value};
use std::collections::BTreeMap;

pub(crate) fn Gather_Nodes(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let mut query = Query::On(
        "SELECT n.node_id, n.kind, n.authority, n.representation, n.title, s.suite_id
         FROM nodes n LEFT JOIN suites s ON s.uid = n.suite_uid
         WHERE n.deleted_at IS NULL",
    );
    Narrow_To_Nodes(&mut query, filter);

    return query.Ordered_By("n.node_id").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let node = columns.Text()?;
        let kind = columns.Text()?;
        let authority = columns.Text()?;
        let representation = columns.Text()?;
        let title = columns.Text()?;
        let suite = columns.Text()?;

        return Ok(Item::Of(&node)
            .With(Name("kind"), Value(&kind))
            .With(Name("authority"), Value(&authority))
            .With(Name("representation"), Value(&representation))
            .With(Name("title"), Value(&title))
            .With(Name("suite"), Value(&suite)));
    });
}

/// Every node one relation away from the node `filter.node_id` names, with what it declared.
///
/// # Why one hop, and why it is not a parameter
///
/// `OD-PROJECT-005` measured the record graph: from any record, one hop reaches a median of 7
/// of 232 records, two reaches a median of 43 and ranges from 7 to 161 by starting point, and
/// three reaches 62 per cent of the corpus. A hop bound stops bounding after the first, because
/// `relates-to` is symmetric and carries 93 per cent of the edges. A caller wanting a
/// neighbour's neighbourhood asks about the neighbour.
///
/// # Why every relation type is followed
///
/// The same record measured the alternative. Restricted to `affects`, `affected_by` and
/// `supersedes`, the median reach is the starting record itself at any hop count, and 177 of
/// 232 records carry no directional edge at all. A traversal that skipped `relates-to` would
/// reach nothing for three quarters of the corpus.
///
/// # Why status is reported and not filtered
///
/// A neighbour's declared lifecycle status rides along so a pack says which of the decisions
/// around its subject are still open — ten of this repository's are. Filtering by it was
/// refused: keeping only `accepted` would hide exactly the unsettled questions an implementer
/// needs flagged, and keeping only `open` would hide the settled ground. A neighbour that
/// declared no status at all — a referenced placeholder carries no front-matter row — is
/// reported with an empty one rather than dropped, for the reason `OD-RULES-003` gives
/// generally: an absence is said rather than inferred.
pub(crate) fn Gather_Neighbourhood(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    // `DISTINCT` because `relations` holds each authored edge and its inverse, so a neighbour
    // is reached twice; `s.node_id <> n.node_id` because a subject is not its own neighbour.
    let mut query = Query::On(
        "SELECT DISTINCT n.node_id, n.kind, n.title, COALESCE(front.status, '')
         FROM nodes n
         JOIN relations r ON r.from_node_uid = n.uid OR r.to_node_uid = n.uid
         JOIN nodes s ON (s.uid = r.from_node_uid OR s.uid = r.to_node_uid) AND s.node_id <> n.node_id
         LEFT JOIN record_front_matter front ON front.node_uid = n.uid
         WHERE n.deleted_at IS NULL",
    );
    query.Equal("s.node_id", filter.node_id.as_ref());

    // By identity, never by the order the walk reached them: two selections of one store must
    // be identical, which is what the freshness sidecar's determinism rests on.
    return query.Ordered_By("n.node_id").Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let node = columns.Text()?;
        let kind = columns.Text()?;
        let title = columns.Text()?;
        let status = columns.Text()?;

        return Ok(Item::Of(&node)
            .With(Name("kind"), Value(&kind))
            .With(Name("title"), Value(&title))
            .With(Name("status"), Value(&status)));
    });
}

/// Every edge between two record families, counted, rather than every edge between two records.
///
/// # What a family is
///
/// The identifier without its ordinal: `OD-RULES-027` and `OD-RULES-029` are both `OD-RULES`,
/// `D-134` is `D`. A node whose identifier carries no trailing ordinal is a family of one
/// rather than a node outside every family, and that is what keeps an edge to it from becoming
/// a dangling end -- the grouping is total, so every edge has a family at both ends.
///
/// # Why this is not `identifier_prefix`
///
/// `OD-PROJECT-006` measured the difference. Narrowing by prefix draws one family's internal
/// edges and every edge leaving it as a dangling end; it answers "what is inside OD-RULES",
/// which is a slice of the one resolution that already exists. This answers "how do OD-RULES
/// and OD-GATE stand to each other", which is a second resolution, and no filter can express
/// it because the answer is about nodes the filter would have excluded.
///
/// # Why the aggregation is in Rust rather than in the query
///
/// A family is the identifier minus a trailing ordinal, and expressing that in SQLite's string
/// functions would put the definition of a family in a place no test can reach directly. The
/// query does what a query is good at -- joining the edges -- and [`Family_Of`] holds what a
/// family is, in one function with its own tests.
pub(crate) fn Gather_Families(connection: &Connection, filter: &Filter) -> Result<Vec<Item>, ProjectError>
{
    let _ = filter;
    // Ordered here as well as grouped below: the aggregation is deterministic on its own, and
    // ordering the input too means a failure reads the same way twice.
    let edges = Query::On(
        "SELECT f.node_id, t.node_id
         FROM relations r
         JOIN nodes f ON f.uid = r.from_node_uid
         JOIN nodes t ON t.uid = r.to_node_uid
         WHERE f.deleted_at IS NULL AND t.deleted_at IS NULL",
    )
    .Ordered_By("f.node_id, t.node_id")
    .Run(connection, |row| {
        let mut columns = Columns::Of(row);
        let from = columns.Text()?;
        let to = columns.Text()?;

        // An edge carried as an item, because that is what `Run` hands back. It is folded into
        // the real answer below and never reaches a projection.
        return Ok(Item::Of(&from).With(Name("to"), Value(&to)));
    })?;

    return Ok(Between_Families(&edges));
}

/// `edges` rolled into one item per ordered pair of distinct families, carrying how many
/// edges run between them.
///
/// A family's internal edges are dropped rather than counted as a self-edge: the question this
/// answers is how families stand to one another, and a family's relationship with itself is
/// the resolution the full diagram already shows.
///
/// `BTreeMap` rather than a hash map because the output order is the answer's order, and two
/// selections of one store must be identical.
fn Between_Families(edges: &[Item]) -> Vec<Item>
{
    let mut counted: BTreeMap<(String, String), usize> = BTreeMap::new();
    for edge in edges
    {
        let from = Family_Of(&edge.identity);
        let to = Family_Of(edge.Field("to").unwrap_or_default());
        if from == to
        {
            continue;
        }
        let tally = counted.entry((from, to)).or_insert(0);
        *tally = tally.saturating_add(1);
    }

    return counted
        .into_iter()
        .map(|((from, to), edges)| {
            return Item::Of(&format!("{from} -> {to}"))
                .With(Name("from"), Value(from.as_str()))
                .With(Name("to"), Value(to.as_str()))
                .With(Name("edges"), Value(&edges.to_string()));
        })
        .collect();
}

/// The family an identifier belongs to: itself without a trailing ordinal.
///
/// A node carrying no ordinal is its own family rather than no family at all. That is what
/// makes the grouping total, and a total grouping is what stops an edge to such a node from
/// being drawn as a dangling end.
fn Family_Of(identity: &str) -> String
{
    let Some((family, ordinal)) = identity.rsplit_once('-')
    else
    {
        return identity.to_owned();
    };

    if !Is_Ordinal(ordinal)
    {
        return identity.to_owned();
    }

    return family.to_owned();
}

/// Whether a hyphen-split tail is an ordinal rather than more of the name.
///
/// A trailing run of ASCII digits is what an ordinal is, and nothing else is: `relates-to`
/// and `preserved-normalized` split at their last hyphen too, and the tail `to` is not a
/// number. An empty tail -- the one `OD-RULES-` leaves -- is not one either, which is why
/// emptiness is answered here rather than by the caller that had to remember it.
fn Is_Ordinal(ordinal: &str) -> bool
{
    return !ordinal.is_empty() && ordinal.bytes().all(|byte| return byte.is_ascii_digit());
}
