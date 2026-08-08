use crate::run::{Rule, RuleOutcome, Violation};
use nomos_spec_store::SpecificationStore;

/// Rows that have no disposition and no omission, plus how many were examined.
fn Undisposed(
    store: &SpecificationStore,
    table: &str,
    lineage_column: &str,
    omission_column: &str,
    label: &str,
) -> RuleOutcome
{
    let total: u32 = match store.Connection().query_row(
        &format!("SELECT count(*) FROM {table}"),
        [],
        |row| row.get(0),
    )
    {
        Ok(count) => count,
        Err(error) => return RuleOutcome::Errored(error.to_string()),
    };

    let sql = format!(
        "SELECT t.uid, coalesce(d.path, '?'), coalesce(t.ordinal, -1)
         FROM {table} t
         LEFT JOIN source_documents d ON d.uid = t.document_uid
         WHERE NOT EXISTS (SELECT 1 FROM lineage l WHERE l.{lineage_column} = t.uid)
           AND NOT EXISTS (SELECT 1 FROM omissions o WHERE o.{omission_column} = t.uid)
         ORDER BY t.uid"
    );

    let mut statement = match store.Connection().prepare(&sql)
    {
        Ok(statement) => statement,
        Err(error) => return RuleOutcome::Errored(error.to_string()),
    };

    let rows = statement.query_map([], |row| {
        let uid: i64 = row.get(0)?;
        let document: String = row.get(1)?;
        let ordinal: i64 = row.get(2)?;
        return Ok(Violation {
            subject: format!("{document}#{ordinal}"),
            detail: format!("{label} {uid} has neither a lineage disposition nor an omission"),
        });
    });

    let violations: Vec<Violation> = match rows
    {
        Ok(rows) => rows.filter_map(Result::ok).collect(),
        Err(error) => return RuleOutcome::Errored(error.to_string()),
    };

    return if violations.is_empty()
    {
        RuleOutcome::Satisfied { checked: total }
    }
    else
    {
        RuleOutcome::Violated(violations)
    };
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
            "source_headings",
            "source_heading_uid",
            "source_heading_uid",
            "heading",
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
            "source_blocks",
            "source_block_uid",
            "source_block_uid",
            "block",
        );
    }
}

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
        let total: u32 = match store.Connection().query_row(
            "SELECT count(*) FROM normative_statements",
            [],
            |row| row.get(0),
        )
        {
            Ok(count) => count,
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        let mut statement = match store.Connection().prepare(
            "SELECT s.statement_id
             FROM normative_statements s
             WHERE s.supersedes_hash IS NOT NULL
               AND NOT EXISTS (
                   SELECT 1 FROM node_history h
                   WHERE h.node_uid = s.node_uid AND h.event = 'content_changed'
               )
             ORDER BY s.statement_id",
        )
        {
            Ok(statement) => statement,
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        let rows = statement.query_map([], |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "supersedes a previous hash with no content_changed history event"
                    .to_owned(),
            });
        });

        let violations: Vec<Violation> = match rows
        {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        return if violations.is_empty()
        {
            RuleOutcome::Satisfied { checked: total }
        }
        else
        {
            RuleOutcome::Violated(violations)
        };
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
        let total: u32 = match store.Connection().query_row(
            "SELECT count(*) FROM normative_statements",
            [],
            |row| row.get(0),
        )
        {
            Ok(count) => count,
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        let mut statement = match store.Connection().prepare(
            "SELECT s.statement_id
             FROM normative_statements s
             WHERE NOT EXISTS (
                 SELECT 1 FROM lineage l
                 WHERE l.target_statement = s.uid
                   AND l.source_block_uid IS NOT NULL
                   AND l.disposition IN ('preserved-verbatim', 'preserved-normalized')
             )
             ORDER BY s.statement_id",
        )
        {
            Ok(statement) => statement,
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        let rows = statement.query_map([], |row| {
            let id: String = row.get(0)?;
            return Ok(Violation {
                subject: id,
                detail: "no preserved lineage row to any source block".to_owned(),
            });
        });

        let violations: Vec<Violation> = match rows
        {
            Ok(rows) => rows.filter_map(Result::ok).collect(),
            Err(error) => return RuleOutcome::Errored(error.to_string()),
        };

        return if violations.is_empty()
        {
            RuleOutcome::Satisfied { checked: total }
        }
        else
        {
            RuleOutcome::Violated(violations)
        };
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
