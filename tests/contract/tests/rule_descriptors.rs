//! The descriptor table and the run's composed rule table describe the same rules.
//!
//! `nomos_rules::DESCRIPTORS` says what every rule reads. `nomos_check_orchestration::
//! Composed_Rules` says what a real run actually judges. Those are two hand-maintained
//! lists of the same population, which is exactly the shape this repository has already
//! paid for elsewhere -- a declared list nothing compares against the reality it
//! enumerates passes by not looking.
//!
//! So this is the mirror for that pair, and it is a boundary assertion rather than a unit
//! test because neither crate can see the other: `nomos-rules` sits below the
//! orchestration that composes it and may not name it, and the composition root is not
//! where a claim about the rules crate belongs. This crate is above both and is the only
//! place the two sets can be put side by side.
//!
//! A rule added to one side and not the other fails here, in both directions.

use nomos_contracts::RuleId;
use std::collections::BTreeSet;

#[test]
fn Test_Every_Composed_Rule_Should_Have_A_Descriptor()
{
    let described = Described_Rules();
    let composed = Composed_Rules();

    let undescribed = composed.difference(&described).count();

    assert_eq!(
        undescribed,
        0,
        "{undescribed} rule(s) are composed into a real run with no descriptor saying what they read; \
         {} composed against {} described",
        composed.len(),
        described.len()
    );
}

#[test]
fn Test_Every_Descriptor_Should_Name_A_Composed_Rule()
{
    let described = Described_Rules();
    let composed = Composed_Rules();

    let uncomposed: Vec<&str> = nomos_rules::DESCRIPTORS
        .iter()
        .filter(|descriptor| return !composed.contains(&descriptor.Rule()))
        .map(|descriptor| return descriptor.id)
        .collect();

    assert!(
        uncomposed.is_empty(),
        "described but never composed, so nothing runs them: {uncomposed:?}; \
         {} composed against {} described",
        composed.len(),
        described.len()
    );
}

/// The descriptor table has no duplicate identifiers, which is what makes comparing the
/// two as sets an honest comparison of their sizes rather than only of their contents.
#[test]
fn Test_The_Descriptor_Table_Should_Not_Repeat_An_Identifier()
{
    assert_eq!(
        Described_Rules().len(),
        nomos_rules::DESCRIPTORS.len(),
        "a rule identifier appears twice in the descriptor table"
    );
}

fn Described_Rules() -> BTreeSet<RuleId>
{
    return nomos_rules::DESCRIPTORS.iter().map(|descriptor| return descriptor.Rule()).collect();
}

fn Composed_Rules() -> BTreeSet<RuleId>
{
    return nomos_check_orchestration::Composed_Rules().into_iter().collect();
}
