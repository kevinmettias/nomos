//! The accept function, exercised.

use super::*;
use crate::Table;
use nomos_spec_model::{DecisionGap, FieldValue, Severity, SubmissionKind};

fn Store() -> SpecificationStore
{
    use crate::Seed_Governing_Records;

    let mut store = SpecificationStore::In_Memory().expect("a store");
    Seed_Governing_Records(&mut store).expect("a seeded store");
    return store;
}

/// The first failure, named rather than indexed.
fn First(failures: &[nomos_spec_model::Failure]) -> &nomos_spec_model::Failure
{
    return failures.first().expect("at least one failure");
}

/// One stored value row, named rather than indexed.
fn Nth(rows: &[(String, u32, Option<String>)], index: usize)
-> &(String, u32, Option<String>)
{
    return rows.get(index).expect("a stored row");
}

fn Value(field: &str, value: &str, origin: Origin) -> FieldValue
{
    return FieldValue {
        field: field.to_owned(),
        value: value.to_owned(),
        origin,
    };
}

fn Request(id: &str, state: SubmissionState) -> Submission
{
    return Submission {
        id: id.to_owned(),
        kind: SubmissionKind::FeatureRequest,
        form_contract_version: 1,
        state,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values: vec![
            Value("title", "Items can be declined", Origin::Submitted),
            Value("goal", "close superseded work", Origin::Submitted),
            Value("behaviour", "a verb writes Declined", Origin::Submitted),
            Value("acceptance", "it stops being claimable", Origin::Submitted),
            Value("invariants", "none", Origin::Submitted),
        ],
        gaps: Vec::new(),
    };
}

#[test]
fn Test_An_Accepted_Request_Should_Land_As_A_Node_And_Its_Values()
{
    let mut store = Store();

    let request = Request("FR-001", SubmissionState::Accepted);
    Accept_Submission(&mut store, &request).expect("accepted");

    assert_eq!(store.Count(Table::Submissions).expect("a count"), 1);
    assert_eq!(store.Count(Table::SubmissionValues).expect("a count"), 5);
    assert!(
        store.Node_Uid("FR-001").expect("a lookup").is_some(),
        "a submission is a node"
    );
}

#[test]
fn Test_An_Incomplete_Submission_Should_Be_Refused_And_Stored_Nowhere()
{
    let mut store = Store();
    let mut submission = Request("FR-002", SubmissionState::Accepted);
    submission.values.retain(|value| return value.field != "goal");
    let error = Accept_Submission(&mut store, &submission).expect_err("refused");

    match error
    {
        AcceptError::Refused(refusal) =>
        {
            assert_eq!(refusal.submission, "FR-002");
            assert_eq!(refusal.failures.len(), 1);
            assert_eq!(First(&refusal.failures).field, "goal");
        }
        // A `Store` variant would mean the write was attempted and the database objected. The
        // three assertions below — no submission row, no value rows, no node — would then be
        // measuring what a failed write happened to leave behind rather than a submission
        // that was refused before anything was written.
        AcceptError::Store(error) => panic!("refused for the wrong reason: {error}"),
    }
    assert_eq!(store.Count(Table::Submissions).expect("a count"), 0);
    assert_eq!(store.Count(Table::SubmissionValues).expect("a count"), 0);
    assert!(
        store.Node_Uid("FR-002").expect("a lookup").is_none(),
        "there is no partially written row"
    );
}

#[test]
fn Test_A_Refusal_Should_Name_Every_Failure_Rather_Than_The_First()
{
    let mut store = Store();
    let mut submission = Request("FR-003", SubmissionState::Accepted);
    submission.values.retain(|value| return value.field == "title");

    let error = Accept_Submission(&mut store, &submission).expect_err("refused");

    let AcceptError::Refused(refusal) = error
    else
    {
        // Only `Refused` carries a list of failures, and the length of that list is the whole
        // claim here: four missing fields must be reported as four, not as the first one. A
        // `Store` variant has nothing to count.
        panic!("refused for the wrong reason");
    };

    assert_eq!(refusal.failures.len(), 4, "four holes, one refusal");
    assert!(refusal.to_string().contains("nothing was stored"));
}

#[test]
fn Test_A_Later_Value_Should_Be_Stored_Beside_The_One_It_Supersedes()
{
    let mut store = Store();
    let mut submission = Request("FR-004", SubmissionState::Accepted);
    let clarified = Value("goal", "what it became", Origin::Clarified);
    submission.values.push(clarified);
    Accept_Submission(&mut store, &submission).expect("accepted");

    let rows = Goal_Values(&store);

    assert_eq!(rows.len(), 2, "the original is still in storage");
    assert_eq!(Nth(&rows, 0).0, "close superseded work");
    assert_eq!(Nth(&rows, 1).0, "what it became");
    assert!(Nth(&rows, 0).2.is_none(), "the first supersedes nothing");
    assert!(Nth(&rows, 1).2.is_some(), "the second names what it superseded");
}

/// Every value ever written for `goal`, oldest first, with what each superseded.
fn Goal_Values(store: &SpecificationStore) -> Vec<(String, u32, Option<String>)>
{
    let mut statement = store
        .Connection()
        .prepare(
            "SELECT value, ordinal, supersedes_hash FROM submission_values
             WHERE field = 'goal' ORDER BY ordinal",
        )
        .expect("a query");

    return statement
        .query_map([], |row| {
            return Ok((row.get(0)?, row.get(1)?, row.get(2)?));
        })
        .expect("rows")
        .map(|row| return row.expect("a row"))
        .collect();
}

#[test]
fn Test_An_Open_Blocking_Gap_Should_Be_A_Row_And_Should_Refuse_Acceptance()
{
    let mut store = Store();
    let mut submission = Request("FR-005", SubmissionState::Draft);
    submission.gaps = vec![DecisionGap {
        question: "which substrate is canonical".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::Blocking,
        closed_by: None,
    }];

    Accept_Submission(&mut store, &submission).expect("a draft may carry an open gap");
    assert_eq!(store.Count(Table::SubmissionGaps).expect("a count"), 1);

    submission.state = SubmissionState::Accepted;
    let error = Accept_Submission(&mut store, &submission).expect_err("refused");

    let AcceptError::Refused(refusal) = error
    else
    {
        // The same submission was accepted as a draft four lines up, so the only thing that
        // changed is its state. A `Store` variant would mean the second write failed for a
        // database reason and the gap rule was never consulted — which is the reading the
        // assertion below would otherwise be unable to distinguish from a passing test.
        panic!("refused for the wrong reason");
    };
    assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");
}

#[test]
fn Test_A_Gap_Should_Not_Be_Closed_By_Supplying_The_Value_It_Blocks()
{
    let mut store = Store();
    let mut submission = Request("FR-006", SubmissionState::Accepted);
    submission.gaps = vec![DecisionGap {
        question: "which substrate is canonical".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::Blocking,
        closed_by: None,
    }];
    let supplied = Value("behaviour", "the blocked value, supplied", Origin::Submitted);
    submission.values.push(supplied);

    let error = Accept_Submission(&mut store, &submission).expect_err("still refused");

    let AcceptError::Refused(refusal) = error
    else
    {
        // The submission supplies `behaviour`, the very field the gap blocks, so the question
        // is whether that value closed the gap. Only `Refused` carries the rule name the
        // assertion below reads, and it is the rule name — not the fact of a refusal — that
        // says the gap was still open rather than some other check firing.
        panic!("refused for the wrong reason");
    };
    assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");

    submission.gaps.first_mut().expect("a gap").closed_by = Some("OD-SPEC-008".to_owned());
    Accept_Submission(&mut store, &submission).expect("a citation closes it");
}

fn Design(id: &str, answers: &str, state: SubmissionState) -> Submission
{
    return Submission {
        id: id.to_owned(),
        kind: SubmissionKind::DesignSpec,
        form_contract_version: 1,
        state,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values: vec![
            Value("title", "a design", Origin::Submitted),
            Value("answers", answers, Origin::Submitted),
            Value("alternatives", "a new verb\ndo nothing", Origin::Submitted),
            Value("selected", "a new verb", Origin::Submitted),
            Value("architecture_delta", "none", Origin::Submitted),
            Value("acceptance", "the tests pass", Origin::Submitted),
        ],
        gaps: Vec::new(),
    };
}

#[test]
fn Test_A_Citation_Naming_Nothing_Should_Be_Refused()
{
    let mut store = Store();

    let design = Design("DS-001", "FR-404", SubmissionState::Draft);
    let error = Accept_Submission(&mut store, &design).expect_err("refused");

    let AcceptError::Refused(refusal) = error
    else
    {
        // `FR-404` was never filed, so the refusal must come from the citation rule. A `Store`
        // variant would mean the design reached the database before its citation was checked,
        // and neither the rule name nor the remedy asserted below would exist to read.
        panic!("refused for the wrong reason");
    };
    assert_eq!(
        First(&refusal.failures).rule,
        "citation-resolves-to-an-accepted-submission"
    );
    assert!(First(&refusal.failures).remedy.contains("no submission is filed under"));
}

#[test]
fn Test_A_Citation_Naming_A_Draft_Should_Be_Refused_Until_It_Is_Accepted()
{
    let mut store = Store();
    let mut request = Request("FR-007", SubmissionState::Draft);
    Accept_Submission(&mut store, &request).expect("a draft request");

    let design = Design("DS-002", "FR-007", SubmissionState::Draft);
    let error = Accept_Submission(&mut store, &design).expect_err("refused");

    let AcceptError::Refused(refusal) = error
    else
    {
        // Unlike the case above, `FR-007` exists — it is merely a draft — so this refusal has
        // to be the citation rule declining an unaccepted target rather than a lookup that
        // found nothing. Only `Refused` carries the remedy that tells those two apart, and
        // the test goes on to accept the same design once the request is promoted.
        panic!("refused for the wrong reason");
    };
    assert!(First(&refusal.failures).remedy.contains("accept it first"));

    request.state = SubmissionState::Accepted;
    Accept_Submission(&mut store, &request).expect("promoted");
    Accept_Submission(&mut store, &design).expect("now it resolves");
}

#[test]
fn Test_An_Accepted_Citation_Should_Write_The_Lifecycle_Edge()
{
    let mut store = Store();
    let request = Request("FR-008", SubmissionState::Accepted);
    let design = Design("DS-003", "FR-008", SubmissionState::Draft);
    Accept_Submission(&mut store, &request).expect("a request");
    Accept_Submission(&mut store, &design).expect("a design");

    let edges: u32 = store
        .Connection()
        .query_row(
            "SELECT COUNT(*) FROM relations WHERE relation_type = 'answers'",
            [],
            |row| return row.get(0),
        )
        .expect("a count");

    assert_eq!(edges, 1, "the edge OD-SPEC-013 added the vocabulary for");
}
