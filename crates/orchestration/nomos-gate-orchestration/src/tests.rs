//! What this crate promises today: a real rule registry, reported back whole.

use crate::{GateCommand, GateOutcome, Run};
use nomos_contracts::RuleId;
use nomos_rules::{COMPLETENESS_MIRROR, CONTRACT_RECORD, CONTRACT_RECORD_VERSION, NAMING_CONVENTION};
use std::path::PathBuf;

fn Command() -> GateCommand
{
    return GateCommand { root: PathBuf::from(".") };
}

#[test]
fn Test_A_Plan_Should_Hold_Both_Shipped_Rules()
{
    let GateOutcome::Planned(plan) = Run(&Command())
    else
    {
        panic!("this crate's own registration must not be contradictory");
    };

    let ids: Vec<RuleId> = plan.rules.iter().map(|offer| return offer.rule.clone()).collect();
    assert_eq!(
        ids,
        vec![RuleId::New(COMPLETENESS_MIRROR), RuleId::New(NAMING_CONVENTION)],
        "in RuleId order: {ids:?}"
    );
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
