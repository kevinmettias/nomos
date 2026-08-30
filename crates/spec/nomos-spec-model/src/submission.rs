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

// What a submission carries: the value of one field, and where it came from.
mod field_value;
mod origin;

pub use field_value::FieldValue;
pub use origin::Origin;

// What a submission is of, and where it stands.
mod kind;
mod state;

pub use kind::Kind as SubmissionKind;
pub use state::State as SubmissionState;

use crate::DecisionGap;
use crate::Failure;

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
pub fn Validate_Submission(submission: &Submission) -> Vec<Failure>
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
    let entries: Vec<&str> = Entries_Of_Value(&alternatives.value);

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

    for (index, entry) in Entries_Of_Value(&deviations.value).iter().enumerate()
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
    if submission.kind == SubmissionKind::FeatureResult && Has_No_Evidence(submission)
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

/// Whether an accepted submission still has no evidence recorded against it.
fn Has_No_Evidence(submission: &Submission) -> bool
{
    return submission.Current("evidence").is_none();
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
        if current.origin.Can_Satisfy_Acceptance()
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
    for question in Open_Blocking_Gap_Questions(submission)
    {
        failures.push(Blocking_Gap_Failure(question));
    }
}

/// The blocking gaps still open, by question — deduplicated so a question shared by more
/// than one gap is not reported once per gap.
fn Open_Blocking_Gap_Questions(submission: &Submission) -> std::collections::BTreeSet<&str>
{
    use crate::Severity;

    return submission
        .gaps
        .iter()
        .filter(|gap| return gap.severity == Severity::Blocking && gap.Is_Open())
        .map(|gap| return gap.question.as_str())
        .collect();
}

/// The failure reported for one open blocking gap.
fn Blocking_Gap_Failure(question: &str) -> Failure
{
    return Failure {
        field: "gaps".to_owned(),
        rule: "no-open-blocking-gap".to_owned(),
        remedy: format!(
            "close `{question}` by citing a governing record or a recorded decision, or \
             submit as a draft; supplying the value it blocks does not close it, because \
             nothing would record that the question was answered"
        ),
    };
}

/// The entries of a multi-entry value: its non-empty lines.
fn Entries_Of_Value(value: &str) -> Vec<&str>
{
    return value
        .lines()
        .map(str::trim)
        .filter(|line| return !line.is_empty())
        .collect();
}

#[cfg(test)]
mod tests;

// `tests` (above) is its own file (`submission/tests.rs`) and is exercised through the
// public rule set. This module stays inline in this file so a test naming `Current`,
// `Required_Fields` or `Validate_Submission` sits beside the declaration it addresses.
#[cfg(test)]
mod address_tests
{
    use super::*;
    use crate::Origin;

    fn Minimal_Submission(values: Vec<FieldValue>) -> Submission
    {
        return Submission {
            id: "FR-001".to_owned(),
            kind: SubmissionKind::FeatureRequest,
            form_contract_version: 1,
            state: SubmissionState::Draft,
            submitted_by: "kevin".to_owned(),
            submitted_through: "cli".to_owned(),
            values,
            gaps: Vec::new(),
        };
    }

    #[test]
    fn Test_Current_Should_Be_The_Last_Value_Given_For_A_Field()
    {
        let submission = Minimal_Submission(vec![
            FieldValue {
                field: "title".to_owned(),
                value: "first".to_owned(),
                origin: Origin::Submitted,
            },
            FieldValue {
                field: "title".to_owned(),
                value: "second".to_owned(),
                origin: Origin::Submitted,
            },
        ]);

        assert_eq!(submission.Current("title").map(|v| return v.value.as_str()), Some("second"));
        assert!(submission.Current("missing").is_none());
    }

    #[test]
    fn Test_Required_Fields_Should_Start_With_The_Universal_Title_Field()
    {
        let submission = Minimal_Submission(Vec::new());

        assert_eq!(submission.Required_Fields().first(), Some(&"title"));
    }

    #[test]
    fn Test_Validate_Submission_Should_Refuse_An_Empty_Submission()
    {
        let submission = Minimal_Submission(Vec::new());

        assert!(!Validate_Submission(&submission).is_empty());
    }
}
