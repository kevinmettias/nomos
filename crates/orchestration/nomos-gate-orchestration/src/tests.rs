//! What this crate promises today: a real rule registry, reported back whole, and a real
//! disposition reduction over an already-judged list of findings.

use crate::{
    AdoptionPolicy, BaselineDebt, BaselinePolicy, CoveragePolicy, Disposition_Of_Findings, Explain_Gate, Explanation, FindingQuery, GateCommand,
    GateEnvironment, GateOutcome, GateRunOutcome, GateRunResult, RuleCalibration, RuleSelector, Run, Run_Gate, ScopeSelector, Suppression,
    SuppressionDisposition, SuppressionPolicy,
};
use nomos_check_orchestration::{Claim, CheckOutcome};
use nomos_contracts::{
    Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, RunId, SubjectId,
};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::{StdEnvironment, StdFileSystem, StdProcessLauncher};
use nomos_rules::{
    SourceFile, COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION,
    DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION,
    NAMING_CONVENTION, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
use nomos_workspace::BuildVariant;
use std::path::PathBuf;

fn Command() -> GateCommand
{
    return GateCommand { root: PathBuf::from("."), ..Default::default() };
}

/// [`Command`] over `root`, everything else select-everything.
fn Command_At(root: PathBuf) -> GateCommand
{
    return GateCommand { root, ..Default::default() };
}

/// [`Command_At`] `root`, with `suppression` as the whole suppression policy and every rule
/// still selected -- the shape both a `run` and an `explain` fixture build once they have a
/// real finding to suppress.
fn Command_With_Suppression(root: PathBuf, suppression: Suppression) -> GateCommand
{
    return GateCommand {
        suppressions: SuppressionPolicy { suppressions: vec![suppression] },
        ..Command_At(root)
    };
}

/// A [`Suppression`] matching `finding` exactly, with a disposition, rationale and owner
/// fixed for every fixture that reaches for one -- what a test needs is that it addresses a
/// specific real finding, never what the suppression itself says.
fn Suppression_Of(finding: &Finding) -> Suppression
{
    return Suppression {
        rule: finding.rule.clone(),
        subject: finding.subject,
        disposition: SuppressionDisposition::FalsePositiveDisposition,
        rationale: "test fixture".to_owned(),
        owner: "test".to_owned(),
    };
}

/// [`Command_At`] `root`, with `debt` as the whole baseline policy and every rule still
/// selected -- the shape both a `run` and an `explain` fixture build once they have a real
/// finding to baseline.
fn Command_With_Baseline(root: PathBuf, debt: BaselineDebt) -> GateCommand
{
    return GateCommand { baseline: BaselinePolicy { debt: vec![debt] }, ..Command_At(root) };
}

/// A [`BaselineDebt`] matching `finding` exactly, with a rationale fixed for every fixture
/// that reaches for one -- the same "what a test needs is that it addresses a specific real
/// finding" discipline [`Suppression_Of`] already keeps.
fn Baseline_Of(finding: &Finding) -> BaselineDebt
{
    return BaselineDebt {
        rule: finding.rule.clone(),
        subject: finding.subject,
        rationale: "test fixture".to_owned(),
    };
}

/// [`Command_At`] `root`, with `calibration` as the whole adoption policy and every rule
/// still selected -- the shape both a `run` and an `explain` fixture build once they have a
/// real finding whose rule to calibrate.
fn Command_With_Calibration(root: PathBuf, calibration: RuleCalibration) -> GateCommand
{
    return GateCommand { adoption: AdoptionPolicy { calibrated: vec![calibration] }, ..Command_At(root) };
}

/// A [`RuleCalibration`] matching `finding`'s rule exactly, with a rationale fixed for every
/// fixture that reaches for one -- the same "what a test needs is that it addresses a
/// specific real finding" discipline [`Suppression_Of`] and [`Baseline_Of`] both keep.
fn Calibration_Of(finding: &Finding) -> RuleCalibration
{
    return RuleCalibration { rule: finding.rule.clone(), rationale: "test fixture".to_owned() };
}

/// Judges one fresh call of `source` with every policy empty, and returns its one real
/// blocking finding -- the shared setup a suppression and a baseline fixture both need
/// before either can address a specific finding with its own policy.
fn One_Real_Blocking_Finding(source: impl Fn() -> SourceFile) -> Finding
{
    let unmatched = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), Test_Run_Id());

    return unmatched
        .findings.blocking_findings
        .into_iter()
        .next()
        .expect("this fixture must produce one real blocking finding");
}

/// Asserts `result` reports no finding able to fail the build, and that `tolerated` --
/// whichever of `result.findings.suppressed_findings`/`result.findings.baselined_findings` the caller's own
/// policy populated -- is not empty, so the one real finding a suppression or a baseline
/// fixture produces is visible somewhere rather than silently disappearing.
fn Assert_Tolerated_Not_Blocking(result: &GateRunResult, tolerated: &[Finding])
{
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty(), "{:?}", result.findings.blocking_findings);
    assert!(!tolerated.is_empty(), "the tolerated finding must still be visible");
}

/// Answers `query` against one fresh call of `source` with every policy empty, and returns
/// the finding it found -- the shared setup a suppression and a baseline explain fixture
/// both need before either can address that finding with its own policy.
fn Real_Finding_For(query: &FindingQuery, source: impl Fn() -> SourceFile) -> Finding
{
    let unmatched = Explain_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), query);
    let Explanation::Found { finding, .. } = unmatched.explanation
    else
    {
        panic!("this fixture must produce the finding the query names");
    };

    return *finding;
}

/// The source, query and real finding a suppression and a baseline "applies" fixture both
/// build identically, before either addresses that finding with its own policy.
fn Explain_Applies_Fixture() -> (impl Fn() -> SourceFile, FindingQuery, Finding)
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
    let real_finding = Real_Finding_For(&query, source);

    return (source, query, real_finding);
}

/// Destructures `explanation`'s `Found` variant into the four fields a calibration, a
/// suppression and a baseline explain fixture each read a different one of, or panics naming
/// what every fixture that reaches this helper has already asserted -- the explain-side
/// counterpart to [`Judged_Findings`].
fn Explained_Found(explanation: Explanation) -> (bool, Option<RuleCalibration>, Option<Suppression>, Option<BaselineDebt>)
{
    let Explanation::Found { would_block, calibrated_by, suppressed_by, baselined_by, .. } = explanation
    else
    {
        panic!("this fixture must still produce the finding the query names");
    };

    return (would_block, calibrated_by, suppressed_by, baselined_by);
}

/// The findings a judged `check_outcome` carries, or a panic naming what every fixture that
/// reaches this helper has already asserted -- `Judged`, checked once here rather than
/// re-destructured at each call site.
fn Judged_Findings(check_outcome: &CheckOutcome) -> &[Finding]
{
    let CheckOutcome::Judged { findings, .. } = check_outcome
    else
    {
        panic!("expected a judged check outcome");
    };

    return findings;
}

fn Source(path: &str, text: &str) -> SourceFile
{
    return SourceFile::New(path, Subject_Of_Path(path), text);
}

fn Test_Variant() -> BuildVariant
{
    return BuildVariant::New("test-target", "test-profile", "test-toolchain", std::iter::empty::<String>());
}

/// A fixed `RunId` for tests that judge a run's findings and disposition, not its identity.
/// `Fresh_Run_Id` has its own tests for that.
fn Test_Run_Id() -> RunId
{
    return RunId::From_Digest(Digest128::From_Bytes([0; Digest128::BYTE_LENGTH]));
}

/// This repository's own real root -- [`Run_Gate`]'s dependency step, through
/// `nomos_check_orchestration::Run`, runs `cargo metadata` against it regardless of what
/// sources a test hands in. Real on purpose, the same choice `nomos-check-orchestration`'s
/// own tests already make.
fn Repository_Root() -> PathBuf
{
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    return manifest
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .map(PathBuf::from)
        .expect("this crate sits three levels below the workspace root");
}

/// The whole plan, by identity rather than by length, and against the list `Run` actually
/// composes rather than one written out here.
///
/// A count agrees with itself: this registry composed two of the three shipped rules until
/// `P13-GATE-REGISTRY-THIRD-RULE`, three of four until `P13-CONTROLFLOW-REACHABILITY-WIRE`,
/// four of five until `OD-GATE-019-REGISTRY-COHERENCE-A-3`, five of eight until
/// `OD-GATE-019-REGISTRY-COHERENCE-B-4`, and eight of fifty-six until
/// `P35-GATE-020-REGISTRY-WHOLE`. An assertion on `plan.rules.len()` would have been green
/// throughout.
///
/// So would the hand-written list of eight identifiers this replaced. That is the part
/// `OD-GATE-020` named: a hand-written expectation checked against a hand-written
/// registration is two hand-written artifacts agreeing with each other, and neither says
/// anything about `Run`. [`nomos_check_orchestration::Composed_Rules`] is the authority both
/// now answer to, read off the same array literal `Run` executes.
#[test]
fn Test_Registered_Should_Compose_Every_Rule_A_Check_Run_Composes()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let mut planned: Vec<RuleId> = plan.rules.iter().map(|offer| return offer.rule.clone()).collect();
    let mut composed = nomos_check_orchestration::Composed_Rules();
    planned.sort();
    composed.sort();

    assert_eq!(
        planned, composed,
        "the plan a caller reads must name the rules a run would judge by, or it is a plan \
         smaller than the run it describes"
    );
}

/// `Check_Unread_Reaches_A_Finding` cites a versioned record, the identical shape
/// `Check_Dependency_Direction`'s own citation test asserts above.
/// `tests/contract/tests/rule_contract_citation.rs` is what keeps that citation honest against
/// `OD-RULES-008`'s own front matter; this asserts the offer carries it at all.
#[test]
fn Test_The_Reachability_Rule_Should_Carry_Its_Real_Contract_Citation()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let reachability = plan
        .rules
        .iter()
        .find(|offer| return offer.rule == RuleId::New(UNREAD_REACHES_FINDING))
        .expect("the reachability rule must be registered");

    assert_eq!(reachability.contract_record, UNREAD_REACHES_FINDING_CONTRACT_RECORD);
    assert_eq!(
        reachability.contract_record_version,
        UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION
    );
}

/// `Check_Dependency_Direction` cites a versioned record, unlike `Check_Naming_Convention`'s
/// sentinel, so its offer carries the real citation rather than a placeholder.
/// `tests/contract/tests/rule_contract_citation.rs` is what keeps that citation honest against
/// `OD-RULES-003`'s own front matter; this asserts the offer carries it at all.
#[test]
fn Test_The_Dependency_Rule_Should_Carry_Its_Real_Contract_Citation()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let dependency = plan
        .rules
        .iter()
        .find(|offer| return offer.rule == RuleId::New(DEPENDENCY_DIRECTION))
        .expect("the dependency rule must be registered");

    assert_eq!(dependency.contract_record, DEPENDENCY_CONTRACT_RECORD);
    assert_eq!(dependency.contract_record_version, DEPENDENCY_CONTRACT_RECORD_VERSION);
}

#[test]
fn Test_The_Mirror_Rule_Should_Carry_Its_Real_Contract_Citation()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let mirror = plan
        .rules
        .iter()
        .find(|offer| return offer.rule == RuleId::New(COMPLETENESS_MIRROR))
        .expect("the mirror rule must be registered");

    assert_eq!(mirror.contract_record, CONTRACT_RECORD);
    assert_eq!(mirror.contract_record_version, CONTRACT_RECORD_VERSION);
}

/// `Check_Naming_Convention` has no versioned record to cite -- `naming.rs`'s own "why this
/// has no `CONTRACT_RECORD`" section says its contract is `README.md` prose. This is the
/// sentinel this crate's own composition documents, not a claim about a real version.
#[test]
fn Test_The_Naming_Rule_Should_Cite_Its_Record_Less_Contract()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let naming = plan
        .rules
        .iter()
        .find(|offer| return offer.rule == RuleId::New(NAMING_CONVENTION))
        .expect("the naming rule must be registered");

    assert_eq!(naming.contract_record, "README.md");
    assert_eq!(naming.contract_record_version, 0);
}

/// This increment does not select by scope: two different roots must plan identically, so a
/// caller cannot mistake this for a filtered answer it does not yet give.
#[test]
fn Test_Run_Should_Plan_Identically_Regardless_Of_Root()
{
    let GateOutcome::Planned(here) = Run(&Command_At(PathBuf::from(".")))
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };
    let GateOutcome::Planned(elsewhere) = Run(&Command_At(PathBuf::from("elsewhere")))
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    assert_eq!(here, elsewhere, "root is not read yet, so the plan must not depend on it");
}

/// One finding, built so `gate` and `applicability` are the only knobs a caller of
/// [`Disposition_Of_Findings`] cares about -- everything else here is filler a reader can ignore.
fn Finding_With(gate: GateCategory, applicability: Applicability) -> Finding
{
    return Finding {
        rule: RuleId::New(COMPLETENESS_MIRROR),
        subject: SubjectId::From_Digest(Digest128::From_Bytes([7; Digest128::BYTE_LENGTH])),
        subject_name: "Example".to_owned(),
        applicability,
        evidence: EvidenceClass::Derived,
        gate,
        summary: "example".to_owned(),
        locations: vec!["a.rs".to_owned()],
    };
}

/// No findings at all is `Passed`, the same "an empty judgment is clean, not unknown"
/// reading `nomos_check_orchestration::Claim_Of(&[])` already gives `Claim::Complete`.
#[test]
fn Test_No_Findings_Should_Pass()
{
    assert_eq!(Disposition_Of_Findings(&[]), GateRunOutcome::Passed);
}

/// A finding that cannot fail a build -- advisory, unreachable, or review -- must not flip
/// the disposition, the whole reason `Finding::Can_Fail_A_Build` exists rather than a bare
/// "any finding at all" check.
#[test]
fn Test_A_Non_Blocking_Finding_Should_Pass()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
        Finding_With(GateCategory::Unreachable, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::NotApplicable),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Passed);
}

/// The one condition that must flip the disposition: a rule whose enforcer is honored,
/// judging a subject it actually reached.
#[test]
fn Test_A_Blocking_Finding_Should_Fail()
{
    let findings = vec![Finding_With(GateCategory::Blocking, Applicability::Supported)];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}

/// Any blocking finding fails the run, even beside findings that would not have.
#[test]
fn Test_One_Blocking_Finding_Among_Many_Should_Fail()
{
    let findings = vec![
        Finding_With(GateCategory::Advisory, Applicability::Supported),
        Finding_With(GateCategory::Blocking, Applicability::Supported),
        Finding_With(GateCategory::Review, Applicability::Supported),
    ];

    assert_eq!(Disposition_Of_Findings(&findings), GateRunOutcome::Failed);
}

/// A root that was never walked -- the composition root's own `None`, the same case
/// `crates/host/nomos-cli/src/gate/run.rs` currently assigns `CheckOutcome::Unreadable` for
/// by hand. [`Run_Gate`] must make the identical assignment, since this crate now performs
/// that composition too.
#[test]
fn Test_Judged_Sources_Should_Report_Unreadable_For_An_Unwalked_Root()
{
    let result = Run_Gate(None, GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::Unreadable));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// A directory that was walked and held nothing -- `Some(Vec::new())` -- is a different
/// claim than a root nobody could walk at all, and must not collapse into the same
/// variant.
#[test]
fn Test_A_Walk_That_Found_No_Source_Should_Be_Indeterminate()
{
    let result = Run_Gate(Some(Vec::new()), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// A clean source, judged through the real `nomos_check_orchestration::Run` this crate now
/// calls directly, must pass -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings` already proves clean
/// under all four shipped rules.
#[test]
fn Test_A_Clean_Source_Should_Pass()
{
    let sources = vec![Source("a.rs", "pub fn Ok()\n{\n}\n")];

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty(), "{:?}", result.findings.blocking_findings);
}

/// A phantom mirror -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete` uses -- must reach [`Run_Gate`]
/// as a real blocking finding and flip the disposition, proving the reduction this crate now
/// owns runs over `nomos_check_orchestration::Run`'s real output rather than a fixture typed
/// to look like it.
#[test]
fn Test_Run_Gate_Should_Fail_On_A_Blocking_Finding()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n",
    )];

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.findings.blocking_findings.is_empty());
    assert!(result
        .findings.blocking_findings
        .iter()
        .all(|finding| return finding.gate == GateCategory::Blocking));
}

/// [`ScopeSelector`] excludes the one source a walk found, so the run reports
/// `CheckOutcome::NoSource` -- the same answer an empty walk gives, because "scoped to
/// nothing" and "found nothing" still mean the same thing to a caller.
///
/// Unchanged by `OD-GATE-025`, and worth saying why, because that record moved the scope off
/// the source set everywhere else. The judging now happens over the whole walk and is
/// discarded here rather than never running; what a caller is told is identical, which is
/// the point. A mistyped `--include` must not read as a clean pass.
#[test]
fn Test_A_Scoped_Out_Source_Should_Not_Be_Judged()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n",
    )];
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty());
}

/// [`RuleSelector`] excludes the rule behind the one blocking finding this fixture would
/// otherwise produce: per `OD-GATE-017`, `Run` itself now skips a deselected rule's own
/// computation, so the finding never exists at all -- real selection of what runs, not only
/// of what a disposition later discards.
#[test]
fn Test_A_Deselected_Rules_Finding_Should_Not_Exist()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n",
    )];
    let command = GateCommand {
        rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.findings.blocking_findings.is_empty());
    assert!(
        Judged_Findings(&result.check_outcome)
            .iter()
            .all(|finding| return finding.rule != RuleId::New(COMPLETENESS_MIRROR)),
        "the deselected rule was never asked to run, so its finding must not exist at all"
    );
}

/// A [`Suppression`] matching the one blocking finding this fixture produces: the run still
/// judges the source and `check_outcome` still carries the finding in full, and it now also
/// appears in `suppressed_findings` rather than `blocking_findings` -- suppressed, not
/// silenced.
///
/// The suppression's `subject` is read off a real, unsuppressed run first, rather than
/// recomputed from the file's own path: `Check_Completeness_Mirrors` addresses a finding by
/// the mirrored item's own subject, not the file's -- `Subject_Of_Path` alone does not name
/// it, and this test does not need to know that addressing scheme to prove suppression
/// works over whatever subject a real finding actually carries.
#[test]
fn Test_A_Suppressed_Finding_Should_Not_Block()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    Assert_Tolerated_Not_Blocking(&result, &result.findings.suppressed_findings);
    assert!(
        Judged_Findings(&result.check_outcome)
            .iter()
            .any(|finding| return finding.rule == RuleId::New(COMPLETENESS_MIRROR)),
        "the suppressed finding must still be judged and carried in check_outcome"
    );
}

/// A [`BaselineDebt`] matching the one blocking finding this fixture produces: the run still
/// judges the source and `check_outcome` still carries the finding in full, and it now also
/// appears in `baselined_findings` rather than `blocking_findings` -- tolerated, not
/// silenced. Mirrors [`Test_A_Suppressed_Finding_Should_Not_Block`] for the second of
/// `Run_Gate`'s two policies.
#[test]
fn Test_A_Baselined_Finding_Should_Not_Block()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Baseline(Repository_Root(), Baseline_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    Assert_Tolerated_Not_Blocking(&result, &result.findings.baselined_findings);
}

/// A finding matched by both a [`Suppression`] and a [`BaselineDebt`] reports as suppressed,
/// not baselined -- `Run_Gate` checks suppression first, so the two lists never double-count
/// the same finding, and this is the one case a passing "not blocking" assertion alone would
/// not catch: `blocking_findings` empty is also true if the ordering were reversed.
#[test]
fn Test_A_Suppressed_And_Baselined_Finding_Should_Report_As_Suppressed()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(!result.findings.suppressed_findings.is_empty(), "the double-matched finding must report as suppressed");
    assert!(
        result.findings.baselined_findings.is_empty(),
        "the double-matched finding must not also report as baselined: {:?}",
        result.findings.baselined_findings
    );
}

/// A [`RuleCalibration`] matching the one blocking finding this fixture produces: the run
/// still judges the source and `check_outcome` still carries the finding in full, and it now
/// also appears in `calibrated_findings` rather than `blocking_findings` -- tolerated, not
/// silenced. Mirrors [`Test_A_Suppressed_Finding_Should_Not_Block`] and
/// [`Test_A_Baselined_Finding_Should_Not_Block`] for the third of `Run_Gate`'s three
/// policies.
#[test]
fn Test_A_Calibrated_Finding_Should_Not_Block()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Calibration(Repository_Root(), Calibration_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    Assert_Tolerated_Not_Blocking(&result, &result.findings.calibrated_findings);
}

/// A finding matched by a [`RuleCalibration`], a [`Suppression`] and a [`BaselineDebt`] all
/// at once reports as calibrated, not suppressed or baselined -- `Run_Gate` checks
/// calibration first, since it is a coarser, rule-wide override, so none of the three lists
/// double-count the same finding.
#[test]
fn Test_A_Calibrated_Suppressed_And_Baselined_Finding_Should_Report_As_Calibrated()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        adoption: AdoptionPolicy { calibrated: vec![Calibration_Of(&real_finding)] },
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert!(!result.findings.calibrated_findings.is_empty(), "the triple-matched finding must report as calibrated");
    assert!(
        result.findings.suppressed_findings.is_empty(),
        "the triple-matched finding must not also report as suppressed: {:?}",
        result.findings.suppressed_findings
    );
    assert!(
        result.findings.baselined_findings.is_empty(),
        "the triple-matched finding must not also report as baselined: {:?}",
        result.findings.baselined_findings
    );
}

/// A query naming a rule and location no finding carries is [`Explanation::NotFound`], not
/// a panic or a default -- the same "an absent answer is a typed state, not a shorter one"
/// discipline `CheckOutcome::NoSource` already keeps one layer down.
#[test]
fn Test_Explain_Should_Report_Not_Found_For_A_Query_Nothing_Answers()
{
    let sources = vec![Source("a.rs", "pub fn Ok()\n{\n}\n")];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "nowhere.rs".to_owned() };

    let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), &query);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.explanation, Explanation::NotFound);
}

/// A real query naming the one blocking finding this fixture produces answers `Found`, with
/// `would_block` true and no suppression -- the everyday case, checked against a real judged
/// finding rather than a fixture built to look like one.
#[test]
fn Test_Explain_Gate_Should_Find_A_Real_Blocking_Finding()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n",
    )];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };

    let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(Repository_Root()), &query);

    let Explanation::Found { finding, would_block, calibrated_by, suppressed_by, baselined_by, contract } = result.explanation
    else
    {
        panic!("this fixture must produce the finding the query names");
    };
    assert_eq!(finding.rule, RuleId::New(COMPLETENESS_MIRROR));
    assert!(would_block);
    assert_eq!(calibrated_by, None);
    assert_eq!(suppressed_by, None);
    assert_eq!(baselined_by, None);
    assert_eq!(contract, Some(("D-134".to_owned(), 2)), "COMPLETENESS_MIRROR's own real contract citation");
}

/// A [`Suppression`] matching the queried finding flips `would_block` to `false` and names
/// itself in `suppressed_by` -- `explain` consults `command.suppressions` even though it
/// ignores `scope` and `rules`, because whether a suppression applies is part of this
/// finding's own explanation.
#[test]
fn Test_Explain_Should_Report_A_Suppression_That_Applies()
{
    let (source, query, real_finding) = Explain_Applies_Fixture();
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&real_finding));

    let result = Explain_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, &query);

    let (would_block, _, suppressed_by, _) = Explained_Found(result.explanation);
    assert!(!would_block);
    assert!(suppressed_by.is_some());
}

/// A [`BaselineDebt`] matching the queried finding flips `would_block` to `false` and names
/// itself in `baselined_by` -- mirrors [`Test_Explain_Should_Report_A_Suppression_That_Applies`]
/// for the second of `explain`'s two consulted policies.
#[test]
fn Test_Explain_Should_Report_A_Baseline_That_Applies()
{
    let (source, query, real_finding) = Explain_Applies_Fixture();
    let command = Command_With_Baseline(Repository_Root(), Baseline_Of(&real_finding));

    let result = Explain_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, &query);

    let (would_block, _, _, baselined_by) = Explained_Found(result.explanation);
    assert!(!would_block);
    assert!(baselined_by.is_some());
}

/// A [`RuleCalibration`] matching the queried finding's rule flips `would_block` to `false`
/// and names itself in `calibrated_by` -- mirrors
/// [`Test_Explain_Should_Report_A_Suppression_That_Applies`] for the third of `explain`'s
/// three consulted policies.
#[test]
fn Test_Explain_Should_Report_A_Calibration_That_Applies()
{
    let (source, query, real_finding) = Explain_Applies_Fixture();
    let command = Command_With_Calibration(Repository_Root(), Calibration_Of(&real_finding));

    let result = Explain_Gate(Some(vec![source()]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, &query);

    let (would_block, calibrated_by, _, _) = Explained_Found(result.explanation);
    assert!(!would_block);
    assert!(calibrated_by.is_some());
}

/// `explain` is independent of `command.scope`: a scope that would exclude `a.rs` from a
/// real `run` must not stop `explain` from finding and reporting the same query.
#[test]
fn Test_Explain_Should_Ignore_Scope()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n",
    )];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Explain_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, &query);

    assert!(matches!(result.explanation, Explanation::Found { .. }));
}

/// A source no provider could materialize a fact for makes this run's own recomputed
/// `Claim` `Incomplete` -- the fixture every [`CoveragePolicy`] test below builds on.
/// Proven once here rather than assumed at each call site: `Claim::Incomplete` is the
/// premise, not the thing under test, for every fixture that reuses this text.
fn Coverage_Debt_Fixture() -> Vec<SourceFile>
{
    return vec![Source("a.rs", "pub fn Ok()\n{\n}\n"), Source("broken.rs", "pub const ??? = ;")];
}

/// [`CoveragePolicy::Unset`] -- `Default`, the state every existing caller is in -- leaves a
/// run's disposition exactly as it always was: `Passed`, even though the run could not
/// materialize a fact for `broken.rs` and its own recomputed `Claim` is `Incomplete`. `Claim`
/// still rides through `check_outcome` for information only, unchanged from every increment
/// before this one -- `OD-GATE-016`'s own "unset behavior is provably unchanged" clause.
#[test]
fn Test_An_Unset_Coverage_Policy_Should_Leave_A_Passed_Disposition_Alone()
{
    let command = Command_At(Repository_Root());

    let result = Run_Gate(Some(Coverage_Debt_Fixture()), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Passed);
    let CheckOutcome::Judged { claim, .. } = result.check_outcome
    else
    {
        panic!("a provider that read at least one file must still be judged");
    };
    assert_eq!(claim, Claim::Incomplete, "the fixture must actually be incomplete coverage, or this test proves nothing");
}

/// The same run under [`CoveragePolicy::RequireCompleteness`] reports [`GateRunOutcome::
/// Indeterminate`] instead of the `Passed` [`Test_An_Unset_Coverage_Policy_Should_Leave_A_
/// Passed_Disposition_Alone`] reports for the identical fixture -- `Run_Gate` recomputed
/// `Claim` over the rule-and-scope-selected findings, found it incomplete, and refused to
/// let that read as a clean run. `OD-GATE-016`'s own decision.
#[test]
fn Test_Required_Completeness_Should_Downgrade_An_Incomplete_Passed_Run()
{
    let command = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..Command_At(Repository_Root()) };

    let result = Run_Gate(Some(Coverage_Debt_Fixture()), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.findings.blocking_findings.is_empty(), "coverage must never manufacture a blocking finding: {:?}", result.findings.blocking_findings);
}

/// [`CoveragePolicy::RequireCompleteness`] does not touch a run that already reports
/// [`GateRunOutcome::Failed`]: a real blocking finding this run did reach a judgment about
/// is not made any less true by `broken.rs`, an unrelated subject the run could not judge --
/// [`CoveragePolicy::RequireCompleteness`]'s own doc says why this variant leaves `Failed`
/// alone rather than downgrading it the way it downgrades `Passed`.
#[test]
fn Test_Required_Completeness_Should_Not_Touch_A_Failed_Run()
{
    let mut sources = Coverage_Debt_Fixture();
    sources.push(Source("phantom.rs", "/// Mirrored by `Test_Nowhere`.\npub const TABLE: &[&str] = &[];\n"));
    let command = GateCommand { coverage: CoveragePolicy::RequireCompleteness, ..Command_At(Repository_Root()) };

    let result = Run_Gate(Some(sources), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &command, Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.findings.blocking_findings.is_empty());
}

/// A tree of this test's own, carrying `policy` as its `nomos-gate.json` -- the declared
/// source `Run_Gate` resolves from, rather than a policy handed to it in a `GateCommand`.
fn Root_Declaring(name: &str, policy: &str) -> PathBuf
{
    let root = std::env::temp_dir().join(format!("nomos-gate-orchestration-policy-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("writable");
    std::fs::write(root.join("nomos-gate.json"), policy).expect("writable");

    return root;
}

/// The end-to-end case this whole item exists for: a suppression and a baseline entry a
/// person wrote in a file, resolved by a real run, tolerating two real findings that would
/// otherwise block.
///
/// Both rules here are addressed by the file's own subject, which is what makes a
/// path-authored entry match them -- see `crate::policy::gate_policy_file`'s own doc for the
/// rules this does not yet reach and why.
#[test]
fn Test_A_Declared_Policy_File_Should_Tolerate_Findings_A_Command_Never_Mentioned()
{
    let sources = || return vec![Source("b.rs", "pub fn badName() {}\n"), Source("c.rs", "// TODO fix this\npub fn Ok()\n{\n}\n")];
    let root = Root_Declaring(
        "tolerates",
        r#"{
            "suppressions": [
                {
                    "rule": "no-single-line-function-bodies",
                    "path": "b.rs",
                    "disposition": "false-positive",
                    "rationale": "test fixture",
                    "owner": "test"
                }
            ],
            "baseline": [
                { "rule": "todo-format-is-todo-name-description-ticket", "path": "c.rs", "rationale": "test fixture" }
            ]
        }"#,
    );

    let result = Run_Gate(Some(sources()), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(root), Test_Run_Id());

    assert!(!result.findings.suppressed_findings.is_empty(), "the file's suppression must have matched a real finding");
    assert!(!result.findings.baselined_findings.is_empty(), "the file's baseline entry must have matched a real finding");
    assert!(
        !result.findings.blocking_findings.iter().any(|finding| return finding.rule == RuleId::New("no-single-line-function-bodies")),
        "the suppressed rule must not still block: {:?}",
        result.findings.blocking_findings
    );
}

/// The same tree with no policy file resolves to today's behavior exactly, which is what
/// makes the file safe to add: every existing caller, and CI's own `gate run --root .`, sits
/// in this case.
#[test]
fn Test_A_Root_With_No_Policy_File_Should_Judge_Exactly_As_Before()
{
    let root = std::env::temp_dir().join(format!("nomos-gate-orchestration-policy-absent-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("writable");
    let _ = std::fs::remove_file(root.join("nomos-gate.json"));

    let result = Run_Gate(Some(vec![Source("b.rs", "pub fn badName() {}\n")]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(root), Test_Run_Id());

    assert!(result.findings.suppressed_findings.is_empty());
    assert!(result.findings.baselined_findings.is_empty());
    assert_eq!(result.disposition, GateRunOutcome::Failed);
}

/// A coverage floor nobody could set before: declared in the file, it downgrades a run that
/// would otherwise report `Passed` rather than riding along for information only.
#[test]
fn Test_A_Declared_Coverage_Floor_Should_Reach_The_Disposition()
{
    let root = Root_Declaring("coverage", r#"{ "coverage": "require-completeness" }"#);

    let result = Run_Gate(Some(Coverage_Debt_Fixture()), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(root), Test_Run_Id());

    assert_ne!(result.disposition, GateRunOutcome::Failed, "this fixture must not block, so the floor is what is being observed");
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate, "a declared coverage floor must reach the disposition");
}

/// A policy file that exists and cannot be parsed refuses the run. The check still happened
/// and `check_outcome` still carries it, but no verdict is reported, because the rules for
/// reaching one were unreadable -- a build that passed here would be passing under a policy
/// nobody authored.
#[test]
fn Test_A_Malformed_Policy_File_Should_Refuse_Rather_Than_Report_A_Verdict()
{
    let root = Root_Declaring("malformed", "{ not json");

    let result = Run_Gate(Some(vec![Source("b.rs", "pub fn Named() {}\n")]), GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment }, &Command_At(root), Test_Run_Id());

    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }), "the check itself still ran: {:?}", result.check_outcome);
}

/// The property `OD-GATE-025` exists for, pinned in both directions: a narrowed run and a
/// whole-workspace run agree about the same file.
///
/// `src/thing.rs` is declared by `src/lib.rs` and is not an orphan. Before the scope moved
/// off the source set, `--include src/thing.rs` collected the file and dropped the `lib.rs`
/// that declares it, so `no-orphan-modules` answered a question about a world where nothing
/// declared it and reported a `Blocking` finding telling a reader to delete or re-declare
/// correct code. Both runs must now say the same thing about it, which is nothing.
#[test]
fn Test_A_Narrowed_Run_And_A_Whole_Run_Should_Agree_About_A_Declared_Module()
{
    let sources = || {
        return vec![
            Source("crates/example/src/lib.rs", "mod thing;\n"),
            Source("crates/example/src/thing.rs", "pub fn Thing() {}\n"),
        ];
    };
    let narrowed = GateCommand {
        scope: ScopeSelector { include: vec!["crates/example/src/thing.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let whole = Orphan_Findings_Of(sources(), &Command_At(Repository_Root()));
    let narrow = Orphan_Findings_Of(sources(), &narrowed);

    assert!(whole.is_empty(), "a declared module is not an orphan to a whole run: {whole:?}");
    assert!(narrow.is_empty(), "nor to a narrowed one -- the run that invented this is the defect: {narrow:?}");
}

/// The other half, so the fix is not bought by blinding the rule: a file nothing declares is
/// still reported, including when the run was narrowed to exactly that file.
///
/// Without this, every assertion above is satisfied by a rule that stopped answering.
#[test]
fn Test_A_Genuinely_Orphaned_Module_Should_Still_Be_Reported_By_A_Narrowed_Run()
{
    let sources = vec![
        Source("crates/example/src/lib.rs", "mod thing;\n"),
        Source("crates/example/src/thing.rs", "pub fn Thing() {}\n"),
        Source("crates/example/src/stray.rs", "pub fn Stray() {}\n"),
    ];
    let narrowed = GateCommand {
        scope: ScopeSelector { include: vec!["crates/example/src/stray.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let reported = Orphan_Findings_Of(sources, &narrowed);

    assert_eq!(reported.len(), 1, "the one file nothing declares must still be named: {reported:?}");
    let named = reported.first().expect("asserted len 1 above");
    assert!(named.contains("stray.rs"), "{reported:?}");
}

/// Every `no-orphan-modules` finding a run reported, as the text a reader would act on.
fn Orphan_Findings_Of(sources: Vec<SourceFile>, command: &GateCommand) -> Vec<String>
{
    let result = Run_Gate(
        Some(sources),
        GateEnvironment { variant: Test_Variant(), launcher: &StdProcessLauncher, filesystem: &StdFileSystem, environment: &StdEnvironment },
        command,
        Test_Run_Id(),
    );

    let CheckOutcome::Judged { findings, .. } = &result.check_outcome
    else
    {
        panic!("these fixtures are real source, so the run judges them: {:?}", result.check_outcome);
    };

    return findings
        .iter()
        .filter(|finding| return finding.rule == RuleId::New("no-orphan-modules"))
        .map(|finding| return finding.subject_name.clone())
        .collect();
}
