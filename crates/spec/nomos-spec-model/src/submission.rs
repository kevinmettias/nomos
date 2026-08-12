//! What a born-structured submission is, and the rule set that refuses one.
//!
//! `OD-SPEC-008` made `FeatureRequest`, `DesignSpec` and `FeatureResult` born structured,
//! `OD-SPEC-009` put every one of them through a single accept function with transports
//! forbidden from validating, and `OD-SPEC-010` stated the rules. This module is the second
//! and third of those as types: the value a transport constructs, and the check that decides
//! whether it may become durable.
//!
//! The rules live here rather than in the store because they are properties of a submission
//! and not of a door — the same rules whichever transport built it and whichever table it
//! lands in. What is *not* here is rule 4, which resolves `implements` and `answers` against
//! other rows; a rule that reads the store cannot be a pure function of one submission, and
//! it runs where the store is.

use std::collections::BTreeSet;

use crate::decision_gap::DecisionGap;
use crate::failure::Failure;
use crate::field_value::FieldValue;
use crate::severity::Severity;
use crate::submission_kind::SubmissionKind;
use crate::submission_state::SubmissionState;

/// What a transport hands to the accept function.
///
/// `OD-SPEC-009` fixes three properties of this seam: it carries what was submitted verbatim
/// separately from anything the transport supplied — which is what [`Origin`] on every value
/// is for — it names the form contract version it was constructed against, and it is complete
/// or refused, never partially stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submission
{
    /// Identity, assigned by the submitter rather than by the store.
    pub id: String,
    /// Which of the three kinds.
    pub kind: SubmissionKind,
    /// The set of questions this was constructed against.
    ///
    /// One version and not two: the form contract *is* the schema of a submission, so this
    /// discharges both `ARC-SPECDB-002`'s schema-version obligation and `OD-SPEC-009`'s
    /// contract-version one. Two fields would be two names for one fact.
    pub form_contract_version: u32,
    /// Draft or accepted.
    pub state: SubmissionState,
    /// Who submitted it.
    pub submitted_by: String,
    /// Which surface they came through.
    pub submitted_through: String,
    /// Every value of every field, in the order they were given.
    pub values: Vec<FieldValue>,
    /// The decisions it needs that nobody has taken.
    pub gaps: Vec<DecisionGap>,
}

impl Submission
{
    /// The current reading of `field`, which is its last value.
    #[must_use]
    pub fn Current(&self, field: &str) -> Option<&FieldValue>
    {
        return self
            .values
            .iter()
            .rev()
            .find(|candidate| return candidate.field == field);
    }

    /// Every field this submission must carry, universal and per-kind.
    ///
    /// `title` is the universal one that is a value rather than a column: it is a sentence
    /// somebody wrote, and somebody may rewrite it. The rest of `OD-SPEC-010`'s universal
    /// table — `id`, `kind`, `form_contract_version`, `provenance`, `state` — are fields of
    /// this type, so a submission that omitted one could not be constructed.
    #[must_use]
    pub fn Required_Fields(&self) -> Vec<&'static str>
    {
        let mut required = vec!["title"];
        required.extend_from_slice(self.kind.Required_Fields());
        return required;
    }
}

/// Every rule `submission` fails that can be decided without reading another row.
///
/// Empty means nothing here refuses it. Rule 4 — `implements` resolving to an accepted design
/// and `answers` to an accepted request — is deliberately absent, because it reads a second
/// row and cannot be a function of one submission; it runs where the store is.
#[must_use]
pub fn Validate(submission: &Submission) -> Vec<Failure>
{
    let mut failures = Vec::new();

    Check_Required_Fields(submission, &mut failures);
    Check_Alternatives(submission, &mut failures);
    Check_Deviations(submission, &mut failures);

    if submission.state == SubmissionState::Accepted
    {
        Check_Acceptance(submission, &mut failures);
    }

    return failures;
}

/// Whether this field's requiredness turns on state rather than on kind.
///
/// Exactly one field does, and `OD-SPEC-010` says so in as many words: `evidence` is required
/// for an accepted result and not for a draft one, because evidence is the thing acceptance is
/// acceptance *of*. It stays in [`SubmissionKind::Required_Fields`] because it is still a field
/// of its kind — the origin rule below reads that list and must see it — and the completeness
/// check skips it in `draft`, which is the whole of what rule 5 asks for.
fn Is_Conditional_On_State(kind: SubmissionKind, field: &str) -> bool
{
    return matches!(kind, SubmissionKind::FeatureResult) && field == "evidence";
}

/// Every required field carries a value.
///
/// A required field is satisfied by content or by an explicit statement that there is none,
/// and never by emptiness. The store refuses an empty value at the column, so a stated
/// absence is a value like `none` and an unlooked-at field is no value at all.
fn Check_Required_Fields(submission: &Submission, failures: &mut Vec<Failure>)
{
    for field in submission.Required_Fields()
    {
        // Skipped in both states rather than only in `draft`, so that one rule has one owner:
        // `Check_Acceptance` is where rule 5 lives, and a field failing two rules for one
        // absence would report the same hole twice.
        if Is_Conditional_On_State(submission.kind, field)
        {
            continue;
        }

        if submission.Current(field).is_none()
        {
            failures.push(Failure {
                field: field.to_owned(),
                rule: "required-field".to_owned(),
                remedy: format!(
                    "give {field} a value; if there is genuinely nothing, say so explicitly \
                     rather than leaving it out, because an empty field means nobody looked"
                ),
            });
        }
    }
}

/// `alternatives` carries at least two entries and `selected` names one of them.
///
/// Entries are the lines of the value. A field is a supersession sequence rather than a
/// multiset — a second row for one field means a later value replaced an earlier one for
/// reading — so several alternatives cannot be several rows without saying that four of them
/// were superseded. `OD-SPEC-013` settles it in the layout; this is the same fact read back.
fn Check_Alternatives(submission: &Submission, failures: &mut Vec<Failure>)
{
    if submission.kind != SubmissionKind::DesignSpec
    {
        return;
    }
    let Some(alternatives) = submission.Current("alternatives")
    else
    {
        return;
    };
    let entries: Vec<&str> = Entries(&alternatives.value);

    Check_At_Least_Two_Were_Weighed(&entries, failures);

    let Some(selected) = submission.Current("selected")
    else
    {
        return;
    };

    Check_The_Selection_Was_Considered(&entries, &selected.value, failures);
}

/// The fewest alternatives that make a choice a choice.
const WEIGHED: usize = 2;

/// A design with one alternative did not choose, it recorded.
fn Check_At_Least_Two_Were_Weighed(entries: &[&str], failures: &mut Vec<Failure>)
{
    if entries.len() >= WEIGHED
    {
        return;
    }

    failures.push(Failure {
        field: "alternatives".to_owned(),
        rule: "at-least-two-alternatives".to_owned(),
        remedy: "give at least two entries, one per line; a design with one alternative \
                 did not choose, it recorded. `do nothing` is an admissible entry"
            .to_owned(),
    });
}

/// A chosen option absent from the options considered means one of the two is false, and a
/// reader cannot tell which.
fn Check_The_Selection_Was_Considered(entries: &[&str], selected: &str, failures: &mut Vec<Failure>)
{
    if entries.iter().any(|entry| return entry.trim() == selected.trim())
    {
        return;
    }

    failures.push(Failure {
        field: "selected".to_owned(),
        rule: "selected-names-an-alternative".to_owned(),
        remedy: format!(
            "make `selected` one of the {} entry(ies) in `alternatives` exactly; a chosen \
             option absent from the options considered means one of the two is false, and \
             a reader cannot tell which",
            entries.len()
        ),
    });
}

/// Each entry in `deviations` names the design clause it departs from.
///
/// A deviation from nothing in particular cannot be reviewed and cannot be closed. An entry
/// names its clause by carrying a `:` — the clause, then what departed from it.
fn Check_Deviations(submission: &Submission, failures: &mut Vec<Failure>)
{
    if submission.kind != SubmissionKind::FeatureResult
    {
        return;
    }

    let Some(deviations) = submission.Current("deviations")
    else
    {
        return;
    };

    for (index, entry) in Entries(&deviations.value).iter().enumerate()
    {
        Check_One_Deviation(entry, index, failures);
    }
}

/// An entry names its clause by carrying a `:` — the clause, then what departed from it.
fn Check_One_Deviation(entry: &str, index: usize, failures: &mut Vec<Failure>)
{
    let names_a_clause = entry
        .split_once(':')
        .is_some_and(|(clause, _)| return !clause.trim().is_empty());
    if names_a_clause
    {
        return;
    }

    failures.push(Failure {
        field: "deviations".to_owned(),
        rule: "deviation-names-its-clause".to_owned(),
        remedy: format!(
            "entry {} reads `{entry}`; write it as `<design clause>: <what departed>`, \
             because a deviation from nothing in particular cannot be reviewed or \
             closed",
            index.saturating_add(1)
        ),
    });
}

/// The rules that apply only to an accepted submission.
///
/// `draft` weakens which rules apply and never weakens whether they are checked — every rule
/// above runs in both states. These are the three that acceptance adds.
fn Check_Acceptance(submission: &Submission, failures: &mut Vec<Failure>)
{
    if submission.kind == SubmissionKind::FeatureResult && Lacks_Evidence(submission)
    {
        failures.push(Failure {
            field: "evidence".to_owned(),
            rule: "accepted-result-carries-evidence".to_owned(),
            remedy: "record what was run and what it exited; evidence is the thing acceptance \
                     is acceptance of, and it is the one field whose requiredness turns on \
                     state rather than on kind"
                .to_owned(),
        });
    }

    Check_Nothing_Required_Was_Inferred(submission, failures);
    Check_No_Blocking_Gap_Is_Open(submission, failures);
}

/// A system that accepts its own inferences accepts them as what somebody wanted.
fn Check_Nothing_Required_Was_Inferred(submission: &Submission, failures: &mut Vec<Failure>)
{
    for field in submission.Required_Fields()
    {
        let Some(current) = submission.Current(field)
        else
        {
            continue;
        };
        if current.origin.Satisfies_Acceptance()
        {
            continue;
        }

        failures.push(Failure {
            field: field.to_owned(),
            rule: "accepted-values-are-not-inferred".to_owned(),
            remedy: format!(
                "the current value of {field} has origin `{}`, which is machinery's guess; \
                 acceptance needs it submitted, clarified or decided, because a system that \
                 accepts its own inferences accepts them as what somebody wanted",
                current.origin.Label()
            ),
        });
    }
}

/// Supplying the value a gap blocks does not close it, because nothing would record that the
/// question was answered.
fn Check_No_Blocking_Gap_Is_Open(submission: &Submission, failures: &mut Vec<Failure>)
{
    let open: BTreeSet<&str> = submission
        .gaps
        .iter()
        .filter(|gap| return gap.severity == Severity::Blocking && gap.Is_Open())
        .map(|gap| return gap.question.as_str())
        .collect();

    for question in open
    {
        failures.push(Failure {
            field: "gaps".to_owned(),
            rule: "no-open-blocking-gap".to_owned(),
            remedy: format!(
                "close `{question}` by citing a governing record or a recorded decision, or \
                 submit as a draft; supplying the value it blocks does not close it, because \
                 nothing would record that the question was answered"
            ),
        });
    }
}

/// Whether an accepted submission still has no evidence recorded against it.
fn Lacks_Evidence(submission: &Submission) -> bool
{
    return submission.Current("evidence").is_none();
}

/// The entries of a multi-entry value: its non-empty lines.
fn Entries(value: &str) -> Vec<&str>
{
    return value
        .lines()
        .map(str::trim)
        .filter(|line| return !line.is_empty())
        .collect();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::origin::Origin;

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

        assert_eq!(Validate(&submission), Vec::new());
    }

    #[test]
    fn Test_A_Refusal_Should_Name_Every_Missing_Field_Rather_Than_The_First()
    {
        let submission = Request(SubmissionState::Draft, vec![Value(
            "title",
            "something",
            Origin::Submitted,
        )]);

        let failures = Validate(&submission);
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
        let draft = Validate(&as_draft);
        let accepted = Validate(&as_accepted);

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
        assert_eq!(Validate(&submission), Vec::new());
    }

    #[test]
    fn Test_An_Inferred_Value_Should_Be_Readable_And_Never_Sufficient()
    {
        let mut values = Complete_Request_Values();
        let inferred = Value("goal", "guessed from the title", Origin::Inferred);
        values.push(inferred);

        let as_draft = Request(SubmissionState::Draft, values.clone());
        let as_accepted = Request(SubmissionState::Accepted, values);
        let draft = Validate(&as_draft);
        let accepted = Validate(&as_accepted);

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

        assert_eq!(Validate(&submission), Vec::new());

        submission.state = SubmissionState::Accepted;
        let failures = Validate(&submission);

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

        assert_eq!(Validate(&submission), Vec::new());
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

        assert_eq!(Validate(&submission), Vec::new());
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
        let failures = Validate(&design);

        assert_eq!(failures.len(), 1);
        assert_eq!(First(&failures).rule, "at-least-two-alternatives");
    }

    #[test]
    fn Test_Do_Nothing_Should_Be_An_Admissible_Alternative()
    {
        let design = Design("a new verb\ndo nothing", "a new verb");

        assert_eq!(Validate(&design), Vec::new());
    }

    #[test]
    fn Test_A_Selected_Option_Absent_From_The_Alternatives_Should_Be_Refused()
    {
        let design = Design("a new verb\ndo nothing", "a third thing");
        let failures = Validate(&design);

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
        let failures = Validate(&submission);

        assert_eq!(failures.len(), 1);
        assert_eq!(First(&failures).rule, "deviation-names-its-clause");
    }

    #[test]
    fn Test_A_Deviation_Naming_Its_Clause_Should_Pass()
    {
        let submission =
            Result_Submission("section 3: used one table, not two", Some("cargo test: 0"));

        assert_eq!(Validate(&submission), Vec::new());
    }

    #[test]
    fn Test_Evidence_Should_Be_Required_For_An_Accepted_Result_And_Not_For_A_Draft()
    {
        let mut submission = Result_Submission("section 3: a departure", None);

        let accepted = Validate(&submission);
        assert_eq!(accepted.len(), 1);
        assert_eq!(First(&accepted).rule, "accepted-result-carries-evidence");

        submission.state = SubmissionState::Draft;
        assert_eq!(Validate(&submission), Vec::new());
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
}
