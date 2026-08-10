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

/// The seeded vocabulary as a sentence, read from [`RELATION_TYPES`].
///
/// Built from the table rather than written out beside it. A restated list is a second
/// authority that goes stale silently, and this one appears in a refusal an author reads
/// while fixing exactly the thing the list describes — so a term added to the table above
/// shows up here with no second edit. `OD-SPEC-011`.
fn Admissible_Relations() -> String
{
    return RELATION_TYPES
        .iter()
        .map(|(name, _)| return *name)
        .collect::<Vec<&str>>()
        .join(", ");
}

/// The refusal for a relation type the seeded vocabulary does not contain, if it is one.
///
/// # Why this is checked here rather than left to the foreign key
///
/// `relations.relation_type` references `relation_types`, so an unknown term was already
/// refused — the mechanism working, and `RELATION_TYPES` above says so. What it was not is
/// diagnosable. The refusal arrived as `StoreError::Sql("FOREIGN KEY constraint failed")`,
/// which is the same eleven words for every cause the schema has, and names neither the
/// record that carried the term nor the term itself. Measured while landing `OD-LEDGER-014`:
/// one record declaring `amends` failed the whole seed, and because every test in this crate
/// seeds first, it read as eighteen unrelated failures across two suites with nothing in any
/// of them pointing at the record or the word.
///
/// So the check is raised where the record is known, which the database layer cannot be — by
/// the time `SQLite` refuses, the path has been out of scope for two call frames. It reports
/// all three things the author's next three questions are: which record stopped the seed,
/// which term it used, and what it could have said instead.
///
/// This deliberately does **not** admit unknown terms. `ADR-ARTIFACT-GRAPH-002`'s vocabulary
/// arrives with the corpus, and widening the seed table now would put an invented answer
/// where a recorded one belongs. The defect was the diagnosis, not the refusal.
fn Refuse_Unknown_Relation(path: &str, term: &str) -> Option<StoreError>
{
    if RELATION_TYPES.iter().any(|(name, _)| return *name == term)
    {
        return None;
    }

    return Some(StoreError::Record {
        path: path.to_owned(),
        cause: format!(
            "declares the relation type `{term}`, which the seeded vocabulary does not \
             contain. It admits {}. The vocabulary is deliberately short — the real one is \
             ADR-ARTIFACT-GRAPH-002's and arrives with the corpus — so the fix is to use an \
             existing term where that is honest, and not to widen the table here",
            Admissible_Relations()
        ),
    });
}

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
            // Before `Reference_Node`, so a record with a bad term does not leave a
            // placeholder node behind on its way out.
            if let Some(refusal) = Refuse_Unknown_Relation(path, &relation.relation)
            {
                return Err(refusal);
            }

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

#[cfg(test)]
mod tests
{
    use super::*;

    /// The three things an author needs, asserted separately so a partial message cannot
    /// pass by containing one of them.
    ///
    /// `OD-SPEC-011`. The term is asserted as well as the record because a message that
    /// merely names the record would pass a `contains` check for the wrong reason — every
    /// refusal in this module names a record.
    #[test]
    fn Test_An_Unknown_Relation_Should_Name_The_Record_The_Term_And_The_Vocabulary()
    {
        let refusal = Refuse_Unknown_Relation("docs/records/OD-LEDGER-014-x.md", "amends")
            .expect("`amends` is not in the seeded vocabulary");

        let StoreError::Record { path, cause } = refusal
        else
        {
            panic!("an unknown relation type is a defect in a record: {refusal:?}");
        };

        assert_eq!(path, "docs/records/OD-LEDGER-014-x.md", "which record stopped the seed");
        assert!(cause.contains("amends"), "which term it used: {cause}");

        // And what it could have said instead — every seeded term, so the author does not
        // have to open governing.rs to find out.
        for (admissible, _) in RELATION_TYPES
        {
            assert!(
                cause.contains(admissible),
                "the refusal must offer `{admissible}`: {cause}"
            );
        }
    }

    /// The negative control, confirmed red alone.
    ///
    /// Without it the assertion above is satisfied by a function that refuses everything,
    /// which would refuse the whole record set on the first relation and be strictly worse
    /// than the foreign key it replaced.
    #[test]
    fn Test_Every_Seeded_Term_Should_Be_Admitted()
    {
        for (admissible, _) in RELATION_TYPES
        {
            assert!(
                Refuse_Unknown_Relation("docs/records/anything.md", admissible).is_none(),
                "`{admissible}` is in the vocabulary and must not be refused"
            );
        }
    }

    /// The vocabulary in the message is the table, not a copy of it.
    ///
    /// `done_when` asks for this directly: a term added to `RELATION_TYPES` must appear in
    /// the refusal without a second edit. Asserted by counting rather than by matching a
    /// fixed string, so it stays true as the table changes.
    #[test]
    fn Test_The_Reported_Vocabulary_Should_Come_From_The_Table()
    {
        let reported = Admissible_Relations();

        assert_eq!(
            reported.split(", ").count(),
            RELATION_TYPES.len(),
            "the refusal reports {} term(s) for a table of {}: {reported}",
            reported.split(", ").count(),
            RELATION_TYPES.len()
        );
    }
}

