//! The shape every preservation rule has, answered once.
//!
//! Each rule counts a table, runs a query naming the rows that offend, and reports those
//! rows. The counting, the two failure paths and the satisfied-or-violated decision live
//! here rather than in each rule, because a rule that swallowed a SQL error would report
//! satisfied over a query that never ran.

use rusqlite::Row;
use crate::RuleOutcome;
use crate::Violation;
use nomos_spec_store::{SpecificationStore, Table};
/// Every rule here has the same shape: count a table, run a query naming the rows that
/// offend, and report those rows. The counting, the two failure paths and the
/// satisfied-or-violated decision are answered once, in [`Offending_Outcome`], because a
/// rule that swallowed a SQL error would report satisfied over a query that never ran.
pub(crate) fn Offending_Outcome(
    store: &SpecificationStore,
    counted: Table,
    offenders: &'static str,
    violation: impl Fn(&Row<'_>) -> rusqlite::Result<Violation>,
) -> RuleOutcome
{
    let (total, violations) = match Counted_And_Found(store, counted, offenders, violation)
    {
        Ok(pair) => pair,
        Err(outcome) => return outcome,
    };

    return Verdict(total, violations);
}

/// Both queries a rule needs, run unconditionally.
///
/// The count is run whether or not `Found_Violations` later reports any rows, on purpose
/// rather than by oversight: running it only when there are no violations would make a rule
/// with real violations and a broken count report `Violated` over a count query that never
/// ran — the same silent swallow this module's own doc comment exists to rule out.
fn Counted_And_Found(
    store: &SpecificationStore,
    counted: Table,
    offenders: &'static str,
    violation: impl Fn(&Row<'_>) -> rusqlite::Result<Violation>,
) -> Result<(u32, Vec<Violation>), RuleOutcome>
{
    let total = Counted_Rows(store, counted).map_err(RuleOutcome::Errored)?;
    let violations = Found_Violations(store, offenders, violation).map_err(RuleOutcome::Errored)?;

    return Ok((total, violations));
}

/// Satisfied when nothing offends, violated otherwise.
pub(crate) fn Verdict(total: u32, violations: Vec<Violation>) -> RuleOutcome
{
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
pub(crate) fn Counted_Rows(store: &SpecificationStore, table: Table) -> Result<u32, String>
{
    return store
        .Connection()
        .query_row(table.Tally_Sql(), [], |row| row.get(1))
        .map_err(|error| return error.to_string());
}

/// Every row the offending query returned, as the rule words it.
pub(crate) fn Found_Violations(
    store: &SpecificationStore,
    offenders: &'static str,
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

/// One traced table's whole offenders statement, joined at COMPILE time.
///
/// A table and a column are identifiers, and no driver binds an identifier — a placeholder
/// occupies a value position, so there is no parameterized form of this statement. `concat!`
/// joins string literals into a constant, which is what keeps the SQL from ever being a
/// value the program assembled: the three names are fixed where the rule is written, not
/// woven in when it runs.
// A function could not do this. `concat!` takes literals, so the three names have to be joined
// where the rule writes them; a `fn(&str, &str, &str) -> String` would produce the statement at
// run time, and `Traced::offenders` — a `&'static str` — could not hold the result.
macro_rules! Undisposed_Statement
{
    ($table:literal, $lineage:literal, $omission:literal) =>
    {
        concat!(
            "SELECT t.uid, coalesce(d.path, '?'), coalesce(t.ordinal, -1)
             FROM ",
            $table,
            " t
             LEFT JOIN source_documents d ON d.uid = t.document_uid
             WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.",
            $lineage,
            " = t.uid)
               AND NOT EXISTS (SELECT 1 FROM omissions o WHERE o.",
            $omission,
            " = t.uid)
             ORDER BY t.uid"
        )
    };
}

pub(crate) use Undisposed_Statement;

/// The table a preservation rule walks, and what would excuse a row of it.
pub(crate) struct Traced
{
    /// The table whose every row must be accounted for.
    pub(crate) table: Table,
    /// The statement naming the rows of it that nothing accounts for, built by
    /// [`Undisposed_Statement`] so that it is a constant rather than assembled text.
    pub(crate) offenders: &'static str,
    /// What one of its rows is called in a violation.
    pub(crate) label: &'static str,
}

/// The three columns every `Traced` offender query selects, in the order it selects them.
/// They are named here rather than at the reader because the queries live in `Traced` and
/// this closure is the only thing that has to agree with them.
const UID: usize = 0;
const DOCUMENT: usize = 1;
const ORDINAL: usize = 2;

/// Rows that have no disposition and no omission.
///
/// Both are checked, because either one accounts for a row: a disposition says what became
/// of it and an omission says why nothing did. A row with neither was dropped silently.
pub(crate) fn Undisposed_Outcome(store: &SpecificationStore, traced: &Traced) -> RuleOutcome
{
    let Traced {
        table,
        offenders,
        label,
    } = *traced;

    return Offending_Outcome(store, table, offenders, |row| {
        let uid: i64 = row.get(UID)?;
        let document: String = row.get(DOCUMENT)?;
        let ordinal: i64 = row.get(ORDINAL)?;
        return Ok(Violation {
            subject: format!("{document}#{ordinal}"),
            detail: format!("{label} {uid} has neither a lineage disposition nor an omission"),
        });
    });
}
