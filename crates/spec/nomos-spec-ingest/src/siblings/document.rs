//! Putting one document of one suite into the store.

use super::{Sibling, SpecificationStore, IngestError, Sourced, Segment, Archive, ArchiveError, StoreError};

/// The suite a document is being ingested into: which sibling, and the row it occupies.
///
/// The two are never useful apart — the row is where a claim is written and the sibling is
/// what a claim is checked against — and carrying them together is what keeps the ingest
/// verbs inside the argument budget.
#[derive(Clone, Copy)]
pub(super) struct Suite
{
    pub(super) sibling: Sibling,
    pub(super) uid: i64,
}

/// Whether this suite took the identifier.
///
/// `false` where another suite already holds it. The node is left where it is: two suites
/// declaring one identifier is an ecosystem problem, and reassigning would make the last
/// ingest right rather than making the conflict visible.
pub(super) fn Claim_Node_Id(
    store: &mut SpecificationStore,
    node_id: &str,
    node: i64,
    suite: Suite,
) -> Result<bool, IngestError>
{
    let held: Option<String> = store.Suite_Of(node_id)?.map(|(holder, _)| return holder);

    return match held.as_deref()
    {
        Some(holder) if holder != suite.sibling.Suite_Id() => Ok(false),
        Some(_) => Ok(true),
        None =>
        {
            store.Assign_Suite(node, suite.uid)?;
            Ok(true)
        }
    };
}

pub(super) fn Ingest_Document(
    store: &mut SpecificationStore,
    suite: Suite,
    document: &Sourced<'_>,
    node_uid: i64,
) -> Result<u32, IngestError>
{
    let suite_id = suite.sibling.Suite_Id();
    let document_uid = store.Put_Source_Document(document.entry, suite_id, document.text)?;
    let blocks = Segment(document.text);
    store.Put_Source_Blocks(document_uid, &blocks)?;

    for block in &blocks
    {
        Dispose_Block(store, document_uid, block.ordinal, node_uid)?;
    }

    return Ok(u32::try_from(blocks.len()).unwrap_or(u32::MAX));
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
    Wrap_Sql_Result(disposed)?;

    return Ok(());
}

pub(super) fn Read_Text(archive: &mut Archive, entry: &str) -> Result<String, IngestError>
{
    return archive
        .Read_Text(entry)
        .map_err(|error: ArchiveError| return IngestError::Parse(error.to_string()));
}

/// `xvpe-spec-seed:target-adapter.schema.json` — the suite, then the name inside it.
pub(super) fn Qualified_Node_Id(sibling: Sibling, entry: &str) -> String
{
    return format!("{}:{}", sibling.Suite_Id(), Path_Stem(entry));
}

/// The entry's file name, without the archive's top-level directory.
pub(super) fn Path_Stem(entry: &str) -> &str
{
    return entry.rsplit('/').next().unwrap_or(entry);
}

pub(super) fn Slug_Of_Name(name: &str) -> String
{
    let mut slug = String::new();
    let mut pending = false;

    for character in name.chars()
    {
        if character.is_ascii_alphanumeric()
        {
            if pending && !slug.is_empty()
            {
                slug.push('-');
            }
            pending = false;
            slug.extend(character.to_uppercase());
        }
        else
        {
            pending = true;
        }
    }

    return slug;
}

pub(super) fn Wrap_Sql_Result<Value>(result: rusqlite::Result<Value>) -> Result<Value, IngestError>
{
    return result.map_err(|error| IngestError::Store(StoreError::Sql(error.to_string())));
}
