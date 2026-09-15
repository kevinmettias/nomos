//! What `plan` reports: the rules a run would judge by, each offer's own contract citation,
//! and the one selection it still does not read.

use super::{Command, Command_At};
use crate::{GateOutcome, Run};
use nomos_contracts::RuleId;
use nomos_rules::{
    COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION, DEPENDENCY_CONTRACT_RECORD, DEPENDENCY_CONTRACT_RECORD_VERSION,
    DEPENDENCY_DIRECTION, NAMING_CONVENTION, UNREAD_REACHES_FINDING, UNREAD_REACHES_FINDING_CONTRACT_RECORD,
    UNREAD_REACHES_FINDING_CONTRACT_RECORD_VERSION,
};
use std::path::PathBuf;

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
