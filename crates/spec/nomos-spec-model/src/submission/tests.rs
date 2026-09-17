//! The rule set, exercised against submissions that pass and submissions that do not.

use super::*;
use crate::Origin;
use crate::Severity;

/// The first failure, named rather than indexed.
fn First_Failure(failures: &[Failure]) -> &Failure
{
    return failures.first().expect("at least one failure");
}

/// One field's name and the text it carries, grouped so a call site writes both by name.
///
/// As two bare `&str` parameters the pair is transposable and nothing would object:
/// `Field_Value_With_Origin("something", "title", origin)` compiles and means the
/// opposite of what it reads as. Naming each position is what makes that a compile
/// error instead.
struct Declared<'a>
{
    field: &'a str,
    value: &'a str,
}

fn Field_Value_With_Origin(declared: Declared<'_>, origin: Origin) -> FieldValue
{
    return FieldValue {
        field: declared.field.to_owned(),
        value: declared.value.to_owned(),
        origin,
    };
}

fn Submission_Request(state: SubmissionState, values: Vec<FieldValue>) -> Submission
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
        Field_Value_With_Origin(
            Declared { field: "title", value: "Ledger items can be declined" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(
            Declared { field: "goal", value: "close superseded work" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(
            Declared { field: "behaviour", value: "a verb writes Declined" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(
            Declared { field: "acceptance", value: "the item stops being claimable" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(Declared { field: "invariants", value: "none" }, Origin::Submitted),
    ];
}

#[test]
fn Test_Validate_Submission_Should_Pass_A_Complete_Request()
{
    let submission = Submission_Request(SubmissionState::Accepted, Complete_Request_Values());

    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_Required_Fields_Should_Include_Title_And_The_Kinds_Own_Fields()
{
    let submission = Submission_Request(SubmissionState::Draft, Vec::new());

    assert_eq!(
        submission.Required_Fields(),
        vec!["title", "goal", "behaviour", "acceptance", "invariants"]
    );
}

#[test]
fn Test_A_Refusal_Should_Name_Every_Missing_Field_Rather_Than_The_First()
{
    let declared = Field_Value_With_Origin(
        Declared { field: "title", value: "something" },
        Origin::Submitted,
    );
    let submission = Submission_Request(SubmissionState::Draft, vec![declared]);

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
        Field_Value_With_Origin(Declared { field: "title", value: "something" }, Origin::Submitted),
        Field_Value_With_Origin(Declared { field: "goal", value: "a goal" }, Origin::Submitted),
    ];

    let as_draft = Submission_Request(SubmissionState::Draft, missing.clone());
    let as_accepted = Submission_Request(SubmissionState::Accepted, missing);
    let draft = Validate_Submission(&as_draft);
    let accepted = Validate_Submission(&as_accepted);

    let draft_fields: Vec<&String> = draft.iter().map(|finding| return &finding.field).collect();
    let accepted_fields: Vec<&String> = accepted
        .iter()
        .filter(|finding| return finding.rule == "required-field")
        .map(|finding| return &finding.field)
        .collect();

    assert_eq!(draft_fields, accepted_fields);
    assert!(!draft.is_empty(), "incompleteness is refused in both states");
}

#[test]
fn Test_A_Stated_Absence_Should_Satisfy_A_Required_Field()
{
    let submission = Submission_Request(SubmissionState::Accepted, Complete_Request_Values());

    assert_eq!(submission.Current("invariants").map(|v| return v.value.as_str()), Some("none"));
    assert_eq!(Validate_Submission(&submission), Vec::new());
}

#[test]
fn Test_An_Inferred_Value_Should_Be_Readable_And_Never_Sufficient()
{
    let mut values = Complete_Request_Values();
    let inferred = Field_Value_With_Origin(
        Declared { field: "goal", value: "guessed from the title" },
        Origin::Inferred,
    );
    values.push(inferred);

    let as_draft = Submission_Request(SubmissionState::Draft, values.clone());
    let as_accepted = Submission_Request(SubmissionState::Accepted, values);
    let draft = Validate_Submission(&as_draft);
    let accepted = Validate_Submission(&as_accepted);

    assert_eq!(draft, Vec::new(), "a draft may carry an inferred value");
    assert_eq!(accepted.len(), 1);
    assert_eq!(First_Failure(&accepted).rule, "accepted-values-are-not-inferred");
    assert_eq!(First_Failure(&accepted).field, "goal");
}

#[test]
fn Test_Current_Should_Read_The_Latest_Value_Not_An_Earlier_One()
{
    let mut values = Complete_Request_Values();
    let clarified = Field_Value_With_Origin(
        Declared { field: "goal", value: "what it became" },
        Origin::Clarified,
    );
    values.push(clarified);

    let submission = Submission_Request(SubmissionState::Accepted, values);

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

    let mut submission = Submission_Request(SubmissionState::Draft, Complete_Request_Values());
    submission.gaps = vec![gap];

    assert_eq!(Validate_Submission(&submission), Vec::new());

    submission.state = SubmissionState::Accepted;
    let failures = Validate_Submission(&submission);

    assert_eq!(failures.len(), 1);
    assert_eq!(First_Failure(&failures).rule, "no-open-blocking-gap");
}

#[test]
fn Test_A_Gap_Closed_By_A_Citation_Should_Stop_Blocking()
{
    let mut submission = Submission_Request(SubmissionState::Accepted, Complete_Request_Values());
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
    let mut submission = Submission_Request(SubmissionState::Accepted, Complete_Request_Values());
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

/// A design's two free-text answers, grouped for the same reason [`Declared`] is: two adjacent
/// `&str` positions are a pair a caller can transpose with nothing to catch it.
struct DesignAnswers<'a>
{
    alternatives: &'a str,
    selected: &'a str,
}

fn Submission_Design(answers: DesignAnswers<'_>) -> Submission
{
    return Submission {
        id: "DS-001".to_owned(),
        kind: SubmissionKind::DesignSpec,
        form_contract_version: 1,
        state: SubmissionState::Accepted,
        submitted_by: "kevin".to_owned(),
        submitted_through: "cli".to_owned(),
        values: vec![
            Field_Value_With_Origin(
                Declared { field: "title", value: "a title" },
                Origin::Submitted,
            ),
            Field_Value_With_Origin(
                Declared { field: "answers", value: "FR-001" },
                Origin::Submitted,
            ),
            Field_Value_With_Origin(
                Declared { field: "alternatives", value: answers.alternatives },
                Origin::Submitted,
            ),
            Field_Value_With_Origin(
                Declared { field: "selected", value: answers.selected },
                Origin::Submitted,
            ),
            Field_Value_With_Origin(
                Declared { field: "architecture_delta", value: "none" },
                Origin::Submitted,
            ),
            Field_Value_With_Origin(
                Declared { field: "acceptance", value: "the tests pass" },
                Origin::Submitted,
            ),
        ],
        gaps: Vec::new(),
    };
}

/// One malformed design, and the rule it should be refused by.
struct DesignRefusalCase
{
    alternatives: &'static str,
    selected: &'static str,
    rule: &'static str,
}

/// How many malformed designs the fixture below carries, one per rule it exercises.
const REFUSAL_CASES: usize = 2;

fn Designs_That_Should_Be_Refused() -> [DesignRefusalCase; REFUSAL_CASES]
{
    return [
        DesignRefusalCase {
            alternatives: "a new verb",
            selected: "a new verb",
            rule: "at-least-two-alternatives",
        },
        DesignRefusalCase {
            alternatives: "a new verb\ndo nothing",
            selected: "a third thing",
            rule: "selected-names-an-alternative",
        },
    ];
}

#[test]
fn Test_A_Malformed_Design_Should_Be_Refused_By_Its_Own_Rule()
{
    for case in Designs_That_Should_Be_Refused()
    {
        let answers = DesignAnswers { alternatives: case.alternatives, selected: case.selected };
        let design = Submission_Design(answers);
        let failures = Validate_Submission(&design);

        assert_eq!(failures.len(), 1, "{}", case.rule);
        assert_eq!(First_Failure(&failures).rule, case.rule);
    }
}

#[test]
fn Test_An_Inaction_Alternative_Should_Be_Admissible()
{
    let answers = DesignAnswers { alternatives: "a new verb\ndo nothing", selected: "a new verb" };
    let design = Submission_Design(answers);

    assert_eq!(Validate_Submission(&design), Vec::new());
}

fn Result_Submission(deviations: &str, evidence: Option<&str>) -> Submission
{
    let mut values = vec![
        Field_Value_With_Origin(
            Declared { field: "title", value: "a title" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(
            Declared { field: "implements", value: "DS-001" },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(
            Declared { field: "deviations", value: deviations },
            Origin::Submitted,
        ),
        Field_Value_With_Origin(Declared { field: "owed", value: "none" }, Origin::Submitted),
    ];
    if let Some(evidence) = evidence
    {
        let value = Field_Value_With_Origin(
            Declared { field: "evidence", value: evidence },
            Origin::Submitted,
        );
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
    assert_eq!(First_Failure(&failures).rule, "deviation-names-its-clause");
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
    assert_eq!(First_Failure(&accepted).rule, "accepted-result-carries-evidence");

    submission.state = SubmissionState::Draft;
    assert_eq!(Validate_Submission(&submission), Vec::new());
}

/// How many variants [`SubmissionKind`] has, which is what its label round trip covers.
const SUBMISSION_KINDS: usize = 3;

fn All_Submission_Kinds() -> [SubmissionKind; SUBMISSION_KINDS]
{
    return [SubmissionKind::FeatureRequest, SubmissionKind::DesignSpec, SubmissionKind::FeatureResult];
}

/// How many variants [`Origin`] has.
const SUBMISSION_ORIGINS: usize = 4;

fn All_Submission_Origins() -> [Origin; SUBMISSION_ORIGINS]
{
    return [Origin::Submitted, Origin::Clarified, Origin::Inferred, Origin::Decided];
}

/// How many variants [`Severity`] has.
const SUBMISSION_SEVERITIES: usize = 2;

fn All_Submission_Severities() -> [Severity; SUBMISSION_SEVERITIES]
{
    return [Severity::Blocking, Severity::NonBlocking];
}

/// How many variants [`SubmissionState`] has.
const SUBMISSION_STATES: usize = 2;

fn All_Submission_States() -> [SubmissionState; SUBMISSION_STATES]
{
    return [SubmissionState::Draft, SubmissionState::Accepted];
}

#[test]
fn Test_Every_Label_Should_Round_Trip_Through_Parse()
{
    for kind in All_Submission_Kinds()
    {
        assert_eq!(SubmissionKind::Parse(kind.Label()), Some(kind));
    }
    for origin in All_Submission_Origins()
    {
        assert_eq!(Origin::Parse(origin.Label()), Some(origin));
    }
    for severity in All_Submission_Severities()
    {
        assert_eq!(Severity::Parse(severity.Label()), Some(severity));
    }
    for state in All_Submission_States()
    {
        assert_eq!(SubmissionState::Parse(state.Label()), Some(state));
    }
}
