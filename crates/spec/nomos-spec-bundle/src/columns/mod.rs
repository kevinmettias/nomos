//! Column-level completeness: every column of the store reaches the bundle.
//!
//! The row-count guard cannot see this failure. Add a column to a table, teach nothing to
//! export it, and the counts still agree on both sides — the bundle carries exactly as
//! many records as the store has rows, each one quietly missing a field. Round-tripping it
//! is a fixpoint too, because the second export drops the same column the first did.
//!
//! So each column declares how it is carried, and the declaration is checked three ways:
//! the schema must hold no column the declaration omits, the declaration must name no
//! column the schema lacks, and a named record field must actually exist on the record.
//! The third is what stops the declaration from being satisfied by typing the column's
//! name into a list.

mod coverage;
#[cfg(test)]
mod tests;

use coverage::{COVERAGE, Carried};

use crate::BundleError;
use crate::row::record::Record;
use nomos_spec_store::Table;
use rusqlite::Connection;
use std::collections::BTreeSet;

/// # Errors
///
/// Returns [`BundleError::UncoveredColumn`] for a column the exporter does not carry,
/// [`BundleError::PhantomColumn`] for a declaration the schema does not have, and
/// [`BundleError::Uncarried`] for a declared field no record of that table holds.
pub(crate) fn Assert_Columns_Covered(
    connection: &Connection,
    records: &[Record],
) -> Result<(), BundleError>
{
    for table in Table::All()
    {
        let name = table.Name();
        let declared = COVERAGE
            .iter()
            .find(|coverage| coverage.table == name)
            .map_or(&[] as &[(&str, Carried)], |coverage| coverage.columns);

        let present = Schema_Columns(connection, name)?;
        Compare(name, &present, declared)?;
        Assert_Fields_Exist(name, declared, records)?;
    }

    return Ok(());
}

/// Both directions. A missing declaration is a column nobody exports; a stale one is a
/// declaration that stopped describing anything and would go on satisfying the guard.
fn Compare(
    table: &str,
    schema: &[String],
    declared: &[(&str, Carried)],
) -> Result<(), BundleError>
{
    let named: BTreeSet<&str> = declared.iter().map(|(column, _)| return *column).collect();
    let present: BTreeSet<&str> = schema.iter().map(String::as_str).collect();

    Assert_Every_Column_Is_Declared(table, schema, &named)?;

    return Assert_Every_Declaration_Is_A_Column(table, &named, &present);
}

/// A column nobody exports.
fn Assert_Every_Column_Is_Declared(
    table: &str,
    schema: &[String],
    named: &BTreeSet<&str>,
) -> Result<(), BundleError>
{
    let uncovered = schema.iter().find(|column| return !named.contains(column.as_str()));
    let Some(column) = uncovered
    else
    {
        return Ok(());
    };

    return Err(BundleError::UncoveredColumn {
        table: table.to_owned(),
        column: column.clone(),
    });
}

/// A declaration that stopped describing anything and would go on satisfying the guard.
fn Assert_Every_Declaration_Is_A_Column(
    table: &str,
    named: &BTreeSet<&str>,
    present: &BTreeSet<&str>,
) -> Result<(), BundleError>
{
    let phantom = named.iter().find(|column| return !present.contains(*column));
    let Some(column) = phantom
    else
    {
        return Ok(());
    };

    return Err(BundleError::PhantomColumn {
        table: table.to_owned(),
        column: (*column).to_owned(),
    });
}

/// The declaration names a field; the record has to have it.
///
/// Checked against the records being exported rather than a constructed sample, so it is
/// the real serialization that answers. A table with no rows cannot answer at all and is
/// skipped — the round-trip fixture holds at least one row of every table precisely so
/// that this is exercised rather than skipped everywhere.
fn Assert_Fields_Exist(
    table: &str,
    declared: &[(&str, Carried)],
    records: &[Record],
) -> Result<(), BundleError>
{
    let Some(sample) = records.iter().find(|record| return record.Table() == table)
    else
    {
        return Ok(());
    };

    let fields = Fields(sample)?;
    for (column, carried) in declared
    {
        Assert_The_Field_Is_Carried(table, column, carried, &fields)?;
    }

    return Ok(());
}

/// One declared column, and the field the record has to have for it.
fn Assert_The_Field_Is_Carried(
    table: &str,
    column: &str,
    carried: &Carried,
    fields: &BTreeSet<String>,
) -> Result<(), BundleError>
{
    let Carried::Field(field) = carried
    else
    {
        return Ok(());
    };
    if fields.contains(*field)
    {
        return Ok(());
    }

    return Err(BundleError::Uncarried {
        table: table.to_owned(),
        column: column.to_owned(),
        field: (*field).to_owned(),
    });
}

fn Fields(record: &Record) -> Result<BTreeSet<String>, BundleError>
{
    let value = serde_json::to_value(record)?;
    let Some(serde_json::Value::Object(payload)) = value.get("record")
    else
    {
        return Err(BundleError::Malformed(format!(
            "a {} record does not serialize as a payload object",
            record.Table()
        )));
    };

    return Ok(payload.keys().cloned().collect());
}

fn Schema_Columns(connection: &Connection, table: &str) -> Result<Vec<String>, BundleError>
{
    // `table` comes from `Table::All()`, which is an enum, so this is not a hole through
    // which arbitrary SQL reaches the database.
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = statement
        .query_map([], |row| return row.get::<_, String>(1))?
        .collect::<Result<Vec<String>, _>>()?;

    if names.is_empty()
    {
        return Err(BundleError::Malformed(format!(
            "the store has no table named {table}, so its columns cannot be checked"
        )));
    }

    return Ok(names);
}
