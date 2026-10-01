//! Rule 4: `implements` resolves to an accepted design, and `answers` to an accepted request.

use crate::{SpecificationStore, StoreError};
use nomos_spec_model::{Failure, Submission, SubmissionState};
use rusqlite::OptionalExtension;

/// Every rule this submission fails about what it cites: the ones about its shape live in
/// [`crate::submission::Failed_Rules`], which calls here for the rest, so a refusal names all
/// of them at once.
///
/// A result built against a draft is a result whose target may still change under it, so this
/// applies in both states — `draft` weakens which rules apply only for rule 5.
///
/// Stated as a rule on the submission rather than as a constraint on the edge, which is what
/// `OD-SPEC-010` says it is and what it can be: `relation_types` carries no domain, range or
/// cardinality, so nothing here stops an `implements` edge joining a suite to a table row.
/// That is `P10-EDGE-CONSTRAINTS`, and it is a different guarantee from this one.
pub(super) fn Unresolved_Citations(
    store: &SpecificationStore,
    submission: &Submission,
) -> Result<Vec<Failure>, StoreError>
{
    let mut failures = Vec::new();

    for (field, wanted) in [("answers", "feature-request"), ("implements", "design-spec")]
    {
        let unresolved = Unresolved_Citation(store, submission, CitedField(field), WantedKind(wanted))?;

        failures.extend(unresolved);
    }

    return Ok(failures);
}

/// The submission field a citation rule checks (`answers`, `implements`).
#[derive(Clone, Copy)]
struct CitedField<'a>(&'a str);
/// The kind of submission a citation rule requires (`feature-request`, `design-spec`).
#[derive(Clone, Copy)]
struct WantedKind<'a>(&'a str);

/// Why one cited field does not resolve to an accepted submission of the kind it must name.
fn Unresolved_Citation(
    store: &SpecificationStore,
    submission: &Submission,
    field: CitedField<'_>,
    wanted: WantedKind<'_>,
) -> Result<Option<Failure>, StoreError>
{
    let Some(target) = Cited_Target(submission, field.0)
    else
    {
        return Ok(None);
    };

    let Some(remedy) = Citation_Remedy(store, target, field, wanted)?
    else
    {
        return Ok(None);
    };

    return Ok(Some(Citation_Failure(field.0, remedy)));
}

/// The value a citation field names, trimmed — `None` when the submission carries no
/// citation in that field at all.
fn Cited_Target<'a>(submission: &'a Submission, field: &str) -> Option<&'a str>
{
    let cited = submission.Current(field)?;

    return Some(cited.value.trim());
}

/// Why a cited target does not resolve to an accepted submission of the required kind —
/// `None` when it does.
fn Citation_Remedy(
    store: &SpecificationStore,
    target: &str,
    field: CitedField<'_>,
    wanted: WantedKind<'_>,
) -> Result<Option<String>, StoreError>
{
    let field = field.0;
    let wanted = wanted.0;
    let found = Cited_State(store, target)?;

    return Ok(match found
    {
        None => Some(format!(
            "no submission is filed under `{target}`; cite one that exists, and one that is \
             a {wanted}"
        )),
        Some((kind, _)) if kind != wanted =>
        {
            Some(format!("`{target}` is a {kind} and `{field}` must name a {wanted}"))
        }
        Some((_, state)) if state != SubmissionState::Accepted.Label() => Some(format!(
            "`{target}` is a {state}; accept it first, because work built against a draft is \
             work whose target may still change under it"
        )),
        Some(_) => None,
    });
}

/// The kind and state of the submission filed under `node_id`, if one is.
fn Cited_State(
    store: &SpecificationStore,
    node_id: &str,
) -> Result<Option<(String, String)>, StoreError>
{
    let found = store
        .Connection()
        .query_row(
            "SELECT s.kind, s.state
             FROM submissions s
             JOIN nodes n ON n.uid = s.node_uid
             WHERE n.node_id = ?1",
            [node_id],
            |row| return Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| return StoreError::Sql(error.to_string()))?;

    return Ok(found);
}

/// Bundles a citation rule's field and remedy into the [`Failure`] shape callers expect.
fn Citation_Failure(field: &str, remedy: String) -> Failure
{
    return Failure {
        field: field.to_owned(),
        rule: "citation-resolves-to-an-accepted-submission".to_owned(),
        remedy,
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::Seed_Governing_Records;
    use nomos_spec_model::{FieldValue, Origin, SubmissionKind};

    #[test]
    fn Test_Unresolved_Citations_Should_Report_A_Target_That_Was_Never_Filed()
    {
        let mut store = SpecificationStore::In_Memory().expect("In_Memory builds its own schema, so opening touches no file");
        Seed_Governing_Records(&mut store).expect("a seeded store");
        let design = Design_Citing("FR-404");

        let failures = Unresolved_Citations(&store, &design).expect("the rule itself does not fail");

        assert_eq!(failures.len(), 1);
        assert_eq!(failures.first().expect("one failure").field, "answers");
        assert!(
            failures.first().expect("one failure").remedy.contains("no submission is filed under"),
            "{:?}",
            failures
        );
    }

    /// A design submission whose `answers` field cites `target`.
    fn Design_Citing(target: &str) -> Submission
    {
        return Submission {
            id: "DS-900".to_owned(),
            kind: SubmissionKind::DesignSpec,
            form_contract_version: 1,
            state: SubmissionState::Draft,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values: vec![FieldValue { field: "answers".to_owned(), value: target.to_owned(), origin: Origin::Submitted }],
            gaps: Vec::new(),
        };
    }
}
