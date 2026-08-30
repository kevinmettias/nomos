//! Writing relation edges, and the constraint each relation type checks them against,
//! through a caller's transaction.

use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;
use crate::read::columns::Columns;
use crate::store::relation::{FromNodeId, ToNodeId, TypeName as RelationTypeName};

/// Records an edge and its inverse through a caller's transaction.
///
/// Both directions run the same checks, because each is its own row in `relation_types`
/// with its own domain, range and cardinality — `answers` and `answered_by` are not one
/// constraint read backwards, they are two, and `OD-SPEC-012` says why.
///
/// # Errors
///
/// Returns [`StoreError::RelationEndpoint`] if an endpoint's kind is not one the relation
/// type admits there, [`StoreError::RelationCardinality`] if the edge would exceed the
/// type's declared cap, and [`StoreError`] on any SQL failure.
pub(crate) fn Write_Relation(
    connection: &Connection,
    from_node_id: FromNodeId<'_>,
    relation_type: RelationTypeName<'_>,
    to_node_id: ToNodeId<'_>,
) -> Result<(), StoreError>
{
    Write_One_Relation(connection, from_node_id, relation_type, to_node_id)?;

    if let Some(inverse) = Inverse_Of(connection, relation_type.0)?
    {
        Write_One_Relation(
            connection,
            FromNodeId(to_node_id.0),
            RelationTypeName(&inverse),
            ToNodeId(from_node_id.0),
        )?;
    }

    return Ok(());
}

/// One edge, checked against its relation type's declared constraint and then written.
///
/// A placeholder endpoint (`authority = EXTERNAL`, minted by `Reference_Node` for a target
/// nothing has ingested yet) is exempt from the domain/range check at its own end: its kind
/// is the sentinel `unknown`, which is not a fact about the node yet, and `OD-SPEC-012`
/// records that the check is deferred rather than widened to admit the sentinel as if it
/// were a real kind. Cardinality is not exempted the same way, because it counts edges from
/// a real endpoint (the `from` side always resolves to a concrete node by the time this
/// runs) rather than judging the placeholder's kind.
fn Write_One_Relation(
    connection: &Connection,
    from_node_id: FromNodeId<'_>,
    relation_type: RelationTypeName<'_>,
    to_node_id: ToNodeId<'_>,
) -> Result<(), StoreError>
{
    let Some(from) = Fetch_Endpoint(connection, from_node_id.0)?
    else
    {
        return Ok(());
    };
    let Some(to) = Fetch_Endpoint(connection, to_node_id.0)?
    else
    {
        return Ok(());
    };

    Enforce_Constraint(connection, relation_type.0, &from, &to)?;

    connection.execute(
        "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
         VALUES (?1, ?2, ?3)",
        params![from.uid, relation_type.0, to.uid],
    )?;

    return Ok(());
}

/// One node, as an edge's endpoint: its own identifier, its surrogate, its kind, and
/// whether it is real yet.
///
/// Carries `node_id` alongside the row `Fetch_Endpoint` reads, rather than leaving the
/// caller to keep the two paired, so a refusal built from an `Endpoint` can always name the
/// node it is about without a second parameter threaded beside it.
struct Endpoint
{
    node_id: String,
    uid: i64,
    kind: String,
    authority: String,
}

/// The node named, if one exists.
///
/// An edge to an identifier no node holds matches nothing and writes nothing — silently,
/// at exit 0. That is `Write_Relation`'s existing contract (`OD-SPEC-011` and the sibling
/// suite tests both rely on it), and this preserves it: a missing endpoint short-circuits
/// before any constraint is checked, rather than becoming a new refusal.
fn Fetch_Endpoint(connection: &Connection, node_id: &str) -> Result<Option<Endpoint>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT uid, kind, authority FROM nodes WHERE node_id = ?1",
            params![node_id],
            |row| {
                let mut columns = Columns::Of(row);
                return Ok(Endpoint {
                    node_id: node_id.to_owned(),
                    uid: columns.Next()?,
                    kind: columns.Next()?,
                    authority: columns.Next()?,
                });
            },
        )
        .optional()?);
}

/// The edge is checked against its relation type's declared constraint, if the type
/// declares one.
///
/// A placeholder endpoint (`authority = EXTERNAL`, minted by `Reference_Node` for a target
/// nothing has ingested yet) is exempt from the domain/range check at its own end: its kind
/// is the sentinel `unknown`, which is not a fact about the node yet, and `OD-SPEC-012`
/// records that the check is deferred rather than widened to admit the sentinel as if it
/// were a real kind. Cardinality is not exempted the same way, because it counts edges from
/// a real endpoint (the `from` side always resolves to a concrete node by the time this
/// runs) rather than judging the placeholder's kind.
fn Enforce_Constraint(
    connection: &Connection,
    relation_type: &str,
    from: &Endpoint,
    to: &Endpoint,
) -> Result<(), StoreError>
{
    use crate::EXTERNAL;

    let Some(constraint) = Fetch_Constraint(connection, relation_type)?
    else
    {
        return Ok(());
    };

    if from.authority != EXTERNAL
    {
        Assert_Admits(&AdmissionCheck { relation_type, role: "domain", endpoint: from, admits: &constraint.domain })?;
    }
    if to.authority != EXTERNAL
    {
        Assert_Admits(&AdmissionCheck { relation_type, role: "range", endpoint: to, admits: &constraint.range })?;
    }

    return Assert_Within_Cardinality(
        connection,
        &CardinalityCheck { relation_type, from, to_uid: to.uid, max_per_node: constraint.max_per_node },
    );
}

/// The declared constraint for a relation type, if the type is registered.
///
/// Every type reaching this point is registered through [`super::super::SpecificationStore::Put_Relation_Type`],
/// which refuses to register one with nothing declared — so in practice this is always
/// `Some` for a type the foreign key on `relations.relation_type` would otherwise admit.
/// It stays an `Option` rather than an assumed row because the caller, not this function,
/// is where "no such type" is diagnosable.
fn Fetch_Constraint(
    connection: &Connection,
    relation_type: &str,
) -> Result<Option<Constraint>, StoreError>
{
    let found: Option<(String, String, i64)> = connection
        .query_row(
            "SELECT domain_kinds_json, range_kinds_json, max_per_node FROM relation_types
             WHERE name = ?1",
            params![relation_type],
            Read_Constraint_Row,
        )
        .optional()?;
    let Some((domain_json, range_json, max_per_node)) = found
    else
    {
        return Ok(None);
    };

    let domain = Decoded_Kinds(&domain_json)?;
    let range = Decoded_Kinds(&range_json)?;

    return Ok(Some(Constraint {
        domain,
        range,
        max_per_node: u32::try_from(max_per_node).unwrap_or(u32::MAX),
    }));
}

fn Decoded_Kinds(json: &str) -> Result<Vec<String>, StoreError>
{
    return serde_json::from_str(json).map_err(|error| return StoreError::Sql(error.to_string()));
}

/// What one endpoint's admission into a relation type's declared role is checked against.
struct AdmissionCheck<'a>
{
    relation_type: &'a str,
    role: &'static str,
    endpoint: &'a Endpoint,
    admits: &'a [String],
}

/// The endpoint's kind is one the relation type admits at that role, or the refusal names
/// the type, the role, the endpoint and what would have satisfied it.
fn Assert_Admits(check: &AdmissionCheck<'_>) -> Result<(), StoreError>
{
    if check.admits.iter().any(|admitted| return admitted == &check.endpoint.kind)
    {
        return Ok(());
    }

    return Err(StoreError::RelationEndpoint {
        relation_type: check.relation_type.to_owned(),
        role: check.role,
        node_id: check.endpoint.node_id.clone(),
        kind: check.endpoint.kind.clone(),
        admits: check.admits.to_vec(),
    });
}

/// What a new edge from a node is checked against: the relation type's own cardinality cap,
/// and the edge it would add.
struct CardinalityCheck<'a>
{
    relation_type: &'a str,
    from: &'a Endpoint,
    to_uid: i64,
    max_per_node: u32,
}

/// A new edge does not push a node past its type's declared cap.
///
/// An edge that already exists is not new: re-inserting one a re-ingest already wrote must
/// stay idempotent, the same property `INSERT OR IGNORE` gives every other edge, so an
/// existing triple is let through without counting against the cap a second time.
fn Assert_Within_Cardinality(connection: &Connection, check: &CardinalityCheck<'_>) -> Result<(), StoreError>
{
    if Edge_Already_Exists(connection, check)?
    {
        return Ok(());
    }

    return Assert_Cap_Not_Exceeded(connection, check);
}

/// The edge this check is about is already one of the ones counted against the cap.
fn Edge_Already_Exists(connection: &Connection, check: &CardinalityCheck<'_>) -> Result<bool, StoreError>
{
    let already: i64 = connection.query_row(
        "SELECT count(*) FROM relations
         WHERE from_node_uid = ?1 AND relation_type = ?2 AND to_node_uid = ?3",
        params![check.from.uid, check.relation_type, check.to_uid],
        |row| return row.get(0),
    )?;

    return Ok(already > 0);
}

/// How many edges of this type the node already carries has not reached the declared cap.
fn Assert_Cap_Not_Exceeded(connection: &Connection, check: &CardinalityCheck<'_>) -> Result<(), StoreError>
{
    let carried: i64 = connection.query_row(
        "SELECT count(*) FROM relations WHERE from_node_uid = ?1 AND relation_type = ?2",
        params![check.from.uid, check.relation_type],
        |row| return row.get(0),
    )?;
    let carried = u32::try_from(carried).unwrap_or(u32::MAX);

    if carried >= check.max_per_node
    {
        return Err(StoreError::RelationCardinality {
            relation_type: check.relation_type.to_owned(),
            node_id: check.from.node_id.clone(),
            max_per_node: check.max_per_node,
        });
    }

    return Ok(());
}

/// The inverse a relation type declares, if it declares one.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Inverse_Of(
    connection: &Connection,
    relation_type: &str,
) -> Result<Option<String>, StoreError>
{
    return Ok(connection
        .query_row(
            "SELECT inverse_of FROM relation_types WHERE name = ?1",
            params![relation_type],
            |row| row.get(0),
        )
        .optional()?
        .flatten());
}

/// What one relation type declares: the node kinds it admits at each end, and how many
/// edges of it a node may carry.
struct Constraint
{
    domain: Vec<String>,
    range: Vec<String>,
    max_per_node: u32,
}

/// A constraint row, read in the order its `SELECT` names: domain kinds, range kinds,
/// max per node.
fn Read_Constraint_Row(row: &rusqlite::Row<'_>) -> rusqlite::Result<(String, String, i64)>
{
    let mut columns = Columns::Of(row);
    return Ok((columns.Next()?, columns.Next()?, columns.Next()?));
}
