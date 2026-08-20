//! Writing content into the schema through a caller's transaction.

use nomos_spec_model::{ContentHash, SourceBlock, TableRow, Table_Defects, Table_Rows};
use rusqlite::{Connection, OptionalExtension, params};

use crate::NodeRow;
use crate::StoreError;
use crate::read::columns::Columns;

/// Writes blocks and their typed rows through a caller's transaction.
///
/// Free rather than a method, and taking a [`Connection`] rather than the store, because
/// [`rusqlite::Transaction`] dereferences to one: the seed, a re-ingest and an authoring
/// commit all reach this same writer, one of them inside a transaction that spans several
/// documents. A second entry point that knew how to write a block would be a second place
/// that could be wrong about what a block is.
///
/// # Errors
///
/// Returns [`StoreError::Table`] if a block's tables do not each carry exactly one
/// delimiter, and [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<usize, StoreError>
{
    Assert_Tables_Are_Sound(document_uid, blocks)?;
    Insert_Blocks(connection, document_uid, blocks)?;

    let uids = Block_Uids(connection, document_uid)?;

    Insert_Table_Rows(connection, document_uid, blocks, &uids)?;

    return Ok(blocks.len());
}

/// Every table in these blocks is one this store will take.
fn Assert_Tables_Are_Sound(document_uid: i64, blocks: &[SourceBlock]) -> Result<(), StoreError>
{
    for block in blocks
    {
        let defects = Table_Defects(&Table_Rows(block));
        if let Some(defect) = defects.first()
        {
            return Err(StoreError::Table {
                document_uid,
                ordinal: block.ordinal,
                cause: defect.to_string(),
            });
        }
    }

    return Ok(());
}

/// The blocks themselves, updated in place where the document already held one.
///
/// Not `INSERT OR REPLACE`. REPLACE deletes the conflicting row and inserts a new one, which
/// hands the block a new `uid` — and `uid` is what every lineage and omission row points at.
/// Re-ingesting a document would silently renumber its blocks and take their dispositions
/// with them.
fn Insert_Blocks(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
) -> Result<(), StoreError>
{
    let mut insert = connection.prepare(
        "INSERT INTO source_blocks
         (document_uid, ordinal, kind, heading_path, text, content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(document_uid, ordinal) DO UPDATE SET
             kind = excluded.kind,
             heading_path = excluded.heading_path,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        insert.execute(params![
            document_uid,
            block.ordinal,
            crate::record::Kind_Label(block.kind),
            block.heading_path.join(" / "),
            block.text,
            block.Content_Hash().As_Str(),
            block.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Every block's uid in one crossing, indexed by ordinal.
///
/// Asking per block cost one round trip for each row of the document, to learn surrogates the
/// same document already decides as a set.
fn Block_Uids(
    connection: &Connection,
    document_uid: i64,
) -> Result<std::collections::BTreeMap<u32, i64>, StoreError>
{
    let mut every_uid = connection
        .prepare("SELECT ordinal, uid FROM source_blocks WHERE document_uid = ?1")?;
    let found = every_uid.query_map(params![document_uid], |row| {
        return Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?));
    })?;
    let mut block_uids = std::collections::BTreeMap::new();

    for entry in found
    {
        let (ordinal, uid) = entry?;
        block_uids.insert(ordinal, uid);
    }

    return Ok(block_uids);
}

/// The rows of every table these blocks carry, under the block that carries them.
///
/// Same reasoning as [`Insert_Blocks`]: updated in place rather than replaced, so a re-ingest
/// does not hand a row a new uid.
fn Insert_Table_Rows(
    connection: &Connection,
    document_uid: i64,
    blocks: &[SourceBlock],
    uids: &std::collections::BTreeMap<u32, i64>,
) -> Result<(), StoreError>
{
    let mut insert_row = connection.prepare(
        "INSERT INTO source_table_rows
         (source_block_uid, ordinal, table_ordinal, kind, cells_json, text,
          content_hash, normalized_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(source_block_uid, ordinal) DO UPDATE SET
             table_ordinal = excluded.table_ordinal,
             kind = excluded.kind,
             cells_json = excluded.cells_json,
             text = excluded.text,
             content_hash = excluded.content_hash,
             normalized_hash = excluded.normalized_hash",
    )?;
    for block in blocks
    {
        let rows = Table_Rows(block);
        let uid = Block_Uid_Of(uids, document_uid, block)?;

        Insert_Rows_Under(&mut insert_row, uid, &rows)?;
    }

    return Ok(());
}

/// The surrogate the block just inserted landed under.
///
/// [`Insert_Blocks`] put every block in, so a missing ordinal is a broken invariant rather
/// than a row that has not arrived yet, and it says so.
fn Block_Uid_Of(
    uids: &std::collections::BTreeMap<u32, i64>,
    document_uid: i64,
    block: &SourceBlock,
) -> Result<i64, StoreError>
{
    let found = uids.get(&block.ordinal).copied();

    return found.ok_or_else(|| {
        return StoreError::Sql(format!(
            "source block {} of document {document_uid} has no uid after insertion",
            block.ordinal
        ));
    });
}

/// One block's table rows, through a prepared insert.
fn Insert_Rows_Under(
    insert_row: &mut rusqlite::Statement<'_>,
    uid: i64,
    rows: &[TableRow],
) -> Result<(), StoreError>
{
    for row in rows
    {
        let cells = serde_json::to_string(&row.cells)
            .map_err(|error| return StoreError::Sql(error.to_string()))?;

        insert_row.execute(params![
            uid,
            row.ordinal,
            row.table_ordinal,
            row.kind.Label(),
            cells,
            row.text,
            row.Content_Hash().As_Str(),
            row.Normalized_Hash().As_Str(),
        ])?;
    }

    return Ok(());
}

/// Every row a mapped query produced, or the first failure it hit.
pub(crate) fn Collected<T>(
    rows: impl Iterator<Item = rusqlite::Result<T>>,
) -> Result<Vec<T>, StoreError>
{
    let mut collected = Vec::new();
    for row in rows
    {
        collected.push(row?);
    }

    return Ok(collected);
}

/// Writes bytes, addressed by content, through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Blob(connection: &Connection, content: &[u8]) -> Result<i64, StoreError>
{
    let digest = ContentHash::Of_Bytes(content);
    let existing: Option<i64> = connection
        .query_row(
            "SELECT uid FROM blobs WHERE sha256 = ?1",
            params![digest.As_Str()],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(uid) = existing
    {
        return Ok(uid);
    }
    connection.execute(
        "INSERT INTO blobs (sha256, byte_length, content) VALUES (?1, ?2, ?3)",
        params![
            digest.As_Str(),
            i64::try_from(content.len()).unwrap_or(i64::MAX),
            content
        ],
    )?;

    return Ok(connection.last_insert_rowid());
}

/// Writes a document and its bytes through a caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Source_Document(
    connection: &Connection,
    path: &str,
    revision: &str,
    content: &str,
) -> Result<i64, StoreError>
{
    let blob_uid = Write_Blob(connection, content.as_bytes())?;

    connection.execute(
        "INSERT OR IGNORE INTO source_documents (path, revision, blob_uid) VALUES (?1, ?2, ?3)",
        params![path, revision, blob_uid],
    )?;

    return Ok(connection.query_row(
        "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
        params![path, revision],
        |row| row.get(0),
    )?);
}

/// Writes a node, upgrading a placeholder but never overwriting a real one, through a
/// caller's transaction.
///
/// # Errors
///
/// Returns [`StoreError`] on any SQL failure.
pub(crate) fn Write_Node(connection: &Connection, node: NodeRow<'_>) -> Result<i64, StoreError>
{
    Upsert_Node_Row(connection, node)?;

    return Node_Uid_By_Id(connection, node.node_id);
}

/// Inserts a node, or upgrades a placeholder — never a real one — through the authority
/// guard on the conflict clause.
fn Upsert_Node_Row(connection: &Connection, node: NodeRow<'_>) -> Result<(), StoreError>
{
    use super::EXTERNAL;

    let NodeRow { node_id, kind, authority, representation, title } = node;
    connection.execute(
        "INSERT INTO nodes (node_id, kind, authority, representation, title)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(node_id) DO UPDATE SET
             kind = excluded.kind,
             authority = excluded.authority,
             representation = excluded.representation,
             title = excluded.title
         WHERE nodes.authority = ?6",
        params![node_id, kind, authority, representation, title, EXTERNAL],
    )?;

    return Ok(());
}

/// The surrogate a node's identifier resolves to.
fn Node_Uid_By_Id(connection: &Connection, node_id: &str) -> Result<i64, StoreError>
{
    return Ok(connection.query_row(
        "SELECT uid FROM nodes WHERE node_id = ?1",
        params![node_id],
        |row| row.get(0),
    )?);
}

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
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    Write_One_Relation(connection, from_node_id, relation_type, to_node_id)?;

    if let Some(inverse) = Inverse_Of(connection, relation_type)?
    {
        Write_One_Relation(connection, to_node_id, &inverse, from_node_id)?;
    }

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

/// The declared constraint for a relation type, if the type is registered.
///
/// Every type reaching this point is registered through [`super::SpecificationStore::Put_Relation_Type`],
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
    from_node_id: &str,
    relation_type: &str,
    to_node_id: &str,
) -> Result<(), StoreError>
{
    let Some(from) = Fetch_Endpoint(connection, from_node_id)?
    else
    {
        return Ok(());
    };
    let Some(to) = Fetch_Endpoint(connection, to_node_id)?
    else
    {
        return Ok(());
    };

    Enforce_Constraint(connection, relation_type, &from, &to)?;

    connection.execute(
        "INSERT OR IGNORE INTO relations (from_node_uid, relation_type, to_node_uid)
         VALUES (?1, ?2, ?3)",
        params![from.uid, relation_type, to.uid],
    )?;

    return Ok(());
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
    use super::EXTERNAL;

    let Some(constraint) = Fetch_Constraint(connection, relation_type)?
    else
    {
        return Ok(());
    };

    if from.authority != EXTERNAL
    {
        Assert_Admits(AdmissionCheck { relation_type, role: "domain", endpoint: from, admits: &constraint.domain })?;
    }
    if to.authority != EXTERNAL
    {
        Assert_Admits(AdmissionCheck { relation_type, role: "range", endpoint: to, admits: &constraint.range })?;
    }

    return Assert_Within_Cardinality(
        connection,
        CardinalityCheck { relation_type, from, to_uid: to.uid, max_per_node: constraint.max_per_node },
    );
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
fn Assert_Admits(check: AdmissionCheck<'_>) -> Result<(), StoreError>
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
fn Assert_Within_Cardinality(connection: &Connection, check: CardinalityCheck<'_>) -> Result<(), StoreError>
{
    if Edge_Already_Exists(connection, &check)?
    {
        return Ok(());
    }

    return Assert_Cap_Not_Exceeded(connection, &check);
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
