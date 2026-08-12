//! Controls. Each runs the real predicate over a constructed input.
//!
//! Without these, a predicate that reported nothing would satisfy every assertion in
//! [`crate::committed`] and catch nothing at all. Each control also asserts the other
//! direction — that the predicate stays quiet over a sound entry — so it cannot be
//! satisfied by one that reports everything.

use crate::assessment::{Assessment, Site, Verdict};
use crate::predicates::{Divergences_With_No_Record, Unresolved_Records, Unresolved_Sites};
use nomos_contract_tests::Workspace;

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
    };
    assert!(
        Unresolved_Records(&root, &[real]).is_empty(),
        "a registered record must resolve, or the control above proves only that the \
         function always reports something"
    );
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
