//! The declarations themselves: every domain in the tree, against the row of the contracts
//! table it occupies.

use nomos_contracts::{DeterminismStrength, Strategy};

/// Every domain this workspace has, with the row of the contracts table it occupies.
///
/// # Why this test was renamed
///
/// It was `Test_The_Declared_Domains_Should_Be_The_Ones_This_Item_Covered`, and
/// `docs/records/OD-DETERMINISM-001` cites it by that name as the place the two undeclared
/// rows were written down. Both are declared now, so the sentence that name asserts is
/// false. A citation that resolves to a test asserting the opposite of what the citing
/// record says is worse than one that resolves to nothing, so the name moved and
/// `docs/records/OD-DETERMINISM-002` records where it went.
///
/// # What is still not covered, and why that is not a gap
///
/// Two of the table's six rows have no domain in this tree. "Correction planning and
/// staging" describes work that applies fixes, and nothing here applies one. "Progress UI,
/// logs, telemetry, agent execution" is the `None` row — the CLI prints, and nothing about
/// what it prints is a fact. Neither is an omission that a declaration would repair;
/// `tests/contract/tests/determinism_declarations.rs` is where they are accounted for, so
/// that a crate arriving to occupy either row cannot do so silently.
#[test]
fn Test_Every_Domain_In_The_Tree_Should_Declare_And_Be_Registered()
{
    use crate::harness::Test_Name_For;
    use nomos_analysis::FactReuse;
    use nomos_lang_rust::SyntaxFactProduction;
    use nomos_lang_rust_scan::ScanFactProduction;
    use nomos_spec_bundle::BundleSerialization;
    use nomos_spec_project::ProjectionOutput;
    use nomos_workspace::SnapshotSerialization;

    let declared = [
        ("syntax-fact-production", SyntaxFactProduction::STRENGTH),
        // The same declaration, discharged over the other thing it covers. Two entries and
        // one strategy is the shape `P10-ROLLUP-DETERMINISM` settled on: `nomos-lang-rust`
        // has two fact producers occupying one row of the contracts table, so they are one
        // promise — and a promise covering two producers has to be run over both, or the
        // second is covered by a sentence and measured by nothing.
        ("module-index-rollup", SyntaxFactProduction::STRENGTH),
        ("scan-fact-production", ScanFactProduction::STRENGTH),
        ("fact-reuse", FactReuse::STRENGTH),
        ("snapshot-serialization", SnapshotSerialization::STRENGTH),
        ("bundle-serialization", BundleSerialization::STRENGTH),
        ("projection-output", ProjectionOutput::STRENGTH),
    ];
    assert_eq!(
        declared.len(),
        7,
        "seven productions are covered by six declarations; a new producer needs a row in \
         this table and a test of its own, whether or not it also needs a declaration of \
         its own"
    );

    for (domain, strength) in declared
    {
        assert!(
            !Test_Name_For(domain).is_empty(),
            "{domain} declares a strategy and has no test registered"
        );
        assert_ne!(
            strength,
            DeterminismStrength::None,
            "{domain} is measured here and promises nothing, so the measurement is \
             discharging no obligation"
        );
    }
}
