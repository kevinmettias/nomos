//! Placing a portable bundle into a store, or refusing it whole.

mod disjoint;
mod graph;
mod import_report;
mod reference;
mod resolve;
mod source;
mod submission;

pub use import_report::ImportReport;

use graph::{
    Insert_Lineage, Insert_Node_Aliases, Insert_Node_History, Insert_Nodes, Insert_Normative_Statements,
    Insert_Omissions, Insert_Record_Front_Matter, Insert_Record_Relations, Insert_Relation_Types,
    Insert_Relations, Insert_Suites,
};
use source::{
    Insert_Blobs, Insert_Source_Blocks, Insert_Source_Documents, Insert_Source_Headings,
    Insert_Source_Table_Rows,
};
use submission::{Insert_Submission_Gaps, Insert_Submission_Values, Insert_Submissions};

use std::collections::BTreeMap;

use nomos_spec_store::{SpecificationStore, Table};
use rusqlite::Transaction;

use crate::BundleError;
use crate::Bundle;
use crate::Record;

/// Places a bundle's content in a store, beside whatever that store already holds.
///
/// It is not a merge and must never become one by accident. What guarantees that used to be
/// that the store was empty, which is a guarantee no store this build assembles can offer:
/// `Assemble_Corpus` seeds the governing records first and unconditionally, so demanding emptiness
/// put the durable text form `OD-SPEC-008` names beyond the reach of the only stores that
/// exist. The guarantee is now stated over content instead — the bundle and the store must
/// name nothing in common, and the bundle must resolve its own references — which is the
/// same promise on a store that has been seeded.
///
/// # Errors
///
/// Returns [`BundleError::Occupied`] if the store already holds something the bundle
/// carries, [`BundleError::Unresolved`] if a record names something the bundle does not
/// carry, and [`BundleError::Incomplete`] if the import does not place exactly what the
/// bundle declared.
pub fn Import_Bundle(store: &mut SpecificationStore, bundle: &Bundle) -> Result<ImportReport, BundleError>
{
    use disjoint::Assert_Disjoint;
    use resolve::Assert_Self_Contained;

    bundle.Verify_Counts()?;
    Assert_Same_Schema(store, bundle)?;

    // Self-containment first, because it reads the bundle alone and so gives the same
    // answer whatever the store holds. Disjointness then asks the one question that does
    // depend on the store.
    Assert_Self_Contained(bundle)?;
    Assert_Disjoint(store, bundle)?;

    let before = Table_Counts(store)?;

    return store.In_Transaction(|transaction| return Insert_All(transaction, bundle, &before));
}

/// The bundle and the store are at the same schema version.
///
/// Refused in both directions. A bundle from a newer store carries columns this build has
/// nowhere to put; one from an older store would need migrating, and importing it unmigrated
/// would be a guess about what the missing columns meant.
fn Assert_Same_Schema(store: &SpecificationStore, bundle: &Bundle) -> Result<(), BundleError>
{
    let supported = store.Version();
    let declared = bundle.Header().schema_version;

    if declared > supported
    {
        return Err(BundleError::TooNew {
            found: declared,
            supported,
        });
    }
    if declared < supported
    {
        return Err(BundleError::Malformed(format!(
            "the bundle is at schema version {declared} and this store is at {supported}; \
             bundle migration is not implemented, so importing it would guess"
        )));
    }

    return Ok(());
}

/// How many rows each table held before the import.
///
/// Taken rather than assumed zero. Once a store may already hold content, "the table is
/// empty afterwards" and "the import placed nothing" stopped being the same sentence, and
/// the completeness guard below is only a guard if it measures the difference.
fn Table_Counts(store: &SpecificationStore) -> Result<BTreeMap<&'static str, u32>, BundleError>
{
    let mut census: BTreeMap<&'static str, u32> = BTreeMap::new();
    for table in Table::All()
    {
        census.insert(table.Name(), store.Count(*table)?);
    }

    return Ok(census);
}

/// Everything the bundle carries, in one transaction, and the guard that all of it landed.
fn Insert_All(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    before: &BTreeMap<&'static str, u32>,
) -> Result<ImportReport, BundleError>
{
    // The relation-type vocabulary is self-referential (`inverse_of` names another row in the
    // same table), so no insertion order satisfies it. Deferring moves every foreign-key check
    // to commit without weakening any of them.
    transaction.pragma_update(None, "defer_foreign_keys", "ON")?;
    Insert_Sources(transaction, bundle)?;
    Insert_Graph(transaction, bundle)?;
    Insert_Declarations(transaction, bundle)?;
    Insert_Submission_Rows(transaction, bundle)?;
    Assert_Landed(transaction, bundle, before)?;

    return Ok(ImportReport {
        records: bundle.Manifest().records,
        counts: bundle.Manifest().counts.clone(),
    });
}

/// The bytes, and the documents cut from them.
fn Insert_Sources(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Blobs(transaction, bundle)?;
    Insert_Source_Documents(transaction, bundle)?;
    Insert_Source_Headings(transaction, bundle)?;
    Insert_Source_Blocks(transaction, bundle)?;

    return Insert_Source_Table_Rows(transaction, bundle);
}

/// The identities those documents were read into, and the edges between them.
fn Insert_Graph(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Suites(transaction, bundle)?;
    Insert_Nodes(transaction, bundle)?;
    Insert_Node_Aliases(transaction, bundle)?;
    Insert_Node_History(transaction, bundle)?;
    Insert_Relation_Types(transaction, bundle)?;
    Insert_Relations(transaction, bundle)?;
    Insert_Normative_Statements(transaction, bundle)?;
    Insert_Lineage(transaction, bundle)?;

    return Insert_Omissions(transaction, bundle);
}

/// What a record declared about itself in its own front matter.
fn Insert_Declarations(transaction: &Transaction<'_>, bundle: &Bundle) -> Result<(), BundleError>
{
    Insert_Record_Front_Matter(transaction, bundle)?;

    return Insert_Record_Relations(transaction, bundle);
}

/// The submissions, the values attributed to them, and the gaps left open.
fn Insert_Submission_Rows(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
) -> Result<(), BundleError>
{
    Insert_Submissions(transaction, bundle)?;
    Insert_Submission_Values(transaction, bundle)?;

    return Insert_Submission_Gaps(transaction, bundle);
}

/// The import placed exactly what the bundle said it would.
///
/// The mirror of the exporter's completeness guard: an insert that collapsed rows, or a
/// record kind nothing inserts, fails the import instead of producing a store that is
/// quietly smaller than its own bundle. Measured as a difference rather than a total,
/// because the rows that were already there are not this bundle's to account for.
fn Assert_Landed(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    before: &BTreeMap<&'static str, u32>,
) -> Result<(), BundleError>
{
    let after = Landed_Counts(transaction)?;

    for table in Table::All()
    {
        let landed = Added_Rows(&after, before, table.Name());
        let declared = bundle.Manifest().counts.get(table.Name()).copied().unwrap_or(0);

        if landed != declared
        {
            return Err(BundleError::Incomplete {
                table: table.Name().to_owned(),
                in_store: landed,
                exported: declared,
            });
        }
    }

    return Ok(());
}

/// Every table's row count, in one crossing rather than one per table.
///
/// The counts are compared against the manifest afterwards, in memory. Asking each table
/// separately made the completeness guard cost one round trip per table for an answer that
/// one statement returns whole.
fn Landed_Counts(transaction: &Transaction<'_>) -> Result<BTreeMap<&'static str, u32>, BundleError>
{
    let tally = Tally_Statement();
    let mut statement = transaction.prepare(&tally)?;
    let counted = statement.query_map([], |row| {
        return Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?));
    })?;

    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for row in counted
    {
        let (which, tally) = row?;
        if let Some(table) = Table::All().iter().find(|table| return table.Name() == which)
        {
            counts.insert(table.Name(), tally);
        }
    }

    return Ok(counts);
}

/// Every table's tally in one statement.
///
/// Every arm is a `&'static str` the table itself carries, joined rather than assembled: no
/// value is woven into the statement text at any point, so there is nothing here for a caller
/// to reach.
fn Tally_Statement() -> String
{
    return Table::All()
        .iter()
        .map(|table| return table.Tally_Sql())
        .collect::<Vec<&'static str>>()
        .join(" UNION ALL ");
}

/// How many rows this import added to one table.
fn Added_Rows(
    after: &BTreeMap<&'static str, u32>,
    before: &BTreeMap<&'static str, u32>,
    table: &'static str,
) -> u32
{
    let now = after.get(table).copied().unwrap_or(0);

    return now.saturating_sub(before.get(table).copied().unwrap_or(0));
}

/// Prepares one insert and offers every record the bundle carries to it.
///
/// Each importer below was the same frame — prepare, walk the records, skip the ones this
/// table is not about, resolve the parent row, execute — around the two lines that say
/// which variant and which columns. The frame is what they share and the reader is what
/// they do not, so the frame lives here and the reader stays at the call.
///
/// The reader returns `Ok(())` for a record of another kind, which is the skip.
pub(super) fn Insert_Each<Bind>(
    transaction: &Transaction<'_>,
    bundle: &Bundle,
    sql: &str,
    mut bind: Bind,
) -> Result<(), BundleError>
where
    Bind: FnMut(&mut rusqlite::Statement<'_>, &Record) -> Result<(), BundleError>,
{
    let mut insert = transaction.prepare(sql)?;

    for record in bundle.Records()
    {
        bind(&mut insert, record)?;
    }

    return Ok(());
}
