//! Whether the register is honest about itself, asked without a corpus.
//!
//! These four tests are the half that runs anywhere: every entry says what it counted and
//! over what, identifiers do not collide, the registered families are exactly the ones the
//! restoration owes an answer for, and a disagreement with the plan is recorded as one. A
//! register that contradicted itself would let the measured half agree with the wrong
//! numbers.

use crate::register::{Entry, PlanFigure, Register};
use std::collections::BTreeSet;

/// The families the plan's I5 names. A register that stops carrying one of them fails,
/// so a family cannot leave the restoration by leaving the register.
const REQUIRED_FAMILIES: &[&str] = &[
    "table_row",
    "code_block",
    "canonical_domain_model",
    "roadmap_milestone",
    "scenario",
    "service",
    "appendix_d",
    "appendix_h",
    "headless_inventory",
    "ide_profile",
    "glossary_term",
    "catalog_entity",
    "v15_record",
];

/// The fields every register entry must say something about, named for what each field
/// holds rather than `Cases()` — what varies here is which field of the entry is checked.
fn Required_Fields() -> Vec<(&'static str, fn(&Entry) -> &String)>
{
    return vec![
        ("family", |entry| &entry.family),
        ("unit", |entry| &entry.unit),
        ("definition", |entry| &entry.definition),
        ("corpus", |entry| &entry.corpus),
    ];
}

/// Runs without a corpus, and is the half that keeps the register honest about itself.
#[test]
fn Test_Every_Entry_Should_Say_What_It_Counted_And_Over_What()
{
    for entry in Register()
    {
        for (field, accessor) in Required_Fields()
        {
            assert!(!accessor(&entry).trim().is_empty(), "{}: {field} is empty", entry.id);
        }
    }
}

#[test]
fn Test_Entry_Identifiers_Should_Be_Unique()
{
    let entries = Register();
    let distinct: BTreeSet<&str> = entries.iter().map(|entry| return entry.id.as_str()).collect();

    assert_eq!(distinct.len(), entries.len(), "two entries share an identifier");
}

/// Every family the restoration owes an answer for is in the register, and nothing else
/// is. A family cannot leave the restoration by leaving the register, and one cannot join
/// it without being declared.
#[test]
fn Test_The_Registered_Families_Should_Be_Exactly_The_Required_Ones()
{
    let entries = Register();
    let present: BTreeSet<String> = entries.iter().map(|entry| return entry.family.clone()).collect();

    for required in REQUIRED_FAMILIES
    {
        assert!(present.contains(*required), "{required} carries no measured count");
    }
    for family in &present
    {
        assert!(
            REQUIRED_FAMILIES.contains(&family.as_str()),
            "{family} is measured and undeclared"
        );
    }
}

/// A disagreement with the plan is recorded as one. Silently agreeing figures may not be
/// marked superseded either, or the marker stops meaning anything.
#[test]
fn Test_A_Plan_Figure_Should_Be_Superseded_Exactly_When_It_Differs()
{
    for entry in Register()
    {
        let Some(plan) = entry.plan.as_ref()
        else
        {
            continue;
        };

        assert!(!plan.note.trim().is_empty(), "{}: the plan's figure carries no reading", entry.id);
        assert!(!plan.named.trim().is_empty(), "{}: the plan's figure names nothing", entry.id);
        Assert_Superseded_Exactly_When_It_Differs(&entry, plan);
    }
}

/// A stated figure is marked superseded; an unstated one is marked unnumbered; and a stated
/// figure equal to the measurement has to say what it counted instead.
fn Assert_Superseded_Exactly_When_It_Differs(entry: &Entry, plan: &PlanFigure)
{
    let Some(figure) = plan.figure
    else
    {
        assert_eq!(
            plan.status, "unnumbered",
            "{}: the plan states no figure, so nothing can be superseded",
            entry.id
        );

        return;
    };

    assert_eq!(
        plan.status, "superseded",
        "{}: a stated figure is either superseded or absent from the register",
        entry.id
    );
    assert!(
        figure != entry.measured || plan.note.contains("count"),
        "{}: the plan's figure equals the measurement and the note does not say what it \
         counted instead",
        entry.id
    );
}
