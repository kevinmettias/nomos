use crate::store::{AUTHORED, SpecificationStore, StoreError};
use nomos_spec_model::Parse_Record;

/// The records that govern this system, embedded so they travel with the binary.
///
/// A seed that reads from disk would be a seed that silently does nothing when the
/// working directory is somewhere else.
///
/// The initializer is assembled by `build.rs` from `records/<ID>.record`, one file per
/// record. The declaration stays here: an `include!` at item position is invisible to the
/// universe scanner, and a table nobody counts is a table nobody checks. See `build.rs`
/// and `src/registration.rs` for the rest, and `OD-SPEC-007` for why the input directory
/// is not `docs/records`.
const RECORDS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/governing_records.rs"));

/// Every record identifier this build claims to govern itself by.
///
/// Compared against what a seeded store actually holds, so a record that stops being
/// embedded fails a test rather than quietly leaving the store.
///
/// Mirrored by `Test_Every_Canonical_Record_On_Disk_Should_Be_Governing`. That is the
/// comparison that matters, and it is deliberately not the one against the seeded store:
/// the store is seeded *from this list*, so comparing the two cannot fail. Six governing
/// records sat outside it for months for exactly that reason — OD-SPEC-005.
///
/// Assembled by `build.rs` from the registration directory, which is authored by hand and
/// is not `docs/records`. Both sides of that comparison therefore remain independently
/// written down, which is the objection `OD-LEDGER-007` raised against deriving this list
/// and the one `OD-SPEC-007` answers. A record is registered by adding
/// `records/<ID>.record`; no author edits this file to add one.
pub const GOVERNING_RECORD_IDS: &[&str] =
    include!(concat!(env!("OUT_DIR"), "/governing_record_ids.rs"));

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
        let written = store.Put_Record(path, AUTHORED, text)?;

        report.headings = report.headings.saturating_add(written.headings);
        report.blocks = report.blocks.saturating_add(written.blocks);
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

