//! A body repeated across enough sections to be a template names its own pattern, or is one.

use crate::RuleOutcome;
use crate::Rule;
use crate::Violation;
use nomos_spec_ingest::{Get_Filler_Pattern, Is_Template_Eligible, SHARED_BY};
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
const TEMPLATES: &str = "SELECT normalized_hash,
                                count(*) AS sections,
                                count(DISTINCT document_uid) AS documents,
                                min(text) AS sample
                         FROM source_blocks
                         WHERE kind = 'prose'
                         GROUP BY normalized_hash
                         HAVING count(*) >= ?1
                         ORDER BY sections DESC, sample";

/// A block body carried by `SHARED_BY` or more sections is a template, not prose. A
/// template `Get_Filler_Pattern` already names is accounted for; one the corpus's own
/// declaration rows admit is accounted for; one neither names is a form letter nobody
/// declared, the exact shape `OD-SPEC-004` measured hollowing 44 restored members with one
/// undeclared adjective.
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
                 pattern, a declared repetition, or a violation";
    }

    fn Evaluate(&self, store: &SpecificationStore) -> RuleOutcome
    {
        let (total, violations) = match Counted_And_Undeclared(store)
        {
            Ok(pair) => pair,
            Err(outcome) => return outcome,
        };

        return crate::offending::Verdict_From_Violations(total, violations);
    }
}

/// Both queries this rule needs, run unconditionally.
///
/// The count is run whether or not `Undeclared_Templates` later reports any rows, on purpose
/// rather than by oversight: running it only when there are no violations would make a rule
/// with real violations and a broken count report `Violated` over a count query that never
/// ran.
fn Counted_And_Undeclared(store: &SpecificationStore) -> Result<(u32, Vec<Violation>), RuleOutcome>
{
    use crate::offending::Counted_Rows;

    let total = Counted_Rows(store, Table::SourceBlocks).map_err(RuleOutcome::Errored)?;
    let violations = Undeclared_Templates(store).map_err(RuleOutcome::Errored)?;

    return Ok((total, violations));
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
    for (normalized_hash, sections, documents, sample) in Template_Rows(store)?
    {
        // Eligibility first, because a body below the floor is not a template that
        // happens to be undeclared -- it is not a template at all, and reporting it as
        // one is what `OD-SPEC-004` version 3 measured 102 times over the sibling
        // suites. Every member of a group shares a normalized body, so deciding this on
        // the sample decides it for the group.
        if !Is_Template_Eligible(&sample)
        {
            continue;
        }

        if Get_Filler_Pattern(&sample).is_some()
        {
            continue;
        }

        // The declaration rows a corpus authors admit a repetition whose occurrences are
        // the ones the declaration names. A block declared for N places that appears N or
        // fewer times is admitted without being called filler; a further occurrence is the
        // same undeclared shape this rule already reports.
        let declared = Declared_Multiplicity(store, &normalized_hash)?;
        if declared >= i64::from(sections)
        {
            continue;
        }

        if declared == 0
        {
            violations.push(Violation {
                subject: format!("{sections} section(s) across {documents} document(s)"),
                detail: format!("repeats an undeclared template: {}", Preview_Text(&sample)),
            });
        }
        else
        {
            let excess = i64::from(sections).saturating_sub(declared);
            violations.push(Violation {
                subject: format!("{excess} section(s) beyond the {declared} the declaration names"),
                detail: format!("declared for {declared} occurrences but appears {sections}: {}", Preview_Text(&sample)),
            });
        }
    }

    return Ok(violations);
}

/// The total multiplicity the declaration rows name for one normalized body -- the number
/// of places the corpus says that body is intentionally projected into, summed across every
/// role it is declared under. Zero when the corpus declares nothing for it.
fn Declared_Multiplicity(store: &SpecificationStore, normalized_hash: &str) -> Result<i64, String>
{
    const DECLARED: &str =
        "SELECT coalesce(sum(multiplicity), 0) FROM repeated_text_declarations WHERE normalized_hash = ?1";

    let connection = store.Connection();
    let mut statement = connection.prepare(DECLARED).map_err(|error| return error.to_string())?;
    let declared: i64 = statement
        .query_row([normalized_hash], |row| return row.get(0))
        .map_err(|error| return error.to_string())?;

    return Ok(declared);
}

/// The column `Template_Rows`'s query selects the shared text sample from.
const SAMPLE_COLUMN: usize = 3;

/// Every group of prose blocks sharing a body across `SHARED_BY` or more sections: the
/// normalized hash that groups them, the section count, the document count, and one sample
/// of the shared text.
fn Template_Rows(store: &SpecificationStore) -> Result<Vec<(String, u32, u32, String)>, String>
{
    let connection = store.Connection();
    let mut statement = connection.prepare(TEMPLATES).map_err(|error| return error.to_string())?;
    let rows = statement
        .query_map([SHARED_BY], |row| {
            let normalized_hash: String = row.get(0)?;
            let sections: u32 = row.get(1)?;
            let documents: u32 = row.get(2)?;
            let sample: String = row.get(SAMPLE_COLUMN)?;
            return Ok((normalized_hash, sections, documents, sample));
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
