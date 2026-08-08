use crate::store::{AUTHORED, SpecificationStore, StoreError};
use nomos_spec_model::{BlockKind, Parse_Record, Segment};
use rusqlite::params;

/// The records that govern this system, embedded so they travel with the binary.
///
/// A seed that reads from disk would be a seed that silently does nothing when the
/// working directory is somewhere else.
const RECORDS: &[(&str, &str)] = &[
    (
        "docs/records/ARC-SPECDB-001-the-specification-is-a-database.md",
        include_str!("../../../../docs/records/ARC-SPECDB-001-the-specification-is-a-database.md"),
    ),
    (
        "docs/records/D-129-the-store-is-the-identity-substrate.md",
        include_str!("../../../../docs/records/D-129-the-store-is-the-identity-substrate.md"),
    ),
    (
        "docs/records/D-130-no-xvpe-dependency-before-phase-5.md",
        include_str!("../../../../docs/records/D-130-no-xvpe-dependency-before-phase-5.md"),
    ),
    (
        "docs/records/D-131-a-byte-order-mark-belongs-to-the-front-matter-fence.md",
        include_str!(
            "../../../../docs/records/D-131-a-byte-order-mark-belongs-to-the-front-matter-fence.md"
        ),
    ),
    (
        "docs/records/OD-SPEC-001-the-storage-backend-question.md",
        include_str!("../../../../docs/records/OD-SPEC-001-the-storage-backend-question.md"),
    ),
    (
        "docs/records/OD-SPEC-002-the-regression-headline-counts-lines-not-blocks.md",
        include_str!(
            "../../../../docs/records/OD-SPEC-002-the-regression-headline-counts-lines-not-blocks.md"
        ),
    ),
    (
        "docs/records/OD-SPEC-003-the-family-counts-and-the-missing-row-lineage.md",
        include_str!(
            "../../../../docs/records/OD-SPEC-003-the-family-counts-and-the-missing-row-lineage.md"
        ),
    ),
    (
        "docs/records/OD-LEDGER-001-territory-is-declared-not-enforced.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-001-territory-is-declared-not-enforced.md"
        ),
    ),
];

/// Every record identifier this build claims to govern itself by.
///
/// Compared against what a seeded store actually holds, so a record that stops being
/// embedded fails a test rather than quietly leaving the store.
pub const GOVERNING_RECORD_IDS: &[&str] = &[
    "ARC-SPECDB-001",
    "D-129",
    "D-130",
    "D-131",
    "OD-SPEC-001",
    "OD-SPEC-002",
    "OD-SPEC-003",
    "OD-LEDGER-001",
];

/// The relation vocabulary the governing records use.
///
/// Tier `seed` rather than `core` or `extended`: the real vocabulary is ADR-ARTIFACT-
/// GRAPH-002's, and it arrives with the corpus. Guessing a tier here would put an
/// invented answer where a recorded one belongs.
const RELATION_TYPES: &[(&str, &str)] = &[
    ("supersedes", "superseded_by"),
    ("superseded_by", "supersedes"),
    ("affects", "affected_by"),
    ("affected_by", "affects"),
];

const SEED_TIER: &str = "seed";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedReport
{
    pub records: u32,
    pub headings: u32,
    pub blocks: u32,
    pub relations: u32,
    /// Targets that had to be created as placeholders because nothing has ingested them
    /// yet. Reported rather than counted, so a growing list is visible.
    pub references: Vec<String>,
}

/// Writes this repository's own governing records into a store.
///
/// Deliberately explicit rather than part of a migration. A migration that seeded rows
/// would mean no store is ever empty, and the bundle importer refuses a non-empty store —
/// so the two would contradict each other and one of them would have to be weakened.
///
/// # Errors
///
/// Returns [`StoreError::Record`] if an embedded record does not read, and
/// [`StoreError`] on any SQL failure.
pub fn Seed_Governing_Records(store: &mut SpecificationStore) -> Result<SeedReport, StoreError>
{
    let mut report = SeedReport {
        records: 0,
        headings: 0,
        blocks: 0,
        relations: 0,
        references: Vec::new(),
    };

    for (name, _) in RELATION_TYPES
    {
        store.Put_Relation_Type(name, SEED_TIER)?;
    }
    for (name, inverse) in RELATION_TYPES
    {
        store.Pair_Relation_Type(name, inverse)?;
    }

    for (path, text) in RECORDS
    {
        let record = Parse_Record(text).map_err(|error| StoreError::Record {
            path: (*path).to_owned(),
            cause: error.to_string(),
        })?;

        let node_uid = store.Upsert_Node(
            &record.front_matter.id,
            &record.front_matter.kind,
            &record.front_matter.authority,
            "document",
            &record.front_matter.title,
        )?;

        let document_uid = store.Put_Source_Document(path, AUTHORED, text)?;
        let blocks = Segment(&record.body);
        store.Put_Source_Blocks(document_uid, &blocks)?;

        for block in &blocks
        {
            if block.kind == BlockKind::Heading
            {
                Put_Heading(store, document_uid, block.ordinal, &block.text)?;
                Dispose_Heading(store, document_uid, block.ordinal, node_uid)?;
                report.headings = report.headings.saturating_add(1);
            }

            Dispose_Block(store, document_uid, block.ordinal, node_uid)?;
            report.blocks = report.blocks.saturating_add(1);
        }

        report.records = report.records.saturating_add(1);
    }

    // A second pass, because a relation may name a record later in the list and an edge
    // to a node that does not exist yet would become a placeholder that never resolves.
    for (path, text) in RECORDS
    {
        let record = Parse_Record(text).map_err(|error| StoreError::Record {
            path: (*path).to_owned(),
            cause: error.to_string(),
        })?;

        for relation in &record.front_matter.relations
        {
            if store.Node_Uid(&relation.target)?.is_none()
            {
                store.Reference_Node(&relation.target)?;
                report.references.push(relation.target.clone());
            }

            store.Put_Relation(&record.front_matter.id, &relation.relation, &relation.target)?;
            report.relations = report.relations.saturating_add(1);
        }
    }

    return Ok(report);
}

fn Put_Heading(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    text: &str,
) -> Result<(), StoreError>
{
    let depth = text.chars().take_while(|character| *character == '#').count();

    store.Connection().execute(
        "INSERT OR IGNORE INTO source_headings (document_uid, ordinal, depth, title)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            document_uid,
            ordinal,
            i64::try_from(depth).unwrap_or(0),
            text.trim_start_matches('#').trim()
        ],
    )?;

    return Ok(());
}

fn Dispose_Block(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), StoreError>
{
    store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_block_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_blocks
         WHERE document_uid = ?1 AND ordinal = ?2",
        params![document_uid, ordinal, node_uid],
    )?;

    return Ok(());
}

fn Dispose_Heading(
    store: &mut SpecificationStore,
    document_uid: i64,
    ordinal: u32,
    node_uid: i64,
) -> Result<(), StoreError>
{
    store.Connection().execute(
        "INSERT OR IGNORE INTO lineage (source_heading_uid, disposition, target_node_uid)
         SELECT uid, 'preserved-verbatim', ?3 FROM source_headings
         WHERE document_uid = ?1 AND ordinal = ?2",
        params![document_uid, ordinal, node_uid],
    )?;

    return Ok(());
}
