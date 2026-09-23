//! What a record establishes, and what a run is allowed to put into one.
//!
//! Driven through [`Recorded_Census`] and [`OccurrenceHistory::Continuity_Of`] directly rather
//! than through a real walk: the three cases this has to tell apart are three *sequences* of
//! runs, and producing a sequence through a real tree would make each assertion depend on which
//! capabilities a machine happens to materialize. `super::super::continuity_tests` is where a
//! real `Run_Gate` proves the wiring.

use super::{Lineage_Key, ObservedRun, OccurrenceHistory, Recorded_Census};
use crate::GateCommand;
use crate::policy::{BaselineAllowance, BaselineDebt, BaselinePolicy, RuleSelector, ScopeSelector};
use nomos_check_orchestration::{CheckOutcome, Claim, Examined, SupportingFactTrail};
use nomos_contracts::{Applicability, EvidenceClass, Finding, GateCategory, RuleId};
use nomos_model::{IdentityTransitionKind, Subject_Of_Path};
use std::path::PathBuf;

/// The rule every finding below belongs to.
const RULE: &str = "single-letter-names";

/// The file every finding below is attributed to.
const SUBJECT_PATH: &str = "src/lib.rs";

/// One file no entry below addresses, so a census can be shown to leave it alone.
const UNADDRESSED_PATH: &str = "src/elsewhere.rs";

/// The sentence one violation describes itself by, held fixed so that a lineage moving means the
/// violation changed rather than the fixture drifting.
const A_VIOLATION: &str = "`x` is a single-letter name outside the local-variable exception";

/// A second violation of the same rule in the same file, so that one can be fixed while the
/// other persists.
const ANOTHER_VIOLATION: &str = "`y` is a single-letter name outside the local-variable exception";

/// The record a repository opts in with, before any run has censused anything.
fn Opted_In() -> OccurrenceHistory
{
    return OccurrenceHistory::default();
}

/// One finding of [`RULE`] against `path`, describing itself by `summary` and found at
/// `location`.
fn Finding_In(path: &str, summary: &str, location: &str) -> Finding
{
    return Finding {
        address: None,
        rule: RuleId::New(RULE),
        subject: Subject_Of_Path(path),
        subject_name: location.to_owned(),
        applicability: Applicability::Supported,
        evidence: EvidenceClass::Derived,
        gate: GateCategory::Blocking,
        summary: summary.to_owned(),
        locations: vec![location.to_owned()],
    };
}

/// [`Finding_In`] over the one file every entry below addresses.
fn Finding_At(summary: &str, location: &str) -> Finding
{
    return Finding_In(SUBJECT_PATH, summary, location);
}

/// A judged outcome holding exactly `findings`, with the complete claim a census requires.
fn Judged(findings: Vec<Finding>) -> CheckOutcome
{
    return CheckOutcome::Judged {
        examined: Examined { files: 1, facts: findings.len() },
        findings,
        claim: Claim::Complete,
        supporting_facts: SupportingFactTrail::New(),
    };
}

/// The baseline every census below is taken against: one unbounded entry over the whole scope,
/// which is the state `OD-GATE-030` says every entry authored before it is in.
fn A_Baseline() -> BaselinePolicy
{
    return BaselinePolicy {
        debt: vec![BaselineDebt {
            rule: RuleId::New(RULE),
            subject: Subject_Of_Path(SUBJECT_PATH),
            rationale: "test fixture".to_owned(),
            allowance: BaselineAllowance::Unbounded,
            declared_path: Some(SUBJECT_PATH.to_owned()),
        }],
    };
}

/// A command selecting everything, which is what a census requires.
fn Whole_Tree() -> GateCommand
{
    return GateCommand { root: PathBuf::from("."), ..Default::default() };
}

/// `history` advanced by one run that observed `findings`, under `command`.
fn Censused(history: &OccurrenceHistory, findings: Vec<Finding>, command: &GateCommand) -> OccurrenceHistory
{
    let baseline = A_Baseline();
    let outcome = Judged(findings);
    let run = ObservedRun { history: Some(history), outcome: &outcome, command, baseline: &baseline };

    return Recorded_Census(&run).expect("this fixture states a record, a judged outcome and an unnarrowed command");
}

/// `history` advanced by one run over the whole tree.
fn Censused_Whole(history: &OccurrenceHistory, findings: Vec<Finding>) -> OccurrenceHistory
{
    return Censused(history, findings, &Whole_Tree());
}

/// The claim the record itself publishes about `finding`'s lineage.
///
/// `None` is a lineage the record holds an entry for and makes no claim about, which is what it
/// says for one the latest state did not observe. That the entry exists at all is asserted by
/// the `expect` rather than returned, because a record that has never heard of the lineage is
/// not a quieter answer than `None` -- it means the census under test never ran.
///
/// Read through the record rather than through `Continuity_Of`, because that function answers
/// *what the counters establish* and this asserts *what a person opening the file is told*.
fn Reported_For(history: &OccurrenceHistory, finding: &Finding) -> Option<IdentityTransitionKind>
{
    let lineage = Lineage_Key(finding);

    return history
        .scopes
        .iter()
        .find_map(|scope| return scope.occurrences.get(&lineage))
        .map(|occurrence| return occurrence.continuity)
        .expect("every caller below names a lineage some state of the record observed");
}

/// The first of the three cases: an occurrence that was there throughout, while an edit
/// elsewhere in the file moved it from one line to another.
#[test]
fn Test_An_Occurrence_Present_At_Every_State_Should_Be_Exact_Continuity()
{
    let first = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let second = Censused_Whole(&first, vec![Finding_At(A_VIOLATION, "src/lib.rs:88")]);

    assert_eq!(
        second.Continuity_Of(&Finding_At(A_VIOLATION, "src/lib.rs:140")),
        Some(IdentityTransitionKind::ExactContinuity),
        "a line moving is not a violation leaving"
    );
}

/// The second: fixed, and then written again. The record saw it go, so the tolerance its entry
/// granted must not reach what came back.
#[test]
fn Test_An_Occurrence_Fixed_And_Written_Again_Should_Be_Recreated()
{
    let first = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let fixed = Censused_Whole(&first, Vec::new());
    let back = Censused_Whole(&fixed, vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);

    let established = back.Continuity_Of(&Finding_At(A_VIOLATION, "src/lib.rs:12"));

    assert_eq!(established, Some(IdentityTransitionKind::Recreated));
    assert!(
        !established.expect("the assertion above established a transition").Is_History_Preserving(),
        "and a recreation is what stops a tolerance carrying over"
    );
}

/// The third: moved within its own file while a neighbour was fixed around it. The neighbour's
/// departure must not read as this one's.
#[test]
fn Test_An_Occurrence_Moved_While_Its_Neighbour_Was_Fixed_Should_Be_Exact_Continuity()
{
    let both = vec![Finding_At(A_VIOLATION, "src/lib.rs:12"), Finding_At(ANOTHER_VIOLATION, "src/lib.rs:20")];
    let first = Censused_Whole(&Opted_In(), both);
    let moved = Censused_Whole(&first, vec![Finding_At(A_VIOLATION, "src/lib.rs:4")]);

    assert_eq!(
        moved.Continuity_Of(&Finding_At(A_VIOLATION, "src/lib.rs:4")),
        Some(IdentityTransitionKind::ExactContinuity),
        "the one that stayed"
    );
    assert_eq!(
        Reported_For(&moved, &Finding_At(ANOTHER_VIOLATION, "src/lib.rs:20")),
        None,
        "and the record makes no claim about the one that left, rather than calling its absence a \
         verdict"
    );
}

/// A record with no state for the scope establishes nothing, which is where every repository
/// starts and is `OD-GATE-030`'s own floor.
#[test]
fn Test_A_Record_With_No_State_Should_Establish_Nothing()
{
    assert_eq!(Opted_In().Continuity_Of(&Finding_At(A_VIOLATION, "src/lib.rs:12")), None);
}

/// A violation the record has never seen is not announced as a return.
///
/// It was not there at the states behind it, and it is here now -- but the lineage may have
/// moved rather than the code, which `OccurrenceLineageId`'s own measurement shows is reachable.
/// Undetermined is the honest answer and a false regression is not.
#[test]
fn Test_A_Lineage_The_Record_Never_Saw_Should_Not_Be_Reported_As_Recreated()
{
    let first = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);

    assert_eq!(first.Continuity_Of(&Finding_At(ANOTHER_VIOLATION, "src/lib.rs:20")), None);
}

/// A run narrowed by `--include` must not census at all.
///
/// The guard this proves is load-bearing rather than defensive: a narrowed run sees part of a
/// tree by design (`OD-GATE-025`), so a census of one would record every scope outside the
/// narrowing as emptied, and the next whole-tree run would report a recreation for each of them.
#[test]
fn Test_A_Scope_Narrowed_Run_Should_Not_Census()
{
    let baseline = A_Baseline();
    let history = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let narrowed = GateCommand { scope: ScopeSelector { include: vec!["src/other".to_owned()], exclude: Vec::new() }, ..Whole_Tree() };
    let outcome = Judged(Vec::new());

    let census = Recorded_Census(&ObservedRun { history: Some(&history), outcome: &outcome, command: &narrowed, baseline: &baseline });

    assert_eq!(census, None, "a partial observation is not a census, and recording one manufactures a recreation");
}

/// A run that deselected the scope's own rule must not census it either -- that rule was never
/// asked, so its scopes coming up empty says nothing.
#[test]
fn Test_A_Run_That_Deselected_The_Rule_Should_Not_Census_Its_Scope()
{
    let history = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let elsewhere = GateCommand { rules: RuleSelector { include: vec![RuleId::New("nesting-depth")] }, ..Whole_Tree() };

    let census = Censused(&history, Vec::new(), &elsewhere);

    assert_eq!(census, history, "the census must leave the scope exactly as it found it");
}

/// A rule whose provider could not look leaves its scopes uncensused for the run.
///
/// The finding here is the shape `Claim_Of` reads as incomplete coverage. Without this guard an
/// unavailable provider would empty every scope that rule addresses, and the next run with the
/// provider back would report each of them recreated.
#[test]
fn Test_A_Rule_That_Could_Not_Look_Should_Not_Census_Its_Scope()
{
    let baseline = A_Baseline();
    let history = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let could_not_look = Finding {
        applicability: Applicability::MissingCapability,
        ..Finding_At("no provider offered the capability this rule requires", "src/lib.rs")
    };
    let outcome = Judged(vec![could_not_look]);
    let command = Whole_Tree();

    let census = Recorded_Census(&ObservedRun { history: Some(&history), outcome: &outcome, command: &command, baseline: &baseline })
        .expect("a judged, unnarrowed run with a record still censuses; the scope inside it is what must be untouched");

    assert_eq!(census, history);
}

/// A tree with no record is never given one, which is the whole of the opt-in.
#[test]
fn Test_A_Run_With_No_Record_Should_Produce_None()
{
    let baseline = A_Baseline();
    let outcome = Judged(vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let command = Whole_Tree();

    let census = Recorded_Census(&ObservedRun { history: None, outcome: &outcome, command: &command, baseline: &baseline });

    assert_eq!(census, None);
}

/// An unjudged run observed nothing, and nothing is not an empty scope.
#[test]
fn Test_An_Unjudged_Run_Should_Not_Census()
{
    let baseline = A_Baseline();
    let history = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);
    let command = Whole_Tree();

    let census =
        Recorded_Census(&ObservedRun { history: Some(&history), outcome: &CheckOutcome::NoSource, command: &command, baseline: &baseline });

    assert_eq!(census, None);
}

/// The record says what it established, in the vocabulary `nomos-model` publishes and in the
/// file a person opens -- including `Unresolved` for the occurrence it cannot place, which is
/// `OD-GATE-030`'s floor written down rather than left to a reader's inference.
///
/// The first run after opting in is the case asserted, because it is the one where a record
/// holding a single state could most easily be made to call itself continuity.
#[test]
fn Test_The_Record_Should_Report_Its_Own_Verdict_For_Every_Lineage_It_Saw()
{
    let first = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);

    let written = serde_json::to_string(&first).expect("a record of owned strings and integers serializes");

    assert_eq!(Reported_For(&first, &Finding_At(A_VIOLATION, "src/lib.rs:12")), Some(IdentityTransitionKind::Unresolved));
    assert!(written.contains("\"continuity\":\"Unresolved\""), "{written}");
    assert!(written.contains("single-letter name"), "a reader must find the sentence, not only a digest: {written}");
}

/// A run that has seen a scope once establishes nothing about it, and specifically does not
/// report the occurrence it just met as having persisted.
#[test]
fn Test_The_First_Census_Of_A_Scope_Should_Establish_Nothing()
{
    let first = Censused_Whole(&Opted_In(), vec![Finding_At(A_VIOLATION, "src/lib.rs:12")]);

    assert_eq!(
        first.Continuity_Of(&Finding_At(A_VIOLATION, "src/lib.rs:12")),
        None,
        "one sighting is not an interval, and a tolerance must not be confirmed by the run that \
         first looked"
    );
}

/// A scope no entry addresses is never censused, whatever the run found there.
#[test]
fn Test_A_Scope_No_Entry_Addresses_Should_Not_Be_Recorded()
{
    let unaddressed = Finding_In(UNADDRESSED_PATH, A_VIOLATION, "src/elsewhere.rs:1");

    let census = Censused_Whole(&Opted_In(), vec![unaddressed]);

    assert_eq!(census.scopes.len(), 1, "only the scope the entry names is censused");
}
