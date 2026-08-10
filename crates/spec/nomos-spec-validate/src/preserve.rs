use crate::run::{Rule, RuleOutcome, Violation};
use nomos_spec_store::SpecificationStore;
use rusqlite::Row;

/// Every rule here has the same shape: count a table, run a query naming the rows that
/// offend, and report those rows. The counting, the two failure paths and the
/// satisfied-or-violated decision are answered once, in [`Offending`], because a rule that
/// swallowed a SQL error would report satisfied over a query that never ran.
fn Offending(
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
fn Counted(store: &SpecificationStore, table: &str) -> Result<u32, String>
{
    return store
        .Connection()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0))
        .map_err(|error| return error.to_string());
}

/// Every row the offending query returned, as the rule words it.
fn Found(
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
struct Traced<'a>
{
    /// The table whose every row must be accounted for.
    table: &'a str,
    /// The lineage column that would carry a disposition for one of its rows.
    lineage: &'a str,
    /// The omissions column that would excuse one.
    omission: &'a str,
    /// What one of its rows is called in a violation.
    label: &'a str,
}

/// Rows that have no disposition and no omission.
///
/// Both are checked, because either one accounts for a row: a disposition says what became
/// of it and an omission says why nothing did. A row with neither was dropped silently.
fn Undisposed(store: &SpecificationStore, traced: &Traced<'_>) -> RuleOutcome
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

pub struct EveryHeadingHasADisposition;

impl Rule for EveryHeadingHasADisposition
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-001";
    }

    fn Describe(&self) -> &'static str
    {
        return "every source heading is preserved, superseded or explicitly omitted";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Undisposed(
            store,
            &Traced {
                table: "source_headings",
                lineage: "source_heading_uid",
                omission: "source_heading_uid",
                label: "heading",
            },
        );
    }
}

/// The rule that would have caught v15.0's 282 dropped table rows — at the granularity
/// v14 recorded, which is the block. A row lost from inside a preserved table block does
/// not violate this rule, because v14's segmenter never saw rows as blocks.
pub struct EveryBlockHasADisposition;

impl Rule for EveryBlockHasADisposition
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-002";
    }

    fn Describe(&self) -> &'static str
    {
        return "no source block is silently dropped";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Undisposed(
            store,
            &Traced {
                table: "source_blocks",
                lineage: "source_block_uid",
                omission: "source_block_uid",
                label: "block",
            },
        );
    }
}

/// Statements that claim to supersede a hash with no history event saying they changed.
const UNJUSTIFIED: &str = "SELECT s.statement_id
     FROM normative_statements s
     WHERE s.supersedes_hash IS NOT NULL
       AND NOT EXISTS (
           SELECT 1 FROM node_history h
           WHERE h.node_uid = s.node_uid AND h.event = 'content_changed'
       )
     ORDER BY s.statement_id";

/// Statements with no preserved lineage row reaching any source block.
///
/// The disposition has to be one of the two preserving ones. A statement whose only lineage
/// row says it was rewritten does not trace to the text it came from.
const UNTRACED: &str = "SELECT s.statement_id
     FROM normative_statements s
     WHERE NOT EXISTS (
         SELECT 1 FROM lineage l
         WHERE l.target_statement = s.uid
           AND l.source_block_uid IS NOT NULL
           AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
     )
     ORDER BY s.statement_id";

/// A changed `canonical_hash` needs a recorded reason and the hash it supersedes.
pub struct ChangedWordingIsJustified;

impl Rule for ChangedWordingIsJustified
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-003";
    }

    fn Describe(&self) -> &'static str
    {
        return "a changed canonical hash carries a supersession and a reason";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Offending(store, "normative_statements", UNJUSTIFIED, |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "supersedes a previous hash with no content_changed history event"
                    .to_owned(),
            });
        });
    }
}

/// Every statement traces back to a source block that was preserved.
///
/// v15.0 shipped exactly this violation for all 363 requirements: the statements existed
/// and nothing connected them to the text they came from.
pub struct EveryStatementTracesToSource;

impl Rule for EveryStatementTracesToSource
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-006";
    }

    fn Describe(&self) -> &'static str
    {
        return "every normative statement traces to a preserved source block";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        return Offending(store, "normative_statements", UNTRACED, |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "no preserved lineage row to any source block".to_owned(),
            });
        });
    }
}

#[must_use]
pub fn Registered() -> Vec<Box<dyn Rule>>
{
    return vec![
        Box::new(EveryHeadingHasADisposition),
        Box::new(EveryBlockHasADisposition),
        Box::new(ChangedWordingIsJustified),
        Box::new(EveryStatementTracesToSource),
    ];
}
