//! Controls. Each runs the real predicate over a constructed input.
//!
//! Without these, a predicate that reported nothing would satisfy every assertion in
//! [`crate::committed`] and catch nothing at all. Each control also asserts the other
//! direction — that the predicate stays quiet over a sound entry — so it cannot be
//! satisfied by one that reports everything.

use crate::assessment::{Assessment, Site, Verdict};
use crate::predicates::{
    Divergences_With_No_Record, Partials_With_No_Gap, Unresolved_Gaps, Unresolved_Records,
    Unresolved_Rules, Unresolved_Sites,
};
use nomos_contract_tests::Workspace;
use nomos_contracts::RuleId;

/// The negative control for
/// [`crate::committed::Test_Every_Assessment_Should_Name_A_Site_That_Exists`].
///
/// A guard that reported every site as present would pass that test and catch nothing.
#[test]
fn Test_An_Entry_Naming_A_Vanished_Site_Should_Be_Reported()
{
    let root = Workspace::Workspace_Root();
    let gone_file = Naming("CHK-003", "crates/contracts/nomos-contracts/src/no_such_file.rs", "Applicability");
    let gone_symbol = Naming(
        "EVID-001",
        "crates/contracts/nomos-contracts/src/evidence.rs",
        "A_Name_This_Workspace_Does_Not_Use",
    );

    assert_eq!(Unresolved_Sites(&root, &[gone_file]).len(), 1);
    assert_eq!(
        Unresolved_Sites(&root, &[gone_symbol]).len(),
        1,
        "a renamed symbol in a file that still exists is the case a path-only check misses, \
         and it is the common one"
    );
}

/// A `Met` assessment naming exactly one site and no record.
fn Naming(requirement: &str, path: &str, symbol: &str) -> Assessment
{
    return Assessment {
        requirement: requirement.to_owned(),
        verdict: Verdict::Met,
        record: None,
        sites: vec![Site {
            path: path.to_owned(),
            symbol: symbol.to_owned(),
        }],
        gaps: Vec::new(),
        rules: Vec::new(),
    };
}

/// A `Partial` assessment naming one site, one gap, and no record.
fn Partial_Naming(requirement: &str, gap_path: &str, gap_symbol: &str) -> Assessment
{
    return Assessment {
        verdict: Verdict::Partial,
        gaps: vec![Site {
            path: gap_path.to_owned(),
            symbol: gap_symbol.to_owned(),
        }],
        ..Naming(requirement, "README.md", "Nomos")
    };
}

/// The negative control for
/// [`crate::committed::Test_Every_Named_Record_Should_Exist_And_Be_Registered`], in both
/// halves.
///
/// A document with no registration is `OD-SPEC-005`'s defect — six governing records were
/// written as files and never reached the store — so citing one has to be caught
/// separately from citing nothing at all.
#[test]
fn Test_A_Record_That_Does_Not_Resolve_Should_Be_Reported()
{
    let root = Workspace::Workspace_Root();
    let invented = Assessment {
        record: Some("OD-NOTHING-999".to_owned()),
        ..Naming(
            "CAP-002",
            "crates/contracts/nomos-contracts/src/guarantee.rs",
            "FactVariant",
        )
    };
    assert_eq!(Unresolved_Records(&root, &[invented]).len(), 1);

    let real = Assessment {
        requirement: "CAP-002".to_owned(),
        verdict: Verdict::Met,
        record: Some("OD-TRACE-001".to_owned()),
        sites: Vec::new(),
        gaps: Vec::new(),
        rules: Vec::new(),
    };
    assert!(
        Unresolved_Records(&root, &[real]).is_empty(),
        "a registered record must resolve, or the control above proves only that the \
         function always reports something"
    );
}

/// The negative control for
/// [`crate::committed::Test_Every_Declared_Rule_Should_Be_A_Rule_This_Build_Composes`], in
/// all three halves.
///
/// A guard that resolved every identifier would pass that test and catch nothing; one that
/// reported every identifier would pass it too, over a committed set whose one declared line
/// it would then be wrong about. The third assertion is the one this record insisted on: an
/// entry that declares nothing is not reported, because absence means nobody declared a rule
/// and never that no rule bears.
#[test]
fn Test_A_Rule_This_Build_Does_Not_Compose_Should_Be_Reported()
{
    let invented = Declaring("AGT-003", "a-rule-this-build-does-not-have");
    assert_eq!(Unresolved_Rules(&[invented]).len(), 1);

    let composed = Declaring("AGT-003", nomos_rules::DEPENDENCY_DIRECTION);
    assert!(
        Unresolved_Rules(&[composed]).is_empty(),
        "a rule this build composes must resolve, or the control above proves only that \
         the function always reports something"
    );

    let silent = Naming("AGT-003", "README.md", "Nomos");
    assert!(
        Unresolved_Rules(&[silent]).is_empty(),
        "an entry declaring no rule owes nothing here. Absence of a line means nobody \
         declared one, never that no rule bears on the requirement — OD-TRACE-001's \
         reading for this registry, which OD-HOST-015 kept for the line"
    );
}

/// A `Met` assessment naming one site and declaring one rule.
fn Declaring(requirement: &str, rule: &str) -> Assessment
{
    return Assessment {
        rules: vec![RuleId::New(rule)],
        ..Naming(requirement, "README.md", "Nomos")
    };
}

/// The negative control for
/// [`crate::committed::Test_Every_Divergence_Should_Name_A_Governing_Record`], which is
/// vacuous over the committed set.
#[test]
fn Test_A_Divergence_With_No_Record_Should_Be_Refused()
{
    let unreasoned = Assessment {
        verdict: Verdict::Diverges,
        ..Naming(
            "WORK-LEDGER-005",
            "crates/substrate/nomos-ledger/src/item.rs",
            "Blocker",
        )
    };
    assert_eq!(
        Divergences_With_No_Record(std::slice::from_ref(&unreasoned)).len(),
        1
    );

    let reasoned = Assessment {
        record: Some("OD-TRACE-001".to_owned()),
        ..unreasoned.clone()
    };
    assert!(Divergences_With_No_Record(&[reasoned]).is_empty());
    let met = Assessment {
        verdict: Verdict::Met,
        ..unreasoned
    };
    assert!(
        Divergences_With_No_Record(&[met]).is_empty(),
        "a Met entry owes no record, and reporting one would make the obligation \
         unstatable rather than merely strict"
    );
}

/// The negative control for
/// [`crate::committed::Test_Every_Gap_Should_Resolve`].
///
/// A guard that reported every gap as present would pass that test and catch nothing —
/// the same defect [`Test_An_Entry_Naming_A_Vanished_Site_Should_Be_Reported`] exists to
/// catch for `sites`, one field over.
#[test]
fn Test_An_Entry_Naming_A_Vanished_Gap_Should_Be_Reported()
{
    let root = Workspace::Workspace_Root();
    let gone = Partial_Naming(
        "CHK-003",
        "crates/contracts/nomos-contracts/src/no_such_file.rs",
        "Applicability",
    );

    assert_eq!(Unresolved_Gaps(&root, &[gone]).len(), 1);
}

/// The negative control for
/// [`crate::committed::Test_Every_Partial_Should_Name_A_Gap`], which is vacuous over the
/// committed set.
#[test]
fn Test_A_Partial_With_No_Gap_Should_Be_Refused()
{
    let unnamed = Assert_Partial_With_No_Gap_Is_Reported();

    Assert_Partial_Naming_A_Gap_Is_Not_Reported();
    Assert_Met_Owing_No_Gap_Is_Not_Reported(unnamed);
}

/// A `Partial` entry naming no gap must be reported, or the negative controls below prove
/// nothing.
fn Assert_Partial_With_No_Gap_Is_Reported() -> Assessment
{
    let unnamed = Assessment {
        verdict: Verdict::Partial,
        ..Naming(
            "CHK-003",
            "crates/contracts/nomos-contracts/src/finding/applicability.rs",
            "Applicability",
        )
    };
    assert_eq!(Partials_With_No_Gap(std::slice::from_ref(&unnamed)).len(), 1);

    return unnamed;
}

/// A `Partial` entry naming a gap must not be reported, or the control above proves only
/// that the function always reports something.
fn Assert_Partial_Naming_A_Gap_Is_Not_Reported()
{
    let named = Partial_Naming("CHK-003", "README.md", "Nomos");
    assert!(
        Partials_With_No_Gap(&[named]).is_empty(),
        "a Partial entry naming a gap must not be reported, or the control above proves \
         only that the function always reports something"
    );
}

/// A `Met` entry owes no gap, and reporting one would make the obligation unstatable rather
/// than merely strict.
fn Assert_Met_Owing_No_Gap_Is_Not_Reported(unnamed: Assessment)
{
    let met = Assessment {
        verdict: Verdict::Met,
        ..unnamed
    };
    assert!(
        Partials_With_No_Gap(&[met]).is_empty(),
        "a Met entry owes no gap, and reporting one would make the obligation unstatable \
         rather than merely strict"
    );
}
