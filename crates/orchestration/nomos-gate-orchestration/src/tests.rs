//! What this crate promises today: a real rule registry, reported back whole, and a real
//! disposition reduction over an already-judged list of findings.

use crate::{Disposition, GateCommand, GateOutcome, GateRunOutcome, RuleSelector, Run, Run_Gate, ScopeSelector};
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
        root: Repository_Root(),
        scope: ScopeSelector { include: vec!["b.rs".to_owned()], exclude: Vec::new() },
        rules: RuleSelector::default(),
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
        root: Repository_Root(),
        scope: ScopeSelector::default(),
        rules: RuleSelector { include: vec![RuleId::New(NAMING_CONVENTION)] },
    };

    let result = Run_Gate(Some(sources), Test_Variant(), &command, &StdProcessLauncher);

    assert!(matches!(result.check_outcome, CheckOutcome::Judged { .. }));
    assert_eq!(result.disposition, GateRunOutcome::Passed);
    assert!(result.blocking_findings.is_empty());
    let CheckOutcome::Judged { findings, .. } = &result.check_outcome
    else
    {
        panic!("just matched Judged above");
    };
    assert!(
        findings.iter().any(|finding| return finding.rule == RuleId::New(COMPLETENESS_MIRROR)),
        "the deselected rule's finding must still be judged and carried, just not blocking"
    );
}
