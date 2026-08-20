//! What this crate promises today: a real rule registry, reported back whole, and a real
//! disposition reduction over an already-judged list of findings.

use crate::{Disposition, GateCommand, GateOutcome, GateRunOutcome, Run};
use nomos_contracts::{
    Applicability, Digest128, EvidenceClass, Finding, GateCategory, RuleId, SubjectId,
};
use nomos_rules::{
    COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION, DEPENDENCY_CONTRACT_RECORD,
    DEPENDENCY_CONTRACT_RECORD_VERSION, DEPENDENCY_DIRECTION, NAMING_CONVENTION,
    UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
use std::path::PathBuf;

fn Command() -> GateCommand
{
    return GateCommand { root: PathBuf::from(".") };
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
    let GateOutcome::Planned(here) = Run(&GateCommand { root: PathBuf::from(".") })
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };
    let GateOutcome::Planned(elsewhere) = Run(&GateCommand { root: PathBuf::from("elsewhere") })
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
