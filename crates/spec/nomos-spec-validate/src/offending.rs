//! The shape every preservation rule has, answered once.
//!
//! Each rule counts a table, runs a query naming the rows that offend, and reports those
//! rows. The counting, the two failure paths and the satisfied-or-violated decision live
//! here rather than in each rule, because a rule that swallowed a SQL error would report
//! satisfied over a query that never ran.

use rusqlite::Row;
use crate::rule_outcome::RuleOutcome;
use crate::violation::Violation;
use nomos_spec_store::SpecificationStore;
/// Every rule here has the same shape: count a table, run a query naming the rows that
/// offend, and report those rows. The counting, the two failure paths and the
/// satisfied-or-violated decision are answered once, in [`Offending`], because a rule that
/// swallowed a SQL error would report satisfied over a query that never ran.
pub(crate) fn Offending(
    store: &SpecificationStore,
    counted: &str,
    offenders: &str,
    violation: impl Fn(&Row<'_>) -> rusqlite::Result<Violation>,
) -> RuleOutcome
{
    let total = match Counted(store, counted)
    {
        Ok(count) => count,
        Err(error) => return RuleOutcome::Errored(error),
    };
    let violations = match Found(store, offenders, violation)
    {
        Ok(found) => found,
        Err(error) => return RuleOutcome::Errored(error),
    };

    if violations.is_empty()
    {
        return RuleOutcome::Satisfied { checked: total };
    }

    return RuleOutcome::Violated(violations);
}

/// How many rows the rule examined.
///
/// Reported alongside a satisfied verdict, because a rule that examined nothing and a rule
/// that examined four hundred rows both pass and only one of them means anything.
pub(crate) fn Counted(store: &SpecificationStore, table: &str) -> Result<u32, String>
{
    return store
        .Connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0))
        .map_err(|error| return error.to_string());
}

/// Every row the offending query returned, as the rule words it.
pub(crate) fn Found(
    store: &SpecificationStore,
    offenders: &str,
    violation: impl Fn(&Row<'_>) -> rusqlite::Result<Violation>,
) -> Result<Vec<Violation>, String>
{
    let connection = store.Connection();
    let mut statement = connection.prepare(offenders).map_err(|error| return error.to_string())?;
    let rows = statement
        .query_map([], |row| return violation(row))
        .map_err(|error| return error.to_string())?;

    return Ok(rows.filter_map(Result::ok).collect());
}

/// The table a preservation rule walks, and what would excuse a row of it.
pub(crate) struct Traced<'a>
{
    /// The table whose every row must be accounted for.
    pub(crate) table: &'a str,
    /// The lineage column that would carry a disposition for one of its rows.
    pub(crate) lineage: &'a str,
    /// The omissions column that would excuse one.
    pub(crate) omission: &'a str,
    /// What one of its rows is called in a violation.
    pub(crate) label: &'a str,
}

/// Rows that have no disposition and no omission.
///
/// Both are checked, because either one accounts for a row: a disposition says what became
/// of it and an omission says why nothing did. A row with neither was dropped silently.
pub(crate) fn Undisposed(store: &SpecificationStore, traced: &Traced<'_>) -> RuleOutcome
{
    let Traced {
        table,
        lineage,
        omission,
        label,
    } = *traced;
    let offenders = format!(
        "SELECT t.uid, coalesce(d.path, '?'), coalesce(t.ordinal, -1)
         FROM {table} t
         LEFT JOIN source_documents d ON d.uid = t.document_uid
         WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.{lineage} = t.uid)
           AND NOT EXISTS (SELECT 1 FROM omissions o WHERE o.{omission} = t.uid)
         ORDER BY t.uid"
    );

    return Offending(store, table, &offenders, |row| {
        let uid: i64 = row.get(0)?;
        let document: String = row.get(1)?;
        let ordinal: i64 = row.get(2)?;
        return Ok(Violation {
            subject: format!("{document}#{ordinal}"),
            detail: format!("{label} {uid} has neither a lineage disposition nor an omission"),
        });
    });
}
