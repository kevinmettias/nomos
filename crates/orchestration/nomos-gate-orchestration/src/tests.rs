//! What this crate promises today: a real rule registry, reported back whole, and a real
//! disposition reduction over an already-judged list of findings.

use crate::{
    AdoptionPolicy, BaselineDebt, BaselinePolicy, Disposition, Explain_Gate, Explanation, FindingQuery, GateCommand,
    GateOutcome, GateRunOutcome, GateRunResult, RuleCalibration, RuleSelector, Run, Run_Gate, ScopeSelector, Suppression,
    SuppressionDisposition, SuppressionPolicy,
};
use nomos_check_orchestration::CheckOutcome;
use nomos_contracts::{
    Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId,
};
use nomos_model::Subject_Of_Path;
use nomos_platform_std::StdProcessLauncher;
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
    let unmatched = Run_Gate(Some(vec![source()]), Test_Variant(), &Command_At(Repository_Root()), &StdProcessLauncher);

    return unmatched
        .blocking_findings
        .into_iter()
        .next()
        .expect("this fixture must produce one real blocking finding");
}

/// Asserts `result` reports no finding able to fail the build, and that `tolerated` --
/// whichever of `result.suppressed_findings`/`result.baselined_findings` the caller's own
/// policy populated -- is not empty, so the one real finding a suppression or a baseline
/// fixture produces is visible somewhere rather than silently disappearing.
fn Assert_Tolerated_Not_Blocking(result: &GateRunResult, tolerated: &[Finding])
{
    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.blocking_findings.is_empty(), "{:?}", result.blocking_findings);
    assert!(!tolerated.is_empty(), "the tolerated finding must still be visible");
}

/// Answers `query` against one fresh call of `source` with every policy empty, and returns
/// the finding it found -- the shared setup a suppression and a baseline explain fixture
/// both need before either can address that finding with its own policy.
fn Real_Finding_For(query: &FindingQuery, source: impl Fn() -> SourceFile) -> Finding
{
    let unmatched = Explain_Gate(Some(vec![source()]), Test_Variant(), &Command_At(Repository_Root()), query, &StdProcessLauncher);
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
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
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

/// The whole plan, by identity and in `RuleId` order, rather than by length. A count agrees
/// with itself: this registry composed two of the three shipped rules until
/// `P13-GATE-REGISTRY-THIRD-RULE`, then three of four until
/// `P13-CONTROLFLOW-REACHABILITY-WIRE`, and an assertion on `plan.rules.len()` would have
/// been green throughout.
#[test]
fn Test_A_Plan_Should_Hold_All_Four_Shipped_Rules()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let ids: Vec<RuleId> = plan.rules.iter().map(|offer| return offer.rule.clone()).collect();
    assert_eq!(
        ids,
        vec![
            RuleId::New(COMPLETENESS_MIRROR),
            RuleId::New(DEPENDENCY_DIRECTION),
            RuleId::New(NAMING_CONVENTION),
            RuleId::New(UNREAD_REACHES_FINDING)
        ],
        "in RuleId order: {ids:?}"
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
fn Test_The_Plan_Should_Not_Vary_By_Root()
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
/// [`Disposition`] cares about -- everything else here is filler a reader can ignore.
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
    assert_eq!(Disposition(&[]), GateRunOutcome::Passed);
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

    assert_eq!(Disposition(&findings), GateRunOutcome::Passed);
}

/// The one condition that must flip the disposition: a rule whose enforcer is honored,
/// judging a subject it actually reached.
#[test]
fn Test_A_Blocking_Finding_Should_Fail()
{
    let findings = vec![Finding_With(GateCategory::Blocking, Applicability::Supported)];

    assert_eq!(Disposition(&findings), GateRunOutcome::Failed);
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

    assert_eq!(Disposition(&findings), GateRunOutcome::Failed);
}

/// A root that was never walked -- the composition root's own `None`, the same case
/// `crates/host/nomos-cli/src/gate/run.rs` currently assigns `CheckOutcome::Unreadable` for
/// by hand. [`Run_Gate`] must make the identical assignment, since this crate now performs
/// that composition too.
#[test]
fn Test_An_Unwalked_Root_Should_Be_Indeterminate()
{
    let result = Run_Gate(None, Test_Variant(), &Command_At(Repository_Root()), &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::Unreadable));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.blocking_findings.is_empty());
}

/// A directory that was walked and held nothing -- `Some(Vec::new())` -- is a different
/// claim than a root nobody could walk at all, and must not collapse into the same
/// variant.
#[test]
fn Test_A_Walk_That_Found_No_Source_Should_Be_Indeterminate()
{
    let result = Run_Gate(Some(Vec::new()), Test_Variant(), &Command_At(Repository_Root()), &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.blocking_findings.is_empty());
}

/// A clean source, judged through the real `nomos_check_orchestration::Run` this crate now
/// calls directly, must pass -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Clean_Tree_Should_Be_Judged_Complete_With_No_Findings` already proves clean
/// under all four shipped rules.
#[test]
fn Test_A_Clean_Source_Should_Pass()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];

    let result = Run_Gate(Some(sources), Test_Variant(), &Command_At(Repository_Root()), &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.blocking_findings.is_empty(), "{:?}", result.blocking_findings);
}

/// A phantom mirror -- the same fixture `nomos-check-orchestration`'s own
/// `Test_A_Blocking_Finding_Should_Still_Be_Judged_Complete` uses -- must reach [`Run_Gate`]
/// as a real blocking finding and flip the disposition, proving the reduction this crate now
/// owns runs over `nomos_check_orchestration::Run`'s real output rather than a fixture typed
/// to look like it.
#[test]
fn Test_A_Blocking_Finding_Should_Fail_The_Run()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];

    let result = Run_Gate(Some(sources), Test_Variant(), &Command_At(Repository_Root()), &StdProcessLauncher);

    assert_eq!(result.disposition, GateRunOutcome::Failed);
    assert!(!result.blocking_findings.is_empty());
    assert!(result
        .blocking_findings
        .iter()
        .all(|finding| return finding.gate == GateCategory::Blocking));
}

/// [`ScopeSelector`] excludes the one source a walk found, so `Run_Gate` never calls
/// `nomos_check_orchestration::Run` at all -- the same `CheckOutcome::NoSource` an empty
/// walk already produces, because "scoped to nothing" and "found nothing" mean the same
/// thing to a caller.
#[test]
fn Test_A_Scoped_Out_Source_Should_Not_Be_Judged()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(sources), Test_Variant(), &command, &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::NoSource));
    assert_eq!(result.disposition, GateRunOutcome::Indeterminate);
    assert!(result.blocking_findings.is_empty());
}

/// [`RuleSelector`] excludes the rule behind the one blocking finding this fixture produces:
/// the run still judges the source, `check_outcome` still carries the finding in full, but
/// it can no longer fail the build -- selection of what blocks, honestly short of
/// selection of what runs.
#[test]
fn Test_A_Deselected_Rules_Finding_Should_Not_Block()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let command = GateCommand {
        rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(sources), Test_Variant(), &command, &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.blocking_findings.is_empty());
    assert!(
        Judged_Findings(&result.check_outcome)
            .iter()
            .any(|finding| return finding.rule == RuleId::New(COMPLETENESS_MIRROR)),
        "the deselected rule's finding must still be judged and carried, just not blocking"
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
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Suppression(Repository_Root(), Suppression_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), Test_Variant(), &command, &StdProcessLauncher);

    Assert_Tolerated_Not_Blocking(&result, &result.suppressed_findings);
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
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Baseline(Repository_Root(), Baseline_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), Test_Variant(), &command, &StdProcessLauncher);

    Assert_Tolerated_Not_Blocking(&result, &result.baselined_findings);
}

/// A finding matched by both a [`Suppression`] and a [`BaselineDebt`] reports as suppressed,
/// not baselined -- `Run_Gate` checks suppression first, so the two lists never double-count
/// the same finding, and this is the one case a passing "not blocking" assertion alone would
/// not catch: `blocking_findings` empty is also true if the ordering were reversed.
#[test]
fn Test_A_Suppressed_And_Baselined_Finding_Should_Report_As_Suppressed()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(vec![source()]), Test_Variant(), &command, &StdProcessLauncher);

    assert!(!result.suppressed_findings.is_empty(), "the double-matched finding must report as suppressed");
    assert!(
        result.baselined_findings.is_empty(),
        "the double-matched finding must not also report as baselined: {:?}",
        result.baselined_findings
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
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = Command_With_Calibration(Repository_Root(), Calibration_Of(&real_finding));

    let result = Run_Gate(Some(vec![source()]), Test_Variant(), &command, &StdProcessLauncher);

    Assert_Tolerated_Not_Blocking(&result, &result.calibrated_findings);
}

/// A finding matched by a [`RuleCalibration`], a [`Suppression`] and a [`BaselineDebt`] all
/// at once reports as calibrated, not suppressed or baselined -- `Run_Gate` checks
/// calibration first, since it is a coarser, rule-wide override, so none of the three lists
/// double-count the same finding.
#[test]
fn Test_A_Calibrated_Suppressed_And_Baselined_Finding_Should_Report_As_Calibrated()
{
    let source = || return Source("a.rs", "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n");
    let real_finding = One_Real_Blocking_Finding(source);
    let command = GateCommand {
        adoption: AdoptionPolicy { calibrated: vec![Calibration_Of(&real_finding)] },
        suppressions: SuppressionPolicy { suppressions: vec![Suppression_Of(&real_finding)] },
        baseline: BaselinePolicy { debt: vec![Baseline_Of(&real_finding)] },
        ..Command_At(Repository_Root())
    };

    let result = Run_Gate(Some(vec![source()]), Test_Variant(), &command, &StdProcessLauncher);

    assert!(!result.calibrated_findings.is_empty(), "the triple-matched finding must report as calibrated");
    assert!(
        result.suppressed_findings.is_empty(),
        "the triple-matched finding must not also report as suppressed: {:?}",
        result.suppressed_findings
    );
    assert!(
        result.baselined_findings.is_empty(),
        "the triple-matched finding must not also report as baselined: {:?}",
        result.baselined_findings
    );
}

/// A query naming a rule and location no finding carries is [`Explanation::NotFound`], not
/// a panic or a default -- the same "an absent answer is a typed state, not a shorter one"
/// discipline `CheckOutcome::NoSource` already keeps one layer down.
#[test]
fn Test_Explain_Should_Report_Not_Found_For_A_Query_Nothing_Answers()
{
    let sources = vec![Source("a.rs", "pub fn Ok() {}\n")];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "nowhere.rs".to_owned() };

    let result = Explain_Gate(Some(sources), Test_Variant(), &Command_At(Repository_Root()), &query, &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.explanation, Explanation::NotFound);
}

/// A real query naming the one blocking finding this fixture produces answers `Found`, with
/// `would_block` true and no suppression -- the everyday case, checked against a real judged
/// finding rather than a fixture built to look like one.
#[test]
fn Test_Explain_Should_Find_A_Real_Blocking_Finding()
{
    let sources = vec![Source(
        "a.rs",
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };

    let result = Explain_Gate(Some(sources), Test_Variant(), &Command_At(Repository_Root()), &query, &StdProcessLauncher);

    let Explanation::Found { finding, would_block, calibrated_by, suppressed_by, baselined_by } = result.explanation
    else
    {
        panic!("this fixture must produce the finding the query names");
    };
    assert_eq!(finding.rule, RuleId::New(COMPLETENESS_MIRROR));
    assert!(would_block);
    assert_eq!(calibrated_by, None);
    assert_eq!(suppressed_by, None);
    assert_eq!(baselined_by, None);
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

    let result = Explain_Gate(Some(vec![source()]), Test_Variant(), &command, &query, &StdProcessLauncher);

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

    let result = Explain_Gate(Some(vec![source()]), Test_Variant(), &command, &query, &StdProcessLauncher);

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

    let result = Explain_Gate(Some(vec![source()]), Test_Variant(), &command, &query, &StdProcessLauncher);

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
        "/// Mirrored by `Test_Nowhere`.\npub const T: &[&str] = &[];\n",
    )];
    let query = FindingQuery { rule: RuleId::New(COMPLETENESS_MIRROR), location: "a.rs".to_owned() };
    let command = GateCommand {
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        ..Command_At(Repository_Root())
    };

    let result = Explain_Gate(Some(sources), Test_Variant(), &command, &query, &StdProcessLauncher);

    assert!(matches!(result.explanation, Explanation::Found { .. }));
}
