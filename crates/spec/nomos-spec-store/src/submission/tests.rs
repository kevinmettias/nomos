//! The accept function, exercised.

use super::*;
use crate::Table;
use nomos_spec_model::{DecisionGap, FieldValue, Severity, SubmissionKind};

/// The five values `Request` files under one submission.
const REQUEST_VALUE_COUNT: u32 = 5;

/// How many of `Request`'s values are unaccounted for once only `title` is supplied.
const HOLES_WHEN_ONLY_THE_TITLE_IS_SUPPLIED: usize = 4;

/// The `goal` rows standing after one clarification follows the original.
const GOAL_ROWS_AFTER_ONE_CLARIFICATION: usize = 2;

/// `supersedes_hash` is the second column of the projection `Goal_Values` selects.
const SUPERSEDES_HASH_COLUMN: usize = 1;

fn Store() -> SpecificationStore
{
    use crate::Seed_Governing_Records;

    let mut store = SpecificationStore::In_Memory()
        .expect("a fresh in-memory database sits at version 0, so only this crate's DDL is applied");
    Seed_Governing_Records(&mut store).expect("a seeded store");
    return store;
}

/// The first failure, named rather than indexed.
fn First(failures: &[nomos_spec_model::Failure]) -> &nomos_spec_model::Failure
{
    return failures.first().expect("at least one failure");
}

/// One stored value row, with a name for each column `Goal_Values` selects — so a caller reads
/// `row.supersedes` rather than counting positions.
struct StoredValue
{
    value: String,
    supersedes: Option<String>,
}

/// The stored row at `index`, named at the call site rather than reached through a position.
fn Nth(rows: &[StoredValue], index: usize) -> &StoredValue
{
    return rows.get(index).expect("a stored row");
}

/// A form field's name, named so a caller cannot transpose it with the value beside it.
struct FieldName<'a>(&'a str);

fn Value(field: FieldName<'_>, value: &str, origin: Origin) -> FieldValue
{
    return FieldValue {
        field: field.0.to_owned(),
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
            Value(FieldName("title"), "Items can be declined", Origin::Submitted),
            Value(FieldName("goal"), "close superseded work", Origin::Submitted),
            Value(FieldName("behaviour"), "a verb writes Declined", Origin::Submitted),
            Value(FieldName("acceptance"), "it stops being claimable", Origin::Submitted),
            Value(FieldName("invariants"), "none", Origin::Submitted),
        ],
        gaps: Vec::new(),
    };
}

#[test]
fn Test_An_Accepted_Request_Should_Land_As_A_Node_And_Its_Values()
{
    let mut store = Store();

    let request = Request("FR-001", SubmissionState::Accepted);
    Accept_Submission(&mut store, &request)
        .expect("FR-001 carries every required field, so acceptance succeeds");

    assert_eq!(store.Count(Table::Submissions).expect("a count"), 1);
    assert_eq!(store.Count(Table::SubmissionValues).expect("a count"), REQUEST_VALUE_COUNT);
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
    let error = Accept_Submission(&mut store, &submission)
        .expect_err("FR-002 supplies no goal, so the completeness rule must refuse it");

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

    let error = Accept_Submission(&mut store, &submission)
        .expect_err("FR-003 supplies only its title, so four required fields are missing");

    let AcceptError::Refused(refusal) = error
    else
    {
        // Only `Refused` carries a list of failures, and the length of that list is the whole
        // claim here: four missing fields must be reported as four, not as the first one. A
        // `Store` variant has nothing to count.
        panic!("refused for the wrong reason");
    };

    assert_eq!(
        refusal.failures.len(),
        HOLES_WHEN_ONLY_THE_TITLE_IS_SUPPLIED,
        "four holes, one refusal"
    );
    assert!(refusal.to_string().contains("nothing was stored"));
}

#[test]
fn Test_A_Later_Value_Should_Be_Stored_Beside_The_One_It_Supersedes()
{
    let mut store = Store();
    let mut submission = Request("FR-004", SubmissionState::Accepted);
    let clarified = Value(FieldName("goal"), "what it became", Origin::Clarified);
    submission.values.push(clarified);
    Accept_Submission(&mut store, &submission)
        .expect("a clarification is an addition, so the submission still passes every rule");

    let rows = Goal_Values(&store);

    assert_eq!(rows.len(), GOAL_ROWS_AFTER_ONE_CLARIFICATION, "the original is still in storage");
    assert_eq!(Nth(&rows, 0).value, "close superseded work");
    assert_eq!(Nth(&rows, 1).value, "what it became");
    assert!(Nth(&rows, 0).supersedes.is_none(), "the first supersedes nothing");
    assert!(Nth(&rows, 1).supersedes.is_some(), "the second names what it superseded");
}

/// Every value ever written for `goal`, oldest first, with what each superseded.
fn Goal_Values(store: &SpecificationStore) -> Vec<StoredValue>
{
    let mut statement = store
        .Connection()
        .prepare(
            "SELECT value, supersedes_hash FROM submission_values
             WHERE field = 'goal' ORDER BY ordinal",
        )
        .expect("the projection names two columns this connection holds");

    return statement
        .query_map([], |row| {
            return Ok(StoredValue {
                value: row.get(0)?,
                supersedes: row.get(SUPERSEDES_HASH_COLUMN)?,
            });
        })
        .expect("the query above compiles against the schema the store applies")
        .map(|row| return row.expect("query_map yields one row per submission_values row"))
        .collect();
}

/// The named request, carrying one open gap over `question`, blocking `behaviour`.
fn Submission_With_A_Blocking_Gap(id: &str, state: SubmissionState) -> Submission
{
    let mut submission = Request(id, state);
    submission.gaps = vec![DecisionGap {
        question: "which substrate is canonical".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::Blocking,
        closed_by: None,
    }];

    return submission;
}

/// The refusal a rejected `Accept_Submission` call carried, or a panic naming the wrong
/// variant.
///
/// Only `Refused` carries the rule name a caller wants to assert on; a `Store` variant would
/// mean the write failed for a database reason and the rule set was never consulted, which is
/// the reading the caller's own assertion would otherwise be unable to distinguish from a
/// passing test.
fn Expect_Refusal(result: Result<i64, AcceptError>, context: &str) -> Refusal
{
    let AcceptError::Refused(refusal) =
        result.expect_err("the caller hands over only a call it expects to be refused")
    else
    {
        panic!("{context}: refused for the wrong reason");
    };

    return refusal;
}

#[test]
fn Test_An_Open_Blocking_Gap_Should_Be_A_Row_And_Should_Refuse_Acceptance()
{
    let mut store = Store();
    let mut submission = Submission_With_A_Blocking_Gap("FR-005", SubmissionState::Draft);

    Accept_Submission(&mut store, &submission).expect("a draft may carry an open gap");
    assert_eq!(store.Count(Table::SubmissionGaps).expect("a count"), 1);

    submission.state = SubmissionState::Accepted;
    let outcome = Accept_Submission(&mut store, &submission);
    let refusal = Expect_Refusal(outcome, "the blocking gap is still open");

    assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");
}

#[test]
fn Test_A_Gap_Should_Not_Be_Closed_By_Supplying_The_Value_It_Blocks()
{
    let mut store = Store();
    let mut submission = Submission_With_A_Blocking_Gap("FR-006", SubmissionState::Accepted);
    let supplied =
        Value(FieldName("behaviour"), "the blocked value, supplied", Origin::Submitted);
    submission.values.push(supplied);

    let outcome = Accept_Submission(&mut store, &submission);
    let refusal = Expect_Refusal(outcome, "supplying the blocked value leaves the gap open");
    assert_eq!(First(&refusal.failures).rule, "no-open-blocking-gap");

    submission
        .gaps
        .first_mut()
        .expect("Submission_With_A_Blocking_Gap put one gap on this submission")
        .closed_by = Some("OD-SPEC-008".to_owned());
    Accept_Submission(&mut store, &submission).expect("a citation closes it");
}

/// The submission a design cites, named so a caller cannot transpose it with the design's id.
struct CitedSubmission<'a>(&'a str);

fn Design(id: &str, answers: CitedSubmission<'_>, state: SubmissionState) -> Submission
{
    return Submission {
        id: id.to_owned(),
        kind: SubmissionKind::DesignSpec,
        form_contract_version: 1,
        state,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values: vec![
            Value(FieldName("title"), "a design", Origin::Submitted),
            Value(FieldName("answers"), answers.0, Origin::Submitted),
            Value(FieldName("alternatives"), "a new verb\ndo nothing", Origin::Submitted),
            Value(FieldName("selected"), "a new verb", Origin::Submitted),
            Value(FieldName("architecture_delta"), "none", Origin::Submitted),
            Value(FieldName("acceptance"), "the tests pass", Origin::Submitted),
        ],
        gaps: Vec::new(),
    };
}

#[test]
fn Test_A_Citation_Naming_Nothing_Should_Be_Refused()
{
    let mut store = Store();

    let design = Design("DS-001", CitedSubmission("FR-404"), SubmissionState::Draft);
    let error = Accept_Submission(&mut store, &design)
        .expect_err("FR-404 is filed nowhere, so the citation rule must refuse the design");

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
    let design = Design("DS-002", CitedSubmission("FR-007"), SubmissionState::Draft);

    let refusal = Refused_Draft_Citation(&mut store, &design);

    assert!(First(&refusal.failures).remedy.contains("accept it first"));

    request.state = SubmissionState::Accepted;
    Accept_Submission(&mut store, &request)
        .expect("FR-007 is Accepted at this point in the test, so this call lands");
    Accept_Submission(&mut store, &design)
        .expect("DS-002 cites FR-007, which the line above just accepted");
}

/// The refusal `design` must draw when it cites a submission that is still a draft.
///
/// `FR-007` exists — it is merely a draft — so the claim here is that the citation rule
/// declined an unaccepted target, not that a lookup found nothing. Only `Refused` carries the
/// remedy that tells those two apart, and a `Store` variant would mean the citation was never
/// consulted at all.
fn Refused_Draft_Citation(store: &mut SpecificationStore, design: &Submission) -> Refusal
{
    let outcome = Accept_Submission(store, design);

    return Expect_Refusal(
        outcome,
        "FR-007 is a draft, so this refusal must be the citation rule declining it",
    );
}

#[test]
fn Test_An_Accepted_Citation_Should_Write_The_Lifecycle_Edge()
{
    let mut store = Store();
    let request = Request("FR-008", SubmissionState::Accepted);
    let design = Design("DS-003", CitedSubmission("FR-008"), SubmissionState::Draft);
    Accept_Submission(&mut store, &request)
        .expect("FR-008 carries every required field, so acceptance succeeds");
    Accept_Submission(&mut store, &design)
        .expect("DS-003 cites FR-008, which the line above just accepted");

    let edges: u32 = store
        .Connection()
        .query_row(
            "SELECT COUNT(*) FROM relations WHERE relation_type = 'answers'",
            [],
            |row| return row.get(0),
        )
        .expect("the relations table answers a COUNT over its single projected column");

    assert_eq!(edges, 1, "the edge OD-SPEC-013 added the vocabulary for");
}
