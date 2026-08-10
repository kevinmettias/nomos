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
        "docs/records/D-132-the-plan-is-a-game-plan.md",
        include_str!("../../../../docs/records/D-132-the-plan-is-a-game-plan.md"),
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
        "docs/records/OD-LEDGER-001-territory-is-declared-not-enforced.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-001-territory-is-declared-not-enforced.md"
        ),
    ),
    (
        "docs/records/OD-SPEC-004-the-filler-blocklist-misses-the-wording-that-hollowed-v15.md",
        include_str!(
            "../../../../docs/records/OD-SPEC-004-the-filler-blocklist-misses-the-wording-that-hollowed-v15.md"
        ),
    ),
    // The records the product phases produced. Every one of them was written as a file and
    // none reached the store until P9-PHASE-GAP went looking — including two that decide
    // how the store itself behaves.
    (
        "docs/records/OD-LEDGER-002-a-ledger-id-is-not-a-plan-phase.md",
        include_str!("../../../../docs/records/OD-LEDGER-002-a-ledger-id-is-not-a-plan-phase.md"),
    ),
    (
        "docs/records/OD-ANALYSIS-001-the-snapshot-in-a-fact-key-defeats-incremental-reuse.md",
        include_str!(
            "../../../../docs/records/OD-ANALYSIS-001-the-snapshot-in-a-fact-key-defeats-incremental-reuse.md"
        ),
    ),
    (
        "docs/records/OD-STORE-001-a-document-kind-is-a-behaviour-not-a-label.md",
        include_str!(
            "../../../../docs/records/OD-STORE-001-a-document-kind-is-a-behaviour-not-a-label.md"
        ),
    ),
    (
        "docs/records/OD-CAPABILITY-001-which-of-several-usable-offers-wins-is-unspecified.md",
        include_str!(
            "../../../../docs/records/OD-CAPABILITY-001-which-of-several-usable-offers-wins-is-unspecified.md"
        ),
    ),
    (
        "docs/records/OD-CAPABILITY-002-a-capability-contract-is-not-a-providers-property.md",
        include_str!(
            "../../../../docs/records/OD-CAPABILITY-002-a-capability-contract-is-not-a-providers-property.md"
        ),
    ),
    (
        "docs/records/OD-GATE-001-a-skipped-test-reports-ok.md",
        include_str!("../../../../docs/records/OD-GATE-001-a-skipped-test-reports-ok.md"),
    ),
    (
        "docs/records/OD-SPEC-005-six-governing-records-were-never-in-the-store.md",
        include_str!(
            "../../../../docs/records/OD-SPEC-005-six-governing-records-were-never-in-the-store.md"
        ),
    ),
    (
        "docs/records/D-133-the-read-surface-assembles-its-store-and-names-what-is-missing.md",
        include_str!(
            "../../../../docs/records/D-133-the-read-surface-assembles-its-store-and-names-what-is-missing.md"
        ),
    ),
    (
        "docs/records/OD-DETERMINISM-001-a-declaration-proven-only-behind-a-corpus-gate.md",
        include_str!(
            "../../../../docs/records/OD-DETERMINISM-001-a-declaration-proven-only-behind-a-corpus-gate.md"
        ),
    ),
    (
        "docs/records/OD-LEDGER-003-finishing-runs-the-gate-lint-step-and-derives-it.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-003-finishing-runs-the-gate-lint-step-and-derives-it.md"
        ),
    ),
    (
        "docs/records/OD-COMPLETENESS-001-a-completeness-guard-is-only-as-complete-as-its-universe.md",
        include_str!(
            "../../../../docs/records/OD-COMPLETENESS-001-a-completeness-guard-is-only-as-complete-as-its-universe.md"
        ),
    ),
    (
        "docs/records/D-134-a-rule-is-a-pure-function-and-a-universe-declares-its-own-mirror.md",
        include_str!(
            "../../../../docs/records/D-134-a-rule-is-a-pure-function-and-a-universe-declares-its-own-mirror.md"
        ),
    ),
    (
        "docs/records/OD-LEDGER-004-the-record-directory-is-the-lock.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-004-the-record-directory-is-the-lock.md"
        ),
    ),
    (
        "docs/records/OD-LEDGER-005-ready-meant-unheld-and-was-read-as-claimable.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-005-ready-meant-unheld-and-was-read-as-claimable.md"
        ),
    ),
    (
        "docs/records/OD-PLATFORM-001-a-port-says-nothing-about-how-its-outcomes-are-obtained.md",
        include_str!(
            "../../../../docs/records/OD-PLATFORM-001-a-port-says-nothing-about-how-its-outcomes-are-obtained.md"
        ),
    ),
    (
        "docs/records/OD-PROJECT-001-the-repository-readme-is-not-the-suites-overview.md",
        include_str!(
            "../../../../docs/records/OD-PROJECT-001-the-repository-readme-is-not-the-suites-overview.md"
        ),
    ),
    (
        "docs/records/OD-GATE-002-the-surface-check-is-derived-here-rather-than-by-a-tool-nobody-has.md",
        include_str!(
            "../../../../docs/records/OD-GATE-002-the-surface-check-is-derived-here-rather-than-by-a-tool-nobody-has.md"
        ),
    ),
    (
        "docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md",
        include_str!(
            "../../../../docs/records/OD-LEDGER-006-a-reason-attached-to-a-transition-does-not-survive-it.md"
        ),
    ),
];

/// Every record identifier this build claims to govern itself by.
///
/// Compared against what a seeded store actually holds, so a record that stops being
/// embedded fails a test rather than quietly leaving the store.
///
/// Mirrored by `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`. That is the
/// comparison that matters, and it is deliberately not the one against the seeded store:
/// the store is seeded *from this list*, so comparing the two cannot fail. Six governing
/// records sat outside it for months for exactly that reason — OD-SPEC-005.
pub const GOVERNING_RECORD_IDS: &[&str] = &[
    "ARC-SPECDB-001",
    "D-129",
    "D-130",
    "D-131",
    "D-132",
    "OD-SPEC-001",
    "OD-SPEC-002",
    "OD-SPEC-004",
    "OD-LEDGER-001",
    "OD-LEDGER-002",
    "OD-ANALYSIS-001",
    "OD-STORE-001",
    "OD-CAPABILITY-001",
    "OD-CAPABILITY-002",
    "OD-GATE-001",
    "OD-SPEC-005",
    "D-133",
    "OD-DETERMINISM-001",
    "OD-LEDGER-003",
    "OD-COMPLETENESS-001",
    "D-134",
    "OD-LEDGER-004",
    "OD-LEDGER-005",
    "OD-PLATFORM-001",
    "OD-PROJECT-001",
    "OD-GATE-002",
    "OD-LEDGER-006",
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
    // Its own inverse, because it is symmetric: two records that bear on each other bear on
    // each other. Added when six product-phase records were first seeded and three of them
    // used a term this vocabulary did not contain — the foreign key refused them, which is
    // the mechanism working, and it had never run because the records were files.
    //
    // The alternative was rewriting those relations as `affects`, and that would have been
    // false. OD-CAPABILITY-002 borrows OD-STORE-001's criterion; it does not affect it, and a
    // wrong edge in the graph this system exists to keep honest is worse than a vocabulary
    // one term short. Still a seed term: ADR-ARTIFACT-GRAPH-002's vocabulary arrives with the
    // corpus and supersedes this whole table.
    ("relates-to", "relates-to"),
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
