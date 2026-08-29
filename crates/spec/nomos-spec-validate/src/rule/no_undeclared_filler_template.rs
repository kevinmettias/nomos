//! A body repeated across enough sections to be a template names its own pattern, or is one.

use crate::RuleOutcome;
use crate::Rule;
use crate::Violation;
use nomos_spec_ingest::{Get_Filler_Pattern, SHARED_BY};
use nomos_spec_store::{SpecificationStore, Table};

/// Every prose block's body, grouped by exact text, widest group first.
///
/// `kind = 'prose'` is what "its own section title elided" means at the row this table
/// already has: a heading is its own block (`OD-SPEC-004`'s "section title"), stored and
/// counted separately, so excluding it here is the elision rather than a second mechanism
/// for it. Without the filter, a heading like "## Rationale" — legitimately repeated
/// across unrelated governing records — reads as an undeclared template; a fence repeated
/// verbatim is a different question `FILLER_PATTERNS` was never written to answer, so it
/// is excluded the same way.
///
/// `SHARED_BY` is bound at query time rather than folded into a `HAVING` literal, so the
/// threshold this rule enforces and the one `archaeology::Shared_Templates` reports over a
/// revision pair stay the same constant rather than two numbers somebody has to notice
/// disagree.
const TEMPLATES: &str = "SELECT count(*) AS sections,
                                count(DISTINCT document_uid) AS documents,
                                min(text) AS sample
                         FROM source_blocks
                         WHERE kind = 'prose'
                         GROUP BY normalized_hash
                         HAVING count(*) >= ?1
                         ORDER BY sections DESC, sample";

/// A block body carried by `SHARED_BY` or more sections is a template, not prose. A
/// template `Get_Filler_Pattern` already names is accounted for; one it does not name is a form
/// letter nobody declared, the exact shape `OD-SPEC-004` measured hollowing 44 restored
/// members with one undeclared adjective.
pub(crate) struct NoUndeclaredFillerTemplate;

impl Rule for NoUndeclaredFillerTemplate
{
    fn Id(&self) -> &'static str
    {
        return "NSV-PRESERVE-004";
    }

    fn Describe(&self) -> &'static str
    {
        return "a body repeated across SHARED_BY or more sections is a declared filler \
                 pattern or a violation";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        use crate::offending::Counted_Rows;

        // `total` is bound here, ahead of the violations check below, on purpose rather than
        // by oversight: it runs the count query unconditionally so a broken count is
        // `Errored` regardless of whether `Undeclared_Templates` later reports any rows.
        // Binding it inside the `if violations.is_empty()` block below would make the count
        // query run only when there are no violations, so a rule with real violations and a
        // broken count would report `Violated` over a count query that never ran.
        let total = match Counted_Rows(store, Table::SourceBlocks)
        {
            Ok(count) => count,
            Err(error) => return RuleOutcome::Errored(error),
        };

        let violations = match Undeclared_Templates(store)
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
}

/// The templates the query names, filtered to the ones `Get_Filler_Pattern` does not.
///
/// The filter runs here rather than in SQL because `FILLER_PATTERNS` is Rust data, not a
/// store table; `Offending_Outcome` (`crate::offending`) could not express a rule with a row
/// it wants to see and then dismiss, so this rule builds the same satisfied-or-violated
/// shape by hand instead of reusing it.
fn Undeclared_Templates(store: &SpecificationStore) -> Result<Vec<Violation>, String>
{
    let mut violations = Vec::new();
    for (sections, documents, sample) in Template_Rows(store)?
    {
        if Get_Filler_Pattern(&sample).is_some()
        {
            continue;
        }

        violations.push(Violation {
            subject: format!("{sections} section(s) across {documents} document(s)"),
            detail: format!("repeats an undeclared template: {}", Preview_Text(&sample)),
        });
    }

    return Ok(violations);
}

/// The column `Template_Rows`'s query selects the shared text sample from.
const SAMPLE_COLUMN: usize = 2;

/// Every group of prose blocks sharing a body across `SHARED_BY` or more sections: the
/// section count, the document count, and one sample of the shared text.
fn Template_Rows(store: &SpecificationStore) -> Result<Vec<(u32, u32, String)>, String>
{
    let connection = store.Connection();
    let mut statement = connection.prepare(TEMPLATES).map_err(|error| return error.to_string())?;
    let rows = statement
        .query_map([SHARED_BY], |row| {
            let sections: u32 = row.get(0)?;
            let documents: u32 = row.get(1)?;
            let sample: String = row.get(SAMPLE_COLUMN)?;
            return Ok((sections, documents, sample));
        })
        .map_err(|error| return error.to_string())?;

    return Ok(rows.filter_map(Result::ok).collect());
}

/// How many characters of a template's shared text `Preview_Text` prints before truncating.
const LIMIT: usize = 80;

/// The first 80 characters of a template's shared text, so a violation names what repeats
/// without printing the whole body.
fn Preview_Text(text: &str) -> String
{
    if text.chars().count() <= LIMIT
    {
        return text.to_owned();
    }

    let mut preview: String = text.chars().take(LIMIT).collect();
    preview.push('\u{2026}');
    return preview;
}
