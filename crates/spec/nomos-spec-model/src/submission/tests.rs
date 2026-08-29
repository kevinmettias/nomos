//! The rule set, exercised against submissions that pass and submissions that do not.

use super::*;
use crate::Origin;
use crate::Severity;

/// The first failure, named rather than indexed.
fn First(failures: &[Failure]) -> &Failure
{
    return failures.first().expect("at least one failure");
}

fn Value(field: &str, value: &str, origin: Origin) -> FieldValue
{
    return FieldValue {
        field: field.to_owned(),
        value: value.to_owned(),
        origin,
    };
}

fn Request(state: SubmissionState, values: Vec<FieldValue>) -> Submission
{
    return Submission {
        id: "FR-001".to_owned(),
        kind: SubmissionKind::FeatureRequest,
        form_contract_version: 1,
        state,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values,
        gaps: Vec::new(),
    };
}

fn Complete_Request_Values() -> Vec<FieldValue>
{
    return vec![
        Value("title", "Ledger items can be declined", Origin::Submitted),
        Value("goal", "close superseded work", Origin::Submitted),
        Value("behaviour", "a verb writes Declined", Origin::Submitted),
        Value("acceptance", "the item stops being claimable", Origin::Submitted),
        Value("invariants", "none", Origin::Submitted),
    ];
}

#[test]
fn Test_A_Complete_Request_Should_Fail_Nothing()
{
    let submission = Request(SubmissionState::Accepted, Complete_Request_Values());

    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_A_Refusal_Should_Name_Every_Missing_Field_Rather_Than_The_First()
{
    let submission = Request(SubmissionState::Draft, vec![Value(
        "title",
        "something",
        Origin::Submitted,
    )]);

    let failures = Validate_Submission(&submission);
    let fields: Vec<&str> = failures
        .iter()
        .map(|failure| return failure.field.as_str())
        .collect();

    assert_eq!(fields, vec!["goal", "behaviour", "acceptance", "invariants"]);
}

#[test]
fn Test_A_Draft_Should_Be_Refused_For_Incompleteness_Exactly_As_An_Accepted_One_Is()
{
    let missing = vec![
        Value("title", "something", Origin::Submitted),
        Value("goal", "a goal", Origin::Submitted),
    ];

    let as_draft = Request(SubmissionState::Draft, missing.clone());
    let as_accepted = Request(SubmissionState::Accepted, missing);
    let draft = Validate_Submission(&as_draft);
    let accepted = Validate_Submission(&as_accepted);

    let draft_fields: Vec<&String> = draft.iter().map(|f| return &f.field).collect();
    let accepted_fields: Vec<&String> = accepted
        .iter()
        .filter(|f| return f.rule == "required-field")
        .map(|f| return &f.field)
        .collect();

    assert_eq!(draft_fields, accepted_fields);
    assert!(!draft.is_empty(), "incompleteness is refused in both states");
}

#[test]
fn Test_A_Stated_Absence_Should_Satisfy_A_Required_Field()
{
    let submission = Request(SubmissionState::Accepted, Complete_Request_Values());

    assert_eq!(submission.Current("invariants").map(|v| return v.value.as_str()), Some("none"));
    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_An_Inferred_Value_Should_Be_Readable_And_Never_Sufficient()
{
    let mut values = Complete_Request_Values();
    let inferred = Value("goal", "guessed from the title", Origin::Inferred);
    values.push(inferred);

    let as_draft = Request(SubmissionState::Draft, values.clone());
    let as_accepted = Request(SubmissionState::Accepted, values);
    let draft = Validate_Submission(&as_draft);
    let accepted = Validate_Submission(&as_accepted);

    assert_eq!(draft, Vec::new(), "a draft may carry an inferred value");
    assert_eq!(accepted.len(), 1);
    assert_eq!(First(&accepted).rule, "accepted-values-are-not-inferred");
    assert_eq!(First(&accepted).field, "goal");
}

#[test]
fn Test_A_Later_Value_Should_Supersede_An_Earlier_One_For_Reading_Only()
{
    let mut values = Complete_Request_Values();
    let clarified = Value("goal", "what it became", Origin::Clarified);
    values.push(clarified);

    let submission = Request(SubmissionState::Accepted, values);

    assert_eq!(
        submission.Current("goal").map(|v| return v.value.as_str()),
        Some("what it became")
    );
    assert!(
        submission
            .values
            .iter()
            .any(|v| return v.value == "close superseded work"),
        "the original is still in storage"
    );
}

#[test]
fn Test_An_Open_Blocking_Gap_Should_Refuse_Acceptance_And_Allow_A_Draft()
{
    let gap = DecisionGap {
        question: "which substrate is canonical".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::Blocking,
        closed_by: None,
    };

    let mut submission = Request(SubmissionState::Draft, Complete_Request_Values());
    submission.gaps = vec![gap];

    assert_eq!(Validate_Submission(&submission), Vec::new());

    submission.state = SubmissionState::Accepted;
    let failures = Validate_Submission(&submission);

    assert_eq!(failures.len(), 1);
    assert_eq!(First(&failures).rule, "no-open-blocking-gap");
}

#[test]
fn Test_A_Gap_Closed_By_A_Citation_Should_Stop_Blocking()
{
    let mut submission = Request(SubmissionState::Accepted, Complete_Request_Values());
    submission.gaps = vec![DecisionGap {
        question: "which substrate is canonical".to_owned(),
        blocks: vec!["behaviour".to_owned()],
        severity: Severity::Blocking,
        closed_by: Some("OD-SPEC-008".to_owned()),
    }];

    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_A_Non_Blocking_Gap_Should_Survive_Acceptance()
{
    let mut submission = Request(SubmissionState::Accepted, Complete_Request_Values());
    submission.gaps = vec![DecisionGap {
        question: "what the fifth surface is".to_owned(),
        blocks: Vec::new(),
        severity: Severity::NonBlocking,
        closed_by: None,
    }];

    assert_eq!(Validate_Submission(&submission), Vec::new());
    assert!(
        submission.gaps.first().expect("a gap").Is_Open(),
        "and it is still open"
    );
}

fn Design(alternatives: &str, selected: &str) -> Submission
{
    return Submission {
        id: "DS-001".to_owned(),
        kind: SubmissionKind::DesignSpec,
        form_contract_version: 1,
        state: SubmissionState::Accepted,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values: vec![
            Value("title", "a title", Origin::Submitted),
            Value("answers", "FR-001", Origin::Submitted),
            Value("alternatives", alternatives, Origin::Submitted),
            Value("selected", selected, Origin::Submitted),
            Value("architecture_delta", "none", Origin::Submitted),
            Value("acceptance", "the tests pass", Origin::Submitted),
        ],
        gaps: Vec::new(),
    };
}

#[test]
fn Test_A_Design_With_One_Alternative_Should_Be_Refused()
{
    let design = Design("a new verb", "a new verb");
    let failures = Validate_Submission(&design);

    assert_eq!(failures.len(), 1);
    assert_eq!(First(&failures).rule, "at-least-two-alternatives");
}

#[test]
fn Test_Do_Nothing_Should_Be_An_Admissible_Alternative()
{
    let design = Design("a new verb\ndo nothing", "a new verb");

    assert_eq!(Validate_Submission(&design), Vec::new());
}

#[test]
fn Test_A_Selected_Option_Absent_From_The_Alternatives_Should_Be_Refused()
{
    let design = Design("a new verb\ndo nothing", "a third thing");
    let failures = Validate_Submission(&design);

    assert_eq!(failures.len(), 1);
    assert_eq!(First(&failures).rule, "selected-names-an-alternative");
}

fn Result_Submission(deviations: &str, evidence: Option<&str>) -> Submission
{
    let mut values = vec![
        Value("title", "a title", Origin::Submitted),
        Value("implements", "DS-001", Origin::Submitted),
        Value("deviations", deviations, Origin::Submitted),
        Value("owed", "none", Origin::Submitted),
    ];
    if let Some(evidence) = evidence
    {
        let value = Value("evidence", evidence, Origin::Submitted);
        values.push(value);
    }

    return Submission {
        id: "FRS-001".to_owned(),
        kind: SubmissionKind::FeatureResult,
        form_contract_version: 1,
        state: SubmissionState::Accepted,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values,
        gaps: Vec::new(),
    };
}

#[test]
fn Test_A_Deviation_Naming_No_Clause_Should_Be_Refused()
{
    let submission = Result_Submission("we did it differently", Some("cargo test: 0"));
    let failures = Validate_Submission(&submission);

    assert_eq!(failures.len(), 1);
    assert_eq!(First(&failures).rule, "deviation-names-its-clause");
}

#[test]
fn Test_A_Deviation_Naming_Its_Clause_Should_Pass()
{
    let submission =
        Result_Submission("section 3: used one table, not two", Some("cargo test: 0"));

    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_Evidence_Should_Be_Required_For_An_Accepted_Result_And_Not_For_A_Draft()
{
    let mut submission = Result_Submission("section 3: a departure", None);

    let accepted = Validate_Submission(&submission);
    assert_eq!(accepted.len(), 1);
    assert_eq!(First(&accepted).rule, "accepted-result-carries-evidence");

    submission.state = SubmissionState::Draft;
    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_Every_Label_Should_Round_Trip_Through_Parse()
{
    let kinds = [
        SubmissionKind::FeatureRequest,
        SubmissionKind::DesignSpec,
        SubmissionKind::FeatureResult,
    ];
    for kind in kinds
    {
        assert_eq!(SubmissionKind::Parse(kind.Label()), Some(kind));
    }
    for origin in [Origin::Submitted, Origin::Clarified, Origin::Inferred, Origin::Decided]
    {
        assert_eq!(Origin::Parse(origin.Label()), Some(origin));
    }
    for severity in [Severity::Blocking, Severity::NonBlocking]
    {
        assert_eq!(Severity::Parse(severity.Label()), Some(severity));
    }
    for state in [SubmissionState::Draft, SubmissionState::Accepted]
    {
        assert_eq!(SubmissionState::Parse(state.Label()), Some(state));
    }
}
