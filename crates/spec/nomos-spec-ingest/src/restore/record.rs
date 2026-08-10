//! Writing what was recognised into the store, and finding it again by name.

use super::*;

/// I5 — restores every family across the corpus into a store that already holds it.
///
/// Takes the whole document set rather than one volume at a time, because both things
/// that have to be unique are corpus-wide. A minted identifier is stable across databases,
/// so two members colliding is a collision wherever they live; and a bare name resolving
/// to one node is a claim about the corpus, not about a file. Restoring volume by volume
/// would make both answers depend on the order the directory was read in.
///
/// Every document must already be ingested. Restoring from text the store never saw would
/// make the source-truth gate optional for exactly the content the restoration cares
/// about, and a node whose lineage points at a block nobody hashed is a node with a
/// provenance nothing checked.
///
/// # Errors
///
/// Returns [`IngestError::Parse`] if a document is not in the store at that revision or
/// two members collide, and [`IngestError::Store`] on any store failure.
pub fn Restore(
    store: &mut SpecificationStore,
    revision: &str,
    documents: &BTreeMap<String, String>,
) -> Result<RestorationReport, IngestError>
{
    let located = Located(store, revision, documents)?;
    let members: Vec<Member> = located.iter().map(|(_, member)| return member.clone()).collect();
    Refuse_Collisions(&members)?;

    let mut report = RestorationReport {
        ambiguous_names: Ambiguous(&members),
        ..RestorationReport::default()
    };
    for (document_uid, member) in located
    {
        Record(store, document_uid, member, &mut report)?;
    }

    return Ok(report);
}

/// Every member every document declares, with the store row its document occupies.
///
/// Read in full before anything is written, because a collision is only visible across
/// documents and half a restoration is harder to undo than none.
pub(super) fn Located(
    store: &mut SpecificationStore,
    revision: &str,
    documents: &BTreeMap<String, String>,
) -> Result<Vec<(i64, Member)>, IngestError>
{
    let mut located: Vec<(i64, Member)> = Vec::new();

    for (document, markdown) in documents
    {
        let document_uid = Document_Uid(store, revision, document)?;
        for member in Extract(document, markdown)?
        {
            located.push((document_uid, member));
        }
    }

    return Ok(located);
}

/// Mints one member's node, ties it to its text, and gives it its name.
pub(super) fn Record(
    store: &mut SpecificationStore,
    document_uid: i64,
    member: Member,
    report: &mut RestorationReport,
) -> Result<(), IngestError>
{
    let node_uid = store.Upsert_Node(
        &member.id,
        member.family.Node_Kind(),
        "canonical",
        "record",
        &member.name,
    )?;

    Trace(store, document_uid, &member, node_uid)?;
    Claim_Alias(store, &member, node_uid, report)?;
    report.members.push(member);

    return Ok(());
}

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
pub(super) fn Trace(
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

/// Points an alias at the node, unless the name is contested.
///
/// A name more than one restored member claims is left unclaimed, and one another authority
/// already owns is reported and left where it is — a restoration may not repoint a name it
/// did not mint.
pub(super) fn Claim_Alias(
    store: &mut SpecificationStore,
    member: &Member,
    node_uid: i64,
    report: &mut RestorationReport,
) -> Result<(), IngestError>
{
    let Some(alias) = &member.alias
    else
    {
        return Ok(());
    };
    if report.ambiguous_names.contains(alias)
    {
        return Ok(());
    }

    if !Alias(store, alias, node_uid)?
    {
        report.contested_aliases.push(alias.clone());
    }

    return Ok(());
}

/// Names claimed by more than one restored member.
///
/// Reported and withheld rather than resolved by a rule such as "the domain model wins".
/// Two nodes really do carry the name; picking one is an answer the corpus does not give.
pub(super) fn Ambiguous(members: &[Member]) -> Vec<String>
{
    let mut claims: BTreeMap<&str, u32> = BTreeMap::new();
    for alias in members.iter().filter_map(|member| return member.alias.as_deref())
    {
        let counter = claims.entry(alias).or_insert(0);
        *counter = counter.saturating_add(1);
    }

    return claims
        .into_iter()
        .filter(|(_, claimed)| return *claimed > 1)
        .map(|(alias, _)| return alias.to_owned())
        .collect();
}

pub(super) fn Document_Uid(
    store: &SpecificationStore,
    revision: &str,
    document: &str,
) -> Result<i64, IngestError>
{
    return store
        .Connection()
        .query_row(
            "SELECT uid FROM source_documents WHERE path = ?1 AND revision = ?2",
            rusqlite::params![document, revision],
            |row| row.get(0),
        )
        .map_err(|_| {
            return IngestError::Parse(format!(
                "{document} is not in the store at {revision}, so there is nothing for a \
                 restored node to trace back to"
            ));
        });
}

pub(super) fn Dispose_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), IngestError>
{
    let disposed = store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        rusqlite::params![document_uid, ordinal, node_uid],
    );
    Sql(disposed)?;

    return Ok(());
}

/// Whether the alias now resolves to this node.
///
/// `false` where something else already owns it. Reported rather than ignored: an alias
/// silently pointing at another node makes "resolve by name" answer confidently and
/// wrongly, which is worse than not resolving at all.
pub(super) fn Alias(store: &SpecificationStore, alias: &str, node_uid: i64) -> Result<bool, IngestError>
{
    let inserted = store.Connection().execute(
        "INSERT OR IGNORE INTO node_aliases (alias, node_uid) VALUES (?1, ?2)",
        rusqlite::params![alias, node_uid],
    );
    Sql(inserted)?;

    let selected_owner = store.Connection().query_row(
        "SELECT node_uid FROM node_aliases WHERE alias = ?1",
        rusqlite::params![alias],
        |row| row.get(0),
    );
    let owner: i64 = Sql(selected_owner)?;

    return Ok(owner == node_uid);
}

/// Resolves an identifier or an authored name to a node.
///
/// The alias table is what makes `WorkspaceContext` answer as well as `CDM-WORKSPACECONTEXT`.
/// Without it a restored concept resolves only by the identifier this build minted, and the
/// name the corpus actually uses would find nothing.
///
/// # Errors
///
/// Returns [`IngestError::Store`] on any store failure.
pub fn Resolve(store: &SpecificationStore, name: &str) -> Result<Option<i64>, IngestError>
{
    if let Some(uid) = store.Node_Uid(name)?
    {
        return Ok(Some(uid));
    }

    return Sql(store
        .Connection()
        .query_row(
            "SELECT node_uid FROM node_aliases WHERE alias = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|error| {
            return match error
            {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            };
        }));
}

pub(super) fn Sql<T>(result: rusqlite::Result<T>) -> Result<T, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}
