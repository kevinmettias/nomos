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

    return Verdict_From_Violations(total, violations);
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
pub(crate) fn Verdict_From_Violations(total: u32, violations: Vec<Violation>) -> RuleOutcome
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

#[cfg(test)]
mod tests
{
    use super::*;
    use nomos_spec_ingest::Ingest_Source_Document;

    /// One H1 and one H2 heading, over two prose paragraphs — four blocks in total, which is
    /// what `Dispose_All` in this crate's own `tests/preservation_holds.rs` disposes of, and
    /// what `Test_An_Undisposed_Block_Should_Violate_Preserve_002` there counts on being
    /// undisposed by default.
    const DOCUMENT: &str = "---\nid: X\n---\n# Title\n\nOne.\n\n## Section\n\nTwo.\n";

    #[test]
    fn Test_Counted_Rows_Should_Count_The_Table_It_Is_Asked_About()
    {
        let store = Ingested();

        let blocks = Counted_Rows(&store, Table::SourceBlocks).expect("counts");

        assert_eq!(blocks, 4, "two headings and two prose paragraphs");
    }

    #[test]
    fn Test_Found_Violations_Should_Turn_Every_Row_The_Query_Names_Into_One_Violation()
    {
        let store = Ingested();

        let violations = Found_Violations(&store, "SELECT uid FROM source_blocks", |row| {
            let uid: i64 = row.get(0)?;
            return Ok(Violation {
                subject: uid.to_string(),
                detail: "named by this test's query".to_owned(),
            });
        })
        .expect("queries");

        assert_eq!(violations.len(), 4, "one violation per block row the query named");
    }

    #[test]
    fn Test_Verdict_From_Violations_Should_Report_Satisfied_Only_When_Nothing_Offends()
    {
        assert!(matches!(
            Verdict_From_Violations(3, Vec::new()),
            RuleOutcome::Satisfied { checked: 3 }
        ));

        let violated = Verdict_From_Violations(3, vec![Violation {
            subject: "s".to_owned(),
            detail: "d".to_owned(),
        }]);
        assert!(matches!(violated, RuleOutcome::Violated(violations) if violations.len() == 1));
    }

    #[test]
    fn Test_Offending_Outcome_Should_Report_Violated_When_The_Offenders_Query_Finds_Rows()
    {
        let store = Ingested();

        let outcome = Offending_Outcome(
            &store,
            Table::SourceBlocks,
            "SELECT uid FROM source_blocks",
            |row| {
                let uid: i64 = row.get(0)?;
                return Ok(Violation {
                    subject: uid.to_string(),
                    detail: "every row offends, for this test".to_owned(),
                });
            },
        );

        assert!(matches!(outcome, RuleOutcome::Violated(violations) if violations.len() == 4));
    }

    #[test]
    fn Test_Offending_Outcome_Should_Report_Satisfied_When_The_Offenders_Query_Finds_Nothing()
    {
        let store = Ingested();

        let outcome = Offending_Outcome(
            &store,
            Table::SourceBlocks,
            "SELECT uid FROM source_blocks WHERE 0",
            |_row| {
                return Ok(Violation {
                    subject: String::new(),
                    detail: String::new(),
                });
            },
        );

        assert!(matches!(outcome, RuleOutcome::Satisfied { checked: 4 }));
    }

    #[test]
    fn Test_Undisposed_Outcome_Should_Violate_A_Block_With_Neither_A_Disposition_Nor_An_Omission()
    {
        let store = Ingested();

        let outcome = Undisposed_Outcome(&store, &Traced {
            table: Table::SourceBlocks,
            offenders: Undisposed_Statement!(
                "source_blocks",
                "source_block_uid",
                "source_block_uid"
            ),
            label: "block",
        });

        assert!(matches!(outcome, RuleOutcome::Violated(violations) if violations.len() == 4));
    }

    fn Ingested() -> SpecificationStore
    {
        let mut store = SpecificationStore::In_Memory().expect("opens");
        Ingest_Source_Document(&mut store, "a.md", "v14.36", DOCUMENT).expect("ingests");
        return store;
    }
}
